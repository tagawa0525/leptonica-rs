//! Line removal regression test
//!
//! Tests line removal from document images using skew detection,
//! grayscale morphology, and masked combination. The C version
//! demonstrates a full pipeline on dave-orig.png: skew detection,
//! rotation, morphological line detection, thresholding, inversion,
//! addition, and masked combination.
//!
//! Partial port: Tests find_skew, gray morphology (close/open/erode),
//! arith_add, invert, and combine_masked. The C version also uses
//! pixThresholdToValue and pixRotateAMGray which are not available.
//!
//! # See also
//!
//! C Leptonica: `prog/lineremoval_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::io::ImageFormat;
use leptonica::morph::{close_gray, erode_gray, open_gray};
use leptonica::recog::skew::{SkewDetectOptions, find_skew};

/// Test skew detection on dave-orig.png (C test section: pixFindSkew).
///
/// C: pixFindSkew(pixb, &angle, &conf)
///    dave-orig.png has a small skew angle that should be detected.
#[test]
fn lineremoval_reg_find_skew() {
    let mut rp = RegParams::new("lineremoval_skew");

    let pix = crate::common::load_test_image("dave-orig.png").expect("load dave-orig.png");

    // Convert to binary for skew detection
    let pix_gray = pix.convert_to_8().expect("convert to gray");
    let pix_bin = threshold_to_binary(&pix_gray, 128).expect("threshold");
    assert_eq!(pix_bin.depth(), PixelDepth::Bit1);

    let opts = SkewDetectOptions::default();
    let result = find_skew(&pix_bin, &opts).expect("find_skew");
    eprintln!(
        "  Skew angle: {}, confidence: {}",
        result.angle, result.confidence
    );

    // Skew angle should be small (document is nearly horizontal)
    rp.compare_values(1.0, if result.angle.abs() < 5.0 { 1.0 } else { 0.0 }, 0.0);

    // Confidence should be non-negative (0.0 means no skew detected, which is valid)
    rp.compare_values(1.0, if result.confidence >= 0.0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "lineremoval find_skew test failed");
}

/// Test grayscale morphology for line detection (C test section: close/erode).
///
/// C: pixCloseGray(pixg, 51, 1) — horizontal close to connect line fragments
///    pixErodeGray(pixg, 51, 1) — horizontal erode to thin lines
///
/// Rust: close_gray and erode_gray with horizontal structuring elements.
#[test]
fn lineremoval_reg_gray_morph() {
    let mut rp = RegParams::new("lineremoval_morph");

    let pix = crate::common::load_test_image("dave-orig.png").expect("load dave-orig.png");
    let pix_gray = pix.convert_to_8().expect("convert to gray");
    assert_eq!(pix_gray.depth(), PixelDepth::Bit8);

    // Horizontal close: connect horizontal line fragments
    let closed = close_gray(&pix_gray, 51, 1).expect("close_gray 51x1");
    rp.compare_values(pix_gray.width() as f64, closed.width() as f64, 0.0);
    rp.compare_values(pix_gray.height() as f64, closed.height() as f64, 0.0);
    assert_eq!(closed.depth(), PixelDepth::Bit8);

    rp.write_pix_and_check(&closed, ImageFormat::Png)
        .expect("write closed lineremoval_morph");

    // Horizontal erode: thin the detected lines
    let eroded = erode_gray(&closed, 51, 1).expect("erode_gray 51x1");
    rp.compare_values(pix_gray.width() as f64, eroded.width() as f64, 0.0);
    rp.compare_values(pix_gray.height() as f64, eroded.height() as f64, 0.0);

    rp.write_pix_and_check(&eroded, ImageFormat::Png)
        .expect("write eroded lineremoval_morph");

    // Open: remove narrow features
    let opened = open_gray(&pix_gray, 1, 5).expect("open_gray 1x5");
    rp.compare_values(pix_gray.width() as f64, opened.width() as f64, 0.0);
    rp.compare_values(pix_gray.height() as f64, opened.height() as f64, 0.0);

    assert!(rp.cleanup(), "lineremoval gray_morph test failed");
}

