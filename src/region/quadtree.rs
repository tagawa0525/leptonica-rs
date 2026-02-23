//! Quadtree (四分木) - Hierarchical image region decomposition
//!
//! This module provides functions for computing statistics on quadtree
//! decompositions of images. A quadtree recursively divides an image into
//! four quadrants, creating a hierarchical representation useful for
//! spatial analysis and adaptive processing.
//!
//! # Overview
//!
//! The quadtree decomposes an image into levels:
//! - Level 0: The entire image (1x1 block)
//! - Level 1: 4 quadrants (2x2 blocks)
//! - Level 2: 16 blocks (4x4)
//! - Level n: 4^n blocks (2^n x 2^n)
//!
//! # Features
//!
//! - **Mean computation**: Calculate average pixel values at each level
//! - **Variance computation**: Calculate variance and root variance at each level
//! - **Integral images**: O(1) computation of statistics for arbitrary rectangles
//! - **Parent/child navigation**: Traverse the quadtree hierarchy
//!
//! # Examples
//!
//! ## Computing quadtree mean values
//!
//! ```
//! use leptonica_region::quadtree::{quadtree_mean, quadtree_max_levels};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! // Create an 8-bit grayscale image
//! let pix = Pix::new(64, 64, PixelDepth::Bit8).unwrap();
//!
//! // Compute maximum allowed levels
//! let max_levels = quadtree_max_levels(64, 64);
//!
//! // Compute mean values at each level
//! let result = quadtree_mean(&pix, 3).unwrap();
//! assert_eq!(result.num_levels(), 3);
//! ```

use crate::error::{RegionError, RegionResult};
use leptonica_core::{Box, Boxa, Boxaa, FPix, Pix, PixelDepth};

// ============================================================================
// Integral Image
// ============================================================================

/// Integral image (Summed Area Table) for O(1) rectangle sum computation
///
/// An integral image stores at each position (x, y) the sum of all pixels
/// in the rectangle from (0, 0) to (x, y). This allows computing the sum
/// of any rectangular region in constant time.
///
/// Uses `u64` to prevent overflow for large images.
#[derive(Debug, Clone)]
pub struct IntegralImage {
    data: Vec<u64>,
    width: u32,
    height: u32,
}

impl IntegralImage {
    /// Create an integral image from an 8-bit grayscale Pix
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 8-bit grayscale.
    pub fn from_pix(pix: &Pix) -> RegionResult<Self> {
        if pix.depth() != PixelDepth::Bit8 {
            return Err(RegionError::UnsupportedDepth {
                expected: "8-bit grayscale",
                actual: pix.depth().bits(),
            });
        }

        let width = pix.width();
        let height = pix.height();

        if width == 0 || height == 0 {
            return Err(RegionError::EmptyImage);
        }

        let mut data = vec![0u64; (width as usize) * (height as usize)];

        // First pass: compute integral image
        for y in 0..height {
            let mut row_sum: u64 = 0;
            for x in 0..width {
                let pixel = pix.get_pixel(x, y).unwrap_or(0) as u64;
                row_sum += pixel;

                let idx = (y as usize) * (width as usize) + (x as usize);
                if y > 0 {
                    let above_idx = ((y - 1) as usize) * (width as usize) + (x as usize);
                    data[idx] = row_sum + data[above_idx];
                } else {
                    data[idx] = row_sum;
                }
            }
        }

        Ok(Self {
            data,
            width,
            height,
        })
    }

    /// Get the integral value at position (x, y)
    ///
    /// Returns the sum of all pixels from (0,0) to (x,y) inclusive.
    #[inline]
    pub fn get(&self, x: u32, y: u32) -> Option<u64> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        Some(self.data[idx])
    }

    /// Get the sum of pixels in a rectangular region
    ///
    /// The region is defined by top-left (x, y) and dimensions (w, h).
    /// Uses the summed area table formula for O(1) computation.
    pub fn sum_rect(&self, x: u32, y: u32, w: u32, h: u32) -> Option<u64> {
        if w == 0 || h == 0 {
            return Some(0);
        }

        let x2 = x + w - 1;
        let y2 = y + h - 1;

        if x2 >= self.width || y2 >= self.height {
            return None;
        }

        // Sum = I(x2,y2) - I(x-1,y2) - I(x2,y-1) + I(x-1,y-1)
        // Using wrapping arithmetic to handle the subtraction correctly
        let val11 = self.get(x2, y2)?;

        let val01 = if x > 0 { self.get(x - 1, y2)? } else { 0 };
        let val10 = if y > 0 { self.get(x2, y - 1)? } else { 0 };
        let val00 = if x > 0 && y > 0 {
            self.get(x - 1, y - 1)?
        } else {
            0
        };

        // Compute: val11 - val01 - val10 + val00
        // Rearrange to avoid overflow: (val11 + val00) - (val01 + val10)
        Some(
            val11
                .wrapping_sub(val01)
                .wrapping_sub(val10)
                .wrapping_add(val00),
        )
    }

    /// Get the width of the integral image
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of the integral image
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }
}

