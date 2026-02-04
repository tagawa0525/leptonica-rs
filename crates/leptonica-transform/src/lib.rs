//! leptonica-transform - Geometric transformations for Leptonica
//!
//! This crate provides geometric transformation operations including:
//!
//! - Orthogonal rotations (90, 180, 270 degrees)
//! - Horizontal and vertical flips
//! - Scaling (linear interpolation, sampling, area mapping)
//! - Affine transformations

mod error;
pub mod rotate;
pub mod scale;

pub use error::{TransformError, TransformResult};
pub use rotate::{flip_lr, flip_tb, rotate_90, rotate_180, rotate_orth};
pub use scale::{ScaleMethod, scale, scale_by_sampling, scale_to_size};
