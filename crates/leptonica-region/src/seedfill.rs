//! Seed fill operations
//!
//! This module provides flood fill and seed fill algorithms for binary
//! and grayscale images. These are useful for region filling, hole filling,
//! and morphological reconstruction.

use crate::conncomp::ConnectivityType;
use crate::error::{RegionError, RegionResult};
use leptonica_core::{Pix, PixMut, PixelDepth};
use std::collections::VecDeque;

/// Options for seed fill operations
#[derive(Debug, Clone)]
pub struct SeedFillOptions {
    /// Connectivity type (4-way or 8-way)
    pub connectivity: ConnectivityType,
    /// Fill value for binary images
    pub fill_value: u32,
}

impl Default for SeedFillOptions {
    fn default() -> Self {
        Self {
            connectivity: ConnectivityType::FourWay,
            fill_value: 1,
        }
    }
}

impl SeedFillOptions {
    /// Create new options with the specified connectivity
    pub fn new(connectivity: ConnectivityType) -> Self {
        Self {
            connectivity,
            fill_value: 1,
        }
    }

    /// Set the fill value
    pub fn with_fill_value(mut self, value: u32) -> Self {
        self.fill_value = value;
        self
    }
}

/// Flood fill in a binary image starting from a seed point
///
/// Fills connected regions of the same value starting from the seed point.
/// This modifies the image in place and returns the number of pixels filled.
///
/// # Arguments
///
/// * `pix` - Mutable binary image (1-bit)
/// * `seed_x` - X coordinate of the seed point
/// * `seed_y` - Y coordinate of the seed point
/// * `new_value` - Value to fill with (0 or 1)
/// * `connectivity` - Connectivity type
///
/// # Returns
///
/// The number of pixels that were filled.
///
/// # Errors
///
/// Returns an error if the seed position is out of bounds or the image has
/// an unsupported depth.
pub fn floodfill(
    pix: &mut PixMut,
    seed_x: u32,
    seed_y: u32,
    new_value: u32,
    connectivity: ConnectivityType,
) -> RegionResult<u32> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    if seed_x >= width || seed_y >= height {
        return Err(RegionError::InvalidSeed {
            x: seed_x,
            y: seed_y,
        });
    }

    let old_value = pix.get_pixel(seed_x, seed_y).unwrap_or(0);
    let new_value = new_value & 1; // Clamp to 0 or 1

    // If the values are the same, nothing to do
    if old_value == new_value {
        return Ok(0);
    }

    let mut filled_count = 0u32;
    let mut queue = VecDeque::new();
    queue.push_back((seed_x, seed_y));

    while let Some((x, y)) = queue.pop_front() {
        // Check bounds and current value
        if x >= width || y >= height {
            continue;
        }

        if let Some(current) = pix.get_pixel(x, y) {
            if current != old_value {
                continue;
            }

            // Fill this pixel
            let _ = pix.set_pixel(x, y, new_value);
            filled_count += 1;

            // Add 4-way neighbors
            if x > 0 {
                queue.push_back((x - 1, y));
            }
            if x + 1 < width {
                queue.push_back((x + 1, y));
            }
            if y > 0 {
                queue.push_back((x, y - 1));
            }
            if y + 1 < height {
                queue.push_back((x, y + 1));
            }

            // Add diagonal neighbors for 8-way connectivity
            if connectivity == ConnectivityType::EightWay {
                if x > 0 && y > 0 {
                    queue.push_back((x - 1, y - 1));
                }
                if x + 1 < width && y > 0 {
                    queue.push_back((x + 1, y - 1));
                }
                if x > 0 && y + 1 < height {
                    queue.push_back((x - 1, y + 1));
                }
                if x + 1 < width && y + 1 < height {
                    queue.push_back((x + 1, y + 1));
                }
            }
        }
    }

    Ok(filled_count)
}

/// Seed fill for binary images
///
/// Creates a new image by flood filling from the seed point in a copy of the input.
///
/// # Arguments
///
/// * `pix` - Input binary image (1-bit)
/// * `seed_x` - X coordinate of the seed point
/// * `seed_y` - Y coordinate of the seed point
/// * `options` - Seed fill options
///
/// # Returns
///
/// A new binary image with the filled region.
pub fn seedfill_binary(
    pix: &Pix,
    seed_x: u32,
    seed_y: u32,
    options: &SeedFillOptions,
) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    if seed_x >= width || seed_y >= height {
        return Err(RegionError::InvalidSeed {
            x: seed_x,
            y: seed_y,
        });
    }

    let mut output = pix.to_mut();
    floodfill(
        &mut output,
        seed_x,
        seed_y,
        options.fill_value,
        options.connectivity,
    )?;

    Ok(output.into())
}

