//! Point array regression test
//!
//! C版: reference/leptonica/prog/pta_reg.c
//! Pta/Ptaaの基本操作、変換、ソート、境界ピクセル取得をテスト。
//!
//! NOTE: C版のptaaGetBoundaryPixels, pixRenderRandomCmapPtaa,
//! ptaSort, ptaEqual, ptaPolygonIsConvex はRust未実装のためスキップ。

use leptonica_core::{Pta, Ptaa};
use leptonica_test::RegParams;

#[test]
fn pta_reg() {
    let mut rp = RegParams::new("pta");

    // --- Test 1: Pta creation and access ---
    let mut pta = Pta::new();
    pta.push(10.0, 20.0);
    pta.push(30.0, 40.0);
    pta.push(50.0, 60.0);

    rp.compare_values(3.0, pta.len() as f64, 0.0);
    let (x, y) = pta.get(0).unwrap();
    rp.compare_values(10.0, x as f64, 0.0);
    rp.compare_values(20.0, y as f64, 0.0);

    // --- Test 2: Pta from_vecs ---
    let pta2 = Pta::from_vecs(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]).unwrap();
    rp.compare_values(3.0, pta2.len() as f64, 0.0);
    rp.compare_values(2.0, pta2.get_x(1).unwrap() as f64, 0.0);
    rp.compare_values(5.0, pta2.get_y(1).unwrap() as f64, 0.0);

    // Mismatched lengths should error
    let err = Pta::from_vecs(vec![1.0], vec![1.0, 2.0]);
    rp.compare_values(1.0, if err.is_err() { 1.0 } else { 0.0 }, 0.0);

    // --- Test 3: Bounding box ---
    let mut pta3 = Pta::new();
    pta3.push(10.0, 5.0);
    pta3.push(30.0, 40.0);
    pta3.push(15.0, 20.0);

    let (x_min, y_min, x_max, y_max) = pta3.bounding_box().unwrap();
    rp.compare_values(10.0, x_min as f64, 0.001);
    rp.compare_values(5.0, y_min as f64, 0.001);
    rp.compare_values(30.0, x_max as f64, 0.001);
    rp.compare_values(40.0, y_max as f64, 0.001);

    // --- Test 4: Centroid ---
    let mut pta4 = Pta::new();
    pta4.push(0.0, 0.0);
    pta4.push(10.0, 0.0);
    pta4.push(10.0, 10.0);
    pta4.push(0.0, 10.0);

    let (cx, cy) = pta4.centroid().unwrap();
    rp.compare_values(5.0, cx as f64, 0.001);
    rp.compare_values(5.0, cy as f64, 0.001);

    // --- Test 5: Translate ---
    let mut pta5 = Pta::new();
    pta5.push(10.0, 20.0);
    pta5.push(30.0, 40.0);
    pta5.translate(5.0, -5.0);

    let (x0, y0) = pta5.get(0).unwrap();
    let (x1, y1) = pta5.get(1).unwrap();
    rp.compare_values(15.0, x0 as f64, 0.001);
    rp.compare_values(15.0, y0 as f64, 0.001);
    rp.compare_values(35.0, x1 as f64, 0.001);
    rp.compare_values(35.0, y1 as f64, 0.001);

    // --- Test 6: Scale ---
    let mut pta6 = Pta::new();
    pta6.push(10.0, 20.0);
    pta6.scale(2.0, 3.0);

    let (sx, sy) = pta6.get(0).unwrap();
    rp.compare_values(20.0, sx as f64, 0.001);
    rp.compare_values(60.0, sy as f64, 0.001);

    // --- Test 7: Rotate ---
    let mut pta7 = Pta::new();
    pta7.push(10.0, 0.0);
    pta7.rotate(std::f32::consts::FRAC_PI_2); // 90 degrees

    let (rx, ry) = pta7.get(0).unwrap();
    rp.compare_values(0.0, rx as f64, 0.01);
    rp.compare_values(10.0, ry as f64, 0.01);

    // --- Test 8: Set and remove ---
    let mut pta8 = Pta::new();
    pta8.push(1.0, 2.0);
    pta8.push(3.0, 4.0);
    pta8.push(5.0, 6.0);

    pta8.set(1, 30.0, 40.0).unwrap();
    let (sx, sy) = pta8.get(1).unwrap();
    rp.compare_values(30.0, sx as f64, 0.0);
    rp.compare_values(40.0, sy as f64, 0.0);

    let (rx, ry) = pta8.remove(0).unwrap();
    rp.compare_values(1.0, rx as f64, 0.0);
    rp.compare_values(2.0, ry as f64, 0.0);
    rp.compare_values(2.0, pta8.len() as f64, 0.0);

    // --- Test 9: Insert ---
    let mut pta9 = Pta::new();
    pta9.push(1.0, 1.0);
    pta9.push(3.0, 3.0);
    pta9.insert(1, 2.0, 2.0).unwrap();
    rp.compare_values(3.0, pta9.len() as f64, 0.0);
    let (ix, iy) = pta9.get(1).unwrap();
    rp.compare_values(2.0, ix as f64, 0.0);
    rp.compare_values(2.0, iy as f64, 0.0);

    // --- Test 10: Iterator / FromIterator ---
    let pta10: Pta = [(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)].into_iter().collect();
    rp.compare_values(3.0, pta10.len() as f64, 0.0);
    let points: Vec<(f32, f32)> = pta10.iter().collect();
    rp.compare_values(3.0, points.len() as f64, 0.0);
    rp.compare_values(5.0, points[2].0 as f64, 0.0);

    // --- Test 11: Ptaa ---
    let mut ptaa = Ptaa::new();
    let mut pa1 = Pta::new();
    pa1.push(0.0, 0.0);
    pa1.push(1.0, 1.0);
    ptaa.push(pa1);

    let mut pa2 = Pta::new();
    pa2.push(10.0, 10.0);
    pa2.push(20.0, 20.0);
    pa2.push(30.0, 30.0);
    ptaa.push(pa2);

    rp.compare_values(2.0, ptaa.len() as f64, 0.0);
    rp.compare_values(5.0, ptaa.total_points() as f64, 0.0);

    let flat = ptaa.flatten();
    rp.compare_values(5.0, flat.len() as f64, 0.0);
    let (fx, fy) = flat.get(3).unwrap();
    rp.compare_values(20.0, fx as f64, 0.0);
    rp.compare_values(20.0, fy as f64, 0.0);

    // --- Test 12: Empty Pta edge cases ---
    let empty = Pta::new();
    rp.compare_values(
        1.0,
        if empty.bounding_box().is_none() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(1.0, if empty.centroid().is_none() { 1.0 } else { 0.0 }, 0.0);

    // NOTE: C版の ptaSort, ptaEqual, ptaPolygonIsConvex,
    // ptaaGetBoundaryPixels, pixRenderRandomCmapPtaa はRust未実装

    assert!(rp.cleanup(), "pta regression test failed");
}
