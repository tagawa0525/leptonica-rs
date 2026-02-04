//! leptonica-morph - Morphological operations for image processing
//!
//! This crate provides morphological operations including:
//!
//! - Structuring elements (SEL) for defining operation neighborhoods
//! - Binary morphology: erosion, dilation, opening, closing
//! - Hit-miss transform for pattern detection
//! - Morphological gradient, top-hat, and bottom-hat transforms

pub mod binary;
mod error;
pub mod sel;

pub use error::{MorphError, MorphResult};
pub use sel::{Sel, SelElement};

// Re-export commonly used binary morphology functions
pub use binary::{
    bottom_hat, close, close_brick, dilate, dilate_brick, erode, erode_brick, gradient,
    hit_miss_transform, open, open_brick, top_hat,
};