// ============================================================================
// Squared Integral Image
// ============================================================================

/// Squared integral image for O(1) variance computation
///
/// Stores the sum of squared pixel values, enabling variance calculation
/// using the formula: Var = E[X^2] - E[X]^2
///
/// Uses `f64` to maintain precision for squared values.
#[derive(Debug, Clone)]
pub struct SquaredIntegralImage {
    data: Vec<f64>,
    width: u32,
    height: u32,
}

impl SquaredIntegralImage {
    /// Create a squared integral image from an 8-bit grayscale Pix
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 8-bit grayscale.
    pub fn from_pix(pix: &Pix) -> RegionResult<Self> {
        if pix.depth() != PixelDepth::Bit8 {
            return Err(RegionError::UnsupportedDepth {
                expected: "8-bit grayscale",
                actual: pix.depth().bits(),
            });
        }

        let width = pix.width();
        let height = pix.height();

        if width == 0 || height == 0 {
            return Err(RegionError::EmptyImage);
        }

        let mut data = vec![0.0f64; (width as usize) * (height as usize)];

        // First pass: compute squared integral image
        for y in 0..height {
            let mut row_sum: f64 = 0.0;
            for x in 0..width {
                let pixel = pix.get_pixel(x, y).unwrap_or(0) as f64;
                row_sum += pixel * pixel;

                let idx = (y as usize) * (width as usize) + (x as usize);
                if y > 0 {
                    let above_idx = ((y - 1) as usize) * (width as usize) + (x as usize);
                    data[idx] = row_sum + data[above_idx];
                } else {
                    data[idx] = row_sum;
                }
            }
        }

        Ok(Self {
            data,
            width,
            height,
        })
    }

    /// Get the squared integral value at position (x, y)
    #[inline]
    pub fn get(&self, x: u32, y: u32) -> Option<f64> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        Some(self.data[idx])
    }

    /// Get the sum of squared pixels in a rectangular region
    pub fn sum_rect(&self, x: u32, y: u32, w: u32, h: u32) -> Option<f64> {
        if w == 0 || h == 0 {
            return Some(0.0);
        }

        let x2 = x + w - 1;
        let y2 = y + h - 1;

        if x2 >= self.width || y2 >= self.height {
            return None;
        }

        let val11 = self.get(x2, y2)?;
        let val01 = if x > 0 { self.get(x - 1, y2)? } else { 0.0 };
        let val10 = if y > 0 { self.get(x2, y - 1)? } else { 0.0 };
        let val00 = if x > 0 && y > 0 {
            self.get(x - 1, y - 1)?
        } else {
            0.0
        };

        Some(val11 - val01 - val10 + val00)
    }

    /// Get the width
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }
}

// ============================================================================
// Quadtree Result
// ============================================================================

/// Result of quadtree statistics computation
///
/// Contains the computed values (mean, variance, etc.) at each level of
/// the quadtree decomposition. Level 0 contains a single value for the
/// entire image, level 1 contains 2x2 values, etc.
#[derive(Debug, Clone)]
pub struct QuadtreeResult {
    /// Statistics at each level (level 0 = 1x1, level 1 = 2x2, etc.)
    levels: Vec<FPix>,
}

impl QuadtreeResult {
    /// Create a new QuadtreeResult with the given levels
    pub fn new(levels: Vec<FPix>) -> Self {
        Self { levels }
    }

    /// Get the number of levels in the quadtree
    #[inline]
    pub fn num_levels(&self) -> usize {
        self.levels.len()
    }

