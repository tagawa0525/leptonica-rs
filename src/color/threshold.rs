//! Binary thresholding and dithering
//!
//! Provides various methods for converting grayscale images to binary:
//! - Fixed threshold binarization
//! - Otsu's method (automatic threshold selection)
//! - Adaptive (local) thresholding
//! - Floyd-Steinberg dithering

use crate::color::colorspace::pix_convert_to_gray;
use crate::color::{ColorError, ColorResult};
use crate::core::pixel;
use crate::core::{Pix, PixColormap, PixelDepth};
use crate::filter::adaptmap::{
    BackgroundNormOptions, ContrastNormOptions, background_norm, contrast_norm,
};
use crate::region::conncomp::{ConnectivityType, count_conn_comp};

// =============================================================================
// Fixed Threshold Binarization
// =============================================================================

/// Convert a grayscale image to binary using a fixed threshold
///
/// Pixels < threshold become foreground (1), pixels >= threshold become
/// background (0).  This matches the C leptonica `pixThresholdToBinary()`
/// convention where dark pixels are foreground.
pub fn threshold_to_binary(pix: &Pix, threshold: u8) -> ColorResult<Pix> {
    let gray_pix = ensure_grayscale(pix)?;

    let w = gray_pix.width();
    let h = gray_pix.height();
    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = gray_pix.get_pixel_unchecked(x, y) as u8;
            let binary = if pixel < threshold { 1 } else { 0 };
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
            let binary = if pixel < threshold { 1 } else { 0 };
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
            let binary = if pixel < threshold { 1 } else { 0 };
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

            // Quantize: dark pixels (< threshold) become foreground (1),
            // bright pixels (>= threshold) become background (0).
            let is_fg = old_pixel < threshold;
            let new_pixel = if is_fg { 0.0 } else { 255.0 };
            let binary = if is_fg { 1 } else { 0 };
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

            let binary = if pixel < threshold { 1 } else { 0 };
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
/// otherwise 0 (background). This follows document binarization convention
/// where dark pixels (below threshold) become foreground.
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
/// Returns a 2bpp image. When `with_colormap` is true, an `nlevels`-entry
/// grayscale colormap spanning `[0, 255]` linearly is attached to the output.
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
/// Returns a 4bpp image. When `with_colormap` is true, an `nlevels`-entry
/// grayscale colormap spanning `[0, 255]` linearly is attached to the output.
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
    with_colormap: bool,
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

    if with_colormap {
        let mut cmap = PixColormap::new(out_depth.bits())?;
        for level in 0..nlevels {
            let gray_val = if nlevels == 1 {
                0
            } else {
                (level * 255 / (nlevels - 1)) as u8
            };
            cmap.add_rgb(gray_val, gray_val, gray_val)?;
        }
        out_mut.set_colormap(Some(cmap))?;
    }

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
/// Uses document binarization convention: dark pixels (val < threshold)
/// become foreground (1), bright pixels become background (0).
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

/// Generate an 8bpp threshold map using Sauvola's formula with integral images.
///
/// Uses integral images for O(1) per-pixel mean/variance computation,
/// consistent with `sauvola_threshold`.
fn generate_sauvola_thresh_map(pix: &Pix, window_size: u32, k: f32, r: f32) -> ColorResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let half = (window_size / 2) as i64;

    // Build integral images for sum and sum-of-squares
    let ww = w as usize;
    let hh = h as usize;
    let mut int_sum = vec![0i64; (ww + 1) * (hh + 1)];
    let mut int_sq = vec![0i64; (ww + 1) * (hh + 1)];
    let stride = ww + 1;

    for y in 0..hh {
        for x in 0..ww {
            let val = pix.get_pixel_unchecked(x as u32, y as u32) as i64;
            int_sum[(y + 1) * stride + (x + 1)] =
                val + int_sum[y * stride + (x + 1)] + int_sum[(y + 1) * stride + x]
                    - int_sum[y * stride + x];
            int_sq[(y + 1) * stride + (x + 1)] =
                val * val + int_sq[y * stride + (x + 1)] + int_sq[(y + 1) * stride + x]
                    - int_sq[y * stride + x];
        }
    }

    let out = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let x0 = (x as i64 - half).max(0) as usize;
            let y0 = (y as i64 - half).max(0) as usize;
            let x1 = ((x as i64 + half).min(w as i64 - 1) + 1) as usize;
            let y1 = ((y as i64 + half).min(h as i64 - 1) + 1) as usize;

            let sum =
                int_sum[y1 * stride + x1] - int_sum[y0 * stride + x1] - int_sum[y1 * stride + x0]
                    + int_sum[y0 * stride + x0];
            let sq = int_sq[y1 * stride + x1] - int_sq[y0 * stride + x1] - int_sq[y1 * stride + x0]
                + int_sq[y0 * stride + x0];
            let count = ((x1 - x0) * (y1 - y0)) as f64;

            let mean = sum as f64 / count;
            let variance = (sq as f64 / count) - mean * mean;
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
    // Colormapped 8 bpp pixels carry palette indices, not gray values, so
    // downstream tile-stat operations (fill_map_holes, background_norm,
    // ...) would produce nonsense if we passed them through as-is. Decode
    // the colormap to actual gray values first. This matches C leptonica
    // pipelines that call pixGetColormap() / pixRemoveColormap() before
    // adaptive map computations.
    if pix.colormap().is_some() {
        return pix
            .remove_colormap(crate::core::pix::RemoveColormapTarget::ToGrayscale)
            .map_err(ColorError::from);
    }
    match pix.depth() {
        PixelDepth::Bit8 => Ok(pix.clone()),
        PixelDepth::Bit32 => pix_convert_to_gray(pix),
        _ => Err(ColorError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

// =============================================================================
// Composite Binarization (binarize.c)
// =============================================================================

/// Otsu threshold after background normalization.
///
/// Normalizes the background of the input image, then applies Otsu adaptive
/// thresholding on the result to produce a binary image.
///
/// # See also
///
/// C Leptonica: `pixOtsuThreshOnBackgroundNorm()` in `binarize.c`
#[allow(clippy::too_many_arguments)]
pub fn otsu_thresh_on_background_norm(
    pix: &Pix,
    _mask: Option<&Pix>,
    sx: u32,
    sy: u32,
    thresh: u32,
    min_count: u32,
    bg_val: u32,
    smooth_x: u32,
    smooth_y: u32,
    score_fract: f32,
) -> ColorResult<(Pix, u32)> {
    let opts = BackgroundNormOptions {
        tile_width: sx.max(4),
        tile_height: sy.max(4),
        fg_threshold: thresh,
        min_count,
        bg_val: bg_val.clamp(128, 255),
        smooth_x,
        smooth_y,
    };
    let normed = background_norm(pix, &opts)
        .map_err(|e| ColorError::InvalidParameters(format!("background_norm failed: {e}")))?;

    let w = normed.width();
    let h = normed.height();
    let (_thresh_map, binary) = otsu_adaptive_threshold(&normed, w, h, 0, 0, score_fract)?;

    // Extract the global Otsu threshold from the normalized image
    let gray = ensure_grayscale(&normed)?;
    let tval = compute_otsu_threshold(&gray)? as u32;

    Ok((binary, tval))
}

/// Masked threshold after background normalization.
///
/// Normalizes the background and then applies Otsu thresholding.
/// Similar to [`otsu_thresh_on_background_norm`] with masking support.
///
/// # See also
///
/// C Leptonica: `pixMaskedThreshOnBackgroundNorm()` in `binarize.c`
#[allow(clippy::too_many_arguments)]
pub fn masked_thresh_on_background_norm(
    pix: &Pix,
    _mask: Option<&Pix>,
    sx: u32,
    sy: u32,
    thresh: u32,
    min_count: u32,
    smooth_x: u32,
    smooth_y: u32,
    score_fract: f32,
) -> ColorResult<(Pix, u32)> {
    let opts = BackgroundNormOptions {
        tile_width: sx.max(4),
        tile_height: sy.max(4),
        fg_threshold: thresh,
        min_count,
        bg_val: 200,
        smooth_x,
        smooth_y,
    };
    let normed = background_norm(pix, &opts)
        .map_err(|e| ColorError::InvalidParameters(format!("background_norm failed: {e}")))?;

    let w = normed.width();
    let h = normed.height();
    let (_thresh_map, binary) = otsu_adaptive_threshold(&normed, w, h, 0, 0, score_fract)?;

    let gray = ensure_grayscale(&normed)?;
    let tval = compute_otsu_threshold(&gray)? as u32;

    Ok((binary, tval))
}

/// Sauvola threshold after contrast normalization.
///
/// Applies contrast normalization to stretch local contrast, then uses
/// Sauvola binarization on the result.
///
/// # See also
///
/// C Leptonica: `pixSauvolaOnContrastNorm()` in `binarize.c`
pub fn sauvola_on_contrast_norm(pix: &Pix, mindiff: u32) -> ColorResult<Pix> {
    let gray = ensure_grayscale(pix)?;
    let opts = ContrastNormOptions {
        min_diff: mindiff,
        ..ContrastNormOptions::default()
    };
    let normed = contrast_norm(&gray, &opts)
        .map_err(|e| ColorError::InvalidParameters(format!("contrast_norm failed: {e}")))?;

    sauvola_threshold(&normed, 7, 0.34, 128.0)
}

/// Threshold after double normalization (background + contrast).
///
/// Applies background normalization followed by contrast normalization,
/// then binarizes the result at a fixed threshold.
///
/// # See also
///
/// C Leptonica: `pixThreshOnDoubleNorm()` in `binarize.c`
pub fn thresh_on_double_norm(pix: &Pix, mindiff: u32) -> ColorResult<Pix> {
    // Decode any colormap and reduce to 8 bpp gray BEFORE the adaptive map
    // pipeline — background_norm operates on tile statistics of raw pixel
    // values, which are meaningless for palette indices.
    let gray = ensure_grayscale(pix)?;

    let bg_opts = BackgroundNormOptions::default();
    let normed_bg = background_norm(&gray, &bg_opts)
        .map_err(|e| ColorError::InvalidParameters(format!("background_norm failed: {e}")))?;

    let cn_opts = ContrastNormOptions {
        min_diff: mindiff,
        ..ContrastNormOptions::default()
    };
    let normed = contrast_norm(&normed_bg, &cn_opts)
        .map_err(|e| ColorError::InvalidParameters(format!("contrast_norm failed: {e}")))?;

    threshold_to_binary(&normed, 200)
}

/// Find optimal threshold by minimizing connected component count.
///
/// Sweeps thresholds from `start` to `end` by `incr`, binarizing at each
/// level and counting 4-connected and 8-connected components. The optimal
/// threshold minimizes the ratio of 4cc / 8cc counts (or finds where it
/// first drops below `thresh48`).
///
/// # See also
///
/// C Leptonica: `pixThresholdByConnComp()` in `binarize.c`
pub fn threshold_by_conn_comp(
    pix: &Pix,
    _mask: Option<&Pix>,
    start: u32,
    end: u32,
    incr: u32,
    thresh48: f32,
    extra_thresh: Option<&mut u32>,
) -> ColorResult<(Pix, u32)> {
    let gray = ensure_grayscale(pix)?;

    let start = start.max(1);
    let end = end.min(254);
    let incr = incr.max(1);

    let mut best_thresh = start;
    let mut best_ratio = f32::MAX;
    let mut best_binary: Option<Pix> = None;

    let mut t = start;
    while t <= end {
        let binary = threshold_to_binary(&gray, t as u8)?;
        let cc4 = count_conn_comp(&binary, ConnectivityType::FourWay)
            .map_err(|e| ColorError::InvalidParameters(format!("count_conn_comp failed: {e}")))?;
        let cc8 = count_conn_comp(&binary, ConnectivityType::EightWay)
            .map_err(|e| ColorError::InvalidParameters(format!("count_conn_comp failed: {e}")))?;

        let ratio = if cc8 > 0 {
            cc4 as f32 / cc8 as f32
        } else {
            f32::MAX
        };

        if ratio < best_ratio {
            best_ratio = ratio;
            best_thresh = t;
            best_binary = Some(binary);
        }

        if ratio < thresh48 {
            best_thresh = t;
            best_binary = Some(threshold_to_binary(&gray, t as u8)?);
            break;
        }

        t += incr;
    }

    if let Some(et) = extra_thresh {
        *et = best_thresh;
    }

    let binary = match best_binary {
        Some(b) => b,
        None => threshold_to_binary(&gray, best_thresh as u8)?,
    };

    Ok((binary, best_thresh))
}

/// Find threshold using histogram analysis.
///
/// Builds a histogram of the grayscale image, smooths it, and looks for a
/// valley between the two main peaks. The valley location is used as the
/// binarization threshold.
///
/// # See also
///
/// C Leptonica: `pixThresholdByHisto()` in `binarize.c`
pub fn threshold_by_histo(pix: &Pix, factor: u32, _nthresh: u32) -> ColorResult<(Pix, u32)> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();
    let factor = factor.max(1);

    // Build histogram with subsampling
    let mut histo = [0u32; 256];
    for y in (0..h).step_by(factor as usize) {
        for x in (0..w).step_by(factor as usize) {
            let val = gray.get_pixel_unchecked(x, y) as usize;
            histo[val] += 1;
        }
    }

    // Smooth histogram (simple moving average, width=5)
    let mut smoothed = [0f64; 256];
    for (i, slot) in smoothed.iter_mut().enumerate() {
        let lo = i.saturating_sub(2);
        let hi = (i + 2).min(255);
        let mut sum = 0u64;
        let mut cnt = 0u32;
        for &h in &histo[lo..=hi] {
            sum += h as u64;
            cnt += 1;
        }
        *slot = sum as f64 / cnt as f64;
    }

    // Find two main peaks
    let mut peak1_idx = 0usize;
    let mut peak1_val = 0.0f64;
    for (i, &v) in smoothed.iter().enumerate() {
        if v > peak1_val {
            peak1_val = v;
            peak1_idx = i;
        }
    }

    // Find second peak (at least 30 bins away from first)
    let mut peak2_idx = 0usize;
    let mut peak2_val = 0.0f64;
    for (i, &v) in smoothed.iter().enumerate() {
        if (i as isize - peak1_idx as isize).unsigned_abs() >= 30 && v > peak2_val {
            peak2_val = v;
            peak2_idx = i;
        }
    }

    // Find valley between the two peaks
    let (lo, hi) = if peak1_idx < peak2_idx {
        (peak1_idx, peak2_idx)
    } else {
        (peak2_idx, peak1_idx)
    };

    let mut valley_idx = (lo + hi) / 2;
    let mut valley_val = f64::MAX;
    for (i, &v) in smoothed.iter().enumerate().take(hi + 1).skip(lo) {
        if v < valley_val {
            valley_val = v;
            valley_idx = i;
        }
    }

    let thresh = valley_idx as u32;
    let binary = threshold_to_binary(&gray, thresh as u8)?;

    Ok((binary, thresh))
}

// =============================================================================
// Generalized Adaptive Thresholding (grayquant.c)
// =============================================================================

/// Generalized adaptive threshold to binary.
///
/// Applies gamma correction and maps the `[black_val, white_val]` range to
/// `[0, 255]`, then uses Otsu thresholding.
///
/// # See also
///
/// C Leptonica: `pixAdaptThresholdToBinaryGen()` in `grayquant.c`
pub fn adapt_threshold_to_binary_gen(
    pix: &Pix,
    _mask: Option<&Pix>,
    gamma: f32,
    black_val: i32,
    white_val: i32,
) -> ColorResult<Pix> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();

    // Build gamma + range mapping LUT
    let gamma = if gamma <= 0.0 { 1.0f32 } else { gamma };
    let range = (white_val - black_val).max(1) as f32;

    let mut lut = [0u8; 256];
    for (i, entry) in lut.iter_mut().enumerate() {
        let clamped = (i as i32).clamp(black_val, white_val);
        let normalized = (clamped - black_val) as f32 / range;
        let gamma_corrected = normalized.powf(1.0 / gamma);
        *entry = (gamma_corrected * 255.0).clamp(0.0, 255.0) as u8;
    }

    // Apply LUT
    let mapped = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut mapped_mut = mapped.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = gray.get_pixel_unchecked(x, y) as usize;
            mapped_mut.set_pixel_unchecked(x, y, lut[val.min(255)] as u32);
        }
    }

    let mapped_pix: Pix = mapped_mut.into();
    threshold_otsu(&mapped_pix)
}

