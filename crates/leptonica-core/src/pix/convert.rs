//! Pixel depth conversion functions
//!
//! Functions for converting between different pixel depths.
//!
//! # See also
//!
//! C Leptonica: `pixconv.c` (`pixConvertTo8`, `pixConvertTo32`, etc.)

use super::{Pix, PixelDepth};
use crate::error::Result;

impl Pix {
    /// Convert any-depth image to 8-bit grayscale.
    ///
    /// Conversion rules:
    /// - **1 bpp**: 0 -> 255 (white), 1 -> 0 (black)
    /// - **2 bpp**: evenly spaced values (0, 85, 170, 255)
    /// - **4 bpp**: evenly spaced values (0, 17, 34, ... 255)
    /// - **8 bpp**: copy (lossless)
    /// - **16 bpp**: use most significant byte
    /// - **32 bpp**: convert to luminance using perceptual weights
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertTo8()` in `pixconv.c`
    pub fn convert_to_8(&self) -> Result<Pix> {
        todo!("Pix::convert_to_8")
    }

    /// Convert any-depth image to 32-bit RGB.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertTo32()` in `pixconv.c`
    pub fn convert_to_32(&self) -> Result<Pix> {
        todo!("Pix::convert_to_32")
    }
}
