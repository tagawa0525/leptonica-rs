//! Image comparison functions
//!
//! This module provides functions for comparing two images:
//!
//! - Equality testing (`equals`, `equals_with_alpha`)
//! - Pixel-wise difference (`diff`, `subtract`, `abs_diff`)
//! - Statistical comparison (`rms_diff`, `mean_abs_diff`, `compare`)
//! - Binary image correlation (`correlation_binary`)
//!
//! These correspond to Leptonica's compare.c functions including
//! pixEqual, pixSubtract, pixAbsDifference, pixGetRMSDiff, and
//! pixCorrelationBinary.

use super::{Pix, PixelDepth};
use crate::color;
use crate::error::{Error, Result};

/// Type of comparison for difference operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareType {
    /// Subtract: pix1 - pix2, clipped to 0
    Subtract,
    /// Absolute difference: |pix1 - pix2|
    AbsDiff,
}

/// Result of comparing two images
#[derive(Debug, Clone, PartialEq)]
pub struct CompareResult {
    /// Whether the images are identical
    pub equal: bool,
    /// Root mean square difference (0.0 for identical images)
    pub rms_diff: f64,
    /// Mean absolute difference
    pub mean_abs_diff: f64,
    /// Maximum difference found
    pub max_diff: u32,
    /// Number of pixels that differ
    pub diff_count: u64,
}

impl Default for CompareResult {
    fn default() -> Self {
        Self {
            equal: true,
            rms_diff: 0.0,
            mean_abs_diff: 0.0,
            max_diff: 0,
            diff_count: 0,
        }
    }
}

impl Pix {
    /// Check if two images have identical pixel values.
    ///
    /// For 32-bit images, this ignores the alpha channel.
    /// Use `equals_with_alpha` to include alpha in comparison.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to compare against
    ///
    /// # Returns
    ///
    /// `true` if images have same dimensions and identical pixel values.
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    /// let pix2 = pix1.deep_clone();
    /// assert!(pix1.equals(&pix2));
    /// ```
    pub fn equals(&self, other: &Pix) -> bool {
        self.equals_with_alpha(other, false)
    }

    /// Check if two images are equal, optionally comparing alpha channel.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to compare against
    /// * `compare_alpha` - If true and images are 32-bit, compare alpha channel
    ///
    /// # Returns
    ///
    /// `true` if images have same dimensions and identical pixel values.
    pub fn equals_with_alpha(&self, other: &Pix, compare_alpha: bool) -> bool {
        // Check dimensions
        if self.width() != other.width()
            || self.height() != other.height()
            || self.depth() != other.depth()
        {
            return false;
        }

        let width = self.width();
        let height = self.height();

        match self.depth() {
            PixelDepth::Bit1 => {
                // For 1-bit images, compare word-by-word
                self.equals_binary(other)
            }
            PixelDepth::Bit32 => {
                // For 32-bit images, handle alpha masking
                let mask = if compare_alpha {
                    0xFFFFFFFF
                } else {
                    0xFFFFFF00 // Mask out alpha (LSB)
                };
                for y in 0..height {
                    for x in 0..width {
                        let v1 = self.get_pixel(x, y).unwrap_or(0) & mask;
                        let v2 = other.get_pixel(x, y).unwrap_or(0) & mask;
                        if v1 != v2 {
                            return false;
                        }
                    }
                }
                true
            }
            _ => {
                // For other depths, compare pixel by pixel
                for y in 0..height {
                    for x in 0..width {
                        if self.get_pixel(x, y) != other.get_pixel(x, y) {
                            return false;
                        }
                    }
                }
                true
            }
        }
    }

