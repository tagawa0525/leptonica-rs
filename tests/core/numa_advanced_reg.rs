//! Regression tests for plan 109 (Numa 高度関数 5 関数).

use leptonica::Numa;
use leptonica::core::numa::advanced::{
    gen_constrained_numa_in_range, numa_crossings_by_threshold, numa_uniform_bin_sizes,
};

// -- Numa::count_reversals ----------------------------------------------

#[test]
fn count_reversals_binary() {
    let na = Numa::from_vec(vec![0.0, 1.0, 1.0, 0.0, 1.0, 0.0]);
    let (nr, _rd) = na.count_reversals(0.5).unwrap();
    // Transitions: 0->1, 1->0, 0->1, 1->0 = 4
    assert_eq!(nr, 4);
}

#[test]
fn count_reversals_negative_threshold_errors() {
    let na = Numa::from_vec(vec![0.0, 1.0]);
    assert!(na.count_reversals(-0.1).is_err());
}

#[test]
fn count_reversals_empty() {
    let na = Numa::new();
    let (nr, rd) = na.count_reversals(0.0).unwrap();
    assert_eq!(nr, 0);
    assert!((rd - 0.0).abs() < 1e-6);
}

// -- Numa::find_peaks ---------------------------------------------------

#[test]
fn find_peaks_single_unimodal() {
    // Triangle shape: 0, 1, 4, 1, 0
    let na = Numa::from_vec(vec![0.0, 1.0, 4.0, 1.0, 0.0]);
    let peaks = na.find_peaks(2, 0.1, 0.5);
    // At least one peak's max_loc should be 2.
    assert!(peaks.len() >= 4);
    // peak format: [lloc, max_loc, rloc, fract]
    let max_loc = peaks.get(1).unwrap();
    assert_eq!(max_loc, 2.0);
}

#[test]
fn find_peaks_empty_returns_empty() {
    let na = Numa::new();
    let peaks = na.find_peaks(3, 0.1, 0.5);
    assert_eq!(peaks.len(), 0);
}

#[test]
fn find_peaks_all_zero_returns_empty() {
    let na = Numa::from_vec(vec![0.0; 5]);
    let peaks = na.find_peaks(3, 0.1, 0.5);
    assert_eq!(peaks.len(), 0);
}

// -- numa_crossings_by_threshold ---------------------------------------

#[test]
fn crossings_by_threshold_simple_rise() {
    // y = [0, 2] crosses thresh=1 between index 0 and 1.
    let nay = Numa::from_vec(vec![0.0, 2.0]);
    let out = numa_crossings_by_threshold(&nay, None, 1.0).unwrap();
    assert_eq!(out.len(), 1);
    // Default x is 0.0 + i*1.0 (parameters default), interpolated at x=0.5.
    assert!((out.get(0).unwrap() - 0.5).abs() < 1e-5);
}

#[test]
fn crossings_by_threshold_no_crossing() {
    let nay = Numa::from_vec(vec![5.0, 6.0, 7.0]);
    let out = numa_crossings_by_threshold(&nay, None, 1.0).unwrap();
    assert_eq!(out.len(), 0);
}

#[test]
fn crossings_by_threshold_nax_mismatch_errors() {
    let nay = Numa::from_vec(vec![0.0, 1.0]);
    let nax = Numa::from_vec(vec![0.0]);
    assert!(numa_crossings_by_threshold(&nay, Some(&nax), 0.5).is_err());
}

// -- numa_uniform_bin_sizes --------------------------------------------

#[test]
fn uniform_bin_sizes_even() {
    let na = numa_uniform_bin_sizes(10, 5).unwrap();
    assert_eq!(na.len(), 5);
    for i in 0..5 {
        assert_eq!(na.get(i).unwrap(), 2.0);
    }
}

#[test]
fn uniform_bin_sizes_uneven() {
    let na = numa_uniform_bin_sizes(10, 3).unwrap();
    assert_eq!(na.len(), 3);
    // Sum must equal ntotal.
    let total: f32 = (0..3).map(|i| na.get(i).unwrap()).sum();
    assert_eq!(total, 10.0);
}

#[test]
fn uniform_bin_sizes_ntotal_less_than_nbins() {
    // 3 items into 5 bins: first 3 bins get 1 each.
    let na = numa_uniform_bin_sizes(3, 5).unwrap();
    assert_eq!(na.len(), 3);
    for i in 0..3 {
        assert_eq!(na.get(i).unwrap(), 1.0);
    }
}

#[test]
fn uniform_bin_sizes_invalid_errors() {
    assert!(numa_uniform_bin_sizes(0, 5).is_err());
    assert!(numa_uniform_bin_sizes(10, 0).is_err());
}

// -- gen_constrained_numa_in_range ------------------------------------

#[test]
fn gen_constrained_numa_simple() {
    let na = gen_constrained_numa_in_range(0, 10, 6, false).unwrap();
    assert_eq!(na.len(), 6);
    assert_eq!(na.get(0).unwrap(), 0.0);
    assert_eq!(na.get(5).unwrap(), 10.0);
}

#[test]
fn gen_constrained_numa_nmax_smaller_than_range() {
    let na = gen_constrained_numa_in_range(0, 100, 3, false).unwrap();
    assert_eq!(na.len(), 3);
    assert_eq!(na.get(0).unwrap(), 0.0);
    assert_eq!(na.get(2).unwrap(), 100.0);
}

#[test]
fn gen_constrained_numa_use_pairs() {
    let na = gen_constrained_numa_in_range(0, 10, 6, true).unwrap();
    // nsets = min(6, 11) / 2 = 3, total = 6 entries
    assert_eq!(na.len(), 6);
    // Each pair (val, val+1)
    for i in 0..3 {
        let v = na.get(i * 2).unwrap();
        let v_next = na.get(i * 2 + 1).unwrap();
        assert_eq!(v_next, v + 1.0);
    }
}

#[test]
fn gen_constrained_numa_invalid_errors() {
    // last < first
    assert!(gen_constrained_numa_in_range(10, 0, 5, false).is_err());
    // nmax < 1
    assert!(gen_constrained_numa_in_range(0, 10, 0, false).is_err());
}
