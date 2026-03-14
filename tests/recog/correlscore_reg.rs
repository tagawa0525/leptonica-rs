//! Correlation score regression test
//!
//! Tests binary image correlation scoring for character recognition.
//!
//! # See also
//!
//! C Leptonica: `prog/correlscore_reg.c`

use leptonica::core::{Pix, PixelDepth};
use leptonica::recog::correlscore;

/// Test correlation_score_simple with identical images.
#[test]
fn correlscore_identical() {
    let mut pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap().to_mut();
    // Draw a small block
    for y in 5..15 {
        for x in 5..15 {
            pix.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pix.into();

    let score = correlscore::correlation_score_simple(&pix, &pix, 0, 0).unwrap();
    // Identical images should have correlation = 1.0
    assert!((score - 1.0).abs() < 0.01, "expected ~1.0, got {score}");
}

/// Test correlation_score_simple with disjoint images.
#[test]
fn correlscore_disjoint() {
    let mut pix1 = Pix::new(40, 20, PixelDepth::Bit1).unwrap().to_mut();
    let mut pix2 = Pix::new(40, 20, PixelDepth::Bit1).unwrap().to_mut();

    // pix1: left block
    for y in 5..15 {
        for x in 0..10 {
            pix1.set_pixel(x, y, 1).unwrap();
        }
    }
    // pix2: right block
    for y in 5..15 {
        for x in 30..40 {
            pix2.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix1: Pix = pix1.into();
    let pix2: Pix = pix2.into();

    let score = correlscore::correlation_score_simple(&pix1, &pix2, 0, 0).unwrap();
    assert!(
        score < 0.01,
        "disjoint images should have near-zero correlation, got {score}"
    );
}

/// Test correlation_score with precomputed areas.
#[test]
fn correlscore_with_areas() {
    let mut pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap().to_mut();
    for y in 5..15 {
        for x in 5..15 {
            pix.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pix.into();
    let area = 100; // 10×10 block

    let score = correlscore::correlation_score(&pix, &pix, area, area, 0, 0).unwrap();
    assert!((score - 1.0).abs() < 0.01);
}

/// Test correlation_score_thresholded.
#[test]
fn correlscore_thresholded() {
    let mut pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap().to_mut();
    for y in 5..15 {
        for x in 5..15 {
            pix.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pix.into();

    let above =
        correlscore::correlation_score_thresholded(&pix, &pix, 100, 100, 0, 0, 0.5).unwrap();
    assert!(above, "identical images should exceed 0.5 threshold");

    let below =
        correlscore::correlation_score_thresholded(&pix, &pix, 100, 100, 0, 0, 1.5).unwrap();
    assert!(!below, "no correlation can exceed 1.5");
}

/// Test correlation_score_shifted.
#[test]
fn correlscore_shifted() {
    let mut pix = Pix::new(30, 30, PixelDepth::Bit1).unwrap().to_mut();
    for y in 10..20 {
        for x in 10..20 {
            pix.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pix.into();

    // With max_shift=0, should get perfect correlation
    let score0 = correlscore::correlation_score_shifted(&pix, &pix, 100, 100, 0, 0, 0).unwrap();
    assert!((score0 - 1.0).abs() < 0.01);

    // With offset and shift, should still find good match
    let score_shift =
        correlscore::correlation_score_shifted(&pix, &pix, 100, 100, 2, 2, 3).unwrap();
    assert!(score_shift > 0.5, "shifted search should find good match");
}
