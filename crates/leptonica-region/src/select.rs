//! Component selection by size
//!
//! This module provides functions for selecting connected components from
//! binary images based on bounding box dimensions.

use crate::conncomp::ConnectivityType;
use crate::error::{RegionError, RegionResult};
use leptonica_core::{Pix, PixelDepth};

/// Selection type for component filtering by bounding box dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeSelectType {
    /// Select if BOTH width and height satisfy the relation
    IfBoth,
    /// Select if EITHER width or height satisfies the relation
    IfEither,
}

/// Selection relation for component filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeSelectRelation {
    /// Select if greater than or equal to threshold
    Gte,
    /// Select if less than or equal to threshold
    Lte,
}

/// Select connected components from a binary image by bounding box size
pub fn pix_select_by_size(
    pixs: &Pix,
    width_thresh: i32,
    height_thresh: i32,
    connectivity: ConnectivityType,
    select_type: SizeSelectType,
    relation: SizeSelectRelation,
) -> RegionResult<Pix> {
    todo!("pix_select_by_size not yet implemented")
}
