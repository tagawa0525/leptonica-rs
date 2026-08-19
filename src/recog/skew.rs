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

use crate::core::{Numa, Pix, PixelDepth};
use crate::recog::{RecogError, RecogResult};
use crate::transform::{
    RotateEmbed, RotateFill, RotateMethod, RotateOptions, ShearFill, reduce_rank_binary_cascade,
    rotate, rotate_orth, v_shear_center, v_shear_corner,
};

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
const MIN_VALID_MAX_SCORE: f32 = 10000.0;
const MIN_SCORE_THRESH_FACTOR: f32 = 0.000002;
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
/// use leptonica::recog::skew::{find_skew, SkewDetectOptions};
/// use leptonica::core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
/// let result = find_skew(&pix, &SkewDetectOptions::default()).unwrap();
/// println!("Skew angle: {} degrees, confidence: {}", result.angle, result.confidence);
/// ```
pub fn find_skew(pix: &Pix, options: &SkewDetectOptions) -> RecogResult<SkewResult> {
    options.validate()?;

    // Convert to 1bpp if necessary
    let binary_pix = ensure_binary(pix)?;

    let (angle, confidence, _) = sweep_and_search_pivot(
        &binary_pix,
        options.sweep_reduction,
        options.bs_reduction,
        0.0,
        options.sweep_range,
        options.sweep_delta,
        options.min_bs_delta,
        SkewPivot::Corner,
    )?;

    Ok(SkewResult { angle, confidence })
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
            deskew_by_angle(pix, result.angle)?
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
pub fn deskew_by_angle(pix: &Pix, angle: f32) -> RecogResult<Pix> {
    if angle.abs() < 0.001 {
        return Ok(pix.deep_clone());
    }

    // C `pixDeskewGeneral` finishes with
    // `pixRotate(pixs, deg2rad * angle, L_ROTATE_AREA_MAP, L_BRING_IN_WHITE, 0, 0)`.
    // Passing 0 for width/height means "do not embed", so the deskewed image
    // keeps the source dimensions.
    let options = RotateOptions {
        method: RotateMethod::AreaMap,
        fill: RotateFill::White,
        center_x: None,
        center_y: None,
        embed: RotateEmbed::None,
    };
    Ok(rotate(pix, DEG2RAD * angle, &options)?)
}

/// Options for the deskew high-level interface
#[derive(Debug, Clone)]
pub struct DeskewOptions {
    /// Reduction factor (1, 2, 4, or 8)
    pub sweep_reduction: u32,
    /// Half the sweep range in degrees
    pub sweep_range: f32,
    /// Angle increment for sweep phase in degrees
    pub sweep_delta: f32,
    /// Additional reduction for binary search phase
    pub search_reduction: u32,
}

impl Default for DeskewOptions {
    fn default() -> Self {
        Self {
            sweep_reduction: 2,
            sweep_range: 7.0,
            sweep_delta: 1.0,
            search_reduction: 2,
        }
    }
}

impl DeskewOptions {
    /// Validate options, returning an error if any field is invalid.
    pub fn validate(&self) -> RecogResult<()> {
        if !matches!(self.sweep_reduction, 1 | 2 | 4 | 8) {
            return Err(RecogError::InvalidParameter(
                "sweep_reduction must be 1, 2, 4, or 8".to_string(),
            ));
        }
        if !matches!(self.search_reduction, 1 | 2 | 4 | 8) {
            return Err(RecogError::InvalidParameter(
                "search_reduction must be 1, 2, 4, or 8".to_string(),
            ));
        }
        if self.search_reduction > self.sweep_reduction {
            return Err(RecogError::InvalidParameter(
                "search_reduction must not exceed sweep_reduction".to_string(),
            ));
        }
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
        Ok(())
    }
}

/// Options for sweep-and-search skew detection (alias for [`SkewDetectOptions`])
pub type SkewSearchOptions = SkewDetectOptions;

/// Pivot point for sweep-and-search skew correction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkewPivot {
    /// Rotate about the top-left corner
    Corner,
    /// Rotate about the image center
    Center,
}

/// Automatically detect and correct skew using default settings.
///
/// # Errors
///
/// Returns an error if the image is empty or skew detection fails.
pub fn deskew(pix: &Pix) -> RecogResult<Pix> {
    let opts = SkewDetectOptions::default();
    let (deskewed, _) = find_skew_and_deskew(pix, &opts)?;
    Ok(deskewed)
}

