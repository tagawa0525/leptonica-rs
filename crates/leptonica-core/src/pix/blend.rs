//! Pixel blending operations
//!
//! Provides various blending modes for combining images.
//! Corresponds to C Leptonica `blend.c`.

use super::{Pix, PixelDepth};
use crate::error::{Error, Result};

/// Mask blend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskBlendType {
    /// Paint mask color where mask is set
    Paint,
    /// Use blend factor for compositing
    Blend,
}

/// Gray blend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrayBlendType {
    /// Arithmetic blend
    Arithmetic,
    /// Multiply blend
    Multiply,
    /// Screen blend
    Screen,
}

/// Blend mode for combining two images
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Normal alpha compositing
    Normal,
    /// Multiply blend mode
    Multiply,
    /// Screen blend mode
    Screen,
    /// Overlay blend mode
    Overlay,
    /// Hard light blend mode
    HardLight,
}

impl Pix {
    /// Blend a color image onto this image at (x, y).
    pub fn blend_color(&self, _blend: &Pix, _x: i32, _y: i32, _fract: f32) -> Result<Pix> {
        todo!()
    }

    /// Blend a gray image onto this image.
    pub fn blend_gray(
        &self,
        _blend: &Pix,
        _x: i32,
        _y: i32,
        _fract: f32,
        _blend_type: GrayBlendType,
    ) -> Result<Pix> {
        todo!()
    }

    /// Blend using a mask image.
    pub fn blend_mask(
        &self,
        _blend: &Pix,
        _mask: &Pix,
        _x: i32,
        _y: i32,
        _fract: f32,
    ) -> Result<Pix> {
        todo!()
    }

    /// Blend two images using a specified mode.
    pub fn blend(&self, _other: &Pix, _mode: BlendMode, _fract: f32) -> Result<Pix> {
        todo!()
    }

    /// Multiply blend two images.
    pub fn blend_multiply(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Screen blend two images.
    pub fn blend_screen(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Overlay blend two images.
    pub fn blend_overlay(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Hard light blend two images.
    pub fn blend_hard_light(&self, _other: &Pix, _fract: f32) -> Result<Pix> {
        todo!()
    }
}

/// Blend two images using a gray mask.
pub fn blend_with_gray_mask(
    _base: &Pix,
    _overlay: &Pix,
    _mask: &Pix,
    _x: i32,
    _y: i32,
) -> Result<Pix> {
    todo!()
}
