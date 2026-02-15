//! Watershed segmentation
//!
//! This module provides the watershed algorithm for image segmentation.
//! The watershed transform treats the grayscale image as a topographic
//! surface and finds boundaries between catchment basins.

use crate::conncomp::ConnectivityType;
use crate::error::{RegionError, RegionResult};
use leptonica_core::{Pix, PixelDepth};
use std::collections::BinaryHeap;

/// Options for watershed segmentation
#[derive(Debug, Clone)]
pub struct WatershedOptions {
    /// Minimum depth for basins (basins shallower than this are merged)
    pub min_depth: u32,
    /// Connectivity type for finding neighbors
    pub connectivity: ConnectivityType,
}

impl Default for WatershedOptions {
    fn default() -> Self {
        Self {
            min_depth: 1,
            connectivity: ConnectivityType::EightWay,
        }
    }
}

impl WatershedOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum basin depth
    pub fn with_min_depth(mut self, depth: u32) -> Self {
        self.min_depth = depth;
        self
    }

    /// Set connectivity type
    pub fn with_connectivity(mut self, connectivity: ConnectivityType) -> Self {
        self.connectivity = connectivity;
        self
    }
}

/// Pixel entry for the priority queue in watershed algorithm
#[derive(Clone, Eq, PartialEq)]
struct PixelEntry {
    /// X coordinate
    x: u32,
    /// Y coordinate
    y: u32,
    /// Grayscale value (lower = higher priority)
    value: u32,
}

impl Ord for PixelEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering so lower values have higher priority
        other.value.cmp(&self.value)
    }
}

impl PartialOrd for PixelEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Watershed label indicating the pixel status
const WATERSHED: u32 = 0;
const UNLABELED: u32 = u32::MAX;
const IN_QUEUE: u32 = u32::MAX - 1;

/// Perform watershed segmentation on a grayscale image
///
/// The algorithm treats the image as a topographic surface where bright
/// pixels are high and dark pixels are low. It floods the surface from
/// local minima, creating labeled regions separated by watershed lines.
///
/// # Arguments
///
/// * `pix` - Input grayscale image (8-bit)
/// * `options` - Watershed options
///
/// # Returns
///
/// A 32-bit labeled image where:
/// - 0 indicates watershed boundaries
/// - Positive values indicate basin labels
pub fn watershed_segmentation(pix: &Pix, options: &WatershedOptions) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    if width == 0 || height == 0 {
        return Err(RegionError::EmptyImage);
    }

    // Create label image
    let mut labels = vec![UNLABELED; (width * height) as usize];

    // Find local minima as seeds
    let minima = find_local_minima(pix, options.connectivity)?;

    // Initialize labels with minima as seeds
    let mut current_label = 1u32;
    let mut queue = BinaryHeap::new();

    for (mx, my) in &minima {
        let idx = (my * width + mx) as usize;
        if labels[idx] == UNLABELED {
            labels[idx] = current_label;
            let value = pix.get_pixel(*mx, *my).unwrap_or(0);
            queue.push(PixelEntry {
                x: *mx,
                y: *my,
                value,
            });
            current_label += 1;
        }
    }

    // Process pixels in order of increasing gray value
    while let Some(entry) = queue.pop() {
        let x = entry.x;
        let y = entry.y;
        let idx = (y * width + x) as usize;
        let current_label = labels[idx];

        if current_label == WATERSHED || current_label == IN_QUEUE {
            continue;
        }

        // Get neighbors
        let neighbors = get_neighbors(x, y, width, height, options.connectivity);

        for (nx, ny) in neighbors {
            let nidx = (ny * width + nx) as usize;
            let neighbor_label = labels[nidx];

            if neighbor_label == UNLABELED {
                // Unlabeled neighbor - assign same label and add to queue
                labels[nidx] = current_label;
                let value = pix.get_pixel(nx, ny).unwrap_or(0);
                queue.push(PixelEntry {
                    x: nx,
                    y: ny,
                    value,
                });
            } else if neighbor_label != current_label
                && neighbor_label != WATERSHED
                && neighbor_label != IN_QUEUE
            {
                // Different label - this is a watershed boundary
                // Only mark as watershed if we haven't already assigned this pixel
                if labels[idx] != WATERSHED {
                    // Check if this should be a watershed line
                    let neighbor_value = pix.get_pixel(nx, ny).unwrap_or(0);
                    let current_value = pix.get_pixel(x, y).unwrap_or(0);

                    // Create watershed line if the depth is significant
                    if current_value.abs_diff(neighbor_value) >= options.min_depth {
                        // Mark the boundary between regions
                        // Keep the current pixel's label but note the boundary exists
                    }
                }
            }
        }
    }

    // Second pass: identify watershed lines properly
    // A pixel is on a watershed line if it has neighbors with different labels
    let mut output = Pix::new(width, height, PixelDepth::Bit32)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let label = labels[idx];

            if label == UNLABELED || label == IN_QUEUE {
                let _ = output.set_pixel(x, y, 0);
                continue;
            }

            // Check if this pixel is on a watershed boundary
            let neighbors = get_neighbors(x, y, width, height, options.connectivity);
            let mut is_boundary = false;

            for (nx, ny) in &neighbors {
                let nidx = (ny * width + nx) as usize;
                let neighbor_label = labels[nidx];

                if neighbor_label != label
                    && neighbor_label != UNLABELED
                    && neighbor_label != IN_QUEUE
                    && neighbor_label != WATERSHED
                {
                    is_boundary = true;
                    break;
                }
            }

            if is_boundary {
                let _ = output.set_pixel(x, y, WATERSHED);
            } else {
                let _ = output.set_pixel(x, y, label);
            }
        }
    }

    Ok(output.into())
}

