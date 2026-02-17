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
/// Index formula: `(R & 0xe0) | ((G >> 3) & 0x1c) | (B >> 6)`.
///
/// # See also
///
/// C Leptonica: `pixFixedOctcubeQuant256()` in `colorquant1.c`
pub fn fixed_octcube_quant_256(pix: &Pix) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();

    // Build the fixed 256-entry colormap: cell centers
    let mut colormap = PixColormap::new(8)?;
    for i in 0u32..256 {
        let r = ((i >> 5) << 5) as u8 | 0x10;
        let g = (((i >> 2) & 0x07) << 5) as u8 | 0x10;
        let b = ((i & 0x03) << 6) as u8 | 0x20;
        colormap.add_rgb(r, g, b)?;
    }

    let out = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(colormap))?;

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let index = (r as u32 & 0xe0) | ((g as u32 >> 3) & 0x1c) | (b as u32 >> 6);
            out_mut.set_pixel_unchecked(x, y, index);
        }
    }

    Ok(out_mut.into())
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
pub fn octree_quant_by_population(pix: &Pix, level: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    if level != 3 && level != 4 {
        return Err(ColorError::InvalidParameters("level must be 3 or 4".into()));
    }
    let w = pix.width();
    let h = pix.height();
    let shift = 8 - level;
    let ncubes = 1usize << (3 * level);

    // Count pixels per octcube and accumulate colors
    let mut counts = vec![0u32; ncubes];
    let mut r_sums = vec![0u64; ncubes];
    let mut g_sums = vec![0u64; ncubes];
    let mut b_sums = vec![0u64; ncubes];

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let ri = (r >> shift) as usize;
            let gi = (g >> shift) as usize;
            let bi = (b >> shift) as usize;
            let idx = (ri << (2 * level)) | (gi << level) | bi;
            counts[idx] = counts[idx].saturating_add(1);
            r_sums[idx] += r as u64;
            g_sums[idx] += g as u64;
            b_sums[idx] += b as u64;
        }
    }

    // Collect occupied octcubes sorted by population (descending)
    let mut occupied: Vec<(usize, u32)> = counts
        .iter()
        .enumerate()
        .filter(|(_, c)| **c > 0)
        .map(|(i, c)| (i, *c))
        .collect();
    occupied.sort_by(|a, b| b.1.cmp(&a.1));

    // Limit to 256 colors
    let ncolors = occupied.len().min(256);
    let occupied = &occupied[..ncolors];

    // Determine output depth
    let out_depth = if ncolors <= 4 {
        PixelDepth::Bit2
    } else if ncolors <= 16 {
        PixelDepth::Bit4
    } else {
        PixelDepth::Bit8
    };

    // Build colormap from average colors of selected octcubes
    let mut colormap = PixColormap::new(out_depth.bits())?;
    let mut octcube_to_cmap = vec![0u32; ncubes];
    for (cmap_idx, &(oct_idx, _)) in occupied.iter().enumerate() {
        let c = counts[oct_idx] as u64;
        let r = (r_sums[oct_idx] / c) as u8;
        let g = (g_sums[oct_idx] / c) as u8;
        let b = (b_sums[oct_idx] / c) as u8;
        colormap.add_rgb(r, g, b)?;
        octcube_to_cmap[oct_idx] = cmap_idx as u32;
    }

    // For non-selected octcubes, find nearest selected one
    for i in 0..ncubes {
        if counts[i] > 0 && !occupied.iter().any(|&(idx, _)| idx == i) {
            let r = (r_sums[i] / counts[i] as u64) as u8;
            let g = (g_sums[i] / counts[i] as u64) as u8;
            let b = (b_sums[i] / counts[i] as u64) as u8;
            if let Some(nearest) = colormap.find_nearest(r, g, b) {
                octcube_to_cmap[i] = nearest as u32;
            }
        }
    }

    // Map pixels
    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(colormap))?;

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let ri = (r >> shift) as usize;
            let gi = (g >> shift) as usize;
            let bi = (b >> shift) as usize;
            let idx = (ri << (2 * level)) | (gi << level) | bi;
            out_mut.set_pixel_unchecked(x, y, octcube_to_cmap[idx]);
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// N-Color Octree Quantization
// =============================================================================

