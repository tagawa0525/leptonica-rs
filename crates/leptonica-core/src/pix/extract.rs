//! Row/column extraction
//!
//! Extract pixel values along lines or specific rows/columns.
//! Corresponds to C Leptonica data extraction utilities.

use super::{Pix, PixelDepth};
use crate::error::{Error, Result};
use crate::numa::Numa;

impl Pix {
    /// Extract pixel values along a line from (x1,y1) to (x2,y2).
    ///
    /// # Arguments
    ///
    /// * `x1`, `y1` - Start point
    /// * `x2`, `y2` - End point
    /// * `factor` - Subsampling factor (1 = every pixel)
    pub fn extract_on_line(
        &self,
        _x1: i32,
        _y1: i32,
        _x2: i32,
        _y2: i32,
        _factor: i32,
    ) -> Result<Numa> {
        todo!()
    }
}
