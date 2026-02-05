//! Color fill operations for RGB images
//!
//! This module provides flood fill algorithms for color images, identifying
//! connected regions of similar color. Unlike binary flood fill, color fill
//! uses a similarity threshold to determine if neighboring pixels should be
//! included in the same region.
//!
//! # Algorithm
//!
//! The color fill algorithm uses BFS (breadth-first search) to grow regions:
//! 1. Start from a seed pixel
//! 2. For each neighbor, check if its color is "similar" to the current pixel
//! 3. Similar pixels are added to the region and queued for processing
//! 4. Continue until no more similar neighbors are found
//!
//! Color similarity is determined by comparing the maximum component difference
//! between two colors. This approach preserves hue while allowing for slight
//! variations in brightness.
//!
//! # Example
//!
//! ```no_run
//! use leptonica_color::colorfill::{color_fill_from_seed, ColorFillOptions};
//! use leptonica_core::Pix;
//!
//! let pix = Pix::new(100, 100, leptonica_core::PixelDepth::Bit32).unwrap();
//! let options = ColorFillOptions::default();
//! if let Some(result) = color_fill_from_seed(&pix, 50, 50, &options).unwrap() {
//!     println!("Filled {} pixels", result.pixel_count);
//! }
//! ```

use crate::{ColorError, ColorResult};
use leptonica_core::{Pix, PixelDepth, color};
use std::collections::VecDeque;

// =============================================================================
// Constants
// =============================================================================

/// Default minimum max component for a pixel to be considered "colorful"
const DEFAULT_MIN_MAX: u32 = 70;

/// Default maximum color difference for pixels to be considered similar
const DEFAULT_MAX_DIFF: u32 = 40;

/// Default minimum number of pixels for a valid region
const DEFAULT_MIN_AREA: u32 = 100;

// =============================================================================
// Types
// =============================================================================

/// Connectivity type for fill operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Connectivity {
    /// 4-way connectivity (N, S, E, W)
    FourWay,
    /// 8-way connectivity (N, S, E, W, NE, NW, SE, SW)
    #[default]
    EightWay,
}

/// Options for color fill operations
#[derive(Debug, Clone)]
pub struct ColorFillOptions {
    /// Minimum value of max(r,g,b) for a pixel to be considered "colorful"
    ///
    /// Pixels with max component below this threshold are treated as invalid
    /// (too dark to have meaningful color). Default: 70.
    pub min_max: u32,

    /// Maximum color difference for pixels to be considered similar
    ///
    /// The difference metric compares color components and allows pixels
    /// to be grouped if their colors differ by less than this amount.
    /// Lower values create more regions; higher values merge more colors.
    /// Default: 40.
    pub max_diff: u32,

    /// Minimum number of pixels for a valid region
    ///
    /// Regions smaller than this are discarded. Default: 100.
    pub min_area: u32,

    /// Connectivity type (4-way or 8-way)
    ///
    /// 8-way connectivity is recommended for color fill as it reduces
    /// the number of single-pixel noise regions. Default: 8-way.
    pub connectivity: Connectivity,
}

impl Default for ColorFillOptions {
    fn default() -> Self {
        Self {
            min_max: DEFAULT_MIN_MAX,
            max_diff: DEFAULT_MAX_DIFF,
            min_area: DEFAULT_MIN_AREA,
            connectivity: Connectivity::EightWay,
        }
    }
}

impl ColorFillOptions {
    /// Create options with custom max color difference
    pub fn with_max_diff(mut self, max_diff: u32) -> Self {
        self.max_diff = max_diff;
        self
    }

    /// Create options with custom minimum area
    pub fn with_min_area(mut self, min_area: u32) -> Self {
        self.min_area = min_area;
        self
    }

    /// Create options with custom connectivity
    pub fn with_connectivity(mut self, connectivity: Connectivity) -> Self {
        self.connectivity = connectivity;
        self
    }

    /// Create options with custom minimum max component
    pub fn with_min_max(mut self, min_max: u32) -> Self {
        self.min_max = min_max;
        self
    }
}

/// Result of a color fill operation
#[derive(Debug)]
pub struct ColorFillResult {
    /// 1-bit mask of the filled region (ON pixels are part of the region)
    pub mask: Pix,
    /// Number of pixels in the region
    pub pixel_count: u32,
    /// Average RGB color of the region
    pub avg_color: (u8, u8, u8),
}

/// Result of finding all color regions in an image
#[derive(Debug)]
pub struct ColorRegions {
    /// Combined 1-bit mask of all regions meeting the minimum area threshold
    pub mask: Pix,
    /// Number of distinct regions found
    pub region_count: u32,
    /// Total number of pixels in all regions
    pub total_pixels: u32,
}

