//! Regression tests for plan 127 (Pixa::convert_to_nup).

use leptonica::core::pixa::Pixa;
use leptonica::{PixMut, PixelDepth};

fn add_pix(pa: &mut Pixa, w: u32, h: u32) {
    pa.push(PixMut::new(w, h, PixelDepth::Bit8).unwrap().into());
}

#[test]

fn convert_to_nup_2x2_one_page() {
    let mut pa = Pixa::new();
    for _ in 0..4 {
        add_pix(&mut pa, 40, 40);
    }
    let out = pa.convert_to_nup(2, 2, 40, 4, 0).unwrap();
    assert_eq!(out.pix_slice().len(), 1);
}

#[test]

fn convert_to_nup_partial_last_page() {
    // 5 images, 2x2 grid -> 2 pages (first full, second has only 1 tile)
    let mut pa = Pixa::new();
    for _ in 0..5 {
        add_pix(&mut pa, 40, 40);
    }
    let out = pa.convert_to_nup(2, 2, 40, 4, 0).unwrap();
    assert_eq!(out.pix_slice().len(), 2);
}

#[test]

fn convert_to_nup_empty_returns_empty() {
    let pa = Pixa::new();
    let out = pa.convert_to_nup(2, 2, 40, 4, 0).unwrap();
    assert_eq!(out.pix_slice().len(), 0);
}

#[test]

fn convert_to_nup_invalid_grid_errors() {
    let mut pa = Pixa::new();
    add_pix(&mut pa, 40, 40);
    assert!(pa.convert_to_nup(0, 2, 40, 4, 0).is_err());
    assert!(pa.convert_to_nup(2, 0, 40, 4, 0).is_err());
    assert!(pa.convert_to_nup(51, 2, 40, 4, 0).is_err());
}

#[test]

fn convert_to_nup_too_small_tile_width_errors() {
    let mut pa = Pixa::new();
    add_pix(&mut pa, 40, 40);
    // C: tw must be >= 20
    assert!(pa.convert_to_nup(2, 2, 19, 4, 0).is_err());
}
