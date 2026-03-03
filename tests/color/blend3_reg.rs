//! Blend regression test 3
//!
//! Tests various blend operations across multiple image type combinations.
//! The C version tests pixBlend, pixBlendGray, pixBlendGrayInverse,
//! pixBlendGrayAdapt, pixBlendColor, and pixBlendColorByChannel with
//! 6 input pairs (color/gray × 1bpp/8bpp/8bpp-cmap).
//!
//! Partial migration: blend_gray_inverse and blend_color_by_channel are
//! tested with available image combinations.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/blend3_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::{Color, PixelDepth};

/// Test blend_gray_inverse (C checks: BlendTest pixBlendGrayInverse portion).
///
/// Verifies inverse gray blending produces output with correct dimensions.
/// Rust impl only supports 8bpp (C supports 8bpp and 32bpp).
#[test]
fn blend3_reg_gray_inverse() {
    let mut rp = RegParams::new("blend3_inv");

    // blend_gray_inverse requires 8bpp base and blend images
    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let blend = crate::common::load_test_image("weasel8.png").expect("load weasel8.png");
    assert_eq!(pix.depth(), PixelDepth::Bit8);
    assert_eq!(blend.depth(), PixelDepth::Bit8);
    let w = pix.width();
    let h = pix.height();

    // C: pixBlendGrayInverse(NULL, pixs1, pixs2, 200, 100, fract)
    let result = pix
        .blend_gray_inverse(&blend, 200, 100, 0.5)
        .expect("blend_gray_inverse 0.5");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write blended gray_inverse");

    // Different position and fraction
    let result2 = pix
        .blend_gray_inverse(&blend, 50, 50, 0.3)
        .expect("blend_gray_inverse 0.3");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend3 gray inverse test failed");
}

/// Test blend_color_by_channel (C checks: BlendTest pixBlendColorByChannel).
///
/// Verifies per-channel color blending produces 32bpp output.
/// Both base and blend must be 32bpp for blend_color_by_channel.
#[test]
fn blend3_reg_color_by_channel() {
    let mut rp = RegParams::new("blend3_chan");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let blend = crate::common::load_test_image("weasel32.png").expect("load weasel32.png");
    let w = pix.width();
    let h = pix.height();

    // C: pixBlendColorByChannel(NULL, pixs1, pixs2, 200, 100,
    //     1.6*fract, fract, 0.5*fract, 1, 0xffffff00)
    let fract = 0.5_f32;
    let result = pix
        .blend_color_by_channel(
            &blend,
            200,
            100,
            1.6 * fract,
            fract,
            0.5 * fract,
            true,
            Color::WHITE,
        )
        .expect("blend_color_by_channel");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write blended color_by_channel");

    // Uniform channels
    let result2 = pix
        .blend_color_by_channel(&blend, 50, 50, 0.3, 0.3, 0.3, true, Color::WHITE)
        .expect("blend_color_by_channel uniform");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend3 color by channel test failed");
}

/// Test blend on 8bpp grayscale base (C checks 3-5: test8.jpg base).
///
/// Verifies blending works when the base image is 8bpp grayscale.
#[test]
fn blend3_reg_gray_base() {
    let mut rp = RegParams::new("blend3_gbase");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let blend = crate::common::load_test_image("weasel8.png").expect("load weasel8.png");
    assert_eq!(pix.depth(), PixelDepth::Bit8);
    assert_eq!(blend.depth(), PixelDepth::Bit8);
    let w = pix.width();
    let h = pix.height();

    // blend_gray_inverse on 8bpp base
    let result = pix
        .blend_gray_inverse(&blend, 140, 40, 0.5)
        .expect("blend_gray_inverse 8bpp");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write blended gray_base");

    // blend_gray_adapt on 8bpp base
    let result2 = pix
        .blend_gray_adapt(&blend, 140, 40, 0.5, 120)
        .expect("blend_gray_adapt 8bpp");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend3 gray base test failed");
}

/// Test pixBlend dispatch (C checks 0: BlendTest first section).
///
/// Requires the high-level pixBlend dispatch which automatically selects
/// blend mode based on image types. The Rust equivalent is pix.blend()
/// with explicit BlendMode.
#[test]
#[ignore = "not yet implemented: pixBlend dispatch not directly mapped to Rust API"]
fn blend3_reg_dispatch() {
    // C version uses pixBlend() which auto-dispatches based on depth.
    // Rust uses pix.blend(other, mode, fract) with explicit BlendMode.
    // This test would need to verify equivalent behavior, but the C pixBlend
    // dispatch logic differs from the Rust blend() API.
}
