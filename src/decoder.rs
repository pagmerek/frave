use std::cmp;
use std::collections::HashMap;

use image::{GrayImage};
use rans::b64_decoder::B64RansDecoderMulti;
use rans::RansDecoderMulti;

use crate::coord::Coord;
use crate::frave_image::FraveImage;
use crate::utils::ans;
use crate::utils::bitwise;
use crate::variants::get_variant;

pub struct Decoder {
    width: u32,
    height: u32,
    variant: [Coord; 30],
    depth: usize,
    center: Coord,
    coef: Vec<u32>,
    pub image: GrayImage,
}

impl Decoder {
    pub fn new(frv: FraveImage) -> Self {
        let width: u32 = frv.width;
        let height: u32 = frv.height;
        let depth: usize = frv.depth;
        let mut coef = vec![];

        let scale_bits = (depth - 1) as u32;
        let mut decoder: B64RansDecoderMulti<3> = B64RansDecoderMulti::new(frv.compressed_coef);
        let ctxs = frv.ans_contexts;
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
                .collect::<HashMap<u32, u32>>();

            let mut cum_freqs_sorted = cum_freqs.to_owned();
            cum_freqs_sorted.sort();

            for _ in layer.to_owned() {
                let cum_freq_decoded =
                    ans::find_nearest_or_equal(decoder.get_at(i, scale_bits), &cum_freqs_sorted);
                let symbol = symbol_map[&cum_freq_decoded];
                decoder.advance_step_at(i, &cum_freq_to_symbols[&cum_freq_decoded], scale_bits);
                decoder.renorm_at(i);
                coef.push(symbol);
            }
        }
        Decoder {
            width: width,
            height:height,
            depth: depth,
            center: Coord {
                x: frv.center.0,
                y: frv.center.1,
            },
            coef,
            image: GrayImage::new(width, height),
            variant: get_variant(frv.variant),
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, v: u32) -> () {
        let gray: u8 = cmp::max(0, cmp::min(v, 255)) as u8;
        self.image.put_pixel(
            x as u32 % self.width,
            y as u32 % self.height,
              image::Luma([gray]) 
        )
    }

    pub fn unquantizate(&mut self) {
        let total = 1 << self.depth;
        self.coef = self
            .coef
            .iter()
            .enumerate()
            .map(|(i, coefficient)| {
                let layer: u32 = bitwise::get_next_power_two(i as u32).trailing_zeros();
                coefficient * ((2u32.pow(layer) as f64).sqrt() as u32)
            })
            .collect::<Vec<u32>>();
    }

    pub fn find_val(&mut self) {
        self.fn_vl(self.coef[0], 1, self.center, self.depth - 1);
    }

    fn fn_vl(&mut self, sum: u32, ps: usize, cn: Coord, dp: usize) {
        let dif: u32 = self.coef[ps];
        let lt: u32 = sum.wrapping_sub(dif) >> 1;
        let rt: u32 = sum.wrapping_add(dif) >> 1;
        if dp > 0 {
            self.fn_vl(lt, ps << 1, cn, dp - 1);
            self.fn_vl(rt, (ps << 1) + 1, cn + self.variant[dp], dp - 1)
        } else {
            self.set_pixel(cn.x, cn.y, lt);
            self.set_pixel(cn.x + self.variant[0].x, cn.y + self.variant[0].y, rt);
        }
    }
}
