//! Blend regression test 1
//!
//! Tests pixBlendGray, pixBlendGrayAdapt, and pixBlendColor. The C version
//! blends a small image repeatedly at grid positions over a larger image,
//! testing gray (straight and inverse), adaptive gray, and color blending.
//!
//! Partial migration: blend_gray, blend_gray_adapt, and blend_color are
//! tested. blender8.png is not available; weasel8.png is used as substitute.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/blend1_reg.c`

mod common;
use common::RegParams;
use leptonica::{GrayBlendType, PixelDepth};

/// Test blend_gray with GrayBlendType::Gray (C checks 0-2).
///
/// Verifies gray blending onto a color image preserves dimensions.
#[test]
fn blend1_reg_gray_straight() {
    let mut rp = RegParams::new("blend1_gray");

    let pix = common::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    // C uses blender8.png; we use weasel8.png as 8bpp blend source
    let blend = common::load_test_image("weasel8.png").expect("load weasel8.png");
    assert_eq!(blend.depth(), PixelDepth::Bit8);
    let w = pix.width();
    let h = pix.height();

    // C: pixBlendGray(pixs, pixs, pixb, x, y, 0.3, L_BLEND_GRAY, 1, 255)
    let result = pix
        .blend_gray(&blend, 30, 20, 0.3, GrayBlendType::Gray)
        .expect("blend_gray straight");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Blend with different fraction
    let result2 = pix
        .blend_gray(&blend, 100, 100, 0.6, GrayBlendType::Gray)
        .expect("blend_gray 0.6");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend1 gray straight test failed");
}

/// Test blend_gray with GrayBlendType::GrayWithInverse (C checks 3-7).
///
/// Verifies gray blending with inverse mode preserves dimensions.
#[test]
fn blend1_reg_gray_inverse() {
    let mut rp = RegParams::new("blend1_inv");

    let pix = common::load_test_image("marge.jpg").expect("load marge.jpg");
    let blend = common::load_test_image("weasel8.png").expect("load weasel8.png");
    let w = pix.width();
    let h = pix.height();

    // C: pixBlendGray(pixs, pixs, pixb, x, y, 0.6, L_BLEND_GRAY_WITH_INVERSE, 1, 255)
    let result = pix
        .blend_gray(&blend, 30, 20, 0.6, GrayBlendType::GrayWithInverse)
        .expect("blend_gray with inverse");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "blend1 gray inverse test failed");
}

/// Test blend_gray_adapt (C checks 8-12).
///
/// Verifies adaptive gray blending preserves dimensions.
/// Rust impl only supports 8bpp (C supports 8bpp and 32bpp).
#[test]
fn blend1_reg_adapt() {
    let mut rp = RegParams::new("blend1_adapt");

    // blend_gray_adapt requires 8bpp base and blend images
    let pix = common::load_test_image("test8.jpg").expect("load test8.jpg");
    let blend = common::load_test_image("weasel8.png").expect("load weasel8.png");
    assert_eq!(pix.depth(), PixelDepth::Bit8);
    assert_eq!(blend.depth(), PixelDepth::Bit8);
    let w = pix.width();
    let h = pix.height();

    // C: pixBlendGrayAdapt(pixs, pixs, pixb, x, y, 0.8, 80)
    let result = pix
        .blend_gray_adapt(&blend, 30, 20, 0.8, 80)
        .expect("blend_gray_adapt");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit8);

    // Different fraction and shift
    let result2 = pix
        .blend_gray_adapt(&blend, 100, 100, 0.3, 120)
        .expect("blend_gray_adapt 0.3");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend1 adapt test failed");
}

/// Test blend_color (C checks 13-15).
///
/// Verifies color blending onto a color image preserves dimensions.
#[test]
fn blend1_reg_color() {
    let mut rp = RegParams::new("blend1_color");

    let pix = common::load_test_image("test24.jpg").expect("load test24.jpg");
    // C uses weasel4.11c.png with colormap removed; load 32bpp color blend source
    let blend = common::load_test_image("weasel32.png").expect("load weasel32.png");
    let w = pix.width();
    let h = pix.height();

    // C: pixBlendColor(pixs, pixs, pixc, x, y, 0.3, 1, 255)
    let result = pix
        .blend_color(&blend, 30, 20, 0.3)
        .expect("blend_color 0.3");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Different fraction
    let result2 = pix
        .blend_color(&blend, 100, 100, 0.15)
        .expect("blend_color 0.15");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend1 color test failed");
}
