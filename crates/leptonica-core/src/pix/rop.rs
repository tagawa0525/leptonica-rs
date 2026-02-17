//! Image raster operations (logical operations)
//!
//! This module provides functions for pixel-wise logical operations:
//!
//! - AND, OR, XOR, NOT operations
//! - NAND, NOR, XNOR operations
//! - In-place operations
//! - Region-based operations
//!
//! These correspond to Leptonica's rop.c functions including
//! pixAnd, pixOr, pixXor, and pixInvert.

use super::{Pix, PixMut, PixelDepth};
use crate::color;
use crate::error::{Error, Result};

/// Color to fill when shifting or translating image regions.
///
/// # See also
///
/// C Leptonica: `L_BRING_IN_WHITE`, `L_BRING_IN_BLACK` in `pix.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InColor {
    /// Fill exposed areas with white (max pixel value)
    White,
    /// Fill exposed areas with black (zero pixel value)
    Black,
}

/// Raster operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RopOp {
    /// Clear: d = 0
    Clear,
    /// Set: d = 1 (all bits set)
    Set,
    /// Copy source: d = s
    Src,
    /// Invert destination: d = ~d
    NotDst,
    /// Invert source: d = ~s
    NotSrc,
    /// AND: d = s & d
    And,
    /// OR: d = s | d
    Or,
    /// XOR: d = s ^ d
    Xor,
    /// NAND: d = ~(s & d)
    Nand,
    /// NOR: d = ~(s | d)
    Nor,
    /// XNOR: d = ~(s ^ d)
    Xnor,
    /// AND with inverted source: d = ~s & d
    AndNotSrc,
    /// AND with inverted dest: d = s & ~d
    AndNotDst,
    /// OR with inverted source: d = ~s | d
    OrNotSrc,
    /// OR with inverted dest: d = s | ~d
    OrNotDst,
}

impl RopOp {
    /// Check if this operation requires a source image
    #[inline]
    pub fn requires_source(self) -> bool {
        !matches!(self, RopOp::Clear | RopOp::Set | RopOp::NotDst)
    }
}

impl Pix {
    /// Perform AND operation with another image.
    ///
    /// Returns a new image where each pixel is the bitwise AND of
    /// the corresponding pixels in self and other.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to AND with
    ///
    /// # Returns
    ///
    /// New image containing the AND result.
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
    /// let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
    /// let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
    /// let result = pix1.and(&pix2).unwrap();
    /// ```
    pub fn and(&self, other: &Pix) -> Result<Pix> {
        self.rop(other, RopOp::And)
    }