    /// Get the value at a specific position and level
    ///
    /// # Arguments
    ///
    /// * `level` - The quadtree level (0 = entire image)
    /// * `x` - X coordinate within the level
    /// * `y` - Y coordinate within the level
    ///
    /// # Returns
    ///
    /// The value at the specified position, or `None` if out of bounds.
    pub fn get_value(&self, level: usize, x: u32, y: u32) -> Option<f32> {
        let fpix = self.levels.get(level)?;
        fpix.get_pixel(x, y).ok()
    }

    /// Get the parent value of a node
    ///
    /// The parent is located at level - 1, position (x/2, y/2).
    ///
    /// # Arguments
    ///
    /// * `level` - Current level (must be > 0)
    /// * `x` - X coordinate at current level
    /// * `y` - Y coordinate at current level
    ///
    /// # Returns
    ///
    /// The parent value, or `None` if level is 0 or coordinates are invalid.
    pub fn get_parent(&self, level: usize, x: u32, y: u32) -> Option<f32> {
        if level == 0 || level >= self.levels.len() {
            return None;
        }

        let parent_x = x / 2;
        let parent_y = y / 2;

        self.get_value(level - 1, parent_x, parent_y)
    }

    /// Get the four child values of a node
    ///
    /// The children are located at level + 1, positions:
    /// - (2x, 2y), (2x+1, 2y), (2x, 2y+1), (2x+1, 2y+1)
    ///
    /// # Arguments
    ///
    /// * `level` - Current level
    /// * `x` - X coordinate at current level
    /// * `y` - Y coordinate at current level
    ///
    /// # Returns
    ///
    /// Array of [top-left, top-right, bottom-left, bottom-right] child values,
    /// or `None` if the node has no children.
    pub fn get_children(&self, level: usize, x: u32, y: u32) -> Option<[f32; 4]> {
        if level + 1 >= self.levels.len() {
            return None;
        }

        let child_x = x * 2;
        let child_y = y * 2;

        let val00 = self.get_value(level + 1, child_x, child_y)?;
        let val10 = self.get_value(level + 1, child_x + 1, child_y)?;
        let val01 = self.get_value(level + 1, child_x, child_y + 1)?;
        let val11 = self.get_value(level + 1, child_x + 1, child_y + 1)?;

        Some([val00, val10, val01, val11])
    }

    /// Get the FPix at a specific level
    pub fn get_level(&self, level: usize) -> Option<&FPix> {
        self.levels.get(level)
    }

    /// Get all levels as a slice
    pub fn levels(&self) -> &[FPix] {
        &self.levels
    }
}

// ============================================================================
// Quadtree Functions
// ============================================================================

/// Calculate the maximum number of levels allowed for a given image size
///
/// The criterion is that subdivision should not go below single pixel level.
/// A factor of 1.5 is used to prevent any rectangle from having zero dimension
/// due to integer truncation.
///
/// # Arguments
///
/// * `width` - Image width
/// * `height` - Image height
///
/// # Returns
///
/// Maximum number of levels. Returns the number of levels that can be used,
/// where level 0 = 1 block (entire image), level 1 = 4 blocks (2x2), etc.
pub fn quadtree_max_levels(width: u32, height: u32) -> u32 {
    if width == 0 || height == 0 {
        return 0;
    }

    let min_side = width.min(height);

    // Find the maximum level where we can still subdivide
    // At level n, we have 2^n blocks per side
    // We need min_side >= 2^n for valid subdivision
    // With safety margin: min_side >= 1.5 * 2^(n-1) for n > 0
    //
    // Following C version logic:
    // for i in 0..20:
    //   if min_side < 1.5 * 2^i:
    //     return i - 1  (but at least 0)
    for i in 0..20u32 {
        let threshold = (1.5 * (1u64 << i) as f64) as u32;
        if min_side < threshold {
            return i.saturating_sub(1);
        }
    }

    19 // Maximum for very large images
}

