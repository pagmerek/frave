use crate::images::CompressedImage;
use crate::stages::wavelet_transform::WaveletImage;
use crate::utils;

use itertools::Itertools;
use std::collections::HashMap;

use rans::b64_decoder::{B64RansDecSymbol, B64RansDecoderMulti};
use rans::b64_encoder::{B64RansEncSymbol, B64RansEncoderMulti};
use rans::RansDecoderMulti;
use rans::RansEncoderMulti;
use rans::{RansDecSymbol, RansEncSymbol};

pub fn encode(image: WaveletImage) -> Result<CompressedImage, String> {
    let mut channel_data: [Option<(Vec<AnsContext>, Vec<u8>)>; 3] = [None, None, None];

    for i in 0..image.metadata.colorspace.num_channels() {
        let depth = image.depth;

        let mut layers: Vec<&[Option<i32>]> = vec![];
        let channel_coefficients = &image.coefficients[i];

        layers.push(&channel_coefficients[1 << (depth - 1)..]);
        layers.push(&channel_coefficients[1 << (depth - 2)..1 << (depth - 1)]);
        layers.push(&channel_coefficients[..1 << (depth - 2)]);

        //let layer2 = &valid_coef[1 << (depth - 2)..1 << (depth - 1)];
        //let layer3 = &valid_coef[..1 << (depth - 2)];

        //let valid_coef: Vec<&i32> = image.coefficients[i].iter().flatten().collect();
        //let depth = utils::get_prev_power_two(valid_coef.len()).trailing_zeros() + 1;
        let valid_coef: Vec<&i32> = image.coefficients[i].iter().flatten().collect();
        let true_depth = utils::get_prev_power_two(valid_coef.len()).trailing_zeros() + 1;
        
        let mut encoder: B64RansEncoderMulti<3> = B64RansEncoderMulti::new(image.coefficients[i].iter().flatten().count());

        let mut ans_contexts = vec![];
        //let filtered_layers Vec<&[i32]> = layers.into_iter().map(|x| x.into_iter().flatten().collect()).collect();
        for (i, sparse_layer) in layers.into_iter().enumerate() {
            let layer: Vec<&i32> = sparse_layer.into_iter().flatten().collect();
            let counter = layer.iter().counts();
            let freq = counter
                .values()
                .map(|e| u32::try_from(*e).unwrap())
                .collect::<Vec<u32>>();
            let symbols = counter.keys().map(|e| ***e).collect::<Vec<i32>>();
            let cdf = cum_sum(&freq);

            let symbol_map = freqs_to_enc_symbols(&cdf, &freq, true_depth as usize);

            let cdf_map = counter
                .into_keys()
                .zip(cdf)
                .collect::<HashMap<&&i32, u32>>();

            layer
                .iter()
                .rev()
                .for_each(|s| encoder.put_at(i, &symbol_map[&cdf_map[s]]));

            ans_contexts.push(AnsContext { symbols, freq });
        }
        encoder.flush_all();
        channel_data[i] = Some((ans_contexts, encoder.data().to_owned()));
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
        let valid_coef: Vec<(u32, i32)> = decoded.coefficients[channel]
            .iter()
            .enumerate()
            .map(|(i, val)| (utils::get_prev_power_two(i+1).trailing_zeros(), val))
            .filter(|(_i, val)| val.is_some())
            .map(|(i, val)| (i, val.unwrap()))
            .collect();

        let depth = decoded.depth;
        let layer3: Vec<&i32> = valid_coef.iter().filter(|(i, _)| *i == (depth.wrapping_sub(1) as u32)).map(|(_, val)| val).collect(); 
        let layer2: Vec<&i32> = valid_coef.iter().filter(|(i, _)| *i == (depth.wrapping_sub(2) as u32)).map(|(_, val)| val).collect(); 
        let layer1: Vec<&i32> = valid_coef.iter().filter(|(i, _)| *i < (depth.wrapping_sub(2) as u32)).map(|(_, val)| val).collect(); 
        let true_depth = utils::get_prev_power_two(valid_coef.len()).trailing_zeros() + 1;
        let scale_bits = (true_depth - 1) as u32;
        let mut decoder: B64RansDecoderMulti<3> = B64RansDecoderMulti::new(bytes);

        let layers = vec![
            layer1.len(),
            layer2.len(),
            layer3.len(),
        ];
        let mut last = 0;
        for (i, layer) in layers.into_iter().enumerate() {
            let mut cum_freqs = cum_sum(&ans_contexts[2 - i].freq);
            let cum_freq_to_symbols = freqs_to_dec_symbols(&cum_freqs, &ans_contexts[2 - i].freq);
            let symbol_map = cum_freqs
                .clone()
                .into_iter()
                .zip(ans_contexts[2 - i].symbols.clone())
                .collect::<HashMap<u32, i32>>();

            cum_freqs.sort_unstable();

            for _l in 0..layer {
                let cum_freq_decoded =
                    find_nearest_or_equal(decoder.get_at(i, scale_bits), &cum_freqs);
                let symbol = symbol_map[&cum_freq_decoded];
                decoder.advance_step_at(i, &cum_freq_to_symbols[&cum_freq_decoded], scale_bits);
                decoder.renorm_at(i);
                last = insert_after_none_starting_from(symbol, last, &mut decoded.coefficients[channel]);
            }
        }
        channel+=1;
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
    pub symbols: Vec<i32>,
    pub freq: Vec<u32>,
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
    cum_freq: &[u32],
    freq: &[u32],
    depth: usize,
) -> HashMap<u32, B64RansEncSymbol> {
    cum_freq
        .iter()
        .zip(freq.iter())
        .map(|(&cum_freq, &freq)| {
            (
                cum_freq,
                B64RansEncSymbol::new(cum_freq, freq, u32::try_from(depth - 1).unwrap()),
            )
        })
        .collect::<HashMap<u32, B64RansEncSymbol>>()
}

#[must_use]
fn freqs_to_dec_symbols(cum_freq: &[u32], freq: &[u32]) -> HashMap<u32, B64RansDecSymbol> {
    cum_freq
        .iter()
        .zip(freq.iter())
        .map(|(&cum_freq, &freq)| (cum_freq, B64RansDecSymbol::new(cum_freq, freq)))
        .collect::<HashMap<u32, B64RansDecSymbol>>()
}
