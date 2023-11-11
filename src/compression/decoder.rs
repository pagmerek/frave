use std::collections::HashMap;

use itertools::Itertools;
use rans::b64_decoder::B64RansDecoderMulti;
use rans::RansDecoderMulti;

use crate::compression::ans::{self, AnsContext};
use crate::fractal::image::FractalImage;
use crate::utils::{self, Coord};

use super::encoder::Encoder;

pub trait Decoder {
    /// Entropy decoding step of decoder procedure.
    ///
    /// Decodes compressed bit stream using provided context.
    fn ans_decode(&mut self, compressed_coef: Vec<u8>, ans_contexts: Vec<AnsContext>);

    /// Unquantization step of decoder procedure.
    ///
    /// Performs multiplication on each Haar coefficient by a corresponding quantization parameter.
    /// Coefficients are grouped by levels of depth in the Haar tree and each level has its own parameter
    fn unquantizate(&mut self, quantization_matrix: &[i32]);

    /// Transform step of decoder procedure.
    ///
    /// Populates the image using `fn_vl` recursively, which calculates the proper luma values for
    /// each pixel
    fn find_val(&mut self);
    fn fn_vl(&mut self, sum: i32, ps: usize, cn: Coord, dp: usize);
}

impl Decoder for FractalImage {
    fn unquantizate(&mut self, quantization_matrix: &[i32]) {
        self.coef = self
            .coef
            .iter()
            .enumerate()
            .map(|(i, coefficient)| {
                let layer = utils::get_prev_power_two(i + 1).trailing_zeros();
                (*coefficient).map(|s| s * quantization_matrix[layer as usize])
            })
            .collect::<Vec<Option<i32>>>();
    }

    fn find_val(&mut self) {
        if let Some(root) = self.coef[0] {
            self.fn_vl(root, 1, self.center, self.depth - 1);
        } else {
            println!("whoops");
        }
    }

    fn fn_vl(&mut self, sum: i32, ps: usize, cn: Coord, dp: usize) {
        if let Some(dif) = self.coef[ps] {
            let lt: i32 = ((sum - dif) * 2) >> 1;
            let rt: i32 = ((sum + dif) * 2) >> 1;
            if dp > 0 {
                self.fn_vl(lt, ps << 1, cn, dp - 1);
                self.fn_vl(rt, (ps << 1) + 1, cn + self.variant[dp], dp - 1);
            } else {
                let secondary_x = cn.x + self.variant[0].x;
                let secondary_y = cn.y + self.variant[0].y;
                self.set_pixel(cn.x, cn.y, lt);
                self.set_pixel(secondary_x, secondary_y, rt);
            }
        }
    }

    fn ans_decode(&mut self, compressed_coef: Vec<u8>, ans_contexts: Vec<AnsContext>) {
        self.find_coef();
        let length = self.coef.iter().filter(|x| x.is_some()).count();
        let mut coef = self.coef.clone();
        let depth = utils::get_prev_power_two(length).trailing_zeros() + 1;
        let scale_bits = u32::try_from(depth - 1).unwrap();
        let mut decoder: B64RansDecoderMulti<3> = B64RansDecoderMulti::new(compressed_coef);
        let ctxs = ans_contexts;
        let layers = vec![
            0..1 << (depth - 2),
            1 << (depth - 2)..1 << (depth - 1),
            1 << (depth - 1)..length,
        ];
        let mut last = 0;
        for (i, layer) in layers.iter().enumerate() {
            let mut cum_freqs = ans::cum_sum(&ctxs[2 - i].freq);
            let cum_freq_to_symbols = ans::freqs_to_dec_symbols(&cum_freqs, &ctxs[2 - i].freq);
            let symbol_map = cum_freqs
                .clone()
                .into_iter()
                .zip(ctxs[2 - i].symbols.clone())
                .collect::<HashMap<u32, i32>>();

            cum_freqs.sort_unstable();

            for _l in layer.clone() {
                let cum_freq_decoded =
                    ans::find_nearest_or_equal(decoder.get_at(i, scale_bits), &cum_freqs);
                let symbol = symbol_map[&cum_freq_decoded];
                decoder.advance_step_at(i, &cum_freq_to_symbols[&cum_freq_decoded], scale_bits);
                decoder.renorm_at(i);
                last = insert_after_none_starting_from(symbol, last, &mut coef);
            }
        }
        self.coef = coef;
    }
}

fn insert_after_none_starting_from(element: i32, i: usize, vec: &mut Vec<Option<i32>>) -> usize {
    let ind = (i..vec.len()).find(|j| vec[*j].is_some()).unwrap();
    vec[ind] = Some(element);
    ind + 1
}
