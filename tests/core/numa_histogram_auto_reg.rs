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
    let (start, delx) = h.parameters();
    assert_eq!(start, 7.0);
    // delx must be positive so downstream APIs that divide by delx
    // (e.g. histogram_rank_from_val) don't produce NaN/inf.
    assert!(delx > 0.0, "constant-input delx must be > 0, got {delx}");
}

#[test]
fn histogram_auto_constant_non_integer_uses_positive_delx() {
    // Non-integer constant: still produces a usable 1-bin histogram.
    let mut na = Numa::new();
    for _ in 0..5 {
        na.push(0.5);
    }
    let h = na.make_histogram_auto(4).unwrap();
    assert_eq!(h.len(), 1);
    let (_, delx) = h.parameters();
    assert!(delx > 0.0, "delx must be > 0, got {delx}");
}

#[test]
fn histogram_auto_rejects_maxbins_zero() {
    let na = Numa::from_i32_slice(&[1, 2, 3]);
    assert!(na.make_histogram_auto(0).is_err());
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
