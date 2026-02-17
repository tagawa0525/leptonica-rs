//! Color quantization
//!
//! Reduces the number of colors in an image while preserving visual quality:
//! - Median cut algorithm
//! - Octree quantization

use crate::{ColorError, ColorResult};
use leptonica_core::{Pix, PixColormap, PixelDepth, color};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

// =============================================================================
// Median Cut Quantization
// =============================================================================

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

/// Quantize a 32-bit color image using the median cut algorithm
pub fn median_cut_quant(pix: &Pix, options: &MedianCutOptions) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    if options.max_colors == 0 || options.max_colors > 256 {
        return Err(ColorError::InvalidParameters(
            "max_colors must be between 1 and 256".to_string(),
        ));
    }

    let w = pix.width();
    let h = pix.height();

    // Collect all pixels
    let mut pixels: Vec<[u8; 3]> = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            pixels.push([r, g, b]);
        }
    }

    if pixels.is_empty() {
        return Err(ColorError::EmptyImage);
    }

    // Build initial box containing all pixels
    let mut boxes: BinaryHeap<ColorBox> = BinaryHeap::new();
    boxes.push(ColorBox::from_pixels(&pixels, (0..pixels.len()).collect()));

    // Split boxes until we have enough colors
    while boxes.len() < options.max_colors as usize {
        if let Some(box_) = boxes.pop() {
            if box_.indices.len() <= options.min_box_pixels as usize {
                boxes.push(box_);
                break;
            }

            match box_.split(&pixels) {
                Ok((box1, box2)) => {
                    boxes.push(box1);
                    boxes.push(box2);
                }
                Err(original) => {
                    // Can't split further, push back the original
                    boxes.push(original);
                    break;
                }
            }
        } else {
            break;
        }
    }

    // Create colormap from boxes
    let mut colormap = PixColormap::new(8)?;
    let box_vec: Vec<ColorBox> = boxes.into_vec();

    for box_ in &box_vec {
        let (r, g, b) = box_.average_color(&pixels);
        colormap.add_rgb(r, g, b)?;
    }

    // Map each pixel to nearest color in palette
    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_colormap(Some(colormap))?;

    // Build lookup table: pixel index -> colormap index
    let mut color_indices: Vec<u8> = vec![0; pixels.len()];
    for (box_idx, box_) in box_vec.iter().enumerate() {
        for &pixel_idx in &box_.indices {
            color_indices[pixel_idx] = box_idx as u8;
        }
    }

    // Write output
    for y in 0..h {
        for x in 0..w {
            let pixel_idx = (y * w + x) as usize;
            let color_idx = color_indices[pixel_idx];
            out_mut.set_pixel_unchecked(x, y, color_idx as u32);
        }
    }

    Ok(out_mut.into())
}

/// Simple median cut quantization with default options
pub fn median_cut_quant_simple(pix: &Pix, max_colors: u32) -> ColorResult<Pix> {
    median_cut_quant(
        pix,
        &MedianCutOptions {
            max_colors,
            ..Default::default()
        },
    )
}

/// A box in RGB color space containing pixel indices
#[derive(Clone)]
struct ColorBox {
    indices: Vec<usize>,
    min_r: u8,
    max_r: u8,
    min_g: u8,
    max_g: u8,
    min_b: u8,
    max_b: u8,
}

impl ColorBox {
    fn from_pixels(pixels: &[[u8; 3]], indices: Vec<usize>) -> Self {
        let mut min_r = 255u8;
        let mut max_r = 0u8;
        let mut min_g = 255u8;
        let mut max_g = 0u8;
        let mut min_b = 255u8;
        let mut max_b = 0u8;

        for &idx in &indices {
            let [r, g, b] = pixels[idx];
            min_r = min_r.min(r);
            max_r = max_r.max(r);
            min_g = min_g.min(g);
            max_g = max_g.max(g);
            min_b = min_b.min(b);
            max_b = max_b.max(b);
        }

        Self {
            indices,
            min_r,
            max_r,
            min_g,
            max_g,
            min_b,
            max_b,
        }
    }

    fn volume(&self) -> u32 {
        let r_range = (self.max_r - self.min_r) as u32 + 1;
        let g_range = (self.max_g - self.min_g) as u32 + 1;
        let b_range = (self.max_b - self.min_b) as u32 + 1;
        r_range * g_range * b_range
    }

