//! Border operations
//!
//! Add or remove borders around images.
//! Corresponds to C Leptonica `border.c`.

use super::Pix;
use crate::error::{Error, Result};

impl Pix {
    /// Add a uniform border of `npix` pixels with the specified value.
    pub fn add_border(&self, _npix: u32, _val: u32) -> Result<Pix> {
        todo!()
    }

    /// Add a border with different sizes on each side.
    pub fn add_border_general(
        &self,
        _left: u32,
        _right: u32,
        _top: u32,
        _bot: u32,
        _val: u32,
    ) -> Result<Pix> {
        todo!()
    }

    /// Remove a uniform border of `npix` pixels.
    pub fn remove_border(&self, _npix: u32) -> Result<Pix> {
        todo!()
    }

    /// Remove a border with different sizes on each side.
    pub fn remove_border_general(
        &self,
        _left: u32,
        _right: u32,
        _top: u32,
        _bot: u32,
    ) -> Result<Pix> {
        todo!()
    }
}
