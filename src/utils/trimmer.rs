use image::GrayImage;

pub fn mirrors(img: GrayImage, frame: u32) -> GrayImage {
    let (w, h) = (img.width(), img.height());
    let mut lattice = GrayImage::new(w + frame * 2, h + frame * 2);
    let mirrored_rows = (0..=(frame - 1))
        .rev()
        .chain(0..=(w - 1))
        .chain(((h - frame)..=(w - 1)).rev());
    for (row, mirrored_row) in mirrored_rows.enumerate() {
        for x in 0..(w + frame * 2) {
            if x < frame {
                lattice.put_pixel(x, row as u32, *img.get_pixel(frame - 1 - x, mirrored_row))
            } else if x >= frame && x < w + frame {
                lattice.put_pixel(x, row as u32, *img.get_pixel(x - frame, mirrored_row))
            } else {
                lattice.put_pixel(
                    x,
                    row as u32,
                    *img.get_pixel(2 * w + frame - 1 - x, mirrored_row),
                )
            }
        }
    }
    lattice
}


