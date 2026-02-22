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
pub fn distance_function(
    pix: &Pix,
    connectivity: ConnectivityType,
    out_depth: PixelDepth,
    boundary_cond: BoundaryCondition,
) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }
    if out_depth != PixelDepth::Bit8 && out_depth != PixelDepth::Bit16 {
        return Err(RegionError::InvalidParameters(
            "out_depth must be 8 or 16".to_string(),
        ));
    }

    let w = pix.width();
    let h = pix.height();
    let max_val: u32 = if out_depth == PixelDepth::Bit8 {
        254
    } else {
        0xfffe
    };

    // Initialize: foreground pixels get max_val, background gets 0
    let init_val = match boundary_cond {
        BoundaryCondition::Background => 0u32,
        BoundaryCondition::Foreground => max_val,
    };

    let mut dist: Vec<u32> = vec![0; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            if pix.get_pixel(x, y).unwrap_or(0) != 0 {
                dist[(y * w + x) as usize] = max_val;
            }
        }
    }

    let use_diag = connectivity == ConnectivityType::EightWay;

    // Forward pass (top-left to bottom-right)
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if dist[idx] == 0 {
                continue;
            }
            let mut min_neighbor = max_val;
            // Left
            if x > 0 {
                min_neighbor = min_neighbor.min(dist[idx - 1]);
            } else {
                min_neighbor = min_neighbor.min(init_val);
            }
            // Above
            if y > 0 {
                min_neighbor = min_neighbor.min(dist[((y - 1) * w + x) as usize]);
            } else {
                min_neighbor = min_neighbor.min(init_val);
            }
            if use_diag {
                // Above-left
                if x > 0 && y > 0 {
                    min_neighbor = min_neighbor.min(dist[((y - 1) * w + x - 1) as usize]);
                } else {
                    min_neighbor = min_neighbor.min(init_val);
                }
                // Above-right
                if x + 1 < w && y > 0 {
                    min_neighbor = min_neighbor.min(dist[((y - 1) * w + x + 1) as usize]);
                } else {
                    min_neighbor = min_neighbor.min(init_val);
                }
            }
            dist[idx] = dist[idx].min(min_neighbor.saturating_add(1));
        }
    }

    // Backward pass (bottom-right to top-left)
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let idx = (y * w + x) as usize;
            if dist[idx] == 0 {
                continue;
            }
            let mut min_neighbor = max_val;
            // Right
            if x + 1 < w {
                min_neighbor = min_neighbor.min(dist[idx + 1]);
            } else {
                min_neighbor = min_neighbor.min(init_val);
            }
            // Below
            if y + 1 < h {
                min_neighbor = min_neighbor.min(dist[((y + 1) * w + x) as usize]);
            } else {
                min_neighbor = min_neighbor.min(init_val);
            }
            if use_diag {
                // Below-right
                if x + 1 < w && y + 1 < h {
                    min_neighbor = min_neighbor.min(dist[((y + 1) * w + x + 1) as usize]);
                } else {
                    min_neighbor = min_neighbor.min(init_val);
                }
                // Below-left
                if x > 0 && y + 1 < h {
                    min_neighbor = min_neighbor.min(dist[((y + 1) * w + x - 1) as usize]);
                } else {
                    min_neighbor = min_neighbor.min(init_val);
                }
            }
            dist[idx] = dist[idx].min(min_neighbor.saturating_add(1));
        }
    }

    // Write to output Pix
    let out_pix = Pix::new(w, h, out_depth).map_err(RegionError::Core)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            out_mut.set_pixel_unchecked(x, y, dist[(y * w + x) as usize]);
        }
    }

    Ok(out_mut.into())
}