/// Seed fill for grayscale images (morphological reconstruction)
///
/// Performs grayscale morphological reconstruction where the seed image
/// is reconstructed under the mask image.
///
/// # Arguments
///
/// * `seed` - Seed image (8-bit grayscale)
/// * `mask` - Mask image (8-bit grayscale)
/// * `connectivity` - Connectivity type
///
/// # Returns
///
/// The reconstructed image.
pub fn seedfill_gray(seed: &Pix, mask: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    if seed.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: seed.depth().bits(),
        });
    }

    if mask.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: mask.depth().bits(),
        });
    }

    let width = seed.width();
    let height = seed.height();

    if mask.width() != width || mask.height() != height {
        return Err(RegionError::InvalidParameters(
            "seed and mask must have the same dimensions".to_string(),
        ));
    }

    // Initialize output with seed values clamped to mask
    let mut output = Pix::new(width, height, PixelDepth::Bit8)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    for y in 0..height {
        for x in 0..width {
            let seed_val = seed.get_pixel(x, y).unwrap_or(0);
            let mask_val = mask.get_pixel(x, y).unwrap_or(0);
            let _ = output.set_pixel(x, y, seed_val.min(mask_val));
        }
    }

    // Iterative reconstruction using queue-based propagation
    let mut changed = true;
    let mut iterations = 0;
    const MAX_ITERATIONS: u32 = 10000;

    while changed && iterations < MAX_ITERATIONS {
        changed = false;
        iterations += 1;

        // Forward pass (top-left to bottom-right)
        for y in 0..height {
            for x in 0..width {
                let current = output.get_pixel(x, y).unwrap_or(0);
                let mask_val = mask.get_pixel(x, y).unwrap_or(0);
                let mut max_neighbor = current;

                // Check neighbors already processed
                if x > 0 {
                    max_neighbor = max_neighbor.max(output.get_pixel(x - 1, y).unwrap_or(0));
                }
                if y > 0 {
                    max_neighbor = max_neighbor.max(output.get_pixel(x, y - 1).unwrap_or(0));
                }
                if connectivity == ConnectivityType::EightWay {
                    if x > 0 && y > 0 {
                        max_neighbor =
                            max_neighbor.max(output.get_pixel(x - 1, y - 1).unwrap_or(0));
                    }
                    if x + 1 < width && y > 0 {
                        max_neighbor =
                            max_neighbor.max(output.get_pixel(x + 1, y - 1).unwrap_or(0));
                    }
                }

                let new_val = max_neighbor.min(mask_val);
                if new_val > current {
                    let _ = output.set_pixel(x, y, new_val);
                    changed = true;
                }
            }
        }

        // Backward pass (bottom-right to top-left)
        for y in (0..height).rev() {
            for x in (0..width).rev() {
                let current = output.get_pixel(x, y).unwrap_or(0);
                let mask_val = mask.get_pixel(x, y).unwrap_or(0);
                let mut max_neighbor = current;

                // Check neighbors already processed
                if x + 1 < width {
                    max_neighbor = max_neighbor.max(output.get_pixel(x + 1, y).unwrap_or(0));
                }
                if y + 1 < height {
                    max_neighbor = max_neighbor.max(output.get_pixel(x, y + 1).unwrap_or(0));
                }
                if connectivity == ConnectivityType::EightWay {
                    if x + 1 < width && y + 1 < height {
                        max_neighbor =
                            max_neighbor.max(output.get_pixel(x + 1, y + 1).unwrap_or(0));
                    }
                    if x > 0 && y + 1 < height {
                        max_neighbor =
                            max_neighbor.max(output.get_pixel(x - 1, y + 1).unwrap_or(0));
                    }
                }

                let new_val = max_neighbor.min(mask_val);
                if new_val > current {
                    let _ = output.set_pixel(x, y, new_val);
                    changed = true;
                }
            }
        }
    }

    Ok(output.into())
}

