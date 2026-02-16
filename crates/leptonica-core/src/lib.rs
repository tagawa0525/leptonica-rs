//! Leptonica Core - Basic data structures for image processing
//!
//! This crate provides the fundamental data structures used throughout
//! the Leptonica image processing library:
//!
//! - [`Pix`] / [`PixMut`] - The main image container (immutable / mutable)
//! - [`Box`] / [`Boxa`] / [`Boxaa`] - Rectangle regions
//! - [`Pta`] / [`Ptaa`] - Point arrays
//! - [`Numa`] / [`Numaa`] - Numeric arrays
//! - [`FPix`] - Floating-point image
//! - [`Pixa`] / [`Pixaa`] - Arrays of images
//! - [`Sarray`] / [`Sarraya`] - String arrays
//! - [`PixColormap`] - Color palette for indexed images
//!
//! # See also
//!
//! C Leptonica: `pix.h`, `box.h`, `pts.h`, `environ.h` (struct definitions)

pub mod box_;
pub mod colormap;
pub mod error;
pub mod fpix;
pub mod numa;
pub mod pix;
pub mod pixa;
pub mod pta;
pub mod sarray;

pub use box_::{Box, Boxa, Boxaa};
pub use colormap::{PixColormap, RgbaQuad};
pub use error::{Error, Result};
pub use fpix::{FPix, NegativeHandling};
pub use numa::{HistogramResult, HistogramStats, Numa, Numaa, SortOrder, WindowedStats};
pub use pix::statistics::{
    DiffDirection, ExtremeResult, ExtremeType, MaxValueResult, PixelMaxType, PixelStatType,
    RowColumnStats, StatsRequest,
};
pub use pix::{
    BlendMode, Color, ColorHistogram, CompareResult, CompareType, ContourOutput, GrayBlendType,
    ImageFormat, MaskBlendType, Pix, PixMut, PixelDepth, PixelDiffResult, PixelOp, RopOp,
    blend_with_gray_mask, correlation_binary,
};
pub use pixa::{Pixa, Pixaa};
pub use pta::{Pta, Ptaa};
pub use sarray::{Sarray, Sarraya};

