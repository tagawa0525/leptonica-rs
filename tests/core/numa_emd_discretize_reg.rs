//! Regression tests for plan 130 (Numa EMD + discretize).

use leptonica::core::numa::Numa;

// -- earth_mover_distance ---------------------------------------------

#[test]
fn emd_identical_arrays() {
    let na = Numa::from_i32_slice(&[1, 2, 3, 4, 5]);
    let d = na.earth_mover_distance(&na).unwrap();
    assert!(d.abs() < 1e-5, "expected 0, got {d}");
}

#[test]
fn emd_shifted_arrays_proportional() {
    // Shift mass to the right by 2 units in a 1-mass 5-element distribution.
    let a = Numa::from_i32_slice(&[1, 0, 0, 0, 0]);
    let b = Numa::from_i32_slice(&[0, 0, 1, 0, 0]);
    let d = a.earth_mover_distance(&b).unwrap();
    // EMD measured in mass*distance units, normalized by sum1 = 1.
    // Move 1 unit at index 0 → 1 unit at index 2 = 2 units of work.
    assert!((d - 2.0).abs() < 1e-4, "expected 2.0, got {d}");
}

#[test]
fn emd_normalizes_different_sums() {
    // Even with different total mass, the second array is renormalized to
    // match the first. Two flat arrays with the same shape but different
    // totals should yield EMD = 0.
    let a = Numa::from_i32_slice(&[1, 1, 1, 1]);
    let b = Numa::from_i32_slice(&[3, 3, 3, 3]);
    let d = a.earth_mover_distance(&b).unwrap();
    assert!(d.abs() < 1e-5, "expected 0 after renormalization, got {d}");
}

#[test]
fn emd_length_mismatch_errors() {
    let a = Numa::from_i32_slice(&[1, 2, 3]);
    let b = Numa::from_i32_slice(&[1, 2]);
    assert!(a.earth_mover_distance(&b).is_err());
}

// -- discretize_sorted_in_bins ----------------------------------------

#[test]
fn discretize_sorted_2_bins() {
    // 1..=10 split into 2 bins: lower half mean = 3.0, upper half = 8.0.
    let na = Numa::from_i32_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let out = na.discretize_sorted_in_bins(2).unwrap();
    assert_eq!(out.len(), 2);
    assert!((out.get(0).unwrap() - 3.0).abs() < 1e-4);
    assert!((out.get(1).unwrap() - 8.0).abs() < 1e-4);
}

#[test]
fn discretize_sorted_rejects_nbins_below_2() {
    let na = Numa::from_i32_slice(&[1, 2, 3]);
    assert!(na.discretize_sorted_in_bins(0).is_err());
    assert!(na.discretize_sorted_in_bins(1).is_err());
}

#[test]
fn discretize_sorted_rejects_empty() {
    let na = Numa::new();
    assert!(na.discretize_sorted_in_bins(4).is_err());
}

// -- discretize_histo_in_bins -----------------------------------------

#[test]
fn discretize_histo_uniform() {
    // Histogram of 100 evenly spread across 10 indices (10 per index).
    // 5 bins (each with 20 entries): bin averages should be approximately
    // (10+11)/2 * 0.5 ... actually let me compute:
    // bins of 20 entries:
    //   bin 0: indices 0..2 (avg ~ 0.95)
    //   bin 1: indices 2..4 (avg ~ 2.95) ...
    let mut na = Numa::new();
    for _ in 0..10 {
        na.push(10.0);
    }
    let (binval, rank) = na.discretize_histo_in_bins(5, true).unwrap();
    assert_eq!(binval.len(), 5);
    // Bin averages should be increasing.
    let mut prev = f32::NEG_INFINITY;
    for i in 0..5 {
        let v = binval.get(i).unwrap();
        assert!(v > prev, "bin {i} value {v} is not increasing from {prev}");
        prev = v;
    }
    let rank = rank.unwrap();
    // Cumulative rank should reach ~1.0 at the end.
    let last = rank.get(rank.len() - 1).unwrap();
    assert!(
        (last - 1.0).abs() < 0.01,
        "cumulative rank should approach 1.0, got {last}"
    );
}

#[test]
fn discretize_histo_no_rank() {
    let mut na = Numa::new();
    for _ in 0..4 {
        na.push(4.0);
    }
    let (binval, rank) = na.discretize_histo_in_bins(2, false).unwrap();
    assert_eq!(binval.len(), 2);
    assert!(rank.is_none());
}

#[test]
fn discretize_histo_rejects_invalid() {
    let na = Numa::from_i32_slice(&[1, 1, 1]);
    assert!(na.discretize_histo_in_bins(1, false).is_err());
    let empty = Numa::new();
    assert!(empty.discretize_histo_in_bins(4, false).is_err());
}
