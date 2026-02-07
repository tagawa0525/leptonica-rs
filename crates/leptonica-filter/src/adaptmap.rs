//! Adaptive mapping operations
//!
//! Implements local adaptive image enhancement techniques including:
//!
//! - Background normalization: Compensates for uneven illumination
//! - Contrast normalization: Expands local dynamic range
//!
//! These operations are tile-based and work well for:
//! - Document image enhancement
//! - Preprocessing for OCR
//! - Correcting uneven lighting conditions
//!
//! # Example
//!
//! ```ignore
//! use leptonica_filter::adaptmap::{background_norm_simple, contrast_norm_simple};
//!
//! // Normalize background with default parameters
//! let normalized = background_norm_simple(&pix)?;
//!
//! // Apply local contrast normalization
//! let enhanced = contrast_norm_simple(&pix)?;
//! ```

use crate::{FilterError, FilterResult};
use leptonica_core::{Pix, PixelDepth, color};

// ============================================================================
// Default parameters (matching C version defaults)
// ============================================================================

/// Default tile width for background normalization
pub const DEFAULT_TILE_WIDTH: u32 = 10;

/// Default tile height for background normalization
pub const DEFAULT_TILE_HEIGHT: u32 = 15;

/// Default foreground threshold (pixels below this are considered foreground)
pub const DEFAULT_FG_THRESHOLD: u32 = 60;

/// Default minimum count of background pixels required per tile
pub const DEFAULT_MIN_COUNT: u32 = 40;

/// Default target background value after normalization
pub const DEFAULT_BG_VAL: u32 = 200;

/// Default smoothing half-width in X direction
pub const DEFAULT_SMOOTH_X: u32 = 2;

/// Default smoothing half-width in Y direction
pub const DEFAULT_SMOOTH_Y: u32 = 1;

// Default parameters for contrast normalization
/// Default minimum difference for contrast normalization
pub const DEFAULT_MIN_DIFF: u32 = 50;

/// Default tile size for contrast normalization
pub const DEFAULT_CONTRAST_TILE_SIZE: u32 = 20;

// ============================================================================
// Option structures
// ============================================================================

/// Options for background normalization
#[derive(Debug, Clone)]
pub struct BackgroundNormOptions {
    /// Tile width in pixels (minimum 4)
    pub tile_width: u32,
    /// Tile height in pixels (minimum 4)
    pub tile_height: u32,
    /// Foreground threshold (pixels below this are excluded from background)
    pub fg_threshold: u32,
    /// Minimum count of background pixels required per tile
    pub min_count: u32,
    /// Target background value after normalization (typically 128-240)
    pub bg_val: u32,
    /// Half-width of smoothing kernel in X direction (0 = no smoothing)
    pub smooth_x: u32,
    /// Half-width of smoothing kernel in Y direction (0 = no smoothing)
    pub smooth_y: u32,
}

impl Default for BackgroundNormOptions {
    fn default() -> Self {
        Self {
            tile_width: DEFAULT_TILE_WIDTH,
            tile_height: DEFAULT_TILE_HEIGHT,
            fg_threshold: DEFAULT_FG_THRESHOLD,
            min_count: DEFAULT_MIN_COUNT,
            bg_val: DEFAULT_BG_VAL,
            smooth_x: DEFAULT_SMOOTH_X,
            smooth_y: DEFAULT_SMOOTH_Y,
        }
    }
}

/// Options for contrast normalization
#[derive(Debug, Clone)]
pub struct ContrastNormOptions {
    /// Tile width in pixels (minimum 5)
    pub tile_width: u32,
    /// Tile height in pixels (minimum 5)
    pub tile_height: u32,
    /// Minimum difference (max - min) to accept as valid contrast
    pub min_diff: u32,
    /// Half-width of smoothing kernel in X direction (0-8)
    pub smooth_x: u32,
    /// Half-width of smoothing kernel in Y direction (0-8)
    pub smooth_y: u32,
}

