//! Statistical operations on Pix
//!
//! Provides pixel counting, row/column averages, variance computation.
//! Corresponds to C Leptonica `pix5.c` and related files.

use super::{Pix, PixelDepth};
use crate::box_::Box;
use crate::error::{Error, Result};
use crate::numa::Numa;

/// Type of pixel value interpretation for average calculations.
///
/// C equivalent: `L_WHITE_IS_MAX` / `L_BLACK_IS_MAX`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelMaxType {
    /// White pixels have maximum value; black pixels are 0.
    /// C equivalent: `L_WHITE_IS_MAX`
    WhiteIsMax,
    /// Black pixels get the maximum value; white pixels get 0.
    /// C equivalent: `L_BLACK_IS_MAX`
    BlackIsMax,
}

impl Pix {
    /// Count the total number of foreground (ON) pixels.
    pub fn count_pixels(&self) -> u64 {
        todo!()
    }

    /// Compute average pixel value per row.
    pub fn average_by_row(&self, _region: Option<&Box>, _pixel_type: PixelMaxType) -> Result<Numa> {
        todo!()
    }

    /// Compute average pixel value per column.
    pub fn average_by_column(
        &self,
        _region: Option<&Box>,
        _pixel_type: PixelMaxType,
    ) -> Result<Numa> {
        todo!()
    }

    /// Compute average pixel value in a rectangular region.
    pub fn average_in_rect(&self, _region: Option<&Box>) -> Result<f32> {
        todo!()
    }

    /// Compute variance of pixel values in a rectangular region.
    pub fn variance_in_rect(&self, _region: Option<&Box>) -> Result<f32> {
        todo!()
    }

    /// Compute variance of pixel values per row.
    pub fn variance_by_row(&self, _region: Option<&Box>) -> Result<Numa> {
        todo!()
    }

    /// Compute variance of pixel values per column.
    pub fn variance_by_column(&self, _region: Option<&Box>) -> Result<Numa> {
        todo!()
    }
}
