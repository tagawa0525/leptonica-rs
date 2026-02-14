//! Pixel access functions
//!
//! Low-level functions for getting and setting individual pixels.
//! These correspond to Leptonica's GET_DATA_* and SET_DATA_* macros.

use super::{Pix, PixMut, PixelDepth};
use crate::error::{Error, Result};

/// Bit masks for pixel extraction
const MASK_1: u32 = 0x1;
const MASK_2: u32 = 0x3;
const MASK_4: u32 = 0xF;
const MASK_8: u32 = 0xFF;
const MASK_16: u32 = 0xFFFF;

impl Pix {
    /// Get a pixel value at (x, y)
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (column)
    /// * `y` - Y coordinate (row)
    ///
    /// # Returns
    ///
    /// The pixel value, or None if coordinates are out of bounds.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<u32> {
        if x >= self.width() || y >= self.height() {
            return None;
        }
        Some(self.get_pixel_unchecked(x, y))
    }

    /// Get a pixel value without bounds checking.
    ///
    /// # Safety contract
    ///
    /// The caller must ensure `x < width()` and `y < height()`. For packed
    /// pixel depths (1/2/4/8/16 bpp), an `x` within the row padding may
    /// not cause a panic but will return undefined padding bits.
    ///
    /// # Panics
    ///
    /// Panics if `y >= height()` (row indexing). May panic if `x` exceeds
    /// the word boundary for 32-bit images.
    #[inline]
    pub fn get_pixel_unchecked(&self, x: u32, y: u32) -> u32 {
        let line = self.row_data(y);
        get_pixel_from_line(line, x, self.depth())
    }
}

impl PixMut {
    /// Get a pixel value at (x, y)
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<u32> {
        if x >= self.width() || y >= self.height() {
            return None;
        }
        Some(self.get_pixel_unchecked(x, y))
    }

    /// Get a pixel value without bounds checking.
    ///
    /// # Safety contract
    ///
    /// The caller must ensure `x < width()` and `y < height()`.
    ///
    /// # Panics
    ///
    /// Panics if `y >= height()` (row indexing).
    #[inline]
    pub fn get_pixel_unchecked(&self, x: u32, y: u32) -> u32 {
        let wpl = self.wpl();
        let start = (y * wpl) as usize;
        let line = &self.data()[start..start + wpl as usize];
        get_pixel_from_line(line, x, self.depth())
    }

    /// Set a pixel value at (x, y)
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (column)
    /// * `y` - Y coordinate (row)
    /// * `val` - Pixel value
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, Err if coordinates are out of bounds.
    pub fn set_pixel(&mut self, x: u32, y: u32, val: u32) -> Result<()> {
        let width = self.width();
        let height = self.height();

        if x >= width {
            return Err(Error::IndexOutOfBounds {
                index: x as usize,
                len: width as usize,
            });
        }

        if y >= height {
            return Err(Error::IndexOutOfBounds {
                index: y as usize,
                len: height as usize,
            });
        }
        self.set_pixel_unchecked(x, y, val);
        Ok(())
    }

    /// Set a pixel value without bounds checking
    ///
    /// # Panics
    ///
    /// Panics if `x` or `y` is out of bounds.
    #[inline]
    pub fn set_pixel_unchecked(&mut self, x: u32, y: u32, val: u32) {
        let wpl = self.wpl();
        let depth = self.depth();
        let start = (y * wpl) as usize;
        let line = &mut self.data_mut()[start..start + wpl as usize];
        set_pixel_in_line(line, x, val, depth);
    }
}

/// Get a pixel value from a line buffer
#[inline]
fn get_pixel_from_line(line: &[u32], x: u32, depth: PixelDepth) -> u32 {
    match depth {
        PixelDepth::Bit1 => get_data_bit(line, x),
        PixelDepth::Bit2 => get_data_dibit(line, x),
        PixelDepth::Bit4 => get_data_qbit(line, x),
        PixelDepth::Bit8 => get_data_byte(line, x),
        PixelDepth::Bit16 => get_data_two_bytes(line, x),
        PixelDepth::Bit32 => line[x as usize],
    }
}

/// Set a pixel value in a line buffer
#[inline]
fn set_pixel_in_line(line: &mut [u32], x: u32, val: u32, depth: PixelDepth) {
    match depth {
        PixelDepth::Bit1 => set_data_bit(line, x, val),
        PixelDepth::Bit2 => set_data_dibit(line, x, val),
        PixelDepth::Bit4 => set_data_qbit(line, x, val),
        PixelDepth::Bit8 => set_data_byte(line, x, val),
        PixelDepth::Bit16 => set_data_two_bytes(line, x, val),
        PixelDepth::Bit32 => line[x as usize] = val,
    }
}

// ============================================================================
// 1-bit access (GET_DATA_BIT / SET_DATA_BIT)
// ============================================================================

