use crate::encoder::EncoderOpts;
use crate::images::CompressedImage;
use crate::stages::wavelet_transform::WaveletImage;
use crate::utils;

use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use rans::b64_decoder::{B64RansDecSymbol, B64RansDecoderMulti};
use rans::b64_encoder::{B64RansEncSymbol, B64RansEncoderMulti};
use rans::RansDecoderMulti;
use rans::RansEncoderMulti;
use rans::{RansDecSymbol, RansEncSymbol};

fn emit_coefficients(layer: &Vec<i32>, layer_id: usize, layer_channel: usize) {
    std::fs::create_dir_all("./coefficients").unwrap();
    let mut f = File::create(format!(
        "coefficients/{}_layer_{}.coef",
        layer_channel, layer_id
    ))
    .expect("Unable to create coef file");
    for i in layer {
        write!(f, "{}\n", i).unwrap();
    }
}

fn pack_signed(k: i32) -> u32 {
    if k >= 0 {
        2 * k as u32
    } else {
        (-2 * k - 1) as u32
    }
}

fn unpack_signed(k: u32) -> i32 {
    if k % 2 == 0 {
        (k / 2) as i32
    } else {
        (k + 1) as i32 / -2
    }
}

pub fn encode(image: WaveletImage, encoder_opts: &EncoderOpts) -> Result<CompressedImage, String> {
    let mut channel_data: [Option<(Vec<AnsContext>, Vec<u8>)>; 3] = [None, None, None];

    for channel in 0..image.metadata.colorspace.num_channels() {
        let depth = image.depth;

        let mut layers: Vec<&[Option<i32>]> = vec![];
        let channel_coefficients = &image.coefficients[channel];

        layers.push(&channel_coefficients[1 << (depth - 1)..]);
        layers.push(&channel_coefficients[1 << (depth - 2)..1 << (depth - 1)]);
        layers.push(&channel_coefficients[..1 << (depth - 2)]);

        let mut encoder: B64RansEncoderMulti<3> =
            B64RansEncoderMulti::new(2*image.coefficients[channel].iter().flatten().count());

        let mut ans_contexts = vec![];
        for (i, sparse_layer) in layers.into_iter().enumerate() {
            let unpacked_layer = sparse_layer
                .into_iter()
                .flatten()
                .map(|c| *c)
                .collect::<Vec<i32>>();

            if encoder_opts.emit_coefficients {
                emit_coefficients(&unpacked_layer, i, channel)
            }

            let max_freq_bits =
                utils::get_prev_power_two(unpacked_layer.len())
                .trailing_zeros()
                + 1;
            
            let layer: Vec<u32> = unpacked_layer.into_iter().map(pack_signed).collect();

            let counter = layer
                .iter()
                .counts();

            let mut histogram : Vec<(&u32, usize)> = counter
                .into_iter()
                .collect();

            histogram.sort_by_key(|(a, _b)| *a);

            let mut freqs: Vec<u32> = histogram.clone().into_iter().map(|(_, b)| b as u32).collect();
            let symbols: Vec<u32> = histogram.clone().into_iter().map(|(a, _)| *a).collect();

            let cdf = normalize_freqs(&mut freqs, 1 << max_freq_bits);
            //let cdf = cum_sum(&freqs);

            let symbol_map = freqs_to_enc_symbols(&cdf, &freqs, max_freq_bits);

            let cdf_map = symbols
                .iter()
                .zip(cdf)
                .collect::<HashMap<&u32, u32>>();

            layer
                .iter()
                .rev()
                .for_each(|s| encoder.put_at(i, &symbol_map[&cdf_map[s]]));

            ans_contexts.push(AnsContext { symbols, freqs });
        }
        encoder.flush_all();
        let data = encoder.data().to_owned();
        dbg!(data.len());
        channel_data[channel] = Some((ans_contexts, data));
    }
    Ok(CompressedImage {
        metadata: image.metadata,
        channel_data,
    })
}

