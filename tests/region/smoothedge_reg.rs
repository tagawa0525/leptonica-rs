//! Smoothed-edge regression test
//!
//! Uses gradient and watershed behavior on an image with a sharp transition.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/smoothedge_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::region::{WatershedOptions, compute_gradient, watershed_segmentation};
use leptonica::{Pix, PixelDepth};

fn make_step_image(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit8).expect("create image");
    let mut pm = pix.try_into_mut().expect("mutable image");
    for y in 0..h {
        for x in 0..w {
            let val = if x < w / 2 { 0 } else { 255 };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

#[test]
fn smoothedge_reg() {
    let mut rp = RegParams::new("smoothedge");

    let pix = make_step_image(48, 24);
    let grad = compute_gradient(&pix).expect("compute_gradient");
    rp.compare_values(48.0, grad.width() as f64, 0.0);
    rp.compare_values(24.0, grad.height() as f64, 0.0);

    let edge_val = grad.get_pixel_unchecked(24, 12);
    let flat_val = grad.get_pixel_unchecked(4, 12);
    rp.compare_values(1.0, if edge_val > flat_val { 1.0 } else { 0.0 }, 0.0);
    rp.write_pix_and_check(&grad, ImageFormat::Png)
        .expect("write grad smoothedge");

    let seg = watershed_segmentation(&pix, &WatershedOptions::default()).expect("watershed");
    rp.compare_values(pix.width() as f64, seg.width() as f64, 0.0);
    rp.compare_values(pix.height() as f64, seg.height() as f64, 0.0);
    rp.write_pix_and_check(&seg, ImageFormat::Png)
        .expect("write seg smoothedge");

    assert!(rp.cleanup(), "smoothedge regression test failed");
}
