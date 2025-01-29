use std::collections::HashMap;
use std::vec;

use itertools::Itertools;
use rans::b64_encoder::B64RansEncoderMulti;
use rans::RansEncoderMulti;

use crate::compression::ans::{self, AnsContext};
use crate::fractal::image::FractalImage;
use crate::utils::{self, Coord};

fn try_apply<T: Copy>(first: Option<T>, second: Option<T>, operation: fn(T, T) -> T) -> Option<T> {
    match (first, second) {
        (Some(f), Some(s)) => Some(operation(f, s)),
        (Some(f), None) => Some(operation(f, f)),
        (None, Some(s)) => Some(operation(s, s)),
        (None, None) => None,
    }
}

pub trait Encoder {
    /// Transform step of encoding procedure.
    /// Using `fn_cf` recursively calculate Haar wavelets of input image.|
    /// For each recursion level divides the biggest fractal into 2 smaller one
    /// according to the complex numerical system describing the transform.
    /// The parts of the fractal that land outside the bitmap are mapped as None
    /// values and are skipped in entropy coding
    fn find_coef(&mut self);
    fn fn_cf(&mut self, cn: Coord, ps: usize, dp: usize) -> Option<i32>;

    /// Quantizations step of encoding procedure.
    /// Performs rounding down division on each Haar coefficient by a corresponding quantization parameter.
    /// Coefficients are grouped by levels of depth in the Haar tree and each level has its own parameter
    fn quantizate(&mut self, quantization_matrix: &[i32]);

    /// Entropy coding step of encoding procedure. Divides Haar coefficients to 3 distinct
    /// layers:
    ///
    /// - Haar tree leaves
    /// - Haar tree pre-last layer
    /// - Rest of the Haar tree
    ///
    /// Applies `rANS` algorithm for coefficient compression
    fn ans_encode(&self) -> (Vec<u8>, Vec<AnsContext>);
}

impl Encoder for FractalImage {
    fn find_coef(&mut self) {
        let lt = self.fn_cf(self.center, 2, self.depth - 2);
        let rt = self.fn_cf(
            self.center + self.variant[self.depth - 1],
            3,
            self.depth - 2,
        );
        self.coef[1] = try_apply(rt, lt, |r, l| (r - l) / 2);
        self.coef[0] = try_apply(rt, lt, |r, l| (r + l) / 2);
    }

    fn fn_cf(&mut self, cn: Coord, ps: usize, dp: usize) -> Option<i32> {
        let (lt, rt): (Option<i32>, Option<i32>);
        if dp > 0 {
            lt = self.fn_cf(cn, ps << 1, dp - 1);
            rt = self.fn_cf(cn + self.variant[dp], (ps << 1) + 1, dp - 1);
        } else {
            lt = self.get_pixel(cn.x, cn.y);
            rt = self.get_pixel(cn.x + self.variant[0].x, cn.y + self.variant[0].y);
        }

        self.coef[ps] = try_apply(rt, lt, |r, l| (r - l) / 2);
        try_apply(rt, lt, |r, l| (r + l) / 2)
    }

    fn quantizate(&mut self, quantization_matrix: &[i32]) {
        self.coef = self
            .coef
            .iter()
            .enumerate()
            .map(|(i, coefficient)| {
                let layer = utils::get_prev_power_two(i + 1).trailing_zeros();
                (*coefficient).map(|s| s / quantization_matrix[layer as usize])
            })
            .collect::<Vec<Option<i32>>>();
    }

    fn ans_encode(&self) -> (Vec<u8>, Vec<AnsContext>) {
        let valid_coef = &self
            .coef
            .clone()
            .into_iter()
            .flatten()
            .collect::<Vec<i32>>();
        let true_depth = utils::get_prev_power_two(valid_coef.len()).trailing_zeros() + 1;
        let layer1 = &valid_coef[1 << (true_depth - 1)..];
        let layer2 = &valid_coef[1 << (true_depth - 2)..1 << (true_depth - 1)];
        let layer3 = &valid_coef[..1 << (true_depth - 2)];

        let mut encoder: B64RansEncoderMulti<3> = B64RansEncoderMulti::new(valid_coef.len());
        let mut contexts: Vec<ans::AnsContext> = vec![];

        for (i, layer) in [layer1, layer2, layer3].into_iter().enumerate() {
            let counter = &(*layer).iter().counts();
            let freq = counter
                .values()
                .map(|e| u32::try_from(*e).unwrap())
                .collect::<Vec<u32>>();
            let symbols = counter.keys().map(|e| **e).collect::<Vec<i32>>();
            let cdf = ans::cum_sum(&freq);

            let symbol_map = ans::freqs_to_enc_symbols(&cdf, &freq, true_depth as usize);

            let cdf_map = counter
                .clone()
                .into_keys()
                .zip(cdf.clone())
                .collect::<HashMap<&i32, u32>>();

            layer
                .iter()
                .rev()
                .for_each(|s| encoder.put_at(i, &symbol_map[&cdf_map[s]]));

            contexts.push(ans::AnsContext { symbols, freq });
        }
        encoder.flush_all();

        (encoder.data().to_owned(), contexts)
    }
}
