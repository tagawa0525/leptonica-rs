//! leptonica-morph - Morphological operations for image processing
//!
//! This crate provides morphological operations including:
//!
//! - Structuring elements (SEL) for defining operation neighborhoods
//! - Binary morphology: erosion, dilation, opening, closing
//! - DWA (Destination Word Accumulation): High-speed morphology using word-aligned operations
//! - Grayscale morphology: erosion, dilation, opening, closing for 8-bpp images
//! - Color morphology: erosion, dilation, opening, closing for 32-bpp images
//! - Hit-miss transform for pattern detection
//! - Morphological gradient, top-hat, and bottom-hat transforms
//! - Connectivity-preserving thinning (skeletonization)
//! - Morphological sequence operations for chaining multiple operations

pub mod binary;
pub mod color;
pub mod dwa;
mod error;
pub mod grayscale;
pub mod morphapp;
pub mod sel;
pub mod sequence;
pub mod thin;
pub mod thin_sels;

pub use error::{MorphError, MorphResult};
pub use sel::{Sel, SelElement};

// Re-export commonly used binary morphology functions
pub use binary::{
    BoundaryType, bottom_hat, close, close_brick, close_generalized, close_safe, close_safe_brick,
    close_safe_comp_brick, dilate, dilate_brick, erode, erode_brick, extract_boundary, gradient,
    hit_miss_transform, open, open_brick, open_generalized, top_hat,
};

// Re-export commonly used grayscale morphology functions
pub use grayscale::{
    bottom_hat_gray, close_gray, dilate_gray, erode_gray, gradient_gray, open_gray, top_hat_gray,
};

// Re-export commonly used color morphology functions
pub use color::{
    ColorChannel, bottom_hat_color, close_color, dilate_color, erode_color, gradient_color,
    open_color, top_hat_color,
};

// Re-export thinning functions
pub use thin::{Connectivity, ThinType, thin_connected, thin_connected_by_set};
pub use thin_sels::{ThinSelSet, make_thin_sels, sels_4and8cc_thin, sels_4cc_thin, sels_8cc_thin};

// Re-export sequence functions
pub use sequence::{
    MorphOp, MorphSequence, color_morph_sequence, gray_morph_sequence, morph_comp_sequence,
    morph_comp_sequence_dwa, morph_sequence, morph_sequence_dwa,
};

// Re-export DWA (high-speed morphology) functions
pub use dwa::{
    close_brick_dwa, close_comp_brick_dwa, close_comp_brick_extend_dwa, dilate_brick_dwa,
    dilate_comp_brick_dwa, dilate_comp_brick_extend_dwa, erode_brick_dwa, erode_comp_brick_dwa,
    erode_comp_brick_extend_dwa, get_extended_composite_parameters, open_brick_dwa,
    open_comp_brick_dwa, open_comp_brick_extend_dwa,
};

// Re-export morphological application functions
pub use morphapp::{
    MorphOpType, intersection_of_morph_ops, morph_gradient, morph_sequence_masked, seedfill_morph,
    union_of_morph_ops,
};

// Re-export SEL set generation functions
pub use sel::{
    sel_make_plus_sign, sela_add_basic, sela_add_cross_junctions, sela_add_dwa_combs,
    sela_add_dwa_linear, sela_add_hit_miss, sela_add_t_junctions,
};