    /// Perform OR operation with another image.
    ///
    /// Returns a new image where each pixel is the bitwise OR of
    /// the corresponding pixels in self and other.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to OR with
    ///
    /// # Returns
    ///
    /// New image containing the OR result.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn or(&self, other: &Pix) -> Result<Pix> {
        self.rop(other, RopOp::Or)
    }

    /// Perform XOR operation with another image.
    ///
    /// Returns a new image where each pixel is the bitwise XOR of
    /// the corresponding pixels in self and other.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to XOR with
    ///
    /// # Returns
    ///
    /// New image containing the XOR result.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn xor(&self, other: &Pix) -> Result<Pix> {
        self.rop(other, RopOp::Xor)
    }

    /// Invert all pixels in the image.
    ///
    /// For binary images, foreground becomes background and vice versa.
    /// For grayscale, each pixel value v becomes (max_value - v).
    /// For RGB, each channel is inverted independently.
    ///
    /// # Returns
    ///
    /// New image with inverted pixels.
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
    /// let inverted = pix.invert();
    /// ```
    pub fn invert(&self) -> Pix {
        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.invert_inplace();
        result_mut.into()
    }

    /// Apply a general raster operation with another image.
    ///
    /// # Arguments
    ///
    /// * `other` - The source image (can be same as self for some ops)
    /// * `op` - The raster operation to perform
    ///
    /// # Returns
    ///
    /// New image containing the result.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Images have different dimensions (for binary operations)
    /// - Images have different depths (for binary operations)
    pub fn rop(&self, other: &Pix, op: RopOp) -> Result<Pix> {
        // For unary operations, we don't need dimension checks
        if !op.requires_source() {
            let result = self.deep_clone();
            let mut result_mut = result.try_into_mut().unwrap();
            result_mut.rop_unary_inplace(op);
            return Ok(result_mut.into());
        }

        // Check dimensions for binary operations
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
            PixelDepth::Bit1 => self.rop_binary(other, op),
            PixelDepth::Bit8 => self.rop_gray(other, op),
            PixelDepth::Bit32 => self.rop_rgb(other, op),
            _ => self.rop_generic(other, op),
        }
    }

    /// Binary image raster operation (1-bit, word-optimized)
    fn rop_binary(&self, other: &Pix, op: RopOp) -> Result<Pix> {
        let width = self.width();
        let height = self.height();
        let wpl = self.wpl();

        let result = Pix::new(width, height, PixelDepth::Bit1)?;
        let mut result_mut = result.try_into_mut().unwrap();

        for y in 0..height {
            let line_d = self.row_data(y);
            let line_s = other.row_data(y);
            let line_out = result_mut.row_data_mut(y);

            for w in 0..wpl as usize {
                let d = line_d[w];
                let s = line_s[w];
                line_out[w] = apply_rop_word(d, s, op);
            }
        }

        Ok(result_mut.into())
    }

    /// Grayscale image raster operation (8-bit)
    fn rop_gray(&self, other: &Pix, op: RopOp) -> Result<Pix> {
        let width = self.width();
        let height = self.height();

        let result = Pix::new(width, height, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();

        for y in 0..height {
            for x in 0..width {
                let d = self.get_pixel(x, y).unwrap_or(0);
                let s = other.get_pixel(x, y).unwrap_or(0);
                let val = apply_rop_value(d, s, op, 255);
                result_mut.set_pixel_unchecked(x, y, val);
            }
        }

        Ok(result_mut.into())
    }

    /// RGB image raster operation (32-bit)
    fn rop_rgb(&self, other: &Pix, op: RopOp) -> Result<Pix> {
        let width = self.width();
        let height = self.height();

        let result = Pix::new(width, height, PixelDepth::Bit32)?;
        let mut result_mut = result.try_into_mut().unwrap();

        for y in 0..height {
            for x in 0..width {
                let d_pixel = self.get_pixel(x, y).unwrap_or(0);
                let s_pixel = other.get_pixel(x, y).unwrap_or(0);

                let (dr, dg, db) = color::extract_rgb(d_pixel);
                let (sr, sg, sb) = color::extract_rgb(s_pixel);

                let rr = apply_rop_value(dr as u32, sr as u32, op, 255) as u8;
                let rg = apply_rop_value(dg as u32, sg as u32, op, 255) as u8;
                let rb = apply_rop_value(db as u32, sb as u32, op, 255) as u8;

                let result_pixel = color::compose_rgb(rr, rg, rb);
                result_mut.set_pixel_unchecked(x, y, result_pixel);
            }
        }

        Ok(result_mut.into())
    }

    /// Generic raster operation for other depths (2, 4, 16-bit)
    fn rop_generic(&self, other: &Pix, op: RopOp) -> Result<Pix> {
        let width = self.width();
        let height = self.height();
        let max_val = self.depth().max_value();

        let result = Pix::new(width, height, self.depth())?;
        let mut result_mut = result.try_into_mut().unwrap();

        for y in 0..height {
            for x in 0..width {
                let d = self.get_pixel(x, y).unwrap_or(0);
                let s = other.get_pixel(x, y).unwrap_or(0);
                let val = apply_rop_value(d, s, op, max_val);
                result_mut.set_pixel_unchecked(x, y, val);
            }
        }

        Ok(result_mut.into())
    }

    /// Translate (shift) an image by the given horizontal and vertical amounts.
    ///
    /// Creates a new image of the same size, shifted by (hshift, vshift).
    /// Exposed areas are filled with the specified color.
    ///
    /// # Arguments
    ///
    /// * `hshift` - Horizontal shift (positive = right, negative = left)
    /// * `vshift` - Vertical shift (positive = down, negative = up)
    /// * `incolor` - Color to fill exposed areas
    ///
    /// # See also
    ///
    /// C Leptonica: `pixTranslate()` in `rop.c`
    pub fn translate(&self, _hshift: i32, _vshift: i32, _incolor: InColor) -> Pix {
        todo!()
    }
}

impl PixMut {
    /// In-place AND operation with another image.
    ///
    /// Modifies this image so that each pixel becomes the AND of
    /// the current value and the corresponding pixel in other.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to AND with
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn and_inplace(&mut self, other: &Pix) -> Result<()> {
        self.rop_inplace(other, RopOp::And)
    }

