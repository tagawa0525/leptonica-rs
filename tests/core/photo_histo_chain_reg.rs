//! Regression tests for the photo-histo chain (plan 144):
//! pix_decide_if_photo_image / pix_gen_photo_histos /
//! pix_compare_photo_regions_by_histo / pixa_compare_photo_regions_by_histo.
//!
//! These tests exercise the API surface rather than the algorithmic
//! verdict (which depends on tile statistics).

use leptonica::core::pix::{
    pix_compare_photo_regions_by_histo, pix_decide_if_photo_image, pix_gen_photo_histos,
    pixa_compare_photo_regions_by_histo,
};
use leptonica::{Pix, PixMut, Pixa, PixelDepth};

fn solid_gray(w: u32, h: u32, val: u32) -> Pix {
    let mut pm = PixMut::new(w, h, PixelDepth::Bit8).unwrap();
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel(x, y, val).unwrap();
        }
    }
    pm.into()
}

fn random_gray(w: u32, h: u32, seed: u32) -> Pix {
    // Simple LCG-based pseudo-random pattern that has varied histogram
    // across tiles → "photo-like".
    let mut state = seed;
    let mut pm = PixMut::new(w, h, PixelDepth::Bit8).unwrap();
    for y in 0..h {
        for x in 0..w {
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            pm.set_pixel(x, y, (state >> 16) & 0xff).unwrap();
        }
    }
    pm.into()
}

// -- pix_decide_if_photo_image --------------------------------------------

#[test]
fn decide_if_photo_image_rejects_non_8bpp() {
    let pix: Pix = PixMut::new(40, 40, PixelDepth::Bit1).unwrap().into();
    assert!(pix_decide_if_photo_image(&pix, 1, 1.3, 4).is_err());
}

#[test]
fn decide_if_photo_image_returns_naa_or_none() {
    // A pseudo-random gray image should produce *some* result; we don't
    // assert isphoto since the verdict depends on the histogram stats,
    // but the function must succeed without panicking.
    let pix = random_gray(120, 120, 0xCAFEBABE);
    let r = pix_decide_if_photo_image(&pix, 1, 1.3, 4);
    assert!(r.is_ok());
}

#[test]
fn decide_if_photo_image_clamps_n_silently() {
    let pix = solid_gray(80, 80, 128);
    let r = pix_decide_if_photo_image(&pix, 1, 1.3, 100);
    assert!(r.is_ok());
}

// -- pix_gen_photo_histos -----------------------------------------------

#[test]
fn pix_gen_photo_histos_rejects_1bpp() {
    let pix: Pix = PixMut::new(80, 80, PixelDepth::Bit1).unwrap().into();
    assert!(pix_gen_photo_histos(&pix, None, 1, 0.0, 4).is_err());
}

#[test]
fn pix_gen_photo_histos_rejects_factor_zero() {
    let pix = solid_gray(80, 80, 128);
    assert!(pix_gen_photo_histos(&pix, None, 0, 0.0, 4).is_err());
}

#[test]
fn pix_gen_photo_histos_returns_dimensions() {
    // Any valid input should return a positive padded width/height.
    let pix = random_gray(120, 80, 42);
    let (_, w, h) = pix_gen_photo_histos(&pix, None, 1, 0.0, 4).unwrap();
    assert!(w > 0 && h > 0);
}

#[test]
fn pix_gen_photo_histos_accepts_box() {
    let pix = random_gray(200, 200, 7);
    let b = leptonica::core::Box::new(20, 20, 100, 100).unwrap();
    let (_, w, h) = pix_gen_photo_histos(&pix, Some(&b), 1, 0.0, 4).unwrap();
    assert!(w >= 100 && h >= 100);
}

// -- pix_compare_photo_regions_by_histo ----------------------------------

#[test]
fn pix_compare_photo_regions_size_filter_returns_zero() {
    let pix1 = solid_gray(40, 80, 128);
    let pix2 = solid_gray(80, 80, 128);
    let score = pix_compare_photo_regions_by_histo(&pix1, &pix2, None, None, 0.9, 1, 4).unwrap();
    assert!(score.abs() < 1e-3);
}

#[test]
fn pix_compare_photo_regions_rejects_invalid_minratio() {
    let pix1 = solid_gray(80, 80, 128);
    let pix2 = solid_gray(80, 80, 128);
    assert!(pix_compare_photo_regions_by_histo(&pix1, &pix2, None, None, 0.3, 1, 4).is_err());
    assert!(pix_compare_photo_regions_by_histo(&pix1, &pix2, None, None, 1.5, 1, 4).is_err());
}

#[test]
fn pix_compare_photo_regions_succeeds_on_random_pair() {
    // Two pseudo-random images. Some non-zero score (or 0 if either fails
    // the photo check) — we just verify no panic.
    let pix1 = random_gray(120, 120, 1);
    let pix2 = random_gray(120, 120, 2);
    let r = pix_compare_photo_regions_by_histo(&pix1, &pix2, None, None, 0.9, 1, 4);
    assert!(r.is_ok());
}

// -- pixa_compare_photo_regions_by_histo ---------------------------------

#[test]
fn pixa_compare_photo_regions_returns_n_squared_scores() {
    let mut pixa = Pixa::new();
    for i in 0..3u32 {
        pixa.push(random_gray(80, 80, i));
    }
    let (classes, scores) = pixa_compare_photo_regions_by_histo(&pixa, 0.9, 1, 4, 0.25).unwrap();
    assert_eq!(classes.len(), 3);
    assert_eq!(scores.len(), 9);
    // Diagonal must be 1.0.
    for i in 0..3 {
        assert!((scores[i * 3 + i] - 1.0).abs() < 1e-4);
    }
}

#[test]
fn pixa_compare_photo_regions_identical_pixs_share_class() {
    let mut pixa = Pixa::new();
    let pix = random_gray(120, 120, 99);
    pixa.push(pix.clone());
    pixa.push(pix.clone());
    pixa.push(pix);
    let (classes, _) = pixa_compare_photo_regions_by_histo(&pixa, 0.9, 1, 4, 0.25).unwrap();
    assert_eq!(classes.len(), 3);
    // All three identical images must end up in the same class.
    assert!(
        classes[0] == classes[1] && classes[1] == classes[2],
        "expected one class, got {classes:?}"
    );
}

#[test]
fn pixa_compare_photo_regions_rejects_invalid_minratio() {
    let pixa = Pixa::new();
    assert!(pixa_compare_photo_regions_by_histo(&pixa, -0.1, 1, 4, 0.25).is_err());
    assert!(pixa_compare_photo_regions_by_histo(&pixa, 1.5, 1, 4, 0.25).is_err());
}