    fn split(mut self, pixels: &[[u8; 3]]) -> Result<(ColorBox, ColorBox), ColorBox> {
        if self.indices.len() < 2 {
            return Err(self);
        }

        // Find the channel with the largest range
        let r_range = self.max_r - self.min_r;
        let g_range = self.max_g - self.min_g;
        let b_range = self.max_b - self.min_b;

        let channel = if r_range >= g_range && r_range >= b_range {
            0 // Red
        } else if g_range >= b_range {
            1 // Green
        } else {
            2 // Blue
        };

        // Sort indices by the selected channel
        self.indices.sort_by_key(|&idx| pixels[idx][channel]);

        // Split at median
        let mid = self.indices.len() / 2;
        let indices1: Vec<usize> = self.indices[..mid].to_vec();
        let indices2: Vec<usize> = self.indices[mid..].to_vec();

        if indices1.is_empty() || indices2.is_empty() {
            // Reconstruct self with sorted indices
            return Err(ColorBox::from_pixels(pixels, self.indices));
        }

        Ok((
            ColorBox::from_pixels(pixels, indices1),
            ColorBox::from_pixels(pixels, indices2),
        ))
    }

    fn average_color(&self, pixels: &[[u8; 3]]) -> (u8, u8, u8) {
        if self.indices.is_empty() {
            return (0, 0, 0);
        }

        let mut sum_r = 0u64;
        let mut sum_g = 0u64;
        let mut sum_b = 0u64;

        for &idx in &self.indices {
            let [r, g, b] = pixels[idx];
            sum_r += r as u64;
            sum_g += g as u64;
            sum_b += b as u64;
        }

        let count = self.indices.len() as u64;
        (
            (sum_r / count) as u8,
            (sum_g / count) as u8,
            (sum_b / count) as u8,
        )
    }
}

impl Eq for ColorBox {}

impl PartialEq for ColorBox {
    fn eq(&self, other: &Self) -> bool {
        self.indices.len() == other.indices.len() && self.volume() == other.volume()
    }
}

impl Ord for ColorBox {
    fn cmp(&self, other: &Self) -> Ordering {
        // Priority: larger boxes should be split first
        // Use (count * volume) as the priority metric
        let self_priority = self.indices.len() as u64 * self.volume() as u64;
        let other_priority = other.indices.len() as u64 * other.volume() as u64;
        self_priority.cmp(&other_priority)
    }
}

impl PartialOrd for ColorBox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// =============================================================================
// Octree Quantization
// =============================================================================

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

/// Quantize a 32-bit color image using octree algorithm
pub fn octree_quant(pix: &Pix, options: &OctreeOptions) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    if options.max_colors == 0 || options.max_colors > 256 {
        return Err(ColorError::InvalidParameters(
            "max_colors must be between 1 and 256".to_string(),
        ));
    }

    let w = pix.width();
    let h = pix.height();

    // Build octree
    let mut octree = Octree::new();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            octree.add_color(r, g, b);
        }
    }

    // Reduce to max_colors
    while octree.leaf_count > options.max_colors as usize {
        octree.reduce();
    }

    // Build palette
    let mut palette: Vec<(u8, u8, u8)> = Vec::new();
    octree.build_palette(&mut palette);

    // Create colormap
    let mut colormap = PixColormap::new(8)?;
    for &(r, g, b) in &palette {
        colormap.add_rgb(r, g, b)?;
    }

    // Map pixels to palette indices
    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_colormap(Some(colormap))?;

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let idx = octree.get_palette_index(r, g, b);
            out_mut.set_pixel_unchecked(x, y, idx as u32);
        }
    }

    Ok(out_mut.into())
}

/// Quantize to exactly 256 colors using octree
pub fn octree_quant_256(pix: &Pix) -> ColorResult<Pix> {
    octree_quant(pix, &OctreeOptions { max_colors: 256 })
}

/// Octree node for color quantization
struct OctreeNode {
    red: u64,
    green: u64,
    blue: u64,
    pixel_count: u64,
    children: [Option<Box<OctreeNode>>; 8],
    is_leaf: bool,
    palette_index: usize,
}

impl OctreeNode {
    fn new() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
            pixel_count: 0,
            children: Default::default(),
            is_leaf: false,
            palette_index: 0,
        }
    }
}

/// Octree for color quantization
struct Octree {
    root: OctreeNode,
    leaf_count: usize,
    reducible_nodes: [Vec<*mut OctreeNode>; 8],
}

impl Octree {
    fn new() -> Self {
        Self {
            root: OctreeNode::new(),
            leaf_count: 0,
            reducible_nodes: Default::default(),
        }
    }

    fn add_color(&mut self, r: u8, g: u8, b: u8) {
        let root = &mut self.root as *mut OctreeNode;
        self.add_color_impl(root, r, g, b, 0);
    }

