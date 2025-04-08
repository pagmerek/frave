use crate::encoder::EncoderOpts;
use crate::images::CompressedImage;
use crate::stages::prediction;
use crate::stages::wavelet_transform::{Fractal, WaveletImage};
use crate::{fractal, utils};

use core::f32;
use num::Complex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use rans::b64_decoder::{B64RansDecSymbol, B64RansDecoderMulti};
use rans::b64_encoder::{B64RansEncSymbol, B64RansEncoderMulti};
use rans::RansDecoderMulti;
use rans::RansEncoderMulti;
use rans::{RansDecSymbol, RansEncSymbol};

use super::prediction::get_hf_context_bucket;

pub const ALPHABET_SIZE: usize = 1024;

fn get_first_some_starting_from(i: usize, vec: &Vec<Option<i32>>) -> usize {
    (i..vec.len()).find(|j| vec[*j].is_some()).unwrap()
}

#[derive(Debug, Clone)]
pub struct AnsContext {
    pub symbols: Vec<u32>,
    pub freqs: [u32; ALPHABET_SIZE],
    pub cdf: [u32; ALPHABET_SIZE],
    pub freqs_to_enc_symbols: Vec<B64RansEncSymbol>,
    pub freqs_to_dec_symbols: HashMap<u32, B64RansDecSymbol>,
    pub max_freq_bits: u32,
}

impl AnsContext {
    pub fn new() -> Self {
        AnsContext {
            freqs: [0; ALPHABET_SIZE],
            cdf: [0; ALPHABET_SIZE],
            symbols: (0..ALPHABET_SIZE).map(|x| x as u32).collect(),
            freqs_to_enc_symbols: Vec::new(),
            freqs_to_dec_symbols: HashMap::new(),
            max_freq_bits: 0,
        }
    }

    fn get_freqs(coefs: &Vec<u32>) -> [u32; ALPHABET_SIZE] {
        let mut freqs = [0; ALPHABET_SIZE];
        for coef in coefs {
            freqs[*coef as usize] += 1;
        }
        return freqs;
    }

    fn get_cdf(&self) -> [u32; ALPHABET_SIZE] {
        self.freqs
            .iter()
            .scan(0_u32, |acc, x| {
                let val = *acc;
                *acc += x;
                Some(val)
            })
            .collect::<Vec<u32>>()
            .try_into().unwrap()
    }

    fn update_freqs(&mut self, new_freqs: [u32; ALPHABET_SIZE]) {
        for i in 0..ALPHABET_SIZE {
            self.freqs[i] += new_freqs[i];
        }
    }

    pub fn bump_freq(&mut self, element: u32) {
        self.freqs[element as usize] += 1;
    }

    pub fn finalize_context(&mut self, normalize: bool) {
        if normalize {
            let max_freq_bits =
                utils::get_prev_power_two(self.freqs.iter().sum::<u32>() as usize).trailing_zeros();
            self.cdf = self.normalize_freqs(1 << max_freq_bits);
        } else {
            self.cdf = self.get_cdf();
        }
        self.max_freq_bits = utils::get_prev_power_two(self.freqs.iter().sum::<u32>() as usize).trailing_zeros();
        self.freqs_to_enc_symbols = self.get_freqs_to_enc_symbols();
        self.freqs_to_dec_symbols = self.get_freqs_to_dec_symbols();
    }

