//! Connectivity-preserving thinning operations
//!
//! Thinning reduces binary images to 1-pixel wide skeletons while preserving
//! connectivity. This module implements algorithms based on:
//!
//! "Connectivity-preserving morphological image transformations"
//! Dan S. Bloomberg, SPIE Visual Communications and Image Processing,
//! Conference 1606, pp. 320-334, November 1991, Boston, MA.
//! (http://www.leptonica.com/papers/conn.pdf)
//!
//! # Algorithm
//!
//! The thinning algorithm uses an iterative approach:
//! 1. For each iteration, apply the SEL set in 4 orthogonal rotations
//! 2. For each rotation, compute the union of HMT results from all SELs
//! 3. Subtract the accumulated result from the image
//! 4. Repeat until no changes occur or max iterations reached
//!
//! # Reference
//!
//! Based on Leptonica's `ccthin.c` implementation.

use crate::thin_sels::{ThinSelSet, make_thin_sels};
use crate::{MorphError, MorphResult, Sel, hit_miss_transform};
use leptonica_core::{Pix, PixelDepth};

/// Type of thinning operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThinType {
    /// Thin the foreground (normal thinning)
    ///
    /// Reduces foreground regions to 1-pixel wide skeletons.
    #[default]
    Foreground,

    /// Thin the background (equivalent to thickening foreground)
    ///
    /// Expands foreground regions by thinning the background.
    /// The alternate connectivity is preserved.
    Background,
}

/// Connectivity type for thinning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Connectivity {
    /// 4-connected (preserves 4-connectivity)
    #[default]
    Four,

    /// 8-connected (preserves 8-connectivity)
    Eight,
}

/// Default maximum iterations for thinning
const DEFAULT_MAX_ITERS: u32 = 10000;

/// Thin a binary image while preserving connectivity
///
/// This is a simple interface for connectivity-preserving thinning.
/// It uses the recommended SEL sets for smooth skeletons.
///
/// # Arguments
///
/// * `pix` - 1-bpp binary image
/// * `thin_type` - Whether to thin foreground or background
/// * `connectivity` - 4 or 8 connectivity to preserve
/// * `max_iters` - Maximum number of iterations (0 = until convergence)
///
/// # Returns
///
/// A new thinned image, or error if input is not 1-bpp.
///
/// # Example
///
/// ```ignore
/// use leptonica_morph::{thin_connected, ThinType, Connectivity};
///
/// let thinned = thin_connected(&pix, ThinType::Foreground, Connectivity::Four, 0)?;
/// ```
///
/// # Notes
///
/// * For 4-connected thinning, uses SEL set 1 (sel_4_1, sel_4_2, sel_4_3)
/// * For 8-connected thinning, uses SEL set 5 (sel_8_2, sel_8_3, sel_8_5, sel_8_6)
/// * Duality: thinning the background with 8-connectivity is equivalent to
///   thickening the foreground while preserving 4-connectivity
pub fn thin_connected(
    pix: &Pix,
    thin_type: ThinType,
    connectivity: Connectivity,
    max_iters: u32,
) -> MorphResult<Pix> {
    check_binary(pix)?;

    // Select the appropriate SEL set based on connectivity
    let sel_set = match connectivity {
        Connectivity::Four => ThinSelSet::Set4cc1,
        Connectivity::Eight => ThinSelSet::Set8cc1,
    };

    let sels = make_thin_sels(sel_set);
    thin_connected_by_set(pix, thin_type, &sels, max_iters)
}

