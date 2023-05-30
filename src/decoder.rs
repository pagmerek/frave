use std::collections::HashMap;

use rans::b64_decoder::B64RansDecoderMulti;
use rans::RansDecoderMulti;

use crate::coord::Coord;
use crate::frave_image::FraveImage;
use crate::utils::ans;
use crate::utils::ans::AnsContext;
use crate::utils::bitwise;

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

impl Decoder for FraveImage {
    fn unquantizate(&mut self, quantization_matrix: &[i32]) {
        self.coef = self
            .coef
            .iter()
            .enumerate()
            .map(|(i, coefficient)| {
                let layer = bitwise::get_prev_power_two(i as u32 + 1).trailing_zeros();
                *coefficient * quantization_matrix[layer as usize]
            })
            .collect::<Vec<i32>>();
    }

    fn find_val(&mut self) {
        self.fn_vl(self.coef[0], 1, self.center, self.depth - 1);
    }

    fn fn_vl(&mut self, sum: i32, ps: usize, cn: Coord, dp: usize) {
        let dif: i32 = self.coef[ps];
        let lt: i32 = ((sum - dif) * 2) >> 1;
        let rt: i32 = ((sum + dif) * 2) >> 1;
        if dp > 0 {
            self.fn_vl(lt, ps << 1, cn, dp - 1);
            self.fn_vl(rt, (ps << 1) + 1, cn + self.variant[dp], dp - 1)
        } else {
            self.set_pixel(cn.x, cn.y, lt);
            self.set_pixel(cn.x + self.variant[0].x, cn.y + self.variant[0].y, rt);
        }
    }

    fn ans_decode(&mut self, compressed_coef: Vec<u8>, ans_contexts: Vec<AnsContext>) {
        let mut coef = vec![];
        let depth = self.depth;
        let scale_bits = (depth - 1) as u32;
        let mut decoder: B64RansDecoderMulti<3> = B64RansDecoderMulti::new(compressed_coef);
        let ctxs = ans_contexts;
        let layers = vec![
            0..(1 << (depth - 2)),
            (1 << (depth - 2))..(1 << (depth - 1)),
            (1 << (depth - 1))..(1 << depth),
        ];
        for (i, layer) in layers.iter().enumerate() {
            let cum_freqs = ans::cum_sum(&ctxs[2 - i].freq);
            let cum_freq_to_symbols = ans::freqs_to_dec_symbols(&cum_freqs, &ctxs[2 - i].freq);
            let symbol_map = cum_freqs
                .iter()
                .map(|e| e.to_owned())
                .zip(ctxs[2 - i].symbols.clone())
                .collect::<HashMap<u32, i32>>();

            let mut cum_freqs_sorted = cum_freqs.to_owned();
            cum_freqs_sorted.sort();

            for _l in layer.clone() {
                let cum_freq_decoded =
                    ans::find_nearest_or_equal(decoder.get_at(i, scale_bits), &cum_freqs_sorted);
                let symbol = symbol_map[&cum_freq_decoded];
                decoder.advance_step_at(i, &cum_freq_to_symbols[&cum_freq_decoded], scale_bits);
                decoder.renorm_at(i);
                coef.push(symbol);
            }
        }
        self.coef = coef;
    }
}
