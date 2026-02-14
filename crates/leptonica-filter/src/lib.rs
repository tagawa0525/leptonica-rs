//! leptonica-filter - Image filtering operations
//!
//! This crate provides image filtering operations including:
//!
//! - Convolution with arbitrary kernels (`convolve`)
//! - Blur operations: box blur, Gaussian blur (`convolve`)
//! - Edge detection: Sobel, Laplacian (`edge`)
//! - Image enhancement: sharpening, unsharp masking, emboss (`edge`)
//! - Bilateral filtering: edge-preserving smoothing (`bilateral`)
//! - Rank filtering: median, min, max filters (`rank`)
//! - Adaptive mapping: background normalization, contrast normalization (`adaptmap`)
//!
//! Corresponds to C Leptonica source files:
//! - `bilateral.c`, `convolve.c`, `rank.c`, `adaptmap.c`, `edge.c`, `kernel.c`

pub mod adaptmap;
pub mod bilateral;
pub mod convolve;
pub mod edge;
mod error;
pub mod kernel;
pub mod rank;

pub use error::{FilterError, FilterResult};
pub use kernel::Kernel;

// Re-export commonly used functions
pub use adaptmap::{
    BackgroundNormOptions, ContrastNormOptions, background_norm, background_norm_simple,
    contrast_norm, contrast_norm_simple,
};
pub use bilateral::{bilateral_exact, bilateral_gray_exact, make_range_kernel};
pub use convolve::{box_blur, convolve, convolve_color, convolve_gray, gaussian_blur};
pub use edge::{EdgeOrientation, emboss, laplacian_edge, sharpen, sobel_edge, unsharp_mask};
pub use rank::{
    max_filter, median_filter, min_filter, rank_filter, rank_filter_color, rank_filter_gray,
};
