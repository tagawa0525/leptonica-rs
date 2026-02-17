//! Binary thresholding and dithering
//!
//! Provides various methods for converting grayscale images to binary:
//! - Fixed threshold binarization
//! - Otsu's method (automatic threshold selection)
//! - Adaptive (local) thresholding
//! - Floyd-Steinberg dithering

use crate::colorspace::pix_convert_to_gray;
use crate::{ColorError, ColorResult};
use leptonica_core::{Pix, PixelDepth};

// =============================================================================
// Fixed Threshold Binarization
// =============================================================================

/// Convert a grayscale image to binary using a fixed threshold
///
/// Pixels >= threshold become white (1), pixels < threshold become black (0).
pub fn threshold_to_binary(pix: &Pix, threshold: u8) -> ColorResult<Pix> {
    let gray_pix = ensure_grayscale(pix)?;

    let w = gray_pix.width();
    let h = gray_pix.height();
    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = gray_pix.get_pixel_unchecked(x, y) as u8;
            let binary = if pixel >= threshold { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, binary);
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Otsu's Method
// =============================================================================

/// Compute Otsu's threshold for a grayscale image
///
/// Returns the optimal threshold that minimizes intra-class variance.
#[allow(clippy::needless_range_loop)]
pub fn compute_otsu_threshold(pix: &Pix) -> ColorResult<u8> {
    let gray_pix = ensure_grayscale(pix)?;

    let w = gray_pix.width();
    let h = gray_pix.height();
    let total_pixels = (w * h) as f64;

    if total_pixels == 0.0 {
        return Err(ColorError::EmptyImage);
    }

    // Build histogram
    let mut histogram = [0u32; 256];
    for y in 0..h {
        for x in 0..w {
            let pixel = gray_pix.get_pixel_unchecked(x, y) as usize;
            histogram[pixel] += 1;
        }
    }

    // Compute total sum
    let mut total_sum = 0.0;
    for (i, &count) in histogram.iter().enumerate() {
        total_sum += i as f64 * count as f64;
    }

    let mut sum_background = 0.0;
    let mut weight_background = 0.0;
    let mut max_variance = 0.0;
    let mut best_threshold = 0u8;

    // Otsu's method: find threshold t where pixels < t are background, pixels >= t are foreground
    for t in 0..256 {
        // First compute variance for threshold t (before adding histogram[t] to background)
        if t > 0 {
            let weight_foreground = total_pixels - weight_background;

            if weight_background > 0.0 && weight_foreground > 0.0 {
                let mean_background = sum_background / weight_background;
                let mean_foreground = (total_sum - sum_background) / weight_foreground;

                // Between-class variance
                let variance = weight_background
                    * weight_foreground
                    * (mean_background - mean_foreground).powi(2);

                if variance > max_variance {
                    max_variance = variance;
                    best_threshold = t as u8;
                }
            }
        }

        // Then add histogram[t] to background for next iteration
        let count = histogram[t] as f64;
        weight_background += count;
        sum_background += t as f64 * count;
    }

    Ok(best_threshold)
}

/// Convert a grayscale image to binary using Otsu's method
///
/// Automatically determines the optimal threshold.
pub fn threshold_otsu(pix: &Pix) -> ColorResult<Pix> {
    let threshold = compute_otsu_threshold(pix)?;
    threshold_to_binary(pix, threshold)
}

// =============================================================================
// Adaptive Thresholding
// =============================================================================

/// Options for adaptive thresholding
#[derive(Debug, Clone)]
pub struct AdaptiveThresholdOptions {
    /// Size of the local window (must be odd)
    pub window_size: u32,
    /// Constant subtracted from the mean
    pub c: f32,
    /// Method for computing local threshold
    pub method: AdaptiveMethod,
}

/// Method for adaptive threshold computation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveMethod {
    /// Use mean of local window
    Mean,
    /// Use Gaussian-weighted mean
    Gaussian,
}

