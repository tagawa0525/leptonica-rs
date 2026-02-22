//! Image comparison regression test
//!
//! Tests image comparison functions including pixel counting,
//! equality checking, and binary correlation.
//!
//! The C version also tests pixBestCorrelation, pixCompareWithTranslation,
//! and pixGetPerceptualDiff which are not available in leptonica-core.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/compare_reg.c`

use leptonica_core::Pix;
use leptonica_test::RegParams;

/// Test count_pixels and basic pixel statistics on binary images.
///
/// Counts foreground pixels and verifies consistency with is_zero.
#[test]
#[ignore = "not yet implemented: requires golden file generation"]
fn compare_reg_count_pixels() {
    let mut rp = RegParams::new("compare_count");

    let pix1 = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");

    // Count foreground pixels
    let count = pix1.count_pixels();
    assert!(count > 0, "feyn.tif should have foreground pixels");

    // Verify non-zero
    assert!(!pix1.is_zero(), "feyn.tif should not be zero");

    // Create empty image and verify zero
    let empty = Pix::new(100, 100, leptonica_core::PixelDepth::Bit1).expect("create empty");
    assert!(empty.is_zero(), "new 1bpp image should be zero");
    rp.compare_values(0.0, empty.count_pixels() as f64, 0.0);

    assert!(rp.cleanup(), "compare count_pixels test failed");
}

/// Test pixEqual for identical and different images.
///
/// Verifies that equals returns true for cloned images and false
/// for different images.
#[test]
#[ignore = "not yet implemented: requires golden file generation"]
fn compare_reg_equals() {
    let mut rp = RegParams::new("compare_equals");

    let pix1 = leptonica_test::load_test_image("test1.png").expect("load test1.png");
    let pix2 = pix1.deep_clone();

    // Same image should be equal
    rp.compare_values(1.0, if pix1.equals(&pix2) { 1.0 } else { 0.0 }, 0.0);

    // Inverted image should NOT be equal
    let pix3 = pix1.invert();
    rp.compare_values(0.0, if pix1.equals(&pix3) { 1.0 } else { 0.0 }, 0.0);

    // Double-inverted should be equal to original
    let pix4 = pix3.invert();
    rp.compare_values(1.0, if pix1.equals(&pix4) { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "compare equals test failed");
}

/// Test correlation_binary between translated images.
///
/// Verifies that correlation is high for identical images and
/// decreases with increasing translation.
#[test]
#[ignore = "not yet implemented: requires golden file generation"]
fn compare_reg_correlation() {
    let mut rp = RegParams::new("compare_correl");

    let pix1 = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");

    // Self-correlation should be 1.0
    let score = leptonica_core::pix::correlation_binary(&pix1, &pix1).expect("self correl");
    rp.compare_values(1.0, score, 0.001);

    // Correlation with inverted should be low (near 0 or negative)
    let pix2 = pix1.invert();
    let score_inv = leptonica_core::pix::correlation_binary(&pix1, &pix2).expect("inv correl");
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
/// Requires pixGetPerceptualDiff which is not available.
#[test]
#[ignore = "not yet implemented: pixGetPerceptualDiff not available"]
fn compare_reg_perceptual_diff() {
    // C version:
    // 1. Reads greencover.jpg and redcover.jpg
    // 2. Compares with pixGetPerceptualDiff (color: fract ~0.061252)
    // 3. Converts to grayscale and compares (gray: fract ~0.046928)
}
