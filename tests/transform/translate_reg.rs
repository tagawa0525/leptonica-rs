//! Translation regression test
//!
//! Tests in-place and allocating translation of images at various depths.
//! The C version tests pixTranslate with L_BRING_IN_WHITE/BLACK on
//! colormapped, grayscale, binary, and RGB images with both positive
//! and negative shifts.
//!
//! Partial migration: the Rust translate() uses floating-point offsets
//! via affine matrix (no explicit fill color parameter). Pixel-level
//! verification uses clip_rectangle to check shifted content.
//!
//! # See also
//!
//! C Leptonica: `prog/translate_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;

/// Test translation with positive offsets (C check 0).
///
/// Translates a grayscale image by positive (x, y) and verifies
/// the output dimensions are preserved.
#[test]
fn translate_reg_positive_shift() {
    let mut rp = RegParams::new("translate_pos");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let w = pix.width();
    let h = pix.height();

    // Translate by (30, 25) pixels
    let shifted = leptonica::transform::translate(&pix, 30.0, 25.0).expect("translate +30,+25");
    rp.compare_values(w as f64, shifted.width() as f64, 0.0);
    rp.compare_values(h as f64, shifted.height() as f64, 0.0);
    rp.write_pix_and_check(&shifted, ImageFormat::Png)
        .expect("write shifted translate_pos");

    // The pixel at (30, 25) in the shifted image should match (0, 0) in original
    let p_orig = pix.get_pixel(0, 0).expect("get_pixel origin");
    let p_shifted = shifted.get_pixel(30, 25).expect("get_pixel shifted");
    rp.compare_values(p_orig as f64, p_shifted as f64, 0.0);

    assert!(rp.cleanup(), "translate positive shift test failed");
}

/// Test translation with negative offsets (C check 2).
///
/// Translates a grayscale image by negative (x, y) and verifies
/// the output dimensions are preserved and content shifted.
#[test]
fn translate_reg_negative_shift() {
    let mut rp = RegParams::new("translate_neg");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let w = pix.width();
    let h = pix.height();

    // Translate by (-20, -15) pixels
    let shifted = leptonica::transform::translate(&pix, -20.0, -15.0).expect("translate -20,-15");
    rp.compare_values(w as f64, shifted.width() as f64, 0.0);
    rp.compare_values(h as f64, shifted.height() as f64, 0.0);
    rp.write_pix_and_check(&shifted, ImageFormat::Png)
        .expect("write shifted translate_neg");

    // The pixel at (0, 0) in shifted should match (20, 15) in original
    let p_orig = pix.get_pixel(20, 15).expect("get_pixel (20,15)");
    let p_shifted = shifted.get_pixel(0, 0).expect("get_pixel shifted origin");
    rp.compare_values(p_orig as f64, p_shifted as f64, 0.0);

    assert!(rp.cleanup(), "translate negative shift test failed");
}

/// Test translation on 32bpp RGB image (C checks 0-2 at depth 32).
///
/// Verifies translation works on color images and preserves RGB values.
#[test]
fn translate_reg_rgb() {
    let mut rp = RegParams::new("translate_rgb");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();

    let shifted = leptonica::transform::translate(&pix, 15.0, 20.0).expect("translate rgb");
    rp.compare_values(w as f64, shifted.width() as f64, 0.0);
    rp.compare_values(h as f64, shifted.height() as f64, 0.0);
    rp.write_pix_and_check(&shifted, ImageFormat::Png)
        .expect("write shifted translate_rgb");

    // Check pixel correspondence
    let p_orig = pix.get_pixel(10, 10).expect("get_pixel (10,10)");
    let p_shifted = shifted
        .get_pixel(25, 30)
        .expect("get_pixel shifted (25,30)");
    rp.compare_values(p_orig as f64, p_shifted as f64, 0.0);

    assert!(rp.cleanup(), "translate rgb test failed");
}

/// Test translation with colormapped and rotated images (C full checks).
///
/// Requires pixRemoveColormap, pixConvertTo1, pixRotateAM with
/// L_BRING_IN_BLACK/WHITE which are partially available.
#[test]
#[ignore = "not yet implemented: full C translate test requires pixRotateAM fill modes"]
fn translate_reg_multitype() {
    // C version:
    // 1. Scale colormapped image, clip to rectangle
    // 2. Remove colormap to grayscale and RGB
    // 3. Convert to 1bpp
    // 4. Rotate with area mapping (bring in black/white)
    // 5. Translate each with +/- shifts and white/black fill
}

