//! Rank histogram regression test
//!
//! Tests rank color array extraction and color mapping. The C version uses
//! pixGetRankColorArray to extract representative colors at rank positions,
//! pixLinearMapToTargetColor for background lightening, and pixGammaTRC
//! for final gamma correction.
//!
//! Partial migration: rank_filter and gamma_trc_pix are available.
//! pixGetRankColorArray and pixLinearMapToTargetColor are not in leptonica-filter.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/rankhisto_reg.c`

use crate::common::RegParams;
use leptonica::filter::{gamma_trc_pix, rank_filter, rank_filter_color};

/// Test rank_filter on 32bpp color image (C rank filter portion).
///
/// Verifies rank_filter works on 32bpp image and produces valid output.
#[test]
fn rankhisto_reg_rank_filter_color() {
    let mut rp = RegParams::new("rankhisto_color");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();

    // Median filter on color image
    let median_color = rank_filter_color(&pix, 5, 5, 0.5).expect("rank_filter_color median");
    rp.compare_values(w as f64, median_color.width() as f64, 0.0);
    rp.compare_values(h as f64, median_color.height() as f64, 0.0);
    assert_eq!(median_color.depth(), leptonica::PixelDepth::Bit32);

    // Min filter on color image
    let min_color = rank_filter_color(&pix, 5, 5, 0.0001).expect("rank_filter_color min");
    rp.compare_values(w as f64, min_color.width() as f64, 0.0);

    // General rank_filter dispatch (auto-selects color or gray)
    let rank_result = rank_filter(&pix, 3, 3, 0.5).expect("rank_filter color dispatch");
    rp.compare_values(w as f64, rank_result.width() as f64, 0.0);

    assert!(rp.cleanup(), "rankhisto rank_filter_color test failed");
}

/// Test gamma_trc_pix on a rank-filtered result (C check 3 gamma step).
///
/// Verifies that gamma correction can be applied after rank filtering.
#[test]
fn rankhisto_reg_gamma_on_filtered() {
    let mut rp = RegParams::new("rankhisto_gamma");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();

    // Apply median rank filter, then gamma correction (C: gamma=1.0, range 0..240)
    let filtered = rank_filter_color(&pix, 7, 7, 0.5).expect("rank_filter_color 7x7");
    let corrected = gamma_trc_pix(&filtered, 1.0, 0, 240).expect("gamma_trc 0..240");
    rp.compare_values(w as f64, corrected.width() as f64, 0.0);
    rp.compare_values(h as f64, corrected.height() as f64, 0.0);
    assert_eq!(corrected.depth(), leptonica::PixelDepth::Bit32);

    assert!(rp.cleanup(), "rankhisto gamma on filtered test failed");
}

/// Test pixGetRankColorArray and color mapping (C checks 0-4).
///
/// Requires pixGetRankColorArray and pixLinearMapToTargetColor which are
/// not available in leptonica-filter.
#[test]
#[ignore = "not yet implemented: pixGetRankColorArray/pixLinearMapToTargetColor not available"]
fn rankhisto_reg_rank_color_array() {
    // C version:
    // 1. pixGetRankColorArray(pixs, nbins=20, L_SELECT_MIN, factor=2, &array, pixa, 6)
    // 2. pixDisplayColorArray to visualize rank colors
    // 3. pixLinearMapToTargetColor to lighten image background
    // 4. pixGammaTRC(NULL, pix1, 1.0, 0, 240) for gamma correction
}
