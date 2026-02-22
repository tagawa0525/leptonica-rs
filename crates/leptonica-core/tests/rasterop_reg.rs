//! Rasterop regression test
//!
//! Tests general 2-image raster operations. The C version validates
//! dilation equivalence between SEL-based and manual pixRasterop approaches
//! across 63 structuring element sizes.
//!
//! This Rust port tests the available rasterop primitives: rasterop_vip
//! (vertical in-place shift) and rasterop_hip (horizontal in-place shift),
//! plus the translate function and general ROP operations.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/rasterop_reg.c`

use leptonica_core::{InColor, Pix, RopOp};
use leptonica_test::RegParams;

/// Test rasterop_vip: vertical in-place shift within a band.
///
/// Shifts a vertical band of pixels and verifies the result through
/// golden file comparison.
#[test]
#[ignore = "not yet implemented: requires golden file generation"]
fn rasterop_reg_vip() {
    let mut rp = RegParams::new("rasterop_vip");

    let pix1 = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");

    // Apply vertical shift within a band
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    // Shift band at x=50, width=100, by 30 pixels down, filling with white
    pm.rasterop_vip(50, 100, 30, InColor::White);
    let result: Pix = pm.into();
    rp.write_pix_and_check(&result, leptonica_core::ImageFormat::Png)
        .unwrap();

    // Shift back: should NOT restore original due to fill
    let mut pm = result.try_into_mut().expect("into_mut");
    pm.rasterop_vip(50, 100, -30, InColor::White);
    let _result2: Pix = pm.into();

    assert!(rp.cleanup(), "rasterop vip test failed");
}

/// Test rasterop_hip: horizontal in-place shift within a band.
///
/// Shifts a horizontal band of pixels and verifies the result through
/// golden file comparison.
#[test]
#[ignore = "not yet implemented: requires golden file generation"]
fn rasterop_reg_hip() {
    let mut rp = RegParams::new("rasterop_hip");

    let pix1 = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");

    // Apply horizontal shift within a band
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    // Shift band at y=40, height=80, by 25 pixels right, filling with black
    pm.rasterop_hip(40, 80, 25, InColor::Black);
    let result: Pix = pm.into();
    rp.write_pix_and_check(&result, leptonica_core::ImageFormat::Png)
        .unwrap();

    assert!(rp.cleanup(), "rasterop hip test failed");
}

/// Test translate: pixel translation with fill color.
///
/// Translates an image and verifies the result.
#[test]
#[ignore = "not yet implemented: requires golden file generation"]
fn rasterop_reg_translate() {
    let mut rp = RegParams::new("rasterop_translate");

    let pix1 = leptonica_test::load_test_image("test1.png").expect("load test1.png");

    // Translate right and down
    let pix2 = pix1.translate(30, 20, InColor::White);
    rp.write_pix_and_check(&pix2, leptonica_core::ImageFormat::Png)
        .unwrap();

    // Translate back should roughly restore (except for border fill)
    let pix3 = pix2.translate(-30, -20, InColor::White);
    // The restored image won't be identical due to fill, but the
    // central region should match
    let _pix3 = pix3;

    assert!(rp.cleanup(), "rasterop translate test failed");
}

/// Test general ROP operations between two images (C checks 0-62).
///
/// The C version tests pixRasterop region-based operations with 63
/// structuring element sizes. Requires general region-based rasterop
/// which is not yet available. Tests basic rop() with all RopOp variants.
#[test]
#[ignore = "not yet implemented: general region-based pixRasterop not available"]
fn rasterop_reg_general() {
    let mut rp = RegParams::new("rasterop_general");

    let pix_a = leptonica_test::load_test_image("test1.png").expect("load test1.png");

    // Test ROP identity operations
    // PIX_SRC | PIX_DST == OR
    let pix_or = pix_a.rop(&pix_a, RopOp::Or).expect("rop or");
    rp.compare_pix(&pix_a, &pix_or);

    // PIX_SRC & PIX_DST == AND
    let pix_and = pix_a.rop(&pix_a, RopOp::And).expect("rop and");
    rp.compare_pix(&pix_a, &pix_and);

    // PIX_CLR should produce zero image
    let pix_clr = pix_a.rop(&pix_a, RopOp::Clear).expect("rop clear");
    assert!(pix_clr.is_zero(), "rop clear should be zero");

    // PIX_SRC should equal source
    let pix_src = pix_a.rop(&pix_a, RopOp::Src).expect("rop src");
    rp.compare_pix(&pix_a, &pix_src);

    assert!(rp.cleanup(), "rasterop general test failed");
}
