//! Logic operations regression test
//!
//! Tests pixel-wise logical operations (AND, OR, XOR, NOT, subtract)
//! in both new-allocation and in-place modes.
//!
//! The C version uses morphological open/dilate to create two different
//! images, then tests all logic ops between them. This Rust port uses
//! two pre-existing test images instead, since morphological operations
//! reside in leptonica-morph (not available from leptonica-core).
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/logicops_reg.c`

use leptonica_core::{ImageFormat, Pix};
use leptonica_test::RegParams;

/// Test pixInvert: new allocation (C check 0) and consistency (C checks 1-2).
///
/// Verifies that inverting an image twice yields the original,
/// and that inverted serialization is stable.
#[test]
#[ignore = "not yet implemented: requires morph ops or golden file comparison"]
fn logicops_reg_invert() {
    let mut rp = RegParams::new("logicops_invert");

    let pix1 = leptonica_test::load_test_image("test1.png").expect("load test1.png");

    // Invert and write golden
    let pix2 = pix1.invert();
    rp.write_pix_and_check(&pix2, ImageFormat::Png).unwrap();

    // Double invert should yield original
    let pix3 = pix2.invert();
    rp.compare_pix(&pix1, &pix3);

    assert!(rp.cleanup(), "logicops invert test failed");
}

/// Test pixAnd, pixOr, pixXor, pixSubtract: new allocation (C checks 3-28).
///
/// Uses two different test images. Verifies basic algebraic properties
/// of logic operations (commutativity, identity, self-cancellation).
#[test]
#[ignore = "not yet implemented: requires morph ops or golden file comparison"]
fn logicops_reg_binary_ops() {
    let mut rp = RegParams::new("logicops_binops");

    let pix_a = leptonica_test::load_test_image("test1.png").expect("load test1.png");
    let pix_b = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");

    // AND: a & a == a
    let pix_and_self = pix_a.and(&pix_a).expect("and self");
    rp.compare_pix(&pix_a, &pix_and_self);

    // OR: a | a == a
    let pix_or_self = pix_a.or(&pix_a).expect("or self");
    rp.compare_pix(&pix_a, &pix_or_self);

    // XOR: a ^ a == 0 (empty)
    let pix_xor_self = pix_a.xor(&pix_a).expect("xor self");
    assert!(pix_xor_self.is_zero(), "xor self should be zero");

    // Subtract: a - a == 0 (empty)
    let pix_sub_self = pix_a.subtract(&pix_a).expect("subtract self");
    assert!(pix_sub_self.is_zero(), "subtract self should be zero");

    // AND golden
    let pix_and = pix_a.and(&pix_b).expect("and");
    rp.write_pix_and_check(&pix_and, ImageFormat::Png).unwrap();

    // OR golden
    let pix_or = pix_a.or(&pix_b).expect("or");
    rp.write_pix_and_check(&pix_or, ImageFormat::Png).unwrap();

    // XOR golden
    let pix_xor = pix_a.xor(&pix_b).expect("xor");
    rp.write_pix_and_check(&pix_xor, ImageFormat::Png).unwrap();

    // Subtract golden
    let pix_sub = pix_a.subtract(&pix_b).expect("subtract");
    rp.write_pix_and_check(&pix_sub, ImageFormat::Png).unwrap();

    assert!(rp.cleanup(), "logicops binary ops test failed");
}

/// Test in-place logic operations via PixMut (C in-place checks).
///
/// Verifies that in-place operations produce the same result as
/// new-allocation operations.
#[test]
#[ignore = "not yet implemented: requires morph ops or golden file comparison"]
fn logicops_reg_inplace() {
    let mut rp = RegParams::new("logicops_inplace");

    let pix_a = leptonica_test::load_test_image("test1.png").expect("load test1.png");
    let pix_b = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");

    // AND: new-alloc vs in-place
    let expected = pix_a.and(&pix_b).expect("and");
    let mut pm = pix_a.deep_clone().try_into_mut().expect("into_mut");
    pm.and_inplace(&pix_b).expect("and_inplace");
    let actual: Pix = pm.into();
    rp.compare_pix(&expected, &actual);

    // OR: new-alloc vs in-place
    let expected = pix_a.or(&pix_b).expect("or");
    let mut pm = pix_a.deep_clone().try_into_mut().expect("into_mut");
    pm.or_inplace(&pix_b).expect("or_inplace");
    let actual: Pix = pm.into();
    rp.compare_pix(&expected, &actual);

    // XOR: new-alloc vs in-place
    let expected = pix_a.xor(&pix_b).expect("xor");
    let mut pm = pix_a.deep_clone().try_into_mut().expect("into_mut");
    pm.xor_inplace(&pix_b).expect("xor_inplace");
    let actual: Pix = pm.into();
    rp.compare_pix(&expected, &actual);

    // Invert: new-alloc vs in-place
    let expected = pix_a.invert();
    let mut pm = pix_a.deep_clone().try_into_mut().expect("into_mut");
    pm.invert_inplace();
    let actual: Pix = pm.into();
    rp.compare_pix(&expected, &actual);

    assert!(rp.cleanup(), "logicops in-place test failed");
}
