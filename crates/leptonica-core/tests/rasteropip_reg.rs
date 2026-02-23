//! In-place rasterop regression test
//!
//! Tests in-place raster operations where source and destination are
//! the same image but non-overlapping regions.
//!
//! The C version copies pixel data column-by-column and row-by-row
//! using general pixRasterop, then tests mirrored border addition.
//! This Rust port tests the available in-place primitives:
//! rasterop_vip and rasterop_hip with various shift amounts.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/rasteropip_reg.c`

use leptonica_core::{InColor, Pix};
use leptonica_test::RegParams;

/// Test in-place vertical shift: opposite shifts within separate bands.
///
/// Applies rasterop_vip with positive and negative shifts on different
/// bands, verifying the operation modifies only the targeted band.
#[test]
fn rasteropip_reg_vip_shifts() {
    let mut rp = RegParams::new("rasteropip_vip");

    let pix1 = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix1.width() as i32;

    // Shift a narrow band, rest should be unchanged
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    pm.rasterop_vip(w / 4, w / 2, 20, InColor::White);
    let shifted: Pix = pm.into();

    // The shifted image should differ from original
    rp.compare_values(0.0, if pix1.equals(&shifted) { 1.0 } else { 0.0 }, 0.0);

    // Dimensions should be preserved
    rp.compare_values(pix1.width() as f64, shifted.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, shifted.height() as f64, 0.0);

    assert!(rp.cleanup(), "rasteropip vip shifts test failed");
}

/// Test in-place horizontal shift: opposite shifts within separate bands.
#[test]
fn rasteropip_reg_hip_shifts() {
    let mut rp = RegParams::new("rasteropip_hip");

    let pix1 = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");
    let h = pix1.height() as i32;

    // Shift a band, rest should be unchanged
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    pm.rasterop_hip(h / 4, h / 2, 15, InColor::Black);
    let shifted: Pix = pm.into();

    // Should differ from original
    rp.compare_values(0.0, if pix1.equals(&shifted) { 1.0 } else { 0.0 }, 0.0);

    // Dimensions should be preserved
    rp.compare_values(pix1.width() as f64, shifted.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, shifted.height() as f64, 0.0);

    assert!(rp.cleanup(), "rasteropip hip shifts test failed");
}

/// Test column-by-column and row-by-row in-place copy (C check 0).
///
/// Requires general region-based pixRasterop which is not available.
#[test]
#[ignore = "not yet implemented: general region-based pixRasterop not available"]
fn rasteropip_reg_copy_consistency() {
    // C version:
    // 1. Column-by-column copy from x=250+j to x=20+j (j=0..199)
    // 2. Row-by-row copy from x=250 to x=20 (i=0..249)
    // 3. Compare results: should be identical
}

/// Test mirrored border operations (C check 1).
///
/// Requires pixRemoveBorder and pixAddMirroredBorder.
#[test]
#[ignore = "not yet implemented: pixRemoveBorder/pixAddMirroredBorder not available"]
fn rasteropip_reg_mirrored_border() {
    // C version:
    // 1. Reads test8.jpg
    // 2. pixRemoveBorder(pixs, 40)
    // 3. pixAddMirroredBorder(pixt, 40, 40, 40, 40)
    // 4. Writes golden file
}
