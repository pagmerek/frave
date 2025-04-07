use crate::{stages::wavelet_transform::WaveletImage, utils};

fn get_quantization_matrix() -> [i32; 32] {
    return [1; 32];
}

pub fn encode(mut image: WaveletImage) -> Result<WaveletImage, String> {
    let quantization_matrix = get_quantization_matrix();
    for (_, fractal) in &mut image.fractal_lattice {
        for (channel, channel_coef) in fractal.coefficients.iter_mut().enumerate() {
            for (i, coef_opt) in channel_coef.iter_mut().enumerate() {
                if let Some(coef) = coef_opt {
                    let layer = utils::get_prev_power_two(i + 1).trailing_zeros();
                        //if layer > 5 {
                        //    *coef = 0;
                        //}
                        *coef /=  quantization_matrix[layer as usize];
                            //(layer as i32)/5 + 1;
                }
            }
        }
    }

    Ok(image)
}

pub fn decode(mut image: WaveletImage) -> Result<WaveletImage, String> {
    let quantization_matrix = get_quantization_matrix();
    for (_, fractal) in &mut image.fractal_lattice {
        for (channel, channel_coef) in fractal.coefficients.iter_mut().enumerate() {
            for (i, coef_opt) in channel_coef.iter_mut().enumerate() {
                if let Some(coef) = coef_opt {
                    let layer = utils::get_prev_power_two(i + 1).trailing_zeros();
                        //if layer > 5 {
                        //    *coef = 0;
                        //}
                        *coef /=  quantization_matrix[layer as usize];
                            //(layer as i32)/5 + 1;
                }
            }
        }
    }

    Ok(image)
}