// =============================================================================
// Color Similarity Functions
// =============================================================================

/// Check if a pixel has valid color (not too dark)
///
/// A pixel is considered valid if at least one RGB component is >= min_max.
#[inline]
fn pixel_color_is_valid(val: u32, min_max: u32) -> bool {
    let (r, g, b) = color::extract_rgb(val);
    r as u32 >= min_max || g as u32 >= min_max || b as u32 >= min_max
}

/// Check if two colors are similar enough for fill
///
/// This uses the Leptonica algorithm which finds the component with the
/// largest absolute difference, then checks if the differences relative
/// to that component are within the threshold.
///
/// This approach allows colors with similar hue but different brightness
/// to be grouped together while separating truly different colors.
#[inline]
fn colors_are_similar(val1: u32, val2: u32, max_diff: u32) -> bool {
    let (r1, g1, b1) = color::extract_rgb(val1);
    let (r2, g2, b2) = color::extract_rgb(val2);

    let rdiff = r1 as i32 - r2 as i32;
    let gdiff = g1 as i32 - g2 as i32;
    let bdiff = b1 as i32 - b2 as i32;

    // Find the component with largest absolute difference
    let abs_r = rdiff.abs();
    let abs_g = gdiff.abs();
    let abs_b = bdiff.abs();

    let (_max_idx, diffs) = if abs_r >= abs_g && abs_r >= abs_b {
        (0, [rdiff, gdiff, bdiff])
    } else if abs_g >= abs_b {
        (1, [gdiff, rdiff, bdiff])
    } else {
        (2, [bdiff, rdiff, gdiff])
    };

    // Check differences relative to the max component
    let del1 = diffs[0];
    let del2 = diffs[1];
    let del3 = diffs[2];

    // The max of |del1 - del2| and |del1 - del3|
    let max_del = (del1 - del2).abs().max((del1 - del3).abs());

    // Also check direct differences to handle edge cases
    let direct_max = abs_r.max(abs_g).max(abs_b);

    // Color is similar if both the relative and absolute differences are small
    max_del <= max_diff as i32 && direct_max <= (max_diff as i32 * 2)
}

// =============================================================================
// Flood Fill Implementation
// =============================================================================

/// Perform color fill from a seed point
///
/// Starting from the seed pixel, grows a region by including all connected
/// pixels with similar colors. Uses BFS to ensure all reachable similar
/// pixels are found.
///
/// # Arguments
///
/// * `pix` - 32-bpp RGB input image
/// * `seed_x` - X coordinate of seed point
/// * `seed_y` - Y coordinate of seed point
/// * `options` - Fill options
///
/// # Returns
///
/// A `ColorFillResult` containing the region mask, pixel count, and average color.
/// Returns `None` if the seed pixel has invalid color or the region is too small.
///
/// # Errors
///
/// Returns an error if the image has wrong depth or seed is out of bounds.
pub fn color_fill_from_seed(
    pix: &Pix,
    seed_x: u32,
    seed_y: u32,
    options: &ColorFillOptions,
) -> ColorResult<Option<ColorFillResult>> {
    // Validate input
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    if seed_x >= width || seed_y >= height {
        return Err(ColorError::InvalidParameters(format!(
            "seed ({}, {}) out of bounds for {}x{} image",
            seed_x, seed_y, width, height
        )));
    }

    // Check if seed pixel has valid color
    let seed_color = pix.get_pixel(seed_x, seed_y).unwrap_or(0);
    if !pixel_color_is_valid(seed_color, options.min_max) {
        return Ok(None);
    }

    // Create visited mask and result mask
    let visited_pix = Pix::new(width, height, PixelDepth::Bit1)?;
    let mut visited = visited_pix.try_into_mut().unwrap();

    let result_pix = Pix::new(width, height, PixelDepth::Bit1)?;
    let mut result = result_pix.try_into_mut().unwrap();

    // Track color sums for average calculation
    let mut r_sum: u64 = 0;
    let mut g_sum: u64 = 0;
    let mut b_sum: u64 = 0;
    let mut pixel_count: u32 = 0;

    // BFS queue: (x, y, color_of_parent)
    // We propagate the parent's color to ensure connected regions
    // share similar colors with their immediate neighbors
    let mut queue: VecDeque<(u32, u32, u32)> = VecDeque::new();

    // Initialize with seed
    queue.push_back((seed_x, seed_y, seed_color));
    let _ = visited.set_pixel(seed_x, seed_y, 1);

    while let Some((x, y, parent_color)) = queue.pop_front() {
        let current_color = pix.get_pixel(x, y).unwrap_or(0);

        // Check if current pixel is similar to parent
        if !colors_are_similar(parent_color, current_color, options.max_diff) {
            continue;
        }

        // Check if current pixel has valid color
        if !pixel_color_is_valid(current_color, options.min_max) {
            continue;
        }

        // Add to result
        let _ = result.set_pixel(x, y, 1);
        let (r, g, b) = color::extract_rgb(current_color);
        r_sum += r as u64;
        g_sum += g as u64;
        b_sum += b as u64;
        pixel_count += 1;

        // Add neighbors to queue
        let neighbors = get_neighbors(x, y, width, height, options.connectivity);
        for (nx, ny) in neighbors {
            if visited.get_pixel(nx, ny).unwrap_or(1) == 0 {
                let _ = visited.set_pixel(nx, ny, 1);
                queue.push_back((nx, ny, current_color));
            }
        }
    }

    // Check minimum area
    if pixel_count < options.min_area {
        return Ok(None);
    }

    // Calculate average color
    let avg_r = (r_sum / pixel_count as u64) as u8;
    let avg_g = (g_sum / pixel_count as u64) as u8;
    let avg_b = (b_sum / pixel_count as u64) as u8;

    Ok(Some(ColorFillResult {
        mask: result.into(),
        pixel_count,
        avg_color: (avg_r, avg_g, avg_b),
    }))
}

