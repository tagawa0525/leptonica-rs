//! Image blending functions
//!
//! This module provides functions for blending/compositing images:
//!
//! - Alpha blending (`blend_color`)
//! - Grayscale blending (`blend_gray`)
//! - Mask-based blending (`blend_mask`)
//! - Multiply, Screen, Overlay, Hard Light blend modes
//! - Gray mask compositing (`blend_with_gray_mask`)
//!
//! These correspond to Leptonica's blend.c functions including
//! pixBlendColor, pixBlendGray, pixBlendMask, and pixBlendHardLight.

use super::graphics::Color;
use super::{Pix, PixMut, PixelDepth};
use crate::color;
use crate::error::{Error, Result};

/// Mask blend type for 1-bit mask blending
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskBlendType {
    /// Blend with inverse: p -> (1-f)*p + f*(1-p)
    /// Pixels under mask move toward their inverse
    WithInverse,
    /// Blend to white: p -> p + f*(1-p)
    /// Pixels under mask fade toward white
    ToWhite,
    /// Blend to black: p -> (1-f)*p
    /// Pixels under mask fade toward black
    ToBlack,
}

/// Gray blend type for grayscale blending
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrayBlendType {
    /// Standard gray blend: linear interpolation
    /// result = (1-f)*base + f*gray
    Gray,
    /// Gray with inverse: blend toward inverse based on gray value
    /// d -> d + f * (0.5 - d) * (1 - c)
    GrayWithInverse,
}

/// Fade type for [`Pix::fade_with_gray`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeWithGrayType {
    /// Fade pixels toward white based on the gray blender value
    ToWhite,
    /// Fade pixels toward black based on the gray blender value
    ToBlack,
}

/// Direction from which fading originates in [`PixMut::linear_edge_fade`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeDirection {
    /// Fade from the left edge inward
    FromLeft,
    /// Fade from the right edge inward
    FromRight,
    /// Fade from the top edge inward
    FromTop,
    /// Fade from the bottom edge inward
    FromBottom,
}

/// Target photometry for edge fading in [`PixMut::linear_edge_fade`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeTarget {
    /// Fade edges toward white
    ToWhite,
    /// Fade edges toward black
    ToBlack,
}

/// Blend mode for standard blend operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Normal alpha blending: result = (1-f)*base + f*blend
    Normal,
    /// Multiply: result = base * blend / 255
    /// Darkens the image; black stays black, white is transparent
    Multiply,
    /// Screen: result = 255 - (255-base)*(255-blend)/255
    /// Lightens the image; white stays white, black is transparent
    Screen,
    /// Overlay: combination of multiply and screen
    /// Preserves highlights and shadows of base
    Overlay,
    /// Hard light: like overlay but based on blend layer
    HardLight,
}