impl Default for ContrastNormOptions {
    fn default() -> Self {
        Self {
            tile_width: DEFAULT_CONTRAST_TILE_SIZE,
            tile_height: DEFAULT_CONTRAST_TILE_SIZE,
            min_diff: DEFAULT_MIN_DIFF,
            smooth_x: 2,
            smooth_y: 2,
        }
    }
}

// ============================================================================
// Public API: Background normalization
// ============================================================================

/// Normalize image background with default parameters
///
/// This is a simplified interface that uses default tile sizes and parameters.
/// Suitable for most document images.
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale or 32bpp RGB image
///
/// # Returns
/// Background-normalized image with the same depth as input
///
/// # Example
/// ```ignore
/// let normalized = background_norm_simple(&document_image)?;
/// ```
pub fn background_norm_simple(pix: &Pix) -> FilterResult<Pix> {
    background_norm(pix, &BackgroundNormOptions::default())
}

/// Normalize image background with custom parameters
///
/// Performs adaptive background normalization by:
/// 1. Estimating background values in each tile
/// 2. Creating an inverted background map
/// 3. Applying the map to normalize the image
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale or 32bpp RGB image
/// * `options` - Normalization parameters
///
/// # Returns
/// Background-normalized image
pub fn background_norm(pix: &Pix, options: &BackgroundNormOptions) -> FilterResult<Pix> {
    // Validate parameters
    if options.tile_width < 4 || options.tile_height < 4 {
        return Err(FilterError::InvalidParameters(
            "tile dimensions must be >= 4".to_string(),
        ));
    }
    if options.bg_val < 128 || options.bg_val > 255 {
        return Err(FilterError::InvalidParameters(
            "bg_val should be between 128 and 255".to_string(),
        ));
    }

    // Adjust min_count if too large for tile
    let mut min_count = options.min_count;
    if min_count > options.tile_width * options.tile_height {
        min_count = (options.tile_width * options.tile_height) / 3;
    }

    match pix.depth() {
        PixelDepth::Bit8 => background_norm_gray(pix, options, min_count),
        PixelDepth::Bit32 => background_norm_color(pix, options, min_count),
        _ => Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

// ============================================================================
// Internal: Grayscale background normalization
// ============================================================================

fn background_norm_gray(
    pix: &Pix,
    options: &BackgroundNormOptions,
    min_count: u32,
) -> FilterResult<Pix> {
    // Get background map
    let bg_map = get_background_gray_map(
        pix,
        options.tile_width,
        options.tile_height,
        options.fg_threshold,
        min_count,
    )?;

    // Get inverted background map
    let inv_map =
        get_inv_background_map(&bg_map, options.bg_val, options.smooth_x, options.smooth_y)?;

    // Apply the map
    apply_inv_background_gray_map(pix, &inv_map, options.tile_width, options.tile_height)
}

fn background_norm_color(
    pix: &Pix,
    options: &BackgroundNormOptions,
    min_count: u32,
) -> FilterResult<Pix> {
    // Extract RGB channels and process each independently
    let (pixr, pixg, pixb) = extract_rgb_channels(pix)?;

    // Get background maps for each channel
    let bg_map_r = get_background_gray_map(
        &pixr,
        options.tile_width,
        options.tile_height,
        options.fg_threshold,
        min_count,
    )?;
    let bg_map_g = get_background_gray_map(
        &pixg,
        options.tile_width,
        options.tile_height,
        options.fg_threshold,
        min_count,
    )?;
    let bg_map_b = get_background_gray_map(
        &pixb,
        options.tile_width,
        options.tile_height,
        options.fg_threshold,
        min_count,
    )?;

    // Get inverted maps
    let inv_map_r = get_inv_background_map(
        &bg_map_r,
        options.bg_val,
        options.smooth_x,
        options.smooth_y,
    )?;
    let inv_map_g = get_inv_background_map(
        &bg_map_g,
        options.bg_val,
        options.smooth_x,
        options.smooth_y,
    )?;
    let inv_map_b = get_inv_background_map(
        &bg_map_b,
        options.bg_val,
        options.smooth_x,
        options.smooth_y,
    )?;

    // Apply maps and combine channels
    let result_r =
        apply_inv_background_gray_map(&pixr, &inv_map_r, options.tile_width, options.tile_height)?;
    let result_g =
        apply_inv_background_gray_map(&pixg, &inv_map_g, options.tile_width, options.tile_height)?;
    let result_b =
        apply_inv_background_gray_map(&pixb, &inv_map_b, options.tile_width, options.tile_height)?;

    // Combine channels back into RGB
    combine_rgb_channels(&result_r, &result_g, &result_b, pix.spp())
}

// ============================================================================
// Internal: Background map generation
// ============================================================================

/// Generate background map for grayscale image
fn get_background_gray_map(
    pix: &Pix,
    tile_width: u32,
    tile_height: u32,
    fg_threshold: u32,
    min_count: u32,
) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    // Calculate map dimensions
    let map_w = w.div_ceil(tile_width);
    let map_h = h.div_ceil(tile_height);

    // Create the background map
    let map_pix = Pix::new(map_w, map_h, PixelDepth::Bit8)?;
    let mut map_mut = map_pix.try_into_mut().unwrap();

    // Number of complete tiles
    let nx = w / tile_width;
    let ny = h / tile_height;

    // Process each complete tile
    for ty in 0..ny {
        for tx in 0..nx {
            let tile_x = tx * tile_width;
            let tile_y = ty * tile_height;

            let mut sum: u32 = 0;
            let mut count: u32 = 0;

            // Accumulate background pixels in this tile
            for y in tile_y..(tile_y + tile_height) {
                for x in tile_x..(tile_x + tile_width) {
                    let val = pix.get_pixel_unchecked(x, y);
                    // Only include pixels above the foreground threshold
                    if val >= fg_threshold {
                        sum += val;
                        count += 1;
                    }
                }
            }

            // Set map value if we have enough background pixels
            if count >= min_count {
                let avg = sum / count;
                map_mut.set_pixel_unchecked(tx, ty, avg);
            }
            // Otherwise leave as 0 (will be filled later)
        }
    }

    // Convert to immutable for hole filling
    let map_pix = map_mut.into();

    // Fill holes in the map (tiles with value 0)
    fill_map_holes(&map_pix, nx, ny)
}

/// Fill holes (zero values) in the map by propagating from neighbors
fn fill_map_holes(pix: &Pix, valid_x: u32, valid_y: u32) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let out_pix = pix.deep_clone();
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // First pass: fill from left and top
    for y in 0..h {
        for x in 0..w {
            let val = out_mut.get_pixel_unchecked(x, y);
            if val == 0 {
                // Try to get value from left neighbor
                if x > 0 {
                    let left = out_mut.get_pixel_unchecked(x - 1, y);
                    if left > 0 {
                        out_mut.set_pixel_unchecked(x, y, left);
                        continue;
                    }
                }
                // Try to get value from top neighbor
                if y > 0 {
                    let top = out_mut.get_pixel_unchecked(x, y - 1);
                    if top > 0 {
                        out_mut.set_pixel_unchecked(x, y, top);
                    }
                }
            }
        }
    }

    // Second pass: fill from right and bottom
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let val = out_mut.get_pixel_unchecked(x, y);
            if val == 0 {
                // Try to get value from right neighbor
                if x + 1 < w {
                    let right = out_mut.get_pixel_unchecked(x + 1, y);
                    if right > 0 {
                        out_mut.set_pixel_unchecked(x, y, right);
                        continue;
                    }
                }
                // Try to get value from bottom neighbor
                if y + 1 < h {
                    let bottom = out_mut.get_pixel_unchecked(x, y + 1);
                    if bottom > 0 {
                        out_mut.set_pixel_unchecked(x, y, bottom);
                    }
                }
            }
        }
    }

    // Extend incomplete tiles at edges
    // Fill right edge from valid_x to w
    for y in 0..h {
        if valid_x < w {
            let last_valid = if valid_x > 0 {
                out_mut.get_pixel_unchecked(valid_x - 1, y)
            } else {
                128 // fallback
            };
            for x in valid_x..w {
                let val = out_mut.get_pixel_unchecked(x, y);
                if val == 0 {
                    out_mut.set_pixel_unchecked(x, y, last_valid);
                }
            }
        }
    }

    // Fill bottom edge from valid_y to h
    for x in 0..w {
        if valid_y < h {
            let last_valid = if valid_y > 0 {
                out_mut.get_pixel_unchecked(x, valid_y - 1)
            } else {
                128 // fallback
            };
            for y in valid_y..h {
                let val = out_mut.get_pixel_unchecked(x, y);
                if val == 0 {
                    out_mut.set_pixel_unchecked(x, y, last_valid);
                }
            }
        }
    }

    Ok(out_mut.into())
}

