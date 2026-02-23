//! Partition regression test
//!
//! Tests partitioning of white space in document images into rectangles.
//! The C version uses boxaGetWhiteblocks to partition whitespace with
//! connected component analysis and box sorting.
//!
//! Partial port: Tests connected component extraction, dilation for block
//! detection, box selection by size, and box drawing on document images.
//! The C version's boxaGetWhiteblocks, boxaPermuteRandom, and
//! pixCopyWithBoxa are not available in the Rust API.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/partition_reg.c`

use leptonica_color::threshold_to_binary;
use leptonica_core::{Pix, PixelDepth, SizeRelation};
use leptonica_morph::dilate_brick;
use leptonica_region::{ConnectivityType, conncomp_pixa};
use leptonica_test::RegParams;

/// Test connected components on test8.jpg (C test: pixConnComp).
///
/// C: pixConnComp(pix1, &boxa, 4)
///    Extract components from binarized photo.
#[test]
fn partition_reg_conncomp_test8() {
    let mut rp = RegParams::new("partition_test8");

    let pix = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");
    let pix_gray = pix.convert_to_8().expect("convert to gray");
    let pix_bin = threshold_to_binary(&pix_gray, 128).expect("threshold");
    assert_eq!(pix_bin.depth(), PixelDepth::Bit1);

    let (boxa, pixa) = conncomp_pixa(&pix_bin, ConnectivityType::FourWay).expect("conncomp test8");

    // Should find components
    rp.compare_values(1.0, if boxa.len() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(boxa.len() as f64, pixa.len() as f64, 0.0);

    assert!(rp.cleanup(), "partition conncomp_test8 test failed");
}

/// Test dilation + connected components on feyn-fract.tif (C test: partition).
///
/// C: pixDilateBrick(NULL, pix1, 5, 5)
///    pixConnComp(pix2, &boxa, 4)
///    boxaSelectBySize(boxa, 3, 3, L_SELECT_IF_BOTH, ...)
#[test]
fn partition_reg_dilate_conncomp() {
    let mut rp = RegParams::new("partition_dilate");

    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };
    assert_eq!(pix_bin.depth(), PixelDepth::Bit1);

    // Dilate to merge nearby components into blocks
    let dilated = dilate_brick(&pix_bin, 5, 5).expect("dilate 5x5");
    rp.compare_values(pix_bin.width() as f64, dilated.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, dilated.height() as f64, 0.0);

    // Extract block-level components
    let (boxa, _) = conncomp_pixa(&dilated, ConnectivityType::FourWay).expect("conncomp dilated");

    rp.compare_values(1.0, if boxa.len() > 0 { 1.0 } else { 0.0 }, 0.0);

    // Filter by minimum size (>= 3x3)
    let filtered = boxa.select_by_size(3, 3, SizeRelation::GreaterThanOrEqual);
    rp.compare_values(
        1.0,
        if filtered.len() <= boxa.len() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "partition dilate_conncomp test failed");
}

/// Test box selection by size on document components.
///
/// C: boxaSelectBySize(boxa, w, h, L_SELECT_IF_BOTH, L_SELECT_IF_GTE, ...)
///
/// Rust: select_by_size filters boxes by minimum dimensions.
#[test]
fn partition_reg_select_by_size() {
    let mut rp = RegParams::new("partition_select");

    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    let dilated = dilate_brick(&pix_bin, 5, 5).expect("dilate");
    let (boxa, _) = conncomp_pixa(&dilated, ConnectivityType::FourWay).expect("conncomp");

    // Small filter: many components
    let small = boxa.select_by_size(2, 2, SizeRelation::GreaterThanOrEqual);
    // Large filter: fewer components
    let large = boxa.select_by_size(50, 50, SizeRelation::GreaterThanOrEqual);

    rp.compare_values(1.0, if large.len() <= small.len() { 1.0 } else { 0.0 }, 0.0);

    // Very large filter: should have fewer still
    let very_large = boxa.select_by_size(200, 200, SizeRelation::GreaterThanOrEqual);
    rp.compare_values(
        1.0,
        if very_large.len() <= large.len() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "partition select_by_size test failed");
}

/// Test box drawing on image (C test: pixDrawBoxaRandom, pixPaintBoxaRandom).
///
/// C: pixDrawBoxaRandom(pix32, boxa, 2)
///    pixPaintBoxaRandom(pix32, boxa)
///
/// Rust: draw_boxa_random and paint_boxa_random on 32bpp output.
#[test]
fn partition_reg_draw_boxes() {
    let mut rp = RegParams::new("partition_draw");

    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix.clone()
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    let dilated = dilate_brick(&pix_bin, 5, 5).expect("dilate");
    let (boxa, _) = conncomp_pixa(&dilated, ConnectivityType::FourWay).expect("conncomp");

    if boxa.len() == 0 {
        rp.compare_values(1.0, 0.0, 0.0);
        assert!(rp.cleanup(), "no boxes found");
        return;
    }

    // Create 32bpp canvas and draw boxes
    let canvas = Pix::new(pix_bin.width(), pix_bin.height(), PixelDepth::Bit32)
        .expect("create 32bpp canvas");
    let mut canvas_mut = canvas.try_into_mut().expect("try_into_mut");

    canvas_mut
        .draw_boxa_random(&boxa, 2)
        .expect("draw_boxa_random");
    let drawn: Pix = canvas_mut.into();

    rp.compare_values(pix_bin.width() as f64, drawn.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, drawn.height() as f64, 0.0);
    assert_eq!(drawn.depth(), PixelDepth::Bit32);

    // Also test paint_boxa_random
    let canvas2 = Pix::new(pix_bin.width(), pix_bin.height(), PixelDepth::Bit32)
        .expect("create 32bpp canvas");
    let mut canvas2_mut = canvas2.try_into_mut().expect("try_into_mut");

    canvas2_mut
        .paint_boxa_random(&boxa)
        .expect("paint_boxa_random");
    let painted: Pix = canvas2_mut.into();

    rp.compare_values(pix_bin.width() as f64, painted.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, painted.height() as f64, 0.0);

    assert!(rp.cleanup(), "partition draw_boxes test failed");
}
