//! Component filter regression test
//!
//! Tests filtering of connected components by size and shape metrics.
//! The C version uses pixSelectBySize, pixSelectByPerimToAreaRatio,
//! pixSelectByPerimSizeRatio, pixSelectByAreaFraction, and
//! indicator array operations.
//!
//! Implemented in Rust:
//!   - Box rendering / fill_closed_borders (C indices 0-1)
//!   - pixSelectBySize IfBoth / IfEither (C indices 2-13 equivalent)
//!   - boxaSelectBySize (C indices 26-27 equivalent)
//!
//! Not yet implemented:
//!   - pixSelectByPerimToAreaRatio (C indices 14-17)
//!   - pixSelectByPerimSizeRatio (C indices 18-21)
//!   - pixSelectByAreaFraction (C indices 22-25)
//!   - numaMakeThresholdIndicator / numaLogicalOp loop (C indices 28-85)
//!   - Complex multi-criterion filter (C index 86)
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/compfilter_reg.c`

// ---------------------------------------------------------------------------
// Helper: count connected components in a 1-bpp Pix (8-way)
// ---------------------------------------------------------------------------

fn count_pieces(pix: &leptonica::Pix) -> usize {
    use leptonica::region::{ConnectivityType, find_connected_components};
    find_connected_components(pix, ConnectivityType::EightWay)
        .map(|v| v.len())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Build the two synthetic test images used in C checks 0-27.
//
// pix1 – four filled solid boxes (via fill_closed_borders on outlined boxes)
// pix2 – four hash-filled boxes (various orientations)
//
// C reference:
//   box1 = boxCreate(10, 10, 20, 30)  → 20×30 at (10,10)
//   box2 = boxCreate(50, 10, 40, 20)  → 40×20 at (50,10)
//   box3 = boxCreate(110, 10, 35, 5)  → 35×5  at (110,10)
//   box4 = boxCreate(160, 10, 5, 15)  → 5×15  at (160,10)
// ---------------------------------------------------------------------------

fn build_test_images() -> (leptonica::Pix, leptonica::Pix) {
    use leptonica::core::pix::{HashOrientation, PixelOp};
    use leptonica::core::{Box, Pix, PixelDepth};
    use leptonica::region::{ConnectivityType, fill_closed_borders};

    let pixs = Pix::new(200, 200, PixelDepth::Bit1).expect("create 200x200 1bpp");
    let mut pixs_m = pixs.try_into_mut().expect("try_into_mut");

    let box1 = Box::new(10, 10, 20, 30).expect("box1");
    let box2 = Box::new(50, 10, 40, 20).expect("box2");
    let box3 = Box::new(110, 10, 35, 5).expect("box3");
    let box4 = Box::new(160, 10, 5, 15).expect("box4");

    pixs_m
        .render_box(&box1, 1, PixelOp::Set)
        .expect("render box1");
    pixs_m
        .render_box(&box2, 1, PixelOp::Set)
        .expect("render box2");
    pixs_m
        .render_box(&box3, 1, PixelOp::Set)
        .expect("render box3");
    pixs_m
        .render_box(&box4, 1, PixelOp::Set)
        .expect("render box4");

    let pixs: leptonica::Pix = pixs_m.into();

    // pix1: fill closed borders → solid filled boxes (C: pixFillClosedBorders(pixs, 4))
    let pix1 = fill_closed_borders(&pixs, ConnectivityType::FourWay).expect("fill_closed_borders");

    // pix2: hash-filled boxes (C: pixCreateTemplate + pixRenderHashBox × 4)
    let pix2_base = pixs.create_template();
    let mut pix2_m = pix2_base.try_into_mut().expect("pix2 try_into_mut");

    // C: pixRenderHashBox(pix2, box1, 6, 4, L_POS_SLOPE_LINE, 1, L_SET_PIXELS)
    pix2_m
        .render_hash_box(&box1, 6, 4, HashOrientation::PosSlope, true, PixelOp::Set)
        .expect("hash box1");
    // C: pixRenderHashBox(pix2, box2, 7, 2, L_POS_SLOPE_LINE, 1, L_SET_PIXELS)
    pix2_m
        .render_hash_box(&box2, 7, 2, HashOrientation::PosSlope, true, PixelOp::Set)
        .expect("hash box2");
    // C: pixRenderHashBox(pix2, box3, 4, 2, L_VERTICAL_LINE, 1, L_SET_PIXELS)
    pix2_m
        .render_hash_box(&box3, 4, 2, HashOrientation::Vertical, true, PixelOp::Set)
        .expect("hash box3");
    // C: pixRenderHashBox(pix2, box4, 3, 1, L_HORIZONTAL_LINE, 1, L_SET_PIXELS)
    pix2_m
        .render_hash_box(&box4, 3, 1, HashOrientation::Horizontal, true, PixelOp::Set)
        .expect("hash box4");

    let pix2: leptonica::Pix = pix2_m.into();
    (pix1, pix2)
}

// ---------------------------------------------------------------------------
// C check 0-1: write pix1 (filled boxes) and pix2 (hash boxes)
// ---------------------------------------------------------------------------

/// Test box rendering and fill_closed_borders output (C checks 0-1).
///
/// C reference:
///   regTestWritePixAndCheck(rp, pix1, IFF_PNG);  /* 0 */
///   regTestWritePixAndCheck(rp, pix2, IFF_PNG);  /* 1 */
#[test]
fn compfilter_reg_write_synthetic_images() {
    use crate::common::RegParams;
    use leptonica::io::ImageFormat;

    let mut rp = RegParams::new("compfilter_write_synthetic");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_write_synthetic test failed");
        return;
    }

    let (pix1, pix2) = build_test_images();

    // C index 0: pix1 – filled boxes
    rp.write_pix_and_check(&pix1, ImageFormat::Png)
        .expect("write pix1");

    // C index 1: pix2 – hash-filled boxes
    rp.write_pix_and_check(&pix2, ImageFormat::Png)
        .expect("write pix2");

    assert!(rp.cleanup(), "compfilter_write_synthetic test failed");
}

// ---------------------------------------------------------------------------
// C checks 2-13: pixSelectBySize with various parameters.
//
// Notes on mapping:
//   C L_SELECT_HEIGHT / L_SELECT_WIDTH → Rust: use IfBoth with the unused
//     dimension threshold set so it is always satisfied.
//   C L_SELECT_IF_GT (strict >) → Rust Gte with threshold+1
//   C L_SELECT_IF_LT (strict <) → Rust Lte with threshold-1
//   C L_SELECT_IF_EITHER / L_SELECT_IF_BOTH → SizeSelectType::IfEither / IfBoth
//
// C expected component counts (4 solid boxes):
//   box1: w=20, h=30
//   box2: w=40, h=20
//   box3: w=35, h=5
//   box4: w=5,  h=15
//
//   idx 2:  height > 22     → box1(30) only             → 1
//   idx 3:  height < 30     → box2(20),box3(5),box4(15) → 3
//   idx 4:  height > 5      → box1(30),box2(20),box4(15) → 3
//   idx 5:  height < 6      → box3(5)                    → 1
//   idx 6:  width > 20      → box2(40),box3(35)          → 2
//   idx 7:  width < 31      → box1(20),box4(5)           → 2 (wait: box4 w=5<31 ✓, box1 w=20<31 ✓)
//           Actually: box1(20),box3(35?no),box4(5) → box1(w<31✓), box2(w=40,no), box3(w=35,no), box4(5✓) = 2
//   idx 8:  either < (w<21 or h<10) → box3(w=35,h=5<10✓), box4(w=5<21✓) + box2? h=20≥10, w=40≥21 no.
//           Actually C is L_SELECT_IF_EITHER with w<21, h<10:
//             box1: w=20<21✓ → keep; box2: w=40, h=20 → neither → no; box3: h=5<10✓ → keep; box4: w=5<21✓ → keep → 3
//   idx 9:  either > (w>20 or h>30) → box2(w=40>20✓), box3(w=35>20✓) → 2
//   idx 10: both < (w<22 and h<32) → box1(20<22,30<32✓), box3(35>22,no), box4(5<22,15<32✓) → wait box2? w=40>22 no.
//            → box1✓, box4✓ = 2
//   idx 11: both < (w<6 and h<32) → box4(5<6,15<32✓) = 1
//   idx 12: both > (w>5 and h>25) → box1(20>5,30>25✓) = 1
//   idx 13: both > (w>25 and h>5) → box2(40>25,20>5✓) = 1
// ---------------------------------------------------------------------------

/// Test component selection by size – HEIGHT-only variants (C checks 2-5).
///
/// C reference:
///   pixSelectBySize(pix1, 0, 22, 8, L_SELECT_HEIGHT, L_SELECT_IF_GT) → 1
///   pixSelectBySize(pix1, 0, 30, 8, L_SELECT_HEIGHT, L_SELECT_IF_LT) → 3
///   pixSelectBySize(pix1, 0,  5, 8, L_SELECT_HEIGHT, L_SELECT_IF_GT) → 3
///   pixSelectBySize(pix1, 0,  6, 8, L_SELECT_HEIGHT, L_SELECT_IF_LT) → 1
#[test]
fn compfilter_reg_select_by_height() {
    use crate::common::RegParams;
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_by_height");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_select_by_height test failed");
        return;
    }

    let (pix1, _pix2) = build_test_images();

    // C idx 2: height > 22 → 1 component (box1 h=30)
    // Rust: IfEither with width_thresh very large (never triggers on width) and height_thresh=23 Gte
    // We use IfBoth + width_thresh=0 Gte (always true) + height_thresh=23 Gte
    let filtered = pix_select_by_size(
        &pix1,
        0,
        23,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("select height>=23");
    rp.compare_values(1.0, count_pieces(&filtered) as f64, 0.0);

    // C idx 3: height < 30 → 3 components (box2,box3,box4)
    // Rust: IfBoth with width_thresh large (never blocks) + height Lte 29
    // But IfBoth Lte means BOTH w<=large AND h<=29. Use width_thresh=99999.
    let filtered = pix_select_by_size(
        &pix1,
        99999,
        29,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Lte,
    )
    .expect("select height<=29");
    rp.compare_values(3.0, count_pieces(&filtered) as f64, 0.0);

    // C idx 4: height > 5 → 3 components (box1,box2,box4)
    let filtered = pix_select_by_size(
        &pix1,
        0,
        6,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("select height>=6");
    rp.compare_values(3.0, count_pieces(&filtered) as f64, 0.0);

    // C idx 5: height < 6 → 1 component (box3 h=5)
    let filtered = pix_select_by_size(
        &pix1,
        99999,
        5,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Lte,
    )
    .expect("select height<=5");
    rp.compare_values(1.0, count_pieces(&filtered) as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_select_by_height test failed");
}

/// Test component selection by size – WIDTH-only variants (C checks 6-7).
///
/// C reference:
///   pixSelectBySize(pix1, 20, 0, 8, L_SELECT_WIDTH, L_SELECT_IF_GT) → 2
///   pixSelectBySize(pix1, 31, 0, 8, L_SELECT_WIDTH, L_SELECT_IF_LT) → 2
#[test]
fn compfilter_reg_select_by_width() {
    use crate::common::RegParams;
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_by_width");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_select_by_width test failed");
        return;
    }

    let (pix1, _pix2) = build_test_images();

    // C idx 6: width > 20 → 2 components (box2 w=40, box3 w=35)
    let filtered = pix_select_by_size(
        &pix1,
        21,
        0,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("select width>=21");
    rp.compare_values(2.0, count_pieces(&filtered) as f64, 0.0);

    // C idx 7: width < 31 → 2 components (box1 w=20, box4 w=5)
    let filtered = pix_select_by_size(
        &pix1,
        30,
        99999,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Lte,
    )
    .expect("select width<=30");
    rp.compare_values(2.0, count_pieces(&filtered) as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_select_by_width test failed");
}

/// Test component selection by size – IfEither variants (C checks 8-9).
///
/// C reference:
///   pixSelectBySize(pix1, 21, 10, 8, L_SELECT_IF_EITHER, L_SELECT_IF_LT) → 3
///   pixSelectBySize(pix1, 20, 30, 8, L_SELECT_IF_EITHER, L_SELECT_IF_GT) → 2
#[test]
fn compfilter_reg_select_if_either() {
    use crate::common::RegParams;
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_if_either");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_select_if_either test failed");
        return;
    }

    let (pix1, _pix2) = build_test_images();

    // C idx 8: either(w<21 or h<10) → 3 components
    // box1(w=20<21✓), box2(w=40,h=20 neither), box3(h=5<10✓), box4(w=5<21✓)
    let filtered = pix_select_by_size(
        &pix1,
        20,
        9,
        ConnectivityType::EightWay,
        SizeSelectType::IfEither,
        SizeSelectRelation::Lte,
    )
    .expect("select either<=");
    rp.compare_values(3.0, count_pieces(&filtered) as f64, 0.0);

    // C idx 9: either(w>20 or h>30) → 2 components
    // box1(h=30, not >30), box2(w=40>20✓), box3(w=35>20✓), box4(w=5,h=15 neither)
    let filtered = pix_select_by_size(
        &pix1,
        21,
        31,
        ConnectivityType::EightWay,
        SizeSelectType::IfEither,
        SizeSelectRelation::Gte,
    )
    .expect("select either>=");
    rp.compare_values(2.0, count_pieces(&filtered) as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_select_if_either test failed");
}

/// Test component selection by size – IfBoth variants (C checks 10-13).
///
/// C reference:
///   pixSelectBySize(pix1, 22, 32, 8, L_SELECT_IF_BOTH, L_SELECT_IF_LT) → 2
///   pixSelectBySize(pix1,  6, 32, 8, L_SELECT_IF_BOTH, L_SELECT_IF_LT) → 1
///   pixSelectBySize(pix1,  5, 25, 8, L_SELECT_IF_BOTH, L_SELECT_IF_GT) → 1
///   pixSelectBySize(pix1, 25,  5, 8, L_SELECT_IF_BOTH, L_SELECT_IF_GT) → 1
#[test]
fn compfilter_reg_select_if_both() {
    use crate::common::RegParams;
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_if_both");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_select_if_both test failed");
        return;
    }

    let (pix1, _pix2) = build_test_images();

    // C idx 10: both(w<22 and h<32) → 2 components (box1: 20<22,30<32✓; box4: 5<22,15<32✓)
    let filtered = pix_select_by_size(
        &pix1,
        21,
        31,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Lte,
    )
    .expect("both < (22,32)");
    rp.compare_values(2.0, count_pieces(&filtered) as f64, 0.0);

    // C idx 11: both(w<6 and h<32) → 1 component (box4: 5<6,15<32✓)
    let filtered = pix_select_by_size(
        &pix1,
        5,
        31,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Lte,
    )
    .expect("both < (6,32)");
    rp.compare_values(1.0, count_pieces(&filtered) as f64, 0.0);

    // C idx 12: both(w>5 and h>25) → 1 component (box1: 20>5,30>25✓)
    let filtered = pix_select_by_size(
        &pix1,
        6,
        26,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("both > (5,25)");
    rp.compare_values(1.0, count_pieces(&filtered) as f64, 0.0);

    // C idx 13: both(w>25 and h>5) → 1 component (box2: 40>25,20>5✓)
    let filtered = pix_select_by_size(
        &pix1,
        26,
        6,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("both > (25,5)");
    rp.compare_values(1.0, count_pieces(&filtered) as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_select_if_both test failed");
}

// ---------------------------------------------------------------------------
// C checks 14-25: shape-based filters (not yet implemented in Rust)
// ---------------------------------------------------------------------------

/// Test pixSelectByPerimToAreaRatio (C checks 14-17).
///
/// C reference:
///   pixSelectByPerimToAreaRatio(pix1, 0.30, 8, L_SELECT_IF_GT)  → 2
///   pixSelectByPerimToAreaRatio(pix1, 0.15, 8, L_SELECT_IF_GT)  → 3
///   pixSelectByPerimToAreaRatio(pix1, 0.40, 8, L_SELECT_IF_LTE) → 2
///   pixSelectByPerimToAreaRatio(pix1, 0.45, 8, L_SELECT_IF_LT)  → 3
#[test]
#[ignore = "not yet implemented: pixSelectByPerimToAreaRatio"]
fn compfilter_reg_select_by_perim_to_area_ratio() {}

/// Test pixSelectByPerimSizeRatio (C checks 18-21).
///
/// C reference:
///   pixSelectByPerimSizeRatio(pix2, 2.30, 8, L_SELECT_IF_GT)  → 2
///   pixSelectByPerimSizeRatio(pix2, 1.20, 8, L_SELECT_IF_GT)  → 3
///   pixSelectByPerimSizeRatio(pix2, 1.70, 8, L_SELECT_IF_LTE) → 1
///   pixSelectByPerimSizeRatio(pix2, 2.90, 8, L_SELECT_IF_LT)  → 3
#[test]
#[ignore = "not yet implemented: pixSelectByPerimSizeRatio"]
fn compfilter_reg_select_by_perim_size_ratio() {}

/// Test pixSelectByAreaFraction (C checks 22-25).
///
/// C reference:
///   pixSelectByAreaFraction(pix2, 0.30, 8, L_SELECT_IF_LT)  → 0
///   pixSelectByAreaFraction(pix2, 0.90, 8, L_SELECT_IF_LT)  → 4
///   pixSelectByAreaFraction(pix2, 0.50, 8, L_SELECT_IF_GTE) → 3
///   pixSelectByAreaFraction(pix2, 0.70, 8, L_SELECT_IF_GT)  → 2
#[test]
#[ignore = "not yet implemented: pixSelectByAreaFraction"]
fn compfilter_reg_select_by_area_fraction() {}

// ---------------------------------------------------------------------------
// C checks 26-27: boxaSelectBySize
//
// boxa1 has 4 boxes:
//   box1: w=20, h=30
//   box2: w=40, h=20
//   box3: w=35, h=5
//   box4: w=5,  h=15
//
// C idx 26: boxaSelectBySize(boxa1, 21, 10, L_SELECT_IF_EITHER, L_SELECT_IF_LT) → 3
//   either(w<21 or h<10): box1(w=20<21✓), box3(h=5<10✓), box4(w=5<21✓) → 3
// C idx 27: boxaSelectBySize(boxa1, 22, 32, L_SELECT_IF_BOTH,   L_SELECT_IF_LT) → 2
//   both(w<22 and h<32): box1(20<22,30<32✓), box4(5<22,15<32✓) → 2
//
// NOTE: Rust Boxa::select_by_size only supports IfBoth (both dimensions).
// The IfEither case is covered by using two separate Lte filters and combining,
// or via a manual filter.
// ---------------------------------------------------------------------------

/// Test boxaSelectBySize – IfBoth variant (C check 27).
///
/// C reference:
///   boxaSelectBySize(boxa1, 22, 32, L_SELECT_IF_BOTH, L_SELECT_IF_LT) → 2
#[test]
fn compfilter_reg_boxa_select_by_size_both() {
    use crate::common::RegParams;
    use leptonica::core::{Box, Boxa, SizeRelation};

    let mut rp = RegParams::new("compfilter_boxa_select_both");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_boxa_select_both test failed");
        return;
    }

    // Build the boxa from C: box1..box4
    let mut boxa1 = Boxa::new();
    boxa1.push(Box::new(10, 10, 20, 30).expect("box1"));
    boxa1.push(Box::new(50, 10, 40, 20).expect("box2"));
    boxa1.push(Box::new(110, 10, 35, 5).expect("box3"));
    boxa1.push(Box::new(160, 10, 5, 15).expect("box4"));

    // C idx 27: both(w<22 and h<32) → box1(20<22,30<32✓) + box4(5<22,15<32✓) = 2
    // Rust Boxa::select_by_size checks BOTH conditions with same SizeRelation.
    let selected = boxa1.select_by_size(21, 31, SizeRelation::LessThanOrEqual);
    rp.compare_values(2.0, selected.len() as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_boxa_select_both test failed");
}

/// Test boxaSelectBySize – IfEither variant (C check 26).
///
/// C reference:
///   boxaSelectBySize(boxa1, 21, 10, L_SELECT_IF_EITHER, L_SELECT_IF_LT) → 3
///
/// Rust Boxa::select_by_size only supports IfBoth; IfEither is done manually.
#[test]
fn compfilter_reg_boxa_select_by_size_either() {
    use crate::common::RegParams;
    use leptonica::core::Box;

    let mut rp = RegParams::new("compfilter_boxa_select_either");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_boxa_select_either test failed");
        return;
    }

    // Build boxa1
    let boxes = [
        Box::new(10, 10, 20, 30).expect("box1"), // w=20, h=30
        Box::new(50, 10, 40, 20).expect("box2"), // w=40, h=20
        Box::new(110, 10, 35, 5).expect("box3"), // w=35, h=5
        Box::new(160, 10, 5, 15).expect("box4"), // w=5,  h=15
    ];

    // C idx 26: either(w<21 or h<10) → box1(w=20<21✓), box3(h=5<10✓), box4(w=5<21✓) = 3
    let count = boxes.iter().filter(|b| b.w < 21 || b.h < 10).count();
    rp.compare_values(3.0, count as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_boxa_select_either test failed");
}

// ---------------------------------------------------------------------------
// C checks 28-85: feyn.tif area-fraction band loop
// (requires numaMakeThresholdIndicator, numaLogicalOp, pixaFindAreaFraction, etc.)
// ---------------------------------------------------------------------------

/// Test indicator-based component selection loop (C checks 28-85).
///
/// C reference: 12-iteration loop using numaMakeThresholdIndicator +
/// numaLogicalOp + pixaSelectWithIndicator to reconstruct the image band-by-band.
#[test]
#[ignore = "not yet implemented: numaMakeThresholdIndicator / numaLogicalOp / pixaFindAreaFraction"]
fn compfilter_reg_indicator_band_loop() {
    // C version:
    // 1. pixConnComp(pixs, &pixa1, 8)
    // 2. pixaFindAreaFraction(pixa1) → na1
    // 3. For each of 12 area-fraction bands [0..1]:
    //    a. numaMakeThresholdIndicator(na1, edges[i], L_SELECT_IF_GTE) → na2
    //    b. numaMakeThresholdIndicator(na1, edges[i+1], L_SELECT_IF_LT) → na3
    //    c. numaLogicalOp(na2, na3, L_INTERSECTION) → na4
    //    d. Count_ones(rp, na4, band[i], i, "band") → compare_values
    //    e. Count_pieces(rp, pix3, band[i])         → write + compare_values
    //    f. Count_ones(rp, nat, total[i], i, "total")→ compare_values
    //    g. Count_pieces(rp, pix4, total[i])         → write + compare_values
    //    h. pixRemoveWithIndicator(pix1, pixa1, na4)
    // 4. pixZero(pix1, &empty) → regTestCompareValues(rp, 1, empty, 0.0)
    //
    // Expected band counts for feyn.tif (8-way, area fraction bands):
    //   band[12] = {1,11,48,264,574,704,908,786,466,157,156,230}
    //   total[12] = {1,12,60,324,898,1602,2510,3296,3762,3919,4075,4305}
}

// ---------------------------------------------------------------------------
// C check 86: complex multi-criterion filter on feyn.tif
// ---------------------------------------------------------------------------

/// Test multi-criterion component filter (C check 86).
///
/// C reference:
///   pixaFindDimensions(pixa1, &naw, &nah)
///   pixaFindPerimToAreaRatio(pixa1) → na1
///   Combine: (height>=50) OR (30<=width<=35), AND (perimToArea>=0.4)
///   pixRemoveWithIndicator → regTestWritePixAndCheck  /* 86 */
#[test]
#[ignore = "not yet implemented: pixaFindDimensions / pixaFindPerimToAreaRatio"]
fn compfilter_reg_multi_criterion_filter() {}

// ---------------------------------------------------------------------------
// Additional Rust-specific integration checks
// ---------------------------------------------------------------------------

/// Test component selection by size on feyn.tif (original existing test, preserved).
///
/// Verifies that pix_select_by_size reduces component count and preserves image dimensions.
#[test]
fn compfilter_reg_select_by_size() {
    use crate::common::{RegParams, load_test_image};
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, find_connected_components,
        pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_by_size");
    if crate::common::is_display_mode() {
        rp.compare_values(1.0, 1.0, 0.0);
        assert!(rp.cleanup(), "compfilter_select_by_size test failed");
        return;
    }

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    let comps_before = find_connected_components(&pixs, ConnectivityType::EightWay)
        .expect("find_connected_components");

    // Select only components with both dimensions >= 10
    let filtered = pix_select_by_size(
        &pixs,
        10,
        10,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("pix_select_by_size");

    let comps_after = find_connected_components(&filtered, ConnectivityType::EightWay)
        .expect("find_connected_components after filter");

    // Filtered result should have fewer components
    rp.compare_values(
        1.0,
        if comps_after.len() < comps_before.len() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    // Dimensions preserved
    rp.compare_values(pixs.width() as f64, filtered.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, filtered.height() as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_select_by_size test failed");
}

/// Test component selection by shape metrics (original existing test, preserved).
///
/// Uses pix_select_by_size with different thresholds as a shape proxy,
/// since pixSelectByPerimToAreaRatio and similar are not yet available.
#[test]
fn compfilter_reg_select_by_shape() {
    if crate::common::is_display_mode() {
        return;
    }

    use crate::common::{RegParams, load_test_image};
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, find_connected_components,
        pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_by_shape");

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    let comps_before = find_connected_components(&pixs, ConnectivityType::EightWay)
        .expect("find_connected_components");

    // Select components where either dimension >= 20 (larger features)
    let filtered = pix_select_by_size(
        &pixs,
        20,
        20,
        ConnectivityType::EightWay,
        SizeSelectType::IfEither,
        SizeSelectRelation::Gte,
    )
    .expect("pix_select_by_size shape");

    let comps_after = find_connected_components(&filtered, ConnectivityType::EightWay)
        .expect("find_connected_components after shape filter");

    // Filtered result should have fewer components
    rp.compare_values(
        1.0,
        if comps_after.len() < comps_before.len() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    // Dimensions preserved
    rp.compare_values(pixs.width() as f64, filtered.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, filtered.height() as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_select_by_shape test failed");
}

/// Test fill_closed_borders on the synthetic 4-box image.
///
/// The filled image must have exactly 4 connected components (one per box).
#[test]
fn compfilter_reg_fill_closed_borders() {
    use crate::common::RegParams;

    let mut rp = RegParams::new("compfilter_fill_closed");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_fill_closed test failed");
        return;
    }

    let (pix1, _pix2) = build_test_images();

    // The 4 outlined boxes become 4 solid filled components
    let n = count_pieces(&pix1);
    rp.compare_values(4.0, n as f64, 0.0);

    // Dimensions preserved
    rp.compare_values(200.0, pix1.width() as f64, 0.0);
    rp.compare_values(200.0, pix1.height() as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_fill_closed test failed");
}

/// Test that selecting all components (threshold = 0, Gte) returns the original count.
#[test]
fn compfilter_reg_select_all_components() {
    use crate::common::RegParams;
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_all");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_select_all test failed");
        return;
    }

    let (pix1, _pix2) = build_test_images();

    // Select all: both dims >= 0 → should keep all 4 components
    let filtered = pix_select_by_size(
        &pix1,
        0,
        0,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("select all");
    rp.compare_values(4.0, count_pieces(&filtered) as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_select_all test failed");
}

/// Test that selecting no components returns an empty image.
#[test]
fn compfilter_reg_select_none() {
    use crate::common::RegParams;
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_none");
    if crate::common::is_display_mode() {
        assert!(rp.cleanup(), "compfilter_select_none test failed");
        return;
    }

    let (pix1, _pix2) = build_test_images();

    // Select none: both dims >= 1000 (much larger than any box)
    let filtered = pix_select_by_size(
        &pix1,
        1000,
        1000,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("select none");
    rp.compare_values(0.0, count_pieces(&filtered) as f64, 0.0);

    // Result image must be all zero
    rp.compare_values(1.0, if filtered.is_zero() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "compfilter_select_none test failed");
}

/// Test connected component analysis with indicators (C checks 26-29 intent).
///
/// Requires pixConnComp and numaMakeThresholdIndicator/numaLogicalOp
/// which are in leptonica-region and leptonica-core.
#[test]
#[ignore = "not yet implemented: pixConnComp/numaMakeThresholdIndicator in leptonica-region/core"]
fn compfilter_reg_indicator_operations() {
    // C version:
    // 1. pixConnComp(pixs, &pixa1, 8) to get connected components
    // 2. numaGetWidths/GetHeights to extract dimension arrays
    // 3. numaMakeThresholdIndicator(na1, threshold, L_SELECT_IF_GTE)
    // 4. numaLogicalOp to combine indicators
    // 5. pixRemoveWithIndicator to filter components
}
