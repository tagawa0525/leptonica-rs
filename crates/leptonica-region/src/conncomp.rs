//! Connected component analysis
//!
//! This module provides functions for finding and labeling connected components
//! in binary images. It uses Union-Find (disjoint set) data structure for
//! efficient labeling.

use crate::error::{RegionError, RegionResult};
use leptonica_core::{Box, Boxa, Pix, Pixa, PixelDepth};

/// Connectivity type for component analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectivityType {
    /// 4-way connectivity (up, down, left, right)
    #[default]
    FourWay,
    /// 8-way connectivity (includes diagonals)
    EightWay,
}

/// A connected component in an image
#[derive(Debug, Clone)]
pub struct ConnectedComponent {
    /// Unique label for this component
    pub label: u32,
    /// Number of pixels in this component
    pub pixel_count: u32,
    /// Bounding box of this component
    pub bounds: Box,
}

impl ConnectedComponent {
    /// Create a new connected component
    pub fn new(label: u32, pixel_count: u32, bounds: Box) -> Self {
        Self {
            label,
            pixel_count,
            bounds,
        }
    }
}

/// Union-Find data structure for efficient connected component labeling
struct UnionFind {
    parent: Vec<u32>,
    rank: Vec<u32>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size as u32).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, x: u32) -> u32 {
        if self.parent[x as usize] != x {
            self.parent[x as usize] = self.find(self.parent[x as usize]);
        }
        self.parent[x as usize]
    }

    fn union(&mut self, x: u32, y: u32) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x != root_y {
            let rank_x = self.rank[root_x as usize];
            let rank_y = self.rank[root_y as usize];

            if rank_x < rank_y {
                self.parent[root_x as usize] = root_y;
            } else if rank_x > rank_y {
                self.parent[root_y as usize] = root_x;
            } else {
                self.parent[root_y as usize] = root_x;
                self.rank[root_x as usize] += 1;
            }
        }
    }
}

/// Find connected components in a binary image
///
/// Returns a vector of connected components found in the foreground (1 pixels).
///
/// # Arguments
///
/// * `pix` - Input binary image (1-bit depth)
/// * `connectivity` - Type of connectivity (4-way or 8-way)
///
/// # Returns
///
/// A vector of `ConnectedComponent` structures describing each component.
///
/// # Errors
///
/// Returns an error if the image is not 1-bit depth.
pub fn find_connected_components(
    pix: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Vec<ConnectedComponent>> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    if width == 0 || height == 0 {
        return Ok(Vec::new());
    }

    // Label the image first
    let labeled = label_connected_components(pix, connectivity)?;

    // Extract component information from labeled image
    extract_components_from_labels(&labeled)
}

/// Label connected components in a binary image
///
/// Creates a labeled image where each connected component has a unique label.
/// Background pixels have label 0.
///
/// # Arguments
///
/// * `pix` - Input binary image (1-bit depth)
/// * `connectivity` - Type of connectivity (4-way or 8-way)
///
/// # Returns
///
/// A 32-bit image where each pixel contains its component label (0 for background).
///
/// # Errors
///
/// Returns an error if the image is not 1-bit depth.
pub fn label_connected_components(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    // Create output image with 32-bit depth for labels
    let mut output = Pix::new(width, height, PixelDepth::Bit32)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    if width == 0 || height == 0 {
        return Ok(output.into());
    }

    // Maximum possible labels (worst case: every other pixel is a separate component)
    let max_labels = ((width as usize) * (height as usize) / 2) + 1;
    let mut uf = UnionFind::new(max_labels);
    let mut next_label: u32 = 1;

    // First pass: assign provisional labels and record equivalences
    for y in 0..height {
        for x in 0..width {
            // Skip background pixels
            if pix.get_pixel(x, y).unwrap_or(0) == 0 {
                continue;
            }

            let mut neighbors = Vec::with_capacity(4);

            // Check neighbors that have already been processed (above and left)
            // For 4-way: check left and top
            // For 8-way: check left, top-left, top, top-right

            // Left neighbor
            if x > 0
                && let Some(label) = output.get_pixel(x - 1, y)
                && label > 0
            {
                neighbors.push(label);
            }

            // Top neighbor
            if y > 0
                && let Some(label) = output.get_pixel(x, y - 1)
                && label > 0
            {
                neighbors.push(label);
            }

            if connectivity == ConnectivityType::EightWay {
                // Top-left neighbor
                if x > 0
                    && y > 0
                    && let Some(label) = output.get_pixel(x - 1, y - 1)
                    && label > 0
                {
                    neighbors.push(label);
                }

                // Top-right neighbor
                if x + 1 < width
                    && y > 0
                    && let Some(label) = output.get_pixel(x + 1, y - 1)
                    && label > 0
                {
                    neighbors.push(label);
                }
            }

            if neighbors.is_empty() {
                // New component
                let _ = output.set_pixel(x, y, next_label);
                next_label += 1;
            } else {
                // Find minimum label among neighbors
                let min_label = *neighbors.iter().min().unwrap();
                let _ = output.set_pixel(x, y, min_label);

                // Union all neighbor labels
                for &label in &neighbors {
                    uf.union(min_label, label);
                }
            }
        }
    }

    // Second pass: resolve labels using union-find
    // Create a mapping from root labels to sequential labels
    let mut label_map = std::collections::HashMap::new();
    let mut final_label: u32 = 1;

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = output.get_pixel(x, y)
                && label > 0
            {
                let root = uf.find(label);
                let mapped = *label_map.entry(root).or_insert_with(|| {
                    let l = final_label;
                    final_label += 1;
                    l
                });
                let _ = output.set_pixel(x, y, mapped);
            }
        }
    }

    Ok(output.into())
}

