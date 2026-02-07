//! Pixel depth conversion functions
//!
//! Functions for converting between different pixel depths.
//! Corresponds to functions in C Leptonica's `pixconv.c`.

use super::{Pix, PixelDepth};
use crate::color;
use crate::error::{Error, Result};

/// Default perceptual weights for RGB-to-gray conversion.
///
/// These match C Leptonica's `L_RED_WEIGHT`, `L_GREEN_WEIGHT`, `L_BLUE_WEIGHT`.
const L_RED_WEIGHT: f32 = 0.3;
const L_GREEN_WEIGHT: f32 = 0.5;
const L_BLUE_WEIGHT: f32 = 0.2;

impl Pix {
    /// Convert any-depth image to 8-bit grayscale.
    ///
    /// This is a top-level conversion function with simple default values.
    /// It always creates a new image (never a clone).
    ///
    /// Conversion rules:
    /// - **1 bpp**: 0 -> 255 (white), 1 -> 0 (black)
    /// - **2 bpp**: evenly spaced values (0, 85, 170, 255)
    /// - **4 bpp**: evenly spaced values (0, 17, 34, ... 255)
    /// - **8 bpp**: copy (lossless)
    /// - **16 bpp**: use most significant byte
    /// - **32 bpp**: convert to luminance using perceptual weights (0.3R + 0.5G + 0.2B)
    ///
    /// C equivalent: `pixConvertTo8(pixs, 0)` (without colormap)
    ///
    /// # Returns
    ///
    /// A new 8-bit grayscale `Pix`.
    ///
    /// # Errors
    ///
    /// Returns an error if the image has a colormap (colormap removal is not
    /// yet implemented).
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// // Convert a 32-bit RGB image to 8-bit grayscale
    /// let pix32 = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    /// let pix8 = pix32.convert_to_8().unwrap();
    /// assert_eq!(pix8.depth(), PixelDepth::Bit8);
    /// assert_eq!(pix8.width(), 10);
    /// assert_eq!(pix8.height(), 10);
    /// ```
    pub fn convert_to_8(&self) -> Result<Pix> {
        if self.has_colormap() {
            return Err(Error::NotSupported(
                "convert_to_8 with colormap not yet implemented".to_string(),
            ));
        }

        let w = self.width();
        let h = self.height();

        match self.depth() {
            PixelDepth::Bit8 => {
                // Already 8-bit: return a deep copy
                Ok(self.deep_clone())
            }
            PixelDepth::Bit1 => {
                // 1-bit: 0 -> 255 (white), 1 -> 0 (black)
                let result = Pix::new(w, h, PixelDepth::Bit8)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        let gray = if val == 0 { 255u32 } else { 0u32 };
                        result_mut.set_pixel_unchecked(x, y, gray);
                    }
                }
                Ok(result_mut.into())
            }
            PixelDepth::Bit2 => {
                // 2-bit: 0->0, 1->85, 2->170, 3->255
                let result = Pix::new(w, h, PixelDepth::Bit8)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        let gray = match val {
                            0 => 0u32,
                            1 => 85,
                            2 => 170,
                            _ => 255,
                        };
                        result_mut.set_pixel_unchecked(x, y, gray);
                    }
                }
                Ok(result_mut.into())
            }
            PixelDepth::Bit4 => {
                // 4-bit: linear mapping 0..15 -> 0..255
                // Each step: 255/15 = 17
                let result = Pix::new(w, h, PixelDepth::Bit8)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        let gray = val * 255 / 15;
                        result_mut.set_pixel_unchecked(x, y, gray);
                    }
                }
                Ok(result_mut.into())
            }
            PixelDepth::Bit16 => {
                // 16-bit: use most significant byte
                let result = Pix::new(w, h, PixelDepth::Bit8)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        let gray = val >> 8; // most significant byte
                        result_mut.set_pixel_unchecked(x, y, gray);
                    }
                }
                Ok(result_mut.into())
            }
            PixelDepth::Bit32 => {
                // 32-bit RGB: convert to luminance
                // val = (int)(rwt * R + gwt * G + bwt * B + 0.5)
                let result = Pix::new(w, h, PixelDepth::Bit8)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let pixel = self.get_pixel_unchecked(x, y);
                        let r = color::red(pixel) as f32;
                        let g = color::green(pixel) as f32;
                        let b = color::blue(pixel) as f32;
                        let gray = (L_RED_WEIGHT * r + L_GREEN_WEIGHT * g + L_BLUE_WEIGHT * b + 0.5)
                            as u32;
                        result_mut.set_pixel_unchecked(x, y, gray.min(255));
                    }
                }
                Ok(result_mut.into())
            }
        }
    }
    /// Convert any-depth image to 32-bit RGB.
    ///
    /// This is a top-level conversion function with simple default values.
    /// It always creates a new image (never a clone).
    ///
    /// Conversion rules:
    /// - **1 bpp**: 0 -> white (R=G=B=255), 1 -> black (R=G=B=0)
    /// - **2 bpp with colormap**: expand using colormap colors
    /// - **2 bpp without colormap**: map 0->0, 1->85, 2->170, 3->255, then pack as R=G=B
    /// - **4 bpp with colormap**: expand using colormap colors
    /// - **4 bpp without colormap**: map linearly (val * 255 / 15), then pack as R=G=B
    /// - **8 bpp with colormap**: expand using colormap colors
    /// - **8 bpp without colormap**: pack gray value as R=G=B
    /// - **16 bpp**: use MSB as gray, pack as R=G=B
    /// - **32 bpp**: return deep clone (identity)
    ///
    /// C equivalent: `pixConvertTo32(pixs)`
    ///
    /// # Returns
    ///
    /// A new 32-bit RGB `Pix`.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth, color};
    ///
    /// // Convert an 8-bit grayscale image to 32-bit RGB
    /// let pix8 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    /// let pix32 = pix8.convert_to_32().unwrap();
    /// assert_eq!(pix32.depth(), PixelDepth::Bit32);
    /// assert_eq!(pix32.width(), 10);
    /// assert_eq!(pix32.height(), 10);
    /// ```
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth, color};
    ///
    /// // Convert a colormapped 8-bit image to 32-bit RGB
    /// use leptonica_core::PixColormap;
    /// use leptonica_core::colormap::RgbaQuad;
    ///
    /// let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
    /// let mut pix_mut = pix.try_into_mut().unwrap();
    /// let mut cmap = PixColormap::new(8).unwrap();
    /// cmap.add_color(RgbaQuad::rgb(255, 0, 0)).unwrap();   // index 0 = red
    /// cmap.add_color(RgbaQuad::rgb(0, 255, 0)).unwrap();   // index 1 = green
    /// pix_mut.set_pixel_unchecked(0, 0, 0); // red
    /// pix_mut.set_pixel_unchecked(1, 0, 1); // green
    /// pix_mut.set_pixel_unchecked(2, 0, 0); // red
    /// pix_mut.set_pixel_unchecked(3, 0, 1); // green
    /// pix_mut.set_colormap(Some(cmap)).unwrap();
    /// let pix: Pix = pix_mut.into();
    ///
    /// let pix32 = pix.convert_to_32().unwrap();
    /// assert_eq!(pix32.depth(), PixelDepth::Bit32);
    /// let (r, g, b) = color::extract_rgb(pix32.get_pixel_unchecked(0, 0));
    /// assert_eq!((r, g, b), (255, 0, 0));
    /// let (r, g, b) = color::extract_rgb(pix32.get_pixel_unchecked(1, 0));
    /// assert_eq!((r, g, b), (0, 255, 0));
    /// ```
    pub fn convert_to_32(&self) -> Result<Pix> {
        let w = self.width();
        let h = self.height();

        // Handle colormapped images: expand colormap to RGB
        if self.has_colormap() {
            let cmap = self.colormap().unwrap();
            let result = Pix::new(w, h, PixelDepth::Bit32)?;
            let mut result_mut = result.try_into_mut().unwrap();
            for y in 0..h {
                for x in 0..w {
                    let idx = self.get_pixel_unchecked(x, y) as usize;
                    let (r, g, b) = cmap.get_rgb(idx).unwrap_or((0, 0, 0));
                    let pixel = color::compose_rgb(r, g, b);
                    result_mut.set_pixel_unchecked(x, y, pixel);
                }
            }
            return Ok(result_mut.into());
        }

        match self.depth() {
            PixelDepth::Bit32 => {
                // Already 32-bit: return a deep copy
                Ok(self.deep_clone())
            }
            PixelDepth::Bit1 => {
                // 1-bit: 0 -> white (255,255,255), 1 -> black (0,0,0)
                let white = color::compose_rgb(255, 255, 255);
                let black = color::compose_rgb(0, 0, 0);
                let result = Pix::new(w, h, PixelDepth::Bit32)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        let pixel = if val == 0 { white } else { black };
                        result_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
                Ok(result_mut.into())
            }
            PixelDepth::Bit2 => {
                // 2-bit: 0->0, 1->85, 2->170, 3->255, then R=G=B
                let result = Pix::new(w, h, PixelDepth::Bit32)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        let gray = match val {
                            0 => 0u8,
                            1 => 85,
                            2 => 170,
                            _ => 255,
                        };
                        let pixel = color::compose_rgb(gray, gray, gray);
                        result_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
                Ok(result_mut.into())
            }
            PixelDepth::Bit4 => {
                // 4-bit: linear mapping 0..15 -> 0..255, then R=G=B
                let result = Pix::new(w, h, PixelDepth::Bit32)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        let gray = (val * 255 / 15) as u8;
                        let pixel = color::compose_rgb(gray, gray, gray);
                        result_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
                Ok(result_mut.into())
            }
            PixelDepth::Bit8 => {
                // 8-bit gray: replicate into R=G=B
                let result = Pix::new(w, h, PixelDepth::Bit32)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let gray = self.get_pixel_unchecked(x, y) as u8;
                        let pixel = color::compose_rgb(gray, gray, gray);
                        result_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
                Ok(result_mut.into())
            }
            PixelDepth::Bit16 => {
                // 16-bit: use most significant byte as gray, then R=G=B
                let result = Pix::new(w, h, PixelDepth::Bit32)?;
                let mut result_mut = result.try_into_mut().unwrap();
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        let gray = (val >> 8) as u8;
                        let pixel = color::compose_rgb(gray, gray, gray);
                        result_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
                Ok(result_mut.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // convert_to_8 tests
    // =========================================================================

    #[test]
    fn test_convert_8bit_identity() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = pix.convert_to_8().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 10);
    }

    #[test]
    fn test_convert_1bit_to_8() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let result = pix.convert_to_8().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        // All-zero 1-bit image -> all 255
        assert_eq!(result.get_pixel(0, 0), Some(255));
    }

    #[test]
    fn test_convert_32bit_to_8() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Set a known RGB value
        pix_mut.set_rgb(0, 0, 100, 100, 100).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_8().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        // 0.3*100 + 0.5*100 + 0.2*100 + 0.5 = 100.5 -> 100
        assert_eq!(result.get_pixel(0, 0), Some(100));
    }

    #[test]
    fn test_convert_16bit_to_8() {
        let pix = Pix::new(5, 5, PixelDepth::Bit16).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 0xAB00);
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_8().unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(0xAB));
    }

    // =========================================================================
    // convert_to_32 tests
    // =========================================================================

    #[test]
    fn test_convert_32bit_identity() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_rgb(0, 0, 100, 150, 200).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 10);
        // Should be a deep copy, preserving pixel values
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (100, 150, 200));
    }

    #[test]
    fn test_convert_1bit_to_32() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        // All-zero 1-bit image -> all white
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_convert_1bit_to_32_black() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 1); // set bit -> black
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_32().unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn test_convert_2bit_to_32() {
        let pix = Pix::new(4, 1, PixelDepth::Bit2).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 0);
        pix_mut.set_pixel_unchecked(1, 0, 1);
        pix_mut.set_pixel_unchecked(2, 0, 2);
        pix_mut.set_pixel_unchecked(3, 0, 3);
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);

        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (0, 0, 0));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (85, 85, 85));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(2, 0));
        assert_eq!((r, g, b), (170, 170, 170));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(3, 0));
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_convert_4bit_to_32() {
        let pix = Pix::new(2, 1, PixelDepth::Bit4).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 0); // gray = 0
        pix_mut.set_pixel_unchecked(1, 0, 15); // gray = 255
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);

        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (0, 0, 0));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_convert_8bit_to_32() {
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 0);
        pix_mut.set_pixel_unchecked(1, 0, 128);
        pix_mut.set_pixel_unchecked(2, 0, 255);
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);

        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (0, 0, 0));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (128, 128, 128));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(2, 0));
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_convert_16bit_to_32() {
        let pix = Pix::new(2, 1, PixelDepth::Bit16).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 0xAB00); // MSB = 0xAB
        pix_mut.set_pixel_unchecked(1, 0, 0xFF00); // MSB = 0xFF
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);

        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (0xAB, 0xAB, 0xAB));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (0xFF, 0xFF, 0xFF));
    }

    #[test]
    fn test_convert_8bit_colormap_to_32() {
        use crate::colormap::{PixColormap, RgbaQuad};

        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_color(RgbaQuad::rgb(255, 0, 0)).unwrap(); // index 0: red
        cmap.add_color(RgbaQuad::rgb(0, 255, 0)).unwrap(); // index 1: green
        cmap.add_color(RgbaQuad::rgb(0, 0, 255)).unwrap(); // index 2: blue

        pix_mut.set_pixel_unchecked(0, 0, 0); // red
        pix_mut.set_pixel_unchecked(1, 0, 1); // green
        pix_mut.set_pixel_unchecked(2, 0, 2); // blue
        pix_mut.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert!(!result.has_colormap());

        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (255, 0, 0));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (0, 255, 0));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(2, 0));
        assert_eq!((r, g, b), (0, 0, 255));
    }

    #[test]
    fn test_convert_4bit_colormap_to_32() {
        use crate::colormap::{PixColormap, RgbaQuad};

        let pix = Pix::new(2, 1, PixelDepth::Bit4).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let mut cmap = PixColormap::new(4).unwrap();
        cmap.add_color(RgbaQuad::rgb(10, 20, 30)).unwrap(); // index 0
        cmap.add_color(RgbaQuad::rgb(40, 50, 60)).unwrap(); // index 1

        pix_mut.set_pixel_unchecked(0, 0, 0);
        pix_mut.set_pixel_unchecked(1, 0, 1);
        pix_mut.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);

        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (10, 20, 30));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (40, 50, 60));
    }

    #[test]
    fn test_convert_to_32_preserves_dimensions() {
        for &depth in &[
            PixelDepth::Bit1,
            PixelDepth::Bit2,
            PixelDepth::Bit4,
            PixelDepth::Bit8,
            PixelDepth::Bit16,
            PixelDepth::Bit32,
        ] {
            let pix = Pix::new(17, 13, depth).unwrap();
            let result = pix.convert_to_32().unwrap();
            assert_eq!(result.depth(), PixelDepth::Bit32);
            assert_eq!(result.width(), 17, "width mismatch for {:?}", depth);
            assert_eq!(result.height(), 13, "height mismatch for {:?}", depth);
        }
    }
}