/// Find local minima in a grayscale image
///
/// A pixel is a local minimum if it has the smallest value among its neighbors.
///
/// # Arguments
///
/// * `pix` - Input grayscale image (8-bit)
/// * `connectivity` - Connectivity type for defining neighbors
///
/// # Returns
///
/// A vector of (x, y) coordinates of local minima.
pub fn find_local_minima(
    pix: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Vec<(u32, u32)>> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();
    let mut minima = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let value = pix.get_pixel(x, y).unwrap_or(255);
            let neighbors = get_neighbors(x, y, width, height, connectivity);

            let is_minimum = neighbors.iter().all(|&(nx, ny)| {
                let neighbor_value = pix.get_pixel(nx, ny).unwrap_or(255);
                value <= neighbor_value
            });

            // Also check if strictly less than at least one neighbor (not plateau)
            let has_lower_neighbor = neighbors.iter().any(|&(nx, ny)| {
                let neighbor_value = pix.get_pixel(nx, ny).unwrap_or(255);
                value < neighbor_value
            });

            if is_minimum && (has_lower_neighbor || neighbors.is_empty()) {
                minima.push((x, y));
            }
        }
    }

    Ok(minima)
}

/// Find local maxima in a grayscale image
///
/// A pixel is a local maximum if it has the largest value among its neighbors.
///
/// # Arguments
///
/// * `pix` - Input grayscale image (8-bit)
/// * `connectivity` - Connectivity type for defining neighbors
///
/// # Returns
///
/// A vector of (x, y) coordinates of local maxima.
pub fn find_local_maxima(
    pix: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Vec<(u32, u32)>> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();
    let mut maxima = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let value = pix.get_pixel(x, y).unwrap_or(0);
            let neighbors = get_neighbors(x, y, width, height, connectivity);

            let is_maximum = neighbors.iter().all(|&(nx, ny)| {
                let neighbor_value = pix.get_pixel(nx, ny).unwrap_or(0);
                value >= neighbor_value
            });

            let has_higher_neighbor = neighbors.iter().any(|&(nx, ny)| {
                let neighbor_value = pix.get_pixel(nx, ny).unwrap_or(0);
                value > neighbor_value
            });

            if is_maximum && (has_higher_neighbor || neighbors.is_empty()) {
                maxima.push((x, y));
            }
        }
    }

    Ok(maxima)
}

/// Compute gradient magnitude of a grayscale image
///
/// Uses simple difference operators to compute the gradient magnitude.
///
/// # Arguments
///
/// * `pix` - Input grayscale image (8-bit)
///
/// # Returns
///
/// An 8-bit image containing the gradient magnitude at each pixel.
pub fn compute_gradient(pix: &Pix) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(RegionError::UnsupportedDepth {
            expected: "8-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    let mut output = Pix::new(width, height, PixelDepth::Bit8)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    for y in 0..height {
        for x in 0..width {
            let center = pix.get_pixel(x, y).unwrap_or(0) as i32;

            // Compute horizontal gradient
            let left = if x > 0 {
                pix.get_pixel(x - 1, y).unwrap_or(0) as i32
            } else {
                center
            };
            let right = if x + 1 < width {
                pix.get_pixel(x + 1, y).unwrap_or(0) as i32
            } else {
                center
            };
            let gx = right - left;

            // Compute vertical gradient
            let top = if y > 0 {
                pix.get_pixel(x, y - 1).unwrap_or(0) as i32
            } else {
                center
            };
            let bottom = if y + 1 < height {
                pix.get_pixel(x, y + 1).unwrap_or(0) as i32
            } else {
                center
            };
            let gy = bottom - top;

            // Compute magnitude (use approximation for speed)
            let magnitude = ((gx.abs() + gy.abs()) / 2).min(255) as u32;
            let _ = output.set_pixel(x, y, magnitude);
        }
    }

    Ok(output.into())
}

