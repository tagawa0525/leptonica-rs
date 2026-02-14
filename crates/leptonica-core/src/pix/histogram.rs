//! Image histogram operations
//!
//! This module provides functions for computing histograms of pixel values:
//!
//! - Grayscale histogram (8bpp and 16bpp)
//! - Color histogram (per-channel for 32bpp RGB)
//!
//! # See also
//!
//! C Leptonica: `pix3.c`, `pixGetGrayHistogram()`,
//! `pixGetColorHistogram()`

use super::{Pix, PixelDepth};
use crate::error::{Error, Result};
use crate::numa::Numa;

/// Per-channel histograms for a color image
#[derive(Debug, Clone)]
pub struct ColorHistogram {
    /// Red channel histogram (256 bins)
    pub red: Numa,
    /// Green channel histogram (256 bins)
    pub green: Numa,
    /// Blue channel histogram (256 bins)
    pub blue: Numa,
}

impl Pix {
    /// Compute a grayscale histogram.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (must be >= 1)
    ///
    /// # Returns
    ///
    /// A [`Numa`] with histogram bin counts (256 bins for 8bpp).
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedDepth`] if depth is not 8bpp.
    /// Returns [`Error::InvalidParameter`] if `factor` < 1.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetGrayHistogram()`
    pub fn gray_histogram(&self, factor: u32) -> Result<Numa> {
        todo!()
    }

    /// Compute per-channel color histograms.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (must be >= 1)
    ///
    /// # Returns
    ///
    /// A [`ColorHistogram`] with red, green, and blue channel histograms.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedDepth`] if depth is not 32bpp.
    /// Returns [`Error::InvalidParameter`] if `factor` < 1.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetColorHistogram()`
    pub fn color_histogram(&self, factor: u32) -> Result<ColorHistogram> {
        todo!()
    }
}
