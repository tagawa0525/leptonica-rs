//! Alpha operations regression test
//!
//! Tests alpha blending with uniform background, alpha removal, and
//! alpha channel generation. The C version tests pixAlphaBlendUniform,
//! pixSetAlphaOverWhite, pixBlendWithGrayMask, and pixMultiplyByColor.
//!
//! Full migration: alpha_blend_uniform, remove_alpha, multiply_by_color,
//! blend_with_gray_mask, set_alpha_over_white, and blend_background_to_color
//! are all tested.
//!
//! # See also
//!
//! C Leptonica: `prog/alphaops_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::{PixelDepth, blend_with_gray_mask};

/// Test alpha_blend_uniform (C checks 0-1, 4).
///
/// Verifies blending an RGBA image over a uniform background color.
#[test]
fn alphaops_reg_blend_uniform() {
    let mut rp = RegParams::new("alphaops_uniform");

    let pix = crate::common::load_test_image("books_logo.png").expect("load books_logo.png");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Blend with white background (C: pixAlphaBlendUniform(pix1, 0xffffff00))
    let blended_white = pix
        .alpha_blend_uniform(0xffffff00)
        .expect("blend with white");
    rp.compare_values(w as f64, blended_white.width() as f64, 0.0);
    rp.compare_values(h as f64, blended_white.height() as f64, 0.0);
    assert_eq!(blended_white.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&blended_white, ImageFormat::Png)
        .expect("write blended blend_uniform");

    // Blend with light yellow background (C: pixAlphaBlendUniform(pix3, 0xffffe000))
    let blended_yellow = pix
        .alpha_blend_uniform(0xffffe000)
        .expect("blend with yellow");
    rp.compare_values(w as f64, blended_yellow.width() as f64, 0.0);
    rp.compare_values(h as f64, blended_yellow.height() as f64, 0.0);

    assert!(rp.cleanup(), "alphaops blend_uniform test failed");
}

/// Test remove_alpha and add_alpha_to_blend (C checks 0-1 round-trip).
///
/// Verifies that alpha can be removed (compositing against white) and
/// that add_alpha_to_blend generates a usable alpha channel.
#[test]
fn alphaops_reg_remove_add_alpha() {
    let mut rp = RegParams::new("alphaops_alpha");

    let pix = crate::common::load_test_image("books_logo.png").expect("load books_logo.png");
    let w = pix.width();
    let h = pix.height();

    // Remove alpha (composite against white)
    let no_alpha = pix.remove_alpha().expect("remove_alpha");
    rp.compare_values(w as f64, no_alpha.width() as f64, 0.0);
    rp.compare_values(h as f64, no_alpha.height() as f64, 0.0);
    assert_eq!(no_alpha.depth(), PixelDepth::Bit32);

    // Add alpha for blending (C: pixAddAlphaToBlend)
    let with_alpha = no_alpha
        .add_alpha_to_blend(0.5, false)
        .expect("add_alpha_to_blend");
    rp.compare_values(w as f64, with_alpha.width() as f64, 0.0);
    rp.compare_values(h as f64, with_alpha.height() as f64, 0.0);
    rp.write_pix_and_check(&with_alpha, ImageFormat::Png)
        .expect("write readded remove_add_alpha");

    assert!(rp.cleanup(), "alphaops remove/add alpha test failed");
}

/// Test multiply_by_color (C check 3 DoBlendTest which==2).
///
/// Verifies component-wise color multiplication preserves dimensions.
#[test]
fn alphaops_reg_multiply_by_color() {
    let mut rp = RegParams::new("alphaops_mult");

    let pix = crate::common::load_test_image("test24.jpg").expect("load test24.jpg");
    let w = pix.width();
    let h = pix.height();

    // C: pixMultiplyByColor(NULL, pix, NULL, 0xffffa000) → RGBA: R=0xff G=0xff B=0xa0
    let color = leptonica::Color::new(0xff, 0xff, 0xa0);
    let result = pix.multiply_by_color(color).expect("multiply_by_color");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write multiplied multiply_by_color");

    assert!(rp.cleanup(), "alphaops multiply_by_color test failed");
}

/// Test blend_with_gray_mask from alphaops (C check 7-8).
///
/// Verifies blending two images using a gray mask.
#[test]
fn alphaops_reg_blend_with_mask() {
    let mut rp = RegParams::new("alphaops_mask");

    let pix1 = crate::common::load_test_image("test24.jpg").expect("load test24.jpg");
    let pix2 = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix1.width();
    let h = pix1.height();

    // Create a simple gradient mask (8bpp)
    let mask = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");

    let blended = blend_with_gray_mask(&pix1, &pix2, &mask, 0, 0).expect("blend_with_gray_mask");
    rp.compare_values(w as f64, blended.width() as f64, 0.0);
    rp.compare_values(h as f64, blended.height() as f64, 0.0);
    assert_eq!(blended.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&blended, ImageFormat::Png)
        .expect("write blended blend_with_mask");

    assert!(rp.cleanup(), "alphaops blend_with_mask test failed");
}