    pub fn normalize_freqs(&mut self, target_total: u32) -> [u32; ALPHABET_SIZE] {
        let mut cum_freqs = self.get_cdf();
        let cur_total = *cum_freqs.last().unwrap() + self.freqs.last().unwrap();
        for i in 1..cum_freqs.len() {
            cum_freqs[i] = ((target_total as u64 * cum_freqs[i] as u64) / cur_total as u64) as u32;
        }

        //Fixing 0 freq values
        for i in 0..cum_freqs.len() - 1 {
            if self.freqs[i] != 0 && cum_freqs[i + 1] == cum_freqs[i] {
                let mut best_freq: u32 = u32::MAX;
                let mut best_steal: usize = usize::MAX;
                for j in 0..cum_freqs.len() - 1 {
                    let freq = cum_freqs[j + 1] - cum_freqs[j];
                    if freq > 1 && freq < best_freq {
                        best_freq = freq;
                        best_steal = j;
                    }
                }

                if best_steal < i {
                    for j in (best_steal + 1)..=i {
                        cum_freqs[j] -= 1;
                    }
                } else {
                    for j in (i + 1)..=best_steal {
                        cum_freqs[j] += 1;
                    }
                }
            }
        }

        for i in 0..(cum_freqs.len() - 1) {
            self.freqs[i] = cum_freqs[i + 1] - cum_freqs[i];
        }
        self.freqs[self.freqs.len() - 1] = cum_freqs[self.freqs.len() - 1] - target_total;
        cum_freqs
    }

    fn get_freqs_to_enc_symbols(&self) -> Vec<B64RansEncSymbol> {
        self.cdf
            .iter()
            .zip(self.freqs.iter())
            .map(|(&cum_freq, &freq)| B64RansEncSymbol::new(cum_freq, freq, self.max_freq_bits))
            .collect()
    }

    fn get_freqs_to_dec_symbols(&self) -> HashMap<u32, B64RansDecSymbol> {
        self.cdf
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
        Err(x) => cum_freqs[x - 1],
    }
}

pub fn encode_symbol<const T: usize>(
    value: i32,
    position: usize,
    depth: u8,
    parent_pos: &Complex<i32>,
    channel: usize,
    ans_contexts: &Vec<AnsContext>,
    fractal_lattice: &HashMap<Complex<i32>, Fractal>,
    encoder: &mut B64RansEncoderMulti<T>,
) {
    let (bucket, prediction) = if position == 0 || position == 1 {
        prediction::get_lf_context_bucket(position, 0, parent_pos, &fractal_lattice, channel)
    } else {
        prediction::get_hf_context_bucket(position, depth, parent_pos, &fractal_lattice, channel)
    };
    let current_context = &ans_contexts[bucket];
    let symbol_map = &current_context.freqs_to_enc_symbols;
    encoder.put_at(
        bucket,
        &symbol_map[utils::pack_signed(value - prediction) as usize],
    );
}


fn decode_symbol<const T: usize>(
    position: usize,
    depth: u8,
    parent_pos: &Complex<i32>,
    channel: usize,
    ans_contexts: &Vec<AnsContext>,
    fractal_lattice: &HashMap<Complex<i32>, Fractal>,
    decoder: &mut B64RansDecoderMulti<T>,
) -> i32 {
    let (bucket, prediction) = if position == 0 || position == 1 {
        prediction::get_lf_context_bucket(position, 0, parent_pos, &fractal_lattice, channel)
    } else {
        prediction::get_hf_context_bucket(position, depth, parent_pos, &fractal_lattice, channel)
    };

    let current_context = &ans_contexts[bucket];
    let cum_freq_to_symbols = &current_context.freqs_to_dec_symbols;

    let cum_freq_decoded = find_nearest_or_equal(decoder.get_at(bucket, current_context.max_freq_bits), &current_context.cdf);
    let symbol = current_context.cdf.iter().position(|&r| r == cum_freq_decoded).unwrap() as u32;
    decoder.advance_step_at(
        bucket,
        &cum_freq_to_symbols[&cum_freq_decoded],
        current_context.max_freq_bits,
    );
    decoder.renorm_at(bucket);

    return utils::unpack_signed(symbol) + prediction;
}

