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
/// C Leptonica: `colorcontent.c` — `L_INTERMED_DIFF`, `L_AVE_MAX_DIFF_2`, `L_MAX_DIFF`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMagnitudeType {
    /// Intermediate value of the three pairwise differences |r-g|, |r-b|, |g-b|
    IntermedDiff,
    /// Max over components of the difference between one component and
    /// the average of the other two (equivalent to averaging the two closest
    /// components and taking their distance to the third)
    AveMaxDiff2,
    /// Maximum of the three pairwise differences |r-g|, |r-b|, |g-b|
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
/// A pixel is considered "color" if `max(r,g,b) - min(r,g,b) >= threshdiff`.
/// If `mindist > 1`, the mask is eroded to remove transition artifacts
/// near edges (requires morphological erosion, currently unimplemented).
///
/// # See also
///
/// C Leptonica: `pixMaskOverColorPixels()` in `colorcontent.c`
pub fn mask_over_color_pixels(pix: &Pix, threshdiff: i32, mindist: i32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    let w = pix.width();
    let h = pix.height();
    let mask = Pix::new(w, h, PixelDepth::Bit1).map_err(ColorError::Core)?;
    let mut mask_mut = mask.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let ri = r as i32;
            let gi = g as i32;
            let bi = b as i32;
            let minval = ri.min(gi).min(bi);
            let maxval = ri.max(gi).max(bi);
            if maxval - minval >= threshdiff {
                mask_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    // Note: erosion for mindist > 1 requires morphological operations
    // from leptonica-morph which is not a dependency of this crate.
    let _ = mindist;

    Ok(mask_mut.into())
}

/// Generate a 1bpp mask over gray pixels.
///
/// A pixel is considered "gray" if:
/// - `max(r,g,b) <= maxlimit` (not too bright)
/// - `max(r,g,b) - min(r,g,b) <= satlimit` (low saturation)
///
/// # See also
///
/// C Leptonica: `pixMaskOverGrayPixels()` in `colorcontent.c`
pub fn mask_over_gray_pixels(pix: &Pix, maxlimit: i32, satlimit: i32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    let w = pix.width();
    let h = pix.height();
    let mask = Pix::new(w, h, PixelDepth::Bit1).map_err(ColorError::Core)?;
    let mut mask_mut = mask.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let ri = r as i32;
            let gi = g as i32;
            let bi = b as i32;
            let maxval = ri.max(gi).max(bi);
            let minval = ri.min(gi).min(bi);
            let sat = maxval - minval;
            if maxval <= maxlimit && sat <= satlimit {
                mask_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok(mask_mut.into())
}

/// Generate a 1bpp mask for pixels within a specified RGB color range.
///
/// A pixel is set in the mask if all three components fall within their
/// respective `[min, max]` ranges simultaneously.
///
/// # See also
///
/// C Leptonica: `pixMaskOverColorRange()` in `colorcontent.c`
pub fn mask_over_color_range(
    pix: &Pix,
    rmin: i32,
    rmax: i32,
    gmin: i32,
    gmax: i32,
    bmin: i32,
    bmax: i32,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    let w = pix.width();
    let h = pix.height();
    let mask = Pix::new(w, h, PixelDepth::Bit1).map_err(ColorError::Core)?;
    let mut mask_mut = mask.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let rv = r as i32;
            let gv = g as i32;
            let bv = b as i32;
            if rv < rmin || rv > rmax {
                continue;
            }
            if gv < gmin || gv > gmax {
                continue;
            }
            if bv < bmin || bv > bmax {
                continue;
            }
            mask_mut.set_pixel_unchecked(x, y, 1);
        }
    }

    Ok(mask_mut.into())
}