/// Extract component information from a labeled image
fn extract_components_from_labels(labeled: &Pix) -> RegionResult<Vec<ConnectedComponent>> {
    let width = labeled.width();
    let height = labeled.height();

    // Collect statistics for each label
    let mut stats: std::collections::HashMap<u32, (u32, i32, i32, i32, i32)> =
        std::collections::HashMap::new();

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = labeled.get_pixel(x, y)
                && label > 0
            {
                let entry = stats
                    .entry(label)
                    .or_insert((0, x as i32, y as i32, x as i32, y as i32));
                entry.0 += 1; // pixel count
                entry.1 = entry.1.min(x as i32); // min_x
                entry.2 = entry.2.min(y as i32); // min_y
                entry.3 = entry.3.max(x as i32); // max_x
                entry.4 = entry.4.max(y as i32); // max_y
            }
        }
    }

    // Convert to ConnectedComponent structures
    let mut components: Vec<ConnectedComponent> = stats
        .into_iter()
        .map(|(label, (count, min_x, min_y, max_x, max_y))| {
            let bounds = Box::new_unchecked(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1);
            ConnectedComponent::new(label, count, bounds)
        })
        .collect();

    // Sort by label for consistent ordering
    components.sort_by_key(|c| c.label);

    Ok(components)
}

/// Extract a single component from a labeled image
///
/// Creates a binary image containing only the specified component.
///
/// # Arguments
///
/// * `labeled` - Labeled image (from `label_connected_components`)
/// * `label` - Label of the component to extract
///
/// # Returns
///
/// A 1-bit binary image containing only the specified component.
pub fn extract_component(labeled: &Pix, label: u32) -> RegionResult<Pix> {
    if labeled.depth() != PixelDepth::Bit32 {
        return Err(RegionError::UnsupportedDepth {
            expected: "32-bit (labeled image)",
            actual: labeled.depth().bits(),
        });
    }

    let width = labeled.width();
    let height = labeled.height();

    let mut output = Pix::new(width, height, PixelDepth::Bit1)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    for y in 0..height {
        for x in 0..width {
            if let Some(pixel_label) = labeled.get_pixel(x, y)
                && pixel_label == label
            {
                let _ = output.set_pixel(x, y, 1);
            }
        }
    }

    Ok(output.into())
}

/// Filter components by size
///
/// Removes components that don't meet the size criteria.
///
/// # Arguments
///
/// * `labeled` - Labeled image
/// * `min_size` - Minimum pixel count (components smaller than this are removed)
/// * `max_size` - Maximum pixel count (components larger than this are removed), 0 means no limit
///
/// # Returns
///
/// A new labeled image with filtered components.
pub fn filter_components_by_size(labeled: &Pix, min_size: u32, max_size: u32) -> RegionResult<Pix> {
    if labeled.depth() != PixelDepth::Bit32 {
        return Err(RegionError::UnsupportedDepth {
            expected: "32-bit (labeled image)",
            actual: labeled.depth().bits(),
        });
    }

    let width = labeled.width();
    let height = labeled.height();

    // Count pixels for each label
    let mut label_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = labeled.get_pixel(x, y)
                && label > 0
            {
                *label_counts.entry(label).or_insert(0) += 1;
            }
        }
    }

    // Determine which labels to keep
    let valid_labels: std::collections::HashSet<u32> = label_counts
        .into_iter()
        .filter(|&(_, count)| count >= min_size && (max_size == 0 || count <= max_size))
        .map(|(label, _)| label)
        .collect();

    // Create output with filtered labels
    let mut output = Pix::new(width, height, PixelDepth::Bit32)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    // Remap labels to sequential values
    let mut label_remap: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut next_label = 1u32;

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = labeled.get_pixel(x, y)
                && label > 0
                && valid_labels.contains(&label)
            {
                let new_label = *label_remap.entry(label).or_insert_with(|| {
                    let l = next_label;
                    next_label += 1;
                    l
                });
                let _ = output.set_pixel(x, y, new_label);
            }
        }
    }

    Ok(output.into())
}

