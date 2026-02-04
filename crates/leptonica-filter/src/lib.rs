//! leptonica-filter - Image filtering operations
//!
//! This crate provides image filtering operations including:
//!
//! - Convolution with arbitrary kernels
//! - Blur operations (box blur, Gaussian blur)
//! - Edge detection (Sobel, Laplacian)
//! - Image enhancement (sharpening, unsharp masking, emboss)

pub mod convolve;
pub mod edge;
mod error;
pub mod kernel;

pub use error::{FilterError, FilterResult};
pub use kernel::Kernel;

// Re-export commonly used functions
pub use convolve::{box_blur, convolve, convolve_color, convolve_gray, gaussian_blur};
pub use edge::{EdgeOrientation, emboss, laplacian_edge, sharpen, sobel_edge, unsharp_mask};
