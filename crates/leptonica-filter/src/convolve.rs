//! Convolution operations
//!
//! Implements image convolution with arbitrary kernels.
//!
//! C API mapping:
//! - `pixConvolve(pixg, kel, outdepth, normflag)` -> `convolve`
//! - `pixBlockconv(pixs, wc, hc)` -> `box_blur`
//! - Custom Gaussian blur -> `gaussian_blur`

use crate::{FilterResult, Kernel};
use leptonica_core::Pix;

/// Convolve an 8-bit grayscale image with a kernel.
///
/// C: `pixConvolve(pixg, kel, 8, 1)` for 8bpp input
pub fn convolve_gray(pix: &Pix, kernel: &Kernel) -> FilterResult<Pix> {
    todo!()
}

/// Convolve a 32-bit color image with a kernel (per-channel).
///
/// C: `pixConvolve` applied per RGB channel
pub fn convolve_color(pix: &Pix, kernel: &Kernel) -> FilterResult<Pix> {
    todo!()
}

/// Convolve an image (auto-dispatch based on depth).
///
/// C: `pixConvolve(pix, kel, outdepth, normflag)`
pub fn convolve(pix: &Pix, kernel: &Kernel) -> FilterResult<Pix> {
    todo!()
}

/// Apply box (average) blur.
///
/// C: `pixBlockconv(pixs, wc, hc)`
pub fn box_blur(pix: &Pix, radius: u32) -> FilterResult<Pix> {
    todo!()
}

/// Apply Gaussian blur with specified sigma.
///
/// C: Custom Gaussian kernel + `pixConvolve`
pub fn gaussian_blur(pix: &Pix, radius: u32, sigma: f32) -> FilterResult<Pix> {
    todo!()
}

/// Apply Gaussian blur with automatic sigma calculation.
///
/// Uses `sigma = radius` as a rule of thumb.
pub fn gaussian_blur_auto(pix: &Pix, radius: u32) -> FilterResult<Pix> {
    todo!()
}
