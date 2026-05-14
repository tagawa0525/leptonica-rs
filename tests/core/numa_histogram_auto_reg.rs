//! Regression tests for Numa::make_histogram_auto (plan 132).

use leptonica::core::numa::Numa;

#[test]
fn histogram_auto_integer_fast_path() {
    // Values are integers in a small range -> unit-width integer histogram.
    let na = Numa::from_i32_slice(&[1, 2, 2, 3, 3, 3, 4]);
    let h = na.make_histogram_auto(10).unwrap();
    assert_eq!(h.len(), 4); // 1..=4
    assert_eq!(h.get(0).unwrap(), 1.0); // count of 1s
    assert_eq!(h.get(1).unwrap(), 2.0); // count of 2s
    assert_eq!(h.get(2).unwrap(), 3.0); // count of 3s
    assert_eq!(h.get(3).unwrap(), 1.0); // count of 4s
    let (start, delx) = h.parameters();
    assert_eq!(start, 1.0);
    assert_eq!(delx, 1.0);
}

#[test]
fn histogram_auto_float_path() {
    // Mixed float values -> float-bin path with maxbins buckets.
    let mut na = Numa::new();
    for v in [0.0_f32, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0] {
        na.push(v);
    }
    let h = na.make_histogram_auto(3).unwrap();
    assert_eq!(h.len(), 3);
    let total: f32 = (0..3).map(|i| h.get(i).unwrap()).sum();
    assert_eq!(total, 7.0, "all input values should be counted");
    let (start, delx) = h.parameters();
    assert!((start - 0.0).abs() < 1e-4);
    assert!((delx - 1.0).abs() < 1e-4); // (3.0 - 0.0) / 3 = 1.0
}

#[test]
fn histogram_auto_constant_collapses_to_single_bin() {
    // All same value -> single-bin histogram with count = n.
    let mut na = Numa::new();
    for _ in 0..10 {
        na.push(7.0);
    }
    let h = na.make_histogram_auto(8).unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h.get(0).unwrap(), 10.0);
    let (start, _) = h.parameters();
    assert_eq!(start, 7.0);
}

#[test]
fn histogram_auto_integer_with_large_range_falls_to_float_path() {
    // 10 integer values spanning 0..10000 -> not integer fast path
    // (range >= maxbins), uses float-binning.
    let mut na = Numa::new();
    for v in [0, 1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9999] {
        na.push(v as f32);
    }
    let h = na.make_histogram_auto(5).unwrap();
    assert_eq!(h.len(), 5);
    let total: f32 = (0..5).map(|i| h.get(i).unwrap()).sum();
    assert_eq!(total, 10.0);
}

#[test]
fn histogram_auto_rejects_empty() {
    let na = Numa::new();
    assert!(na.make_histogram_auto(8).is_err());
}
