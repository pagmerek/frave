
use std::collections::HashMap;

use lstsq::lstsq;
use nalgebra::Dynamic;
use nalgebra::{self as na, DMatrix, DVector, U2};
use num::Complex;

use crate::stages::wavelet_transform::Fractal;
use crate::stages::wavelet_transform::WaveletImage;

pub struct ContextModeler {
    pub parameters: [f32; 6],
}

impl ContextModeler {
    pub fn new() -> Self {
        ContextModeler { 
            parameters: [0.0; 6]
        }
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

    pub fn search_for_fractal(depth: usize, pos: &Complex<i32>, fractal: &Fractal, fractal_lattice: &HashMap<Complex<i32>, Fractal>, channel: usize) -> i32 {
        if fractal.position_map[depth].get(pos).is_none() {
            if let Some(nposition) =
                Self::get_containing_fractal(pos, depth, &fractal, fractal_lattice)
            {
                let containing_fractal = &fractal_lattice[&nposition];
                let loc = containing_fractal.position_map[depth][pos];
                containing_fractal.coefficients[channel][loc].unwrap_or(0)
            } else {
                0
            }
        } else {
            let loc = fractal.position_map[depth][pos];
            fractal.coefficients[channel][loc].unwrap_or(0)
        }
    }

    pub fn get_neighbour_values(
        position: usize,
        current_depth: u8,
        parent_fractal_pos: &Complex<i32>,
        fractal_lattice: &HashMap<Complex<i32>, Fractal>,
        channel: usize,
    ) -> Vec<i32> {
        assert!(current_depth > 0);
        let parent_level = current_depth as usize - 1;

        let fractal = &fractal_lattice[parent_fractal_pos];
        let position_in_image = fractal.image_positions[position];
        let parent_position_in_image = fractal.image_positions[position / 2];
        let neighbours = vec![
            (parent_level,  Fractal::get_left(position_in_image, fractal.depth - parent_level as u8)),
            (parent_level,  Fractal::get_up_left(position_in_image, fractal.depth - parent_level as u8)),
            (parent_level,  Fractal::get_up_right(position_in_image, fractal.depth - parent_level as u8)),
            (parent_level,  Fractal::get_right(parent_position_in_image, fractal.depth - parent_level as u8)),
            (parent_level,  Fractal::get_down_left(parent_position_in_image, fractal.depth - parent_level as u8)),
            (parent_level,  Fractal::get_down_right(parent_position_in_image, fractal.depth - parent_level as u8)),
        ];

        neighbours
            .iter()
            .map(|(depth, pos)| 
                Self::search_for_fractal(*depth, pos, fractal, fractal_lattice, channel))
            .collect()
    }

    pub fn optimize_parameters(&self, wavelet_image: &WaveletImage, channel: usize) -> [f32; 6] {
        let global_depth = wavelet_image.fractal_lattice.values().nth(0).unwrap().depth;
        let num_ctx = wavelet_image.fractal_lattice.len() * (1 << global_depth);
        let num_parameters = 6; 
        let mut position_matrix = DMatrix::<f32>::zeros(num_ctx, num_parameters);
        let sorted_keys = wavelet_image.get_sorted_lattice();
        let mut values = DVector::<f32>::zeros(num_ctx);
        for (i, key) in sorted_keys.iter().enumerate() {
            for level in 1..global_depth{
                let fractal = &wavelet_image.fractal_lattice[key];
                for pos in 1 << level..1 << (level + 1) {
                    if let Some(value) = fractal.coefficients[channel][pos] {
                        let vals = Self::get_neighbour_values(
                            pos,
                            level,
                            key,
                            &wavelet_image.fractal_lattice,
                            channel,
                        );
                        values[i*(1<<global_depth as usize) + pos] = value as f32;
                        for j in 0..6 {
                            position_matrix[(i*(1<<global_depth as usize) + pos, j)] = vals[j] as f32;
                        }
                    } else {
                        values[i*(1<<global_depth as usize) + pos] = 0 as f32;

                        for j in 0..6 {
                            position_matrix[(i*(1<<global_depth as usize) + pos, j)] = 0 as f32;
                        }
                    }
               }
            }
        }
        let results = lstsq(&position_matrix, &values, 1e-14).unwrap();
        return [
            results.solution[0],
            results.solution[1],
            results.solution[2],
            results.solution[3],
            results.solution[4],
            results.solution[5],
        ]

    } 
}


#[cfg(test)]
mod test {
    use crate::images::ImageMetadata;

    use super::*;

    #[test]
    fn unit_test() {
        let wavelet_image = WaveletImage::from_metadata(ImageMetadata::new(444,258));
        let ctx_modeler = ContextModeler::new();
        ctx_modeler.optimize_parameters(&wavelet_image, 0);


    }
}
