//! Recog training functions

use super::types::Recog;
use crate::RecogResult;
use leptonica_core::Pix;

/// Binarize a pix for recognition
pub fn binarize_pix(_pix: &Pix, _threshold: u8) -> RecogResult<Pix> {
    todo!("binarize_pix not yet implemented")
}

/// Create a new recognizer
pub fn create(
    _scale_w: i32,
    _scale_h: i32,
    _line_w: i32,
    _threshold: i32,
    _max_y_shift: i32,
) -> RecogResult<Recog> {
    todo!("create not yet implemented")
}

/// Create a recognizer from a Pixa of labeled templates
pub fn create_from_pixa(
    _pixa: &leptonica_core::Pixa,
    _scale_w: i32,
    _scale_h: i32,
    _line_w: i32,
    _threshold: i32,
    _max_y_shift: i32,
) -> RecogResult<Recog> {
    todo!("create_from_pixa not yet implemented")
}

/// Make a centroid lookup table (256 entries)
pub fn make_centtab() -> Vec<i32> {
    todo!("make_centtab not yet implemented")
}

/// Make a pixel sum lookup table (256 entries)
pub fn make_sumtab() -> Vec<i32> {
    todo!("make_sumtab not yet implemented")
}

/// Compute centroid of a 1bpp pix
pub fn compute_centroid(_pix: &Pix, _centtab: &[i32]) -> RecogResult<(f32, f32)> {
    todo!("compute_centroid not yet implemented")
}

/// Compute foreground area of a 1bpp pix
pub fn compute_area(_pix: &Pix, _sumtab: &[i32]) -> RecogResult<i32> {
    todo!("compute_area not yet implemented")
}

/// Compute correlation score with centering between two pix
pub fn compute_correlation_with_centering(
    _pix1: &Pix,
    _pix2: &Pix,
    _area1: i32,
    _cx1: f32,
    _cy1: f32,
    _cx2: f32,
    _cy2: f32,
    _tab: &[i32],
) -> RecogResult<f32> {
    todo!("compute_correlation_with_centering not yet implemented")
}
