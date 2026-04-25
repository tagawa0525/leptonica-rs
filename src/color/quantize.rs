//! Color quantization
//!
//! Reduces the number of colors in an image while preserving visual quality:
//! - Median cut algorithm
//! - Octree quantization

use crate::color::{ColorError, ColorResult};
use crate::core::{Pix, PixColormap, PixelDepth, pixel};
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
            let (r, g, b) = pixel::extract_rgb(pixel);
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
            let (r, g, b) = pixel::extract_rgb(pixel);
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
            let (r, g, b) = pixel::extract_rgb(pixel);
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
    reducible_nodes: [Vec<u32>; 8],
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
        Self::add_color_impl(
            &mut self.root,
            r,
            g,
            b,
            0,
            0,
            &mut self.leaf_count,
            &mut self.reducible_nodes,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn add_color_impl(
        node: &mut OctreeNode,
        r: u8,
        g: u8,
        b: u8,
        level: usize,
        path: u32,
        leaf_count: &mut usize,
        reducible_nodes: &mut [Vec<u32>; 8],
    ) {
        if level == 8 || node.is_leaf {
            node.red += r as u64;
            node.green += g as u64;
            node.blue += b as u64;
            node.pixel_count += 1;
            if !node.is_leaf {
                node.is_leaf = true;
                *leaf_count += 1;
            }
            return;
        }

        let idx = Self::get_color_index(r, g, b, level);

        if node.children[idx].is_none() {
            // Only register as reducible when the node gets its first child
            let is_first_child = node.children.iter().all(|c| c.is_none());
            node.children[idx] = Some(Box::new(OctreeNode::new()));
            if is_first_child {
                reducible_nodes[level].push(path);
            }
        }

        if let Some(child) = node.children[idx].as_deref_mut() {
            Self::add_color_impl(
                child,
                r,
                g,
                b,
                level + 1,
                (path << 3) | idx as u32,
                leaf_count,
                reducible_nodes,
            );
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
            while let Some(path) = self.reducible_nodes[level].pop() {
                let Some(node) = Self::node_mut_at_path(&mut self.root, level, path) else {
                    continue;
                };

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

    fn node_mut_at_path(root: &mut OctreeNode, level: usize, path: u32) -> Option<&mut OctreeNode> {
        let mut node = root;
        for depth in 0..level {
            let shift = (level - depth - 1) * 3;
            let idx = ((path >> shift) & 0x07) as usize;
            node = node.children[idx].as_deref_mut()?;
        }
        Some(node)
    }

    fn build_palette(&mut self, palette: &mut Vec<(u8, u8, u8)>) {
        Self::build_palette_impl(&mut self.root, palette);
    }

    fn build_palette_impl(node: &mut OctreeNode, palette: &mut Vec<(u8, u8, u8)>) {
        if node.is_leaf {
            if let Some(count) = std::num::NonZeroU64::new(node.pixel_count) {
                let count = count.get();
                let r = (node.red / count) as u8;
                let g = (node.green / count) as u8;
                let b = (node.blue / count) as u8;
                node.palette_index = palette.len();
                palette.push((r, g, b));
            }
            return;
        }

        for child in node.children.iter_mut().flatten() {
            Self::build_palette_impl(child, palette);
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
            let (r, g, b) = pixel::extract_rgb(pixel);
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
            let (r, g, b) = pixel::extract_rgb(pixel);
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
    occupied.sort_by_key(|a| std::cmp::Reverse(a.1));

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
            let (r, g, b) = pixel::extract_rgb(pixel);
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
            let (r, g, b) = pixel::extract_rgb(pixel);
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
            let (r, g, b) = pixel::extract_rgb(pixel);
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
            let (r, g, b) = pixel::extract_rgb(pixel);
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
                let (r, g, b) = pixel::extract_rgb(pixel);
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
                let (r, g, b) = pixel::extract_rgb(pixel);
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
                let (r, g, b) = pixel::extract_rgb(pixel);
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
                let (r, g, b) = pixel::extract_rgb(pixel);
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

// =============================================================================
// Octcube Index Helper
// =============================================================================

/// Compute octcube index for an (r,g,b) pixel at given level.
/// level 1: 8 cubes, level 2: 64 cubes, level 3: 512 cubes, etc.
fn octcube_index(r: u8, g: u8, b: u8, level: u32) -> u32 {
    let shift = 8 - level;
    let ri = (r as u32 >> shift) & ((1 << level) - 1);
    let gi = (g as u32 >> shift) & ((1 << level) - 1);
    let bi = (b as u32 >> shift) & ((1 << level) - 1);
    (ri << (2 * level)) | (gi << level) | bi
}

/// Compute the center RGB values for an octcube at a given level and index.
fn octcube_center(index: u32, level: u32) -> (u8, u8, u8) {
    let mask = (1u32 << level) - 1;
    let ri = (index >> (2 * level)) & mask;
    let gi = (index >> level) & mask;
    let bi = index & mask;
    let shift = 8 - level;
    let half = 1u32 << (shift - 1);
    let r = ((ri << shift) + half).min(255) as u8;
    let g = ((gi << shift) + half).min(255) as u8;
    let b = ((bi << shift) + half).min(255) as u8;
    (r, g, b)
}

// =============================================================================
// Octcube Quantization with Gray Mixing
// =============================================================================

/// Quantize using octcube for color pixels and gray levels for near-gray pixels.
///
/// Generates a colormapped image where the colormap has two sections:
/// octcube entries for color pixels, and grayscale entries for near-gray pixels.
/// The `delta` threshold determines whether a pixel is color or gray based on
/// the maximum difference between the min and max of its RGB components.
///
/// # See also
///
/// C Leptonica: `pixOctcubeQuantMixedWithGray()` in `colorquant1.c`
pub fn octcube_quant_mixed_with_gray(
    pix: &Pix,
    depth: u32,
    gray_levels: u32,
    delta: u32,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if gray_levels < 2 {
        return Err(ColorError::InvalidParameters(
            "gray_levels must be at least 2".into(),
        ));
    }

    let (octlevels, size) = if depth == 4 {
        if gray_levels > 8 {
            return Err(ColorError::InvalidParameters(
                "max 8 gray levels for depth 4".into(),
            ));
        }
        (1u32, 8u32)
    } else if depth == 8 {
        if gray_levels > 192 {
            return Err(ColorError::InvalidParameters(
                "max 192 gray levels for depth 8".into(),
            ));
        }
        (2u32, 64u32)
    } else {
        return Err(ColorError::InvalidParameters("depth must be 4 or 8".into()));
    };

    let w = pix.width();
    let h = pix.height();

    // Build gray quantization lookup table
    let mut tabval = vec![0u32; 256];
    for i in 0u32..256 {
        tabval[i as usize] = (i * gray_levels / 256).min(gray_levels - 1);
    }

    // Create colormap: first `size` entries for color octcubes (placeholder),
    // then `gray_levels` entries for gray
    let out_depth = if depth == 4 {
        PixelDepth::Bit4
    } else {
        PixelDepth::Bit8
    };
    let mut colormap = PixColormap::new(depth)?;
    for _ in 0..size {
        colormap.add_rgb(1, 1, 1)?; // placeholder for octcube colors
    }
    for j in 0..gray_levels {
        let val = (255 * j / (gray_levels - 1)) as u8;
        colormap.add_rgb(val, val, val)?;
    }

    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();

    // Per-octcube accumulators
    let mut carray = vec![0u64; size as usize];
    let mut rarray = vec![0u64; size as usize];
    let mut garray = vec![0u64; size as usize];
    let mut barray = vec![0u64; size as usize];

    // Assign each pixel to color octcube or gray level
    for y in 0..h {
        for x in 0..w {
            let pval = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pval);
            let rval = r as i32;
            let gval = g as i32;
            let bval = b as i32;

            // Compute max-min difference and midval
            let (del, midval) = if rval > gval {
                if gval > bval {
                    (rval - bval, gval)
                } else if rval > bval {
                    (rval - gval, bval)
                } else {
                    (bval - gval, rval)
                }
            } else if rval > bval {
                (gval - bval, rval)
            } else if gval > bval {
                (gval - rval, bval)
            } else {
                (bval - rval, gval)
            };

            if del > delta as i32 {
                // Color pixel → octcube
                let octindex = octcube_index(r, g, b, octlevels);
                carray[octindex as usize] += 1;
                rarray[octindex as usize] += rval as u64;
                garray[octindex as usize] += gval as u64;
                barray[octindex as usize] += bval as u64;
                out_mut.set_pixel_unchecked(x, y, octindex);
            } else {
                // Gray pixel
                let val = size + tabval[midval as usize];
                out_mut.set_pixel_unchecked(x, y, val);
            }
        }
    }

    // Average the colors in each occupied octcube and update the colormap
    for i in 0..size as usize {
        if let Some(count) = std::num::NonZeroU64::new(carray[i]) {
            let count = count.get();
            let r = (rarray[i] / count) as u8;
            let g = (garray[i] / count) as u8;
            let b = (barray[i] / count) as u8;
            if let Some(entry) = colormap.get_mut(i) {
                entry.red = r;
                entry.green = g;
                entry.blue = b;
            }
        }
    }

    out_mut.set_colormap(Some(colormap))?;
    Ok(out_mut.into())
}

// =============================================================================
// Few-Colors Octcube Quantization
// =============================================================================

/// Quantize an image with few colors using octcube at given level.
///
/// Builds an octcube histogram at the given level, averages colors in each
/// occupied cube to produce the colormap, then maps pixels. Fails if more
/// than 256 cubes are occupied. Output depth is 2, 4, or 8 bpp.
///
/// # See also
///
/// C Leptonica: `pixFewColorsOctcubeQuant1()` in `colorquant1.c`
pub fn few_colors_octcube_quant1(pix: &Pix, level: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if !(1..=6).contains(&level) {
        return Err(ColorError::InvalidParameters(
            "level must be between 1 and 6".into(),
        ));
    }

    let w = pix.width();
    let h = pix.height();
    let size = 1u32 << (3 * level);

    // First pass: accumulate per-octcube
    let mut carray = vec![0u64; size as usize];
    let mut rarray = vec![0u64; size as usize];
    let mut garray = vec![0u64; size as usize];
    let mut barray = vec![0u64; size as usize];

    for y in 0..h {
        for x in 0..w {
            let pval = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pval);
            let idx = octcube_index(r, g, b, level) as usize;
            carray[idx] += 1;
            rarray[idx] += r as u64;
            garray[idx] += g as u64;
            barray[idx] += b as u64;
        }
    }

    // Count occupied cubes and build mapping
    let mut ncolors = 0u32;
    let mut cube_to_cmap = vec![0u32; size as usize];
    for i in 0..size as usize {
        if carray[i] > 0 {
            cube_to_cmap[i] = ncolors;
            ncolors += 1;
        }
    }

    if ncolors > 256 {
        return Err(ColorError::InvalidParameters(format!(
            "too many occupied octcubes: {ncolors} (max 256)"
        )));
    }

    let out_depth_bits = if ncolors <= 4 {
        2
    } else if ncolors <= 16 {
        4
    } else {
        8
    };
    let out_depth = match out_depth_bits {
        2 => PixelDepth::Bit2,
        4 => PixelDepth::Bit4,
        _ => PixelDepth::Bit8,
    };

    // Build colormap from averaged colors
    let mut colormap = PixColormap::new(out_depth_bits)?;
    for i in 0..size as usize {
        if let Some(count) = std::num::NonZeroU64::new(carray[i]) {
            let count = count.get();
            let r = (rarray[i] / count) as u8;
            let g = (garray[i] / count) as u8;
            let b = (barray[i] / count) as u8;
            colormap.add_rgb(r, g, b)?;
        }
    }

    // Second pass: map pixels
    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(colormap))?;

    for y in 0..h {
        for x in 0..w {
            let pval = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pval);
            let idx = octcube_index(r, g, b, level) as usize;
            out_mut.set_pixel_unchecked(x, y, cube_to_cmap[idx]);
        }
    }

    Ok(out_mut.into())
}

/// Quantize with few colors, using a pre-computed ncolors estimate.
///
/// Similar to `few_colors_octcube_quant1` but uses the provided `ncolors`
/// estimate. Assigns the color of the first pixel found in each octcube
/// rather than the average.
///
/// # See also
///
/// C Leptonica: `pixFewColorsOctcubeQuant2()` in `colorquant1.c`
pub fn few_colors_octcube_quant2(pix: &Pix, level: u32, ncolors: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if !(3..=6).contains(&level) {
        return Err(ColorError::InvalidParameters(
            "level must be between 3 and 6".into(),
        ));
    }
    if ncolors > 256 {
        return Err(ColorError::InvalidParameters(
            "ncolors must be <= 256".into(),
        ));
    }

    let w = pix.width();
    let h = pix.height();
    let size = 1u32 << (3 * level);

    // octarray maps octcube index → (cindex+1), 0 means unoccupied
    let mut octarray = vec![0u32; size as usize];
    // colorarray stores the first pixel color for each cmap entry
    let mut colorarray: Vec<u32> = vec![0; ncolors as usize + 1];

    let out_depth_bits = if ncolors <= 4 {
        2
    } else if ncolors <= 16 {
        4
    } else {
        8
    };
    let out_depth = match out_depth_bits {
        2 => PixelDepth::Bit2,
        4 => PixelDepth::Bit4,
        _ => PixelDepth::Bit8,
    };

    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();

    let mut cindex = 1u32;
    for y in 0..h {
        for x in 0..w {
            let pval = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pval);
            let octindex = octcube_index(r, g, b, level) as usize;
            let oval = octarray[octindex];
            if oval == 0 {
                octarray[octindex] = cindex;
                colorarray[cindex as usize] = pval;
                out_mut.set_pixel_unchecked(x, y, cindex - 1);
                cindex += 1;
            } else {
                out_mut.set_pixel_unchecked(x, y, oval - 1);
            }
        }
    }

    // Build colormap from first-found pixels
    let actual_colors = cindex - 1;
    let mut colormap = PixColormap::new(out_depth_bits)?;
    for i in 1..=actual_colors {
        let (r, g, b) = pixel::extract_rgb(colorarray[i as usize]);
        colormap.add_rgb(r, g, b)?;
    }

    out_mut.set_colormap(Some(colormap))?;
    Ok(out_mut.into())
}

/// Few-colors quantization with gray mixing.
///
/// First tries `few_colors_octcube_quant1`. If that fails (too many colors),
/// falls back to `octcube_quant_mixed_with_gray` with depth 8.
///
/// # See also
///
/// C Leptonica: `pixFewColorsOctcubeQuantMixed()` in `colorquant1.c`
pub fn few_colors_octcube_quant_mixed(pix: &Pix, level: u32, dark_thresh: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let level = if level == 0 { 3 } else { level };
    if level > 6 {
        return Err(ColorError::InvalidParameters("level must be <= 6".into()));
    }
    let dark_thresh = if dark_thresh == 0 { 20 } else { dark_thresh };

    // Try few-colors quantization first
    match few_colors_octcube_quant1(pix, level) {
        Ok(quant1) => {
            // Separate color from gray entries and re-quantize gray pixels
            let cmap = quant1.colormap().ok_or_else(|| {
                ColorError::InvalidParameters("expected colormapped image".into())
            })?;
            let ncolors = cmap.len();
            let w = pix.width();
            let h = pix.height();

            // Classify cmap entries as color or gray
            let diff_thresh = 20u32;
            let light_thresh = 244u32;
            let mut is_color_entry = vec![false; ncolors];
            let mut color_lut = vec![-1i32; ncolors]; // old index → new color index
            let mut cmapd = PixColormap::new(8)?;

            let mut color_idx = 0i32;
            for i in 0..ncolors {
                if let Some((r, g, b)) = cmap.get_rgb(i) {
                    let minval = r.min(g).min(b) as u32;
                    let maxval = r.max(g).max(b) as u32;
                    if minval > light_thresh || maxval < dark_thresh {
                        continue;
                    }
                    if maxval - minval >= diff_thresh {
                        is_color_entry[i] = true;
                        cmapd.add_rgb(r, g, b)?;
                        color_lut[i] = color_idx;
                        color_idx += 1;
                    }
                }
            }

            // Build output: color pixels get mapped, gray pixels filled later
            let out = Pix::new(w, h, PixelDepth::Bit8)?;
            let mut out_mut = out.try_into_mut().unwrap();

            let n_color_entries = cmapd.len();

            // Add gray entries (256 levels minus color entries)
            let ngray = (256 - n_color_entries).max(2);
            for i in 0..ngray {
                let val = (i * 255 / (ngray - 1)) as u8;
                if cmapd.len() >= 256 {
                    break;
                }
                cmapd.add_rgb(val, val, val)?;
            }

            out_mut.set_colormap(Some(cmapd))?;

            for y in 0..h {
                for x in 0..w {
                    let old_idx = quant1.get_pixel_unchecked(x, y) as usize;
                    if old_idx < ncolors && is_color_entry[old_idx] {
                        out_mut.set_pixel_unchecked(x, y, color_lut[old_idx] as u32);
                    } else {
                        // Map from source RGB to gray
                        let pval = pix.get_pixel_unchecked(x, y);
                        let (r, g, b) = pixel::extract_rgb(pval);
                        let gray = ((r as u32 + g as u32 + b as u32) / 3) as u8;
                        let gray_idx = if ngray > 1 {
                            let scaled = gray as f32 / 255.0 * (ngray - 1) as f32;
                            scaled.round() as u32
                        } else {
                            0
                        };
                        let cmap_idx = n_color_entries as u32 + gray_idx.min(ngray as u32 - 1);
                        out_mut.set_pixel_unchecked(x, y, cmap_idx);
                    }
                }
            }

            Ok(out_mut.into())
        }
        Err(_) => {
            // Fallback to octcube quant mixed with gray
            octcube_quant_mixed_with_gray(pix, 8, 64, dark_thresh)
        }
    }
}

// =============================================================================
// Fixed Partition Octcube Quantization with RGB Output
// =============================================================================

/// Fixed octcube quantization generating RGB output (not colormapped).
///
/// For each pixel, replaces it with the center color of the octcube
/// it falls into at the given level. This produces a 32bpp RGB image
/// with quantized colors rather than a colormapped image.
///
/// # See also
///
/// C Leptonica: `pixFixedOctcubeQuantGenRGB()` in `colorquant1.c`
pub fn fixed_octcube_quant_gen_rgb(pix: &Pix, level: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if !(1..=6).contains(&level) {
        return Err(ColorError::InvalidParameters(
            "level must be between 1 and 6".into(),
        ));
    }

    let w = pix.width();
    let h = pix.height();

    let out = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pval = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pval);
            let idx = octcube_index(r, g, b, level);
            let (qr, qg, qb) = octcube_center(idx, level);
            out_mut.set_pixel_unchecked(x, y, pixel::compose_rgb(qr, qg, qb));
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Octcube Quantization from Existing Colormap
// =============================================================================

/// Quantize a 32bpp RGB image using an existing colormap via nearest-color search.
///
/// For each pixel, finds the nearest color in the colormap using Euclidean
/// distance in RGB space. Output depth is determined by colormap size and mindepth.
///
/// # See also
///
/// C Leptonica: `pixOctcubeQuantFromCmap()` in `colorquant1.c`
pub fn octcube_quant_from_cmap(pix: &Pix, cmap: &PixColormap, mindepth: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if cmap.is_empty() {
        return Err(ColorError::InvalidParameters(
            "colormap must not be empty".into(),
        ));
    }
    if !matches!(mindepth, 2 | 4 | 8) {
        return Err(ColorError::InvalidParameters(format!(
            "mindepth must be 2, 4, or 8; got {mindepth}"
        )));
    }

    let w = pix.width();
    let h = pix.height();

    let needed_depth = if cmap.len() <= 4 {
        2
    } else if cmap.len() <= 16 {
        4
    } else {
        8
    };
    let out_depth_bits = needed_depth.max(mindepth);
    let out_depth = match out_depth_bits {
        2 => PixelDepth::Bit2,
        4 => PixelDepth::Bit4,
        _ => PixelDepth::Bit8,
    };

    let mut out_cmap = PixColormap::new(out_depth_bits)?;
    for i in 0..cmap.len() {
        if let Some((r, g, b)) = cmap.get_rgb(i) {
            out_cmap.add_rgb(r, g, b)?;
        }
    }

    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(out_cmap))?;

    for y in 0..h {
        for x in 0..w {
            let pval = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pval);
            if let Some(idx) = cmap.find_nearest(r, g, b) {
                out_mut.set_pixel_unchecked(x, y, idx as u32);
            }
        }
    }

    Ok(out_mut.into())
}