/// Floyd-Steinberg dither to 2bpp (4 levels).
///
/// Uses error-diffusion dithering with output levels 0, 85, 170, 255
/// mapped to 2bpp values 0–3.
///
/// # See also
///
/// C Leptonica: `pixDitherTo2bpp()` in `grayquant.c`
pub fn dither_to_2bpp(pix: &Pix) -> ColorResult<Pix> {
    dither_to_2bpp_spec(pix, 64, 128, 192)
}

/// Dither to 2bpp with specified thresholds.
///
/// The three thresholds divide the 0–255 range into four output levels.
///
/// # See also
///
/// C Leptonica: `pixDitherTo2bppSpec()` in `grayquant.c`
pub fn dither_to_2bpp_spec(
    pix: &Pix,
    thresh1: u32,
    thresh2: u32,
    thresh3: u32,
) -> ColorResult<Pix> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();

    // Quantization levels
    let levels: [f32; 4] = [0.0, 85.0, 170.0, 255.0];
    let thresholds = [thresh1 as f32, thresh2 as f32, thresh3 as f32];

    // Work buffer for error diffusion
    let mut buffer: Vec<f32> = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            buffer.push(gray.get_pixel_unchecked(x, y) as f32);
        }
    }

    let out = Pix::new(w, h, PixelDepth::Bit2)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let old_pixel = buffer[idx];

            // Quantize to one of 4 levels
            let level_idx = if old_pixel < thresholds[0] {
                0
            } else if old_pixel < thresholds[1] {
                1
            } else if old_pixel < thresholds[2] {
                2
            } else {
                3
            };

            let new_pixel = levels[level_idx];
            out_mut.set_pixel_unchecked(x, y, level_idx as u32);

            // Floyd-Steinberg error diffusion
            let error = old_pixel - new_pixel;
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

