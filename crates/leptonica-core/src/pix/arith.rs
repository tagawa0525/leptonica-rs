//! Image arithmetic operations
//!
//! This module provides functions for pixel-wise arithmetic operations:
//!
//! - Addition (`add`, `add_constant`)
//! - Subtraction (`subtract`)
//! - Multiplication (`multiply_constant`, `multiply_gray`)
//! - Division
//! - Absolute difference (`abs_difference`)
//! - Min/Max operations
//! - In-place operations
//!
//! These correspond to Leptonica's pixarith.c functions including
//! pixAddGray, pixSubtractGray, pixAddConstantGray, pixMultConstantGray,
//! pixAbsDifference, and pixMinOrMax.

use super::{Pix, PixMut, PixelDepth};
use crate::color;
use crate::error::{Error, Result};

impl Pix {
    /// Add a constant value to all pixels.
    ///
    /// Creates a new image where each pixel value is increased by the constant.
    /// Values are clipped to the valid range for the pixel depth.
    ///
    /// # Arguments
    ///
    /// * `val` - Value to add (can be negative for subtraction)
    ///
    /// # Returns
    ///
    /// New image with added constant.
    ///
    /// # Errors
    ///
    /// Returns error if depth is not supported (1bpp not allowed).
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    /// let brightened = pix.add_constant(50).unwrap();
    /// ```
    pub fn add_constant(&self, val: i32) -> Result<Pix> {
        if self.depth() == PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(1));
        }

        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.add_constant_inplace(val);
        Ok(result_mut.into())
    }

    /// Multiply all pixels by a constant factor.
    ///
    /// Creates a new image where each pixel value is multiplied by the factor.
    /// Values are clipped to the valid range for the pixel depth.
    ///
    /// # Arguments
    ///
    /// * `factor` - Multiplication factor (must be >= 0.0)
    ///
    /// # Returns
    ///
    /// New image with multiplied values.
    ///
    /// # Errors
    ///
    /// Returns error if factor is negative or depth is not supported.
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    /// let darkened = pix.multiply_constant(0.5).unwrap();
    /// ```
    pub fn multiply_constant(&self, factor: f32) -> Result<Pix> {
        if self.depth() == PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(1));
        }

        if factor < 0.0 {
            return Err(Error::InvalidParameter("factor must be >= 0.0".to_string()));
        }

        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.multiply_constant_inplace(factor)?;
        Ok(result_mut.into())
    }

    /// Add another image to this one: self + other
    ///
    /// Creates a new image where each pixel is the sum of corresponding
    /// pixels in self and other. Values are clipped to the valid range.
    ///
    /// # Arguments
    ///
    /// * `other` - Image to add
    ///
    /// # Returns
    ///
    /// New image containing the sum.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    /// let pix2 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    /// let sum = pix1.arith_add(&pix2).unwrap();
    /// ```
    pub fn arith_add(&self, other: &Pix) -> Result<Pix> {
        self.arith_binary_op(other, ArithBinaryOp::Add)
    }

    /// Subtract another image from this one: self - other
    ///
    /// Creates a new image where each pixel is the difference of corresponding
    /// pixels. Values are clipped to 0 (no negative values).
    ///
    /// # Arguments
    ///
    /// * `other` - Image to subtract
    ///
    /// # Returns
    ///
    /// New image containing the difference.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn arith_subtract(&self, other: &Pix) -> Result<Pix> {
        self.arith_binary_op(other, ArithBinaryOp::Subtract)
    }

    /// Compute absolute difference: |self - other|
    ///
    /// Creates a new image where each pixel is the absolute value of the
    /// difference between corresponding pixels.
    ///
    /// # Arguments
    ///
    /// * `other` - Image to compare against
    ///
    /// # Returns
    ///
    /// New image containing absolute differences.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn arith_abs_diff(&self, other: &Pix) -> Result<Pix> {
        self.arith_binary_op(other, ArithBinaryOp::AbsDiff)
    }

    /// Compute pixel-wise minimum: min(self, other)
    ///
    /// Creates a new image where each pixel is the minimum of corresponding
    /// pixels in self and other.
    ///
    /// # Arguments
    ///
    /// * `other` - Image to compare against
    ///
    /// # Returns
    ///
    /// New image containing minimum values.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn arith_min(&self, other: &Pix) -> Result<Pix> {
        self.arith_binary_op(other, ArithBinaryOp::Min)
    }

    /// Compute pixel-wise maximum: max(self, other)
    ///
    /// Creates a new image where each pixel is the maximum of corresponding
    /// pixels in self and other.
    ///
    /// # Arguments
    ///
    /// * `other` - Image to compare against
    ///
    /// # Returns
    ///
    /// New image containing maximum values.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn arith_max(&self, other: &Pix) -> Result<Pix> {
        self.arith_binary_op(other, ArithBinaryOp::Max)
    }

    /// Multiply by a grayscale image for illumination correction.
    ///
    /// This is useful for correcting scanned images under non-uniform
    /// illumination. Each pixel is multiplied by the corresponding gray
    /// value times a normalization factor.
    ///
    /// # Arguments
    ///
    /// * `gray` - 8-bit grayscale image (values inversely related to illumination)
    /// * `norm` - Normalization factor. If None, uses 1.0 / max_gray to avoid overflow.
    ///
    /// # Returns
    ///
    /// New image with corrected illumination.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - self is not 8 or 32 bpp
    /// - gray is not 8 bpp
    pub fn multiply_gray(&self, gray: &Pix, norm: Option<f32>) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit8 && self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if gray.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(gray.depth().bits()));
        }

        let width = self.width().min(gray.width());
        let height = self.height().min(gray.height());

        // Compute norm if not provided (find max gray value)
        let norm = norm.unwrap_or_else(|| {
            let mut max_gray: u32 = 0;
            for y in 0..height {
                for x in 0..width {
                    let g = gray.get_pixel(x, y).unwrap_or(0) & 0xFF;
                    max_gray = max_gray.max(g);
                }
            }
            if max_gray > 0 {
                1.0 / max_gray as f32
            } else {
                1.0
            }
        });

        let result = Pix::new(self.width(), self.height(), self.depth())?;
        let mut result_mut = result.try_into_mut().unwrap();

        match self.depth() {
            PixelDepth::Bit8 => {
                for y in 0..height {
                    for x in 0..width {
                        let val_s = (self.get_pixel(x, y).unwrap_or(0) & 0xFF) as f32;
                        let val_g = (gray.get_pixel(x, y).unwrap_or(0) & 0xFF) as f32;
                        let result_val = (val_s * val_g * norm + 0.5) as u32;
                        let clipped = result_val.min(255);
                        unsafe { result_mut.set_pixel_unchecked(x, y, clipped) };
                    }
                }
                // Fill remaining pixels from self
                for y in 0..self.height() {
                    for x in 0..self.width() {
                        if x >= width || y >= height {
                            let val = self.get_pixel(x, y).unwrap_or(0);
                            unsafe { result_mut.set_pixel_unchecked(x, y, val) };
                        }
                    }
                }
            }
            PixelDepth::Bit32 => {
                for y in 0..height {
                    for x in 0..width {
                        let pixel_s = self.get_pixel(x, y).unwrap_or(0);
                        let val_g = (gray.get_pixel(x, y).unwrap_or(0) & 0xFF) as f32;

                        let (r, g, b) = color::extract_rgb(pixel_s);
                        let r_new = ((r as f32 * val_g * norm + 0.5) as u32).min(255) as u8;
                        let g_new = ((g as f32 * val_g * norm + 0.5) as u32).min(255) as u8;
                        let b_new = ((b as f32 * val_g * norm + 0.5) as u32).min(255) as u8;

                        let result_pixel = color::compose_rgb(r_new, g_new, b_new);
                        unsafe { result_mut.set_pixel_unchecked(x, y, result_pixel) };
                    }
                }
                // Fill remaining pixels from self
                for y in 0..self.height() {
                    for x in 0..self.width() {
                        if x >= width || y >= height {
                            let val = self.get_pixel(x, y).unwrap_or(0);
                            unsafe { result_mut.set_pixel_unchecked(x, y, val) };
                        }
                    }
                }
            }
            _ => unreachable!(),
        }

        Ok(result_mut.into())
    }

    /// Internal helper for binary arithmetic operations
    fn arith_binary_op(&self, other: &Pix, op: ArithBinaryOp) -> Result<Pix> {
        if self.depth() != other.depth() {
            return Err(Error::IncompatibleDepths(
                self.depth().bits(),
                other.depth().bits(),
            ));
        }

        if self.depth() == PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(1));
        }

        let width = self.width().min(other.width());
        let height = self.height().min(other.height());

        let result = Pix::new(self.width(), self.height(), self.depth())?;
        let mut result_mut = result.try_into_mut().unwrap();

        // Copy self to result first
        for y in 0..self.height() {
            for x in 0..self.width() {
                let val = self.get_pixel(x, y).unwrap_or(0);
                unsafe { result_mut.set_pixel_unchecked(x, y, val) };
            }
        }

        // Apply operation in overlap region
        match self.depth() {
            PixelDepth::Bit8 => {
                for y in 0..height {
                    for x in 0..width {
                        let v1 = (self.get_pixel(x, y).unwrap_or(0) & 0xFF) as i32;
                        let v2 = (other.get_pixel(x, y).unwrap_or(0) & 0xFF) as i32;

                        let result_val = match op {
                            ArithBinaryOp::Add => (v1 + v2).clamp(0, 255) as u32,
                            ArithBinaryOp::Subtract => (v1 - v2).max(0) as u32,
                            ArithBinaryOp::AbsDiff => (v1 - v2).unsigned_abs(),
                            ArithBinaryOp::Min => v1.min(v2) as u32,
                            ArithBinaryOp::Max => v1.max(v2) as u32,
                        };

                        unsafe { result_mut.set_pixel_unchecked(x, y, result_val) };
                    }
                }
            }
            PixelDepth::Bit16 => {
                for y in 0..height {
                    for x in 0..width {
                        let v1 = (self.get_pixel(x, y).unwrap_or(0) & 0xFFFF) as i32;
                        let v2 = (other.get_pixel(x, y).unwrap_or(0) & 0xFFFF) as i32;

                        let result_val = match op {
                            ArithBinaryOp::Add => (v1 + v2).clamp(0, 65535) as u32,
                            ArithBinaryOp::Subtract => (v1 - v2).max(0) as u32,
                            ArithBinaryOp::AbsDiff => (v1 - v2).unsigned_abs(),
                            ArithBinaryOp::Min => v1.min(v2) as u32,
                            ArithBinaryOp::Max => v1.max(v2) as u32,
                        };

                        unsafe { result_mut.set_pixel_unchecked(x, y, result_val) };
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

                        let (r_out, g_out, b_out) = match op {
                            ArithBinaryOp::Add => (
                                ((r1 as i32 + r2 as i32).min(255)) as u8,
                                ((g1 as i32 + g2 as i32).min(255)) as u8,
                                ((b1 as i32 + b2 as i32).min(255)) as u8,
                            ),
                            ArithBinaryOp::Subtract => (
                                (r1 as i32 - r2 as i32).max(0) as u8,
                                (g1 as i32 - g2 as i32).max(0) as u8,
                                (b1 as i32 - b2 as i32).max(0) as u8,
                            ),
                            ArithBinaryOp::AbsDiff => (
                                (r1 as i32 - r2 as i32).unsigned_abs() as u8,
                                (g1 as i32 - g2 as i32).unsigned_abs() as u8,
                                (b1 as i32 - b2 as i32).unsigned_abs() as u8,
                            ),
                            ArithBinaryOp::Min => (r1.min(r2), g1.min(g2), b1.min(b2)),
                            ArithBinaryOp::Max => (r1.max(r2), g1.max(g2), b1.max(b2)),
                        };

                        let result_pixel = color::compose_rgb(r_out, g_out, b_out);
                        unsafe { result_mut.set_pixel_unchecked(x, y, result_pixel) };
                    }
                }
            }
            _ => {
                return Err(Error::UnsupportedDepth(self.depth().bits()));
            }
        }

        Ok(result_mut.into())
    }
}

