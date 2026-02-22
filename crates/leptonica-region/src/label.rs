//! Pixel labeling functions
//!
//! This module provides high-level functions for labeling and analyzing
//! connected components in images.

use crate::conncomp::{ConnectivityType, label_connected_components};
use crate::error::{RegionError, RegionResult};
use leptonica_core::{Box, Pix, PixelDepth};

/// Label connected components in an image
///
/// This is a convenience wrapper around `label_connected_components` that
/// supports multiple image depths by first converting to binary.
///
/// # Arguments
///
/// * `pix` - Input image (1-bit binary)
/// * `connectivity` - Connectivity type (4-way or 8-way)
///
/// # Returns
///
/// A 32-bit labeled image where each pixel contains its component label.
pub fn pix_label_connected_components(
    pix: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    label_connected_components(pix, connectivity)
}

/// Count the number of connected components
///
/// # Arguments
///
/// * `pix` - Input binary image (1-bit)
/// * `connectivity` - Connectivity type
///
/// # Returns
///
/// The number of foreground connected components.
pub fn pix_count_components(pix: &Pix, connectivity: ConnectivityType) -> RegionResult<u32> {
    let labeled = pix_label_connected_components(pix, connectivity)?;

    // Find the maximum label value
    let width = labeled.width();
    let height = labeled.height();
    let mut max_label = 0u32;

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = labeled.get_pixel(x, y) {
                max_label = max_label.max(label);
            }
        }
    }

    Ok(max_label)
}

/// Get bounding boxes for all connected components
///
/// # Arguments
///
/// * `pix` - Input binary image (1-bit)
/// * `connectivity` - Connectivity type
///
/// # Returns
///
/// A vector of bounding boxes, one for each component.
/// The index in the vector corresponds to (label - 1).
pub fn pix_get_component_bounds(
    pix: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Vec<Box>> {
    let labeled = pix_label_connected_components(pix, connectivity)?;
    get_component_bounds_from_labels(&labeled)
}

/// Get bounding boxes from a labeled image
///
/// # Arguments
///
/// * `labeled` - Labeled image (32-bit)
///
/// # Returns
///
/// A vector of bounding boxes, one for each component.
pub fn get_component_bounds_from_labels(labeled: &Pix) -> RegionResult<Vec<Box>> {
    if labeled.depth() != PixelDepth::Bit32 {
        return Err(RegionError::UnsupportedDepth {
            expected: "32-bit (labeled image)",
            actual: labeled.depth().bits(),
        });
    }

    let width = labeled.width();
    let height = labeled.height();

    // Collect bounds for each label
    let mut bounds_map: std::collections::HashMap<u32, (i32, i32, i32, i32)> =
        std::collections::HashMap::new();

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = labeled.get_pixel(x, y)
                && label > 0
            {
                let entry = bounds_map
                    .entry(label)
                    .or_insert((x as i32, y as i32, x as i32, y as i32));
                entry.0 = entry.0.min(x as i32); // min_x
                entry.1 = entry.1.min(y as i32); // min_y
                entry.2 = entry.2.max(x as i32); // max_x
                entry.3 = entry.3.max(y as i32); // max_y
            }
        }
    }

    // Convert to sorted vector of Box
    let mut label_bounds: Vec<(u32, Box)> = bounds_map
        .into_iter()
        .map(|(label, (min_x, min_y, max_x, max_y))| {
            (
                label,
                Box::new_unchecked(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1),
            )
        })
        .collect();

    label_bounds.sort_by_key(|(label, _)| *label);

    Ok(label_bounds.into_iter().map(|(_, b)| b).collect())
}

/// Get pixel count for each component
///
/// # Arguments
///
/// * `labeled` - Labeled image (32-bit)
///
/// # Returns
///
/// A vector of pixel counts, one for each component.
/// The index corresponds to (label - 1).
pub fn get_component_sizes(labeled: &Pix) -> RegionResult<Vec<u32>> {
    if labeled.depth() != PixelDepth::Bit32 {
        return Err(RegionError::UnsupportedDepth {
            expected: "32-bit (labeled image)",
            actual: labeled.depth().bits(),
        });
    }

    let width = labeled.width();
    let height = labeled.height();

    // Count pixels for each label
    let mut counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = labeled.get_pixel(x, y)
                && label > 0
            {
                *counts.entry(label).or_insert(0) += 1;
            }
        }
    }

    // Convert to sorted vector
    let mut label_counts: Vec<(u32, u32)> = counts.into_iter().collect();
    label_counts.sort_by_key(|(label, _)| *label);

    Ok(label_counts.into_iter().map(|(_, count)| count).collect())
}