// ============================================================================
// Internal: Inverted background map
// ============================================================================

/// Generate inverted background map for normalization
///
/// The inverted map contains multiplication factors such that:
/// output = (input * factor) / 256
fn get_inv_background_map(
    pix: &Pix,
    bg_val: u32,
    smooth_x: u32,
    smooth_y: u32,
) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    // First smooth the map if requested
    let smoothed = if smooth_x > 0 || smooth_y > 0 {
        block_convolve_gray(pix, smooth_x, smooth_y)?
    } else {
        pix.deep_clone()
    };

    // Create 16bpp output map (stored as values 0-65535 in 32bpp for simplicity)
    // Actually, we'll use a Vec<u16> internally and create an 8bpp approximation
    // for the final map since we apply it tile by tile

    // For simplicity, we'll store the inverted factors as 16-bit values
    // but pack them into a structure we can use efficiently

    // Create output map with 16-bit precision stored as two 8-bit values
    // We'll use 32bpp to store the 16-bit values easily
    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val = smoothed.get_pixel_unchecked(x, y);
            let factor = if val > 0 {
                (256 * bg_val) / val
            } else {
                bg_val / 2 // fallback for zero values
            };
            // Store as 32-bit value (will be used directly in application)
            out_mut.set_pixel_unchecked(x, y, factor.min(65535));
        }
    }

    Ok(out_mut.into())
}