impl PixMut {
    /// Add a constant value to all pixels in place.
    ///
    /// Modifies this image so that each pixel value is increased by the constant.
    /// Values are clipped to the valid range for the pixel depth.
    ///
    /// # Arguments
    ///
    /// * `val` - Value to add (can be negative for subtraction)
    pub fn add_constant_inplace(&mut self, val: i32) {
        let width = self.width();
        let height = self.height();
        let depth = self.depth();

        match depth {
            PixelDepth::Bit8 => {
                for y in 0..height {
                    for x in 0..width {
                        let pval = (self.get_pixel(x, y).unwrap_or(0) & 0xFF) as i32;
                        let new_val = if val < 0 {
                            (pval + val).max(0) as u32
                        } else {
                            (pval + val).min(255) as u32
                        };
                        unsafe { self.set_pixel_unchecked(x, y, new_val) };
                    }
                }
            }
            PixelDepth::Bit16 => {
                for y in 0..height {
                    for x in 0..width {
                        let pval = (self.get_pixel(x, y).unwrap_or(0) & 0xFFFF) as i32;
                        let new_val = if val < 0 {
                            (pval + val).max(0) as u32
                        } else {
                            (pval + val).min(65535) as u32
                        };
                        unsafe { self.set_pixel_unchecked(x, y, new_val) };
                    }
                }
            }
            PixelDepth::Bit32 => {
                for y in 0..height {
                    for x in 0..width {
                        let pixel = self.get_pixel(x, y).unwrap_or(0);
                        let (r, g, b) = color::extract_rgb(pixel);

                        let r_new = if val < 0 {
                            (r as i32 + val).max(0) as u8
                        } else {
                            (r as i32 + val).min(255) as u8
                        };
                        let g_new = if val < 0 {
                            (g as i32 + val).max(0) as u8
                        } else {
                            (g as i32 + val).min(255) as u8
                        };
                        let b_new = if val < 0 {
                            (b as i32 + val).max(0) as u8
                        } else {
                            (b as i32 + val).min(255) as u8
                        };

                        let new_pixel = color::compose_rgb(r_new, g_new, b_new);
                        unsafe { self.set_pixel_unchecked(x, y, new_pixel) };
                    }
                }
            }
            _ => {} // Other depths not supported, no-op
        }
    }

