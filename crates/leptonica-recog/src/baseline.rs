//! Baseline detection
//!
//! This module provides functionality to detect text baselines in document images.
//! Baselines are useful for:
//! - Text line segmentation
//! - Local skew correction
//! - OCR preprocessing
//!
//! # Algorithm Overview
//!
//! 1. **Horizontal Projection**: Count pixels per row to create a projection histogram
//! 2. **Differential Signal**: Compute row-to-row differences to find transitions
//! 3. **Peak Detection**: Find peaks in the differential signal (baselines)
//! 4. **Endpoint Detection**: For each baseline, find left and right text boundaries

use crate::skew::SkewDetectOptions;
use crate::{RecogError, RecogResult};
use leptonica_core::{Pix, PixelDepth};

/// Options for baseline detection
#[derive(Debug, Clone)]
pub struct BaselineOptions {
    /// Minimum text block width in pixels (default: 80)
    /// Blocks narrower than this are ignored
    pub min_block_width: u32,

    /// Peak threshold ratio (default: 80)
    /// Peaks must be at least this percentage of max peak
    pub peak_threshold: u32,

    /// Number of slices for local skew detection (default: 10)
    pub num_slices: u32,
}

impl Default for BaselineOptions {
    fn default() -> Self {
        Self {
            min_block_width: 80,
            peak_threshold: 80,
            num_slices: 10,
        }
    }
}

impl BaselineOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum block width
    pub fn with_min_block_width(mut self, width: u32) -> Self {
        self.min_block_width = width;
        self
    }

    /// Set the peak threshold ratio
    pub fn with_peak_threshold(mut self, threshold: u32) -> Self {
        self.peak_threshold = threshold;
        self
    }

    /// Set the number of slices for local skew
    pub fn with_num_slices(mut self, slices: u32) -> Self {
        self.num_slices = slices;
        self
    }

    /// Validate options
    fn validate(&self) -> RecogResult<()> {
        if self.min_block_width == 0 {
            return Err(RecogError::InvalidParameter(
                "min_block_width must be positive".to_string(),
            ));
        }
        if self.peak_threshold == 0 || self.peak_threshold > 100 {
            return Err(RecogError::InvalidParameter(
                "peak_threshold must be between 1 and 100".to_string(),
            ));
        }
        if self.num_slices < 2 || self.num_slices > 20 {
            return Err(RecogError::InvalidParameter(
                "num_slices must be between 2 and 20".to_string(),
            ));
        }
        Ok(())
    }
}

/// Result of baseline detection
#[derive(Debug, Clone)]
pub struct BaselineResult {
    /// Y coordinates of detected baselines
    pub baselines: Vec<i32>,

    /// Optional endpoints for each baseline: (x1, y1, x2, y2)
    /// x1, y1 = left endpoint; x2, y2 = right endpoint
    pub endpoints: Option<Vec<(i32, i32, i32, i32)>>,
}

// Constants for peak detection
const MIN_DIST_FROM_PEAK: i32 = 30;
const ZERO_THRESHOLD_RATIO: u32 = 100;

/// Find baselines in a binary image
///
/// # Arguments
/// * `pix` - Input image (should be binary, 1 bpp)
/// * `options` - Detection options
///
/// # Returns
/// BaselineResult containing y-coordinates of baselines
///
/// # Example
/// ```no_run
/// use leptonica_recog::baseline::{find_baselines, BaselineOptions};
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(500, 300, PixelDepth::Bit1).unwrap();
/// let result = find_baselines(&pix, &BaselineOptions::default()).unwrap();
/// for y in &result.baselines {
///     println!("Baseline at y = {}", y);
/// }
/// ```
pub fn find_baselines(pix: &Pix, options: &BaselineOptions) -> RecogResult<BaselineResult> {
    options.validate()?;

    // Ensure binary
    let binary = ensure_binary(pix)?;

    let h = binary.height();

    // Step 1: Compute horizontal projection (row sums)
    let row_sums = compute_row_sums(&binary);

    // Step 2: Compute differential (row-to-row differences)
    let diff = compute_differential(&row_sums);

    // Step 3: Find peaks in differential signal
    let baselines = find_peaks(&diff, options.peak_threshold);

    // Step 4: Find endpoints for each baseline
    let endpoints = find_endpoints(&binary, &baselines, options.min_block_width);

    // Filter baselines without valid endpoints
    let (filtered_baselines, filtered_endpoints) = filter_baselines(baselines, endpoints, h);

    Ok(BaselineResult {
        baselines: filtered_baselines,
        endpoints: Some(filtered_endpoints),
    })
}

