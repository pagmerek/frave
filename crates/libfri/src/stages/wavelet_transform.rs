use crate::fractal::{LITERALS, CENTERS};
use crate::images::{ImageMetadata, RasterImage};
use crate::encoder::EncoderOpts;

use num::Complex;

fn try_apply<T: Copy>(first: Option<T>, second: Option<T>, operation: fn(T, T) -> T, default: T) -> Option<T> {
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

//fn calculate_depth(img_w: u32, img_h: u32) -> u8 {
//    LITERALS
//        .iter()
//        .scan((0, 0, 0), |accum, value| {
//            *accum = (accum.0 + 1, value.re.abs(), value.im.abs());
//            Some(*accum)
//        })
//        .find(|&(_i, rw, rh)| img_w as i32 <= rw && img_h as i32 <= rh)
//        .unwrap()
//        .0
//        - 1
//}

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
            data: vec![0; wavelet_image.metadata.height as usize * wavelet_image.metadata.width as usize * wavelet_image.metadata.colorspace.num_channels()],
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
            let right_subtree: i32 = sum - dif/2;
            let left_subtree: i32 = dif + right_subtree;
            if depth > 0 {
                self.extract_values(wavelet_coefs, channel, left_subtree, position << 1, center, depth - 1);
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
    coefficients: &mut Vec<Option<i32>>,
    channel: usize,
    raster_image: &RasterImage,
    center: Complex<i32>,
    position: usize,
    depth: u8,
) -> Option<i32> {
    let (left_coef, right_coef): (Option<i32>, Option<i32>);
    if depth > 0 {
        left_coef = extract_coefficients(coefficients, channel, raster_image, center, position << 1, depth - 1);
        right_coef = extract_coefficients(
            coefficients,
            channel,
            raster_image,
            center + LITERALS[depth as usize],
            (position << 1) + 1,
            depth - 1,
        );
    } else {
        left_coef = raster_image.get_pixel(center.re, center.im, channel);
        right_coef =
            raster_image.get_pixel(center.re + LITERALS[0].re, center.im + LITERALS[0].im, channel);
    }

    coefficients[position] = try_apply(left_coef, right_coef, |r, l| (r - l), 0);
    try_apply(right_coef, coefficients[position], |r, l| (r + l/2), 0)
}
impl WaveletImage {
    pub fn from_metadata(metadata: ImageMetadata) -> WaveletImage {
        let image = RasterImage {
            data: vec![0; (metadata.width) as usize * (metadata.height) as usize * metadata.colorspace.num_channels()],
            metadata,
        };
        return Self::from_raster(image);
    }

    pub fn from_raster(raster_image: RasterImage) -> WaveletImage {
        let (depth, center) = calculate_depth_center(raster_image.metadata.width, raster_image.metadata.height);
        let mut coefficients = [vec![None; 1<<depth], vec![None; 1<<depth], vec![None; 1<<depth]];
        for i in 0..raster_image.metadata.colorspace.num_channels() {
            let left_coef =
                extract_coefficients(&mut coefficients[i], i, &raster_image, center, 2, depth - 2);
            let right_coef = extract_coefficients(
                &mut coefficients[i],
                i,
                &raster_image,
                center + LITERALS[depth as usize - 1],
                3,
                depth - 2,
            );
            coefficients[i][1] = try_apply(left_coef, right_coef, |r, l| (r - l), 0);
            coefficients[i][0] = try_apply(right_coef, coefficients[i][1], |r, l| (r + l/2), 0);
        }
        WaveletImage {
            metadata: raster_image.metadata,
            depth,
            center,
            coefficients,
        }
    }
}

pub fn encode(raster_image: RasterImage, _encoder_opts: &EncoderOpts) -> Result<WaveletImage, String> {
    let wavelet_img = WaveletImage::from_raster(raster_image);
    Ok(wavelet_img)
}

pub fn decode(wavelet_image: WaveletImage) -> Result<RasterImage, String> {
    Ok(RasterImage::from_wavelet(wavelet_image))
}
