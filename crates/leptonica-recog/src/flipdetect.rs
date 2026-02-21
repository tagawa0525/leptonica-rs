//! Flip detection - page orientation and mirror detection
//!
//! Detects the orientation of text in document images and corrects it.
//! Uses Hit-Miss Transform (HMT) with specialized text structuring elements
//! to detect ascender/descender patterns.
//!
//! Based on C leptonica `flipdetect.c`.
//!
//! # Functions
//!
//! - [`orient_detect`]: Detect page orientation (0/90/180/270 degrees)
//! - [`make_orient_decision`]: Make orientation decision from confidence values
//! - [`orient_correct`]: Detect and correct page orientation
//! - [`up_down_detect`]: Detect if text is rightside-up or upside-down
//! - [`mirror_detect`]: Detect if text is mirror-reversed

use leptonica_core::{Pix, PixelDepth};
use leptonica_morph::Sel;
use leptonica_morph::{hit_miss_transform, morph_comp_sequence};
use leptonica_transform::{rotate_90, rotate_orth};

use crate::{RecogError, RecogResult};

// --- Text SEL patterns (6 wide x 5 tall) ---
// These detect ascenders (characters like b, d, h, k, l) and descenders (g, p, q)
// 'x' = hit, 'o' = miss, ' ' = don't care

/// Right-facing UP ascender pattern
const TEXTSEL1: &str = "\
x  oo \n\
x oOo \n\
x  o  \n\
x     \n\
xxxxxx";
const TEXTSEL1_ORIGIN: (u32, u32) = (3, 1);

/// Left-facing UP ascender pattern
const TEXTSEL2: &str = "\
 oo  x\n\
 oOo x\n\
  o  x\n\
     x\n\
xxxxxx";
const TEXTSEL2_ORIGIN: (u32, u32) = (2, 1);

/// Right-facing DOWN descender pattern
const TEXTSEL3: &str = "\
xxxxxx\n\
x     \n\
x  o  \n\
x oOo \n\
x  oo ";
const TEXTSEL3_ORIGIN: (u32, u32) = (3, 3);

/// Left-facing DOWN descender pattern
const TEXTSEL4: &str = "\
xxxxxx\n\
     x\n\
  o  x\n\
 oOo x\n\
 oo  x";
const TEXTSEL4_ORIGIN: (u32, u32) = (2, 3);

// --- Default parameters ---

/// Default minimum count of (up + down) hits for orientation detection
const DEFAULT_MIN_UP_DOWN_COUNT: u32 = 70;

/// Default minimum confidence for orientation decision
const DEFAULT_MIN_UP_DOWN_CONF: f32 = 8.0;

/// Default minimum ratio for orientation decision
const DEFAULT_MIN_UP_DOWN_RATIO: f32 = 2.5;

/// Default minimum count for mirror flip detection
const DEFAULT_MIN_MIRROR_FLIP_COUNT: u32 = 100;

// --- Types ---

/// Text orientation detected from ascender/descender analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOrientation {
    /// Not enough evidence to determine orientation
    Unknown,
    /// Text is rightside-up (no rotation needed)
    Up,
    /// Landscape, text oriented left (needs 90 deg CW rotation)
    Left,
    /// Text is upside-down (needs 180 deg rotation)
    Down,
    /// Landscape, text oriented right (needs 270 deg CW rotation)
    Right,
}

/// Result of orientation detection
#[derive(Debug, Clone)]
pub struct OrientDetectResult {
    /// Confidence that text is rightside-up (positive = up, negative = down)
    pub up_confidence: f32,
    /// Confidence for left orientation (positive = left, negative = right)
    pub left_confidence: f32,
}

/// Result of orientation correction
#[derive(Debug)]
pub struct OrientCorrectResult {
    /// The corrected image
    pub pix: Pix,
    /// Confidence that text is rightside-up
    pub up_confidence: f32,
    /// Confidence for left orientation
    pub left_confidence: f32,
    /// Rotation applied in degrees (0, 90, 180, 270)
    pub rotation: u32,
    /// Detected orientation
    pub orientation: TextOrientation,
}