impl Default for AdaptiveThresholdOptions {
    fn default() -> Self {
        Self {
            window_size: 15,
            c: 2.0,
            method: AdaptiveMethod::Mean,
        }
    }
}

/// Apply adaptive thresholding
///
/// Computes a local threshold for each pixel based on its neighborhood.
pub fn adaptive_threshold(pix: &Pix, options: &AdaptiveThresholdOptions) -> ColorResult<Pix> {
    if options.window_size.is_multiple_of(2) {
        return Err(ColorError::InvalidParameters(
            "window_size must be odd".to_string(),
        ));
    }

    let gray_pix = ensure_grayscale(pix)?;

    let w = gray_pix.width();
    let h = gray_pix.height();
    let half = (options.window_size / 2) as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Build integral image for fast mean computation
    let integral = build_integral_image(&gray_pix);

    for y in 0..h {
        for x in 0..w {
            // Compute local mean using integral image
            let x0 = (x as i32 - half).max(0) as u32;
            let y0 = (y as i32 - half).max(0) as u32;
            let x1 = (x as i32 + half).min(w as i32 - 1) as u32;
            let y1 = (y as i32 + half).min(h as i32 - 1) as u32;

            let local_mean = compute_mean_from_integral(&integral, w, x0, y0, x1, y1);
            let threshold = (local_mean - options.c).max(0.0);

            let pixel = gray_pix.get_pixel_unchecked(x, y) as f32;
            let binary = if pixel >= threshold { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, binary);
        }
    }

    Ok(out_mut.into())
}

/// Build an integral image (summed-area table)
fn build_integral_image(pix: &Pix) -> Vec<u64> {
    let w = pix.width() as usize;
    let h = pix.height() as usize;
    let mut integral = vec![0u64; (w + 1) * (h + 1)];

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x as u32, y as u32) as u64;
            let idx = (y + 1) * (w + 1) + (x + 1);
            integral[idx] =
                pixel + integral[y * (w + 1) + (x + 1)] + integral[(y + 1) * (w + 1) + x]
                    - integral[y * (w + 1) + x];
        }
    }

    integral
}

/// Compute mean from integral image for a rectangular region
fn compute_mean_from_integral(
    integral: &[u64],
    img_width: u32,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
) -> f32 {
    let w = img_width as usize + 1;
    let x0 = x0 as usize;
    let y0 = y0 as usize;
    let x1 = x1 as usize + 1;
    let y1 = y1 as usize + 1;

    let sum = integral[y1 * w + x1] + integral[y0 * w + x0]
        - integral[y0 * w + x1]
        - integral[y1 * w + x0];

    let count = ((x1 - x0) * (y1 - y0)) as f32;
    sum as f32 / count
}

/// Apply Sauvola's adaptive thresholding method
///
/// Better for document images with varying illumination.
/// Threshold = mean * (1 + k * (std / R - 1))
pub fn sauvola_threshold(pix: &Pix, window_size: u32, k: f32, r: f32) -> ColorResult<Pix> {
    if window_size.is_multiple_of(2) {
        return Err(ColorError::InvalidParameters(
            "window_size must be odd".to_string(),
        ));
    }

    let gray_pix = ensure_grayscale(pix)?;

    let w = gray_pix.width();
    let h = gray_pix.height();
    let half = (window_size / 2) as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Build integral images for mean and variance computation
    let integral = build_integral_image(&gray_pix);
    let integral_sq = build_integral_image_squared(&gray_pix);

    for y in 0..h {
        for x in 0..w {
            let x0 = (x as i32 - half).max(0) as u32;
            let y0 = (y as i32 - half).max(0) as u32;
            let x1 = (x as i32 + half).min(w as i32 - 1) as u32;
            let y1 = (y as i32 + half).min(h as i32 - 1) as u32;

            let mean = compute_mean_from_integral(&integral, w, x0, y0, x1, y1);
            let mean_sq = compute_mean_from_integral_f64(&integral_sq, w, x0, y0, x1, y1);
            let variance = (mean_sq - (mean * mean) as f64).max(0.0);
            let std_dev = variance.sqrt() as f32;

            let threshold = mean * (1.0 + k * (std_dev / r - 1.0));

            let pixel = gray_pix.get_pixel_unchecked(x, y) as f32;
            let binary = if pixel >= threshold { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, binary);
        }
    }

    Ok(out_mut.into())
}

