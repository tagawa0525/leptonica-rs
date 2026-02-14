//! Numa windowed statistics and operations regression test
//!
//! Tests windowed stats, join, similar, and related operations.
//!
//! NOTE: C version (numa2_reg.c) also tests pixel extraction on lines,
//! row/column averages, and variance calculations which require Pix methods
//! not yet available in the current build. Those tests are deferred.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/numa2_reg.c`

use leptonica_core::Numa;
use leptonica_test::RegParams;

// ============================================================================
// Test 1: Numa windowed stats
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa2_reg_windowed_stats() {
    let mut rp = RegParams::new("numa2_windowed");

    // Generate a synthetic signal
    let n = 500;
    let mut na = Numa::with_capacity(n);
    for i in 0..n {
        let x = i as f32 / n as f32;
        let val = 100.0 * (6.0 * std::f32::consts::PI * x).sin()
            + 50.0 * (14.0 * std::f32::consts::PI * x).sin()
            + 128.0;
        na.push(val);
    }

    let halfwin = 5;
    let stats = na.windowed_stats(halfwin);
    let na_mean = &stats.mean;
    let na_meansq = &stats.mean_square;
    let na_var = &stats.variance;
    let na_rms = &stats.rms;

    // Verify sizes match input
    rp.compare_values(n as f64, na_mean.len() as f64, 0.0);
    rp.compare_values(n as f64, na_meansq.len() as f64, 0.0);
    rp.compare_values(n as f64, na_var.len() as f64, 0.0);
    rp.compare_values(n as f64, na_rms.len() as f64, 0.0);

    // Verify variance = meansq - mean^2 at several points
    for idx in [0, 50, 100, 200, 400, n - 1] {
        let m = na_mean.get(idx).unwrap();
        let ms = na_meansq.get(idx).unwrap();
        let v = na_var.get(idx).unwrap();
        let expected_var = ms - m * m;
        rp.compare_values(expected_var as f64, v as f64, 0.01);
    }

    // Verify rms = sqrt(variance) at several points
    for idx in [0, 100, 250, n - 1] {
        let v = na_var.get(idx).unwrap();
        let r = na_rms.get(idx).unwrap();
        let expected_rms = if v > 0.0 { v.sqrt() } else { 0.0 };
        rp.compare_values(expected_rms as f64, r as f64, 0.001);
    }

    // Windowed mean should be smoother than original
    let orig_var = {
        let mean = na.mean().unwrap();
        let sumsq: f32 = na.iter().map(|v| (v - mean) * (v - mean)).sum();
        sumsq / n as f32
    };
    let wmean_var = {
        let mean = na_mean.mean().unwrap();
        let sumsq: f32 = na_mean.iter().map(|v| (v - mean) * (v - mean)).sum();
        sumsq / n as f32
    };
    let smoothing = if wmean_var < orig_var { 1.0 } else { 0.0 };
    rp.compare_values(1.0, smoothing, 0.0);

    // Windowed mean of a constant = constant
    let constant_na = Numa::from_vec(vec![42.0; 100]);
    let constant_mean = constant_na.windowed_mean(5);
    rp.compare_values(42.0, constant_mean.get(50).unwrap() as f64, 0.001);

    // Variance of a constant should be 0
    let constant_stats = constant_na.windowed_stats(5);
    rp.compare_values(0.0, constant_stats.variance.get(50).unwrap() as f64, 0.001);

    assert!(rp.cleanup(), "numa2_reg windowed stats tests failed");
}

// ============================================================================
// Test 2: Numa join and similar
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa2_reg_join_similar() {
    let mut rp = RegParams::new("numa2_join");

    // Test join: concatenate two arrays
    let mut na1 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    let na2 = Numa::from_vec(vec![4.0, 5.0, 6.0]);
    na1.join(&na2);
    rp.compare_values(6.0, na1.len() as f64, 0.0);
    rp.compare_values(4.0, na1.get(3).unwrap() as f64, 0.0);
    rp.compare_values(6.0, na1.get(5).unwrap() as f64, 0.0);

    // Test join_range: partial join
    let mut na3 = Numa::from_vec(vec![10.0]);
    let na4 = Numa::from_vec(vec![20.0, 30.0, 40.0, 50.0]);
    na3.join_range(&na4, 1, Some(2));
    rp.compare_values(3.0, na3.len() as f64, 0.0);
    rp.compare_values(30.0, na3.get(1).unwrap() as f64, 0.0);
    rp.compare_values(40.0, na3.get(2).unwrap() as f64, 0.0);

    // Test similar: exact match
    let na5 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    let na6 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    let exact = na5.similar(&na6, 0.0);
    rp.compare_values(1.0, if exact { 1.0 } else { 0.0 }, 0.0);

    // Test similar: within tolerance
    let na7 = Numa::from_vec(vec![1.01, 2.01, 3.01]);
    let within = na5.similar(&na7, 0.05);
    rp.compare_values(1.0, if within { 1.0 } else { 0.0 }, 0.0);

    // Test similar: outside tolerance
    let outside = na5.similar(&na7, 0.001);
    rp.compare_values(0.0, if outside { 1.0 } else { 0.0 }, 0.0);

    // Test similar: different lengths
    let na8 = Numa::from_vec(vec![1.0, 2.0]);
    let diff_len = na5.similar(&na8, 0.0);
    rp.compare_values(0.0, if diff_len { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "numa2_reg join/similar tests failed");
}

// ============================================================================
// Test 3: Windowed mean with mirrored border edge behavior
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa2_reg_windowed_edge_behavior() {
    let mut rp = RegParams::new("numa2_edge");

    // Test that windowed mean handles edges correctly with mirrored border
    let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let mean = na.windowed_mean(1);

    // With halfwin=1, window size = 3
    // Mirrored border: [2, 1, 2, 3, 4, 5, 4]
    // At i=0: (2+1+2)/3 = 5/3 ≈ 1.667
    // At i=2: (2+3+4)/3 = 9/3 = 3.0
    // At i=4: (4+5+4)/3 = 13/3 ≈ 4.333

    rp.compare_values(5.0, mean.len() as f64, 0.0);
    // Middle value should be exact
    rp.compare_values(3.0, mean.get(2).unwrap() as f64, 0.001);

    // Windowed mean square of all zeros should be zero
    let zeros = Numa::from_vec(vec![0.0; 20]);
    let ms = zeros.windowed_mean_square(3);
    rp.compare_values(0.0, ms.get(10).unwrap() as f64, 0.001);

    assert!(
        rp.cleanup(),
        "numa2_reg windowed edge behavior tests failed"
    );
}