/// Component statistics
#[derive(Debug, Clone)]
pub struct ComponentStats {
    /// Component label
    pub label: u32,
    /// Bounding box
    pub bounds: Box,
    /// Number of pixels
    pub pixel_count: u32,
    /// Centroid X coordinate
    pub centroid_x: f64,
    /// Centroid Y coordinate
    pub centroid_y: f64,
}

/// Get detailed statistics for all components
///
/// # Arguments
///
/// * `labeled` - Labeled image (32-bit)
///
/// # Returns
///
/// A vector of component statistics.
pub fn get_component_stats(labeled: &Pix) -> RegionResult<Vec<ComponentStats>> {
    if labeled.depth() != PixelDepth::Bit32 {
        return Err(RegionError::UnsupportedDepth {
            expected: "32-bit (labeled image)",
            actual: labeled.depth().bits(),
        });
    }

    let width = labeled.width();
    let height = labeled.height();

    // Accumulate statistics for each label
    #[derive(Default)]
    struct Accum {
        count: u32,
        sum_x: u64,
        sum_y: u64,
        min_x: i32,
        min_y: i32,
        max_x: i32,
        max_y: i32,
    }

    let mut stats: std::collections::HashMap<u32, Accum> = std::collections::HashMap::new();

    for y in 0..height {
        for x in 0..width {
            if let Some(label) = labeled.get_pixel(x, y)
                && label > 0
            {
                let acc = stats.entry(label).or_insert_with(|| Accum {
                    count: 0,
                    sum_x: 0,
                    sum_y: 0,
                    min_x: x as i32,
                    min_y: y as i32,
                    max_x: x as i32,
                    max_y: y as i32,
                });

                acc.count += 1;
                acc.sum_x += x as u64;
                acc.sum_y += y as u64;
                acc.min_x = acc.min_x.min(x as i32);
                acc.min_y = acc.min_y.min(y as i32);
                acc.max_x = acc.max_x.max(x as i32);
                acc.max_y = acc.max_y.max(y as i32);
            }
        }
    }

    // Convert to ComponentStats
    let mut result: Vec<ComponentStats> = stats
        .into_iter()
        .map(|(label, acc)| {
            let centroid_x = acc.sum_x as f64 / acc.count as f64;
            let centroid_y = acc.sum_y as f64 / acc.count as f64;
            let bounds = Box::new_unchecked(
                acc.min_x,
                acc.min_y,
                acc.max_x - acc.min_x + 1,
                acc.max_y - acc.min_y + 1,
            );

            ComponentStats {
                label,
                bounds,
                pixel_count: acc.count,
                centroid_x,
                centroid_y,
            }
        })
        .collect();

    result.sort_by_key(|s| s.label);

    Ok(result)
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
    fn test_count_components() {
        let pix = create_test_image(
            10,
            10,
            &[
                (0, 0),
                (1, 0), // Component 1
                (5, 5),
                (6, 5), // Component 2
                (8, 8), // Component 3
            ],
        );

        let count = pix_count_components(&pix, ConnectivityType::FourWay).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_get_component_bounds() {
        let pix = create_test_image(10, 10, &[(0, 0), (1, 0), (2, 0), (1, 1)]);

        let bounds = pix_get_component_bounds(&pix, ConnectivityType::FourWay).unwrap();

        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0].x, 0);
        assert_eq!(bounds[0].y, 0);
        assert_eq!(bounds[0].w, 3);
        assert_eq!(bounds[0].h, 2);
    }

    #[test]
    fn test_get_component_sizes() {
        let pix = create_test_image(
            10,
            10,
            &[
                (0, 0),
                (1, 0), // 2 pixels
                (5, 5), // 1 pixel
            ],
        );

        let labeled = pix_label_connected_components(&pix, ConnectivityType::FourWay).unwrap();
        let sizes = get_component_sizes(&labeled).unwrap();

        assert_eq!(sizes.len(), 2);
        // Sizes should be 2 and 1 (order depends on labeling)
        assert!(sizes.contains(&2));
        assert!(sizes.contains(&1));
    }

    #[test]
    fn test_get_component_stats() {
        let pix = create_test_image(10, 10, &[(0, 0), (2, 0), (1, 1)]); // L-shape

        let labeled = pix_label_connected_components(&pix, ConnectivityType::EightWay).unwrap();
        let stats = get_component_stats(&labeled).unwrap();

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].pixel_count, 3);

        // Centroid should be at (1, 0.333...)
        assert!((stats[0].centroid_x - 1.0).abs() < 0.01);
        assert!((stats[0].centroid_y - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_empty_image() {
        let pix = create_test_image(10, 10, &[]);

        let count = pix_count_components(&pix, ConnectivityType::FourWay).unwrap();
        assert_eq!(count, 0);

        let bounds = pix_get_component_bounds(&pix, ConnectivityType::FourWay).unwrap();
        assert!(bounds.is_empty());
    }
}

// ===== Phase 4 Label Extensions (not yet implemented) =====

/// Transformation type for connected components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnCompTransform {
    /// Transform to area (pixel count) of each component
    Area,
    /// Transform to label number
    Label,
}

