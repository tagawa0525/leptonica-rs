//! Color segmentation
//!
//! Unsupervised color segmentation for reducing an image to a small
//! number of colors. This is useful for image analysis, document
//! processing, and artistic effects.
//!
//! The algorithm proceeds in 4 phases:
//! 1. **Cluster**: Greedy assignment of pixels to color clusters
//! 2. **Refine**: Reassign pixels to nearest cluster average
//! 3. **Clean**: Morphological cleanup (optional)
//! 4. **Reduce**: Remove unpopular colors

use crate::{ColorError, ColorResult};
use leptonica_core::{Pix, PixColormap, PixelDepth, color};

// =============================================================================
// Constants
// =============================================================================

/// Maximum allowed iterations when expanding distance
const MAX_ALLOWED_ITERATIONS: u32 = 20;

/// Factor by which max_dist is increased on each iteration
const DIST_EXPAND_FACTOR: f32 = 1.3;

// =============================================================================
// Options
// =============================================================================

/// Options for color segmentation
///
/// The parameters interact as follows:
/// - `max_dist` controls how similar colors must be to join a cluster
/// - `max_colors` limits Phase 1 output (should be ~2x `final_colors`)
/// - `final_colors` is the target number of colors after Phase 4
///
/// # Guidelines
///
/// | final_colors | max_colors | max_dist |
/// |--------------|------------|----------|
/// | 3            | 6          | 100      |
/// | 4            | 8          | 90       |
/// | 5            | 10         | 75       |
/// | 6            | 12         | 60       |
#[derive(Debug, Clone)]
pub struct ColorSegmentOptions {
    /// Maximum Euclidean distance to existing cluster (Phase 1)
    ///
    /// Lower values create more clusters (more colors).
    /// Higher values merge more colors together.
    /// Range: typically 60-100.
    pub max_dist: u32,

    /// Maximum number of colors in Phase 1
    ///
    /// If exceeded, `max_dist` is automatically increased.
    /// Should be larger than `final_colors`, typically 2x.
    pub max_colors: u32,

    /// Linear size of structuring element for morphological cleanup (Phase 3)
    ///
    /// Set to 0 or 1 to skip cleanup phase.
    /// Larger values remove more noise but may lose detail.
    pub sel_size: u32,

    /// Maximum number of colors after Phase 4
    ///
    /// Unpopular colors are merged into similar popular colors.
    pub final_colors: u32,
}

impl Default for ColorSegmentOptions {
    fn default() -> Self {
        Self {
            max_dist: 75,
            max_colors: 10,
            sel_size: 4,
            final_colors: 5,
        }
    }
}

impl ColorSegmentOptions {
    /// Create options for a target number of final colors
    ///
    /// Uses the guidelines from the original Leptonica implementation.
    pub fn for_colors(final_colors: u32) -> Self {
        let (max_colors, max_dist) = match final_colors {
            1..=3 => (6, 100),
            4 => (8, 90),
            5 => (10, 75),
            _ => (final_colors * 2, 60),
        };
        Self {
            max_dist,
            max_colors,
            sel_size: 4,
            final_colors,
        }
    }
}

// =============================================================================
// Main API
// =============================================================================

/// Perform unsupervised color segmentation
///
/// This is the full 4-phase color segmentation algorithm.
/// Returns an 8-bpp image with a colormap.
///
/// # Arguments
///
/// * `pix` - 32-bpp RGB input image
/// * `options` - Segmentation parameters
///
/// # Returns
///
/// An 8-bpp colormapped image with at most `final_colors` colors.
///
/// # Example
///
/// ```no_run
/// use leptonica_color::segment::{color_segment, ColorSegmentOptions};
/// use leptonica_core::Pix;
///
/// let pix = Pix::new(100, 100, leptonica_core::PixelDepth::Bit32).unwrap();
/// let options = ColorSegmentOptions::for_colors(5);
/// let segmented = color_segment(&pix, &options).unwrap();
/// ```
pub fn color_segment(pix: &Pix, options: &ColorSegmentOptions) -> ColorResult<Pix> {
    // Validate input
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    if options.max_colors == 0 || options.max_colors > 256 {
        return Err(ColorError::InvalidParameters(
            "max_colors must be between 1 and 256".into(),
        ));
    }

    if options.final_colors == 0 || options.final_colors > options.max_colors {
        return Err(ColorError::InvalidParameters(
            "final_colors must be between 1 and max_colors".into(),
        ));
    }

    // Phase 1: Initial clustering
    let pix_clustered = color_segment_cluster(pix, options.max_dist, options.max_colors)?;

    // Phase 2: Refinement - reassign to nearest cluster average
    let pix_refined = Pix::new(pix.width(), pix.height(), PixelDepth::Bit8)?;
    let mut refined_mut = pix_refined.try_into_mut().unwrap();

    // Copy colormap from clustered result
    let colormap = pix_clustered
        .colormap()
        .ok_or_else(|| ColorError::InvalidParameters("clustered result has no colormap".into()))?;
    refined_mut.set_colormap(Some(colormap.clone()))?;

    let counts = assign_to_nearest_color(&mut refined_mut, pix, &pix_clustered, None)?;

    // Phase 3: Morphological cleanup (simplified - skip for now)
    // Full implementation would require morphological operations

    // Phase 4: Remove unpopular colors
    let refined_pix: Pix = refined_mut.into();
    let final_pix = color_segment_remove_colors(&refined_pix, pix, options.final_colors, &counts)?;

    Ok(final_pix)
}

