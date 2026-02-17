//! Test advanced color content analysis functions
//!
//! # See also
//!
//! C Leptonica: `colorcontent.c`
//! - pixMaskOverColorPixels, pixMaskOverGrayPixels, pixMaskOverColorRange
//! - pixColorFraction, pixNumSignificantGrayColors, pixColorsForQuantization
//! - pixGetRGBHistogram, pixGetMostPopulatedColors

use leptonica_color::analysis::{
    color_fraction, colors_for_quantization, mask_over_color_pixels, mask_over_color_range,
    mask_over_gray_pixels, most_populated_colors, num_significant_gray_colors, rgb_histogram,
};
use leptonica_core::{Pix, PixelDepth, color};

/// Create a uniform RGB image
fn make_uniform_rgb(r: u8, g: u8, b: u8, w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let pixel = color::compose_rgb(r, g, b);
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, pixel);
        }
    }
    pm.into()
}

/// Create a 3-color image: red (left), green (middle), blue (right)
fn make_tricolor(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let third = w / 3;
    for y in 0..h {
        for x in 0..w {
            let pixel = if x < third {
                color::compose_rgb(255, 0, 0)
            } else if x < 2 * third {
                color::compose_rgb(0, 255, 0)
            } else {
                color::compose_rgb(0, 0, 255)
            };
            pm.set_pixel_unchecked(x, y, pixel);
        }
    }
    pm.into()
}

/// Create a grayscale image stored as 32bpp RGB
fn make_gray_as_rgb(w: u32, h: u32, val: u8) -> Pix {
    make_uniform_rgb(val, val, val, w, h)
}

/// Create a gradient gray image stored as 32bpp RGB (values 0..255 across width)
#[allow(dead_code)]
fn make_gray_gradient_rgb(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let v = ((x as f32 / w as f32) * 255.0) as u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(v, v, v));
        }
    }
    pm.into()
}

// ============================================================================
// mask_over_color_pixels
// ============================================================================

#[test]
fn test_mask_over_color_pixels_all_colored() {
    let pix = make_tricolor(30, 10);
    let mask = mask_over_color_pixels(&pix, 30, 0).unwrap();
    assert_eq!(mask.depth(), PixelDepth::Bit1);
    assert_eq!(mask.width(), 30);
    // All pixels are strongly colored → all ON
    let on_count: u32 = (0..10)
        .flat_map(|y| (0..30).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 300);
}

#[test]
fn test_mask_over_color_pixels_gray_image() {
    let pix = make_gray_as_rgb(20, 20, 128);
    let mask = mask_over_color_pixels(&pix, 30, 0).unwrap();
    // Gray pixels have no color difference → all OFF
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0);
}

#[test]
fn test_mask_over_color_pixels_invalid_depth() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(mask_over_color_pixels(&pix, 30, 0).is_err());
}

// ============================================================================
// mask_over_gray_pixels
// ============================================================================

#[test]
fn test_mask_over_gray_pixels_gray_image() {
    let pix = make_gray_as_rgb(20, 20, 128);
    let mask = mask_over_gray_pixels(&pix, 200, 20).unwrap();
    // All gray (val=128 <= maxlimit=200, sat=0 <= satlimit=20) → all ON
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 400);
}

#[test]
fn test_mask_over_gray_pixels_color_image() {
    let pix = make_tricolor(30, 10);
    let mask = mask_over_gray_pixels(&pix, 200, 20).unwrap();
    // Saturated colors should not be masked
    let on_count: u32 = (0..10)
        .flat_map(|y| (0..30).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0);
}

// ============================================================================
// mask_over_color_range
// ============================================================================

#[test]
fn test_mask_over_color_range_exact() {
    let pix = make_uniform_rgb(100, 150, 200, 20, 20);
    let mask = mask_over_color_range(&pix, 90, 110, 140, 160, 190, 210).unwrap();
    // All pixels in range → all ON
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 400);
}

#[test]
fn test_mask_over_color_range_none() {
    let pix = make_uniform_rgb(100, 150, 200, 20, 20);
    let mask = mask_over_color_range(&pix, 200, 255, 0, 50, 0, 50).unwrap();
    // No pixels in range → all OFF
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0);
}

#[test]
fn test_mask_over_color_range_partial() {
    // Left half red, right half blue
    let pix = Pix::new(20, 10, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..10u32 {
        for x in 0..20u32 {
            let pixel = if x < 10 {
                color::compose_rgb(255, 0, 0)
            } else {
                color::compose_rgb(0, 0, 255)
            };
            pm.set_pixel_unchecked(x, y, pixel);
        }
    }
    let pix: Pix = pm.into();
    // Select red range only
    let mask = mask_over_color_range(&pix, 200, 255, 0, 50, 0, 50).unwrap();
    let on_count: u32 = (0..10)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 100); // Only left half
}

// ============================================================================
// color_fraction
// ============================================================================

#[test]
fn test_color_fraction_all_colored() {
    let pix = make_tricolor(30, 10);
    let (pix_fract, color_fract) = color_fraction(&pix, 20, 235, 15, 1).unwrap();
    // All pixels are colored (pure R/G/B), none are dark or light
    assert!(pix_fract > 0.9, "pix_fract = {pix_fract}");
    assert!(color_fract > 0.9, "color_fract = {color_fract}");
}

