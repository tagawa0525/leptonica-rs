//! Page dewarping (curvature correction)
//!
//! This module provides dewarping functionality to correct page curvature
//! in scanned document images.

pub mod apply;
pub mod model;
pub mod textline;
mod types;

pub use apply::{
    apply_disparity, apply_horizontal_disparity, apply_vertical_disparity,
    estimate_disparity_magnitude,
};
pub use model::{build_horizontal_disparity, build_vertical_disparity, populate_full_resolution};
pub use textline::{
    find_textline_centers, is_line_coverage_valid, remove_short_lines, sort_lines_by_y,
};
pub use types::{Dewarp, DewarpOptions, DewarpResult, TextLine};

use crate::RecogResult;
use leptonica_core::Pix;

/// Dewarp a single page
///
/// This is the top-level API for dewarping a single page image.
pub fn dewarp_single_page(_pix: &Pix, _options: &DewarpOptions) -> RecogResult<DewarpResult> {
    todo!("dewarp_single_page not yet implemented")
}

/// Check if a page needs dewarping
pub fn needs_dewarping(_pix: &Pix) -> RecogResult<bool> {
    todo!("needs_dewarping not yet implemented")
}