/// Fill holes in a binary image
///
/// Fills interior holes (regions of 0s completely surrounded by 1s).
///
/// # Arguments
///
/// * `pix` - Input binary image (1-bit)
/// * `connectivity` - Connectivity type
///
/// # Returns
///
/// A new image with holes filled.
pub fn fill_holes(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    // Create a marker image for background connected to border
    // Initialize with 0s, then mark border-connected background
    let mut background = Pix::new(width, height, PixelDepth::Bit1)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    // Use queue-based flood fill with mask
    // Fill background pixels (0 in input) connected to the border
    let mut queue = VecDeque::new();

    // Add border pixels that are background (0) in the original image
    for x in 0..width {
        if pix.get_pixel(x, 0).unwrap_or(1) == 0 && background.get_pixel(x, 0).unwrap_or(1) == 0 {
            let _ = background.set_pixel(x, 0, 1);
            queue.push_back((x, 0));
        }
        if pix.get_pixel(x, height - 1).unwrap_or(1) == 0
            && background.get_pixel(x, height - 1).unwrap_or(1) == 0
        {
            let _ = background.set_pixel(x, height - 1, 1);
            queue.push_back((x, height - 1));
        }
    }
    for y in 1..height - 1 {
        if pix.get_pixel(0, y).unwrap_or(1) == 0 && background.get_pixel(0, y).unwrap_or(1) == 0 {
            let _ = background.set_pixel(0, y, 1);
            queue.push_back((0, y));
        }
        if pix.get_pixel(width - 1, y).unwrap_or(1) == 0
            && background.get_pixel(width - 1, y).unwrap_or(1) == 0
        {
            let _ = background.set_pixel(width - 1, y, 1);
            queue.push_back((width - 1, y));
        }
    }

    // Propagate background marker using input as mask
    while let Some((x, y)) = queue.pop_front() {
        let neighbors: Vec<(u32, u32)> = {
            let mut n = Vec::with_capacity(8);
            if x > 0 {
                n.push((x - 1, y));
            }
            if x + 1 < width {
                n.push((x + 1, y));
            }
            if y > 0 {
                n.push((x, y - 1));
            }
            if y + 1 < height {
                n.push((x, y + 1));
            }
            if connectivity == ConnectivityType::EightWay {
                if x > 0 && y > 0 {
                    n.push((x - 1, y - 1));
                }
                if x + 1 < width && y > 0 {
                    n.push((x + 1, y - 1));
                }
                if x > 0 && y + 1 < height {
                    n.push((x - 1, y + 1));
                }
                if x + 1 < width && y + 1 < height {
                    n.push((x + 1, y + 1));
                }
            }
            n
        };

        for (nx, ny) in neighbors {
            // Only propagate to background pixels (0 in input) not yet marked
            if pix.get_pixel(nx, ny).unwrap_or(1) == 0
                && background.get_pixel(nx, ny).unwrap_or(1) == 0
            {
                let _ = background.set_pixel(nx, ny, 1);
                queue.push_back((nx, ny));
            }
        }
    }

    // Result = original OR (NOT background)
    // Pixels that are foreground OR not connected to border background
    let mut result = pix.to_mut();
    for y in 0..height {
        for x in 0..width {
            let orig = result.get_pixel(x, y).unwrap_or(0);
            let bg = background.get_pixel(x, y).unwrap_or(0);
            // Fill holes: keep original foreground, and fill background pixels
            // that are NOT connected to border (bg == 0 means interior hole)
            let _ = result.set_pixel(x, y, orig | (1 - bg));
        }
    }

    Ok(result.into())
}

/// Clear pixels connected to the border
///
/// Removes foreground regions that touch the image border.
///
/// # Arguments
///
/// * `pix` - Input binary image (1-bit)
/// * `connectivity` - Connectivity type
///
/// # Returns
///
/// A new image with border-connected regions removed.
pub fn clear_border(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    let mut result = pix.to_mut();

    // Clear all 1-pixels connected to the border
    // Top and bottom edges
    for x in 0..width {
        if result.get_pixel(x, 0).unwrap_or(0) == 1 {
            let _ = floodfill(&mut result, x, 0, 0, connectivity);
        }
        if result.get_pixel(x, height - 1).unwrap_or(0) == 1 {
            let _ = floodfill(&mut result, x, height - 1, 0, connectivity);
        }
    }

    // Left and right edges
    for y in 0..height {
        if result.get_pixel(0, y).unwrap_or(0) == 1 {
            let _ = floodfill(&mut result, 0, y, 0, connectivity);
        }
        if result.get_pixel(width - 1, y).unwrap_or(0) == 1 {
            let _ = floodfill(&mut result, width - 1, y, 0, connectivity);
        }
    }

    Ok(result.into())
}

