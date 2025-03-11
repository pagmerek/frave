use std::io::BufRead;
use std::{fs, io::BufReader};
use std::path::PathBuf;
use libfri::encoder::{EncoderOpts, FRIEncoder};

use std::fs::File;
use itertools::Itertools;
use plotters::prelude::*;

#[derive(clap::Args)]
pub struct PlotsCommand {
    pub dataset_path: PathBuf,
}

pub fn plot_coeffs(coef_path: PathBuf, original_img_name: &str) { 
        let file = File::open(&coef_path).expect("file wasn't found.");
        let reader = BufReader::new(file);

        let coefficients: Vec<i32> = reader
            .lines()
            .map(|line| line.unwrap().parse::<i32>().unwrap())
            .collect();

        let coeff_name = &coef_path.file_name().unwrap().to_str().unwrap();
        let plot_name = format!("{}-{}.png", original_img_name, coeff_name);
        let root_area = BitMapBackend::new(&plot_name, (1024, 1024))
            .into_drawing_area();
        root_area.fill(&WHITE).unwrap();

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption(format!("coefficients distribution for {} with layer: {}", original_img_name, coeff_name), ("sans-serif", 40))
            .build_cartesian_2d((-50..50).into_segmented(), 0..50000)
            .unwrap();


        ctx.configure_mesh()
            .disable_x_mesh()
            .bold_line_style(&WHITE.mix(0.3))
            .y_desc("Count")
            .x_desc("Bucket")
            .axis_desc_style(("sans-serif", 15))
            .draw().unwrap();


        ctx.draw_series(
            Histogram::vertical(&ctx).margin(5)
                .data(coefficients.iter().map(|x| (*x, 1)))
        ).unwrap();
}

pub fn plot(cmd: PlotsCommand) {
    let paths = fs::read_dir(cmd.dataset_path).expect(&format!("No such directory"));
    for path in paths {
        let img_path = path.unwrap().path();
        let img = match image::open(&img_path) {
            Ok(data) => data,
            Err(_) => continue,
        };

        println!("COMPRESSION {}", img_path.file_name().unwrap().to_str().unwrap());
        println!("======================================");
        println!("PNG size: {}", fs::metadata(&img_path).unwrap().len());
        let luma_img = img;
        let encoder = FRIEncoder::new(EncoderOpts { emit_coefficients: true, ..Default::default() });

        let height = luma_img.height();
        let width = luma_img.width();
        let color = luma_img.color();
        let data = luma_img.into_bytes();

        let frifcolor = match color {
            image::ColorType::L8 => libfri::images::ColorSpace::Luma,
            image::ColorType::Rgb8 => libfri::images::ColorSpace::RGB,
            _ => panic!("Unsupported color scheme for frif image, expected rgb8 or luma8"),
        };
        let uncompressed_size = data.len();

        let result = encoder.encode(data, height, width, frifcolor).unwrap_or_else(|e| panic!("Cannot encode {}, reason: {}", img_path.file_name().unwrap().to_str().unwrap(), e));

        for coeff in fs::read_dir("./coefficients/").expect("No coefficients emitted") {
            plot_coeffs(coeff.unwrap().path(), img_path.file_name().unwrap().to_str().unwrap())
        }
    }
}
