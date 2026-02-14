//! Image arithmetic operations
//!
//! Provides pixel-wise arithmetic: add, subtract, multiply, absolute difference,
//! min/max. Corresponds to C Leptonica `pixarith.c`.

use super::{Pix, PixMut, PixelDepth};
use crate::error::{Error, Result};

impl Pix {
    /// Add a constant value to all pixels (clipped to valid range).
    ///
    /// # Errors
    ///
    /// Returns error if depth is 1bpp (not supported).
    pub fn add_constant(&self, _val: i32) -> Result<Pix> {
        todo!()
    }

    /// Multiply all pixels by a constant factor (clipped to valid range).
    ///
    /// # Errors
    ///
    /// Returns error if factor is negative or depth is not supported.
    pub fn multiply_constant(&self, _factor: f32) -> Result<Pix> {
        todo!()
    }

    /// Pixel-wise addition of two images.
    pub fn arith_add(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Pixel-wise subtraction of two images.
    pub fn arith_subtract(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Pixel-wise absolute difference of two images.
    pub fn arith_abs_diff(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Pixel-wise minimum of two images.
    pub fn arith_min(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Pixel-wise maximum of two images.
    pub fn arith_max(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Multiply two grayscale images pixel-wise.
    pub fn multiply_gray(&self, _gray: &Pix, _norm: Option<f32>) -> Result<Pix> {
        todo!()
    }
}

impl PixMut {
    /// Add a constant to all pixels in-place.
    pub fn add_constant_inplace(&mut self, _val: i32) {
        todo!()
    }

    /// Multiply all pixels by a constant in-place.
    pub fn multiply_constant_inplace(&mut self, _factor: f32) -> Result<()> {
        todo!()
    }

    /// Add another image to this one in-place.
    pub fn arith_add_inplace(&mut self, _other: &Pix) -> Result<()> {
        todo!()
    }

    /// Subtract another image from this one in-place.
    pub fn arith_subtract_inplace(&mut self, _other: &Pix) -> Result<()> {
        todo!()
    }
}
