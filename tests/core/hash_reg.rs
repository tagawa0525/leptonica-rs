//! Hash regression test
//!
//! Covers hash-based color counting and hash-line rendering.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/hash_reg.c`

use crate::common::RegParams;
use leptonica::core::pix::HashOrientation;
use leptonica::core::pixel;
use leptonica::{Box, Pix, PixelDepth, PixelOp};

#[test]
fn hash_reg() {
    let mut rp = RegParams::new("hash");

    // Hash-based color counting
    let pix = Pix::new(16, 12, PixelDepth::Bit32).expect("create 32bpp");
    let mut pm = pix.to_mut();
    pm.set_pixel(0, 0, pixel::compose_rgb(255, 0, 0))
        .expect("set red");
    pm.set_pixel(1, 0, pixel::compose_rgb(0, 255, 0))
        .expect("set green");
    let pix: Pix = pm.into();

    // default black + red + green
    let ncolors = pix.count_rgb_colors_by_hash().expect("count by hash");
    rp.compare_values(3.0, ncolors as f64, 0.0);

    // Hash-line rendering
    let dest = Pix::new(40, 20, PixelDepth::Bit1).expect("create 1bpp");
    let mut dm = dest.to_mut();
    let b = Box::new(5, 3, 24, 12).expect("box");
    dm.render_hash_box(&b, 4, 1, HashOrientation::Horizontal, false, PixelOp::Set)
        .expect("render_hash_box");
    let rendered: Pix = dm.into();

    rp.compare_values(rendered.width() as f64, 40.0, 0.0);
    let on = rendered.count_pixels();
    rp.compare_values(1.0, if on > 0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "hash regression test failed");
}
