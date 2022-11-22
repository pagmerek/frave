use crate::coord::Coord;
use bmp::Image;
use std::vec;

pub struct Frave {
    width: u32,
    height: u32,
    variant: &'static [Coord],
    // max_x: u32,
    // max_y: u32,
    // min_x: u32,
    // min_y: u32,
    pub depth: u32,
    center: Coord,
    image: Image,
    coef: Vec<u32>,
}

impl Frave {
    pub fn new(image: Image, variant: &'static [Coord]) -> Self {
        let img_w: u32 = image.get_width();
        let img_h: u32 = image.get_height();
        Frave {
            width: img_w,
            height: img_h,
            depth: Self::calculate_depth(img_w, img_h, variant),
            center: Coord { x: 0, y: 0 },
            coef: vec![],
            image,
            variant,
        }
    }

    #[inline]
    pub fn get_pixel(self, x: i32, y: i32) -> u32 {
        0
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
            .find(|&(_i, rw, rh)| rw <= 0 || rh <= 0)
            .unwrap()
            .0
            - 1
    }
}
