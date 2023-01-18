use std::cmp;
use std::collections::HashMap;
use std::vec;

use image::{GrayImage};
use rans::b64_encoder::B64RansEncoderMulti;
use rans::RansEncoderMulti;

use crate::coord::Coord;
use crate::utils::ans;
use crate::utils::bitwise;
use itertools::Itertools;

pub struct Encoder {
    pub image: GrayImage,
    pub width: u32,
    pub height: u32,
    pub depth: usize,
    pub center: Coord,
    pub coef: Vec<u32>,
    variant: [Coord; 30],
}

impl Encoder {
    pub fn new(image: GrayImage, variant: [Coord; 30]) -> Self {
        let width: u32 = image.width();
        let height: u32 = image.height();
        let depth: usize = Self::calculate_depth(width, height, variant);

        Encoder {
            width,
            height,
            depth,
            center: Self::find_center(depth, variant),
            coef: vec![0; 1 << depth],
            image,
            variant,
        }
    }

    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> u32 {
        let [gray] = self
            .image
            .get_pixel(x as u32 % self.width, y as u32 % self.height).0; // we assume grayscale for now
         gray as u32
    }

    fn calculate_depth(img_w: u32, img_h: u32, variant: [Coord; 30]) -> usize {
        variant
            .into_iter()
            .scan((0, img_w as i32, img_h as i32), |accum, value| {
                *accum = (
                    accum.0 + 1,
                    accum.1 - value.x.abs(),
                    accum.2 - value.y.abs(),
                );
                Some(*accum)
            })
            .find(|&(_i, rw, rh)| rw < 0 && rh < 0)
            .unwrap()
            .0
    }

    fn find_center(depth: usize, variant: [Coord; 30]) -> Coord {
        variant[0..depth]
            .into_iter()
            .fold(Coord { x: 0, y: 0 }, |accum, value| Coord {
                x: accum.x - cmp::min(0, value.x),
                y: accum.y - cmp::min(0, value.y),
            })
    }

    pub fn trim_coef(&mut self, dp: usize) {
        if dp < self.depth {
            let mut x = (&self.coef[0..(1 << dp)]).to_vec();
            x.extend(vec![0; (1 << self.depth) - (1 << dp)]);
            self.coef = x;
        }
    }
    pub fn find_coef(&mut self) {
        let lt: u32 = self.fn_cf(self.center.clone(), 2, self.depth - 2);
        let rt: u32 = self.fn_cf(
            (self.center + self.variant[self.depth - 1]).clone(),
            3,
            self.depth - 2,
        );
        self.coef[1] = rt.saturating_sub(lt);
        self.coef[0] = rt.wrapping_add(lt);
    }

    fn fn_cf(&mut self, cn: Coord, ps: usize, dp: usize) -> u32 {
        let (lt, rt): (u32, u32);
        if dp > 0 {
            lt = self.fn_cf(cn, ps << 1, dp - 1);
            rt = self.fn_cf(cn + self.variant[dp], (ps << 1) + 1, dp - 1);
        } else {
            lt = self.get_pixel(cn.x, cn.y);
            rt = self.get_pixel(cn.x + self.variant[0].x, cn.y + self.variant[0].y);
        }
        self.coef[ps] = rt.saturating_sub(lt);
        rt.wrapping_add(lt)
    }

    pub fn quantizate(&mut self) {
        let total = 1 << self.depth;
        self.coef = self
            .coef
            .iter()
            .enumerate()
            .map(|(i, coefficient)| {
                let layer: u32 = bitwise::get_next_power_two(i as u32).trailing_zeros();
                //dbg!(i, layer, &self.depth,coefficient, coefficient / ((2 as f64).sqrt() as u32).pow(2*layer) );
                coefficient / ((2u32.pow(layer) as f64).sqrt() as u32) 
            })
            .collect::<Vec<u32>>();
    }

    pub fn ans_encode(&self) -> (Vec<u8>, Vec<ans::AnsContext>) {
        let layer1 = &self.coef[1 << (self.depth - 1)..];
        let layer2 = &self.coef[1 << (self.depth - 2)..1 << (self.depth - 1)];
        let layer3 = &self.coef[..1 << (self.depth - 2)];

        let mut encoder: B64RansEncoderMulti<3> = B64RansEncoderMulti::new(1 << self.depth);
        let mut contexts: Vec<ans::AnsContext> = vec![];

        for (i, layer) in [layer1, layer2, layer3].iter().enumerate() {
            let counter = layer.into_iter().counts();
            let freq = counter.values().map(|e| *e as u32).collect::<Vec<u32>>();
            let symbols = counter.keys().map(|e| **e as u32).collect::<Vec<u32>>();
            let cum_freq = ans::cum_sum(&freq);

            let symbol_map = ans::freqs_to_enc_symbols(&cum_freq, &freq, self.depth);

            let cum_freq_map = counter
                .clone()
                .into_keys()
                .map(|e| e.to_owned())
                .zip(cum_freq.clone())
                .collect::<HashMap<u32, u32>>();

            layer
                .iter()
                .rev()
                .for_each(|s| encoder.put_at(i, &symbol_map[&cum_freq_map[s]]));

            contexts.push(ans::AnsContext { symbols, freq });
        }
        encoder.flush_all();

        (encoder.data().to_owned(), contexts)
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