/// Get a 1-bit pixel value
///
/// Pixels are packed MSB to LSB within each 32-bit word.
#[inline]
pub fn get_data_bit(line: &[u32], x: u32) -> u32 {
    let word_index = (x >> 5) as usize; // x / 32
    let bit_index = 31 - (x & 31); // MSB to LSB ordering
    (line[word_index] >> bit_index) & MASK_1
}

/// Set a 1-bit pixel value
#[inline]
pub fn set_data_bit(line: &mut [u32], x: u32, val: u32) {
    let word_index = (x >> 5) as usize;
    let bit_index = 31 - (x & 31);
    if val != 0 {
        line[word_index] |= 1 << bit_index;
    } else {
        line[word_index] &= !(1 << bit_index);
    }
}

/// Set a 1-bit pixel to 1
#[inline]
pub fn set_data_bit_val(line: &mut [u32], x: u32) {
    let word_index = (x >> 5) as usize;
    let bit_index = 31 - (x & 31);
    line[word_index] |= 1 << bit_index;
}

/// Clear a 1-bit pixel to 0
#[inline]
pub fn clear_data_bit(line: &mut [u32], x: u32) {
    let word_index = (x >> 5) as usize;
    let bit_index = 31 - (x & 31);
    line[word_index] &= !(1 << bit_index);
}

// ============================================================================
// 2-bit access (GET_DATA_DIBIT / SET_DATA_DIBIT)
// ============================================================================

/// Get a 2-bit pixel value
#[inline]
pub fn get_data_dibit(line: &[u32], x: u32) -> u32 {
    let word_index = (x >> 4) as usize; // x / 16
    let bit_index = 2 * (15 - (x & 15)); // MSB to LSB, 2 bits per pixel
    (line[word_index] >> bit_index) & MASK_2
}

/// Set a 2-bit pixel value
#[inline]
pub fn set_data_dibit(line: &mut [u32], x: u32, val: u32) {
    let word_index = (x >> 4) as usize;
    let bit_index = 2 * (15 - (x & 15));
    let mask = MASK_2 << bit_index;
    line[word_index] = (line[word_index] & !mask) | ((val & MASK_2) << bit_index);
}

// ============================================================================
// 4-bit access (GET_DATA_QBIT / SET_DATA_QBIT)
// ============================================================================

/// Get a 4-bit pixel value
#[inline]
pub fn get_data_qbit(line: &[u32], x: u32) -> u32 {
    let word_index = (x >> 3) as usize; // x / 8
    let bit_index = 4 * (7 - (x & 7)); // MSB to LSB, 4 bits per pixel
    (line[word_index] >> bit_index) & MASK_4
}

/// Set a 4-bit pixel value
#[inline]
pub fn set_data_qbit(line: &mut [u32], x: u32, val: u32) {
    let word_index = (x >> 3) as usize;
    let bit_index = 4 * (7 - (x & 7));
    let mask = MASK_4 << bit_index;
    line[word_index] = (line[word_index] & !mask) | ((val & MASK_4) << bit_index);
}

// ============================================================================
// 8-bit access (GET_DATA_BYTE / SET_DATA_BYTE)
// ============================================================================

/// Get an 8-bit pixel value
#[inline]
pub fn get_data_byte(line: &[u32], x: u32) -> u32 {
    let word_index = (x >> 2) as usize; // x / 4
    let byte_index = 3 - (x & 3); // MSB to LSB, 4 bytes per word
    (line[word_index] >> (byte_index * 8)) & MASK_8
}

/// Set an 8-bit pixel value
#[inline]
pub fn set_data_byte(line: &mut [u32], x: u32, val: u32) {
    let word_index = (x >> 2) as usize;
    let byte_index = 3 - (x & 3);
    let shift = byte_index * 8;
    let mask = MASK_8 << shift;
    line[word_index] = (line[word_index] & !mask) | ((val & MASK_8) << shift);
}

// ============================================================================
// 16-bit access (GET_DATA_TWO_BYTES / SET_DATA_TWO_BYTES)
// ============================================================================

/// Get a 16-bit pixel value
#[inline]
pub fn get_data_two_bytes(line: &[u32], x: u32) -> u32 {
    let word_index = (x >> 1) as usize; // x / 2
    let half_index = 1 - (x & 1); // MSB to LSB, 2 half-words per word
    (line[word_index] >> (half_index * 16)) & MASK_16
}

/// Set a 16-bit pixel value
#[inline]
pub fn set_data_two_bytes(line: &mut [u32], x: u32, val: u32) {
    let word_index = (x >> 1) as usize;
    let half_index = 1 - (x & 1);
    let shift = half_index * 16;
    let mask = MASK_16 << shift;
    line[word_index] = (line[word_index] & !mask) | ((val & MASK_16) << shift);
}

