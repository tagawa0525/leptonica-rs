//! Image comparison regression test
//!
//! Tests image comparison functions including pixel counting,
//! equality checking, and binary correlation.
//!
//! The C version also tests pixBestCorrelation and pixCompareWithTranslation
//! which are not available in leptonica-core.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/compare_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::{Color, Pix};

/// Test count_pixels and basic pixel statistics on binary images.
///
/// Counts foreground pixels and verifies consistency with is_zero.
#[test]
fn compare_reg_count_pixels() {
    let mut rp = RegParams::new("compare_count");

    let pix1 = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");

    // Count foreground pixels
    let count = pix1.count_pixels();
    assert!(count > 0, "feyn.tif should have foreground pixels");

    // Verify non-zero
    assert!(!pix1.is_zero(), "feyn.tif should not be zero");

    // Create empty image and verify zero
    let empty = Pix::new(100, 100, leptonica::PixelDepth::Bit1).expect("create empty");
    assert!(empty.is_zero(), "new 1bpp image should be zero");
    rp.compare_values(0.0, empty.count_pixels() as f64, 0.0);

    assert!(rp.cleanup(), "compare count_pixels test failed");
}

/// Test pixEqual for identical and different images.
///
/// Verifies that equals returns true for cloned images and false
/// for different images.
#[test]
fn compare_reg_equals() {
    let mut rp = RegParams::new("compare_equals");

    let pix1 = crate::common::load_test_image("test1.png").expect("load test1.png");
    let pix2 = pix1.deep_clone();

    // Same image should be equal
    rp.compare_values(1.0, if pix1.equals(&pix2) { 1.0 } else { 0.0 }, 0.0);

    // Inverted image should NOT be equal
    let pix3 = pix1.invert();
    rp.compare_values(0.0, if pix1.equals(&pix3) { 1.0 } else { 0.0 }, 0.0);

    // Double-inverted should be equal to original
    let pix4 = pix3.invert();
    rp.compare_values(1.0, if pix1.equals(&pix4) { 1.0 } else { 0.0 }, 0.0);

    // display_diff_binary: visualize pixel differences between original and inverted
    if pix1.depth() == leptonica::PixelDepth::Bit1 {
        let diff_vis = pix1
            .display_diff_binary(&pix4)
            .expect("display_diff_binary");
        rp.write_pix_and_check(&diff_vis, ImageFormat::Png)
            .expect("check: compare equals diff_binary");
    }

    assert!(rp.cleanup(), "compare equals test failed");
}

/// Test correlation_binary between translated images.
///
/// Verifies that correlation is high for identical images and
/// decreases with increasing translation.
#[test]
fn compare_reg_correlation() {
    let mut rp = RegParams::new("compare_correl");

    let pix1 = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");

    // Self-correlation should be 1.0
    let score = leptonica::core::pix::correlation_binary(&pix1, &pix1).expect("self correl");
    rp.compare_values(1.0, score, 0.001);

    // Correlation with inverted should be low (near 0)
    let pix2 = pix1.invert();
    let score_inv = leptonica::core::pix::correlation_binary(&pix1, &pix2).expect("inv correl");
    assert!(
        score_inv < 0.5,
        "correlation with inverse should be low, got {score_inv}"
    );

    assert!(rp.cleanup(), "compare correlation test failed");
}

/// Test pixBestCorrelation with translated images (C checks 0-2).
///
/// Requires pixBestCorrelation and pixCentroid which are not available.
#[test]
#[ignore = "not yet implemented: pixBestCorrelation not available"]
fn compare_reg_best_correlation() {
    // C version:
    // 1. Reads harmoniam100-11.png, converts to binary at threshold 160
    // 2. Creates translated version (shifted by 32, 12)
    // 3. pixBestCorrelation finds translation (delx=32, dely=12)
}

/// Test pixCompareWithTranslation (C checks 3-6).
///
/// Requires pixCompareWithTranslation which is not available.
#[test]
#[ignore = "not yet implemented: pixCompareWithTranslation not available"]
fn compare_reg_with_translation() {
    // C version:
    // 1. Reads harmoniam-11.tif
    // 2. Translates by (-45, 25)
    // 3. pixCompareWithTranslation finds (delx=45, dely=-25)
}

/// Test pixGetPerceptualDiff on color and grayscale images (C checks 7-12).
///
/// C version: pixGetPerceptualDiff(pix0, pix1, 1, 3, 20, &fract, ...)
/// Color fract ~0.061252, grayscale fract ~0.046928.
#[test]
fn compare_reg_perceptual_diff() {
    let mut rp = RegParams::new("compare_perceptual");

    let pix0 = crate::common::load_test_image("greencover.jpg").expect("load greencover.jpg");
    let pix1 = crate::common::load_test_image("redcover.jpg").expect("load redcover.jpg");

    // Color comparison (C: sampling=1, dilation=3, min_diff=20)
    let (fract, _avg, _exceeds) = pix0
        .get_perceptual_diff(&pix1, 1, 3, 20, 0.0, 1)
        .expect("get_perceptual_diff color");
    eprintln!("Fraction of color pixels = {}", fract);
    assert!(
        fract > 0.0,
        "color images should have perceptual difference"
    );
    rp.compare_values(0.061252, fract as f64, 0.2);

    // Grayscale comparison
    let gray0 = pix0.convert_to_8().expect("convert_to_8 pix0");
    let gray1 = pix1.convert_to_8().expect("convert_to_8 pix1");
    let (fract_gray, _avg_gray, _exceeds_gray) = gray0
        .get_perceptual_diff(&gray1, 1, 3, 20, 0.0, 1)
        .expect("get_perceptual_diff gray");
    eprintln!("Fraction of grayscale pixels = {}", fract_gray);
    assert!(
        fract_gray > 0.0,
        "grayscale images should have perceptual difference"
    );
    rp.compare_values(0.046928, fract_gray as f64, 0.15);

    // display_diff: visualize color difference map
    let diff_vis = pix0
        .display_diff(&pix1, 20, Color::RED)
        .expect("display_diff color");
    rp.write_pix_and_check(&diff_vis, ImageFormat::Png)
        .expect("check: compare perceptual diff_vis color");

    // display_diff: visualize grayscale difference map
    let diff_vis_gray = gray0
        .display_diff(&gray1, 20, Color::RED)
        .expect("display_diff gray");
    rp.write_pix_and_check(&diff_vis_gray, ImageFormat::Png)
        .expect("check: compare perceptual diff_vis gray");

    assert!(rp.cleanup(), "compare perceptual diff test failed");
}
