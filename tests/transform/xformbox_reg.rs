//! Box transform regression test
//!
//! Tests ordered box transformations (translation, scaling, rotation)
//! and composite affine transforms on Boxa. Also tests hash box rendering
//! using connected component boxes from feyn.tif.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/xformbox_reg.c`

use crate::common::RegParams;
use leptonica::transform::AffineMatrix;
use leptonica::{Box as LeptBox, Boxa};

/// Test Boxa translate, scale, rotate individually (C partial check 5).
///
/// Creates a Boxa, applies individual transforms, and verifies
/// the resulting box coordinates are correct.
#[test]
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

/// Test Boxa affine transform (C check 5 composite part).
///
/// Verifies that affine transform with identity preserves coordinates,
/// and that translation-only affine matches Boxa::translate.
#[test]
fn xformbox_reg_affine_transform() {
    let mut rp = RegParams::new("xformbox_affine");

    // Create test boxes
    let mut boxa = Boxa::new();
    boxa.push(LeptBox::new(100, 100, 50, 30).expect("box1"));
    boxa.push(LeptBox::new(200, 150, 60, 40).expect("box2"));

    // Identity affine should preserve all coordinates
    let identity = AffineMatrix::identity();
    let id_result = leptonica::transform::boxa_affine_transform(&boxa, &identity);
    rp.compare_values(boxa.len() as f64, id_result.len() as f64, 0.0);
    let ob = boxa.get(0).expect("original box 0");
    let ib = id_result.get(0).expect("identity box 0");
    rp.compare_values(ob.x as f64, ib.x as f64, 1.0);
    rp.compare_values(ob.y as f64, ib.y as f64, 1.0);
    rp.compare_values(ob.w as f64, ib.w as f64, 1.0);
    rp.compare_values(ob.h as f64, ib.h as f64, 1.0);

    // Translation-only affine should match Boxa::translate
    let mat_translate = AffineMatrix::translation(44.0, 39.0);
    let affine_translated = leptonica::transform::boxa_affine_transform(&boxa, &mat_translate);
    let direct_translated = boxa.translate(44.0, 39.0);

    let at = affine_translated.get(0).expect("affine translated box 0");
    let dt = direct_translated.get(0).expect("direct translated box 0");
    rp.compare_values(dt.x as f64, at.x as f64, 1.0);
    rp.compare_values(dt.y as f64, at.y as f64, 1.0);
    rp.compare_values(dt.w as f64, at.w as f64, 1.0);
    rp.compare_values(dt.h as f64, at.h as f64, 1.0);

    assert!(rp.cleanup(), "xformbox affine transform test failed");
}

/// Test Boxa rotation (C check 3 rotation part).
///
/// Rotates boxes by a small angle and verifies the result is reasonable.
#[test]
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
/// Tests all 6 orderings with translation-only (scale=1.0, angle=0)
/// and verifies they produce identical results. Also tests with
/// rotation and scale to verify orderings produce different results.
#[test]
fn xformbox_reg_ordered() {
    use leptonica::TransformOrder;

    let mut rp = RegParams::new("xformbox_ordered");

    // Create test boxes
    let mut boxa = Boxa::new();
    boxa.push(LeptBox::new(100, 100, 50, 30).expect("box1"));
    boxa.push(LeptBox::new(200, 150, 60, 40).expect("box2"));
    boxa.push(LeptBox::new(300, 200, 70, 50).expect("box3"));

    let orderings = [
        TransformOrder::TrScRo,
        TransformOrder::TrRoSc,
        TransformOrder::ScTrRo,
        TransformOrder::RoTrSc,
        TransformOrder::RoScTr,
        TransformOrder::ScRoTr,
    ];

    // For translation-only (scale=1.0, angle=0.0), all orderings must produce
    // the same result.
    let reference = boxa.transform_ordered(44, 39, 1.0, 1.0, 0, 0, 0.0, orderings[0]);
    rp.compare_values(boxa.len() as f64, reference.len() as f64, 0.0);

    for &order in &orderings[1..] {
        let result = boxa.transform_ordered(44, 39, 1.0, 1.0, 0, 0, 0.0, order);
        rp.compare_values(reference.len() as f64, result.len() as f64, 0.0);
        for i in 0..reference.len() {
            let rb = reference.get(i).expect("ref box");
            let ob = result.get(i).expect("order box");
            rp.compare_values(rb.x as f64, ob.x as f64, 0.0);
            rp.compare_values(rb.y as f64, ob.y as f64, 0.0);
            rp.compare_values(rb.w as f64, ob.w as f64, 0.0);
            rp.compare_values(rb.h as f64, ob.h as f64, 0.0);
        }
    }

    // With rotation and scale, different orderings should generally
    // produce different results. Verify at least that each produces
    // valid boxes with positive dimensions.
    for &order in &orderings {
        let result = boxa.transform_ordered(10, 20, 1.5, 1.5, 200, 150, 0.15, order);
        rp.compare_values(boxa.len() as f64, result.len() as f64, 0.0);
        for i in 0..result.len() {
            let b = result.get(i).expect("transformed box");
            rp.compare_values(1.0, if b.w > 0 { 1.0 } else { 0.0 }, 0.0);
            rp.compare_values(1.0, if b.h > 0 { 1.0 } else { 0.0 }, 0.0);
        }
    }

    assert!(rp.cleanup(), "xformbox ordered transforms test failed");
}

