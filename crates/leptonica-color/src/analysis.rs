//! Color content analysis
//!
//! Provides tools for analyzing the color content of images:
//! - Color statistics
//! - Color counting
//! - Grayscale detection

use crate::colorspace::rgb_to_hsv;
use crate::{ColorError, ColorResult};
use leptonica_core::{Pix, PixelDepth, color};
use std::collections::HashSet;

/// Method for computing the color magnitude of a pixel.
///
/// # See also
///
/// C Leptonica: `colorcontent.c` — `L_MAX_DIFF_FROM_AVERAGE_2`, `L_MAX_MIN_DIFF_FROM_2`, `L_MAX_DIFF`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMagnitudeType {
    /// max(|r-g|, |r-b|, |g-b|)
    MaxDiffFromAverage2,
    /// max(r,g,b) - min(r,g,b)
    MaxMinDiffFrom2,
    /// max(|r-g|, |r-b|, |g-b|) using absolute differences
    MaxDiff,
}

/// Statistics about the color content of an image
#[derive(Debug, Clone)]
pub struct ColorStats {
    /// Number of unique colors in the image
    pub unique_colors: u32,
    /// Average hue (0.0-1.0, NaN if grayscale)
    pub mean_hue: f32,
    /// Average saturation (0.0-1.0)
    pub mean_saturation: f32,
    /// Average value/brightness (0.0-1.0)
    pub mean_value: f32,
    /// Whether the image appears to be grayscale
    pub is_grayscale: bool,
    /// Dominant colors (up to 5, sorted by frequency)
    pub dominant_colors: Vec<(u8, u8, u8, u32)>,
}

impl ColorStats {
    /// Create empty stats
    pub fn empty() -> Self {
        Self {
            unique_colors: 0,
            mean_hue: f32::NAN,
            mean_saturation: 0.0,
            mean_value: 0.0,
            is_grayscale: true,
            dominant_colors: Vec::new(),
        }
    }
}

/// Analyze the color content of an image
///
/// Returns statistics about the colors in the image.
pub fn color_content(pix: &Pix) -> ColorResult<ColorStats> {
    match pix.depth() {
        PixelDepth::Bit8 => analyze_grayscale(pix),
        PixelDepth::Bit32 => analyze_color(pix),
        _ => Err(ColorError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

fn analyze_grayscale(pix: &Pix) -> ColorResult<ColorStats> {
    let w = pix.width();
    let h = pix.height();
    let total = (w * h) as f64;

    if total == 0.0 {
        return Ok(ColorStats::empty());
    }

    let mut histogram = [0u32; 256];
    let mut sum_value = 0u64;

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y) as usize;
            histogram[pixel] += 1;
            sum_value += pixel as u64;
        }
    }

    // Count unique values
    let unique_colors = histogram.iter().filter(|&c| *c > 0).count() as u32;

    // Find dominant values
    let mut indexed: Vec<(usize, u32)> = histogram
        .iter()
        .enumerate()
        .filter(|(_, c)| **c > 0)
        .map(|(i, c)| (i, *c))
        .collect();
    indexed.sort_by(|a, b| b.1.cmp(&a.1));

    let dominant_colors: Vec<(u8, u8, u8, u32)> = indexed
        .iter()
        .take(5)
        .map(|&(gray, count)| (gray as u8, gray as u8, gray as u8, count))
        .collect();

    Ok(ColorStats {
        unique_colors,
        mean_hue: f32::NAN,
        mean_saturation: 0.0,
        mean_value: (sum_value as f64 / total / 255.0) as f32,
        is_grayscale: true,
        dominant_colors,
    })
}

