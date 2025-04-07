use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use num::Complex;

use crate::stages::entropy_coding::AnsContext;
use crate::stages::wavelet_transform::{Fractal, WaveletImage};
use crate::utils;

fn emit_coefficients(data: &[u32], ctx_id: usize, ctx_channel: usize) {
    std::fs::create_dir_all("./coefficients").unwrap();
    let mut f = File::create(format!(
        "coefficients/{}_context_{}.coef",
        ctx_channel, ctx_id
    ))
    .expect("Unable to create coef file");
    for i in data {
        write!(f, "{}\n", i).unwrap();
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

pub fn get_lf_context_bucket(
    current_depth: u8,
    parent_fractal_pos: &Complex<i32>,
    fractal_lattice: &HashMap<Complex<i32>, Fractal>,
    channel: usize,
) -> (usize, i32) {
    let fractal = &fractal_lattice[parent_fractal_pos];
    let position_in_image = fractal.image_positions[0];
    let neighbours = vec![
        Fractal::get_left(position_in_image, current_depth),
        Fractal::get_up_left(position_in_image, current_depth),
        Fractal::get_up_right(position_in_image, current_depth),
    ];
    let level: usize = current_depth as usize - 1;

    let values: Vec<i32> = neighbours
        .iter()
        .map(|pos| {
            if fractal.position_map[level].get(pos).is_none() {
                if let Some(position) = get_containing_fractal(pos, level, fractal, fractal_lattice)
                {
                    let left_fractal = &fractal_lattice[&position];
                    let l = left_fractal.position_map[level][pos];
                    left_fractal.coefficients[channel][l].unwrap_or(0)
                } else {
                    fractal.coefficients[channel][0].unwrap_or(0)
                }
            } else {
                0
            }
        })
        .collect();

    let difference: u32 = (values[0] - values[2]).abs() as u32;

    let bucket = match difference {
        0 => 0,
        1..=10 => 1,
        11.. => 2,
    };

    let prediction = values[0] as f32 * 0.5 + values[1] as f32 * 0.5 - values[2] as f32 * 0.5;

    (bucket, prediction as i32)
}

pub fn get_hf_context_bucket(
    position: usize,
    current_depth: u8,
    parent_fractal_pos: &Complex<i32>,
    fractal_lattice: &HashMap<Complex<i32>, Fractal>,
    channel: usize,
) -> (usize, i32) {
    //let position_in_image = fractal.image_positions[position];
    
    let fractal = &fractal_lattice[parent_fractal_pos];
    let parent_position_in_image = fractal.image_positions[position/2];
    let neighbours = vec![
        Fractal::get_left(parent_position_in_image, current_depth + 1),
        Fractal::get_up_left(parent_position_in_image, current_depth + 1),
        Fractal::get_up_right(parent_position_in_image, current_depth + 1),
        Fractal::get_right(parent_position_in_image, current_depth + 1),
        Fractal::get_down_left(parent_position_in_image, current_depth + 1),
        Fractal::get_down_right(parent_position_in_image, current_depth + 1),
    ];
    let level: usize = current_depth as usize;

    let values: Vec<i32> = neighbours
        .iter()
        .map(|pos| {
            if fractal.position_map[level].get(pos).is_none() {
                if let Some(position) = get_containing_fractal(pos, level, &fractal, fractal_lattice)
                {
                    let left_fractal = &fractal_lattice[&position];
                    let l = left_fractal.position_map[level][pos];
                    left_fractal.coefficients[channel][l].unwrap_or(0)
                } else {
                    fractal.coefficients[channel][0].unwrap_or(0)
                }
            } else {
                0
            }
        })
        .collect();

    let difference: u32 = (values[0] - values[3]).abs() as u32;

    let bucket = match difference {
        0 => 0,
        1..=10 => 1,
        11.. => 2,
    };

    let prediction = values[0] as f32 * 0.5 + values[1] as f32 * 0.5 - values[2] as f32 * 0.5;

    (bucket, prediction as i32)
}

fn get_entropy(histogram: &[u32], total_size: usize) -> f32 {
    let mut entropy = 0f32;
    for val in histogram {
        let symbol_prob = *val as f32 / total_size as f32;
        if symbol_prob >= f32::EPSILON {
            entropy += symbol_prob * symbol_prob.log2();
        }
    }
    -entropy
}

pub fn encode(wavelet_image: &mut WaveletImage) -> Result<[Vec<AnsContext>; 3], String> {
    let mut contexts: [Vec<AnsContext>; 3] = [vec![], vec![], vec![]];
    for channel in 0..wavelet_image.metadata.colorspace.num_channels() {
        contexts[channel] = vec![AnsContext::new(); 3];
        let mut sorted_keys: Vec<Complex<i32>> =
            wavelet_image.fractal_lattice.keys().cloned().collect();
        sorted_keys.sort_by(utils::order_complex);
        //let flatten = fractal.coefficients[channel].iter().flatten().map(|x| pack_signed(*x)).collect();
        //let new_freqs = AnsContext::get_freqs(&flatten);
        //ans_contexts[0].update_freqs(new_freqs);
        for level in 0..wavelet_image.fractal_lattice[&sorted_keys[0]].depth {
            for key in sorted_keys.iter() {
                let fractal = &wavelet_image.fractal_lattice[key];
                if level == 0 {
                    if let Some(value) = fractal.coefficients[channel][0] {
                        let (bucket, prediction) = get_lf_context_bucket(
                            fractal.depth - level,
                            key,
                            &wavelet_image.fractal_lattice,
                            channel,
                        );
                        contexts[channel][bucket].bump_freq(utils::pack_signed(value - prediction));
                    }
                } else {
                    for pos in 1 << level..1 << (level + 1) {
                        if let Some(value) = fractal.coefficients[channel][pos] {
                            let (bucket, prediction) = get_hf_context_bucket(
                                pos,
                                fractal.depth - level,
                                key,
                                &wavelet_image.fractal_lattice,
                                channel,
                            );
                            contexts[channel][bucket]
                                .bump_freq(utils::pack_signed(value - prediction));
                        }
                    }
                }
            }
        }

        for (i, ctx) in contexts[channel].iter_mut().enumerate() {
            let max_freq_bits =
                utils::get_prev_power_two(ctx.freqs.iter().sum::<u32>() as usize).trailing_zeros();
            ctx.normalize_freqs(1 << max_freq_bits);
            dbg!(ctx.freqs.iter().sum::<u32>() as usize, channel);
            println!(
                "CHANNEL: {}, entropy: {}",
                channel,
                get_entropy(&ctx.freqs, ctx.freqs.iter().sum::<u32>() as usize)
            );

            emit_coefficients(&ctx.freqs, i, channel)
        }
    }

    Ok(contexts)
}