/// Create a binary mask where two 8-bpp images have equal pixel values.
///
/// Returns a 1-bpp image with foreground pixels at locations where
/// `pix1` and `pix2` have the same value.
///
/// # See also
///
/// C Leptonica: `pixFindEqualValues()` in `seedfill.c`
pub fn find_equal_values(pix1: &Pix, pix2: &Pix) -> RegionResult<Pix> {
    if pix1.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: pix1.depth().bits(),
        });
    }
    if pix2.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: pix2.depth().bits(),
        });
    }

    let w = pix1.width().min(pix2.width());
    let h = pix1.height().min(pix2.height());

    let out_pix = Pix::new(w, h, PixelDepth::Bit1).map_err(RegionError::Core)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let v1 = pix1.get_pixel(x, y).unwrap_or(0);
            let v2 = pix2.get_pixel(x, y).unwrap_or(0);
            if v1 == v2 {
                out_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok(out_mut.into())
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
pub fn fill_closed_borders(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    // Algorithm: Create seed at border, use inverted image as mask,
    // seedfill, then invert result.
    // fill_holes already implements this exact logic (fill interior
    // background regions surrounded by foreground), so delegate.
    fill_holes(pix, connectivity)
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
pub fn remove_seeded_components(
    seed: &Pix,
    mask: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Pix> {
    if seed.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: seed.depth().bits(),
        });
    }
    if mask.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: mask.depth().bits(),
        });
    }

    let w = mask.width();
    let h = mask.height();

    // Binary morphological reconstruction: expand seed within mask
    // using BFS to find all mask pixels connected to any seed pixel.
    let mut filled = vec![false; (w * h) as usize];
    let mut queue = VecDeque::new();

    // Initialize queue with seed pixels that are also in the mask
    for y in 0..h {
        for x in 0..w {
            if seed.get_pixel(x, y).unwrap_or(0) != 0 && mask.get_pixel(x, y).unwrap_or(0) != 0 {
                let idx = (y * w + x) as usize;
                if !filled[idx] {
                    filled[idx] = true;
                    queue.push_back((x, y));
                }
            }
        }
    }

    // BFS expansion within mask
    while let Some((x, y)) = queue.pop_front() {
        let neighbors = get_neighbors(x, y, w, h, connectivity);
        for (nx, ny) in neighbors {
            let nidx = (ny * w + nx) as usize;
            if !filled[nidx] && mask.get_pixel(nx, ny).unwrap_or(0) != 0 {
                filled[nidx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    // Result = mask XOR filled (removes seeded components)
    let out_pix = Pix::new(w, h, PixelDepth::Bit1).map_err(RegionError::Core)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let mask_val = mask.get_pixel(x, y).unwrap_or(0);
            let fill_val = if filled[(y * w + x) as usize] { 1 } else { 0 };
            // XOR: keep mask pixels NOT in seeded components
            out_mut.set_pixel_unchecked(x, y, mask_val ^ fill_val);
        }
    }

    Ok(out_mut.into())
}

/// Get neighbors for a pixel based on connectivity.
fn get_neighbors(
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    connectivity: ConnectivityType,
) -> Vec<(u32, u32)> {
    let mut n = Vec::with_capacity(8);
    if x > 0 {
        n.push((x - 1, y));
    }
    if x + 1 < w {
        n.push((x + 1, y));
    }
    if y > 0 {
        n.push((x, y - 1));
    }
    if y + 1 < h {
        n.push((x, y + 1));
    }
    if connectivity == ConnectivityType::EightWay {
        if x > 0 && y > 0 {
            n.push((x - 1, y - 1));
        }
        if x + 1 < w && y > 0 {
            n.push((x + 1, y - 1));
        }
        if x > 0 && y + 1 < h {
            n.push((x - 1, y + 1));
        }
        if x + 1 < w && y + 1 < h {
            n.push((x + 1, y + 1));
        }
    }
    n
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
pub fn seedfill_gray_inv(
    seed: &Pix,
    mask: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Pix> {
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

    // Initialize output: seed clamped to be >= mask (since we propagate downward)
    let mut output = Pix::new(width, height, PixelDepth::Bit8)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    for y in 0..height {
        for x in 0..width {
            let seed_val = seed.get_pixel(x, y).unwrap_or(0);
            let mask_val = mask.get_pixel(x, y).unwrap_or(0);
            let _ = output.set_pixel(x, y, seed_val.max(mask_val));
        }
    }

    // Iterative reconstruction: propagate minimum values, clipped by mask from below
    let mut changed = true;
    let mut iterations = 0;
    const MAX_ITERATIONS: u32 = 10000;

    while changed && iterations < MAX_ITERATIONS {
        changed = false;
        iterations += 1;

        // Forward pass
        for y in 0..height {
            for x in 0..width {
                let current = output.get_pixel(x, y).unwrap_or(0);
                let mask_val = mask.get_pixel(x, y).unwrap_or(0);
                let mut min_neighbor = current;

                if x > 0 {
                    min_neighbor = min_neighbor.min(output.get_pixel(x - 1, y).unwrap_or(255));
                }
                if y > 0 {
                    min_neighbor = min_neighbor.min(output.get_pixel(x, y - 1).unwrap_or(255));
                }
                if connectivity == ConnectivityType::EightWay {
                    if x > 0 && y > 0 {
                        min_neighbor =
                            min_neighbor.min(output.get_pixel(x - 1, y - 1).unwrap_or(255));
                    }
                    if x + 1 < width && y > 0 {
                        min_neighbor =
                            min_neighbor.min(output.get_pixel(x + 1, y - 1).unwrap_or(255));
                    }
                }

                let new_val = min_neighbor.max(mask_val);
                if new_val < current {
                    let _ = output.set_pixel(x, y, new_val);
                    changed = true;
                }
            }
        }

        // Backward pass
        for y in (0..height).rev() {
            for x in (0..width).rev() {
                let current = output.get_pixel(x, y).unwrap_or(0);
                let mask_val = mask.get_pixel(x, y).unwrap_or(0);
                let mut min_neighbor = current;

                if x + 1 < width {
                    min_neighbor = min_neighbor.min(output.get_pixel(x + 1, y).unwrap_or(255));
                }
                if y + 1 < height {
                    min_neighbor = min_neighbor.min(output.get_pixel(x, y + 1).unwrap_or(255));
                }
                if connectivity == ConnectivityType::EightWay {
                    if x + 1 < width && y + 1 < height {
                        min_neighbor =
                            min_neighbor.min(output.get_pixel(x + 1, y + 1).unwrap_or(255));
                    }
                    if x > 0 && y + 1 < height {
                        min_neighbor =
                            min_neighbor.min(output.get_pixel(x - 1, y + 1).unwrap_or(255));
                    }
                }

                let new_val = min_neighbor.max(mask_val);
                if new_val < current {
                    let _ = output.set_pixel(x, y, new_val);
                    changed = true;
                }
            }
        }
    }

    Ok(output.into())
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
pub fn seedfill_binary_restricted(
    seed: &Pix,
    mask: &Pix,
    connectivity: ConnectivityType,
    xmax: u32,
    ymax: u32,
) -> RegionResult<Pix> {
    if seed.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: seed.depth().bits(),
        });
    }
    if mask.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: mask.depth().bits(),
        });
    }

    let w = mask.width();
    let h = mask.height();
    let no_limit = xmax == 0 && ymax == 0;

    // BFS seedfill within mask, tracking distance from nearest seed
    let mut filled = vec![false; (w * h) as usize];
    let mut queue = VecDeque::new();

    // Initialize with seed pixels that are in the mask
    for y in 0..h {
        for x in 0..w {
            if seed.get_pixel(x, y).unwrap_or(0) != 0 && mask.get_pixel(x, y).unwrap_or(0) != 0 {
                let idx = (y * w + x) as usize;
                if !filled[idx] {
                    filled[idx] = true;
                    queue.push_back((x, y, x, y)); // (current_x, current_y, seed_x, seed_y)
                }
            }
        }
    }

    // BFS with distance tracking
    while let Some((x, y, sx, sy)) = queue.pop_front() {
        let neighbors = get_neighbors(x, y, w, h, connectivity);
        for (nx, ny) in neighbors {
            let nidx = (ny * w + nx) as usize;
            if !filled[nidx] && mask.get_pixel(nx, ny).unwrap_or(0) != 0 {
                // Check distance restriction
                if !no_limit {
                    let dx = (nx as i64 - sx as i64).unsigned_abs() as u32;
                    let dy = (ny as i64 - sy as i64).unsigned_abs() as u32;
                    if (xmax > 0 && dx > xmax) || (ymax > 0 && dy > ymax) {
                        continue;
                    }
                }
                filled[nidx] = true;
                queue.push_back((nx, ny, sx, sy));
            }
        }
    }

    // Write result
    let out_pix = Pix::new(w, h, PixelDepth::Bit1).map_err(RegionError::Core)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            if filled[(y * w + x) as usize] {
                out_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok(out_mut.into())
}

