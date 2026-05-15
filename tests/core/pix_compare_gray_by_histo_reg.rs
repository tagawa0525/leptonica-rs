//! Regression tests for pix_compare_gray_by_histo (plan 143).

use leptonica::core::pix::pix_compare_gray_by_histo;
use leptonica::{PixMut, PixelDepth};

fn solid_gray(w: u32, h: u32, val: u32) -> leptonica::Pix {
    let mut pm = PixMut::new(w, h, PixelDepth::Bit8).unwrap();
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel(x, y, val).unwrap();
        }
    }
    pm.into()
}

fn shifted_gray(w: u32, h: u32, peak_val: u32) -> leptonica::Pix {
    // Half left = peak_val, half right = 50.
    let mut pm = PixMut::new(w, h, PixelDepth::Bit8).unwrap();
    for y in 0..h {
        for x in 0..w {
            let v = if x < w / 2 { peak_val } else { 50 };
            pm.set_pixel(x, y, v).unwrap();
        }
    }
    pm.into()
}

#[test]
fn pix_compare_gray_identical_images_score_high() {
    // Two identical mid-gray images → score should be close to 1.
    let pix1 = solid_gray(80, 80, 128);
    let pix2 = solid_gray(80, 80, 128);
    let score = pix_compare_gray_by_histo(&pix1, &pix2, None, None, 0.9, 230, 1, 4).unwrap();
    assert!(
        score > 0.95,
        "expected near-1 score for identical images, got {score}"
    );
}

#[test]
fn pix_compare_gray_different_images_score_low() {
    // Same shape but very different histograms.
    let pix1 = shifted_gray(80, 80, 30);
    let pix2 = shifted_gray(80, 80, 220);
    let score = pix_compare_gray_by_histo(&pix1, &pix2, None, None, 0.9, 230, 1, 4).unwrap();
    assert!(
        score < 0.5,
        "expected low score for very different images, got {score}"
    );
}

#[test]
fn pix_compare_gray_size_filter_returns_zero() {
    // 40×80 vs 80×80 → wratio = 0.5 < 0.9 → 0.0.
    let pix1 = solid_gray(40, 80, 128);
    let pix2 = solid_gray(80, 80, 128);
    let score = pix_compare_gray_by_histo(&pix1, &pix2, None, None, 0.9, 230, 1, 4).unwrap();
    assert!(score.abs() < 1e-3, "expected size-filter 0, got {score}");
}

#[test]
fn pix_compare_gray_rejects_invalid_params() {
    let pix1 = solid_gray(80, 80, 128);
    let pix2 = solid_gray(80, 80, 128);
    // minratio out of [0.5, 1.0]
    assert!(pix_compare_gray_by_histo(&pix1, &pix2, None, None, 0.4, 230, 1, 4).is_err());
    assert!(pix_compare_gray_by_histo(&pix1, &pix2, None, None, 1.1, 230, 1, 4).is_err());
    // maxgray < 200
    assert!(pix_compare_gray_by_histo(&pix1, &pix2, None, None, 0.9, 100, 1, 4).is_err());
    // factor = 0
    assert!(pix_compare_gray_by_histo(&pix1, &pix2, None, None, 0.9, 230, 0, 4).is_err());
}

#[test]
fn pix_compare_gray_accepts_box_inputs() {
    // Same image, but a box-clipped subregion. Should still score high.
    let pix1 = solid_gray(120, 120, 128);
    let pix2 = solid_gray(120, 120, 128);
    let box1 = leptonica::core::Box::new(10, 10, 80, 80).unwrap();
    let box2 = leptonica::core::Box::new(10, 10, 80, 80).unwrap();
    let score =
        pix_compare_gray_by_histo(&pix1, &pix2, Some(&box1), Some(&box2), 0.9, 230, 1, 4).unwrap();
    assert!(score > 0.95);
}

#[test]
fn pix_compare_gray_clamps_n_silently() {
    // n = 100 (out of [1, 7]) should be clamped to 4 silently → succeeds.
    let pix1 = solid_gray(80, 80, 128);
    let pix2 = solid_gray(80, 80, 128);
    let r = pix_compare_gray_by_histo(&pix1, &pix2, None, None, 0.9, 230, 1, 100);
    assert!(r.is_ok());
}

#[test]
fn pix_compare_gray_handles_all_white_after_maxgray_clip() {
    // 8 bpp image of pure white (255). With maxgray = 200, all mass is
    // clipped, leaving both tile histograms empty. Both tiles are then
    // "identical empty" and the score should be high (~1.0), not 0.
    let white1 = solid_gray(80, 80, 255);
    let white2 = solid_gray(80, 80, 255);
    let score = pix_compare_gray_by_histo(&white1, &white2, None, None, 0.9, 200, 1, 4).unwrap();
    assert!(
        score > 0.95,
        "expected ~1.0 for identical empty histograms, got {score}"
    );
}

#[test]
fn pix_compare_gray_handles_negative_box_origin() {
    // Box with negative origin should clip to the image rather than panicking
    // or returning a giant u32 value.
    let pix1 = solid_gray(100, 100, 128);
    let pix2 = solid_gray(100, 100, 128);
    let box1 = leptonica::core::Box::new(-10, -10, 80, 80).unwrap();
    let box2 = leptonica::core::Box::new(-10, -10, 80, 80).unwrap();
    let score =
        pix_compare_gray_by_histo(&pix1, &pix2, Some(&box1), Some(&box2), 0.9, 230, 1, 4).unwrap();
    assert!(
        score > 0.9,
        "negative-origin box should clip not panic, got {score}"
    );
}