/// Simple block convolution for smoothing
fn block_convolve_gray(pix: &Pix, half_width_x: u32, half_width_y: u32) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let kw = 2 * half_width_x + 1;
    let kh = 2 * half_width_y + 1;
    let kernel_size = kw * kh;

    for y in 0..h {
        for x in 0..w {
            let mut sum: u32 = 0;

            for ky in 0..kh {
                for kx in 0..kw {
                    let sx =
                        (x as i32 + kx as i32 - half_width_x as i32).clamp(0, w as i32 - 1) as u32;
                    let sy =
                        (y as i32 + ky as i32 - half_width_y as i32).clamp(0, h as i32 - 1) as u32;
                    sum += pix.get_pixel_unchecked(sx, sy);
                }
            }

            let avg = sum / kernel_size;
            out_mut.set_pixel_unchecked(x, y, avg);
        }
    }

    Ok(out_mut.into())
}

// ============================================================================
// Internal: Apply background map
// ============================================================================

/// Apply inverted background map to grayscale image
fn apply_inv_background_gray_map(
    pix: &Pix,
    inv_map: &Pix,
    tile_width: u32,
    tile_height: u32,
) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let map_w = inv_map.width();
    let map_h = inv_map.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for ty in 0..map_h {
        for tx in 0..map_w {
            // Get the multiplication factor for this tile
            let factor = inv_map.get_pixel_unchecked(tx, ty);

            // Calculate tile boundaries
            let x_start = tx * tile_width;
            let y_start = ty * tile_height;
            let x_end = (x_start + tile_width).min(w);
            let y_end = (y_start + tile_height).min(h);

            // Apply factor to each pixel in the tile
            for y in y_start..y_end {
                for x in x_start..x_end {
                    let val = pix.get_pixel_unchecked(x, y);
                    let normalized = (val * factor) / 256;
                    out_mut.set_pixel_unchecked(x, y, normalized.min(255));
                }
            }
        }
    }

    Ok(out_mut.into())
}