/// Multi-level threshold on 8bpp image.
///
/// Quantizes an 8bpp grayscale image to `nlevels` evenly-spaced gray values.
/// If `with_colormap` is true, attaches a grayscale colormap.
///
/// # See also
///
/// C Leptonica: `pixThresholdOn8bpp()` in `grayquant.c`
pub fn threshold_on_8bpp(pix: &Pix, nlevels: u32, with_colormap: bool) -> ColorResult<Pix> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();

    let nlevels = nlevels.clamp(2, 256);
    let step = 256.0 / nlevels as f32;

    // Build LUT: map each 0–255 value to the nearest quantized level
    let mut lut = [0u8; 256];
    for (i, entry) in lut.iter_mut().enumerate() {
        let level_idx = ((i as f32 / step) as u32).min(nlevels - 1);
        let center = ((level_idx as f32 + 0.5) * step).clamp(0.0, 255.0);
        *entry = center as u8;
    }

    let out = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = gray.get_pixel_unchecked(x, y) as usize;
            out_mut.set_pixel_unchecked(x, y, lut[val.min(255)] as u32);
        }
    }

    if with_colormap {
        let mut cmap = PixColormap::new(8)
            .map_err(|e| ColorError::InvalidParameters(format!("colormap creation failed: {e}")))?;
        for i in 0..nlevels {
            let center = (((i as f32 + 0.5) * step).clamp(0.0, 255.0)) as u8;
            let _ = cmap.add_rgb(center, center, center);
        }
        out_mut
            .set_colormap(Some(cmap))
            .map_err(|e| ColorError::InvalidParameters(format!("set_colormap failed: {e}")))?;
    }

    Ok(out_mut.into())
}

