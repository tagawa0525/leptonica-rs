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

use leptonica_core::PixelDepth;
use leptonica_test::RegParams;

/// Test blend_hard_light on 32bpp color images (C checks 0-1).
///
/// Verifies hard light blending produces 32bpp output with correct dimensions.
#[test]
#[ignore = "not yet implemented: blend_hard_light on color images"]
fn hardlight_reg_color() {
    let mut rp = RegParams::new("hardlight_color");

    // C uses hardlight1_1.jpg + hardlight1_2.jpg; we substitute with available images
    let pix1 = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    let pix2 = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");
    assert_eq!(pix1.depth(), PixelDepth::Bit32);
    assert_eq!(pix2.depth(), PixelDepth::Bit32);
    let w = pix1.width();
    let h = pix1.height();

    // C: pixBlendHardLight(NULL, pixs1, pixs2, 0, 0, 1.0)
    let result = pix1
        .blend_hard_light(&pix2, 1.0)
        .expect("blend_hard_light full");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Reverse order (C: pixBlendHardLight(NULL, pixs2, pixs1, 0, 0, 1.0))
    let reversed = pix2
        .blend_hard_light(&pix1, 1.0)
        .expect("blend_hard_light reversed");
    rp.compare_values(pix2.width() as f64, reversed.width() as f64, 0.0);
    rp.compare_values(pix2.height() as f64, reversed.height() as f64, 0.0);

    // Partial fraction
    let partial = pix1
        .blend_hard_light(&pix2, 0.5)
        .expect("blend_hard_light 0.5");
    rp.compare_values(w as f64, partial.width() as f64, 0.0);

    assert!(rp.cleanup(), "hardlight color test failed");
}

/// Test blend_hard_light with grayscale image (C checks with 8bpp).
///
/// Verifies hard light blending works when one image is 8bpp grayscale.
#[test]
#[ignore = "not yet implemented: blend_hard_light with grayscale"]
fn hardlight_reg_gray() {
    let mut rp = RegParams::new("hardlight_gray");

    let pix_color = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    let pix_gray = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");
    assert_eq!(pix_color.depth(), PixelDepth::Bit32);
    assert_eq!(pix_gray.depth(), PixelDepth::Bit8);

    // C: pixBlendHardLight(NULL, pix_color, pix_gray, 0, 0, 1.0)
    let result = pix_color
        .blend_hard_light(&pix_gray, 1.0)
        .expect("blend_hard_light color+gray");
    rp.compare_values(pix_color.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(pix_color.height() as f64, result.height() as f64, 0.0);

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