/// Deskew and return both the corrected image and a binarised version.
///
/// Returns `(corrected_original, corrected_1bpp)`.
///
/// # Errors
///
/// Returns an error if the image is empty or skew detection fails.
pub fn deskew_both(pix: &Pix) -> RecogResult<(Pix, Pix)> {
    let opts = SkewDetectOptions::default();
    let (corrected, _) = find_skew_and_deskew(pix, &opts)?;
    let binary = ensure_binary(&corrected)?;
    Ok((corrected, binary))
}

/// Deskew with explicit options; returns `(corrected_image, detected_angle_deg)`.
///
/// # Errors
///
/// Returns an error if parameters are invalid or skew detection fails.
pub fn deskew_general(pix: &Pix, options: &DeskewOptions) -> RecogResult<(Pix, f32)> {
    options.validate()?;
    let detect_opts = SkewDetectOptions {
        sweep_range: options.sweep_range,
        sweep_delta: options.sweep_delta,
        min_bs_delta: 0.01,
        sweep_reduction: options.sweep_reduction,
        bs_reduction: options.search_reduction,
    };
    detect_opts.validate()?;
    let (corrected, result) = find_skew_and_deskew(pix, &detect_opts)?;
    Ok((corrected, result.angle))
}

/// Coarse sweep followed by binary-search refinement.
///
/// Equivalent to [`find_skew`] but exposed under the Leptonica-style name.
///
/// # Errors
///
/// Returns an error if the image is empty or parameters are invalid.
pub fn find_skew_sweep_and_search(
    pix: &Pix,
    options: &SkewSearchOptions,
) -> RecogResult<SkewResult> {
    find_skew(pix, options)
}

/// Sweep-and-search with score information.
///
/// Returns `(angle_deg, confidence, end_score)`.
///
/// The `end_score` is the differential-square-sum score at the final angle.
///
/// # Errors
///
/// Returns an error if the image is empty or parameters are invalid.
pub fn find_skew_sweep_and_search_score(
    pix: &Pix,
    options: &SkewSearchOptions,
) -> RecogResult<(f32, f32, f32)> {
    find_skew_sweep_and_search_score_pivot(pix, options, SkewPivot::Corner)
}

/// Sweep-and-search with score information and a pivot point.
///
/// Returns `(angle_deg, confidence, end_score)`.
///
/// * `SkewPivot::Corner` – standard shear-based sweep from the top-left corner.
/// * `SkewPivot::Center` – image is shifted so that the pivot is the center before sweeping.
///
/// # Errors
///
/// Returns an error if the image is empty or parameters are invalid.
pub fn find_skew_sweep_and_search_score_pivot(
    pix: &Pix,
    options: &SkewSearchOptions,
    pivot: SkewPivot,
) -> RecogResult<(f32, f32, f32)> {
    options.validate()?;

    let binary_pix = ensure_binary(pix)?;
    sweep_and_search_pivot(
        &binary_pix,
        options.sweep_reduction,
        options.bs_reduction,
        0.0,
        options.sweep_range,
        options.sweep_delta,
        options.min_bs_delta,
        pivot,
    )
}

/// Degrees-to-radians factor. C Leptonica hardcodes this literal (not `M_PI`)
/// in every skew function, and the sweep angles are sensitive to it.
#[allow(clippy::approx_constant, clippy::excessive_precision)]
const DEG2RAD: f32 = 3.1415926535 / 180.0;

/// C `pixReduceRankBinaryCascade` cascade for a requested reduction factor.
///
/// C uses `(1, 0, 0, 0)` for 2x, `(1, 1, 0, 0)` for 4x and `(1, 1, 2, 0)` for
/// 8x. A trailing 0 terminates the cascade.
fn reduce_for_search(pix: &Pix, reduction: u32) -> RecogResult<Pix> {
    let levels: &[u8] = match reduction {
        1 => return Ok(pix.clone()),
        2 => &[1],
        4 => &[1, 1],
        8 => &[1, 1, 2],
        _ => {
            return Err(RecogError::InvalidParameter(
                "reduction must be 1, 2, 4, or 8".to_string(),
            ));
        }
    };
    Ok(reduce_rank_binary_cascade(pix, levels)?)
}

/// C's second cascade, applied to the already-reduced search image to get the
/// sweep image: `(1, 0, 0, 0)`, `(1, 2, 0, 0)` or `(1, 2, 2, 0)`.
fn reduce_for_sweep(pix: &Pix, ratio: u32) -> RecogResult<Pix> {
    let levels: &[u8] = match ratio {
        1 => return Ok(pix.clone()),
        2 => &[1],
        4 => &[1, 2],
        _ => &[1, 2, 2],
    };
    Ok(reduce_rank_binary_cascade(pix, levels)?)
}