/// Test arithmetic and logical operations (C test section: add/invert/combine).
///
/// C: pixInvert(pixd, pixd)
///    pixAddGray(NULL, pixg, pixd)
///    pixCombineMasked(pixd, pixg, pixm)
///
/// Rust: invert(), arith_add(), combine_masked() on grayscale images.
#[test]
fn lineremoval_reg_arith_combine() {
    let mut rp = RegParams::new("lineremoval_arith");

    let pix = crate::common::load_test_image("dave-orig.png").expect("load dave-orig.png");
    let pix_gray = pix.convert_to_8().expect("convert to gray");
    let w = pix_gray.width();
    let h = pix_gray.height();

    // Invert
    let inverted = pix_gray.invert();
    rp.compare_values(w as f64, inverted.width() as f64, 0.0);
    rp.compare_values(h as f64, inverted.height() as f64, 0.0);
    assert_eq!(inverted.depth(), PixelDepth::Bit8);

    // Add: original + inverted should produce near-white (255) for 8bpp
    let added = pix_gray.arith_add(&inverted).expect("arith_add");
    rp.compare_values(w as f64, added.width() as f64, 0.0);
    rp.compare_values(h as f64, added.height() as f64, 0.0);

    // Create a binary mask from the image for combine_masked test
    let mask = threshold_to_binary(&pix_gray, 128).expect("threshold for mask");

    // combine_masked: replace dest pixels with source where mask is ON
    let mut dest = pix_gray.to_mut();
    let src_gray = open_gray(&pix_gray, 5, 5).expect("open for combine source");
    dest.combine_masked(&src_gray, &mask)
        .expect("combine_masked");
    let result: leptonica::Pix = dest.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);

    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result lineremoval_arith");

    assert!(rp.cleanup(), "lineremoval arith_combine test failed");
}

