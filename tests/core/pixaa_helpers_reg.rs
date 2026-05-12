//! Regression tests for plan 122 (Pixaa helpers 3 関数).

use leptonica::core::pixa::Pixaa;
use leptonica::{Pix, Pixa, PixelDepth};

fn make_inner(items: &[(u32, u32)]) -> Pixa {
    let mut pa = Pixa::with_capacity(items.len());
    for &(w, h) in items {
        pa.push(Pix::new(w, h, PixelDepth::Bit8).unwrap());
    }
    pa
}

// -- flatten_to_pixa ----------------------------------------------------

#[test]
fn flatten_to_pixa_no_index() {
    let mut paa = Pixaa::new();
    paa.push(make_inner(&[(2, 2), (3, 3)]));
    paa.push(make_inner(&[(4, 4)]));
    let (pixa, na) = paa.flatten_to_pixa(false);
    assert_eq!(pixa.pix_slice().len(), 3);
    assert!(na.is_none());
}

#[test]
fn flatten_to_pixa_with_index() {
    let mut paa = Pixaa::new();
    paa.push(make_inner(&[(2, 2), (3, 3)])); // index 0
    paa.push(make_inner(&[(4, 4)])); // index 1
    paa.push(make_inner(&[(5, 5), (6, 6)])); // index 2
    let (pixa, na) = paa.flatten_to_pixa(true);
    assert_eq!(pixa.pix_slice().len(), 5);
    let na = na.unwrap();
    assert_eq!(na.len(), 5);
    assert_eq!(na.get(0).unwrap(), 0.0);
    assert_eq!(na.get(1).unwrap(), 0.0);
    assert_eq!(na.get(2).unwrap(), 1.0);
    assert_eq!(na.get(3).unwrap(), 2.0);
    assert_eq!(na.get(4).unwrap(), 2.0);
}

#[test]
fn flatten_to_pixa_empty() {
    let paa = Pixaa::new();
    let (pixa, na) = paa.flatten_to_pixa(true);
    assert_eq!(pixa.pix_slice().len(), 0);
    assert_eq!(na.unwrap().len(), 0);
}

#[test]
fn flatten_to_pixa_preserves_box_absence() {
    use leptonica::core::Box;
    // Inner 0: no boxes attached. Inner 1: boxes attached.
    let mut inner0 = Pixa::with_capacity(2);
    inner0.push(Pix::new(2, 2, PixelDepth::Bit8).unwrap());
    inner0.push(Pix::new(3, 3, PixelDepth::Bit8).unwrap());
    let mut inner1 = Pixa::with_capacity(1);
    inner1.push_with_box(
        Pix::new(4, 4, PixelDepth::Bit8).unwrap(),
        Box::new(10, 20, 4, 4).unwrap(),
    );
    let mut paa = Pixaa::new();
    paa.push(inner0);
    paa.push(inner1);
    let (pixa, _) = paa.flatten_to_pixa(false);
    assert_eq!(pixa.pix_slice().len(), 3);
    // Only the box from inner 1 should be carried over — no default (0,0,0,0)
    // boxes are inserted for inner 0's box-less Pix.
    assert_eq!(pixa.boxa().len(), 1);
    let b = pixa.boxa().get(0).unwrap();
    assert_eq!((b.x, b.y, b.w, b.h), (10, 20, 4, 4));
}

#[test]
fn flatten_to_pixa_deep_clones() {
    let mut paa = Pixaa::new();
    paa.push(make_inner(&[(4, 4)]));
    let (pixa, _) = paa.flatten_to_pixa(false);
    // Both source and result must have refcount 1 (independent buffers).
    assert_eq!(paa.get(0).unwrap().pix_slice()[0].ref_count(), 1);
    assert_eq!(pixa.pix_slice()[0].ref_count(), 1);
}

// -- select_range ------------------------------------------------------

#[test]
fn select_range_typical() {
    let mut paa = Pixaa::new();
    for w in 1u32..=5 {
        paa.push(make_inner(&[(w, w)]));
    }
    let out = paa.select_range(1, 3).unwrap();
    assert_eq!(out.len(), 3);
    assert_eq!(out.get(0).unwrap().pix_slice()[0].width(), 2);
    assert_eq!(out.get(2).unwrap().pix_slice()[0].width(), 4);
}

#[test]
fn select_range_negative_last_means_end() {
    let mut paa = Pixaa::new();
    for w in 1u32..=4 {
        paa.push(make_inner(&[(w, w)]));
    }
    let out = paa.select_range(2, -1).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out.get(0).unwrap().pix_slice()[0].width(), 3);
    assert_eq!(out.get(1).unwrap().pix_slice()[0].width(), 4);
}

#[test]
fn select_range_first_too_large_errors() {
    let mut paa = Pixaa::new();
    paa.push(make_inner(&[(1, 1)]));
    assert!(paa.select_range(5, -1).is_err());
}

#[test]
fn select_range_empty_pixaa_errors() {
    let paa = Pixaa::new();
    assert!(paa.select_range(0, -1).is_err());
}

// -- size_range --------------------------------------------------------

#[test]
fn pixaa_size_range_aggregates() {
    let mut paa = Pixaa::new();
    paa.push(make_inner(&[(7, 5), (3, 9)]));
    paa.push(make_inner(&[(11, 2), (6, 8)]));
    let (minw, minh, maxw, maxh) = paa.size_range().unwrap();
    assert_eq!(minw, 3);
    assert_eq!(minh, 2);
    assert_eq!(maxw, 11);
    assert_eq!(maxh, 9);
}

#[test]
fn pixaa_size_range_empty_is_none() {
    let paa = Pixaa::new();
    assert!(paa.size_range().is_none());
}