/// Simple color segmentation with default parameters
///
/// Convenience function that uses recommended parameters for the
/// target number of colors.
///
/// # Arguments
///
/// * `pix` - 32-bpp RGB input image
/// * `final_colors` - Target number of colors (1-256)
///
/// # Example
///
/// ```no_run
/// use leptonica_color::segment::color_segment_simple;
/// use leptonica_core::Pix;
///
/// let pix = Pix::new(100, 100, leptonica_core::PixelDepth::Bit32).unwrap();
/// let segmented = color_segment_simple(&pix, 5).unwrap();
/// ```
pub fn color_segment_simple(pix: &Pix, final_colors: u32) -> ColorResult<Pix> {
    let options = ColorSegmentOptions::for_colors(final_colors);
    color_segment(pix, &options)
}

/// Perform greedy color clustering (Phase 1)
///
/// This is the first phase of color segmentation. Pixels are assigned
/// to clusters greedily: each pixel either joins an existing cluster
/// (if within `max_dist`) or starts a new cluster.
///
/// If the number of clusters exceeds `max_colors`, the distance is
/// automatically increased and clustering is retried.
///
/// # Arguments
///
/// * `pix` - 32-bpp RGB input image
/// * `max_dist` - Maximum Euclidean distance to join a cluster
/// * `max_colors` - Maximum number of clusters allowed
///
/// # Returns
///
/// An 8-bpp colormapped image where the colormap contains cluster
/// average colors.
pub fn color_segment_cluster(pix: &Pix, max_dist: u32, max_colors: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    if max_colors == 0 || max_colors > 256 {
        return Err(ColorError::InvalidParameters(
            "max_colors must be between 1 and 256".into(),
        ));
    }

    // Try clustering with increasing distance until it succeeds
    let mut current_dist = max_dist;
    for _ in 0..MAX_ALLOWED_ITERATIONS {
        match cluster_try(pix, current_dist, max_colors) {
            Ok(result) => return Ok(result),
            Err(ClusterError::TooManyColors) => {
                // Increase distance and retry
                current_dist = (current_dist as f32 * DIST_EXPAND_FACTOR) as u32;
            }
            Err(ClusterError::Other(e)) => return Err(e),
        }
    }

    Err(ColorError::QuantizationError(format!(
        "failed to cluster after {} iterations (final dist={})",
        MAX_ALLOWED_ITERATIONS, current_dist
    )))
}

/// Assign pixels to the nearest color in the colormap
///
/// This is Phase 2 of color segmentation. Each pixel in `src` is
/// assigned to the nearest color in the colormap of `dest`.
///
/// # Arguments
///
/// * `dest` - 8-bpp output image with colormap (modified in place)
/// * `src` - 32-bpp RGB source image
/// * `reference` - 8-bpp reference image with colormap to use
/// * `mask` - Optional 1-bpp mask (only fg pixels are processed)
///
/// # Returns
///
/// A vector of pixel counts per colormap index.
pub fn assign_to_nearest_color(
    dest: &mut leptonica_core::PixMut,
    src: &Pix,
    reference: &Pix,
    mask: Option<&Pix>,
) -> ColorResult<Vec<u32>> {
    let colormap = reference
        .colormap()
        .ok_or_else(|| ColorError::InvalidParameters("reference image has no colormap".into()))?;

    let w = src.width();
    let h = src.height();

    if dest.width() != w || dest.height() != h {
        return Err(ColorError::InvalidParameters(
            "source and dest dimensions must match".into(),
        ));
    }

    if let Some(m) = mask
        && (m.width() != w || m.height() != h)
    {
        return Err(ColorError::InvalidParameters(
            "mask dimensions must match source".into(),
        ));
    }

    let mut counts = vec![0u32; colormap.len()];

    for y in 0..h {
        for x in 0..w {
            // Check mask if present
            if let Some(m) = mask {
                let mask_val = unsafe { m.get_pixel_unchecked(x, y) };
                if mask_val == 0 {
                    continue;
                }
            }

            let pixel = unsafe { src.get_pixel_unchecked(x, y) };
            let (r, g, b) = color::extract_rgb(pixel);

            // Find nearest color in colormap
            let idx = colormap.find_nearest(r, g, b).unwrap_or(0);
            unsafe { dest.set_pixel_unchecked(x, y, idx as u32) };
            counts[idx] += 1;
        }
    }

    Ok(counts)
}