// ============================================================================
// Convenience functions for 32-bit color
// ============================================================================

impl PixMut {
    /// Set an RGB pixel at (x, y)
    ///
    /// Only valid for 32-bit images.
    pub fn set_rgb(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) -> Result<()> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::IncompatibleDepths(self.depth().bits(), 32));
        }
        let pixel = crate::color::compose_rgb(r, g, b);
        self.set_pixel(x, y, pixel)
    }

    /// Set an RGBA pixel at (x, y).
    ///
    /// Only valid for 32-bit images. Does not modify the `spp` metadata;
    /// if writing alpha data, set `spp` to 4 separately via [`PixMut::set_spp`].
    pub fn set_rgba(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) -> Result<()> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::IncompatibleDepths(self.depth().bits(), 32));
        }
        let pixel = crate::color::compose_rgba(r, g, b, a);
        self.set_pixel(x, y, pixel)
    }
}

impl Pix {
    /// Get RGB values at (x, y)
    ///
    /// Only valid for 32-bit images.
    pub fn get_rgb(&self, x: u32, y: u32) -> Option<(u8, u8, u8)> {
        if self.depth() != PixelDepth::Bit32 {
            return None;
        }
        self.get_pixel(x, y).map(crate::color::extract_rgb)
    }

    /// Get RGBA values at (x, y)
    ///
    /// Only valid for 32-bit images.
    pub fn get_rgba(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if self.depth() != PixelDepth::Bit32 {
            return None;
        }
        self.get_pixel(x, y).map(crate::color::extract_rgba)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1bit_access() {
        let mut line = [0u32; 2];

        // Set some bits
        set_data_bit(&mut line, 0, 1); // First bit (MSB of first word)
        set_data_bit(&mut line, 31, 1); // Last bit of first word
        set_data_bit(&mut line, 32, 1); // First bit of second word

        assert_eq!(get_data_bit(&line, 0), 1);
        assert_eq!(get_data_bit(&line, 1), 0);
        assert_eq!(get_data_bit(&line, 31), 1);
        assert_eq!(get_data_bit(&line, 32), 1);
        assert_eq!(get_data_bit(&line, 33), 0);

        // Clear a bit
        set_data_bit(&mut line, 0, 0);
        assert_eq!(get_data_bit(&line, 0), 0);
    }

    #[test]
    fn test_2bit_access() {
        let mut line = [0u32; 1];

        set_data_dibit(&mut line, 0, 3); // First 2 bits
        set_data_dibit(&mut line, 15, 2); // Last 2 bits of word

        assert_eq!(get_data_dibit(&line, 0), 3);
        assert_eq!(get_data_dibit(&line, 1), 0);
        assert_eq!(get_data_dibit(&line, 15), 2);
    }

    #[test]
    fn test_4bit_access() {
        let mut line = [0u32; 1];

        set_data_qbit(&mut line, 0, 0xF);
        set_data_qbit(&mut line, 7, 0xA);

        assert_eq!(get_data_qbit(&line, 0), 0xF);
        assert_eq!(get_data_qbit(&line, 1), 0);
        assert_eq!(get_data_qbit(&line, 7), 0xA);
    }

    #[test]
    fn test_8bit_access() {
        let mut line = [0u32; 1];

        set_data_byte(&mut line, 0, 0xFF);
        set_data_byte(&mut line, 3, 0x42);

        assert_eq!(get_data_byte(&line, 0), 0xFF);
        assert_eq!(get_data_byte(&line, 1), 0);
        assert_eq!(get_data_byte(&line, 3), 0x42);

        // Verify word layout: byte 0 is MSB
        assert_eq!(line[0], 0xFF000042);
    }

    #[test]
    fn test_16bit_access() {
        let mut line = [0u32; 1];

        set_data_two_bytes(&mut line, 0, 0xABCD);
        set_data_two_bytes(&mut line, 1, 0x1234);

        assert_eq!(get_data_two_bytes(&line, 0), 0xABCD);
        assert_eq!(get_data_two_bytes(&line, 1), 0x1234);

        // Verify word layout
        assert_eq!(line[0], 0xABCD1234);
    }

    #[test]
    fn test_pix_pixel_access() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        pix_mut.set_pixel(5, 5, 128).unwrap();
        assert_eq!(pix_mut.get_pixel(5, 5), Some(128));

        // Out of bounds
        assert!(pix_mut.set_pixel(100, 5, 128).is_err());
        assert_eq!(pix_mut.get_pixel(100, 5), None);
    }

    #[test]
    fn test_rgb_access() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        pix_mut.set_rgb(0, 0, 255, 128, 64).unwrap();

        let pix: Pix = pix_mut.into();
        assert_eq!(pix.get_rgb(0, 0), Some((255, 128, 64)));
    }
}
