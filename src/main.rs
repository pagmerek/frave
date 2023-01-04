use clap::Parser;

use crate::encoder::Encoder;
use crate::utils::trimmer;
use crate::variants::*;
use bmp;

mod coord;
mod encoder;
mod utils;
mod variants;

#[derive(Parser)]
#[clap(author="P. Gmerek", version, about)]
/// Image compression program based on complex based numeral systems. 
enum Frave {
    Decode(Decode),
    Encode(Encode),
}

#[derive(clap::Args)]
/// Image compression program based on complex based numeral systems. 
struct Decode {
   bmp_path: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
struct Encode { 
   fr_path: Option<std::path::PathBuf>,

}

fn encode() {
    let img = bmp::open("img/lena.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });
    let lattice = trimmer::mirrors(img, 256);

    let mut enc = Encoder::new(lattice, TAME_TWINDRAGON);
    //dbg!(fr.depth);
    enc.find_coef();
    enc.quantizate();

    enc.unquantizate();
    //enc.trim_coef(16);
    enc.find_val();
 
    trimmer::trim(enc.image, 512, 512, 256).save("result.bmp");
}

fn decode() {
}

fn main() {
    let app = Frave::parse(); 
    //let matches = app.get_matches();
}
