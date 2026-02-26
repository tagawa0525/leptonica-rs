//! Leptonica Color - Color processing for image analysis
//!
//! This crate provides color manipulation and analysis functions:
//!
//! - **Color space conversion** ([`colorspace`]): RGB ↔ HSV, LAB, XYZ, YUV
//! - **Thresholding** ([`threshold`]): Binary conversion, Otsu's method, adaptive thresholding
//! - **Color quantization** ([`quantize`]): Median cut, octree algorithms
//! - **Color segmentation** ([`segment`]): Unsupervised color segmentation
//! - **Color analysis** ([`analysis`]): Statistics, color counting, grayscale detection
//! - **Color fill** ([`colorfill`]): Flood fill for RGB images based on color similarity
//! - **Coloring** ([`coloring`]): Colorize grayscale pixels, snap colors, fractional shifts
//!
//! # Example
//!
//! ```no_run
//! use leptonica::color::{pix_convert_to_gray, threshold_otsu, rgb_to_hsv};
//!
//! // Convert RGB to HSV
//! let hsv = rgb_to_hsv(255, 128, 64);
//! println!("H: {:.2}, S: {:.2}, V: {:.2}", hsv.h, hsv.s, hsv.v);
//! ```

pub mod analysis;
pub mod colorfill;
pub mod coloring;
pub mod colorspace;
pub mod error;
pub mod paintcmap;
pub mod quantize;
pub mod segment;
pub mod threshold;

// Re-export core types
pub use crate::core;

// Re-export error types
pub use error::{ColorError, ColorResult};

// Re-export color space types and functions
pub use colorspace::{
    // Types
    ColorChannel,
    Hsv,
    HsvHistoType,
    Lab,
    RegionFlag,
    Xyz,
    Yuv,
    // HSV histogram peak finding
    find_histo_peaks_hsv,
    // FPix-level conversions
    fpixa_convert_lab_to_rgb,
    fpixa_convert_lab_to_xyz,
    fpixa_convert_xyz_to_lab,
    fpixa_convert_xyz_to_rgb,
    // Pixel-level conversions
    hsv_to_rgb,
    lab_to_rgb,
    lab_to_xyz,
    // HSV histograms
    make_histo_hs,
    make_histo_hv,
    make_histo_sv,
    // HSV range masks
    make_range_mask_hs,
    make_range_mask_hv,
    make_range_mask_sv,
    // Image-level conversions
    pix_colormap_convert_hsv_to_rgb,
    pix_colormap_convert_rgb_to_hsv,
    pix_colormap_convert_rgb_to_yuv,
    pix_colormap_convert_yuv_to_rgb,
    pix_convert_hsv_to_rgb,
    pix_convert_rgb_to_hsv,
    pix_convert_rgb_to_lab,
    pix_convert_rgb_to_xyz,
    pix_convert_rgb_to_yuv,
    pix_convert_to_gray,
    pix_convert_yuv_to_rgb,
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
    adapt_threshold_to_binary_gen,
    adaptive_threshold,
    compute_otsu_threshold,
    dither_to_2bpp,
    dither_to_2bpp_spec,
    dither_to_binary,
    dither_to_binary_with_threshold,
    generate_mask_by_band,
    generate_mask_by_band_32,
    generate_mask_by_discr_32,
    generate_mask_by_value,
    gray_quant_from_cmap,
    gray_quant_from_histo,
    masked_thresh_on_background_norm,
    ordered_dither,
    otsu_adaptive_threshold,
    otsu_thresh_on_background_norm,
    sauvola_binarize_tiled,
    sauvola_on_contrast_norm,
    sauvola_threshold,
    thresh_on_double_norm,
    threshold_by_conn_comp,
    threshold_by_histo,
    threshold_gray_arb,
    threshold_on_8bpp,
    threshold_otsu,
    threshold_to_2bpp,
    threshold_to_4bpp,
    threshold_to_binary,
    var_threshold_to_binary,
};

// Re-export quantization functions
pub use quantize::{
    // Types
    MedianCutOptions,
    OctcubeTree,
    OctreeOptions,
    // Functions
    few_colors_median_cut_quant_mixed,
    few_colors_octcube_quant_mixed,
    few_colors_octcube_quant1,
    few_colors_octcube_quant2,
    fixed_octcube_quant_256,
    fixed_octcube_quant_gen_rgb,
    median_cut_quant,
    median_cut_quant_mixed,
    median_cut_quant_simple,
    number_occupied_octcubes,
    octcube_quant_from_cmap,
    octcube_quant_from_cmap_lut,
    octcube_quant_mixed_with_gray,
    octcube_tree,
    octree_quant,
    octree_quant_256,
    octree_quant_by_population,
    octree_quant_num_colors,
    quant_from_cmap,
    remove_unused_colors,
};

// Re-export analysis functions
pub use analysis::{
    // Types
    ColorMagnitudeType,
    ColorStats,
    // Functions
    color_content,
    color_fraction,
    color_magnitude,
    color_shift_white_point,
    colors_for_quantization,
    convert_rgb_to_cmap_lossless,
    count_colors,
    find_color_regions,
    grayscale_histogram,
    has_highlight_red,
    is_grayscale,
    is_grayscale_tolerant,
    mask_over_color_pixels,
    mask_over_color_range,
    mask_over_gray_pixels,
    most_populated_colors,
    num_significant_gray_colors,
    rgb_histogram,
    simple_color_quantize,
};

// Re-export segmentation functions
pub use segment::{
    // Types
    ColorSegmentOptions,
    // Functions
    assign_to_nearest_color,
    color_segment,
    color_segment_clean,
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
    color_content_by_location,
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
    color_gray_regions,
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
    snap_color_cmap,
};

// Re-export paintcmap functions
pub use paintcmap::{
    add_colorized_gray_to_cmap, pix_color_gray_cmap, pix_color_gray_masked_cmap,
    pix_color_gray_regions_cmap, pix_set_masked_cmap, pix_set_select_cmap,
    pix_set_select_masked_cmap,
};
