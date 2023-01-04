use crate::coord::Coord;
use crate::utils::bitwise;
use bmp::{Image, Pixel};
use std::cmp;
use std::vec;

pub struct Encoder {
    pub image: Image,
    width: u32,
    height: u32,
    variant: [Coord; 30],
    depth: usize,
    center: Coord,
    coef: Vec<u32>,
}

impl Encoder {
    pub fn new(image: Image, variant: [Coord; 30]) -> Self {
        let width: u32 = image.get_width();
        let height: u32 = image.get_height();
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
        let Pixel { r, g: _, b: _ } = self
            .image
            .get_pixel(x as u32 % self.width, y as u32 % self.width); // we assume grayscale for now
        r as u32
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, v: u32) -> () {
        let gray: u8 = cmp::max(0, cmp::min(v, 255)) as u8;
        self.image.set_pixel(
            x as u32 % self.width,
            y as u32 % self.height,
            Pixel {
                r: gray,
                g: gray,
                b: gray,
            },
        )
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

    pub fn quantizate(&mut self) {
        let total = 1 << self.depth;
        self.coef = self
            .coef
            .iter()
            .enumerate()
            .map(|(i, coefficient)| {
                coefficient / (2 * total / bitwise::get_next_power_two(i as u32))
            })
            .collect::<Vec<u32>>();
    }

    pub fn unquantizate(&mut self) {
        let total = 1 << self.depth;
        self.coef = self
            .coef
            .iter()
            .enumerate()
            .map(|(i, coefficient)| {
                coefficient * (2 * total / bitwise::get_next_power_two(i as u32))
            })
            .collect::<Vec<u32>>();
    }

    pub fn find_coef(&mut self) {
        let lt: u32 = self.fn_cf(self.center.clone(), 2, self.depth - 2);
        let rt: u32 = self.fn_cf(
            (self.center + self.variant[self.depth - 1]).clone(),
            3,
            self.depth - 2,
        );
        self.coef[1] = rt.wrapping_sub(lt);
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
        self.coef[ps] = rt.wrapping_sub(lt);
        rt.wrapping_add(lt)
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
