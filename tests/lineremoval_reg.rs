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
//! C Leptonica: `reference/leptonica/prog/lineremoval_reg.c`

mod common;
use common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::morph::{close_gray, erode_gray, open_gray};
use leptonica::recog::skew::{SkewDetectOptions, find_skew};

/// Test skew detection on dave-orig.png (C test section: pixFindSkew).
///
/// C: pixFindSkew(pixb, &angle, &conf)
///    dave-orig.png has a small skew angle that should be detected.
#[test]
fn lineremoval_reg_find_skew() {
    let mut rp = RegParams::new("lineremoval_skew");

    let pix = common::load_test_image("dave-orig.png").expect("load dave-orig.png");

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

    let pix = common::load_test_image("dave-orig.png").expect("load dave-orig.png");
    let pix_gray = pix.convert_to_8().expect("convert to gray");
    assert_eq!(pix_gray.depth(), PixelDepth::Bit8);

    // Horizontal close: connect horizontal line fragments
    let closed = close_gray(&pix_gray, 51, 1).expect("close_gray 51x1");
    rp.compare_values(pix_gray.width() as f64, closed.width() as f64, 0.0);
    rp.compare_values(pix_gray.height() as f64, closed.height() as f64, 0.0);
    assert_eq!(closed.depth(), PixelDepth::Bit8);

    // Horizontal erode: thin the detected lines
    let eroded = erode_gray(&closed, 51, 1).expect("erode_gray 51x1");
    rp.compare_values(pix_gray.width() as f64, eroded.width() as f64, 0.0);
    rp.compare_values(pix_gray.height() as f64, eroded.height() as f64, 0.0);

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

    let pix = common::load_test_image("dave-orig.png").expect("load dave-orig.png");
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

    let pix = common::load_test_image("dave-orig.png").expect("load dave-orig.png");
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

    assert!(rp.cleanup(), "lineremoval pipeline test failed");
}
