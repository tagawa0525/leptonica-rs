//! Skew detection and correction
//!
//! This module provides functionality to detect and correct document skew.
//! The algorithm uses differential square sum scoring to find the angle
//! that best aligns text lines horizontally.
//!
//! # Algorithm Overview
//!
//! 1. **Coarse Sweep**: Scan through angles in the range ±sweep_range degrees
//!    at sweep_delta intervals to find the approximate skew angle.
//!
//! 2. **Binary Search**: Refine the angle using interval-halving search
//!    until the desired precision (min_bs_delta) is reached.
//!
//! 3. **Scoring**: For each angle, the image is vertically sheared and the
//!    differential square sum of row pixel counts is computed. Text lines
//!    produce maximum score when horizontal.

use crate::{RecogError, RecogResult};
use leptonica_core::{Pix, PixelDepth};
use leptonica_transform::rotate_by_angle;

/// Options for skew detection
#[derive(Debug, Clone)]
pub struct SkewDetectOptions {
    /// Half the sweep range in degrees (default: 7.0)
    /// The full sweep range is ±sweep_range degrees
    pub sweep_range: f32,

    /// Angle increment for sweep phase in degrees (default: 1.0)
    pub sweep_delta: f32,

    /// Minimum angle increment for binary search in degrees (default: 0.01)
    pub min_bs_delta: f32,

    /// Reduction factor for sweep phase: 1, 2, 4, or 8 (default: 4)
    pub sweep_reduction: u32,

    /// Reduction factor for binary search phase: 1, 2, 4, or 8 (default: 2)
    pub bs_reduction: u32,
}

impl Default for SkewDetectOptions {
    fn default() -> Self {
        Self {
            sweep_range: 7.0,
            sweep_delta: 1.0,
            min_bs_delta: 0.01,
            sweep_reduction: 4,
            bs_reduction: 2,
        }
    }
}

impl SkewDetectOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sweep range (half the full range)
    pub fn with_sweep_range(mut self, range: f32) -> Self {
        self.sweep_range = range;
        self
    }

    /// Set the sweep delta (angle increment)
    pub fn with_sweep_delta(mut self, delta: f32) -> Self {
        self.sweep_delta = delta;
        self
    }

    /// Set the minimum binary search delta
    pub fn with_min_bs_delta(mut self, delta: f32) -> Self {
        self.min_bs_delta = delta;
        self
    }

    /// Set the sweep reduction factor
    pub fn with_sweep_reduction(mut self, reduction: u32) -> Self {
        self.sweep_reduction = reduction;
        self
    }

    /// Set the binary search reduction factor
    pub fn with_bs_reduction(mut self, reduction: u32) -> Self {
        self.bs_reduction = reduction;
        self
    }

    /// Validate options
    pub fn validate(&self) -> RecogResult<()> {
        if self.sweep_range <= 0.0 {
            return Err(RecogError::InvalidParameter(
                "sweep_range must be positive".to_string(),
            ));
        }
        if self.sweep_delta <= 0.0 {
            return Err(RecogError::InvalidParameter(
                "sweep_delta must be positive".to_string(),
            ));
        }
        if self.min_bs_delta <= 0.0 {
            return Err(RecogError::InvalidParameter(
                "min_bs_delta must be positive".to_string(),
            ));
        }
        if !matches!(self.sweep_reduction, 1 | 2 | 4 | 8) {
            return Err(RecogError::InvalidParameter(
                "sweep_reduction must be 1, 2, 4, or 8".to_string(),
            ));
        }
        if !matches!(self.bs_reduction, 1 | 2 | 4 | 8) {
            return Err(RecogError::InvalidParameter(
                "bs_reduction must be 1, 2, 4, or 8".to_string(),
            ));
        }
        if self.bs_reduction > self.sweep_reduction {
            return Err(RecogError::InvalidParameter(
                "bs_reduction must not exceed sweep_reduction".to_string(),
            ));
        }
        Ok(())
    }
}