#[test]
fn test_color_fraction_all_gray() {
    let pix = make_gray_as_rgb(20, 20, 128);
    let (pix_fract, color_fract) = color_fraction(&pix, 20, 235, 15, 1).unwrap();
    // Gray image: no colored pixels
    assert!(pix_fract > 0.9, "pix_fract = {pix_fract}");
    assert!(color_fract < 0.01, "color_fract = {color_fract}");
}

#[test]
fn test_color_fraction_dark_image() {
    let pix = make_gray_as_rgb(20, 20, 10); // very dark
    let (_pix_fract, _color_fract) = color_fraction(&pix, 20, 235, 15, 1).unwrap();
    // Dark pixels are excluded from "non-color" fraction
}

// ============================================================================
// num_significant_gray_colors
// ============================================================================

#[test]
fn test_num_significant_gray_colors_uniform() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..20 {
        for x in 0..20 {
            pm.set_pixel_unchecked(x, y, 128);
        }
    }
    let pix: Pix = pm.into();
    let n = num_significant_gray_colors(&pix, 20, 235, 0.01, 1).unwrap();
    assert_eq!(n, 1);
}

#[test]
fn test_num_significant_gray_colors_gradient() {
    // Create 8bpp gradient with many distinct gray levels
    let pix = Pix::new(256, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..10 {
        for x in 0..256u32 {
            pm.set_pixel_unchecked(x, y, x);
        }
    }
    let pix: Pix = pm.into();
    // Between 20 and 235, many levels are significant
    let n = num_significant_gray_colors(&pix, 20, 235, 0.001, 1).unwrap();
    assert!(n > 100, "expected many gray colors, got {n}");
}

#[test]
fn test_num_significant_gray_colors_invalid_depth() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    assert!(num_significant_gray_colors(&pix, 20, 235, 0.01, 1).is_err());
}

// ============================================================================
// colors_for_quantization
// ============================================================================

#[test]
fn test_colors_for_quantization_few_colors() {
    let pix = make_tricolor(30, 10);
    let (ncolors, is_color) = colors_for_quantization(&pix, 15).unwrap();
    assert!(is_color);
    assert!(ncolors <= 10, "expected few colors, got {ncolors}");
}

#[test]
fn test_colors_for_quantization_grayscale() {
    let pix = make_gray_as_rgb(20, 20, 128);
    let (_ncolors, is_color) = colors_for_quantization(&pix, 15).unwrap();
    assert!(!is_color);
}

// ============================================================================
// rgb_histogram
// ============================================================================

#[test]
fn test_rgb_histogram_uniform() {
    let pix = make_uniform_rgb(100, 150, 200, 20, 20);
    let hist = rgb_histogram(&pix, 5, 1).unwrap();
    // With 5 sigbits, histogram has 2^15 = 32768 bins
    assert_eq!(hist.len(), 32768);
    // Only one bin should be non-zero (count = 400)
    let nonzero: Vec<_> = hist.iter().filter(|&&v| v > 0.0).collect();
    assert_eq!(nonzero.len(), 1);
    assert_eq!(*nonzero[0], 400.0);
}

#[test]
fn test_rgb_histogram_sigbits() {
    let pix = make_uniform_rgb(100, 150, 200, 10, 10);
    // sigbits=2: 2^6 = 64 bins
    let hist2 = rgb_histogram(&pix, 2, 1).unwrap();
    assert_eq!(hist2.len(), 64);
    // sigbits=6: 2^18 = 262144 bins
    let hist6 = rgb_histogram(&pix, 6, 1).unwrap();
    assert_eq!(hist6.len(), 262144);
}

#[test]
fn test_rgb_histogram_invalid_sigbits() {
    let pix = make_uniform_rgb(100, 150, 200, 10, 10);
    assert!(rgb_histogram(&pix, 1, 1).is_err()); // too low
    assert!(rgb_histogram(&pix, 7, 1).is_err()); // too high
}

#[test]
fn test_rgb_histogram_invalid_depth() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(rgb_histogram(&pix, 5, 1).is_err());
}

// ============================================================================
// most_populated_colors
// ============================================================================

#[test]
fn test_most_populated_colors_basic() {
    let pix = make_tricolor(30, 10);
    let colors = most_populated_colors(&pix, 5, 1, 5).unwrap();
    // Should find 3 dominant colors
    assert_eq!(colors.len(), 3);
}

#[test]
fn test_most_populated_colors_uniform() {
    let pix = make_uniform_rgb(100, 150, 200, 20, 20);
    let colors = most_populated_colors(&pix, 5, 1, 5).unwrap();
    assert!(colors.len() >= 1);
    // The single color should dominate
    let (r, g, b, count) = colors[0];
    // With sigbits=5, quantized values: 100>>3<<3 = 96, 150>>3<<3 = 148 (approx)
    assert_eq!(count, 400);
    // Values are approximate due to quantization
    assert!((r as i32 - 100).abs() < 10);
    assert!((g as i32 - 150).abs() < 10);
    assert!((b as i32 - 200).abs() < 10);
}