impl Pix {
    /// Blend a color image onto this image.
    ///
    /// Performs linear interpolation between base and blend images:
    /// `result = (1 - fract) * self + fract * blend`
    ///
    /// # Arguments
    ///
    /// * `blend` - The image to blend onto self
    /// * `x`, `y` - Position of blend image relative to self (can be negative)
    /// * `fract` - Blending fraction (0.0 = all self, 1.0 = all blend)
    ///
    /// # Returns
    ///
    /// New blended image with same dimensions as self.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - self is 1-bit (binary images not supported for color blend)
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let base = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
    /// let overlay = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
    /// let result = base.blend_color(&overlay, 25, 25, 0.5).unwrap();
    /// ```
    pub fn blend_color(&self, blend: &Pix, x: i32, y: i32, fract: f32) -> Result<Pix> {
        if self.depth() == PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(1));
        }
        if self.depth() != blend.depth() {
            return Err(Error::IncompatibleDepths(
                self.depth().bits(),
                blend.depth().bits(),
            ));
        }

        let fract = fract.clamp(0.0, 1.0);

        // Create output as copy of self
        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();

        let base_w = self.width() as i32;
        let base_h = self.height() as i32;
        let blend_w = blend.width() as i32;
        let blend_h = blend.height() as i32;

        // Process each pixel in the blend image
        for by in 0..blend_h {
            let dy = by + y;
            if dy < 0 || dy >= base_h {
                continue;
            }

            for bx in 0..blend_w {
                let dx = bx + x;
                if dx < 0 || dx >= base_w {
                    continue;
                }

                let base_pixel = self.get_pixel(dx as u32, dy as u32).unwrap();
                let blend_pixel = blend.get_pixel(bx as u32, by as u32).unwrap();

                let result_pixel = match self.depth() {
                    PixelDepth::Bit8 => {
                        let bp = (blend_pixel & 0xFF) as f32;
                        let base_val = (base_pixel & 0xFF) as f32;
                        let result_val = (1.0 - fract) * base_val + fract * bp;
                        (result_val.round() as u32).min(255)
                    }
                    PixelDepth::Bit32 => {
                        let (br, bg, bb) = color::extract_rgb(base_pixel);
                        let (or, og, ob) = color::extract_rgb(blend_pixel);

                        let rr = ((1.0 - fract) * br as f32 + fract * or as f32).round() as u8;
                        let rg = ((1.0 - fract) * bg as f32 + fract * og as f32).round() as u8;
                        let rb = ((1.0 - fract) * bb as f32 + fract * ob as f32).round() as u8;

                        color::compose_rgb(rr, rg, rb)
                    }
                    _ => {
                        // For other depths, blend in pixel value space
                        let base_val = base_pixel as f32;
                        let blend_val = blend_pixel as f32;
                        let result_val = (1.0 - fract) * base_val + fract * blend_val;
                        (result_val.round() as u32).min(self.depth().max_value())
                    }
                };

                result_mut.set_pixel_unchecked(dx as u32, dy as u32, result_pixel);
            }
        }

        Ok(result_mut.into())
    }

    /// Blend using a grayscale image as blender.
    ///
    /// # Arguments
    ///
    /// * `blend` - Grayscale image to use as blender
    /// * `x`, `y` - Position of blend relative to self
    /// * `fract` - Blending fraction
    /// * `blend_type` - Type of gray blending to perform
    ///
    /// # Returns
    ///
    /// New blended image.
    pub fn blend_gray(
        &self,
        blend: &Pix,
        x: i32,
        y: i32,
        fract: f32,
        blend_type: GrayBlendType,
    ) -> Result<Pix> {
        if self.depth() == PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(1));
        }

        let fract = fract.clamp(0.0, 1.0);

        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();

        let base_w = self.width() as i32;
        let base_h = self.height() as i32;
        let blend_w = blend.width() as i32;
        let blend_h = blend.height() as i32;

        for by in 0..blend_h {
            let dy = by + y;
            if dy < 0 || dy >= base_h {
                continue;
            }

            for bx in 0..blend_w {
                let dx = bx + x;
                if dx < 0 || dx >= base_w {
                    continue;
                }

                let base_pixel = self.get_pixel(dx as u32, dy as u32).unwrap();
                let blend_pixel = blend.get_pixel(bx as u32, by as u32).unwrap();

                // Get gray value from blend (convert to 8-bit if needed)
                let gray_val = (blend_pixel & 0xFF) as f32;

                let result_pixel = match self.depth() {
                    PixelDepth::Bit8 => {
                        let base_val = (base_pixel & 0xFF) as f32;
                        let result_val = match blend_type {
                            GrayBlendType::Gray => (1.0 - fract) * base_val + fract * gray_val,
                            GrayBlendType::GrayWithInverse => {
                                // d -> d + f * (128 - d) * (255 - c) / 256
                                let delta = (128.0 - base_val) * (255.0 - gray_val) / 256.0;
                                base_val + fract * delta
                            }
                        };
                        (result_val.round() as u32).clamp(0, 255)
                    }
                    PixelDepth::Bit32 => {
                        let (br, bg, bb) = color::extract_rgb(base_pixel);

                        let (rr, rg, rb) = match blend_type {
                            GrayBlendType::Gray => {
                                let rr =
                                    ((1.0 - fract) * br as f32 + fract * gray_val).round() as u8;
                                let rg =
                                    ((1.0 - fract) * bg as f32 + fract * gray_val).round() as u8;
                                let rb =
                                    ((1.0 - fract) * bb as f32 + fract * gray_val).round() as u8;
                                (rr, rg, rb)
                            }
                            GrayBlendType::GrayWithInverse => {
                                let delta_r = (128.0 - br as f32) * (255.0 - gray_val) / 256.0;
                                let delta_g = (128.0 - bg as f32) * (255.0 - gray_val) / 256.0;
                                let delta_b = (128.0 - bb as f32) * (255.0 - gray_val) / 256.0;

                                let rr =
                                    (br as f32 + fract * delta_r).round().clamp(0.0, 255.0) as u8;
                                let rg =
                                    (bg as f32 + fract * delta_g).round().clamp(0.0, 255.0) as u8;
                                let rb =
                                    (bb as f32 + fract * delta_b).round().clamp(0.0, 255.0) as u8;
                                (rr, rg, rb)
                            }
                        };

                        color::compose_rgb(rr, rg, rb)
                    }
                    _ => base_pixel, // Other depths: no change
                };

                result_mut.set_pixel_unchecked(dx as u32, dy as u32, result_pixel);
            }
        }

        Ok(result_mut.into())
    }

    /// Blend using a 1-bit mask.
    ///
    /// Only pixels where the mask is foreground (1) are affected.
    ///
    /// # Arguments
    ///
    /// * `mask` - 1-bit mask image
    /// * `x`, `y` - Position of mask relative to self
    /// * `fract` - Blending fraction
    /// * `blend_type` - Type of mask blending to perform
    ///
    /// # Returns
    ///
    /// New blended image.
    ///
    /// # Errors
    ///
    /// Returns error if mask is not 1-bit.
    pub fn blend_mask(
        &self,
        mask: &Pix,
        x: i32,
        y: i32,
        fract: f32,
        blend_type: MaskBlendType,
    ) -> Result<Pix> {
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }

        if self.depth() == PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(1));
        }

        let fract = fract.clamp(0.0, 1.0);

        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();

        let base_w = self.width() as i32;
        let base_h = self.height() as i32;
        let mask_w = mask.width() as i32;
        let mask_h = mask.height() as i32;

        for my in 0..mask_h {
            let dy = my + y;
            if dy < 0 || dy >= base_h {
                continue;
            }

            for mx in 0..mask_w {
                let dx = mx + x;
                if dx < 0 || dx >= base_w {
                    continue;
                }

                // Check if mask pixel is set (foreground)
                let mask_val = mask.get_pixel(mx as u32, my as u32).unwrap();
                if mask_val == 0 {
                    continue; // Background - no blending
                }

                let base_pixel = self.get_pixel(dx as u32, dy as u32).unwrap();

                let result_pixel = match self.depth() {
                    PixelDepth::Bit8 => {
                        let p = (base_pixel & 0xFF) as f32 / 255.0;
                        let result_val = match blend_type {
                            MaskBlendType::WithInverse => {
                                // p -> (1-f)*p + f*(1-p) = p + f*(1-2*p)
                                p + fract * (1.0 - 2.0 * p)
                            }
                            MaskBlendType::ToWhite => {
                                // p -> p + f*(1-p)
                                p + fract * (1.0 - p)
                            }
                            MaskBlendType::ToBlack => {
                                // p -> (1-f)*p
                                (1.0 - fract) * p
                            }
                        };
                        ((result_val * 255.0).round() as u32).clamp(0, 255)
                    }
                    PixelDepth::Bit32 => {
                        let (r, g, b) = color::extract_rgb(base_pixel);
                        let pr = r as f32 / 255.0;
                        let pg = g as f32 / 255.0;
                        let pb = b as f32 / 255.0;

                        let (rr, rg, rb) = match blend_type {
                            MaskBlendType::WithInverse => {
                                let rr = pr + fract * (1.0 - 2.0 * pr);
                                let rg = pg + fract * (1.0 - 2.0 * pg);
                                let rb = pb + fract * (1.0 - 2.0 * pb);
                                (rr, rg, rb)
                            }
                            MaskBlendType::ToWhite => {
                                let rr = pr + fract * (1.0 - pr);
                                let rg = pg + fract * (1.0 - pg);
                                let rb = pb + fract * (1.0 - pb);
                                (rr, rg, rb)
                            }
                            MaskBlendType::ToBlack => {
                                let rr = (1.0 - fract) * pr;
                                let rg = (1.0 - fract) * pg;
                                let rb = (1.0 - fract) * pb;
                                (rr, rg, rb)
                            }
                        };

                        let rr = (rr * 255.0).round().clamp(0.0, 255.0) as u8;
                        let rg = (rg * 255.0).round().clamp(0.0, 255.0) as u8;
                        let rb = (rb * 255.0).round().clamp(0.0, 255.0) as u8;

                        color::compose_rgb(rr, rg, rb)
                    }
                    _ => base_pixel,
                };

                result_mut.set_pixel_unchecked(dx as u32, dy as u32, result_pixel);
            }
        }

        Ok(result_mut.into())
    }

    /// Apply a blend mode with another image.
    ///
    /// Blends the entire image using the specified blend mode.
    /// Images must have the same dimensions.
    ///
    /// # Arguments
    ///
    /// * `other` - Image to blend with
    /// * `mode` - Blend mode to use
    /// * `fract` - Blending fraction (for Normal mode; 1.0 for full effect of other modes)
    ///
    /// # Returns
    ///
    /// New blended image.
    pub fn blend(&self, other: &Pix, mode: BlendMode, fract: f32) -> Result<Pix> {
        if self.width() != other.width() || self.height() != other.height() {
            return Err(Error::DimensionMismatch {
                expected: (self.width(), self.height()),
                actual: (other.width(), other.height()),
            });
        }

        match mode {
            BlendMode::Normal => self.blend_color(other, 0, 0, fract),
            BlendMode::Multiply => self.blend_multiply(other),
            BlendMode::Screen => self.blend_screen(other),
            BlendMode::Overlay => self.blend_overlay(other),
            BlendMode::HardLight => self.blend_hard_light(other, fract),
        }
    }

    /// Multiply blend: darker colors darken the image.
    ///
    /// Formula: `result = base * blend / 255`
    ///
    /// Properties:
    /// - Black (0) stays black
    /// - White (255) is transparent (returns base)
    /// - Always darkens or maintains brightness
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let base = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    /// let blend = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    /// let result = base.blend_multiply(&blend).unwrap();
    /// ```
    pub fn blend_multiply(&self, other: &Pix) -> Result<Pix> {
        self.apply_blend_op(other, |base, blend| {
            (base as u32 * blend as u32 / 255) as u8
        })
    }

    /// Screen blend: lighter colors lighten the image.
    ///
    /// Formula: `result = 255 - (255 - base) * (255 - blend) / 255`
    ///
    /// Properties:
    /// - White (255) stays white
    /// - Black (0) is transparent (returns base)
    /// - Always lightens or maintains brightness
    pub fn blend_screen(&self, other: &Pix) -> Result<Pix> {
        self.apply_blend_op(other, |base, blend| {
            255 - ((255 - base as u32) * (255 - blend as u32) / 255) as u8
        })
    }

    /// Overlay blend: combines multiply and screen.
    ///
    /// Formula:
    /// - If base < 128: `result = 2 * base * blend / 255`
    /// - If base >= 128: `result = 255 - 2 * (255 - base) * (255 - blend) / 255`
    ///
    /// Properties:
    /// - Preserves highlights and shadows of base layer
    /// - Increases contrast
    pub fn blend_overlay(&self, other: &Pix) -> Result<Pix> {
        self.apply_blend_op(other, |base, blend| {
            if base < 128 {
                (2 * base as u32 * blend as u32 / 255) as u8
            } else {
                255 - (2 * (255 - base as u32) * (255 - blend as u32) / 255) as u8
            }
        })
    }

    /// Hard light blend.
    ///
    /// Like overlay but the test is based on the blend layer instead of base.
    ///
    /// Formula (with fract=1.0):
    /// - If blend < 128: `result = 2 * base * blend / 255`
    /// - If blend >= 128: `result = 255 - 2 * (255 - base) * (255 - blend) / 255`
    ///
    /// The fract parameter adjusts how far blend is from 128 (neutral gray):
    /// - fract=0: no blending (result = base)
    /// - fract=1: full hard light effect
    pub fn blend_hard_light(&self, other: &Pix, fract: f32) -> Result<Pix> {
        let fract = fract.clamp(0.0, 1.0);

        // Short-circuit: fract == 0 means no blending, return base unchanged
        if fract == 0.0 {
            return Ok(self.deep_clone());
        }

        self.apply_blend_op(other, |base, blend| {
            hard_light_component(base, blend, fract)
        })
    }

    /// Internal helper to apply a per-component blend operation
    fn apply_blend_op<F>(&self, other: &Pix, op: F) -> Result<Pix>
    where
        F: Fn(u8, u8) -> u8,
    {
        if self.width() != other.width() || self.height() != other.height() {
            return Err(Error::DimensionMismatch {
                expected: (self.width(), self.height()),
                actual: (other.width(), other.height()),
            });
        }

        if self.depth() != other.depth() {
            return Err(Error::IncompatibleDepths(
                self.depth().bits(),
                other.depth().bits(),
            ));
        }

        let width = self.width();
        let height = self.height();

        let result = Pix::new(width, height, self.depth())?;
        let mut result_mut = result.try_into_mut().unwrap();

        match self.depth() {
            PixelDepth::Bit8 => {
                for y in 0..height {
                    for x in 0..width {
                        let base = (self.get_pixel(x, y).unwrap() & 0xFF) as u8;
                        let blend = (other.get_pixel(x, y).unwrap() & 0xFF) as u8;
                        let result_val = op(base, blend) as u32;
                        result_mut.set_pixel_unchecked(x, y, result_val);
                    }
                }
            }
            PixelDepth::Bit32 => {
                for y in 0..height {
                    for x in 0..width {
                        let base_pixel = self.get_pixel(x, y).unwrap();
                        let blend_pixel = other.get_pixel(x, y).unwrap();

                        let (br, bg, bb) = color::extract_rgb(base_pixel);
                        let (or, og, ob) = color::extract_rgb(blend_pixel);

                        let rr = op(br, or);
                        let rg = op(bg, og);
                        let rb = op(bb, ob);

                        let result_pixel = color::compose_rgb(rr, rg, rb);
                        result_mut.set_pixel_unchecked(x, y, result_pixel);
                    }
                }
            }
            _ => {
                return Err(Error::UnsupportedDepth(self.depth().bits()));
            }
        }

        Ok(result_mut.into())
    }

    /// Blend using the inverse of the blender pixel values.
    ///
    /// Corresponds to `pixBlendGrayInverse()` in Leptonica's `blend.c`.
    pub fn blend_gray_inverse(&self, other: &Pix, x: i32, y: i32, fract: f32) -> Result<Pix> {
        let fract = fract.clamp(0.0, 1.0);
        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();
        let base_w = self.width() as i32;
        let base_h = self.height() as i32;
        let blend_w = other.width() as i32;
        let blend_h = other.height() as i32;
        for by in 0..blend_h {
            let dy = by + y;
            if dy < 0 || dy >= base_h {
                continue;
            }
            for bx in 0..blend_w {
                let dx = bx + x;
                if dx < 0 || dx >= base_w {
                    continue;
                }
                let dval = (self.get_pixel_unchecked(dx as u32, dy as u32) & 0xFF) as f32;
                let cval = (other.get_pixel_unchecked(bx as u32, by as u32) & 0xFF) as f32;
                let a = (1.0 - fract) * dval + fract * (255.0 - dval);
                let new_val = ((cval * dval + a * (255.0 - cval)) / 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u32;
                result_mut.set_pixel_unchecked(dx as u32, dy as u32, new_val);
            }
        }
        Ok(result_mut.into())
    }

    /// Blend with separate per-channel blending fractions.
    ///
    /// Corresponds to `pixBlendColorByChannel()` in Leptonica's `blend.c`.
    pub fn blend_color_by_channel(
        &self,
        other: &Pix,
        x: i32,
        y: i32,
        rfract: f32,
        gfract: f32,
        bfract: f32,
        transparent: bool,
        transpix: Color,
    ) -> Result<Pix> {
        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();
        let base_w = self.width() as i32;
        let base_h = self.height() as i32;
        let blend_w = other.width() as i32;
        let blend_h = other.height() as i32;
        for by in 0..blend_h {
            let dy = by + y;
            if dy < 0 || dy >= base_h {
                continue;
            }
            for bx in 0..blend_w {
                let dx = bx + x;
                if dx < 0 || dx >= base_w {
                    continue;
                }
                let blend_pixel = other.get_pixel_unchecked(bx as u32, by as u32);
                if transparent {
                    let (br, bg, bb) = color::extract_rgb(blend_pixel);
                    if br == transpix.r && bg == transpix.g && bb == transpix.b {
                        continue;
                    }
                }
                let base_pixel = self.get_pixel_unchecked(dx as u32, dy as u32);
                let (pr, pg, pb) = color::extract_rgb(base_pixel);
                let (cr, cg, cb) = color::extract_rgb(blend_pixel);
                let new_r = blend_component(pr, cr, rfract);
                let new_g = blend_component(pg, cg, gfract);
                let new_b = blend_component(pb, cb, bfract);
                result_mut.set_pixel_unchecked(
                    dx as u32,
                    dy as u32,
                    color::compose_rgb(new_r, new_g, new_b),
                );
            }
        }
        Ok(result_mut.into())
    }

    /// Adaptive gray blend that adjusts based on local pixel values.
    ///
    /// Corresponds to `pixBlendGrayAdapt()` in Leptonica's `blend.c`.
    pub fn blend_gray_adapt(
        &self,
        other: &Pix,
        x: i32,
        y: i32,
        fract: f32,
        shift: i32,
    ) -> Result<Pix> {
        let fract = fract.clamp(0.0, 1.0);
        let base_w = self.width() as i32;
        let base_h = self.height() as i32;
        let blend_w = other.width() as i32;
        let blend_h = other.height() as i32;
        // Determine overlap region in destination coordinates
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = ((x + blend_w).min(base_w)) as u32;
        let y1 = ((y + blend_h).min(base_h)) as u32;
        // Collect destination pixel values in overlap
        let mut vals: Vec<u32> = Vec::new();
        for dy in y0..y1 {
            for dx in x0..x1 {
                vals.push(self.get_pixel_unchecked(dx, dy) & 0xFF);
            }
        }
        if vals.is_empty() {
            return Ok(self.deep_clone());
        }
        vals.sort_unstable();
        let median = vals[vals.len() / 2] as i32;
        let raw_pivot = if median < 128 {
            median + shift
        } else {
            median - shift
        };
        let pivot = raw_pivot.clamp(85, 170) as f32;
        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();
        for by in 0..blend_h {
            let dy = by + y;
            if dy < 0 || dy >= base_h {
                continue;
            }
            for bx in 0..blend_w {
                let dx = bx + x;
                if dx < 0 || dx >= base_w {
                    continue;
                }
                let dval = (self.get_pixel_unchecked(dx as u32, dy as u32) & 0xFF) as f32;
                let cval = (other.get_pixel_unchecked(bx as u32, by as u32) & 0xFF) as f32;
                let new_val = (dval + fract * (pivot - dval) * (1.0 - cval / 256.0))
                    .round()
                    .clamp(0.0, 255.0) as u32;
                result_mut.set_pixel_unchecked(dx as u32, dy as u32, new_val);
            }
        }
        Ok(result_mut.into())
    }

    /// Fade a color or grayscale image using a grayscale blender image.
    ///
    /// Corresponds to `pixFadeWithGray()` in Leptonica's `blend.c`.
    pub fn fade_with_gray(
        &self,
        blender: &Pix,
        factor: f32,
        fade_type: FadeWithGrayType,
    ) -> Result<Pix> {
        let factor_norm = (factor / 255.0).clamp(0.0, 1.0);
        let result = self.deep_clone();
        let mut result_mut = result.try_into_mut().unwrap();
        let w = self.width().min(blender.width());
        let h = self.height().min(blender.height());
        for y in 0..h {
            for x in 0..w {
                let valb = (blender.get_pixel_unchecked(x, y) & 0xFF) as f32;
                let fract = (factor_norm * valb / 255.0).clamp(0.0, 1.0);
                let src_pixel = self.get_pixel_unchecked(x, y);
                let new_pixel = match self.depth() {
                    PixelDepth::Bit8 => {
                        let p = (src_pixel & 0xFF) as f32;
                        match fade_type {
                            FadeWithGrayType::ToWhite => {
                                (p + fract * (255.0 - p)).round().clamp(0.0, 255.0) as u32
                            }
                            FadeWithGrayType::ToBlack => {
                                (p * (1.0 - fract)).round().clamp(0.0, 255.0) as u32
                            }
                        }
                    }
                    PixelDepth::Bit32 => {
                        let (r, g, b) = color::extract_rgb(src_pixel);
                        let (nr, ng, nb) = match fade_type {
                            FadeWithGrayType::ToWhite => (
                                (r as f32 + fract * (255.0 - r as f32))
                                    .round()
                                    .clamp(0.0, 255.0) as u8,
                                (g as f32 + fract * (255.0 - g as f32))
                                    .round()
                                    .clamp(0.0, 255.0) as u8,
                                (b as f32 + fract * (255.0 - b as f32))
                                    .round()
                                    .clamp(0.0, 255.0) as u8,
                            ),
                            FadeWithGrayType::ToBlack => (
                                (r as f32 * (1.0 - fract)).round().clamp(0.0, 255.0) as u8,
                                (g as f32 * (1.0 - fract)).round().clamp(0.0, 255.0) as u8,
                                (b as f32 * (1.0 - fract)).round().clamp(0.0, 255.0) as u8,
                            ),
                        };
                        color::compose_rgb(nr, ng, nb)
                    }
                    _ => src_pixel,
                };
                result_mut.set_pixel_unchecked(x, y, new_pixel);
            }
        }
        Ok(result_mut.into())
    }

    /// Multiply each pixel by a color factor (component-wise).
    ///
    /// Corresponds to `pixMultiplyByColor()` in Leptonica's `blend.c`.
    pub fn multiply_by_color(&self, color: Color) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit32)?;
        let mut result_mut = result.try_into_mut().unwrap();
        let fr = color.r as f32 / 255.0;
        let fg = color.g as f32 / 255.0;
        let fb = color.b as f32 / 255.0;
        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let (r, g, b) = crate::color::extract_rgb(pixel);
                let nr = (r as f32 * fr).round().clamp(0.0, 255.0) as u8;
                let ng = (g as f32 * fg).round().clamp(0.0, 255.0) as u8;
                let nb = (b as f32 * fb).round().clamp(0.0, 255.0) as u8;
                result_mut.set_pixel_unchecked(x, y, crate::color::compose_rgb(nr, ng, nb));
            }
        }
        Ok(result_mut.into())
    }

    /// Alpha-blend a 32bpp RGBA image against a uniform background color.
    ///
    /// Corresponds to `pixAlphaBlendUniform()` in Leptonica's `blend.c`.
    pub fn alpha_blend_uniform(&self, bg_color: u32) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width();
        let h = self.height();
        let (bg_r, bg_g, bg_b) = color::extract_rgb(bg_color);
        let result = Pix::new(w, h, PixelDepth::Bit32)?;
        let mut result_mut = result.try_into_mut().unwrap();
        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let (r, g, b, a) = color::extract_rgba(pixel);
                let alpha = a as f32 / 255.0;
                let nr = (alpha * r as f32 + (1.0 - alpha) * bg_r as f32)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                let ng = (alpha * g as f32 + (1.0 - alpha) * bg_g as f32)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                let nb = (alpha * b as f32 + (1.0 - alpha) * bg_b as f32)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                result_mut.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
            }
        }
        Ok(result_mut.into())
    }

    /// Generate an alpha channel for the image based on gray values.
    ///
    /// Corresponds to `pixAddAlphaToBlend()` in Leptonica's `blend.c`.
    pub fn add_alpha_to_blend(&self, fract: f32, invert: bool) -> Result<Pix> {
        let fract = fract.clamp(0.0, 1.0);
        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit32)?;
        let mut result_mut = result.try_into_mut().unwrap();
        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let (mut r, mut g, mut b) = match self.depth() {
                    PixelDepth::Bit8 => {
                        let v = (pixel & 0xFF) as u8;
                        (v, v, v)
                    }
                    PixelDepth::Bit32 => color::extract_rgb(pixel),
                    _ => return Err(Error::UnsupportedDepth(self.depth().bits())),
                };
                if invert {
                    r = 255 - r;
                    g = 255 - g;
                    b = 255 - b;
                }
                let gray = (r as u32 + g as u32 + b as u32) / 3;
                let alpha = ((255 - gray) as f32 * fract).round().clamp(0.0, 255.0) as u8;
                result_mut.set_pixel_unchecked(x, y, color::compose_rgba(r, g, b, alpha));
            }
        }
        Ok(result_mut.into())
    }
}