    /// Multiply all pixels by a constant factor in place.
    ///
    /// Modifies this image so that each pixel value is multiplied by the factor.
    /// Values are clipped to the valid range for the pixel depth.
    ///
    /// # Arguments
    ///
    /// * `factor` - Multiplication factor (must be >= 0.0)
    ///
    /// # Errors
    ///
    /// Returns error if factor is negative.
    pub fn multiply_constant_inplace(&mut self, factor: f32) -> Result<()> {
        if factor < 0.0 {
            return Err(Error::InvalidParameter("factor must be >= 0.0".to_string()));
        }

        let width = self.width();
        let height = self.height();
        let depth = self.depth();

        match depth {
            PixelDepth::Bit8 => {
                for y in 0..height {
                    for x in 0..width {
                        let pval = (self.get_pixel(x, y).unwrap_or(0) & 0xFF) as f32;
                        let new_val = ((factor * pval) as u32).min(255);
                        unsafe { self.set_pixel_unchecked(x, y, new_val) };
                    }
                }
            }
            PixelDepth::Bit16 => {
                for y in 0..height {
                    for x in 0..width {
                        let pval = (self.get_pixel(x, y).unwrap_or(0) & 0xFFFF) as f32;
                        let new_val = ((factor * pval) as u32).min(65535);
                        unsafe { self.set_pixel_unchecked(x, y, new_val) };
                    }
                }
            }
            PixelDepth::Bit32 => {
                for y in 0..height {
                    for x in 0..width {
                        let pixel = self.get_pixel(x, y).unwrap_or(0);
                        let (r, g, b) = color::extract_rgb(pixel);

                        let r_new = ((factor * r as f32) as u32).min(255) as u8;
                        let g_new = ((factor * g as f32) as u32).min(255) as u8;
                        let b_new = ((factor * b as f32) as u32).min(255) as u8;

                        let new_pixel = color::compose_rgb(r_new, g_new, b_new);
                        unsafe { self.set_pixel_unchecked(x, y, new_pixel) };
                    }
                }
            }
            _ => {} // Other depths not supported, no-op
        }

        Ok(())
    }

