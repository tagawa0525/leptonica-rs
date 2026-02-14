//! Pixel access functions
//!
//! Low-level functions for getting and setting individual pixels.
//! These correspond to Leptonica's `GET_DATA_*` and `SET_DATA_*` macros
//! in `arrayaccess.h`.
//!
//! # Pixel packing
//!
//! Pixels are packed MSB-to-LSB within each 32-bit word. For example,
//! in a 1-bit image, pixel 0 occupies bit 31 (MSB) of the first word.
//!
//! # See also
//!
//! C Leptonica: `arrayaccess.h` (macros), `pix2.c` (`pixGetPixel` / `pixSetPixel`)

use super::{Pix, PixMut, PixelDepth};
use crate::error::{Error, Result};

impl Pix {
    /// Get a pixel value at (x, y).
    ///
    /// Returns `None` if coordinates are out of bounds.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetPixel()` in `pix2.c`
    pub fn get_pixel(&self, _x: u32, _y: u32) -> Option<u32> {
        todo!("Pix::get_pixel")
    }

    /// Get a pixel value without bounds checking.
    ///
    /// # Panics
    ///
    /// Panics if `x >= width` or `y >= height`.
    #[inline]
    pub fn get_pixel_unchecked(&self, _x: u32, _y: u32) -> u32 {
        todo!("Pix::get_pixel_unchecked")
    }

    /// Get RGB values at (x, y).
    ///
    /// Only valid for 32-bit images.
    pub fn get_rgb(&self, _x: u32, _y: u32) -> Option<(u8, u8, u8)> {
        todo!("Pix::get_rgb")
    }

    /// Get RGBA values at (x, y).
    ///
    /// Only valid for 32-bit images.
    pub fn get_rgba(&self, _x: u32, _y: u32) -> Option<(u8, u8, u8, u8)> {
        todo!("Pix::get_rgba")
    }
}

impl PixMut {
    /// Get a pixel value at (x, y).
    pub fn get_pixel(&self, _x: u32, _y: u32) -> Option<u32> {
        todo!("PixMut::get_pixel")
    }

    /// Get a pixel value without bounds checking.
    #[inline]
    pub fn get_pixel_unchecked(&self, _x: u32, _y: u32) -> u32 {
        todo!("PixMut::get_pixel_unchecked")
    }

    /// Set a pixel value at (x, y).
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexOutOfBounds`] if coordinates are out of bounds.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetPixel()` in `pix2.c`
    pub fn set_pixel(&mut self, _x: u32, _y: u32, _val: u32) -> Result<()> {
        todo!("PixMut::set_pixel")
    }

    /// Set a pixel value without bounds checking.
    ///
    /// # Panics
    ///
    /// Panics if `x >= width` or `y >= height`.
    #[inline]
    pub fn set_pixel_unchecked(&mut self, _x: u32, _y: u32, _val: u32) {
        todo!("PixMut::set_pixel_unchecked")
    }

    /// Set an RGB pixel at (x, y).
    ///
    /// Only valid for 32-bit images.
    pub fn set_rgb(&mut self, _x: u32, _y: u32, _r: u8, _g: u8, _b: u8) -> Result<()> {
        todo!("PixMut::set_rgb")
    }

    /// Set an RGBA pixel at (x, y).
    ///
    /// Only valid for 32-bit images with spp=4.
    pub fn set_rgba(&mut self, _x: u32, _y: u32, _r: u8, _g: u8, _b: u8, _a: u8) -> Result<()> {
        todo!("PixMut::set_rgba")
    }
}

/// Get a 1-bit pixel value.
///
/// Pixels are packed MSB to LSB within each 32-bit word.
///
/// # See also
///
/// C Leptonica: `GET_DATA_BIT` macro in `arrayaccess.h`
#[inline]
pub fn get_data_bit(_line: &[u32], _x: u32) -> u32 {
    todo!("get_data_bit")
}

/// Set a 1-bit pixel value.
#[inline]
pub fn set_data_bit(_line: &mut [u32], _x: u32, _val: u32) {
    todo!("set_data_bit")
}

/// Set a 1-bit pixel to 1.
#[inline]
pub fn set_data_bit_val(_line: &mut [u32], _x: u32) {
    todo!("set_data_bit_val")
}

/// Clear a 1-bit pixel to 0.
#[inline]
pub fn clear_data_bit(_line: &mut [u32], _x: u32) {
    todo!("clear_data_bit")
}

/// Get a 2-bit pixel value.
///
/// # See also
///
/// C Leptonica: `GET_DATA_DIBIT` macro
#[inline]
pub fn get_data_dibit(_line: &[u32], _x: u32) -> u32 {
    todo!("get_data_dibit")
}

/// Set a 2-bit pixel value.
#[inline]
pub fn set_data_dibit(_line: &mut [u32], _x: u32, _val: u32) {
    todo!("set_data_dibit")
}

/// Get a 4-bit pixel value.
///
/// # See also
///
/// C Leptonica: `GET_DATA_QBIT` macro
#[inline]
pub fn get_data_qbit(_line: &[u32], _x: u32) -> u32 {
    todo!("get_data_qbit")
}

/// Set a 4-bit pixel value.
#[inline]
pub fn set_data_qbit(_line: &mut [u32], _x: u32, _val: u32) {
    todo!("set_data_qbit")
}

/// Get an 8-bit pixel value.
///
/// # See also
///
/// C Leptonica: `GET_DATA_BYTE` macro
#[inline]
pub fn get_data_byte(_line: &[u32], _x: u32) -> u32 {
    todo!("get_data_byte")
}

/// Set an 8-bit pixel value.
#[inline]
pub fn set_data_byte(_line: &mut [u32], _x: u32, _val: u32) {
    todo!("set_data_byte")
}

/// Get a 16-bit pixel value.
///
/// # See also
///
/// C Leptonica: `GET_DATA_TWO_BYTES` macro
#[inline]
pub fn get_data_two_bytes(_line: &[u32], _x: u32) -> u32 {
    todo!("get_data_two_bytes")
}

/// Set a 16-bit pixel value.
#[inline]
pub fn set_data_two_bytes(_line: &mut [u32], _x: u32, _val: u32) {
    todo!("set_data_two_bytes")
}
