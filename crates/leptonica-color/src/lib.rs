//! Leptonica Color - Color processing for image analysis
//!
//! This crate provides color manipulation and analysis functions:
//!
//! - **Color space conversion** ([`colorspace`]): RGB ↔ HSV, LAB, XYZ, YUV
//! - **Thresholding** ([`threshold`]): Binary conversion, Otsu's method, adaptive thresholding
//! - **Color quantization** ([`quantize`]): Median cut, octree algorithms
//! - **Color analysis** ([`analysis`]): Statistics, color counting, grayscale detection
//!
//! # Example
//!
//! ```no_run
//! use leptonica_color::{pix_convert_to_gray, threshold_otsu, rgb_to_hsv};
//!
//! // Convert RGB to HSV
//! let hsv = rgb_to_hsv(255, 128, 64);
//! println!("H: {:.2}, S: {:.2}, V: {:.2}", hsv.h, hsv.s, hsv.v);
//! ```

pub mod analysis;
pub mod colorspace;
pub mod error;
pub mod quantize;
pub mod threshold;

// Re-export core types
pub use leptonica_core;

// Re-export error types
pub use error::{ColorError, ColorResult};

// Re-export color space types and functions
pub use colorspace::{
    // Types
    ColorChannel,
    Hsv,
    Lab,
    Xyz,
    Yuv,
    // Pixel-level conversions
    hsv_to_rgb,
    lab_to_rgb,
    lab_to_xyz,
    // Image-level conversions
    pix_convert_hsv_to_rgb,
    pix_convert_rgb_to_hsv,
    pix_convert_to_gray,
    pix_extract_channel,
    rgb_to_gray,
    rgb_to_hsv,
    rgb_to_lab,
    rgb_to_xyz,
    rgb_to_yuv,
    xyz_to_lab,
    xyz_to_rgb,
    yuv_to_rgb,
};

// Re-export threshold functions
pub use threshold::{
    // Types
    AdaptiveMethod,
    AdaptiveThresholdOptions,
    // Functions
    adaptive_threshold,
    compute_otsu_threshold,
    dither_to_binary,
    dither_to_binary_with_threshold,
    ordered_dither,
    sauvola_threshold,
    threshold_otsu,
    threshold_to_binary,
};

// Re-export quantization functions
pub use quantize::{
    // Types
    MedianCutOptions,
    OctreeOptions,
    // Functions
    median_cut_quant,
    median_cut_quant_simple,
    octree_quant,
    octree_quant_256,
};

// Re-export analysis functions
pub use analysis::{
    // Types
    ColorStats,
    // Functions
    color_content,
    count_colors,
    grayscale_histogram,
    is_grayscale,
    is_grayscale_tolerant,
};