/// Spread seed pixel values to fill the entire image (Voronoi-like).
///
/// Takes an 8bpp image with sparse nonzero seed pixels and spreads their
/// values to fill all zero pixels with the value of the nearest seed.
/// This is similar to computing a Voronoi tessellation where each seed
/// defines a region, and all pixels within that region take the seed's value.
///
/// The algorithm uses a two-pass raster/anti-raster distance propagation
/// (Ray Smith's method) that runs in O(n) time.
///
/// # Arguments
///
/// * `pixs` - 8bpp grayscale image with sparse nonzero seed pixels
/// * `connectivity` - 4-way or 8-way connectivity for distance computation
///
/// # See also
///
/// C Leptonica: `pixSeedspread()` in `seedfill.c`
pub fn seedspread(pixs: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    if pixs.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: pixs.depth().bits(),
        });
    }

    let orig_w = pixs.width();
    let orig_h = pixs.height();

    // Add a 4-pixel border to simplify boundary handling.
    // The border pixels are 0 in the value image and get max distance.
    let border = 4u32;
    let w = orig_w + 2 * border;
    let h = orig_h + 2 * border;

    // Build the bordered value image (8bpp stored as u8)
    let mut val = vec![0u8; (w * h) as usize];
    for y in 0..orig_h {
        for x in 0..orig_w {
            val[((y + border) * w + (x + border)) as usize] =
                pixs.get_pixel(x, y).unwrap_or(0) as u8;
        }
    }

    // Build the 16-bit distance image:
    //   0 at seed points (nonzero input), 1 at non-seed, 0xffff at borders
    let max_dist: u16 = 0xffff;
    let mut dist = vec![0u16; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if y < border || y >= h - border || x < border || x >= w - border {
                // Border region: set to max
                dist[idx] = max_dist;
            } else if val[idx] == 0 {
                // Non-seed: needs filling
                dist[idx] = 1;
            }
            // Seed points (val != 0 and inside): dist stays 0
        }
    }
    // Also set the 1-pixel boundary just inside the border to max
    // (top row, bottom row, left col, right col of the padded image)
    for x in 0..w {
        dist[x as usize] = max_dist;
        dist[((h - 1) * w + x) as usize] = max_dist;
    }
    for y in 0..h {
        dist[(y * w) as usize] = max_dist;
        dist[(y * w + w - 1) as usize] = max_dist;
    }

    // Two-pass seed spread: raster then anti-raster
    let imax = h - 1;
    let jmax = w - 1;

    // Use u32 arithmetic to avoid u16 overflow (C version uses l_int32).
    match connectivity {
        ConnectivityType::FourWay => {
            // UL → LR scan
            for i in 1..h {
                for j in 1..jmax {
                    let idx = (i * w + j) as usize;
                    let valt = dist[idx] as u32;
                    if valt > 0 {
                        let val2t = dist[((i - 1) * w + j) as usize] as u32; // top
                        let val4t = dist[(i * w + j - 1) as usize] as u32; // left
                        let minval = val2t.min(val4t).min(0xfffe);
                        dist[idx] = (minval + 1) as u16;
                        if val2t < val4t {
                            val[idx] = val[((i - 1) * w + j) as usize];
                        } else {
                            val[idx] = val[(i * w + j - 1) as usize];
                        }
                    }
                }
            }
            // LR → UL scan
            for i in (1..imax).rev() {
                for j in (1..jmax).rev() {
                    let idx = (i * w + j) as usize;
                    let valt = dist[idx] as u32;
                    if valt > 0 {
                        let val7t = dist[((i + 1) * w + j) as usize] as u32; // bottom
                        let val5t = dist[(i * w + j + 1) as usize] as u32; // right
                        let minval = val5t.min(val7t);
                        let minval = (minval + 1).min(valt);
                        if valt > minval {
                            dist[idx] = minval as u16;
                            if val5t < val7t {
                                val[idx] = val[(i * w + j + 1) as usize];
                            } else {
                                val[idx] = val[((i + 1) * w + j) as usize];
                            }
                        }
                    }
                }
            }
        }
        ConnectivityType::EightWay => {
            // UL → LR scan (check: UL, U, UR, L)
            for i in 1..h {
                for j in 1..jmax {
                    let idx = (i * w + j) as usize;
                    let valt = dist[idx] as u32;
                    if valt > 0 {
                        let val1t = dist[((i - 1) * w + j - 1) as usize] as u32; // UL
                        let val2t = dist[((i - 1) * w + j) as usize] as u32; // U
                        let val3t = dist[((i - 1) * w + j + 1) as usize] as u32; // UR
                        let val4t = dist[(i * w + j - 1) as usize] as u32; // L
                        let minval = val1t.min(val2t).min(val3t).min(val4t).min(0xfffe);
                        dist[idx] = (minval + 1) as u16;
                        if minval == val1t {
                            val[idx] = val[((i - 1) * w + j - 1) as usize];
                        } else if minval == val2t {
                            val[idx] = val[((i - 1) * w + j) as usize];
                        } else if minval == val3t {
                            val[idx] = val[((i - 1) * w + j + 1) as usize];
                        } else {
                            val[idx] = val[(i * w + j - 1) as usize];
                        }
                    }
                }
            }
            // LR → UL scan (check: LR, D, LL, R)
            for i in (1..imax).rev() {
                for j in (1..jmax).rev() {
                    let idx = (i * w + j) as usize;
                    let valt = dist[idx] as u32;
                    if valt > 0 {
                        let val8t = dist[((i + 1) * w + j + 1) as usize] as u32; // LR
                        let val7t = dist[((i + 1) * w + j) as usize] as u32; // D
                        let val6t = dist[((i + 1) * w + j - 1) as usize] as u32; // LL
                        let val5t = dist[(i * w + j + 1) as usize] as u32; // R
                        let minval = val8t.min(val7t).min(val6t).min(val5t);
                        let minval = (minval + 1).min(valt);
                        if valt > minval {
                            dist[idx] = minval as u16;
                            if minval == val5t + 1 {
                                val[idx] = val[(i * w + j + 1) as usize];
                            } else if minval == val6t + 1 {
                                val[idx] = val[((i + 1) * w + j - 1) as usize];
                            } else if minval == val7t + 1 {
                                val[idx] = val[((i + 1) * w + j) as usize];
                            } else {
                                val[idx] = val[((i + 1) * w + j + 1) as usize];
                            }
                        }
                    }
                }
            }
        }
    }

    // Extract the result: remove the border
    let out = Pix::new(orig_w, orig_h, PixelDepth::Bit8).map_err(RegionError::Core)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..orig_h {
        for x in 0..orig_w {
            out_mut.set_pixel_unchecked(
                x,
                y,
                val[((y + border) * w + (x + border)) as usize] as u32,
            );
        }
    }
    Ok(out_mut.into())
}

