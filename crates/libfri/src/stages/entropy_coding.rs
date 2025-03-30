use crate::encoder::EncoderOpts;
use crate::images::CompressedImage;
use crate::stages::wavelet_transform::WaveletImage;
use crate::utils;

use itertools::Itertools;
use num::Complex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use rans::b64_decoder::{B64RansDecSymbol, B64RansDecoderMulti};
use rans::b64_encoder::{B64RansEncSymbol, B64RansEncoderMulti};
use rans::RansDecoderMulti;
use rans::RansEncoderMulti;
use rans::{RansDecSymbol, RansEncSymbol};

fn emit_coefficients(layer: &Vec<i32>, center: Complex<i32>, layer_channel: usize) {
    std::fs::create_dir_all("./coefficients").unwrap();
    let mut f = File::create(format!(
        "coefficients/{}-fractal-{}-{}.coef",
        layer_channel, center.re, center.im
    ))
    .expect("Unable to create coef file");
    for i in layer {
        write!(f, "{}\n", i).unwrap();
    }
}

const ALPHABET_SIZE: usize = 512;

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

fn insert_after_none_starting_from(element: i32, i: usize, vec: &mut Vec<Option<i32>>) -> usize {
    let ind = (i..vec.len()).find(|j| vec[*j].is_some()).unwrap();
    vec[ind] = Some(element);
    ind + 1
}

#[derive(Debug)]
pub struct AnsContext {
    pub symbols: Vec<u32>,
    pub freqs: [u32; ALPHABET_SIZE],
}

impl AnsContext {
    fn new() -> Self {
        AnsContext {
            freqs: [0; ALPHABET_SIZE],
            symbols: (0..ALPHABET_SIZE).map(|x| x as u32).collect(),
        }
    }

    fn get_freqs(coefs: &Vec<u32>) -> [u32; ALPHABET_SIZE] {
        let mut freqs = [0; ALPHABET_SIZE];
        for coef in coefs {
            freqs[*coef as usize] += 1;
        }
        return freqs;
    }

    fn get_cdf(&self) -> Vec<u32> {
        self.freqs
            .iter()
            .scan(0_u32, |acc, x| {
                let val = *acc;
                *acc += x;
                Some(val)
            })
            .collect::<Vec<u32>>()
    }

    fn update_freqs(&mut self, new_freqs: [u32; ALPHABET_SIZE]) {
        for i in 0..ALPHABET_SIZE {
            self.freqs[i] += new_freqs[i];
        }
    }

    fn normalize_freqs(&mut self, target_total: u32) -> Vec<u32> {
        let mut cum_freqs = self.get_cdf();
        let cur_total = *cum_freqs.last().unwrap() + self.freqs.last().unwrap();
        for i in 1..cum_freqs.len() {
            cum_freqs[i] = ((target_total as u64 * cum_freqs[i] as u64) / cur_total as u64) as u32;
        }

        //NOTE:  Fixing nuked values -> commented out due to performance degradation
        for i in 0..cum_freqs.len() - 1 {
            if self.freqs[i] != 0 && cum_freqs[i+1]  == cum_freqs[i] {
                let mut best_freq: u32 = u32::MAX;
                let mut best_steal: usize = usize::MAX;
                for j in 0 .. cum_freqs.len() - 1 {
                    let freq = cum_freqs[j+1] - cum_freqs[j];
                    if freq > 1 && freq < best_freq {
                        best_freq = freq;
                        best_steal = j;
                    }
                }

                if best_steal < i {
                    for j in (best_steal+1)..=i {
                        cum_freqs[j] -= 1;
                    }
                } else {
                    for j in (i+1)..= best_steal {
                        cum_freqs[j] += 1;
                    }
                }

            }
        }

        for i in 0..(cum_freqs.len() - 1) {
            self.freqs[i] = cum_freqs[i + 1] - cum_freqs[i];
        }

        cum_freqs
    }

