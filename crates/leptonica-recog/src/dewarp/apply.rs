//! Dewarp application - applying disparity arrays to images

use super::types::Dewarp;
use crate::RecogResult;
use leptonica_core::{FPix, Pix};

/// Apply vertical disparity to an image
pub fn apply_vertical_disparity(_pix: &Pix, _v_disparity: &FPix, _gray_in: u8) -> RecogResult<Pix> {
    todo!("apply_vertical_disparity not yet implemented")
}

/// Apply horizontal disparity to an image
pub fn apply_horizontal_disparity(
    _pix: &Pix,
    _h_disparity: &FPix,
    _gray_in: u8,
) -> RecogResult<Pix> {
    todo!("apply_horizontal_disparity not yet implemented")
}

/// Apply both vertical and horizontal disparity
pub fn apply_disparity(_pix: &Pix, _dewarp: &Dewarp, _gray_in: u8) -> RecogResult<Pix> {
    todo!("apply_disparity not yet implemented")
}

/// Estimate the magnitude of disparity from text lines
pub fn estimate_disparity_magnitude(_lines: &[super::types::TextLine]) -> f32 {
    todo!("estimate_disparity_magnitude not yet implemented")
}
