//! Regression tests for plan 803 (pageseg.c の補助 4 関数).

use leptonica::recog::pageseg::{
    pix_find_thresh_fg_extent, pix_gen_halftone_mask, pix_gen_textblock_mask, pix_gen_textline_mask,
};
use leptonica::{Pix, PixelDepth};

fn make_with_rows(w: u32, h: u32, ys: &[u32]) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for &y in ys {
        for x in 0..w {
            m.set_pixel(x, y, 1).unwrap();
        }
    }
    m.into()
}

// -- pix_find_thresh_fg_extent --------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn find_thresh_fg_extent_all_zero() {
    let p = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let (top, bot) = pix_find_thresh_fg_extent(&p, 1).unwrap();
    assert_eq!((top, bot), (0, 0));
}

#[test]
#[ignore = "not yet implemented"]
fn find_thresh_fg_extent_single_row() {
    let p = make_with_rows(10, 10, &[3]);
    let (top, bot) = pix_find_thresh_fg_extent(&p, 1).unwrap();
    assert_eq!((top, bot), (3, 3));
}

#[test]
#[ignore = "not yet implemented"]
fn find_thresh_fg_extent_range() {
    let p = make_with_rows(10, 10, &[2, 4, 5, 7]);
    let (top, bot) = pix_find_thresh_fg_extent(&p, 1).unwrap();
    assert_eq!((top, bot), (2, 7));
}

#[test]
#[ignore = "not yet implemented"]
fn find_thresh_fg_extent_thresh_filters_low_rows() {
    let p = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let mut m = p.try_into_mut().unwrap();
    // Row 2: 1 FG pixel; row 5: 5 FG pixels; row 8: 1 FG pixel.
    m.set_pixel(0, 2, 1).unwrap();
    for x in 0..5 {
        m.set_pixel(x, 5, 1).unwrap();
    }
    m.set_pixel(0, 8, 1).unwrap();
    let p: Pix = m.into();
    let (top, bot) = pix_find_thresh_fg_extent(&p, 3).unwrap();
    // Only row 5 has count >= 3.
    assert_eq!((top, bot), (5, 5));
}

#[test]
#[ignore = "not yet implemented"]
fn find_thresh_fg_extent_rejects_non_1bpp() {
    let p = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(pix_find_thresh_fg_extent(&p, 1).is_err());
}

// -- pix_gen_halftone_mask ------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn gen_halftone_mask_rejects_non_1bpp() {
    let p = Pix::new(120, 120, PixelDepth::Bit8).unwrap();
    assert!(pix_gen_halftone_mask(&p).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn gen_halftone_mask_empty_image_returns_no_halftone() {
    let p = Pix::new(120, 120, PixelDepth::Bit1).unwrap();
    let (mask, _text, found) = pix_gen_halftone_mask(&p).unwrap();
    assert!(!found);
    assert_eq!(mask.depth(), PixelDepth::Bit1);
}

// -- pix_gen_textline_mask ------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn gen_textline_mask_rejects_non_1bpp() {
    let p = Pix::new(120, 120, PixelDepth::Bit8).unwrap();
    assert!(pix_gen_textline_mask(&p).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn gen_textline_mask_rejects_too_small() {
    let p = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
    assert!(pix_gen_textline_mask(&p).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn gen_textline_mask_empty_image_runs() {
    let p = Pix::new(120, 120, PixelDepth::Bit1).unwrap();
    let (tl, _vws, found) = pix_gen_textline_mask(&p).unwrap();
    // An empty image has no textlines.
    assert!(!found);
    assert_eq!(tl.width(), 120);
    assert_eq!(tl.height(), 120);
}

// -- pix_gen_textblock_mask -----------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn gen_textblock_mask_rejects_non_1bpp() {
    let p = Pix::new(120, 120, PixelDepth::Bit8).unwrap();
    let v = Pix::new(120, 120, PixelDepth::Bit1).unwrap();
    assert!(pix_gen_textblock_mask(&p, &v).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn gen_textblock_mask_rejects_too_small() {
    let p = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
    let v = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
    assert!(pix_gen_textblock_mask(&p, &v).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn gen_textblock_mask_empty_image_returns_none() {
    let p = Pix::new(120, 120, PixelDepth::Bit1).unwrap();
    let v = Pix::new(120, 120, PixelDepth::Bit1).unwrap();
    let out = pix_gen_textblock_mask(&p, &v).unwrap();
    // Empty input gives no FG after the morph step.
    assert!(out.is_none());
}
