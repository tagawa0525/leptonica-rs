//! Regression tests for plan 125 (Pixa convert-to-8-cmap + tiled extraction).

use leptonica::core::Box;
use leptonica::core::box_::Boxa;
use leptonica::{Pix, PixMut, Pixa, PixelDepth};

// -- convert_to_8_colormap ---------------------------------------------

#[test]

fn convert_to_8_colormap_preserves_boxa() {
    let mut pa = Pixa::new();
    let p32: Pix = PixMut::new(8, 8, PixelDepth::Bit32).unwrap().into();
    pa.push_with_box(p32, Box::new(3, 5, 8, 8).unwrap());
    let out = pa.convert_to_8_colormap(false).unwrap();
    assert_eq!(out.pix_slice().len(), 1);
    let p = &out.pix_slice()[0];
    assert_eq!(p.depth(), PixelDepth::Bit8);
    assert!(p.colormap().is_some());
    let b = out.boxa().get(0).unwrap();
    assert_eq!((b.x, b.y, b.w, b.h), (3, 5, 8, 8));
}

// -- Pix::make_tiled_pixa ---------------------------------------------

fn mkpix(w: u32, h: u32) -> Pix {
    PixMut::new(w, h, PixelDepth::Bit8).unwrap().into()
}

#[test]

fn pix_make_tiled_pixa_grid() {
    // 30x20 image, 10x10 tiles -> 3 x 2 = 6 tiles
    let pixs = mkpix(30, 20);
    let pa = pixs.make_tiled_pixa(10, 10, 0, 0, None).unwrap();
    assert_eq!(pa.pix_slice().len(), 6);
    for pix in pa.pix_slice() {
        assert_eq!(pix.width(), 10);
        assert_eq!(pix.height(), 10);
    }
}

#[test]

fn pix_make_tiled_pixa_start_num() {
    // 30x20 image, 10x10 tiles, start=2, num=2 -> tiles 2..4
    let pixs = mkpix(30, 20);
    let pa = pixs.make_tiled_pixa(10, 10, 2, 2, None).unwrap();
    assert_eq!(pa.pix_slice().len(), 2);
}

#[test]

fn pix_make_tiled_pixa_with_boxa() {
    // boxa overrides the uniform grid case
    let pixs = mkpix(30, 20);
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 5, 5).unwrap());
    boxa.push(Box::new(20, 10, 8, 8).unwrap());
    let pa = pixs.make_tiled_pixa(10, 10, 0, 0, Some(&boxa)).unwrap();
    assert_eq!(pa.pix_slice().len(), 2);
    assert_eq!(pa.pix_slice()[0].width(), 5);
    assert_eq!(pa.pix_slice()[1].width(), 8);
}

#[test]

fn pix_make_tiled_pixa_respects_tile_count() {
    // If text field encodes "n = 5" (a valid hint <= 6 tiles), only 5 are
    // produced when num = 0 (take all).
    let pixs: Pix = {
        let mut pm = PixMut::new(30, 20, PixelDepth::Bit8).unwrap();
        pm.set_text(Some("n = 5".to_string()));
        pm.into()
    };
    let pa = pixs.make_tiled_pixa(10, 10, 0, 0, None).unwrap();
    assert_eq!(pa.pix_slice().len(), 5);
}

#[test]

fn pix_make_tiled_pixa_invalid_tile_size() {
    let pixs = mkpix(10, 10);
    // 20 > 10, so nx = 0, error
    assert!(pixs.make_tiled_pixa(20, 5, 0, 0, None).is_err());
}

// -- Pixa::make_tiled_pixa --------------------------------------------

#[test]

fn pixa_make_tiled_pixa_joins_all_inner() {
    // Two 30x10 (3 tiles) Pixs in a Pixa -> each contributes 3 tiles -> total 6
    let mut pa = Pixa::new();
    pa.push(mkpix(30, 10));
    pa.push(mkpix(30, 10));
    let out = pa.make_tiled_pixa(10, 10, 3).unwrap();
    assert_eq!(out.pix_slice().len(), 6);
}

#[test]

fn pixa_make_tiled_pixa_clamps_nsamp() {
    // Each Pix has only 2 tiles but nsamp = 5 -> still 2 tiles per Pix (clamp)
    let mut pa = Pixa::new();
    pa.push(mkpix(20, 10));
    pa.push(mkpix(20, 10));
    let out = pa.make_tiled_pixa(10, 10, 5).unwrap();
    assert_eq!(out.pix_slice().len(), 4);
}