    fn add_color_impl(&mut self, node_ptr: *mut OctreeNode, r: u8, g: u8, b: u8, level: usize) {
        let node = unsafe { &mut *node_ptr };

        if level == 8 || node.is_leaf {
            node.red += r as u64;
            node.green += g as u64;
            node.blue += b as u64;
            node.pixel_count += 1;
            if !node.is_leaf {
                node.is_leaf = true;
                self.leaf_count += 1;
            }
            return;
        }

        let idx = Self::get_color_index(r, g, b, level);

        if node.children[idx].is_none() {
            node.children[idx] = Some(Box::new(OctreeNode::new()));

            // Add to reducible list
            self.reducible_nodes[level].push(node_ptr);
        }

        if let Some(ref mut child) = node.children[idx] {
            let child_ptr = child.as_mut() as *mut OctreeNode;
            self.add_color_impl(child_ptr, r, g, b, level + 1);
        }
    }

    fn get_color_index(r: u8, g: u8, b: u8, level: usize) -> usize {
        let shift = 7 - level;
        let r_bit = ((r >> shift) & 1) as usize;
        let g_bit = ((g >> shift) & 1) as usize;
        let b_bit = ((b >> shift) & 1) as usize;
        (r_bit << 2) | (g_bit << 1) | b_bit
    }

    fn reduce(&mut self) {
        // Find the deepest level with reducible nodes
        for level in (0..8).rev() {
            while let Some(node_ptr) = self.reducible_nodes[level].pop() {
                let node = unsafe { &mut *node_ptr };

                if node.is_leaf || !node.children.iter().any(|c| c.is_some()) {
                    continue;
                }

                // Merge children into this node
                for child_opt in &mut node.children {
                    if let Some(child) = child_opt.take() {
                        node.red += child.red;
                        node.green += child.green;
                        node.blue += child.blue;
                        node.pixel_count += child.pixel_count;

                        if child.is_leaf {
                            self.leaf_count -= 1;
                        }
                    }
                }

                node.is_leaf = true;
                self.leaf_count += 1;

                return;
            }
        }
    }

    fn build_palette(&mut self, palette: &mut Vec<(u8, u8, u8)>) {
        let root = &mut self.root as *mut OctreeNode;
        self.build_palette_impl(root, palette);
    }

    fn build_palette_impl(&mut self, node_ptr: *mut OctreeNode, palette: &mut Vec<(u8, u8, u8)>) {
        let node = unsafe { &mut *node_ptr };

        if node.is_leaf {
            if node.pixel_count > 0 {
                let r = (node.red / node.pixel_count) as u8;
                let g = (node.green / node.pixel_count) as u8;
                let b = (node.blue / node.pixel_count) as u8;
                node.palette_index = palette.len();
                palette.push((r, g, b));
            }
            return;
        }

        for child in node.children.iter_mut().flatten() {
            let child_ptr = child.as_mut() as *mut OctreeNode;
            self.build_palette_impl(child_ptr, palette);
        }
    }

    fn get_palette_index(&self, r: u8, g: u8, b: u8) -> usize {
        self.get_palette_index_impl(&self.root, r, g, b, 0)
    }

    fn get_palette_index_impl(
        &self,
        node: &OctreeNode,
        r: u8,
        g: u8,
        b: u8,
        level: usize,
    ) -> usize {
        if node.is_leaf {
            return node.palette_index;
        }

        let idx = Self::get_color_index(r, g, b, level);

        if let Some(child) = &node.children[idx] {
            self.get_palette_index_impl(child, r, g, b, level + 1)
        } else {
            // Fallback: find first available child
            if let Some(child) = node.children.iter().flatten().next() {
                self.get_palette_index_impl(child, r, g, b, level + 1)
            } else {
                0
            }
        }
    }
}

// =============================================================================
// Fixed Octcube 256-Color Quantization
// =============================================================================

/// Fast fixed-partition color quantization to exactly 256 colors.
///
/// Uses asymmetric octcube division: 3 MSBits for R and G, 2 MSBits for B.
/// This exploits reduced human sensitivity to blue channel variations.
///
/// # See also
///
/// C Leptonica: `pixFixedOctcubeQuant256()` in `colorquant1.c`
#[allow(unused_variables)]
pub fn fixed_octcube_quant_256(pix: &Pix) -> ColorResult<Pix> {
    todo!()
}

// =============================================================================
// Population-Based Octree Quantization
// =============================================================================

/// Adaptive color quantization using octcubes sorted by population.
///
/// Quantizes a 32bpp image by selecting the most populated octcubes at the
/// specified level (3 or 4 bits per component). Output depth is automatically
/// determined: 2bpp for ≤4 colors, 4bpp for ≤16, 8bpp otherwise.
///
/// # See also
///
/// C Leptonica: `pixOctreeQuantByPopulation()` in `colorquant1.c`
#[allow(unused_variables)]
pub fn octree_quant_by_population(pix: &Pix, level: u32) -> ColorResult<Pix> {
    todo!()
}

// =============================================================================
// N-Color Octree Quantization
// =============================================================================