/// C-comparable translate series (plan 902 PR 15).
///
/// Mirrors C translate_reg exactly: weasel2.4c.png sampled 3x and
/// clipped to 209x214, then the cmapped / gray / RGB / binary variants
/// (plus four `rotate_am` results) are each translated four ways
/// (±shift x white/black) and tiled with
/// `display_tiled_in_columns(4, 1.0, 30, 3)`.
#[test]
fn translate_c_compat() {
    use leptonica::core::pix::RemoveColormapTarget;
    use leptonica::core::pix::rop::InColor;
    use leptonica::transform::{RotateFill, rotate_am, scale_by_sampling};
    use leptonica::{Pix, Pixa};

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("translate_c");

    // C: pixScaleBySampling(weasel2.4c.png, 3, 3) clipped to 209x214.
    let src = crate::common::load_test_image("weasel2.4c.png").expect("load weasel2.4c.png");
    let scaled = scale_by_sampling(&src, 3.0, 3.0).expect("scale 3x");
    let pixs = scaled.clip_rectangle(0, 0, 209, 214).expect("clip");

    let pix1 = pixs
        .remove_colormap(RemoveColormapTarget::ToGrayscale)
        .expect("to gray");
    let pix2 = pixs
        .remove_colormap(RemoveColormapTarget::ToFullColor)
        .expect("to full color");
    let pix3 = pixs.convert_to_1(128).expect("convert to 1bpp");
    let pix4 = rotate_am(&pix1, 0.25, RotateFill::Black).expect("rotate +0.25 black");
    let pix5 = rotate_am(&pix1, -0.25, RotateFill::White).expect("rotate -0.25 white");
    let pix6 = rotate_am(&pix2, -0.15, RotateFill::Black).expect("rotate -0.15 black");
    let pix7 = rotate_am(&pix2, 0.15, RotateFill::White).expect("rotate +0.15 white");

    // C TranslateAndSave{1,2}: four translations per image.
    let push_four = |pixa: &mut Pixa, pix: &Pix, xs: i32, ys: i32| {
        pixa.push(pix.translate(xs, ys, InColor::White));
        pixa.push(pix.translate(xs, ys, InColor::Black));
        pixa.push(pix.translate(-xs, -ys, InColor::White));
        pixa.push(pix.translate(-xs, -ys, InColor::Black));
    };

    // C check 0
    let mut pixa = Pixa::new();
    push_four(&mut pixa, &pixs, 30, 30);
    push_four(&mut pixa, &pix1, 35, 20);
    push_four(&mut pixa, &pix2, 20, 35);
    push_four(&mut pixa, &pix3, 20, 35);
    let out = pixa
        .display_tiled_in_columns(4, 1.0, 30, 3)
        .expect("tiled 0");
    rp.write_pix_and_check(&out, ImageFormat::Png)
        .expect("check: translate set 0");

    // C check 1
    let mut pixa = Pixa::new();
    push_four(&mut pixa, &pix1, 35, 20);
    push_four(&mut pixa, &pix4, 35, 20);
    let out = pixa
        .display_tiled_in_columns(4, 1.0, 30, 3)
        .expect("tiled 1");
    rp.write_pix_and_check(&out, ImageFormat::Png)
        .expect("check: translate set 1");

    // C check 2
    let mut pixa = Pixa::new();
    push_four(&mut pixa, &pixs, 30, 30);
    push_four(&mut pixa, &pix1, 30, 30);
    push_four(&mut pixa, &pix2, 35, 20);
    push_four(&mut pixa, &pix3, 20, 35);
    push_four(&mut pixa, &pix4, 25, 25);
    push_four(&mut pixa, &pix5, 25, 25);
    push_four(&mut pixa, &pix6, 25, 25);
    push_four(&mut pixa, &pix7, 25, 25);
    let out = pixa
        .display_tiled_in_columns(4, 1.0, 30, 3)
        .expect("tiled 2");
    rp.write_pix_and_check(&out, ImageFormat::Png)
        .expect("check: translate set 2");

    assert!(rp.cleanup(), "translate c-compat test failed");
}
