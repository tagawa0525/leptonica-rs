//! Regression tests for plan 128 (filter::stroke_width_transform).

use leptonica::filter::stroke_width_transform;
use leptonica::{PixMut, PixelDepth};

fn solid_block(w: u32, h: u32) -> leptonica::Pix {
    let mut pm = PixMut::new(w, h, PixelDepth::Bit1).unwrap();
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel(x, y, 1).unwrap();
        }
    }
    pm.into()
}

#[test]
#[ignore = "not yet implemented"]
fn swt_solid_block_reports_width() {
    // A 10x10 solid 1bpp block: every fg pixel's stroke width is non-zero,
    // and the center pixel sees the maximum value among the candidates.
    let pix = solid_block(10, 10);
    let out = stroke_width_transform(&pix, 1, PixelDepth::Bit8, 2).unwrap();
    assert_eq!(out.depth(), PixelDepth::Bit8);
    assert_eq!(out.width(), 10);
    assert_eq!(out.height(), 10);
    let center = out.get_pixel(5, 5).unwrap();
    assert!(center > 0, "expected center stroke width > 0, got {center}");
}

#[test]
#[ignore = "not yet implemented"]
fn swt_color_zero_inverts_input() {
    // color = 0: background pixels are treated as foreground.
    // Build a Pix that is mostly 0 (white) with a small fg block in the
    // corner. swt(color=1) reports widths only inside the fg block, but
    // swt(color=0) reports widths in the (much larger) background region.
    let mut pm = PixMut::new(20, 20, PixelDepth::Bit1).unwrap();
    for y in 0..4 {
        for x in 0..4 {
            pm.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: leptonica::Pix = pm.into();
    let out1 = stroke_width_transform(&pix, 1, PixelDepth::Bit8, 2).unwrap();
    let out0 = stroke_width_transform(&pix, 0, PixelDepth::Bit8, 2).unwrap();
    // Pixel at (1, 1): foreground -> non-zero in out1; not foreground in out0.
    assert!(out1.get_pixel(1, 1).unwrap() > 0);
    assert_eq!(out0.get_pixel(1, 1).unwrap(), 0);
    // Pixel at (15, 15): background -> 0 in out1, non-zero in out0.
    assert_eq!(out1.get_pixel(15, 15).unwrap(), 0);
    assert!(out0.get_pixel(15, 15).unwrap() > 0);
}

#[test]
#[ignore = "not yet implemented"]
fn swt_rejects_non_1bpp_input() {
    let pix: leptonica::Pix = PixMut::new(10, 10, PixelDepth::Bit8).unwrap().into();
    assert!(stroke_width_transform(&pix, 1, PixelDepth::Bit8, 2).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn swt_rejects_invalid_depth() {
    let pix = solid_block(10, 10);
    assert!(stroke_width_transform(&pix, 1, PixelDepth::Bit4, 2).is_err());
    assert!(stroke_width_transform(&pix, 1, PixelDepth::Bit32, 2).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn swt_rejects_invalid_nangles() {
    let pix = solid_block(10, 10);
    for n in [0u32, 1, 3, 5, 7, 9, 16] {
        assert!(
            stroke_width_transform(&pix, 1, PixelDepth::Bit8, n).is_err(),
            "nangles={n} should error"
        );
    }
}

#[test]
#[ignore = "not yet implemented"]
fn swt_supports_all_valid_nangles() {
    let pix = solid_block(10, 10);
    for n in [2u32, 4, 6, 8] {
        assert!(
            stroke_width_transform(&pix, 1, PixelDepth::Bit8, n).is_ok(),
            "nangles={n} should succeed"
        );
    }
}
