use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::encoder::EncoderOpts;
use crate::fractal::{CENTERS, LITERALS};
use crate::images::{ImageMetadata, RasterImage};

use num::Complex;

fn try_apply<T: Copy>(
    first: Option<T>,
    second: Option<T>,
    operation: fn(T, T) -> T,
    default: T,
) -> Option<T> {
    match (first, second) {
        (Some(f), Some(s)) => Some(operation(f, s)),
        (Some(f), None) => Some(operation(f, default)),
        (None, Some(s)) => Some(operation(default, s)),
        (None, None) => None,
    }
}

#[derive(Debug)]
pub struct Fractal {
    pub depth: u8,
    pub center: Complex<i32>,
    pub coefficients: [Vec<Option<i32>>; 3],
    pub context: Option<usize>,
}

impl Fractal {
    fn new(depth: u8, center: Complex<i32>) -> Self {
        Fractal {
            depth,
            center,
            coefficients: [vec![], vec![], vec![]],
            context: None,
        }
    }
    //fn get_left(&self) -> Complex<i32> {
    //}
    //
    //fn get_right(&self) -> Complex<i32> {
    //}
    //
    //fn get_up_left(&self) -> Complex<i32> {
    //}
    //fn get_up_right(&self) -> Complex<i32> {
    //}
    //fn get_down_left(&self) -> Complex<i32> {
    //}
    //fn get_down_right(&self) -> Complex<i32> {
    //}
    fn get_neighbour_locations(&self) -> [Complex<i32>; 6] {
        let zl = LITERALS[self.depth as usize];
        let zmd = LITERALS[self.depth as usize + 1] + zl;

        return [
            self.center + zl,
            self.center + zl - zmd,
            self.center - zmd,
            self.center - zl,
            self.center + zmd - zl,
            self.center + zmd,
        ];
    }
}

fn calculate_depth_center(img_w: u32, img_h: u32) -> (u8, Complex<i32>) {
    let ((_, _), center, depth) = CENTERS
        .into_iter()
        .find(|&((w, h), _, _)| w >= (img_w as i32) && h >= (img_h as i32))
        .unwrap();

    return (depth, center);
}

impl RasterImage {
    pub fn from_wavelet(wavelet_image: WaveletImage) -> RasterImage {
        let mut raster = RasterImage {
            data: vec![
                0;
                wavelet_image.metadata.height as usize
                    * wavelet_image.metadata.width as usize
                    * wavelet_image.metadata.colorspace.num_channels()
            ],
            metadata: wavelet_image.metadata,
        };

        for (center, fractal) in wavelet_image.fractal_lattice.iter() {
            raster.extract_values(&fractal.coefficients, fractal.center, fractal.depth);
        }

        return raster;
    }

    fn extract_values(
        &mut self,
        wavelet_coefs: &[Vec<Option<i32>>; 3],
        center: Complex<i32>,
        depth: u8,
    ) {
        let mut image_positions = vec![Complex::<i32>::new(0, 0); 1 << (depth + 1)];
        image_positions[1] = center;
        for level in 0..depth {
            for pos in 1 << level..1 << (level + 1) {
                image_positions[2 * pos] = image_positions[pos];
                image_positions[2 * pos + 1] =
                    image_positions[pos] + LITERALS[(depth - level - 1) as usize];
            }
        }
        for channel in 0..self.metadata.colorspace.num_channels() {
            let mut low_pass_values = vec![0; 1 << depth];
            low_pass_values[1] = wavelet_coefs[channel][0].unwrap();

            for level in 0..depth {
                for pos in 1 << level..1 << (level + 1) {
                    if let Some(dif) = wavelet_coefs[channel][pos] {
                        let right_subtree: i32 = low_pass_values[pos] - dif / 2;
                        let left_subtree: i32 = dif + right_subtree;
                        if level == depth - 1 {
                            let left_pixel = image_positions[2 * pos];
                            let right_pixel = image_positions[2 * pos + 1];
                            self.set_pixel(left_pixel.re, left_pixel.im, left_subtree, channel);
                            self.set_pixel(right_pixel.re, right_pixel.im, right_subtree, channel);
                        } else {
                            low_pass_values[2 * pos] = left_subtree;
                            low_pass_values[2 * pos + 1] = right_subtree;
                        }
                    }
                }
            }
        }
    }
}

pub struct WaveletImage {
    pub metadata: ImageMetadata,
    pub fractal_lattice: HashMap<Complex<i32>, Fractal>,
}

impl WaveletImage {
    pub fn from_metadata(metadata: ImageMetadata) -> WaveletImage {
        let image = RasterImage {
            data: vec![
                0;
                (metadata.width) as usize
                    * (metadata.height) as usize
                    * metadata.colorspace.num_channels()
            ],
            metadata,
        };
        return Self::from_raster(image);
    }

    pub fn from_raster(raster_image: RasterImage) -> WaveletImage {
        //let (depth, center) =
        //    calculate_depth_center(raster_image.metadata.width, raster_image.metadata.height);

        let mut fractal_lattice =
            Self::fractal_divide(raster_image.metadata.width, raster_image.metadata.height, 9);

        for (_, fractal) in fractal_lattice.iter_mut() {
            fractal.coefficients =
                Self::extract_coefficients(&raster_image, fractal.center, fractal.depth);
        }
        fractal_lattice.retain(|_, frac| frac.coefficients.iter().all(|channel| channel[0].is_some()));

        WaveletImage {
            metadata: raster_image.metadata,
            fractal_lattice,
        }
    }

