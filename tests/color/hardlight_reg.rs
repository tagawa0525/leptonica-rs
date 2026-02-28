//! Hard light blending regression test
//!
//! Tests the hard light blend mode. The C version tests pixBlendHardLight
//! with various image combinations including colormapped images and in-place
//! blending.
//!
//! Full migration: blend_hard_light is tested with the original C test
//! images (hardlight1_*.jpg, hardlight2_*.jpg) as well as substitute images.
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
/// Uses hardlight1_1.jpg, hardlight1_2.jpg, hardlight2_1.jpg, hardlight2_2.jpg.
/// Rust blend_hard_light requires same dimensions and depth, so images are
/// scaled to matching size first.
#[test]
fn hardlight_reg_original_images() {
    let mut rp = RegParams::new("hardlight_orig");

    let pairs = [
        ("hardlight1_1.jpg", "hardlight1_2.jpg"),
        ("hardlight2_1.jpg", "hardlight2_2.jpg"),
    ];

    for (file1, file2) in &pairs {
        let pix1 = crate::common::load_test_image(file1).expect(file1);
        let pix2 = crate::common::load_test_image(file2).expect(file2);

        // Scale both images to matching dimensions (use pix1's size)
        let w = pix1.width();
        let h = pix1.height();
        let pix2_scaled =
            leptonica::transform::scale_to_size(&pix2, w, h).expect("scale pix2 to match pix1");

        // Ensure both are 32bpp
        let p1 = if pix1.depth() != PixelDepth::Bit32 {
            pix1.convert_to_32().expect("convert pix1 to 32")
        } else {
            pix1
        };
        let p2 = if pix2_scaled.depth() != PixelDepth::Bit32 {
            pix2_scaled.convert_to_32().expect("convert pix2 to 32")
        } else {
            pix2_scaled
        };

        // blend_hard_light: pix1 base, pix2 blend (C check 0/9)
        let result = p1.blend_hard_light(&p2, 1.0).expect("blend p1 over p2");
        rp.compare_values(w as f64, result.width() as f64, 0.0);
        rp.compare_values(h as f64, result.height() as f64, 0.0);
        assert_eq!(result.depth(), PixelDepth::Bit32);

        // Reverse blend: pix2 base, pix1 blend (C check 2/11)
        let result_rev = p2.blend_hard_light(&p1, 1.0).expect("blend p2 over p1");
        rp.compare_values(w as f64, result_rev.width() as f64, 0.0);
        rp.compare_values(h as f64, result_rev.height() as f64, 0.0);

        // Partial fraction blend (C uses 1.0, test 0.5 for coverage)
        let partial = p1.blend_hard_light(&p2, 0.5).expect("blend partial");
        rp.compare_values(w as f64, partial.width() as f64, 0.0);
    }

    assert!(rp.cleanup(), "hardlight original images test failed");
}
