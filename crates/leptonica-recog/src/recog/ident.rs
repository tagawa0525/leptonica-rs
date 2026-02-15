//! Recog identification functions

use crate::RecogResult;
use leptonica_core::Pix;

/// Compute correlation score between two pix
pub fn compute_correlation_score(_pix1: &Pix, _pix2: &Pix, _tab: &[i32]) -> RecogResult<f32> {
    todo!("compute_correlation_score not yet implemented")
}
