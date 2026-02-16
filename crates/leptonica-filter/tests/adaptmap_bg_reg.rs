//! Test background map extraction and application functions
//!
//! C版: reference/leptonica/src/adaptmap.c
//! - pixGetBackgroundGrayMap
//! - pixGetBackgroundRGBMap
//! - pixGetInvBackgroundMap
//! - pixApplyInvBackgroundGrayMap
//! - pixApplyInvBackgroundRGBMap
//! - pixFillMapHoles
//! - pixBackgroundNormGrayArray
//! - pixBackgroundNormRGBArrays
//! - pixCleanBackgroundToWhite

use leptonica_core::{Pix, PixelDepth, color};
use leptonica_filter::adaptmap::{self, BackgroundNormOptions};

/// Create a grayscale test image with uneven background
fn make_gray_test_image() -> Pix {
    let pix = Pix::new(60, 60, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..60 {
        for x in 0..60 {
            // Background gradient (brighter right side)
            let bg = 120 + (x as u32 * 2).min(135);
            // Dark "text" region in center
            let val = if x > 15 && x < 45 && y > 15 && y < 45 {
                bg / 3
            } else {
                bg
            };
            pm.set_pixel_unchecked(x, y, val.min(255));
        }
    }
    pm.into()
}

/// Create a 32bpp RGB test image with uneven background
fn make_color_test_image() -> Pix {
    let pix = Pix::new(60, 60, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..60 {
        for x in 0..60 {
            let r = (120 + x * 2).min(255) as u8;
            let g = (140 + y).min(255) as u8;
            let b = 180u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(r, g, b));
        }
    }
    pm.into()
}

// ============================================================================
// pixGetBackgroundGrayMap
// ============================================================================

#[test]
fn test_get_background_gray_map_basic() {
    let pix = make_gray_test_image();
    let map = adaptmap::get_background_gray_map(&pix, None, 10, 10, 60, 40).unwrap();

    // Map should be smaller than original (tile-based)
    assert!(map.width() < pix.width());
    assert!(map.height() < pix.height());
    assert_eq!(map.depth(), PixelDepth::Bit8);

    // Map values should be > 0 (no unfilled holes)
    let map_w = map.width();
    let map_h = map.height();
    for y in 0..map_h {
        for x in 0..map_w {
            assert!(map.get_pixel_unchecked(x, y) > 0, "hole at ({x}, {y})");
        }
    }
}

// ============================================================================
// pixGetBackgroundRGBMap
// ============================================================================

#[test]
fn test_get_background_rgb_map_basic() {
    let pix = make_color_test_image();
    let (map_r, map_g, map_b) =
        adaptmap::get_background_rgb_map(&pix, None, None, 10, 10, 60, 40).unwrap();

    // All maps should have the same dimensions
    assert_eq!(map_r.width(), map_g.width());
    assert_eq!(map_r.height(), map_b.height());
    assert_eq!(map_r.depth(), PixelDepth::Bit8);
}

// ============================================================================
// pixGetInvBackgroundMap
// ============================================================================

#[test]
fn test_get_inv_background_map() {
    let pix = make_gray_test_image();
    let bg_map = adaptmap::get_background_gray_map(&pix, None, 10, 10, 60, 40).unwrap();
    let inv_map = adaptmap::get_inv_background_map(&bg_map, 200, 2, 1).unwrap();

    assert_eq!(inv_map.width(), bg_map.width());
    assert_eq!(inv_map.height(), bg_map.height());
}

// ============================================================================
// pixApplyInvBackgroundGrayMap
// ============================================================================

#[test]
fn test_apply_inv_background_gray_map() {
    let pix = make_gray_test_image();
    let bg_map = adaptmap::get_background_gray_map(&pix, None, 10, 10, 60, 40).unwrap();
    let inv_map = adaptmap::get_inv_background_map(&bg_map, 200, 2, 1).unwrap();
    let result = adaptmap::apply_inv_background_gray_map(&pix, &inv_map, 10, 10).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit8);
}

// ============================================================================
// pixApplyInvBackgroundRGBMap
// ============================================================================

#[test]
fn test_apply_inv_background_rgb_map() {
    let pix = make_color_test_image();
    let (map_r, map_g, map_b) =
        adaptmap::get_background_rgb_map(&pix, None, None, 10, 10, 60, 40).unwrap();
    let inv_r = adaptmap::get_inv_background_map(&map_r, 200, 2, 1).unwrap();
    let inv_g = adaptmap::get_inv_background_map(&map_g, 200, 2, 1).unwrap();
    let inv_b = adaptmap::get_inv_background_map(&map_b, 200, 2, 1).unwrap();
    let result =
        adaptmap::apply_inv_background_rgb_map(&pix, &inv_r, &inv_g, &inv_b, 10, 10).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit32);
}

// ============================================================================
// pixFillMapHoles
// ============================================================================

#[test]
fn test_fill_map_holes_public() {
    // Create small map with holes (zero values)
    let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(0, 0, 100);
    pm.set_pixel_unchecked(4, 0, 200);
    pm.set_pixel_unchecked(0, 4, 150);
    pm.set_pixel_unchecked(4, 4, 180);
    let pix: Pix = pm.into();

    let filled = adaptmap::fill_map_holes(&pix, 5, 5).unwrap();

    // All holes should be filled
    for y in 0..5 {
        for x in 0..5 {
            assert!(filled.get_pixel_unchecked(x, y) > 0, "hole at ({x}, {y})");
        }
    }
}

// ============================================================================
// pixBackgroundNormGrayArray / pixBackgroundNormRGBArrays
// ============================================================================

#[test]
fn test_background_norm_gray_array() {
    let pix = make_gray_test_image();
    let inv_map =
        adaptmap::background_norm_gray_array(&pix, &BackgroundNormOptions::default()).unwrap();

    // Returns the inverted background map
    assert!(inv_map.width() > 0);
    assert!(inv_map.height() > 0);
}

#[test]
fn test_background_norm_rgb_arrays() {
    let pix = make_color_test_image();
    let (inv_r, inv_g, inv_b) =
        adaptmap::background_norm_rgb_arrays(&pix, &BackgroundNormOptions::default()).unwrap();

    assert_eq!(inv_r.width(), inv_g.width());
    assert_eq!(inv_r.height(), inv_b.height());
}

// ============================================================================
// pixCleanBackgroundToWhite
// ============================================================================

#[test]
fn test_clean_background_to_white_gray() {
    let pix = make_gray_test_image();
    let result = adaptmap::clean_background_to_white(&pix, None, None).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit8);
}

#[test]
fn test_clean_background_to_white_color() {
    let pix = make_color_test_image();
    let result = adaptmap::clean_background_to_white(&pix, None, None).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit32);
}