/// Result of skew detection
#[derive(Debug, Clone)]
pub struct SkewResult {
    /// Detected skew angle in degrees
    /// Positive angle indicates counterclockwise rotation needed to deskew
    pub angle: f32,

    /// Confidence score (ratio of max/min scores)
    /// Higher values indicate more reliable detection
    /// Typical threshold is 3.0-6.0
    pub confidence: f32,
}

// Constants for confidence calculation
const MIN_VALID_MAX_SCORE: f64 = 10000.0;
const MIN_SCORE_THRESH_FACTOR: f64 = 0.000002;
const MIN_DESKEW_ANGLE: f32 = 0.1;
const MIN_ALLOWED_CONFIDENCE: f32 = 3.0;

/// Detect skew angle in an image
///
/// # Arguments
/// * `pix` - Input image (1 bpp binary image works best)
/// * `options` - Detection options
///
/// # Returns
/// SkewResult containing the detected angle and confidence
///
/// # Example
/// ```no_run
/// use leptonica_recog::skew::{find_skew, SkewDetectOptions};
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
/// let result = find_skew(&pix, &SkewDetectOptions::default()).unwrap();
/// println!("Skew angle: {} degrees, confidence: {}", result.angle, result.confidence);
/// ```
pub fn find_skew(pix: &Pix, options: &SkewDetectOptions) -> RecogResult<SkewResult> {
    options.validate()?;

    // Convert to 1bpp if necessary
    let binary_pix = ensure_binary(pix)?;

    // Check for empty image
    if is_image_empty(&binary_pix) {
        return Err(RecogError::NoContent(
            "image is empty or all white".to_string(),
        ));
    }

    // Reduce image for sweep
    let sweep_pix = reduce_image(&binary_pix, options.sweep_reduction)?;

    // Reduce image for binary search (may be same as sweep)
    let search_pix = if options.bs_reduction == options.sweep_reduction {
        sweep_pix.deep_clone()
    } else {
        reduce_image(&binary_pix, options.bs_reduction)?
    };

    // Phase 1: Coarse sweep
    let (best_angle, _best_score) = sweep_angles(
        &sweep_pix,
        -options.sweep_range,
        options.sweep_range,
        options.sweep_delta,
    )?;

    // Phase 2: Binary search refinement
    let (refined_angle, max_score, min_score) = binary_search_angle(
        &search_pix,
        best_angle,
        options.sweep_delta,
        options.min_bs_delta,
    )?;

    // Calculate confidence
    let confidence = calculate_confidence(
        &search_pix,
        max_score,
        min_score,
        refined_angle,
        options.sweep_range,
        options.sweep_delta,
    );

    Ok(SkewResult {
        angle: refined_angle,
        confidence,
    })
}

/// Detect skew and deskew the image
///
/// # Arguments
/// * `pix` - Input image
/// * `options` - Detection options
///
/// # Returns
/// Tuple of (deskewed image, skew result)
pub fn find_skew_and_deskew(
    pix: &Pix,
    options: &SkewDetectOptions,
) -> RecogResult<(Pix, SkewResult)> {
    let result = find_skew(pix, options)?;

    // Only deskew if angle is significant and confidence is sufficient
    let deskewed =
        if result.angle.abs() >= MIN_DESKEW_ANGLE && result.confidence >= MIN_ALLOWED_CONFIDENCE {
            deskew(pix, result.angle)?
        } else {
            pix.deep_clone()
        };

    Ok((deskewed, result))
}

/// Deskew an image by a given angle
///
/// # Arguments
/// * `pix` - Input image
/// * `angle` - Rotation angle in degrees (positive = counterclockwise)
///
/// # Returns
/// The deskewed image
pub fn deskew(pix: &Pix, angle: f32) -> RecogResult<Pix> {
    if angle.abs() < 0.001 {
        return Ok(pix.deep_clone());
    }

    // Rotate by the detected angle to correct skew
    let rotated = rotate_by_angle(pix, angle)?;
    Ok(rotated)
}

