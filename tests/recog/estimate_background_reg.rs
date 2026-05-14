//! Regression tests for plan 129 (recog::estimate_background).

use leptonica::recog::pageseg::estimate_background;
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

#[test]
fn estimate_background_solid_gray() {
    let pix = solid_gray(40, 40, 200);
    let bg = estimate_background(&pix, 0, 0.0).unwrap();
    // Allow ±1 to absorb histogram-bin rounding in rank_value_masked.
    assert!(
        (199..=201).contains(&bg),
        "expected background near 200, got {bg}"
    );
}

#[test]
fn estimate_background_with_darkthresh_excludes_dark() {
    // Top half: dark (gray=50), bottom half: light (gray=220).
    // With darkthresh=100, the dark region is masked out, so the
    // background should reflect the light region (~220).
    let mut pm = PixMut::new(40, 40, PixelDepth::Bit8).unwrap();
    for y in 0..40 {
        let v = if y < 20 { 50 } else { 220 };
        for x in 0..40 {
            pm.set_pixel(x, y, v).unwrap();
        }
    }
    let pix: leptonica::Pix = pm.into();
    let bg = estimate_background(&pix, 100, 0.0).unwrap();
    assert!(
        (219..=221).contains(&bg),
        "expected background near 220, got {bg}"
    );
}

#[test]
fn estimate_background_rejects_non_8bpp() {
    let pix: leptonica::Pix = PixMut::new(40, 40, PixelDepth::Bit1).unwrap().into();
    assert!(estimate_background(&pix, 0, 0.0).is_err());
    let pix32: leptonica::Pix = PixMut::new(40, 40, PixelDepth::Bit32).unwrap().into();
    assert!(estimate_background(&pix32, 0, 0.0).is_err());
}

#[test]
fn estimate_background_rejects_invalid_edgecrop() {
    let pix = solid_gray(40, 40, 128);
    assert!(estimate_background(&pix, 0, -0.1).is_err());
    assert!(estimate_background(&pix, 0, 1.0).is_err());
    assert!(estimate_background(&pix, 0, 1.5).is_err());
}

#[test]
fn estimate_background_edgecrop_constrains_to_inner() {
    // Outer border = dark (50), inner = light (220). With edgecrop = 0.5,
    // only the inner half is sampled and the background should be 220.
    let mut pm = PixMut::new(40, 40, PixelDepth::Bit8).unwrap();
    for y in 0..40 {
        for x in 0..40 {
            let inner = (10..30).contains(&x) && (10..30).contains(&y);
            pm.set_pixel(x, y, if inner { 220 } else { 50 }).unwrap();
        }
    }
    let pix: leptonica::Pix = pm.into();
    let bg = estimate_background(&pix, 0, 0.5).unwrap();
    assert!(
        (219..=221).contains(&bg),
        "expected background near 220, got {bg}"
    );
}
