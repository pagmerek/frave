use std::collections::HashMap;

use lstsq::lstsq;
use nalgebra::Dynamic;
use nalgebra::{self as na, DMatrix, DVector, U2};
use num::Complex;

use crate::stages::wavelet_transform::Fractal;
use crate::stages::wavelet_transform::WaveletImage;

#[derive(Debug)]
pub struct ContextModeler {
    pub value_predictors: [Vec<[f32; 6]>; 3],
    pub width_predictors: [Vec<[f32; 2]>; 3],
}

impl ContextModeler {
    pub fn new() -> Self {
        ContextModeler {
            value_predictors: [vec![], vec![], vec![]],
            width_predictors: [vec![], vec![], vec![]],
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

    pub fn search_for_fractal(
        depth: usize,
        pos: &Complex<i32>,
        fractal: &Fractal,
        fractal_lattice: &HashMap<Complex<i32>, Fractal>,
        channel: usize,
    ) -> i32 {
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
        let parent_level = current_depth as usize;

        let fractal = &fractal_lattice[parent_fractal_pos];
        let position_in_image = fractal.image_positions[position];
        let parent_position_in_image = fractal.image_positions[position / 2];
        let neighbours = vec![
            (
                parent_level,
                Fractal::get_left(position_in_image, fractal.depth - parent_level as u8),
            ),
            (
                parent_level,
                Fractal::get_up_left(position_in_image, fractal.depth - parent_level as u8),
            ),
            (
                parent_level,
                Fractal::get_up_right(position_in_image, fractal.depth - parent_level as u8),
            ),
            (
                parent_level,
                Fractal::get_right(parent_position_in_image, fractal.depth - parent_level as u8),
            ),
            (
                parent_level,
                Fractal::get_down_left(
                    parent_position_in_image,
                    fractal.depth - parent_level as u8,
                ),
            ),
            (
                parent_level,
                Fractal::get_down_right(
                    parent_position_in_image,
                    fractal.depth - parent_level as u8,
                ),
            ),
        ];

        neighbours
            .iter()
            .map(|(depth, pos)| {
                Self::search_for_fractal(*depth, pos, fractal, fractal_lattice, channel)
            })
            .collect()
    }

    fn get_image_neighbour_matrices(
        wavelet_image: &WaveletImage,
        global_depth: u8,
        channel: usize,
    ) -> (Vec<DVector<f32>>, Vec<DMatrix<f32>>) {
        let num_ctx_last_layer = wavelet_image.fractal_lattice.len() * (1 << (global_depth - 1));
        let num_ctx_middle_layer = wavelet_image.fractal_lattice.len() * (1 << (global_depth - 2));
        let num_parameters = 6;
        let mut matrices = vec![
            DMatrix::<f32>::zeros(num_ctx_last_layer, num_parameters),
            DMatrix::<f32>::zeros(num_ctx_middle_layer, num_parameters),
            DMatrix::<f32>::zeros(num_ctx_middle_layer, num_parameters),
        ];

        let mut value_vectors = vec![
            DVector::<f32>::zeros(num_ctx_last_layer),
            DVector::<f32>::zeros(num_ctx_middle_layer),
            DVector::<f32>::zeros(num_ctx_middle_layer),
        ];

        let sorted_lattice = wavelet_image.get_sorted_lattice();

        let mut ind = 0;
        for level in (1..global_depth).rev()  {
            let mut cnt_in = 0;
            let mut cnt_out = 0;
            for (i, image_pos) in sorted_lattice[level as usize].iter().enumerate() {
                let parent_pos = &wavelet_image.global_position_map[level as usize][&image_pos];
                let fractal = &wavelet_image.fractal_lattice.get(parent_pos).unwrap();
                let haar_tree_pos = fractal.position_map[level as usize].get(&image_pos).unwrap();
                    cnt_in += 1;
                    if let Some(value) = fractal.coefficients[channel][*haar_tree_pos] {
                        let vals = Self::get_neighbour_values(
                            *haar_tree_pos,
                            level,
                            parent_pos,
                            &wavelet_image.fractal_lattice,
                            channel,
                        );
                        if level == global_depth-1 {
                            value_vectors[0][i] = value as f32;
                            for j in 0..6 {
                                matrices[0][(i, j)] = vals[j] as f32;
                            }
                        }
                        else if level == global_depth-2 {
                            value_vectors[1][i] = value as f32;
                            for j in 0..6 {
                                matrices[1][(i, j)] = vals[j] as f32;
                            }
                        }
                        else {
                            value_vectors[2][ind] = value as f32;
                            for j in 0..6 {
                                matrices[2][(ind, j)] = vals[j] as f32;
                            }
                        }
                    }
                if level < global_depth - 2 {
                    ind += 1;
                }
            }
        }

        (value_vectors, matrices)
    }

    fn optimize_width_prediction(
        &mut self,
        neighbourhood_matrices: &Vec<DMatrix<f32>>,
        residuals: &Vec<DVector<f32>>,
        global_depth: u8,
        channel: usize,
    ) {
        let mut width_predictors: Vec<[f32; 2]> = vec![];
        for (matrix, residual_vector) in neighbourhood_matrices.iter().zip(residuals.iter()) {
            let mut width_compounds = DMatrix::<f32>::zeros(matrix.nrows(), 2);
            for (i, row) in matrix.row_iter().enumerate() {
                let value = (row[0] - row[2]).abs();
                width_compounds[(i, 0)] = 1.0;
                width_compounds[(i, 1)] = value;
            }

            let least_squares_result = lstsq(&width_compounds, residual_vector, 1e-14).unwrap();
            width_predictors.push(least_squares_result.solution.fixed_rows::<2>(0).into());
        }

        self.width_predictors[channel] = width_predictors;
    }

    fn optimize_value_prediction(
        &mut self,
        neighbourhood_matrices: &Vec<DMatrix<f32>>,
        values: &Vec<DVector<f32>>,
        global_depth: u8,
        channel: usize,
    ) -> Vec<DVector<f32>> {
        let results = neighbourhood_matrices
            .iter()
            .zip(values.iter())
            .map(|(matrix, vector)| lstsq(matrix, vector, 1e-14).unwrap())
            .collect::<Vec<_>>();

        self.value_predictors[channel] = results
            .iter()
            .map(|result| result.solution.fixed_rows::<6>(0).into())
            .collect();

        neighbourhood_matrices
            .iter()
            .zip(values.iter())
            .zip(results.iter())
            .map(|((matrix, value_vector), result)| {
                let prediction = matrix * &result.solution;
                (value_vector - prediction).abs()
            })
            .collect()
    }

    pub fn optimize_parameters(&mut self, wavelet_image: &WaveletImage, channel: usize) {
        let global_depth = wavelet_image.fractal_lattice.values().nth(0).unwrap().depth;

        let (values, matrices) =
            Self::get_image_neighbour_matrices(&wavelet_image, global_depth, channel);

        let residuals = self.optimize_value_prediction(&matrices, &values, global_depth, channel);

        self.optimize_width_prediction(&matrices, &residuals, global_depth, channel);
    }
}

#[cfg(test)]
mod test {
    use crate::images::ImageMetadata;

    use super::*;

    #[test]
    fn unit_test() {
        let wavelet_image = WaveletImage::from_metadata(ImageMetadata::new(10, 10));
        let mut ctx_modeler = ContextModeler::new();
        ctx_modeler.optimize_parameters(&wavelet_image, 0);
    }
}