fn analyze_color(pix: &Pix) -> ColorResult<ColorStats> {
    let w = pix.width();
    let h = pix.height();
    let total = (w * h) as f64;

    if total == 0.0 {
        return Ok(ColorStats::empty());
    }

    let mut color_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut sum_h = 0.0f64;
    let mut sum_s = 0.0f64;
    let mut sum_v = 0.0f64;
    let mut chromatic_count = 0u64;
    let mut is_grayscale = true;

    const SATURATION_THRESHOLD: f32 = 0.05;

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);

            // Count unique colors (ignore alpha)
            let rgb_key = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            *color_counts.entry(rgb_key).or_insert(0) += 1;

            // Compute HSV
            let hsv = rgb_to_hsv(r, g, b);

            sum_v += hsv.v as f64;

            if hsv.s > SATURATION_THRESHOLD {
                sum_h += hsv.h as f64;
                sum_s += hsv.s as f64;
                chromatic_count += 1;
                is_grayscale = false;
            } else {
                sum_s += hsv.s as f64;
            }
        }
    }

    let unique_colors = color_counts.len() as u32;

    // Compute means
    let mean_hue = if chromatic_count > 0 {
        (sum_h / chromatic_count as f64) as f32
    } else {
        f32::NAN
    };
    let mean_saturation = (sum_s / total) as f32;
    let mean_value = (sum_v / total) as f32;

    // Find dominant colors
    let mut color_vec: Vec<(u32, u32)> = color_counts.into_iter().collect();
    color_vec.sort_by(|a, b| b.1.cmp(&a.1));

    let dominant_colors: Vec<(u8, u8, u8, u32)> = color_vec
        .iter()
        .take(5)
        .map(|&(rgb, count)| {
            let r = ((rgb >> 16) & 0xff) as u8;
            let g = ((rgb >> 8) & 0xff) as u8;
            let b = (rgb & 0xff) as u8;
            (r, g, b, count)
        })
        .collect();

    Ok(ColorStats {
        unique_colors,
        mean_hue,
        mean_saturation,
        mean_value,
        is_grayscale,
        dominant_colors,
    })
}

/// Count the number of unique colors in an image
pub fn count_colors(pix: &Pix) -> ColorResult<u32> {
    match pix.depth() {
        PixelDepth::Bit1 => Ok(2), // Binary always has 2 possible values
        PixelDepth::Bit8 => count_colors_8bpp(pix),
        PixelDepth::Bit32 => count_colors_32bpp(pix),
        _ => Err(ColorError::UnsupportedDepth {
            expected: "1, 8, or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

fn count_colors_8bpp(pix: &Pix) -> ColorResult<u32> {
    let w = pix.width();
    let h = pix.height();
    let mut seen = [false; 256];

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y) as usize;
            seen[pixel] = true;
        }
    }

    Ok(seen.iter().filter(|&&s| s).count() as u32)
}

fn count_colors_32bpp(pix: &Pix) -> ColorResult<u32> {
    let w = pix.width();
    let h = pix.height();
    let mut colors: HashSet<u32> = HashSet::new();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let rgb_key = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            colors.insert(rgb_key);
        }
    }

    Ok(colors.len() as u32)
}

