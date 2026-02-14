//! Pixel value extraction along lines
//!
//! This module provides functions for extracting pixel values along
//! arbitrary lines through an image.
//!
//! # See also
//!
//! C Leptonica: `pix3.c`, `pixExtractOnLine()`

use super::{Pix, PixelDepth};
use crate::error::{Error, Result};
use crate::numa::Numa;

impl Pix {
    /// Extract pixel values along a line between two points.
    ///
    /// Samples pixel values at regular intervals along the line from
    /// `(x1, y1)` to `(x2, y2)`.
    ///
    /// # Arguments
    ///
    /// * `x1`, `y1` - Start point
    /// * `x2`, `y2` - End point
    /// * `factor` - Subsampling factor (must be >= 1)
    ///
    /// # Returns
    ///
    /// A [`Numa`] containing the sampled pixel values.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if `factor` < 1.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixExtractOnLine()`
    pub fn extract_on_line(&self, x1: i32, y1: i32, x2: i32, y2: i32, factor: i32) -> Result<Numa> {
        todo!()
    }
}