/// Generate quadtree regions for each level
///
/// Creates a Boxaa where each Boxa contains the rectangular regions
/// for that level of the quadtree.
///
/// # Arguments
///
/// * `width` - Image width
/// * `height` - Image height
/// * `nlevels` - Number of levels to generate
///
/// # Returns
///
/// A Boxaa with `nlevels` Boxa, each containing the regions for that level.
///
/// # Errors
///
/// Returns an error if nlevels is 0 or too large for the image dimensions.
pub fn quadtree_regions(width: u32, height: u32, nlevels: u32) -> RegionResult<Boxaa> {
    if nlevels == 0 {
        return Err(RegionError::InvalidParameters(
            "nlevels must be >= 1".to_string(),
        ));
    }

    let max_levels = quadtree_max_levels(width, height);
    if nlevels > max_levels + 1 {
        return Err(RegionError::InvalidParameters(format!(
            "nlevels {} exceeds maximum {} for {}x{} image",
            nlevels,
            max_levels + 1,
            width,
            height
        )));
    }

    let mut baa = Boxaa::with_capacity(nlevels as usize);

    for level in 0..nlevels {
        let nside = 1u32 << level; // Number of boxes per side at this level
        let nbox = nside * nside;

        let mut boxa = Boxa::with_capacity(nbox as usize);

        // Calculate box boundaries for this level
        let mut xstart = vec![0i32; nside as usize];
        let mut xend = vec![0i32; nside as usize];
        let mut ystart = vec![0i32; nside as usize];
        let mut yend = vec![0i32; nside as usize];

        for i in 0..nside {
            xstart[i as usize] = ((width - 1) * i / nside) as i32;
            if i > 0 {
                xstart[i as usize] += 1;
            }
            xend[i as usize] = ((width - 1) * (i + 1) / nside) as i32;

            ystart[i as usize] = ((height - 1) * i / nside) as i32;
            if i > 0 {
                ystart[i as usize] += 1;
            }
            yend[i as usize] = ((height - 1) * (i + 1) / nside) as i32;
        }

        // Create boxes in raster order (row by row)
        for row in 0..nside {
            let bh = yend[row as usize] - ystart[row as usize] + 1;
            for col in 0..nside {
                let bw = xend[col as usize] - xstart[col as usize] + 1;
                let b = Box::new_unchecked(xstart[col as usize], ystart[row as usize], bw, bh);
                boxa.push(b);
            }
        }

        baa.push(boxa);
    }

    Ok(baa)
}

/// Compute the mean value in a rectangular region using an integral image
///
/// This function performs the computation in O(1) time, regardless of
/// the rectangle size.
///
/// # Arguments
///
/// * `rect` - The rectangular region
/// * `integral` - The precomputed integral image
///
/// # Returns
///
/// The mean pixel value in the region.
pub fn mean_in_rectangle(rect: &Box, integral: &IntegralImage) -> RegionResult<f32> {
    if rect.w <= 0 || rect.h <= 0 {
        return Err(RegionError::InvalidParameters(
            "rectangle has zero or negative dimension".to_string(),
        ));
    }

    let x = rect.x.max(0) as u32;
    let y = rect.y.max(0) as u32;
    let w = rect.w as u32;
    let h = rect.h as u32;

    // Clip to image bounds
    let x2 = (x + w).min(integral.width());
    let y2 = (y + h).min(integral.height());
    let clipped_w = x2.saturating_sub(x);
    let clipped_h = y2.saturating_sub(y);

    if clipped_w == 0 || clipped_h == 0 {
        return Err(RegionError::InvalidParameters(
            "rectangle outside image bounds".to_string(),
        ));
    }

    let sum = integral
        .sum_rect(x, y, clipped_w, clipped_h)
        .ok_or_else(|| RegionError::InvalidParameters("failed to compute sum".to_string()))?;

    let count = (clipped_w as u64) * (clipped_h as u64);
    Ok((sum as f64 / count as f64) as f32)
}

