use crate::encoder::Encoder;
use crate::utils::trimmer::{mirrors, trim};
use crate::variants::{get_variant, Variant};
use bmp;

fn encode(path: std::path::PathBuf, var: Variant) {
    unimplemented!();
}

fn decode(path: std::path::PathBuf, var: Variant) {
    unimplemented!();
}

fn fractalize(path: std::path::PathBuf, amount: usize, var: Variant) {
    let img = bmp::open(path).unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });
    let lattice = mirrors(img, 256);

    let mut enc = Encoder::new(lattice, get_variant(var));
    enc.find_coef();
    enc.trim_coef(amount);
    enc.find_val();

    let _ = trim(enc.image, 512, 512, 256)
        .save("result.bmp")
        .unwrap_or_else(|e| {
            panic!("Failed to save: {}", e);
        });
}
