//! Test advanced adaptive mapping functions
//!
//! C版: reference/leptonica/src/adaptmap.c
//! - pixApplyVariableGrayMap
//! - pixGlobalNormRGB
//! - pixConvertTo8MinMax

use leptonica_core::{Pix, PixelDepth, color};
use leptonica_filter::adaptmap;

/// Create a grayscale test image
fn make_gray_test_image() -> Pix {
    let pix = Pix::new(60, 60, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..60 {
        for x in 0..60 {
            let val = (x * 4).min(255) as u32;
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

/// Create a 32bpp RGB test image with color cast
fn make_color_cast_image() -> Pix {
    let pix = Pix::new(60, 60, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..60 {
        for x in 0..60 {
            // Warm color cast: red stronger than green/blue
            let r = (200 + x).min(255) as u8;
            let g = (170 + x).min(255) as u8;
            let b = (150 + x).min(255) as u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(r, g, b));
        }
    }
    pm.into()
}

// ============================================================================
// pixApplyVariableGrayMap
// ============================================================================

#[test]
fn test_apply_variable_gray_map_basic() {
    let pix = make_gray_test_image();
    // Create a uniform gray map at 128
    let map = Pix::new(60, 60, PixelDepth::Bit8).unwrap();
    let mut map_mut = map.try_into_mut().unwrap();
    for y in 0..60 {
        for x in 0..60 {
            map_mut.set_pixel_unchecked(x, y, 128);
        }
    }
    let map: Pix = map_mut.into();

    let result = adaptmap::apply_variable_gray_map(&pix, &map, 128).unwrap();

    assert_eq!(result.width(), 60);
    assert_eq!(result.height(), 60);
    assert_eq!(result.depth(), PixelDepth::Bit8);
}

#[test]
fn test_apply_variable_gray_map_identity() {
    // When map value equals target, output should equal input
    let pix = make_gray_test_image();
    let map = Pix::new(60, 60, PixelDepth::Bit8).unwrap();
    let mut map_mut = map.try_into_mut().unwrap();
    let target = 128u32;
    for y in 0..60 {
        for x in 0..60 {
            map_mut.set_pixel_unchecked(x, y, target);
        }
    }
    let map: Pix = map_mut.into();

    let result = adaptmap::apply_variable_gray_map(&pix, &map, target).unwrap();

    // Each pixel: output = input * target / (map_val + 0.5) ≈ input
    for y in 0..60 {
        for x in 0..60 {
            let orig = pix.get_pixel_unchecked(x, y);
            let mapped = result.get_pixel_unchecked(x, y);
            let diff = (orig as i32 - mapped as i32).unsigned_abs();
            assert!(diff <= 1, "pixel ({x},{y}): orig={orig}, mapped={mapped}");
        }
    }
}

#[test]
fn test_apply_variable_gray_map_size_mismatch() {
    let pix = make_gray_test_image();
    let map = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
    assert!(adaptmap::apply_variable_gray_map(&pix, &map, 128).is_err());
}

#[test]
fn test_apply_variable_gray_map_invalid_map_depth() {
    let pix = make_gray_test_image();
    let map = Pix::new(60, 60, PixelDepth::Bit32).unwrap();
    assert!(adaptmap::apply_variable_gray_map(&pix, &map, 128).is_err());
}

// ============================================================================
// pixGlobalNormRGB
// ============================================================================

#[test]
fn test_global_norm_rgb_basic() {
    let pix = make_color_cast_image();
    let result = adaptmap::global_norm_rgb(&pix, 200, 170, 150, 255).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit32);
}

#[test]
fn test_global_norm_rgb_white_mapping() {
    // If rval=gval=bval and mapval=255, the mapping should be identity-like
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..10 {
        for x in 0..10 {
            pm.set_pixel_unchecked(x, y, color::compose_rgb(128, 128, 128));
        }
    }
    let pix: Pix = pm.into();

    let result = adaptmap::global_norm_rgb(&pix, 255, 255, 255, 255).unwrap();

    // With rval=gval=bval=255 and mapval=255, pixels should be unchanged
    let pixel = result.get_pixel_unchecked(5, 5);
    let (r, g, b, _) = color::extract_rgba(pixel);
    assert_eq!(r, 128);
    assert_eq!(g, 128);
    assert_eq!(b, 128);
}

// ============================================================================
// pixConvertTo8MinMax
// ============================================================================

#[test]
fn test_convert_to_8_min_max_from_8bpp() {
    let pix = make_gray_test_image();
    let result = adaptmap::convert_to_8_min_max(&pix).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit8);
    assert_eq!(result.width(), 60);
}

#[test]
fn test_convert_to_8_min_max_from_32bpp() {
    let pix = make_color_cast_image();
    let result = adaptmap::convert_to_8_min_max(&pix).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit8);
    assert_eq!(result.width(), pix.width());
}

#[test]
fn test_convert_to_8_min_max_from_1bpp() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let result = adaptmap::convert_to_8_min_max(&pix).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit8);
}