/// Detect if text is rightside-up or upside-down.
///
/// Uses HMT with specialized text SELs to count ascenders pointing up vs down.
/// Returns a confidence value: positive means rightside-up, negative means upside-down.
///
/// # Arguments
/// * `pix` - 1 bpp deskewed document image (150-300 ppi)
/// * `min_count` - Minimum number of (up + down) hits; use 0 for default (70)
/// * `npixels` - Number of pixels to trim from word boundaries; use 0 for typical mode
pub fn up_down_detect(pix: &Pix, min_count: u32, npixels: u32) -> RecogResult<f32> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    let min_count = if min_count == 0 {
        DEFAULT_MIN_UP_DOWN_COUNT
    } else {
        min_count
    };

    // Create the 4 text SELs
    let sel1 = Sel::from_string(TEXTSEL1, TEXTSEL1_ORIGIN.0, TEXTSEL1_ORIGIN.1)?;
    let sel2 = Sel::from_string(TEXTSEL2, TEXTSEL2_ORIGIN.0, TEXTSEL2_ORIGIN.1)?;
    let sel3 = Sel::from_string(TEXTSEL3, TEXTSEL3_ORIGIN.0, TEXTSEL3_ORIGIN.1)?;
    let sel4 = Sel::from_string(TEXTSEL4, TEXTSEL4_ORIGIN.0, TEXTSEL4_ORIGIN.1)?;

    // Pre-filter: close holes in x-height characters and join at x-height
    let pix0 = morph_comp_sequence(pix, "c1.8 + c30.1")?;

    // Optionally create a mask trimming word boundaries
    let mask = if npixels > 0 {
        create_word_boundary_mask(&pix0, npixels)?
    } else {
        None
    };

    // Find UP ascenders (sel1 + sel2) and count
    let hmt_up1 = hit_miss_transform(&pix0, &sel1)?;
    let hmt_up2 = hit_miss_transform(&pix0, &sel2)?;
    let mut hmt_up = hmt_up1.or(&hmt_up2)?;
    if let Some(ref m) = mask {
        hmt_up = hmt_up.and(m)?;
    }
    let reduced_up = reduce_rank_binary_cascade_2(&hmt_up)?;
    let count_up = reduced_up.count_pixels();

    // Find DOWN descenders (sel3 + sel4) and count
    let hmt_down1 = hit_miss_transform(&pix0, &sel3)?;
    let hmt_down2 = hit_miss_transform(&pix0, &sel4)?;
    let mut hmt_down = hmt_down1.or(&hmt_down2)?;
    if let Some(ref m) = mask {
        hmt_down = hmt_down.and(m)?;
    }
    let reduced_down = reduce_rank_binary_cascade_2(&hmt_down)?;
    let count_down = reduced_down.count_pixels();

    // Compute confidence using gaussian-like statistic
    let nup = count_up as f32;
    let ndown = count_down as f32;
    let nmax = count_up.max(count_down);

    let conf = if nmax > min_count as u64 {
        2.0 * (nup - ndown) / (nup + ndown).sqrt()
    } else {
        0.0
    };

    Ok(conf)
}

/// Detect page orientation (four 90 degree angles).
///
/// Analyzes the image both as-is and rotated 90 degrees to determine
/// text orientation. Returns confidence values for up/down and left/right.
///
/// # Arguments
/// * `pix` - 1 bpp deskewed document image (150-300 ppi)
/// * `min_count` - Minimum hit count; use 0 for default (70)
pub fn orient_detect(pix: &Pix, min_count: u32) -> RecogResult<OrientDetectResult> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    let min_count = if min_count == 0 {
        DEFAULT_MIN_UP_DOWN_COUNT
    } else {
        min_count
    };

    // Detect up/down confidence on original image
    let up_confidence = up_down_detect(pix, min_count, 0)?;

    // Detect left/right by rotating 90 degrees CW and testing up/down
    let rotated = rotate_90(pix, true)?;
    let left_confidence = up_down_detect(&rotated, min_count, 0)?;

    Ok(OrientDetectResult {
        up_confidence,
        left_confidence,
    })
}

/// Make an orientation decision from confidence values.
///
/// # Arguments
/// * `up_conf` - Confidence from up/down detection (must be nonzero)
/// * `left_conf` - Confidence from left/right detection (must be nonzero)
/// * `min_up_conf` - Minimum confidence for decision; use 0.0 for default (8.0)
/// * `min_ratio` - Minimum ratio for decision; use 0.0 for default (2.5)
pub fn make_orient_decision(
    up_conf: f32,
    left_conf: f32,
    min_up_conf: f32,
    min_ratio: f32,
) -> TextOrientation {
    if up_conf == 0.0 || left_conf == 0.0 {
        return TextOrientation::Unknown;
    }

    let min_up_conf = if min_up_conf == 0.0 {
        DEFAULT_MIN_UP_DOWN_CONF
    } else {
        min_up_conf
    };
    let min_ratio = if min_ratio == 0.0 {
        DEFAULT_MIN_UP_DOWN_RATIO
    } else {
        min_ratio
    };

    let abs_up = up_conf.abs();
    let abs_left = left_conf.abs();

    if up_conf > min_up_conf && abs_up > min_ratio * abs_left {
        TextOrientation::Up
    } else if left_conf > min_up_conf && abs_left > min_ratio * abs_up {
        TextOrientation::Left
    } else if up_conf < -min_up_conf && abs_up > min_ratio * abs_left {
        TextOrientation::Down
    } else if left_conf < -min_up_conf && abs_left > min_ratio * abs_up {
        TextOrientation::Right
    } else {
        TextOrientation::Unknown
    }
}

