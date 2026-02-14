//! Raster operations (ROP)
//!
//! Provides bitwise operations on images: AND, OR, XOR, invert,
//! and region clear/set. Corresponds to C Leptonica `rop.c`.

use super::{Pix, PixMut, PixelDepth};
use crate::error::{Error, Result};

/// Raster operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RopOp {
    /// Clear (set to 0)
    Clear,
    /// AND
    And,
    /// AND with inverted source
    AndInvertSrc,
    /// Copy destination (no-op)
    Dst,
    /// AND with inverted destination
    AndInvertDst,
    /// Copy source
    Src,
    /// XOR
    Xor,
    /// OR
    Or,
    /// NOR
    Nor,
    /// XNOR (equivalence)
    Xnor,
    /// Invert destination
    InvertDst,
    /// OR with inverted source
    OrInvertSrc,
    /// Invert source
    InvertSrc,
    /// OR with inverted destination
    OrInvertDst,
    /// NAND
    Nand,
    /// Set (all 1s)
    Set,
}

impl RopOp {
    /// Check if this operation requires a source image.
    pub fn requires_source(self) -> bool {
        !matches!(
            self,
            RopOp::Clear | RopOp::Dst | RopOp::InvertDst | RopOp::Set
        )
    }
}

impl Pix {
    /// Bitwise AND of two images.
    pub fn and(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Bitwise OR of two images.
    pub fn or(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Bitwise XOR of two images.
    pub fn xor(&self, _other: &Pix) -> Result<Pix> {
        todo!()
    }

    /// Invert all pixels.
    pub fn invert(&self) -> Pix {
        todo!()
    }

    /// Apply a raster operation with another image.
    pub fn rop(&self, _other: &Pix, _op: RopOp) -> Result<Pix> {
        todo!()
    }
}

impl PixMut {
    /// Bitwise AND in-place.
    pub fn and_inplace(&mut self, _other: &Pix) -> Result<()> {
        todo!()
    }

    /// Bitwise OR in-place.
    pub fn or_inplace(&mut self, _other: &Pix) -> Result<()> {
        todo!()
    }

    /// Bitwise XOR in-place.
    pub fn xor_inplace(&mut self, _other: &Pix) -> Result<()> {
        todo!()
    }

    /// Invert all pixels in-place.
    pub fn invert_inplace(&mut self) {
        todo!()
    }

    /// Apply a raster operation in-place.
    pub fn rop_inplace(&mut self, _other: &Pix, _op: RopOp) -> Result<()> {
        todo!()
    }

    /// Clear a rectangular region.
    pub fn clear_region(&mut self, _x: u32, _y: u32, _w: u32, _h: u32) {
        todo!()
    }

    /// Set all pixels in a rectangular region.
    pub fn set_region(&mut self, _x: u32, _y: u32, _w: u32, _h: u32) {
        todo!()
    }
}
