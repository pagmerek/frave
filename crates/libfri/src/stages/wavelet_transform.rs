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

pub struct WaveletImage {
    pub metadata: ImageMetadata,
    pub depth: u8,
    pub center: Complex<i32>,
    pub coefficients: [Vec<Option<i32>>; 3],
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

        for i in 0..raster.metadata.colorspace.num_channels() {
            if let Some(root) = wavelet_image.coefficients[i][0] {
                raster.extract_values(
                    &wavelet_image.coefficients[i],
                    i,
                    root,
                    1,
                    wavelet_image.center,
                    wavelet_image.depth as usize - 1,
                );
            }
        }
        return raster;
    }

    fn extract_values(
        &mut self,
        wavelet_coefs: &Vec<Option<i32>>,
        channel: usize,
        sum: i32,
        position: usize,
        center: Complex<i32>,
        depth: usize,
    ) {
        if let Some(dif) = wavelet_coefs[position] {
            let right_subtree: i32 = sum - dif / 2;
            let left_subtree: i32 = dif + right_subtree;
            if depth > 0 {
                self.extract_values(
                    wavelet_coefs,
                    channel,
                    left_subtree,
                    position << 1,
                    center,
                    depth - 1,
                );
                self.extract_values(
                    wavelet_coefs,
                    channel,
                    right_subtree,
                    (position << 1) + 1,
                    center + LITERALS[depth],
                    depth - 1,
                );
            } else {
                let secondary_x = center.re + LITERALS[0].re;
                let secondary_y = center.im + LITERALS[0].im;
                self.set_pixel(center.re, center.im, left_subtree, channel);
                self.set_pixel(secondary_x, secondary_y, right_subtree, channel);
            }
        }
    }
}

fn extract_coefficients(
    raster_image: &RasterImage,
    center: Complex<i32>,
    depth: u8,
) -> [Vec<Option<i32>>; 3] {
    //fn extract_coefficients(
    //    coefficients: &mut Vec<Option<i32>>,
    //    channel: usize,
    //    raster_image: &RasterImage,
    //    center: Complex<i32>,
    //    position: usize,
    //    depth: u8,
    //) -> Option<i32> {
    //    let (left_coef, right_coef): (Option<i32>, Option<i32>);
    //    if depth > 0 {
    //        left_coef = extract_coefficients(coefficients, channel, raster_image, center, position << 1, depth - 1);
    //        right_coef = extract_coefficients(
    //            coefficients,
    //            channel,
    //            raster_image,
    //            center + LITERALS[depth as usize],
    //            (position << 1) + 1,
    //            depth - 1,
    //        );
    //    } else {
    //        left_coef = raster_image.get_pixel(center.re, center.im, channel);
    //        right_coef =
    //            raster_image.get_pixel(center.re + LITERALS[0].re, center.im + LITERALS[0].im, channel);
    //    }
    //
    //    coefficients[position] = try_apply(left_coef, right_coef, |r, l| (r - l), 0);
    //    try_apply(right_coef, coefficients[position], |r, l| (r + l/2), 0)
    //}
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
                coefficients[channel][pos] = try_apply(left_coef, right_coef, |l, r| (l - r), 0);
                low_pass_values[pos] = try_apply(
                    right_coef,
                    coefficients[channel][pos],
                    |l, r| (l + r / 2),
                    0,
                );
            }
            // compute local slope
            for pos in 1 << level..1 << (level + 1) {
                let (left_coef, right_coef): (Option<i32>, Option<i32>);
                if pos - 1 < 1 << level {
                    left_coef = low_pass_values[pos];
                } else {
                    left_coef = low_pass_values[pos - 1];
                }
                if pos + 1 >= 1 << (level + 1) {
                    right_coef = low_pass_values[pos];
                } else {
                    right_coef = low_pass_values[pos + 1];
                }

                if coefficients[channel][pos].is_some() && left_coef.is_some() && right_coef.is_some() {
                    let slope = try_apply(left_coef, right_coef, |l, r| (l - r + 2)/4, 0);

                    // update
                    coefficients[channel][pos] =
                        try_apply(coefficients[channel][pos], slope, |l, r| l + r, 0)
                }
            }
        }
        coefficients[channel][0] = low_pass_values[1];
    }
    coefficients
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
        let (depth, center) =
            calculate_depth_center(raster_image.metadata.width, raster_image.metadata.height);
        let coefficients = extract_coefficients(&raster_image, center, depth);
        //    let left_coef =
        //        extract_coefficients(&mut coefficients[i], i, &raster_image, center, 2, depth - 2);
        //    let right_coef = extract_coefficients(
        //        &mut coefficients[i],
        //        i,
        //        &raster_image,
        //        center + LITERALS[depth as usize - 1],
        //        3,
        //        depth - 2,
        //    );
        //    coefficients[i][1] = try_apply(left_coef, right_coef, |r, l| (r - l), 0);
        //    coefficients[i][0] = try_apply(right_coef, coefficients[i][1], |r, l| (r + l/2), 0);
        //}
        WaveletImage {
            metadata: raster_image.metadata,
            depth,
            center,
            coefficients,
        }
    }
}

pub fn encode(
    raster_image: RasterImage,
    _encoder_opts: &EncoderOpts,
) -> Result<WaveletImage, String> {
    let wavelet_img = WaveletImage::from_raster(raster_image);
    Ok(wavelet_img)
}

pub fn decode(wavelet_image: WaveletImage) -> Result<RasterImage, String> {
    Ok(RasterImage::from_wavelet(wavelet_image))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extract_coefficient_test() {
        let img = RasterImage {
            metadata: ImageMetadata {
                height: 8,
                width: 8,
                colorspace: crate::images::ColorSpace::RGB,
                variant: crate::images::FractalVariant::TameTwindragon,
            },
            data: vec![10; 8 * 8 * 3],
        };

        let (depth, center) = calculate_depth_center(img.metadata.width, img.metadata.height);

        let coefficients = extract_coefficients(&img, center, depth - 1);
        dbg!(&coefficients[0].iter().flatten().count());
    }
}