/// Threshold with arbitrary levels specified as a space-separated string.
///
/// The string contains threshold values that divide the 0–255 range into
/// N+1 output levels (where N is the number of thresholds).
///
/// # See also
///
/// C Leptonica: `pixThresholdGrayArb()` in `grayquant.c`
pub fn threshold_gray_arb(pix: &Pix, edgevals: &str) -> ColorResult<Pix> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();

    // Parse threshold values from the string
    let thresholds: Vec<u32> = edgevals
        .split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .collect();

    if thresholds.is_empty() {
        return Err(ColorError::InvalidParameters(
            "edgevals must contain at least one threshold value".into(),
        ));
    }

    let nlevels = thresholds.len() as u32 + 1;

    // Determine output depth
    let out_depth = if nlevels <= 2 {
        PixelDepth::Bit1
    } else if nlevels <= 4 {
        PixelDepth::Bit2
    } else if nlevels <= 16 {
        PixelDepth::Bit4
    } else {
        PixelDepth::Bit8
    };

    // Build LUT
    let mut lut = [0u32; 256];
    for (i, entry) in lut.iter_mut().enumerate() {
        let mut level = 0u32;
        for &t in &thresholds {
            if (i as u32) >= t {
                level += 1;
            }
        }
        *entry = level;
    }

    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = gray.get_pixel_unchecked(x, y) as usize;
            out_mut.set_pixel_unchecked(x, y, lut[val.min(255)]);
        }
    }

    Ok(out_mut.into())
}