impl PixMut {
    /// Blend a colormapped source image onto this colormapped image in-place.
    ///
    /// Only pixels in the source with index > `sindex` are blended.
    ///
    /// Corresponds to `pixBlendCmap()` in Leptonica's `blend.c`.
    pub fn blend_cmap(&mut self, other: &Pix, x: i32, y: i32, sindex: usize) -> Result<()> {
        if !self.has_colormap() || !other.has_colormap() {
            return Err(Error::InvalidParameter(
                "blend_cmap: both images must have colormaps".to_string(),
            ));
        }
        let w_dst = self.width() as i32;
        let h_dst = self.height() as i32;
        let w_src = other.width() as i32;
        let h_src = other.height() as i32;
        let src_len = other.colormap().map(|c| c.len()).unwrap_or(0);
        // Build LUT: source colormap index -> destination colormap index
        let mut new_dst_cmap = self.colormap().unwrap().clone();
        let mut lut = vec![0u32; src_len.max(1)];
        for i in 0..src_len {
            if let Some(src_cmap) = other.colormap() {
                if let Some((r, g, b)) = src_cmap.get_rgb(i) {
                    let idx = if let Some(existing) = new_dst_cmap.get_index(r, g, b) {
                        existing
                    } else {
                        new_dst_cmap.add_rgb(r, g, b).unwrap_or(0)
                    };
                    lut[i] = idx as u32;
                }
            }
        }
        self.set_colormap(Some(new_dst_cmap))?;
        // Substitute pixels
        for by in 0..h_src {
            let dy = by + y;
            if dy < 0 || dy >= h_dst {
                continue;
            }
            for bx in 0..w_src {
                let dx = bx + x;
                if dx < 0 || dx >= w_dst {
                    continue;
                }
                let src_idx = other.get_pixel_unchecked(bx as u32, by as u32) as usize;
                if src_idx > sindex && src_idx < src_len {
                    self.set_pixel_unchecked(dx as u32, dy as u32, lut[src_idx]);
                }
            }
        }
        Ok(())
    }