/// Boundary condition for distance function computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryCondition {
    /// Treat boundary as background (distance clamps at edges).
    Background,
    /// Treat boundary as foreground (distance increases at edges).
    Foreground,
}

/// Compute the distance from each foreground pixel to the nearest
/// background pixel using a Chamfer distance transform.
///
/// Uses forward/backward raster scans for O(n) performance.
///
/// # Arguments
///
/// * `pix` - 1-bpp binary input image
/// * `connectivity` - 4 or 8-way connectivity
/// * `out_depth` - Output depth: [`PixelDepth::Bit8`] (max 254) or
///   [`PixelDepth::Bit16`] (max 65534)
/// * `boundary_cond` - How to handle pixels at image boundaries
///
/// # See also
///
/// C Leptonica: `pixDistanceFunction()` in `seedfill.c`
#[allow(unused_variables)]
pub fn distance_function(
    pix: &Pix,
    connectivity: ConnectivityType,
    out_depth: PixelDepth,
    boundary_cond: BoundaryCondition,
) -> RegionResult<Pix> {
    todo!()
}

/// Create a binary mask where two 8-bpp images have equal pixel values.
///
/// Returns a 1-bpp image with foreground pixels at locations where
/// `pix1` and `pix2` have the same value.
///
/// # See also
///
/// C Leptonica: `pixFindEqualValues()` in `seedfill.c`
#[allow(unused_variables)]
pub fn find_equal_values(pix1: &Pix, pix2: &Pix) -> RegionResult<Pix> {
    todo!()
}

/// Fill regions completely enclosed by foreground borders.
///
/// Identifies regions of background that are entirely surrounded by
/// foreground and fills them. Unlike [`fill_holes`], this operates
/// on the complement: it fills closed bordered regions.
///
/// Algorithm: seed from border → invert → OR with original.
///
/// # See also
///
/// C Leptonica: `pixFillClosedBorders()` in `seedfill.c`
#[allow(unused_variables)]
pub fn fill_closed_borders(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    todo!()
}

