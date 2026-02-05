//! Rank filtering (order statistic filters)
//!
//! Rank filtering evaluates, for each pixel, a rectangular neighborhood
//! and outputs the pixel value at a specified rank position in the sorted order.
//!
//! # Algorithm
//!
//! Uses a two-histogram approach for efficiency:
//! - Coarse histogram (16 bins): Quick search for the approximate rank position
//! - Fine histogram (256 bins): Precise rank value determination
//!
//! With incremental updates, this achieves O(1) per-pixel complexity regardless
//! of filter size.
//!
//! # Example
//!
//! ```ignore
//! use leptonica_filter::rank_filter;
//!
//! // Apply median filter (rank = 0.5)
//! let median = median_filter(&pix, 3, 3)?;
//!
//! // Apply minimum filter (rank = 0.0)
//! let min = min_filter(&pix, 3, 3)?;
//!
//! // Apply maximum filter (rank = 1.0)
//! let max = max_filter(&pix, 3, 3)?;
//!
//! // Apply custom rank filter
//! let ranked = rank_filter(&pix, 5, 5, 0.25)?;
//! ```

use crate::{FilterError, FilterResult};
use leptonica_core::{Pix, PixelDepth, color};

/// Two-histogram structure for efficient rank computation
///
/// Uses a coarse histogram (16 bins) for quick search and a fine histogram
/// (256 bins) for precise value determination.
struct RankHistogram {
    /// Coarse histogram with 16 bins (each bin covers 16 intensity values)
    coarse: [u32; 16],
    /// Fine histogram with 256 bins (one per intensity value)
    fine: [u32; 256],
}

impl RankHistogram {
    /// Create a new empty histogram
    fn new() -> Self {
        Self {
            coarse: [0; 16],
            fine: [0; 256],
        }
    }

    /// Clear the histogram
    fn clear(&mut self) {
        self.coarse.fill(0);
        self.fine.fill(0);
    }

    /// Add a pixel value to the histogram
    #[inline]
    fn add(&mut self, value: u8) {
        let idx = value as usize;
        self.fine[idx] += 1;
        self.coarse[idx >> 4] += 1;
    }

    /// Remove a pixel value from the histogram
    #[inline]
    fn remove(&mut self, value: u8) {
        let idx = value as usize;
        debug_assert!(self.fine[idx] > 0, "Removing value not in histogram");
        debug_assert!(self.coarse[idx >> 4] > 0, "Coarse histogram underflow");
        self.fine[idx] = self.fine[idx].saturating_sub(1);
        self.coarse[idx >> 4] = self.coarse[idx >> 4].saturating_sub(1);
    }

    /// Get the value at the specified rank position
    ///
    /// # Arguments
    /// * `rank_position` - The 0-based position in sorted order (0 = minimum, count-1 = maximum)
    ///
    /// # Returns
    /// The intensity value at the specified rank position
    fn get_rank_value(&self, rank_position: u32) -> u8 {
        let mut sum = 0u32;

        // Search coarse histogram first
        let mut coarse_bin = 0usize;
        for i in 0..16 {
            sum += self.coarse[i];
            if sum > rank_position {
                sum -= self.coarse[i];
                coarse_bin = i;
                break;
            }
            if i == 15 {
                coarse_bin = 15;
                sum -= self.coarse[15];
            }
        }

        // Search fine histogram within the coarse bin
        let start = coarse_bin * 16;
        for i in 0..16 {
            let idx = start + i;
            if idx >= 256 {
                return 255;
            }
            sum += self.fine[idx];
            if sum > rank_position {
                return idx as u8;
            }
        }

        // Fallback to maximum value in the bin
        (coarse_bin * 16 + 15).min(255) as u8
    }
}

