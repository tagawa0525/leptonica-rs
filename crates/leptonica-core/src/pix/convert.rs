//! Pixel depth conversion functions
//!
//! Functions for converting between different pixel depths.
//!
//! # See also
//!
//! C Leptonica: `pixconv.c` (`pixConvertTo8`, `pixConvertTo32`, etc.)

use super::{Pix, PixelDepth};
use crate::color;
use crate::error::{Error, Result};

/// Default perceptual weights for RGB-to-gray conversion.
///
/// These match C Leptonica's `L_RED_WEIGHT`, `L_GREEN_WEIGHT`, `L_BLUE_WEIGHT`.
const L_RED_WEIGHT: f32 = 0.3;
const L_GREEN_WEIGHT: f32 = 0.5;
const L_BLUE_WEIGHT: f32 = 0.2;

/// Default neutral boost reference value for min/max boost conversions.
const DEFAULT_NEUTRAL_BOOST_VAL: i32 = 180;

/// Selection type for min/max gray conversion.
///
/// # See also
///
/// C Leptonica: `L_CHOOSE_MIN`, `L_CHOOSE_MAX`, etc. in `pix.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinMaxType {
    /// Choose minimum of R, G, B
    Min,
    /// Choose maximum of R, G, B
    Max,
    /// Choose max - min (chroma)
    MaxDiff,
    /// Min boosted around neutral point
    MinBoost,
    /// Max boosted around neutral point
    MaxBoost,
}

/// Color selection type for general RGB-to-gray conversion.
///
/// # See also
///
/// C Leptonica: `L_SELECT_RED`, `L_SELECT_GREEN`, etc. in `pix.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrayConversionType {
    /// Use red channel only
    Red,
    /// Use green channel only
    Green,
    /// Use blue channel only
    Blue,
    /// Use minimum of R, G, B
    Min,
    /// Use maximum of R, G, B
    Max,
    /// Use average (equal weights)
    Average,
    /// Use custom weights
    Weighted,
}

