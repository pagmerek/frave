use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::encoder::EncoderOpts;
use crate::fractal::{self, CENTERS, LITERALS};
use crate::images::{ImageMetadata, RasterImage};
use crate::utils;

use itertools::Position;
use num::complex::ComplexFloat;
use num::{Complex, Float};

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
    pub values: [Vec<Option<i32>>; 3],
    pub buckets: [Vec<u32>; 3],
    pub position_map: Vec<HashMap<Complex<i32>, usize>>,
    pub image_positions: Vec<Complex<i32>>,
}

impl Fractal {
    fn new(depth: u8, center: Complex<i32>) -> Self {
        let mut position_map = vec![HashMap::new(); depth as usize];
        let mut image_positions = vec![Complex::<i32>::new(0, 0); 1 << (depth + 1)];
        image_positions[0] = center;
        image_positions[1] = center;
        for level in 0..depth {
            for pos in 1 << level..1 << (level + 1) {
                position_map[level as usize].insert(image_positions[pos], pos);
                image_positions[2 * pos] = image_positions[pos];
                image_positions[2 * pos + 1] =
                    image_positions[pos] + LITERALS[(depth - level - 1) as usize];
            }
        }

        Fractal {
            depth,
            center,
            coefficients: [vec![], vec![], vec![]],
            buckets: [
                vec![0; 1 << depth],
                vec![0; 1 << depth],
                vec![0; 1 << depth],
            ],
            position_map,
            image_positions,
            values: [vec![], vec![], vec![]],
        }
    }

    fn get_nearby_vectors(depth: u8) -> [Complex<i32>; 6] {
        if depth == 0 {
            let zl = Complex::new(-1, 1);
            let zmd = Complex::new(-1,-1);
            return [zl, zl - zmd, -zmd, -zl, zmd - zl, zmd];
        } else if depth == 1 {
            let zl = Complex::new(2, 0);
            let zmd = Complex::new(2,-2);
            return [zl, zl - zmd, -zmd, -zl, zmd - zl, zmd];
        } else if depth == 2 {
            let zl = Complex::new(-3, -1);
            let zmd = Complex::new(2,-2);
            return [zl, zl - zmd, -zmd, -zl, zmd - zl, zmd];
        } else {
            let zl = LITERALS[depth as usize];
            let zmd = LITERALS[depth as usize + 1] + zl;

            return [zl, zl - zmd, -zmd, -zl, zmd - zl, zmd];
        }
    }

    fn get_sorted_neighbours(depth: u8) -> [Complex<i32>; 6] {
        let mut vectors = Self::get_nearby_vectors(depth);
        let rightmost = vectors.iter().max_by_key(|r| r.re).unwrap().clone();
        let rightmost_angle = (rightmost.im as f32).atan2(rightmost.re as f32);

        vectors.sort_by(|a, b| {
            let mut angle1 = (a.im as f32).atan2(a.re as f32) - rightmost_angle;
            if angle1 < 0. {
                angle1 += 2. * std::f32::consts::PI;
            }

            let mut angle2 = (b.im as f32).atan2(b.re as f32) - rightmost_angle;
            if angle2 < 0. {
                angle2 += 2. * std::f32::consts::PI;
            }
            angle1.total_cmp(&angle2)
        });
        vectors
    }

    pub fn get_neighbour_locations(&self) -> [Complex<i32>; 6] {
        let vectors = Self::get_nearby_vectors(self.depth);
        return vectors.map(|x| self.center + x).try_into().unwrap();
    }

    pub fn get_left(center: Complex<i32>, depth: u8) -> Complex<i32> {
        let vectors = Self::get_sorted_neighbours(depth);
        center + vectors[3]
    }

    pub fn get_right(center: Complex<i32>, depth: u8) -> Complex<i32> {
        let vectors = Self::get_sorted_neighbours(depth);
        center + vectors[0]
    }

    pub fn get_down_left(center: Complex<i32>, depth: u8) -> Complex<i32> {
        let vectors = Self::get_sorted_neighbours(depth);
        center + vectors[2]
    }

    pub fn get_down_right(center: Complex<i32>, depth: u8) -> Complex<i32> {
        let vectors = Self::get_sorted_neighbours(depth);
        center + vectors[1]
    }

    pub fn get_up_right(center: Complex<i32>, depth: u8) -> Complex<i32> {
        let vectors = Self::get_sorted_neighbours(depth);
        center + vectors[5]
    }