/// Find the minimum pixel value in each connected component of a mask.
///
/// For each connected component in the 1bpp mask, finds the pixel in the
/// 8bpp source image that has the minimum value within that component.
/// Returns the coordinates and values of these minimum pixels.
///
/// # Arguments
///
/// * `pixs` - 8bpp grayscale image
/// * `pixm` - 1bpp mask defining connected components
///
/// # Returns
///
/// A tuple of (`Pta`, `Numa`) where:
/// - `Pta` contains the (x, y) coordinates of the minimum pixel in each component
/// - `Numa` contains the corresponding minimum values
///
/// # See also
///
/// C Leptonica: `pixSelectMinInConnComp()` in `seedfill.c`
pub fn select_min_in_conncomp(
    pixs: &Pix,
    pixm: &Pix,
) -> RegionResult<(leptonica_core::Pta, leptonica_core::Numa)> {
    use crate::conncomp::conncomp_pixa;

    if pixs.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: pixs.depth().bits(),
        });
    }
    if pixm.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixm.depth().bits(),
        });
    }

    // Validate matching dimensions (C version uses pixCropToMatch)
    if pixs.width() != pixm.width() || pixs.height() != pixm.height() {
        return Err(RegionError::InvalidParameters(format!(
            "pixs ({}x{}) and pixm ({}x{}) must have the same dimensions",
            pixs.width(),
            pixs.height(),
            pixm.width(),
            pixm.height()
        )));
    }

    // Find connected components in the mask (8-connectivity as in C version)
    let (boxa, pixa) = conncomp_pixa(pixm, ConnectivityType::EightWay)?;

    let n = boxa.len();
    let mut pta = leptonica_core::Pta::with_capacity(n);
    let mut numa = leptonica_core::Numa::with_capacity(n);

    for i in 0..n {
        let b = boxa.get(i).unwrap();
        let bx = b.x as u32;
        let by = b.y as u32;
        let bw = b.w as u32;
        let bh = b.h as u32;
        let comp_pix = pixa.get(i).unwrap();

        let mut min_val = u32::MAX;
        let mut min_x = bx;
        let mut min_y = by;

        for dy in 0..bh {
            for dx in 0..bw {
                // Check if this pixel is part of the component (1bpp mask)
                if comp_pix.get_pixel(dx, dy).unwrap_or(0) != 0 {
                    let sx = bx + dx;
                    let sy = by + dy;
                    let v = pixs.get_pixel(sx, sy).unwrap_or(u32::MAX);
                    if v < min_val {
                        min_val = v;
                        min_x = sx;
                        min_y = sy;
                    }
                }
            }
        }

        pta.push(min_x as f32, min_y as f32);
        numa.push(min_val as f32);
    }

    Ok((pta, numa))
}

// -----------------------------------------------------------------------
//  Phase 1: Seedfill extensions
// -----------------------------------------------------------------------

