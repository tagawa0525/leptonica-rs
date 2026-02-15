//! JBIG2 classification algorithms

use super::types::{JbClasser, JbComponent};
use crate::RecogResult;
use leptonica_core::Pix;

/// Initialize a rank Hausdorff classifier
pub fn rank_haus_init(
    _components: JbComponent,
    _max_width: i32,
    _max_height: i32,
    _size_haus: i32,
    _rank_haus: f32,
) -> RecogResult<JbClasser> {
    todo!("rank_haus_init not yet implemented")
}

/// Initialize a correlation classifier
pub fn correlation_init(
    _components: JbComponent,
    _max_width: i32,
    _max_height: i32,
    _thresh: f32,
    _weight_factor: f32,
) -> RecogResult<JbClasser> {
    todo!("correlation_init not yet implemented")
}

/// Compute Hausdorff distance between two images
pub fn hausdorff_distance(_pix1: &Pix, _pix2: &Pix, _size: i32, _rank: f32) -> RecogResult<bool> {
    todo!("hausdorff_distance not yet implemented")
}
