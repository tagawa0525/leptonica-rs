//! Test morph-based background map extraction and normalization
//!
//! C版: reference/leptonica/src/adaptmap.c
//! - pixBackgroundNormMorph
//! - pixBackgroundNormGrayArrayMorph
//! - pixBackgroundNormRGBArraysMorph
//! - pixGetBackgroundGrayMapMorph
//! - pixGetBackgroundRGBMapMorph

use leptonica_core::{Pix, PixelDepth, color};
use leptonica_filter::adaptmap;

/// Create a grayscale test image with uneven background
fn make_gray_test_image() -> Pix {
    let pix = Pix::new(80, 80, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..80 {
        for x in 0..80 {
            // Background gradient (brighter right side)
            let bg = 120 + (x * 2).min(135);
            // Dark "text" region in center
            let val = if x > 20 && x < 60 && y > 20 && y < 60 {
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
    let pix = Pix::new(80, 80, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..80 {
        for x in 0..80 {
            let r = (120 + x * 2).min(255) as u8;
            let g = (140 + y).min(255) as u8;
            let b = 180u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(r, g, b));
        }
    }
    pm.into()
}

// ============================================================================
// pixGetBackgroundGrayMapMorph
// ============================================================================

#[test]
fn test_get_background_gray_map_morph_basic() {
    let pix = make_gray_test_image();
    let map = adaptmap::get_background_gray_map_morph(&pix, None, 2, 7).unwrap();

    // Map should be smaller than original (reduced by factor)
    assert!(map.width() < pix.width());
    assert!(map.height() < pix.height());
    assert_eq!(map.depth(), PixelDepth::Bit8);
}

#[test]
fn test_get_background_gray_map_morph_values_nonzero() {
    let pix = make_gray_test_image();
    let map = adaptmap::get_background_gray_map_morph(&pix, None, 2, 7).unwrap();

    // All map pixels should be non-zero (holes filled)
    let map_w = map.width();
    let map_h = map.height();
    for y in 0..map_h {
        for x in 0..map_w {
            assert!(map.get_pixel_unchecked(x, y) > 0, "hole at ({x}, {y})");
        }
    }
}

#[test]
fn test_get_background_gray_map_morph_invalid_params() {
    let pix = make_gray_test_image();
    // reduction must be between 2 and 16
    assert!(adaptmap::get_background_gray_map_morph(&pix, None, 1, 7).is_err());
    assert!(adaptmap::get_background_gray_map_morph(&pix, None, 17, 7).is_err());
}

// ============================================================================
// pixGetBackgroundRGBMapMorph
// ============================================================================

#[test]
fn test_get_background_rgb_map_morph_basic() {
    let pix = make_color_test_image();
    let (map_r, map_g, map_b) = adaptmap::get_background_rgb_map_morph(&pix, None, 2, 7).unwrap();

    // All three maps should have identical dimensions
    assert_eq!(map_r.width(), map_g.width());
    assert_eq!(map_r.width(), map_b.width());
    assert_eq!(map_r.height(), map_g.height());
    assert_eq!(map_r.height(), map_b.height());
    assert_eq!(map_r.depth(), PixelDepth::Bit8);
    assert_eq!(map_g.depth(), PixelDepth::Bit8);
    assert_eq!(map_b.depth(), PixelDepth::Bit8);
}

#[test]
fn test_get_background_rgb_map_morph_values_nonzero() {
    let pix = make_color_test_image();
    let (map_r, map_g, map_b) = adaptmap::get_background_rgb_map_morph(&pix, None, 2, 7).unwrap();

    for map in [&map_r, &map_g, &map_b] {
        let w = map.width();
        let h = map.height();
        for y in 0..h {
            for x in 0..w {
                assert!(map.get_pixel_unchecked(x, y) > 0, "hole at ({x}, {y})");
            }
        }
    }
}

// ============================================================================
// pixBackgroundNormMorph
// ============================================================================

#[test]
fn test_background_norm_morph_gray() {
    let pix = make_gray_test_image();
    let result = adaptmap::background_norm_morph(&pix, None, 2, 7, 200).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit8);
}

#[test]
fn test_background_norm_morph_color() {
    let pix = make_color_test_image();
    let result = adaptmap::background_norm_morph(&pix, None, 2, 7, 200).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit32);
}

#[test]
fn test_background_norm_morph_background_brightened() {
    let pix = make_gray_test_image();
    let bgval = 200u32;
    let result = adaptmap::background_norm_morph(&pix, None, 2, 7, bgval).unwrap();

    // Background region (outside center text) should be near bgval
    let val = result.get_pixel_unchecked(5, 5);
    assert!(
        val > bgval - 50 && val < 256,
        "background pixel at (5,5) = {val}, expected near {bgval}"
    );
}

// ============================================================================
// pixBackgroundNormGrayArrayMorph
// ============================================================================

#[test]
fn test_background_norm_gray_array_morph() {
    let pix = make_gray_test_image();
    let inv_map = adaptmap::background_norm_gray_array_morph(&pix, None, 2, 7, 200).unwrap();

    // Returns the inverted background map (32bpp, 16-bit precision values)
    assert!(inv_map.width() > 0);
    assert!(inv_map.height() > 0);
    assert_eq!(inv_map.depth(), PixelDepth::Bit32);
}

// ============================================================================
// pixBackgroundNormRGBArraysMorph
// ============================================================================

#[test]
fn test_background_norm_rgb_arrays_morph() {
    let pix = make_color_test_image();
    let (inv_r, inv_g, inv_b) =
        adaptmap::background_norm_rgb_arrays_morph(&pix, None, 2, 7, 200).unwrap();

    assert_eq!(inv_r.width(), inv_g.width());
    assert_eq!(inv_r.width(), inv_b.width());
    assert_eq!(inv_r.height(), inv_g.height());
    assert_eq!(inv_r.height(), inv_b.height());
    assert_eq!(inv_r.depth(), PixelDepth::Bit32);
}
