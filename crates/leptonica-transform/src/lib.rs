//! leptonica-transform - Geometric transformations for Leptonica
//!
//! This crate provides geometric transformation operations including:
//!
//! - Orthogonal rotations (90, 180, 270 degrees)
//! - Horizontal and vertical flips
//! - Scaling (linear interpolation, sampling, area mapping)
//! - Affine transformations (3-point correspondence, matrix-based)
//! - Bilinear transformations (4-point correspondence, nonlinear)
//! - Projective transformations (4-point correspondence, homography)
//! - Shear transformations (horizontal and vertical)

pub mod affine;
pub mod bilinear;
mod error;
pub mod projective;
pub mod rotate;
pub mod scale;
pub mod shear;

pub use affine::{
    AffineFill, AffineMatrix, Point, affine, affine_pta, affine_rotate, affine_sampled,
    affine_sampled_pta, affine_scale, translate,
};
pub use bilinear::{
    BilinearCoeffs, bilinear, bilinear_pta, bilinear_sampled, bilinear_sampled_pta,
};
pub use error::{TransformError, TransformResult};
pub use projective::{
    ProjectiveCoeffs, projective, projective_pta, projective_sampled, projective_sampled_pta,
};
pub use rotate::{
    RotateFill, RotateMethod, RotateOptions, flip_lr, flip_tb, rotate, rotate_90, rotate_180,
    rotate_180_in_place, rotate_about_center, rotate_by_angle, rotate_by_angle_with_options,
    rotate_by_radians, rotate_orth, rotate_with_method,
};
pub use scale::{ScaleMethod, scale, scale_by_sampling, scale_to_size};
pub use shear::{
    ShearFill, h_shear, h_shear_center, h_shear_corner, h_shear_ip, h_shear_li, v_shear,
    v_shear_center, v_shear_corner, v_shear_ip, v_shear_li,
};