/// Adaptive octree quantization to a specified number of colors.
///
/// Uses the existing octree quantization infrastructure, reducing to the
/// target number of colors. The `subsample` parameter controls how many
/// pixels are sampled for building the color distribution (0 = use all).
///
/// # See also
///
/// C Leptonica: `pixOctreeQuantNumColors()` in `colorquant1.c`
pub fn octree_quant_num_colors(pix: &Pix, max_colors: u32, subsample: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    if !(8..=256).contains(&max_colors) {
        return Err(ColorError::InvalidParameters(
            "max_colors must be between 8 and 256".into(),
        ));
    }
    let w = pix.width();
    let h = pix.height();
    let sub = if subsample == 0 {
        (w.min(h) / 200).max(1)
    } else {
        subsample.max(1)
    };

    // Build octree with optional subsampling
    let mut octree = Octree::new();
    for y in (0..h).step_by(sub as usize) {
        for x in (0..w).step_by(sub as usize) {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            octree.add_color(r, g, b);
        }
    }

    // Reduce to max_colors
    while octree.leaf_count > max_colors as usize {
        octree.reduce();
    }

    // Build palette
    let mut palette: Vec<(u8, u8, u8)> = Vec::new();
    octree.build_palette(&mut palette);

    // Determine output depth
    let out_depth = if palette.len() <= 16 {
        PixelDepth::Bit4
    } else {
        PixelDepth::Bit8
    };

    // Create colormap
    let mut colormap = PixColormap::new(out_depth.bits())?;
    for &(r, g, b) in &palette {
        colormap.add_rgb(r, g, b)?;
    }

    // Map all pixels to nearest palette entry
    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(colormap.clone()))?;

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

// =============================================================================
// Mixed Gray/Color Median Cut Quantization
// =============================================================================

