//! Test fast separable bilateral filtering
//!
//! C版: reference/leptonica/src/bilateral.c
//! - pixBilateral
//! - pixBilateralGray

use leptonica_core::{Pix, PixelDepth, color};
use leptonica_filter::bilateral;

/// Create a grayscale test image with a sharp edge
fn make_gray_edge_image() -> Pix {
    let pix = Pix::new(60, 60, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..60 {
        for x in 0..60 {
            let val = if x < 30 { 50u32 } else { 200 };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

/// Create a 32bpp color test image with edges
fn make_color_edge_image() -> Pix {
    let pix = Pix::new(60, 60, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..60 {
        for x in 0..60 {
            let r = if x < 30 { 40u8 } else { 220 };
            let g = if y < 30 { 60u8 } else { 180 };
            let b = 128u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(r, g, b));
        }
    }
    pm.into()
}

// ============================================================================
// pixBilateralGray
// ============================================================================

#[test]
fn test_bilateral_gray_basic() {
    let pix = make_gray_edge_image();
    let result = bilateral::bilateral_gray(&pix, 5.0, 50.0, 6, 2).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit8);
}

#[test]
fn test_bilateral_gray_preserves_edges() {
    let pix = make_gray_edge_image();
    let result = bilateral::bilateral_gray(&pix, 5.0, 30.0, 6, 1).unwrap();

    // Edge should be preserved: pixels far from edge should remain distinct
    let val_left = result.get_pixel_unchecked(10, 30);
    let val_right = result.get_pixel_unchecked(50, 30);
    assert!(
        val_right > val_left + 50,
        "edge not preserved: left={val_left}, right={val_right}"
    );
}

#[test]
fn test_bilateral_gray_reduction_2() {
    let pix = make_gray_edge_image();
    let result = bilateral::bilateral_gray(&pix, 5.0, 50.0, 6, 2).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
}

#[test]
fn test_bilateral_gray_reduction_4() {
    let pix = make_gray_edge_image();
    let result = bilateral::bilateral_gray(&pix, 10.0, 50.0, 6, 4).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
}

#[test]
fn test_bilateral_gray_invalid_params() {
    let pix = make_gray_edge_image();

    // reduction must be 1, 2, or 4
    assert!(bilateral::bilateral_gray(&pix, 5.0, 50.0, 6, 3).is_err());

    // range_stdev must be > 5.0
    assert!(bilateral::bilateral_gray(&pix, 5.0, 4.0, 6, 1).is_err());

    // ncomps must be in [4..30]
    assert!(bilateral::bilateral_gray(&pix, 5.0, 50.0, 3, 1).is_err());
    assert!(bilateral::bilateral_gray(&pix, 5.0, 50.0, 31, 1).is_err());

    // ncomps * range_stdev >= 100
    assert!(bilateral::bilateral_gray(&pix, 5.0, 10.0, 4, 1).is_err());
}

#[test]
fn test_bilateral_gray_small_image_returns_copy() {
    // Image too small for filter — should return a copy
    let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
    let result = bilateral::bilateral_gray(&pix, 5.0, 50.0, 6, 1).unwrap();
    assert_eq!(result.width(), 5);
    assert_eq!(result.height(), 5);
}

#[test]
fn test_bilateral_gray_non_divisible_size() {
    // Test with dimensions not evenly divisible by reduction factor
    let pix = Pix::new(65, 65, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..65 {
        for x in 0..65 {
            let val = if x < 32 { 50u32 } else { 200 };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    let pix: Pix = pm.into();

    // Should work without panicking for all reduction factors
    let result = bilateral::bilateral_gray(&pix, 5.0, 50.0, 6, 2).unwrap();
    assert_eq!(result.width(), 65);
    assert_eq!(result.height(), 65);

    let result = bilateral::bilateral_gray(&pix, 10.0, 50.0, 6, 4).unwrap();
    assert_eq!(result.width(), 65);
    assert_eq!(result.height(), 65);
}

// ============================================================================
// pixBilateral
// ============================================================================

#[test]
fn test_bilateral_fast_gray() {
    let pix = make_gray_edge_image();
    let result = bilateral::bilateral(&pix, 5.0, 50.0, 6, 2).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit8);
}

#[test]
fn test_bilateral_fast_color() {
    let pix = make_color_edge_image();
    let result = bilateral::bilateral(&pix, 5.0, 50.0, 6, 2).unwrap();

    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
    assert_eq!(result.depth(), PixelDepth::Bit32);
}

#[test]
fn test_bilateral_fast_color_preserves_edges() {
    let pix = make_color_edge_image();
    let result = bilateral::bilateral(&pix, 5.0, 30.0, 6, 1).unwrap();

    // Check red channel edge preservation
    let left_pixel = result.get_pixel_unchecked(10, 30);
    let right_pixel = result.get_pixel_unchecked(50, 30);
    let (r_left, _, _, _) = color::extract_rgba(left_pixel);
    let (r_right, _, _, _) = color::extract_rgba(right_pixel);
    assert!(
        r_right > r_left + 50,
        "red edge not preserved: left={r_left}, right={r_right}"
    );
}

#[test]
fn test_bilateral_fast_invalid_depth() {
    let pix = Pix::new(60, 60, PixelDepth::Bit1).unwrap();
    assert!(bilateral::bilateral(&pix, 5.0, 50.0, 6, 1).is_err());
}