    pub fn get_up_left(center: Complex<i32>, depth: u8) -> Complex<i32> {
        let vectors = Self::get_sorted_neighbours(depth);
        center + vectors[4]
    }

    fn extract_coefficients(&mut self, raster_image: &RasterImage, depth: u8) {
        let mut coefficients = [
            vec![None; 1 << depth],
            vec![None; 1 << depth],
            vec![None; 1 << depth],
        ];

        let mut low_pass_values = [
            vec![None; 1 << depth],
            vec![None; 1 << depth],
            vec![None; 1 << depth],
        ];
;
        for channel in 0..raster_image.metadata.colorspace.num_channels() {
            for level in (0..depth).rev() {
                // compute high-pass and low-pass components
                for pos in 1 << level..1 << (level + 1) {
                    let (left_coef, right_coef): (Option<i32>, Option<i32>);
                    if level == depth - 1 {
                        left_coef = raster_image.get_pixel(
                            self.image_positions[2 * pos].re,
                            self.image_positions[2 * pos].im,
                            channel,
                        );
                        right_coef = raster_image.get_pixel(
                            self.image_positions[2 * pos + 1].re,
                            self.image_positions[2 * pos + 1].im,
                            channel,
                        );
                    } else {
                        left_coef = low_pass_values[channel][2 * pos];
                        right_coef = low_pass_values[channel][2 * pos + 1];
                    }
                    coefficients[channel][pos] =
                        try_apply(left_coef, right_coef, |l, r| (l - r), 0);
                    low_pass_values[channel][pos] = try_apply(
                        right_coef,
                        coefficients[channel][pos],
                        |l, r| (l + r / 2),
                        0,
                    );
                }
            }
            coefficients[channel][0] = low_pass_values[channel][1];
        }
        self.values = low_pass_values;
        self.coefficients = coefficients;
    }
}

fn calculate_depth_center(img_w: u32, img_h: u32) -> (u8, Complex<i32>) {
    let ((_, _), center, depth) = CENTERS
        .into_iter()
        .find(|&((w, h), _, _)| w >= (img_w as i32) && h >= (img_h as i32))
        .unwrap();

    return (depth, center);
}

fn color_pixel(raster: &mut RasterImage, key: &Complex<i32>, color: i32, channel: usize) {
    raster.set_pixel(key.re, key.im, color, channel);
    //raster.set_pixel(key.re+1, key.im, color, 0);
    //raster.set_pixel(key.re-1, key.im, color, 0);
    //raster.set_pixel(key.re, key.im+1, color, 0);
    //raster.set_pixel(key.re, key.im-1, color, 0);
}

fn get_containing_fractal(
    pos: &Complex<i32>,
    level: usize,
    fractal: &Fractal,
    fractal_lattice: &HashMap<Complex<i32>, Fractal>,
) -> Option<Complex<i32>> {
    for location in fractal.get_neighbour_locations() {
        if let Some(neighbour) = fractal_lattice.get(&location) {
            if neighbour.position_map[level].contains_key(&pos) {
                return Some(location);
            }
        }
    }
    None
}

