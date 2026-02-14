//! Numa (numeric array) operations regression test
//!
//! Tests histograms, interpolation, integration/differentiation on Numa.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/numa1_reg.c`

use leptonica_core::Numa;
use leptonica_test::RegParams;

// ========================================================================
// Test: Histograms (C tests 0-10)
// ========================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa1_reg_histograms() {
    let mut rp = RegParams::new("numa1_histo");

    let pi: f32 = std::f32::consts::PI;

    // Generate sin-wave data (same as C version)
    let mut na = Numa::with_capacity(500000);
    for i in 0..500000u32 {
        let angle = 0.02293 * (i as f32) * pi;
        let val = 999.0 * angle.sin();
        na.push(val);
    }

    // Verify basic stats on the raw data
    let n = na.len();
    rp.compare_values(500000.0, n as f64, 0.0);

    // numaMakeHistogramClipped
    let nahisto_clipped = na.make_histogram_clipped(6.0, 2000.0).unwrap();
    let nbins_clipped = nahisto_clipped.len();
    assert!(nbins_clipped > 0, "Clipped histogram should have bins");
    rp.compare_values(1.0, if nbins_clipped > 0 { 1.0 } else { 0.0 }, 0.0);

    // numaMakeHistogram
    let hist_result = na.make_histogram(1000).unwrap();
    let nbins = hist_result.histogram.len();
    assert!(nbins > 0, "Histogram should have bins");
    rp.compare_values(1.0, if nbins > 0 { 1.0 } else { 0.0 }, 0.0);

    // numaGetStatsUsingHistogram
    let (minval, maxval, meanval, variance, median, rankval) =
        na.stats_using_histogram(2000, 0.80).unwrap();
    let rmsdev = (variance as f64).sqrt();

    rp.compare_values(-999.00, minval as f64, 0.1);
    rp.compare_values(999.00, maxval as f64, 0.1);
    rp.compare_values(0.055, meanval as f64, 1.0);
    rp.compare_values(0.30, median as f64, 5.0);
    rp.compare_values(706.41, rmsdev, 5.0);
    rp.compare_values(808.15, rankval as f64, 10.0);

    // histogram_rank_from_val
    let hr = na.make_histogram(2000).unwrap();
    let histo_with_params = hr.histogram;
    let rank = histo_with_params
        .histogram_rank_from_val(rankval)
        .unwrap_or(0.0);
    rp.compare_values(0.800, rank as f64, 0.02);

    assert!(rp.cleanup(), "numa1_reg histogram tests failed");
}

// ========================================================================
// Test: Basic Numa operations
// ========================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa1_reg_basic_operations() {
    let mut rp = RegParams::new("numa1_basic");

    // Numa creation
    let mut na = Numa::new();
    rp.compare_values(0.0, na.len() as f64, 0.0);
    rp.compare_values(1.0, if na.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // Push and access
    na.push(10.0);
    na.push(20.0);
    na.push(30.0);
    rp.compare_values(3.0, na.len() as f64, 0.0);
    rp.compare_values(10.0, na.get(0).unwrap() as f64, 0.0);
    rp.compare_values(20.0, na.get(1).unwrap() as f64, 0.0);
    rp.compare_values(30.0, na.get(2).unwrap() as f64, 0.0);

    // Set
    na.set(1, 25.0).unwrap();
    rp.compare_values(25.0, na.get(1).unwrap() as f64, 0.0);

    // Insert
    na.insert(0, 5.0).unwrap();
    rp.compare_values(4.0, na.len() as f64, 0.0);
    rp.compare_values(5.0, na.get(0).unwrap() as f64, 0.0);
    rp.compare_values(10.0, na.get(1).unwrap() as f64, 0.0);

    // Remove
    let removed = na.remove(0).unwrap();
    rp.compare_values(5.0, removed as f64, 0.0);
    rp.compare_values(3.0, na.len() as f64, 0.0);

    // Pop
    let popped = na.pop().unwrap();
    rp.compare_values(30.0, popped as f64, 0.0);

    // Parameters
    let mut na_params = Numa::new();
    let (startx, delx) = na_params.parameters();
    rp.compare_values(0.0, startx as f64, 0.0);
    rp.compare_values(1.0, delx as f64, 0.0);

    na_params.set_parameters(10.0, 0.5);
    let (startx, delx) = na_params.parameters();
    rp.compare_values(10.0, startx as f64, 0.0);
    rp.compare_values(0.5, delx as f64, 0.0);

    // Statistics on sin-wave data
    let pi: f32 = std::f32::consts::PI;
    let mut na_sin = Numa::with_capacity(500000);
    for i in 0..500000u32 {
        let angle = 0.02293 * (i as f32) * pi;
        let val = 999.0 * angle.sin();
        na_sin.push(val);
    }

    let (min_val, _) = na_sin.min().unwrap();
    let (max_val, _) = na_sin.max().unwrap();
    rp.compare_values(-999.0, min_val as f64, 0.01);
    rp.compare_values(999.0, max_val as f64, 0.01);

    let mean_val = na_sin.mean().unwrap();
    rp.compare_values(0.0, mean_val as f64, 0.1);

    let sum_val = na_sin.sum().unwrap();
    rp.compare_values(0.0, sum_val as f64, 50000.0);

    assert!(rp.cleanup(), "numa1_reg basic operations tests failed");
}