/// Hybrid quantization separating gray and color pixels.
///
/// Separates pixels into gray and color regions, quantizes color pixels
/// using median cut, and adds grayscale levels for gray pixels.
/// Gray pixels are those where `max(|R-G|, |R-B|, |G-B|) <= diffthresh`.
///
/// # See also
///
/// C Leptonica: `pixMedianCutQuantMixed()` in `colorquant2.c`
pub fn median_cut_quant_mixed(
    pix: &Pix,
    ncolor: u32,
    ngray: u32,
    darkthresh: u32,
    lightthresh: u32,
    diffthresh: u32,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    if ncolor == 0 {
        return Err(ColorError::InvalidParameters(
            "ncolor must be at least 1".into(),
        ));
    }
    if ngray < 2 {
        return Err(ColorError::InvalidParameters(
            "ngray must be at least 2".into(),
        ));
    }
    if ncolor + ngray > 255 {
        return Err(ColorError::InvalidParameters(
            "ncolor + ngray must be ≤ 255".into(),
        ));
    }

    let w = pix.width();
    let h = pix.height();

    // Classify each pixel as gray or color
    let mut is_gray = vec![false; (w * h) as usize];
    let mut gray_count = 0u32;

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let rg = (r as i32 - g as i32).unsigned_abs();
            let rb = (r as i32 - b as i32).unsigned_abs();
            let gb = (g as i32 - b as i32).unsigned_abs();
            let max_diff = rg.max(rb).max(gb);
            let idx = (y * w + x) as usize;
            if max_diff <= diffthresh {
                is_gray[idx] = true;
                gray_count += 1;
            }
        }
    }

    // First, quantize color pixels using median cut
    let color_count = w * h - gray_count;
    let mut color_pixels: Vec<[u8; 3]> = Vec::with_capacity(color_count as usize);
    let mut color_indices: Vec<usize> = Vec::with_capacity(color_count as usize);

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if !is_gray[idx] {
                let pixel = pix.get_pixel_unchecked(x, y);
                let (r, g, b) = color::extract_rgb(pixel);
                color_indices.push(color_pixels.len());
                color_pixels.push([r, g, b]);
            }
        }
    }

    // Build colormap: first ncolor entries for color, then ngray entries for gray
    let mut colormap = PixColormap::new(8)?;

    // Quantize color pixels if any
    let mut color_palette: Vec<(u8, u8, u8)> = Vec::new();
    if !color_pixels.is_empty() {
        let mut boxes: BinaryHeap<ColorBox> = BinaryHeap::new();
        boxes.push(ColorBox::from_pixels(
            &color_pixels,
            (0..color_pixels.len()).collect(),
        ));

        while boxes.len() < ncolor as usize {
            if let Some(box_) = boxes.pop() {
                if box_.indices.len() <= 1 {
                    boxes.push(box_);
                    break;
                }
                match box_.split(&color_pixels) {
                    Ok((b1, b2)) => {
                        boxes.push(b1);
                        boxes.push(b2);
                    }
                    Err(original) => {
                        boxes.push(original);
                        break;
                    }
                }
            } else {
                break;
            }
        }

        let box_vec: Vec<ColorBox> = boxes.into_vec();
        for b in &box_vec {
            let (r, g, b_val) = b.average_color(&color_pixels);
            color_palette.push((r, g, b_val));
        }

        for &(r, g, b) in &color_palette {
            colormap.add_rgb(r, g, b)?;
        }
    }

    // Add grayscale entries
    let color_entries = colormap.len();
    for i in 0..ngray {
        let val = if ngray > 1 {
            (i * 255 / (ngray - 1)) as u8
        } else {
            128
        };
        colormap.add_rgb(val, val, val)?;
    }

    // Map all pixels
    let out = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(colormap.clone()))?;

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if is_gray[idx] {
                // Map to nearest gray level
                let pixel = pix.get_pixel_unchecked(x, y);
                let (r, g, b) = color::extract_rgb(pixel);
                let gray_val = ((r as u32 + g as u32 + b as u32) / 3) as u8;

                // Clamp to dark/light thresholds
                let gray_val = if (gray_val as u32) <= darkthresh {
                    0u8
                } else if (gray_val as u32) >= lightthresh {
                    255u8
                } else {
                    gray_val
                };

                // Find nearest gray entry
                let gray_idx = if ngray > 1 {
                    let scaled = gray_val as f32 / 255.0 * (ngray - 1) as f32;
                    scaled.round() as u32
                } else {
                    0
                };
                let cmap_idx = color_entries as u32 + gray_idx.min(ngray - 1);
                out_mut.set_pixel_unchecked(x, y, cmap_idx);
            } else {
                // Find nearest color entry (search only color portion of colormap)
                let pixel = pix.get_pixel_unchecked(x, y);
                let (r, g, b) = color::extract_rgb(pixel);
                let mut best_idx = 0u32;
                let mut best_dist = u32::MAX;
                for i in 0..color_entries {
                    if let Some((cr, cg, cb)) = colormap.get_rgb(i) {
                        let dr = r as i32 - cr as i32;
                        let dg = g as i32 - cg as i32;
                        let db = b as i32 - cb as i32;
                        let dist = (dr * dr + dg * dg + db * db) as u32;
                        if dist < best_dist {
                            best_dist = dist;
                            best_idx = i as u32;
                        }
                    }
                }
                out_mut.set_pixel_unchecked(x, y, best_idx);
            }
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Quantize to Existing Colormap
// =============================================================================