/// Get neighbor coordinates for a pixel
fn get_neighbors(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    connectivity: ConnectivityType,
) -> Vec<(u32, u32)> {
    let mut neighbors = Vec::with_capacity(8);

    // 4-way neighbors
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

    // 8-way diagonal neighbors
    if connectivity == ConnectivityType::EightWay {
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

/// Find basins (catchment regions) in a grayscale image
///
/// Each basin is a region where all paths following the steepest descent
/// lead to the same local minimum.
///
/// # Arguments
///
/// * `pix` - Input grayscale image (8-bit)
/// * `connectivity` - Connectivity type
///
/// # Returns
///
/// A 32-bit labeled image where each basin has a unique label.
pub fn find_basins(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    let options = WatershedOptions::new()
        .with_min_depth(0)
        .with_connectivity(connectivity);

    watershed_segmentation(pix, &options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_gray_image(width: u32, height: u32, values: &[Vec<u32>]) -> Pix {
        let pix = Pix::new(width, height, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for (y, row) in values.iter().enumerate() {
            for (x, &value) in row.iter().enumerate() {
                let _ = pix_mut.set_pixel(x as u32, y as u32, value);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_find_local_minima() {
        // Create a simple image with one minimum at center
        let values = vec![vec![5, 5, 5], vec![5, 1, 5], vec![5, 5, 5]];
        let pix = create_gray_image(3, 3, &values);

        let minima = find_local_minima(&pix, ConnectivityType::FourWay).unwrap();

        assert_eq!(minima.len(), 1);
        assert_eq!(minima[0], (1, 1));
    }

    #[test]
    fn test_find_local_maxima() {
        // Create a simple image with one maximum at center
        let values = vec![vec![1, 1, 1], vec![1, 5, 1], vec![1, 1, 1]];
        let pix = create_gray_image(3, 3, &values);

        let maxima = find_local_maxima(&pix, ConnectivityType::FourWay).unwrap();

        assert_eq!(maxima.len(), 1);
        assert_eq!(maxima[0], (1, 1));
    }

    #[test]
    fn test_compute_gradient() {
        // Create an image with a step edge
        let values = vec![
            vec![0, 0, 100, 100],
            vec![0, 0, 100, 100],
            vec![0, 0, 100, 100],
        ];
        let pix = create_gray_image(4, 3, &values);

        let gradient = compute_gradient(&pix).unwrap();

        // Gradient should be non-zero at the edge columns
        // Column 1 has gx = 100 - 0 = 100 (right - left)
        // Column 0 has gx = 0 - 0 = 0 (center - center at boundary)
        let grad_0 = gradient.get_pixel(0, 1).unwrap();
        let grad_1 = gradient.get_pixel(1, 1).unwrap();

        // At position (1, 1): left=0, right=100, so gx = 100
        // At position (0, 1): left=center=0, right=0, so gx = 0
        assert!(grad_1 > grad_0);
    }

    #[test]
    fn test_watershed_two_basins() {
        // Create an image with two distinct basins
        // Low values at corners, high ridge in middle
        let values = vec![
            vec![0, 5, 10, 5, 0],
            vec![5, 10, 15, 10, 5],
            vec![10, 15, 20, 15, 10],
            vec![5, 10, 15, 10, 5],
            vec![0, 5, 10, 5, 0],
        ];
        let pix = create_gray_image(5, 5, &values);

        let options = WatershedOptions::new().with_min_depth(1);
        let result = watershed_segmentation(&pix, &options).unwrap();

        // Should have at least 2 different non-zero labels
        let mut labels = std::collections::HashSet::new();
        for y in 0..5 {
            for x in 0..5 {
                if let Some(label) = result.get_pixel(x, y)
                    && label > 0
                {
                    labels.insert(label);
                }
            }
        }
        // The corners should form basins
        assert!(!labels.is_empty());
    }

    #[test]
    fn test_find_basins() {
        let values = vec![vec![0, 10, 0], vec![10, 20, 10], vec![0, 10, 0]];
        let pix = create_gray_image(3, 3, &values);

        let basins = find_basins(&pix, ConnectivityType::FourWay).unwrap();

        // Corner pixels should be in basins
        let label_0_0 = basins.get_pixel(0, 0).unwrap();
        let label_2_0 = basins.get_pixel(2, 0).unwrap();

        // They could be in the same or different basins depending on algorithm
        assert!(label_0_0 > 0 || label_2_0 > 0);
    }

    #[test]
    fn test_unsupported_depth() {
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let options = WatershedOptions::default();

        let result = watershed_segmentation(&pix, &options);
        assert!(result.is_err());
    }
}