/// Remove connected components in a mask that contain seed pixels.
///
/// Performs seedfill from each seed pixel into the mask, then subtracts
/// the filled result from the mask to remove seeded components.
///
/// # Arguments
///
/// * `seed` - 1-bpp seed image (pixels marking components to remove)
/// * `mask` - 1-bpp mask image (components to filter)
/// * `connectivity` - 4 or 8-way connectivity
///
/// # See also
///
/// C Leptonica: `pixRemoveSeededComponents()` in `seedfill.c`
#[allow(unused_variables)]
pub fn remove_seeded_components(
    seed: &Pix,
    mask: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Pix> {
    todo!()
}

/// Inverse grayscale seedfill (basin filling).
///
/// Like [`seedfill_gray`], but the seed value is propagated *downward*
/// (clipped from below by the mask) rather than upward. In each pass,
/// the minimum of the current value and its neighbors is taken, clipped
/// to be no less than the mask value.
///
/// # Arguments
///
/// * `seed` - 8-bpp seed image (values ≥ mask everywhere)
/// * `mask` - 8-bpp mask image (lower bound)
/// * `connectivity` - 4 or 8-way connectivity
///
/// # See also
///
/// C Leptonica: `pixSeedfillGrayInv()` in `seedfill.c`
#[allow(unused_variables)]
pub fn seedfill_gray_inv(
    seed: &Pix,
    mask: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Pix> {
    todo!()
}

/// Binary seedfill restricted to a maximum distance from seed pixels.
///
/// Performs a full binary seedfill within the mask, then restricts the
/// result to only include pixels within `xmax` and `ymax` distance
/// from the nearest seed pixel.
///
/// # Arguments
///
/// * `seed` - 1-bpp seed image
/// * `mask` - 1-bpp filling mask
/// * `connectivity` - 4 or 8-way connectivity
/// * `xmax` - Maximum horizontal fill distance (0 = no limit)
/// * `ymax` - Maximum vertical fill distance (0 = no limit)
///
/// # See also
///
/// C Leptonica: `pixSeedfillBinaryRestricted()` in `seedfill.c`
#[allow(unused_variables)]
pub fn seedfill_binary_restricted(
    seed: &Pix,
    mask: &Pix,
    connectivity: ConnectivityType,
    xmax: u32,
    ymax: u32,
) -> RegionResult<Pix> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32, pixels: &[(u32, u32)]) -> Pix {
        let pix = Pix::new(width, height, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for &(x, y) in pixels {
            let _ = pix_mut.set_pixel(x, y, 1);
        }

        pix_mut.into()
    }

    #[test]
    fn test_floodfill_basic() {
        // Create a 5x5 image with a 3x3 block of 0s
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let count = floodfill(&mut pix_mut, 2, 2, 1, ConnectivityType::FourWay).unwrap();

        // All 25 pixels should be filled
        assert_eq!(count, 25);
    }

    #[test]
    fn test_floodfill_bounded() {
        // Create a ring of 1s
        let mut pixels = Vec::new();
        for x in 1..4 {
            pixels.push((x, 1));
            pixels.push((x, 3));
        }
        pixels.push((1, 2));
        pixels.push((3, 2));

        let pix = create_test_image(5, 5, &pixels);
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill outside the ring (from corner)
        let count = floodfill(&mut pix_mut, 0, 0, 1, ConnectivityType::FourWay).unwrap();

        // Should fill the exterior only
        assert!(count > 0);

        // Interior should still be 0
        assert_eq!(pix_mut.get_pixel(2, 2), Some(0));
    }

    #[test]
    fn test_seedfill_binary() {
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let options = SeedFillOptions::new(ConnectivityType::FourWay).with_fill_value(1);

        let filled = seedfill_binary(&pix, 2, 2, &options).unwrap();

        // All pixels should be 1
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(filled.get_pixel(x, y), Some(1));
            }
        }
    }

    #[test]
    fn test_fill_holes() {
        // Create a ring with a hole in the middle
        // 00000
        // 01110
        // 01010  <- hole at (2,2)
        // 01110
        // 00000
        let mut pixels = Vec::new();
        for x in 1..4 {
            pixels.push((x, 1));
            pixels.push((x, 3));
        }
        pixels.push((1, 2));
        pixels.push((3, 2));

        let pix = create_test_image(5, 5, &pixels);
        let filled = fill_holes(&pix, ConnectivityType::FourWay).unwrap();

        // The hole should now be filled
        assert_eq!(filled.get_pixel(2, 2), Some(1));

        // But corners should still be 0
        assert_eq!(filled.get_pixel(0, 0), Some(0));
    }

    #[test]
    fn test_clear_border() {
        // Create an image with a region touching the border and one not
        let mut pixels = vec![(0, 2), (1, 2)];

        // Region not touching border
        pixels.push((3, 3));
        pixels.push((4, 3));
        pixels.push((3, 4));
        pixels.push((4, 4));

        let pix = create_test_image(7, 7, &pixels);
        let cleared = clear_border(&pix, ConnectivityType::FourWay).unwrap();

        // Border-touching region should be gone
        assert_eq!(cleared.get_pixel(0, 2), Some(0));
        assert_eq!(cleared.get_pixel(1, 2), Some(0));

        // Interior region should remain
        assert_eq!(cleared.get_pixel(3, 3), Some(1));
        assert_eq!(cleared.get_pixel(4, 4), Some(1));
    }

    #[test]
    fn test_invalid_seed() {
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let result = floodfill(&mut pix_mut, 10, 10, 1, ConnectivityType::FourWay);
        assert!(result.is_err());
    }

    #[test]
    fn test_seedfill_gray() {
        // Create a simple seed and mask
        let seed = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut seed_mut = seed.try_into_mut().unwrap();
        let _ = seed_mut.set_pixel(2, 2, 100);
        let seed: Pix = seed_mut.into();

        let mask = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut mask_mut = mask.try_into_mut().unwrap();
        // Create a plus-sign pattern in the mask
        for i in 0..5 {
            let _ = mask_mut.set_pixel(2, i, 150);
            let _ = mask_mut.set_pixel(i, 2, 150);
        }
        let mask: Pix = mask_mut.into();

        let result = seedfill_gray(&seed, &mask, ConnectivityType::FourWay).unwrap();

        // The seed value should propagate along the mask pattern
        assert_eq!(result.get_pixel(2, 2), Some(100));
        // Values should propagate where mask allows
        assert!(result.get_pixel(2, 0).unwrap_or(0) > 0);
    }
}