/// Transform labeled image to area values
///
/// Creates an image where each pixel contains the area (pixel count) of its component.
///
/// # Arguments
///
/// * `labeled` - Labeled image (from `label_connected_components`)
///
/// # Returns
///
/// A 32-bit image where each pixel contains its component's pixel count.
pub fn component_area_transform(labeled: &Pix) -> RegionResult<Pix> {
    if labeled.depth() != PixelDepth::Bit32 {
        return Err(RegionError::UnsupportedDepth {
            expected: "32-bit (labeled image)",
            actual: labeled.depth().bits(),
        });
    }

    let width = labeled.width();
    let height = labeled.height();

    // Count pixels for each label
    let mut label_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = labeled.get_pixel(x, y)
                && label > 0
            {
                *label_counts.entry(label).or_insert(0) += 1;
            }
        }
    }

    // Create output with area values
    let mut output = Pix::new(width, height, PixelDepth::Bit32)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = labeled.get_pixel(x, y)
                && label > 0
            {
                let area = label_counts.get(&label).copied().unwrap_or(0);
                let _ = output.set_pixel(x, y, area);
            }
        }
    }

    Ok(output.into())
}

/// Find connected components and return as Pixa with bounding boxes
///
/// Finds all foreground connected components in a binary image and returns
/// them as a `Pixa` (array of component images clipped to their bounding boxes)
/// along with a `Boxa` of the bounding boxes in the original image.
///
/// This is the Rust equivalent of C Leptonica's `pixConnCompPixa`.
///
/// # Arguments
///
/// * `pix` - Input binary image (1-bit depth)
/// * `connectivity` - Type of connectivity (4-way or 8-way)
///
/// # Returns
///
/// A tuple of `(Boxa, Pixa)` where each entry corresponds to one connected
/// component. The Pixa images are clipped to the bounding box of each component.
///
/// # Errors
///
/// Returns an error if the image is not 1-bit depth.
pub fn conncomp_pixa(_pix: &Pix, _connectivity: ConnectivityType) -> RegionResult<(Boxa, Pixa)> {
    todo!("not yet implemented")
}

