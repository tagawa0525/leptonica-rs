//! Hard light blending regression test
//!
//! Tests the hard light blend mode. The C version tests pixBlendHardLight
//! with various image combinations including colormapped images and in-place
//! blending.
//!
//! Partial migration: blend_hard_light is tested with available images.
//! C test images (hardlight1_*.jpg, hardlight2_*.jpg) are not available;
//! substitute images are used instead.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/hardlight_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;

/// Test blend_hard_light on 32bpp color images (C checks 0-1).
///
/// Verifies hard light blending produces 32bpp output with correct dimensions.
/// Rust requires same dimensions and depth; C allows position offsets.
#[test]
fn hardlight_reg_color() {
    let mut rp = RegParams::new("hardlight_color");

    // Rust blend_hard_light requires same dimensions; use same image as both layers
    let pix = crate::common::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Full fraction hard light (self-blend)
    let result = pix
        .blend_hard_light(&pix, 1.0)
        .expect("blend_hard_light full");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Partial fraction
    let partial = pix
        .blend_hard_light(&pix, 0.5)
        .expect("blend_hard_light 0.5");
    rp.compare_values(w as f64, partial.width() as f64, 0.0);

    // Zero fraction should return copy of base
    let zero = pix
        .blend_hard_light(&pix, 0.0)
        .expect("blend_hard_light 0.0");
    rp.compare_values(w as f64, zero.width() as f64, 0.0);

    assert!(rp.cleanup(), "hardlight color test failed");
}

/// Test blend_hard_light on 8bpp grayscale images (C checks with 8bpp).
///
/// Verifies hard light blending works on same-size 8bpp grayscale images.
#[test]
fn hardlight_reg_gray() {
    let mut rp = RegParams::new("hardlight_gray");

    // blend_hard_light requires same dimensions and depth
    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit8);
    let w = pix.width();
    let h = pix.height();

    let result = pix
        .blend_hard_light(&pix, 0.7)
        .expect("blend_hard_light gray");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit8);

    assert!(rp.cleanup(), "hardlight gray test failed");
}

/// Test blend_hard_light with original C test images (C checks 0-8, 9-17).
///
/// Requires hardlight1_1.jpg, hardlight1_2.jpg, hardlight2_1.jpg, hardlight2_2.jpg
/// which are not available in the test data.
#[test]
#[ignore = "not yet implemented: hardlight test images (hardlight*_*.jpg) not available"]
fn hardlight_reg_original_images() {
    // C version:
    // TestHardlight("hardlight1_1.jpg", "hardlight1_2.jpg", rp);
    // TestHardlight("hardlight2_1.jpg", "hardlight2_2.jpg", rp);
    // Tests no-colormap, colormap, and in-place variants
}
