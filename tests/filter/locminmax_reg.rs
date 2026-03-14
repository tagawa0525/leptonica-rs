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
//! C Leptonica: `prog/locminmax_reg.c`

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
#[test]
fn locminmax_reg_extrema() {
    use leptonica::region::local_extrema;

    let mut rp = RegParams::new("locminmax_extrema");

    let pix8 = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    // Convert to 8bpp if needed
    let pix8 = if pix8.depth() == leptonica::PixelDepth::Bit8 {
        pix8
    } else {
        pix8.convert_rgb_to_gray_fast().expect("convert to 8bpp")
    };

    // Smooth with blockconv before extrema detection (C: pixBlockconv(pix2, 10, 10))
    let smoothed = blockconv_gray(&pix8, None, 10, 10).expect("blockconv_gray 10x10");

    // C: pixLocalExtrema(pixs, 50, 50, &pix_min, &pix_max)
    // min_max_size must be odd
    let (pix_min, pix_max) = local_extrema(&smoothed, 51, 50).expect("local_extrema");

    // Both masks should be 1bpp
    rp.compare_values(1.0, pix_min.depth().bits() as f64, 0.0);
    rp.compare_values(1.0, pix_max.depth().bits() as f64, 0.0);

    // Masks should have same dimensions as smoothed image
    rp.compare_values(smoothed.width() as f64, pix_min.width() as f64, 0.0);
    rp.compare_values(smoothed.height() as f64, pix_min.height() as f64, 0.0);
    rp.compare_values(smoothed.width() as f64, pix_max.width() as f64, 0.0);
    rp.compare_values(smoothed.height() as f64, pix_max.height() as f64, 0.0);

    assert!(rp.cleanup(), "locminmax_extrema test failed");
}