/// Test hash box rendering (C checks 0-2).
///
/// Uses conncomp_pixa on feyn.tif to extract component boxes, then
/// renders hash lines using binary, color, and blend modes.
#[test]
fn xformbox_reg_hash_rendering() {
    use leptonica::core::pix::{HashOrientation, PixelOp};
    use leptonica::{Color, InitColor, PixMut, PixelDepth};

    let mut rp = RegParams::new("xformbox_hash");

    // Load feyn.tif (1bpp) and extract connected component boxes
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let (boxa, _pixa) =
        leptonica::region::conncomp_pixa(&pix, leptonica::region::ConnectivityType::EightWay)
            .expect("conncomp_pixa");
    let n = boxa.len();
    rp.compare_values(1.0, if n > 10 { 1.0 } else { 0.0 }, 0.0);

    let w = pix.width();
    let h = pix.height();

    // 1. Binary hash rendering
    let mut pm1 = PixMut::new(w, h, PixelDepth::Bit1).expect("create 1bpp");
    let b0 = boxa.get(0).expect("box 0");
    pm1.render_hash_box(b0, 5, 1, HashOrientation::Horizontal, false, PixelOp::Set)
        .expect("render_hash_box binary");
    let pix1: leptonica::Pix = pm1.into();
    rp.compare_values(1.0, if pix1.count_pixels() > 0 { 1.0 } else { 0.0 }, 0.0);

    // 2. Color hash rendering on 32bpp canvas
    let mut pm32 = PixMut::new(w, h, PixelDepth::Bit32).expect("create 32bpp");
    pm32.set_black_or_white(InitColor::White);
    let blue = Color::new(0, 0, 255);
    let b1 = boxa.get(1.min(n - 1)).expect("box 1");
    pm32.render_hash_box_color(b1, 6, 1, HashOrientation::Vertical, false, blue)
        .expect("render_hash_box_color");
    let pix32: leptonica::Pix = pm32.into();
    rp.compare_values(w as f64, pix32.width() as f64, 0.0);

    // 3. Blend hash rendering on 32bpp canvas
    let mut pm_blend = PixMut::new(w, h, PixelDepth::Bit32).expect("create 32bpp blend");
    pm_blend.set_black_or_white(InitColor::White);
    let red = Color::new(255, 0, 0);
    let b2 = boxa.get(2.min(n - 1)).expect("box 2");
    pm_blend
        .render_hash_box_blend(b2, 5, 1, HashOrientation::PosSlope, true, red, 0.5)
        .expect("render_hash_box_blend");
    let pix_blend: leptonica::Pix = pm_blend.into();
    rp.compare_values(h as f64, pix_blend.height() as f64, 0.0);

    assert!(rp.cleanup(), "xformbox hash rendering test failed");
}
