use crate::frave::Frave;
use crate::utils::trimmer;
use crate::variants::*;
use bmp;

mod coord;
mod frave;
mod utils;
mod variants;

fn main() {
    let img = bmp::open("img/lena.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });
    let lattice = trimmer::mirrors(img, 256);
    let mut fr: Frave = Frave::new(lattice, TWINDRAGON);
    //dbg!(fr.depth);
    fr.find_coef();
    fr.quantizate();
    fr.unquantizate();
    //fr.trim_coef(16);
    fr.find_val();

    trimmer::trim(fr.image, 512, 512, 256).save("result.bmp");
}
