//! Pixel depth conversion functions
//!
//! Functions for converting between different pixel depths.
//!
//! # See also
//!
//! C Leptonica: `pixconv.c` (`pixConvertTo8`, `pixConvertTo32`, etc.)

use super::{Pix, PixelDepth};
use crate::color;
use crate::error::Result;

/// Default perceptual weights for RGB-to-gray conversion.
///
/// These match C Leptonica's `L_RED_WEIGHT`, `L_GREEN_WEIGHT`, `L_BLUE_WEIGHT`.
const L_RED_WEIGHT: f32 = 0.3;
const L_GREEN_WEIGHT: f32 = 0.5;
const L_BLUE_WEIGHT: f32 = 0.2;

/// Default neutral boost reference value for min/max boost conversions.
#[allow(dead_code)]
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
    pub fn convert_rgb_to_gray(&self, _rwt: f32, _gwt: f32, _bwt: f32) -> Result<Pix> {
        todo!("convert_rgb_to_gray not yet implemented")
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
        todo!("convert_rgb_to_gray_fast not yet implemented")
    }

    /// Convert 32 bpp RGB to 8 bpp grayscale using min/max channel selection.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertRGBToGrayMinMax()` in `pixconv.c`
    pub fn convert_rgb_to_gray_min_max(&self, _mm_type: MinMaxType) -> Result<Pix> {
        todo!("convert_rgb_to_gray_min_max not yet implemented")
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
    pub fn convert_rgb_to_gray_sat_boost(&self, _refval: i32) -> Result<Pix> {
        todo!("convert_rgb_to_gray_sat_boost not yet implemented")
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
        _conv_type: GrayConversionType,
        _rwt: f32,
        _gwt: f32,
        _bwt: f32,
    ) -> Result<Pix> {
        todo!("convert_rgb_to_gray_general not yet implemented")
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_convert_rgb_to_gray_rejects_non_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_rgb_to_gray(0.0, 0.0, 0.0).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_convert_rgb_to_gray_sat_boost_invalid_refval() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        assert!(pix.convert_rgb_to_gray_sat_boost(0).is_err());
        assert!(pix.convert_rgb_to_gray_sat_boost(256).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_convert_rgb_to_gray_preserves_resolution() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_resolution(300, 300);
        let pix: Pix = pm.into();

        let gray = pix.convert_rgb_to_luminance().unwrap();
        assert_eq!(gray.xres(), 300);
        assert_eq!(gray.yres(), 300);
    }
}
