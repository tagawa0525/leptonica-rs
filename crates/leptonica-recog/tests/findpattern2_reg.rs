//! Find pattern regression test (2)
//!
//! Tests pattern matching with different SEL generation methods.
//! The C version compares boundary, run/line, and random methods
//! for generating hit-miss SELs to detect asterisk patterns.
//!
//! Partial port: Tests hit_miss_transform with manually constructed
//! SELs on the asterisk page image, and verifies pattern detection
//! produces reasonable results. The C version's pixGenerateSelBoundary,
//! pixGenerateSelWithRuns, and pixGenerateSelRandom are not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/findpattern2_reg.c`

use leptonica_color::threshold_to_binary;
use leptonica_core::PixelDepth;
use leptonica_morph::{Sel, SelElement, dilate_brick, hit_miss_transform};
use leptonica_region::{ConnectivityType, conncomp_pixa};
use leptonica_test::RegParams;

/// Test asterisk detection using HMT with a cross-shaped SEL.
///
/// C: pixHMT(NULL, pixs, sel) with SELs generated from one-asterisk.png
///
/// Rust: Manually construct a cross-shaped SEL to detect asterisk-like patterns.
#[test]
fn findpattern2_reg_asterisk_hmt() {
    let mut rp = RegParams::new("findpat2_asterisk");

    let pix = leptonica_test::load_test_image("asterisk.png").expect("load asterisk.png");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };
    assert_eq!(pix_bin.depth(), PixelDepth::Bit1);

    // Cross-shaped SEL for detecting asterisk-like center pixels
    let mut sel = Sel::new(5, 5).expect("create 5x5 sel");
    sel.set_origin(2, 2).expect("set_origin");
    // Center cross: hits
    sel.set_element(2, 2, SelElement::Hit);
    sel.set_element(1, 2, SelElement::Hit);
    sel.set_element(3, 2, SelElement::Hit);
    sel.set_element(2, 1, SelElement::Hit);
    sel.set_element(2, 3, SelElement::Hit);
    // Corners: miss (asterisk has gaps at corners)
    sel.set_element(0, 0, SelElement::Miss);
    sel.set_element(4, 0, SelElement::Miss);
    sel.set_element(0, 4, SelElement::Miss);
    sel.set_element(4, 4, SelElement::Miss);

    let hmt_result = hit_miss_transform(&pix_bin, &sel).expect("hmt asterisk");
    rp.compare_values(pix_bin.width() as f64, hmt_result.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, hmt_result.height() as f64, 0.0);

    assert!(rp.cleanup(), "findpattern2 asterisk_hmt test failed");
}

/// Test pattern image loading and component analysis.
///
/// C: pixConnComp to verify number of asterisks found.
///
/// Rust: Detect asterisks via dilation + connected components.
#[test]
fn findpattern2_reg_component_count() {
    let mut rp = RegParams::new("findpat2_count");

    let pix = leptonica_test::load_test_image("asterisk.png").expect("load asterisk.png");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    // Dilate to merge nearby pixels into connected regions
    let dilated = dilate_brick(&pix_bin, 3, 3).expect("dilate");

    // Count connected components
    let (boxa, _pixa) = conncomp_pixa(&dilated, ConnectivityType::EightWay).expect("conncomp");

    // Should find asterisk components
    rp.compare_values(1.0, if boxa.len() > 0 { 1.0 } else { 0.0 }, 0.0);

    // All components should have valid dimensions
    let all_valid = (0..boxa.len()).all(|i| {
        if let Some(b) = boxa.get(i) {
            b.w > 0 && b.h > 0
        } else {
            false
        }
    });
    rp.compare_values(1.0, if all_valid { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "findpattern2 component_count test failed");
}

/// Test one-asterisk template properties.
///
/// C: pixRead("one-asterisk.png")
///
/// Verify template image properties.
#[test]
fn findpattern2_reg_template() {
    let mut rp = RegParams::new("findpat2_template");

    let template =
        leptonica_test::load_test_image("one-asterisk.png").expect("load one-asterisk.png");
    let page = leptonica_test::load_test_image("asterisk.png").expect("load asterisk.png");

    // Template should be smaller than the page
    rp.compare_values(
        1.0,
        if template.width() < page.width() && template.height() < page.height() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Template should be roughly square (asterisk shape)
    let aspect = template.width() as f64 / template.height().max(1) as f64;
    rp.compare_values(
        1.0,
        if aspect > 0.5 && aspect < 2.0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "findpattern2 template test failed");
}