// ============================================================================
// Internal: RGB channel operations
// ============================================================================

/// Extract R, G, B channels from a 32bpp image
fn extract_rgb_channels(pix: &Pix) -> FilterResult<(Pix, Pix, Pix)> {
    let w = pix.width();
    let h = pix.height();

    let pix_r = Pix::new(w, h, PixelDepth::Bit8)?;
    let pix_g = Pix::new(w, h, PixelDepth::Bit8)?;
    let pix_b = Pix::new(w, h, PixelDepth::Bit8)?;

    let mut r_mut = pix_r.try_into_mut().unwrap();
    let mut g_mut = pix_g.try_into_mut().unwrap();
    let mut b_mut = pix_b.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b, _) = color::extract_rgba(pixel);
            r_mut.set_pixel_unchecked(x, y, r as u32);
            g_mut.set_pixel_unchecked(x, y, g as u32);
            b_mut.set_pixel_unchecked(x, y, b as u32);
        }
    }

    Ok((r_mut.into(), g_mut.into(), b_mut.into()))
}

/// Combine R, G, B channels into a 32bpp image
fn combine_rgb_channels(pix_r: &Pix, pix_g: &Pix, pix_b: &Pix, spp: u32) -> FilterResult<Pix> {
    let w = pix_r.width();
    let h = pix_r.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(spp);

    for y in 0..h {
        for x in 0..w {
            let r = pix_r.get_pixel_unchecked(x, y) as u8;
            let g = pix_g.get_pixel_unchecked(x, y) as u8;
            let b = pix_b.get_pixel_unchecked(x, y) as u8;
            let pixel = color::compose_rgb(r, g, b);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }

    Ok(out_mut.into())
}

// ============================================================================
// Public API: Contrast normalization
// ============================================================================

/// Local contrast normalization with default parameters
///
/// Adaptively expands the dynamic range in each tile to the full 8-bit range.
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale image
///
/// # Returns
/// Contrast-normalized image
pub fn contrast_norm_simple(pix: &Pix) -> FilterResult<Pix> {
    contrast_norm(pix, &ContrastNormOptions::default())
}

/// Local contrast normalization with custom parameters
///
/// For each tile, expands the local dynamic range so that the minimum
/// value maps to 0 and the maximum value maps to 255.
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale image
/// * `options` - Normalization parameters
///
/// # Returns
/// Contrast-normalized image
pub fn contrast_norm(pix: &Pix, options: &ContrastNormOptions) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 bpp",
            actual: pix.depth().bits(),
        });
    }

    // Validate parameters
    if options.tile_width < 5 || options.tile_height < 5 {
        return Err(FilterError::InvalidParameters(
            "tile dimensions must be >= 5".to_string(),
        ));
    }
    if options.smooth_x > 8 || options.smooth_y > 8 {
        return Err(FilterError::InvalidParameters(
            "smooth parameters must be <= 8".to_string(),
        ));
    }

    // Get min/max tiles
    let (pix_min, pix_max) = min_max_tiles(
        pix,
        options.tile_width,
        options.tile_height,
        options.min_diff,
        options.smooth_x,
        options.smooth_y,
    )?;

    // Apply linear TRC tiled
    linear_trc_tiled(
        pix,
        options.tile_width,
        options.tile_height,
        &pix_min,
        &pix_max,
    )
}