/// Detect and correct page orientation.
///
/// High-level function that detects orientation and rotates the image
/// to make text rightside-up.
///
/// # Arguments
/// * `pix` - 1 bpp deskewed document image (150-300 ppi)
/// * `min_up_conf` - Minimum confidence for decision; use 0.0 for default
/// * `min_ratio` - Minimum ratio for decision; use 0.0 for default
pub fn orient_correct(
    pix: &Pix,
    min_up_conf: f32,
    min_ratio: f32,
) -> RecogResult<OrientCorrectResult> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    // Detect orientation
    let up_confidence = up_down_detect(pix, 0, 0)?;
    let rotated_90 = rotate_90(pix, true)?;
    let left_confidence = up_down_detect(&rotated_90, 0, 0)?;

    let orientation = make_orient_decision(up_confidence, left_confidence, min_up_conf, min_ratio);

    let (result_pix, rotation) = match orientation {
        TextOrientation::Unknown | TextOrientation::Up => (pix.clone(), 0),
        TextOrientation::Left => (rotate_orth(pix, 1)?, 90),
        TextOrientation::Down => (rotate_orth(pix, 2)?, 180),
        TextOrientation::Right => (rotate_orth(pix, 3)?, 270),
    };

    Ok(OrientCorrectResult {
        pix: result_pix,
        up_confidence,
        left_confidence,
        rotation,
        orientation,
    })
}

/// Detect if text is mirror-reversed (left-right flip).
///
/// For this test, text must be horizontally oriented with ascenders going up.
/// Returns a confidence value: positive means normal, negative means mirror-reversed.
///
/// # Arguments
/// * `pix` - 1 bpp deskewed document image (150-300 ppi)
/// * `min_count` - Minimum count of (left + right) hits; use 0 for default (100)
pub fn mirror_detect(pix: &Pix, min_count: u32) -> RecogResult<f32> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    let min_count = if min_count == 0 {
        DEFAULT_MIN_MIRROR_FLIP_COUNT
    } else {
        min_count
    };

    let sel1 = Sel::from_string(TEXTSEL1, TEXTSEL1_ORIGIN.0, TEXTSEL1_ORIGIN.1)?;
    let sel2 = Sel::from_string(TEXTSEL2, TEXTSEL2_ORIGIN.0, TEXTSEL2_ORIGIN.1)?;

    // Pre-filter: fill x-height characters but not space between them
    // 1. Vertical dilation to extend vertically, XOR with original
    let pix_vdil = morph_comp_sequence(pix, "d1.30")?;
    let pix_vdil_xor = pix_vdil.xor(pix)?;

    // 2. Horizontal close to fill x-height, XOR with original
    let pix_hclose = morph_comp_sequence(pix, "c15.1")?;
    let pix_hclose_xor = pix_hclose.xor(pix)?;

    // 3. Intersect the two XOR results, then add back original
    let pix_combined = pix_hclose_xor.and(&pix_vdil_xor)?;
    let pix0 = pix_combined.or(pix)?;

    // Filter right-facing characters (sel1)
    let hmt_right = hit_miss_transform(&pix0, &sel1)?;
    let reduced_right = reduce_rank_binary_cascade_2(&hmt_right)?;
    let count_right = reduced_right.count_pixels();

    // Filter left-facing characters (sel2)
    let hmt_left = hit_miss_transform(&pix0, &sel2)?;
    let reduced_left = reduce_rank_binary_cascade_2(&hmt_left)?;
    let count_left = reduced_left.count_pixels();

    let nright = count_right as f32;
    let nleft = count_left as f32;
    let nmax = count_right.max(count_left);

    let conf = if nmax > min_count as u64 {
        2.0 * (nright - nleft) / (nright + nleft).sqrt()
    } else {
        0.0
    };

    Ok(conf)
}

// --- Private helpers ---