pub fn encode(
    image: WaveletImage,
    contexts: [Vec<AnsContext>; 3],
    encoder_opts: &EncoderOpts,
) -> Result<CompressedImage, String> {
    let mut channel_data: [Option<(Vec<AnsContext>, Vec<u8>)>; 3] = [None, None, None];

    let mut sorted_keys: Vec<Complex<i32>> = image.fractal_lattice.keys().cloned().collect();
    sorted_keys.sort_by(utils::order_complex);

    let global_depth = image.fractal_lattice[&sorted_keys[0]].depth;
    for channel in 0..image.metadata.colorspace.num_channels() {
        let mut encoder: B64RansEncoderMulti<3> =
            B64RansEncoderMulti::new(image.fractal_lattice.len() * 2 * (1 << global_depth));

        // First scan -> Low frequency coefficients
        for key in sorted_keys.iter() {
            let fractal = &image.fractal_lattice[&key];
            if let Some(value) = fractal.coefficients[channel][0] {
                encode_symbol(value, 0, global_depth, key, channel, &contexts[channel], &image.fractal_lattice, &mut encoder);
            }
        }

        // Second scan -> High frequency coefficient root
        for key in sorted_keys.iter() {
            let fractal = &image.fractal_lattice[&key];
            if let Some(value) = fractal.coefficients[channel][0] {
                encode_symbol(value, 1, global_depth, key, channel, &contexts[channel], &image.fractal_lattice, &mut encoder);
            }
        }

        // Remaining levels
        for level in 1..image.fractal_lattice[&sorted_keys[0]].depth {
            for key in sorted_keys.iter() {
                let fractal = &image.fractal_lattice[&key];
                for pos in 1 << level..1 << (level + 1) {
                    if let Some(value) = fractal.coefficients[channel][pos] {
                        encode_symbol(
                            value,
                            pos,
                            level,
                            key,
                            channel,
                            &contexts[channel],
                            &image.fractal_lattice,
                            &mut encoder,
                        );
                    }
                }
            }
        }

        encoder.flush_all();
        let data = encoder.data().to_owned();
        let bpp = data.len() as f32 / (image.metadata.width * image.metadata.height) as f32 * 8.;
        println!("bits per pixel: {}", bpp);
        channel_data[channel] = Some((contexts[channel].clone(), data));
    }
    Ok(CompressedImage {
        metadata: image.metadata,
        channel_data,
    })
}

pub fn decode(mut compressed_image: CompressedImage) -> Result<WaveletImage, String> {
    let mut decoded = WaveletImage::from_metadata(compressed_image.metadata);

    let mut sorted_keys: Vec<Complex<i32>> = decoded.fractal_lattice.keys().cloned().collect();
    sorted_keys.sort_by(utils::order_complex);
    let mut channel = 0;
    let global_depth = decoded.fractal_lattice[&sorted_keys[0]].depth;
    while let Some((ans_contexts, bytes)) = compressed_image.channel_data[channel].take() {
        let mut decoder: B64RansDecoderMulti<3> = B64RansDecoderMulti::new(bytes);
        // First scan -> Low frequency coefficients
        for key in sorted_keys.iter() {
            let symbol = decode_symbol(0, global_depth, key, channel, &ans_contexts, &decoded.fractal_lattice, &mut decoder);
            let fractal = decoded.fractal_lattice.get_mut(&key).unwrap();
            fractal.coefficients[channel][0] = Some(symbol);
        }

        // Second scan -> High frequency coefficient root
        for key in sorted_keys.iter() {
            let symbol = decode_symbol(1, global_depth, key, channel, &ans_contexts, &decoded.fractal_lattice, &mut decoder);
            let fractal = decoded.fractal_lattice.get_mut(&key).unwrap();
            fractal.coefficients[channel][1] = Some(symbol);
        }

        // Remaining levels
        for level in 1..global_depth {
            for key in sorted_keys.iter().rev() {
                for pos in 1 << level..1 << (level + 1) {
                    if decoded.fractal_lattice[key].coefficients[channel][pos].is_none() {
                        continue;
                    }
                    let symbol = decode_symbol(
                        pos,
                        level,
                        key,
                        channel,
                        &ans_contexts,
                        &decoded.fractal_lattice,
                        &mut decoder,
                    );

                    let fractal = decoded.fractal_lattice.get_mut(&key).unwrap();
                    fractal.coefficients[channel][pos] = Some(symbol);
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