// ========================================================================
// Test: Histogram statistics
// ========================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa1_reg_histogram_stats() {
    let mut rp = RegParams::new("numa1_histstat");

    let pi: f32 = std::f32::consts::PI;
    let mut na = Numa::with_capacity(500000);
    for i in 0..500000u32 {
        let angle = 0.02293 * (i as f32) * pi;
        let val = 999.0 * angle.sin();
        na.push(val);
    }

    let hr = na.make_histogram(2000).unwrap();
    let histo = hr.histogram;
    let binsize = hr.binsize;
    let binstart = hr.binstart;
    let nbins = histo.len();

    let stats = histo
        .histogram_stats(binstart as f32, binsize as f32)
        .unwrap();

    // Mean should be close to 0
    rp.compare_values(0.0, stats.mean as f64, 2.0);

    // RMS deviation should be close to 706
    let rmsdev = (stats.variance as f64).sqrt();
    rp.compare_values(706.41, rmsdev, 10.0);

    // Mode should be a valid number
    rp.compare_values(
        1.0,
        if stats.mode.abs() > 0.0 || stats.mode == 0.0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Test histogram_stats_on_interval
    let center = nbins / 2;
    let range = nbins / 4;
    let ifirst = center.saturating_sub(range);
    let ilast = (center + range).min(nbins - 1);
    let interval_stats = histo
        .histogram_stats_on_interval(binstart as f32, binsize as f32, ifirst, Some(ilast))
        .unwrap();
    rp.compare_values(0.0, interval_stats.mean as f64, 200.0);

    assert!(rp.cleanup(), "numa1_reg histogram stats tests failed");
}

// ========================================================================
// Test: Histogram rank/value operations
// ========================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa1_reg_rank_operations() {
    let mut rp = RegParams::new("numa1_rank");

    // Uniform histogram: 256 bins, each with count 100
    let mut uniform_hist = Numa::from_vec(vec![100.0; 256]);
    uniform_hist.set_parameters(0.0, 1.0);

    let rank_at_0 = uniform_hist.histogram_rank_from_val(0.0).unwrap();
    rp.compare_values(0.0, rank_at_0 as f64, 0.01);

    let rank_at_128 = uniform_hist.histogram_rank_from_val(128.0).unwrap();
    rp.compare_values(0.5, rank_at_128 as f64, 0.01);

    let rank_at_256 = uniform_hist.histogram_rank_from_val(256.0).unwrap();
    rp.compare_values(1.0, rank_at_256 as f64, 0.01);

    let val_at_0 = uniform_hist.histogram_val_from_rank(0.0).unwrap();
    rp.compare_values(0.0, val_at_0 as f64, 0.5);

    let val_at_half = uniform_hist.histogram_val_from_rank(0.5).unwrap();
    rp.compare_values(128.0, val_at_half as f64, 1.0);

    let val_at_1 = uniform_hist.histogram_val_from_rank(1.0).unwrap();
    rp.compare_values(256.0, val_at_1 as f64, 1.0);

    // Roundtrip test
    let val_at_80 = uniform_hist.histogram_val_from_rank(0.80).unwrap();
    let rank_back = uniform_hist.histogram_rank_from_val(val_at_80).unwrap();
    rp.compare_values(0.80, rank_back as f64, 0.02);

    // Sin-wave histogram
    let pi: f32 = std::f32::consts::PI;
    let mut na = Numa::with_capacity(500000);
    for i in 0..500000u32 {
        let angle = 0.02293 * (i as f32) * pi;
        let val = 999.0 * angle.sin();
        na.push(val);
    }

    let hr = na.make_histogram(2000).unwrap();
    let histo_params = hr.histogram;

    let rankval = histo_params.histogram_val_from_rank(0.80).unwrap();
    rp.compare_values(808.15, rankval as f64, 10.0);

    let rank = histo_params.histogram_rank_from_val(rankval).unwrap();
    rp.compare_values(0.80, rank as f64, 0.02);

    assert!(rp.cleanup(), "numa1_reg rank operations tests failed");
}

