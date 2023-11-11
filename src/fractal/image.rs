use std::cmp;

use image::GrayImage;
use serde::{Deserialize, Serialize};

use crate::compression::ans::AnsContext;
use crate::fractal::variants::Variant;
use crate::utils::Coord;

#[derive(Serialize, Deserialize)]
pub struct Frv {
    height: u32,
    width: u32,
    variant: Variant,
    pub ans_contexts: Vec<AnsContext>,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

impl Frv {
    pub fn new(
        height: u32,
        width: u32,
        ans_contexts: Vec<AnsContext>,
        variant: Variant,
        data: Vec<u8>,
    ) -> Self {
        Self {
            height,
            width,
            variant,
            ans_contexts,
            data,
        }
    }
}

pub struct FractalImage {
    pub height: u32,
    pub width: u32,
    pub depth: usize,
    pub center: Coord,
    pub variant: [Coord; 30],
    pub coef: Vec<Option<i32>>,
    pub image: GrayImage,
}

impl FractalImage {
    pub fn new_from_img(image: GrayImage, variant: Variant) -> Self {
        let width: u32 = image.width();
        let height: u32 = image.height();
        let variant = variant.get_variant();
        let depth: usize = Self::calculate_depth(width, height, variant);
        let center = Coord {
            x: width as i32 / 2,
            y: height as i32 / 2,
        }; //;Self::find_center(depth, variant);
        dbg!(depth, center);

        Self {
            height,
            width,
            depth,
            center,
            variant,
            coef: vec![None; 1 << depth],
            image,
        }
    }

    pub fn new_from_frv(frv: &Frv) -> Self {
        let width = frv.width;
        let height = frv.height;
        let variant = frv.variant.get_variant();
        let depth = Self::calculate_depth(width, height, variant);
        let center = Coord {
            x: width as i32 / 2,
            y: height as i32 / 2,
        }; //;Self::find_center(depth, variant);

        Self {
            height,
            width,
            depth,
            center,
            variant,
            coef: vec![None; 1 << depth],
            image: GrayImage::new(width, height),
        }
    }

    fn calculate_depth(img_w: u32, img_h: u32, variant: [Coord; 30]) -> usize {
        variant
            .into_iter()
            .scan((0, 0, 0), |accum, value| {
                *accum = (accum.0 + 1, value.x.abs(), value.y.abs());
                Some(*accum)
            })
            .find(|&(_i, rw, rh)| img_w as i32 <= rw && img_h as i32 <= rh)
            .unwrap()
            .0
            -1 
    }

    fn find_center(depth: usize, variant: [Coord; 30]) -> Coord {
        variant[0..depth]
            .iter()
            .fold(Coord { x: 0, y: 0 }, |accum, value| Coord {
                x: accum.x - cmp::min(0, value.x),
                y: accum.y - cmp::min(0, value.y),
            })
    }

    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<i32> {
        let (x, y) = (x as u32, y as u32);
        if x < self.width && y < self.height {
            let [gray] = self.image.get_pixel(x % self.width, y % self.height).0; // we assume grayscale for now
            Some(i32::from(gray))
        } else {
            None
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, v: i32) {
        let (x, y) = (x as u32, y as u32);
        let gray = v.clamp(0, 255) as u8;
        if x < self.width && y < self.height {
            self.image.put_pixel(x, y, image::Luma([gray]));
        }
    }
}

#[must_use]
pub fn get_quantization_matrix_soft() -> Vec<i32> {
    vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 6, 2]
}

#[must_use]
pub fn get_quantization_matrix() -> Vec<i32> {
    vec![
        1, 1, 1,1,1,1,1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 10, 10, 10, 10, 10, 10, 16, 9, 2,
    ]
}