/// Color channel indices and helper functions for 32-bit RGBA pixels.
///
/// # Pixel format
///
/// 32-bit pixels are stored as `0xRRGGBBAA` (red in MSB, alpha in LSB).
///
/// # See also
///
/// C Leptonica: color component macros in `pix.h`
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

    /// Extract red component from a 32-bit pixel.
    #[inline]
    pub fn red(pixel: u32) -> u8 {
        ((pixel >> RED_SHIFT) & 0xff) as u8
    }

    /// Extract green component from a 32-bit pixel.
    #[inline]
    pub fn green(pixel: u32) -> u8 {
        ((pixel >> GREEN_SHIFT) & 0xff) as u8
    }

    /// Extract blue component from a 32-bit pixel.
    #[inline]
    pub fn blue(pixel: u32) -> u8 {
        ((pixel >> BLUE_SHIFT) & 0xff) as u8
    }

    /// Extract alpha component from a 32-bit pixel.
    #[inline]
    pub fn alpha(pixel: u32) -> u8 {
        ((pixel >> ALPHA_SHIFT) & 0xff) as u8
    }

    /// Compose a 32-bit RGB pixel (alpha = 255).
    #[inline]
    pub fn compose_rgb(r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << RED_SHIFT)
            | ((g as u32) << GREEN_SHIFT)
            | ((b as u32) << BLUE_SHIFT)
            | (255 << ALPHA_SHIFT)
    }

    /// Compose a 32-bit RGBA pixel.
    #[inline]
    pub fn compose_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
        ((r as u32) << RED_SHIFT)
            | ((g as u32) << GREEN_SHIFT)
            | ((b as u32) << BLUE_SHIFT)
            | ((a as u32) << ALPHA_SHIFT)
    }

    /// Extract RGB values from a 32-bit pixel.
    #[inline]
    pub fn extract_rgb(pixel: u32) -> (u8, u8, u8) {
        (red(pixel), green(pixel), blue(pixel))
    }

    /// Extract RGBA values from a 32-bit pixel.
    #[inline]
    pub fn extract_rgba(pixel: u32) -> (u8, u8, u8, u8) {
        (red(pixel), green(pixel), blue(pixel), alpha(pixel))
    }

    /// HSV color values.
    ///
    /// Ranges: h [0..239] (h=240 wraps to 0), s [0..255], v [0..255].
    /// Hue wraps: h=0 and h=240 are equivalent.
    ///
    /// Hue correspondence (same as C Leptonica `convertRGBToHSV()`):
    /// - 0: red
    /// - 40: yellow
    /// - 80: green
    /// - 120: cyan
    /// - 160: blue
    /// - 200: magenta
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Hsv {
        pub h: i32,
        pub s: i32,
        pub v: i32,
    }

    /// Convert RGB to HSV color space.
    ///
    /// # See also
    ///
    /// C Leptonica: `convertRGBToHSV()` in `colorspace.c`
    pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> Hsv {
        let ri = r as i32;
        let gi = g as i32;
        let bi = b as i32;

        let min = ri.min(gi).min(bi);
        let max = ri.max(gi).max(bi);
        let delta = max - min;

        let v = max;
        if delta == 0 {
            return Hsv { h: 0, s: 0, v };
        }

        let s = (255.0 * delta as f32 / max as f32 + 0.5) as i32;
        let h_raw = if ri == max {
            (gi - bi) as f32 / delta as f32
        } else if gi == max {
            2.0 + (bi - ri) as f32 / delta as f32
        } else {
            4.0 + (ri - gi) as f32 / delta as f32
        };

        let mut h = h_raw * 40.0;
        if h < 0.0 {
            h += 240.0;
        }
        if h >= 239.5 {
            h = 0.0;
        }
        let h = (h + 0.5) as i32;

        Hsv { h, s, v }
    }

    /// Convert HSV to RGB color space.
    ///
    /// # See also
    ///
    /// C Leptonica: `convertHSVToRGB()` in `colorspace.c`
    pub fn hsv_to_rgb(hsv: Hsv) -> (u8, u8, u8) {
        let Hsv {
            mut h,
            s: sval,
            v: vval,
        } = hsv;

        if sval == 0 {
            return (vval as u8, vval as u8, vval as u8);
        }

        if h == 240 {
            h = 0;
        }
        let hf = h as f32 / 40.0;
        let i = hf as i32;
        let f = hf - i as f32;
        let s = sval as f32 / 255.0;
        let x = (vval as f32 * (1.0 - s) + 0.5) as i32;
        let y = (vval as f32 * (1.0 - s * f) + 0.5) as i32;
        let z = (vval as f32 * (1.0 - s * (1.0 - f)) + 0.5) as i32;

        let (r, g, b) = match i {
            0 => (vval, z, x),
            1 => (y, vval, x),
            2 => (x, vval, z),
            3 => (x, y, vval),
            4 => (z, x, vval),
            5 => (vval, x, y),
            _ => (0, 0, 0),
        };

        (r as u8, g as u8, b as u8)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_rgb_to_hsv_pure_red() {
            // Pure red sits at sector boundary h=0 in Leptonica's HSV
            let hsv = rgb_to_hsv(255, 0, 0);
            assert_eq!(hsv.h, 0);
            assert_eq!(hsv.s, 255);
            assert_eq!(hsv.v, 255);
        }

        #[test]
        fn test_rgb_to_hsv_pure_green() {
            // Pure green at sector boundary h=80
            let hsv = rgb_to_hsv(0, 255, 0);
            assert_eq!(hsv.h, 80);
            assert_eq!(hsv.s, 255);
            assert_eq!(hsv.v, 255);
        }

        #[test]
        fn test_rgb_to_hsv_pure_blue() {
            // Pure blue at sector boundary h=160
            let hsv = rgb_to_hsv(0, 0, 255);
            assert_eq!(hsv.h, 160);
            assert_eq!(hsv.s, 255);
            assert_eq!(hsv.v, 255);
        }

        #[test]
        fn test_rgb_to_hsv_gray() {
            let hsv = rgb_to_hsv(128, 128, 128);
            assert_eq!(hsv.h, 0);
            assert_eq!(hsv.s, 0);
            assert_eq!(hsv.v, 128);
        }

        #[test]
        fn test_rgb_to_hsv_black() {
            let hsv = rgb_to_hsv(0, 0, 0);
            assert_eq!(hsv.h, 0);
            assert_eq!(hsv.s, 0);
            assert_eq!(hsv.v, 0);
        }

        #[test]
        fn test_rgb_to_hsv_white() {
            let hsv = rgb_to_hsv(255, 255, 255);
            assert_eq!(hsv.h, 0);
            assert_eq!(hsv.s, 0);
            assert_eq!(hsv.v, 255);
        }

        #[test]
        fn test_hsv_roundtrip() {
            // Test roundtrip for several colors
            let colors = [
                (255, 0, 0),
                (0, 255, 0),
                (0, 0, 255),
                (255, 255, 0),
                (0, 255, 255),
                (128, 64, 32),
            ];
            for (r, g, b) in colors {
                let hsv = rgb_to_hsv(r, g, b);
                let (rr, rg, rb) = hsv_to_rgb(hsv);
                assert!(
                    (rr as i32 - r as i32).abs() <= 1
                        && (rg as i32 - g as i32).abs() <= 1
                        && (rb as i32 - b as i32).abs() <= 1,
                    "roundtrip failed for ({r},{g},{b}): got ({rr},{rg},{rb})"
                );
            }
        }

        #[test]
        fn test_hsv_to_rgb_gray() {
            let (r, g, b) = hsv_to_rgb(Hsv { h: 0, s: 0, v: 128 });
            assert_eq!((r, g, b), (128, 128, 128));
        }
    }
}