/// Ensure image is binary (1 bpp)
fn ensure_binary(pix: &Pix) -> RecogResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit1 => Ok(pix.deep_clone()),
        PixelDepth::Bit8 => {
            // Simple threshold at 128
            let binary = threshold_to_binary(pix, 128)?;
            Ok(binary)
        }
        PixelDepth::Bit32 => {
            // Convert to grayscale first, then threshold
            let gray = rgb_to_grayscale(pix)?;
            let binary = threshold_to_binary(&gray, 128)?;
            Ok(binary)
        }
        _ => Err(RecogError::UnsupportedDepth {
            expected: "1, 8, or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

/// Convert RGB to grayscale
fn rgb_to_grayscale(pix: &Pix) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let gray = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut gray_mut = gray.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let r = (pixel >> 24) & 0xFF;
            let g = (pixel >> 16) & 0xFF;
            let b = (pixel >> 8) & 0xFF;
            // Standard luminance formula
            let gray_val = (r * 77 + g * 150 + b * 29) >> 8;
            gray_mut.set_pixel_unchecked(x, y, gray_val);
        }
    }

    Ok(gray_mut.into())
}

/// Threshold grayscale to binary
fn threshold_to_binary(pix: &Pix, threshold: u32) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let binary = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut binary_mut = binary.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            // 1 = black (foreground), 0 = white (background)
            let bit = if val < threshold { 1 } else { 0 };
            binary_mut.set_pixel_unchecked(x, y, bit);
        }
    }

    Ok(binary_mut.into())
}

/// Check if image is empty (all white/zero pixels)
fn is_image_empty(pix: &Pix) -> bool {
    let w = pix.width();
    let h = pix.height();

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            if val != 0 {
                return false;
            }
        }
    }
    true
}

/// Reduce image by factor (simple subsampling for binary images)
fn reduce_image(pix: &Pix, factor: u32) -> RecogResult<Pix> {
    if factor == 1 {
        return Ok(pix.deep_clone());
    }

    let w = pix.width();
    let h = pix.height();
    let new_w = w / factor;
    let new_h = h / factor;

    if new_w == 0 || new_h == 0 {
        return Err(RecogError::ImageTooSmall {
            min_width: factor,
            min_height: factor,
            actual_width: w,
            actual_height: h,
        });
    }

    let reduced = Pix::new(new_w, new_h, pix.depth())?;
    let mut reduced_mut = reduced.try_into_mut().unwrap();

    // For binary images, use OR reduction (any black pixel makes output black)
    for ny in 0..new_h {
        for nx in 0..new_w {
            let mut has_black = false;
            for dy in 0..factor {
                for dx in 0..factor {
                    let sx = nx * factor + dx;
                    let sy = ny * factor + dy;
                    if sx < w && sy < h {
                        let val = pix.get_pixel_unchecked(sx, sy);
                        if val != 0 {
                            has_black = true;
                            break;
                        }
                    }
                }
                if has_black {
                    break;
                }
            }
            let out_val = if has_black { 1 } else { 0 };
            reduced_mut.set_pixel_unchecked(nx, ny, out_val);
        }
    }

    Ok(reduced_mut.into())
}

/// Sweep through angles and find the one with maximum score
fn sweep_angles(
    pix: &Pix,
    start_angle: f32,
    end_angle: f32,
    delta: f32,
) -> RecogResult<(f32, f64)> {
    let mut best_angle = 0.0f32;
    let mut best_score = f64::MIN;

    let mut angle = start_angle;
    while angle <= end_angle {
        let sheared = vertical_shear(pix, angle)?;
        let score = compute_differential_square_sum(&sheared);

        if score > best_score {
            best_score = score;
            best_angle = angle;
        }

        angle += delta;
    }

    Ok((best_angle, best_score))
}

