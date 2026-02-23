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
//! C Leptonica: `reference/leptonica/prog/translate_reg.c`

use leptonica_test::RegParams;

/// Test translation with positive offsets (C check 0).
///
/// Translates a grayscale image by positive (x, y) and verifies
/// the output dimensions are preserved.
#[test]
#[ignore = "not yet implemented"]
fn translate_reg_positive_shift() {
    let mut rp = RegParams::new("translate_pos");

    let pix = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let w = pix.width();
    let h = pix.height();

    // Translate by (30, 25) pixels
    let shifted = leptonica_transform::translate(&pix, 30.0, 25.0).expect("translate +30,+25");
    rp.compare_values(w as f64, shifted.width() as f64, 0.0);
    rp.compare_values(h as f64, shifted.height() as f64, 0.0);

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
#[ignore = "not yet implemented"]
fn translate_reg_negative_shift() {
    let mut rp = RegParams::new("translate_neg");

    let pix = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let w = pix.width();
    let h = pix.height();

    // Translate by (-20, -15) pixels
    let shifted = leptonica_transform::translate(&pix, -20.0, -15.0).expect("translate -20,-15");
    rp.compare_values(w as f64, shifted.width() as f64, 0.0);
    rp.compare_values(h as f64, shifted.height() as f64, 0.0);

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
#[ignore = "not yet implemented"]
fn translate_reg_rgb() {
    let mut rp = RegParams::new("translate_rgb");

    let pix = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();

    let shifted = leptonica_transform::translate(&pix, 15.0, 20.0).expect("translate rgb");
    rp.compare_values(w as f64, shifted.width() as f64, 0.0);
    rp.compare_values(h as f64, shifted.height() as f64, 0.0);

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