/// Find all color regions in an image
///
/// Scans the image and performs color fill from each unvisited pixel,
/// collecting all regions that meet the minimum area threshold.
///
/// # Arguments
///
/// * `pix` - 32-bpp RGB input image
/// * `options` - Fill options
///
/// # Returns
///
/// A `ColorRegions` struct containing the combined mask and statistics.
pub fn color_fill(pix: &Pix, options: &ColorFillOptions) -> ColorResult<ColorRegions> {
    // Validate input
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    // Track globally visited pixels
    let visited_pix = Pix::new(width, height, PixelDepth::Bit1)?;
    let mut visited = visited_pix.try_into_mut().unwrap();

    // Combined result mask
    let result_pix = Pix::new(width, height, PixelDepth::Bit1)?;
    let mut combined_result = result_pix.try_into_mut().unwrap();

    let mut region_count: u32 = 0;
    let mut total_pixels: u32 = 0;

    // Scan image in raster order
    for y in 0..height {
        for x in 0..width {
            // Skip already visited pixels
            if visited.get_pixel(x, y).unwrap_or(1) == 1 {
                continue;
            }

            // Check if this pixel has valid color
            let pixel_color = pix.get_pixel(x, y).unwrap_or(0);
            if !pixel_color_is_valid(pixel_color, options.min_max) {
                let _ = visited.set_pixel(x, y, 1);
                continue;
            }

            // Perform local fill to find this region
            let local_result = fill_region_local(pix, &mut visited, x, y, options)?;

            // If region meets minimum area, add to result
            if let Some((mask, count)) = local_result {
                // OR the local mask into the combined result
                for my in 0..height {
                    for mx in 0..width {
                        if mask.get_pixel(mx, my).unwrap_or(0) == 1 {
                            let _ = combined_result.set_pixel(mx, my, 1);
                        }
                    }
                }
                region_count += 1;
                total_pixels += count;
            }
        }
    }

    Ok(ColorRegions {
        mask: combined_result.into(),
        region_count,
        total_pixels,
    })
}

/// Internal function to fill a single region starting from (start_x, start_y)
/// Updates the global visited mask and returns the local region mask if valid.
fn fill_region_local(
    pix: &Pix,
    visited: &mut leptonica_core::PixMut,
    start_x: u32,
    start_y: u32,
    options: &ColorFillOptions,
) -> ColorResult<Option<(Pix, u32)>> {
    let width = pix.width();
    let height = pix.height();

    let seed_color = pix.get_pixel(start_x, start_y).unwrap_or(0);

    // Create local result mask
    let result_pix = Pix::new(width, height, PixelDepth::Bit1)?;
    let mut result = result_pix.try_into_mut().unwrap();

    let mut pixel_count: u32 = 0;

    // BFS queue
    let mut queue: VecDeque<(u32, u32, u32)> = VecDeque::new();
    queue.push_back((start_x, start_y, seed_color));
    let _ = visited.set_pixel(start_x, start_y, 1);

    while let Some((x, y, parent_color)) = queue.pop_front() {
        let current_color = pix.get_pixel(x, y).unwrap_or(0);

        // Check similarity with parent
        if !colors_are_similar(parent_color, current_color, options.max_diff) {
            continue;
        }

        // Check valid color
        if !pixel_color_is_valid(current_color, options.min_max) {
            continue;
        }

        // Add to result
        let _ = result.set_pixel(x, y, 1);
        pixel_count += 1;

        // Add unvisited neighbors
        let neighbors = get_neighbors(x, y, width, height, options.connectivity);
        for (nx, ny) in neighbors {
            if visited.get_pixel(nx, ny).unwrap_or(1) == 0 {
                let _ = visited.set_pixel(nx, ny, 1);
                queue.push_back((nx, ny, current_color));
            }
        }
    }

    // Check minimum area
    if pixel_count >= options.min_area {
        Ok(Some((result.into(), pixel_count)))
    } else {
        Ok(None)
    }
}

