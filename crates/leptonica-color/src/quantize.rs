//! Color quantization
//!
//! Reduces the number of colors in an image while preserving visual quality:
//! - Median cut algorithm
//! - Octree quantization

use crate::ColorResult;
use leptonica_core::Pix;

/// Options for median cut quantization
#[derive(Debug, Clone)]
pub struct MedianCutOptions {
    /// Maximum number of colors in the output palette
    pub max_colors: u32,
    /// Minimum number of pixels in a box before it can be split
    pub min_box_pixels: u32,
}

impl Default for MedianCutOptions {
    fn default() -> Self {
        Self {
            max_colors: 256,
            min_box_pixels: 1,
        }
    }
}

/// Options for octree quantization
#[derive(Debug, Clone)]
pub struct OctreeOptions {
    /// Maximum number of colors in the output palette
    pub max_colors: u32,
}

impl Default for OctreeOptions {
    fn default() -> Self {
        Self { max_colors: 256 }
    }
}

/// Quantize a 32-bit color image using the median cut algorithm
pub fn median_cut_quant(_pix: &Pix, _options: &MedianCutOptions) -> ColorResult<Pix> {
    todo!()
}

/// Simple median cut quantization with default options
pub fn median_cut_quant_simple(_pix: &Pix, _max_colors: u32) -> ColorResult<Pix> {
    todo!()
}

/// Quantize a 32-bit color image using octree algorithm
pub fn octree_quant(_pix: &Pix, _options: &OctreeOptions) -> ColorResult<Pix> {
    todo!()
}

/// Quantize to exactly 256 colors using octree
pub fn octree_quant_256(_pix: &Pix) -> ColorResult<Pix> {
    todo!()
}
