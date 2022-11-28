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

    let mut fr: Frave = Frave::new(img, TAME_TWINDRAGON);
    //dbg!(fr.center.x, fr.center.y, fr.depth);
    fr.find_coef();
    fr.trim_coef(12);
    fr.find_val();
    fr.image.save("result.bmp");
}