/// Test pixSetAlphaOverWhite and pixBlendBackgroundToColor (C checks 2-3).
///
/// Loads blend-green1.jpg, applies set_alpha_over_white to generate alpha
/// from white background, then blends back to a color with
/// blend_background_to_color.
#[test]
fn alphaops_reg_set_alpha_over_white() {
    let mut rp = RegParams::new("alphaops_alpha_white");

    // Load a 32bpp image (blend-green1.jpg)
    let pix = crate::common::load_test_image("blend-green1.jpg").expect("load blend-green1.jpg");
    let w = pix.width();
    let h = pix.height();

    // Ensure 32bpp for set_alpha_over_white
    let pix32 = if pix.depth() != PixelDepth::Bit32 {
        pix.convert_to_32().expect("convert to 32bpp")
    } else {
        pix
    };

    // C: pixSetAlphaOverWhite(pix2) — generate alpha from white background
    let with_alpha = pix32.set_alpha_over_white().expect("set_alpha_over_white");
    rp.compare_values(w as f64, with_alpha.width() as f64, 0.0);
    rp.compare_values(h as f64, with_alpha.height() as f64, 0.0);
    assert_eq!(with_alpha.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&with_alpha, ImageFormat::Png)
        .expect("write over_white set_alpha_over_white");

    // C: pixBlendBackgroundToColor(NULL, pix, ..., color, ...)
    // Blend the alpha image back toward a light yellow background
    let blended = with_alpha
        .blend_background_to_color(0xffffe000)
        .expect("blend_background_to_color");
    rp.compare_values(w as f64, blended.width() as f64, 0.0);
    rp.compare_values(h as f64, blended.height() as f64, 0.0);
    assert_eq!(blended.depth(), PixelDepth::Bit32);

    // Also test with blend-orange.jpg
    let pix_orange =
        crate::common::load_test_image("blend-orange.jpg").expect("load blend-orange.jpg");
    let pix_orange32 = if pix_orange.depth() != PixelDepth::Bit32 {
        pix_orange.convert_to_32().expect("convert orange to 32bpp")
    } else {
        pix_orange
    };
    let orange_alpha = pix_orange32
        .set_alpha_over_white()
        .expect("set_alpha_over_white orange");
    rp.compare_values(
        pix_orange32.width() as f64,
        orange_alpha.width() as f64,
        0.0,
    );
    rp.compare_values(
        pix_orange32.height() as f64,
        orange_alpha.height() as f64,
        0.0,
    );

    assert!(rp.cleanup(), "alphaops set_alpha_over_white test failed");
}

/// C-compatible port of the first section of `prog/alphaops_reg.c`
/// (C indices 0, 1, 3 and 4).
///
/// Only this section is mapped: everything after it either reads a JPEG or
/// writes one, so the decode rounding makes a pixel-exact match impossible
/// (see `docs/porting/c-compat-findings/001-*`). Here both the input
/// (`books_logo.png`) and the outputs are lossless.
///
/// C index 2 writes the same pix as 3 with `spp` forced to 3, and is checked
/// by file rather than by hash, so it has no counterpart here.
#[test]
fn alphaops_c_compat() {
    let mut rp = RegParams::new("alphaops_c");

    let pix1 = crate::common::load_test_image("books_logo.png").expect("load books_logo.png");
    // 0: the input itself, which also checks that PNG round-trips like C's.
    rp.write_pix_and_check(&pix1, ImageFormat::Png)
        .expect("write source");

    // 1: composite over white.
    let pix2 = pix1
        .alpha_blend_uniform(0xffffff00)
        .expect("alpha_blend_uniform white");
    rp.write_pix_and_check(&pix2, ImageFormat::Png)
        .expect("write blended over white");

    // 3: rebuild an alpha layer from what is now a white background.
    let pix3 = pix2.set_alpha_over_white().expect("set_alpha_over_white");
    rp.write_pix_and_check(&pix3, ImageFormat::Png)
        .expect("write regenerated alpha");

    // 4: composite that over light yellow.
    let pix4 = pix3
        .alpha_blend_uniform(0xffffe000)
        .expect("alpha_blend_uniform yellow");
    rp.write_pix_and_check(&pix4, ImageFormat::Png)
        .expect("write blended over yellow");

    assert!(rp.cleanup(), "alphaops c-compat test failed");
}
