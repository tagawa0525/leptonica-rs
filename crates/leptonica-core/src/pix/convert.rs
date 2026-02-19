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

/// Conversion type for 32 bpp to 16 bpp conversion.
///
/// # See also
///
/// C Leptonica: `L_LS_TWO_BYTES`, `L_MS_TWO_BYTES`, `L_CLIP_TO_FFFF` in `pix.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Convert32To16Type {
    /// Use lower 16 bits (least significant two bytes)
    LsTwoBytes,
    /// Use upper 16 bits (most significant two bytes)
    MsTwoBytes,
    /// If upper 16 bits nonzero, clamp to 0xFFFF; else use lower 16 bits
    ClipToFfff,
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

        // Build LUT: gray -> RGBA with alpha = 255 (fully opaque)
        let mut tab = [0u32; 256];
        for (i, entry) in tab.iter_mut().enumerate() {
            *entry = (i as u32) << 24 | (i as u32) << 16 | (i as u32) << 8 | 255;
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
    pub fn unpack_binary(&self, depth: u32, invert: bool) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        match depth {
            2 => {
                let (v0, v1) = if invert { (3, 0) } else { (0, 3) };
                self.convert_1_to_2(v0, v1)
            }
            4 => {
                let (v0, v1) = if invert { (15, 0) } else { (0, 15) };
                self.convert_1_to_4(v0, v1)
            }
            8 => {
                let (v0, v1) = if invert { (255, 0) } else { (0, 255) };
                self.convert_1_to_8(v0, v1)
            }
            16 => {
                let (v0, v1) = if invert { (0xffff, 0) } else { (0, 0xffff) };
                self.convert_1_to_16(v0, v1)
            }
            32 => {
                let (v0, v1) = if invert {
                    (0xffffffff, 0)
                } else {
                    (0, 0xffffffff)
                };
                self.convert_1_to_32(v0, v1)
            }
            _ => Err(Error::InvalidParameter(format!(
                "target depth must be 2, 4, 8, 16, or 32; got {}",
                depth
            ))),
        }
    }

    /// Convert 1 bpp to the specified depth with value mapping.
    ///
    /// Each 0-bit in the source becomes `val0` in the output,
    /// and each 1-bit becomes `val1`.
    fn convert_1_to(&self, target_depth: PixelDepth, val0: u32, val1: u32) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, target_depth)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let bit = self.get_pixel_unchecked(x, y);
                let val = if bit == 0 { val0 } else { val1 };
                result_mut.set_pixel_unchecked(x, y, val);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 1 bpp to 2 bpp with value mapping.
    ///
    /// Each 0-bit in the source becomes `val0` (0-3) in the output,
    /// and each 1-bit becomes `val1` (0-3).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To2()` in `pixconv.c`
    pub fn convert_1_to_2(&self, val0: u32, val1: u32) -> Result<Pix> {
        self.convert_1_to(PixelDepth::Bit2, val0, val1)
    }

    /// Convert 1 bpp to 4 bpp with value mapping.
    ///
    /// Each 0-bit in the source becomes `val0` (0-15) in the output,
    /// and each 1-bit becomes `val1` (0-15).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To4()` in `pixconv.c`
    pub fn convert_1_to_4(&self, val0: u32, val1: u32) -> Result<Pix> {
        self.convert_1_to(PixelDepth::Bit4, val0, val1)
    }

    /// Convert 1 bpp to 8 bpp with value mapping.
    ///
    /// Each 0-bit in the source becomes `val0` (0-255) in the output,
    /// and each 1-bit becomes `val1` (0-255).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To8()` in `pixconv.c`
    pub fn convert_1_to_8(&self, val0: u32, val1: u32) -> Result<Pix> {
        self.convert_1_to(PixelDepth::Bit8, val0, val1)
    }

    /// Convert 1 bpp to 16 bpp with value mapping.
    ///
    /// Each 0-bit becomes `val0` and each 1-bit becomes `val1`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To16()` in `pixconv.c`
    pub fn convert_1_to_16(&self, val0: u32, val1: u32) -> Result<Pix> {
        self.convert_1_to(PixelDepth::Bit16, val0, val1)
    }

    /// Convert 1 bpp to 32 bpp with value mapping.
    ///
    /// Each 0-bit becomes `val0` and each 1-bit becomes `val1`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert1To32()` in `pixconv.c`
    pub fn convert_1_to_32(&self, val0: u32, val1: u32) -> Result<Pix> {
        self.convert_1_to(PixelDepth::Bit32, val0, val1)
    }

    /// Convert 2 bpp to 8 bpp with custom value mapping.
    ///
    /// Each 2-bit pixel value (0-3) is mapped to the corresponding 8-bit value:
    /// - 0 → `val0`
    /// - 1 → `val1`
    /// - 2 → `val2`
    /// - 3 → `val3`
    ///
    /// If `add_colormap` is true, the result has a 4-entry colormap using the
    /// specified values. Otherwise, the values are written directly as 8-bit pixels.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert2To8()` in `pixconv.c`
    pub fn convert_2_to_8(
        &self,
        val0: u8,
        val1: u8,
        val2: u8,
        val3: u8,
        add_colormap: bool,
    ) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit2 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let vals = [val0, val1, val2, val3];

        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        if add_colormap {
            use crate::PixColormap;
            let mut cmap = PixColormap::new(8)?;
            for &v in &vals {
                cmap.add_rgb(v, v, v)?;
            }
            result_mut.set_colormap(Some(cmap))?;

            // Pixel values are colormap indices (0-3)
            for y in 0..h {
                for x in 0..w {
                    let val = self.get_pixel_unchecked(x, y);
                    result_mut.set_pixel_unchecked(x, y, val);
                }
            }
        } else {
            // Direct value mapping
            for y in 0..h {
                for x in 0..w {
                    let val = self.get_pixel_unchecked(x, y) as usize;
                    result_mut.set_pixel_unchecked(x, y, vals[val] as u32);
                }
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 4 bpp to 8 bpp.
    ///
    /// If `add_colormap` is true, the result is an 8-bit image with a 16-entry
    /// linear colormap (0, 17, 34, ..., 255). Otherwise, each 4-bit value is
    /// expanded to 8 bits by replication: `(val << 4) | val`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert4To8()` in `pixconv.c`
    pub fn convert_4_to_8(&self, add_colormap: bool) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit4 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();

        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        if add_colormap {
            use crate::PixColormap;
            let mut cmap = PixColormap::new(8)?;
            for i in 0..16u32 {
                let v = (i * 255 / 15) as u8;
                cmap.add_rgb(v, v, v)?;
            }
            result_mut.set_colormap(Some(cmap))?;

            // Pixel values are colormap indices (0-15)
            for y in 0..h {
                for x in 0..w {
                    let val = self.get_pixel_unchecked(x, y);
                    result_mut.set_pixel_unchecked(x, y, val);
                }
            }
        } else {
            // Replicate nibble: (val << 4) | val
            for y in 0..h {
                for x in 0..w {
                    let val = self.get_pixel_unchecked(x, y);
                    result_mut.set_pixel_unchecked(x, y, (val << 4) | val);
                }
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 8 bpp grayscale to 2 bpp by taking the top 2 bits.
    ///
    /// Each 8-bit pixel value is quantized to 2 bits by taking bits 7-6.
    /// This is a lossy reduction: values 0-63→0, 64-127→1, 128-191→2, 192-255→3.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert8To2()` in `pixconv.c`
    pub fn convert_8_to_2(&self) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();

        let result = Pix::new(w, h, PixelDepth::Bit2)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y);
                result_mut.set_pixel_unchecked(x, y, val >> 6);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert 8 bpp grayscale to 4 bpp by taking the top 4 bits.
    ///
    /// Each 8-bit pixel value is quantized to 4 bits by right-shifting by 4.
    /// This is a lossy reduction: values are grouped in ranges of 16.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert8To4()` in `pixconv.c`
    pub fn convert_8_to_4(&self) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();

        let result = Pix::new(w, h, PixelDepth::Bit4)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y);
                result_mut.set_pixel_unchecked(x, y, val >> 4);
            }
        }

        Ok(result_mut.into())
    }

    /// Convert any-depth image to 2 bpp grayscale.
    ///
    /// Conversion rules:
    /// - **1 bpp**: 0→0, 1→3
    /// - **2 bpp**: identity (deep clone, strips colormap if present)
    /// - **4 bpp**: convert via 8 bpp intermediate
    /// - **8 bpp**: take top 2 bits
    /// - **16 bpp**: convert to 8 bpp, then take top 2 bits
    /// - **32 bpp**: convert to 8 bpp, then take top 2 bits
    ///
    /// If the source has a colormap, it is removed to grayscale first.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertTo2()` in `pixconv.c`
    pub fn convert_to_2(&self) -> Result<Pix> {
        // If colormap present, remove it first
        if self.has_colormap() {
            let gray = self.remove_colormap(RemoveColormapTarget::ToGrayscale)?;
            return gray.convert_to_2();
        }

        match self.depth() {
            PixelDepth::Bit1 => self.convert_1_to_2(0, 3),
            PixelDepth::Bit2 => Ok(self.deep_clone()),
            PixelDepth::Bit4 | PixelDepth::Bit16 | PixelDepth::Bit32 => {
                let gray8 = self.convert_to_8()?;
                gray8.convert_8_to_2()
            }
            PixelDepth::Bit8 => self.convert_8_to_2(),
        }
    }

    /// Convert any-depth image to 4 bpp grayscale.
    ///
    /// Conversion rules:
    /// - **1 bpp**: 0→0, 1→15
    /// - **2 bpp**: convert via 8 bpp intermediate (0→0, 1→0x55, 2→0xaa, 3→0xff)
    /// - **4 bpp**: identity (deep clone, strips colormap if present)
    /// - **8 bpp**: take top 4 bits
    /// - **16 bpp**: convert to 8 bpp, then take top 4 bits
    /// - **32 bpp**: convert to 8 bpp, then take top 4 bits
    ///
    /// If the source has a colormap, it is removed to grayscale first.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertTo4()` in `pixconv.c`
    pub fn convert_to_4(&self) -> Result<Pix> {
        // If colormap present, remove it first
        if self.has_colormap() {
            let gray = self.remove_colormap(RemoveColormapTarget::ToGrayscale)?;
            return gray.convert_to_4();
        }

        match self.depth() {
            PixelDepth::Bit1 => self.convert_1_to_4(0, 15),
            PixelDepth::Bit2 => {
                // 2bpp → 8bpp (0→0, 1→0x55, 2→0xaa, 3→0xff) → 4bpp
                let gray8 = self.convert_2_to_8(0, 0x55, 0xaa, 0xff, false)?;
                gray8.convert_8_to_4()
            }
            PixelDepth::Bit4 => Ok(self.deep_clone()),
            PixelDepth::Bit8 => self.convert_8_to_4(),
            PixelDepth::Bit16 | PixelDepth::Bit32 => {
                let gray8 = self.convert_to_8()?;
                gray8.convert_8_to_4()
            }
        }
    }

    /// Add a colormap to a grayscale image without quantization loss.
    ///
    /// Works for 2, 4, and 8 bpp grayscale images. For 8 bpp, delegates
    /// to [`convert_gray_to_colormap_8`](Self::convert_gray_to_colormap_8)
    /// with `min_depth = 2`.
    ///
    /// For 2 and 4 bpp, creates a linear colormap spanning the full
    /// 8-bit range (e.g., for 2 bpp: 0, 85, 170, 255).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertGrayToColormap()` in `pixconv.c`
    pub fn convert_gray_to_colormap(&self) -> Result<Pix> {
        // Already has colormap: return deep clone
        if self.has_colormap() {
            return Ok(self.deep_clone());
        }

        match self.depth() {
            PixelDepth::Bit2 | PixelDepth::Bit4 => {
                use crate::PixColormap;
                let d = self.depth().bits();
                let n = 1u32 << d;
                let mut cmap = PixColormap::new(d)?;
                for i in 0..n {
                    let v = (i * 255 / (n - 1)) as u8;
                    cmap.add_rgb(v, v, v)?;
                }

                let mut result_mut = self.deep_clone().try_into_mut().unwrap();
                result_mut.set_colormap(Some(cmap))?;
                Ok(result_mut.into())
            }
            PixelDepth::Bit8 => self.convert_gray_to_colormap_8(2),
            _ => Err(Error::UnsupportedDepth(self.depth().bits())),
        }
    }

    /// Lossless conversion of 8 bpp grayscale to colormapped image.
    ///
    /// Determines the optimal output depth based on the number of unique
    /// gray values in the image:
    /// - ≤ 4 unique values: 2 bpp (if `min_depth` ≤ 2)
    /// - ≤ 16 unique values: 4 bpp (if `min_depth` ≤ 4)
    /// - Otherwise: 8 bpp
    ///
    /// The pixel values are remapped to colormap indices, and a colormap
    /// is created with the actual gray values used.
    ///
    /// # Arguments
    ///
    /// * `min_depth` - Minimum output depth (2, 4, or 8)
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertGrayToColormap8()` in `pixconv.c`
    pub fn convert_gray_to_colormap_8(&self, min_depth: u32) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if self.has_colormap() {
            return Ok(self.deep_clone());
        }

        use crate::PixColormap;

        let w = self.width();
        let h = self.height();

        // Build histogram of gray values
        let mut histogram = [0u32; 256];
        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y) as usize;
                histogram[val] += 1;
            }
        }

        // Count unique values and build mapping table
        let mut ncolors = 0usize;
        let mut gray_to_index = [0u8; 256];
        for (gray_val, &count) in histogram.iter().enumerate() {
            if count > 0 {
                gray_to_index[gray_val] = ncolors as u8;
                ncolors += 1;
            }
        }

        // Determine output depth
        let out_depth = if ncolors <= 4 && min_depth <= 2 {
            2
        } else if ncolors <= 16 && min_depth <= 4 {
            4
        } else {
            8
        };

        let out_pixel_depth = match out_depth {
            2 => PixelDepth::Bit2,
            4 => PixelDepth::Bit4,
            _ => PixelDepth::Bit8,
        };

        // Build colormap with actual gray values
        let mut cmap = PixColormap::new(out_depth)?;
        for (gray_val, &count) in histogram.iter().enumerate() {
            if count > 0 {
                let v = gray_val as u8;
                cmap.add_rgb(v, v, v)?;
            }
        }

        // Create output image
        let result = Pix::new(w, h, out_pixel_depth)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());
        result_mut.set_colormap(Some(cmap))?;

        // Map pixels to colormap indices
        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y) as usize;
                result_mut.set_pixel_unchecked(x, y, gray_to_index[val] as u32);
            }
        }

        Ok(result_mut.into())
    }
}

// -----------------------------------------------------------------------
// Phase 13.2: 高ビット・特殊変換
// -----------------------------------------------------------------------

impl Pix {
    /// Convert 32 bpp label image to 16 bpp.
    ///
    /// # Arguments
    ///
    /// * `conversion_type` - How to extract 16 bits from each 32-bit word
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvert32To16()` in `pixconv.c`
    pub fn convert_32_to_16(&self, conversion_type: Convert32To16Type) -> Result<Pix> {
        todo!("Phase 13.2: convert_32_to_16 not yet implemented")
    }

    /// Add RGBA colormap to a 1 bpp image.
    ///
    /// Background (0) pixels become transparent white; foreground (1) become opaque black.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddAlphaTo1bpp()` in `pixconv.c`
    pub fn add_alpha_to_1bpp(&self) -> Result<Pix> {
        todo!("Phase 13.2: add_alpha_to_1bpp not yet implemented")
    }

    /// Convert 32 bpp RGB to 8 bpp gray using arbitrary linear weights.
    ///
    /// Unlike `convert_rgb_to_gray`, weights may be negative, but at least one
    /// must be positive. Output values are clamped to [0, 255].
    ///
    /// # Arguments
    ///
    /// * `rc` - Red channel weight
    /// * `gc` - Green channel weight
    /// * `bc` - Blue channel weight
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertRGBToGrayArb()` in `pixconv.c`
    pub fn convert_rgb_to_gray_arb(&self, rc: f32, gc: f32, bc: f32) -> Result<Pix> {
        todo!("Phase 13.2: convert_rgb_to_gray_arb not yet implemented")
    }

    /// Colorize an 8 bpp gray (or colormapped) image with a given color.
    ///
    /// # Arguments
    ///
    /// * `color` - 32-bit RGBA pixel value for the tint color
    /// * `cmapflag` - If `true`, return 8 bpp colormapped; if `false`, return 32 bpp RGB
    ///
    /// # See also
    ///
    /// C Leptonica: `pixColorizeGray()` in `pixconv.c`
    pub fn colorize_gray(&self, color: u32, cmapflag: bool) -> Result<Pix> {
        todo!("Phase 13.2: colorize_gray not yet implemented")
    }

    /// Convert 8 or 16 bpp gray image to 8 bpp with false color colormap.
    ///
    /// # Arguments
    ///
    /// * `gamma` - Gamma factor (0.0 or 1.0 = default; > 1.0 for brighter)
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertGrayToFalseColor()` in `pixconv.c`
    pub fn convert_gray_to_false_color(&self, gamma: f32) -> Result<Pix> {
        todo!("Phase 13.2: convert_gray_to_false_color not yet implemented")
    }

    /// Convert a colormapped image to 1 bpp binary using heuristic FG/BG classification.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertCmapTo1()` in `pixconv.c`
    pub fn convert_cmap_to_1(&self) -> Result<Pix> {
        todo!("Phase 13.2: convert_cmap_to_1 not yet implemented")
    }

    /// Convert image to 1, 8, or 32 bpp for PostScript wrapping.
    ///
    /// Colormaps are removed. 2/4 bpp without colormaps are converted to 8 bpp.
    /// 16 bpp is converted to 8 bpp using the most significant byte.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixConvertForPSWrap()` in `pixconv.c`
    pub fn convert_for_ps_wrap(&self) -> Result<Pix> {
        todo!("Phase 13.2: convert_for_ps_wrap not yet implemented")
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
        assert_eq!(color::alpha(result.get_pixel_unchecked(0, 0)), 255);
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (128, 128, 128));
        assert_eq!(color::alpha(result.get_pixel_unchecked(1, 0)), 255);
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(2, 0));
        assert_eq!((r, g, b), (255, 255, 255));
        assert_eq!(color::alpha(result.get_pixel_unchecked(2, 0)), 255);
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
    fn test_unpack_binary_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.unpack_binary(1, false).is_err());
    }

    #[test]
    fn test_unpack_binary_not_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.unpack_binary(32, false).is_err());
    }

    #[test]
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
    fn test_convert_1_to_2_not_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_1_to_2(0, 3).is_err());
    }

    #[test]
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
    fn test_convert_1_to_preserves_resolution() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_resolution(600, 600);
        let pix: Pix = pm.into();

        let result = pix.convert_1_to_8(0, 255).unwrap();
        assert_eq!(result.xres(), 600);
        assert_eq!(result.yres(), 600);
    }

    // ---- Phase 13.1: Low bit depth conversion tests ----

    #[test]
    fn test_convert_2_to_8_direct() {
        // 2bpp → 8bpp with custom values, no colormap
        let pix = Pix::new(4, 1, PixelDepth::Bit2).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        pm.set_pixel_unchecked(2, 0, 2);
        pm.set_pixel_unchecked(3, 0, 3);
        let pix: Pix = pm.into();

        let result = pix.convert_2_to_8(10, 20, 30, 40, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(!result.has_colormap());
        assert_eq!(result.get_pixel(0, 0), Some(10));
        assert_eq!(result.get_pixel(1, 0), Some(20));
        assert_eq!(result.get_pixel(2, 0), Some(30));
        assert_eq!(result.get_pixel(3, 0), Some(40));
    }

    #[test]
    fn test_convert_2_to_8_with_colormap() {
        // 2bpp → 8bpp with colormap
        let pix = Pix::new(4, 1, PixelDepth::Bit2).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        pm.set_pixel_unchecked(2, 0, 2);
        pm.set_pixel_unchecked(3, 0, 3);
        let pix: Pix = pm.into();

        let result = pix.convert_2_to_8(0, 85, 170, 255, true).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.has_colormap());
        let cmap = result.colormap().unwrap();
        assert_eq!(cmap.len(), 4);
        // Colormap entries should be the specified gray values
        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        assert_eq!(cmap.get_rgb(1), Some((85, 85, 85)));
        assert_eq!(cmap.get_rgb(2), Some((170, 170, 170)));
        assert_eq!(cmap.get_rgb(3), Some((255, 255, 255)));
        // Pixel values are colormap indices
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(1));
        assert_eq!(result.get_pixel(2, 0), Some(2));
        assert_eq!(result.get_pixel(3, 0), Some(3));
    }

    #[test]
    fn test_convert_2_to_8_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit4).unwrap();
        assert!(pix.convert_2_to_8(0, 85, 170, 255, false).is_err());
    }

    #[test]
    fn test_convert_2_to_8_preserves_resolution() {
        let pix = Pix::new(4, 1, PixelDepth::Bit2).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_resolution(300, 300);
        let pix: Pix = pm.into();

        let result = pix.convert_2_to_8(0, 85, 170, 255, false).unwrap();
        assert_eq!(result.xres(), 300);
        assert_eq!(result.yres(), 300);
    }

    #[test]
    fn test_convert_4_to_8_direct() {
        // 4bpp → 8bpp without colormap: (val << 4) | val
        let pix = Pix::new(4, 1, PixelDepth::Bit4).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 5);
        pm.set_pixel_unchecked(2, 0, 10);
        pm.set_pixel_unchecked(3, 0, 15);
        let pix: Pix = pm.into();

        let result = pix.convert_4_to_8(false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(!result.has_colormap());
        // (0<<4)|0=0, (5<<4)|5=0x55=85, (10<<4)|10=0xaa=170, (15<<4)|15=0xff=255
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(0x55));
        assert_eq!(result.get_pixel(2, 0), Some(0xaa));
        assert_eq!(result.get_pixel(3, 0), Some(0xff));
    }

    #[test]
    fn test_convert_4_to_8_with_colormap() {
        // 4bpp → 8bpp with linear colormap
        let pix = Pix::new(2, 1, PixelDepth::Bit4).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 15);
        let pix: Pix = pm.into();

        let result = pix.convert_4_to_8(true).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.has_colormap());
        let cmap = result.colormap().unwrap();
        assert_eq!(cmap.len(), 16);
        // Linear colormap: i*255/15
        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        assert_eq!(cmap.get_rgb(15), Some((255, 255, 255)));
        // Pixel values are colormap indices
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(15));
    }

    #[test]
    fn test_convert_4_to_8_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_4_to_8(false).is_err());
    }

    #[test]
    fn test_convert_8_to_2_basic() {
        // 8bpp → 2bpp: take top 2 bits
        let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0); // top 2 bits = 0b00 = 0
        pm.set_pixel_unchecked(1, 0, 64); // top 2 bits = 0b01 = 1
        pm.set_pixel_unchecked(2, 0, 128); // top 2 bits = 0b10 = 2
        pm.set_pixel_unchecked(3, 0, 255); // top 2 bits = 0b11 = 3
        let pix: Pix = pm.into();

        let result = pix.convert_8_to_2().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(1));
        assert_eq!(result.get_pixel(2, 0), Some(2));
        assert_eq!(result.get_pixel(3, 0), Some(3));
    }

    #[test]
    fn test_convert_8_to_2_boundary_values() {
        // Test values near boundaries: 63→0, 64→1, 127→1, 128→2, 191→2, 192→3
        let pix = Pix::new(6, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 63);
        pm.set_pixel_unchecked(1, 0, 64);
        pm.set_pixel_unchecked(2, 0, 127);
        pm.set_pixel_unchecked(3, 0, 128);
        pm.set_pixel_unchecked(4, 0, 191);
        pm.set_pixel_unchecked(5, 0, 192);
        let pix: Pix = pm.into();

        let result = pix.convert_8_to_2().unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(1));
        assert_eq!(result.get_pixel(2, 0), Some(1));
        assert_eq!(result.get_pixel(3, 0), Some(2));
        assert_eq!(result.get_pixel(4, 0), Some(2));
        assert_eq!(result.get_pixel(5, 0), Some(3));
    }

    #[test]
    fn test_convert_8_to_2_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit4).unwrap();
        assert!(pix.convert_8_to_2().is_err());
    }

    #[test]
    fn test_convert_8_to_2_preserves_resolution() {
        let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_resolution(600, 600);
        let pix: Pix = pm.into();

        let result = pix.convert_8_to_2().unwrap();
        assert_eq!(result.xres(), 600);
        assert_eq!(result.yres(), 600);
    }

    #[test]
    fn test_convert_8_to_4_basic() {
        // 8bpp → 4bpp: take top 4 bits (val >> 4)
        let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 0x50); // >> 4 = 5
        pm.set_pixel_unchecked(2, 0, 0xa0); // >> 4 = 10
        pm.set_pixel_unchecked(3, 0, 0xff); // >> 4 = 15
        let pix: Pix = pm.into();

        let result = pix.convert_8_to_4().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(5));
        assert_eq!(result.get_pixel(2, 0), Some(10));
        assert_eq!(result.get_pixel(3, 0), Some(15));
    }

    #[test]
    fn test_convert_8_to_4_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit2).unwrap();
        assert!(pix.convert_8_to_4().is_err());
    }

    #[test]
    fn test_convert_to_2_from_1bpp() {
        let pix = Pix::new(4, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.convert_to_2().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(3));
    }

    #[test]
    fn test_convert_to_2_from_2bpp() {
        // Identity case
        let pix = Pix::new(4, 1, PixelDepth::Bit2).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        pm.set_pixel_unchecked(2, 0, 2);
        pm.set_pixel_unchecked(3, 0, 3);
        let pix: Pix = pm.into();

        let result = pix.convert_to_2().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        assert!(!result.has_colormap());
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(1));
        assert_eq!(result.get_pixel(2, 0), Some(2));
        assert_eq!(result.get_pixel(3, 0), Some(3));
    }

    #[test]
    fn test_convert_to_2_from_4bpp() {
        // 4bpp → 8bpp → 2bpp
        let pix = Pix::new(4, 1, PixelDepth::Bit4).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 5);
        pm.set_pixel_unchecked(2, 0, 10);
        pm.set_pixel_unchecked(3, 0, 15);
        let pix: Pix = pm.into();

        let result = pix.convert_to_2().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        // 4bpp val*255/15 → 8bpp → top 2 bits
        // 0*17=0→0, 5*17=85→1, 10*17=170→2, 15*17=255→3
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(1));
        assert_eq!(result.get_pixel(2, 0), Some(2));
        assert_eq!(result.get_pixel(3, 0), Some(3));
    }

    #[test]
    fn test_convert_to_2_from_8bpp() {
        let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 100);
        pm.set_pixel_unchecked(2, 0, 200);
        pm.set_pixel_unchecked(3, 0, 255);
        let pix: Pix = pm.into();

        let result = pix.convert_to_2().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        assert_eq!(result.get_pixel(0, 0), Some(0)); // 0 >> 6 = 0
        assert_eq!(result.get_pixel(1, 0), Some(1)); // 100 >> 6 = 1
        assert_eq!(result.get_pixel(2, 0), Some(3)); // 200 >> 6 = 3
        assert_eq!(result.get_pixel(3, 0), Some(3)); // 255 >> 6 = 3
    }

    #[test]
    fn test_convert_to_2_from_32bpp() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 200, 200, 200).unwrap();
        let pix: Pix = pm.into();

        let result = pix.convert_to_2().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        // 200 luminance → top 2 bits = 3
        assert_eq!(result.get_pixel(0, 0), Some(3));
    }

    #[test]
    fn test_convert_to_2_from_16bpp() {
        let pix = Pix::new(1, 1, PixelDepth::Bit16).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0x8000);
        let pix: Pix = pm.into();

        let result = pix.convert_to_2().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        // 0x8000 MSB = 0x80 = 128, top 2 bits = 2
        assert_eq!(result.get_pixel(0, 0), Some(2));
    }

    #[test]
    fn test_convert_to_2_with_colormap() {
        use crate::PixColormap;
        // 2bpp with colormap → strip colormap, convert to grayscale then 2bpp
        let pix = Pix::new(2, 1, PixelDepth::Bit2).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap(); // color → remove cmap first
        cmap.add_rgb(0, 255, 0).unwrap();
        cmap.add_rgb(0, 0, 255).unwrap();
        cmap.add_rgb(255, 255, 255).unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        pm.set_pixel_unchecked(0, 0, 3); // white
        let pix: Pix = pm.into();

        let result = pix.convert_to_2().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        assert!(!result.has_colormap());
    }

    #[test]
    fn test_convert_to_4_from_1bpp() {
        let pix = Pix::new(4, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix.convert_to_4().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(15));
    }

    #[test]
    fn test_convert_to_4_from_2bpp() {
        // 2bpp → 8bpp (0→0, 1→0x55, 2→0xaa, 3→0xff) → 4bpp
        let pix = Pix::new(4, 1, PixelDepth::Bit2).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        pm.set_pixel_unchecked(2, 0, 2);
        pm.set_pixel_unchecked(3, 0, 3);
        let pix: Pix = pm.into();

        let result = pix.convert_to_4().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
        // 0→0>>4=0, 0x55=85>>4=5, 0xaa=170>>4=10, 0xff=255>>4=15
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(5));
        assert_eq!(result.get_pixel(2, 0), Some(10));
        assert_eq!(result.get_pixel(3, 0), Some(15));
    }

    #[test]
    fn test_convert_to_4_from_4bpp() {
        // Identity case
        let pix = Pix::new(4, 1, PixelDepth::Bit4).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 7);
        pm.set_pixel_unchecked(2, 0, 15);
        let pix: Pix = pm.into();

        let result = pix.convert_to_4().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
        assert!(!result.has_colormap());
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(7));
        assert_eq!(result.get_pixel(2, 0), Some(15));
    }

    #[test]
    fn test_convert_to_4_from_8bpp() {
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 0x80);
        pm.set_pixel_unchecked(2, 0, 0xff);
        let pix: Pix = pm.into();

        let result = pix.convert_to_4().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(8)); // 0x80 >> 4 = 8
        assert_eq!(result.get_pixel(2, 0), Some(15)); // 0xff >> 4 = 15
    }

    #[test]
    fn test_convert_to_4_from_32bpp() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 128, 128, 128).unwrap();
        let pix: Pix = pm.into();

        let result = pix.convert_to_4().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
        // 128 luminance → top 4 bits = 8
        assert_eq!(result.get_pixel(0, 0), Some(8));
    }

    #[test]
    fn test_convert_gray_to_colormap_2bpp() {
        let pix = Pix::new(4, 1, PixelDepth::Bit2).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        pm.set_pixel_unchecked(2, 0, 2);
        pm.set_pixel_unchecked(3, 0, 3);
        let pix: Pix = pm.into();

        let result = pix.convert_gray_to_colormap().unwrap();
        assert!(result.has_colormap());
        let cmap = result.colormap().unwrap();
        // 2bpp linear colormap: 4 entries spanning 0-255
        assert_eq!(cmap.len(), 4);
        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        assert_eq!(cmap.get_rgb(3), Some((255, 255, 255)));
    }

    #[test]
    fn test_convert_gray_to_colormap_4bpp() {
        let pix = Pix::new(2, 1, PixelDepth::Bit4).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 15);
        let pix: Pix = pm.into();

        let result = pix.convert_gray_to_colormap().unwrap();
        assert!(result.has_colormap());
        let cmap = result.colormap().unwrap();
        assert_eq!(cmap.len(), 16);
        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        assert_eq!(cmap.get_rgb(15), Some((255, 255, 255)));
    }

    #[test]
    fn test_convert_gray_to_colormap_8bpp() {
        // For 8bpp, delegates to convert_gray_to_colormap_8 with min_depth=2
        let pix = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 255);
        let pix: Pix = pm.into();

        let result = pix.convert_gray_to_colormap().unwrap();
        assert!(result.has_colormap());
    }

    #[test]
    fn test_convert_gray_to_colormap_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.convert_gray_to_colormap().is_err());
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.convert_gray_to_colormap().is_err());
    }

    #[test]
    fn test_convert_gray_to_colormap_already_has_colormap() {
        use crate::PixColormap;
        let pix = Pix::new(10, 10, PixelDepth::Bit4).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let cmap = PixColormap::create_linear(4, true).unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();

        // Should return a deep clone since it already has a colormap
        let result = pix.convert_gray_to_colormap().unwrap();
        assert!(result.has_colormap());
    }

    #[test]
    fn test_convert_gray_to_colormap_8_few_colors() {
        // Only 3 unique values → should get 2bpp output
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 50);
        pm.set_pixel_unchecked(1, 0, 100);
        pm.set_pixel_unchecked(2, 0, 200);
        let pix: Pix = pm.into();

        let result = pix.convert_gray_to_colormap_8(2).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit2);
        assert!(result.has_colormap());
        let cmap = result.colormap().unwrap();
        assert_eq!(cmap.len(), 3);
        // Verify roundtrip: pixel values should map to correct gray values
        let idx0 = result.get_pixel(0, 0).unwrap() as usize;
        let idx1 = result.get_pixel(1, 0).unwrap() as usize;
        let idx2 = result.get_pixel(2, 0).unwrap() as usize;
        assert_eq!(cmap.get_rgb(idx0), Some((50, 50, 50)));
        assert_eq!(cmap.get_rgb(idx1), Some((100, 100, 100)));
        assert_eq!(cmap.get_rgb(idx2), Some((200, 200, 200)));
    }

    #[test]
    fn test_convert_gray_to_colormap_8_many_colors() {
        // More than 16 unique values → 8bpp output
        let pix = Pix::new(20, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for i in 0..20 {
            pm.set_pixel_unchecked(i, 0, (i * 13) as u32);
        }
        let pix: Pix = pm.into();

        let result = pix.convert_gray_to_colormap_8(2).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.has_colormap());
    }

    #[test]
    fn test_convert_gray_to_colormap_8_min_depth_4() {
        // Only 3 unique values, but min_depth=4 → should get 4bpp
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 10);
        pm.set_pixel_unchecked(1, 0, 100);
        pm.set_pixel_unchecked(2, 0, 200);
        let pix: Pix = pm.into();

        let result = pix.convert_gray_to_colormap_8(4).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit4);
        assert!(result.has_colormap());
    }

    #[test]
    fn test_convert_gray_to_colormap_8_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit4).unwrap();
        assert!(pix.convert_gray_to_colormap_8(2).is_err());
    }

    #[test]
    fn test_convert_gray_to_colormap_8_roundtrip() {
        // Create image, convert to colormap, remove colormap, verify values match
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..5u32 {
            for x in 0..5u32 {
                pm.set_pixel_unchecked(x, y, (x * 50 + y * 10) as u32);
            }
        }
        let pix: Pix = pm.into();

        let cmapped = pix.convert_gray_to_colormap_8(2).unwrap();
        let restored = cmapped
            .remove_colormap(RemoveColormapTarget::ToGrayscale)
            .unwrap();

        // Verify all pixels match
        for y in 0..5u32 {
            for x in 0..5u32 {
                assert_eq!(
                    pix.get_pixel(x, y),
                    restored.get_pixel(x, y),
                    "mismatch at ({x}, {y})"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Phase 13.2: 高ビット・特殊変換テスト
    // -----------------------------------------------------------------------

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_32_to_16_ls_two_bytes() {
        // 32bpp label image → 16bpp: lower 16 bits extracted
        let pix = Pix::new(4, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0x0001_0002); // lower=0x0002
        pm.set_pixel_unchecked(1, 0, 0x0000_FFFF); // lower=0xFFFF
        pm.set_pixel_unchecked(2, 0, 0xFFFF_0000); // lower=0x0000
        pm.set_pixel_unchecked(3, 0, 0x0001_ABCD); // lower=0xABCD
        let pix: Pix = pm.into();

        let result = pix.convert_32_to_16(Convert32To16Type::LsTwoBytes).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit16);
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 1);
        assert_eq!(result.get_pixel(0, 0), Some(0x0002));
        assert_eq!(result.get_pixel(1, 0), Some(0xFFFF));
        assert_eq!(result.get_pixel(2, 0), Some(0x0000));
        assert_eq!(result.get_pixel(3, 0), Some(0xABCD));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_32_to_16_ms_two_bytes() {
        // 32bpp → 16bpp: upper 16 bits extracted
        let pix = Pix::new(3, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0x1234_5678);
        pm.set_pixel_unchecked(1, 0, 0x0000_ABCD);
        pm.set_pixel_unchecked(2, 0, 0xFFFF_0000);
        let pix: Pix = pm.into();

        let result = pix.convert_32_to_16(Convert32To16Type::MsTwoBytes).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit16);
        assert_eq!(result.get_pixel(0, 0), Some(0x1234));
        assert_eq!(result.get_pixel(1, 0), Some(0x0000));
        assert_eq!(result.get_pixel(2, 0), Some(0xFFFF));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_32_to_16_clip_to_ffff() {
        // 32bpp → 16bpp: clip: if upper 16 bits nonzero → 0xFFFF, else keep lower
        let pix = Pix::new(3, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0x0001_ABCD); // upper nonzero → 0xFFFF
        pm.set_pixel_unchecked(1, 0, 0x0000_1234); // upper zero → 0x1234
        pm.set_pixel_unchecked(2, 0, 0x0000_0000); // → 0x0000
        let pix: Pix = pm.into();

        let result = pix.convert_32_to_16(Convert32To16Type::ClipToFfff).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(0xFFFF));
        assert_eq!(result.get_pixel(1, 0), Some(0x1234));
        assert_eq!(result.get_pixel(2, 0), Some(0x0000));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_32_to_16_invalid_depth() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_32_to_16(Convert32To16Type::LsTwoBytes).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_alpha_to_1bpp() {
        // 1bpp image gets colormap: 0→transparent white, 1→opaque black
        let pix = Pix::new(4, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0); // BG
        pm.set_pixel_unchecked(1, 0, 1); // FG
        pm.set_pixel_unchecked(2, 0, 1); // FG
        pm.set_pixel_unchecked(3, 0, 0); // BG
        let pix: Pix = pm.into();

        let result = pix.add_alpha_to_1bpp().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert!(result.has_colormap());
        let cmap = result.colormap().unwrap();
        assert_eq!(cmap.len(), 2);
        // index 0: white + transparent (alpha=0)
        assert_eq!(cmap.get_rgba(0), Some((255, 255, 255, 0)));
        // index 1: black + opaque (alpha=255)
        assert_eq!(cmap.get_rgba(1), Some((0, 0, 0, 255)));
        // Pixel values are preserved
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(1, 0), Some(1));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_alpha_to_1bpp_invalid_depth() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        assert!(pix.add_alpha_to_1bpp().is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_rgb_to_gray_arb_basic() {
        // Simple test: all-red image → only rc contributes
        let pix = Pix::new(3, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // pure red pixel: R=200, G=0, B=0
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(200, 0, 0));
        // pure green pixel: R=0, G=100, B=0
        pm.set_pixel_unchecked(1, 0, color::compose_rgb(0, 100, 0));
        // mixed: R=100, G=100, B=100 with weights 0.5, 0.3, 0.2 → 60.0 → 60
        pm.set_pixel_unchecked(2, 0, color::compose_rgb(100, 100, 100));
        let pix: Pix = pm.into();

        let result = pix.convert_rgb_to_gray_arb(0.5, 0.3, 0.2).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        // R=200, weight 0.5 → 100
        assert_eq!(result.get_pixel(0, 0), Some(100));
        // G=100, weight 0.3 → 30
        assert_eq!(result.get_pixel(1, 0), Some(30));
        // (100*0.5 + 100*0.3 + 100*0.2) = 100.0 → 100
        assert_eq!(result.get_pixel(2, 0), Some(100));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_rgb_to_gray_arb_clipping() {
        // Large weights cause overflow: clipped to 255
        let pix = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(255, 255, 255));
        pm.set_pixel_unchecked(1, 0, color::compose_rgb(0, 0, 0));
        let pix: Pix = pm.into();

        // weights 2.0 each → 255*2+255*2+255*2=1530 → clip to 255
        let result = pix.convert_rgb_to_gray_arb(2.0, 2.0, 2.0).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(255));
        // 0 → 0
        assert_eq!(result.get_pixel(1, 0), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_rgb_to_gray_arb_negative_weights_allowed() {
        // At least one positive coefficient is required
        let pix = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 50, 0));
        pm.set_pixel_unchecked(1, 0, color::compose_rgb(50, 100, 0));
        let pix: Pix = pm.into();

        // rc positive, gc negative → R dominates, result clipped at 0
        let result = pix.convert_rgb_to_gray_arb(1.0, -0.5, 0.0).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        // 100*1.0 + 50*(-0.5) = 75
        assert_eq!(result.get_pixel(0, 0), Some(75));
        // 50*1.0 + 100*(-0.5) = 0
        assert_eq!(result.get_pixel(1, 0), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_rgb_to_gray_arb_all_nonpositive_fails() {
        let pix = Pix::new(2, 2, PixelDepth::Bit32).unwrap();
        // All weights <= 0 → error
        assert!(pix.convert_rgb_to_gray_arb(-1.0, -1.0, -1.0).is_err());
        assert!(pix.convert_rgb_to_gray_arb(0.0, 0.0, 0.0).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_rgb_to_gray_arb_wrong_depth() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_rgb_to_gray_arb(0.3, 0.5, 0.2).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_colorize_gray_to_rgb() {
        // 8bpp gray image colorized to 32bpp RGB
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0); // darkest
        pm.set_pixel_unchecked(1, 0, 128); // mid
        pm.set_pixel_unchecked(2, 0, 255); // brightest
        let pix: Pix = pm.into();

        // Use red tint: color = 0xFF000000 (red pixel in RGBA format)
        let result = pix
            .colorize_gray(color::compose_rgb(255, 0, 0), false)
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        // pixel 0 (gray=0): darkest red → (0,0,0)
        let p0 = result.get_pixel(0, 0).unwrap();
        let (r0, g0, b0) = color::extract_rgb(p0);
        assert_eq!((r0, g0, b0), (0, 0, 0));
        // pixel 2 (gray=255): brightest red → (255,0,0)
        let p2 = result.get_pixel(2, 0).unwrap();
        let (r2, g2, b2) = color::extract_rgb(p2);
        assert_eq!((r2, g2, b2), (255, 0, 0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_colorize_gray_to_colormap() {
        // 8bpp gray colorized with cmapflag=true → 8bpp cmapped output
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 128);
        pm.set_pixel_unchecked(2, 0, 255);
        let pix: Pix = pm.into();

        let result = pix
            .colorize_gray(color::compose_rgb(0, 0, 255), true)
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.has_colormap());
        // Colormap should have 256 entries
        let cmap = result.colormap().unwrap();
        assert_eq!(cmap.len(), 256);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_colorize_gray_from_cmapped() {
        // Colormapped input is first converted to gray, then colorized
        use crate::colormap::PixColormap;
        let pix = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(100, 100, 100, 255).unwrap();
        cmap.add_rgba(200, 200, 200, 255).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        let pix: Pix = pm.into();

        let result = pix
            .colorize_gray(color::compose_rgb(255, 128, 0), false)
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_colorize_gray_invalid_depth() {
        // 32bpp without colormap → error
        let pix = Pix::new(4, 4, PixelDepth::Bit32).unwrap();
        assert!(
            pix.colorize_gray(color::compose_rgb(255, 0, 0), false)
                .is_err()
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_gray_to_false_color_8bpp() {
        // 8bpp gray → 8bpp with false color colormap
        let pix = Pix::new(256, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for i in 0..256u32 {
            pm.set_pixel_unchecked(i, 0, i);
        }
        let pix: Pix = pm.into();

        let result = pix.convert_gray_to_false_color(1.0).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.has_colormap());
        let cmap = result.colormap().unwrap();
        assert_eq!(cmap.len(), 256);
        // Pixel values are unchanged (only colormap is attached)
        for i in 0..256u32 {
            assert_eq!(result.get_pixel(i, 0), Some(i));
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_gray_to_false_color_16bpp() {
        // 16bpp gray → 8bpp with false color colormap (MS byte used)
        let pix = Pix::new(4, 1, PixelDepth::Bit16).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0x0000); // → gray 0
        pm.set_pixel_unchecked(1, 0, 0x0100); // → gray 1 (MS byte)
        pm.set_pixel_unchecked(2, 0, 0xFF00); // → gray 255
        pm.set_pixel_unchecked(3, 0, 0x8000); // → gray 128
        let pix: Pix = pm.into();

        let result = pix.convert_gray_to_false_color(1.0).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(result.has_colormap());
        assert_eq!(result.get_pixel(0, 0), Some(0));
        assert_eq!(result.get_pixel(2, 0), Some(255));
        assert_eq!(result.get_pixel(3, 0), Some(128));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_gray_to_false_color_invalid_depth() {
        let pix = Pix::new(4, 4, PixelDepth::Bit32).unwrap();
        assert!(pix.convert_gray_to_false_color(1.0).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_cmap_to_1_basic() {
        // Create a colormapped image with clear dark (FG) and light (BG) colors
        use crate::colormap::PixColormap;
        // 1bpp colormapped: dark color at index 0, light at index 1
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(10, 10, 10, 255).unwrap(); // index 0: very dark
        cmap.add_rgba(240, 240, 240, 255).unwrap(); // index 1: very light

        let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        pm.set_pixel_unchecked(0, 0, 0); // dark
        pm.set_pixel_unchecked(1, 0, 1); // light
        pm.set_pixel_unchecked(2, 0, 0); // dark
        pm.set_pixel_unchecked(3, 0, 1); // light
        let pix: Pix = pm.into();

        let result = pix.convert_cmap_to_1().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert!(!result.has_colormap());
        // Dark colors (index 0) should be FG (1-bit set), light should be BG (0)
        assert_eq!(result.get_pixel(0, 0), Some(1)); // dark → FG
        assert_eq!(result.get_pixel(1, 0), Some(0)); // light → BG
        assert_eq!(result.get_pixel(2, 0), Some(1)); // dark → FG
        assert_eq!(result.get_pixel(3, 0), Some(0)); // light → BG
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_cmap_to_1_requires_colormap() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        assert!(pix.convert_cmap_to_1().is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_for_ps_wrap_1bpp_passthrough() {
        // 1bpp → clone (no change)
        let pix = Pix::new(8, 4, PixelDepth::Bit1).unwrap();
        let result = pix.convert_for_ps_wrap().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_for_ps_wrap_32bpp_passthrough() {
        // 32bpp → clone (no change)
        let pix = Pix::new(4, 4, PixelDepth::Bit32).unwrap();
        let result = pix.convert_for_ps_wrap().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_for_ps_wrap_2bpp_no_cmap() {
        // 2bpp without cmap → convert to 8bpp gray
        let pix = Pix::new(4, 1, PixelDepth::Bit2).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 1);
        pm.set_pixel_unchecked(2, 0, 2);
        pm.set_pixel_unchecked(3, 0, 3);
        let pix: Pix = pm.into();

        let result = pix.convert_for_ps_wrap().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert!(!result.has_colormap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_for_ps_wrap_8bpp_no_cmap() {
        // 8bpp without cmap → stays 8bpp gray (remove_colormap on BasedOnSrc with no cmap = identity)
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 128);
        let pix: Pix = pm.into();

        let result = pix.convert_for_ps_wrap().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convert_for_ps_wrap_16bpp() {
        // 16bpp → 8bpp using MS byte
        let pix = Pix::new(4, 1, PixelDepth::Bit16).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0xAB00);
        pm.set_pixel_unchecked(1, 0, 0x0000);
        let pix: Pix = pm.into();

        let result = pix.convert_for_ps_wrap().unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.get_pixel(0, 0), Some(0xAB));
        assert_eq!(result.get_pixel(1, 0), Some(0x00));
    }
}
