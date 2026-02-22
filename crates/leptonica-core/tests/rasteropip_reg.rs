//! In-place rasterop regression test
//!
//! Tests in-place raster operations where source and destination are
//! the same image but non-overlapping regions.
//!
//! The C version copies pixel data column-by-column and row-by-row
//! using general pixRasterop, then tests mirrored border addition.
//! This Rust port tests the available in-place primitives:
//! rasterop_vip and rasterop_hip with various parameters.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/rasteropip_reg.c`

use leptonica_core::InColor;
use leptonica_test::RegParams;

/// Test in-place vertical and horizontal shifts produce consistent results.
///
/// The C version copies columns one at a time (check 0) and verifies
/// column-copy matches row-copy. This port applies rasterop_vip and
/// rasterop_hip to the same image and verifies golden file output.
#[test]
#[ignore = "not yet implemented: general region-based pixRasterop not available"]
fn rasteropip_reg_shift_consistency() {
    let mut rp = RegParams::new("rasteropip_shift");

    let pix1 = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");

    // Vertical shift: shift a wide band down
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    pm.rasterop_vip(0, 200, 40, InColor::White);
    let result_v: leptonica_core::Pix = pm.into();
    rp.write_pix_and_check(&result_v, leptonica_core::ImageFormat::Png)
        .unwrap();

    // Horizontal shift: shift a tall band right
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    pm.rasterop_hip(0, 200, 40, InColor::White);
    let result_h: leptonica_core::Pix = pm.into();
    rp.write_pix_and_check(&result_h, leptonica_core::ImageFormat::Png)
        .unwrap();

    assert!(rp.cleanup(), "rasteropip shift consistency test failed");
}

/// Test mirrored border operations (C check 1).
///
/// Requires pixRemoveBorder and pixAddMirroredBorder which are not
/// available in leptonica-core.
#[test]
#[ignore = "not yet implemented: pixRemoveBorder/pixAddMirroredBorder not available"]
fn rasteropip_reg_mirrored_border() {
    // The C version:
    // 1. Reads test8.jpg
    // 2. pixRemoveBorder(pixs, 40)
    // 3. pixAddMirroredBorder(pixt, 40, 40, 40, 40)
    // 4. Writes golden file
}
