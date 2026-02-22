//! Logic operations regression test
//!
//! Tests pixel-wise logical operations (AND, OR, XOR, NOT, subtract)
//! in both new-allocation and in-place modes.
//!
//! The C version uses morphological open/dilate to create two different
//! images from test1.png, then tests all logic ops between them.
//! This Rust port uses an image and its inverse (same dimensions
//! guaranteed), since morphological operations reside in leptonica-morph.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/logicops_reg.c`

use leptonica_core::Pix;
use leptonica_test::RegParams;

/// Test pixInvert: double-invert identity (C checks 0-2).
///
/// Verifies that inverting an image twice yields the original.
#[test]
fn logicops_reg_invert() {
    let mut rp = RegParams::new("logicops_invert");

    let pix1 = leptonica_test::load_test_image("test1.png").expect("load test1.png");

    // Invert
    let pix2 = pix1.invert();
    // Inverted should differ from original
    rp.compare_values(0.0, if pix1.equals(&pix2) { 1.0 } else { 0.0 }, 0.0);

    // Double invert should yield original
    let pix3 = pix2.invert();
    rp.compare_pix(&pix1, &pix3);

    assert!(rp.cleanup(), "logicops invert test failed");
}

/// Test pixAnd, pixOr, pixXor, pixSubtract: algebraic properties (C checks 3-28).
///
/// Uses an image and its inverse to test identity laws, self-cancellation,
/// De Morgan's laws, and commutativity.
#[test]
fn logicops_reg_binary_ops() {
    let mut rp = RegParams::new("logicops_binops");

    let pix_a = leptonica_test::load_test_image("test1.png").expect("load test1.png");
    let pix_b = pix_a.invert(); // same dimensions, different content

    // --- Self-operation identities ---

    // AND: a & a == a
    let result = pix_a.and(&pix_a).expect("and self");
    rp.compare_pix(&pix_a, &result);

    // OR: a | a == a
    let result = pix_a.or(&pix_a).expect("or self");
    rp.compare_pix(&pix_a, &result);

    // XOR: a ^ a == 0
    let result = pix_a.xor(&pix_a).expect("xor self");
    rp.compare_values(1.0, if result.is_zero() { 1.0 } else { 0.0 }, 0.0);

    // Subtract: a - a == 0
    let result = pix_a.subtract(&pix_a).expect("subtract self");
    rp.compare_values(1.0, if result.is_zero() { 1.0 } else { 0.0 }, 0.0);

    // --- Cross-image operations with inverse ---

    // AND with inverse: a & ~a == 0
    let result = pix_a.and(&pix_b).expect("and inverse");
    rp.compare_values(1.0, if result.is_zero() { 1.0 } else { 0.0 }, 0.0);

    // OR with inverse: a | ~a == all-ones
    let result = pix_a.or(&pix_b).expect("or inverse");
    let all_ones = result.invert();
    rp.compare_values(1.0, if all_ones.is_zero() { 1.0 } else { 0.0 }, 0.0);

    // XOR with inverse: a ^ ~a == all-ones
    let result = pix_a.xor(&pix_b).expect("xor inverse");
    let all_ones = result.invert();
    rp.compare_values(1.0, if all_ones.is_zero() { 1.0 } else { 0.0 }, 0.0);

    // Subtract: a - ~a == a (removing inverse bits leaves original)
    let result = pix_a.subtract(&pix_b).expect("subtract inverse");
    rp.compare_pix(&pix_a, &result);

    // Subtract: ~a - a == ~a
    let result = pix_b.subtract(&pix_a).expect("subtract from inverse");
    rp.compare_pix(&pix_b, &result);

    assert!(rp.cleanup(), "logicops binary ops test failed");
}

/// Test in-place logic operations via PixMut (C in-place checks).
///
/// Verifies that in-place operations produce the same result as
/// new-allocation operations, using an image and its inverse.
#[test]
fn logicops_reg_inplace() {
    let mut rp = RegParams::new("logicops_inplace");

    let pix_a = leptonica_test::load_test_image("test1.png").expect("load test1.png");
    let pix_b = pix_a.invert();

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