/// Thin a binary image using a specific SEL set
///
/// This provides more control over the thinning algorithm by allowing
/// selection of specific SEL sets.
///
/// # Arguments
///
/// * `pix` - 1-bpp binary image
/// * `thin_type` - Whether to thin foreground or background
/// * `sels` - Array of SELs for parallel composite HMTs
/// * `max_iters` - Maximum number of iterations (0 = until convergence)
///
/// # Returns
///
/// A new thinned image, or error if input is not 1-bpp.
///
/// # Algorithm
///
/// For each iteration:
/// 1. For each of 4 rotations (0, 90, 180, 270 degrees):
///    a. For each SEL in the set, compute HMT with rotated SEL
///    b. OR all HMT results together
///    c. Subtract accumulated result from the image
/// 2. If image unchanged, stop (converged)
///
/// # Notes
///
/// * The "parallel" operations compute HMTs on the same source and accumulate
///   results before subtracting from the source
/// * The "sequential" operations apply 4 thinnings (one per rotation direction)
///   in sequence, each modifying the image
pub fn thin_connected_by_set(
    pix: &Pix,
    thin_type: ThinType,
    sels: &[Sel],
    max_iters: u32,
) -> MorphResult<Pix> {
    check_binary(pix)?;

    if sels.is_empty() {
        return Err(MorphError::InvalidParameters(
            "SEL set cannot be empty".to_string(),
        ));
    }

    let max_iters = if max_iters == 0 {
        DEFAULT_MAX_ITERS
    } else {
        max_iters
    };

    // Set up initial image
    let mut pixd = if thin_type == ThinType::Foreground {
        pix.clone()
    } else {
        invert(pix)?
    };

    // Thin the foreground with up to max_iters iterations
    for _iter in 0..max_iters {
        let pix_prev = pixd.clone();

        // For each of 4 rotations
        for rotation in 0..4 {
            // Compute HMT for each SEL and accumulate
            let mut accumulated: Option<Pix> = None;

            for sel in sels {
                let rotated_sel = sel.rotate_orth(rotation);
                let hmt_result = hit_miss_transform(&pixd, &rotated_sel)?;

                accumulated = Some(match accumulated {
                    None => hmt_result,
                    Some(acc) => or_images(&acc, &hmt_result)?,
                });
            }

            // Subtract accumulated result from image
            if let Some(acc) = accumulated {
                pixd = subtract_images(&pixd, &acc)?;
            }
        }

        // Check for convergence
        if images_equal(&pixd, &pix_prev) {
            break;
        }
    }

    // Post-processing for background thinning
    if thin_type == ThinType::Background {
        pixd = invert(&pixd)?;
        // Remove border-connected components that were created by thickening
        let diff = subtract_images(&pixd, pix)?;
        let border_cc = extract_border_connected_components(&diff)?;
        pixd = subtract_images(&pixd, &border_cc)?;
    }

    Ok(pixd)
}

/// Invert a binary image
fn invert(pix: &Pix) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            let inverted = if val == 0 { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, inverted);
        }
    }

    Ok(out_mut.into())
}

