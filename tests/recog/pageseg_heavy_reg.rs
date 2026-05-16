//! Regression tests for plan 804
//! (pixCleanImage / pixCountTextColumns / pixCropImage /
//! pixDecideIfText / pixExtractRawTextlines).

use leptonica::recog::pageseg::{
    pix_clean_image, pix_count_text_columns, pix_crop_image, pix_decide_if_text,
    pix_extract_raw_textlines,
};
use leptonica::{Pix, PixMut, PixelDepth};

// -- shared fixtures ------------------------------------------------------

fn make_blank(w: u32, h: u32, depth: PixelDepth) -> Pix {
    let pix = Pix::new(w, h, depth).unwrap();
    if depth != PixelDepth::Bit1 {
        let mut m = pix.try_into_mut().unwrap();
        let bg = if depth == PixelDepth::Bit32 {
            0x00ffffff
        } else {
            255
        };
        for y in 0..h {
            for x in 0..w {
                m.set_pixel(x, y, bg).unwrap();
            }
        }
        return m.into();
    }
    pix
}

fn paint_horiz_text_lines(pix: &mut PixMut, ys: &[u32], xs: std::ops::Range<u32>) {
    for &y in ys {
        for x in xs.clone() {
            pix.set_pixel(x, y, 1).unwrap();
        }
    }
}

fn make_text_page_1bpp(w: u32, h: u32, line_step: u32) -> Pix {
    let pix = make_blank(w, h, PixelDepth::Bit1);
    let mut m = pix.try_into_mut().unwrap();
    let lines: Vec<u32> = (10..(h - 10)).step_by(line_step as usize).collect();
    paint_horiz_text_lines(&mut m, &lines, 20..(w - 20));
    m.into()
}

// -- pix_clean_image -------------------------------------------------------

/// Dimensions may shift by a few pixels due to deskew border-expansion; we
/// just check the result is "close" to the expected size.
fn assert_close(actual: u32, expected: u32) {
    let diff = actual.abs_diff(expected);
    assert!(
        diff <= 8,
        "expected dim ≈ {expected}, got {actual} (diff {diff})"
    );
}

#[test]
fn clean_image_1bpp_identity_rotation_0() {
    let pix = make_text_page_1bpp(200, 200, 20);
    let cleaned = pix_clean_image(&pix, 1, 0, 1, 0).unwrap();
    assert_eq!(cleaned.depth(), PixelDepth::Bit1);
    assert_close(cleaned.width(), pix.width());
    assert_close(cleaned.height(), pix.height());
}

#[test]
fn clean_image_1bpp_rotation_90() {
    let pix = make_text_page_1bpp(200, 150, 20);
    let cleaned = pix_clean_image(&pix, 1, 1, 1, 0).unwrap();
    assert_eq!(cleaned.depth(), PixelDepth::Bit1);
    // 1 quad cw → swap dims (allowing slight deskew expansion).
    assert_close(cleaned.width(), pix.height());
    assert_close(cleaned.height(), pix.width());
}

#[test]
fn clean_image_1bpp_scale_2_doubles_dims() {
    let pix = make_text_page_1bpp(150, 150, 20);
    let cleaned = pix_clean_image(&pix, 1, 0, 2, 0).unwrap();
    assert_close(cleaned.width(), pix.width() * 2);
    assert_close(cleaned.height(), pix.height() * 2);
}

