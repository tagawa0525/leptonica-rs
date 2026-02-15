//! leptonica-filter - Image filtering operations
//!
//! This crate provides image filtering operations including:
//!
//! - Convolution with arbitrary kernels
//! - Blur operations (box blur, Gaussian blur)
//! - Edge detection (Sobel, Laplacian)
//! - Image enhancement (sharpening, unsharp masking, emboss)
//! - Bilateral filtering (edge-preserving smoothing)
//! - Rank filtering (median, min, max filters)
//! - Adaptive mapping (background normalization, contrast normalization)

pub mod adaptmap;
pub mod bilateral;
pub mod block_conv;
pub mod convolve;
pub mod edge;
pub mod enhance;
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
pub use block_conv::{blockconv, blockconv_accum, blockconv_gray, blockconv_gray_unnormalized};
pub use convolve::{box_blur, convolve, convolve_color, convolve_gray, gaussian_blur};
pub use edge::{EdgeOrientation, emboss, laplacian_edge, sharpen, sobel_edge, unsharp_mask};
pub use enhance::{
    TrcLut, color_shift_rgb, contrast_trc, contrast_trc_masked, contrast_trc_pix, darken_gray,
    equalize_trc, equalize_trc_pix, gamma_trc, gamma_trc_masked, gamma_trc_pix,
    gamma_trc_with_alpha, measure_saturation, modify_brightness, modify_hue, modify_saturation,
    mult_constant_color, mult_matrix_color, trc_map, trc_map_general,
};
pub use rank::{
    max_filter, median_filter, min_filter, rank_filter, rank_filter_color, rank_filter_gray,
};
