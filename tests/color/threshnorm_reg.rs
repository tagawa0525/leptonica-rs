//! Threshold normalization regression test
//!
//! Tests adaptive threshold normalization for binarization.
//! The C version uses pixThresholdSpreadNorm to normalize illumination
//! then applies pixThresholdToBinary at various thresholds.
//!
//! Partial migration: threshold_to_binary at various thresholds is tested
//! as the inner loop of the C test. pixThresholdSpreadNorm is not available.
//! Test image stampede2.jpg is not available; test8.jpg is used instead.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/threshnorm_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;

/// Test threshold_to_binary at multiple thresholds (partial C AddTestSet).
///
/// The C inner function applies pixThresholdToBinary at targetthresh-20,
/// targetthresh, targetthresh+20, targetthresh+40. We test the same
/// threshold sweep on raw 8bpp gray.
#[test]
fn threshnorm_reg_threshold_sweep() {
    let mut rp = RegParams::new("threshnorm_sweep");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // C: pix3 = pixThresholdToBinary(pix2, targetthresh - 20);
    // Sweep thresholds around target=128 as in C AddTestSet
    let thresholds: &[u8] = &[108, 128, 148, 168];
    for &thresh in thresholds {
        let result = threshold_to_binary(&pix, thresh)
            .unwrap_or_else(|_| panic!("threshold_to_binary at {thresh}"));
        rp.compare_values(w as f64, result.width() as f64, 0.0);
        rp.compare_values(h as f64, result.height() as f64, 0.0);
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    assert!(rp.cleanup(), "threshnorm threshold sweep test failed");
}

/// Test pixThresholdSpreadNorm (C checks 0: full normalization pipeline).
///
/// Requires pixThresholdSpreadNorm which is not available in the Rust API.
/// Test image stampede2.jpg is also not available.
#[test]
#[ignore = "not yet implemented: pixThresholdSpreadNorm not available"]
fn threshnorm_reg_spread_norm() {
    // C version:
    // pixThresholdSpreadNorm(pixs, L_SOBEL_EDGE, 18, 40, 40, 0.7, -25, 280, 128,
    //                        &pix1, NULL, &pix2);
    // pix3 = pixThresholdToBinary(pix2, targetthresh);
}
