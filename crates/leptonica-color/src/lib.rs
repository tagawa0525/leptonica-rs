//! Leptonica Color - Color processing for image analysis
//!
//! This crate provides color manipulation and analysis functions:
//!
//! - **Color space conversion** ([`colorspace`]): RGB <-> HSV, LAB, XYZ, YUV
//! - **Thresholding** ([`threshold`]): Binary conversion, Otsu's method, adaptive thresholding
//! - **Color quantization** ([`quantize`]): Median cut, octree algorithms
//! - **Color segmentation** ([`segment`]): Unsupervised color segmentation
//! - **Color analysis** ([`analysis`]): Statistics, color counting, grayscale detection
//! - **Color fill** ([`colorfill`]): Flood fill for RGB images based on color similarity
//! - **Coloring** ([`coloring`]): Colorize grayscale pixels, snap colors, fractional shifts

pub mod analysis;
pub mod colorfill;
pub mod coloring;
pub mod colorspace;
pub mod error;
pub mod quantize;
pub mod segment;
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

// Re-export segmentation functions
pub use segment::{
    // Types
    ColorSegmentOptions,
    // Functions
    assign_to_nearest_color,
    color_segment,
    color_segment_cluster,
    color_segment_simple,
};

// Re-export color fill functions
pub use colorfill::{
    // Types
    ColorFillOptions,
    ColorFillResult,
    ColorRegions,
    Connectivity,
    // Functions
    color_fill,
    color_fill_from_seed,
    pixel_is_on_color_boundary,
};

// Re-export coloring functions
pub use coloring::{
    // Types
    ColorGrayOptions,
    PaintType,
    // Image-level functions
    pix_color_gray,
    pix_color_gray_masked,
    pix_linear_map_to_target_color,
    pix_map_with_invariant_hue,
    pix_shift_by_component,
    pix_snap_color,
    // Pixel-level functions
    pixel_fractional_shift,
    pixel_linear_map_to_target_color,
    pixel_shift_by_component,
};
