//! Box/Boxa regression test - arithmetic and sorting
//!
//! C版: reference/leptonica/prog/boxa2_reg.c
//! Boxa内の統計的操作、異常サイズ検出、パリティ分離をテスト。
//!
//! NOTE: C版の統計操作(boxaGetWhiteblocks, boxaSizeVariation等)は
//! Rust未実装のためスキップ。利用可能なBox算術演算をテスト。

use leptonica_core::{Box, Boxa};
use leptonica_test::RegParams;

#[test]
fn boxa2_reg() {
    let mut rp = RegParams::new("boxa2");

    // --- Test: Box area calculations ---
    let b1 = Box::new(0, 0, 100, 50).unwrap();
    let b2 = Box::new(0, 0, 200, 100).unwrap();
    rp.compare_values(5000.0, b1.area() as f64, 0.0);
    rp.compare_values(20000.0, b2.area() as f64, 0.0);

    // --- Test: Box expand ---
    let expanded = b1.expand(10);
    rp.compare_values(-10.0, expanded.x as f64, 0.0);
    rp.compare_values(-10.0, expanded.y as f64, 0.0);
    rp.compare_values(120.0, expanded.w as f64, 0.0);
    rp.compare_values(70.0, expanded.h as f64, 0.0);

    // --- Test: Box clip ---
    let large = Box::new_unchecked(-10, -10, 200, 200);
    let clipped = large.clip(100, 80).expect("clip");
    rp.compare_values(0.0, clipped.x as f64, 0.0);
    rp.compare_values(0.0, clipped.y as f64, 0.0);
    rp.compare_values(100.0, clipped.w as f64, 0.0);
    rp.compare_values(80.0, clipped.h as f64, 0.0);

    // Fully outside → None
    let outside = Box::new_unchecked(200, 200, 10, 10);
    rp.compare_values(
        1.0,
        if outside.clip(100, 100).is_none() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // --- Test: Box from_corners ---
    let bc = Box::from_corners(50, 30, 10, 10);
    rp.compare_values(10.0, bc.x as f64, 0.0);
    rp.compare_values(10.0, bc.y as f64, 0.0);
    rp.compare_values(40.0, bc.w as f64, 0.0);
    rp.compare_values(20.0, bc.h as f64, 0.0);

    // --- Test: Boxa sort by area ---
    let mut boxa = Boxa::new();
    boxa.push(Box::new_unchecked(0, 0, 10, 10)); // area = 100
    boxa.push(Box::new_unchecked(0, 0, 50, 50)); // area = 2500
    boxa.push(Box::new_unchecked(0, 0, 20, 20)); // area = 400
    boxa.push(Box::new_unchecked(0, 0, 5, 5)); // area = 25

    boxa.sort_by_area(true); // ascending
    rp.compare_values(25.0, boxa.get(0).unwrap().area() as f64, 0.0);
    rp.compare_values(100.0, boxa.get(1).unwrap().area() as f64, 0.0);
    rp.compare_values(400.0, boxa.get(2).unwrap().area() as f64, 0.0);
    rp.compare_values(2500.0, boxa.get(3).unwrap().area() as f64, 0.0);

    boxa.sort_by_area(false); // descending
    rp.compare_values(2500.0, boxa.get(0).unwrap().area() as f64, 0.0);
    rp.compare_values(25.0, boxa.get(3).unwrap().area() as f64, 0.0);

    // --- Test: Boxa operations (push, remove, replace) ---
    let mut boxa2 = Boxa::new();
    boxa2.push(Box::new_unchecked(0, 0, 10, 10));
    boxa2.push(Box::new_unchecked(10, 10, 20, 20));
    boxa2.push(Box::new_unchecked(20, 20, 30, 30));
    rp.compare_values(3.0, boxa2.len() as f64, 0.0);

    let removed = boxa2.remove(1).unwrap();
    rp.compare_values(10.0, removed.x as f64, 0.0);
    rp.compare_values(2.0, boxa2.len() as f64, 0.0);

    let old = boxa2.replace(0, Box::new_unchecked(99, 99, 1, 1)).unwrap();
    rp.compare_values(0.0, old.x as f64, 0.0);
    rp.compare_values(99.0, boxa2.get(0).unwrap().x as f64, 0.0);

    // --- Test: Box overlaps ---
    let ov1 = Box::new_unchecked(0, 0, 50, 50);
    let ov2 = Box::new_unchecked(25, 25, 50, 50);
    let ov3 = Box::new_unchecked(100, 100, 10, 10);
    rp.compare_values(1.0, if ov1.overlaps(&ov2) { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(0.0, if ov1.overlaps(&ov3) { 1.0 } else { 0.0 }, 0.0);

    // --- Test: Box center ---
    let bc2 = Box::new_unchecked(10, 20, 100, 60);
    rp.compare_values(60.0, bc2.center_x() as f64, 0.0);
    rp.compare_values(50.0, bc2.center_y() as f64, 0.0);

    // NOTE: C版の boxaSizeVariation, boxaGetWhiteblocks,
    // boxaGetRankSizeL, boxaPermutePseudorandom 等はRust未実装

    assert!(rp.cleanup(), "boxa2 regression test failed");
}