    /// Apply a linear fade from one edge of the image inward.
    ///
    /// Corresponds to `pixLinearEdgeFade()` in Leptonica's `blend.c`.
    pub fn linear_edge_fade(
        &mut self,
        dir: FadeDirection,
        fadeto: FadeTarget,
        distfract: f32,
        maxfade: f32,
    ) -> Result<()> {
        let w = self.width();
        let h = self.height();
        let limit = match fadeto {
            FadeTarget::ToWhite => 255.0f32,
            FadeTarget::ToBlack => 0.0f32,
        };
        let dim = match dir {
            FadeDirection::FromLeft | FadeDirection::FromRight => w,
            FadeDirection::FromTop | FadeDirection::FromBottom => h,
        };
        let range = (distfract * dim as f32) as i32;
        if range == 0 {
            return Ok(());
        }
        let maxfade = maxfade.clamp(0.0, 1.0);
        let slope = maxfade / range as f32;
        for y in 0..h {
            for x in 0..w {
                let dist = match dir {
                    FadeDirection::FromLeft => x as i32,
                    FadeDirection::FromRight => w as i32 - 1 - x as i32,
                    FadeDirection::FromTop => y as i32,
                    FadeDirection::FromBottom => h as i32 - 1 - y as i32,
                };
                if dist >= range {
                    continue;
                }
                let del = (maxfade - slope * dist as f32).clamp(0.0, 1.0);
                let pixel = self.get_pixel_unchecked(x, y);
                match self.depth() {
                    PixelDepth::Bit8 => {
                        let val = (pixel & 0xFF) as f32;
                        let new_val = (val + (limit - val) * del).round().clamp(0.0, 255.0) as u32;
                        self.set_pixel_unchecked(x, y, new_val);
                    }
                    PixelDepth::Bit32 => {
                        let (r, g, b) = color::extract_rgb(pixel);
                        let new_r = (r as f32 + (limit - r as f32) * del)
                            .round()
                            .clamp(0.0, 255.0) as u8;
                        let new_g = (g as f32 + (limit - g as f32) * del)
                            .round()
                            .clamp(0.0, 255.0) as u8;
                        let new_b = (b as f32 + (limit - b as f32) * del)
                            .round()
                            .clamp(0.0, 255.0) as u8;
                        self.set_pixel_unchecked(x, y, color::compose_rgb(new_r, new_g, new_b));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

/// Blend a single channel component by fract.
///
/// - `fract < 0.0`: return min(p, c)
/// - `fract > 1.0`: return max(p, c)
/// - `0.0 <= fract <= 1.0`: linear interpolation
fn blend_component(p: u8, c: u8, fract: f32) -> u8 {
    if fract < 0.0 {
        p.min(c)
    } else if fract > 1.0 {
        p.max(c)
    } else {
        ((1.0 - fract) * p as f32 + fract * c as f32)
            .round()
            .clamp(0.0, 255.0) as u8
    }
}

/// Hard light component calculation
fn hard_light_component(base: u8, blend: u8, fract: f32) -> u8 {
    if blend < 128 {
        // Adjust blend toward 128 based on fract
        let adjusted = 128.0 - fract * (128.0 - blend as f32);
        let result = (base as f32 * adjusted / 128.0).round();
        result.clamp(0.0, 255.0) as u8
    } else {
        let adjusted = 128.0 + fract * (blend as f32 - 128.0);
        let result = 255.0 - ((255.0 - adjusted) * (255.0 - base as f32) / 128.0);
        result.clamp(0.0, 255.0) as u8
    }
}

/// Blend two images using a grayscale mask as alpha.
///
/// The mask values determine the opacity of the overlay:
/// - 0 = fully transparent (result = base)
/// - 255 = fully opaque (result = overlay)
///
/// # Arguments
///
/// * `base` - Background image
/// * `overlay` - Foreground image to blend on top
/// * `mask` - 8-bit grayscale alpha mask (0 = transparent, 255 = opaque)
/// * `x`, `y` - Position of overlay and mask relative to base
///
/// # Returns
///
/// New blended image with same dimensions as base.
///
/// # Errors
///
/// Returns error if mask is not 8-bit grayscale.
pub fn blend_with_gray_mask(base: &Pix, overlay: &Pix, mask: &Pix, x: i32, y: i32) -> Result<Pix> {
    if mask.depth() != PixelDepth::Bit8 {
        return Err(Error::UnsupportedDepth(mask.depth().bits()));
    }
    if base.depth() != overlay.depth() {
        return Err(Error::IncompatibleDepths(
            base.depth().bits(),
            overlay.depth().bits(),
        ));
    }

    let result = base.deep_clone();
    let mut result_mut = result.try_into_mut().unwrap();

    let base_w = base.width() as i32;
    let base_h = base.height() as i32;
    let overlay_w = overlay.width() as i32;
    let overlay_h = overlay.height() as i32;
    let mask_w = mask.width() as i32;
    let mask_h = mask.height() as i32;

    // Use minimum of overlay and mask dimensions
    let blend_w = overlay_w.min(mask_w);
    let blend_h = overlay_h.min(mask_h);

    for by in 0..blend_h {
        let dy = by + y;
        if dy < 0 || dy >= base_h {
            continue;
        }

        for bx in 0..blend_w {
            let dx = bx + x;
            if dx < 0 || dx >= base_w {
                continue;
            }

            // Get mask value (alpha)
            let alpha = (mask.get_pixel(bx as u32, by as u32).unwrap() & 0xFF) as f32 / 255.0;

            if alpha == 0.0 {
                continue; // Fully transparent, skip
            }

            let base_pixel = base.get_pixel(dx as u32, dy as u32).unwrap();
            let overlay_pixel = overlay.get_pixel(bx as u32, by as u32).unwrap();

            let result_pixel = match base.depth() {
                PixelDepth::Bit8 => {
                    let base_val = (base_pixel & 0xFF) as f32;
                    let overlay_val = (overlay_pixel & 0xFF) as f32;
                    let result_val = (1.0 - alpha) * base_val + alpha * overlay_val;
                    (result_val.round() as u32).min(255)
                }
                PixelDepth::Bit32 => {
                    let (br, bg, bb) = color::extract_rgb(base_pixel);
                    let (or, og, ob) = color::extract_rgb(overlay_pixel);

                    let rr = ((1.0 - alpha) * br as f32 + alpha * or as f32).round() as u8;
                    let rg = ((1.0 - alpha) * bg as f32 + alpha * og as f32).round() as u8;
                    let rb = ((1.0 - alpha) * bb as f32 + alpha * ob as f32).round() as u8;

                    color::compose_rgb(rr, rg, rb)
                }
                _ => base_pixel,
            };

            result_mut.set_pixel_unchecked(dx as u32, dy as u32, result_pixel);
        }
    }

    Ok(result_mut.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_color_fract_0() {
        // fract=0 should return original
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(5, 5, 100).unwrap();
        let base: Pix = base_mut.into();

        let blend = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut blend_mut = blend.to_mut();
        blend_mut.set_pixel(5, 5, 200).unwrap();
        let blend: Pix = blend_mut.into();

        let result = base.blend_color(&blend, 0, 0, 0.0).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(100));
    }

    #[test]
    fn test_blend_color_fract_1() {
        // fract=1 should return blend image
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(5, 5, 100).unwrap();
        let base: Pix = base_mut.into();

        let blend = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut blend_mut = blend.to_mut();
        blend_mut.set_pixel(5, 5, 200).unwrap();
        let blend: Pix = blend_mut.into();

        let result = base.blend_color(&blend, 0, 0, 1.0).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(200));
    }

    #[test]
    fn test_blend_color_fract_half() {
        // fract=0.5 should average
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(5, 5, 100).unwrap();
        let base: Pix = base_mut.into();

        let blend = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut blend_mut = blend.to_mut();
        blend_mut.set_pixel(5, 5, 200).unwrap();
        let blend: Pix = blend_mut.into();

        let result = base.blend_color(&blend, 0, 0, 0.5).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(150)); // (100+200)/2
    }

    #[test]
    fn test_blend_color_rgb() {
        use crate::color::compose_rgb;

        let base = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut base_mut = base.to_mut();
        base_mut
            .set_pixel(0, 0, compose_rgb(100, 100, 100))
            .unwrap();
        let base: Pix = base_mut.into();

        let blend = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut blend_mut = blend.to_mut();
        blend_mut
            .set_pixel(0, 0, compose_rgb(200, 50, 150))
            .unwrap();
        let blend: Pix = blend_mut.into();

        let result = base.blend_color(&blend, 0, 0, 0.5).unwrap();
        let (r, g, b) = result.get_rgb(0, 0).unwrap();

        assert_eq!(r, 150); // (100+200)/2
        assert_eq!(g, 75); // (100+50)/2
        assert_eq!(b, 125); // (100+150)/2
    }

    #[test]
    fn test_blend_multiply_black() {
        // Multiply with black = black
        let base = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(0, 0, 200).unwrap();
        let base: Pix = base_mut.into();

        let black = Pix::new(1, 1, PixelDepth::Bit8).unwrap(); // 0

        let result = base.blend_multiply(&black).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(0));
    }

    #[test]
    fn test_blend_multiply_white() {
        // Multiply with white = original
        let base = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(0, 0, 200).unwrap();
        let base: Pix = base_mut.into();

        let white = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut white_mut = white.to_mut();
        white_mut.set_pixel(0, 0, 255).unwrap();
        let white: Pix = white_mut.into();

        let result = base.blend_multiply(&white).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(200));
    }

    #[test]
    fn test_blend_screen_black() {
        // Screen with black = original
        let base = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(0, 0, 200).unwrap();
        let base: Pix = base_mut.into();

        let black = Pix::new(1, 1, PixelDepth::Bit8).unwrap(); // 0

        let result = base.blend_screen(&black).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(200));
    }

    #[test]
    fn test_blend_screen_white() {
        // Screen with white = white
        let base = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(0, 0, 100).unwrap();
        let base: Pix = base_mut.into();

        let white = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut white_mut = white.to_mut();
        white_mut.set_pixel(0, 0, 255).unwrap();
        let white: Pix = white_mut.into();

        let result = base.blend_screen(&white).unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(255));
    }

