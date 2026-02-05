//! Leptonica Core - Basic data structures for image processing
//!
//! This crate provides the fundamental data structures used throughout
//! the Leptonica image processing library:
//!
//! - [`Pix`] - The main image container
//! - [`Pixa`] / [`Pixaa`] - Arrays of images
//! - [`Box`] / [`Boxa`] - Rectangle regions
//! - [`Pta`] / [`Ptaa`] - Point arrays
//! - [`Numa`] / [`Numaa`] - Numeric arrays
//! - [`PixColormap`] - Color palette for indexed images

pub mod box_;
pub mod colormap;
pub mod error;
pub mod numa;
pub mod pix;
pub mod pixa;
pub mod pta;

pub use box_::{Box, Boxa, Boxaa};
pub use colormap::PixColormap;
pub use error::{Error, Result};
pub use numa::{Numa, Numaa};
pub use pix::{ImageFormat, Pix, PixMut, PixelDepth};
pub use pixa::{Pixa, Pixaa};
pub use pta::{Pta, Ptaa};

/// Color channel indices for 32-bit RGBA pixels
pub mod color {
    /// Red channel (MSB, byte 0)
    pub const RED: usize = 0;
    /// Green channel (byte 1)
    pub const GREEN: usize = 1;
    /// Blue channel (byte 2)
    pub const BLUE: usize = 2;
    /// Alpha channel (LSB, byte 3)
    pub const ALPHA: usize = 3;

    /// Shift amounts for extracting color channels
    pub const RED_SHIFT: u32 = 24;
    pub const GREEN_SHIFT: u32 = 16;
    pub const BLUE_SHIFT: u32 = 8;
    pub const ALPHA_SHIFT: u32 = 0;

    /// Extract red component from a 32-bit pixel
    #[inline]
    pub fn red(pixel: u32) -> u8 {
        ((pixel >> RED_SHIFT) & 0xff) as u8
    }

    /// Extract green component from a 32-bit pixel
    #[inline]
    pub fn green(pixel: u32) -> u8 {
        ((pixel >> GREEN_SHIFT) & 0xff) as u8
    }

    /// Extract blue component from a 32-bit pixel
    #[inline]
    pub fn blue(pixel: u32) -> u8 {
        ((pixel >> BLUE_SHIFT) & 0xff) as u8
    }

    /// Extract alpha component from a 32-bit pixel
    #[inline]
    pub fn alpha(pixel: u32) -> u8 {
        ((pixel >> ALPHA_SHIFT) & 0xff) as u8
    }

    /// Compose a 32-bit RGB pixel (alpha = 255)
    #[inline]
    pub fn compose_rgb(r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << RED_SHIFT)
            | ((g as u32) << GREEN_SHIFT)
            | ((b as u32) << BLUE_SHIFT)
            | (255 << ALPHA_SHIFT)
    }

    /// Compose a 32-bit RGBA pixel
    #[inline]
    pub fn compose_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
        ((r as u32) << RED_SHIFT)
            | ((g as u32) << GREEN_SHIFT)
            | ((b as u32) << BLUE_SHIFT)
            | ((a as u32) << ALPHA_SHIFT)
    }

    /// Extract RGB values from a 32-bit pixel
    #[inline]
    pub fn extract_rgb(pixel: u32) -> (u8, u8, u8) {
        (red(pixel), green(pixel), blue(pixel))
    }

    /// Extract RGBA values from a 32-bit pixel
    #[inline]
    pub fn extract_rgba(pixel: u32) -> (u8, u8, u8, u8) {
        (red(pixel), green(pixel), blue(pixel), alpha(pixel))
    }
}