/// Compute variance and root variance in a rectangular region
///
/// Uses both the integral image (for mean) and squared integral image
/// (for mean of squares) to compute variance in O(1) time.
///
/// Variance = E[X^2] - E[X]^2
///
/// # Arguments
///
/// * `rect` - The rectangular region
/// * `integral` - The precomputed integral image
/// * `sq_integral` - The precomputed squared integral image
///
/// # Returns
///
/// A tuple of (variance, root_variance).
pub fn variance_in_rectangle(
    rect: &Box,
    integral: &IntegralImage,
    sq_integral: &SquaredIntegralImage,
) -> RegionResult<(f32, f32)> {
    if rect.w <= 0 || rect.h <= 0 {
        return Err(RegionError::InvalidParameters(
            "rectangle has zero or negative dimension".to_string(),
        ));
    }

    let x = rect.x.max(0) as u32;
    let y = rect.y.max(0) as u32;
    let w = rect.w as u32;
    let h = rect.h as u32;

    // Clip to image bounds
    let x2 = (x + w).min(integral.width());
    let y2 = (y + h).min(integral.height());
    let clipped_w = x2.saturating_sub(x);
    let clipped_h = y2.saturating_sub(y);

    if clipped_w == 0 || clipped_h == 0 {
        return Err(RegionError::InvalidParameters(
            "rectangle outside image bounds".to_string(),
        ));
    }

    let sum = integral
        .sum_rect(x, y, clipped_w, clipped_h)
        .ok_or_else(|| RegionError::InvalidParameters("failed to compute sum".to_string()))?;

    let sq_sum = sq_integral
        .sum_rect(x, y, clipped_w, clipped_h)
        .ok_or_else(|| {
            RegionError::InvalidParameters("failed to compute squared sum".to_string())
        })?;

    let count = (clipped_w as f64) * (clipped_h as f64);
    let mean = sum as f64 / count;
    let mean_sq = sq_sum / count;

    // Variance = E[X^2] - E[X]^2
    let variance = (mean_sq - mean * mean).max(0.0); // Ensure non-negative due to floating point
    let root_variance = variance.sqrt();

    Ok((variance as f32, root_variance as f32))
}

/// Compute quadtree mean values for an image
///
/// Creates a hierarchical representation where each level contains
/// the mean pixel values for that level's subdivision.
///
/// # Arguments
///
/// * `pix` - Input 8-bit grayscale image
/// * `nlevels` - Number of quadtree levels
///
/// # Returns
///
/// A `QuadtreeResult` containing the mean values at each level.
///
/// # Errors
///
/// Returns an error if the image is not 8-bit grayscale or nlevels is invalid.
pub fn quadtree_mean(pix: &Pix, nlevels: u32) -> RegionResult<QuadtreeResult> {
    let integral = IntegralImage::from_pix(pix)?;
    quadtree_mean_with_integral(pix, nlevels, &integral)
}

/// Compute quadtree mean values using a precomputed integral image
///
/// This is more efficient when computing multiple quadtrees for the same image.
///
/// # Arguments
///
/// * `pix` - Input 8-bit grayscale image
/// * `nlevels` - Number of quadtree levels
/// * `integral` - Precomputed integral image
pub fn quadtree_mean_with_integral(
    pix: &Pix,
    nlevels: u32,
    integral: &IntegralImage,
) -> RegionResult<QuadtreeResult> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit grayscale",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    let max_levels = quadtree_max_levels(width, height);
    if nlevels > max_levels + 1 {
        return Err(RegionError::InvalidParameters(format!(
            "nlevels {} exceeds maximum {} for {}x{} image",
            nlevels,
            max_levels + 1,
            width,
            height
        )));
    }

    let regions = quadtree_regions(width, height, nlevels)?;
    let mut levels = Vec::with_capacity(nlevels as usize);

    for level in 0..nlevels {
        let boxa = regions
            .get(level as usize)
            .ok_or_else(|| RegionError::InvalidParameters(format!("missing level {}", level)))?;

        let size = 1u32 << level;
        let mut fpix = FPix::new(size, size).map_err(RegionError::Core)?;

        for (j, b) in boxa.iter().enumerate() {
            let mean = mean_in_rectangle(b, integral)?;
            let x = (j as u32) % size;
            let y = (j as u32) / size;
            fpix.set_pixel(x, y, mean).map_err(RegionError::Core)?;
        }

        levels.push(fpix);
    }

    Ok(QuadtreeResult::new(levels))
}