// ========================================================================
// Test: Normalize and CDF
// ========================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa1_reg_normalize_cdf() {
    let mut rp = RegParams::new("numa1_cdf");

    let hist = Numa::from_vec(vec![10.0, 20.0, 30.0, 40.0]);

    let normalized = hist.normalize_histogram().unwrap();
    let total = normalized.sum().unwrap();
    rp.compare_values(1.0, total as f64, 0.001);

    rp.compare_values(0.1, normalized.get(0).unwrap() as f64, 0.001);
    rp.compare_values(0.2, normalized.get(1).unwrap() as f64, 0.001);
    rp.compare_values(0.3, normalized.get(2).unwrap() as f64, 0.001);
    rp.compare_values(0.4, normalized.get(3).unwrap() as f64, 0.001);

    let cdf = hist.cumulative_distribution().unwrap();
    rp.compare_values(0.1, cdf.get(0).unwrap() as f64, 0.001);
    rp.compare_values(0.3, cdf.get(1).unwrap() as f64, 0.001);
    rp.compare_values(0.6, cdf.get(2).unwrap() as f64, 0.001);
    rp.compare_values(1.0, cdf.get(3).unwrap() as f64, 0.001);

    // CDF should be monotonically non-decreasing
    for i in 1..cdf.len() {
        let prev = cdf.get(i - 1).unwrap();
        let curr = cdf.get(i).unwrap();
        assert!(curr >= prev, "CDF should be non-decreasing at index {}", i);
    }
    rp.compare_values(1.0, 1.0, 0.0);

    assert!(rp.cleanup(), "numa1_reg normalize/CDF tests failed");
}

// ========================================================================
// Test: Partial sums
// ========================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa1_reg_partial_sums() {
    let mut rp = RegParams::new("numa1_psums");

    let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let psums = na.partial_sums();

    rp.compare_values(5.0, psums.len() as f64, 0.0);
    rp.compare_values(1.0, psums.get(0).unwrap() as f64, 0.001);
    rp.compare_values(3.0, psums.get(1).unwrap() as f64, 0.001);
    rp.compare_values(6.0, psums.get(2).unwrap() as f64, 0.001);
    rp.compare_values(10.0, psums.get(3).unwrap() as f64, 0.001);
    rp.compare_values(15.0, psums.get(4).unwrap() as f64, 0.001);

    let total = na.sum().unwrap();
    let last_psum = psums.get(psums.len() - 1).unwrap();
    rp.compare_values(total as f64, last_psum as f64, 0.001);

    assert!(rp.cleanup(), "numa1_reg partial sums tests failed");
}

// ========================================================================
// Test: Make sequence
// ========================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa1_reg_make_sequence() {
    let mut rp = RegParams::new("numa1_seq");

    let seq1 = Numa::make_sequence(0.0, 1.0, 5);
    rp.compare_values(5.0, seq1.len() as f64, 0.0);
    rp.compare_values(0.0, seq1.get(0).unwrap() as f64, 0.001);
    rp.compare_values(4.0, seq1.get(4).unwrap() as f64, 0.001);

    let seq2 = Numa::make_sequence(10.0, 0.5, 6);
    rp.compare_values(6.0, seq2.len() as f64, 0.0);
    rp.compare_values(10.0, seq2.get(0).unwrap() as f64, 0.001);
    rp.compare_values(10.5, seq2.get(1).unwrap() as f64, 0.001);
    rp.compare_values(12.5, seq2.get(5).unwrap() as f64, 0.001);

    let seq3 = Numa::make_sequence(-2.0, 0.04, 51);
    rp.compare_values(51.0, seq3.len() as f64, 0.0);
    rp.compare_values(-2.0, seq3.get(0).unwrap() as f64, 0.001);
    rp.compare_values(0.0, seq3.get(50).unwrap() as f64, 0.001);

    assert!(rp.cleanup(), "numa1_reg make sequence tests failed");
}

// ========================================================================
// Test: Numaa (array of Numa)
// ========================================================================

#[test]
#[ignore = "not yet implemented"]
fn numa1_reg_numaa() {
    let mut rp = RegParams::new("numa1_numaa");

    use leptonica_core::Numaa;

    let mut naa = Numaa::new();
    rp.compare_values(0.0, naa.len() as f64, 0.0);

    let mut na1 = Numa::with_capacity(100);
    for i in 0..100 {
        na1.push((i as f32).sin());
    }
    naa.push(na1);
    rp.compare_values(1.0, naa.len() as f64, 0.0);
    rp.compare_values(100.0, naa.get(0).unwrap().len() as f64, 0.0);

    let mut na2 = Numa::with_capacity(200);
    for i in 0..200 {
        na2.push((i as f32).cos());
    }
    naa.push(na2);
    rp.compare_values(2.0, naa.len() as f64, 0.0);
    rp.compare_values(300.0, naa.total_count() as f64, 0.0);

    let flat = naa.flatten();
    rp.compare_values(300.0, flat.len() as f64, 0.0);

    let v = naa.get_value(0, 0).unwrap();
    rp.compare_values(0.0f32.sin() as f64, v as f64, 0.001);

    assert!(rp.cleanup(), "numa1_reg numaa tests failed");
}
