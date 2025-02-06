use crate::fractal::LITERALS;
use crate::images::{ImageMetadata, RasterImage};

use num::Complex;

fn try_apply<T: Copy>(first: Option<T>, second: Option<T>, operation: fn(T, T) -> T) -> Option<T> {
    match (first, second) {
        (Some(f), Some(s)) => Some(operation(f, s)),
        (Some(f), None) => Some(operation(f, f)),
        (None, Some(s)) => Some(operation(s, s)),
        (None, None) => None,
    }
}

pub struct WaveletImage {
    pub metadata: ImageMetadata,
    pub depth: u8,
    pub center: Complex<i32>,
    pub coefficients: [Vec<Option<i32>>; 3],
}

fn calculate_depth(img_w: u32, img_h: u32) -> u8 {
    LITERALS
        .iter()
        .scan((0, 0, 0), |accum, value| {
            *accum = (accum.0 + 1, value.re.abs(), value.im.abs());
            Some(*accum)
        })
        .find(|&(_i, rw, rh)| img_w as i32 <= rw && img_h as i32 <= rh)
        .unwrap()
        .0
        - 1
}

fn extract_coefficients(
    coefficients: &mut Vec<Option<i32>>,
    channel: usize,
    raster_image: &RasterImage,
    cn: Complex<i32>,
    ps: usize,
    dp: u8,
) -> Option<i32> {
    let (left_coef, right_coef): (Option<i32>, Option<i32>);
    if dp > 0 {
        left_coef = extract_coefficients(coefficients, channel, raster_image, cn, ps << 1, dp - 1);
        right_coef = extract_coefficients(
            coefficients,
            channel,
            raster_image,
            cn + LITERALS[dp as usize],
            (ps << 1) + 1,
            dp - 1,
        );
    } else {
        left_coef = raster_image.get_pixel(cn.re, cn.im, channel);
        right_coef =
            raster_image.get_pixel(cn.re + LITERALS[0].re, cn.im + LITERALS[0].im, channel);
    }

    coefficients[ps] = try_apply(right_coef, left_coef, |r, l| (r - l) / 2);
    try_apply(right_coef, left_coef, |r, l| (r + l) / 2)
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
        ps: usize,
        cn: Complex<i32>,
        dp: usize,
    ) {
        if let Some(dif) = wavelet_coefs[ps] {
            let left_subtree: i32 = ((sum - dif) * 2) >> 1;
            let right_subtree: i32 = ((sum + dif) * 2) >> 1;
            if dp > 0 {
                self.extract_values(wavelet_coefs, channel, left_subtree, ps << 1, cn, dp - 1);
                self.extract_values(
                    wavelet_coefs,
                    channel,
                    right_subtree,
                    (ps << 1) + 1,
                    cn + LITERALS[dp],
                    dp - 1,
                );
            } else {
                let secondary_x = cn.re + LITERALS[0].re;
                let secondary_y = cn.im + LITERALS[0].im;
                self.set_pixel(cn.re, cn.im, left_subtree, channel);
                self.set_pixel(secondary_x, secondary_y, right_subtree, channel);
            }
        }
    }
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
        let depth: u8 = calculate_depth(raster_image.metadata.width, raster_image.metadata.height);
        let center = Complex::<i32>::new(raster_image.metadata.width as i32 /2, raster_image.metadata.height as i32 /2);
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
            coefficients[i][1] = try_apply(right_coef, left_coef, |r, l| (r - l) / 2);
            coefficients[i][0] = try_apply(right_coef, left_coef, |r, l| (r + l) / 2);
        }
        WaveletImage {
            metadata: raster_image.metadata,
            depth,
            center,
            coefficients,
        }
    }
}

pub fn encode(raster_image: RasterImage) -> Result<WaveletImage, String> {
    Ok(WaveletImage::from_raster(raster_image))
}

pub fn decode(wavelet_image: WaveletImage) -> Result<RasterImage, String> {
    Ok(RasterImage::from_wavelet(wavelet_image))
}