// ============================================================================
// Internal: Contrast normalization helpers
// ============================================================================

/// Compute min and max values for each tile
fn min_max_tiles(
    pix: &Pix,
    tile_width: u32,
    tile_height: u32,
    min_diff: u32,
    smooth_x: u32,
    smooth_y: u32,
) -> FilterResult<(Pix, Pix)> {
    let w = pix.width();
    let h = pix.height();

    // Map dimensions
    let map_w = w.div_ceil(tile_width);
    let map_h = h.div_ceil(tile_height);

    // Create min and max maps
    let pix_min = Pix::new(map_w, map_h, PixelDepth::Bit8)?;
    let pix_max = Pix::new(map_w, map_h, PixelDepth::Bit8)?;

    let mut min_mut = pix_min.try_into_mut().unwrap();
    let mut max_mut = pix_max.try_into_mut().unwrap();

    let nx = w / tile_width;
    let ny = h / tile_height;

    // Compute min/max for each complete tile
    for ty in 0..ny {
        for tx in 0..nx {
            let tile_x = tx * tile_width;
            let tile_y = ty * tile_height;

            let mut min_val = 255u32;
            let mut max_val = 0u32;

            for y in tile_y..(tile_y + tile_height) {
                for x in tile_x..(tile_x + tile_width) {
                    let val = pix.get_pixel_unchecked(x, y);
                    min_val = min_val.min(val);
                    max_val = max_val.max(val);
                }
            }

            // Add 1 to avoid zeros (which indicate holes)
            min_mut.set_pixel_unchecked(tx, ty, min_val.saturating_add(1).min(255));
            max_mut.set_pixel_unchecked(tx, ty, max_val.saturating_add(1).min(255));
        }
    }

    // Extend to edges
    for ty in 0..map_h {
        for tx in nx..map_w {
            let src_x = if nx > 0 { nx - 1 } else { 0 };
            let min_val = min_mut.get_pixel_unchecked(src_x, ty);
            let max_val = max_mut.get_pixel_unchecked(src_x, ty);
            min_mut.set_pixel_unchecked(tx, ty, min_val);
            max_mut.set_pixel_unchecked(tx, ty, max_val);
        }
    }
    for tx in 0..map_w {
        for ty in ny..map_h {
            let src_y = if ny > 0 { ny - 1 } else { 0 };
            let min_val = min_mut.get_pixel_unchecked(tx, src_y);
            let max_val = max_mut.get_pixel_unchecked(tx, src_y);
            min_mut.set_pixel_unchecked(tx, ty, min_val);
            max_mut.set_pixel_unchecked(tx, ty, max_val);
        }
    }

    let pix_min: Pix = min_mut.into();
    let pix_max: Pix = max_mut.into();

    // Set low contrast tiles to zero (will be filled later)
    let (pix_min, pix_max) = set_low_contrast(pix_min, pix_max, min_diff)?;

    // Fill holes
    let pix_min = fill_map_holes(&pix_min, map_w, map_h)?;
    let pix_max = fill_map_holes(&pix_max, map_w, map_h)?;

    // Smooth if requested
    let pix_min = if smooth_x > 0 || smooth_y > 0 {
        let sx = smooth_x.min((map_w - 1) / 2);
        let sy = smooth_y.min((map_h - 1) / 2);
        block_convolve_gray(&pix_min, sx, sy)?
    } else {
        pix_min
    };

    let pix_max = if smooth_x > 0 || smooth_y > 0 {
        let sx = smooth_x.min((map_w - 1) / 2);
        let sy = smooth_y.min((map_h - 1) / 2);
        block_convolve_gray(&pix_max, sx, sy)?
    } else {
        pix_max
    };

    Ok((pix_min, pix_max))
}

