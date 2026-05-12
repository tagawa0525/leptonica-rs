//! Regression tests for plan 107 (pixafunc1/2.c の Pixa 変換 8 関数).

use leptonica::core::Box;
use leptonica::core::pix::rop::InColor;
use leptonica::{Pix, Pixa, PixelDepth};

fn make_pixa(items: &[(u32, u32, PixelDepth)]) -> Pixa {
    let mut pa = Pixa::with_capacity(items.len());
    for (i, &(w, h, d)) in items.iter().enumerate() {
        let pix = Pix::new(w, h, d).unwrap();
        let b = Box::new(i as i32 * 10, i as i32 * 5, w as i32, h as i32).unwrap();
        pa.push_with_box(pix, b);
    }
    pa
}

// -- scale ---------------------------------------------------------------

#[test]
fn pixa_scale_doubles_dimensions() {
    let pa = make_pixa(&[(10, 10, PixelDepth::Bit8), (20, 10, PixelDepth::Bit8)]);
    let out = pa.scale(2.0, 2.0).unwrap();
    assert_eq!(out.pix_slice().len(), 2);
    assert_eq!(out.pix_slice()[0].width(), 20);
    assert_eq!(out.pix_slice()[1].width(), 40);
    // Boxes should also be scaled.
    let b1 = out.boxa().get(1).unwrap();
    assert_eq!(b1.w, 40);
}

#[test]
fn pixa_scale_invalid_factor_errors() {
    let pa = make_pixa(&[(10, 10, PixelDepth::Bit8)]);
    assert!(pa.scale(0.0, 1.0).is_err());
    assert!(pa.scale(1.0, -1.0).is_err());
}

#[test]
fn pixa_scale_empty_returns_empty() {
    let pa = Pixa::new();
    let out = pa.scale(2.0, 2.0).unwrap();
    assert!(out.pix_slice().is_empty());
}

// -- scale_by_sampling ----------------------------------------------------

#[test]
fn pixa_scale_by_sampling_halves() {
    let pa = make_pixa(&[(20, 20, PixelDepth::Bit8)]);
    let out = pa.scale_by_sampling(0.5, 0.5).unwrap();
    assert_eq!(out.pix_slice()[0].width(), 10);
    assert_eq!(out.pix_slice()[0].height(), 10);
}

// -- rotate_orth ----------------------------------------------------------

#[test]
fn pixa_rotate_orth_0_returns_clone() {
    let pa = make_pixa(&[(10, 6, PixelDepth::Bit8)]);
    let out = pa.rotate_orth(0).unwrap();
    assert_eq!(out.pix_slice()[0].width(), 10);
    assert_eq!(out.pix_slice()[0].height(), 6);
}

#[test]
fn pixa_rotate_orth_90_swaps_dims() {
    let pa = make_pixa(&[(10, 6, PixelDepth::Bit8)]);
    let out = pa.rotate_orth(1).unwrap();
    assert_eq!(out.pix_slice()[0].width(), 6);
    assert_eq!(out.pix_slice()[0].height(), 10);
}

#[test]
fn pixa_rotate_orth_out_of_range_errors() {
    let pa = make_pixa(&[(10, 6, PixelDepth::Bit8)]);
    assert!(pa.rotate_orth(4).is_err());
}

// -- translate ------------------------------------------------------------

#[test]
fn pixa_translate_zero_shift_returns_clone() {
    let pa = make_pixa(&[(8, 8, PixelDepth::Bit8)]);
    let out = pa.translate(0, 0, InColor::White).unwrap();
    assert_eq!(out.pix_slice()[0].width(), 8);
}

#[test]
fn pixa_translate_shifts_box() {
    let pa = make_pixa(&[(8, 8, PixelDepth::Bit8)]);
    let out = pa.translate(3, 4, InColor::White).unwrap();
    let b = out.boxa().get(0).unwrap();
    // make_pixa places Box[0] at (0, 0, 8, 8) → after (3, 4) shift → (3, 4, 8, 8).
    assert_eq!((b.x, b.y, b.w, b.h), (3, 4, 8, 8));
}

// -- convert_to_1 / convert_to_8 / convert_to_32 -------------------------

#[test]
fn pixa_convert_to_1() {
    let pa = make_pixa(&[(8, 8, PixelDepth::Bit8), (4, 4, PixelDepth::Bit8)]);
    let out = pa.convert_to_1(128).unwrap();
    assert!(
        out.pix_slice()
            .iter()
            .all(|p| p.depth() == PixelDepth::Bit1)
    );
}

#[test]
fn pixa_convert_to_8() {
    let pa = make_pixa(&[(8, 8, PixelDepth::Bit1), (4, 4, PixelDepth::Bit32)]);
    let out = pa.convert_to_8(false).unwrap();
    assert!(
        out.pix_slice()
            .iter()
            .all(|p| p.depth() == PixelDepth::Bit8)
    );
}

#[test]
fn pixa_convert_to_32() {
    let pa = make_pixa(&[(8, 8, PixelDepth::Bit1), (4, 4, PixelDepth::Bit8)]);
    let out = pa.convert_to_32().unwrap();
    assert!(
        out.pix_slice()
            .iter()
            .all(|p| p.depth() == PixelDepth::Bit32)
    );
}