/// Compute quadtree variance values for an image
///
/// Creates a hierarchical representation containing both variance
/// and root variance (standard deviation) at each level.
///
/// # Arguments
///
/// * `pix` - Input 8-bit grayscale image
/// * `nlevels` - Number of quadtree levels
///
/// # Returns
///
/// A tuple of (variance_result, root_variance_result).
///
/// # Errors
///
/// Returns an error if the image is not 8-bit grayscale or nlevels is invalid.
pub fn quadtree_variance(
    pix: &Pix,
    nlevels: u32,
) -> RegionResult<(QuadtreeResult, QuadtreeResult)> {
    let integral = IntegralImage::from_pix(pix)?;
    let sq_integral = SquaredIntegralImage::from_pix(pix)?;
    quadtree_variance_with_integral(pix, nlevels, &integral, &sq_integral)
}

/// Compute quadtree variance using precomputed integral images
///
/// This is more efficient when computing multiple statistics for the same image.
///
/// # Arguments
///
/// * `pix` - Input 8-bit grayscale image
/// * `nlevels` - Number of quadtree levels
/// * `integral` - Precomputed integral image
/// * `sq_integral` - Precomputed squared integral image
pub fn quadtree_variance_with_integral(
    pix: &Pix,
    nlevels: u32,
    integral: &IntegralImage,
    sq_integral: &SquaredIntegralImage,
) -> RegionResult<(QuadtreeResult, QuadtreeResult)> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit grayscale",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    let max_levels = quadtree_max_levels(width, height);
    if nlevels > max_levels + 1 {
        return Err(RegionError::InvalidParameters(format!(
            "nlevels {} exceeds maximum {} for {}x{} image",
            nlevels,
            max_levels + 1,
            width,
            height
        )));
    }

    let regions = quadtree_regions(width, height, nlevels)?;
    let mut var_levels = Vec::with_capacity(nlevels as usize);
    let mut rvar_levels = Vec::with_capacity(nlevels as usize);

    for level in 0..nlevels {
        let boxa = regions
            .get(level as usize)
            .ok_or_else(|| RegionError::InvalidParameters(format!("missing level {}", level)))?;

        let size = 1u32 << level;
        let mut fpix_var = FPix::new(size, size).map_err(RegionError::Core)?;
        let mut fpix_rvar = FPix::new(size, size).map_err(RegionError::Core)?;

        for (j, b) in boxa.iter().enumerate() {
            let (var, rvar) = variance_in_rectangle(b, integral, sq_integral)?;
            let x = (j as u32) % size;
            let y = (j as u32) / size;
            fpix_var.set_pixel(x, y, var).map_err(RegionError::Core)?;
            fpix_rvar.set_pixel(x, y, rvar).map_err(RegionError::Core)?;
        }

        var_levels.push(fpix_var);
        rvar_levels.push(fpix_rvar);
    }

    Ok((
        QuadtreeResult::new(var_levels),
        QuadtreeResult::new(rvar_levels),
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32, value: u8) -> Pix {
        let pix = Pix::new(width, height, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..height {
            for x in 0..width {
                let _ = pix_mut.set_pixel(x, y, value as u32);
            }
        }

        pix_mut.into()
    }

    fn create_gradient_image(width: u32, height: u32) -> Pix {
        let pix = Pix::new(width, height, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..height {
            for x in 0..width {
                // Simple gradient based on position
                let value = (x + y) % 256;
                let _ = pix_mut.set_pixel(x, y, value);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_quadtree_max_levels() {
        // C version returns i-1 where i is the first index where minside < 1.5 * 2^i
        // 1x1: 1 < 1.5*1 = 1.5, so i=0, returns -1 -> we use 0
        assert_eq!(quadtree_max_levels(1, 1), 0);

        // 2x2: 2 >= 1.5, 2 < 3, so i=1, returns 0
        assert_eq!(quadtree_max_levels(2, 2), 0);

        // 4x4: 4 >= 1.5, 4 >= 3, 4 < 6, so i=2, returns 1
        assert_eq!(quadtree_max_levels(4, 4), 1);

        // 8x8: 8 >= 6, 8 < 12, so i=3, returns 2
        assert_eq!(quadtree_max_levels(8, 8), 2);

        // 16x16: 16 >= 12, 16 < 24, so i=4, returns 3
        assert_eq!(quadtree_max_levels(16, 16), 3);

        // Non-square images use minimum dimension
        assert_eq!(quadtree_max_levels(16, 4), 1);
        assert_eq!(quadtree_max_levels(4, 16), 1);

        // Zero dimensions
        assert_eq!(quadtree_max_levels(0, 10), 0);
        assert_eq!(quadtree_max_levels(10, 0), 0);
    }

    #[test]
    fn test_quadtree_regions() {
        let baa = quadtree_regions(8, 8, 3).unwrap();

        // Level 0: 1 box
        assert_eq!(baa.get(0).unwrap().len(), 1);
        let b0 = baa.get(0).unwrap().get(0).unwrap();
        assert_eq!((b0.x, b0.y, b0.w, b0.h), (0, 0, 8, 8));

        // Level 1: 4 boxes
        assert_eq!(baa.get(1).unwrap().len(), 4);

        // Level 2: 16 boxes
        assert_eq!(baa.get(2).unwrap().len(), 16);
    }

    #[test]
    fn test_quadtree_regions_error() {
        // nlevels = 0
        assert!(quadtree_regions(8, 8, 0).is_err());

        // Too many levels
        assert!(quadtree_regions(4, 4, 10).is_err());
    }

    #[test]
    fn test_integral_image_uniform() {
        let pix = create_test_image(4, 4, 100);
        let integral = IntegralImage::from_pix(&pix).unwrap();

        // Single pixel
        assert_eq!(integral.sum_rect(0, 0, 1, 1), Some(100));

        // 2x2 region
        assert_eq!(integral.sum_rect(0, 0, 2, 2), Some(400));

        // Entire image
        assert_eq!(integral.sum_rect(0, 0, 4, 4), Some(1600));

        // Region not at origin
        assert_eq!(integral.sum_rect(1, 1, 2, 2), Some(400));
    }

    #[test]
    fn test_integral_image_gradient() {
        // Create a simple 2x2 image with known values
        let pix = Pix::new(2, 2, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel(0, 0, 1).unwrap(); // top-left
        pix_mut.set_pixel(1, 0, 2).unwrap(); // top-right
        pix_mut.set_pixel(0, 1, 3).unwrap(); // bottom-left
        pix_mut.set_pixel(1, 1, 4).unwrap(); // bottom-right
        let pix: Pix = pix_mut.into();

        let integral = IntegralImage::from_pix(&pix).unwrap();

        // Check integral values
        assert_eq!(integral.get(0, 0), Some(1));
        assert_eq!(integral.get(1, 0), Some(3)); // 1+2
        assert_eq!(integral.get(0, 1), Some(4)); // 1+3
        assert_eq!(integral.get(1, 1), Some(10)); // 1+2+3+4

        // Check sum_rect
        assert_eq!(integral.sum_rect(0, 0, 2, 2), Some(10));
        assert_eq!(integral.sum_rect(1, 1, 1, 1), Some(4));
    }

    #[test]
    fn test_mean_in_rectangle() {
        let pix = create_test_image(8, 8, 100);
        let integral = IntegralImage::from_pix(&pix).unwrap();

        let rect = Box::new_unchecked(0, 0, 8, 8);
        let mean = mean_in_rectangle(&rect, &integral).unwrap();
        assert!((mean - 100.0).abs() < 0.001);

        let rect2 = Box::new_unchecked(2, 2, 4, 4);
        let mean2 = mean_in_rectangle(&rect2, &integral).unwrap();
        assert!((mean2 - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_variance_uniform_image() {
        // Uniform image should have variance = 0
        let pix = create_test_image(8, 8, 100);
        let integral = IntegralImage::from_pix(&pix).unwrap();
        let sq_integral = SquaredIntegralImage::from_pix(&pix).unwrap();

        let rect = Box::new_unchecked(0, 0, 8, 8);
        let (var, rvar) = variance_in_rectangle(&rect, &integral, &sq_integral).unwrap();

        assert!(var.abs() < 0.001);
        assert!(rvar.abs() < 0.001);
    }

    #[test]
    fn test_variance_known_values() {
        // Create image with known variance
        let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Values: 0, 0, 10, 10 -> mean = 5, variance = 25
        pix_mut.set_pixel(0, 0, 0).unwrap();
        pix_mut.set_pixel(1, 0, 0).unwrap();
        pix_mut.set_pixel(2, 0, 10).unwrap();
        pix_mut.set_pixel(3, 0, 10).unwrap();
        let pix: Pix = pix_mut.into();

        let integral = IntegralImage::from_pix(&pix).unwrap();
        let sq_integral = SquaredIntegralImage::from_pix(&pix).unwrap();

        let rect = Box::new_unchecked(0, 0, 4, 1);
        let (var, rvar) = variance_in_rectangle(&rect, &integral, &sq_integral).unwrap();

        // Mean = 5, E[X^2] = (0+0+100+100)/4 = 50, Var = 50 - 25 = 25
        assert!((var - 25.0).abs() < 0.001);
        assert!((rvar - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_quadtree_mean_uniform() {
        let pix = create_test_image(8, 8, 128);
        let result = quadtree_mean(&pix, 3).unwrap();

        assert_eq!(result.num_levels(), 3);

        // All values should be 128
        for level in 0..3 {
            let size = 1u32 << level;
            for y in 0..size {
                for x in 0..size {
                    let val = result.get_value(level, x, y).unwrap();
                    assert!((val - 128.0).abs() < 0.001);
                }
            }
        }
    }

    #[test]
    fn test_quadtree_variance_uniform() {
        let pix = create_test_image(8, 8, 100);
        let (var_result, rvar_result) = quadtree_variance(&pix, 3).unwrap();

        // All variances should be 0 for uniform image
        for level in 0..3 {
            let size = 1u32 << level;
            for y in 0..size {
                for x in 0..size {
                    let var = var_result.get_value(level, x, y).unwrap();
                    let rvar = rvar_result.get_value(level, x, y).unwrap();
                    assert!(var.abs() < 0.001);
                    assert!(rvar.abs() < 0.001);
                }
            }
        }
    }

    #[test]
    fn test_quadtree_get_parent() {
        let pix = create_gradient_image(8, 8);
        let result = quadtree_mean(&pix, 3).unwrap();

        // Level 0 has no parent
        assert!(result.get_parent(0, 0, 0).is_none());

        // Children at level 2, position (0,0), (1,0), (0,1), (1,1) should have parent at level 1, (0,0)
        let parent = result.get_parent(2, 0, 0);
        assert!(parent.is_some());

        let parent2 = result.get_parent(2, 1, 1);
        assert!(parent2.is_some());
        assert_eq!(parent, parent2); // Same parent
    }

    #[test]
    fn test_quadtree_get_children() {
        let pix = create_gradient_image(8, 8);
        let result = quadtree_mean(&pix, 3).unwrap();

        // Level 2 has no children
        assert!(result.get_children(2, 0, 0).is_none());

        // Level 0 should have 4 children at level 1
        let children = result.get_children(0, 0, 0);
        assert!(children.is_some());
        let children = children.unwrap();

        // Verify children match level 1 values
        assert_eq!(children[0], result.get_value(1, 0, 0).unwrap());
        assert_eq!(children[1], result.get_value(1, 1, 0).unwrap());
        assert_eq!(children[2], result.get_value(1, 0, 1).unwrap());
        assert_eq!(children[3], result.get_value(1, 1, 1).unwrap());
    }

    #[test]
    fn test_unsupported_depth() {
        let pix = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
        assert!(quadtree_mean(&pix, 2).is_err());
        assert!(quadtree_variance(&pix, 2).is_err());
    }

    #[test]
    fn test_quadtree_result_get_level() {
        let pix = create_test_image(8, 8, 100);
        let result = quadtree_mean(&pix, 3).unwrap();

        let level0 = result.get_level(0).unwrap();
        assert_eq!(level0.width(), 1);
        assert_eq!(level0.height(), 1);

        let level1 = result.get_level(1).unwrap();
        assert_eq!(level1.width(), 2);
        assert_eq!(level1.height(), 2);

        let level2 = result.get_level(2).unwrap();
        assert_eq!(level2.width(), 4);
        assert_eq!(level2.height(), 4);

        assert!(result.get_level(3).is_none());
    }

    #[test]
    fn test_integral_image_boundary_cases() {
        let pix = create_test_image(1, 1, 42);
        let integral = IntegralImage::from_pix(&pix).unwrap();

        assert_eq!(integral.sum_rect(0, 0, 1, 1), Some(42));
        assert_eq!(integral.sum_rect(0, 0, 0, 0), Some(0));
        assert!(integral.sum_rect(1, 0, 1, 1).is_none());
    }
}