/// Create a mask that trims `npixels` from each end of word bounding boxes.
/// Used in `up_down_detect` when `npixels > 0`.
fn create_word_boundary_mask(pix: &Pix, npixels: u32) -> RecogResult<Option<Pix>> {
    use leptonica_morph::morph_sequence;

    // Open to find word regions
    let word_pix = morph_sequence(pix, "o10.1")?;

    let w = word_pix.width();
    let h = word_pix.height();
    let mask = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut mask_mut = mask.try_into_mut().unwrap();

    // Scan for horizontal runs (connected components on rows)
    // Simple approach: find horizontal runs of ON pixels in the opened image
    for y in 0..h {
        let mut x = 0u32;
        while x < w {
            // Find start of a run
            if word_pix.get_pixel(x, y).unwrap_or(0) != 0 {
                let run_start = x;
                // Find end of run
                while x < w && word_pix.get_pixel(x, y).unwrap_or(0) != 0 {
                    x += 1;
                }
                let run_end = x; // exclusive
                let run_width = run_end - run_start;

                // Only include if wider than 2*npixels
                if run_width > 2 * npixels {
                    let trimmed_start = run_start + npixels;
                    let trimmed_end = run_end - npixels;
                    // Set a vertical band in the mask (extend 6 pixels above, 13 below)
                    let y_start = y.saturating_sub(6);
                    let y_end = (y + 13).min(h);
                    for my in y_start..y_end {
                        for mx in trimmed_start..trimmed_end {
                            mask_mut.set_pixel(mx, my, 1).ok();
                        }
                    }
                }
            } else {
                x += 1;
            }
        }
    }

    Ok(Some(Pix::from(mask_mut)))
}

/// Reduce a 1bpp image by 2x using rank-1 (OR) reduction.
/// Each 2x2 block becomes ON if any pixel in the block is ON.
fn reduce_rank_binary_2x(pix: &Pix) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let new_w = w.div_ceil(2);
    let new_h = h.div_ceil(2);

    let result = Pix::new(new_w, new_h, PixelDepth::Bit1)?;
    let mut result_mut = result.try_into_mut().unwrap();

    for dy in 0..new_h {
        let sy = dy * 2;
        for dx in 0..new_w {
            let sx = dx * 2;
            let p00 = pix.get_pixel_unchecked(sx, sy);
            let p10 = if sx + 1 < w {
                pix.get_pixel_unchecked(sx + 1, sy)
            } else {
                0
            };
            let p01 = if sy + 1 < h {
                pix.get_pixel_unchecked(sx, sy + 1)
            } else {
                0
            };
            let p11 = if sx + 1 < w && sy + 1 < h {
                pix.get_pixel_unchecked(sx + 1, sy + 1)
            } else {
                0
            };
            if (p00 | p10 | p01 | p11) != 0 {
                result_mut.set_pixel_unchecked(dx, dy, 1);
            }
        }
    }

    Ok(Pix::from(result_mut))
}