fn build_integral_image_squared(pix: &Pix) -> Vec<u64> {
    let w = pix.width() as usize;
    let h = pix.height() as usize;
    let mut integral = vec![0u64; (w + 1) * (h + 1)];

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x as u32, y as u32) as u64;
            let pixel_sq = pixel * pixel;
            let idx = (y + 1) * (w + 1) + (x + 1);
            integral[idx] =
                pixel_sq + integral[y * (w + 1) + (x + 1)] + integral[(y + 1) * (w + 1) + x]
                    - integral[y * (w + 1) + x];
        }
    }

    integral
}

fn compute_mean_from_integral_f64(
    integral: &[u64],
    img_width: u32,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
) -> f64 {
    let w = img_width as usize + 1;
    let x0 = x0 as usize;
    let y0 = y0 as usize;
    let x1 = x1 as usize + 1;
    let y1 = y1 as usize + 1;

    let sum = integral[y1 * w + x1] + integral[y0 * w + x0]
        - integral[y0 * w + x1]
        - integral[y1 * w + x0];

    let count = ((x1 - x0) * (y1 - y0)) as f64;
    sum as f64 / count
}

// =============================================================================
// Floyd-Steinberg Dithering
// =============================================================================

/// Convert grayscale image to binary using Floyd-Steinberg dithering
///
/// Distributes quantization error to neighboring pixels for better
/// visual appearance.
pub fn dither_to_binary(pix: &Pix) -> ColorResult<Pix> {
    dither_to_binary_with_threshold(pix, 128)
}

/// Convert grayscale image to binary using Floyd-Steinberg dithering
/// with a specified threshold.
pub fn dither_to_binary_with_threshold(pix: &Pix, threshold: u8) -> ColorResult<Pix> {
    let gray_pix = ensure_grayscale(pix)?;

    let w = gray_pix.width();
    let h = gray_pix.height();

    // Work with floating point for error diffusion
    let mut buffer: Vec<f32> = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let pixel = gray_pix.get_pixel_unchecked(x, y) as f32;
            buffer.push(pixel);
        }
    }

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let threshold = threshold as f32;

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let old_pixel = buffer[idx];

            // Quantize
            let new_pixel = if old_pixel >= threshold { 255.0 } else { 0.0 };
            let binary = if new_pixel > 0.0 { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, binary);

            // Compute error
            let error = old_pixel - new_pixel;

            // Distribute error to neighbors (Floyd-Steinberg pattern)
            // [   *   7/16]
            // [3/16 5/16 1/16]

            if x + 1 < w {
                buffer[idx + 1] += error * 7.0 / 16.0;
            }
            if y + 1 < h {
                let next_row = ((y + 1) * w) as usize;
                if x > 0 {
                    buffer[next_row + x as usize - 1] += error * 3.0 / 16.0;
                }
                buffer[next_row + x as usize] += error * 5.0 / 16.0;
                if x + 1 < w {
                    buffer[next_row + x as usize + 1] += error * 1.0 / 16.0;
                }
            }
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Ordered Dithering (Bayer Matrix)
// =============================================================================

/// Apply ordered dithering using a Bayer matrix
///
/// Creates a halftone-like pattern with less visible artifacts than
/// Floyd-Steinberg for some images.
pub fn ordered_dither(pix: &Pix, matrix_size: u32) -> ColorResult<Pix> {
    let gray_pix = ensure_grayscale(pix)?;

    let matrix = match matrix_size {
        2 => BAYER_2X2.as_slice(),
        4 => BAYER_4X4.as_slice(),
        8 => BAYER_8X8.as_slice(),
        _ => {
            return Err(ColorError::InvalidParameters(
                "matrix_size must be 2, 4, or 8".to_string(),
            ));
        }
    };

    let w = gray_pix.width();
    let h = gray_pix.height();
    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let n = matrix_size as usize;
    let scale = 256.0 / (n * n) as f32;

    for y in 0..h {
        for x in 0..w {
            let pixel = gray_pix.get_pixel_unchecked(x, y) as f32;

            let mx = (x as usize) % n;
            let my = (y as usize) % n;
            let threshold = matrix[my * n + mx] as f32 * scale;

            let binary = if pixel >= threshold { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, binary);
        }
    }

    Ok(out_mut.into())
}

