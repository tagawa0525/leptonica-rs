//! Threshold normalization regression test
//!
//! Tests adaptive threshold normalization for binarization.
//! The C version uses pixThresholdSpreadNorm to normalize illumination
//! then applies pixThresholdToBinary at various thresholds.
//!
//! Partial migration: threshold_to_binary at various thresholds and
//! threshold_spread_norm on stampede2.jpg are tested.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/threshnorm_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::filter::{EdgeFilterType, threshold_spread_norm};
use leptonica::io::ImageFormat;

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
        if thresh == 128 {
            rp.write_pix_and_check(&result, ImageFormat::Tiff)
                .expect("write result threshold_sweep");
        }
    }

    assert!(rp.cleanup(), "threshnorm threshold sweep test failed");
}

/// Test pixThresholdSpreadNorm (C checks 0: full normalization pipeline).
///
/// Applies threshold_spread_norm to stampede2.jpg then binarizes the result.
#[test]
fn threshnorm_reg_spread_norm() {
    let mut rp = RegParams::new("threshnorm_spread");

    let pix = crate::common::load_test_image("stampede2.jpg").expect("load stampede2.jpg");
    let pix8 = pix.convert_to_8().expect("convert to 8bpp");
    let w = pix8.width();
    let h = pix8.height();

    // C: pixThresholdSpreadNorm(pixs, L_SOBEL_EDGE, 18, 40, 40, 0.7, ...)
    let norm = threshold_spread_norm(&pix8, EdgeFilterType::Sobel, 18, 40, 40, 0.7)
        .expect("threshold_spread_norm");
    rp.compare_values(w as f64, norm.width() as f64, 0.0);
    rp.compare_values(h as f64, norm.height() as f64, 0.0);
    assert_eq!(norm.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&norm, ImageFormat::Png)
        .expect("write normed spread_norm");

    // C: pix3 = pixThresholdToBinary(pix2, targetthresh=128);
    let binary = threshold_to_binary(&norm, 128).expect("threshold_to_binary");
    rp.compare_values(w as f64, binary.width() as f64, 0.0);
    rp.compare_values(h as f64, binary.height() as f64, 0.0);
    assert_eq!(binary.depth(), PixelDepth::Bit1);

    assert!(rp.cleanup(), "threshnorm spread_norm test failed");
}