fn get_hf_context_bucket(
    raster: &mut RasterImage,
    position: usize,
    current_depth: u8,
    parent_fractal_pos: &Complex<i32>,
    fractal_lattice: &HashMap<Complex<i32>, Fractal>,
    value_prediction_params: &[f32; 6],
    channel: usize,
) {
    assert!(current_depth > 0);
    let parent_level = current_depth as usize - 1;

    let fractal = &fractal_lattice[parent_fractal_pos];
    let position_in_image = fractal.image_positions[position];
    let parent_position_in_image = fractal.image_positions[position / 2];
    let neighbours = vec![
        Fractal::get_left(parent_position_in_image, fractal.depth - parent_level as u8),
        Fractal::get_up_left(parent_position_in_image, fractal.depth - parent_level as u8),
        Fractal::get_up_right(parent_position_in_image, fractal.depth - parent_level as u8),
        Fractal::get_right(parent_position_in_image, fractal.depth - parent_level as u8),
        Fractal::get_down_left(parent_position_in_image, fractal.depth - parent_level as u8),
        Fractal::get_down_right(parent_position_in_image, fractal.depth - parent_level as u8),
    ];

    let values: Vec<i32> = neighbours
        .iter()
        .map(|pos| {
            color_pixel(raster, pos, 255, 0);
            if fractal.position_map[parent_level].get(pos).is_none() {
                if let Some(nposition) =
                    get_containing_fractal(pos, parent_level, &fractal, fractal_lattice)
                {
                    let containing_fractal = &fractal_lattice[&nposition];
                    let loc = containing_fractal.position_map[parent_level][pos];
                    0
                } else {
                    println!("out of bounds {} {}", pos, parent_level);
                    //dbg!(&fractal.position_map[parent_level]);
                    0
                }
            } else {
                let loc = fractal.position_map[parent_level+1][pos];
                fractal.coefficients[channel][loc].unwrap_or(0)
            }
        })
        .collect();
}
impl RasterImage {
    pub fn from_wavelet(wavelet_image: WaveletImage) -> RasterImage {
        let sorted_keys: Vec<Complex<i32>> = wavelet_image.get_sorted_lattice();

        let mut raster = RasterImage {
            data: vec![
                0;
                wavelet_image.metadata.height as usize
                    * wavelet_image.metadata.width as usize
                    * wavelet_image.metadata.colorspace.num_channels()
            ],
            metadata: wavelet_image.metadata,
        };

        for (_center, fractal) in wavelet_image.fractal_lattice.iter() {
            raster.extract_values(&fractal);
        }

        if false {
            let center = Complex::<i32>::new(
                raster.metadata.width as i32 / 2,
                raster.metadata.height as i32 / 2,
            );
            let fractal = &wavelet_image.fractal_lattice[&center];
            let depth = fractal.depth;

            for level in 2..depth {
                for pos in 1 << level..1 << (level + 1) {
                        get_hf_context_bucket(
                            &mut raster,
                            pos,
                            level,
                            &center,
                            &wavelet_image.fractal_lattice,
                            &[1., 1., 1., 1., 1., 1.],
                            0,
                        );
                }
            }
            let find = Complex::new(220, 129);
            for (center, fractal) in wavelet_image.fractal_lattice.iter() {
                for (i, dep) in fractal.position_map.iter().enumerate() {
                    if(dep.contains_key(&find)) {
                        println!("found! {} {}", i, center);
                    }
                    

                }
                color_pixel(&mut raster, center, 255, 2);
            }
        }

        return raster;
    }

    fn extract_values(&mut self, fractal: &Fractal) {
        for channel in 0..self.metadata.colorspace.num_channels() {
            let mut low_pass_values = vec![0; 1 << fractal.depth];
            low_pass_values[1] = fractal.coefficients[channel][0].unwrap();

            for level in 0..fractal.depth {
                for pos in 1 << level..1 << (level + 1) {
                    if let Some(dif) = fractal.coefficients[channel][pos] {
                        let right_subtree: i32 = low_pass_values[pos] - dif / 2;
                        let left_subtree: i32 = dif + right_subtree;
                        if level == fractal.depth - 1 {
                            let left_pixel = fractal.image_positions[2 * pos];
                            let right_pixel = fractal.image_positions[2 * pos + 1];
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
        let mut fractal_lattice =
            Self::fractal_divide(raster_image.metadata.width, raster_image.metadata.height, 9);

        for (_, fractal) in fractal_lattice.iter_mut() {
            fractal.extract_coefficients(&raster_image, fractal.depth);
        }
        fractal_lattice
            .retain(|_, frac| frac.coefficients.iter().all(|channel| channel[0].is_some()));

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

    pub fn get_sorted_lattice(&self) -> Vec<Complex<i32>> {
        let mut keys: Vec<Complex<i32>> = self.fractal_lattice.keys().cloned().collect();
        let depth = self.fractal_lattice[&keys[0]].depth;

        let mut sorted_fractalwise: Vec<Complex<i32>> = Vec::new();

        let mut first = keys.iter().min_by_key(|a| a.re + a.im).unwrap().clone();

        loop {
            let mut scan = first;
            loop {
                if self.fractal_lattice.get(&scan).is_some() {
                    sorted_fractalwise.push(scan);
                }
                if scan.re > self.metadata.width as i32 {
                    break;
                }
                scan = Fractal::get_right(scan, depth);
            }

            if sorted_fractalwise.len() == keys.len() {
                break;
            }
            first = Fractal::get_down_left(first, depth);
        }
        sorted_fractalwise
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