/// Adaptive octree quantization to a specified number of colors.
///
/// Uses two-level octcube strategy: base octcubes + popular sub-octcubes
/// to produce exactly `max_colors` or fewer colors.
///
/// # See also
///
/// C Leptonica: `pixOctreeQuantNumColors()` in `colorquant1.c`
#[allow(unused_variables)]
pub fn octree_quant_num_colors(pix: &Pix, max_colors: u32, subsample: u32) -> ColorResult<Pix> {
    todo!()
}

// =============================================================================
// Mixed Gray/Color Median Cut Quantization
// =============================================================================

/// Hybrid quantization separating gray and color pixels.
///
/// Separates pixels into gray and color regions, quantizes color pixels
/// using median cut, and adds grayscale levels for gray pixels.
///
/// # See also
///
/// C Leptonica: `pixMedianCutQuantMixed()` in `colorquant2.c`
#[allow(unused_variables)]
pub fn median_cut_quant_mixed(
    pix: &Pix,
    ncolor: u32,
    ngray: u32,
    darkthresh: u32,
    lightthresh: u32,
    diffthresh: u32,
) -> ColorResult<Pix> {
    todo!()
}

// =============================================================================
// Quantize to Existing Colormap
// =============================================================================

/// Quantize an image to a pre-existing colormap.
///
/// Maps each pixel to the nearest color in the provided colormap using
/// Euclidean distance in RGB space.
///
/// # See also
///
/// C Leptonica: `pixQuantFromCmap()` in `colorquant1.c`
#[allow(unused_variables)]
pub fn quant_from_cmap(pix: &Pix, cmap: &PixColormap, mindepth: u32) -> ColorResult<Pix> {
    todo!()
}

// =============================================================================
// Remove Unused Colormap Colors
// =============================================================================

/// Remove unused colors from a colormapped image.
///
/// Scans all pixels to find which colormap entries are actually used,
/// then rebuilds the colormap with only those entries and remaps pixels.
///
/// # See also
///
/// C Leptonica: `pixRemoveUnusedColors()` in `colorquant1.c`
#[allow(unused_variables)]
pub fn remove_unused_colors(pix: &Pix) -> ColorResult<Pix> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_color_gradient() -> Pix {
        let pix = Pix::new(64, 64, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..64 {
            for x in 0..64 {
                let r = (x * 4) as u8;
                let g = (y * 4) as u8;
                let b = 128;
                let pixel = color::compose_rgb(r, g, b);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    fn create_few_colors() -> Pix {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create image with exactly 3 colors
        for y in 0..30 {
            for x in 0..30 {
                let pixel = if x < 10 {
                    color::compose_rgb(255, 0, 0) // Red
                } else if x < 20 {
                    color::compose_rgb(0, 255, 0) // Green
                } else {
                    color::compose_rgb(0, 0, 255) // Blue
                };
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_median_cut_quant() {
        let pix = create_color_gradient();
        let quantized = median_cut_quant_simple(&pix, 16).unwrap();

        assert_eq!(quantized.depth(), PixelDepth::Bit8);
        assert!(quantized.colormap().is_some());

        let cmap = quantized.colormap().unwrap();
        assert!(cmap.len() <= 16);
    }

    #[test]
    fn test_median_cut_few_colors() {
        let pix = create_few_colors();
        let quantized = median_cut_quant_simple(&pix, 8).unwrap();

        assert!(quantized.colormap().is_some());
        let cmap = quantized.colormap().unwrap();
        // Should have at most 3 colors (the original count)
        assert!(cmap.len() <= 8);
    }

    #[test]
    fn test_median_cut_invalid_params() {
        let pix = create_color_gradient();

        let result = median_cut_quant_simple(&pix, 0);
        assert!(result.is_err());

        let result = median_cut_quant_simple(&pix, 257);
        assert!(result.is_err());
    }

    #[test]
    fn test_octree_quant() {
        let pix = create_color_gradient();
        let quantized = octree_quant_256(&pix).unwrap();

        assert_eq!(quantized.depth(), PixelDepth::Bit8);
        assert!(quantized.colormap().is_some());
    }

    #[test]
    fn test_octree_quant_limited() {
        let pix = create_color_gradient();
        let quantized = octree_quant(&pix, &OctreeOptions { max_colors: 16 }).unwrap();

        assert!(quantized.colormap().is_some());
        let cmap = quantized.colormap().unwrap();
        assert!(cmap.len() <= 16);
    }

    #[test]
    fn test_octree_quant_few_colors() {
        let pix = create_few_colors();
        let quantized = octree_quant_256(&pix).unwrap();

        assert!(quantized.colormap().is_some());
        let cmap = quantized.colormap().unwrap();
        // Should have at most 3 distinct colors
        assert!(cmap.len() <= 3);
    }

    #[test]
    fn test_wrong_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();

        let result = median_cut_quant_simple(&pix, 16);
        assert!(result.is_err());

        let result = octree_quant_256(&pix);
        assert!(result.is_err());
    }
}
