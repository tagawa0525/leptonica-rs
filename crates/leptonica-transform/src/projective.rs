//! Projective transformations for images
//!
//! Projective (homography) transformations use 4-point correspondences to define
//! a perspective mapping. These can correct for perspective distortion in images.
//!
//! # C API correspondence
//!
//! | Rust function | C function |
//! |---|---|
//! | `projective_sampled` | `pixProjectiveSampled` |
//! | `projective_sampled_pta` | `pixProjectiveSampledPta` |
//! | `projective` | `pixProjective` |
//! | `projective_pta` | `pixProjectivePta` |

use crate::{
    TransformResult,
    affine::{AffineFill, Point},
};
use leptonica_core::Pix;

/// Projective transformation coefficients (8 values)
///
/// Defines the mapping:
/// ```text
/// x' = (a*x + b*y + c) / (g*x + h*y + 1)
/// y' = (d*x + e*y + f) / (g*x + h*y + 1)
/// ```
///
/// Corresponds to the coefficient arrays used by `getProjectiveXformCoeffs()` in C Leptonica.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectiveCoeffs {
    /// Coefficients [a, b, c, d, e, f, g, h]
    coeffs: [f32; 8],
}

impl Default for ProjectiveCoeffs {
    fn default() -> Self {
        Self {
            coeffs: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        }
    }
}

/// Apply sampled projective transformation using coefficients
///
/// Corresponds to `pixProjectiveSampled()` in C Leptonica.
pub fn projective_sampled(
    pix: &Pix,
    coeffs: &ProjectiveCoeffs,
    fill: AffineFill,
) -> TransformResult<Pix> {
    todo!()
}

/// Apply sampled projective transformation using 4-point correspondences
///
/// Corresponds to `pixProjectiveSampledPta()` in C Leptonica.
///
/// # Arguments
/// * `pix` - Input image
/// * `src_pts` - 4 source points
/// * `dst_pts` - 4 destination points
/// * `fill` - Background fill color
pub fn projective_sampled_pta(
    pix: &Pix,
    src_pts: [Point; 4],
    dst_pts: [Point; 4],
    fill: AffineFill,
) -> TransformResult<Pix> {
    todo!()
}

/// Apply interpolated projective transformation using coefficients
///
/// Corresponds to `pixProjective()` in C Leptonica.
pub fn projective(pix: &Pix, coeffs: &ProjectiveCoeffs, fill: AffineFill) -> TransformResult<Pix> {
    todo!()
}

/// Apply interpolated projective transformation using 4-point correspondences
///
/// Corresponds to `pixProjectivePta()` in C Leptonica.
///
/// # Arguments
/// * `pix` - Input image
/// * `src_pts` - 4 source points
/// * `dst_pts` - 4 destination points
/// * `fill` - Background fill color
pub fn projective_pta(
    pix: &Pix,
    src_pts: [Point; 4],
    dst_pts: [Point; 4],
    fill: AffineFill,
) -> TransformResult<Pix> {
    todo!()
}