/// Apply two levels of 2x rank-1 reduction (4x total).
/// Equivalent to `pixReduceRankBinaryCascade(pix, 1, 1, 0, 0)`.
fn reduce_rank_binary_cascade_2(pix: &Pix) -> RecogResult<Pix> {
    let reduced1 = reduce_rank_binary_2x(pix)?;
    reduce_rank_binary_2x(&reduced1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::PixelDepth;

    fn make_1bpp_image(w: u32, h: u32) -> Pix {
        Pix::new(w, h, PixelDepth::Bit1).unwrap()
    }

    // --- up_down_detect tests ---

    #[test]
    fn test_up_down_detect_requires_1bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let result = up_down_detect(&pix, 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_up_down_detect_empty_image_returns_zero_conf() {
        let pix = make_1bpp_image(200, 200);
        let conf = up_down_detect(&pix, 0, 0).unwrap();
        assert!((conf).abs() < 0.001);
    }

    #[test]
    fn test_up_down_detect_default_min_count() {
        // When min_count=0, uses DEFAULT_MIN_UP_DOWN_COUNT (70)
        let pix = make_1bpp_image(200, 200);
        let conf = up_down_detect(&pix, 0, 0).unwrap();
        assert!((conf).abs() < 0.001);
    }

    // --- orient_detect tests ---

    #[test]
    fn test_orient_detect_requires_1bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let result = orient_detect(&pix, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_orient_detect_empty_image() {
        let pix = make_1bpp_image(200, 200);
        let result = orient_detect(&pix, 0).unwrap();
        assert!((result.up_confidence).abs() < 0.001);
        assert!((result.left_confidence).abs() < 0.001);
    }

    // --- make_orient_decision tests ---

    #[test]
    fn test_make_orient_decision_zero_conf_returns_unknown() {
        let orient = make_orient_decision(0.0, 0.0, 0.0, 0.0);
        assert_eq!(orient, TextOrientation::Unknown);
    }

    #[test]
    fn test_make_orient_decision_strong_up() {
        // Strong up confidence, weak left confidence
        let orient = make_orient_decision(20.0, 2.0, 0.0, 0.0);
        assert_eq!(orient, TextOrientation::Up);
    }

    #[test]
    fn test_make_orient_decision_strong_left() {
        let orient = make_orient_decision(2.0, 20.0, 0.0, 0.0);
        assert_eq!(orient, TextOrientation::Left);
    }

    #[test]
    fn test_make_orient_decision_strong_down() {
        let orient = make_orient_decision(-20.0, 2.0, 0.0, 0.0);
        assert_eq!(orient, TextOrientation::Down);
    }

    #[test]
    fn test_make_orient_decision_strong_right() {
        let orient = make_orient_decision(2.0, -20.0, 0.0, 0.0);
        assert_eq!(orient, TextOrientation::Right);
    }

    #[test]
    fn test_make_orient_decision_ambiguous_returns_unknown() {
        // Both similar magnitude → unknown
        let orient = make_orient_decision(10.0, 9.0, 0.0, 0.0);
        assert_eq!(orient, TextOrientation::Unknown);
    }

    #[test]
    fn test_make_orient_decision_custom_thresholds() {
        // With higher min_up_conf, a moderate confidence is not enough
        let orient = make_orient_decision(5.0, 1.0, 10.0, 2.5);
        assert_eq!(orient, TextOrientation::Unknown);
    }

    // --- orient_correct tests ---

    #[test]
    fn test_orient_correct_requires_1bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let result = orient_correct(&pix, 0.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_orient_correct_empty_returns_copy() {
        let pix = make_1bpp_image(200, 200);
        let result = orient_correct(&pix, 0.0, 0.0).unwrap();
        assert_eq!(result.rotation, 0);
        assert_eq!(result.pix.width(), 200);
        assert_eq!(result.pix.height(), 200);
    }

    // --- mirror_detect tests ---

    #[test]
    fn test_mirror_detect_requires_1bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let result = mirror_detect(&pix, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_mirror_detect_empty_image_returns_zero_conf() {
        let pix = make_1bpp_image(200, 200);
        let conf = mirror_detect(&pix, 0).unwrap();
        assert!((conf).abs() < 0.001);
    }

    // --- reduce_rank_binary tests ---

    #[test]
    fn test_reduce_rank_binary_2x_dimensions() {
        let pix = make_1bpp_image(100, 80);
        let reduced = reduce_rank_binary_2x(&pix).unwrap();
        assert_eq!(reduced.width(), 50);
        assert_eq!(reduced.height(), 40);
    }

    #[test]
    fn test_reduce_rank_binary_2x_odd_dimensions() {
        let pix = make_1bpp_image(101, 81);
        let reduced = reduce_rank_binary_2x(&pix).unwrap();
        // div_ceil: 101.div_ceil(2) = 51, 81.div_ceil(2) = 41
        assert_eq!(reduced.width(), 51);
        assert_eq!(reduced.height(), 41);
    }

    #[test]
    fn test_reduce_rank_binary_2x_preserves_on_pixels() {
        let pix = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Set one pixel in the top-left 2x2 block
        pix_mut.set_pixel(0, 0, 1).unwrap();
        let pix = Pix::from(pix_mut);

        let reduced = reduce_rank_binary_2x(&pix).unwrap();
        assert_eq!(reduced.width(), 2);
        assert_eq!(reduced.height(), 2);
        // Top-left block should be ON
        assert_eq!(reduced.get_pixel(0, 0).unwrap(), 1);
        // Other blocks should be OFF
        assert_eq!(reduced.get_pixel(1, 0).unwrap(), 0);
        assert_eq!(reduced.get_pixel(0, 1).unwrap(), 0);
        assert_eq!(reduced.get_pixel(1, 1).unwrap(), 0);
    }

    #[test]
    fn test_reduce_rank_binary_cascade_2_dimensions() {
        let pix = make_1bpp_image(100, 80);
        let reduced = reduce_rank_binary_cascade_2(&pix).unwrap();
        assert_eq!(reduced.width(), 25);
        assert_eq!(reduced.height(), 20);
    }
}