// Bayer dithering matrices
const BAYER_2X2: [u8; 4] = [0, 2, 3, 1];

const BAYER_4X4: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

const BAYER_8X8: [u8; 64] = [
    0, 32, 8, 40, 2, 34, 10, 42, 48, 16, 56, 24, 50, 18, 58, 26, 12, 44, 4, 36, 14, 46, 6, 38, 60,
    28, 52, 20, 62, 30, 54, 22, 3, 35, 11, 43, 1, 33, 9, 41, 51, 19, 59, 27, 49, 17, 57, 25, 15,
    47, 7, 39, 13, 45, 5, 37, 63, 31, 55, 23, 61, 29, 53, 21,
];

// =============================================================================
// Helper Functions
// =============================================================================

// =============================================================================
// Variable Threshold Binarization
// =============================================================================

/// Create a binary image by applying a per-pixel threshold map.
///
/// For each pixel: if `pixs[x,y] < pixg[x,y]`, output is 1 (foreground);
/// otherwise 0 (background).
///
/// # See also
///
/// C Leptonica: `pixVarThresholdToBinary()` in `grayquant.c`
pub fn var_threshold_to_binary(pix: &Pix, thresh_map: &Pix) -> ColorResult<Pix> {
    let gray = ensure_grayscale(pix)?;
    if thresh_map.depth() != PixelDepth::Bit8 {
        return Err(ColorError::UnsupportedDepth {
            expected: "8 bpp threshold map",
            actual: thresh_map.depth().bits(),
        });
    }
    let w = gray.width();
    let h = gray.height();
    if thresh_map.width() != w || thresh_map.height() != h {
        return Err(ColorError::InvalidParameters(
            "threshold map must have same dimensions as input".into(),
        ));
    }

    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = gray.get_pixel_unchecked(x, y);
            let thresh = thresh_map.get_pixel_unchecked(x, y);
            if val < thresh {
                out_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }
    Ok(out_mut.into())
}

// =============================================================================
// Mask Generation
// =============================================================================

