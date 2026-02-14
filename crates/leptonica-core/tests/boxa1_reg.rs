//! Box/Boxa regression test - basic operations
//!
//! Tests Box creation, Boxa construction, bounding box, intersection,
//! union, translation, scaling, containment, and sorting.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/boxa1_reg.c`

use leptonica_core::{Box, Boxa};
use leptonica_test::RegParams;

#[test]
fn boxa1_reg() {
    let mut rp = RegParams::new("boxa1");

    // --- Test 1: Box creation and properties ---
    let b1 = Box::new(60, 60, 40, 20).expect("box create");
    rp.compare_values(60.0, b1.x as f64, 0.0);
    rp.compare_values(60.0, b1.y as f64, 0.0);
    rp.compare_values(40.0, b1.w as f64, 0.0);
    rp.compare_values(20.0, b1.h as f64, 0.0);
    rp.compare_values(100.0, b1.right() as f64, 0.0);
    rp.compare_values(80.0, b1.bottom() as f64, 0.0);
    rp.compare_values(800.0, b1.area() as f64, 0.0);

    // --- Test 2: Build a Boxa (same boxes as C version) ---
    let mut boxa1 = Boxa::with_capacity(6);
    boxa1.push(Box::new_unchecked(60, 60, 40, 20));
    boxa1.push(Box::new_unchecked(120, 50, 20, 50));
    boxa1.push(Box::new_unchecked(50, 140, 46, 60));
    boxa1.push(Box::new_unchecked(166, 130, 64, 28));
    boxa1.push(Box::new_unchecked(64, 224, 44, 34));
    boxa1.push(Box::new_unchecked(117, 206, 26, 74));

    rp.compare_values(6.0, boxa1.len() as f64, 0.0);

    // --- Test 3: Bounding box ---
    let bb = boxa1.bounding_box().expect("bounding box");
    rp.compare_values(50.0, bb.x as f64, 0.0);
    rp.compare_values(50.0, bb.y as f64, 0.0);
    rp.compare_values(230.0, bb.right() as f64, 0.0);
    rp.compare_values(280.0, bb.bottom() as f64, 0.0);

    // --- Test 4: Box intersection ---
    let a = Box::new_unchecked(60, 60, 40, 20);
    let b = Box::new_unchecked(80, 70, 60, 40);
    let inter = a.intersect(&b).expect("intersection");
    rp.compare_values(80.0, inter.x as f64, 0.0);
    rp.compare_values(70.0, inter.y as f64, 0.0);
    rp.compare_values(20.0, inter.w as f64, 0.0);
    rp.compare_values(10.0, inter.h as f64, 0.0);

    // Non-overlapping -> None
    let c = Box::new_unchecked(200, 200, 10, 10);
    rp.compare_values(1.0, if a.intersect(&c).is_none() { 1.0 } else { 0.0 }, 0.0);

    // --- Test 5: Box union ---
    let u = a.union(&b);
    rp.compare_values(60.0, u.x as f64, 0.0);
    rp.compare_values(60.0, u.y as f64, 0.0);
    rp.compare_values(80.0, u.w as f64, 0.0);
    rp.compare_values(50.0, u.h as f64, 0.0);

    // --- Test 6: Box translate ---
    let shifted = a.translate(-13, -13);
    rp.compare_values(47.0, shifted.x as f64, 0.0);
    rp.compare_values(47.0, shifted.y as f64, 0.0);
    rp.compare_values(40.0, shifted.w as f64, 0.0);
    rp.compare_values(20.0, shifted.h as f64, 0.0);

    // --- Test 7: Box scale ---
    let scaled = a.scale(2.0);
    rp.compare_values(120.0, scaled.x as f64, 0.0);
    rp.compare_values(120.0, scaled.y as f64, 0.0);
    rp.compare_values(80.0, scaled.w as f64, 0.0);
    rp.compare_values(40.0, scaled.h as f64, 0.0);

    // --- Test 8: Box contains_point ---
    rp.compare_values(1.0, if a.contains_point(70, 70) { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(0.0, if a.contains_point(50, 50) { 1.0 } else { 0.0 }, 0.0);

    // --- Test 9: Box contains_box ---
    let inner = Box::new_unchecked(65, 65, 10, 10);
    rp.compare_values(1.0, if a.contains_box(&inner) { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(0.0, if a.contains_box(&b) { 1.0 } else { 0.0 }, 0.0);

    // --- Test 10: Boxa sort ---
    let mut boxa2 = Boxa::new();
    boxa2.push(Box::new_unchecked(100, 100, 10, 10));
    boxa2.push(Box::new_unchecked(0, 0, 10, 10));
    boxa2.push(Box::new_unchecked(50, 50, 10, 10));

    boxa2.sort_by_position();
    rp.compare_values(0.0, boxa2.get(0).unwrap().x as f64, 0.0);
    rp.compare_values(50.0, boxa2.get(1).unwrap().x as f64, 0.0);
    rp.compare_values(100.0, boxa2.get(2).unwrap().x as f64, 0.0);

    assert!(rp.cleanup(), "boxa1 regression test failed");
}