/// Binary search to refine angle
#[allow(clippy::needless_range_loop)]
fn binary_search_angle(
    pix: &Pix,
    center_angle: f32,
    initial_delta: f32,
    min_delta: f32,
) -> RecogResult<(f32, f64, f64)> {
    let mut center = center_angle;
    let mut delta = initial_delta / 2.0;

    // Initial scores at center and neighbors
    let sheared_center = vertical_shear(pix, center)?;
    let mut scores = [0.0f64; 5];
    scores[2] = compute_differential_square_sum(&sheared_center);

    let sheared_left = vertical_shear(pix, center - initial_delta)?;
    scores[0] = compute_differential_square_sum(&sheared_left);

    let sheared_right = vertical_shear(pix, center + initial_delta)?;
    scores[4] = compute_differential_square_sum(&sheared_right);

    let mut max_score = scores[0].max(scores[2]).max(scores[4]);
    let mut min_score = scores[0].min(scores[2]).min(scores[4]);

    while delta >= min_delta {
        // Compute left intermediate
        let left_angle = center - delta;
        let sheared_left = vertical_shear(pix, left_angle)?;
        scores[1] = compute_differential_square_sum(&sheared_left);

        // Compute right intermediate
        let right_angle = center + delta;
        let sheared_right = vertical_shear(pix, right_angle)?;
        scores[3] = compute_differential_square_sum(&sheared_right);

        // Find maximum among center three
        let mut max_idx = 1;
        let mut max_val = scores[1];
        for i in 2..4 {
            if scores[i] > max_val {
                max_val = scores[i];
                max_idx = i;
            }
        }

        // Update tracking
        max_score = max_score.max(max_val);
        min_score = min_score.min(scores[1]).min(scores[3]);

        // Update for next iteration
        let left_temp = scores[max_idx - 1];
        let right_temp = scores[max_idx + 1];
        scores[2] = max_val;
        scores[0] = left_temp;
        scores[4] = right_temp;

        center += delta * (max_idx as f32 - 2.0);
        delta *= 0.5;
    }

    Ok((center, max_score, min_score))
}

/// Vertical shear transformation
/// This is a key operation for skew detection - it shears the image
/// vertically by an amount proportional to the x-coordinate
fn vertical_shear(pix: &Pix, angle_deg: f32) -> RecogResult<Pix> {
    if angle_deg.abs() < 0.001 {
        return Ok(pix.deep_clone());
    }

    let w = pix.width();
    let h = pix.height();
    let angle_rad = angle_deg.to_radians();
    let tan_a = angle_rad.tan();

    // Calculate new height needed to contain sheared image
    let max_shear = (w as f32 * tan_a.abs()).ceil() as u32;
    let new_h = h + max_shear;

    let sheared = Pix::new(w, new_h, pix.depth())?;
    let mut sheared_mut = sheared.try_into_mut().unwrap();

    // Fill with white (0 for binary)
    for y in 0..new_h {
        for x in 0..w {
            sheared_mut.set_pixel_unchecked(x, y, 0);
        }
    }

    // Apply vertical shear: y' = y + x * tan(angle)
    let y_offset = if tan_a < 0.0 { max_shear } else { 0 };

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            if val != 0 {
                let shear_amount = (x as f32 * tan_a).round() as i32;
                let new_y = y as i32 + shear_amount + y_offset as i32;
                if new_y >= 0 && (new_y as u32) < new_h {
                    sheared_mut.set_pixel_unchecked(x, new_y as u32, val);
                }
            }
        }
    }

    Ok(sheared_mut.into())
}

/// Compute differential square sum score
/// This measures how well text lines are aligned horizontally
fn compute_differential_square_sum(pix: &Pix) -> f64 {
    let w = pix.width();
    let h = pix.height();

    // Count pixels per row
    let mut row_sums: Vec<u32> = Vec::with_capacity(h as usize);
    for y in 0..h {
        let mut sum = 0u32;
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            if val != 0 {
                sum += 1;
            }
        }
        row_sums.push(sum);
    }

    // Skip some rows at top and bottom to avoid edge effects
    let skip_h = ((w as f32 * 0.05) as u32).max(1);
    let skip = (h / 10).min(skip_h);
    let n_skip = (skip / 2).max(1) as usize;

    if row_sums.len() <= 2 * n_skip {
        return 0.0;
    }

    // Compute sum of squared differences
    let mut sum = 0.0f64;
    for i in n_skip..(row_sums.len() - n_skip) {
        let diff = row_sums[i] as f64 - row_sums[i - 1] as f64;
        sum += diff * diff;
    }

    sum
}