/// Get unique sorted neighbor label values at a pixel location
///
/// For a labeled image (8, 16, or 32 bpp), returns the unique non-zero
/// label values of the neighbors of the pixel at (x, y), sorted in
/// ascending order.
///
/// This is the Rust equivalent of C Leptonica's `pixGetSortedNeighborValues`.
///
/// # Arguments
///
/// * `pix` - Labeled image (8, 16, or 32 bpp)
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `connectivity` - 4-way or 8-way connectivity
///
/// # Returns
///
/// A sorted `Vec<u32>` of unique non-zero neighbor values. Empty if no
/// non-zero neighbors exist.
///
/// # Errors
///
/// Returns an error if depth is less than 8 bpp.
pub fn get_sorted_neighbor_values(
    _pix: &Pix,
    _x: u32,
    _y: u32,
    _connectivity: ConnectivityType,
) -> RegionResult<Vec<u32>> {
    todo!("not yet implemented")
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
    fn test_single_component_4way() {
        // Create a 2x2 square
        let pix = create_test_image(10, 10, &[(1, 1), (2, 1), (1, 2), (2, 2)]);

        let components = find_connected_components(&pix, ConnectivityType::FourWay).unwrap();

        assert_eq!(components.len(), 1);
        assert_eq!(components[0].pixel_count, 4);
        assert_eq!(components[0].bounds.x, 1);
        assert_eq!(components[0].bounds.y, 1);
        assert_eq!(components[0].bounds.w, 2);
        assert_eq!(components[0].bounds.h, 2);
    }

    #[test]
    fn test_two_separate_components() {
        // Create two separate 2x2 squares
        let pix = create_test_image(
            10,
            10,
            &[
                (0, 0),
                (1, 0),
                (0, 1),
                (1, 1), // First square
                (5, 5),
                (6, 5),
                (5, 6),
                (6, 6), // Second square
            ],
        );

        let components = find_connected_components(&pix, ConnectivityType::FourWay).unwrap();

        assert_eq!(components.len(), 2);
        assert!(components.iter().all(|c| c.pixel_count == 4));
    }

    #[test]
    fn test_diagonal_4way_vs_8way() {
        // Create two diagonally adjacent pixels
        let pix = create_test_image(10, 10, &[(0, 0), (1, 1)]);

        // 4-way should see them as separate
        let components_4 = find_connected_components(&pix, ConnectivityType::FourWay).unwrap();
        assert_eq!(components_4.len(), 2);

        // 8-way should see them as one component
        let components_8 = find_connected_components(&pix, ConnectivityType::EightWay).unwrap();
        assert_eq!(components_8.len(), 1);
        assert_eq!(components_8[0].pixel_count, 2);
    }

    #[test]
    fn test_empty_image() {
        let pix = create_test_image(10, 10, &[]);
        let components = find_connected_components(&pix, ConnectivityType::FourWay).unwrap();
        assert!(components.is_empty());
    }

    #[test]
    fn test_label_connected_components() {
        let pix = create_test_image(10, 10, &[(0, 0), (1, 0), (5, 5)]);

        let labeled = label_connected_components(&pix, ConnectivityType::FourWay).unwrap();

        assert_eq!(labeled.depth(), PixelDepth::Bit32);
        // (0,0) and (1,0) should have the same label
        let label_0_0 = labeled.get_pixel(0, 0).unwrap();
        let label_1_0 = labeled.get_pixel(1, 0).unwrap();
        let label_5_5 = labeled.get_pixel(5, 5).unwrap();

        assert!(label_0_0 > 0);
        assert_eq!(label_0_0, label_1_0);
        assert_ne!(label_0_0, label_5_5);
    }

    #[test]
    fn test_extract_component() {
        let pix = create_test_image(10, 10, &[(0, 0), (1, 0), (5, 5)]);
        let labeled = label_connected_components(&pix, ConnectivityType::FourWay).unwrap();

        let label = labeled.get_pixel(0, 0).unwrap();
        let extracted = extract_component(&labeled, label).unwrap();

        assert_eq!(extracted.depth(), PixelDepth::Bit1);
        assert_eq!(extracted.get_pixel(0, 0), Some(1));
        assert_eq!(extracted.get_pixel(1, 0), Some(1));
        assert_eq!(extracted.get_pixel(5, 5), Some(0));
    }

    #[test]
    fn test_filter_by_size() {
        // Create components of different sizes
        let pix = create_test_image(
            20,
            10,
            &[
                (0, 0), // 1 pixel
                (5, 0),
                (6, 0),
                (5, 1),
                (6, 1), // 4 pixels
                (10, 0),
                (11, 0),
                (12, 0),
                (10, 1),
                (11, 1),
                (12, 1),
                (10, 2),
                (11, 2),
                (12, 2), // 9 pixels
            ],
        );

        let labeled = label_connected_components(&pix, ConnectivityType::FourWay).unwrap();

        // Filter to keep only components with 2-5 pixels
        let filtered = filter_components_by_size(&labeled, 2, 5).unwrap();

        // Count non-zero pixels (should only be the 4-pixel component)
        let mut count = 0;
        for y in 0..10 {
            for x in 0..20 {
                if filtered.get_pixel(x, y).unwrap_or(0) > 0 {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_component_area_transform() {
        let pix = create_test_image(10, 10, &[(0, 0), (1, 0), (5, 5)]);
        let labeled = label_connected_components(&pix, ConnectivityType::FourWay).unwrap();
        let area_image = component_area_transform(&labeled).unwrap();

        // (0,0) and (1,0) form a 2-pixel component
        assert_eq!(area_image.get_pixel(0, 0), Some(2));
        assert_eq!(area_image.get_pixel(1, 0), Some(2));

        // (5,5) is a 1-pixel component
        assert_eq!(area_image.get_pixel(5, 5), Some(1));

        // Background should be 0
        assert_eq!(area_image.get_pixel(3, 3), Some(0));
    }

    #[test]
    fn test_unsupported_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = find_connected_components(&pix, ConnectivityType::FourWay);
        assert!(result.is_err());
    }

    #[test]
    fn test_l_shaped_component() {
        // Create an L-shaped component
        let pix = create_test_image(10, 10, &[(0, 0), (0, 1), (0, 2), (1, 2), (2, 2)]);

        let components = find_connected_components(&pix, ConnectivityType::FourWay).unwrap();

        assert_eq!(components.len(), 1);
        assert_eq!(components[0].pixel_count, 5);
        assert_eq!(components[0].bounds.x, 0);
        assert_eq!(components[0].bounds.y, 0);
        assert_eq!(components[0].bounds.w, 3);
        assert_eq!(components[0].bounds.h, 3);
    }
}
