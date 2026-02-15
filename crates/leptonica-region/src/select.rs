//! Component selection by size
//!
//! This module provides functions for selecting connected components from
//! binary images based on bounding box dimensions.
//!
//! C equivalent: `pixSelectBySize()` and related functions in `pixafunc1.c`

use crate::conncomp::{ConnectivityType, find_connected_components, label_connected_components};
use crate::error::{RegionError, RegionResult};
use leptonica_core::{Pix, PixelDepth};
use std::collections::HashSet;

/// Selection type for component filtering by bounding box dimensions.
///
/// Determines how width and height thresholds are combined when deciding
/// whether to keep a component.
///
/// C equivalent: `L_SELECT_IF_BOTH` / `L_SELECT_IF_EITHER` in Leptonica
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeSelectType {
    /// Select if BOTH width and height satisfy the relation
    IfBoth,
    /// Select if EITHER width or height satisfies the relation
    IfEither,
}

/// Selection relation for component filtering.
///
/// Determines the comparison operator used against the threshold.
///
/// C equivalent: `L_SELECT_IF_GTE` / `L_SELECT_IF_LTE` in Leptonica
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeSelectRelation {
    /// Select if greater than or equal to threshold
    Gte,
    /// Select if less than or equal to threshold
    Lte,
}

/// Select connected components from a binary image by bounding box size.
///
/// Finds connected components in the input binary image and returns a new
/// binary image containing only those components whose bounding box dimensions
/// satisfy the given size constraint.
///
/// Uses a labeled image internally to correctly identify which pixels belong
/// to each component (avoids accidentally merging components with overlapping
/// bounding boxes).
///
/// C equivalent: `pixSelectBySize()` in `pixafunc1.c`
///
/// # Arguments
///
/// * `pixs` - Input binary image (1-bit depth)
/// * `width_thresh` - Width threshold for selection
/// * `height_thresh` - Height threshold for selection
/// * `connectivity` - Type of connectivity (4-way or 8-way)
/// * `select_type` - How to combine width/height criteria ([`SizeSelectType`])
/// * `relation` - Comparison relation ([`SizeSelectRelation`])
///
/// # Returns
///
/// A new 1-bit binary image containing only the selected components.
///
/// # Errors
///
/// Returns an error if the input image is not 1-bit depth.
///
/// # Examples
///
/// ```
/// use leptonica_core::{Pix, PixelDepth};
/// use leptonica_region::{
///     ConnectivityType, SizeSelectType, SizeSelectRelation, pix_select_by_size,
///     find_connected_components,
/// };
///
/// // Create a binary image with components of different sizes
/// let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
/// let mut pix_mut = pix.try_into_mut().unwrap();
///
/// // Large 5x5 block
/// for y in 0..5 {
///     for x in 0..5 {
///         pix_mut.set_pixel(x, y, 1).unwrap();
///     }
/// }
/// // Small 2x2 block
/// for y in 20..22 {
///     for x in 20..22 {
///         pix_mut.set_pixel(x, y, 1).unwrap();
///     }
/// }
///
/// let pix: Pix = pix_mut.into();
///
/// // Keep only components where both dimensions >= 4
/// let result = pix_select_by_size(
///     &pix, 4, 4,
///     ConnectivityType::FourWay,
///     SizeSelectType::IfBoth,
///     SizeSelectRelation::Gte,
/// ).unwrap();
///
/// let comps = find_connected_components(&result, ConnectivityType::FourWay).unwrap();
/// assert_eq!(comps.len(), 1); // Only the 5x5 block remains
/// ```
pub fn pix_select_by_size(
    pixs: &Pix,
    width_thresh: i32,
    height_thresh: i32,
    connectivity: ConnectivityType,
    select_type: SizeSelectType,
    relation: SizeSelectRelation,
) -> RegionResult<Pix> {
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixs.depth().bits(),
        });
    }

    let w = pixs.width();
    let h = pixs.height();

    // Find connected components with bounding boxes
    let components = find_connected_components(pixs, connectivity)?;

    if components.is_empty() {
        // No components: return a copy of the input (matches C behavior)
        return Pix::new(w, h, PixelDepth::Bit1).map_err(RegionError::Core);
    }

    // Get the labeled image so we can identify which component each pixel belongs to
    let labeled = label_connected_components(pixs, connectivity)?;

    // Build a set of labels to keep
    let mut keep_labels = HashSet::new();

    for comp in &components {
        let cw = comp.bounds.w; // bounding box width
        let ch = comp.bounds.h; // bounding box height

        let keep = match (select_type, relation) {
            (SizeSelectType::IfBoth, SizeSelectRelation::Gte) => {
                cw >= width_thresh && ch >= height_thresh
            }
            (SizeSelectType::IfBoth, SizeSelectRelation::Lte) => {
                cw <= width_thresh && ch <= height_thresh
            }
            (SizeSelectType::IfEither, SizeSelectRelation::Gte) => {
                cw >= width_thresh || ch >= height_thresh
            }
            (SizeSelectType::IfEither, SizeSelectRelation::Lte) => {
                cw <= width_thresh || ch <= height_thresh
            }
        };

        if keep {
            keep_labels.insert(comp.label);
        }
    }

    // Create output image using labeled image to selectively copy pixels
    let mut output = Pix::new(w, h, PixelDepth::Bit1)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    for y in 0..h {
        for x in 0..w {
            if let Some(label) = labeled.get_pixel(x, y)
                && label > 0
                && keep_labels.contains(&label)
            {
                let _ = output.set_pixel(x, y, 1);
            }
        }
    }

    Ok(output.into())
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
    fn test_select_by_size_gte_both() {
        // Create two components: a 3x3 block and a 1x1 pixel
        let mut pixels = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                pixels.push((x, y));
            }
        }
        pixels.push((8, 8)); // single pixel

        let pix = create_test_image(10, 10, &pixels);

        // Keep only components where both w>=2 and h>=2
        let result = pix_select_by_size(
            &pix,
            2,
            2,
            ConnectivityType::FourWay,
            SizeSelectType::IfBoth,
            SizeSelectRelation::Gte,
        )
        .unwrap();

        let comps = find_connected_components(&result, ConnectivityType::FourWay).unwrap();
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].bounds.w, 3);
        assert_eq!(comps[0].bounds.h, 3);
    }

    #[test]
    fn test_select_by_size_lte_both() {
        // Create two components: a 3x3 block and a 1x1 pixel
        let mut pixels = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                pixels.push((x, y));
            }
        }
        pixels.push((8, 8)); // single pixel

        let pix = create_test_image(10, 10, &pixels);

        // Keep only components where both w<=1 and h<=1
        let result = pix_select_by_size(
            &pix,
            1,
            1,
            ConnectivityType::FourWay,
            SizeSelectType::IfBoth,
            SizeSelectRelation::Lte,
        )
        .unwrap();

        let comps = find_connected_components(&result, ConnectivityType::FourWay).unwrap();
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].pixel_count, 1);
    }

    #[test]
    fn test_select_by_size_gte_either() {
        // Create a 4x1 horizontal bar, a 1x4 vertical bar, and a 1x1 pixel
        let mut pixels = Vec::new();
        // Horizontal bar: w=4, h=1
        for x in 0..4 {
            pixels.push((x, 0));
        }
        // Vertical bar: w=1, h=4
        for y in 5..9 {
            pixels.push((0, y));
        }
        // Single pixel: w=1, h=1
        pixels.push((8, 8));

        let pix = create_test_image(10, 10, &pixels);

        // Keep if either w>=3 or h>=3
        let result = pix_select_by_size(
            &pix,
            3,
            3,
            ConnectivityType::FourWay,
            SizeSelectType::IfEither,
            SizeSelectRelation::Gte,
        )
        .unwrap();

        let comps = find_connected_components(&result, ConnectivityType::FourWay).unwrap();
        // Both the horizontal bar (w=4>=3) and vertical bar (h=4>=3) should be kept
        assert_eq!(comps.len(), 2);
    }

    #[test]
    fn test_select_by_size_empty_image() {
        let pix = create_test_image(10, 10, &[]);

        let result = pix_select_by_size(
            &pix,
            5,
            5,
            ConnectivityType::FourWay,
            SizeSelectType::IfBoth,
            SizeSelectRelation::Gte,
        )
        .unwrap();

        let comps = find_connected_components(&result, ConnectivityType::FourWay).unwrap();
        assert!(comps.is_empty());
    }

    #[test]
    fn test_select_by_size_wrong_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = pix_select_by_size(
            &pix,
            5,
            5,
            ConnectivityType::FourWay,
            SizeSelectType::IfBoth,
            SizeSelectRelation::Gte,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_select_preserves_dimensions() {
        let pix = create_test_image(100, 80, &[(10, 10), (50, 50)]);

        let result = pix_select_by_size(
            &pix,
            1,
            1,
            ConnectivityType::FourWay,
            SizeSelectType::IfBoth,
            SizeSelectRelation::Gte,
        )
        .unwrap();

        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 80);
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }
}