    /// In-place OR operation with another image.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to OR with
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn or_inplace(&mut self, other: &Pix) -> Result<()> {
        self.rop_inplace(other, RopOp::Or)
    }

    /// In-place XOR operation with another image.
    ///
    /// # Arguments
    ///
    /// * `other` - The image to XOR with
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn xor_inplace(&mut self, other: &Pix) -> Result<()> {
        self.rop_inplace(other, RopOp::Xor)
    }

    /// Invert all pixels in place.
    ///
    /// For binary images, foreground becomes background and vice versa.
    /// For grayscale, each pixel value v becomes (max_value - v).
    /// For RGB, each channel is inverted independently.
    pub fn invert_inplace(&mut self) {
        self.rop_unary_inplace(RopOp::NotDst);
    }

    /// Apply a unary raster operation in place (Clear, Set, NotDst)
    fn rop_unary_inplace(&mut self, op: RopOp) {
        match op {
            RopOp::Clear => {
                self.clear();
            }
            RopOp::Set => {
                self.set_all();
            }
            RopOp::NotDst => {
                if self.depth() == PixelDepth::Bit32 {
                    // For 32bpp, invert only the RGB channels (bytes 1-3),
                    // preserving the alpha channel (byte 0, LSB).
                    for word in self.data_mut().iter_mut() {
                        let alpha = *word & 0xFF;
                        *word = (!*word & 0xFFFFFF00) | alpha;
                    }
                } else {
                    // For all other depths, invert the raw data
                    for word in self.data_mut().iter_mut() {
                        *word = !*word;
                    }
                }
            }
            _ => {}
        }
    }

    /// Apply a general raster operation in place.
    ///
    /// # Arguments
    ///
    /// * `other` - The source image
    /// * `op` - The raster operation to perform
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn rop_inplace(&mut self, other: &Pix, op: RopOp) -> Result<()> {
        // For unary operations, we don't need dimension checks
        if !op.requires_source() {
            self.rop_unary_inplace(op);
            return Ok(());
        }

        // Check dimensions for binary operations
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
            PixelDepth::Bit1 => self.rop_binary_inplace(other, op),
            PixelDepth::Bit8 => self.rop_gray_inplace(other, op),
            PixelDepth::Bit32 => self.rop_rgb_inplace(other, op),
            _ => self.rop_generic_inplace(other, op),
        }