    /// Add another image in place: self += other
    ///
    /// Modifies this image so that each pixel is increased by the corresponding
    /// pixel from other. Values are clipped to the valid range.
    ///
    /// # Arguments
    ///
    /// * `other` - Image to add
    ///
    /// # Errors
    ///
    /// Returns error if images have different depths.
    pub fn arith_add_inplace(&mut self, other: &Pix) -> Result<()> {
        self.arith_binary_op_inplace(other, ArithBinaryOp::Add)
    }

    /// Subtract another image in place: self -= other
    ///
    /// Modifies this image so that each pixel is decreased by the corresponding
    /// pixel from other. Values are clipped to 0.
    ///
    /// # Arguments
    ///
    /// * `other` - Image to subtract
    ///
    /// # Errors
    ///
    /// Returns error if images have different depths.
    pub fn arith_subtract_inplace(&mut self, other: &Pix) -> Result<()> {
        self.arith_binary_op_inplace(other, ArithBinaryOp::Subtract)
    }

    /// Internal helper for in-place binary arithmetic operations
    fn arith_binary_op_inplace(&mut self, other: &Pix, op: ArithBinaryOp) -> Result<()> {
        if self.depth() != other.depth() {
            return Err(Error::IncompatibleDepths(
                self.depth().bits(),
                other.depth().bits(),
            ));
        }

        let width = self.width().min(other.width());
        let height = self.height().min(other.height());
        let depth = self.depth();

        match depth {
            PixelDepth::Bit8 => {
                for y in 0..height {
                    for x in 0..width {
                        let v1 = (self.get_pixel(x, y).unwrap_or(0) & 0xFF) as i32;
                        let v2 = (other.get_pixel(x, y).unwrap_or(0) & 0xFF) as i32;

                        let result_val = match op {
                            ArithBinaryOp::Add => (v1 + v2).clamp(0, 255) as u32,
                            ArithBinaryOp::Subtract => (v1 - v2).max(0) as u32,
                            ArithBinaryOp::AbsDiff => (v1 - v2).unsigned_abs(),
                            ArithBinaryOp::Min => v1.min(v2) as u32,
                            ArithBinaryOp::Max => v1.max(v2) as u32,
                        };

                        unsafe { self.set_pixel_unchecked(x, y, result_val) };
                    }
                }
            }
            PixelDepth::Bit16 => {
                for y in 0..height {
                    for x in 0..width {
                        let v1 = (self.get_pixel(x, y).unwrap_or(0) & 0xFFFF) as i32;
                        let v2 = (other.get_pixel(x, y).unwrap_or(0) & 0xFFFF) as i32;

                        let result_val = match op {
                            ArithBinaryOp::Add => (v1 + v2).clamp(0, 65535) as u32,
                            ArithBinaryOp::Subtract => (v1 - v2).max(0) as u32,
                            ArithBinaryOp::AbsDiff => (v1 - v2).unsigned_abs(),
                            ArithBinaryOp::Min => v1.min(v2) as u32,
                            ArithBinaryOp::Max => v1.max(v2) as u32,
                        };

                        unsafe { self.set_pixel_unchecked(x, y, result_val) };
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

                        let (r_out, g_out, b_out) = match op {
                            ArithBinaryOp::Add => (
                                ((r1 as i32 + r2 as i32).min(255)) as u8,
                                ((g1 as i32 + g2 as i32).min(255)) as u8,
                                ((b1 as i32 + b2 as i32).min(255)) as u8,
                            ),
                            ArithBinaryOp::Subtract => (
                                (r1 as i32 - r2 as i32).max(0) as u8,
                                (g1 as i32 - g2 as i32).max(0) as u8,
                                (b1 as i32 - b2 as i32).max(0) as u8,
                            ),
                            ArithBinaryOp::AbsDiff => (
                                (r1 as i32 - r2 as i32).unsigned_abs() as u8,
                                (g1 as i32 - g2 as i32).unsigned_abs() as u8,
                                (b1 as i32 - b2 as i32).unsigned_abs() as u8,
                            ),
                            ArithBinaryOp::Min => (r1.min(r2), g1.min(g2), b1.min(b2)),
                            ArithBinaryOp::Max => (r1.max(r2), g1.max(g2), b1.max(b2)),
                        };

                        let result_pixel = color::compose_rgb(r_out, g_out, b_out);
                        unsafe { self.set_pixel_unchecked(x, y, result_pixel) };
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Binary arithmetic operations (internal use)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArithBinaryOp {
    Add,
    Subtract,
    AbsDiff,
    Min,
    Max,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_constant_gray() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(5, 5, 100).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.add_constant(50).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(150));
        assert_eq!(result.get_pixel(0, 0), Some(50)); // 0 + 50
    }

    #[test]
    fn test_add_constant_negative() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(5, 5, 100).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.add_constant(-150).unwrap();
        // 100 - 150 should clip to 0
        assert_eq!(result.get_pixel(5, 5), Some(0));
    }

    #[test]
    fn test_add_constant_clipping() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(5, 5, 200).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.add_constant(100).unwrap();
        // 200 + 100 = 300, should clip to 255
        assert_eq!(result.get_pixel(5, 5), Some(255));
    }

    #[test]
    fn test_multiply_constant_gray() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(5, 5, 100).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.multiply_constant(0.5).unwrap();
        // 100 * 0.5 = 50
        assert_eq!(result.get_pixel(5, 5), Some(50));
    }

    #[test]
    fn test_multiply_constant_clipping() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(5, 5, 200).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.multiply_constant(2.0).unwrap();
        // 200 * 2 = 400, should clip to 255
        assert_eq!(result.get_pixel(5, 5), Some(255));
    }

