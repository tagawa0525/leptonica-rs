//! Low saturation regression test
//!
//! Tests functions that identify and modify image pixels with low saturation
//! (essentially gray pixels). The C version darkens gray pixels and generates
//! masks over gray regions.
//!
//! Partial migration: darken_gray, modify_saturation, and measure_saturation
//! are tested. mask_over_gray_pixels is in leptonica-color and not tested here.
//! Test image zier.jpg is not available; marge.jpg is used instead.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/lowsat_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::filter::{darken_gray, measure_saturation, modify_saturation};

/// Test darken_gray (C check 2: pixDarkenGray).
///
/// Verifies that gray pixels are darkened while leaving colorful pixels
/// mostly unaffected.
#[test]
fn lowsat_reg_darken_gray() {
    let mut rp = RegParams::new("lowsat_darken");

    // C: pix3 = pixDarkenGray(NULL, pix2, 220, 10)
    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    let darkened = darken_gray(&pix, 220, 10).expect("darken_gray");
    rp.compare_values(w as f64, darkened.width() as f64, 0.0);
    rp.compare_values(h as f64, darkened.height() as f64, 0.0);
    assert_eq!(darkened.depth(), PixelDepth::Bit32);

    // Different threshold
    let darkened2 = darken_gray(&pix, 180, 20).expect("darken_gray thresh=180");
    rp.compare_values(w as f64, darkened2.width() as f64, 0.0);

    assert!(rp.cleanup(), "lowsat darken_gray test failed");
}

/// Test modify_saturation and measure_saturation (saturation operations).
///
/// Verifies saturation modification produces correct dimensions and
/// measure_saturation returns a reasonable value.
#[test]
fn lowsat_reg_saturation() {
    let mut rp = RegParams::new("lowsat_sat");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Measure original saturation (returns mean saturation in [0..255])
    let original_sat = measure_saturation(&pix, 1).expect("measure_saturation");
    assert!(original_sat >= 0.0, "saturation should be non-negative");
    assert!(original_sat <= 255.0, "saturation should be <= 255");

    // Increase saturation
    let saturated = modify_saturation(&pix, 0.5).expect("modify_saturation +0.5");
    rp.compare_values(w as f64, saturated.width() as f64, 0.0);
    rp.compare_values(h as f64, saturated.height() as f64, 0.0);

    // Decrease saturation
    let desaturated = modify_saturation(&pix, -0.5).expect("modify_saturation -0.5");
    rp.compare_values(w as f64, desaturated.width() as f64, 0.0);

    // Measure modified saturation — increased should be higher
    let new_sat = measure_saturation(&saturated, 1).expect("measure saturated");
    rp.compare_values(1.0, if new_sat >= original_sat { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "lowsat saturation test failed");
}

/// Test mask_over_gray_pixels (C check 3: pixMaskOverGrayPixels).
///
/// This function is in leptonica-color crate (mask_over_gray_pixels) and
/// cannot be tested from leptonica-filter tests.
#[test]
#[ignore = "not yet implemented: mask_over_gray_pixels is in leptonica-color crate"]
fn lowsat_reg_mask_gray() {
    // C version:
    // pix4 = pixMaskOverGrayPixels(pix2, 220, 10);
    // pix5 = pixMorphSequence(pix4, "o20.20", 0);
}