        Ok(())
    }

    /// Binary image raster operation in place (1-bit, word-optimized)
    fn rop_binary_inplace(&mut self, other: &Pix, op: RopOp) {
        let height = self.height();
        let wpl = self.wpl();

        for y in 0..height {
            let line_s = other.row_data(y);
            let line_d = self.row_data_mut(y);

            for w in 0..wpl as usize {
                let d = line_d[w];
                let s = line_s[w];
                line_d[w] = apply_rop_word(d, s, op);
            }
        }
    }

    /// Grayscale image raster operation in place (8-bit)
    fn rop_gray_inplace(&mut self, other: &Pix, op: RopOp) {
        let width = self.width();
        let height = self.height();

        for y in 0..height {
            for x in 0..width {
                let d = self.get_pixel(x, y).unwrap_or(0);
                let s = other.get_pixel(x, y).unwrap_or(0);
                let val = apply_rop_value(d, s, op, 255);
                self.set_pixel_unchecked(x, y, val);
            }
        }
    }

    /// RGB image raster operation in place (32-bit)
    fn rop_rgb_inplace(&mut self, other: &Pix, op: RopOp) {
        let width = self.width();
        let height = self.height();

        for y in 0..height {
            for x in 0..width {
                let d_pixel = self.get_pixel(x, y).unwrap_or(0);
                let s_pixel = other.get_pixel(x, y).unwrap_or(0);

                let (dr, dg, db) = color::extract_rgb(d_pixel);
                let (sr, sg, sb) = color::extract_rgb(s_pixel);

                let rr = apply_rop_value(dr as u32, sr as u32, op, 255) as u8;
                let rg = apply_rop_value(dg as u32, sg as u32, op, 255) as u8;
                let rb = apply_rop_value(db as u32, sb as u32, op, 255) as u8;

                let result_pixel = color::compose_rgb(rr, rg, rb);
                self.set_pixel_unchecked(x, y, result_pixel);
            }
        }
    }

    /// Generic raster operation in place for other depths
    fn rop_generic_inplace(&mut self, other: &Pix, op: RopOp) {
        let width = self.width();
        let height = self.height();
        let max_val = self.depth().max_value();

        for y in 0..height {
            for x in 0..width {
                let d = self.get_pixel(x, y).unwrap_or(0);
                let s = other.get_pixel(x, y).unwrap_or(0);
                let val = apply_rop_value(d, s, op, max_val);
                self.set_pixel_unchecked(x, y, val);
            }
        }
    }

    /// Clear a rectangular region to zero.
    ///
    /// # Arguments
    ///
    /// * `x`, `y` - Top-left corner of the region
    /// * `w`, `h` - Width and height of the region
    pub fn clear_region(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.fill_region(x, y, w, h, 0);
    }

    /// Set a rectangular region to all ones.
    ///
    /// # Arguments
    ///
    /// * `x`, `y` - Top-left corner of the region
    /// * `w`, `h` - Width and height of the region
    pub fn set_region(&mut self, x: u32, y: u32, w: u32, h: u32) {
        let max_val = self.depth().max_value();
        self.fill_region(x, y, w, h, max_val);
    }

    /// In-place vertical band shift.
    ///
    /// Shifts a vertical band of the image up or down. The band extends the
    /// full height of the image. Exposed areas are filled with the specified
    /// color.
    ///
    /// # Arguments
    ///
    /// * `bx` - Left edge of vertical band
    /// * `bw` - Width of vertical band
    /// * `vshift` - Vertical shift (positive = down, negative = up)
    /// * `incolor` - Color to fill exposed areas
    ///
    /// # See also
    ///
    /// C Leptonica: `pixRasteropVip()` in `rop.c`
    pub fn rasterop_vip(&mut self, _bx: i32, _bw: i32, _vshift: i32, _incolor: InColor) {
        todo!()
    }

    /// In-place horizontal band shift.
    ///
    /// Shifts a horizontal band of the image left or right. The band extends
    /// the full width of the image. Exposed areas are filled with the specified
    /// color.
    ///
    /// # Arguments
    ///
    /// * `by` - Top of horizontal band
    /// * `bh` - Height of horizontal band
    /// * `hshift` - Horizontal shift (positive = right, negative = left)
    /// * `incolor` - Color to fill exposed areas
    ///
    /// # See also
    ///
    /// C Leptonica: `pixRasteropHip()` in `rop.c`
    pub fn rasterop_hip(&mut self, _by: i32, _bh: i32, _hshift: i32, _incolor: InColor) {
        todo!()
    }

    /// Fill a rectangular region with a constant value.
    fn fill_region(&mut self, x: u32, y: u32, w: u32, h: u32, value: u32) {
        let img_w = self.width();
        let img_h = self.height();

        // Clip to image bounds
        let x_end = (x + w).min(img_w);
        let y_end = (y + h).min(img_h);

        if x >= img_w || y >= img_h {
            return;
        }

        for py in y..y_end {
            for px in x..x_end {
                self.set_pixel_unchecked(px, py, value);
            }
        }
    }
}

/// Apply a raster operation to a 32-bit word (for binary images)
#[inline]
fn apply_rop_word(d: u32, s: u32, op: RopOp) -> u32 {
    match op {
        RopOp::Clear => 0,
        RopOp::Set => 0xFFFFFFFF,
        RopOp::Src => s,
        RopOp::NotDst => !d,
        RopOp::NotSrc => !s,
        RopOp::And => s & d,
        RopOp::Or => s | d,
        RopOp::Xor => s ^ d,
        RopOp::Nand => !(s & d),
        RopOp::Nor => !(s | d),
        RopOp::Xnor => !(s ^ d),
        RopOp::AndNotSrc => !s & d,
        RopOp::AndNotDst => s & !d,
        RopOp::OrNotSrc => !s | d,
        RopOp::OrNotDst => s | !d,
    }
}