pub fn decode(mut compressed_image: CompressedImage) -> Result<WaveletImage, String> {
    let mut decoded = WaveletImage::from_metadata(compressed_image.metadata);

    let mut channel = 0;
    while let Some((ans_contexts, bytes)) = compressed_image.channel_data[channel].take() {
        let mut layers: Vec<usize> = vec![];

        let depth = decoded.depth;

        layers.push(
            decoded.coefficients[channel][..1 << (depth - 2)]
                .iter()
                .flatten()
                .count(),
        );
        layers.push(
            decoded.coefficients[channel][1 << (depth - 2)..1 << (depth - 1)]
                .iter()
                .flatten()
                .count(),
        );
        layers.push(
            decoded.coefficients[channel][1 << (depth - 1)..]
                .iter()
                .flatten()
                .count(),
        );

        let mut decoder: B64RansDecoderMulti<3> = B64RansDecoderMulti::new(bytes);
        let mut last = 0;
        for (i, (layer, ans_context)) in layers.into_iter().zip(ans_contexts.into_iter().rev()).enumerate() {
            let mut cum_freqs = cum_sum(&ans_context.freqs);
            let cum_freq_to_symbols = freqs_to_dec_symbols(&cum_freqs, &ans_context.freqs);
            let symbol_map = cum_freqs
                .clone()
                .into_iter()
                .zip(ans_context.symbols.clone())
                .collect::<HashMap<u32, u32>>();

            cum_freqs.sort_unstable();

            let max_freq_bits = utils::get_prev_power_two(layer)
                    .trailing_zeros()
                    + 1;

            for _l in 0..layer {
                let cum_freq_decoded =
                    find_nearest_or_equal(decoder.get_at(i, max_freq_bits), &cum_freqs);
                let symbol = symbol_map[&cum_freq_decoded];
                decoder.advance_step_at(i, &cum_freq_to_symbols[&cum_freq_decoded], max_freq_bits);
                decoder.renorm_at(i);
                last = insert_after_none_starting_from(
                    unpack_signed(symbol),
                    last,
                    &mut decoded.coefficients[channel],
                );
            }
        }
        channel += 1;
        if channel >= decoded.metadata.colorspace.num_channels() {
            break;
        }
    }
    return Ok(decoded);
}

fn insert_after_none_starting_from(element: i32, i: usize, vec: &mut Vec<Option<i32>>) -> usize {
    let ind = (i..vec.len()).find(|j| vec[*j].is_some()).unwrap();
    vec[ind] = Some(element);
    ind + 1
}

#[derive(Debug)]
pub struct AnsContext {
    pub symbols: Vec<u32>,
    pub freqs: Vec<u32>,
}

const ALPHABET_SIZE: usize = 512;

fn get_freqs(coefs: &Vec<u32>) -> Vec<u32> {
    let mut freqs = vec![0; ALPHABET_SIZE];
    for coef in coefs {
        freqs[*coef as usize] += 1;
    }
    return freqs;
}

#[must_use]
fn cum_sum(sum: &[u32]) -> Vec<u32> {
    sum.iter()
        .scan(0_u32, |acc, x| {
            let val = *acc;
            *acc += x;
            Some(val)
        })
        .collect::<Vec<u32>>()
}

fn normalize_freqs(freqs: &mut Vec<u32>, target_total: u32) -> Vec<u32> {
    let mut cum_freqs = cum_sum(freqs);
    let cur_total = *cum_freqs.last().unwrap() + freqs.last().unwrap();
    for i in 1..cum_freqs.len() {
        cum_freqs[i] = ((target_total as u64 * cum_freqs[i] as u64)/cur_total as u64) as u32; 
    }

    for i in 0..(cum_freqs.len() - 1) {
        freqs[i] = cum_freqs[i+1] - cum_freqs[i];
    }

    cum_freqs
}

// TODO: Implement Alias sampling method
#[must_use]
fn find_nearest_or_equal(cum_freq: u32, cum_freqs: &[u32]) -> u32 {
    match cum_freqs.binary_search(&cum_freq) {
        Ok(x) => cum_freqs[x],
        Err(x) => cum_freqs[x - 1],
    }
}

#[must_use]
fn freqs_to_enc_symbols(
    cum_freqs: &[u32],
    freqs: &[u32],
    depth: u32,
) -> HashMap<u32, B64RansEncSymbol> {
    cum_freqs
        .iter()
        .zip(freqs.iter())
        .map(|(&cum_freq, &freq)| (cum_freq, B64RansEncSymbol::new(cum_freq, freq, depth)))
        .collect()
}

#[must_use]
fn freqs_to_dec_symbols(cum_freqs: &[u32], freqs: &[u32]) -> HashMap<u32, B64RansDecSymbol> {
    cum_freqs
        .iter()
        .zip(freqs.iter())
        .map(|(&cum_freqs, &freqs)| (cum_freqs, B64RansDecSymbol::new(cum_freqs, freqs)))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // fn cum_sum_test() {
    //     let x = [1, 2, 3, 4, 5];
    //     let cum_x = cum_sum(&x);
    //     assert_eq!(cum_x, &[0, 1, 3, 6, 10])
    // }

    #[test]
    fn logic_test() {
        let layer = vec![1, 1, 1, 1, 1, 2, 3, 2, 3, 10, 11, 12];
        let freqs = vec![0; 13];
        let counter = layer.iter().counts();

        let freqs = counter
            .values()
            .map(|e| u32::try_from(*e).unwrap())
            .collect::<Vec<u32>>();

        println!("{:?}", &freqs);
        let symbols = counter.keys().map(|e| **e).collect::<Vec<i32>>();
        println!("{:?}", &symbols);

        let cum_freqs = cum_sum(&freqs);
        println!("{:?}", &cum_freqs);

        let symbol_map = freqs_to_enc_symbols(&cum_freqs, &freqs, 8);

        let cdf_map = counter
            .into_keys()
            .zip(cum_freqs)
            .collect::<HashMap<&i32, u32>>();

        println!("{:?}", &cdf_map);
    }
}
