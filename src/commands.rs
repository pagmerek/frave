use std::fs::File;
use std::path::PathBuf;

use crate::decoder::Decoder;
use crate::encoder::Encoder;
use crate::frave_image::FraveImage;
use crate::utils::trimmer::{mirrors};
use crate::variants::{get_variant, Variant};

pub fn encode(path: PathBuf, var: Variant, output: String) {
    let img = image::open(path).unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });
    // let lattice = mirrors(img, 256);

    let mut enc = Encoder::new(img.into_luma8(), get_variant(var));
    enc.find_coef();
    enc.quantizate();
    let (compressed, contexts) = enc.ans_encode();
    let frave_image = FraveImage {
        height: enc.height,
        width: enc.width,
        depth: enc.depth,
        center: (enc.center.x, enc.center.y),
        variant: var,
        ans_contexts: contexts,
        compressed_coef: compressed,
    };
    let mut f = File::create(output).unwrap();
    bincode::serialize_into(&mut f, &frave_image).unwrap();
}

pub fn decode(path: PathBuf, output: String) {
    let img = File::open(path).unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });
    let frv_img: FraveImage = bincode::deserialize_from(img).unwrap();
    let mut dec = Decoder::new(frv_img);
    dec.unquantizate();
    dec.find_val();
    let _ = dec.image.save(output).unwrap_or_else(|e| {
        panic!("Failed to save: {}", e);
    });
}

pub fn fractalize(path: PathBuf, amount: usize, var: Variant, output: String) {
    let img = image::open(path).unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });
    //let lattice = mirrors(img.into_luma8(), 256);

    let mut enc = Encoder::new(img.into_luma8(), get_variant(var));
    enc.find_coef();
    enc.trim_coef(amount);
    enc.find_val();

    let _ = enc.image.save(output)
        .unwrap_or_else(|e| {
            panic!("Failed to save: {}", e);
        });
}
