//! Numa advanced operations regression test
//!
//! Tests sorted insertion, windowed mean, histogram statistics,
//! and similarity comparison.
//!
//! The C version tests histogram-based rank extraction, numa morphological
//! operations (erode/dilate/open/close), windowed mean smoothing,
//! and sorted insertion with verification.
//! This Rust port tests available Numa operations: sorted insertion,
//! windowed mean, histogram statistics, and similarity checking.
//! Rank extraction, morphology, and threshold finding are not yet available.
//!
//! # See also
//!
//! C Leptonica: `prog/numa3_reg.c`

use crate::common::RegParams;
use leptonica::Numa;
use leptonica::core::numa::SortOrder;

/// Test sorted insertion and sort verification (C checks 11-12).
///
/// Inserts random values into a sorted Numa via add_sorted,
/// then verifies the result matches a sorted-then-reversed version.
#[test]
fn numa3_reg_sorted_insertion() {
    let mut rp = RegParams::new("numa3_sorted");

    // Test 1: Insert into decreasing order (like C srand(5) test)
    let mut na1 = Numa::new();
    na1.push(27.0);
    na1.push(13.0);
    // Use a deterministic pseudo-random sequence
    let vals = [
        142, 17, 93, 55, 186, 31, 77, 120, 4, 168, 62, 99, 145, 38, 111, 87, 159, 23, 71, 133, 49,
        105, 176, 12, 84, 156, 66, 128, 41, 97, 189, 8, 74, 146, 53, 118, 35, 163, 81, 107, 19,
        140, 58, 123, 91, 171, 26, 68, 151, 44, 115, 79, 195, 14, 100, 137, 61, 180, 33, 89, 153,
        47, 109, 72, 166, 21, 83, 144, 56, 126,
    ];
    for &v in &vals {
        na1.add_sorted(v as f32).expect("add_sorted");
    }

    // Verify it's sorted: clone, sort increasing, reverse, compare
    let mut na2 = na1.clone();
    na2.sort(SortOrder::Increasing);
    // After sort(Increasing), na2 is ascending. na1 (from add_sorted starting
    // with [27, 13]) is in descending order. So reverse na2 to compare.
    let reversed: Vec<f32> = (0..na2.len()).rev().map(|i| na2[i]).collect();
    let mut na2_rev = Numa::new();
    for &v in &reversed {
        na2_rev.push(v);
    }
    let same = na1.similar(&na2_rev, 0.0);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);

    // Test 2: Insert into increasing order
    let mut na3 = Numa::new();
    na3.push(13.0);
    na3.push(27.0);
    let vals2 = [
        58, 173, 91, 22, 145, 67, 114, 39, 186, 78, 103, 51, 161, 84, 129, 16, 140, 63, 97, 175,
        34, 108, 72, 155, 45, 119, 88, 166, 28, 99, 53, 138, 76, 112, 41, 183, 65, 127, 93, 159,
        19, 106, 48, 171, 82, 134, 56, 118, 37, 148, 69, 100, 190, 24, 85, 143, 61, 176, 43, 110,
        74, 157, 31, 96, 126, 55, 180, 87, 152, 14,
    ];
    for &v in &vals2 {
        na3.add_sorted(v as f32).expect("add_sorted");
    }

    // na3 starts with [13, 27] so add_sorted should produce ascending order
    let mut na4 = na3.clone();
    na4.sort(SortOrder::Decreasing);
    // After sort(Decreasing), na4 is descending. Reverse to get ascending.
    let reversed4: Vec<f32> = (0..na4.len()).rev().map(|i| na4[i]).collect();
    let mut na4_rev = Numa::new();
    for &v in &reversed4 {
        na4_rev.push(v);
    }
    let same4 = na3.similar(&na4_rev, 0.0);
    rp.compare_values(1.0, if same4 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "numa3 sorted insertion test failed");
}