/// Set low contrast tiles to zero (to be filled later)
/// Consumes the inputs and returns new Pix with low contrast tiles set to 0
fn set_low_contrast(pix_min: Pix, pix_max: Pix, min_diff: u32) -> FilterResult<(Pix, Pix)> {
    let w = pix_min.width();
    let h = pix_min.height();

    let out_min = pix_min.deep_clone();
    let out_max = pix_max.deep_clone();

    let mut min_mut = out_min.try_into_mut().unwrap();
    let mut max_mut = out_max.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let min_val = pix_min.get_pixel_unchecked(x, y);
            let max_val = pix_max.get_pixel_unchecked(x, y);

            // Values have been offset by 1, so subtract to get actual diff
            let actual_min = min_val.saturating_sub(1);
            let actual_max = max_val.saturating_sub(1);

            if actual_max.saturating_sub(actual_min) < min_diff {
                min_mut.set_pixel_unchecked(x, y, 0);
                max_mut.set_pixel_unchecked(x, y, 0);
            }
        }
    }

    Ok((min_mut.into(), max_mut.into()))
}

/// Apply linear TRC tiled to expand contrast
fn linear_trc_tiled(
    pix: &Pix,
    tile_width: u32,
    tile_height: u32,
    pix_min: &Pix,
    pix_max: &Pix,
) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let map_w = pix_min.width();
    let map_h = pix_min.height();

    let out_pix = pix.deep_clone();
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Build LUT cache (indexed by diff value)
    let mut lut_cache: [Option<[u8; 256]>; 256] = [None; 256];

    for ty in 0..map_h {
        for tx in 0..map_w {
            let min_val = pix_min.get_pixel_unchecked(tx, ty).saturating_sub(1);
            let max_val = pix_max.get_pixel_unchecked(tx, ty).saturating_sub(1);

            if max_val <= min_val {
                continue; // Skip tiles with no contrast
            }

            let diff = (max_val - min_val) as usize;

            // Get or create LUT for this diff
            let lut = if let Some(existing) = &lut_cache[diff] {
                existing
            } else {
                let mut new_lut = [0u8; 256];
                let factor = 255.0 / diff as f32;
                for (i, lut_val) in new_lut.iter_mut().enumerate().take(diff + 1) {
                    *lut_val = ((factor * i as f32) + 0.5).min(255.0) as u8;
                }
                for lut_val in new_lut.iter_mut().skip(diff + 1) {
                    *lut_val = 255;
                }
                lut_cache[diff] = Some(new_lut);
                lut_cache[diff].as_ref().unwrap()
            };

            // Apply LUT to tile
            let x_start = tx * tile_width;
            let y_start = ty * tile_height;
            let x_end = (x_start + tile_width).min(w);
            let y_end = (y_start + tile_height).min(h);

            for y in y_start..y_end {
                for x in x_start..x_end {
                    let val = pix.get_pixel_unchecked(x, y);
                    let shifted = val.saturating_sub(min_val) as usize;
                    let mapped = lut[shifted.min(255)];
                    out_mut.set_pixel_unchecked(x, y, mapped as u32);
                }
            }
        }
    }

    Ok(out_mut.into())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_gray_image() -> Pix {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create an image with uneven lighting (gradient + content)
        for y in 0..50 {
            for x in 0..50 {
                // Background gradient (darker on left, brighter on right)
                let bg = 100 + x * 2;
                // Add some "text" (dark) in the center
                let val = if x > 15 && x < 35 && y > 15 && y < 35 {
                    bg / 2
                } else {
                    bg
                };
                pix_mut.set_pixel_unchecked(x, y, val.min(255));
            }
        }

        pix_mut.into()
    }

    fn create_test_color_image() -> Pix {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..50 {
            for x in 0..50 {
                let r = (100 + x * 2).min(255) as u8;
                let g = (150 + y).min(255) as u8;
                let b = 180u8;
                let pixel = color::compose_rgb(r, g, b);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    fn create_low_contrast_image() -> Pix {
        let pix = Pix::new(40, 40, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Uniform low contrast image
        for y in 0..40 {
            for x in 0..40 {
                // Values between 100 and 120
                let val = 100 + ((x + y) % 20);
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_background_norm_simple_gray() {
        let pix = create_test_gray_image();
        let result = background_norm_simple(&pix).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_background_norm_simple_color() {
        let pix = create_test_color_image();
        let result = background_norm_simple(&pix).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_background_norm_custom_options() {
        let pix = create_test_gray_image();
        let options = BackgroundNormOptions {
            tile_width: 8,
            tile_height: 8,
            fg_threshold: 50,
            min_count: 20,
            bg_val: 180,
            smooth_x: 1,
            smooth_y: 1,
        };
        let result = background_norm(&pix, &options).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_background_norm_invalid_tile_size() {
        let pix = create_test_gray_image();
        let options = BackgroundNormOptions {
            tile_width: 2, // Too small
            ..Default::default()
        };
        assert!(background_norm(&pix, &options).is_err());
    }

    #[test]
    fn test_background_norm_invalid_bg_val() {
        let pix = create_test_gray_image();
        let options = BackgroundNormOptions {
            bg_val: 50, // Too small
            ..Default::default()
        };
        assert!(background_norm(&pix, &options).is_err());
    }

    #[test]
    fn test_contrast_norm_simple() {
        let pix = create_test_gray_image();
        let result = contrast_norm_simple(&pix).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_contrast_norm_custom_options() {
        let pix = create_test_gray_image();
        let options = ContrastNormOptions {
            tile_width: 10,
            tile_height: 10,
            min_diff: 30,
            smooth_x: 1,
            smooth_y: 1,
        };
        let result = contrast_norm(&pix, &options).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_contrast_norm_invalid_tile_size() {
        let pix = create_test_gray_image();
        let options = ContrastNormOptions {
            tile_width: 3, // Too small
            ..Default::default()
        };
        assert!(contrast_norm(&pix, &options).is_err());
    }

    #[test]
    fn test_contrast_norm_invalid_smooth() {
        let pix = create_test_gray_image();
        let options = ContrastNormOptions {
            smooth_x: 10, // Too large
            ..Default::default()
        };
        assert!(contrast_norm(&pix, &options).is_err());
    }

    #[test]
    fn test_contrast_norm_color_not_supported() {
        let pix = create_test_color_image();
        let result = contrast_norm_simple(&pix);
        assert!(result.is_err());
    }

    #[test]
    fn test_contrast_norm_low_contrast_image() {
        let pix = create_low_contrast_image();
        // Should still work even with low contrast input
        let result = contrast_norm_simple(&pix).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_block_convolve_gray() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..20 {
            for x in 0..20 {
                pix_mut.set_pixel_unchecked(x, y, 128);
            }
        }
        let pix = pix_mut.into();

        let result = block_convolve_gray(&pix, 2, 2).unwrap();
        // Uniform input should give uniform output
        for y in 0..20 {
            for x in 0..20 {
                let val = result.get_pixel_unchecked(x, y);
                assert!((127..=129).contains(&val), "Expected ~128, got {}", val);
            }
        }
    }

    #[test]
    fn test_fill_map_holes() {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set some values, leave others as holes (0)
        pix_mut.set_pixel_unchecked(0, 0, 100);
        pix_mut.set_pixel_unchecked(4, 4, 200);

        let pix = pix_mut.into();

        let filled = fill_map_holes(&pix, 5, 5).unwrap();

        // Check that holes are filled
        for y in 0..5 {
            for x in 0..5 {
                let val = filled.get_pixel_unchecked(x, y);
                assert!(val > 0, "Hole at ({}, {}) not filled", x, y);
            }
        }
    }
}