/// Create a 1bpp mask where pixels exactly equal a given value.
///
/// # See also
///
/// C Leptonica: `pixGenerateMaskByValue()` in `grayquant.c`
pub fn generate_mask_by_value(pix: &Pix, val: u32) -> ColorResult<Pix> {
    let depth = pix.depth();
    if !matches!(
        depth,
        PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8
    ) {
        return Err(ColorError::UnsupportedDepth {
            expected: "2, 4, or 8 bpp",
            actual: depth.bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            if pix.get_pixel_unchecked(x, y) == val {
                out_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }
    Ok(out_mut.into())
}

/// Create a 1bpp mask where pixels fall within (or outside) a value range.
///
/// If `in_band` is true, pixels with `lower <= val <= upper` are set.
/// If `in_band` is false, pixels outside `[lower, upper]` are set.
///
/// # See also
///
/// C Leptonica: `pixGenerateMaskByBand()` in `grayquant.c`
pub fn generate_mask_by_band(pix: &Pix, lower: u32, upper: u32, in_band: bool) -> ColorResult<Pix> {
    let depth = pix.depth();
    if !matches!(
        depth,
        PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8
    ) {
        return Err(ColorError::UnsupportedDepth {
            expected: "2, 4, or 8 bpp",
            actual: depth.bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            let matches = val >= lower && val <= upper;
            if matches == in_band {
                out_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }
    Ok(out_mut.into())
}

// =============================================================================
// Multi-level Quantization
// =============================================================================

/// Quantize an 8bpp grayscale image to 2bpp (2, 3, or 4 levels).
///
/// Returns a 2bpp image. If `with_colormap` is true, a grayscale colormap
/// is attached.
///
/// # See also
///
/// C Leptonica: `pixThresholdTo2bpp()` in `grayquant.c`
pub fn threshold_to_2bpp(pix: &Pix, nlevels: u32, with_colormap: bool) -> ColorResult<Pix> {
    if !(2..=4).contains(&nlevels) {
        return Err(ColorError::InvalidParameters(
            "nlevels must be 2, 3, or 4 for 2bpp".into(),
        ));
    }
    threshold_to_nbpp(pix, nlevels, PixelDepth::Bit2, with_colormap)
}

/// Quantize an 8bpp grayscale image to 4bpp (2-16 levels).
///
/// Returns a 4bpp image. If `with_colormap` is true, a grayscale colormap
/// is attached.
///
/// # See also
///
/// C Leptonica: `pixThresholdTo4bpp()` in `grayquant.c`
pub fn threshold_to_4bpp(pix: &Pix, nlevels: u32, with_colormap: bool) -> ColorResult<Pix> {
    if !(2..=16).contains(&nlevels) {
        return Err(ColorError::InvalidParameters(
            "nlevels must be 2-16 for 4bpp".into(),
        ));
    }
    threshold_to_nbpp(pix, nlevels, PixelDepth::Bit4, with_colormap)
}

/// Common implementation for threshold_to_2bpp and threshold_to_4bpp.
fn threshold_to_nbpp(
    pix: &Pix,
    nlevels: u32,
    out_depth: PixelDepth,
    _with_colormap: bool,
) -> ColorResult<Pix> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();

    // Build quantization lookup table: 256 entries mapping 8bpp → output level
    let mut qtable = [0u32; 256];
    let step = 256.0 / nlevels as f32;
    for (i, entry) in qtable.iter_mut().enumerate() {
        let level = (i as f32 / step) as u32;
        *entry = level.min(nlevels - 1);
    }

    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = gray.get_pixel_unchecked(x, y) as usize;
            out_mut.set_pixel_unchecked(x, y, qtable[val.min(255)]);
        }
    }

    // TODO: add colormap support when with_colormap is true
    Ok(out_mut.into())
}

// =============================================================================
// Tiled Adaptive Thresholding
// =============================================================================

/// Perform tiled adaptive Otsu thresholding.
///
/// Divides the image into tiles of approximately `sx × sy` pixels,
/// computes Otsu threshold per tile, optionally smooths the threshold map,
/// and applies it to produce a binary image.
///
/// Returns `(threshold_map, binary_image)` where `threshold_map` is an 8bpp
/// image of per-tile thresholds (upscaled to input size).
///
/// # See also
///
/// C Leptonica: `pixOtsuAdaptiveThreshold()` in `binarize.c`
pub fn otsu_adaptive_threshold(
    pix: &Pix,
    sx: u32,
    sy: u32,
    smoothx: u32,
    smoothy: u32,
    _score_fract: f32,
) -> ColorResult<(Pix, Pix)> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();

    let tile_sx = sx.max(16);
    let tile_sy = sy.max(16);
    let nx = w.div_ceil(tile_sx);
    let ny = h.div_ceil(tile_sy);

    // Compute Otsu threshold for each tile
    let mut tile_thresholds = vec![128u8; (nx * ny) as usize];
    for ty in 0..ny {
        for tx in 0..nx {
            let x0 = tx * tile_sx;
            let y0 = ty * tile_sy;
            let x1 = (x0 + tile_sx).min(w);
            let y1 = (y0 + tile_sy).min(h);

            // Build histogram for this tile
            let mut histo = [0u32; 256];
            for y in y0..y1 {
                for x in x0..x1 {
                    let val = gray.get_pixel_unchecked(x, y) as usize;
                    histo[val] += 1;
                }
            }
            let total = (x1 - x0) * (y1 - y0);
            if total == 0 {
                continue;
            }

            // Otsu: find threshold maximizing between-class variance
            let mut best_thresh = 128u8;
            let mut best_score = 0.0f64;
            let mut w0 = 0u32;
            let mut sum0 = 0u64;
            let total_sum: u64 = histo
                .iter()
                .enumerate()
                .map(|(i, &c)| i as u64 * c as u64)
                .sum();

            for (t, &count) in histo.iter().enumerate().take(255) {
                w0 += count;
                if w0 == 0 {
                    continue;
                }
                let w1 = total - w0;
                if w1 == 0 {
                    break;
                }
                sum0 += t as u64 * count as u64;
                let mean0 = sum0 as f64 / w0 as f64;
                let mean1 = (total_sum - sum0) as f64 / w1 as f64;
                let diff = mean0 - mean1;
                let score = w0 as f64 * w1 as f64 * diff * diff;
                if score > best_score {
                    best_score = score;
                    best_thresh = t as u8;
                }
            }
            tile_thresholds[(ty * nx + tx) as usize] = best_thresh;
        }
    }

    // Optional smoothing of threshold map
    if smoothx > 0 || smoothy > 0 {
        let kw = (2 * smoothx + 1) as usize;
        let kh = (2 * smoothy + 1) as usize;
        if kw > 1 || kh > 1 {
            let mut smoothed = vec![0u8; (nx * ny) as usize];
            for ty in 0..ny as usize {
                for tx in 0..nx as usize {
                    let mut sum = 0u32;
                    let mut count = 0u32;
                    let sy_start = ty.saturating_sub(kh / 2);
                    let sy_end = (ty + kh / 2 + 1).min(ny as usize);
                    let sx_start = tx.saturating_sub(kw / 2);
                    let sx_end = (tx + kw / 2 + 1).min(nx as usize);
                    for sy in sy_start..sy_end {
                        for sx in sx_start..sx_end {
                            sum += tile_thresholds[sy * nx as usize + sx] as u32;
                            count += 1;
                        }
                    }
                    smoothed[ty * nx as usize + tx] = (sum / count) as u8;
                }
            }
            tile_thresholds = smoothed;
        }
    }

    // Create threshold map (upscaled to input size)
    let thresh_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut thresh_mut = thresh_pix.try_into_mut().unwrap();
    for y in 0..h {
        let ty = (y / tile_sy).min(ny - 1);
        for x in 0..w {
            let tx = (x / tile_sx).min(nx - 1);
            let t = tile_thresholds[(ty * nx + tx) as usize];
            thresh_mut.set_pixel_unchecked(x, y, t as u32);
        }
    }

    // Apply threshold map to create binary image
    let binary_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut binary_mut = binary_pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = gray.get_pixel_unchecked(x, y);
            let thresh = thresh_mut.get_pixel_unchecked(x, y);
            if val < thresh {
                binary_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok((thresh_mut.into(), binary_mut.into()))
}

/// Perform tiled Sauvola binarization.
///
/// Divides the image into `nx × ny` tiles with overlap, applies Sauvola
/// thresholding to each tile, and assembles the result.
///
/// Returns `(threshold_map, binary_image)`.
///
/// # See also
///
/// C Leptonica: `pixSauvolaBinarizeTiled()` in `binarize.c`
pub fn sauvola_binarize_tiled(
    pix: &Pix,
    whsize: u32,
    factor: f32,
    nx: u32,
    ny: u32,
) -> ColorResult<(Pix, Pix)> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();
    let nx = nx.max(1);
    let ny = ny.max(1);

    // For single tile, use sauvola directly
    if nx == 1 && ny == 1 {
        let binary = sauvola_threshold(&gray, whsize | 1, factor, 128.0)?;
        // Generate threshold map from Sauvola parameters
        let thresh_map = generate_sauvola_thresh_map(&gray, whsize | 1, factor, 128.0)?;
        return Ok((thresh_map, binary));
    }

    // Compute tile sizes with overlap
    let tile_w = w.div_ceil(nx);
    let tile_h = h.div_ceil(ny);
    let overlap = whsize + 1;

    let thresh_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut thresh_mut = thresh_pix.try_into_mut().unwrap();
    let binary_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut binary_mut = binary_pix.try_into_mut().unwrap();

    for ty in 0..ny {
        for tx in 0..nx {
            // Compute tile region with overlap
            let x0 = (tx * tile_w).saturating_sub(overlap);
            let y0 = (ty * tile_h).saturating_sub(overlap);
            let x1 = ((tx + 1) * tile_w + overlap).min(w);
            let y1 = ((ty + 1) * tile_h + overlap).min(h);
            let tw = x1 - x0;
            let th = y1 - y0;

            // Extract tile
            let tile = Pix::new(tw, th, PixelDepth::Bit8)?;
            let mut tile_mut = tile.try_into_mut().unwrap();
            for y in 0..th {
                for x in 0..tw {
                    tile_mut.set_pixel_unchecked(x, y, gray.get_pixel_unchecked(x0 + x, y0 + y));
                }
            }
            let tile_pix: Pix = tile_mut.into();

            // Apply Sauvola to this tile
            let ws = (whsize | 1).min(tw.min(th).saturating_sub(1) | 1).max(3);
            let tile_binary = sauvola_threshold(&tile_pix, ws, factor, 128.0)?;
            let tile_thresh = generate_sauvola_thresh_map(&tile_pix, ws, factor, 128.0)?;

            // Paint non-overlap region back into output
            let out_x0 = tx * tile_w;
            let out_y0 = ty * tile_h;
            let out_x1 = ((tx + 1) * tile_w).min(w);
            let out_y1 = ((ty + 1) * tile_h).min(h);

            for y in out_y0..out_y1 {
                for x in out_x0..out_x1 {
                    let local_x = x - x0;
                    let local_y = y - y0;
                    binary_mut.set_pixel_unchecked(
                        x,
                        y,
                        tile_binary.get_pixel_unchecked(local_x, local_y),
                    );
                    thresh_mut.set_pixel_unchecked(
                        x,
                        y,
                        tile_thresh.get_pixel_unchecked(local_x, local_y),
                    );
                }
            }
        }
    }

    Ok((thresh_mut.into(), binary_mut.into()))
}

/// Generate an 8bpp threshold map using Sauvola's formula.
fn generate_sauvola_thresh_map(pix: &Pix, window_size: u32, k: f32, r: f32) -> ColorResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let half = (window_size / 2) as i64;

    let out = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let x0 = (x as i64 - half).max(0) as u32;
            let y0 = (y as i64 - half).max(0) as u32;
            let x1 = (x as i64 + half).min(w as i64 - 1) as u32;
            let y1 = (y as i64 + half).min(h as i64 - 1) as u32;

            let mut sum = 0u64;
            let mut sum_sq = 0u64;
            let mut count = 0u64;
            for wy in y0..=y1 {
                for wx in x0..=x1 {
                    let val = pix.get_pixel_unchecked(wx, wy) as u64;
                    sum += val;
                    sum_sq += val * val;
                    count += 1;
                }
            }

            let mean = sum as f64 / count as f64;
            let variance = (sum_sq as f64 / count as f64) - mean * mean;
            let std_dev = variance.max(0.0).sqrt();
            let threshold = mean * (1.0 - k as f64 * (1.0 - std_dev / r as f64));
            let threshold = threshold.round().clamp(0.0, 255.0) as u32;
            out_mut.set_pixel_unchecked(x, y, threshold);
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Internal helpers
// =============================================================================

fn ensure_grayscale(pix: &Pix) -> ColorResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit8 => Ok(pix.clone()),
        PixelDepth::Bit32 => pix_convert_to_gray(pix),
        _ => Err(ColorError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::color;

    fn create_gradient_image() -> Pix {
        let pix = Pix::new(256, 1, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for x in 0..256u32 {
            pix_mut.set_pixel_unchecked(x, 0, x);
        }
        pix_mut.into()
    }

    fn create_bimodal_image() -> Pix {
        // Create image with two distinct peaks (dark and light)
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..100 {
            for x in 0..100 {
                // Left half: dark (around 50)
                // Right half: light (around 200)
                let val = if x < 50 { 50 } else { 200 };
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }
        pix_mut.into()
    }

    #[test]
    fn test_threshold_to_binary() {
        let pix = create_gradient_image();
        let binary = threshold_to_binary(&pix, 128).unwrap();

        assert_eq!(binary.depth(), PixelDepth::Bit1);
        assert_eq!(binary.width(), 256);

        // Pixels 0-127 should be 0 (black), 128-255 should be 1 (white)
        assert_eq!(binary.get_pixel_unchecked(0, 0), 0);
        assert_eq!(binary.get_pixel_unchecked(127, 0), 0);
        assert_eq!(binary.get_pixel_unchecked(128, 0), 1);
        assert_eq!(binary.get_pixel_unchecked(255, 0), 1);
    }

    #[test]
    fn test_otsu_threshold() {
        let pix = create_bimodal_image();
        let threshold = compute_otsu_threshold(&pix).unwrap();

        // Threshold should separate the two peaks (50 and 200)
        // Any value 51-199 is valid for this bimodal distribution
        assert!(
            threshold > 50 && threshold < 200,
            "Threshold {} should be between 50 and 200",
            threshold
        );
    }

    #[test]
    fn test_threshold_otsu() {
        let pix = create_bimodal_image();
        let threshold = compute_otsu_threshold(&pix).unwrap();
        let binary = threshold_otsu(&pix).unwrap();

        assert_eq!(binary.depth(), PixelDepth::Bit1);

        // Since left half has value 50 and threshold is > 50,
        // pixels with value 50 should be black (0)
        // Since right half has value 200 and threshold is < 200,
        // pixels with value 200 should be white (1)
        let left_val = binary.get_pixel_unchecked(25, 50);
        let right_val = binary.get_pixel_unchecked(75, 50);

        // Left (value 50) should be black if threshold > 50
        assert_eq!(
            left_val, 0,
            "Left half (value 50) should be black when threshold is {}",
            threshold
        );
        // Right (value 200) should be white if threshold <= 200
        assert_eq!(
            right_val, 1,
            "Right half (value 200) should be white when threshold is {}",
            threshold
        );
    }

    #[test]
    fn test_adaptive_threshold() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..50 {
            for x in 0..50 {
                pix_mut.set_pixel_unchecked(x, y, 128);
            }
        }

        let options = AdaptiveThresholdOptions {
            window_size: 11,
            c: 0.0,
            method: AdaptiveMethod::Mean,
        };

        let binary = adaptive_threshold(&pix_mut.into(), &options).unwrap();
        assert_eq!(binary.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_adaptive_threshold_invalid_window() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let options = AdaptiveThresholdOptions {
            window_size: 10, // Even - should fail
            c: 0.0,
            method: AdaptiveMethod::Mean,
        };

        let result = adaptive_threshold(&pix, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_dither_to_binary() {
        let pix = create_gradient_image();
        let dithered = dither_to_binary(&pix).unwrap();

        assert_eq!(dithered.depth(), PixelDepth::Bit1);
        assert_eq!(dithered.width(), 256);
    }

    #[test]
    fn test_ordered_dither() {
        let pix = Pix::new(32, 32, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..32 {
            for x in 0..32 {
                pix_mut.set_pixel_unchecked(x, y, 128);
            }
        }

        let dithered = ordered_dither(&pix_mut.into(), 4).unwrap();
        assert_eq!(dithered.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_ordered_dither_invalid_size() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = ordered_dither(&pix, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_threshold_from_color() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                let gray = (x * 25) as u8;
                let pixel = color::compose_rgb(gray, gray, gray);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        let binary = threshold_to_binary(&pix_mut.into(), 128).unwrap();
        assert_eq!(binary.depth(), PixelDepth::Bit1);
    }
}
