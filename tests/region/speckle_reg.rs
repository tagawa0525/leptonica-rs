//! Speckle removal regression test
//!
//! Tests region-based speckle removal using connected component analysis.
//! The C version uses pixBackgroundNormFlex, pixHMT, pixDilate, and pixSubtract
//! to remove speckle noise from a document image. Since those functions live in
//! leptonica-filter and leptonica-morph, this Rust test instead exercises
//! the leptonica-region APIs that are relevant to speckle analysis:
//! clear_border, pix_count_components, and pix_select_by_size.
//!
//! Partial migration: clear_border, pix_count_components, find_connected_components,
//! and pix_select_by_size are tested using feyn.tif (already 1bpp).
//! The full speckle pipeline (pixHMT, pixDilate, pixSubtract) requires
//! leptonica-morph and is not available here.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/speckle_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::region::{
    ConnectivityType, SizeSelectRelation, SizeSelectType, clear_border, find_connected_components,
    pix_count_components, pix_select_by_size,
};

/// Test clear_border on a binary image (border noise removal).
///
/// Clears all foreground pixels connected to the border.
#[test]
fn speckle_reg_clear_border() {
    let mut rp = RegParams::new("speckle_border");

    // Use feyn.tif as a binary test image (already 1bpp)
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let region = pix.clip_rectangle(383, 338, 400, 300).expect("clip region");
    let w = region.width();
    let h = region.height();

    // clear_border removes foreground pixels connected to the border
    let cleared = clear_border(&region, ConnectivityType::FourWay).expect("clear_border 4-way");
    rp.compare_values(w as f64, cleared.width() as f64, 0.0);
    rp.compare_values(h as f64, cleared.height() as f64, 0.0);
    assert_eq!(cleared.depth(), PixelDepth::Bit1);

    // 8-way connectivity clears more border pixels
    let cleared8 = clear_border(&region, ConnectivityType::EightWay).expect("clear_border 8-way");
    rp.compare_values(w as f64, cleared8.width() as f64, 0.0);

    assert!(rp.cleanup(), "speckle clear_border test failed");
}

/// Test pix_count_components and find_connected_components on a binary image.
///
/// Counts and finds connected components in feyn.tif (already 1bpp).
#[test]
fn speckle_reg_count_components() {
    let mut rp = RegParams::new("speckle_count");

    // feyn.tif is already 1bpp — no threshold needed
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let binary = pix.clip_rectangle(383, 338, 400, 300).expect("clip region");
    assert_eq!(binary.depth(), PixelDepth::Bit1);

    // Count connected components
    let count =
        pix_count_components(&binary, ConnectivityType::EightWay).expect("pix_count_components");
    rp.compare_values(1.0, if count > 0 { 1.0 } else { 0.0 }, 0.0);

    // Find connected components with bounding boxes
    let components = find_connected_components(&binary, ConnectivityType::EightWay)
        .expect("find_connected_components");
    rp.compare_values(count as f64, components.len() as f64, 0.0);

    assert!(rp.cleanup(), "speckle count_components test failed");
}

/// Test pix_select_by_size to remove small speckle components (C: speckle removal).
///
/// Removes connected components smaller than a given size threshold to
/// simulate speckle noise removal.
#[test]
fn speckle_reg_select_by_size() {
    let mut rp = RegParams::new("speckle_size");

    // feyn.tif is already 1bpp
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let binary = pix.clip_rectangle(383, 338, 400, 300).expect("clip region");
    let w = binary.width();
    let h = binary.height();

    // Count components before filtering
    let count_before =
        pix_count_components(&binary, ConnectivityType::FourWay).expect("count before");

    // Remove components smaller than 3x3 (speckle removal)
    // C: sel1 removes components up to 2x2; we use size threshold 3x3
    let filtered = pix_select_by_size(
        &binary,
        3,
        3,
        ConnectivityType::FourWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("pix_select_by_size");
    rp.compare_values(w as f64, filtered.width() as f64, 0.0);
    rp.compare_values(h as f64, filtered.height() as f64, 0.0);
    assert_eq!(filtered.depth(), PixelDepth::Bit1);

    // After filtering, there should be fewer or equal components
    let count_after =
        pix_count_components(&filtered, ConnectivityType::FourWay).expect("count after");
    rp.compare_values(
        1.0,
        if count_after <= count_before {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "speckle select_by_size test failed");
}

/// Test full speckle removal pipeline (C checks 0-8).
///
/// Requires pixBackgroundNormFlex (leptonica-filter), pixGammaTRCMasked,
/// pixHMT (leptonica-morph), pixDilate, and pixSubtract which are in
/// different crates and cannot be used from leptonica-region tests.
#[test]
#[ignore = "not yet implemented: speckle pipeline requires leptonica-morph/filter functions"]
fn speckle_reg_full_pipeline() {
    // C version:
    // pix1 = pixBackgroundNormFlex(pixs, 7, 7, 1, 1, 10);
    // pix2 = pixGammaTRCMasked(NULL, pix1, NULL, 1.0, 100, 175);
    // pix3 = pixThresholdToBinary(pix2, 180);
    // sel1 = selCreateFromString(selstr2, 4, 4, "speckle2");
    // pix4 = pixHMT(NULL, pix3, sel1);
    // sel2 = selCreateBrick(2, 2, 0, 0, SEL_HIT);
    // pix5 = pixDilate(NULL, pix4, sel2);
    // pix6 = pixSubtract(NULL, pix3, pix5);
}