    /// Optimized equality check for binary images
    fn equals_binary(&self, other: &Pix) -> bool {
        let height = self.height();
        let wpl = self.wpl();

        // Calculate how many bits are actually used in the last word of each row
        let width = self.width();
        let bits_used = width % 32;
        let end_mask = if bits_used == 0 {
            0xFFFFFFFF
        } else {
            // Mask for the used bits (MSB side)
            !((1u32 << (32 - bits_used)) - 1)
        };
        let full_words = (width / 32) as usize;

        for y in 0..height {
            let line1 = self.row_data(y);
            let line2 = other.row_data(y);

            // Compare full words
            for w in 0..full_words {
                if line1[w] != line2[w] {
                    return false;
                }
            }

            // Compare partial last word if exists
            if bits_used != 0
                && (full_words as u32) < wpl
                && (line1[full_words] ^ line2[full_words]) & end_mask != 0
            {
                return false;
            }
        }
        true
    }

    /// Compute difference image.
    ///
    /// Creates a new image containing the pixel-wise difference between
    /// this image and another.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to subtract
    /// * `compare_type` - Type of difference (Subtract or AbsDiff)
    ///
    /// # Returns
    ///
    /// A new image containing the difference. For `Subtract`, negative values
    /// are clipped to 0. For `AbsDiff`, the absolute value is taken.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or incompatible depths.
    pub fn diff(&self, other: &Pix, compare_type: CompareType) -> Result<Pix> {
        // Check dimensions
        if self.width() != other.width() || self.height() != other.height() {
            return Err(Error::DimensionMismatch {
                expected: (self.width(), self.height()),
                actual: (other.width(), other.height()),
            });
        }

        if self.depth() != other.depth() {
            return Err(Error::IncompatibleDepths(
                self.depth().bits(),
                other.depth().bits(),
            ));
        }

        match self.depth() {
            PixelDepth::Bit1 => self.diff_binary(other, compare_type),
            PixelDepth::Bit8 => self.diff_gray(other, compare_type),
            PixelDepth::Bit32 => self.diff_rgb(other, compare_type),
            _ => Err(Error::UnsupportedDepth(self.depth().bits())),
        }
    }

    /// Subtract another image from this one: self - other
    ///
    /// Convenience method for `diff(other, CompareType::Subtract)`.
    pub fn subtract(&self, other: &Pix) -> Result<Pix> {
        self.diff(other, CompareType::Subtract)
    }

    /// Compute absolute difference: |self - other|
    ///
    /// Convenience method for `diff(other, CompareType::AbsDiff)`.
    pub fn abs_diff(&self, other: &Pix) -> Result<Pix> {
        self.diff(other, CompareType::AbsDiff)
    }

    /// Diff for binary images
    fn diff_binary(&self, other: &Pix, compare_type: CompareType) -> Result<Pix> {
        let width = self.width();
        let height = self.height();
        let wpl = self.wpl();

        let result = Pix::new(width, height, PixelDepth::Bit1)?;
        let mut result_mut = result.try_into_mut().unwrap();

        for y in 0..height {
            let line1 = self.row_data(y);
            let line2 = other.row_data(y);
            let line_out = result_mut.row_data_mut(y);

            for w in 0..wpl as usize {
                match compare_type {
                    CompareType::Subtract => {
                        // self - other: pixels in self but not in other
                        line_out[w] = line1[w] & !line2[w];
                    }
                    CompareType::AbsDiff => {
                        // |self - other|: XOR (symmetric difference)
                        line_out[w] = line1[w] ^ line2[w];
                    }
                }
            }
        }

        Ok(result_mut.into())
    }

