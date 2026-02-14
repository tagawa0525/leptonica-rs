//! Border operations for images
//!
//! This module provides functions for adding and removing borders
//! (padding) around images:
//!
//! - Uniform borders (same size on all sides)
//! - General borders (different size per side)
//!
//! # See also
//!
//! C Leptonica: `pix2.c`, `pixAddBorder()`, `pixRemoveBorder()`

use super::{Pix, PixelDepth};
use crate::error::{Error, Result};

impl Pix {
    /// Add a uniform border around the image.
    ///
    /// Creates a new image with `npix` pixels of border on all sides,
    /// filled with the specified value.
    ///
    /// # Arguments
    ///
    /// * `npix` - Border width in pixels
    /// * `val` - Border pixel value
    ///
    /// # Returns
    ///
    /// New image with dimensions `(width + 2*npix, height + 2*npix)`.
    ///
    /// # Errors
    ///
    /// Returns error if dimensions would overflow or npix is 0.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddBorder()`
    pub fn add_border(&self, npix: u32, val: u32) -> Result<Pix> {
        todo!()
    }

    /// Add a general border with different sizes per side.
    ///
    /// # Arguments
    ///
    /// * `left` - Left border width
    /// * `right` - Right border width
    /// * `top` - Top border height
    /// * `bot` - Bottom border height
    /// * `val` - Border pixel value
    ///
    /// # Returns
    ///
    /// New image with dimensions `(width + left + right, height + top + bot)`.
    ///
    /// # Errors
    ///
    /// Returns error if dimensions would overflow.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddBorderGeneral()`
    pub fn add_border_general(
        &self,
        left: u32,
        right: u32,
        top: u32,
        bot: u32,
        val: u32,
    ) -> Result<Pix> {
        todo!()
    }

    /// Remove a uniform border from the image.
    ///
    /// # Arguments
    ///
    /// * `npix` - Border width to remove
    ///
    /// # Returns
    ///
    /// New image with dimensions `(width - 2*npix, height - 2*npix)`.
    ///
    /// # Errors
    ///
    /// Returns error if border is larger than the image.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixRemoveBorder()`
    pub fn remove_border(&self, npix: u32) -> Result<Pix> {
        todo!()
    }

    /// Remove a general border with different sizes per side.
    ///
    /// # Arguments
    ///
    /// * `left` - Left border width to remove
    /// * `right` - Right border width to remove
    /// * `top` - Top border height to remove
    /// * `bot` - Bottom border height to remove
    ///
    /// # Returns
    ///
    /// New image with dimensions `(width - left - right, height - top - bot)`.
    ///
    /// # Errors
    ///
    /// Returns error if border is larger than the image.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixRemoveBorderGeneral()`
    pub fn remove_border_general(&self, left: u32, right: u32, top: u32, bot: u32) -> Result<Pix> {
        todo!()
    }
}
