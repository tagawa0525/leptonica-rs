//! Regression tests for plan 120 (Pixa transform 残り 4 関数).

use leptonica::core::Box;
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

// -- add_border_general -------------------------------------------------

#[test]
fn add_border_general_grows_pix_and_shifts_box() {
    let pa = make_pixa(&[(8, 6, PixelDepth::Bit8)]);
    let out = pa.add_border_general(2, 3, 1, 4, 0).unwrap();
    assert_eq!(out.pix_slice()[0].width(), 8 + 2 + 3);
    assert_eq!(out.pix_slice()[0].height(), 6 + 1 + 4);
    let b = out.boxa().get(0).unwrap();
    assert_eq!((b.x, b.y), (-2, -1));
    assert_eq!((b.w, b.h), (8, 6)); // original size, only origin shifted
}

#[test]
fn add_border_general_zero_border_is_clone() {
    let pa = make_pixa(&[(4, 4, PixelDepth::Bit8)]);
    let out = pa.add_border_general(0, 0, 0, 0, 0).unwrap();
    assert_eq!(out.pix_slice()[0].width(), 4);
    let b = out.boxa().get(0).unwrap();
    assert_eq!((b.x, b.y, b.w, b.h), (0, 0, 4, 4));
}

// -- clip_to_foreground_all ---------------------------------------------

#[test]
fn clip_to_foreground_all_with_fg() {
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    m.set_pixel(3, 4, 1).unwrap();
    m.set_pixel(5, 6, 1).unwrap();
    let pix: Pix = m.into();
    let mut pa = Pixa::new();
    pa.push(pix);
    let (pixa, boxa) = pa.clip_to_foreground_all().unwrap();
    assert_eq!(pixa.pix_slice().len(), 1);
    assert_eq!(boxa.len(), 1);
    let b = boxa.get(0).unwrap();
    // Bounding box of (3,4) and (5,6) is (3,4,3,3)
    assert_eq!((b.x, b.y, b.w, b.h), (3, 4, 3, 3));
}

#[test]
fn clip_to_foreground_all_empty_fg_keeps_pix() {
    let pix = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
    let mut pa = Pixa::new();
    pa.push(pix);
    let (pixa, boxa) = pa.clip_to_foreground_all().unwrap();
    assert_eq!(pixa.pix_slice().len(), 1);
    let b = boxa.get(0).unwrap();
    // No FG → Box covers whole image.
    assert_eq!((b.x, b.y, b.w, b.h), (0, 0, 8, 8));
}

// -- convert_to_given_depth --------------------------------------------

#[test]
fn convert_to_given_depth_8() {
    let pa = make_pixa(&[(4, 4, PixelDepth::Bit1), (4, 4, PixelDepth::Bit32)]);
    let out = pa.convert_to_given_depth(8).unwrap();
    assert!(
        out.pix_slice()
            .iter()
            .all(|p| p.depth() == PixelDepth::Bit8)
    );
}

#[test]
fn convert_to_given_depth_32() {
    let pa = make_pixa(&[(4, 4, PixelDepth::Bit1), (4, 4, PixelDepth::Bit8)]);
    let out = pa.convert_to_given_depth(32).unwrap();
    assert!(
        out.pix_slice()
            .iter()
            .all(|p| p.depth() == PixelDepth::Bit32)
    );
}

#[test]
fn convert_to_given_depth_invalid_errors() {
    let pa = make_pixa(&[(4, 4, PixelDepth::Bit8)]);
    assert!(pa.convert_to_given_depth(1).is_err());
    assert!(pa.convert_to_given_depth(16).is_err());
}

// -- convert_to_same_depth --------------------------------------------

#[test]
fn convert_to_same_depth_uniform_passthrough() {
    let pa = make_pixa(&[(4, 4, PixelDepth::Bit8), (4, 4, PixelDepth::Bit8)]);
    let out = pa.convert_to_same_depth().unwrap();
    assert!(
        out.pix_slice()
            .iter()
            .all(|p| p.depth() == PixelDepth::Bit8)
    );
}

#[test]
fn convert_to_same_depth_mixed_promotes_to_8() {
    let pa = make_pixa(&[(4, 4, PixelDepth::Bit1), (4, 4, PixelDepth::Bit8)]);
    let out = pa.convert_to_same_depth().unwrap();
    assert!(
        out.pix_slice()
            .iter()
            .all(|p| p.depth() == PixelDepth::Bit8)
    );
}

#[test]
fn convert_to_same_depth_mixed_promotes_to_32_with_color() {
    let pa = make_pixa(&[(4, 4, PixelDepth::Bit8), (4, 4, PixelDepth::Bit32)]);
    let out = pa.convert_to_same_depth().unwrap();
    assert!(
        out.pix_slice()
            .iter()
            .all(|p| p.depth() == PixelDepth::Bit32)
    );
}

#[test]
fn convert_to_same_depth_empty_errors() {
    let pa = Pixa::new();
    assert!(pa.convert_to_same_depth().is_err());
}
