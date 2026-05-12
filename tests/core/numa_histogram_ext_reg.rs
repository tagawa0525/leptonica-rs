//! Regression tests for plan 119 (Numa histogram extension 2 関数).

use leptonica::Numa;
use leptonica::core::numa::{make_rank_from_histogram, numa_rebin_histogram};

// -- numa_rebin_histogram ----------------------------------------------

#[test]
fn rebin_histogram_even_group() {
    // 6 bins rebinned to size-2 groups -> 3 output bins.
    let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let out = numa_rebin_histogram(&na, 2).unwrap();
    assert_eq!(out.len(), 3);
    assert_eq!(out.get(0).unwrap(), 3.0); // 1+2
    assert_eq!(out.get(1).unwrap(), 7.0); // 3+4
    assert_eq!(out.get(2).unwrap(), 11.0); // 5+6
}

#[test]
fn rebin_histogram_partial_last_group() {
    // 5 bins rebinned to size-2 -> 3 outputs (last has 1 value).
    let na = Numa::from_vec(vec![1.0, 1.0, 1.0, 1.0, 1.0]);
    let out = numa_rebin_histogram(&na, 2).unwrap();
    assert_eq!(out.len(), 3);
    assert_eq!(out.get(0).unwrap(), 2.0);
    assert_eq!(out.get(1).unwrap(), 2.0);
    assert_eq!(out.get(2).unwrap(), 1.0); // partial
}

#[test]
fn rebin_histogram_scales_deltax() {
    // Tag the source with deltax=0.5; after rebin by 4 we expect 2.0.
    let mut na = Numa::from_vec(vec![0.0; 8]);
    na.set_parameters(10.0, 0.5);
    let out = numa_rebin_histogram(&na, 4).unwrap();
    let (start, delx) = out.parameters();
    assert_eq!(start, 10.0);
    assert!((delx - 2.0).abs() < 1e-6);
}

#[test]
fn rebin_histogram_invalid_size_errors() {
    let na = Numa::from_vec(vec![1.0, 2.0]);
    assert!(numa_rebin_histogram(&na, 0).is_err());
    assert!(numa_rebin_histogram(&na, 1).is_err());
}

#[test]
fn rebin_histogram_empty_errors() {
    let na = Numa::new();
    assert!(numa_rebin_histogram(&na, 2).is_err());
}

// -- make_rank_from_histogram -----------------------------------------

#[test]
fn rank_from_uniform_histogram_is_linear() {
    // Uniform histogram → cumulative distribution is linear, so the
    // sampled rank curve must be monotonically increasing from 0 to 1.
    let na = Numa::from_vec(vec![1.0, 1.0, 1.0, 1.0]);
    let (nax, nay) = make_rank_from_histogram(0.0, 1.0, &na, 5).unwrap();
    assert_eq!(nax.len(), 5);
    assert_eq!(nay.len(), 5);
    // Monotonic increase.
    let mut prev = nay.get(0).unwrap();
    for i in 1..5 {
        let cur = nay.get(i).unwrap();
        assert!(cur >= prev, "rank should be monotone");
        prev = cur;
    }
    // Bounds: first ~= 0.0, last ~= 1.0.
    assert!((nay.get(0).unwrap() - 0.0).abs() < 1e-5);
    assert!((nay.get(4).unwrap() - 1.0).abs() < 1e-3);
}

#[test]
fn rank_from_histogram_invalid_inputs_error() {
    let na = Numa::from_vec(vec![1.0, 2.0]);
    // npts < 3
    assert!(make_rank_from_histogram(0.0, 1.0, &na, 2).is_err());
    // deltax <= 0
    assert!(make_rank_from_histogram(0.0, 0.0, &na, 5).is_err());
    // empty histogram
    assert!(make_rank_from_histogram(0.0, 1.0, &Numa::new(), 5).is_err());
}