/// Vertical shear about the pivot, as C's sweep and binary search do it.
fn shear_about_pivot(pix: &Pix, theta_deg: f32, pivot: SkewPivot) -> RecogResult<Pix> {
    let radang = DEG2RAD * theta_deg;
    Ok(match pivot {
        SkewPivot::Corner => v_shear_corner(pix, radang, ShearFill::White)?,
        SkewPivot::Center => v_shear_center(pix, radang, ShearFill::White)?,
    })
}

/// C `pixFindSkewSweepAndSearchScorePivot`.
///
/// Returns `(angle_deg, confidence, end_score)`. When the sweep maximum lands
/// on either end of the sweep range, C warns and returns zeros; this does the
/// same rather than reporting an untrustworthy angle.
#[allow(clippy::too_many_arguments)]
fn sweep_and_search_pivot(
    pixs: &Pix,
    redsweep: u32,
    redsearch: u32,
    sweepcenter: f32,
    sweeprange: f32,
    sweepdelta: f32,
    minbsdelta: f32,
    pivot: SkewPivot,
) -> RecogResult<(f32, f32, f32)> {
    if !matches!(redsweep, 1 | 2 | 4 | 8) || !matches!(redsearch, 1 | 2 | 4 | 8) {
        return Err(RecogError::InvalidParameter(
            "reductions must be 1, 2, 4, or 8".to_string(),
        ));
    }
    if redsearch > redsweep {
        return Err(RecogError::InvalidParameter(
            "redsearch must not exceed redsweep".to_string(),
        ));
    }

    // Reduced image for the binary search, and a further reduced one for
    // the sweep. C derives the sweep image from the search image, not from
    // the source, so the cascades compose.
    let pixsch = reduce_for_search(pixs, redsearch)?;
    if is_image_empty(&pixsch) {
        return Err(RecogError::NoContent(
            "image is empty or all white".to_string(),
        ));
    }
    let pixsw = reduce_for_sweep(&pixsch, redsweep / redsearch)?;

    // C: nangles = (l_int32)((2. * sweeprange) / sweepdelta + 1), in double.
    let nangles = ((2.0_f64 * sweeprange as f64) / sweepdelta as f64 + 1.0) as i32;
    if nangles <= 0 {
        return Err(RecogError::InvalidParameter(
            "sweep range and delta yield no angles".to_string(),
        ));
    }

    // Sweep.
    let rangeleft = sweepcenter - sweeprange;
    let mut maxscore = f32::MIN;
    let mut maxindex = 0i32;
    let mut maxangle = 0.0f32;
    for i in 0..nangles {
        let theta = rangeleft + i as f32 * sweepdelta;
        let sheared = shear_about_pivot(&pixsw, theta, pivot)?;
        let sum = find_differential_square_sum(&sheared)?;
        if sum > maxscore {
            maxscore = sum;
            maxindex = i;
            maxangle = theta;
        }
    }

    // C warns and bails out when the maximum is at a sweep edge.
    if maxindex == 0 || maxindex == nangles - 1 {
        return Ok((0.0, 0.0, 0.0));
    }

    // Binary search. `scores` holds C's bsearchscore[5]; `bs_scores` collects
    // every score evaluated here, which is what the confidence minimum uses
    // (the sweep scores are discarded first).
    let mut centerangle = maxangle;
    let mut scores = [0.0f32; 5];
    scores[2] = find_differential_square_sum(&shear_about_pivot(&pixsch, centerangle, pivot)?)?;
    scores[0] = find_differential_square_sum(&shear_about_pivot(
        &pixsch,
        centerangle - sweepdelta,
        pivot,
    )?)?;
    scores[4] = find_differential_square_sum(&shear_about_pivot(
        &pixsch,
        centerangle + sweepdelta,
        pivot,
    )?)?;
    let mut bs_scores = vec![scores[2], scores[0], scores[4]];

    // C reuses the single `maxscore` variable for both phases: it still holds
    // the sweep maximum here (skew.c:774) and is only overwritten inside the
    // loop body (skew.c:868). So when `minbsdelta > 0.5 * sweepdelta` and the
    // loop never runs, the confidence below is computed from the sweep score.
    // That mixes scales (the sweep runs on `pixsw`, the search on `pixsch`),
    // but it is C's behaviour and callers depend on the same numbers.
    let mut delta = 0.5 * sweepdelta;
    while delta >= minbsdelta {
        let leftcenterangle = centerangle - delta;
        scores[1] =
            find_differential_square_sum(&shear_about_pivot(&pixsch, leftcenterangle, pivot)?)?;
        bs_scores.push(scores[1]);

        let rightcenterangle = centerangle + delta;
        scores[3] =
            find_differential_square_sum(&shear_about_pivot(&pixsch, rightcenterangle, pivot)?)?;
        bs_scores.push(scores[3]);

        // The maximum must be one of the middle three, not an end value.
        maxscore = scores[1];
        let mut bsindex = 1usize;
        for (i, &score) in scores.iter().enumerate().take(4).skip(2) {
            if score > maxscore {
                maxscore = score;
                bsindex = i;
            }
        }

        let lefttemp = scores[bsindex - 1];
        let righttemp = scores[bsindex + 1];
        scores[2] = maxscore;
        scores[0] = lefttemp;
        scores[4] = righttemp;

        centerangle += delta * (bsindex as f32 - 2.0);
        delta *= 0.5;
    }
    let endscore = scores[2];

    // Confidence: max/min score ratio, distrusted when the minimum is too
    // small for the image dimensions, when the angle sits at the edge of the
    // sweep range, or when the maximum score itself is tiny.
    let minscore = bs_scores.iter().copied().fold(f32::MAX, f32::min);
    let width = pixsch.width() as f32;
    let height = pixsch.height() as f32;
    let minthresh = MIN_SCORE_THRESH_FACTOR * width * width * height;
    let mut conf = if minscore > minthresh {
        maxscore / minscore
    } else {
        0.0
    };
    if centerangle > rangeleft + 2.0 * sweeprange - sweepdelta
        || centerangle < rangeleft + sweepdelta
        || maxscore < MIN_VALID_MAX_SCORE
    {
        conf = 0.0;
    }

    Ok((centerangle, conf, endscore))
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
    if pix.depth() == PixelDepth::Bit1 {
        let w = pix.width();
        let wpl = pix.wpl() as usize;
        let bits_used = w % 32;
        let full_words = (w / 32) as usize;
        let end_mask = if bits_used == 0 {
            0xFFFF_FFFF
        } else {
            !((1u32 << (32 - bits_used)) - 1)
        };
        for y in 0..pix.height() {
            let line = pix.row_data(y);
            if line[..full_words].iter().any(|&word| word != 0) {
                return false;
            }
            if bits_used != 0 && full_words < wpl && (line[full_words] & end_mask) != 0 {
                return false;
            }
        }
        return true;
    }

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if pix.get_pixel_unchecked(x, y) != 0 {
                return false;
            }
        }
    }
    true
}