/// Quantize using an existing colormap via octcube lookup table for speed.
///
/// Builds a LUT mapping each octcube index (at level 4) to the nearest
/// colormap entry, then uses it to map pixels. Faster than per-pixel
/// nearest search for large images.
///
/// # See also
///
/// C Leptonica: `pixOctcubeQuantFromCmapLUT()` in `colorquant1.c`
pub fn octcube_quant_from_cmap_lut(
    pix: &Pix,
    cmap: &PixColormap,
    mindepth: u32,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if cmap.is_empty() {
        return Err(ColorError::InvalidParameters(
            "colormap must not be empty".into(),
        ));
    }
    if !matches!(mindepth, 2 | 4 | 8) {
        return Err(ColorError::InvalidParameters(format!(
            "mindepth must be 2, 4, or 8; got {mindepth}"
        )));
    }

    let lut_level = 4u32;
    let lut_size = 1u32 << (3 * lut_level); // 4096

    // Build LUT: for each octcube center, find nearest cmap entry
    let mut lut = vec![0u32; lut_size as usize];
    for i in 0..lut_size {
        let (cr, cg, cb) = octcube_center(i, lut_level);
        if let Some(idx) = cmap.find_nearest(cr, cg, cb) {
            lut[i as usize] = idx as u32;
        }
    }

    let w = pix.width();
    let h = pix.height();

    let needed_depth = if cmap.len() <= 4 {
        2
    } else if cmap.len() <= 16 {
        4
    } else {
        8
    };
    let out_depth_bits = needed_depth.max(mindepth);
    let out_depth = match out_depth_bits {
        2 => PixelDepth::Bit2,
        4 => PixelDepth::Bit4,
        _ => PixelDepth::Bit8,
    };

    let mut out_cmap = PixColormap::new(out_depth_bits)?;
    for i in 0..cmap.len() {
        if let Some((r, g, b)) = cmap.get_rgb(i) {
            out_cmap.add_rgb(r, g, b)?;
        }
    }

    let out = Pix::new(w, h, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();
    out_mut.set_colormap(Some(out_cmap))?;

    for y in 0..h {
        for x in 0..w {
            let pval = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pval);
            let idx = octcube_index(r, g, b, lut_level) as usize;
            out_mut.set_pixel_unchecked(x, y, lut[idx]);
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Octcube Tree (Histogram)
// =============================================================================

/// Result of building an octcube tree: a histogram of pixel counts per octcube.
#[derive(Debug, Clone)]
pub struct OctcubeTree {
    /// Count of pixels in each octcube at the given level.
    pub histogram: Vec<u32>,
    /// The octcube level used.
    pub level: u32,
}

/// Build octcube tree (histogram of pixel counts per octcube).
///
/// Counts how many pixels fall into each octcube at the given level.
///
/// # See also
///
/// C Leptonica: `pixOctcubeTree()` (histogram generation part)
pub fn octcube_tree(pix: &Pix, level: u32) -> ColorResult<OctcubeTree> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if !(1..=6).contains(&level) {
        return Err(ColorError::InvalidParameters(
            "level must be between 1 and 6".into(),
        ));
    }

    let w = pix.width();
    let h = pix.height();
    let size = 1u32 << (3 * level);
    let mut histogram = vec![0u32; size as usize];

    for y in 0..h {
        for x in 0..w {
            let pval = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pval);
            let idx = octcube_index(r, g, b, level) as usize;
            histogram[idx] += 1;
        }
    }

    Ok(OctcubeTree { histogram, level })
}

// =============================================================================
// Number of Occupied Octcubes
// =============================================================================

/// Count number of octcubes that contain at least `min_count` pixels.
///
/// # See also
///
/// C Leptonica: `pixNumberOccupiedOctcubes()` in `colorquant1.c`
pub fn number_occupied_octcubes(pix: &Pix, level: u32, min_count: u32) -> ColorResult<u32> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if !(1..=6).contains(&level) {
        return Err(ColorError::InvalidParameters(
            "level must be between 1 and 6".into(),
        ));
    }

    let min_count = min_count.max(1);
    let tree = octcube_tree(pix, level)?;
    let count = tree.histogram.iter().filter(|&&c| c >= min_count).count() as u32;
    Ok(count)
}

