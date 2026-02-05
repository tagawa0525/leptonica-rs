//! leptonica-morph - Morphological operations for image processing
//!
//! This crate provides morphological operations including:
//!
//! - Structuring elements (SEL) for defining operation neighborhoods
//! - Binary morphology: erosion, dilation, opening, closing
//! - Grayscale morphology: erosion, dilation, opening, closing for 8-bpp images
//! - Hit-miss transform for pattern detection
//! - Morphological gradient, top-hat, and bottom-hat transforms

pub mod binary;
mod error;
pub mod grayscale;
pub mod sel;

pub use error::{MorphError, MorphResult};
pub use sel::{Sel, SelElement};

// Re-export commonly used binary morphology functions
pub use binary::{
    bottom_hat, close, close_brick, dilate, dilate_brick, erode, erode_brick, gradient,
    hit_miss_transform, open, open_brick, top_hat,
};

// Re-export commonly used grayscale morphology functions
pub use grayscale::{
    bottom_hat_gray, close_gray, dilate_gray, erode_gray, gradient_gray, open_gray, top_hat_gray,
};
