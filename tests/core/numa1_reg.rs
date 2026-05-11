//! Numa (numeric array) operations regression test
//!
//! Tests histograms, interpolation, integration/differentiation on Numa.
//!
//! # See also
//!
//! C Leptonica: `prog/numa1_reg.c`

use crate::common::RegParams;
use leptonica::Numa;

// ========================================================================
// Test: Histograms (C tests 0-10)
// ========================================================================

#[test]

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

fn numa1_reg_numaa() {
    let mut rp = RegParams::new("numa1_numaa");

    use leptonica::Numaa;

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

// =====================================================================
// gap-fill 第2弾 (plan 501): parseStringForNumbers
// =====================================================================

/// C: parseStringForNumbers(str, seps) — basic whitespace-delimited parse
#[test]

fn numa1_reg_parse_from_string_basic() {
    let na = Numa::parse_from_string("1.5 -2 6.25", " ").expect("parse_from_string");
    assert_eq!(na.len(), 3);
    assert!((na.get(0).unwrap() - 1.5).abs() < 1e-6);
    assert!((na.get(1).unwrap() - (-2.0)).abs() < 1e-6);
    assert!((na.get(2).unwrap() - 6.25).abs() < 1e-5);
}

/// C: parseStringForNumbers — comma-separated values
#[test]

fn numa1_reg_parse_from_string_csv() {
    let na = Numa::parse_from_string("1,2,3", ",").expect("parse csv");
    assert_eq!(na.len(), 3);
    assert!((na.get(0).unwrap() - 1.0).abs() < 1e-6);
    assert!((na.get(1).unwrap() - 2.0).abs() < 1e-6);
    assert!((na.get(2).unwrap() - 3.0).abs() < 1e-6);
}

/// C atof tolerates whitespace; ensure tokens with surrounding spaces parse
/// when the separator is e.g. just `,`.
#[test]
fn numa1_reg_parse_from_string_csv_with_spaces() {
    let na = Numa::parse_from_string("1, 2, 3", ",").expect("parse 'a, b, c'");
    assert_eq!(na.len(), 3);
    assert!((na.get(0).unwrap() - 1.0).abs() < 1e-6);
    assert!((na.get(1).unwrap() - 2.0).abs() < 1e-6);
    assert!((na.get(2).unwrap() - 3.0).abs() < 1e-6);
}

/// C: parseStringForNumbers — multiple separators (space + tab + newline)
#[test]

fn numa1_reg_parse_from_string_mixed_separators() {
    let na = Numa::parse_from_string("1\t2\n3 4", " \t\n").expect("parse mixed");
    assert_eq!(na.len(), 4);
    for i in 0..4 {
        assert!((na.get(i).unwrap() - (i as f32 + 1.0)).abs() < 1e-6);
    }
}

// =====================================================================
// gap-fill 第2弾 (plan 116): numabasic.c 拡張 5 関数
// =====================================================================

use leptonica::core::numa::{NumaSarrayType, Numaa};

#[test]
fn numa1_reg_create_from_string() {
    let na = leptonica::Numa::create_from_string("1.5, -2, 6.25").expect("create_from_string");
    assert_eq!(na.len(), 3);
    assert!((na.get(0).unwrap() - 1.5).abs() < 1e-6);
    assert!((na.get(1).unwrap() - (-2.0)).abs() < 1e-6);
    assert!((na.get(2).unwrap() - 6.25).abs() < 1e-6);
}

#[test]
fn numa1_reg_create_from_string_empty_errors() {
    assert!(leptonica::Numa::create_from_string("").is_err());
}

#[test]
fn numa1_reg_copy_parameters() {
    let mut a = leptonica::Numa::new();
    let mut b = leptonica::Numa::new();
    b.set_parameters(10.0, 0.5);
    a.copy_parameters(&b);
    let (s, d) = a.parameters();
    assert!((s - 10.0).abs() < 1e-6);
    assert!((d - 0.5).abs() < 1e-6);
}

#[test]
fn numa1_reg_convert_to_sarray_integer() {
    let na = leptonica::Numa::from_slice(&[1.0, 23.0, 4.0]);
    let sa = na.convert_to_sarray(3, 0, true, NumaSarrayType::Integer);
    assert_eq!(sa.len(), 3);
    assert_eq!(sa.get(0).unwrap(), "001");
    assert_eq!(sa.get(1).unwrap(), "023");
    assert_eq!(sa.get(2).unwrap(), "004");
}

#[test]
fn numa1_reg_convert_to_sarray_integer_negative_padding() {
    // Sign-aware zero pad: matches C printf("%03d", -2) = "-02"
    let na = leptonica::Numa::from_slice(&[-2.0, -45.0]);
    let sa = na.convert_to_sarray(3, 0, true, NumaSarrayType::Integer);
    assert_eq!(sa.get(0).unwrap(), "-02");
    assert_eq!(sa.get(1).unwrap(), "-45");
}

#[test]
fn numa1_reg_convert_to_sarray_float() {
    let na = leptonica::Numa::from_slice(&[1.5, -2.25]);
    let sa = na.convert_to_sarray(8, 3, false, NumaSarrayType::Float);
    assert_eq!(sa.len(), 2);
    // 8 wide, 3 decimal places, e.g. "   1.500"
    assert_eq!(sa.get(0).unwrap(), "   1.500");
    assert_eq!(sa.get(1).unwrap(), "  -2.250");
}

#[test]
fn numa1_reg_numaa_create_full() {
    let naa = Numaa::create_full(4, 8);
    assert_eq!(naa.len(), 4);
    for i in 0..4usize {
        let na = naa.get(i).expect("get");
        assert_eq!(na.len(), 0);
    }
    // total_count() == 0 (numaaGetNumberCount equivalent)
    assert_eq!(naa.total_count(), 0);
}

#[test]
fn numa1_reg_numaa_total_count_matches_get_number_count() {
    let mut naa = Numaa::create_full(3, 0);
    naa.get_mut(0).unwrap().push(1.0);
    naa.get_mut(0).unwrap().push(2.0);
    naa.get_mut(2).unwrap().push(3.0);
    // C: numaaGetNumberCount returns 3
    assert_eq!(naa.total_count(), 3);
}

// =====================================================================
// gap-fill 第2弾 (plan 501/116) — 境界条件テスト
// =====================================================================

#[test]
fn numa1_reg_parse_from_string_empty_separators_errors() {
    assert!(leptonica::Numa::parse_from_string("1 2 3", "").is_err());
}

#[test]
fn numa1_reg_parse_from_string_non_numeric_errors() {
    let r = leptonica::Numa::parse_from_string("1, hello, 3", ",");
    assert!(r.is_err(), "non-numeric token must error");
}

#[test]
fn numa1_reg_create_from_string_non_numeric_errors() {
    assert!(leptonica::Numa::create_from_string("1, abc, 3").is_err());
}

#[test]
fn numa1_reg_convert_to_sarray_width_zero() {
    // Format width 0 — values render at natural width
    let na = leptonica::Numa::from_slice(&[5.0, 100.0]);
    let sa = na.convert_to_sarray(0, 0, false, NumaSarrayType::Integer);
    assert_eq!(sa.get(0).unwrap(), "5");
    assert_eq!(sa.get(1).unwrap(), "100");
}
