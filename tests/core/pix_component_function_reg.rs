//! Regression tests for pix_component_function (plan 141).

use leptonica::core::fpix::pix_component_function;
use leptonica::core::pixel::compose_rgba;
use leptonica::{PixMut, PixelDepth};

fn make_rgb(w: u32, h: u32, r: u8, g: u8, b: u8) -> leptonica::Pix {
    let mut pm = PixMut::new(w, h, PixelDepth::Bit32).unwrap();
    let px = compose_rgba(r, g, b, 0);
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel(x, y, px).unwrap();
        }
    }
    pm.into()
}

#[test]
fn pix_component_function_zero_denom_returns_numerator() {
    // r=100, g=50, b=25. (rnum, gnum, bnum) = (1, 1, 1) → sum = 175.
    let pix = make_rgb(4, 4, 100, 50, 25);
    let fp = pix_component_function(&pix, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0).unwrap();
    for y in 0..4 {
        for x in 0..4 {
            let v = fp.get_pixel(x, y).unwrap();
            assert!((v - 175.0).abs() < 1e-3);
        }
    }
}

#[test]
fn pix_component_function_one_denom_r_uses_reciprocal_table() {
    // r=80, g=120, b=60. (rnum, gnum, bnum) = (0, 1, 0).
    // (rdenom, gdenom, bdenom) = (1, 0, 0) → fast path: g / r = 120/80 = 1.5.
    let pix = make_rgb(4, 4, 80, 120, 60);
    let fp = pix_component_function(&pix, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0).unwrap();
    let v = fp.get_pixel(2, 2).unwrap();
    assert!(
        (v - 1.5).abs() < 1e-3,
        "expected 1.5 (g/r = 120/80), got {v}"
    );
}

#[test]
fn pix_component_function_one_denom_r_zero_uses_sentinel() {
    // r = 0 → reciprocal table returns 256.0 (large sentinel). num = g = 30.
    // val = 256 * 30 = 7680.
    let pix = make_rgb(4, 4, 0, 30, 50);
    let fp = pix_component_function(&pix, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0).unwrap();
    let v = fp.get_pixel(0, 0).unwrap();
    assert!(
        (v - 7680.0).abs() < 1e-3,
        "expected 7680 (256*30 sentinel for r=0), got {v}"
    );
}

#[test]
fn pix_component_function_general_case_returns_ratio() {
    // r=100, g=200, b=50. (rnum, gnum, bnum) = (1, 0, 0),
    // (rdenom, gdenom, bdenom) = (0, 1, 0.5). Output = r / (g + 0.5*b)
    //   = 100 / (200 + 25) = 100 / 225 ≈ 0.4444.
    let pix = make_rgb(4, 4, 100, 200, 50);
    let fp = pix_component_function(&pix, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5).unwrap();
    let v = fp.get_pixel(2, 2).unwrap();
    assert!(
        (v - 100.0 / 225.0).abs() < 1e-3,
        "expected 100/225, got {v}"
    );
}

#[test]
fn pix_component_function_general_case_zero_denom_uses_256_sentinel() {
    // r=0, g=0, b=0 with general-case denom (no fast path) → fnum=0, fdenom=0.
    // Output = 256.0 * fnum = 0.0 (because fnum is 0).
    let pix = make_rgb(4, 4, 0, 0, 0);
    let fp = pix_component_function(&pix, 1.0, 0.5, 0.25, 0.5, 1.0, 0.5).unwrap();
    let v = fp.get_pixel(0, 0).unwrap();
    assert!(v.abs() < 1e-3);

    // Now with non-zero fnum but zero fdenom: r=10, g=0, b=0,
    // (rnum=0, gnum=0, bnum=1), (rdenom=0, gdenom=1, bdenom=0.5)
    //   → fnum = 0, fdenom = 0 + 0 + 0 = 0 → 0.
    // To force fnum != 0, use rnum=1: fnum = 10, fdenom = 0 + 0 + 0 = 0 → 256*10 = 2560.
    let pix2 = make_rgb(4, 4, 10, 0, 0);
    let fp2 = pix_component_function(&pix2, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5).unwrap();
    let v2 = fp2.get_pixel(0, 0).unwrap();
    assert!((v2 - 2560.0).abs() < 1e-3, "expected 256*10=2560, got {v2}");
}

#[test]
fn pix_component_function_rejects_non_32bpp() {
    let pix: leptonica::Pix = PixMut::new(4, 4, PixelDepth::Bit8).unwrap().into();
    assert!(pix_component_function(&pix, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0).is_err());
}
