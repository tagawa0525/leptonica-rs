//! Box transform regression test
//!
//! Tests ordered box transformations (translation, scaling, rotation)
//! and composite affine transforms on Boxa. The C version also tests
//! hash rendering for visual verification.
//!
//! Partial migration: boxaTransformOrdered, pixConnComp, and hash
//! rendering are not available. Tests Boxa::translate, scale, rotate,
//! and affine_transform with consistency checks.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/xformbox_reg.c`

use leptonica_core::{Box as LeptBox, Boxa};
use leptonica_test::RegParams;
use leptonica_transform::AffineMatrix;

/// Test Boxa translate, scale, rotate individually (C partial check 5).
///
/// Creates a Boxa, applies individual transforms, and verifies
/// the resulting box coordinates are correct.
#[test]
#[ignore = "not yet implemented"]
fn xformbox_reg_individual_transforms() {
    let mut rp = RegParams::new("xformbox_indiv");

    // Create test boxes
    let mut boxa = Boxa::new();
    boxa.push(LeptBox::new(100, 100, 50, 30).expect("box1"));
    boxa.push(LeptBox::new(200, 150, 60, 40).expect("box2"));
    boxa.push(LeptBox::new(300, 200, 70, 50).expect("box3"));

    let n = boxa.len();
    rp.compare_values(3.0, n as f64, 0.0);

    // Translate by (44, 39)
    let translated = boxa.translate(44.0, 39.0);
    rp.compare_values(n as f64, translated.len() as f64, 0.0);
    let tb = translated.get(0).expect("get translated box 0");
    rp.compare_values(144.0, tb.x as f64, 1.0);
    rp.compare_values(139.0, tb.y as f64, 1.0);
    rp.compare_values(50.0, tb.w as f64, 1.0);
    rp.compare_values(30.0, tb.h as f64, 1.0);

    // Scale by (0.83, 0.78)
    let scaled = boxa.scale(0.83, 0.78);
    rp.compare_values(n as f64, scaled.len() as f64, 0.0);
    let sb = scaled.get(0).expect("get scaled box 0");
    // Scaled coordinates: x=100*0.83=83, y=100*0.78=78, w=50*0.83≈41.5, h=30*0.78≈23.4
    rp.compare_values(83.0, sb.x as f64, 1.0);
    rp.compare_values(78.0, sb.y as f64, 1.0);

    assert!(rp.cleanup(), "xformbox individual transforms test failed");
}

/// Test Boxa affine transform consistency (C check 5 composite part).
///
/// Verifies that composing translation + scaling as an affine matrix
/// produces the same result as sequential individual operations.
#[test]
#[ignore = "not yet implemented"]
fn xformbox_reg_affine_consistency() {
    let mut rp = RegParams::new("xformbox_affine");

    // Create test boxes
    let mut boxa = Boxa::new();
    boxa.push(LeptBox::new(100, 100, 50, 30).expect("box1"));
    boxa.push(LeptBox::new(200, 150, 60, 40).expect("box2"));

    // Method (a): Sequential translate then scale
    let translated = boxa.translate(44.0, 39.0);
    let sequential = translated.scale(0.83, 0.78);

    // Method (b): Composite affine matrix (translate then scale)
    let mat_translate = AffineMatrix::translation(44.0, 39.0);
    let mat_scale = AffineMatrix::scale(0.83, 0.78);
    let composed = mat_scale.compose(&mat_translate);
    let composite = leptonica_transform::boxa_affine_transform(&boxa, &composed);

    // Compare box 0 from both methods (allow small rounding differences)
    let seq_b = sequential.get(0).expect("sequential box 0");
    let comp_b = composite.get(0).expect("composite box 0");
    rp.compare_values(seq_b.x as f64, comp_b.x as f64, 2.0);
    rp.compare_values(seq_b.y as f64, comp_b.y as f64, 2.0);
    rp.compare_values(seq_b.w as f64, comp_b.w as f64, 2.0);
    rp.compare_values(seq_b.h as f64, comp_b.h as f64, 2.0);

    // Compare box 1
    let seq_b1 = sequential.get(1).expect("sequential box 1");
    let comp_b1 = composite.get(1).expect("composite box 1");
    rp.compare_values(seq_b1.x as f64, comp_b1.x as f64, 2.0);
    rp.compare_values(seq_b1.y as f64, comp_b1.y as f64, 2.0);

    assert!(rp.cleanup(), "xformbox affine consistency test failed");
}

/// Test Boxa rotation (C check 3 rotation part).
///
/// Rotates boxes by a small angle and verifies the result is reasonable.
#[test]
#[ignore = "not yet implemented"]
fn xformbox_reg_rotation() {
    let mut rp = RegParams::new("xformbox_rotate");

    let mut boxa = Boxa::new();
    boxa.push(LeptBox::new(100, 50, 80, 60).expect("box1"));
    boxa.push(LeptBox::new(250, 100, 90, 70).expect("box2"));

    let n = boxa.len();

    // Rotate by 0.10 radians about (200, 150)
    let rotated = boxa.rotate(200.0, 150.0, 0.10);
    rp.compare_values(n as f64, rotated.len() as f64, 0.0);

    // After small rotation, boxes should still have positive dimensions
    let rb = rotated.get(0).expect("get rotated box 0");
    rp.compare_values(1.0, if rb.w > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if rb.h > 0 { 1.0 } else { 0.0 }, 0.0);

    // Identity rotation (angle=0) should preserve coordinates
    let identity = boxa.rotate(200.0, 150.0, 0.0);
    let ib = identity.get(0).expect("identity box 0");
    let ob = boxa.get(0).expect("original box 0");
    rp.compare_values(ob.x as f64, ib.x as f64, 1.0);
    rp.compare_values(ob.y as f64, ib.y as f64, 1.0);
    rp.compare_values(ob.w as f64, ib.w as f64, 1.0);
    rp.compare_values(ob.h as f64, ib.h as f64, 1.0);

    assert!(rp.cleanup(), "xformbox rotation test failed");
}

/// Test boxaTransformOrdered (C checks 4-5 ordered transforms).
///
/// Requires boxaTransformOrdered which is not available.
#[test]
#[ignore = "not yet implemented: boxaTransformOrdered not available"]
fn xformbox_reg_ordered() {
    // C version:
    // 1. boxaTransformOrdered with all 6 orderings: TR_SC_RO, TR_RO_SC,
    //    SC_TR_RO, RO_TR_SC, RO_SC_TR, SC_RO_TR
    // 2. Verify that for translation-only (scale=1.0), different orderings
    //    produce identical results
    // 3. Verify hash rendering of transformed boxes
}

/// Test hash box rendering (C checks 0-2).
///
/// Requires pixRenderHashBox, pixRenderHashBoxArb, pixRenderHashBoxBlend
/// and pixConnComp which are not available in leptonica-transform.
#[test]
#[ignore = "not yet implemented: hash rendering and pixConnComp not available"]
fn xformbox_reg_hash_rendering() {
    // C version:
    // 1. pixConnComp() for extracting component boxes
    // 2. pixRenderHashBox() in binary mode
    // 3. pixRenderHashBoxArb() in grayscale with arbitrary colors
    // 4. pixRenderHashBoxBlend() in 32bpp with alpha blending
}
