use bmp::Image;

pub fn mirrors(img: Image, frame: u32) -> Image {
    let (w, h) = (img.get_width(), img.get_height());
    let mut lattice = Image::new(w + frame * 2, h + frame * 2);
    let mirrored_rows = (0..=(frame - 1))
        .rev()
        .chain(0..=(w - 1))
        .chain(((h - frame)..=(w - 1)).rev());
    for (row, mirrored_row) in mirrored_rows.enumerate() {
        for x in 0..(w + frame * 2) {
            if x < frame {
                lattice.set_pixel(x, row as u32, img.get_pixel(frame - 1 - x, mirrored_row))
            } else if x >= frame && x < w + frame {
                lattice.set_pixel(x, row as u32, img.get_pixel(x - frame, mirrored_row))
            } else {
                lattice.set_pixel(
                    x,
                    row as u32,
                    img.get_pixel(2 * w + frame - 1 - x, mirrored_row),
                )
            }
        }
    }
    lattice
}

pub fn trim(framed_img: Image, width: u32, height: u32, frame: u32) -> Image {
    let mut image = Image::new(width, height);
    for (x, y) in image.coordinates() {
        image.set_pixel(x, y, framed_img.get_pixel(x + frame, y + frame))
    }
    image
}