/// Finds the skew angle using only the sweep phase (no binary search).
///
/// This is a low-level function that does a single pass through the angle
/// range.  For most uses, prefer [`find_skew`] which also refines with
/// binary search.
///
/// Corresponds to `pixFindSkewSweep` in C Leptonica.
///
/// # Arguments
///
/// * `pix` - Input image (1 bpp binary recommended)
/// * `reduction` - Reduction factor: 1, 2, 4, or 8
/// * `sweep_range` - Half the sweep range in degrees
/// * `sweep_delta` - Angle increment in degrees
pub fn find_skew_sweep(
    pix: &Pix,
    reduction: u32,
    sweep_range: f32,
    sweep_delta: f32,
) -> RecogResult<f32> {
    if !matches!(reduction, 1 | 2 | 4 | 8) {
        return Err(RecogError::InvalidParameter(
            "reduction must be 1, 2, 4, or 8".to_string(),
        ));
    }
    if sweep_range <= 0.0 || sweep_delta <= 0.0 {
        return Err(RecogError::InvalidParameter(
            "sweep_range and sweep_delta must be positive".to_string(),
        ));
    }

    let binary_pix = ensure_binary(pix)?;
    let reduced = reduce_for_search(&binary_pix, reduction)?;
    if is_image_empty(&reduced) {
        return Err(RecogError::NoContent(
            "image is empty or all white".to_string(),
        ));
    }

    let nangles = ((2.0_f64 * sweep_range as f64) / sweep_delta as f64 + 1.0) as i32;
    let mut nascore = Numa::new();
    let mut natheta = Numa::new();
    for i in 0..nangles {
        let theta = -sweep_range + i as f32 * sweep_delta;
        let sheared = shear_about_pivot(&reduced, theta, SkewPivot::Corner)?;
        nascore.push(find_differential_square_sum(&sheared)?);
        natheta.push(theta);
    }

    // C interpolates a parabola through the maximum rather than taking the
    // raw argmax, so the returned angle can lie between two sweep steps.
    let (_, maxangle) = nascore.fit_max(Some(&natheta))?;
    Ok(maxangle)
}