/// Check if an image is grayscale
///
/// For 8-bit images, always returns true.
/// For 32-bit images, checks if R == G == B for all pixels.
pub fn is_grayscale(pix: &Pix) -> ColorResult<bool> {
    match pix.depth() {
        PixelDepth::Bit8 => Ok(true),
        PixelDepth::Bit32 => check_grayscale_32bpp(pix),
        _ => Err(ColorError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

fn check_grayscale_32bpp(pix: &Pix) -> ColorResult<bool> {
    let w = pix.width();
    let h = pix.height();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);

            if r != g || g != b {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Check if an image is grayscale with tolerance
///
/// Allows small differences between R, G, B channels.
pub fn is_grayscale_tolerant(pix: &Pix, tolerance: u8) -> ColorResult<bool> {
    match pix.depth() {
        PixelDepth::Bit8 => Ok(true),
        PixelDepth::Bit32 => check_grayscale_tolerant_32bpp(pix, tolerance),
        _ => Err(ColorError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

fn check_grayscale_tolerant_32bpp(pix: &Pix, tolerance: u8) -> ColorResult<bool> {
    let w = pix.width();
    let h = pix.height();
    let tol = tolerance as i32;

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);

            let r = r as i32;
            let g = g as i32;
            let b = b as i32;

            if (r - g).abs() > tol || (g - b).abs() > tol || (r - b).abs() > tol {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Get histogram of grayscale values
///
/// Returns an array of 256 counts.
pub fn grayscale_histogram(pix: &Pix) -> ColorResult<[u32; 256]> {
    let gray_pix = match pix.depth() {
        PixelDepth::Bit8 => pix.clone(),
        PixelDepth::Bit32 => crate::colorspace::pix_convert_to_gray(pix)?,
        _ => {
            return Err(ColorError::UnsupportedDepth {
                expected: "8 or 32 bpp",
                actual: pix.depth().bits(),
            });
        }
    };

    let w = gray_pix.width();
    let h = gray_pix.height();
    let mut histogram = [0u32; 256];

    for y in 0..h {
        for x in 0..w {
            let pixel = gray_pix.get_pixel_unchecked(x, y) as usize;
            histogram[pixel] += 1;
        }
    }

    Ok(histogram)
}

// ============================================================================
// Advanced color content analysis functions
// ============================================================================

/// Generate a 1bpp mask over pixels with significant color content.
///
/// # See also
///
/// C Leptonica: `pixMaskOverColorPixels()` in `colorcontent.c`
#[allow(unused_variables)]
pub fn mask_over_color_pixels(pix: &Pix, threshdiff: i32, mindist: i32) -> ColorResult<Pix> {
    todo!()
}

/// Generate a 1bpp mask over gray pixels.
///
/// # See also
///
/// C Leptonica: `pixMaskOverGrayPixels()` in `colorcontent.c`
#[allow(unused_variables)]
pub fn mask_over_gray_pixels(pix: &Pix, maxlimit: i32, satlimit: i32) -> ColorResult<Pix> {
    todo!()
}

/// Generate a 1bpp mask for pixels within a specified RGB color range.
///
/// # See also
///
/// C Leptonica: `pixMaskOverColorRange()` in `colorcontent.c`
#[allow(unused_variables)]
pub fn mask_over_color_range(
    pix: &Pix,
    rmin: i32,
    rmax: i32,
    gmin: i32,
    gmax: i32,
    bmin: i32,
    bmax: i32,
) -> ColorResult<Pix> {
    todo!()
}

/// Determine the fraction of pixels that are colored vs gray.
///
/// Returns `(pix_fract, color_fract)` where:
/// - `pix_fract`: fraction of pixels that are neither too dark nor too light
/// - `color_fract`: fraction of those pixels that are colored (not gray)
///
/// # See also
///
/// C Leptonica: `pixColorFraction()` in `colorcontent.c`
#[allow(unused_variables)]
pub fn color_fraction(
    pix: &Pix,
    darkthresh: i32,
    lightthresh: i32,
    diffthresh: i32,
    factor: i32,
) -> ColorResult<(f32, f32)> {
    todo!()
}

/// Count the number of significant gray levels in an 8bpp image.
///
/// # See also
///
/// C Leptonica: `pixNumSignificantGrayColors()` in `colorcontent.c`
#[allow(unused_variables)]
pub fn num_significant_gray_colors(
    pix: &Pix,
    darkthresh: i32,
    lightthresh: i32,
    minfract: f64,
    factor: i32,
) -> ColorResult<u32> {
    todo!()
}

/// Estimate the number of colors for quantization.
///
/// Returns `(ncolors, is_color)` where:
/// - `ncolors`: approximate number of significant colors
/// - `is_color`: whether the image has significant color content
///
/// # See also
///
/// C Leptonica: `pixColorsForQuantization()` in `colorcontent.c`
#[allow(unused_variables)]
pub fn colors_for_quantization(pix: &Pix, thresh: i32) -> ColorResult<(u32, bool)> {
    todo!()
}

/// Compute an RGB histogram with reduced bit resolution.
///
/// Each RGB component is quantized to `sigbits` significant bits, producing
/// a histogram with `2^(3*sigbits)` bins. Valid range for `sigbits`: 2..=6.
///
/// # See also
///
/// C Leptonica: `pixGetRGBHistogram()` in `colorcontent.c`
#[allow(unused_variables)]
pub fn rgb_histogram(pix: &Pix, sigbits: i32, factor: i32) -> ColorResult<Vec<f32>> {
    todo!()
}

/// Find the most populated colors in an RGB image.
///
/// Returns a Vec of `(r, g, b, count)` tuples sorted by count (descending),
/// where the colors are quantized to `sigbits` significant bits.
///
/// # See also
///
/// C Leptonica: `pixGetMostPopulatedColors()` in `colorcontent.c`
#[allow(unused_variables)]
pub fn most_populated_colors(
    pix: &Pix,
    sigbits: i32,
    factor: i32,
    ncolors: i32,
) -> ColorResult<Vec<(u8, u8, u8, u32)>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_grayscale_image() -> Pix {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                let val = x * 25;
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }

        pix_mut.into()
    }

    fn create_color_image() -> Pix {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                let r = (x * 25) as u8;
                let g = (y * 25) as u8;
                let b = 128;
                let pixel = color::compose_rgb(r, g, b);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    fn create_grayscale_as_rgb() -> Pix {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                let gray = ((x + y) * 12) as u8;
                let pixel = color::compose_rgb(gray, gray, gray);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_color_content_grayscale() {
        let pix = create_grayscale_image();
        let stats = color_content(&pix).unwrap();

        assert!(stats.is_grayscale);
        assert!(stats.mean_hue.is_nan());
        assert_eq!(stats.mean_saturation, 0.0);
    }

    #[test]
    fn test_color_content_color() {
        let pix = create_color_image();
        let stats = color_content(&pix).unwrap();

        assert!(!stats.is_grayscale);
        assert!(stats.unique_colors > 1);
        assert!(stats.mean_saturation > 0.0);
    }

    #[test]
    fn test_count_colors_grayscale() {
        let pix = create_grayscale_image();
        let count = count_colors(&pix).unwrap();

        // 10 distinct values (0, 25, 50, 75, 100, 125, 150, 175, 200, 225)
        assert_eq!(count, 10);
    }

    #[test]
    fn test_count_colors_color() {
        let pix = create_color_image();
        let count = count_colors(&pix).unwrap();

        // 10x10 distinct colors
        assert_eq!(count, 100);
    }

    #[test]
    fn test_is_grayscale() {
        let gray_pix = create_grayscale_image();
        assert!(is_grayscale(&gray_pix).unwrap());

        let gray_rgb = create_grayscale_as_rgb();
        assert!(is_grayscale(&gray_rgb).unwrap());

        let color_pix = create_color_image();
        assert!(!is_grayscale(&color_pix).unwrap());
    }

    #[test]
    fn test_is_grayscale_tolerant() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Almost gray - small differences
        for y in 0..5 {
            for x in 0..5 {
                let pixel = color::compose_rgb(128, 130, 127);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        let pix = pix_mut.into();

        // Should fail strict grayscale check
        assert!(!is_grayscale(&pix).unwrap());

        // Should pass with tolerance
        assert!(is_grayscale_tolerant(&pix, 5).unwrap());
    }

    #[test]
    fn test_grayscale_histogram() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with value 100
        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel_unchecked(x, y, 100);
            }
        }

        let histogram = grayscale_histogram(&pix_mut.into()).unwrap();

        // All pixels have value 100
        assert_eq!(histogram[100], 100);

        // No other values
        for (i, &count) in histogram.iter().enumerate() {
            if i != 100 {
                assert_eq!(count, 0);
            }
        }
    }

    #[test]
    fn test_dominant_colors() {
        let pix = Pix::new(30, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // 20 pixels red, 10 pixels blue
        for y in 0..10 {
            for x in 0..30 {
                let pixel = if x < 20 {
                    color::compose_rgb(255, 0, 0)
                } else {
                    color::compose_rgb(0, 0, 255)
                };
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        let stats = color_content(&pix_mut.into()).unwrap();

        assert_eq!(stats.dominant_colors.len(), 2);
        // Red should be first (more frequent)
        assert_eq!(stats.dominant_colors[0], (255, 0, 0, 200));
        assert_eq!(stats.dominant_colors[1], (0, 0, 255, 100));
    }
}