    fn freqs_to_enc_symbols(&self, max_freq_bits: u32) -> HashMap<u32, B64RansEncSymbol> {
        let cum_freqs = self.get_cdf();

        cum_freqs
            .iter()
            .zip(self.freqs.iter())
            .map(|(&cum_freq, &freq)| {
                (
                    cum_freq,
                    B64RansEncSymbol::new(cum_freq, freq, max_freq_bits),
                )
            })
            .collect()
    }

    fn freqs_to_dec_symbols(&self) -> HashMap<u32, B64RansDecSymbol> {
        let cum_freqs = self.get_cdf();

        cum_freqs
            .iter()
            .zip(self.freqs.iter())
            .map(|(&cum_freqs, &freqs)| (cum_freqs, B64RansDecSymbol::new(cum_freqs, freqs)))
            .collect()
    }
}

// TODO: Implement Alias sampling method
#[must_use]
fn find_nearest_or_equal(cum_freq: u32, cum_freqs: &[u32]) -> u32 {
    match cum_freqs.binary_search(&cum_freq) {
        Ok(x) => cum_freqs[x],
        Err(x) => cum_freqs[x-1],
    }
}

fn prepare_contexts(image: &WaveletImage, channel: usize) -> Vec<AnsContext> {
    let mut ans_contexts = vec![AnsContext::new()];

    for (_, fractal) in image.fractal_lattice.iter() {
        let mut layers: Vec<&[Option<i32>]> = vec![];
        let channel_coefficients = &fractal.coefficients[channel];

        layers.push(&channel_coefficients[..]);

        for sparse_layer in layers.into_iter() {
            let unpacked_layer = sparse_layer
                .into_iter()
                .flatten()
                .map(|c| *c)
                .collect::<Vec<i32>>();

            let layer: Vec<u32> = unpacked_layer.into_iter().map(pack_signed).collect();
            let freqs = AnsContext::get_freqs(&layer);

            ans_contexts[0].update_freqs(freqs);
        }
    }

    for ctx in ans_contexts.iter_mut() {
        let max_freq_bits =
            utils::get_prev_power_two(ctx.freqs.iter().sum::<u32>() as usize).trailing_zeros();
        ctx.normalize_freqs(1 << max_freq_bits);
    }

    ans_contexts
}

