//! Black/white border regression test
//!
//! Tests functions that handle black and white pixel values in images:
//! get_black_or_white_val, add_border_general, and alpha_blend_uniform.
//! The C version iterates over 11 images of varying depth, scales them,
//! and adds white or black borders.
//!
//! Full migration: get_black_or_white_val, add_border_general, and
//! alpha_blend_uniform are tested with all 11 C test images.
//!
//! # See also
//!
//! C Leptonica: `prog/blackwhite_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::{InitColor, PixMut, PixelDepth};

/// Test add_border_general with white border (C check 0: white boundary loop).
///
/// Verifies adding a white border to images of various depths.
#[test]
fn blackwhite_reg_white_border() {
    let mut rp = RegParams::new("bw_white");

    // Test with various depth images
    let images = ["marge.jpg", "test8.jpg", "dreyfus8.png"];

    for (i, name) in images.iter().enumerate() {
        let pix = crate::common::load_test_image(name).expect(name);
        let wval = PixMut::get_black_or_white_val(&pix, InitColor::White);

        let bordered = pix
            .add_border_general(30, 30, 20, 20, wval)
            .expect("add white border");
        rp.compare_values((pix.width() + 60) as f64, bordered.width() as f64, 0.0);
        rp.compare_values((pix.height() + 40) as f64, bordered.height() as f64, 0.0);
        if i == 0 {
            rp.write_pix_and_check(&bordered, ImageFormat::Png)
                .expect("write bordered white_border");
        }
    }

    assert!(rp.cleanup(), "blackwhite white border test failed");
}

/// Test add_border_general with black border (C check 1: black boundary loop).
///
/// Verifies adding a black border to images of various depths.
#[test]
fn blackwhite_reg_black_border() {
    let mut rp = RegParams::new("bw_black");

    let images = ["marge.jpg", "test8.jpg", "dreyfus8.png"];

    for (i, name) in images.iter().enumerate() {
        let pix = crate::common::load_test_image(name).expect(name);
        let bval = PixMut::get_black_or_white_val(&pix, InitColor::Black);

        let bordered = pix
            .add_border_general(30, 30, 20, 20, bval)
            .expect("add black border");
        rp.compare_values((pix.width() + 60) as f64, bordered.width() as f64, 0.0);
        rp.compare_values((pix.height() + 40) as f64, bordered.height() as f64, 0.0);
        if i == 0 {
            rp.write_pix_and_check(&bordered, ImageFormat::Png)
                .expect("write bordered black_border");
        }
    }

    assert!(rp.cleanup(), "blackwhite black border test failed");
}

/// Test alpha_blend_uniform + add_border_general on alpha images (C loop spp==4).
///
/// Verifies removing alpha channel then adding a border.
#[test]
fn blackwhite_reg_alpha_blend() {
    let mut rp = RegParams::new("bw_alpha");

    // test-gray-alpha.png has alpha channel (spp == 4)
    let pix =
        crate::common::load_test_image("test-gray-alpha.png").expect("load test-gray-alpha.png");
    let wval = PixMut::get_black_or_white_val(&pix, InitColor::White);

    // C: pixAlphaBlendUniform(pixs, wval) — remove alpha over white
    let blended = pix.alpha_blend_uniform(wval).expect("alpha_blend_uniform");
    assert_eq!(blended.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&blended, ImageFormat::Png)
        .expect("write blended alpha_blend");

    let bordered = blended
        .add_border_general(30, 30, 20, 20, wval)
        .expect("add border after alpha blend");
    rp.compare_values((pix.width() + 60) as f64, bordered.width() as f64, 0.0);
    rp.compare_values((pix.height() + 40) as f64, bordered.height() as f64, 0.0);

    assert!(rp.cleanup(), "blackwhite alpha blend test failed");
}

/// Test full image set from C (C checks 0-1 with all 11 images).
///
/// Iterates all 11 C test images, scales to 150×150, adds white/black borders.
/// For spp==4 images, applies alpha_blend_uniform before adding border.
#[test]
fn blackwhite_reg_full_image_set() {
    let mut rp = RegParams::new("bw_full");

    let images = [
        "test1.png",
        "speckle2.png",
        "weasel2.4g.png",
        "speckle4.png",
        "weasel4.11c.png",
        "dreyfus8.png",
        "weasel8.240c.png",
        "test16.tif",
        "marge.jpg",
        "test-cmap-alpha.png",
        "test-gray-alpha.png",
    ];

    // White border pass (C check 0)
    for name in &images {
        let pix = crate::common::load_test_image(name).expect(name);
        let wval = PixMut::get_black_or_white_val(&pix, InitColor::White);
        let pix1 = if pix.spp() == 4 {
            pix.alpha_blend_uniform(wval).expect("alpha_blend_uniform")
        } else {
            pix.deep_clone()
        };
        let scaled = leptonica::transform::scale_to_size(&pix1, 150, 150).expect("scale_to_size");
        let wval2 = PixMut::get_black_or_white_val(&scaled, InitColor::White);
        let bordered = scaled
            .add_border_general(30, 30, 20, 20, wval2)
            .expect("add white border");
        rp.compare_values(210.0, bordered.width() as f64, 0.0);
        rp.compare_values(190.0, bordered.height() as f64, 0.0);
    }

    // Black border pass (C check 1)
    for name in &images {
        let pix = crate::common::load_test_image(name).expect(name);
        let wval = PixMut::get_black_or_white_val(&pix, InitColor::White);
        let bval = PixMut::get_black_or_white_val(&pix, InitColor::Black);
        let pix1 = if pix.spp() == 4 {
            pix.alpha_blend_uniform(wval).expect("alpha_blend_uniform")
        } else {
            pix.deep_clone()
        };
        let scaled = leptonica::transform::scale_to_size(&pix1, 150, 150).expect("scale_to_size");
        let bordered = scaled
            .add_border_general(30, 30, 20, 20, bval)
            .expect("add black border");
        rp.compare_values(210.0, bordered.width() as f64, 0.0);
        rp.compare_values(190.0, bordered.height() as f64, 0.0);
    }

    assert!(rp.cleanup(), "blackwhite full image set test failed");
}
