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

/// Conversion type for 16 bpp to 8 bpp conversion.
///
/// # See also
///
/// C Leptonica: `L_LS_BYTE`, `L_MS_BYTE`, etc. in `pix.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Convert16To8Type {
    /// Use least significant byte
    LsByte,
    /// Use most significant byte
    MsByte,
    /// Use LSB if max(val) < 256; else MSB
    AutoByte,
    /// Saturate to 255: min(val, 0xFF)
    ClipToFf,
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
    pub fn remove_colormap(&self, target: RemoveColormapTarget) -> Result<Pix> {
        // If no colormap, return deep clone
        let Some(cmap) = self.colormap() else {
            return Ok(self.deep_clone());
        };

        let w = self.width();
        let h = self.height();
        let d = self.depth();

        // Validate depth
        if !matches!(
            d,
            PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8
        ) {
            return Err(Error::UnsupportedDepth(d.bits()));
        }

        // Determine actual target type
        let target = match target {
            RemoveColormapTarget::ToBinary if d != PixelDepth::Bit1 => {
                // Can't convert to binary if not 1 bpp
                RemoveColormapTarget::BasedOnSrc
            }
            RemoveColormapTarget::BasedOnSrc => {
                // Auto-detect based on colormap content
                if !cmap.is_opaque() {
                    RemoveColormapTarget::WithAlpha
                } else if cmap.has_color() {
                    RemoveColormapTarget::ToFullColor
                } else if d == PixelDepth::Bit1 && cmap.is_black_and_white() {
                    RemoveColormapTarget::ToBinary
                } else {
                    RemoveColormapTarget::ToGrayscale
                }
            }
            other => other,
        };

        match target {
            RemoveColormapTarget::ToBinary => {
                // Copy data and remove colormap, inverting if needed
                let result = self.deep_clone();
                let result_mut = result.try_into_mut().unwrap();

                // Check if we need to invert (if color 0 is darker than color 1)
                let (r0, g0, b0) = cmap.get_rgb(0).unwrap_or((0, 0, 0));
                let (r1, g1, b1) = cmap.get_rgb(1).unwrap_or((255, 255, 255));
                let val0 = r0 as u32 + g0 as u32 + b0 as u32;
                let val1 = r1 as u32 + g1 as u32 + b1 as u32;

                let mut result_mut = if val0 < val1 {
                    // Inverted: flip all bits
                    let pix: Pix = result_mut.into();
                    pix.invert().try_into_mut().unwrap()
                } else {
                    result_mut
                };

                result_mut.set_colormap(None)?;
                Ok(result_mut.into())
            }
            RemoveColormapTarget::ToGrayscale => {
                // Convert to 8 bpp grayscale using luminance
                let result = Pix::new(w, h, PixelDepth::Bit8)?;
                let mut result_mut = result.try_into_mut().unwrap();
                result_mut.set_resolution(self.xres(), self.yres());

                // Build grayscale LUT
                let max_entries = 1 << d.bits();
                let mut graymap = vec![0u8; max_entries];
                for (i, gray_val) in graymap.iter_mut().enumerate().take(cmap.len()) {
                    if let Some((r, g, b)) = cmap.get_rgb(i) {
                        *gray_val = (L_RED_WEIGHT * r as f32
                            + L_GREEN_WEIGHT * g as f32
                            + L_BLUE_WEIGHT * b as f32
                            + 0.5) as u8;
                    }
                }

                // Apply LUT
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y) as usize;
                        let gray = graymap[val];
                        result_mut.set_pixel_unchecked(x, y, gray as u32);
                    }
                }

                Ok(result_mut.into())
            }
            RemoveColormapTarget::ToFullColor | RemoveColormapTarget::WithAlpha => {
                // Convert to 32 bpp RGB or RGBA
                let result = Pix::new(w, h, PixelDepth::Bit32)?;
                let mut result_mut = result.try_into_mut().unwrap();
                result_mut.set_resolution(self.xres(), self.yres());

                if target == RemoveColormapTarget::WithAlpha {
                    result_mut.set_spp(4);
                }

                // Build color LUT
                let max_entries = 1 << d.bits();
                let mut lut = vec![0u32; max_entries];
                for (i, pixel_val) in lut.iter_mut().enumerate().take(cmap.len()) {
                    *pixel_val = if target == RemoveColormapTarget::ToFullColor {
                        if let Some((r, g, b)) = cmap.get_rgb(i) {
                            color::compose_rgb(r, g, b)
                        } else {
                            0
                        }
                    } else if let Some((r, g, b, a)) = cmap.get_rgba(i) {
                        color::compose_rgba(r, g, b, a)
                    } else {
                        0
                    };
                }

                // Apply LUT
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y) as usize;
                        let pixel = lut[val];
                        result_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }

                Ok(result_mut.into())
            }
            RemoveColormapTarget::BasedOnSrc => {
                unreachable!("BasedOnSrc should have been resolved earlier")
            }
        }
    }

    /// Add a full 256-entry grayscale colormap to an 8 bpp image.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddGrayColormap8()` in `pixconv.c`
    pub fn add_gray_colormap_8(&self) -> Result<Pix> {
        use crate::PixColormap;

        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        // If already has colormap, return deep clone
        if self.has_colormap() {
            return Ok(self.deep_clone());
        }

        // Create a copy with a linear grayscale colormap
        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();
        let cmap = PixColormap::create_linear(8, true)?;
        result_mut.set_colormap(Some(cmap))?;
        Ok(result_mut.into())
    }

    /// Add a minimal grayscale colormap containing only used gray values.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddMinimalGrayColormap8()` in `pixconv.c`
    pub fn add_minimal_gray_colormap_8(&self) -> Result<Pix> {
        use crate::PixColormap;

        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();

        // Find all unique gray levels
        let mut used = [false; 256];
        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y) as usize;
                used[val] = true;
            }
        }

        // Build colormap with only used values
        let mut cmap = PixColormap::new(8)?;
        let mut revmap = [0u8; 256];
        for (i, &is_used) in used.iter().enumerate() {
            if is_used {
                let idx = cmap.add_rgb(i as u8, i as u8, i as u8)?;
                revmap[i] = idx as u8;
            }
        }

        // Create new image with remapped pixel values
        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());
        result_mut.set_colormap(Some(cmap))?;

        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y) as usize;
                let new_val = revmap[val];
                result_mut.set_pixel_unchecked(x, y, new_val as u32);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert any 1 or 8 bpp image to 16 bpp.
    ///
    /// - **1 bpp**: 0 -> 0xffff (white), 1 -> 0 (black)
    /// - **8 bpp**: replicate value in both MSB and LSB (val | val << 8)
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertTo16()` in `pixconv.c`
    pub fn convert_to_16(&self) -> Result<Pix> {
        match self.depth() {
            PixelDepth::Bit1 => {
                let w = self.width();
                let h = self.height();
                let result = Pix::new(w, h, PixelDepth::Bit16)?;
                let mut result_mut = result.try_into_mut().unwrap();
                result_mut.set_resolution(self.xres(), self.yres());

                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        let val16 = if val == 0 { 0xffff } else { 0 };
                        result_mut.set_pixel_unchecked(x, y, val16);
                    }
                }
                Ok(result_mut.into())
            }
            PixelDepth::Bit8 => self.convert_8_to_16(8),
            _ => Err(Error::UnsupportedDepth(self.depth().bits())),
        }
    }

    /// Convert 8 bpp grayscale to 32 bpp RGB.
    ///
    /// Replicates gray into R=G=B channels. If colormap is present,
    /// removes it first via [`remove_colormap`].
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert8To32()` in `pixconv.c`
    pub fn convert_8_to_32(&self) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        // Handle colormap case
        if self.has_colormap() {
            return self.remove_colormap(RemoveColormapTarget::ToFullColor);
        }

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit32)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        // Build LUT: gray -> RGB (no alpha)
        let mut tab = [0u32; 256];
        for (i, entry) in tab.iter_mut().enumerate() {
            *entry = (i as u32) << 24 | (i as u32) << 16 | (i as u32) << 8;
        }

        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y) as usize;
                result_mut.set_pixel_unchecked(x, y, tab[val]);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 8 bpp grayscale to 16 bpp with configurable shift.
    ///
    /// - `left_shift == 8`: proportional mapping (val | val << 8)
    /// - `left_shift < 8`: simple left shift (val << left_shift)
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if `left_shift` is not in `[0, 8]`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert8To16()` in `pixconv.c`
    pub fn convert_8_to_16(&self, left_shift: u32) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if left_shift > 8 {
            return Err(Error::InvalidParameter(
                "left_shift must be in [0, 8]".into(),
            ));
        }

        // Remove colormap if present
        let src = if self.has_colormap() {
            self.remove_colormap(RemoveColormapTarget::ToGrayscale)?
        } else {
            self.deep_clone()
        };

        let w = src.width();
        let h = src.height();
        let result = Pix::new(w, h, PixelDepth::Bit16)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let val = src.get_pixel_unchecked(x, y);
                let val16 = if left_shift == 8 {
                    val | (val << 8)
                } else {
                    val << left_shift
                };
                result_mut.set_pixel_unchecked(x, y, val16);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 16 bpp to 8 bpp using the specified extraction strategy.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert16To8()` in `pixconv.c`
    pub fn convert_16_to_8(&self, conversion_type: Convert16To8Type) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit16 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        // For AutoByte, determine which byte to use based on max value
        let effective_type = match conversion_type {
            Convert16To8Type::AutoByte => {
                let w = self.width();
                let h = self.height();
                let mut max_val: u32 = 0;
                for y in 0..h {
                    for x in 0..w {
                        let val = self.get_pixel_unchecked(x, y);
                        if val > max_val {
                            max_val = val;
                        }
                    }
                }
                if max_val <= 255 {
                    Convert16To8Type::LsByte
                } else {
                    Convert16To8Type::MsByte
                }
            }
            other => other,
        };

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y);
                let val8 = match effective_type {
                    Convert16To8Type::LsByte => val & 0xff,
                    Convert16To8Type::MsByte => (val >> 8) & 0xff,
                    Convert16To8Type::ClipToFf => val.min(255),
                    Convert16To8Type::AutoByte => {
                        unreachable!("AutoByte resolved above")
                    }
                };
                result_mut.set_pixel_unchecked(x, y, val8);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert any depth to 8 or 32 bpp, removing colormap if present.
    ///
    /// Returns 8 bpp for grayscale content, 32 bpp for color content.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertTo8Or32()` in `pixconv.c`
    pub fn convert_to_8_or_32(&self) -> Result<Pix> {
        // Handle colormap: remove it based on source content
        if self.has_colormap() {
            return self.remove_colormap(RemoveColormapTarget::BasedOnSrc);
        }

        // Already 8 or 32 bpp: deep copy
        if self.depth() == PixelDepth::Bit8 || self.depth() == PixelDepth::Bit32 {
            return Ok(self.deep_clone());
        }

        // All other depths: convert to 8 bpp
        self.convert_to_8()
    }

    /// Lossless depth expansion from lower to higher depth.
    ///
    /// Only expands 1/2/4 bpp to a higher depth (2/4/8 bpp).
    /// No colormap is allowed on the source.
    ///
    /// # Errors
    ///
    /// Returns error if target depth is less than source depth,
    /// or if source has a colormap.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertLossless()` in `pixconv.c`
    pub fn convert_lossless(&self, target_depth: u32) -> Result<Pix> {
        let src_d = self.depth().bits();

        // Validate target depth
        let target = PixelDepth::from_bits(target_depth)?;
        if !matches!(
            target,
            PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8
        ) {
            return Err(Error::InvalidParameter(format!(
                "target depth must be 2, 4, or 8; got {}",
                target_depth
            )));
        }

        if target_depth < src_d {
            return Err(Error::InvalidParameter(format!(
                "target depth {} < source depth {}: lossy conversion not allowed",
                target_depth, src_d
            )));
        }

        if self.has_colormap() {
            return Err(Error::InvalidParameter(
                "source must not have a colormap".into(),
            ));
        }

        // Same depth: return copy
        if target_depth == src_d {
            return Ok(self.deep_clone());
        }

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, target)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y);
                result_mut.set_pixel_unchecked(x, y, val);
            }
        }

        Ok(result_mut.into())
    }

    /// Remove alpha channel by blending over white background.
    ///
    /// If the image is 32 bpp RGBA (spp=4), blends each pixel over white.
    /// Otherwise returns a deep clone unchanged.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixRemoveAlpha()` in `pixconv.c`
    pub fn remove_alpha(&self) -> Result<Pix> {
        // Only 32 bpp RGBA needs processing
        if self.depth() != PixelDepth::Bit32 || self.spp() != 4 {
            return Ok(self.deep_clone());
        }

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit32)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let (r, g, b, a) = color::extract_rgba(pixel);
                let a32 = a as u32;
                let inv_a = 255 - a32;

                let nr = ((a32 * r as u32 + inv_a * 255) / 255).min(255) as u8;
                let ng = ((a32 * g as u32 + inv_a * 255) / 255).min(255) as u8;
                let nb = ((a32 * b as u32 + inv_a * 255) / 255).min(255) as u8;

                result_mut.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
            }
        }

        Ok(result_mut.into())
    }

    /// Unpack 1 bpp binary image to a higher depth.
    ///
    /// Maps 0-bits and 1-bits to the appropriate max values for the
    /// target depth. If `invert` is true, the mapping is reversed.
    ///
    /// # Arguments
    ///
    /// * `depth` - Target depth (2, 4, 8, 16, or 32)
    /// * `invert` - If true, swap the val0/val1 mapping
    ///
    /// # See also
    ///
    /// C Leptonica: `pixUnpackBinary()` in `pixconv.c`
    pub fn unpack_binary(&self, _depth: u32, _invert: bool) -> Result<Pix> {
        todo!()
    }

    /// Convert 1 bpp to 2 bpp with value mapping.
    ///
    /// Each 0-bit in the source becomes `val0` (0-3) in the output,
    /// and each 1-bit becomes `val1` (0-3).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To2()` in `pixconv.c`
    pub fn convert_1_to_2(&self, _val0: u32, _val1: u32) -> Result<Pix> {
        todo!()
    }

    /// Convert 1 bpp to 4 bpp with value mapping.
    ///
    /// Each 0-bit in the source becomes `val0` (0-15) in the output,
    /// and each 1-bit becomes `val1` (0-15).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To4()` in `pixconv.c`
    pub fn convert_1_to_4(&self, _val0: u32, _val1: u32) -> Result<Pix> {
        todo!()
    }

    /// Convert 1 bpp to 8 bpp with value mapping.
    ///
    /// Each 0-bit in the source becomes `val0` (0-255) in the output,
    /// and each 1-bit becomes `val1` (0-255).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To8()` in `pixconv.c`
    pub fn convert_1_to_8(&self, _val0: u32, _val1: u32) -> Result<Pix> {
        todo!()
    }

    /// Convert 1 bpp to 16 bpp with value mapping.
    ///
    /// Each 0-bit becomes `val0` and each 1-bit becomes `val1`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To16()` in `pixconv.c`
    pub fn convert_1_to_16(&self, _val0: u32, _val1: u32) -> Result<Pix> {
        todo!()
    }

    /// Convert 1 bpp to 32 bpp with value mapping.
    ///
    /// Each 0-bit becomes `val0` and each 1-bit becomes `val1`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To32()` in `pixconv.c`
    pub fn convert_1_to_32(&self, _val0: u32, _val1: u32) -> Result<Pix> {
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
    fn test_add_gray_colormap_8_invalid_depth() {
        // Only works for 8 bpp
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.add_gray_colormap_8().is_err());
    }

    #[test]
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

    // ---- Depth conversion tests (Phase 1.4) ----

    #[test]
    fn test_convert_to_16_from_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let result = pix.convert_to_16().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit16);
        // 1bpp 0 (white) -> 0xffff
        assert_eq!(result.get_pixel(0, 0), Some(0xffff));
    }

    #[test]
    fn test_convert_to_16_from_8bpp() {
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 128);
        pm.set_pixel_unchecked(2, 0, 255);
        let pix: Pix = pm.into();

        let result = pix.convert_to_16().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit16);
        // 8bpp value replicated: val | (val << 8)
        assert_eq!(result.get_pixel(0, 0), Some(0x0000));
        assert_eq!(result.get_pixel(1, 0), Some(0x8080));
        assert_eq!(result.get_pixel(2, 0), Some(0xffff));
    }

    #[test]
    fn test_convert_to_16_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.convert_to_16().is_err());
    }

    #[test]
    fn test_convert_8_to_32() {
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 128);
        pm.set_pixel_unchecked(2, 0, 255);
        let pix: Pix = pm.into();

        let result = pix.convert_8_to_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (0, 0, 0));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (128, 128, 128));
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(2, 0));
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_convert_8_to_32_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.convert_8_to_32().is_err());
    }

    #[test]
    fn test_convert_8_to_16_shift_8() {
        let pix = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0x80);
        pm.set_pixel_unchecked(1, 0, 0xff);
        let pix: Pix = pm.into();

        let result = pix.convert_8_to_16(8).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit16);
        assert_eq!(result.get_pixel(0, 0), Some(0x8080));
        assert_eq!(result.get_pixel(1, 0), Some(0xffff));
    }

    #[test]
    fn test_convert_8_to_16_shift_0() {
        let pix = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0x80);
        let pix: Pix = pm.into();

        let result = pix.convert_8_to_16(0).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit16);
        assert_eq!(result.get_pixel(0, 0), Some(0x80));
    }

    #[test]
    fn test_convert_8_to_16_invalid_shift() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_8_to_16(9).is_err());
    }

    #[test]
    fn test_convert_16_to_8_ls_byte() {
        let pix = Pix::new(2, 1, PixelDepth::Bit16).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0xABCD);
        pm.set_pixel_unchecked(1, 0, 0x00FF);
        let pix: Pix = pm.into();

        let result = pix.convert_16_to_8(Convert16To8Type::LsByte).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.get_pixel(0, 0), Some(0xCD));
        assert_eq!(result.get_pixel(1, 0), Some(0xFF));
    }

    #[test]
    fn test_convert_16_to_8_ms_byte() {
        let pix = Pix::new(2, 1, PixelDepth::Bit16).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0xABCD);
        pm.set_pixel_unchecked(1, 0, 0x00FF);
        let pix: Pix = pm.into();

        let result = pix.convert_16_to_8(Convert16To8Type::MsByte).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.get_pixel(0, 0), Some(0xAB));
        assert_eq!(result.get_pixel(1, 0), Some(0x00));
    }

    #[test]
    fn test_convert_16_to_8_auto_byte_low_values() {
        // All values <= 255: use LSB
        let pix = Pix::new(2, 1, PixelDepth::Bit16).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 100);
        pm.set_pixel_unchecked(1, 0, 200);
        let pix: Pix = pm.into();

        let result = pix.convert_16_to_8(Convert16To8Type::AutoByte).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(100));
        assert_eq!(result.get_pixel(1, 0), Some(200));
    }

    #[test]
    fn test_convert_16_to_8_auto_byte_high_values() {
        // Some values > 255: use MSB
        let pix = Pix::new(2, 1, PixelDepth::Bit16).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0x8000);
        pm.set_pixel_unchecked(1, 0, 0xFF00);
        let pix: Pix = pm.into();

        let result = pix.convert_16_to_8(Convert16To8Type::AutoByte).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(0x80));
        assert_eq!(result.get_pixel(1, 0), Some(0xFF));
    }

    #[test]
    fn test_convert_16_to_8_clip_to_ff() {
        let pix = Pix::new(2, 1, PixelDepth::Bit16).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 100);
        pm.set_pixel_unchecked(1, 0, 0x0300); // > 255
        let pix: Pix = pm.into();

        let result = pix.convert_16_to_8(Convert16To8Type::ClipToFf).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(100));
        assert_eq!(result.get_pixel(1, 0), Some(255)); // Clipped
    }

    #[test]
    fn test_convert_16_to_8_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_16_to_8(Convert16To8Type::MsByte).is_err());
    }

    #[test]
    fn test_convert_to_8_or_32_from_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let result = pix.convert_to_8_or_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_convert_to_8_or_32_from_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = pix.convert_to_8_or_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_convert_to_8_or_32_from_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let result = pix.convert_to_8_or_32().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_convert_lossless_1_to_8() {
        let pix = Pix::new(8, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(1, 0, 0);
        let pix: Pix = pm.into();

        let result = pix.convert_lossless(8).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.get_pixel(0, 0), Some(1));
        assert_eq!(result.get_pixel(1, 0), Some(0));
    }

    #[test]
    fn test_convert_lossless_2_to_4() {
        let pix = Pix::new(4, 1, PixelDepth::Bit2).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 3);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.convert_lossless(4).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
        assert_eq!(result.get_pixel(0, 0), Some(3));
        assert_eq!(result.get_pixel(1, 0), Some(1));
    }

    #[test]
    fn test_convert_lossless_same_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit4).unwrap();
        let result = pix.convert_lossless(4).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
    }

    #[test]
    fn test_convert_lossless_invalid_reduction() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_lossless(4).is_err());
    }

    #[test]
    fn test_convert_lossless_rejects_colormap() {
        use crate::PixColormap;
        let pix = Pix::new(10, 10, PixelDepth::Bit4).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let cmap = PixColormap::create_linear(4, true).unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();
        assert!(pix.convert_lossless(8).is_err());
    }

    #[test]
    fn test_remove_alpha_rgba() {
        let pix = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_spp(4);
        // Semi-transparent red pixel (R=255, G=0, B=0, A=128)
        pm.set_pixel_unchecked(0, 0, color::compose_rgba(255, 0, 0, 128));
        // Fully opaque blue pixel
        pm.set_pixel_unchecked(1, 0, color::compose_rgba(0, 0, 255, 255));
        let pix: Pix = pm.into();

        let result = pix.remove_alpha().unwrap();
        assert_eq!(result.spp(), 3);
        // Semi-transparent red over white: r = (128*255 + 127*255)/255 = 255
        // g = (128*0 + 127*255)/255 = 127, b = (128*0 + 127*255)/255 = 127
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!(r, 255);
        assert!((g as i32 - 127).abs() <= 1);
        assert!((b as i32 - 127).abs() <= 1);

        // Fully opaque blue stays blue
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (0, 0, 255));
    }

    #[test]
    fn test_remove_alpha_no_alpha() {
        // Non-RGBA image: returns deep clone
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = pix.remove_alpha().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_convert_to_16_preserves_resolution() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_resolution(300, 300);
        let pix: Pix = pm.into();

        let result = pix.convert_to_16().unwrap();
        assert_eq!(result.xres(), 300);
        assert_eq!(result.yres(), 300);
    }

    // ---- Binary unpack tests (Phase 1.5) ----

    #[test]
    #[ignore = "not yet implemented"]
    fn test_unpack_binary_to_2() {
        let pix = Pix::new(4, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 1); // black
        pm.set_pixel_unchecked(1, 0, 0); // white
        let pix: Pix = pm.into();

        let result = pix.unpack_binary(2, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        assert_eq!(result.get_pixel(0, 0), Some(3)); // max value for 2bpp
        assert_eq!(result.get_pixel(1, 0), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_unpack_binary_to_8_inverted() {
        let pix = Pix::new(2, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 1); // black
        pm.set_pixel_unchecked(1, 0, 0); // white
        let pix: Pix = pm.into();

        let result = pix.unpack_binary(8, true).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        // Inverted: 1-bit -> 0, 0-bit -> 255
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(255));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_unpack_binary_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.unpack_binary(1, false).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_unpack_binary_not_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.unpack_binary(32, false).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_1_to_2() {
        let pix = Pix::new(4, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        pm.set_pixel_unchecked(2, 0, 0);
        pm.set_pixel_unchecked(3, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.convert_1_to_2(0, 3).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(3));
        assert_eq!(result.get_pixel(2, 0), Some(0));
        assert_eq!(result.get_pixel(3, 0), Some(3));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_1_to_4() {
        let pix = Pix::new(4, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.convert_1_to_4(0, 15).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(15));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_1_to_8() {
        let pix = Pix::new(4, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        pm.set_pixel_unchecked(2, 0, 1);
        pm.set_pixel_unchecked(3, 0, 0);
        let pix: Pix = pm.into();

        let result = pix.convert_1_to_8(0, 255).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(255));
        assert_eq!(result.get_pixel(2, 0), Some(255));
        assert_eq!(result.get_pixel(3, 0), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_1_to_8_custom_values() {
        let pix = Pix::new(2, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.convert_1_to_8(100, 200).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(100));
        assert_eq!(result.get_pixel(1, 0), Some(200));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_1_to_16() {
        let pix = Pix::new(2, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.convert_1_to_16(0, 0xffff).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit16);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(0xffff));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_1_to_32() {
        let pix = Pix::new(2, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.convert_1_to_32(0, 0xffffffff).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.get_pixel_unchecked(0, 0), 0);
        assert_eq!(result.get_pixel_unchecked(1, 0), 0xffffffff);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_1_to_2_not_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_1_to_2(0, 3).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_unpack_binary_to_16() {
        let pix = Pix::new(2, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.unpack_binary(16, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit16);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(0xffff));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_unpack_binary_to_32() {
        let pix = Pix::new(2, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.unpack_binary(32, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.get_pixel_unchecked(0, 0), 0);
        assert_eq!(result.get_pixel_unchecked(1, 0), 0xffffffff);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_1_to_preserves_resolution() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_resolution(600, 600);
        let pix: Pix = pm.into();

        let result = pix.convert_1_to_8(0, 255).unwrap();
        assert_eq!(result.xres(), 600);
        assert_eq!(result.yres(), 600);
    }
}