/// Test windowed mean smoothing.
///
/// Creates a noisy signal, applies windowed mean, and verifies
/// the smoothed output is less variable than the input.
#[test]
fn numa3_reg_windowed_mean() {
    let mut rp = RegParams::new("numa3_windowed");

    // Create a sine wave with noise
    let mut na = Numa::new();
    for i in 0..200 {
        let base = (i as f64 * std::f64::consts::PI / 50.0).sin();
        // Deterministic "noise" via simple hash
        let noise = ((i * 7 + 13) % 20) as f64 / 100.0 - 0.1;
        na.push((base + noise) as f32);
    }
    rp.compare_values(200.0, na.len() as f64, 0.0);

    // Apply windowed mean with halfwin=5
    let smoothed = na.windowed_mean(5);
    rp.compare_values(200.0, smoothed.len() as f64, 0.0);

    // Smoothed signal should have lower variance
    let orig_var: f64 = {
        let mean: f64 = (0..na.len()).map(|i| na[i] as f64).sum::<f64>() / na.len() as f64;
        (0..na.len())
            .map(|i| (na[i] as f64 - mean).powi(2))
            .sum::<f64>()
            / na.len() as f64
    };
    let smooth_var: f64 = {
        let mean: f64 =
            (0..smoothed.len()).map(|i| smoothed[i] as f64).sum::<f64>() / smoothed.len() as f64;
        (0..smoothed.len())
            .map(|i| (smoothed[i] as f64 - mean).powi(2))
            .sum::<f64>()
            / smoothed.len() as f64
    };
    rp.compare_values(1.0, if smooth_var < orig_var { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "numa3 windowed mean test failed");
}

/// Test Numa histogram and statistics.
///
/// Creates a Numa, computes histogram, verifies basic statistics.
#[test]
fn numa3_reg_histogram() {
    let mut rp = RegParams::new("numa3_histogram");

    // Create a Numa with known distribution
    let mut na = Numa::new();
    // Values concentrated around 100 and 200 (bimodal)
    for _ in 0..50 {
        na.push(100.0);
    }
    for _ in 0..50 {
        na.push(200.0);
    }
    rp.compare_values(100.0, na.len() as f64, 0.0);

    // Compute histogram
    let hist_result = na.make_histogram(256).expect("make_histogram");
    let hist = &hist_result.histogram;

    // Histogram should have non-zero entries
    rp.compare_values(1.0, if !hist.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // Verify total count sums to the number of input values
    let total_count: f64 = (0..hist.len()).map(|i| hist[i] as f64).sum();
    rp.compare_values(na.len() as f64, total_count, 0.0);

    // Verify the two peaks: bins for values 100 and 200 should each have count 50
    let bin100 = ((100 - hist_result.binstart) / hist_result.binsize) as usize;
    let bin200 = ((200 - hist_result.binstart) / hist_result.binsize) as usize;
    rp.compare_values(50.0, hist[bin100] as f64, 0.0);
    rp.compare_values(50.0, hist[bin200] as f64, 0.0);

    // Compute histogram stats
    let stats = hist.histogram_stats(hist_result.binstart as f32, hist_result.binsize as f32);
    if let Some(stats) = stats {
        // Mean should be near 150 (average of 100 and 200)
        rp.compare_values(150.0, stats.mean as f64, 1.0);
    } else {
        // Stats not available, record failure
        rp.compare_values(1.0, 0.0, 0.0);
    }

    assert!(rp.cleanup(), "numa3 histogram test failed");
}

/// Test Numa similarity comparison.
///
/// Verifies that identical arrays are similar and different arrays are not.
#[test]
fn numa3_reg_similar() {
    let mut rp = RegParams::new("numa3_similar");

    let mut na1 = Numa::new();
    let mut na2 = Numa::new();
    for i in 0..20 {
        na1.push(i as f32 * 10.0);
        na2.push(i as f32 * 10.0);
    }

    // Identical arrays should be similar with max_diff=0
    let same = na1.similar(&na2, 0.0);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);

    // Modify one element
    let mut na3 = Numa::new();
    for i in 0..20 {
        if i == 10 {
            na3.push(999.0);
        } else {
            na3.push(i as f32 * 10.0);
        }
    }

    // Different arrays should not be similar with max_diff=0
    let diff = na1.similar(&na3, 0.0);
    rp.compare_values(0.0, if diff { 1.0 } else { 0.0 }, 0.0);

    // But with large enough max_diff, they should be similar
    let diff_large = na1.similar(&na3, 1000.0);
    rp.compare_values(1.0, if diff_large { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "numa3 similar test failed");
}

/// Test rank extraction (C checks 0-1).
///
/// Requires pixGetGrayHistogramMasked, numaMakeRankFromHistogram,
/// numaHistogramGetValFromRank which are not available.
#[test]
#[ignore = "not yet implemented: rank extraction APIs not available"]
fn numa3_reg_rank_extraction() {
    // C version:
    // 1. Get masked gray histogram from test8.jpg
    // 2. Make rank function from histogram
    // 3. Extract rank values point by point
}

/// Test numa-morphology operations (C checks 2-6).
///
/// Creates a sine waveform and verifies erode/dilate/open/close morphological
/// properties: dilated >= original >= eroded at each point.
#[test]
fn numa3_reg_morphology() {
    let mut rp = RegParams::new("numa3_morphology");

    let n = 200usize;
    let mut na = Numa::new();
    for i in 0..n {
        na.push((i as f32 * 0.1).sin());
    }

    let ne = na.erode(21).expect("erode");
    let nd = na.dilate(21).expect("dilate");

    // dilated >= original >= eroded at each point
    let erode_le_orig = (0..n).all(|i| ne[i] <= na[i] + 1e-5);
    let dilate_ge_orig = (0..n).all(|i| nd[i] >= na[i] - 1e-5);
    rp.compare_values(1.0, if erode_le_orig { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if dilate_ge_orig { 1.0 } else { 0.0 }, 0.0);

    // open <= dilate, close >= erode at each point
    let no = na.open(21).expect("open");
    let nc = na.close(21).expect("close");
    let open_le_dilate = (0..n).all(|i| no[i] <= nd[i] + 1e-5);
    let close_ge_erode = (0..n).all(|i| nc[i] >= ne[i] - 1e-5);
    rp.compare_values(1.0, if open_le_dilate { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if close_ge_erode { 1.0 } else { 0.0 }, 0.0);

    // transform: shift=1.0, scale=2.0 → each value becomes 2*(x+1)
    let nt = na.transform(1.0, 2.0);
    rp.compare_values((2.0 * (na[0] + 1.0)) as f64, nt[0] as f64, 1e-4);

    assert!(rp.cleanup(), "numa3 morphology test failed");
}

/// Test threshold finding from histogram (C checks 7-10).
///
/// Builds a bimodal histogram with peaks at bins 50 and 180,
/// normalizes it with transform, then finds the valley threshold
/// between the two peaks using find_loc_for_threshold.
#[test]
fn numa3_reg_threshold_finding() {
    let mut rp = RegParams::new("numa3_threshold");

    // Bimodal distribution: two Gaussian peaks (around 50 and 180)
    let mut na = Numa::new();
    for i in 0..256usize {
        let x = i as f32;
        let peak1 = (-(x - 50.0).powi(2) / 200.0).exp() * 100.0;
        let peak2 = (-(x - 180.0).powi(2) / 200.0).exp() * 60.0;
        na.push(peak1 + peak2);
    }

    // Normalize with transform (divide by sum to make total equal to 1)
    let sum: f32 = (0..na.len()).map(|i| na[i]).sum();
    let nt = na.transform(0.0, 1.0 / sum);

    // Threshold should be detected between the two peaks (50 and 180)
    let (thresh, frac) = nt
        .find_loc_for_threshold(0)
        .expect("find_loc_for_threshold");
    let in_range = thresh > 80 && thresh < 160;
    rp.compare_values(1.0, if in_range { 1.0 } else { 0.0 }, 0.0);

    // frac: fraction below first peak (0.0 to 1.0)
    rp.compare_values(
        1.0,
        if (0.0..=1.0).contains(&frac) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "numa3 threshold finding test failed");
}