fn order_complex<T: std::cmp::PartialEq + std::cmp::PartialOrd>(
    a: &Complex<T>,
    b: &Complex<T>,
) -> Ordering {
    if a.re > b.re {
        Ordering::Greater
    } else if a.re < b.re {
        Ordering::Less
    } else if a.re == b.re && a.im > b.im {
        Ordering::Greater
    } else if a.re == b.re && a.im < b.im {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}

pub fn encode(image: WaveletImage, encoder_opts: &EncoderOpts) -> Result<CompressedImage, String> {
    let center = Complex::<i32>::new(
        image.metadata.width as i32 / 2,
        image.metadata.height as i32 / 2,
    );

    let middle = image
        .fractal_lattice
        .get(&center)
        .expect("middle must be inside of the image");

    let mut channel_data: [Option<(Vec<AnsContext>, Vec<u8>)>; 3] = [None, None, None];

    let mut sorted_keys: Vec<Complex<i32>> = image.fractal_lattice.keys().cloned().collect();
    sorted_keys.sort_by(order_complex);

    for channel in 0..image.metadata.colorspace.num_channels() {
        let ans_contexts = prepare_contexts(&image, channel);
        let mut encoder: B64RansEncoderMulti<1> =
            B64RansEncoderMulti::new(image.fractal_lattice.len() * 2 * (1 << middle.depth));

        for key in sorted_keys.iter() {
            let fractal = image.fractal_lattice.get(&key).unwrap();
            let mut layers: Vec<&[Option<i32>]> = vec![];
            let channel_coefficients = &fractal.coefficients[channel];

            //layers.push(&channel_coefficients[1 << (depth - 1)..]);
            //layers.push(&channel_coefficients[1 << (depth - 2)..1 << (depth - 1)]);
            //layers.push(&channel_coefficients[..1 << (depth - 2)]);
            layers.push(&channel_coefficients[..]);

            for (i, sparse_layer) in layers.into_iter().enumerate() {
                let unpacked_layer = sparse_layer
                    .into_iter()
                    .flatten()
                    .map(|c| *c)
                    .collect::<Vec<i32>>();

                let current_context = &ans_contexts[0];
                let max_freq_bits =
                    utils::get_prev_power_two(current_context.freqs.iter().sum::<u32>() as usize)
                        .trailing_zeros();

                let symbol_map = current_context.freqs_to_enc_symbols(max_freq_bits);
                let cdf = current_context.get_cdf();

                let cdf_map = current_context
                    .symbols
                    .iter()
                    .zip(cdf)
                    .collect::<HashMap<&u32, u32>>();

                let layer: Vec<u32> = unpacked_layer.into_iter().map(pack_signed).collect();
                layer
                    .iter()
                    .rev()
                    .for_each(|s| encoder.put_at(0, &symbol_map[&cdf_map[s]]));
            }
        }
        encoder.flush_all();
        let data = encoder.data().to_owned();
        channel_data[channel] = Some((ans_contexts, data));
    }
    Ok(CompressedImage {
        metadata: image.metadata,
        channel_data,
    })
}

pub fn decode(mut compressed_image: CompressedImage) -> Result<WaveletImage, String> {
    let mut decoded = WaveletImage::from_metadata(compressed_image.metadata);

    let mut sorted_keys: Vec<Complex<i32>> = decoded.fractal_lattice.keys().cloned().collect();
    sorted_keys.sort_by(order_complex);

    let mut channel = 0;
    while let Some((ans_contexts, bytes)) = compressed_image.channel_data[channel].take() {
        let mut decoder: B64RansDecoderMulti<1> = B64RansDecoderMulti::new(bytes);
        for key in sorted_keys.iter().rev() {
            let fractal = decoded.fractal_lattice.get_mut(&key).unwrap();

            let mut layers: Vec<usize> = vec![];
            layers.push(fractal.coefficients[channel][..].iter().flatten().count());

            let mut last = 0;
            for layer in layers.into_iter() {
                let cum_freqs = ans_contexts[0].get_cdf();
                let cum_freq_to_symbols = ans_contexts[0].freqs_to_dec_symbols();
                let symbol_map = cum_freqs
                    .clone()
                    .into_iter()
                    .zip(ans_contexts[0].symbols.clone())
                    .collect::<HashMap<u32, u32>>();

                let max_freq_bits =
                    utils::get_prev_power_two(ans_contexts[0].freqs.iter().sum::<u32>() as usize)
                        .trailing_zeros();

                for _l in 0..layer {
                    let cum_freq_decoded =
                        find_nearest_or_equal(decoder.get_at(0, max_freq_bits), &cum_freqs);
                    let symbol = symbol_map[&cum_freq_decoded];
                    decoder.advance_step_at(
                        0,
                        &cum_freq_to_symbols[&cum_freq_decoded],
                        max_freq_bits,
                    );
                    decoder.renorm_at(0);
                    last = insert_after_none_starting_from(
                        unpack_signed(symbol),
                        last,
                        &mut fractal.coefficients[channel],
                    );
                }
            }
        }
        channel += 1;
        if channel >= decoded.metadata.colorspace.num_channels() {
            break;
        }
    }

    return Ok(decoded);
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // fn cum_sum_test() {
    //     let x = [1, 2, 3, 4, 5];
    //     let cum_x = get_cdf(&x);
    //     assert_eq!(cum_x, &[0, 1, 3, 6, 10])
    // }

    #[test]
    fn logic_test() {}
}