/// Get local skew angles for vertical slices of the image
///
/// # Arguments
/// * `pix` - Input image
/// * `num_slices` - Number of vertical slices
/// * `sweep_range` - Angle range for skew detection (degrees)
///
/// # Returns
/// Vector of skew angles, one per slice
pub fn get_local_skew_angles(
    pix: &Pix,
    num_slices: u32,
    sweep_range: f32,
) -> RecogResult<Vec<f32>> {
    if num_slices < 2 || num_slices > 20 {
        return Err(RecogError::InvalidParameter(
            "num_slices must be between 2 and 20".to_string(),
        ));
    }

    let binary = ensure_binary(pix)?;
    let h = binary.height();
    let slice_height = h / num_slices;

    if slice_height < 10 {
        return Err(RecogError::ImageTooSmall {
            min_width: pix.width(),
            min_height: num_slices * 10,
            actual_width: pix.width(),
            actual_height: h,
        });
    }

    let mut angles = Vec::with_capacity(num_slices as usize);
    let skew_options = SkewDetectOptions::default()
        .with_sweep_range(sweep_range)
        .with_sweep_reduction(2)
        .with_bs_reduction(1);

    // Add overlap (50% of slice height)
    let overlap = slice_height / 2;

    for i in 0..num_slices {
        let y_start = if i == 0 {
            0
        } else {
            (i * slice_height).saturating_sub(overlap)
        };
        let y_end = if i == num_slices - 1 {
            h
        } else {
            ((i + 1) * slice_height + overlap).min(h)
        };

        // Extract slice
        let slice = extract_horizontal_slice(&binary, y_start, y_end)?;

        // Detect skew for this slice
        match crate::skew::find_skew(&slice, &skew_options) {
            Ok(result) => angles.push(result.angle),
            Err(_) => angles.push(0.0), // Default to 0 if detection fails
        }
    }

    Ok(angles)
}

/// Apply local deskew based on baseline analysis
///
/// This corrects for varying skew across the page (keystone effect)
///
/// # Arguments
/// * `pix` - Input image
/// * `options` - Baseline options
/// * `skew_options` - Skew detection options
///
/// # Returns
/// The locally deskewed image
pub fn deskew_local(
    pix: &Pix,
    options: &BaselineOptions,
    skew_options: &SkewDetectOptions,
) -> RecogResult<Pix> {
    options.validate()?;
    skew_options.validate()?;

    let binary = ensure_binary(pix)?;

    // Get local skew angles
    let angles = get_local_skew_angles(&binary, options.num_slices, skew_options.sweep_range)?;

    // If all angles are similar, just do global deskew
    let angle_range = angles.iter().cloned().fold(f32::NAN, f32::max)
        - angles.iter().cloned().fold(f32::NAN, f32::min);

    if angle_range < 0.5 || angles.is_empty() {
        // Use average angle for global correction
        let avg_angle: f32 = angles.iter().sum::<f32>() / angles.len().max(1) as f32;
        return crate::skew::deskew(pix, avg_angle);
    }

    // Apply local correction using vertical shear interpolation
    apply_local_deskew(pix, &angles)
}

// ============================================================================
// Internal functions
// ============================================================================

