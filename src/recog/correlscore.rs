//! Correlation score computation for binary images
//!
//! Functions for computing correlation between binary (1-bpp) images.
//! These are used primarily in character recognition to compare unknown
//! characters against templates.
//!
//! # Reference
//!
//! Based on Leptonica's `correlscore.c`.

use crate::core::{Error, Pix, PixelDepth, Result};

/// Compute the correlation score between two binary images.
///
/// The correlation is computed over the overlap region defined by the
/// given area and sum counts. This is an optimized version that uses
/// pre-computed pixel counts.
///
/// # Arguments
///
/// * `pix1` - First 1-bpp image
/// * `pix2` - Second 1-bpp image
/// * `area1` - Number of ON pixels in pix1
/// * `area2` - Number of ON pixels in pix2
/// * `x_offset` - Horizontal offset of pix2 relative to pix1
/// * `y_offset` - Vertical offset of pix2 relative to pix1
///
/// # Returns
///
/// The correlation score in [0.0, 1.0].
///
/// # Reference
///
/// C Leptonica: `pixCorrelationScore()`
pub fn correlation_score(
    pix1: &Pix,
    pix2: &Pix,
    area1: u32,
    area2: u32,
    x_offset: i32,
    y_offset: i32,
) -> Result<f32> {
    validate_binary(pix1, "pix1")?;
    validate_binary(pix2, "pix2")?;

    if area1 == 0 || area2 == 0 {
        return Ok(0.0);
    }

    let count_and = count_and_pixels(pix1, pix2, x_offset, y_offset);

    // Correlation = count_and / sqrt(area1 * area2)
    let denom = (area1 as f64 * area2 as f64).sqrt();
    if denom == 0.0 {
        return Ok(0.0);
    }

    Ok((count_and as f64 / denom) as f32)
}

/// Compute correlation with a threshold for early termination.
///
/// Returns true if the correlation exceeds the threshold. May terminate
/// early if it determines the threshold cannot be reached.
///
/// # Reference
///
/// C Leptonica: `pixCorrelationScoreThresholded()`
pub fn correlation_score_thresholded(
    pix1: &Pix,
    pix2: &Pix,
    area1: u32,
    area2: u32,
    x_offset: i32,
    y_offset: i32,
    threshold: f32,
) -> Result<bool> {
    let score = correlation_score(pix1, pix2, area1, area2, x_offset, y_offset)?;
    Ok(score >= threshold)
}

/// Simple correlation score between two binary images.
///
/// This is a convenience function that counts the ON pixels internally.
///
/// # Reference
///
/// C Leptonica: `pixCorrelationScoreSimple()`
pub fn correlation_score_simple(
    pix1: &Pix,
    pix2: &Pix,
    x_offset: i32,
    y_offset: i32,
) -> Result<f32> {
    validate_binary(pix1, "pix1")?;
    validate_binary(pix2, "pix2")?;

    let area1 = count_on_pixels(pix1);
    let area2 = count_on_pixels(pix2);

    correlation_score(pix1, pix2, area1, area2, x_offset, y_offset)
}

/// Correlation score with sub-pixel shift.
///
/// Computes the best correlation over integer shifts in a small range
/// around the given offset.
///
/// # Arguments
///
/// * `pix1` - First 1-bpp image
/// * `pix2` - Second 1-bpp image
/// * `area1` - ON pixel count for pix1
/// * `area2` - ON pixel count for pix2
/// * `x_offset` - Base horizontal offset
/// * `y_offset` - Base vertical offset
/// * `max_shift` - Maximum shift in each direction
///
/// # Returns
///
/// The best correlation score found.
///
/// # Reference
///
/// C Leptonica: `pixCorrelationScoreShifted()`
pub fn correlation_score_shifted(
    pix1: &Pix,
    pix2: &Pix,
    area1: u32,
    area2: u32,
    x_offset: i32,
    y_offset: i32,
    max_shift: i32,
) -> Result<f32> {
    validate_binary(pix1, "pix1")?;
    validate_binary(pix2, "pix2")?;

    let mut best_score = 0.0f32;

    for dy in -max_shift..=max_shift {
        for dx in -max_shift..=max_shift {
            let score = correlation_score(pix1, pix2, area1, area2, x_offset + dx, y_offset + dy)?;
            if score > best_score {
                best_score = score;
            }
        }
    }

    Ok(best_score)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn validate_binary(pix: &Pix, name: &str) -> Result<()> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(Error::UnsupportedDepth(pix.depth().bits()));
    }
    let _ = name;
    Ok(())
}

fn count_on_pixels(pix: &Pix) -> u32 {
    let mut count = 0u32;
    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if pix.get_pixel(x, y) == Some(1) {
                count += 1;
            }
        }
    }
    count
}

fn count_and_pixels(pix1: &Pix, pix2: &Pix, x_offset: i32, y_offset: i32) -> u32 {
    let w1 = pix1.width() as i32;
    let h1 = pix1.height() as i32;
    let w2 = pix2.width() as i32;
    let h2 = pix2.height() as i32;

    // Determine overlap region in pix1 coordinates
    let x_start = x_offset.max(0);
    let y_start = y_offset.max(0);
    let x_end = (x_offset + w2).min(w1);
    let y_end = (y_offset + h2).min(h1);

    let mut count = 0u32;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let p1 = pix1.get_pixel(x as u32, y as u32).unwrap_or(0);
            let x2 = (x - x_offset) as u32;
            let y2 = (y - y_offset) as u32;
            let p2 = pix2.get_pixel(x2, y2).unwrap_or(0);
            if p1 == 1 && p2 == 1 {
                count += 1;
            }
        }
    }
    count
}