    #[test]
    fn test_blend_overlay() {
        // Overlay with 50% gray should be mostly neutral
        let base = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(0, 0, 100).unwrap();
        let base: Pix = base_mut.into();

        let gray = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut gray_mut = gray.to_mut();
        gray_mut.set_pixel(0, 0, 128).unwrap();
        let gray: Pix = gray_mut.into();

        let result = base.blend_overlay(&gray).unwrap();
        // 100 < 128, so multiply mode: 2 * 100 * 128 / 255 = 100
        assert_eq!(result.get_pixel(0, 0), Some(100));
    }

    #[test]
    fn test_blend_mask_to_white() {
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(5, 5, 100).unwrap();
        let base: Pix = base_mut.into();

        let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut mask_mut = mask.to_mut();
        mask_mut.set_pixel(5, 5, 1).unwrap(); // Set mask at (5,5)
        let mask: Pix = mask_mut.into();

        let result = base
            .blend_mask(&mask, 0, 0, 0.5, MaskBlendType::ToWhite)
            .unwrap();

        // p = 100/255, fract = 0.5
        // result = p + 0.5*(1-p) = 100/255 + 0.5*(1-100/255)
        // = 100/255 + 0.5*155/255 = 100/255 + 77.5/255 = 177.5/255 * 255 = ~178
        let val = result.get_pixel(5, 5).unwrap();
        assert!(val > 100); // Should be brighter
        assert!(val < 255); // But not white
    }