/// Finds the skew angle by searching in two orthogonal directions.
///
/// First searches around 0° (landscape), then around 90° (portrait),
/// and returns the result with higher confidence.
///
/// Corresponds to `pixFindSkewOrthogonalRange` in C Leptonica.
///
/// # Arguments
///
/// * `pix` - Input binary image
/// * `redsweep` - Reduction factor for sweep phase
/// * `redsearch` - Reduction factor for search phase
/// * `sweep_range` - Half sweep range in degrees
/// * `sweep_delta` - Angle increment in degrees
/// * `minbs_delta` - Minimum binary search delta
/// * `confprior` - Confidence penalty for 90° result
///
/// # Returns
///
/// `(angle, confidence)` for best direction
pub fn find_skew_orthogonal_range(
    pix: &Pix,
    redsweep: u32,
    redsearch: u32,
    sweep_range: f32,
    sweep_delta: f32,
    minbs_delta: f32,
    confprior: f32,
) -> RecogResult<(f32, f32)> {
    let binary_pix = ensure_binary(pix)?;

    let (angle1, conf1, _) = sweep_and_search_pivot(
        &binary_pix,
        redsweep,
        redsearch,
        0.0,
        sweep_range,
        sweep_delta,
        minbs_delta,
        SkewPivot::Corner,
    )?;

    // C rotates by one quadrant (90 degrees clockwise) and searches again.
    let rotated = rotate_orth(&binary_pix, 1)?;
    let (angle2, conf2, _) = sweep_and_search_pivot(
        &rotated,
        redsweep,
        redsearch,
        0.0,
        sweep_range,
        sweep_delta,
        minbs_delta,
        SkewPivot::Corner,
    )?;

    if conf1 > conf2 - confprior {
        Ok((angle1, conf1))
    } else {
        Ok((-90.0 + angle2, conf2))
    }
}

/// Sum of squared differences between adjacent row pixel-counts.
///
/// C Leptonica equivalent: `pixFindDifferentialSquareSum`.
///
/// At the top and bottom we skip at least one scanline, no more than 10% of
/// the image height, and no more than 5% of the image width. This is the
/// score used internally by `find_skew_sweep_and_search`.
pub fn find_differential_square_sum(pix: &Pix) -> RecogResult<f32> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1bpp",
            actual: pix.depth().bits(),
        });
    }
    let na = pix.count_by_row(None)?;
    let w = pix.width() as i32;
    let h = pix.height() as i32;
    // Match C `pixFindDifferentialSquareSum`: skip = min(h/10, 0.05*w),
    // nskip = max(skip/2, 1).
    // C evaluates `0.05 * w` in double, then truncates.
    let skiph = (0.05 * w as f64) as i32;
    let skip = (h / 10).min(skiph);
    let nskip = (skip / 2).max(1) as usize;
    let n = na.len();
    if n <= nskip {
        return Ok(0.0);
    }
    let mut sum = 0.0f32;
    for i in nskip..(n - nskip) {
        let v1 = na.get(i - 1).unwrap_or(0.0);
        let v2 = na.get(i).unwrap_or(0.0);
        let diff = v2 - v1;
        sum += diff * diff;
    }
    Ok(sum)
}