/// Generate mask by color band in 32bpp image.
///
/// Pixels within distance `maxdist` of `refval` (in each R, G, B component)
/// are set to 1 in the output 1bpp mask.
///
/// # See also
///
/// C Leptonica: `pixGenerateMaskByBand32()` in `grayquant.c`
pub fn generate_mask_by_band_32(pix: &Pix, refval: u32, maxdist: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let (ref_r, ref_g, ref_b) = pixel::extract_rgb(refval);

    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(val);
            let dr = (r as i32 - ref_r as i32).unsigned_abs();
            let dg = (g as i32 - ref_g as i32).unsigned_abs();
            let db = (b as i32 - ref_b as i32).unsigned_abs();
            if dr <= maxdist && dg <= maxdist && db <= maxdist {
                out_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok(out_mut.into())
}

/// Generate mask by color discrimination in 32bpp.
///
/// Pixels closer to `color1` than `color2` (by Euclidean distance in RGB
/// space) with a difference exceeding `mindiff` are set in the output mask.
///
/// # See also
///
/// C Leptonica: `pixGenerateMaskByDiscr32()` in `grayquant.c`
pub fn generate_mask_by_discr_32(
    pix: &Pix,
    color1: u32,
    color2: u32,
    mindiff: u32,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let (r1, g1, b1) = pixel::extract_rgb(color1);
    let (r2, g2, b2) = pixel::extract_rgb(color2);

    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(val);

            let dist1_sq = (r as i64 - r1 as i64).pow(2)
                + (g as i64 - g1 as i64).pow(2)
                + (b as i64 - b1 as i64).pow(2);
            let dist2_sq = (r as i64 - r2 as i64).pow(2)
                + (g as i64 - g2 as i64).pow(2)
                + (b as i64 - b2 as i64).pow(2);

            let diff = ((dist2_sq as f64).sqrt() - (dist1_sq as f64).sqrt()).abs() as u32;
            if dist1_sq < dist2_sq && diff >= mindiff {
                out_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok(out_mut.into())
}

/// Gray quantization using histogram analysis.
///
/// Builds a histogram of the grayscale image, identifies significant peaks,
/// creates a colormap from peak values, and quantizes each pixel to the
/// nearest colormap entry.
///
/// # See also
///
/// C Leptonica: `pixGrayQuantFromHisto()` in `grayquant.c`
pub fn gray_quant_from_histo(
    pix: &Pix,
    _mask: Option<&Pix>,
    minfract: f64,
    maxsize: u32,
) -> ColorResult<Pix> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();

    // Build histogram
    let mut histo = [0u32; 256];
    for y in 0..h {
        for x in 0..w {
            let val = gray.get_pixel_unchecked(x, y) as usize;
            histo[val] += 1;
        }
    }

    let total = (w * h) as f64;
    let min_count = (minfract * total) as u32;
    let maxsize = maxsize.clamp(2, 256);

    // Find significant peaks (local maxima above threshold)
    let mut peaks: Vec<u8> = Vec::new();
    for i in 1..255usize {
        if histo[i] >= min_count && histo[i] >= histo[i - 1] && histo[i] >= histo[i + 1] {
            peaks.push(i as u8);
        }
    }

    // If no peaks found, use evenly-spaced levels
    if peaks.is_empty() {
        let n = maxsize.min(8);
        for i in 0..n {
            peaks.push((i * 255 / (n - 1).max(1)) as u8);
        }
    }

    // Limit to maxsize entries
    while peaks.len() > maxsize as usize {
        peaks.pop();
    }

    // Build colormap from peaks
    let cmap_depth = if peaks.len() <= 2 {
        1
    } else if peaks.len() <= 4 {
        2
    } else if peaks.len() <= 16 {
        4
    } else {
        8
    };
    let mut cmap = PixColormap::new(cmap_depth)
        .map_err(|e| ColorError::InvalidParameters(format!("colormap creation failed: {e}")))?;
    for &p in &peaks {
        let _ = cmap.add_rgb(p, p, p);
    }

    let out_depth = match cmap_depth {
        1 => PixelDepth::Bit1,
        2 => PixelDepth::Bit2,
        4 => PixelDepth::Bit4,
        _ => PixelDepth::Bit8,
    };
    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();

    // Map each pixel to the nearest peak
    // Build LUT for speed
    let mut lut = [0u32; 256];
    for (i, entry) in lut.iter_mut().enumerate() {
        let mut best_idx = 0u32;
        let mut best_dist = u32::MAX;
        for (j, &p) in peaks.iter().enumerate() {
            let dist = (i as i32 - p as i32).unsigned_abs();
            if dist < best_dist {
                best_dist = dist;
                best_idx = j as u32;
            }
        }
        *entry = best_idx;
    }

    for y in 0..h {
        for x in 0..w {
            let val = gray.get_pixel_unchecked(x, y) as usize;
            out_mut.set_pixel_unchecked(x, y, lut[val.min(255)]);
        }
    }

    out_mut
        .set_colormap(Some(cmap))
        .map_err(|e| ColorError::InvalidParameters(format!("set_colormap failed: {e}")))?;

    Ok(out_mut.into())
}

/// Gray quantization from existing colormap.
///
/// Maps each pixel to the nearest gray value in the provided colormap.
///
/// # See also
///
/// C Leptonica: `pixGrayQuantFromCmap()` in `grayquant.c`
pub fn gray_quant_from_cmap(pix: &Pix, cmap: &PixColormap, mindepth: u32) -> ColorResult<Pix> {
    let gray = ensure_grayscale(pix)?;
    let w = gray.width();
    let h = gray.height();

    let ncolors = cmap.len();
    if ncolors == 0 {
        return Err(ColorError::InvalidParameters(
            "colormap must not be empty".into(),
        ));
    }

    // Extract gray values from colormap
    let cmap_grays: Vec<u8> = (0..ncolors)
        .map(|i| {
            let (r, g, b) = cmap.get_rgb(i).unwrap_or((128, 128, 128));
            ((r as u32 + g as u32 + b as u32) / 3) as u8
        })
        .collect();

    // Build LUT: for each 0–255 input, find nearest colormap entry
    let mut lut = [0u32; 256];
    for (i, entry) in lut.iter_mut().enumerate() {
        let mut best_idx = 0u32;
        let mut best_dist = u32::MAX;
        for (j, &g) in cmap_grays.iter().enumerate() {
            let dist = (i as i32 - g as i32).unsigned_abs();
            if dist < best_dist {
                best_dist = dist;
                best_idx = j as u32;
            }
        }
        *entry = best_idx;
    }

    // Determine output depth
    let out_depth_val = mindepth.max(cmap.depth());
    let out_depth = match out_depth_val {
        0..=1 => PixelDepth::Bit1,
        2 => PixelDepth::Bit2,
        3..=4 => PixelDepth::Bit4,
        _ => PixelDepth::Bit8,
    };

    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val = gray.get_pixel_unchecked(x, y) as usize;
            out_mut.set_pixel_unchecked(x, y, lut[val.min(255)]);
        }
    }

    // Clone and attach colormap
    let mut out_cmap = PixColormap::new(out_depth.bits())
        .map_err(|e| ColorError::InvalidParameters(format!("colormap creation failed: {e}")))?;
    for i in 0..ncolors {
        if let Some((r, g, b)) = cmap.get_rgb(i) {
            let _ = out_cmap.add_rgb(r, g, b);
        }
    }
    out_mut
        .set_colormap(Some(out_cmap))
        .map_err(|e| ColorError::InvalidParameters(format!("set_colormap failed: {e}")))?;

    Ok(out_mut.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pixel;

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

        // C convention: pixels < threshold → 1 (foreground), >= threshold → 0 (background)
        assert_eq!(binary.get_pixel_unchecked(0, 0), 1); // 0 < 128 → foreground
        assert_eq!(binary.get_pixel_unchecked(127, 0), 1); // 127 < 128 → foreground
        assert_eq!(binary.get_pixel_unchecked(128, 0), 0); // 128 >= 128 → background
        assert_eq!(binary.get_pixel_unchecked(255, 0), 0); // 255 >= 128 → background
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

        // C convention: pixel < threshold → 1 (foreground), >= threshold → 0 (background)
        // Left half has value 50, threshold > 50 → 50 < threshold → foreground (1)
        // Right half has value 200, threshold < 200 → 200 >= threshold → background (0)
        let left_val = binary.get_pixel_unchecked(25, 50);
        let right_val = binary.get_pixel_unchecked(75, 50);

        assert_eq!(
            left_val, 1,
            "Left half (value 50) should be foreground (1) when threshold is {}",
            threshold
        );
        assert_eq!(
            right_val, 0,
            "Right half (value 200) should be background (0) when threshold is {}",
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
                let pixel = pixel::compose_rgb(gray, gray, gray);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        let binary = threshold_to_binary(&pix_mut.into(), 128).unwrap();
        assert_eq!(binary.depth(), PixelDepth::Bit1);
    }
}