/// Target type for colormap removal.
///
/// # See also
///
/// C Leptonica: `REMOVE_CMAP_TO_BINARY`, `REMOVE_CMAP_TO_GRAYSCALE`, etc. in `pix.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveColormapTarget {
    /// Convert to 1 bpp binary (only for 1 bpp images)
    ToBinary,
    /// Convert to 8 bpp grayscale
    ToGrayscale,
    /// Convert to 32 bpp RGB (spp=3)
    ToFullColor,
    /// Convert to 32 bpp RGBA (spp=4)
    WithAlpha,
    /// Auto-detect best target based on colormap content
    BasedOnSrc,
}

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
    /// # See also
    ///
    /// C Leptonica: `pixConvertTo8()` in `pixconv.c`
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix32 = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    /// let pix8 = pix32.convert_to_8().unwrap();
    /// assert_eq!(pix8.depth(), PixelDepth::Bit8);
    /// ```
    pub fn convert_to_8(&self) -> Result<Pix> {
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
    /// Conversion rules:
    /// - **1 bpp**: 0 -> white (R=G=B=255), 1 -> black (R=G=B=0)
    /// - **2 bpp**: map 0->0, 1->85, 2->170, 3->255, then pack as R=G=B
    /// - **4 bpp**: map linearly (val * 255 / 15), then pack as R=G=B
    /// - **8 bpp**: pack gray value as R=G=B
    /// - **16 bpp**: use MSB as gray, pack as R=G=B
    /// - **32 bpp**: return deep clone (identity)
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertTo32()` in `pixconv.c`
    pub fn convert_to_32(&self) -> Result<Pix> {
        let w = self.width();
        let h = self.height();

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

    /// Convert 32 bpp RGB to 8 bpp grayscale using standard luminance weights.
    ///
    /// Uses the default perceptual weights: 0.3R + 0.5G + 0.2B.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertRGBToLuminance()` in `pixconv.c`
    pub fn convert_rgb_to_luminance(&self) -> Result<Pix> {
        self.convert_rgb_to_gray(0.0, 0.0, 0.0)
    }

    /// Convert 32 bpp RGB to 8 bpp grayscale with custom weights.
    ///
    /// If all weights are 0.0, default perceptual weights are used.
    /// Weights are normalized to sum to 1.0 if they don't already.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedDepth`] if the image is not 32 bpp.
    /// Returns [`Error::InvalidParameter`] if any weight is negative.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertRGBToGray()` in `pixconv.c`
    pub fn convert_rgb_to_gray(&self, rwt: f32, gwt: f32, bwt: f32) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if rwt < 0.0 || gwt < 0.0 || bwt < 0.0 {
            return Err(Error::InvalidParameter("weights must all be >= 0.0".into()));
        }

        let (rwt, gwt, bwt) = if rwt == 0.0 && gwt == 0.0 && bwt == 0.0 {
            (L_RED_WEIGHT, L_GREEN_WEIGHT, L_BLUE_WEIGHT)
        } else {
            let sum = rwt + gwt + bwt;
            if (sum - 1.0).abs() > 0.0001 {
                (rwt / sum, gwt / sum, bwt / sum)
            } else {
                (rwt, gwt, bwt)
            }
        };

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let r = color::red(pixel) as f32;
                let g = color::green(pixel) as f32;
                let b = color::blue(pixel) as f32;
                let gray = (rwt * r + gwt * g + bwt * b + 0.5) as u32;
                result_mut.set_pixel_unchecked(x, y, gray.min(255));
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 32 bpp RGB to 8 bpp grayscale using the green channel.
    ///
    /// This is the fastest RGB-to-gray conversion, extracting only the
    /// green channel as a reasonable approximation of luminance.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertRGBToGrayFast()` in `pixconv.c`
    pub fn convert_rgb_to_gray_fast(&self) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let g = color::green(pixel) as u32;
                result_mut.set_pixel_unchecked(x, y, g);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 32 bpp RGB to 8 bpp grayscale using min/max channel selection.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertRGBToGrayMinMax()` in `pixconv.c`
    pub fn convert_rgb_to_gray_min_max(&self, mm_type: MinMaxType) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let (r, g, b) = color::extract_rgb(pixel);
                let (r, g, b) = (r as i32, g as i32, b as i32);
                let min_val = r.min(g).min(b);
                let max_val = r.max(g).max(b);

                let val = match mm_type {
                    MinMaxType::Min => min_val,
                    MinMaxType::Max => max_val,
                    MinMaxType::MaxDiff => max_val - min_val,
                    MinMaxType::MinBoost => {
                        (min_val * min_val / DEFAULT_NEUTRAL_BOOST_VAL).min(255)
                    }
                    MinMaxType::MaxBoost => {
                        (max_val * max_val / DEFAULT_NEUTRAL_BOOST_VAL).min(255)
                    }
                };

                result_mut.set_pixel_unchecked(x, y, val as u32);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 32 bpp RGB to 8 bpp grayscale with saturation boost.
    ///
    /// Returns the max component value, boosted by the saturation.
    /// The maximum boost occurs where the max component value equals `refval`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if `refval` is not in `[1, 255]`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertRGBToGraySatBoost()` in `pixconv.c`
    pub fn convert_rgb_to_gray_sat_boost(&self, refval: i32) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if !(1..=255).contains(&refval) {
            return Err(Error::InvalidParameter("refval must be in [1, 255]".into()));
        }

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        // Pre-compute lookup tables
        let mut invmax = [0.0f32; 256];
        let mut ratio = [0.0f32; 256];
        for i in 1..256 {
            invmax[i] = 1.0 / i as f32;
            ratio[i] = i as f32 / refval as f32;
        }

        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let (r, g, b) = color::extract_rgb(pixel);
                let (r, g, b) = (r as i32, g as i32, b as i32);
                let min_val = r.min(g).min(b);
                let max_val = r.max(g).max(b);
                let delta = max_val - min_val;

                let sval = if delta == 0 {
                    0
                } else {
                    (255.0 * delta as f32 * invmax[max_val as usize] + 0.5) as i32
                };

                let fullsat = (255.0 * ratio[max_val as usize]).min(255.0) as i32;
                let newval = (sval * fullsat + (255 - sval) * max_val) / 255;

                result_mut.set_pixel_unchecked(x, y, newval as u32);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 32 bpp RGB to 8 bpp grayscale with selectable method.
    ///
    /// Dispatches to the appropriate conversion function based on `conv_type`.
    /// The weights `rwt`, `gwt`, `bwt` are only used when `conv_type` is
    /// [`GrayConversionType::Weighted`].
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertRGBToGrayGeneral()` in `pixconv.c`
    pub fn convert_rgb_to_gray_general(
        &self,
        conv_type: GrayConversionType,
        rwt: f32,
        gwt: f32,
        bwt: f32,
    ) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        match conv_type {
            GrayConversionType::Red => self.convert_rgb_to_gray(1.0, 0.0, 0.0),
            GrayConversionType::Green => self.convert_rgb_to_gray(0.0, 1.0, 0.0),
            GrayConversionType::Blue => self.convert_rgb_to_gray(0.0, 0.0, 1.0),
            GrayConversionType::Min => self.convert_rgb_to_gray_min_max(MinMaxType::Min),
            GrayConversionType::Max => self.convert_rgb_to_gray_min_max(MinMaxType::Max),
            GrayConversionType::Average => self.convert_rgb_to_gray(0.34, 0.33, 0.33),
            GrayConversionType::Weighted => self.convert_rgb_to_gray(rwt, gwt, bwt),
        }
    }

    /// Remove colormap and convert to specified target format.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixRemoveColormap()` in `pixconv.c`
    pub fn remove_colormap(&self, _target: RemoveColormapTarget) -> Result<Pix> {
        todo!()
    }

    /// Add a full 256-entry grayscale colormap to an 8 bpp image.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddGrayColormap8()` in `pixconv.c`
    pub fn add_gray_colormap_8(&self) -> Result<Pix> {
        todo!()
    }

    /// Add a minimal grayscale colormap containing only used gray values.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddMinimalGrayColormap8()` in `pixconv.c`
    pub fn add_minimal_gray_colormap_8(&self) -> Result<Pix> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_convert_32bit_identity() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_rgb(0, 0, 100, 150, 200).unwrap();
        let pix: Pix = pix_mut.into();

        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (100, 150, 200));
    }

    #[test]
    fn test_convert_1bit_to_32() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let result = pix.convert_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
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

    // ---- RGB→Gray conversion tests (Phase 1.1) ----

    #[test]
    fn test_convert_rgb_to_luminance() {
        let pix = Pix::new(3, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 100, 100, 100).unwrap();
        pm.set_rgb(1, 0, 255, 0, 0).unwrap();
        pm.set_rgb(2, 0, 0, 255, 0).unwrap();
        let pix: Pix = pm.into();

        let gray = pix.convert_rgb_to_luminance().unwrap();
        assert_eq!(gray.depth(), PixelDepth::Bit8);
        assert_eq!(gray.width(), 3);
        // (0.3*100 + 0.5*100 + 0.2*100 + 0.5) = 100
        assert_eq!(gray.get_pixel(0, 0), Some(100));
        // (0.3*255 + 0.5*0 + 0.2*0 + 0.5) = 77
        assert_eq!(gray.get_pixel(1, 0), Some(77));
        // (0.3*0 + 0.5*255 + 0.2*0 + 0.5) = 128
        assert_eq!(gray.get_pixel(2, 0), Some(128));
    }

    #[test]
    fn test_convert_rgb_to_gray_custom_weights() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 200, 100, 50).unwrap();
        let pix: Pix = pm.into();

        // Equal weights: (200+100+50)/3 = 116.67 -> 117
        let gray = pix
            .convert_rgb_to_gray(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
            .unwrap();
        assert_eq!(gray.get_pixel(0, 0), Some(117));
    }

    #[test]
    fn test_convert_rgb_to_gray_default_weights() {
        // When all weights are 0.0, use default L_RED/GREEN/BLUE_WEIGHT
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 100, 100, 100).unwrap();
        let pix: Pix = pm.into();

        let gray = pix.convert_rgb_to_gray(0.0, 0.0, 0.0).unwrap();
        assert_eq!(gray.get_pixel(0, 0), Some(100));
    }

    #[test]
    fn test_convert_rgb_to_gray_rejects_non_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_rgb_to_gray(0.0, 0.0, 0.0).is_err());
    }

    #[test]
    fn test_convert_rgb_to_gray_fast() {
        let pix = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 100, 150, 200).unwrap();
        pm.set_rgb(1, 0, 0, 255, 0).unwrap();
        let pix: Pix = pm.into();

        let gray = pix.convert_rgb_to_gray_fast().unwrap();
        assert_eq!(gray.depth(), PixelDepth::Bit8);
        // Fast uses green channel only
        assert_eq!(gray.get_pixel(0, 0), Some(150));
        assert_eq!(gray.get_pixel(1, 0), Some(255));
    }

    #[test]
    fn test_convert_rgb_to_gray_min_max() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 50, 100, 200).unwrap();
        let pix: Pix = pm.into();

        let min_gray = pix.convert_rgb_to_gray_min_max(MinMaxType::Min).unwrap();
        assert_eq!(min_gray.get_pixel(0, 0), Some(50));

        let max_gray = pix.convert_rgb_to_gray_min_max(MinMaxType::Max).unwrap();
        assert_eq!(max_gray.get_pixel(0, 0), Some(200));

        let diff_gray = pix
            .convert_rgb_to_gray_min_max(MinMaxType::MaxDiff)
            .unwrap();
        assert_eq!(diff_gray.get_pixel(0, 0), Some(150)); // 200 - 50

        // MinBoost: min(255, (50*50)/180) = min(255, 13) = 13
        let min_boost = pix
            .convert_rgb_to_gray_min_max(MinMaxType::MinBoost)
            .unwrap();
        assert_eq!(min_boost.get_pixel(0, 0), Some(13));

        // MaxBoost: min(255, (200*200)/180) = min(255, 222) = 222
        let max_boost = pix
            .convert_rgb_to_gray_min_max(MinMaxType::MaxBoost)
            .unwrap();
        assert_eq!(max_boost.get_pixel(0, 0), Some(222));
    }

    #[test]
    fn test_convert_rgb_to_gray_sat_boost() {
        // Gray pixel: saturation=0, returns intensity
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 128, 128, 128).unwrap();
        let pix: Pix = pm.into();

        let gray = pix.convert_rgb_to_gray_sat_boost(100).unwrap();
        assert_eq!(gray.get_pixel(0, 0), Some(128));
    }

    #[test]
    fn test_convert_rgb_to_gray_sat_boost_invalid_refval() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        assert!(pix.convert_rgb_to_gray_sat_boost(0).is_err());
        assert!(pix.convert_rgb_to_gray_sat_boost(256).is_err());
    }

    #[test]
    fn test_convert_rgb_to_gray_general_red() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 200, 100, 50).unwrap();
        let pix: Pix = pm.into();

        let gray = pix
            .convert_rgb_to_gray_general(GrayConversionType::Red, 0.0, 0.0, 0.0)
            .unwrap();
        assert_eq!(gray.get_pixel(0, 0), Some(200));
    }

    #[test]
    fn test_convert_rgb_to_gray_general_average() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 90, 120, 90).unwrap();
        let pix: Pix = pm.into();

        let gray = pix
            .convert_rgb_to_gray_general(GrayConversionType::Average, 0.0, 0.0, 0.0)
            .unwrap();
        // Average uses (0.34, 0.33, 0.33): 0.34*90 + 0.33*120 + 0.33*90 + 0.5 = 100.1 -> 100
        assert_eq!(gray.get_pixel(0, 0), Some(100));
    }

    #[test]
    fn test_convert_rgb_to_gray_preserves_resolution() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_resolution(300, 300);
        let pix: Pix = pm.into();

        let gray = pix.convert_rgb_to_luminance().unwrap();
        assert_eq!(gray.xres(), 300);
        assert_eq!(gray.yres(), 300);
    }

    // ---- Colormap removal tests (Phase 1.3) ----

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_colormap_no_colormap() {
        use super::RemoveColormapTarget;
        // Image without colormap should return deep clone
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = pix
            .remove_colormap(RemoveColormapTarget::BasedOnSrc)
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 10);
        assert!(!result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_colormap_to_binary() {
        use super::RemoveColormapTarget;
        use crate::PixColormap;
        // 1 bpp with black/white colormap
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let mut cmap = PixColormap::new(1).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap(); // 0 -> black
        cmap.add_rgb(255, 255, 255).unwrap(); // 1 -> white
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();

        let result = pix.remove_colormap(RemoveColormapTarget::ToBinary).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert!(!result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_colormap_to_binary_inverted() {
        use super::RemoveColormapTarget;
        use crate::PixColormap;
        // 1 bpp with inverted colormap (0 -> white, 1 -> black)
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let mut cmap = PixColormap::new(1).unwrap();
        cmap.add_rgb(255, 255, 255).unwrap(); // 0 -> white
        cmap.add_rgb(0, 0, 0).unwrap(); // 1 -> black
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();

        let result = pix.remove_colormap(RemoveColormapTarget::ToBinary).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert!(!result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_colormap_to_grayscale() {
        use super::RemoveColormapTarget;
        use crate::PixColormap;
        // 4 bpp with grayscale colormap
        let pix = Pix::new(10, 10, PixelDepth::Bit4).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let cmap = PixColormap::create_linear(4, true).unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();

        let result = pix
            .remove_colormap(RemoveColormapTarget::ToGrayscale)
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(!result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_colormap_to_full_color() {
        use super::RemoveColormapTarget;
        use crate::PixColormap;
        // 8 bpp with color colormap
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap(); // Red
        cmap.add_rgb(0, 255, 0).unwrap(); // Green
        cmap.add_rgb(0, 0, 255).unwrap(); // Blue
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();

        let result = pix
            .remove_colormap(RemoveColormapTarget::ToFullColor)
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.spp(), 3); // RGB, no alpha
        assert!(!result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_colormap_with_alpha() {
        use super::RemoveColormapTarget;
        use crate::PixColormap;
        // 8 bpp with colormap that has alpha
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(255, 0, 0, 128).unwrap(); // Semi-transparent red
        cmap.add_rgba(0, 255, 0, 255).unwrap(); // Opaque green
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();

        let result = pix
            .remove_colormap(RemoveColormapTarget::WithAlpha)
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.spp(), 4); // RGBA
        assert!(!result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_colormap_based_on_src_grayscale() {
        use super::RemoveColormapTarget;
        use crate::PixColormap;
        // 8 bpp with grayscale colormap -> should convert to 8 bpp gray
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let cmap = PixColormap::create_linear(8, true).unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();

        let result = pix
            .remove_colormap(RemoveColormapTarget::BasedOnSrc)
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(!result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_colormap_based_on_src_color() {
        use super::RemoveColormapTarget;
        use crate::PixColormap;
        // 8 bpp with color colormap -> should convert to 32 bpp
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        cmap.add_rgb(0, 255, 0).unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();

        let result = pix
            .remove_colormap(RemoveColormapTarget::BasedOnSrc)
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert!(!result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_gray_colormap_8() {
        // Add linear grayscale colormap to 8 bpp image
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = pix.add_gray_colormap_8().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.has_colormap());
        let cmap = result.colormap().unwrap();
        assert_eq!(cmap.len(), 256);
        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        assert_eq!(cmap.get_rgb(255), Some((255, 255, 255)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_gray_colormap_8_already_has_colormap() {
        use crate::PixColormap;
        // If already has colormap, return deep clone
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let cmap = PixColormap::create_linear(8, true).unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();

        let result = pix.add_gray_colormap_8().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_gray_colormap_8_invalid_depth() {
        // Only works for 8 bpp
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.add_gray_colormap_8().is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_minimal_gray_colormap_8() {
        // Create 8 bpp image with only a few gray levels
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Set some pixels to specific gray values
        pm.set_pixel_unchecked(0, 0, 50);
        pm.set_pixel_unchecked(1, 0, 100);
        pm.set_pixel_unchecked(2, 0, 150);
        pm.set_pixel_unchecked(3, 0, 50); // Duplicate
        let pix: Pix = pm.into();

        let result = pix.add_minimal_gray_colormap_8().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.has_colormap());
        let cmap = result.colormap().unwrap();
        // Should have 4 entries: 0 (background), 50, 100, 150
        assert_eq!(cmap.len(), 4);
    }
}