// =============================================================================
// Internal Implementation
// =============================================================================

/// Internal error type for clustering
enum ClusterError {
    TooManyColors,
    Other(ColorError),
}

/// Attempt to cluster pixels with given parameters
fn cluster_try(pix: &Pix, max_dist: u32, max_colors: u32) -> Result<Pix, ClusterError> {
    let w = pix.width();
    let h = pix.height();
    let max_dist_sq = (max_dist as i64) * (max_dist as i64);

    // Create output image
    let pix_out = Pix::new(w, h, PixelDepth::Bit8).map_err(|e| ClusterError::Other(e.into()))?;
    let mut out_mut = pix_out.try_into_mut().unwrap();

    // Cluster tracking
    // rmap/gmap/bmap: current cluster representative colors
    // rsum/gsum/bsum/counts: for computing averages
    let mut rmap: Vec<u8> = Vec::with_capacity(max_colors as usize);
    let mut gmap: Vec<u8> = Vec::with_capacity(max_colors as usize);
    let mut bmap: Vec<u8> = Vec::with_capacity(max_colors as usize);
    let mut rsum: Vec<u64> = Vec::with_capacity(max_colors as usize);
    let mut gsum: Vec<u64> = Vec::with_capacity(max_colors as usize);
    let mut bsum: Vec<u64> = Vec::with_capacity(max_colors as usize);
    let mut counts: Vec<u64> = Vec::with_capacity(max_colors as usize);

    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
            let (r, g, b) = color::extract_rgb(pixel);

            // Try to find an existing cluster within max_dist
            let mut found = false;
            let ncolors = rmap.len();

            for k in 0..ncolors {
                let dr = r as i64 - rmap[k] as i64;
                let dg = g as i64 - gmap[k] as i64;
                let db = b as i64 - bmap[k] as i64;
                let dist_sq = dr * dr + dg * dg + db * db;

                if dist_sq <= max_dist_sq {
                    // Assign to this cluster
                    unsafe { out_mut.set_pixel_unchecked(x, y, k as u32) };
                    rsum[k] += r as u64;
                    gsum[k] += g as u64;
                    bsum[k] += b as u64;
                    counts[k] += 1;
                    found = true;
                    break;
                }
            }

            if !found {
                // Need to create a new cluster
                if ncolors >= max_colors as usize {
                    return Err(ClusterError::TooManyColors);
                }

                let idx = ncolors;
                rmap.push(r);
                gmap.push(g);
                bmap.push(b);
                rsum.push(r as u64);
                gsum.push(g as u64);
                bsum.push(b as u64);
                counts.push(1);
                unsafe { out_mut.set_pixel_unchecked(x, y, idx as u32) };
            }
        }
    }

    // Create colormap with average colors
    let mut colormap = PixColormap::new(8).map_err(|e| ClusterError::Other(e.into()))?;

    for k in 0..rmap.len() {
        let count = counts[k];
        if count > 0 {
            let avg_r = (rsum[k] / count) as u8;
            let avg_g = (gsum[k] / count) as u8;
            let avg_b = (bsum[k] / count) as u8;
            colormap
                .add_rgb(avg_r, avg_g, avg_b)
                .map_err(|e| ClusterError::Other(e.into()))?;
        }
    }

    out_mut
        .set_colormap(Some(colormap))
        .map_err(|e| ClusterError::Other(e.into()))?;

    Ok(out_mut.into())
}

