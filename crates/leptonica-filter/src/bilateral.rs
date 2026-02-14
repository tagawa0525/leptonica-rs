//! Bilateral filtering (edge-preserving smoothing)
//!
//! Bilateral filtering is a non-linear, edge-preserving smoothing filter.
//! It combines a spatial Gaussian filter with a range (intensity) Gaussian filter.
//!
//! The bilateral filter has the property of smoothing uniform regions while
//! preserving edges.
//!
//! C API mapping:
//! - `pixBlockBilateralExact` -> `bilateral_exact`
//! - `pixBilateralExact` -> `bilateral_gray_exact`
//!
//! Note: The separable approximate version (`pixBilateral`) is not implemented.

use crate::{FilterResult, Kernel};
use leptonica_core::Pix;

/// Create a range kernel for bilateral filtering.
///
/// Creates a 256-element array where element `i` represents the weight
/// for an intensity difference of `i` (0-255).
///
/// C: Part of `pixBlockBilateralExact` implementation.
pub fn make_range_kernel(range_stdev: f32) -> FilterResult<[f32; 256]> {
    todo!()
}

/// Apply exact bilateral filter to an 8bpp grayscale image.
///
/// This is the slow but exact implementation of bilateral filtering.
///
/// C: `pixBilateralExact(pixs, spatial_kel, range_kel)`
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale image
/// * `spatial_kernel` - 2D spatial Gaussian kernel
/// * `range_kernel` - Optional 256-element range kernel. If None, degenerates to regular Gaussian.
pub fn bilateral_gray_exact(
    pix: &Pix,
    spatial_kernel: &Kernel,
    range_kernel: Option<&[f32; 256]>,
) -> FilterResult<Pix> {
    todo!()
}

/// Apply exact bilateral filter to an image (8bpp or 32bpp).
///
/// High-level interface that creates the spatial and range kernels automatically.
///
/// C: `pixBlockBilateralExact(pixs, spatial_stdev, range_stdev)`
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale or 32bpp color image
/// * `spatial_stdev` - Spatial Gaussian standard deviation
/// * `range_stdev` - Range Gaussian standard deviation
pub fn bilateral_exact(pix: &Pix, spatial_stdev: f32, range_stdev: f32) -> FilterResult<Pix> {
    todo!()
}