/// Extract all foreground connected components that touch the image border.
///
/// Creates a seed image with border pixels set, then expands within the
/// input image as mask. Only components touching the border survive.
///
/// # Arguments
///
/// * `pix` - 1-bpp input image
/// * `connectivity` - 4-way or 8-way connectivity
///
/// # See also
///
/// C Leptonica: `pixExtractBorderConnComps()` in `seedfill.c`
pub fn extract_border_conn_comps(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();

    // BFS: expand seed (border pixels) within pix as mask
    let mut filled = vec![false; (w * h) as usize];
    let mut queue = VecDeque::new();

    // Seed: border pixels that are foreground in pix
    for x in 0..w {
        for &y in &[0, h - 1] {
            let idx = (y * w + x) as usize;
            if pix.get_pixel(x, y).unwrap_or(0) != 0 && !filled[idx] {
                filled[idx] = true;
                queue.push_back((x, y));
            }
        }
    }
    for y in 1..h.saturating_sub(1) {
        for &x in &[0, w - 1] {
            let idx = (y * w + x) as usize;
            if pix.get_pixel(x, y).unwrap_or(0) != 0 && !filled[idx] {
                filled[idx] = true;
                queue.push_back((x, y));
            }
        }
    }

    // BFS expansion within pix (mask)
    while let Some((x, y)) = queue.pop_front() {
        let neighbors = get_neighbors(x, y, w, h, connectivity);
        for (nx, ny) in neighbors {
            let nidx = (ny * w + nx) as usize;
            if !filled[nidx] && pix.get_pixel(nx, ny).unwrap_or(0) != 0 {
                filled[nidx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    // Write result
    let out = Pix::new(w, h, PixelDepth::Bit1).map_err(RegionError::Core)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            if filled[(y * w + x) as usize] {
                out_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok(out_mut.into())
}

/// Fill all background components touching the border to foreground.
///
/// Inverts the input, extracts border-touching components from the
/// inverted image, then ORs with the original.
///
/// # Arguments
///
/// * `pix` - 1-bpp input image
/// * `connectivity` - 4-way or 8-way connectivity
///
/// # See also
///
/// C Leptonica: `pixFillBgFromBorder()` in `seedfill.c`
pub fn fill_bg_from_border(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();

    // Invert the image: background becomes foreground
    let inverted = Pix::new(w, h, PixelDepth::Bit1).map_err(RegionError::Core)?;
    let mut inv_mut = inverted.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            inv_mut.set_pixel_unchecked(x, y, 1 - pix.get_pixel(x, y).unwrap_or(0));
        }
    }
    let inverted: Pix = inv_mut.into();

    // Extract border-connected components from the inverted image
    let border_bg = extract_border_conn_comps(&inverted, connectivity)?;

    // OR with original: fills border background to foreground
    let out = Pix::new(w, h, PixelDepth::Bit1).map_err(RegionError::Core)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let orig = pix.get_pixel(x, y).unwrap_or(0);
            let bg = border_bg.get_pixel(x, y).unwrap_or(0);
            out_mut.set_pixel_unchecked(x, y, orig | bg);
        }
    }

    Ok(out_mut.into())
}

/// Fill holes in connected components, optionally expanding to bounding rect.
///
/// For each connected component, examines the hole area fraction and
/// foreground area fraction to decide whether to fill just the holes
/// or expand the component to its bounding rectangle.
///
/// # Arguments
///
/// * `pix` - 1-bpp input image
/// * `minsize` - Minimum component area (in pixels) to consider
/// * `maxhfract` - Maximum hole area fraction relative to foreground (0.0-1.0)
/// * `minfgfract` - Minimum foreground fraction relative to bounding rect (0.0-1.0)
///
/// # See also
///
/// C Leptonica: `pixFillHolesToBoundingRect()` in `seedfill.c`
pub fn fill_holes_to_bounding_rect(
    pix: &Pix,
    minsize: u32,
    maxhfract: f32,
    minfgfract: f32,
) -> RegionResult<Pix> {
    use crate::conncomp::conncomp_pixa;

    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let maxhfract = maxhfract.clamp(0.0, 1.0);
    let minfgfract = minfgfract.clamp(0.0, 1.0);

    let w = pix.width();
    let h = pix.height();

    // Start with a copy of the input
    let mut result = pix.to_mut();

    // Find connected components (8-connectivity, as in C version)
    let (boxa, pixa) = conncomp_pixa(pix, ConnectivityType::EightWay)?;
    let n = boxa.len();

    for i in 0..n {
        let b = boxa.get(i).unwrap();
        let bx = b.x as u32;
        let by = b.y as u32;
        let bw = b.w as u32;
        let bh = b.h as u32;
        let area = bw * bh;

        if area < minsize {
            continue;
        }

        let comp = pixa.get(i).unwrap();

        // Find holes using 4-connectivity (as in C version)
        let hole_pix = holes_by_filling(comp, ConnectivityType::FourWay)?;

        // Count foreground pixels and hole pixels
        let mut nfg = 0u32;
        let mut nh = 0u32;
        for dy in 0..bh {
            for dx in 0..bw {
                if comp.get_pixel(dx, dy).unwrap_or(0) != 0 {
                    nfg += 1;
                }
                if hole_pix.get_pixel(dx, dy).unwrap_or(0) != 0 {
                    nh += 1;
                }
            }
        }

        if nfg == 0 {
            continue;
        }

        let hfract = nh as f32 / nfg as f32;
        let ntot = if hfract <= maxhfract { nfg + nh } else { nfg };
        let fgfract = ntot as f32 / area as f32;

        if fgfract >= minfgfract {
            // Fill entire bounding rect to foreground
            for dy in 0..bh {
                for dx in 0..bw {
                    let px = bx + dx;
                    let py = by + dy;
                    if px < w && py < h {
                        let _ = result.set_pixel(px, py, 1);
                    }
                }
            }
        } else if hfract <= maxhfract {
            // Fill just the holes
            for dy in 0..bh {
                for dx in 0..bw {
                    if hole_pix.get_pixel(dx, dy).unwrap_or(0) != 0 {
                        let px = bx + dx;
                        let py = by + dy;
                        if px < w && py < h {
                            let _ = result.set_pixel(px, py, 1);
                        }
                    }
                }
            }
        }
    }

    Ok(result.into())
}

/// Extract holes in a binary image as foreground using seedfill.
///
/// Uses a seedfill-based approach: creates a border seed, fills through
/// the inverted image, ORs with original, then inverts. The result has
/// holes as foreground pixels.
///
/// # Arguments
///
/// * `pix` - 1-bpp input image
/// * `connectivity` - 4-way or 8-way connectivity
///
/// # See also
///
/// C Leptonica: `pixHolesByFilling()` in `seedfill.c`
pub fn holes_by_filling(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();

    // Invert pix
    let mut inv = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            if pix.get_pixel(x, y).unwrap_or(0) == 0 {
                inv[(y * w + x) as usize] = 1;
            }
        }
    }

    // BFS from border through inverted image (binary seedfill).
    // Seed only from border pixels that are in the inverted image
    // (i.e., background in the original), matching C pixSeedfillBinary.
    let mut filled = vec![false; (w * h) as usize];
    let mut queue = VecDeque::new();

    for x in 0..w {
        for &y in &[0, h - 1] {
            let idx = (y * w + x) as usize;
            if inv[idx] != 0 && !filled[idx] {
                filled[idx] = true;
                queue.push_back((x, y));
            }
        }
    }
    for y in 1..h.saturating_sub(1) {
        for &x in &[0, w - 1] {
            let idx = (y * w + x) as usize;
            if inv[idx] != 0 && !filled[idx] {
                filled[idx] = true;
                queue.push_back((x, y));
            }
        }
    }

    // Expand seed within inverted image (inverted pixels are passable)
    while let Some((x, y)) = queue.pop_front() {
        let neighbors = get_neighbors(x, y, w, h, connectivity);
        for (nx, ny) in neighbors {
            let nidx = (ny * w + nx) as usize;
            if !filled[nidx] && inv[nidx] != 0 {
                filled[nidx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    // OR filled result with original, then invert
    // filled = everything reachable from border through bg
    // OR with original = everything that is fg OR reachable bg
    // Invert = holes only (bg not reachable from border)
    let out = Pix::new(w, h, PixelDepth::Bit1).map_err(RegionError::Core)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let orig = pix.get_pixel(x, y).unwrap_or(0);
            let fill = if filled[(y * w + x) as usize] { 1 } else { 0 };
            // OR then invert: NOT (orig | fill)
            let val = 1 - (orig | fill);
            out_mut.set_pixel_unchecked(x, y, val);
        }
    }

    Ok(out_mut.into())
}

/// Private helper: iterative raster/anti-raster scan with min-clamp (grayscale reconstruction).
///
/// Modifies `data` in-place. Pixels where `mask_data[i] == 0` are skipped.
/// Each pixel is updated to `max(self, causal_neighbors).min(mask_data[i])`.
fn scan_gray_reconstruct(data: &mut [u8], mask_data: &[u8], w: u32, h: u32, use_8: bool) {
    const MAX_ITERS: u32 = 40;
    for _ in 0..MAX_ITERS {
        let prev = data.to_vec();

        // Raster scan (UL -> LR)
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                let maskval = mask_data[idx];
                if maskval == 0 {
                    continue;
                }
                let mut maxval = data[idx];
                if y > 0 {
                    maxval = maxval.max(data[((y - 1) * w + x) as usize]);
                }
                if x > 0 {
                    maxval = maxval.max(data[(y * w + x - 1) as usize]);
                }
                if use_8 {
                    if x > 0 && y > 0 {
                        maxval = maxval.max(data[((y - 1) * w + x - 1) as usize]);
                    }
                    if x + 1 < w && y > 0 {
                        maxval = maxval.max(data[((y - 1) * w + x + 1) as usize]);
                    }
                }
                data[idx] = maxval.min(maskval);
            }
        }

        // Anti-raster scan (LR -> UL)
        for y in (0..h).rev() {
            for x in (0..w).rev() {
                let idx = (y * w + x) as usize;
                let maskval = mask_data[idx];
                if maskval == 0 {
                    continue;
                }
                let mut maxval = data[idx];
                if y + 1 < h {
                    maxval = maxval.max(data[((y + 1) * w + x) as usize]);
                }
                if x + 1 < w {
                    maxval = maxval.max(data[(y * w + x + 1) as usize]);
                }
                if use_8 {
                    if x + 1 < w && y + 1 < h {
                        maxval = maxval.max(data[((y + 1) * w + x + 1) as usize]);
                    }
                    if x > 0 && y + 1 < h {
                        maxval = maxval.max(data[((y + 1) * w + x - 1) as usize]);
                    }
                }
                data[idx] = maxval.min(maskval);
            }
        }

        if data == prev.as_slice() {
            break;
        }
    }
}