/// Determine the fraction of pixels that are colored vs gray.
///
/// Returns `(pix_fract, color_fract)` where:
/// - `pix_fract`: fraction of sampled pixels that are neither too dark nor too light
/// - `color_fract`: fraction of those valid pixels that have sufficient color
///
/// A pixel is excluded if `min(r,g,b) > lightthresh` (near white) or
/// `max(r,g,b) < darkthresh` (near black). Of the remaining valid pixels,
/// those with `max - min >= diffthresh` are counted as colored.
///
/// # See also
///
/// C Leptonica: `pixColorFraction()` in `colorcontent.c`
pub fn color_fraction(
    pix: &Pix,
    darkthresh: i32,
    lightthresh: i32,
    diffthresh: i32,
    factor: i32,
) -> ColorResult<(f32, f32)> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    let w = pix.width();
    let h = pix.height();
    let factor = factor.max(1) as u32;
    let mut total = 0u32;
    let mut npix = 0u32;
    let mut ncolor = 0u32;

    let mut y = 0;
    while y < h {
        let mut x = 0;
        while x < w {
            total += 1;
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let ri = r as i32;
            let gi = g as i32;
            let bi = b as i32;
            let minval = ri.min(gi).min(bi);
            let maxval = ri.max(gi).max(bi);
            if minval > lightthresh {
                x += factor;
                continue;
            }
            if maxval < darkthresh {
                x += factor;
                continue;
            }
            npix += 1;
            if maxval - minval >= diffthresh {
                ncolor += 1;
            }
            x += factor;
        }
        y += factor;
    }

    if npix == 0 {
        return Ok((0.0, 0.0));
    }

    let pix_fract = npix as f32 / total as f32;
    let color_fract = ncolor as f32 / npix as f32;
    Ok((pix_fract, color_fract))
}

/// Count the number of significant gray levels in an 8bpp image.
///
/// Gray levels in `[darkthresh, lightthresh]` are counted if their population
/// fraction meets `minfract`. The `factor` parameter controls subsampling.
///
/// # See also
///
/// C Leptonica: `pixNumSignificantGrayColors()` in `colorcontent.c`
pub fn num_significant_gray_colors(
    pix: &Pix,
    darkthresh: i32,
    lightthresh: i32,
    minfract: f64,
    factor: i32,
) -> ColorResult<u32> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(ColorError::UnsupportedDepth {
            expected: "8 bpp",
            actual: pix.depth().bits(),
        });
    }
    let w = pix.width();
    let h = pix.height();
    let factor = factor.max(1) as u32;

    // Build histogram with subsampling
    let mut histogram = [0u32; 256];
    let mut y = 0;
    while y < h {
        let mut x = 0;
        while x < w {
            let val = pix.get_pixel_unchecked(x, y) as usize;
            histogram[val] += 1;
            x += factor;
        }
        y += factor;
    }

    // Minimum count threshold (matches C: minfract * w * h * factor^2)
    let mincount = (minfract * w as f64 * h as f64 * factor as f64 * factor as f64).max(0.0) as u32;

    let dt = darkthresh.max(0) as usize;
    let lt = lightthresh.min(255) as usize;
    let ncolors = histogram[dt..=lt]
        .iter()
        .filter(|&&count| count >= mincount)
        .count() as u32;

    Ok(ncolors)
}

/// Estimate the number of colors for quantization.
///
/// Returns `(ncolors, is_color)` where:
/// - `ncolors`: approximate number of significant colors in smooth regions
/// - `is_color`: whether the image has significant color content
///
/// For 32bpp images, uses `color_fraction` to determine if the image is
/// primarily grayscale or color, then counts colors accordingly.
///
/// Note: This is a simplified implementation that omits edge masking
/// (which would require morphological operations from leptonica-morph).
///
/// # See also
///
/// C Leptonica: `pixColorsForQuantization()` in `colorcontent.c`
pub fn colors_for_quantization(pix: &Pix, thresh: i32) -> ColorResult<(u32, bool)> {
    let w = pix.width();
    let h = pix.height();
    let d = pix.depth();
    let _thresh = if thresh <= 0 { 15 } else { thresh };

    if d == PixelDepth::Bit8 {
        let ncolors = num_significant_gray_colors(pix, 20, 236, 0.0001, 1)?;
        return Ok((ncolors, false));
    }

    if d != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: d.bits(),
        });
    }

    // Check if image has significant color
    let minside = w.min(h);
    let factor = (minside / 400).max(1) as i32;
    let (pixfract, colorfract) = color_fraction(pix, 20, 248, 30, factor)?;

    if pixfract * colorfract < 0.00025 {
        // Treat as grayscale: count significant gray levels from red channel
        let mut histogram = [0u32; 256];
        for y in 0..h {
            for x in 0..w {
                let pixel = pix.get_pixel_unchecked(x, y);
                let r = color::red(pixel);
                histogram[r as usize] += 1;
            }
        }
        let mincount = (0.0001f64 * w as f64 * h as f64).max(1.0) as u32;
        let mut ncolors = 0u32;
        for count in histogram.iter().take(237).skip(20) {
            if *count >= mincount {
                ncolors += 1;
            }
        }
        Ok((ncolors, false))
    } else {
        // Color image: count occupied level-4 octcubes (16 divisions per component)
        let mut cubes = std::collections::HashMap::new();
        for y in 0..h {
            for x in 0..w {
                let pixel = pix.get_pixel_unchecked(x, y);
                let (r, g, b) = color::extract_rgb(pixel);
                let cube = ((r >> 4) as u32) << 8 | ((g >> 4) as u32) << 4 | (b >> 4) as u32;
                *cubes.entry(cube).or_insert(0u32) += 1;
            }
        }
        let min_octcube_count = 20u32;
        let ncolors = cubes.values().filter(|&&c| c >= min_octcube_count).count() as u32;
        Ok((ncolors, true))
    }
}