/// Apply a raster operation to pixel values (for grayscale/RGB)
#[inline]
fn apply_rop_value(d: u32, s: u32, op: RopOp, max_val: u32) -> u32 {
    match op {
        RopOp::Clear => 0,
        RopOp::Set => max_val,
        RopOp::Src => s,
        RopOp::NotDst => max_val - d,
        RopOp::NotSrc => max_val - s,
        RopOp::And => s & d,
        RopOp::Or => s | d,
        RopOp::Xor => s ^ d,
        RopOp::Nand => max_val - (s & d),
        RopOp::Nor => max_val - (s | d),
        RopOp::Xnor => max_val - (s ^ d),
        RopOp::AndNotSrc => (max_val - s) & d,
        RopOp::AndNotDst => s & (max_val - d),
        RopOp::OrNotSrc => (max_val - s) | d,
        RopOp::OrNotDst => s | (max_val - d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and_binary() {
        // Create two binary images with some pixels set
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

        let result = pix1.and(&pix2).unwrap();

        // Only pixel (1,0) should be set (intersection)
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(1));
        assert_eq!(result.get_pixel(2, 0), Some(0));
    }

    #[test]
    fn test_or_binary() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 1).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(1, 0, 1).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.or(&pix2).unwrap();

        // Both pixels should be set (union)
        assert_eq!(result.get_pixel(0, 0), Some(1));
        assert_eq!(result.get_pixel(1, 0), Some(1));
        assert_eq!(result.get_pixel(2, 0), Some(0));
    }

    #[test]
    fn test_xor_binary() {
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

        let result = pix1.xor(&pix2).unwrap();

        // XOR: only pixels in exactly one image
        assert_eq!(result.get_pixel(0, 0), Some(1)); // only in pix1
        assert_eq!(result.get_pixel(1, 0), Some(0)); // in both
        assert_eq!(result.get_pixel(2, 0), Some(1)); // only in pix2
    }

    #[test]
    fn test_invert_binary() {
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(0, 0, 1).unwrap();
        let pix: Pix = pix_mut.into();

        let inverted = pix.invert();

        // Pixel (0,0) was 1, should now be 0
        assert_eq!(inverted.get_pixel(0, 0), Some(0));
        // Pixel (1,0) was 0, should now be 1
        assert_eq!(inverted.get_pixel(1, 0), Some(1));
    }

    #[test]
    fn test_invert_gray() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(0, 0, 100).unwrap();
        pix_mut.set_pixel(1, 0, 0).unwrap();
        pix_mut.set_pixel(2, 0, 255).unwrap();
        let pix: Pix = pix_mut.into();

        let inverted = pix.invert();

        assert_eq!(inverted.get_pixel(0, 0), Some(155)); // 255 - 100
        assert_eq!(inverted.get_pixel(1, 0), Some(255)); // 255 - 0
        assert_eq!(inverted.get_pixel(2, 0), Some(0)); // 255 - 255
    }

    #[test]
    fn test_and_gray() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 0b11110000).unwrap(); // 240
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(0, 0, 0b10101010).unwrap(); // 170
        let pix2: Pix = pix2_mut.into();

        let result = pix1.and(&pix2).unwrap();

        // 11110000 & 10101010 = 10100000 = 160
        assert_eq!(result.get_pixel(0, 0), Some(160));
    }

    #[test]
    fn test_or_gray() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 0b11110000).unwrap(); // 240
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(0, 0, 0b10101010).unwrap(); // 170
        let pix2: Pix = pix2_mut.into();

        let result = pix1.or(&pix2).unwrap();

        // 11110000 | 10101010 = 11111010 = 250
        assert_eq!(result.get_pixel(0, 0), Some(250));
    }

    #[test]
    fn test_xor_gray() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 0b11110000).unwrap(); // 240
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(0, 0, 0b10101010).unwrap(); // 170
        let pix2: Pix = pix2_mut.into();

        let result = pix1.xor(&pix2).unwrap();

        // 11110000 ^ 10101010 = 01011010 = 90
        assert_eq!(result.get_pixel(0, 0), Some(90));
    }

    #[test]
    fn test_dimension_mismatch_error() {
        let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(200, 100, PixelDepth::Bit8).unwrap();

        let result = pix1.and(&pix2);
        assert!(result.is_err());
    }

    #[test]
    fn test_depth_mismatch_error() {
        let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(100, 100, PixelDepth::Bit1).unwrap();

        let result = pix1.and(&pix2);
        assert!(result.is_err());
    }

    #[test]
    fn test_inplace_and() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 1).unwrap();
        pix1_mut.set_pixel(1, 0, 1).unwrap();

        let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(1, 0, 1).unwrap();
        let pix2: Pix = pix2_mut.into();

        pix1_mut.and_inplace(&pix2).unwrap();

        assert_eq!(pix1_mut.get_pixel(0, 0), Some(0));
        assert_eq!(pix1_mut.get_pixel(1, 0), Some(1));
    }

    #[test]
    fn test_inplace_invert() {
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.to_mut();
        pix_mut.set_pixel(0, 0, 1).unwrap();

        pix_mut.invert_inplace();

        assert_eq!(pix_mut.get_pixel(0, 0), Some(0));
        assert_eq!(pix_mut.get_pixel(1, 0), Some(1));
    }

    #[test]
    fn test_clear_region() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();

        // Set some pixels
        for y in 0..100 {
            for x in 0..100 {
                pix_mut.set_pixel(x, y, 200).unwrap();
            }
        }

        // Clear a region
        pix_mut.clear_region(10, 10, 20, 20);

        // Check pixels inside region are cleared
        assert_eq!(pix_mut.get_pixel(15, 15), Some(0));
        // Check pixels outside region are unchanged
        assert_eq!(pix_mut.get_pixel(5, 5), Some(200));
    }

    #[test]
    fn test_set_region() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();

        // Set a region
        pix_mut.set_region(10, 10, 20, 20);

        // Check pixels inside region are set
        assert_eq!(pix_mut.get_pixel(15, 15), Some(255));
        // Check pixels outside region are unchanged
        assert_eq!(pix_mut.get_pixel(5, 5), Some(0));
    }

    #[test]
    fn test_nand() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 1).unwrap();
        pix1_mut.set_pixel(1, 0, 1).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(1, 0, 1).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.rop(&pix2, RopOp::Nand).unwrap();

        // NAND: ~(s & d)
        // (0,0): ~(1 & 0) = ~0 = 1
        // (1,0): ~(1 & 1) = ~1 = 0
        // (2,0): ~(0 & 0) = ~0 = 1
        assert_eq!(result.get_pixel(0, 0), Some(1));
        assert_eq!(result.get_pixel(1, 0), Some(0));
        assert_eq!(result.get_pixel(2, 0), Some(1));
    }

    #[test]
    fn test_nor() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_pixel(0, 0, 1).unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut.set_pixel(1, 0, 1).unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.rop(&pix2, RopOp::Nor).unwrap();

        // NOR: ~(s | d)
        // (0,0): ~(1 | 0) = ~1 = 0
        // (1,0): ~(0 | 1) = ~1 = 0
        // (2,0): ~(0 | 0) = ~0 = 1
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(0));
        assert_eq!(result.get_pixel(2, 0), Some(1));
    }

    #[test]
    fn test_rgb_and() {
        use crate::color::compose_rgb;

        let pix1 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut
            .set_pixel(0, 0, compose_rgb(0b11110000, 0b10101010, 0b11001100))
            .unwrap();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pix2_mut = pix2.to_mut();
        pix2_mut
            .set_pixel(0, 0, compose_rgb(0b10101010, 0b11110000, 0b00110011))
            .unwrap();
        let pix2: Pix = pix2_mut.into();

        let result = pix1.and(&pix2).unwrap();
        let (r, g, b) = result.get_rgb(0, 0).unwrap();

        // R: 11110000 & 10101010 = 10100000 = 160
        // G: 10101010 & 11110000 = 10100000 = 160
        // B: 11001100 & 00110011 = 00000000 = 0
        assert_eq!(r, 160);
        assert_eq!(g, 160);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_rop_op_requires_source() {
        assert!(!RopOp::Clear.requires_source());
        assert!(!RopOp::Set.requires_source());
        assert!(!RopOp::NotDst.requires_source());
        assert!(RopOp::And.requires_source());
        assert!(RopOp::Or.requires_source());
        assert!(RopOp::Xor.requires_source());
        assert!(RopOp::Src.requires_source());
    }

    #[test]
    fn test_clear_rop() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.to_mut();
        pix1_mut.set_all();
        let pix1: Pix = pix1_mut.into();

        let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let result = pix1.rop(&pix2, RopOp::Clear).unwrap();

        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(31, 31), Some(0));
    }

    #[test]
    fn test_set_rop() {
        let pix1 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let pix2 = Pix::new(64, 64, PixelDepth::Bit1).unwrap();

        let result = pix1.rop(&pix2, RopOp::Set).unwrap();

        assert_eq!(result.get_pixel(0, 0), Some(1));
        assert_eq!(result.get_pixel(31, 31), Some(1));
    }
}
