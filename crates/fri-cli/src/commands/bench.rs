use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read};


use libfri::encoder::{EncoderOpts, FRIEncoder};
use libfri::decoder::FRIDecoder;

#[derive(clap::Args)]
pub struct BenchCommand {
    pub dataset_path: PathBuf,

}

pub fn benchmark(cmd: BenchCommand) {
    let paths = fs::read_dir(cmd.dataset_path).expect(&format!("No such directory"));
    fs::create_dir_all("./output").unwrap(); 
    let mut compression_rates: Vec<f32> = vec![];
    for path in paths {
        let mut img_path = path.unwrap().path();
        let original_path = img_path.clone();
        let img = match image::open(&img_path) {
            Ok(data) => data,
            Err(_) => continue,
        };

        println!("COMPRESSION {}", img_path.file_name().unwrap().to_str().unwrap());
        println!("======================================");
        println!("PNG size: {}", fs::metadata(&img_path).unwrap().len());
        let encoder = FRIEncoder::new(EncoderOpts::default());

        let height = img.height();
        let width = img.width();
        let color = img.color();
        let data = img.into_bytes();

        let frifcolor = match color {
            image::ColorType::L8 => libfri::images::ColorSpace::Luma,
            image::ColorType::Rgb8 => libfri::images::ColorSpace::RGB,
            _ => panic!("Unsupported color scheme for frif image, expected rgb8 or luma8"),
        };
        let uncompressed_size = data.len();

        let result = encoder.encode(data, height, width, frifcolor).unwrap_or_else(|e| panic!("Cannot encode {}, reason: {}", img_path.file_name().unwrap().to_str().unwrap(), e));

        let compression_rate = (uncompressed_size as f32 - result.len() as f32)/uncompressed_size as f32 * 100.;
        println!("FILE {}", img_path.file_name().unwrap().to_str().unwrap());
        println!("Before compression size: {}", uncompressed_size);
        println!("After compression size: {}", result.len());
        println!("Compression rate: {}%", compression_rate);
        img_path.set_extension("frif");
        compression_rates.push(compression_rate);

        if false {
            fs::write(&img_path, &result).unwrap_or_else(|e| panic!("Failed to encode frv image: {e}"));
        }

        let decoder = FRIDecoder{};

        match decoder.decode(result) {
            Ok(decoded) => {
               let img: image::RgbImage = match image::ImageBuffer::from_vec(decoded.metadata.width as u32, decoded.metadata.height as u32, decoded.data) {
                    Some(buf) => buf,
                    None => {
                        eprintln!("Failed to create image buffer.");
                        return;
                    }
                };

                let mut output_path: PathBuf = PathBuf::from(r"./output/");
                output_path.push(img_path.file_name().unwrap());
                output_path.set_extension("bmp");
                let file = File::create(&output_path).unwrap();
                let ref mut w = BufWriter::new(file);

                img.write_to(w, image::ImageOutputFormat::Bmp).expect("Failed to write image");

                let original_img = match image::open(&original_path) {
                    Ok(data) => data.into_bytes(),
                    Err(_) => continue,
                };
                let decoded_img = img.bytes();
                let len = original_img.len() as f32;

                let mut mse: u32 = 0;
                for (x, y) in decoded_img.into_iter().zip(original_img.into_iter()) {
                    mse += (x.unwrap() - y).pow(2) as u32;
                }
                println!("MSE: {}", mse as f32/len);
            }
            Err(msg) => println!("Cannot decode, reason: {msg}"),
        }
        println!();
    }

    let avg_compression_rate = compression_rates.iter().sum::<f32>() / compression_rates.len() as f32;
    println!("====SUMMARY====");
    println!("AVG compression rate: {}%", avg_compression_rate);

}

