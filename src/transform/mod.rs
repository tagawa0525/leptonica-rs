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
pub mod binreduce;
mod error;
pub mod projective;
pub mod rotate;
pub mod scale;
pub mod shear;
pub mod warper;

pub use affine::{
    AffineFill, AffineMatrix, Point, affine, affine_pta, affine_pta_with_alpha, affine_rotate,
    affine_sampled, affine_sampled_pta, affine_scale, boxa_affine_transform, boxa_rotate,
    boxa_scale, boxa_translate, pta_affine_transform, pta_scale, pta_translate, translate,
};
pub use bilinear::{
    BilinearCoeffs, bilinear, bilinear_pta, bilinear_pta_with_alpha, bilinear_sampled,
    bilinear_sampled_pta,
};
pub use binreduce::{reduce_rank_binary_2, reduce_rank_binary_cascade};
pub use error::{TransformError, TransformResult};
pub use projective::{
    ProjectiveCoeffs, projective, projective_pta, projective_pta_with_alpha, projective_sampled,
    projective_sampled_pta,
};
pub use rotate::{
    RotateFill, RotateMethod, RotateOptions, embed_for_rotation, flip_lr, flip_tb, rotate,
    rotate_90, rotate_180, rotate_180_in_place, rotate_about_center, rotate_am_color_corner,
    rotate_am_corner, rotate_am_gray_corner, rotate_binary_nice, rotate_by_angle,
    rotate_by_angle_with_options, rotate_by_radians, rotate_orth, rotate_shear,
    rotate_shear_center, rotate_shear_center_ip, rotate_shear_ip, rotate_with_alpha,
    rotate_with_method,
};
pub use scale::{
    GrayMinMaxMode, ScaleMethod, expand_replicate, scale, scale_area_map_2, scale_area_map_to_size,
    scale_binary, scale_binary_with_shift, scale_by_int_sampling, scale_by_sampling,
    scale_by_sampling_to_size, scale_by_sampling_with_shift, scale_color_2x_li, scale_color_4x_li,
    scale_color_li, scale_general, scale_gray_2x_li, scale_gray_2x_li_dither,
    scale_gray_2x_li_thresh, scale_gray_4x_li, scale_gray_4x_li_dither, scale_gray_4x_li_thresh,
    scale_gray_li, scale_gray_min_max, scale_gray_min_max_2, scale_gray_rank_2,
    scale_gray_rank_cascade, scale_li, scale_smooth, scale_smooth_to_size, scale_to_gray,
    scale_to_gray_2, scale_to_gray_3, scale_to_gray_4, scale_to_gray_6, scale_to_gray_8,
    scale_to_gray_16, scale_to_gray_fast, scale_to_gray_mipmap, scale_to_resolution, scale_to_size,
    scale_to_size_rel, scale_with_alpha,
};
pub use shear::{
    ShearFill, h_shear, h_shear_center, h_shear_corner, h_shear_ip, h_shear_li, v_shear,
    v_shear_center, v_shear_corner, v_shear_ip, v_shear_li,
};
pub use warper::{
    StereoscopicParams, WarpDirection, WarpFill, WarpOperation, WarpType, quadratic_v_shear,
    quadratic_v_shear_li, quadratic_v_shear_sampled, random_harmonic_warp, stereo_from_pair,
    stretch_horizontal, stretch_horizontal_li, stretch_horizontal_sampled, warp_stereoscopic,
};
