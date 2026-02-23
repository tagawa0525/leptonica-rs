//! Local min/max regression test
//!
//! Tests detection of local extrema (minima and maxima) in grayscale images.
//! The C version uses pixLocalExtrema to find pixels that are locally
//! minimum or maximum, then overlays them on the smoothed source image.
//!
//! Not yet migrated: pixLocalExtrema is in leptonica-region. The related
//! pixBlockconv (smoothing before extrema detection) is available in
//! leptonica-filter and tested here.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/locminmax_reg.c`

use crate::common::RegParams;
use leptonica::filter::blockconv_gray;

/// Test blockconv smoothing as preprocessing for local extrema (C check 0).
///
/// Verifies that blockconv produces a smoothed 8bpp image at original dimensions.
/// In C, pixBlockconv is used to smooth before calling pixLocalExtrema.
#[test]
fn locminmax_reg_blockconv_smooth() {
    let mut rp = RegParams::new("locminmax_smooth");

    let pix8 = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix8.width();
    let h = pix8.height();

    // C: pixBlockconv(pix2, 10, 10) for smoothing before extrema detection
    let smoothed = blockconv_gray(&pix8, None, 10, 10).expect("blockconv_gray 10x10");
    rp.compare_values(w as f64, smoothed.width() as f64, 0.0);
    rp.compare_values(h as f64, smoothed.height() as f64, 0.0);

    // Smoothed image should be 8bpp
    assert_eq!(smoothed.depth(), leptonica::PixelDepth::Bit8);

    assert!(rp.cleanup(), "locminmax blockconv smooth test failed");
}

/// Test pixLocalExtrema for local minima/maxima detection (C checks 1-2).
///
/// Requires pixLocalExtrema which is in leptonica-region, not leptonica-filter.
#[test]
#[ignore = "not yet implemented: pixLocalExtrema is in leptonica-region"]
fn locminmax_reg_extrema() {
    // C version:
    // 1. Smooth image with pixBlockconv(pix2, 10, 10)
    // 2. pixLocalExtrema(pixs, minmax=50, maxmin=50, &pix1, &pix2)
    //    pix1 = local minima mask, pix2 = local maxima mask
    // 3. pixPaintThroughMask to overlay extrema on smoothed image
    // 4. regTestWritePixAndCheck for result
}
