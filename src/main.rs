use bmp;
use frave::Frave;
use variants::*;

mod coord;
mod frave;
mod variants;

fn main() {
    let img = bmp::open("img/lena.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });

    let fr: Frave = Frave::new(img, TAME_TWINDRAGON);
    println!("{}", fr.depth);
}
