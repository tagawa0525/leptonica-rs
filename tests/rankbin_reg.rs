//! Rank bin regression test
//!
//! Tests rank filter operations and rank bin analysis. The C version tests
//! numaGetRankBinValues, numaDiscretizeSortedInBins, numaDiscretizeHistoInBins,
//! and pixRankBinByStrip on color and grayscale images.
//!
//! Partial migration: rank_filter and median_filter are available and tested.
//! numaGetRankBinValues and pixRankBinByStrip are not available in leptonica-filter.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/rankbin_reg.c`

mod common;
use common::RegParams;
use leptonica::filter::{max_filter, median_filter, min_filter, rank_filter_gray};

/// Test rank_filter_gray: basic and extreme ranks (C ranks 0/1 approx).
///
/// Verifies rank filter at 0.0 (min), 0.5 (median), 1.0 (max) approximates
/// the corresponding min/median/max filters.
#[test]
fn rankbin_reg_rank_filter_gray() {
    let mut rp = RegParams::new("rankbin_gray");

    let pix = common::load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let w = pix.width();
    let h = pix.height();

    // Rank = 0.0 (min) should match min_filter
    let rank_min = rank_filter_gray(&pix, 7, 7, 0.0001).expect("rank_filter min");
    let min = min_filter(&pix, 7, 7).expect("min_filter");
    rp.compare_values(w as f64, rank_min.width() as f64, 0.0);
    rp.compare_values(h as f64, rank_min.height() as f64, 0.0);

    // Rank = 0.5 (median) should match median_filter
    let rank_med = rank_filter_gray(&pix, 7, 7, 0.5).expect("rank_filter median");
    let median = median_filter(&pix, 7, 7).expect("median_filter");
    rp.compare_values(w as f64, rank_med.width() as f64, 0.0);

    // Rank = 1.0 (max) should match max_filter
    let rank_max = rank_filter_gray(&pix, 7, 7, 0.9999).expect("rank_filter max");
    let max = max_filter(&pix, 7, 7).expect("max_filter");
    rp.compare_values(w as f64, rank_max.width() as f64, 0.0);

    // Verify rank_min ≈ min (center pixel should be close)
    let rmin_px = rank_min.get_pixel(w / 2, h / 2).unwrap_or(0);
    let min_px = min.get_pixel(w / 2, h / 2).unwrap_or(0);
    rp.compare_values(rmin_px as f64, min_px as f64, 2.0);

    // Verify rank_med ≈ median
    let rmed_px = rank_med.get_pixel(w / 2, h / 2).unwrap_or(0);
    let med_px = median.get_pixel(w / 2, h / 2).unwrap_or(0);
    rp.compare_values(rmed_px as f64, med_px as f64, 2.0);

    // Verify rank_max ≈ max
    let rmax_px = rank_max.get_pixel(w / 2, h / 2).unwrap_or(0);
    let max_px = max.get_pixel(w / 2, h / 2).unwrap_or(0);
    rp.compare_values(rmax_px as f64, max_px as f64, 2.0);

    assert!(rp.cleanup(), "rankbin rank_filter_gray test failed");
}

/// Test numaGetRankBinValues and pixRankBinByStrip (C checks 0-4).
///
/// Requires numaGetRankBinValues (Numa operation in leptonica-core) and
/// pixRankBinByStrip which is not exported from leptonica-filter.
#[test]
#[ignore = "not yet implemented: numaGetRankBinValues in core, pixRankBinByStrip not available"]
fn rankbin_reg_bin_values() {
    // C version:
    // 1. numaGetRankBinValues(na1, nbins=10, &na3) – discretize into rank bins
    // 2. numaGetRankBinValues(na2, nbins=30, &na4) – 30 bins
    // 3. pixRankBinByStrip(pix1, L_SCAN_HORIZONTAL, 16, 10, L_SELECT_HUE)
    // 4. pixRankBinByStrip with L_SELECT_SATURATION and L_SELECT_RED
}

/// Test numaDiscretize functions (C checks 5-10).
///
/// Requires numaDiscretizeSortedInBins and numaDiscretizeHistoInBins
/// which are Numa operations in leptonica-core.
#[test]
#[ignore = "not yet implemented: numaDiscretizeSortedInBins/HistoInBins not available"]
fn rankbin_reg_discretize() {
    // C version:
    // 1. numaDiscretizeSortedInBins(na1, nbins, &na2) – bin sorted data
    // 2. numaDiscretizeHistoInBins(na1, nbins, &na2) – bin histogram data
    // 3. Verify equivalence of both methods
}