    #[test]
    fn test_blend_mask_to_black() {
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(5, 5, 200).unwrap();
        let base: Pix = base_mut.into();

        let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut mask_mut = mask.to_mut();
        mask_mut.set_pixel(5, 5, 1).unwrap();
        let mask: Pix = mask_mut.into();

        let result = base
            .blend_mask(&mask, 0, 0, 0.5, MaskBlendType::ToBlack)
            .unwrap();

        // p = 200/255, fract = 0.5
        // result = (1-0.5) * p = 0.5 * 200/255 * 255 = 100
        assert_eq!(result.get_pixel(5, 5), Some(100));
    }

    #[test]
    fn test_blend_with_gray_mask_transparent() {
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(5, 5, 100).unwrap();
        let base: Pix = base_mut.into();

        let overlay = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut overlay_mut = overlay.to_mut();
        overlay_mut.set_pixel(5, 5, 200).unwrap();
        let overlay: Pix = overlay_mut.into();

        // Fully transparent mask (all 0)
        let mask = Pix::new(10, 10, PixelDepth::Bit8).unwrap();

        let result = blend_with_gray_mask(&base, &overlay, &mask, 0, 0).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(100)); // No change
    }

    #[test]
    fn test_blend_with_gray_mask_opaque() {
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(5, 5, 100).unwrap();
        let base: Pix = base_mut.into();

        let overlay = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut overlay_mut = overlay.to_mut();
        overlay_mut.set_pixel(5, 5, 200).unwrap();
        let overlay: Pix = overlay_mut.into();

        // Fully opaque mask
        let mask = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut mask_mut = mask.to_mut();
        mask_mut.set_pixel(5, 5, 255).unwrap();
        let mask: Pix = mask_mut.into();

        let result = blend_with_gray_mask(&base, &overlay, &mask, 0, 0).unwrap();
        assert_eq!(result.get_pixel(5, 5), Some(200)); // Full overlay
    }

    #[test]
    fn test_blend_with_gray_mask_half() {
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(5, 5, 100).unwrap();
        let base: Pix = base_mut.into();

        let overlay = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut overlay_mut = overlay.to_mut();
        overlay_mut.set_pixel(5, 5, 200).unwrap();
        let overlay: Pix = overlay_mut.into();

        // Half transparent mask
        let mask = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut mask_mut = mask.to_mut();
        // Use 127 which is approximately 0.5 when normalized
        mask_mut.set_pixel(5, 5, 127).unwrap();
        let mask: Pix = mask_mut.into();

        let result = blend_with_gray_mask(&base, &overlay, &mask, 0, 0).unwrap();
        // alpha = 127/255 ~ 0.498
        // result = (1 - 0.498) * 100 + 0.498 * 200 ~ 150
        let val = result.get_pixel(5, 5).unwrap();
        assert!((148..=152).contains(&val)); // Allow rounding error
    }

    #[test]
    fn test_blend_color_negative_position() {
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(5, 5, 100).unwrap(); // Set a pixel outside blend region
        let base: Pix = base_mut.into();

        let blend = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut blend_mut = blend.to_mut();
        blend_mut.set_pixel(4, 4, 200).unwrap(); // Will be at (-1+4, -1+4) = (3,3) in result
        let blend: Pix = blend_mut.into();

        // Position blend at (-1, -1)
        // Blend covers base pixels from (0,0) to (3,3) (where blend (1,1) to (4,4) overlap)
        let result = base.blend_color(&blend, -1, -1, 1.0).unwrap();

        // Pixel at (3, 3) should be from blend (4, 4)
        assert_eq!(result.get_pixel(3, 3), Some(200));
        // Pixel at (5, 5) is outside blend region, should be unchanged
        assert_eq!(result.get_pixel(5, 5), Some(100));
    }

    #[test]
    fn test_blend_1bit_error() {
        let base = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let blend = Pix::new(10, 10, PixelDepth::Bit8).unwrap();

        let result = base.blend_color(&blend, 0, 0, 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_hard_light_fract_0() {
        // fract=0 should return original
        let base = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut base_mut = base.to_mut();
        base_mut.set_pixel(0, 0, 100).unwrap();
        let base: Pix = base_mut.into();

        let blend = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut blend_mut = blend.to_mut();
        blend_mut.set_pixel(0, 0, 200).unwrap();
        let blend: Pix = blend_mut.into();

        let result = base.blend_hard_light(&blend, 0.0).unwrap();
        // With fract=0, adjusted = 128, result = base * 128 / 128 = base
        // Actually for blend=200 >= 128: adjusted = 128, result = 255 - (255-128)*(255-100)/128
        // = 255 - 127*155/128 = 255 - 153.7 = 101
        // Hmm, let me recalculate...
        // For fract=0: adjusted = 128 + 0*(200-128) = 128
        // result = 255 - (255-128)*(255-100)/128 = 255 - 127*155/128 ≈ 101
        // This is close to base=100, which is the expected "no effect" behavior
        let val = result.get_pixel(0, 0).unwrap();
        assert!((98..=102).contains(&val)); // Close to original
    }

    #[test]
    fn test_blend_gray_inverse_basic() {
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut bm = base.to_mut();
        for y in 0..10 {
            for x in 0..10 {
                bm.set_pixel(x, y, 128).unwrap();
            }
        }
        let base: Pix = bm.into();
        let blender = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = base.blend_gray_inverse(&blender, 0, 0, 0.5).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_blend_color_by_channel_basic() {
        use super::super::graphics::Color;
        let base = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let blend = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let result = base
            .blend_color_by_channel(
                &blend,
                0,
                0,
                0.5,
                0.5,
                0.5,
                false,
                Color { r: 0, g: 0, b: 0 },
            )
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_blend_gray_adapt_basic() {
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut bm = base.to_mut();
        for y in 0..10 {
            for x in 0..10 {
                bm.set_pixel(x, y, 128).unwrap();
            }
        }
        let base: Pix = bm.into();
        let blender = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = base.blend_gray_adapt(&blender, 0, 0, 0.5, -1).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_fade_with_gray_to_white() {
        let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut bm = base.to_mut();
        for y in 0..10 {
            for x in 0..10 {
                bm.set_pixel(x, y, 100).unwrap();
            }
        }
        let base: Pix = bm.into();
        let gray = {
            let g = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
            let mut gm = g.to_mut();
            for y in 0..10 {
                for x in 0..10 {
                    gm.set_pixel(x, y, 255).unwrap();
                }
            }
            let g: Pix = gm.into();
            g
        };
        let result = base
            .fade_with_gray(&gray, 255.0, FadeWithGrayType::ToWhite)
            .unwrap();
        // Fully white blender with factor=255 → pixel becomes 255
        let val = result.get_pixel(0, 0).unwrap();
        assert_eq!(val, 255);
    }

    #[test]
    fn test_multiply_by_color_basic() {
        use super::super::graphics::Color;
        use crate::color::compose_rgb;
        let pix = Pix::new(4, 4, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        for y in 0..4 {
            for x in 0..4 {
                pm.set_pixel(x, y, compose_rgb(255, 255, 255)).unwrap();
            }
        }
        let pix: Pix = pm.into();
        // white * red(255,0,0) → red
        let result = pix.multiply_by_color(Color { r: 255, g: 0, b: 0 }).unwrap();
        let pixel = result.get_pixel(0, 0).unwrap();
        assert_eq!(crate::color::red(pixel), 255);
        assert_eq!(crate::color::green(pixel), 0);
        assert_eq!(crate::color::blue(pixel), 0);
    }

    #[test]
    fn test_alpha_blend_uniform_basic() {
        let pix = Pix::new(4, 4, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        pm.set_spp(4);
        for y in 0..4 {
            for x in 0..4 {
                pm.set_pixel(x, y, crate::color::compose_rgba(100, 100, 100, 128))
                    .unwrap();
            }
        }
        let pix: Pix = pm.into();
        let result = pix
            .alpha_blend_uniform(crate::color::compose_rgb(255, 255, 255))
            .unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_add_alpha_to_blend_basic() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = pix.add_alpha_to_blend(0.5, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_blend_cmap_basic() {
        use crate::colormap::PixColormap;
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 255, 255).unwrap(); // index 0 = white
        cmap.add_rgb(255, 0, 0).unwrap(); // index 1 = red
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        pm.set_colormap(Some(cmap)).unwrap();
        // All pixels are 0 (white); blend a small red image at origin
        let mut cmap2 = PixColormap::new(8).unwrap();
        cmap2.add_rgb(255, 0, 0).unwrap();
        let pix2 = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pm2 = pix2.to_mut();
        pm2.set_colormap(Some(cmap2)).unwrap();
        let pix2: Pix = pm2.into();
        pm.blend_cmap(&pix2, 0, 0, 0).unwrap();
    }

    #[test]
    fn test_linear_edge_fade_8bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        for y in 0..20 {
            for x in 0..20 {
                pm.set_pixel(x, y, 200).unwrap();
            }
        }
        pm.linear_edge_fade(FadeDirection::FromLeft, FadeTarget::ToBlack, 0.5, 1.0)
            .unwrap();
        // Left edge pixels should be darker
        let left = pm.get_pixel(0, 10).unwrap();
        let mid = pm.get_pixel(10, 10).unwrap();
        assert!(left < mid);
    }
}
