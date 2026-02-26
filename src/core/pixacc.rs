//! Pixel accumulator for image averaging
//!
//! `PixAcc` stores a 32-bit accumulator image used for summing and averaging
//! multiple images. It supports addition, subtraction, and scalar multiplication,
//! making it useful for computing mean images or weighted combinations.
//!
//! # Reference
//!
//! Based on Leptonica's `pixacc.c`.

use crate::core::{Error, Pix, PixMut, PixelDepth, Result};

/// Pixel accumulator for image averaging.
///
/// Internally stores a 32-bit image. An optional offset is applied when
/// `negflag` is set, allowing the accumulator to handle subtraction
/// without underflow.
pub struct PixAcc {
    /// Width of the accumulator
    w: u32,
    /// Height of the accumulator
    h: u32,
    /// Offset added to all pixel values (0 or 0x40000000)
    offset: u32,
    /// Internal 32-bit accumulator image
    pix: PixMut,
}

/// Offset used to handle negative intermediate values
const PIXACC_OFFSET: u32 = 0x40000000;

impl PixAcc {
    /// Create a new pixel accumulator.
    ///
    /// # Arguments
    ///
    /// * `w` - Width
    /// * `h` - Height
    /// * `negflag` - If true, apply an offset to support negative values
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixaccCreate()`
    pub fn create(w: u32, h: u32, negflag: bool) -> Result<Self> {
        let mut pix = Pix::new(w, h, PixelDepth::Bit32)?.to_mut();
        let offset = if negflag { PIXACC_OFFSET } else { 0 };
        if negflag {
            // Initialize all pixels to offset
            for y in 0..h {
                for x in 0..w {
                    pix.set_pixel_unchecked(x, y, offset);
                }
            }
        }
        Ok(Self { w, h, offset, pix })
    }

    /// Create a pixel accumulator from an existing Pix.
    ///
    /// The input image is converted to 32-bit and used as the initial state.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixaccCreateFromPix()`
    pub fn create_from_pix(src: &Pix, negflag: bool) -> Result<Self> {
        let w = src.width();
        let h = src.height();
        let offset = if negflag { PIXACC_OFFSET } else { 0 };
        let mut pix = Pix::new(w, h, PixelDepth::Bit32)?.to_mut();

        for y in 0..h {
            for x in 0..w {
                let val = src.get_pixel(x, y).unwrap_or(0);
                pix.set_pixel_unchecked(x, y, val + offset);
            }
        }
        Ok(Self { w, h, offset, pix })
    }

    /// Finalize the accumulator to an output Pix.
    ///
    /// Subtracts the offset and clamps values to the output depth range.
    ///
    /// # Arguments
    ///
    /// * `outdepth` - Output bit depth (8, 16, or 32)
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixaccFinal()`
    pub fn finish(&self, outdepth: PixelDepth) -> Result<Pix> {
        let max_val = outdepth.max_value();
        let mut out = Pix::new(self.w, self.h, outdepth)?.to_mut();

        for y in 0..self.h {
            for x in 0..self.w {
                let raw = self.pix.get_pixel(x, y).unwrap_or(0);
                let val = raw.saturating_sub(self.offset);
                let clamped = val.min(max_val);
                out.set_pixel_unchecked(x, y, clamped);
            }
        }
        Ok(out.into())
    }

    /// Get the current state as a Pix (including offset).
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixaccGetPix()`
    pub fn get_pix(&self) -> Pix {
        let pix: Pix = Pix::new(self.w, self.h, PixelDepth::Bit32).unwrap();
        let mut dst = pix.to_mut();
        for y in 0..self.h {
            for x in 0..self.w {
                let v = self.pix.get_pixel(x, y).unwrap_or(0);
                dst.set_pixel_unchecked(x, y, v);
            }
        }
        dst.into()
    }

    /// Get the offset value.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Add a Pix to the accumulator.
    ///
    /// Each pixel value of `src` is added to the corresponding accumulator pixel.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixaccAdd()`
    pub fn add(&mut self, src: &Pix) -> Result<()> {
        if src.width() != self.w || src.height() != self.h {
            return Err(Error::IncompatibleSizes(
                self.w,
                self.h,
                src.width(),
                src.height(),
            ));
        }
        for y in 0..self.h {
            for x in 0..self.w {
                let acc = self.pix.get_pixel(x, y).unwrap_or(0);
                let val = src.get_pixel(x, y).unwrap_or(0);
                self.pix.set_pixel_unchecked(x, y, acc.saturating_add(val));
            }
        }
        Ok(())
    }

    /// Subtract a Pix from the accumulator.
    ///
    /// Each pixel value of `src` is subtracted from the corresponding accumulator pixel.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixaccSubtract()`
    pub fn subtract(&mut self, src: &Pix) -> Result<()> {
        if src.width() != self.w || src.height() != self.h {
            return Err(Error::IncompatibleSizes(
                self.w,
                self.h,
                src.width(),
                src.height(),
            ));
        }
        for y in 0..self.h {
            for x in 0..self.w {
                let acc = self.pix.get_pixel(x, y).unwrap_or(0);
                let val = src.get_pixel(x, y).unwrap_or(0);
                self.pix.set_pixel_unchecked(x, y, acc.saturating_sub(val));
            }
        }
        Ok(())
    }

    /// Multiply all accumulator pixels by a constant.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixaccMultConst()`
    pub fn mult_const(&mut self, factor: f32) -> Result<()> {
        let offset_f = self.offset as f64;
        for y in 0..self.h {
            for x in 0..self.w {
                let raw = self.pix.get_pixel(x, y).unwrap_or(0);
                let centered = raw as f64 - offset_f;
                let scaled = centered * factor as f64 + offset_f;
                let clamped = scaled.round().clamp(0.0, u32::MAX as f64) as u32;
                self.pix.set_pixel_unchecked(x, y, clamped);
            }
        }
        Ok(())
    }

    /// Multiply a Pix by a constant and add to the accumulator.
    ///
    /// Equivalent to `acc[x,y] += factor * src[x,y]`.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixaccMultConstAccumulate()`
    pub fn mult_const_accumulate(&mut self, src: &Pix, factor: f32) -> Result<()> {
        if src.width() != self.w || src.height() != self.h {
            return Err(Error::IncompatibleSizes(
                self.w,
                self.h,
                src.width(),
                src.height(),
            ));
        }
        for y in 0..self.h {
            for x in 0..self.w {
                let acc = self.pix.get_pixel(x, y).unwrap_or(0);
                let val = src.get_pixel(x, y).unwrap_or(0);
                let delta = (val as f64 * factor as f64).round() as i64;
                let new_val = (acc as i64 + delta).clamp(0, u32::MAX as i64) as u32;
                self.pix.set_pixel_unchecked(x, y, new_val);
            }
        }
        Ok(())
    }
}
