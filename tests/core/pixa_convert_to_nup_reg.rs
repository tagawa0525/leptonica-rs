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
    // Page geometry: 2 columns × 40 px wide + 1 spacing of 4 px + 2 outer
    // spacings = 88 px wide. Height: 2 rows × 40 px + 1 spacing of 4 px +
    // 2 outer spacings = 88 px tall. Allow ±1 wiggle for rounding from
    // display_tiled_and_scaled's internal layout.
    let page = &out.pix_slice()[0];
    assert!(
        page.width() >= 80 && page.width() <= 100,
        "got width {}",
        page.width()
    );
    assert!(
        page.height() >= 80 && page.height() <= 100,
        "got height {}",
        page.height()
    );
}

#[test]
fn convert_to_nup_border_widens_output() {
    // A non-zero border must enlarge the output relative to border = 0,
    // catching implementations that silently drop the border parameter.
    let mut pa = Pixa::new();
    for _ in 0..4 {
        add_pix(&mut pa, 40, 40);
    }
    let no_border = pa.convert_to_nup(2, 2, 40, 4, 0).unwrap();
    let with_border = pa.convert_to_nup(2, 2, 40, 4, 5).unwrap();
    let w0 = no_border.pix_slice()[0].width();
    let w1 = with_border.pix_slice()[0].width();
    assert!(
        w1 > w0,
        "border did not widen output: no_border={w0} with_border={w1}"
    );
}

#[test]
fn convert_to_nup_preserves_depth() {
    // 32 bpp input must produce 32 bpp page; 1 bpp input → 1 bpp page.
    let mut pa32 = Pixa::new();
    for _ in 0..2 {
        pa32.push(PixMut::new(40, 40, PixelDepth::Bit32).unwrap().into());
    }
    let out32 = pa32.convert_to_nup(2, 1, 40, 0, 0).unwrap();
    assert_eq!(out32.pix_slice()[0].depth(), PixelDepth::Bit32);

    let mut pa1 = Pixa::new();
    for _ in 0..2 {
        pa1.push(PixMut::new(40, 40, PixelDepth::Bit1).unwrap().into());
    }
    let out1 = pa1.convert_to_nup(2, 1, 40, 0, 0).unwrap();
    assert_eq!(out1.pix_slice()[0].depth(), PixelDepth::Bit1);
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