#[test]
fn clean_image_8bpp_produces_1bpp_output() {
    // 200x200 8bpp page with dark horizontal bands to give deskew real
    // content to work on.
    let pix = Pix::new(200, 200, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for y in 0..200 {
        for x in 0..200 {
            m.set_pixel(x, y, 255).unwrap();
        }
    }
    for y in (20..180).step_by(20) {
        for x in 20..180 {
            m.set_pixel(x, y, 30).unwrap();
        }
    }
    let pix: Pix = m.into();
    let cleaned = pix_clean_image(&pix, 1, 0, 1, 0).unwrap();
    assert_eq!(cleaned.depth(), PixelDepth::Bit1);
}

#[test]
fn clean_image_rejects_invalid_rotation() {
    let pix = make_text_page_1bpp(200, 200, 20);
    assert!(pix_clean_image(&pix, 1, 4, 1, 0).is_err());
}

#[test]
fn clean_image_rejects_invalid_contrast() {
    let pix = make_text_page_1bpp(200, 200, 20);
    assert!(pix_clean_image(&pix, 0, 0, 1, 0).is_err());
    assert!(pix_clean_image(&pix, 11, 0, 1, 0).is_err());
}

#[test]
fn clean_image_rejects_invalid_scale() {
    let pix = make_text_page_1bpp(200, 200, 20);
    assert!(pix_clean_image(&pix, 1, 0, 3, 0).is_err());
}

#[test]
fn clean_image_rejects_invalid_opensize() {
    let pix = make_text_page_1bpp(200, 200, 20);
    assert!(pix_clean_image(&pix, 1, 0, 1, 4).is_err());
}

#[test]
fn clean_image_opensize_2_keeps_dims() {
    let pix = make_text_page_1bpp(200, 200, 20);
    let cleaned = pix_clean_image(&pix, 1, 0, 1, 2).unwrap();
    assert_close(cleaned.width(), pix.width());
    assert_close(cleaned.height(), pix.height());
}

// -- pix_count_text_columns -----------------------------------------------

fn make_two_column_page() -> Pix {
    // 600x400 1bpp page with two vertical text bands separated by a wide
    // whitespace gutter (peak of inverse column count near center).
    let pix = Pix::new(600, 400, PixelDepth::Bit1).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    m.set_resolution(150, 150);
    let lines: Vec<u32> = (40..360).step_by(20).collect();
    for &y in &lines {
        for x in 60..260 {
            m.set_pixel(x, y, 1).unwrap();
        }
        for x in 340..540 {
            m.set_pixel(x, y, 1).unwrap();
        }
    }
    m.into()
}

fn make_blank_page() -> Pix {
    let p = Pix::new(600, 400, PixelDepth::Bit1).unwrap();
    let mut m = p.try_into_mut().unwrap();
    m.set_resolution(150, 150);
    m.into()
}

#[test]
fn count_text_columns_blank_returns_zero() {
    let pix = make_blank_page();
    let n = pix_count_text_columns(&pix, 0.3, 0.5, 0.1).unwrap();
    assert_eq!(n, 0);
}

#[test]
fn count_text_columns_rejects_non_1bpp() {
    let pix = Pix::new(300, 200, PixelDepth::Bit8).unwrap();
    assert!(pix_count_text_columns(&pix, 0.3, 0.5, 0.1).is_err());
}

#[test]
fn count_text_columns_rejects_clipfract_out_of_range() {
    let pix = make_blank_page();
    assert!(pix_count_text_columns(&pix, 0.3, 0.5, -0.1).is_err());
    assert!(pix_count_text_columns(&pix, 0.3, 0.5, 0.5).is_err());
}

#[test]
fn count_text_columns_two_column_layout() {
    let pix = make_two_column_page();
    let n = pix_count_text_columns(&pix, 0.3, 0.5, 0.1).unwrap();
    assert!(n >= 1, "expected >= 1 columns detected, got {n}");
}

// -- pix_decide_if_text ----------------------------------------------------

#[test]
#[ignore = "plan 804: not yet implemented"]
fn decide_if_text_empty_returns_none() {
    let pix = make_blank(400, 400, PixelDepth::Bit1);
    let res = pix_decide_if_text(&pix, None).unwrap();
    assert!(res.is_none());
}

#[test]
#[ignore = "plan 804: not yet implemented"]
fn decide_if_text_rejects_invalid_box() {
    use leptonica::core::Box as LeptBox;
    let pix = make_blank(400, 400, PixelDepth::Bit1);
    let outside = LeptBox {
        x: 500,
        y: 0,
        w: 100,
        h: 100,
    };
    assert!(pix_decide_if_text(&pix, Some(&outside)).is_err());
}

#[test]
#[ignore = "plan 804: not yet implemented"]
fn decide_if_text_photo_returns_some_false() {
    // Single huge fg blob (no horizontal text-line structure) → photo.
    let pix = Pix::new(500, 500, PixelDepth::Bit1).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for y in 50..450 {
        for x in 50..450 {
            m.set_pixel(x, y, 1).unwrap();
        }
    }
    m.set_resolution(300, 300);
    let pix: Pix = m.into();
    let res = pix_decide_if_text(&pix, None).unwrap();
    assert_eq!(res, Some(false));
}

// -- pix_extract_raw_textlines --------------------------------------------

#[test]
fn extract_raw_textlines_empty_returns_empty() {
    let pix = make_blank(500, 500, PixelDepth::Bit1);
    let pixa = pix_extract_raw_textlines(&pix, 0, 0, 0, 0).unwrap();
    assert_eq!(pixa.len(), 0);
}

#[test]
fn extract_raw_textlines_finds_lines() {
    // 600x400 1bpp page with 5 horizontal text lines, each made up of many
    // small char-like glyphs (so each component is < maxw before the
    // horizontal close merges them into a line).
    let pix = Pix::new(600, 400, PixelDepth::Bit1).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for line in 0..5u32 {
        let y_start = 40 + line * 60;
        let mut x = 50u32;
        while x + 16 < 550 {
            for y in y_start..(y_start + 10) {
                for xx in x..(x + 12) {
                    m.set_pixel(xx, y, 1).unwrap();
                }
            }
            x += 20;
        }
    }
    m.set_resolution(300, 300);
    let pix: Pix = m.into();
    let pixa = pix_extract_raw_textlines(&pix, 0, 0, 0, 0).unwrap();
    assert!(
        !pixa.is_empty(),
        "expected at least 1 textline, got {}",
        pixa.len()
    );
}

// -- pix_crop_image -------------------------------------------------------

#[test]
#[ignore = "plan 804: not yet implemented"]
fn crop_image_rejects_small() {
    let pix = make_blank(50, 50, PixelDepth::Bit8);
    assert!(pix_crop_image(&pix, 0, 0, 0, 0, 0, 1.0, 0).is_err());
}

#[test]
#[ignore = "plan 804: not yet implemented"]
fn crop_image_returns_same_size_and_some_crop_box() {
    let pix = make_blank(400, 400, PixelDepth::Bit8);
    let mut m = pix.try_into_mut().unwrap();
    // Paint a centered black rectangle as the "content".
    for y in 80..320 {
        for x in 80..320 {
            m.set_pixel(x, y, 30).unwrap();
        }
    }
    let pix: Pix = m.into();
    let (out, b) = pix_crop_image(&pix, 0, 0, 0, 10, 10, 1.0, 0).unwrap();
    // Output is full-resolution 1bpp with same dims as input.
    assert_eq!(out.depth(), PixelDepth::Bit1);
    assert_eq!(out.width(), pix.width());
    assert_eq!(out.height(), pix.height());
    // Crop box must be a non-empty sub-rectangle of the input.
    assert!(b.w > 0 && b.h > 0);
    assert!(b.x >= 0 && b.y >= 0);
    assert!(b.x + b.w <= pix.width() as i32);
    assert!(b.y + b.h <= pix.height() as i32);
}

#[test]
#[ignore = "plan 804: not yet implemented"]
fn crop_image_rejects_clear_too_large() {
    let pix = make_blank(400, 400, PixelDepth::Bit8);
    // lr_clear = w/3 > w/6 → reject.
    assert!(pix_crop_image(&pix, 200, 0, 0, 0, 0, 1.0, 0).is_err());
}
