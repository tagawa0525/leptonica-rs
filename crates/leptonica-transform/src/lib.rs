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
pub use rotate::{
    RotateFill, RotateMethod, RotateOptions, flip_lr, flip_tb, rotate, rotate_90, rotate_180,
    rotate_180_in_place, rotate_about_center, rotate_by_angle, rotate_by_angle_with_options,
    rotate_by_radians, rotate_orth, rotate_with_method,
};
pub use scale::{ScaleMethod, scale, scale_by_sampling, scale_to_size};
