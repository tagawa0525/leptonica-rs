//! Raster operations (ROP)
//!
//! This module provides bitwise raster operations on images:
//!
//! - AND, OR, XOR (binary operations on two images)
//! - Invert (NOT, unary operation)
//! - General ROP with named operations
//! - Region clear and set operations
//!
//! # See also
//!
//! C Leptonica: `rop.c`, `pixRasterop()`

use super::{Pix, PixMut, PixelDepth};
use crate::error::{Error, Result};

/// Raster operation type
///
/// Named raster operations for combining source and destination pixels.
///
/// # See also
///
/// C Leptonica: `PIX_*` ROP constants in `pix.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RopOp {
    /// Clear destination to 0
    Clear,
    /// Set destination to all 1s
    Set,
    /// Copy source to destination
    Src,
    /// Keep destination unchanged
    Dst,
    /// NOT source
    NotSrc,
    /// NOT destination
    NotDst,
    /// Source AND destination
    SrcAndDst,
    /// Source OR destination
    SrcOrDst,
    /// Source XOR destination
    SrcXorDst,
    /// NOT (Source AND destination)
    NotSrcAndDst,
    /// NOT (Source OR destination)
    NotSrcOrDst,
    /// NOT (Source XOR destination)
    NotSrcXorDst,
    /// Source AND (NOT destination)
    SrcAndNotDst,
    /// Source OR (NOT destination)
    SrcOrNotDst,
    /// (NOT source) AND destination
    NotSrcAndNotDst,
    /// Paint: Source OR destination (alias for SrcOrDst)
    Paint,
    /// Subtract: (NOT source) AND destination
    Subtract,
}

impl RopOp {
    /// Check if this operation requires a source image.
    ///
    /// Operations like `Clear`, `Set`, `Dst`, and `NotDst` operate
    /// only on the destination and do not need a source.
    pub fn requires_source(self) -> bool {
        !matches!(self, Self::Clear | Self::Set | Self::Dst | Self::NotDst)
    }
}

impl Pix {
    /// Bitwise AND of two images.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn and(&self, other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Bitwise OR of two images.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn or(&self, other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Bitwise XOR of two images.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn xor(&self, other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Invert all pixels.
    ///
    /// For 32bpp images, only the RGB channels are inverted;
    /// the alpha byte is preserved.
    ///
    /// # Returns
    ///
    /// New image with inverted pixels.
    pub fn invert(&self) -> Pix {
        todo!()
    }

    /// Apply a general raster operation.
    ///
    /// # Arguments
    ///
    /// * `other` - Source image (may be unused for some ops)
    /// * `op` - The raster operation to apply
    ///
    /// # Errors
    ///
    /// Returns error if operation requires source and dimensions/depths
    /// are incompatible.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixRasterop()`
    pub fn rop(&self, other: &Pix, op: RopOp) -> Result<Pix> {
        todo!()
    }
}

impl PixMut {
    /// Bitwise AND with another image, in-place.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn and_inplace(&mut self, other: &Pix) -> Result<()> {
        todo!()
    }

    /// Bitwise OR with another image, in-place.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn or_inplace(&mut self, other: &Pix) -> Result<()> {
        todo!()
    }

    /// Bitwise XOR with another image, in-place.
    ///
    /// # Errors
    ///
    /// Returns error if images have different dimensions or depths.
    pub fn xor_inplace(&mut self, other: &Pix) -> Result<()> {
        todo!()
    }

    /// Invert all pixels in-place.
    ///
    /// For 32bpp images, only the RGB channels are inverted;
    /// the alpha byte is preserved.
    pub fn invert_inplace(&mut self) {
        todo!()
    }

    /// Apply a general raster operation in-place.
    ///
    /// # Arguments
    ///
    /// * `other` - Source image (may be unused for some ops)
    /// * `op` - The raster operation to apply
    ///
    /// # Errors
    ///
    /// Returns error if operation requires source and dimensions/depths
    /// are incompatible.
    pub fn rop_inplace(&mut self, other: &Pix, op: RopOp) -> Result<()> {
        todo!()
    }

    /// Clear a rectangular region to zero.
    ///
    /// # Arguments
    ///
    /// * `x` - Left edge of region
    /// * `y` - Top edge of region
    /// * `w` - Width of region
    /// * `h` - Height of region
    pub fn clear_region(&mut self, x: u32, y: u32, w: u32, h: u32) {
        todo!()
    }

    /// Set a rectangular region to maximum pixel value.
    ///
    /// # Arguments
    ///
    /// * `x` - Left edge of region
    /// * `y` - Top edge of region
    /// * `w` - Width of region
    /// * `h` - Height of region
    pub fn set_region(&mut self, x: u32, y: u32, w: u32, h: u32) {
        todo!()
    }
}
