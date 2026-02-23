//! Find pattern regression test (1)
//!
//! Tests pattern matching using hit-miss structural elements (SELs).
//! The C version uses pixGenerateSelBoundary to create hit-miss SELs
//! from pattern images, then pixHMT to find matches at multiple scales.
//!
//! Partial port: Tests hit_miss_transform with manually constructed
//! SELs on the tribune page image. The C version also uses
//! pixGenerateSelBoundary, pixDisplayMatchedPattern, and
//! pixRemoveMatchedPattern which are not available in the Rust API.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/findpattern1_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::morph::{Sel, SelElement, hit_miss_transform};

/// Test hit-miss transform on tribune page (C test: pixHMT).
///
/// C: sel = pixGenerateSelBoundary(pixt, HIT_DIST, MISS_DIST, ...)
///    pixd = pixHMT(NULL, pixs, sel)
///
/// Rust: Create a small rectangular SEL and run HMT on the binarized page.
#[test]
fn findpattern1_reg_hmt_basic() {
    let mut rp = RegParams::new("findpat1_hmt");

    let pix = crate::common::load_test_image("tribune-page-4x.png").expect("load tribune page");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("convert to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };
    assert_eq!(pix_bin.depth(), PixelDepth::Bit1);

    // Create a simple 3x3 HMT SEL: detect isolated foreground pixels
    // surrounded by background (a basic pattern detector)
    let mut sel = Sel::new(3, 3).expect("create sel");
    sel.set_origin(1, 1).expect("set_origin");
    // Center is hit, borders are miss
    sel.set_element(1, 1, SelElement::Hit);
    sel.set_element(0, 0, SelElement::Miss);
    sel.set_element(0, 2, SelElement::Miss);
    sel.set_element(2, 0, SelElement::Miss);
    sel.set_element(2, 2, SelElement::Miss);

    let hmt_result = hit_miss_transform(&pix_bin, &sel).expect("hit_miss_transform");

    rp.compare_values(pix_bin.width() as f64, hmt_result.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, hmt_result.height() as f64, 0.0);
    assert_eq!(hmt_result.depth(), PixelDepth::Bit1);

    assert!(rp.cleanup(), "findpattern1 hmt_basic test failed");
}

/// Test pattern image loading and properties.
///
/// C: pixt = pixRead("tribune-word.png")
///    pixt2 = pixRead("tribune-t.png")
///
/// Verify that pattern images load correctly and have expected properties.
#[test]
fn findpattern1_reg_pattern_images() {
    let mut rp = RegParams::new("findpat1_images");

    let word = crate::common::load_test_image("tribune-word.png").expect("load tribune-word.png");
    let letter_t = crate::common::load_test_image("tribune-t.png").expect("load tribune-t.png");
    let page = crate::common::load_test_image("tribune-page-4x.png").expect("load tribune-page-4x");

    // Pattern images should be binary
    let word_bin = if word.depth() == PixelDepth::Bit1 {
        word
    } else {
        let gray = word.convert_to_8().expect("word to gray");
        threshold_to_binary(&gray, 128).expect("word threshold")
    };

    let t_bin = if letter_t.depth() == PixelDepth::Bit1 {
        letter_t
    } else {
        let gray = letter_t.convert_to_8().expect("t to gray");
        threshold_to_binary(&gray, 128).expect("t threshold")
    };

    // Word pattern should be wider than tall (horizontal text)
    rp.compare_values(
        1.0,
        if word_bin.width() > word_bin.height() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Letter pattern should be smaller than word
    rp.compare_values(
        1.0,
        if t_bin.width() < word_bin.width() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Page should be larger than both patterns
    rp.compare_values(
        1.0,
        if page.width() > word_bin.width() && page.height() > word_bin.height() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "findpattern1 pattern_images test failed");
}

/// Test HMT with brick SEL on pattern page.
///
/// C: pixHMT with various SELs at different scales.
///
/// Rust: Use a horizontal brick SEL to detect horizontal features.
#[test]
fn findpattern1_reg_hmt_brick() {
    let mut rp = RegParams::new("findpat1_brick");

    let pix = crate::common::load_test_image("tribune-page-4x.png").expect("load page");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    // Horizontal line detector: wide hit bar with miss above and below
    let mut sel = Sel::new(7, 3).expect("create 7x3 sel");
    sel.set_origin(3, 1).expect("set_origin");
    // Top row: all miss
    for x in 0..7 {
        sel.set_element(x, 0, SelElement::Miss);
    }
    // Middle row: all hit
    for x in 0..7 {
        sel.set_element(x, 1, SelElement::Hit);
    }
    // Bottom row: all miss
    for x in 0..7 {
        sel.set_element(x, 2, SelElement::Miss);
    }

    let result = hit_miss_transform(&pix_bin, &sel).expect("hmt brick");
    rp.compare_values(pix_bin.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, result.height() as f64, 0.0);

    assert!(rp.cleanup(), "findpattern1 hmt_brick test failed");
}
