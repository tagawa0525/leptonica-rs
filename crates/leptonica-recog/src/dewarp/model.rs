//! Dewarp model building - disparity array construction

use super::types::{Dewarp, TextLine};
use crate::RecogResult;

/// Build vertical disparity model from text lines
pub fn build_vertical_disparity(_dewarp: &mut Dewarp, _lines: &[TextLine]) -> RecogResult<()> {
    todo!("build_vertical_disparity not yet implemented")
}

/// Build horizontal disparity model from text lines
pub fn build_horizontal_disparity(_dewarp: &mut Dewarp, _lines: &[TextLine]) -> RecogResult<()> {
    todo!("build_horizontal_disparity not yet implemented")
}

/// Populate full resolution disparity from sampled arrays
pub fn populate_full_resolution(_dewarp: &mut Dewarp) -> RecogResult<()> {
    todo!("populate_full_resolution not yet implemented")
}
