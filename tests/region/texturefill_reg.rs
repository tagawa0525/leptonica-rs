//! Texture-fill regression test
//!
//! Uses hole filling and closed-border filling operations.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/texturefill_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::region::{
    ConnectivityType, fill_closed_borders, fill_holes_to_bounding_rect, holes_by_filling,
};
use leptonica::{Pix, PixelDepth};

fn make_ring(w: u32, h: u32, x0: u32, y0: u32, x1: u32, y1: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).expect("create ring image");
    let mut pm = pix.try_into_mut().expect("mutable ring image");
    for y in y0..y1 {
        for x in x0..x1 {
            if y == y0 || y + 1 == y1 || x == x0 || x + 1 == x1 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
    }
    pm.into()
}

#[test]
fn texturefill_reg() {
    let mut rp = RegParams::new("texturefill");

    let pix = make_ring(40, 30, 5, 4, 34, 25);
    let holes = holes_by_filling(&pix, ConnectivityType::FourWay).expect("holes_by_filling");
    rp.compare_values(1.0, if holes.count_pixels() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.write_pix_and_check(&holes, ImageFormat::Tiff)
        .expect("write holes texturefill");

    let filled = fill_closed_borders(&pix, ConnectivityType::FourWay).expect("fill_closed_borders");
    rp.compare_values(
        1.0,
        if filled.count_pixels() > pix.count_pixels() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.write_pix_and_check(&filled, ImageFormat::Tiff)
        .expect("write filled texturefill");

    let rect_filled =
        fill_holes_to_bounding_rect(&pix, 20, 0.9, 0.2).expect("fill_holes_to_bounding_rect");
    rp.compare_values(
        1.0,
        if rect_filled.count_pixels() >= pix.count_pixels() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.write_pix_and_check(&rect_filled, ImageFormat::Tiff)
        .expect("write rect_filled texturefill");

    assert!(rp.cleanup(), "texturefill regression test failed");
}