/// Test the full line removal pipeline (simplified version).
///
/// C: Full pipeline: threshold → skew → close → erode → threshold_to_value
///    → invert → add → open → combine_masked
///
/// Rust: Simplified pipeline using available APIs.
#[test]
fn lineremoval_reg_pipeline() {
    let mut rp = RegParams::new("lineremoval_pipe");

    let pix = crate::common::load_test_image("dave-orig.png").expect("load dave-orig.png");
    let pix_gray = pix.convert_to_8().expect("convert to gray");

    // Step 1: Detect horizontal lines via close + erode
    let lines = close_gray(&pix_gray, 51, 1).expect("close for lines");
    let lines = erode_gray(&lines, 51, 1).expect("erode for lines");

    // Step 2: Create line mask by thresholding
    let line_mask = threshold_to_binary(&lines, 150).expect("line mask");
    assert_eq!(line_mask.depth(), PixelDepth::Bit1);

    // Step 3: Remove lines by combining original with opened version
    let cleaned = open_gray(&pix_gray, 1, 5).expect("open for clean");

    // Step 4: Use mask to replace line regions
    let mut result = pix_gray.to_mut();
    result
        .combine_masked(&cleaned, &line_mask)
        .expect("combine_masked");
    let result: leptonica::Pix = result.into();

    rp.compare_values(pix_gray.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(pix_gray.height() as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit8);

    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result lineremoval_pipe");

    assert!(rp.cleanup(), "lineremoval pipeline test failed");
}

/// C-comparable line removal pipeline (plan 902 PR 20).
///
/// Mirrors C lineremoval_reg exactly: dave-orig.png thresholded, skew
/// detected and corrected with `rotate_am_gray`, lines isolated with
/// gray close/erode and two `threshold_to_value` steps, then merged back
/// with `arith_add` and `combine_masked`. All ten outputs are PNG.
#[test]
fn lineremoval_c_compat() {
    use leptonica::morph::{close_gray, erode_gray, open_gray};
    use leptonica::recog::skew::{SkewDetectOptions, find_skew};
    use leptonica::transform::{RotateFill, rotate_am_gray};

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("lineremoval_c");

    let pixs = crate::common::load_test_image("dave-orig.png").expect("load dave-orig.png");
    // C: `l_float32 deg2rad = 3.14159 / 180.` — a double division
    // rounded once to float.
    #[allow(clippy::approx_constant)] // C hard-codes this truncated pi
    let deg2rad = (3.14159f64 / 180.0) as f32;

    // C 0: threshold to binary at 170
    let pix1 = threshold_to_binary(&pixs, 170).expect("threshold 170");
    rp.write_pix_and_check(&pix1, ImageFormat::Png)
        .expect("check 0");

    // C 1: deskew the grayscale original by the detected angle
    let skew = find_skew(&pix1, &SkewDetectOptions::default()).expect("find_skew");
    let pix2 =
        rotate_am_gray(&pixs, deg2rad * skew.angle, RotateFill::White).expect("rotate_am_gray");
    rp.write_pix_and_check(&pix2, ImageFormat::Png)
        .expect("check 1");

    // C 2-3: isolate horizontal lines
    let pix3 = close_gray(&pix2, 51, 1).expect("close_gray 51x1");
    rp.write_pix_and_check(&pix3, ImageFormat::Png)
        .expect("check 2");
    let pix4 = erode_gray(&pix3, 1, 5).expect("erode_gray 1x5");
    rp.write_pix_and_check(&pix4, ImageFormat::Png)
        .expect("check 3");

    // C 4-5: flatten the background, then the lines
    let pix5 = pix4
        .threshold_to_value(210, 255)
        .expect("threshold_to_value 210");
    rp.write_pix_and_check(&pix5, ImageFormat::Png)
        .expect("check 4");
    let pix6 = pix5
        .threshold_to_value(200, 0)
        .expect("threshold_to_value 200");
    rp.write_pix_and_check(&pix6, ImageFormat::Png)
        .expect("check 5");

    // C 6: line mask
    let pix7 = threshold_to_binary(&pix6, 210).expect("threshold 210");
    rp.write_pix_and_check(&pix7, ImageFormat::Png)
        .expect("check 6");

    // C 7: add the inverted line image back to the deskewed original
    let pix6_inv = pix6.invert();
    let pix8 = pix2.arith_add(&pix6_inv).expect("arith_add");
    rp.write_pix_and_check(&pix8, ImageFormat::Png)
        .expect("check 7");

    // C 8-9: vertical opening, then paste it back through the line mask
    let pix9 = open_gray(&pix8, 1, 9).expect("open_gray 1x9");
    rp.write_pix_and_check(&pix9, ImageFormat::Png)
        .expect("check 8");
    let merged = {
        let mut m = pix8.deep_clone().to_mut();
        m.combine_masked(&pix9, &pix7).expect("combine_masked");
        let p: leptonica::Pix = m.into();
        p
    };
    rp.write_pix_and_check(&merged, ImageFormat::Png)
        .expect("check 9");

    assert!(rp.cleanup(), "lineremoval c-compat test failed");
}

/// Area-map rotation must derive sin/cos like C (plan 902 PR 20).
///
/// C `rotateAMGrayLow` computes `sina = 16.f * sin(angle)` where `sin()`
/// returns a double, so the scaling happens in double precision and is
/// rounded to float once. Computing `angle.sin()` in f32 and scaling in
/// f32 shifts a few sub-pixel positions across a truncation boundary.
#[test]
fn lineremoval_rotate_am_gray_matches_c() {
    use leptonica::transform::{RotateFill, rotate_am_gray};

    let pixs = crate::common::load_test_image("dave-orig.png").expect("load dave-orig.png");
    // C: `l_float32 deg2rad = 3.14159 / 180.` — a double division
    // rounded once to float.
    #[allow(clippy::approx_constant)] // C hard-codes this truncated pi
    let deg2rad = (3.14159f64 / 180.0) as f32;
    let out = rotate_am_gray(&pixs, deg2rad * -0.656250, RotateFill::White).expect("rotate");

    // Pixels where the f32 scaling picks the neighbouring source sample.
    // Values here come from the C reference (pixRotateAMGray on the same
    // input and angle).
    assert_eq!(out.get_pixel(491, 246).unwrap(), 254);
}