/// Quantize an image to a pre-existing colormap.
///
/// Maps each pixel to the nearest color in the provided colormap using
/// Euclidean distance in RGB space. Accepts 8bpp grayscale or 32bpp RGB.
///
/// # See also
///
/// C Leptonica: `pixQuantFromCmap()` in `colorquant1.c`
pub fn quant_from_cmap(pix: &Pix, cmap: &PixColormap, mindepth: u32) -> ColorResult<Pix> {
    let depth = pix.depth();
    if !matches!(depth, PixelDepth::Bit8 | PixelDepth::Bit32) {
        return Err(ColorError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: depth.bits(),
        });
    }

    if cmap.is_empty() {
        return Err(ColorError::InvalidParameters(
            "colormap must not be empty".into(),
        ));
    }

    if !matches!(mindepth, 1 | 2 | 4 | 8) {
        return Err(ColorError::InvalidParameters(format!(
            "mindepth must be 1, 2, 4, or 8; got {mindepth}"
        )));
    }

    let w = pix.width();
    let h = pix.height();

    // Determine output depth: enough for cmap size, at least mindepth
    let needed_depth = if cmap.len() <= 2 {
        1
    } else if cmap.len() <= 4 {
        2
    } else if cmap.len() <= 16 {
        4
    } else {
        8
    };
    let out_depth_bits = needed_depth.max(mindepth);
    let out_depth = match out_depth_bits {
        1 => PixelDepth::Bit1,
        2 => PixelDepth::Bit2,
        4 => PixelDepth::Bit4,
        _ => PixelDepth::Bit8,
    };

    // Rebuild colormap at output depth
    let mut out_cmap = PixColormap::new(out_depth_bits)?;
    for i in 0..cmap.len() {
        if let Some((r, g, b)) = cmap.get_rgb(i) {
            out_cmap.add_rgb(r, g, b)?;
        }
    }

    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(out_cmap))?;

    if depth == PixelDepth::Bit8 {
        if let Some(src_cmap) = pix.colormap() {
            // 8bpp colormapped: build LUT over source colormap indices
            let src_len = src_cmap.len();
            let mut lut = vec![0u32; src_len];
            for (i, entry) in lut.iter_mut().enumerate() {
                if let Some((r, g, b)) = src_cmap.get_rgb(i)
                    && let Some(idx) = cmap.find_nearest(r, g, b)
                {
                    *entry = idx as u32;
                }
            }

            for y in 0..h {
                for x in 0..w {
                    let idx = pix.get_pixel_unchecked(x, y) as usize;
                    let new_idx = if idx < lut.len() { lut[idx] } else { 0 };
                    out_mut.set_pixel_unchecked(x, y, new_idx);
                }
            }
        } else {
            // Grayscale: build LUT for 256 gray values
            let mut lut = [0u32; 256];
            for val in 0u16..256 {
                let v = val as u8;
                if let Some(idx) = cmap.find_nearest(v, v, v) {
                    lut[val as usize] = idx as u32;
                }
            }

            for y in 0..h {
                for x in 0..w {
                    let val = pix.get_pixel_unchecked(x, y) as usize;
                    out_mut.set_pixel_unchecked(x, y, lut[val.min(255)]);
                }
            }
        }
    } else {
        // RGB: map each pixel to nearest
        for y in 0..h {
            for x in 0..w {
                let pixel = pix.get_pixel_unchecked(x, y);
                let (r, g, b) = color::extract_rgb(pixel);
                if let Some(idx) = cmap.find_nearest(r, g, b) {
                    out_mut.set_pixel_unchecked(x, y, idx as u32);
                }
            }
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Remove Unused Colormap Colors
// =============================================================================

/// Remove unused colors from a colormapped image.
///
/// Scans all pixels to find which colormap entries are actually used,
/// then rebuilds the colormap with only those entries and remaps pixels.
/// Returns an error if the image has no colormap.
///
/// # See also
///
/// C Leptonica: `pixRemoveUnusedColors()` in `colorquant1.c`
pub fn remove_unused_colors(pix: &Pix) -> ColorResult<Pix> {
    let cmap = pix
        .colormap()
        .ok_or_else(|| ColorError::InvalidParameters("image must have a colormap".into()))?;

    let w = pix.width();
    let h = pix.height();
    let cmap_len = cmap.len();

    // Build usage histogram
    let mut used = vec![false; cmap_len];
    for y in 0..h {
        for x in 0..w {
            let idx = pix.get_pixel_unchecked(x, y) as usize;
            if idx < cmap_len {
                used[idx] = true;
            }
        }
    }

    // Build old→new index mapping
    let mut old_to_new = vec![0u32; cmap_len];
    let mut new_cmap = PixColormap::new(cmap.depth())?;
    for (old_idx, &is_used) in used.iter().enumerate() {
        if is_used {
            let new_idx = new_cmap.len();
            old_to_new[old_idx] = new_idx as u32;
            if let Some((r, g, b)) = cmap.get_rgb(old_idx) {
                new_cmap.add_rgb(r, g, b)?;
            }
        }
    }

    // Remap pixels
    let out = Pix::new(w, h, pix.depth())?;
    let mut out_mut = out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(new_cmap))?;

    for y in 0..h {
        for x in 0..w {
            let old_idx = pix.get_pixel_unchecked(x, y) as usize;
            let new_idx = if old_idx < cmap_len {
                old_to_new[old_idx]
            } else {
                0
            };
            out_mut.set_pixel_unchecked(x, y, new_idx);
        }
    }

    Ok(out_mut.into())
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