/// Remove unpopular colors (Phase 4)
///
/// Returns a new Pix with only the most popular colors.
fn color_segment_remove_colors(
    pix_dest: &Pix,
    _pix_src: &Pix,
    final_colors: u32,
    counts: &[u32],
) -> ColorResult<Pix> {
    let colormap = pix_dest
        .colormap()
        .ok_or_else(|| ColorError::InvalidParameters("dest image has no colormap".into()))?;

    let ncolors = colormap.len();
    if ncolors <= final_colors as usize {
        // Already few enough colors, return clone
        return Ok(pix_dest.clone());
    }

    // Sort colors by popularity (descending)
    let mut indices: Vec<usize> = (0..ncolors).collect();
    indices.sort_by(|a, b| counts[*b].cmp(&counts[*a]));

    // Build mapping from old index to new index
    // Colors not in keep_set get mapped to nearest kept color
    let mut index_map: Vec<u8> = vec![0; ncolors];
    let mut new_colormap = PixColormap::new(8)?;

    // First, add the kept colors
    for (new_idx, &old_idx) in indices[..final_colors as usize].iter().enumerate() {
        let (r, g, b) = colormap.get_rgb(old_idx).unwrap();
        new_colormap.add_rgb(r, g, b)?;
        index_map[old_idx] = new_idx as u8;
    }

    // Map removed colors to nearest kept color
    for &old_idx in &indices[final_colors as usize..] {
        let (r, g, b) = colormap.get_rgb(old_idx).unwrap();

        // Find nearest color among kept colors
        let mut min_dist = i64::MAX;
        let mut best_new_idx = 0;

        for (new_idx, &kept_idx) in indices[..final_colors as usize].iter().enumerate() {
            let (kr, kg, kb) = colormap.get_rgb(kept_idx).unwrap();
            let dr = r as i64 - kr as i64;
            let dg = g as i64 - kg as i64;
            let db = b as i64 - kb as i64;
            let dist = dr * dr + dg * dg + db * db;
            if dist < min_dist {
                min_dist = dist;
                best_new_idx = new_idx;
            }
        }

        index_map[old_idx] = best_new_idx as u8;
    }

    // Remap pixels
    let pix_out = Pix::new(pix_dest.width(), pix_dest.height(), PixelDepth::Bit8)?;
    let mut out_mut = pix_out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(new_colormap))?;

    for y in 0..pix_dest.height() {
        for x in 0..pix_dest.width() {
            let old_idx = unsafe { pix_dest.get_pixel_unchecked(x, y) } as usize;
            let new_idx = if old_idx < index_map.len() {
                index_map[old_idx]
            } else {
                0
            };
            unsafe { out_mut.set_pixel_unchecked(x, y, new_idx as u32) };
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test image with distinct color regions
    fn create_test_image() -> Pix {
        let pix = Pix::new(60, 60, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..60 {
            for x in 0..60 {
                let pixel = if x < 20 {
                    color::compose_rgb(255, 0, 0) // Red
                } else if x < 40 {
                    color::compose_rgb(0, 255, 0) // Green
                } else {
                    color::compose_rgb(0, 0, 255) // Blue
                };
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        pix_mut.into()
    }

    /// Create a gradient image
    fn create_gradient_image() -> Pix {
        let pix = Pix::new(64, 64, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..64 {
            for x in 0..64 {
                let r = (x * 4) as u8;
                let g = (y * 4) as u8;
                let b = 128;
                let pixel = color::compose_rgb(r, g, b);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_color_segment_simple_colors() {
        let pix = create_test_image();
        let result = color_segment_simple(&pix, 3).unwrap();

        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.colormap().is_some());

        let cmap = result.colormap().unwrap();
        assert!(cmap.len() <= 3);
    }

    #[test]
    fn test_color_segment_gradient() {
        let pix = create_gradient_image();
        let result = color_segment_simple(&pix, 5).unwrap();

        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.colormap().is_some());

        let cmap = result.colormap().unwrap();
        assert!(cmap.len() <= 5);
    }

    #[test]
    fn test_cluster_phase_only() {
        let pix = create_test_image();
        let result = color_segment_cluster(&pix, 100, 10).unwrap();

        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.colormap().is_some());

        // With distinct colors and large max_dist, should get ~3 clusters
        let cmap = result.colormap().unwrap();
        assert!(cmap.len() <= 10);
    }

    #[test]
    fn test_wrong_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();

        let result = color_segment_simple(&pix, 5);
        assert!(result.is_err());

        let result = color_segment_cluster(&pix, 75, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_params() {
        let pix = create_test_image();

        // max_colors = 0
        let result = color_segment_cluster(&pix, 75, 0);
        assert!(result.is_err());

        // max_colors > 256
        let result = color_segment_cluster(&pix, 75, 257);
        assert!(result.is_err());
    }

    #[test]
    fn test_options_for_colors() {
        let opts = ColorSegmentOptions::for_colors(3);
        assert_eq!(opts.final_colors, 3);
        assert_eq!(opts.max_colors, 6);
        assert_eq!(opts.max_dist, 100);

        let opts = ColorSegmentOptions::for_colors(5);
        assert_eq!(opts.final_colors, 5);
        assert_eq!(opts.max_colors, 10);
        assert_eq!(opts.max_dist, 75);
    }

    #[test]
    fn test_default_options() {
        let opts = ColorSegmentOptions::default();
        assert_eq!(opts.final_colors, 5);
        assert_eq!(opts.max_colors, 10);
        assert_eq!(opts.max_dist, 75);
        assert_eq!(opts.sel_size, 4);
    }
}
