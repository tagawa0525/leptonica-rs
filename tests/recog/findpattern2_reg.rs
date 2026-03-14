//! Find pattern regression test (2)
//!
//! Tests pattern matching with different SEL generation methods.
//! The C version compares boundary, run/line, and random methods
//! for generating hit-miss SELs to detect asterisk patterns.
//!
//! Expanded in Phase 5 to use all three SEL generation APIs:
//! generate_sel_boundary, generate_sel_with_runs, generate_sel_random.
//!
//! # See also
//!
//! C Leptonica: `prog/findpattern2_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::io::ImageFormat;
use leptonica::morph::selgen::{
    generate_sel_boundary, generate_sel_random, generate_sel_with_runs,
};
use leptonica::morph::{
    Sel, SelElement, dilate_brick, display_matched_pattern, hit_miss_transform,
    remove_matched_pattern,
};
use leptonica::region::{ConnectivityType, conncomp_pixa};

fn load_binary(name: &str) -> leptonica::Pix {
    let pix = crate::common::load_test_image(name).expect("load image");
    if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("convert to gray");
        threshold_to_binary(&gray, 128).expect("threshold to binary")
    }
}

/// Test asterisk detection using HMT with a cross-shaped SEL.
///
/// C: pixHMT(NULL, pixs, sel) with SELs generated from one-asterisk.png
///
/// Rust: Manually construct a cross-shaped SEL to detect asterisk-like patterns.
#[test]
fn findpattern2_reg_asterisk_hmt() {
    let mut rp = RegParams::new("findpat2_asterisk");

    let pix = crate::common::load_test_image("asterisk.png").expect("load asterisk.png");
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

    rp.write_pix_and_check(&hmt_result, ImageFormat::Tiff)
        .expect("write hmt_result findpat2_asterisk");

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

    let pix = crate::common::load_test_image("asterisk.png").expect("load asterisk.png");
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
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);

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
        crate::common::load_test_image("one-asterisk.png").expect("load one-asterisk.png");
    let page = crate::common::load_test_image("asterisk.png").expect("load asterisk.png");

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

/// Test boundary SEL generation (recommended method).
///
/// C: pixGenerateSelBoundary(pixt, 2, 4, 1, 1, 1, 1, 1, 1)
///    pixHMT → pixDisplayMatchedPattern → pixRemoveMatchedPattern
///
/// The boundary method places hits/misses at specified distances from
/// the pattern boundary — the most reliable of the three methods.
#[test]
fn findpattern2_reg_boundary_sel() {
    let mut rp = RegParams::new("findpat2_boundary");

    let page_bin = load_binary("asterisk.png");
    let template_bin = load_binary("one-asterisk.png");

    // Generate SEL using boundary method
    let sel = generate_sel_boundary(&template_bin, 2, 4, 1, 1, true, true, true, true)
        .expect("generate_sel_boundary");
    let (sh, sw, _, _) = sel.get_parameters();
    assert!(sh > 0 && sw > 0, "boundary SEL dimensions > 0");

    let hmt = hit_miss_transform(&page_bin, &sel).expect("hmt boundary");
    rp.write_pix_and_check(&hmt, ImageFormat::Tiff)
        .expect("write boundary hmt");

    let x0 = sel.origin_x() as i32;
    let y0 = sel.origin_y() as i32;

    // Display and remove matches
    let displayed =
        display_matched_pattern(&page_bin, &template_bin, &hmt, x0, y0, 0xff000000, 1.0)
            .expect("display boundary matches");
    rp.write_pix_and_check(&displayed, ImageFormat::Tiff)
        .expect("write displayed boundary");

    let removed = remove_matched_pattern(&page_bin, &template_bin, &hmt, x0, y0, 2)
        .expect("remove boundary matches");
    rp.write_pix_and_check(&removed, ImageFormat::Tiff)
        .expect("write removed boundary");

    assert!(rp.cleanup(), "findpattern2 boundary_sel test failed");
}

/// Test run-based SEL generation.
///
/// C: pixGenerateSelWithRuns(pixt, nhlines=5, nvlines=5, distance=1, min_length=3, ...)
///    pixHMT → pixDisplayMatchedPattern
///
/// The runs method samples horizontal/vertical lines through the pattern.
#[test]
fn findpattern2_reg_runs_sel() {
    let mut rp = RegParams::new("findpat2_runs");

    let page_bin = load_binary("asterisk.png");
    let template_bin = load_binary("one-asterisk.png");

    // Generate SEL using runs method (5 horizontal + 5 vertical lines, min run length=3)
    let sel = generate_sel_with_runs(&template_bin, 5, 5, 1, 3, 2, 2, 2, 2)
        .expect("generate_sel_with_runs");
    let (sh, sw, _, _) = sel.get_parameters();
    assert!(sh > 0 && sw > 0, "runs SEL dimensions > 0");

    let hmt = hit_miss_transform(&page_bin, &sel).expect("hmt runs");
    rp.write_pix_and_check(&hmt, ImageFormat::Tiff)
        .expect("write runs hmt");

    let x0 = sel.origin_x() as i32;
    let y0 = sel.origin_y() as i32;

    let displayed =
        display_matched_pattern(&page_bin, &template_bin, &hmt, x0, y0, 0xff000000, 1.0)
            .expect("display runs matches");
    rp.write_pix_and_check(&displayed, ImageFormat::Tiff)
        .expect("write displayed runs");

    assert!(rp.cleanup(), "findpattern2 runs_sel test failed");
}

/// Test random SEL generation.
///
/// C: pixGenerateSelRandom(pixt, hit_fract=0.5, miss_fract=0.5, distance=2, ...)
///    pixHMT → pixDisplayMatchedPattern
///
/// The random method subsamples safe FG/BG pixels at given fractions.
#[test]
fn findpattern2_reg_random_sel() {
    let mut rp = RegParams::new("findpat2_random");

    let page_bin = load_binary("asterisk.png");
    let template_bin = load_binary("one-asterisk.png");

    // Generate SEL using random method (50% hit, 50% miss fractions)
    let sel =
        generate_sel_random(&template_bin, 0.5, 0.5, 2, 2, 2, 2, 2).expect("generate_sel_random");
    let (sh, sw, _, _) = sel.get_parameters();
    assert!(sh > 0 && sw > 0, "random SEL dimensions > 0");

    let hmt = hit_miss_transform(&page_bin, &sel).expect("hmt random");
    rp.write_pix_and_check(&hmt, ImageFormat::Tiff)
        .expect("write random hmt");

    let x0 = sel.origin_x() as i32;
    let y0 = sel.origin_y() as i32;

    let displayed =
        display_matched_pattern(&page_bin, &template_bin, &hmt, x0, y0, 0xff000000, 1.0)
            .expect("display random matches");
    rp.write_pix_and_check(&displayed, ImageFormat::Tiff)
        .expect("write displayed random");

    assert!(rp.cleanup(), "findpattern2 random_sel test failed");
}
