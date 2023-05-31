use crate::coord::Coord;
use crate::utils::ans::AnsContext;
use crate::variants::{get_variant, Variant};
use image::GrayImage;
use serde::{Deserialize, Serialize};
use std::cmp;

pub struct FraveImage {
    pub height: u32,
    pub width: u32,
    pub depth: usize,
    pub center: Coord,
    pub variant: [Coord; 30],
    pub coef: Vec<i32>,
    pub image: GrayImage,
}

impl FraveImage {
    pub fn new_from_img(image: GrayImage, variant: Variant) -> Self {
        let width: u32 = image.width();
        let height: u32 = image.height();
        let variant = get_variant(variant);
        let depth: usize = Self::calculate_depth(width, height, variant);

        FraveImage {
            width,
            height,
            depth,
            center: Self::find_center(depth, variant),
            coef: vec![0; 1 << depth],
            image,
            variant,
        }
    }

    pub fn new_from_frv(frv: &Frv) -> Self {
        let width = frv.width;
        let height = frv.height;
        let variant = get_variant(frv.variant);
        let depth = Self::calculate_depth(width, height, variant);

        FraveImage {
            width,
            height,
            depth,
            center: Self::find_center(depth, variant),
            coef: vec![0; 1 << depth],
            variant,
            image: GrayImage::new(width, height),
        }
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
            .find(|&(_i, rw, rh)| rw <= 0 && rh <= 0)
            .unwrap()
            .0
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
    pub fn get_pixel(&self, x: i32, y: i32) -> i32 {
        let [gray] = self
            .image
            .get_pixel(x as u32 % self.width, y as u32 % self.height)
            .0; // we assume grayscale for now
        gray as i32
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, v: i32) {
        let gray: u8 = cmp::max(0, cmp::min(v, 255)) as u8;
        self.image.put_pixel(
            x as u32 % self.width,
            y as u32 % self.height,
            image::Luma([gray]),
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct Frv {
    height: u32,
    width: u32,
    variant: Variant,
    pub ans_contexts: Vec<AnsContext>,
    #[serde(with = "serde_bytes")]
    pub compressed_coef: Vec<u8>,
}

impl Frv {
    pub fn new(
        height: u32,
        width: u32,
        ans_contexts: Vec<AnsContext>,
        variant: Variant,
        compressed_coef: Vec<u8>,
    ) -> Self {
        Frv {
            height,
            width,
            ans_contexts,
            variant,
            compressed_coef,
        }
    }
}

pub fn get_quantization_matrix_soft() -> Vec<i32> {
    vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 6, 2]
}

pub fn get_quantization_matrix() -> Vec<i32> {
    vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 5, 9, 2]
}
