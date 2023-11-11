use std::fs::File;
use std::path::PathBuf;

use crate::compression::decoder::Decoder;
use crate::compression::encoder::Encoder;
use crate::fractal::image::{get_quantization_matrix, FractalImage, Frv};
use crate::fractal::variants::Variant;

pub fn encode(path: PathBuf, var: Variant, output: String) {
    let img = image::open(path).unwrap_or_else(|e| {
        panic!("Failed to open: {e}");
    });

    let mut enc = FractalImage::new_from_img(img.into_luma8(), var);
    enc.find_coef();
    enc.quantizate(&get_quantization_matrix());
    let (coef, contexts) = enc.ans_encode();
    let compressed = Frv::new(enc.height, enc.width, contexts, var, coef);
    let mut f = File::create(output).unwrap_or_else(|e| panic!("Failed to encode frv image: {e}"));
    bincode::serialize_into(&mut f, &compressed).unwrap();
}

pub fn decode(path: PathBuf, output: String) {
    let img = File::open(path).unwrap_or_else(|e| {
        panic!("Failed to open: {e}");
    });
    let frv_img: Frv = bincode::deserialize_from(img).unwrap();
    let mut dec = FractalImage::new_from_frv(&frv_img);
    dec.ans_decode(frv_img.data, frv_img.ans_contexts);
    dec.unquantizate(&get_quantization_matrix());
    dec.find_val();
    dec.image.save(output).unwrap_or_else(|e| {
        panic!("Failed to decode frv image: {e}");
    });
}
