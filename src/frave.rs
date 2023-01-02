use crate::coord::Coord;
use bmp::{Image, Pixel};
use std::cmp;
use std::vec;

pub struct Frave {
    pub width: u32,
    pub height: u32,
    variant: &'static [Coord],
    // max_x: u32,
    // max_y: u32,
    // min_x: u32,
    // min_y: u32,
    pub depth: u32,
    pub center: Coord,
    pub image: Image,
    pub coef: Vec<u32>,
}

impl Frave {
    pub fn new(image: Image, variant: &'static [Coord]) -> Self {
        let width: u32 = image.get_width();
        let height: u32 = image.get_height();
        let depth: u32 = Self::calculate_depth(width, height, variant);

        Frave {
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
    pub fn get_pixel(&self, x: u32, y: u32) -> u8 {
        let Pixel { r, g: _, b: _ } = self.image.get_pixel(x % self.width, y % self.width); // we assume grayscale for now
        r
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, v: i32) -> () {
        let gray: u8 = cmp::max(0, cmp::min(v, 255)) as u8;
        self.image.set_pixel(
            x % self.width,
            y % self.height,
            Pixel {
                r: gray,
                g: gray,
                b: gray,
            },
        )
    }

    fn calculate_depth(img_w: u32, img_h: u32, variant: &'static [Coord]) -> u32 {
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

    fn find_center(depth: u32, variant: &'static [Coord]) -> Coord {
        variant[0..depth as usize]
            .into_iter()
            .fold(Coord { x: 0, y: 0 }, |accum, value| Coord {
                x: accum.x - cmp::min(0, value.x),
                y: accum.y - cmp::min(0, value.y),
            })
    }

    pub fn trim_coef(&mut self, dp: u32) {
        if dp < self.depth {
            let mut x = (&self.coef[0..(1 << dp) as usize]).to_vec();
            x.extend(vec![0 as u32; (1 << self.depth) - (1 << dp)]);
            self.coef = x;
        }
    }

    pub fn quantizate_coef() {
      self.depth  
    }

    pub fn find_coef(&mut self) {
        let lt: i32 = self.fn_cf(self.center.clone(), 2, self.depth as i32 - 2);
        let rt: i32 = self.fn_cf(
            (self.center + self.variant[(self.depth - 1) as usize]).clone(),
            3,
            self.depth as i32 - 2,
        );
        self.coef[1] = (rt - lt) as u32;
        self.coef[0] = (rt + lt) as u32;
    }

    fn fn_cf(&mut self, cn: Coord, ps: i32, dp: i32) -> i32 {
        let (mut lt, mut rt) = (0, 0);
        if dp > 0 {
            lt = self.fn_cf(cn, ps << 1, dp - 1);
            rt = self.fn_cf(cn + self.variant[dp as usize], (ps << 1) + 1, dp - 1);
        } else {
            lt = self.get_pixel(cn.x as u32, cn.y as u32) as i32;
            rt = self.get_pixel(
                (cn.x + self.variant[0].x) as u32,
                (cn.y + self.variant[0].y) as u32,
            ) as i32;
        }
        self.coef[ps as usize] = (rt - lt) as u32;
        rt + lt
    }

    pub fn find_val(&mut self) {
        self.fn_vl(self.coef[0] as i32, 1, self.center, (self.depth - 1) as i32);
    }

    fn fn_vl(&mut self, sum: i32, ps: i32, cn: Coord, dp: i32) {
        let dif: i32 = self.coef[ps as usize] as i32;
        let lt: i32 = (sum - dif) >> 1;
        let rt: i32 = (sum + dif) >> 1;
        if dp > 0 {
            self.fn_vl(lt, ps << 1, cn, dp - 1);
            self.fn_vl(rt, (ps << 1) + 1, cn + self.variant[dp as usize], dp - 1)
        } else {
            self.set_pixel(cn.x as u32, cn.y as u32, lt);
            self.set_pixel(
                (cn.x + self.variant[0].x) as u32,
                (cn.y + self.variant[0].y) as u32,
                rt,
            );
        }
    }
}