/// Per-axis normalized sum of squared row/column pixel-counts on a 1bpp image.
///
/// Returns `(hratio, vratio, fract)` where:
/// - `hratio` — ratio of horizontal-row sum-of-squares to the uniform value
/// - `vratio` — same for vertical (after a 90° rotation)
/// - `fract`  — ratio of foreground pixels to total pixels
///
/// All three are `0.0` if the image has no foreground.
///
/// C Leptonica equivalent: `pixFindNormalizedSquareSum`.
pub fn find_normalized_square_sum(pix: &Pix) -> RecogResult<(f32, f32, f32)> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1bpp",
            actual: pix.depth().bits(),
        });
    }
    let w = pix.width() as f32;
    let h = pix.height() as f32;

    // Horizontal (per-row) ratio
    let na_h = pix.count_by_row(None)?;
    let sum_h: f32 = na_h.iter().sum();
    let fract = if w > 0.0 && h > 0.0 {
        sum_h / (w * h)
    } else {
        0.0
    };
    if sum_h == 0.0 {
        return Ok((0.0, 0.0, 0.0));
    }
    let uniform_h = sum_h * sum_h / h;
    let sumsq_h: f32 = na_h.iter().map(|v| v * v).sum();
    let hratio = sumsq_h / uniform_h;

    // Vertical (per-column via 90° rotation)
    let pix_rot = crate::transform::rotate_orth(pix, 1)?;
    let na_v = pix_rot.count_by_row(None)?;
    let sum_v: f32 = na_v.iter().sum();
    if sum_v == 0.0 {
        return Ok((hratio, 0.0, fract));
    }
    let uniform_v = sum_v * sum_v / w;
    let sumsq_v: f32 = na_v.iter().map(|v| v * v).sum();
    let vratio = sumsq_v / uniform_v;

    Ok((hratio, vratio, fract))
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn create_horizontal_lines_image(w: u32, h: u32, line_spacing: u32) -> Pix {
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
    fn test_shear_about_pivot_zero_angle() {
        let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
        let sheared = shear_about_pivot(&pix, 0.0, SkewPivot::Corner).unwrap();
        assert_eq!(sheared.width(), 50);
        assert_eq!(sheared.height(), 50);
    }

    #[test]
    fn test_shear_about_pivot_keeps_size() {
        // C's pixVShear writes into a pix of the same size as the source, so
        // the sheared image never grows; content shifts out of frame instead.
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        for pivot in [SkewPivot::Corner, SkewPivot::Center] {
            let sheared = shear_about_pivot(&pix, 5.0, pivot).unwrap();
            assert_eq!(sheared.width(), 100);
            assert_eq!(sheared.height(), 100);
        }
    }

    #[test]
    fn test_differential_square_sum_positive() {
        let pix = create_horizontal_lines_image(200, 200, 20);
        let score = find_differential_square_sum(&pix).unwrap();
        assert!(score > 0.0);
    }

    #[test]
    fn test_reduce_for_search() {
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let reduced = reduce_for_search(&pix, 2).unwrap();
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
    fn test_deskew_by_angle_zero() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let deskewed = deskew_by_angle(&pix, 0.0).unwrap();
        assert_eq!(deskewed.width(), 100);
        assert_eq!(deskewed.height(), 100);
    }

    #[test]
    fn test_ensure_binary_from_grayscale() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let binary = ensure_binary(&pix).unwrap();
        assert_eq!(binary.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_deskew_auto_smoke() {
        // deskew on a 1bpp image with horizontal lines should succeed.
        let pix = create_horizontal_lines_image(400, 400, 30);
        let result = deskew(&pix);
        assert!(result.is_ok());
        let out = result.unwrap();
        assert_eq!(out.width(), pix.width());
    }

    #[test]
    fn test_deskew_both_smoke() {
        let pix = create_horizontal_lines_image(400, 400, 30);
        let (orig_out, bpp1_out) = deskew_both(&pix).unwrap();
        assert_eq!(orig_out.width(), pix.width());
        assert_eq!(bpp1_out.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_deskew_general_returns_angle() {
        let pix = create_horizontal_lines_image(400, 400, 30);
        let opts = DeskewOptions::default();
        let (out, angle) = deskew_general(&pix, &opts).unwrap();
        assert_eq!(out.width(), pix.width());
        assert!(angle.abs() < 10.0, "angle {angle} out of expected range");
    }

    #[test]
    fn test_find_skew_sweep_and_search_smoke() {
        let pix = create_horizontal_lines_image(400, 400, 30);
        let opts = SkewSearchOptions::default();
        let result = find_skew_sweep_and_search(&pix, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_skew_sweep_and_search_score_smoke() {
        let pix = create_horizontal_lines_image(400, 400, 30);
        let opts = SkewSearchOptions::default();
        let (angle, conf, endscore) = find_skew_sweep_and_search_score(&pix, &opts).unwrap();
        assert!(angle.abs() < 10.0);
        let _ = (conf, endscore);
    }

    #[test]
    fn test_find_skew_sweep_and_search_score_pivot_center() {
        let pix = create_horizontal_lines_image(400, 400, 30);
        let opts = SkewSearchOptions::default();
        let r_corner = find_skew_sweep_and_search_score_pivot(&pix, &opts, SkewPivot::Corner);
        let r_center = find_skew_sweep_and_search_score_pivot(&pix, &opts, SkewPivot::Center);
        assert!(r_corner.is_ok());
        assert!(r_center.is_ok());
    }
}
