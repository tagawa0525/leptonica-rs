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
//! - Warping (random harmonic, stereoscopic, horizontal stretch, quadratic shear)

pub mod affine;
pub mod bilinear;
mod error;
pub mod projective;
pub mod rotate;
pub mod scale;
pub mod shear;
pub mod warper;

pub use affine::{
    AffineFill, AffineMatrix, Point, affine, affine_pta, affine_pta_with_alpha, affine_rotate,
    affine_sampled, affine_sampled_pta, affine_scale, boxa_affine_transform, pta_affine_transform,
    translate,
};
pub use bilinear::{
    BilinearCoeffs, bilinear, bilinear_pta, bilinear_pta_with_alpha, bilinear_sampled,
    bilinear_sampled_pta,
};
pub use error::{TransformError, TransformResult};
pub use projective::{
    ProjectiveCoeffs, projective, projective_pta, projective_pta_with_alpha, projective_sampled,
    projective_sampled_pta,
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
pub use warper::{
    StereoscopicParams, WarpDirection, WarpFill, WarpOperation, WarpType, quadratic_v_shear,
    quadratic_v_shear_li, quadratic_v_shear_sampled, random_harmonic_warp, stereo_from_pair,
    stretch_horizontal, stretch_horizontal_li, stretch_horizontal_sampled, warp_stereoscopic,
};
