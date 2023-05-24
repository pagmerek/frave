use std::collections::HashMap;

use rans::b64_decoder::B64RansDecSymbol;
use rans::b64_encoder::B64RansEncSymbol;
use rans::{RansDecSymbol, RansEncSymbol};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnsContext {
    pub symbols: Vec<i32>,
    pub freq: Vec<u32>,
}

pub fn cum_sum(sum: &[u32]) -> Vec<u32> {
    sum.iter()
        .scan(0_u32, |acc, x| {
            let val = *acc;
            *acc += x;
            Some(val)
        })
        .collect::<Vec<u32>>()
}

pub fn find_nearest_or_equal(cum_freq: u32, cum_freqs: &[u32]) -> u32 {
    match cum_freqs.binary_search(&cum_freq) {
        Ok(x) => cum_freqs[x],
        Err(x) => cum_freqs[x - 1],
    }
}

pub fn freqs_to_enc_symbols(
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
                B64RansEncSymbol::new(cum_freq, freq, (depth - 1) as u32),
            )
        })
        .collect::<HashMap<u32, B64RansEncSymbol>>()
}

pub fn freqs_to_dec_symbols(cum_freq: &[u32], freq: &[u32]) -> HashMap<u32, B64RansDecSymbol> {
    cum_freq
        .iter()
        .zip(freq.iter())
        .map(|(&cum_freq, &freq)| (cum_freq, B64RansDecSymbol::new(cum_freq, freq)))
        .collect::<HashMap<u32, B64RansDecSymbol>>()
}