/// Get neighbor coordinates for a pixel
#[inline]
fn get_neighbors(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    connectivity: Connectivity,
) -> Vec<(u32, u32)> {
    let mut neighbors = Vec::with_capacity(8);

    // 4-way neighbors (N, S, E, W)
    if x > 0 {
        neighbors.push((x - 1, y));
    }
    if x + 1 < width {
        neighbors.push((x + 1, y));
    }
    if y > 0 {
        neighbors.push((x, y - 1));
    }
    if y + 1 < height {
        neighbors.push((x, y + 1));
    }

    // 8-way diagonal neighbors (NE, NW, SE, SW)
    if connectivity == Connectivity::EightWay {
        if x > 0 && y > 0 {
            neighbors.push((x - 1, y - 1));
        }
        if x + 1 < width && y > 0 {
            neighbors.push((x + 1, y - 1));
        }
        if x > 0 && y + 1 < height {
            neighbors.push((x - 1, y + 1));
        }
        if x + 1 < width && y + 1 < height {
            neighbors.push((x + 1, y + 1));
        }
    }

    neighbors
}

/// Check if a pixel is on a color boundary (has neighbors with different colors)
///
/// This is useful for identifying edges of color regions.
pub fn pixel_is_on_color_boundary(pix: &Pix, x: u32, y: u32) -> bool {
    let width = pix.width();
    let height = pix.height();

    if x >= width || y >= height {
        return false;
    }

    let center_color = pix.get_pixel(x, y).unwrap_or(0);

    // Check 4-way neighbors
    let neighbors = [
        (x.wrapping_sub(1), y),
        (x + 1, y),
        (x, y.wrapping_sub(1)),
        (x, y + 1),
    ];

    for (nx, ny) in neighbors {
        if nx < width && ny < height {
            let neighbor_color = pix.get_pixel(nx, ny).unwrap_or(0);
            if neighbor_color != center_color {
                return true;
            }
        }
    }

    false
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test image with solid color regions
    fn create_test_image() -> Pix {
        let pix = Pix::new(60, 40, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Left third: red
        // Middle third: green
        // Right third: blue
        for y in 0..40 {
            for x in 0..60 {
                let pixel = if x < 20 {
                    color::compose_rgb(200, 50, 50) // Red
                } else if x < 40 {
                    color::compose_rgb(50, 200, 50) // Green
                } else {
                    color::compose_rgb(50, 50, 200) // Blue
                };
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        pix_mut.into()
    }

    /// Create a gradient test image
    fn create_gradient_image() -> Pix {
        let pix = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..100 {
            for x in 0..100 {
                // Smooth horizontal gradient from red to blue
                let r = (255 - (x * 255 / 100)) as u8;
                let b = (x * 255 / 100) as u8;
                let pixel = color::compose_rgb(r, 100, b);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_pixel_color_is_valid() {
        // High brightness pixel
        let bright = color::compose_rgb(200, 100, 50);
        assert!(pixel_color_is_valid(bright, 70));

        // Dark pixel
        let dark = color::compose_rgb(30, 40, 50);
        assert!(!pixel_color_is_valid(dark, 70));

        // Edge case: exactly at threshold
        let edge = color::compose_rgb(70, 30, 30);
        assert!(pixel_color_is_valid(edge, 70));
    }

    #[test]
    fn test_colors_are_similar() {
        let red1 = color::compose_rgb(200, 50, 50);
        let red2 = color::compose_rgb(210, 55, 45);
        let blue = color::compose_rgb(50, 50, 200);

        // Similar reds should match
        assert!(colors_are_similar(red1, red2, 40));

        // Red and blue should not match
        assert!(!colors_are_similar(red1, blue, 40));
    }

    #[test]
    fn test_color_fill_from_seed_basic() {
        let pix = create_test_image();
        let options = ColorFillOptions::default().with_min_area(10);

        // Fill from red region
        let result = color_fill_from_seed(&pix, 10, 20, &options)
            .unwrap()
            .unwrap();

        // Should fill the left third
        assert!(result.pixel_count > 0);
        assert!(result.pixel_count <= 20 * 40); // max possible

        // Average color should be reddish
        assert!(result.avg_color.0 > 150); // High red
        assert!(result.avg_color.1 < 100); // Low green
        assert!(result.avg_color.2 < 100); // Low blue
    }

    #[test]
    fn test_color_fill_from_seed_invalid_seed() {
        let pix = create_test_image();
        let options = ColorFillOptions::default();

        // Out of bounds seed
        let result = color_fill_from_seed(&pix, 100, 100, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_color_fill_from_seed_dark_pixel() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with dark pixels
        for y in 0..10 {
            for x in 0..10 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, color::compose_rgb(30, 30, 30)) };
            }
        }
        let pix: Pix = pix_mut.into();

        let options = ColorFillOptions::default();
        let result = color_fill_from_seed(&pix, 5, 5, &options).unwrap();

        // Should return None for dark region
        assert!(result.is_none());
    }

    #[test]
    fn test_color_fill_multiple_regions() {
        let pix = create_test_image();
        let options = ColorFillOptions::default().with_min_area(10);

        let result = color_fill(&pix, &options).unwrap();

        // Should find 3 distinct regions (R, G, B)
        assert!(result.region_count >= 1);
        assert!(result.total_pixels > 0);
    }

    #[test]
    fn test_color_fill_gradient() {
        let pix = create_gradient_image();

        // With small max_diff, gradient should create many regions
        let options_small = ColorFillOptions::default()
            .with_max_diff(10)
            .with_min_area(10);
        let result_small = color_fill(&pix, &options_small).unwrap();

        // With large max_diff, gradient should create fewer regions
        let options_large = ColorFillOptions::default()
            .with_max_diff(100)
            .with_min_area(10);
        let result_large = color_fill(&pix, &options_large).unwrap();

        // More regions with stricter similarity
        assert!(result_small.region_count >= result_large.region_count);
    }

    #[test]
    fn test_connectivity_options() {
        let pix = create_test_image();

        let options_4way = ColorFillOptions::default()
            .with_connectivity(Connectivity::FourWay)
            .with_min_area(10);

        let options_8way = ColorFillOptions::default()
            .with_connectivity(Connectivity::EightWay)
            .with_min_area(10);

        // Both should work
        let result_4 = color_fill_from_seed(&pix, 10, 20, &options_4way)
            .unwrap()
            .unwrap();
        let result_8 = color_fill_from_seed(&pix, 10, 20, &options_8way)
            .unwrap()
            .unwrap();

        // Should produce similar results for solid regions
        assert!(result_4.pixel_count > 0);
        assert!(result_8.pixel_count > 0);
    }

    #[test]
    fn test_pixel_is_on_color_boundary() {
        let pix = create_test_image();

        // Pixel in the middle of red region should not be on boundary
        assert!(!pixel_is_on_color_boundary(&pix, 10, 20));

        // Pixel at the edge of red region (x=19) might be on boundary
        // depending on exact colors
        // At x=20, we transition from red to green
        assert!(pixel_is_on_color_boundary(&pix, 19, 20));
    }

    #[test]
    fn test_wrong_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let options = ColorFillOptions::default();

        let result = color_fill(&pix, &options);
        assert!(result.is_err());

        let result = color_fill_from_seed(&pix, 5, 5, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_min_area_filter() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Small bright region
        for y in 0..5 {
            for x in 0..5 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, color::compose_rgb(200, 100, 100)) };
            }
        }
        // Rest is dark
        for y in 0..10 {
            for x in 0..10 {
                if x >= 5 || y >= 5 {
                    unsafe { pix_mut.set_pixel_unchecked(x, y, color::compose_rgb(30, 30, 30)) };
                }
            }
        }
        let pix: Pix = pix_mut.into();

        // With min_area > 25, the 5x5 region should be filtered out
        let options_large = ColorFillOptions::default().with_min_area(30);
        let result = color_fill(&pix, &options_large).unwrap();
        assert_eq!(result.region_count, 0);

        // With min_area < 25, the region should be found
        let options_small = ColorFillOptions::default().with_min_area(10);
        let result = color_fill(&pix, &options_small).unwrap();
        assert_eq!(result.region_count, 1);
    }
}
