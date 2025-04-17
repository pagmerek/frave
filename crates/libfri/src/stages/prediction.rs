use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use num::pow::Pow;
use num::Complex;

use crate::encoder::EncoderOpts;
use crate::stages::entropy_coding::AnsContext;
use crate::stages::wavelet_transform::{Fractal, WaveletImage};
use crate::context_modeling::{ContextModeler};
use crate::{fractal, utils};

pub const CONTEXT_AMOUNT: usize = 1;

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

fn emit_mse(mse: &Vec<i32>, ctx_channel: usize) {
    std::fs::create_dir_all("./mse").unwrap();
    let mut f = File::create(format!("mse/errors_{}.mse", ctx_channel))
        .expect("Unable to create coef file");
    for i in mse {
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
    position: usize,
    current_depth: u8,
    parent_fractal_pos: &Complex<i32>,
    fractal_lattice: &HashMap<Complex<i32>, Fractal>,
    channel: usize,
) -> (usize, i32) {
    let fractal = &fractal_lattice[parent_fractal_pos];
    let position_in_image = fractal.image_positions[position];
    let neighbours = vec![
        Fractal::get_left(position_in_image, fractal.depth - current_depth),
        Fractal::get_up_left(position_in_image, fractal.depth - current_depth),
        Fractal::get_up_right(position_in_image, fractal.depth - current_depth),
    ];
    let level: usize = current_depth as usize;

    let values: Vec<i32> = neighbours
        .iter()
        .map(|pos| {
            if fractal.position_map[level].get(pos).is_none() {
                if let Some(nposition) =
                    get_containing_fractal(pos, level, fractal, fractal_lattice)
                {
                    let containing_fractal = &fractal_lattice[&nposition];
                    containing_fractal.coefficients[channel][position].unwrap_or(0)
                } else {
                    0
                }
            } else {
                let loc = fractal.position_map[level][pos];
                fractal.coefficients[channel][loc].unwrap_or(0)
            }
        })
        .collect();

    let difference: u32 = (values[0] - values[2]).abs() as u32;

    //let bucket = match difference {
    //    0..3 => 0,
    //    3..6 => 1,
    //    6..10 => 2,
    //    10..20 => 3,
    //    20..40 => 4,
    //    40..60 => 5,
    //    60..90 => 6,
    //    90.. => 7,
    //};
    let bucket = 0;

    let prediction = if values[1] >= max(values[0], values[2]) {
        max(values[0], values[2])
    } else if values[1] <= min(values[0], values[2]) {
        min(values[0], values[2])
    } else {
        values[0] + values[2] - values[1]
    };
    //let prediction = 0;

    (bucket, prediction as i32)
}

pub fn get_hf_context_bucket(
    position: usize,
    current_depth: u8,
    parent_fractal_pos: &Complex<i32>,
    fractal_lattice: &HashMap<Complex<i32>, Fractal>,
    value_prediction_params: &[f32; 6],
    channel: usize,
) -> (usize, i32) {
    assert!(current_depth > 0);
    let parent_level = current_depth as usize - 1;
    let values = ContextModeler::get_neighbour_values(position, current_depth, parent_fractal_pos, fractal_lattice, channel);
    let difference: u32 = (values[0] - values[3]).abs() as u32;
    let difference_horiz1: u32 = (values[1] - values[4]).abs() as u32;
    let difference_horiz2: u32 = (values[2] - values[5]).abs() as u32;

    //let bucket = match difference {
    //    0..3 => 0,
    //    3..6 => 1,
    //    6..10 => 2,
    //    10..20 => 3,
    //    20..40 => 4,
    //    40..60 => 5,
    //    60..90 => 6,
    //    90.. => 7,
    //};
    let bucket = 0;

    let prediction = (values[0] as f32) * value_prediction_params[0]
        + (values[1] as f32) * value_prediction_params[1]
        + (values[2] as f32) * value_prediction_params[2]
        + (values[3] as f32) * value_prediction_params[3]
        + (values[4] as f32) * value_prediction_params[4]
        + (values[5] as f32) * value_prediction_params[5];

    (bucket as usize, prediction as i32)
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

pub fn encode(
    wavelet_image: &mut WaveletImage,
    encoder_opts: &mut EncoderOpts,
) -> Result<[Vec<AnsContext>; 3], String> {
    let mut contexts: [Vec<AnsContext>; 3] = [vec![], vec![], vec![]];
    for channel in 0..wavelet_image.metadata.colorspace.num_channels() {
        let ctx_mod = ContextModeler::new();
        let prediction_params = ctx_mod.optimize_parameters(&wavelet_image, channel);
        encoder_opts.value_prediction_params[channel] = prediction_params;

        contexts[channel] = vec![AnsContext::new(); CONTEXT_AMOUNT];
        let sorted_keys: Vec<Complex<i32>> = wavelet_image.get_sorted_lattice();
        let mut mse: Vec<i32> = vec![];

        for key in sorted_keys.iter() {
            let fractal = &wavelet_image.fractal_lattice[key];
            if let Some(value) = fractal.coefficients[channel][0] {
                let (bucket, prediction) =
                    get_lf_context_bucket(0, 0, key, &wavelet_image.fractal_lattice, channel);
                contexts[channel][bucket].bump_freq(utils::pack_signed(value - prediction));
            }
        }

        // Second scan -> High frequency coefficient root
        for key in sorted_keys.iter() {
            let fractal = &wavelet_image.fractal_lattice[key];
            if let Some(value) = fractal.coefficients[channel][1] {
                let (bucket, prediction) =
                    get_lf_context_bucket(1, 0, key, &wavelet_image.fractal_lattice, channel);
                contexts[channel][bucket].bump_freq(utils::pack_signed(value - prediction));
            }
        }

        for level in 1..wavelet_image.fractal_lattice[&sorted_keys[0]].depth {
            for key in sorted_keys.iter() {
                let fractal = &wavelet_image.fractal_lattice[key];
                for pos in 1 << level..1 << (level + 1) {
                    if let Some(value) = fractal.coefficients[channel][pos] {
                        let (bucket, prediction) = get_hf_context_bucket(
                            pos,
                            level,
                            key,
                            &wavelet_image.fractal_lattice,
                            &encoder_opts.value_prediction_params[channel],
                            channel,
                        );
                        mse.push((value - prediction).pow(2));
                        contexts[channel][bucket].bump_freq(utils::pack_signed(value - prediction));
                    }
                }
            }
        }

        emit_mse(&mse, channel);

        for (i, ctx) in contexts[channel].iter_mut().enumerate() {
            ctx.finalize_context(true);
            if encoder_opts.verbose {
                println!(
                    "CHANNEL: {}, size: {}, entropy: {}",
                    channel,
                    ctx.freqs.iter().sum::<u32>() as usize,
                    get_entropy(&ctx.freqs, ctx.freqs.iter().sum::<u32>() as usize)
                );
            }

            if encoder_opts.emit_coefficients {
                emit_coefficients(&ctx.freqs, i, channel)
            }
        }
    }

    Ok(contexts)
}