    fn fractal_divide(width: u32, height: u32, depth: u8) -> HashMap<Complex<i32>, Fractal> {
        let mut fractal_lattice = HashMap::<Complex<i32>, Fractal>::new();
        let center = Complex::<i32>::new(width as i32 / 2, height as i32 / 2);
        let mut to_add = VecDeque::<Complex<i32>>::new();
        to_add.push_back(center);

        let mut boundary = VecDeque::<Complex<i32>>::new();

        while let Some(position) = to_add.pop_front() {
            if position.re < 0
                || position.im < 0
                || position.re > width as i32
                || position.im > height as i32
            {
                boundary.push_back(position);
                continue;
            }

            let fractal = Fractal::new(depth, position);
            for neighbour in fractal.get_neighbour_locations() {
                if !fractal_lattice.contains_key(&neighbour) && !to_add.contains(&neighbour) {
                    to_add.push_back(neighbour);
                }
            }

            fractal_lattice.insert(position, fractal);
        }

        while let Some(position) = boundary.pop_front() {
            let boundary_fractal = Fractal::new(depth, position);
            fractal_lattice.insert(position, boundary_fractal);
        }

        fractal_lattice
    }

    fn extract_coefficients(
        raster_image: &RasterImage,
        center: Complex<i32>,
        depth: u8,
    ) -> [Vec<Option<i32>>; 3] {
        let mut coefficients = [
            vec![None; 1 << depth + 1],
            vec![None; 1 << depth + 1],
            vec![None; 1 << depth + 1],
        ];
        let mut image_positions = vec![Complex::<i32>::new(0, 0); 1 << (depth + 1)];
        image_positions[1] = center;
        for level in 0..depth {
            for pos in 1 << level..1 << (level + 1) {
                image_positions[2 * pos] = image_positions[pos];
                image_positions[2 * pos + 1] =
                    image_positions[pos] + LITERALS[(depth - level - 1) as usize];
            }
        }

        for channel in 0..raster_image.metadata.colorspace.num_channels() {
            let mut low_pass_values = vec![None; 1 << depth];
            for level in (0..depth).rev() {
                // compute high-pass and low-pass components
                for pos in 1 << level..1 << (level + 1) {
                    let (left_coef, right_coef): (Option<i32>, Option<i32>);
                    if level == depth - 1 {
                        left_coef = raster_image.get_pixel(
                            image_positions[2 * pos].re,
                            image_positions[2 * pos].im,
                            channel,
                        );
                        right_coef = raster_image.get_pixel(
                            image_positions[2 * pos + 1].re,
                            image_positions[2 * pos + 1].im,
                            channel,
                        );
                    } else {
                        left_coef = low_pass_values[2 * pos];
                        right_coef = low_pass_values[2 * pos + 1];
                    }
                    coefficients[channel][pos] =
                        try_apply(left_coef, right_coef, |l, r| (l - r), 0);
                    low_pass_values[pos] = try_apply(
                        right_coef,
                        coefficients[channel][pos],
                        |l, r| (l + r / 2),
                        0,
                    );
                }
                // compute local slope
                //for pos in 1 << level..1 << (level + 1) {
                //    if coefficients[channel][pos].is_some() {
                //        let (left_coef, right_coef): (Option<i32>, Option<i32>);
                //        let left_ind = (1<<level..pos).rev().find(|e| low_pass_values[*e].is_some());
                //        let right_ind = (pos..1<<(level+1)).find(|e| low_pass_values[*e].is_some());
                //
                //        if let Some(ind) = left_ind {
                //            left_coef = low_pass_values[ind];
                //        } else {
                //            left_coef = low_pass_values[pos];
                //        }
                //
                //        if let Some(ind) = right_ind {
                //            right_coef = low_pass_values[ind];
                //        } else {
                //            right_coef = low_pass_values[pos];
                //        }
                //
                //        let slope = try_apply(left_coef, right_coef, |l, r| (l - r) >> 2, 0);
                //
                //        // update
                //        coefficients[channel][pos] =
                //            try_apply(coefficients[channel][pos], slope, |l, r| l - r, 0)
                //    }
                //}
            }
            coefficients[channel][0] = low_pass_values[1];
        }
        coefficients
    }
}

pub fn encode(
    raster_image: RasterImage,
    _encoder_opts: &EncoderOpts,
) -> Result<WaveletImage, String> {
    Ok(WaveletImage::from_raster(raster_image))
}

pub fn decode(wavelet_image: WaveletImage) -> Result<RasterImage, String> {
    Ok(RasterImage::from_wavelet(wavelet_image))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extract_coefficient_test() {
        //let img = RasterImage {
        //    metadata: ImageMetadata {
        //        height: 8,
        //        width: 8,
        //        colorspace: crate::images::ColorSpace::RGB,
        //        variant: crate::images::FractalVariant::TameTwindragon,
        //    },
        //    data: vec![10; 8 * 8 * 3],
        //};

        //let (depth, center) = calculate_depth_center(img.metadata.width, img.metadata.height);

        //let coefficients = extract_coefficients(&img, center, depth - 1);
    }
}