/// Apply rank filter to an image
///
/// The rank filter evaluates a rectangular neighborhood for each pixel and
/// outputs the value at the specified rank position.
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale or 32bpp color image
/// * `width` - Filter width (must be >= 1)
/// * `height` - Filter height (must be >= 1)
/// * `rank` - Rank value in [0.0, 1.0] where 0.0=minimum, 0.5=median, 1.0=maximum
///
/// # Returns
/// Filtered image with same dimensions and depth as input
///
/// # Example
/// ```ignore
/// let filtered = rank_filter(&pix, 5, 5, 0.5)?; // median filter
/// ```
pub fn rank_filter(pix: &Pix, width: u32, height: u32, rank: f32) -> FilterResult<Pix> {
    // Validate parameters
    if width < 1 || height < 1 {
        return Err(FilterError::InvalidParameters(
            "filter dimensions must be >= 1".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&rank) {
        return Err(FilterError::InvalidParameters(
            "rank must be in [0.0, 1.0]".to_string(),
        ));
    }

    // No-op for 1x1 filter
    if width == 1 && height == 1 {
        return Ok(pix.deep_clone());
    }

    match pix.depth() {
        PixelDepth::Bit8 => rank_filter_gray(pix, width, height, rank),
        PixelDepth::Bit32 => rank_filter_color(pix, width, height, rank),
        _ => Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

/// Apply rank filter to an 8bpp grayscale image
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale image
/// * `width` - Filter width (must be >= 1)
/// * `height` - Filter height (must be >= 1)
/// * `rank` - Rank value in [0.0, 1.0]
///
/// # Returns
/// Filtered 8bpp grayscale image
pub fn rank_filter_gray(pix: &Pix, width: u32, height: u32, rank: f32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }

    // Validate parameters
    if width < 1 || height < 1 {
        return Err(FilterError::InvalidParameters(
            "filter dimensions must be >= 1".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&rank) {
        return Err(FilterError::InvalidParameters(
            "rank must be in [0.0, 1.0]".to_string(),
        ));
    }

    // No-op for 1x1 filter
    if width == 1 && height == 1 {
        return Ok(pix.deep_clone());
    }

    let img_w = pix.width();
    let img_h = pix.height();
    let wf = width;
    let hf = height;

    // Calculate filter half sizes (for centering)
    let half_w = (wf / 2) as i32;
    let half_h = (hf / 2) as i32;

    // Calculate rank position
    let filter_size = wf * hf;
    let rank_position = ((rank * (filter_size - 1) as f32) + 0.5) as u32;

    // Create output image
    let out_pix = Pix::new(img_w, img_h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let mut histogram = RankHistogram::new();

    // Choose traversal order based on filter dimensions
    if hf > wf {
        // Row-major scanning (column-wise iteration is faster)
        for x in 0..img_w {
            histogram.clear();

            for y in 0..img_h {
                if y == 0 {
                    // Initialize histogram for first row
                    for ky in 0..hf {
                        for kx in 0..wf {
                            let sx =
                                (x as i32 + kx as i32 - half_w).clamp(0, img_w as i32 - 1) as u32;
                            let sy = (ky as i32 - half_h).clamp(0, img_h as i32 - 1) as u32;
                            let val = unsafe { pix.get_pixel_unchecked(sx, sy) } as u8;
                            histogram.add(val);
                        }
                    }
                } else {
                    // Incremental update: remove top row, add bottom row
                    let old_y = (y as i32 - 1 - half_h).clamp(0, img_h as i32 - 1) as u32;
                    let new_y =
                        (y as i32 + hf as i32 - 1 - half_h).clamp(0, img_h as i32 - 1) as u32;

                    for kx in 0..wf {
                        let sx = (x as i32 + kx as i32 - half_w).clamp(0, img_w as i32 - 1) as u32;

                        let old_val = unsafe { pix.get_pixel_unchecked(sx, old_y) } as u8;
                        histogram.remove(old_val);

                        let new_val = unsafe { pix.get_pixel_unchecked(sx, new_y) } as u8;
                        histogram.add(new_val);
                    }
                }

                let result = histogram.get_rank_value(rank_position);
                unsafe { out_mut.set_pixel_unchecked(x, y, result as u32) };
            }
        }
    } else {
        // Column-major scanning (row-wise iteration is faster)
        for y in 0..img_h {
            histogram.clear();

            for x in 0..img_w {
                if x == 0 {
                    // Initialize histogram for first column
                    for ky in 0..hf {
                        for kx in 0..wf {
                            let sx = (kx as i32 - half_w).clamp(0, img_w as i32 - 1) as u32;
                            let sy =
                                (y as i32 + ky as i32 - half_h).clamp(0, img_h as i32 - 1) as u32;
                            let val = unsafe { pix.get_pixel_unchecked(sx, sy) } as u8;
                            histogram.add(val);
                        }
                    }
                } else {
                    // Incremental update: remove left column, add right column
                    let old_x = (x as i32 - 1 - half_w).clamp(0, img_w as i32 - 1) as u32;
                    let new_x =
                        (x as i32 + wf as i32 - 1 - half_w).clamp(0, img_w as i32 - 1) as u32;

                    for ky in 0..hf {
                        let sy = (y as i32 + ky as i32 - half_h).clamp(0, img_h as i32 - 1) as u32;

                        let old_val = unsafe { pix.get_pixel_unchecked(old_x, sy) } as u8;
                        histogram.remove(old_val);

                        let new_val = unsafe { pix.get_pixel_unchecked(new_x, sy) } as u8;
                        histogram.add(new_val);
                    }
                }

                let result = histogram.get_rank_value(rank_position);
                unsafe { out_mut.set_pixel_unchecked(x, y, result as u32) };
            }
        }
    }

    Ok(out_mut.into())
}

/// Apply rank filter to a 32bpp color image
///
/// Each color channel (R, G, B) is processed independently.
///
/// # Arguments
/// * `pix` - Input 32bpp color image
/// * `width` - Filter width (must be >= 1)
/// * `height` - Filter height (must be >= 1)
/// * `rank` - Rank value in [0.0, 1.0]
///
/// # Returns
/// Filtered 32bpp color image
pub fn rank_filter_color(pix: &Pix, width: u32, height: u32, rank: f32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32-bpp color",
            actual: pix.depth().bits(),
        });
    }

    // Validate parameters
    if width < 1 || height < 1 {
        return Err(FilterError::InvalidParameters(
            "filter dimensions must be >= 1".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&rank) {
        return Err(FilterError::InvalidParameters(
            "rank must be in [0.0, 1.0]".to_string(),
        ));
    }

    // No-op for 1x1 filter
    if width == 1 && height == 1 {
        return Ok(pix.deep_clone());
    }

    let img_w = pix.width();
    let img_h = pix.height();
    let wf = width;
    let hf = height;

    // Calculate filter half sizes (for centering)
    let half_w = (wf / 2) as i32;
    let half_h = (hf / 2) as i32;

    // Calculate rank position
    let filter_size = wf * hf;
    let rank_position = ((rank * (filter_size - 1) as f32) + 0.5) as u32;

    // Create output image
    let out_pix = Pix::new(img_w, img_h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    let mut hist_r = RankHistogram::new();
    let mut hist_g = RankHistogram::new();
    let mut hist_b = RankHistogram::new();
    let mut hist_a = RankHistogram::new();

    // Choose traversal order based on filter dimensions
    if hf > wf {
        // Row-major scanning (column-wise iteration is faster)
        for x in 0..img_w {
            hist_r.clear();
            hist_g.clear();
            hist_b.clear();
            hist_a.clear();

            for y in 0..img_h {
                if y == 0 {
                    // Initialize histograms for first row
                    for ky in 0..hf {
                        for kx in 0..wf {
                            let sx =
                                (x as i32 + kx as i32 - half_w).clamp(0, img_w as i32 - 1) as u32;
                            let sy = (ky as i32 - half_h).clamp(0, img_h as i32 - 1) as u32;
                            let pixel = unsafe { pix.get_pixel_unchecked(sx, sy) };
                            let (r, g, b, a) = color::extract_rgba(pixel);
                            hist_r.add(r);
                            hist_g.add(g);
                            hist_b.add(b);
                            hist_a.add(a);
                        }
                    }
                } else {
                    // Incremental update: remove top row, add bottom row
                    let old_y = (y as i32 - 1 - half_h).clamp(0, img_h as i32 - 1) as u32;
                    let new_y =
                        (y as i32 + hf as i32 - 1 - half_h).clamp(0, img_h as i32 - 1) as u32;

                    for kx in 0..wf {
                        let sx = (x as i32 + kx as i32 - half_w).clamp(0, img_w as i32 - 1) as u32;

                        let old_pixel = unsafe { pix.get_pixel_unchecked(sx, old_y) };
                        let (or, og, ob, oa) = color::extract_rgba(old_pixel);
                        hist_r.remove(or);
                        hist_g.remove(og);
                        hist_b.remove(ob);
                        hist_a.remove(oa);

                        let new_pixel = unsafe { pix.get_pixel_unchecked(sx, new_y) };
                        let (nr, ng, nb, na) = color::extract_rgba(new_pixel);
                        hist_r.add(nr);
                        hist_g.add(ng);
                        hist_b.add(nb);
                        hist_a.add(na);
                    }
                }

                let result_r = hist_r.get_rank_value(rank_position);
                let result_g = hist_g.get_rank_value(rank_position);
                let result_b = hist_b.get_rank_value(rank_position);
                let result_a = hist_a.get_rank_value(rank_position);

                let result = color::compose_rgba(result_r, result_g, result_b, result_a);
                unsafe { out_mut.set_pixel_unchecked(x, y, result) };
            }
        }
    } else {
        // Column-major scanning (row-wise iteration is faster)
        for y in 0..img_h {
            hist_r.clear();
            hist_g.clear();
            hist_b.clear();
            hist_a.clear();

            for x in 0..img_w {
                if x == 0 {
                    // Initialize histograms for first column
                    for ky in 0..hf {
                        for kx in 0..wf {
                            let sx = (kx as i32 - half_w).clamp(0, img_w as i32 - 1) as u32;
                            let sy =
                                (y as i32 + ky as i32 - half_h).clamp(0, img_h as i32 - 1) as u32;
                            let pixel = unsafe { pix.get_pixel_unchecked(sx, sy) };
                            let (r, g, b, a) = color::extract_rgba(pixel);
                            hist_r.add(r);
                            hist_g.add(g);
                            hist_b.add(b);
                            hist_a.add(a);
                        }
                    }
                } else {
                    // Incremental update: remove left column, add right column
                    let old_x = (x as i32 - 1 - half_w).clamp(0, img_w as i32 - 1) as u32;
                    let new_x =
                        (x as i32 + wf as i32 - 1 - half_w).clamp(0, img_w as i32 - 1) as u32;

                    for ky in 0..hf {
                        let sy = (y as i32 + ky as i32 - half_h).clamp(0, img_h as i32 - 1) as u32;

                        let old_pixel = unsafe { pix.get_pixel_unchecked(old_x, sy) };
                        let (or, og, ob, oa) = color::extract_rgba(old_pixel);
                        hist_r.remove(or);
                        hist_g.remove(og);
                        hist_b.remove(ob);
                        hist_a.remove(oa);

                        let new_pixel = unsafe { pix.get_pixel_unchecked(new_x, sy) };
                        let (nr, ng, nb, na) = color::extract_rgba(new_pixel);
                        hist_r.add(nr);
                        hist_g.add(ng);
                        hist_b.add(nb);
                        hist_a.add(na);
                    }
                }

                let result_r = hist_r.get_rank_value(rank_position);
                let result_g = hist_g.get_rank_value(rank_position);
                let result_b = hist_b.get_rank_value(rank_position);
                let result_a = hist_a.get_rank_value(rank_position);

                let result = color::compose_rgba(result_r, result_g, result_b, result_a);
                unsafe { out_mut.set_pixel_unchecked(x, y, result) };
            }
        }
    }

    Ok(out_mut.into())
}

/// Apply median filter to an image
///
/// Median filter is a special case of rank filter with rank = 0.5.
/// It is effective for removing salt-and-pepper noise while preserving edges.
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale or 32bpp color image
/// * `width` - Filter width (must be >= 1)
/// * `height` - Filter height (must be >= 1)
///
/// # Returns
/// Filtered image with same dimensions and depth as input
///
/// # Example
/// ```ignore
/// let filtered = median_filter(&pix, 3, 3)?;
/// ```
pub fn median_filter(pix: &Pix, width: u32, height: u32) -> FilterResult<Pix> {
    rank_filter(pix, width, height, 0.5)
}

/// Apply minimum filter (erosion-like operation)
///
/// Minimum filter outputs the smallest value in the neighborhood.
/// This is equivalent to rank filter with rank = 0.0.
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale or 32bpp color image
/// * `width` - Filter width (must be >= 1)
/// * `height` - Filter height (must be >= 1)
///
/// # Returns
/// Filtered image with same dimensions and depth as input
///
/// # Example
/// ```ignore
/// let filtered = min_filter(&pix, 3, 3)?;
/// ```
pub fn min_filter(pix: &Pix, width: u32, height: u32) -> FilterResult<Pix> {
    rank_filter(pix, width, height, 0.0)
}

/// Apply maximum filter (dilation-like operation)
///
/// Maximum filter outputs the largest value in the neighborhood.
/// This is equivalent to rank filter with rank = 1.0.
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale or 32bpp color image
/// * `width` - Filter width (must be >= 1)
/// * `height` - Filter height (must be >= 1)
///
/// # Returns
/// Filtered image with same dimensions and depth as input
///
/// # Example
/// ```ignore
/// let filtered = max_filter(&pix, 3, 3)?;
/// ```
pub fn max_filter(pix: &Pix, width: u32, height: u32) -> FilterResult<Pix> {
    rank_filter(pix, width, height, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_gray_image() -> Pix {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a gradient with some noise
        for y in 0..10 {
            for x in 0..10 {
                let val = (x * 25 + y * 5) as u32;
                unsafe { pix_mut.set_pixel_unchecked(x, y, val.min(255)) };
            }
        }

        pix_mut.into()
    }

    fn create_test_color_image() -> Pix {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                let r = (x * 25) as u8;
                let g = (y * 25) as u8;
                let b = 128;
                let pixel = color::compose_rgb(r, g, b);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        pix_mut.into()
    }

    fn create_noisy_gray_image() -> Pix {
        // Image with salt-and-pepper noise
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with mid-gray
        for y in 0..10 {
            for x in 0..10 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, 128) };
            }
        }

        // Add salt (white) noise
        unsafe {
            pix_mut.set_pixel_unchecked(2, 3, 255);
            pix_mut.set_pixel_unchecked(7, 5, 255);
        }

        // Add pepper (black) noise
        unsafe {
            pix_mut.set_pixel_unchecked(4, 6, 0);
            pix_mut.set_pixel_unchecked(8, 2, 0);
        }

        pix_mut.into()
    }

    #[test]
    fn test_rank_filter_invalid_params() {
        let pix = create_test_gray_image();

        // Invalid filter dimensions
        assert!(rank_filter(&pix, 0, 3, 0.5).is_err());
        assert!(rank_filter(&pix, 3, 0, 0.5).is_err());

        // Invalid rank
        assert!(rank_filter(&pix, 3, 3, -0.1).is_err());
        assert!(rank_filter(&pix, 3, 3, 1.1).is_err());
    }

    #[test]
    fn test_rank_filter_noop() {
        let pix = create_test_gray_image();

        // 1x1 filter should return a copy
        let result = rank_filter(&pix, 1, 1, 0.5).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), pix.depth());

        // Values should be identical
        for y in 0..pix.height() {
            for x in 0..pix.width() {
                let orig = unsafe { pix.get_pixel_unchecked(x, y) };
                let res = unsafe { result.get_pixel_unchecked(x, y) };
                assert_eq!(orig, res);
            }
        }
    }

    #[test]
    fn test_rank_filter_gray_basic() {
        let pix = create_test_gray_image();
        let result = rank_filter(&pix, 3, 3, 0.5).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_rank_filter_color_basic() {
        let pix = create_test_color_image();
        let result = rank_filter(&pix, 3, 3, 0.5).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_median_filter_noise_removal() {
        let pix = create_noisy_gray_image();
        let result = median_filter(&pix, 3, 3).unwrap();

        // Check that noise at (2, 3) is reduced
        let orig = unsafe { pix.get_pixel_unchecked(2, 3) };
        let filtered = unsafe { result.get_pixel_unchecked(2, 3) };
        assert_eq!(orig, 255); // Was white noise
        assert!(filtered < 200); // Should be smoothed toward 128
    }

    #[test]
    fn test_min_filter_erosion_like() {
        let pix = create_test_gray_image();
        let result = min_filter(&pix, 3, 3).unwrap();

        // Minimum filter should generally reduce values
        let center_orig = unsafe { pix.get_pixel_unchecked(5, 5) };
        let center_filtered = unsafe { result.get_pixel_unchecked(5, 5) };
        assert!(center_filtered <= center_orig);
    }

    #[test]
    fn test_max_filter_dilation_like() {
        let pix = create_test_gray_image();
        let result = max_filter(&pix, 3, 3).unwrap();

        // Maximum filter should generally increase values
        let center_orig = unsafe { pix.get_pixel_unchecked(5, 5) };
        let center_filtered = unsafe { result.get_pixel_unchecked(5, 5) };
        assert!(center_filtered >= center_orig);
    }

    #[test]
    fn test_rank_filter_row_major_traversal() {
        // Test with hf > wf to trigger row-major traversal
        let pix = create_test_gray_image();
        let result = rank_filter(&pix, 3, 5, 0.5).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_rank_filter_column_major_traversal() {
        // Test with wf >= hf to trigger column-major traversal
        let pix = create_test_gray_image();
        let result = rank_filter(&pix, 5, 3, 0.5).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_rank_histogram() {
        let mut hist = RankHistogram::new();

        // Add values
        hist.add(10);
        hist.add(20);
        hist.add(30);
        hist.add(40);
        hist.add(50);

        // Rank 0 should be minimum (10)
        assert_eq!(hist.get_rank_value(0), 10);

        // Rank 2 should be median (30)
        assert_eq!(hist.get_rank_value(2), 30);

        // Rank 4 should be maximum (50)
        assert_eq!(hist.get_rank_value(4), 50);
    }

    #[test]
    fn test_rank_histogram_remove() {
        let mut hist = RankHistogram::new();

        // Add values
        hist.add(10);
        hist.add(20);
        hist.add(30);

        // Remove one value
        hist.remove(20);

        // Now should have [10, 30]
        assert_eq!(hist.get_rank_value(0), 10);
        assert_eq!(hist.get_rank_value(1), 30);
    }

    #[test]
    fn test_unsupported_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(rank_filter(&pix, 3, 3, 0.5).is_err());
    }
}
