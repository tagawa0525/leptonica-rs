//! Histogram generation from Pix
//!
//! Provides grayscale and color histogram computation.
//! Corresponds to C Leptonica `histogram.c`.

use super::{Pix, PixelDepth};
use crate::error::{Error, Result};
use crate::numa::Numa;

/// Color histogram (separate histograms for R, G, B channels)
#[derive(Debug, Clone)]
pub struct ColorHistogram {
    /// Red channel histogram
    pub red: Numa,
    /// Green channel histogram
    pub green: Numa,
    /// Blue channel histogram
    pub blue: Numa,
}

impl Pix {
    /// Compute a grayscale histogram.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (1 = every pixel)
    pub fn gray_histogram(&self, _factor: u32) -> Result<Numa> {
        todo!()
    }

    /// Compute separate color channel histograms.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (1 = every pixel)
    pub fn color_histogram(&self, _factor: u32) -> Result<ColorHistogram> {
        todo!()
    }
}
