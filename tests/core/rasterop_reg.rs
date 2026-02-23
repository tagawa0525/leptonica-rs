//! Rasterop regression test
//!
//! Tests general 2-image raster operations. The C version validates
//! dilation equivalence between SEL-based and manual pixRasterop approaches
//! across 63 structuring element sizes.
//!
//! This Rust port tests the available rasterop primitives: rasterop_vip
//! (vertical in-place shift), rasterop_hip (horizontal in-place shift),
//! translate, and general ROP operations via algebraic properties.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/rasterop_reg.c`

use crate::common::RegParams;
use leptonica::{InColor, Pix, RopOp};

/// Test rasterop_vip: vertical in-place shift (zero shift is identity).
#[test]
fn rasterop_reg_vip() {
    let mut rp = RegParams::new("rasterop_vip");

    let pix1 = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");

    // Zero shift should be identity
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    pm.rasterop_vip(0, 100, 0, InColor::White);
    let result: Pix = pm.into();
    rp.compare_pix(&pix1, &result);

    // Full-width zero shift should also be identity
    let w = pix1.width() as i32;
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    pm.rasterop_vip(0, w, 0, InColor::White);
    let result: Pix = pm.into();
    rp.compare_pix(&pix1, &result);

    assert!(rp.cleanup(), "rasterop vip test failed");
}

/// Test rasterop_hip: horizontal in-place shift (zero shift is identity).
#[test]
fn rasterop_reg_hip() {
    let mut rp = RegParams::new("rasterop_hip");

    let pix1 = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");

    // Zero shift should be identity
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    pm.rasterop_hip(0, 100, 0, InColor::White);
    let result: Pix = pm.into();
    rp.compare_pix(&pix1, &result);

    // Full-height zero shift should also be identity
    let h = pix1.height() as i32;
    let mut pm = pix1.deep_clone().try_into_mut().expect("into_mut");
    pm.rasterop_hip(0, h, 0, InColor::White);
    let result: Pix = pm.into();
    rp.compare_pix(&pix1, &result);

    assert!(rp.cleanup(), "rasterop hip test failed");
}

/// Test translate: zero translation is identity.
#[test]
fn rasterop_reg_translate() {
    let mut rp = RegParams::new("rasterop_translate");

    let pix1 = crate::common::load_test_image("test1.png").expect("load test1.png");

    // Zero translate should be identity
    let pix2 = pix1.translate(0, 0, InColor::White);
    rp.compare_pix(&pix1, &pix2);

    // Translate should preserve dimensions
    let pix3 = pix1.translate(30, 20, InColor::White);
    rp.compare_values(pix1.width() as f64, pix3.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, pix3.height() as f64, 0.0);

    assert!(rp.cleanup(), "rasterop translate test failed");
}

/// Test general ROP operations: algebraic identities (C checks 0-62).
///
/// The C version tests pixRasterop region-based operations with 63
/// structuring element sizes. This port tests basic rop() identities.
#[test]
fn rasterop_reg_general() {
    let mut rp = RegParams::new("rasterop_general");

    let pix_a = crate::common::load_test_image("test1.png").expect("load test1.png");

    // Src: rop(a, a, Src) == a
    let result = pix_a.rop(&pix_a, RopOp::Src).expect("rop src");
    rp.compare_pix(&pix_a, &result);

    // Or: rop(a, a, Or) == a
    let result = pix_a.rop(&pix_a, RopOp::Or).expect("rop or");
    rp.compare_pix(&pix_a, &result);

    // And: rop(a, a, And) == a
    let result = pix_a.rop(&pix_a, RopOp::And).expect("rop and");
    rp.compare_pix(&pix_a, &result);

    // Clear: rop(a, a, Clear) == 0
    let result = pix_a.rop(&pix_a, RopOp::Clear).expect("rop clear");
    rp.compare_values(1.0, if result.is_zero() { 1.0 } else { 0.0 }, 0.0);

    // Xor: rop(a, a, Xor) == 0
    let result = pix_a.rop(&pix_a, RopOp::Xor).expect("rop xor");
    rp.compare_values(1.0, if result.is_zero() { 1.0 } else { 0.0 }, 0.0);

    // NotSrc: rop(a, a, NotSrc) == ~a
    let result = pix_a.rop(&pix_a, RopOp::NotSrc).expect("rop notsrc");
    let expected = pix_a.invert();
    rp.compare_pix(&expected, &result);

    assert!(rp.cleanup(), "rasterop general test failed");
}

/// Test dilation equivalence (C checks 0-62).
///
/// Requires general region-based pixRasterop and morphological operations.
#[test]
#[ignore = "not yet implemented: requires general region-based pixRasterop and morph ops"]
fn rasterop_reg_dilation_equivalence() {
    // C version:
    // For 63 structuring element sizes (width × height):
    //   1. pixDilate with SEL
    //   2. Manual dilation via pixRasterop per foreground pixel
    //   3. pixEqual to verify equivalence
}
