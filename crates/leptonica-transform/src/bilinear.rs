//! Bilinear transformations for images
//!
//! Bilinear transformations use 4-point correspondences to define a non-linear
//! mapping. Unlike affine transforms that preserve parallel lines, bilinear
//! transforms can map any quadrilateral to any other quadrilateral.
//!
//! # C API correspondence
//!
//! | Rust function | C function |
//! |---|---|
//! | `bilinear_sampled` | `pixBilinearSampled` |
//! | `bilinear_sampled_pta` | `pixBilinearSampledPta` |
//! | `bilinear` | `pixBilinear` |
//! | `bilinear_pta` | `pixBilinearPta` |

use crate::{
    TransformResult,
    affine::{AffineFill, Point},
};
use leptonica_core::Pix;

/// Bilinear transformation coefficients (8 values)
///
/// Defines the mapping:
/// ```text
/// x' = a*x + b*y + c*x*y + d
/// y' = e*x + f*y + g*x*y + h
/// ```
///
/// Corresponds to the coefficient arrays used by `getBilinearXformCoeffs()` in C Leptonica.
#[derive(Debug, Clone, PartialEq)]
pub struct BilinearCoeffs {
    /// Coefficients [a, b, c, d, e, f, g, h]
    coeffs: [f32; 8],
}

impl Default for BilinearCoeffs {
    fn default() -> Self {
        Self {
            coeffs: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}

/// Apply sampled bilinear transformation using coefficients
///
/// Corresponds to `pixBilinearSampled()` in C Leptonica.
pub fn bilinear_sampled(
    pix: &Pix,
    coeffs: &BilinearCoeffs,
    fill: AffineFill,
) -> TransformResult<Pix> {
    todo!()
}

/// Apply sampled bilinear transformation using 4-point correspondences
///
/// Corresponds to `pixBilinearSampledPta()` in C Leptonica.
///
/// # Arguments
/// * `pix` - Input image
/// * `src_pts` - 4 source points
/// * `dst_pts` - 4 destination points
/// * `fill` - Background fill color
pub fn bilinear_sampled_pta(
    pix: &Pix,
    src_pts: [Point; 4],
    dst_pts: [Point; 4],
    fill: AffineFill,
) -> TransformResult<Pix> {
    todo!()
}

/// Apply interpolated bilinear transformation using coefficients
///
/// Corresponds to `pixBilinear()` in C Leptonica.
pub fn bilinear(pix: &Pix, coeffs: &BilinearCoeffs, fill: AffineFill) -> TransformResult<Pix> {
    todo!()
}

/// Apply interpolated bilinear transformation using 4-point correspondences
///
/// Corresponds to `pixBilinearPta()` in C Leptonica.
///
/// # Arguments
/// * `pix` - Input image
/// * `src_pts` - 4 source points
/// * `dst_pts` - 4 destination points
/// * `fill` - Background fill color
pub fn bilinear_pta(
    pix: &Pix,
    src_pts: [Point; 4],
    dst_pts: [Point; 4],
    fill: AffineFill,
) -> TransformResult<Pix> {
    todo!()
}
