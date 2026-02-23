//! Blend regression test (4)
//!
//! Tests alpha blending with uniform backgrounds and gray mask blending.
//! The C version uses pixAddAlphaToBlend to create RGBA blenders,
//! pixMirroredTiling for tiled patterns, and pixBlendWithGrayMask
//! for mask-based compositing.
//!
//! Partial port: Tests add_alpha_to_blend with multiple blender images,
//! alpha_blend_uniform compositing, and blend_with_gray_mask with
//! a synthetic gradient mask. pixMirroredTiling is not available in the
//! Rust API, so blenders are used without tiling.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/blend4_reg.c`

use leptonica_core::pix::blend::blend_with_gray_mask;
use leptonica_core::{Pix, PixelDepth};
use leptonica_test::RegParams;
use leptonica_transform::scale_by_sampling;

/// Test add_alpha_to_blend on various blender images (C checks 0-4).
///
/// C: pixAddAlphaToBlend(pixt, 0.3, 0) on feyn-word, weasel, karen
///    Creates RGBA blenders with 30% opacity.
#[test]
fn blend4_reg_add_alpha() {
    let mut rp = RegParams::new("blend4_alpha");

    // Load blender images
    // feyn-word.tif is 1bpp; add_alpha_to_blend requires 8 or 32bpp, so convert first
    let feyn_raw = leptonica_test::load_test_image("feyn-word.tif").expect("load feyn-word.tif");
    let feyn = feyn_raw.convert_to_8().expect("feyn to 8bpp");
    let weasel_raw =
        leptonica_test::load_test_image("weasel4.16c.png").expect("load weasel4.16c.png");
    let weasel = weasel_raw.convert_to_32().expect("weasel to 32bpp");
    let karen = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");

    // add_alpha_to_blend at 30% opacity, normal (not inverted)
    let feyn_alpha = feyn.add_alpha_to_blend(0.3, false).expect("feyn alpha");
    assert_eq!(feyn_alpha.depth(), PixelDepth::Bit32);
    rp.compare_values(feyn.width() as f64, feyn_alpha.width() as f64, 0.0);
    rp.compare_values(feyn.height() as f64, feyn_alpha.height() as f64, 0.0);

    let weasel_alpha = weasel.add_alpha_to_blend(0.3, false).expect("weasel alpha");
    assert_eq!(weasel_alpha.depth(), PixelDepth::Bit32);
    rp.compare_values(weasel.width() as f64, weasel_alpha.width() as f64, 0.0);

    let karen_alpha = karen.add_alpha_to_blend(0.3, false).expect("karen alpha");
    assert_eq!(karen_alpha.depth(), PixelDepth::Bit32);
    rp.compare_values(karen.width() as f64, karen_alpha.width() as f64, 0.0);

    // Also test inverted alpha (C checks 3-4)
    let karen_inv = karen
        .add_alpha_to_blend(0.3, true)
        .expect("karen inverted alpha");
    assert_eq!(karen_inv.depth(), PixelDepth::Bit32);
    rp.compare_values(karen.width() as f64, karen_inv.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend4 add_alpha test failed");
}

/// Test alpha_blend_uniform compositing (C checks 0-4 continued).
///
/// C: Composite RGBA blender onto white/light background.
///
/// Rust: alpha_blend_uniform flattens RGBA to RGB with solid background.
#[test]
fn blend4_reg_alpha_composite() {
    let mut rp = RegParams::new("blend4_composite");

    let karen = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let rgba = karen.add_alpha_to_blend(0.3, false).expect("alpha");

    // Composite on white background
    let composited = rgba
        .alpha_blend_uniform(0xFFFFFF00)
        .expect("composite white");
    assert_eq!(composited.depth(), PixelDepth::Bit32);
    rp.compare_values(karen.width() as f64, composited.width() as f64, 0.0);
    rp.compare_values(karen.height() as f64, composited.height() as f64, 0.0);

    // Composite on black background
    let composited_black = rgba
        .alpha_blend_uniform(0x00000000)
        .expect("composite black");
    rp.compare_values(karen.width() as f64, composited_black.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend4 alpha_composite test failed");
}

/// Test blend_with_gray_mask (C checks 5-8).
///
/// C: pixBlendWithGrayMask(pix1, pix2, pixg, x, y)
///    Uses a gradient mask for smooth alpha blending between two images.
///
/// Rust: Create a synthetic gradient mask and blend fish24 with wyom.
#[test]
fn blend4_reg_gray_mask_blend() {
    let mut rp = RegParams::new("blend4_mask");

    let fish = leptonica_test::load_test_image("fish24.jpg").expect("load fish24.jpg");
    let wyom = leptonica_test::load_test_image("wyom.jpg").expect("load wyom.jpg");

    // Scale wyom to match fish dimensions
    let w = fish.width();
    let h = fish.height();
    let scale_x = w as f32 / wyom.width() as f32;
    let scale_y = h as f32 / wyom.height() as f32;
    let wyom_scaled = scale_by_sampling(&wyom, scale_x, scale_y).expect("scale wyom");

    // Create an 8bpp gradient mask (horizontal gradient from 0 to 255)
    let mask = Pix::new(w, h, PixelDepth::Bit8).expect("create mask");
    let mut mask_mut = mask.try_into_mut().unwrap();
    for y_pos in 0..h {
        for x_pos in 0..w {
            let val = ((x_pos as f64 / w as f64) * 255.0) as u32;
            mask_mut.set_pixel(x_pos, y_pos, val).unwrap();
        }
    }
    let mask: Pix = mask_mut.into();

    // Blend fish and wyom using the gradient mask
    let blended = blend_with_gray_mask(&fish, &wyom_scaled, &mask, 0, 0).expect("gray mask blend");

    rp.compare_values(w as f64, blended.width() as f64, 0.0);
    rp.compare_values(h as f64, blended.height() as f64, 0.0);
    assert_eq!(blended.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "blend4 gray_mask_blend test failed");
}

/// Test blend_with_gray_mask with offset positioning.
///
/// C: pixBlendWithGrayMask(pix1, pix2, pixg, x, y) with non-zero offset.
#[test]
fn blend4_reg_mask_offset() {
    let mut rp = RegParams::new("blend4_offset");

    let fish = leptonica_test::load_test_image("fish24.jpg").expect("load fish24.jpg");
    let karen = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let karen32 = karen.add_alpha_to_blend(0.5, false).expect("karen alpha");
    let karen_rgb = karen32.alpha_blend_uniform(0x80808000).expect("flatten");

    // Create a uniform 128 mask (50% blend)
    let mw = karen.width().min(fish.width() / 2);
    let mh = karen.height().min(fish.height() / 2);
    let mask = Pix::new(mw, mh, PixelDepth::Bit8).expect("create mask");
    let mut mask_mut = mask.try_into_mut().unwrap();
    for y_pos in 0..mh {
        for x_pos in 0..mw {
            mask_mut.set_pixel(x_pos, y_pos, 128).unwrap();
        }
    }
    let mask: Pix = mask_mut.into();

    // Blend with offset
    let blended = blend_with_gray_mask(&fish, &karen_rgb, &mask, 50, 50).expect("offset blend");
    rp.compare_values(fish.width() as f64, blended.width() as f64, 0.0);
    rp.compare_values(fish.height() as f64, blended.height() as f64, 0.0);

    assert!(rp.cleanup(), "blend4 mask_offset test failed");
}
