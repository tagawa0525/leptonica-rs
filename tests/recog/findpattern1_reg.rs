//! Find pattern regression test (1)
//!
//! Tests pattern matching using hit-miss structural elements (SELs).
//! The C version uses pixGenerateSelBoundary to create hit-miss SELs
//! from pattern images, then pixHMT to find matches at multiple scales.
//!
//! Expanded in Phase 5 to use generate_sel_boundary, display_matched_pattern,
//! and remove_matched_pattern APIs.
//!
//! # See also
//!
//! C Leptonica: `prog/findpattern1_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::io::ImageFormat;
use leptonica::morph::selgen::generate_sel_boundary;
use leptonica::morph::{
    Sel, SelElement, display_matched_pattern, hit_miss_transform, remove_matched_pattern,
};

fn load_binary(name: &str) -> leptonica::Pix {
    let pix = crate::common::load_test_image(name).expect("load image");
    if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("convert to gray");
        threshold_to_binary(&gray, 128).expect("threshold to binary")
    }
}

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

    rp.write_pix_and_check(&hmt_result, ImageFormat::Tiff)
        .expect("write hmt_result findpat1_hmt");

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

    rp.write_pix_and_check(&result, ImageFormat::Tiff)
        .expect("write result findpat1_brick");

    assert!(rp.cleanup(), "findpattern1 hmt_brick test failed");
}

/// Test automatic SEL generation from pattern boundary.
///
/// C: sel = pixGenerateSelBoundary(pixt, HIT_DIST=2, MISS_DIST=5, ...)
///    pixhmt = pixHMT(NULL, pix_page_bin, sel)
///
/// Uses both tribune-word.png and tribune-t.png patterns.
#[test]
fn findpattern1_reg_sel_boundary_hmt() {
    let mut rp = RegParams::new("findpat1_sel_boundary");

    let page_bin = load_binary("tribune-page-4x.png");

    // Generate SEL from word pattern boundary (C: HIT_DIST=2, MISS_DIST=4)
    let word_bin = load_binary("tribune-word.png");
    let sel_word = generate_sel_boundary(&word_bin, 2, 4, 1, 1, true, true, true, true)
        .expect("generate_sel_boundary word");
    // SEL should have positive dimensions
    let (sh, sw, _, _) = sel_word.get_parameters();
    assert!(sh > 0, "SEL word height > 0");
    assert!(sw > 0, "SEL word width > 0");

    let hmt_word = hit_miss_transform(&page_bin, &sel_word).expect("hmt word");
    rp.write_pix_and_check(&hmt_word, ImageFormat::Tiff)
        .expect("write hmt word");

    // Generate SEL from letter T pattern
    let t_bin = load_binary("tribune-t.png");
    let sel_t = generate_sel_boundary(&t_bin, 2, 4, 1, 1, true, true, true, true)
        .expect("generate_sel_boundary t");
    let (th, tw, _, _) = sel_t.get_parameters();
    assert!(th > 0 && tw > 0, "SEL t dimensions > 0");

    let hmt_t = hit_miss_transform(&page_bin, &sel_t).expect("hmt t");
    rp.write_pix_and_check(&hmt_t, ImageFormat::Tiff)
        .expect("write hmt t");

    assert!(rp.cleanup(), "findpattern1 sel_boundary_hmt test failed");
}

/// Test display and removal of matched patterns.
///
/// C: pixDisplayMatchedPattern(pix, pixt, pixhmt, x0, y0, color, scale)
///    pixRemoveMatchedPattern(pix, pixt, pixhmt, x0, y0, dsize)
///
/// Applies SEL-based HMT, then highlights and removes matches.
#[test]
fn findpattern1_reg_display_and_remove() {
    let mut rp = RegParams::new("findpat1_display_remove");

    let page_bin = load_binary("tribune-page-4x.png");

    // Generate SEL from word pattern
    let word_bin = load_binary("tribune-word.png");
    let sel_word = generate_sel_boundary(&word_bin, 2, 4, 1, 1, true, true, true, true)
        .expect("generate_sel_boundary word");
    let x0 = sel_word.origin_x() as i32;
    let y0 = sel_word.origin_y() as i32;

    let hmt_word = hit_miss_transform(&page_bin, &sel_word).expect("hmt word");

    // Display matched word patterns in red (0xff000000 = red in RGBA)
    let displayed =
        display_matched_pattern(&page_bin, &word_bin, &hmt_word, x0, y0, 0xff000000, 1.0)
            .expect("display_matched_pattern word");
    rp.write_pix_and_check(&displayed, ImageFormat::Tiff)
        .expect("write displayed word");

    // Remove matched word patterns (dsize=2 for slightly expanded removal)
    let removed = remove_matched_pattern(&page_bin, &word_bin, &hmt_word, x0, y0, 2)
        .expect("remove_matched_pattern word");
    rp.write_pix_and_check(&removed, ImageFormat::Tiff)
        .expect("write removed word");

    // Verify removal reduced foreground pixels
    let orig_count = page_bin.count_pixels();
    let removed_count = removed.count_pixels();
    // Removing patterns should reduce pixel count (or leave same if no matches)
    assert!(
        removed_count <= orig_count,
        "removal should not add pixels: {removed_count} > {orig_count}"
    );

    // Same pipeline with letter T pattern
    let t_bin = load_binary("tribune-t.png");
    let sel_t = generate_sel_boundary(&t_bin, 2, 4, 1, 1, true, true, true, true)
        .expect("generate_sel_boundary t");
    let tx0 = sel_t.origin_x() as i32;
    let ty0 = sel_t.origin_y() as i32;

    let hmt_t = hit_miss_transform(&page_bin, &sel_t).expect("hmt t");

    let displayed_t = display_matched_pattern(&page_bin, &t_bin, &hmt_t, tx0, ty0, 0xff000000, 1.0)
        .expect("display_matched_pattern t");
    rp.write_pix_and_check(&displayed_t, ImageFormat::Tiff)
        .expect("write displayed t");

    let removed_t = remove_matched_pattern(&page_bin, &t_bin, &hmt_t, tx0, ty0, 2)
        .expect("remove_matched_pattern t");
    rp.write_pix_and_check(&removed_t, ImageFormat::Tiff)
        .expect("write removed t");

    assert!(rp.cleanup(), "findpattern1 display_and_remove test failed");
}