// =============================================================================
// Few Colors Median Cut Quantization Mixed
// =============================================================================

/// Median cut quantization for few-color images with gray mixing.
///
/// First estimates the number of colors in the image. If more than
/// `ncolors`, returns an error. If the image has no significant color
/// content, converts to grayscale and quantizes. Otherwise uses
/// `median_cut_quant_mixed` for the actual quantization.
///
/// # See also
///
/// C Leptonica: `pixFewColorsMedianCutQuantMixed()` in `colorquant2.c`
pub fn few_colors_median_cut_quant_mixed(pix: &Pix, ncolors: u32, ngray: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let max_ncolors = 20u32;
    let dark_thresh = 20u32;
    let light_thresh = 244u32;
    let diff_thresh = 15u32;

    let ncolors = ncolors.max(max_ncolors);
    let ngray = ngray.max(max_ncolors);

    // Estimate the number of significant colors using octcube level 3
    let estimated = number_occupied_octcubes(pix, 3, 1)?;
    if estimated > max_ncolors {
        return Err(ColorError::InvalidParameters(format!(
            "too many colors: {estimated} (max {max_ncolors})"
        )));
    }

    // Check if image has significant color content
    let w = pix.width();
    let h = pix.height();
    let total = (w * h) as u64;
    let mut color_count = 0u64;

    for y in 0..h {
        for x in 0..w {
            let pval = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pval);
            let minv = r.min(g).min(b);
            let maxv = r.max(g).max(b);
            if maxv as u32 >= dark_thresh
                && (minv as u32) <= light_thresh
                && (maxv - minv) as u32 >= diff_thresh
            {
                color_count += 1;
            }
        }
    }

    let is_color = color_count > total / 20; // > 5% color pixels

    if !is_color {
        // Convert to grayscale and quantize uniformly
        let out = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut out_mut = out.try_into_mut().unwrap();
        let mut colormap = PixColormap::new(8)?;
        for i in 0..ngray {
            let val = if ngray > 1 {
                (i * 255 / (ngray - 1)) as u8
            } else {
                128
            };
            colormap.add_rgb(val, val, val)?;
        }
        out_mut.set_colormap(Some(colormap))?;

        for y in 0..h {
            for x in 0..w {
                let pval = pix.get_pixel_unchecked(x, y);
                let (r, g, b) = pixel::extract_rgb(pval);
                let gray = ((r as u32 + g as u32 + b as u32) / 3) as u8;
                let idx = if ngray > 1 {
                    let scaled = gray as f32 / 255.0 * (ngray - 1) as f32;
                    scaled.round() as u32
                } else {
                    0
                };
                out_mut.set_pixel_unchecked(x, y, idx.min(ngray - 1));
            }
        }

        return Ok(out_mut.into());
    }

    // Use mixed gray/color quantizer
    median_cut_quant_mixed(pix, ncolors, ngray, dark_thresh, light_thresh, diff_thresh)
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
                let pixel = pixel::compose_rgb(r, g, b);
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
                    pixel::compose_rgb(255, 0, 0) // Red
                } else if x < 20 {
                    pixel::compose_rgb(0, 255, 0) // Green
                } else {
                    pixel::compose_rgb(0, 0, 255) // Blue
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