/// Compute an RGB histogram with reduced bit resolution.
///
/// Each RGB component is quantized to `sigbits` significant bits, producing
/// a histogram with `2^(3*sigbits)` bins. Valid range for `sigbits`: 2..=6.
///
/// # See also
///
/// C Leptonica: `pixGetRGBHistogram()` in `colorcontent.c`
pub fn rgb_histogram(pix: &Pix, sigbits: i32, factor: i32) -> ColorResult<Vec<f32>> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if !(2..=6).contains(&sigbits) {
        return Err(ColorError::InvalidParameters(
            "sigbits must be in [2, 6]".into(),
        ));
    }
    let factor = factor.max(1) as u32;
    let w = pix.width();
    let h = pix.height();

    let size = 1usize << (3 * sigbits);
    let mut hist = vec![0.0f32; size];

    let (rtab, gtab, btab) = make_rgb_index_tables(sigbits);

    let mut y = 0;
    while y < h {
        let mut x = 0;
        while x < w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let rgbindex = rtab[r as usize] | gtab[g as usize] | btab[b as usize];
            hist[rgbindex as usize] += 1.0;
            x += factor;
        }
        y += factor;
    }

    Ok(hist)
}

/// Find the most populated colors in an RGB image.
///
/// Returns a Vec of `(r, g, b, count)` tuples sorted by count (descending),
/// where the colors are quantized to `sigbits` significant bits.
/// Only bins with count > 0 are returned.
///
/// # See also
///
/// C Leptonica: `pixGetMostPopulatedColors()` in `colorcontent.c`
pub fn most_populated_colors(
    pix: &Pix,
    sigbits: i32,
    factor: i32,
    ncolors: i32,
) -> ColorResult<Vec<(u8, u8, u8, u32)>> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if !(2..=6).contains(&sigbits) {
        return Err(ColorError::InvalidParameters(
            "sigbits must be in [2, 6]".into(),
        ));
    }
    if factor < 1 || ncolors < 1 {
        return Err(ColorError::InvalidParameters(
            "factor and ncolors must be >= 1".into(),
        ));
    }

    let hist = rgb_histogram(pix, sigbits, factor)?;

    // Collect non-zero bins with their indices
    let mut bins: Vec<(usize, f32)> = hist
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0.0)
        .map(|(idx, &count)| (idx, count))
        .collect();

    // Sort by count descending
    bins.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top ncolors entries
    let n = (ncolors as usize).min(bins.len());
    let result = bins[..n]
        .iter()
        .map(|&(idx, count)| {
            let (r, g, b) = get_rgb_from_index(idx as u32, sigbits);
            (r, g, b, count as u32)
        })
        .collect();

    Ok(result)
}

// ============================================================================
// Helper functions for RGB histogram indexing
// ============================================================================

/// Build RGB index lookup tables for histogram computation.
///
/// Returns `(rtab, gtab, btab)` where `rgbindex = rtab[r] | gtab[g] | btab[b]`.
/// The index format is: `r_bits g_bits b_bits` (red most significant).
fn make_rgb_index_tables(sigbits: i32) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut rtab = vec![0u32; 256];
    let mut gtab = vec![0u32; 256];
    let mut btab = vec![0u32; 256];

    for i in 0..256u32 {
        let idx = i as usize;
        match sigbits {
            2 => {
                rtab[idx] = (i & 0xc0) >> 2;
                gtab[idx] = (i & 0xc0) >> 4;
                btab[idx] = (i & 0xc0) >> 6;
            }
            3 => {
                rtab[idx] = (i & 0xe0) << 1;
                gtab[idx] = (i & 0xe0) >> 2;
                btab[idx] = (i & 0xe0) >> 5;
            }
            4 => {
                rtab[idx] = (i & 0xf0) << 4;
                gtab[idx] = i & 0xf0;
                btab[idx] = (i & 0xf0) >> 4;
            }
            5 => {
                rtab[idx] = (i & 0xf8) << 7;
                gtab[idx] = (i & 0xf8) << 2;
                btab[idx] = (i & 0xf8) >> 3;
            }
            6 => {
                rtab[idx] = (i & 0xfc) << 10;
                gtab[idx] = (i & 0xfc) << 4;
                btab[idx] = (i & 0xfc) >> 2;
            }
            _ => unreachable!("sigbits must be 2..=6"),
        }
    }

    (rtab, gtab, btab)
}