/// Ensure image is binary
fn ensure_binary(pix: &Pix) -> RecogResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit1 => Ok(pix.deep_clone()),
        PixelDepth::Bit8 => {
            let w = pix.width();
            let h = pix.height();
            let binary = Pix::new(w, h, PixelDepth::Bit1)?;
            let mut binary_mut = binary.try_into_mut().unwrap();

            for y in 0..h {
                for x in 0..w {
                    let val = unsafe { pix.get_pixel_unchecked(x, y) };
                    let bit = if val < 128 { 1 } else { 0 };
                    unsafe { binary_mut.set_pixel_unchecked(x, y, bit) };
                }
            }
            Ok(binary_mut.into())
        }
        _ => Err(RecogError::UnsupportedDepth {
            expected: "1 or 8 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

/// Compute row sums (horizontal projection)
fn compute_row_sums(pix: &Pix) -> Vec<u32> {
    let w = pix.width();
    let h = pix.height();
    let mut sums = Vec::with_capacity(h as usize);

    for y in 0..h {
        let mut sum = 0u32;
        for x in 0..w {
            let val = unsafe { pix.get_pixel_unchecked(x, y) };
            if val != 0 {
                sum += 1;
            }
        }
        sums.push(sum);
    }

    sums
}

/// Compute differential signal (difference between adjacent rows)
fn compute_differential(row_sums: &[u32]) -> Vec<i32> {
    if row_sums.len() < 2 {
        return Vec::new();
    }

    let mut diff = Vec::with_capacity(row_sums.len() - 1);
    for i in 0..row_sums.len() - 1 {
        diff.push(row_sums[i] as i32 - row_sums[i + 1] as i32);
    }
    diff
}

/// Find peaks in differential signal
fn find_peaks(diff: &[i32], threshold_ratio: u32) -> Vec<i32> {
    if diff.is_empty() {
        return Vec::new();
    }

    // Find maximum value
    let max_val = diff.iter().cloned().max().unwrap_or(0);
    if max_val <= 0 {
        return Vec::new();
    }

    let peak_thresh = (max_val as u32 * threshold_ratio / 100) as i32;
    let zero_thresh = (max_val as u32 * ZERO_THRESHOLD_RATIO / 100) as i32;

    let mut baselines = Vec::new();
    let mut in_peak = false;
    let mut max_in_peak = 0i32;
    let mut max_loc = 0i32;
    let mut min_to_search = 0i32;

    for (i, &val) in diff.iter().enumerate() {
        let i = i as i32;

        if !in_peak {
            if val > peak_thresh {
                // Transition to in-peak
                in_peak = true;
                min_to_search = i + MIN_DIST_FROM_PEAK;
                max_in_peak = val;
                max_loc = i;
            }
        } else {
            // Looking for peak maximum
            if val > max_in_peak {
                max_in_peak = val;
                max_loc = i;
                min_to_search = i + MIN_DIST_FROM_PEAK;
            } else if i >= min_to_search && val <= zero_thresh {
                // Found end of peak, record baseline
                in_peak = false;
                baselines.push(max_loc);
            }
        }
    }

    // Handle case where peak extends to end
    if in_peak {
        baselines.push(max_loc);
    }

    baselines
}

/// Find left and right endpoints for each baseline
fn find_endpoints(
    pix: &Pix,
    baselines: &[i32],
    min_width: u32,
) -> Vec<Option<(i32, i32, i32, i32)>> {
    let w = pix.width() as i32;
    let h = pix.height() as i32;

    baselines
        .iter()
        .map(|&y| {
            if y < 0 || y >= h {
                return None;
            }

            // Find leftmost and rightmost black pixels near baseline
            let search_range = 5; // Search ±5 rows
            let mut left_x = w;
            let mut right_x = 0i32;

            for dy in -search_range..=search_range {
                let sy = y + dy;
                if sy < 0 || sy >= h {
                    continue;
                }

                for x in 0..w {
                    let val = unsafe { pix.get_pixel_unchecked(x as u32, sy as u32) };
                    if val != 0 {
                        left_x = left_x.min(x);
                        right_x = right_x.max(x);
                    }
                }
            }

            if right_x - left_x >= min_width as i32 {
                Some((left_x, y, right_x, y))
            } else {
                None
            }
        })
        .collect()
}

/// Filter baselines that don't have valid endpoints
fn filter_baselines(
    baselines: Vec<i32>,
    endpoints: Vec<Option<(i32, i32, i32, i32)>>,
    _max_y: u32,
) -> (Vec<i32>, Vec<(i32, i32, i32, i32)>) {
    let mut filtered_baselines = Vec::new();
    let mut filtered_endpoints = Vec::new();

    for (baseline, endpoint) in baselines.into_iter().zip(endpoints.into_iter()) {
        if let Some(ep) = endpoint {
            filtered_baselines.push(baseline);
            filtered_endpoints.push(ep);
        }
    }

    (filtered_baselines, filtered_endpoints)
}

/// Extract a horizontal slice from an image
fn extract_horizontal_slice(pix: &Pix, y_start: u32, y_end: u32) -> RecogResult<Pix> {
    let w = pix.width();
    let new_h = y_end - y_start;

    if new_h == 0 {
        return Err(RecogError::InvalidParameter(
            "slice height is zero".to_string(),
        ));
    }

    let slice = Pix::new(w, new_h, pix.depth())?;
    let mut slice_mut = slice.try_into_mut().unwrap();

    for y in 0..new_h {
        for x in 0..w {
            let val = unsafe { pix.get_pixel_unchecked(x, y_start + y) };
            unsafe { slice_mut.set_pixel_unchecked(x, y, val) };
        }
    }

    Ok(slice_mut.into())
}

/// Apply local deskew using interpolated shear
fn apply_local_deskew(pix: &Pix, angles: &[f32]) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let num_slices = angles.len();

    if num_slices == 0 {
        return Ok(pix.deep_clone());
    }

    let slice_height = h as f32 / num_slices as f32;

    let result = Pix::new(w, h, pix.depth())?;
    let mut result_mut = result.try_into_mut().unwrap();

    // Fill with background
    for y in 0..h {
        for x in 0..w {
            unsafe { result_mut.set_pixel_unchecked(x, y, 0) };
        }
    }

    // Apply varying shear based on interpolated angle
    for y in 0..h {
        // Determine which slice and interpolation weight
        let slice_pos = y as f32 / slice_height;
        let slice_idx = (slice_pos as usize).min(num_slices - 1);
        let next_idx = (slice_idx + 1).min(num_slices - 1);
        let t = slice_pos - slice_idx as f32;

        // Interpolate angle
        let angle = angles[slice_idx] * (1.0 - t) + angles[next_idx] * t;
        let tan_a = angle.to_radians().tan();

        for x in 0..w {
            let val = unsafe { pix.get_pixel_unchecked(x, y) };
            if val != 0 {
                // Apply horizontal shear
                let shear = (x as f32 - w as f32 / 2.0) * tan_a;
                let new_x = (x as f32 + shear).round() as i32;

                if new_x >= 0 && new_x < w as i32 {
                    unsafe { result_mut.set_pixel_unchecked(new_x as u32, y, val) };
                }
            }
        }
    }

    Ok(result_mut.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_text_like_image(w: u32, h: u32, num_lines: u32, line_height: u32) -> Pix {
        let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let spacing = h / (num_lines + 1);

        for line in 1..=num_lines {
            let y_base = line * spacing;
            // Draw a "text line" as a horizontal band
            for dy in 0..line_height {
                let y = y_base + dy;
                if y < h {
                    for x in (w / 10)..(w * 9 / 10) {
                        unsafe { pix_mut.set_pixel_unchecked(x, y, 1) };
                    }
                }
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_baseline_options_default() {
        let opts = BaselineOptions::default();
        assert_eq!(opts.min_block_width, 80);
        assert_eq!(opts.peak_threshold, 80);
        assert_eq!(opts.num_slices, 10);
    }

    #[test]
    fn test_baseline_options_validation() {
        let opts = BaselineOptions::default();
        assert!(opts.validate().is_ok());

        let invalid = BaselineOptions::default().with_min_block_width(0);
        assert!(invalid.validate().is_err());

        let invalid = BaselineOptions::default().with_num_slices(1);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_compute_row_sums() {
        let pix = Pix::new(10, 5, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill row 2 with black pixels
        for x in 0..10 {
            unsafe { pix_mut.set_pixel_unchecked(x, 2, 1) };
        }

        let pix: Pix = pix_mut.into();
        let sums = compute_row_sums(&pix);

        assert_eq!(sums.len(), 5);
        assert_eq!(sums[0], 0);
        assert_eq!(sums[1], 0);
        assert_eq!(sums[2], 10);
        assert_eq!(sums[3], 0);
        assert_eq!(sums[4], 0);
    }

    #[test]
    fn test_compute_differential() {
        let row_sums = vec![0, 0, 10, 0, 0];
        let diff = compute_differential(&row_sums);

        assert_eq!(diff.len(), 4);
        assert_eq!(diff[0], 0); // 0 - 0
        assert_eq!(diff[1], -10); // 0 - 10
        assert_eq!(diff[2], 10); // 10 - 0
        assert_eq!(diff[3], 0); // 0 - 0
    }

    #[test]
    fn test_find_baselines() {
        let pix = create_text_like_image(400, 300, 5, 10);
        let opts = BaselineOptions::default().with_min_block_width(50);

        let result = find_baselines(&pix, &opts).unwrap();

        // Should find approximately 5 baselines
        assert!(!result.baselines.is_empty());
        assert!(result.baselines.len() <= 6);
    }

    #[test]
    fn test_extract_horizontal_slice() {
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let slice = extract_horizontal_slice(&pix, 20, 40).unwrap();

        assert_eq!(slice.width(), 100);
        assert_eq!(slice.height(), 20);
    }

    #[test]
    fn test_get_local_skew_angles() {
        let pix = create_text_like_image(400, 400, 10, 10);
        let angles = get_local_skew_angles(&pix, 4, 5.0).unwrap();

        assert_eq!(angles.len(), 4);
        // All angles should be near zero for horizontal lines
        for angle in angles {
            assert!(angle.abs() < 2.0);
        }
    }
}