    /// Diff for 8-bit grayscale images
    fn diff_gray(&self, other: &Pix, compare_type: CompareType) -> Result<Pix> {
        let width = self.width();
        let height = self.height();

        let result = Pix::new(width, height, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();

        for y in 0..height {
            for x in 0..width {
                let v1 = self.get_pixel(x, y).unwrap_or(0) as i32;
                let v2 = other.get_pixel(x, y).unwrap_or(0) as i32;

                let diff_val = match compare_type {
                    CompareType::Subtract => (v1 - v2).max(0) as u32,
                    CompareType::AbsDiff => (v1 - v2).unsigned_abs(),
                };

                result_mut.set_pixel_unchecked(x, y, diff_val.min(255));
            }
        }

        Ok(result_mut.into())
    }

    /// Diff for 32-bit RGB images
    fn diff_rgb(&self, other: &Pix, compare_type: CompareType) -> Result<Pix> {
        let width = self.width();
        let height = self.height();

        let result = Pix::new(width, height, PixelDepth::Bit32)?;
        let mut result_mut = result.try_into_mut().unwrap();

        for y in 0..height {
            for x in 0..width {
                let pixel1 = self.get_pixel(x, y).unwrap_or(0);
                let pixel2 = other.get_pixel(x, y).unwrap_or(0);

                let (r1, g1, b1) = color::extract_rgb(pixel1);
                let (r2, g2, b2) = color::extract_rgb(pixel2);

                let (r_diff, g_diff, b_diff) = match compare_type {
                    CompareType::Subtract => (
                        (r1 as i32 - r2 as i32).max(0) as u8,
                        (g1 as i32 - g2 as i32).max(0) as u8,
                        (b1 as i32 - b2 as i32).max(0) as u8,
                    ),
                    CompareType::AbsDiff => (
                        (r1 as i32 - r2 as i32).unsigned_abs() as u8,
                        (g1 as i32 - g2 as i32).unsigned_abs() as u8,
                        (b1 as i32 - b2 as i32).unsigned_abs() as u8,
                    ),
                };

                let result_pixel = color::compose_rgb(r_diff, g_diff, b_diff);
                result_mut.set_pixel_unchecked(x, y, result_pixel);
            }
        }

        Ok(result_mut.into())
    }

    /// Compute RMS (Root Mean Square) difference between two images.
    ///
    /// For grayscale images, computes sqrt(mean((pix1 - pix2)^2)).
    /// For color images, computes the RMS over all channels averaged.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to compare against
    ///
    /// # Returns
    ///
    /// The RMS difference. Returns 0.0 for identical images.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or incompatible depths.
    pub fn rms_diff(&self, other: &Pix) -> Result<f64> {
        let stats = self.compute_diff_stats(other)?;
        Ok(stats.0)
    }

    /// Compute mean absolute difference between two images.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to compare against
    ///
    /// # Returns
    ///
    /// The mean absolute difference.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or incompatible depths.
    pub fn mean_abs_diff(&self, other: &Pix) -> Result<f64> {
        let stats = self.compute_diff_stats(other)?;
        Ok(stats.1)
    }

    /// Compute full comparison statistics between two images.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to compare against
    ///
    /// # Returns
    ///
    /// A `CompareResult` containing equality, RMS diff, mean abs diff,
    /// max diff, and count of differing pixels.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or incompatible depths.
    pub fn compare(&self, other: &Pix) -> Result<CompareResult> {
        // Check dimensions
        if self.width() != other.width() || self.height() != other.height() {
            return Err(Error::DimensionMismatch {
                expected: (self.width(), self.height()),
                actual: (other.width(), other.height()),
            });
        }

        if self.depth() != other.depth() {
            return Err(Error::IncompatibleDepths(
                self.depth().bits(),
                other.depth().bits(),
            ));
        }

        let (rms, mean_abs, max_diff, diff_count) = self.compute_full_stats(other)?;

        Ok(CompareResult {
            equal: diff_count == 0,
            rms_diff: rms,
            mean_abs_diff: mean_abs,
            max_diff,
            diff_count,
        })
    }

    /// Internal helper to compute RMS and mean absolute difference
    fn compute_diff_stats(&self, other: &Pix) -> Result<(f64, f64)> {
        // Check dimensions
        if self.width() != other.width() || self.height() != other.height() {
            return Err(Error::DimensionMismatch {
                expected: (self.width(), self.height()),
                actual: (other.width(), other.height()),
            });
        }

        if self.depth() != other.depth() {
            return Err(Error::IncompatibleDepths(
                self.depth().bits(),
                other.depth().bits(),
            ));
        }

        let width = self.width();
        let height = self.height();
        let total_pixels = (width as u64) * (height as u64);

        if total_pixels == 0 {
            return Ok((0.0, 0.0));
        }

        let mut sum_sq: f64 = 0.0;
        let mut sum_abs: f64 = 0.0;

        match self.depth() {
            PixelDepth::Bit1 => {
                // For binary images, difference is 0 or 1
                for y in 0..height {
                    for x in 0..width {
                        let v1 = self.get_pixel(x, y).unwrap_or(0);
                        let v2 = other.get_pixel(x, y).unwrap_or(0);
                        let diff = if v1 != v2 { 1.0 } else { 0.0 };
                        sum_sq += diff;
                        sum_abs += diff;
                    }
                }
            }
            PixelDepth::Bit8 | PixelDepth::Bit16 => {
                for y in 0..height {
                    for x in 0..width {
                        let v1 = self.get_pixel(x, y).unwrap_or(0) as i64;
                        let v2 = other.get_pixel(x, y).unwrap_or(0) as i64;
                        let diff = (v1 - v2).abs() as f64;
                        sum_sq += diff * diff;
                        sum_abs += diff;
                    }
                }
            }
            PixelDepth::Bit32 => {
                // For RGB, average over channels
                for y in 0..height {
                    for x in 0..width {
                        let pixel1 = self.get_pixel(x, y).unwrap_or(0);
                        let pixel2 = other.get_pixel(x, y).unwrap_or(0);

                        let (r1, g1, b1) = color::extract_rgb(pixel1);
                        let (r2, g2, b2) = color::extract_rgb(pixel2);

                        let r_diff = (r1 as i32 - r2 as i32).abs() as f64;
                        let g_diff = (g1 as i32 - g2 as i32).abs() as f64;
                        let b_diff = (b1 as i32 - b2 as i32).abs() as f64;

                        // Average over channels
                        let avg_diff = (r_diff + g_diff + b_diff) / 3.0;
                        let avg_sq = (r_diff * r_diff + g_diff * g_diff + b_diff * b_diff) / 3.0;

                        sum_sq += avg_sq;
                        sum_abs += avg_diff;
                    }
                }
            }
            _ => {
                return Err(Error::UnsupportedDepth(self.depth().bits()));
            }
        }

        let rms = (sum_sq / total_pixels as f64).sqrt();
        let mean_abs = sum_abs / total_pixels as f64;

        Ok((rms, mean_abs))
    }

    /// Internal helper to compute all stats including max diff and diff count
    fn compute_full_stats(&self, other: &Pix) -> Result<(f64, f64, u32, u64)> {
        let width = self.width();
        let height = self.height();
        let total_pixels = (width as u64) * (height as u64);

        if total_pixels == 0 {
            return Ok((0.0, 0.0, 0, 0));
        }

        let mut sum_sq: f64 = 0.0;
        let mut sum_abs: f64 = 0.0;
        let mut max_diff: u32 = 0;
        let mut diff_count: u64 = 0;

        match self.depth() {
            PixelDepth::Bit1 => {
                for y in 0..height {
                    for x in 0..width {
                        let v1 = self.get_pixel(x, y).unwrap_or(0);
                        let v2 = other.get_pixel(x, y).unwrap_or(0);
                        if v1 != v2 {
                            sum_sq += 1.0;
                            sum_abs += 1.0;
                            max_diff = 1;
                            diff_count += 1;
                        }
                    }
                }
            }
            PixelDepth::Bit8 | PixelDepth::Bit16 => {
                for y in 0..height {
                    for x in 0..width {
                        let v1 = self.get_pixel(x, y).unwrap_or(0);
                        let v2 = other.get_pixel(x, y).unwrap_or(0);
                        let diff = v1.abs_diff(v2);

                        if diff > 0 {
                            let diff_f = diff as f64;
                            sum_sq += diff_f * diff_f;
                            sum_abs += diff_f;
                            max_diff = max_diff.max(diff);
                            diff_count += 1;
                        }
                    }
                }
            }
            PixelDepth::Bit32 => {
                for y in 0..height {
                    for x in 0..width {
                        let pixel1 = self.get_pixel(x, y).unwrap_or(0);
                        let pixel2 = other.get_pixel(x, y).unwrap_or(0);

                        let (r1, g1, b1) = color::extract_rgb(pixel1);
                        let (r2, g2, b2) = color::extract_rgb(pixel2);

                        let r_diff = (r1 as i32 - r2 as i32).unsigned_abs();
                        let g_diff = (g1 as i32 - g2 as i32).unsigned_abs();
                        let b_diff = (b1 as i32 - b2 as i32).unsigned_abs();

                        let channel_max = r_diff.max(g_diff).max(b_diff);

                        if channel_max > 0 {
                            let r_diff_f = r_diff as f64;
                            let g_diff_f = g_diff as f64;
                            let b_diff_f = b_diff as f64;

                            let avg_sq =
                                (r_diff_f * r_diff_f + g_diff_f * g_diff_f + b_diff_f * b_diff_f)
                                    / 3.0;
                            let avg_abs = (r_diff_f + g_diff_f + b_diff_f) / 3.0;

                            sum_sq += avg_sq;
                            sum_abs += avg_abs;
                            max_diff = max_diff.max(channel_max);
                            diff_count += 1;
                        }
                    }
                }
            }
            _ => {
                return Err(Error::UnsupportedDepth(self.depth().bits()));
            }
        }

        let rms = (sum_sq / total_pixels as f64).sqrt();
        let mean_abs = sum_abs / total_pixels as f64;

        Ok((rms, mean_abs, max_diff, diff_count))
    }
}

/// Compute binary correlation between two 1-bit images.
///
/// The correlation is a number between 0.0 and 1.0 based on foreground
/// (pixel value = 1) similarity:
///
/// ```text
/// correlation = (|pix1 AND pix2|)^2 / (|pix1| * |pix2|)
/// ```
///
/// where |x| is the count of foreground pixels.
///
/// # Arguments
///
/// * `pix1` - First binary image
/// * `pix2` - Second binary image
///
/// # Returns
///
/// Correlation value between 0.0 (no overlap) and 1.0 (identical).
/// Returns 0.0 if either image has no foreground pixels.
///
/// # Errors
///
/// Returns error if either image is not 1-bit depth.
///
/// # Example
///
/// ```
/// use leptonica_core::{Pix, PixelDepth};
/// use leptonica_core::pix::compare::correlation_binary;
///
/// let pix1 = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
/// let pix2 = pix1.deep_clone();
///
/// // Identical empty images have 0.0 correlation (no foreground)
/// let corr = correlation_binary(&pix1, &pix2).unwrap();
/// assert_eq!(corr, 0.0);
/// ```
pub fn correlation_binary(pix1: &Pix, pix2: &Pix) -> Result<f64> {
    if pix1.depth() != PixelDepth::Bit1 {
        return Err(Error::UnsupportedDepth(pix1.depth().bits()));
    }
    if pix2.depth() != PixelDepth::Bit1 {
        return Err(Error::UnsupportedDepth(pix2.depth().bits()));
    }

    let count1 = count_foreground_pixels(pix1);
    let count2 = count_foreground_pixels(pix2);

    if count1 == 0 || count2 == 0 {
        return Ok(0.0);
    }

    // Compute AND of the two images and count its pixels
    let count_and = count_and_pixels(pix1, pix2);

    let correlation = (count_and as f64 * count_and as f64) / (count1 as f64 * count2 as f64);

    Ok(correlation)
}

/// Count foreground pixels (value = 1) in a binary image.
///
/// This is a module-level helper retained for internal use by
/// [`correlation_binary`]. Public callers should use [`Pix::count_pixels`].
fn count_foreground_pixels(pix: &Pix) -> u64 {
    let width = pix.width();
    let height = pix.height();
    let wpl = pix.wpl();

    // Mask for valid bits in the last word of each row
    let bits_used = width % 32;
    let end_mask = if bits_used == 0 {
        0xFFFFFFFF
    } else {
        !((1u32 << (32 - bits_used)) - 1)
    };
    let full_words = (width / 32) as usize;

    let mut count: u64 = 0;

    for y in 0..height {
        let line = pix.row_data(y);

        // Count full words
        for word in line.iter().take(full_words) {
            count += word.count_ones() as u64;
        }

        // Count partial last word
        if bits_used != 0 && (full_words as u32) < wpl {
            count += (line[full_words] & end_mask).count_ones() as u64;
        }
    }

    count
}

/// Count pixels where both images have foreground (AND)
fn count_and_pixels(pix1: &Pix, pix2: &Pix) -> u64 {
    let width = pix1.width().min(pix2.width());
    let height = pix1.height().min(pix2.height());

    // Mask for valid bits in the last word of each row
    let bits_used = width % 32;
    let end_mask = if bits_used == 0 {
        0xFFFFFFFF
    } else {
        !((1u32 << (32 - bits_used)) - 1)
    };
    let full_words = (width / 32) as usize;

    let wpl1 = pix1.wpl();
    let wpl2 = pix2.wpl();

    let mut count: u64 = 0;

    for y in 0..height {
        let line1 = pix1.row_data(y);
        let line2 = pix2.row_data(y);

        // Count full words
        for w in 0..full_words {
            count += (line1[w] & line2[w]).count_ones() as u64;
        }

        // Count partial last word
        if bits_used != 0 && (full_words as u32) < wpl1 && (full_words as u32) < wpl2 {
            count += (line1[full_words] & line2[full_words] & end_mask).count_ones() as u64;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equals_same_image() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        assert!(pix.equals(&pix));
    }

    #[test]
    fn test_equals_deep_clone() {
        let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let pix2 = pix1.deep_clone();
        assert!(pix1.equals(&pix2));
    }

    #[test]
    fn test_equals_different_pixel() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix1.to_mut();
        pix2_mut.set_pixel(5, 5, 128).unwrap();
        let pix2: Pix = pix2_mut.into();

        assert!(!pix1.equals(&pix2));
    }

    #[test]
    fn test_equals_different_dimensions() {
        let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(100, 200, PixelDepth::Bit8).unwrap();
        assert!(!pix1.equals(&pix2));
    }

    #[test]
    fn test_equals_binary() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let pix2 = pix1.deep_clone();
        assert!(pix1.equals(&pix2));
    }