/// Private helper: iterative raster/anti-raster scan with max-propagate (inverse grayscale reconstruction).
///
/// Modifies `data` in-place. Pixels where `mask_data[i] == 255` are skipped.
/// Each pixel is updated to `max(self, causal_neighbors)` only when that max exceeds the mask.
fn scan_gray_inv_reconstruct(data: &mut [u8], mask_data: &[u8], w: u32, h: u32, use_8: bool) {
    const MAX_ITERS: u32 = 40;
    for _ in 0..MAX_ITERS {
        let prev = data.to_vec();

        // Raster scan (UL -> LR)
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                let maskval = mask_data[idx];
                if maskval == 255 {
                    continue;
                }
                let mut maxval = data[idx];
                if y > 0 {
                    maxval = maxval.max(data[((y - 1) * w + x) as usize]);
                }
                if x > 0 {
                    maxval = maxval.max(data[(y * w + x - 1) as usize]);
                }
                if use_8 {
                    if x > 0 && y > 0 {
                        maxval = maxval.max(data[((y - 1) * w + x - 1) as usize]);
                    }
                    if x + 1 < w && y > 0 {
                        maxval = maxval.max(data[((y - 1) * w + x + 1) as usize]);
                    }
                }
                if maxval > maskval {
                    data[idx] = maxval;
                }
            }
        }

        // Anti-raster scan (LR -> UL)
        for y in (0..h).rev() {
            for x in (0..w).rev() {
                let idx = (y * w + x) as usize;
                let maskval = mask_data[idx];
                if maskval == 255 {
                    continue;
                }
                let mut maxval = data[idx];
                if y + 1 < h {
                    maxval = maxval.max(data[((y + 1) * w + x) as usize]);
                }
                if x + 1 < w {
                    maxval = maxval.max(data[(y * w + x + 1) as usize]);
                }
                if use_8 {
                    if x + 1 < w && y + 1 < h {
                        maxval = maxval.max(data[((y + 1) * w + x + 1) as usize]);
                    }
                    if x > 0 && y + 1 < h {
                        maxval = maxval.max(data[((y + 1) * w + x - 1) as usize]);
                    }
                }
                if maxval > maskval {
                    data[idx] = maxval;
                }
            }
        }

        if data == prev.as_slice() {
            break;
        }
    }
}

/// Simple iterative grayscale seedfill (morphological reconstruction).
///
/// Performs grayscale morphological reconstruction using sequential
/// raster and anti-raster scans. The mask clips the seed from above
/// (seed values cannot exceed mask values).
///
/// # Arguments
///
/// * `seed` - 8-bpp seed image
/// * `mask` - 8-bpp filling mask (upper bound)
/// * `connectivity` - 4-way or 8-way connectivity
///
/// # See also
///
/// C Leptonica: `pixSeedfillGraySimple()` in `seedfill.c`
pub fn seedfill_gray_simple(
    seed: &Pix,
    mask: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Pix> {
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

    let w = seed.width();
    let h = seed.height();
    if mask.width() != w || mask.height() != h {
        return Err(RegionError::InvalidParameters(
            "seed and mask must have the same dimensions".to_string(),
        ));
    }

    // Initialize output with seed clamped to mask
    let mut data = vec![0u8; (w * h) as usize];
    let mut mask_data = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let sv = seed.get_pixel(x, y).unwrap_or(0) as u8;
            let mv = mask.get_pixel(x, y).unwrap_or(0) as u8;
            let idx = (y * w + x) as usize;
            data[idx] = sv.min(mv);
            mask_data[idx] = mv;
        }
    }

    let use_8 = connectivity == ConnectivityType::EightWay;
    scan_gray_reconstruct(&mut data, &mask_data, w, h, use_8);

    // Write output
    let out = Pix::new(w, h, PixelDepth::Bit8).map_err(RegionError::Core)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            out_mut.set_pixel_unchecked(x, y, data[(y * w + x) as usize] as u32);
        }
    }

    Ok(out_mut.into())
}

/// Simple iterative inverse grayscale seedfill.
///
/// Like [`seedfill_gray_simple`], but the mask clips from below rather
/// than above. Seed values propagate upward (maximized) where they
/// exceed the mask.
///
/// # Arguments
///
/// * `seed` - 8-bpp seed image
/// * `mask` - 8-bpp filling mask (lower bound)
/// * `connectivity` - 4-way or 8-way connectivity
///
/// # See also
///
/// C Leptonica: `pixSeedfillGrayInvSimple()` in `seedfill.c`
pub fn seedfill_gray_inv_simple(
    seed: &Pix,
    mask: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Pix> {
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

    let w = seed.width();
    let h = seed.height();
    if mask.width() != w || mask.height() != h {
        return Err(RegionError::InvalidParameters(
            "seed and mask must have the same dimensions".to_string(),
        ));
    }

    // Initialize output: seed clamped to be >= mask
    let mut data = vec![0u8; (w * h) as usize];
    let mut mask_data = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let sv = seed.get_pixel(x, y).unwrap_or(0) as u8;
            let mv = mask.get_pixel(x, y).unwrap_or(0) as u8;
            let idx = (y * w + x) as usize;
            data[idx] = sv.max(mv);
            mask_data[idx] = mv;
        }
    }

    let use_8 = connectivity == ConnectivityType::EightWay;
    scan_gray_inv_reconstruct(&mut data, &mask_data, w, h, use_8);

    let out = Pix::new(w, h, PixelDepth::Bit8).map_err(RegionError::Core)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            out_mut.set_pixel_unchecked(x, y, data[(y * w + x) as usize] as u32);
        }
    }

    Ok(out_mut.into())
}