/// Convert an RGB index back to RGB values at the center of the quantized cube.
fn get_rgb_from_index(index: u32, sigbits: i32) -> (u8, u8, u8) {
    let (r, g, b) = match sigbits {
        2 => (
            ((index << 2) & 0xc0) | 0x20,
            ((index << 4) & 0xc0) | 0x20,
            ((index << 6) & 0xc0) | 0x20,
        ),
        3 => (
            ((index >> 1) & 0xe0) | 0x10,
            ((index << 2) & 0xe0) | 0x10,
            ((index << 5) & 0xe0) | 0x10,
        ),
        4 => (
            ((index >> 4) & 0xf0) | 0x08,
            (index & 0xf0) | 0x08,
            ((index << 4) & 0xf0) | 0x08,
        ),
        5 => (
            ((index >> 7) & 0xf8) | 0x04,
            ((index >> 2) & 0xf8) | 0x04,
            ((index << 3) & 0xf8) | 0x04,
        ),
        6 => (
            ((index >> 10) & 0xfc) | 0x02,
            ((index >> 4) & 0xfc) | 0x02,
            ((index << 2) & 0xfc) | 0x02,
        ),
        _ => unreachable!("sigbits must be 2..=6"),
    };
    (r as u8, g as u8, b as u8)
}

/// Compute the color magnitude of each pixel in a 32bpp RGB image.
///
/// For each pixel, computes a measure of how "colorful" it is (i.e., how
/// far from gray). Returns an 8bpp grayscale image where 0 = pure gray
/// and 255 = maximum color deviation.
///
/// Three computation methods are available:
/// - [`ColorMagnitudeType::IntermedDiff`]: Median of the three pairwise
///   component differences |r-g|, |r-b|, |g-b|
/// - [`ColorMagnitudeType::AveMaxDiff2`]: Maximum distance each component
///   has from the average of the other two
/// - [`ColorMagnitudeType::MaxDiff`]: max(r,g,b) - min(r,g,b)
///
/// # Arguments
///
/// * `pix` - 32bpp RGB image
/// * `mag_type` - Method for computing the color magnitude
///
/// # See also
///
/// C Leptonica: `pixColorMagnitude()` in `colorcontent.c`
pub fn color_magnitude(pix: &Pix, mag_type: ColorMagnitudeType) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit8)
        .map_err(|e| ColorError::InvalidParameters(format!("failed to create output: {e}")))?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let (ri, gi, bi) = (r as i32, g as i32, b as i32);

            let colorval = match mag_type {
                ColorMagnitudeType::IntermedDiff => {
                    // Median of the three pairwise absolute differences
                    let rgdist = (ri - gi).abs();
                    let rbdist = (ri - bi).abs();
                    let gbdist = (gi - bi).abs();
                    let maxdist = rgdist.max(rbdist);
                    if gbdist >= maxdist {
                        maxdist
                    } else {
                        let mindist = rgdist.min(rbdist);
                        mindist.max(gbdist)
                    }
                }
                ColorMagnitudeType::AveMaxDiff2 => {
                    // Max distance each component has from average of other two
                    let rdist = ((gi + bi) / 2 - ri).abs();
                    let gdist = ((ri + bi) / 2 - gi).abs();
                    let bdist = ((ri + gi) / 2 - bi).abs();
                    rdist.max(gdist).max(bdist)
                }
                ColorMagnitudeType::MaxDiff => {
                    // max(r,g,b) - min(r,g,b)
                    let minval = ri.min(gi).min(bi);
                    let maxval = ri.max(gi).max(bi);
                    maxval - minval
                }
            };

            out_mut.set_pixel_unchecked(x, y, colorval as u32);
        }
    }

    Ok(out_mut.into())
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