    #[test]
    fn test_equals_binary_different() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix1.to_mut();
        pix2_mut.set_pixel(32, 32, 1).unwrap();
        let pix2: Pix = pix2_mut.into();

        assert!(!pix1.equals(&pix2));
    }

    #[test]
    fn test_diff_gray_subtract() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 200).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(0, 0, 50).unwrap();
        let pix2: Pix = pix2_mut.into();

        let diff = pix1.subtract(&pix2).unwrap();
        assert_eq!(diff.get_pixel(0, 0), Some(150));
    }

    #[test]
    fn test_diff_gray_subtract_clipped() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 50).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(0, 0, 200).unwrap();
        let pix2: Pix = pix2_mut.into();

        // 50 - 200 should be clipped to 0
        let diff = pix1.subtract(&pix2).unwrap();
        assert_eq!(diff.get_pixel(0, 0), Some(0));
    }

    #[test]
    fn test_diff_gray_abs_diff() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 50).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(0, 0, 200).unwrap();
        let pix2: Pix = pix2_mut.into();

        // |50 - 200| = 150
        let diff = pix1.abs_diff(&pix2).unwrap();
        assert_eq!(diff.get_pixel(0, 0), Some(150));
    }

    #[test]
    fn test_diff_binary_xor() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 1).unwrap();
        pix1_mut.set_pixel(1, 0, 1).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(1, 0, 1).unwrap();
        pix2_mut.set_pixel(2, 0, 1).unwrap();
        let pix2: Pix = pix2_mut.into();

        // XOR should give pixels 0, 2
        let diff = pix1.abs_diff(&pix2).unwrap();
        assert_eq!(diff.get_pixel(0, 0), Some(1)); // in pix1 only
        assert_eq!(diff.get_pixel(1, 0), Some(0)); // in both
        assert_eq!(diff.get_pixel(2, 0), Some(1)); // in pix2 only
    }

    #[test]
    fn test_rms_diff_identical() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let pix2 = pix1.deep_clone();

        let rms = pix1.rms_diff(&pix2).unwrap();
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_rms_diff_known_value() {
        // Create two 1-pixel images with known difference
        let pix1 = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = Pix::new(1, 1, PixelDepth::Bit8).unwrap().to_mut();
        pix2_mut.set_pixel(0, 0, 10).unwrap();
        let pix2: Pix = pix2_mut.into();

        // RMS of single pixel with diff=10 is 10.0
        let rms = pix1.rms_diff(&pix2).unwrap();
        assert_eq!(rms, 10.0);
    }

    #[test]
    fn test_compare_result() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let pix2 = pix1.deep_clone();

        let result = pix1.compare(&pix2).unwrap();
        assert!(result.equal);
        assert_eq!(result.rms_diff, 0.0);
        assert_eq!(result.mean_abs_diff, 0.0);
        assert_eq!(result.max_diff, 0);
        assert_eq!(result.diff_count, 0);
    }

    #[test]
    fn test_compare_result_different() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix1.to_mut();
        pix2_mut.set_pixel(0, 0, 100).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.compare(&pix2).unwrap();
        assert!(!result.equal);
        assert!(result.rms_diff > 0.0);
        assert!(result.mean_abs_diff > 0.0);
        assert_eq!(result.max_diff, 100);
        assert_eq!(result.diff_count, 1);
    }

    #[test]
    fn test_correlation_binary_identical() {
        // Create image with some foreground pixels
        let pix = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.to_mut();
        for i in 0..10 {
            pix_mut.set_pixel(i, i, 1).unwrap();
        }
        let pix: Pix = pix_mut.into();
        let pix2 = pix.deep_clone();

        let corr = correlation_binary(&pix, &pix2).unwrap();
        assert!((corr - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_correlation_binary_no_foreground() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();

        // Empty images have 0.0 correlation
        let corr = correlation_binary(&pix1, &pix2).unwrap();
        assert_eq!(corr, 0.0);
    }

    #[test]
    fn test_correlation_binary_no_overlap() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 1).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(63, 63, 1).unwrap();
        let pix2: Pix = pix2_mut.into();

        // No overlap means 0.0 correlation
        let corr = correlation_binary(&pix1, &pix2).unwrap();
        assert_eq!(corr, 0.0);
    }

    #[test]
    fn test_correlation_binary_wrong_depth() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(64, 64, PixelDepth::Bit8).unwrap();

        // Should error for non-binary images
        assert!(correlation_binary(&pix1, &pix2).is_err());
    }

    #[test]
    fn test_diff_rgb() {
        use crate::color::compose_rgb;

        let pix1 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, compose_rgb(200, 100, 50)).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(0, 0, compose_rgb(100, 80, 30)).unwrap();
        let pix2: Pix = pix2_mut.into();

        let diff = pix1.abs_diff(&pix2).unwrap();
        let (r, g, b) = diff.get_rgb(0, 0).unwrap();

        assert_eq!(r, 100); // |200 - 100|
        assert_eq!(g, 20); // |100 - 80|
        assert_eq!(b, 20); // |50 - 30|
    }

    #[test]
    fn test_equals_rgb_ignore_alpha() {
        use crate::color::compose_rgba;

        let pix1 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut
            .set_pixel(0, 0, compose_rgba(100, 100, 100, 255))
            .unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut
            .set_pixel(0, 0, compose_rgba(100, 100, 100, 0))
            .unwrap();
        let pix2: Pix = pix2_mut.into();

        // Without alpha comparison, should be equal
        assert!(pix1.equals(&pix2));
        // With alpha comparison, should be different
        assert!(!pix1.equals_with_alpha(&pix2, true));
    }
}