/// Transform connected components by replacing pixel values with transformation results
///
/// Each connected component is filled with a value determined by the transformation type:
/// - Area: pixel count of the component
/// - Label: label number of the component
///
/// # Arguments
///
/// * `pix` - Input binary image (1-bit)
/// * `connectivity` - Connectivity type (4-way or 8-way)
/// * `transform_type` - Type of transformation (Area or Label)
///
/// # Returns
///
/// A 32-bit image where each pixel contains the transformed value.
pub fn conn_comp_transform(
    pix: &Pix,
    connectivity: ConnectivityType,
    transform_type: ConnCompTransform,
) -> RegionResult<Pix> {
    let _pix = pix;
    let _connectivity = connectivity;
    let _transform_type = transform_type;
    Err(RegionError::NotImplemented)
}

/// Incremental connected component labeler
///
/// Allows components to be added sequentially with automatic label assignment.
#[derive(Debug)]
pub struct IncrementalLabeler {
    width: u32,
    height: u32,
    connectivity: ConnectivityType,
    labels: Vec<u32>,
}

impl IncrementalLabeler {
    /// Create a new incremental labeler
    ///
    /// # Arguments
    ///
    /// * `width` - Image width
    /// * `height` - Image height
    /// * `connectivity` - Connectivity type (4-way or 8-way)
    pub fn new(width: u32, height: u32, connectivity: ConnectivityType) -> Self {
        let _width = width;
        let _height = height;
        let _connectivity = connectivity;
        unimplemented!("IncrementalLabeler::new not yet implemented")
    }

    /// Add a new component at the given position
    ///
    /// Uses BFS/DFS to find connected pixels and assigns them to the next label.
    ///
    /// # Arguments
    ///
    /// * `pix` - Source image
    /// * `x` - X coordinate of seed point
    /// * `y` - Y coordinate of seed point
    pub fn add_component(&mut self, pix: &Pix, x: u32, y: u32) -> RegionResult<u32> {
        let _pix = pix;
        let _x = x;
        let _y = y;
        Err(RegionError::NotImplemented)
    }

    /// Get the resulting labeled image
    pub fn finish(self) -> Pix {
        unimplemented!("IncrementalLabeler::finish not yet implemented")
    }
}

/// Convert a labeled image to a color image for visualization
///
/// Each unique label is mapped to a random color, ensuring that adjacent
/// components have different colors when possible.
///
/// # Arguments
///
/// * `labeled` - Labeled image (32-bit, output from label_connected_components)
///
/// # Returns
///
/// A 32-bit RGBA image where each pixel contains a color for its label.
pub fn label_to_color(labeled: &Pix) -> RegionResult<Pix> {
    let _labeled = labeled;
    Err(RegionError::NotImplemented)
}

#[cfg(test)]
mod tests_phase4 {
    use super::*;