/// Calculate confidence score
fn calculate_confidence(
    pix: &Pix,
    max_score: f64,
    min_score: f64,
    angle: f32,
    sweep_range: f32,
    sweep_delta: f32,
) -> f32 {
    let w = pix.width() as f64;
    let h = pix.height() as f64;

    // Minimum threshold based on image dimensions
    let min_thresh = MIN_SCORE_THRESH_FACTOR * w * w * h;

    // Check if scores are valid
    if max_score < MIN_VALID_MAX_SCORE {
        return 0.0;
    }

    if min_score <= min_thresh {
        return 0.0;
    }

    // Check if angle is at edge of sweep range
    if angle.abs() > sweep_range - sweep_delta {
        return 0.0;
    }

    (max_score / min_score) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_horizontal_lines_image(w: u32, h: u32, line_spacing: u32) -> Pix {
        let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Draw horizontal lines
        let mut y = line_spacing;
        while y < h {
            for x in (w / 10)..(w * 9 / 10) {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
            y += line_spacing;
        }

        pix_mut.into()
    }

    #[test]
    fn test_skew_detect_options_default() {
        let opts = SkewDetectOptions::default();
        assert!((opts.sweep_range - 7.0).abs() < 0.001);
        assert!((opts.sweep_delta - 1.0).abs() < 0.001);
        assert!((opts.min_bs_delta - 0.01).abs() < 0.001);
        assert_eq!(opts.sweep_reduction, 4);
        assert_eq!(opts.bs_reduction, 2);
    }

    #[test]
    fn test_skew_detect_options_validation() {
        let opts = SkewDetectOptions::default();
        assert!(opts.validate().is_ok());

        let invalid = SkewDetectOptions::default().with_sweep_range(-1.0);
        assert!(invalid.validate().is_err());

        let invalid = SkewDetectOptions::default().with_sweep_reduction(3);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_vertical_shear_zero_angle() {
        let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
        let sheared = vertical_shear(&pix, 0.0).unwrap();
        assert_eq!(sheared.width(), 50);
        assert_eq!(sheared.height(), 50);
    }

    #[test]
    fn test_vertical_shear_nonzero() {
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let sheared = vertical_shear(&pix, 5.0).unwrap();
        assert_eq!(sheared.width(), 100);
        assert!(sheared.height() > 100);
    }

    #[test]
    fn test_compute_differential_square_sum() {
        let pix = create_horizontal_lines_image(200, 200, 20);
        let score = compute_differential_square_sum(&pix);
        assert!(score > 0.0);
    }

    #[test]
    fn test_reduce_image() {
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let reduced = reduce_image(&pix, 2).unwrap();
        assert_eq!(reduced.width(), 50);
        assert_eq!(reduced.height(), 50);
    }

    #[test]
    fn test_find_skew_horizontal_lines() {
        // Create image with horizontal lines (zero skew)
        let pix = create_horizontal_lines_image(400, 400, 30);

        let opts = SkewDetectOptions::default()
            .with_sweep_reduction(2)
            .with_bs_reduction(1);

        let result = find_skew(&pix, &opts).unwrap();

        // Should detect near-zero angle
        assert!(
            result.angle.abs() < 1.0,
            "Expected near-zero angle, got {}",
            result.angle
        );
    }

    #[test]
    fn test_deskew_zero_angle() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let deskewed = deskew(&pix, 0.0).unwrap();
        assert_eq!(deskewed.width(), 100);
        assert_eq!(deskewed.height(), 100);
    }

    #[test]
    fn test_ensure_binary_from_grayscale() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let binary = ensure_binary(&pix).unwrap();
        assert_eq!(binary.depth(), PixelDepth::Bit1);
    }
}