/// OR two binary images
fn or_images(a: &Pix, b: &Pix) -> MorphResult<Pix> {
    let w = a.width();
    let h = a.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let va = a.get_pixel_unchecked(x, y);
            let vb = b.get_pixel_unchecked(x, y);
            let result = if va != 0 || vb != 0 { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Subtract two binary images (a AND NOT b)
fn subtract_images(a: &Pix, b: &Pix) -> MorphResult<Pix> {
    let w = a.width();
    let h = a.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let va = a.get_pixel_unchecked(x, y);
            let vb = b.get_pixel_unchecked(x, y);
            let result = if va != 0 && vb == 0 { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Check if two binary images are equal
fn images_equal(a: &Pix, b: &Pix) -> bool {
    if a.width() != b.width() || a.height() != b.height() {
        return false;
    }

    for y in 0..a.height() {
        for x in 0..a.width() {
            let va = a.get_pixel_unchecked(x, y);
            let vb = b.get_pixel_unchecked(x, y);
            if va != vb {
                return false;
            }
        }
    }

    true
}

/// Extract connected components that touch the border
///
/// Uses 4-connectivity for flood fill from border pixels.
fn extract_border_connected_components(pix: &Pix) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Track which pixels have been visited
    let mut visited = vec![false; (w * h) as usize];

    // Find border pixels that are foreground and flood fill from them
    let mut seeds: Vec<(u32, u32)> = Vec::new();

    // Top and bottom borders
    for x in 0..w {
        if pix.get_pixel_unchecked(x, 0) != 0 {
            seeds.push((x, 0));
        }
        if h > 1 && pix.get_pixel_unchecked(x, h - 1) != 0 {
            seeds.push((x, h - 1));
        }
    }

    // Left and right borders (excluding corners already added)
    for y in 1..h.saturating_sub(1) {
        if pix.get_pixel_unchecked(0, y) != 0 {
            seeds.push((0, y));
        }
        if w > 1 && pix.get_pixel_unchecked(w - 1, y) != 0 {
            seeds.push((w - 1, y));
        }
    }

    // Flood fill from each seed using 4-connectivity
    while let Some((x, y)) = seeds.pop() {
        let idx = (y * w + x) as usize;
        if visited[idx] {
            continue;
        }
        if pix.get_pixel_unchecked(x, y) == 0 {
            continue;
        }

        visited[idx] = true;
        out_mut.set_pixel_unchecked(x, y, 1);

        // Add 4-connected neighbors
        if x > 0 {
            seeds.push((x - 1, y));
        }
        if x + 1 < w {
            seeds.push((x + 1, y));
        }
        if y > 0 {
            seeds.push((x, y - 1));
        }
        if y + 1 < h {
            seeds.push((x, y + 1));
        }
    }

    Ok(out_mut.into())
}

/// Check that the image is binary (1-bpp)
fn check_binary(pix: &Pix) -> MorphResult<()> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_vertical_line(width: u32, height: u32) -> Pix {
        let pix = Pix::new(width, height, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let center_x = width / 2;
        for y in 0..height {
            pix_mut.set_pixel_unchecked(center_x, y, 1);
        }

        pix_mut.into()
    }

    fn create_horizontal_line(width: u32, height: u32) -> Pix {
        let pix = Pix::new(width, height, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let center_y = height / 2;
        for x in 0..width {
            pix_mut.set_pixel_unchecked(x, center_y, 1);
        }

        pix_mut.into()
    }

    fn create_thick_horizontal_line(width: u32, height: u32, thickness: u32) -> Pix {
        let pix = Pix::new(width, height, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let center_y = height / 2;
        let half_thick = thickness / 2;

        for y in center_y.saturating_sub(half_thick)..=(center_y + half_thick).min(height - 1) {
            for x in 0..width {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }

        pix_mut.into()
    }

    fn count_foreground_pixels(pix: &Pix) -> u32 {
        let mut count = 0;
        for y in 0..pix.height() {
            for x in 0..pix.width() {
                if pix.get_pixel_unchecked(x, y) != 0 {
                    count += 1;
                }
            }
        }
        count
    }

    #[test]
    fn test_thin_vertical_line() {
        // A 1-pixel wide vertical line should remain unchanged
        let pix = create_vertical_line(9, 9);
        let original_count = count_foreground_pixels(&pix);

        let thinned = thin_connected(&pix, ThinType::Foreground, Connectivity::Four, 0).unwrap();

        // Line should remain connected
        let thinned_count = count_foreground_pixels(&thinned);
        assert!(thinned_count > 0);
        assert!(thinned_count <= original_count);
    }

    #[test]
    fn test_thin_horizontal_line() {
        // A 1-pixel wide horizontal line should remain unchanged
        let pix = create_horizontal_line(9, 9);
        let original_count = count_foreground_pixels(&pix);

        let thinned = thin_connected(&pix, ThinType::Foreground, Connectivity::Four, 0).unwrap();

        // Line should remain connected
        let thinned_count = count_foreground_pixels(&thinned);
        assert!(thinned_count > 0);
        assert!(thinned_count <= original_count);
    }

    #[test]
    fn test_thin_thick_line() {
        // A thick horizontal line should be thinned to 1 pixel wide
        let pix = create_thick_horizontal_line(15, 15, 5);
        let original_count = count_foreground_pixels(&pix);

        let thinned = thin_connected(&pix, ThinType::Foreground, Connectivity::Four, 0).unwrap();

        let thinned_count = count_foreground_pixels(&thinned);

        // Should be significantly reduced
        assert!(thinned_count < original_count);
        // But should still have some pixels (the skeleton)
        assert!(thinned_count > 0);
    }

    #[test]
    fn test_thin_8_connected() {
        let pix = create_thick_horizontal_line(15, 15, 5);

        let thinned = thin_connected(&pix, ThinType::Foreground, Connectivity::Eight, 0).unwrap();

        // Should produce a skeleton
        let thinned_count = count_foreground_pixels(&thinned);
        assert!(thinned_count > 0);
    }

    #[test]
    fn test_thin_with_max_iters() {
        let pix = create_thick_horizontal_line(15, 15, 5);

        // With only 1 iteration, shouldn't fully thin
        let thinned_1 = thin_connected(&pix, ThinType::Foreground, Connectivity::Four, 1).unwrap();

        // With unlimited iterations
        let thinned_full =
            thin_connected(&pix, ThinType::Foreground, Connectivity::Four, 0).unwrap();

        let count_1 = count_foreground_pixels(&thinned_1);
        let count_full = count_foreground_pixels(&thinned_full);

        // 1 iteration should result in more pixels than full thinning
        assert!(count_1 >= count_full);
    }

    #[test]
    fn test_invert() {
        let pix = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(1, 1, 1);

        let pix: Pix = pix_mut.into();

        let inverted = invert(&pix).unwrap();

        // Center should be 0, others should be 1
        assert_eq!(inverted.get_pixel_unchecked(1, 1), 0);
        assert_eq!(inverted.get_pixel_unchecked(0, 0), 1);
        assert_eq!(inverted.get_pixel_unchecked(2, 2), 1);
    }

    #[test]
    fn test_or_images() {
        let pix1 = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.try_into_mut().unwrap();
        pix1_mut.set_pixel_unchecked(0, 0, 1);

        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix2.try_into_mut().unwrap();
        pix2_mut.set_pixel_unchecked(2, 2, 1);

        let pix2: Pix = pix2_mut.into();

        let result = or_images(&pix1, &pix2).unwrap();

        assert_eq!(result.get_pixel_unchecked(0, 0), 1);
        assert_eq!(result.get_pixel_unchecked(2, 2), 1);
        assert_eq!(result.get_pixel_unchecked(1, 1), 0);
    }

    #[test]
    fn test_subtract_images() {
        let pix1 = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.try_into_mut().unwrap();
        pix1_mut.set_pixel_unchecked(0, 0, 1);
        pix1_mut.set_pixel_unchecked(1, 1, 1);

        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix2.try_into_mut().unwrap();
        pix2_mut.set_pixel_unchecked(1, 1, 1);

        let pix2: Pix = pix2_mut.into();

        let result = subtract_images(&pix1, &pix2).unwrap();

        assert_eq!(result.get_pixel_unchecked(0, 0), 1); // In a, not in b
        assert_eq!(result.get_pixel_unchecked(1, 1), 0); // In both
    }

    #[test]
    fn test_images_equal() {
        let pix1 = create_vertical_line(5, 5);
        let pix2 = create_vertical_line(5, 5);
        let pix3 = create_horizontal_line(5, 5);

        assert!(images_equal(&pix1, &pix2));
        assert!(!images_equal(&pix1, &pix3));
    }

    #[test]
    fn test_empty_sel_set_error() {
        let pix = create_vertical_line(5, 5);
        let result = thin_connected_by_set(&pix, ThinType::Foreground, &[], 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_binary_error() {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let result = thin_connected(&pix, ThinType::Foreground, Connectivity::Four, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_border_connected() {
        // Create image with border-touching and non-touching components
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Border-touching component (top-left)
        pix_mut.set_pixel_unchecked(0, 0, 1);
        pix_mut.set_pixel_unchecked(1, 0, 1);
        pix_mut.set_pixel_unchecked(0, 1, 1);

        // Non-border-touching component (center)
        pix_mut.set_pixel_unchecked(2, 2, 1);

        let pix: Pix = pix_mut.into();

        let border_cc = extract_border_connected_components(&pix).unwrap();

        // Border component should be extracted
        assert_eq!(border_cc.get_pixel_unchecked(0, 0), 1);
        assert_eq!(border_cc.get_pixel_unchecked(1, 0), 1);
        assert_eq!(border_cc.get_pixel_unchecked(0, 1), 1);

        // Center pixel should NOT be in result
        assert_eq!(border_cc.get_pixel_unchecked(2, 2), 0);
    }
}