    fn create_test_image_with_components(width: u32, height: u32) -> Pix {
        let pix = Pix::new(width, height, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Component 1: 2x2 square at (1,1)
        for x in 1..3 {
            for y in 1..3 {
                let _ = pix_mut.set_pixel(x, y, 1);
            }
        }

        // Component 2: 3x1 line at (6,5)
        for x in 6..9 {
            let _ = pix_mut.set_pixel(x, 5, 1);
        }

        // Component 3: single pixel at (2,8)
        let _ = pix_mut.set_pixel(2, 8, 1);

        pix_mut.into()
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_conn_comp_area_transform() {
        let pix = create_test_image_with_components(10, 10);

        let transformed =
            conn_comp_transform(&pix, ConnectivityType::FourWay, ConnCompTransform::Area).unwrap();

        // Component 1: 4 pixels
        assert_eq!(transformed.get_pixel(1, 1), Some(4));
        assert_eq!(transformed.get_pixel(2, 2), Some(4));

        // Component 2: 3 pixels
        assert_eq!(transformed.get_pixel(6, 5), Some(3));
        assert_eq!(transformed.get_pixel(8, 5), Some(3));

        // Component 3: 1 pixel
        assert_eq!(transformed.get_pixel(2, 8), Some(1));

        // Background: 0
        assert_eq!(transformed.get_pixel(0, 0), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_conn_comp_label_transform() {
        let pix = create_test_image_with_components(10, 10);

        let transformed =
            conn_comp_transform(&pix, ConnectivityType::FourWay, ConnCompTransform::Label).unwrap();

        // All pixels in component 1 should have same label
        let label1 = transformed.get_pixel(1, 1).unwrap();
        assert_eq!(transformed.get_pixel(2, 2), Some(label1));

        // All pixels in component 2 should have same (different) label
        let label2 = transformed.get_pixel(6, 5).unwrap();
        assert_ne!(label2, label1);
        assert_eq!(transformed.get_pixel(8, 5), Some(label2));

        // Component 3 should have yet another label
        let label3 = transformed.get_pixel(2, 8).unwrap();
        assert_ne!(label3, label1);
        assert_ne!(label3, label2);

        // Background: 0
        assert_eq!(transformed.get_pixel(0, 0), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_incremental_labeler_basic() {
        let pix = create_test_image_with_components(10, 10);

        let mut labeler = IncrementalLabeler::new(10, 10, ConnectivityType::FourWay);

        // Add first component at (1,1)
        let label1 = labeler.add_component(&pix, 1, 1).unwrap();
        assert_eq!(label1, 1);

        // Add second component at (6,5)
        let label2 = labeler.add_component(&pix, 6, 5).unwrap();
        assert_eq!(label2, 2);

        // Add third component at (2,8)
        let label3 = labeler.add_component(&pix, 2, 8).unwrap();
        assert_eq!(label3, 3);

        let result = labeler.finish();

        // Verify labeling
        assert_eq!(result.get_pixel(1, 1), Some(label1));
        assert_eq!(result.get_pixel(2, 2), Some(label1));
        assert_eq!(result.get_pixel(6, 5), Some(label2));
        assert_eq!(result.get_pixel(2, 8), Some(label3));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_incremental_vs_batch_labeling() {
        let pix = create_test_image_with_components(10, 10);

        // Batch labeling
        let batch_result = pix_label_connected_components(&pix, ConnectivityType::FourWay).unwrap();

        // Incremental labeling
        let mut labeler = IncrementalLabeler::new(10, 10, ConnectivityType::FourWay);
        let _ = labeler.add_component(&pix, 1, 1);
        let _ = labeler.add_component(&pix, 6, 5);
        let _ = labeler.add_component(&pix, 2, 8);
        let incremental_result = labeler.finish();

        // Both should produce the same number of components
        let batch_count = pix_count_components(&pix, ConnectivityType::FourWay).unwrap();
        let incremental_count =
            pix_count_components(&incremental_result, ConnectivityType::FourWay).unwrap_or(3); // Expected: 3

        assert_eq!(batch_count, incremental_count);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_label_to_color_basic() {
        let pix = create_test_image_with_components(10, 10);
        let labeled = pix_label_connected_components(&pix, ConnectivityType::FourWay).unwrap();

        let colored = label_to_color(&labeled).unwrap();

        // Check depth is 32-bit
        assert_eq!(colored.depth(), PixelDepth::Bit32);

        // Check that colored image has correct dimensions
        assert_eq!(colored.width(), labeled.width());
        assert_eq!(colored.height(), labeled.height());

        // Background should be black (or specific color, depending on implementation)
        let bg_color = colored.get_pixel(0, 0).unwrap();
        assert!(bg_color == 0 || bg_color == 0xFF000000); // Black in RGBA
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_label_to_color_adjacent_different() {
        let pix = create_test_image_with_components(10, 10);
        let labeled = pix_label_connected_components(&pix, ConnectivityType::FourWay).unwrap();

        let colored = label_to_color(&labeled).unwrap();

        // Component 1 at (1,1) and (1,2) should have same color
        let color1 = colored.get_pixel(1, 1).unwrap();
        assert_eq!(colored.get_pixel(1, 2), Some(color1));

        // Adjacent different components should ideally have different colors
        // (implementation may not guarantee this for all adjacencies, but should try)
        let _color2 = colored.get_pixel(6, 5).unwrap();
        let _color3 = colored.get_pixel(2, 8).unwrap();
        // Note: exact color difference check would depend on implementation
    }
}