/// Grayscale basin filling with delta above the mask.
///
/// At seed locations (pixb foreground), the seed value is mask + delta.
/// Elsewhere, the seed is 255. Fills basins by running standard gray
/// seedfill on inverted seed and mask, then re-inverting.
///
/// # Arguments
///
/// * `pixb` - 1-bpp binary mask giving seed locations
/// * `pixm` - 8-bpp grayscale mask (basin topography)
/// * `delta` - Amount of seed value above mask at seed locations
/// * `connectivity` - 4-way or 8-way connectivity
///
/// # See also
///
/// C Leptonica: `pixSeedfillGrayBasin()` in `seedfill.c`
pub fn seedfill_gray_basin(
    pixb: &Pix,
    pixm: &Pix,
    delta: i32,
    connectivity: ConnectivityType,
) -> RegionResult<Pix> {
    if pixb.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixb.depth().bits(),
        });
    }
    if pixm.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: pixm.depth().bits(),
        });
    }

    // delta <= 0: return copy of pixm
    if delta <= 0 {
        return Ok(pixm.deep_clone());
    }

    let w = pixm.width();
    let h = pixm.height();

    // Build seed: pixm + delta, then set to 255 where pixb is 0
    let mut seed_data = vec![0u8; (w * h) as usize];
    let mut mask_data = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let mv = pixm.get_pixel(x, y).unwrap_or(0) as i32;
            mask_data[idx] = mv as u8;

            let bv = if x < pixb.width() && y < pixb.height() {
                pixb.get_pixel(x, y).unwrap_or(0)
            } else {
                0
            };

            if bv != 0 {
                // Seed location: mask + delta, clamped to 255
                seed_data[idx] = (mv + delta).clamp(0, 255) as u8;
            } else {
                // Non-seed: 255
                seed_data[idx] = 255;
            }
        }
    }

    // Invert both seed and mask
    for v in &mut seed_data {
        *v = 255 - *v;
    }
    for v in &mut mask_data {
        *v = 255 - *v;
    }

    // Run standard gray seedfill (simple) on inverted images.
    // Seed is clipped from above by mask.
    let use_8 = connectivity == ConnectivityType::EightWay;

    // Initialize: seed clamped to mask
    for i in 0..seed_data.len() {
        seed_data[i] = seed_data[i].min(mask_data[i]);
    }

    scan_gray_reconstruct(&mut seed_data, &mask_data, w, h, use_8);

    // Re-invert
    for v in &mut seed_data {
        *v = 255 - *v;
    }

    // Write output
    let out = Pix::new(w, h, PixelDepth::Bit8).map_err(RegionError::Core)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            out_mut.set_pixel_unchecked(x, y, seed_data[(y * w + x) as usize] as u32);
        }
    }

    Ok(out_mut.into())
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

    // -----------------------------------------------------------------------
    //  Phase 1: Seedfill extensions tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_border_conn_comps_basic() {
        // 8x8 image with two components:
        // - Component A touching the left border (x=0)
        // - Component B in the interior (not touching border)
        let mut pixels = vec![(0, 3), (1, 3), (1, 4)]; // border-touching
        pixels.extend([(4, 4), (5, 4), (4, 5), (5, 5)]); // interior

        let pix = create_test_image(8, 8, &pixels);
        let result = extract_border_conn_comps(&pix, ConnectivityType::FourWay).unwrap();

        // Border-touching component should be present
        assert_eq!(result.get_pixel(0, 3), Some(1));
        assert_eq!(result.get_pixel(1, 3), Some(1));
        assert_eq!(result.get_pixel(1, 4), Some(1));

        // Interior component should NOT be present
        assert_eq!(result.get_pixel(4, 4), Some(0));
        assert_eq!(result.get_pixel(5, 5), Some(0));
    }

    #[test]
    fn test_extract_border_conn_comps_empty() {
        // Empty image: no foreground pixels
        let pix = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
        let result = extract_border_conn_comps(&pix, ConnectivityType::EightWay).unwrap();

        // Result should also be empty
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(result.get_pixel(x, y), Some(0));
            }
        }
    }

    #[test]
    fn test_fill_bg_from_border_basic() {
        // Create a box with background inside and outside.
        // The outside background touches the border; inside does not.
        // 00000000
        // 01111110
        // 01000010  <- interior background (holes)
        // 01000010
        // 01111110
        // 00000000
        let mut pixels = Vec::new();
        for x in 1..7 {
            pixels.push((x, 1));
            pixels.push((x, 4));
        }
        for y in 2..4 {
            pixels.push((1, y));
            pixels.push((6, y));
        }

        let pix = create_test_image(8, 6, &pixels);
        let result = fill_bg_from_border(&pix, ConnectivityType::FourWay).unwrap();

        // Border background should be filled to foreground
        assert_eq!(result.get_pixel(0, 0), Some(1));
        assert_eq!(result.get_pixel(7, 5), Some(1));

        // Interior background (holes) should remain as background
        assert_eq!(result.get_pixel(3, 2), Some(0));
        assert_eq!(result.get_pixel(3, 3), Some(0));

        // Original foreground should remain
        assert_eq!(result.get_pixel(1, 1), Some(1));
    }

    #[test]
    fn test_fill_holes_to_bounding_rect_fill_all() {
        // A component with a small hole; use maxhfract=1.0 and minfgfract=1.0
        // to fill all holes but not expand to bounding rect.
        // 00000000
        // 01111100
        // 01001100
        // 01111100
        // 00000000
        let mut pixels = Vec::new();
        for x in 1..6 {
            pixels.push((x, 1));
            pixels.push((x, 3));
        }
        pixels.push((1, 2));
        pixels.push((4, 2));
        pixels.push((5, 2));

        let pix = create_test_image(8, 5, &pixels);
        let result = fill_holes_to_bounding_rect(&pix, 0, 1.0, 1.0).unwrap();

        // Holes within the component should be filled
        assert_eq!(result.get_pixel(2, 2), Some(1));
        assert_eq!(result.get_pixel(3, 2), Some(1));

        // Background outside should remain
        assert_eq!(result.get_pixel(0, 0), Some(0));
    }

    #[test]
    fn test_holes_by_filling_basic() {
        // Ring with a hole at (2,2)
        // 00000
        // 01110
        // 01010
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
        let result = holes_by_filling(&pix, ConnectivityType::FourWay).unwrap();

        // The hole at (2,2) should be foreground in the result
        assert_eq!(result.get_pixel(2, 2), Some(1));

        // Non-hole pixels should be background
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 1), Some(0)); // original foreground is NOT a hole
    }

    #[test]
    fn test_holes_by_filling_no_holes() {
        // Solid rectangle: no holes
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }

        let pix = create_test_image(5, 5, &pixels);
        let result = holes_by_filling(&pix, ConnectivityType::FourWay).unwrap();

        // No holes, so result should be all zeros
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(result.get_pixel(x, y), Some(0));
            }
        }
    }

    #[test]
    fn test_seedfill_gray_simple_basic() {
        // Create seed with single high point and mask as a plateau
        let seed = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut seed_mut = seed.try_into_mut().unwrap();
        let _ = seed_mut.set_pixel(2, 2, 100);
        let seed: Pix = seed_mut.into();

        let mask = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut mask_mut = mask.try_into_mut().unwrap();
        for y in 0..5 {
            for x in 0..5 {
                let _ = mask_mut.set_pixel(x, y, 150);
            }
        }
        let mask: Pix = mask_mut.into();

        let result = seedfill_gray_simple(&seed, &mask, ConnectivityType::FourWay).unwrap();

        // Seed value should propagate everywhere (all mask >= seed)
        assert_eq!(result.get_pixel(2, 2), Some(100));
        assert_eq!(result.get_pixel(0, 0), Some(100));
        assert_eq!(result.get_pixel(4, 4), Some(100));
    }

    #[test]
    fn test_seedfill_gray_simple_matches_seedfill_gray() {
        // The simple version should produce the same result as seedfill_gray
        let seed = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let mut seed_mut = seed.try_into_mut().unwrap();
        let _ = seed_mut.set_pixel(4, 4, 200);
        let _ = seed_mut.set_pixel(1, 1, 50);
        let seed: Pix = seed_mut.into();

        let mask = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let mut mask_mut = mask.try_into_mut().unwrap();
        for y in 0..8 {
            for x in 0..8 {
                let _ = mask_mut.set_pixel(x, y, 180);
            }
        }
        let mask: Pix = mask_mut.into();

        let result_simple = seedfill_gray_simple(&seed, &mask, ConnectivityType::FourWay).unwrap();
        let result_existing = seedfill_gray(&seed, &mask, ConnectivityType::FourWay).unwrap();

        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(
                    result_simple.get_pixel(x, y),
                    result_existing.get_pixel(x, y),
                    "mismatch at ({}, {})",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_seedfill_gray_inv_simple_basic() {
        // Inverse seedfill: propagates max values where they exceed mask.
        // A high seed value propagates to all connected pixels where
        // the propagated value exceeds the mask.
        let seed = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut seed_mut = seed.try_into_mut().unwrap();
        for y in 0..5 {
            for x in 0..5 {
                let _ = seed_mut.set_pixel(x, y, 50);
            }
        }
        let _ = seed_mut.set_pixel(2, 2, 200);
        let seed: Pix = seed_mut.into();

        let mask = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut mask_mut = mask.try_into_mut().unwrap();
        for y in 0..5 {
            for x in 0..5 {
                let _ = mask_mut.set_pixel(x, y, 100);
            }
        }
        let mask: Pix = mask_mut.into();

        let result = seedfill_gray_inv_simple(&seed, &mask, ConnectivityType::FourWay).unwrap();

        // Init: data = max(seed, mask) = all 100, center = 200
        // 200 > mask(100) everywhere, so 200 propagates to all pixels
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(
                    result.get_pixel(x, y),
                    Some(200),
                    "mismatch at ({}, {})",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_seedfill_gray_basin_basic() {
        // Create a simple basin: a 5x5 image with low center and high edges
        let pixm = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut pixm_mut = pixm.try_into_mut().unwrap();
        for y in 0..5 {
            for x in 0..5 {
                let _ = pixm_mut.set_pixel(x, y, 200);
            }
        }
        // Basin at center
        let _ = pixm_mut.set_pixel(2, 2, 50);
        let _ = pixm_mut.set_pixel(1, 2, 100);
        let _ = pixm_mut.set_pixel(3, 2, 100);
        let _ = pixm_mut.set_pixel(2, 1, 100);
        let _ = pixm_mut.set_pixel(2, 3, 100);
        let pixm: Pix = pixm_mut.into();

        // Seed location at the basin minimum
        let pixb = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pixb_mut = pixb.try_into_mut().unwrap();
        let _ = pixb_mut.set_pixel(2, 2, 1);
        let pixb: Pix = pixb_mut.into();

        let delta = 30;
        let result = seedfill_gray_basin(&pixb, &pixm, delta, ConnectivityType::FourWay).unwrap();

        // At the seed location, the value should be mask + delta = 50 + 30 = 80
        let center_val = result.get_pixel(2, 2).unwrap_or(0);
        assert!(
            center_val >= 80,
            "expected center >= 80, got {}",
            center_val
        );
    }

    #[test]
    fn test_seedfill_gray_basin_zero_delta() {
        let pixm = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut pixm_mut = pixm.try_into_mut().unwrap();
        for y in 0..5 {
            for x in 0..5 {
                let _ = pixm_mut.set_pixel(x, y, 100);
            }
        }
        let pixm: Pix = pixm_mut.into();

        let pixb = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pixb_mut = pixb.try_into_mut().unwrap();
        let _ = pixb_mut.set_pixel(2, 2, 1);
        let pixb: Pix = pixb_mut.into();

        // delta <= 0 should return a copy of pixm
        let result = seedfill_gray_basin(&pixb, &pixm, 0, ConnectivityType::FourWay).unwrap();

        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(result.get_pixel(x, y), Some(100));
            }
        }
    }
}