    #[test]
    fn test_multiply_constant_negative_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = pix.multiply_constant(-1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_arith_add_gray() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(5, 5, 100).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(5, 5, 50).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.arith_add(&pix2).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(150));
    }

    #[test]
    fn test_arith_add_clipping() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(5, 5, 200).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(5, 5, 100).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.arith_add(&pix2).unwrap();
        // 200 + 100 = 300, clips to 255
        assert_eq!(result.get_pixel(5, 5), Some(255));
    }

    #[test]
    fn test_arith_subtract_gray() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(5, 5, 200).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(5, 5, 50).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.arith_subtract(&pix2).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(150));
    }

    #[test]
    fn test_arith_subtract_clipping() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(5, 5, 50).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(5, 5, 200).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.arith_subtract(&pix2).unwrap();
        // 50 - 200 = -150, clips to 0
        assert_eq!(result.get_pixel(5, 5), Some(0));
    }

    #[test]
    fn test_arith_abs_diff_gray() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(5, 5, 50).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(5, 5, 200).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.arith_abs_diff(&pix2).unwrap();
        // |50 - 200| = 150
        assert_eq!(result.get_pixel(5, 5), Some(150));
    }

    #[test]
    fn test_arith_min_gray() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(5, 5, 200).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(5, 5, 50).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.arith_min(&pix2).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(50));
    }

    #[test]
    fn test_arith_max_gray() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(5, 5, 200).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(5, 5, 50).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.arith_max(&pix2).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(200));
    }

    #[test]
    fn test_add_constant_rgb() {
        use crate::color::compose_rgb;

        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(0, 0, compose_rgb(100, 50, 200)).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.add_constant(30).unwrap();
        let (r, g, b) = result.get_rgb(0, 0).unwrap();

        assert_eq!(r, 130); // 100 + 30
        assert_eq!(g, 80); // 50 + 30
        assert_eq!(b, 230); // 200 + 30
    }

    #[test]
    fn test_add_constant_rgb_clipping() {
        use crate::color::compose_rgb;

        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(0, 0, compose_rgb(200, 50, 100)).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.add_constant(100).unwrap();
        let (r, g, b) = result.get_rgb(0, 0).unwrap();

        assert_eq!(r, 255); // 200 + 100 = 300, clips to 255
        assert_eq!(g, 150); // 50 + 100
        assert_eq!(b, 200); // 100 + 100
    }

    #[test]
    fn test_arith_add_rgb() {
        use crate::color::compose_rgb;

        let pix1 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, compose_rgb(100, 50, 200)).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(0, 0, compose_rgb(30, 20, 10)).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.arith_add(&pix2).unwrap();
        let (r, g, b) = result.get_rgb(0, 0).unwrap();

        assert_eq!(r, 130);
        assert_eq!(g, 70);
        assert_eq!(b, 210);
    }

    #[test]
    fn test_arith_abs_diff_rgb() {
        use crate::color::compose_rgb;

        let pix1 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, compose_rgb(100, 200, 50)).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(0, 0, compose_rgb(50, 150, 100)).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.arith_abs_diff(&pix2).unwrap();
        let (r, g, b) = result.get_rgb(0, 0).unwrap();

        assert_eq!(r, 50); // |100 - 50|
        assert_eq!(g, 50); // |200 - 150|
        assert_eq!(b, 50); // |50 - 100|
    }

    #[test]
    fn test_inplace_add_constant() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(5, 5, 100).unwrap();

        pix_mut.add_constant_inplace(50);

        assert_eq!(pix_mut.get_pixel(5, 5), Some(150));
    }

    #[test]
    fn test_inplace_multiply_constant() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(5, 5, 100).unwrap();

        pix_mut.multiply_constant_inplace(2.0).unwrap();

        assert_eq!(pix_mut.get_pixel(5, 5), Some(200));
    }

    #[test]
    fn test_inplace_add() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(5, 5, 100).unwrap();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(5, 5, 50).unwrap();
        let pix2: Pix = pix2_mut.into();

        pix1_mut.arith_add_inplace(&pix2).unwrap();

        assert_eq!(pix1_mut.get_pixel(5, 5), Some(150));
    }

    #[test]
    fn test_inplace_subtract() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(5, 5, 200).unwrap();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(5, 5, 50).unwrap();
        let pix2: Pix = pix2_mut.into();

        pix1_mut.arith_subtract_inplace(&pix2).unwrap();

        assert_eq!(pix1_mut.get_pixel(5, 5), Some(150));
    }

    #[test]
    fn test_depth_mismatch_error() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(10, 10, PixelDepth::Bit32).unwrap();

        let result = pix1.arith_add(&pix2);
        assert!(result.is_err());
    }

    #[test]
    fn test_1bpp_unsupported() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        assert!(pix.add_constant(50).is_err());
        assert!(pix.multiply_constant(1.5).is_err());
    }

    #[test]
    fn test_multiply_gray_basic() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(5, 5, 100).unwrap();
        let pix: Pix = pix_mut.into();

        let gray = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut gray_mut = gray.to_mut();
        gray_mut.set_pixel(5, 5, 255).unwrap();
        let gray: Pix = gray_mut.into();

        let result = pix.multiply_gray(&gray, Some(1.0 / 255.0)).unwrap();
        // 100 * 255 * (1/255) = 100
        assert_eq!(result.get_pixel(5, 5), Some(100));
    }
}
