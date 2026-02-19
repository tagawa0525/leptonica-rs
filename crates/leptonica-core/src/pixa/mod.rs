//! Pixa, Pixaa - Arrays of Pix images
//!
//! These structures manage collections of images, optionally with
//! associated bounding boxes for each image.

use crate::box_::{Box, Boxa, SizeRelation};
use crate::error::{Error, Result};
use crate::numa::{Numa, SortOrder};
use crate::pix::{Pix, PixMut, PixelDepth, statistics::RowColStatType};

/// Sort key for Pixa sorting operations.
///
/// Determines which property of the bounding box or image is used
/// as the sort key.
///
/// # See also
///
/// C Leptonica: `L_SORT_BY_*` constants in `pix.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixaSortType {
    /// Sort by left edge (box.x)
    ByX,
    /// Sort by top edge (box.y)
    ByY,
    /// Sort by right edge (box.x + box.w)
    ByRight,
    /// Sort by bottom edge (box.y + box.h)
    ByBottom,
    /// Sort by width
    ByWidth,
    /// Sort by height
    ByHeight,
    /// Sort by min(width, height)
    ByMinDimension,
    /// Sort by max(width, height)
    ByMaxDimension,
    /// Sort by perimeter (2*(w+h))
    ByPerimeter,
    /// Sort by area (w*h)
    ByArea,
    /// Sort by aspect ratio (w/h as f64)
    ByAspectRatio,
}

/// Array of Pix images
///
/// `Pixa` manages a collection of `Pix` images along with optional
/// bounding boxes for each image. This is useful for storing
/// segmented regions, connected components, or any collection of
/// related images.
///
/// # Examples
///
/// ```
/// use leptonica_core::{Pixa, Pix, PixelDepth};
///
/// let mut pixa = Pixa::new();
/// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
/// pixa.push(pix);
/// assert_eq!(pixa.len(), 1);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Pixa {
    /// The Pix images
    pix: Vec<Pix>,
    /// Bounding boxes for each Pix (may have fewer entries than pix)
    boxa: Boxa,
}

impl Pixa {
    /// Create a new empty Pixa
    pub fn new() -> Self {
        Self {
            pix: Vec::new(),
            boxa: Boxa::new(),
        }
    }

    /// Create a Pixa with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pix: Vec::with_capacity(capacity),
            boxa: Boxa::with_capacity(capacity),
        }
    }

    /// Get the number of Pix images
    #[inline]
    pub fn len(&self) -> usize {
        self.pix.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pix.is_empty()
    }

    /// Get a reference to a Pix by index
    ///
    /// Returns a reference to the Pix without cloning.
    pub fn get(&self, index: usize) -> Option<&Pix> {
        self.pix.get(index)
    }

    /// Get a mutable reference to a Pix by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Pix> {
        self.pix.get_mut(index)
    }

    /// Get a cloned Pix by index
    ///
    /// This returns a clone of the Pix at the given index.
    /// Since `Pix` uses `Arc` internally, this is a cheap reference
    /// count increment (shallow copy).
    pub fn get_cloned(&self, index: usize) -> Option<Pix> {
        self.pix.get(index).cloned()
    }

    /// Get the dimensions of a Pix by index
    ///
    /// Returns (width, height, depth) or None if index is out of bounds.
    pub fn get_dimensions(&self, index: usize) -> Option<(u32, u32, PixelDepth)> {
        self.pix
            .get(index)
            .map(|p| (p.width(), p.height(), p.depth()))
    }

    /// Add a Pix to the array
    pub fn push(&mut self, pix: Pix) {
        self.pix.push(pix);
    }

    /// Add a Pix with an associated bounding box
    ///
    /// The box is added to the internal Boxa at the same index.
    pub fn push_with_box(&mut self, pix: Pix, b: Box) {
        self.pix.push(pix);
        self.boxa.push(b);
    }

    /// Remove and return the last Pix
    pub fn pop(&mut self) -> Option<Pix> {
        self.pix.pop()
    }

    /// Remove a Pix at index
    pub fn remove(&mut self, index: usize) -> Result<Pix> {
        if index >= self.pix.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.pix.len(),
            });
        }
        // Also remove the box if it exists at this index
        if index < self.boxa.len() {
            let _ = self.boxa.remove(index);
        }
        Ok(self.pix.remove(index))
    }

    /// Insert a Pix at index
    pub fn insert(&mut self, index: usize, pix: Pix) -> Result<()> {
        if index > self.pix.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.pix.len(),
            });
        }
        self.pix.insert(index, pix);
        Ok(())
    }

    /// Replace a Pix at index
    pub fn replace(&mut self, index: usize, pix: Pix) -> Result<Pix> {
        if index >= self.pix.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.pix.len(),
            });
        }
        Ok(std::mem::replace(&mut self.pix[index], pix))
    }

    /// Clear all Pix images and boxes
    pub fn clear(&mut self) {
        self.pix.clear();
        self.boxa.clear();
    }

    /// Extend the array to accommodate at least `size` elements
    ///
    /// Corresponds to C `pixaExtendArrayToSize()`.
    /// In the Rust implementation, this reserves capacity but does not
    /// change the number of stored elements. Use [`init_full`](Pixa::init_full)
    /// after this to fill the allocated slots.
    pub fn extend_to_size(&mut self, size: usize) {
        if size > self.pix.capacity() {
            self.pix.reserve(size - self.pix.len());
        }
    }

    /// Initialize all slots with copies of the given Pix and optional Box
    ///
    /// Corresponds to C `pixaInitFull()`.
    /// This fills the pixa so that it contains exactly `count` elements,
    /// each being a clone of `pix`. If `pix` is `None`, a tiny 1x1x1
    /// placeholder Pix is used. Any existing elements are replaced.
    ///
    /// If a Box is provided, the boxa is also filled with copies.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of elements to fill
    /// * `pix` - Optional Pix to clone into each slot; if None, uses 1x1 Bit1
    /// * `b` - Optional Box to clone into each boxa slot
    pub fn init_full(&mut self, count: usize, pix: Option<&Pix>, b: Option<&Box>) {
        let template = match pix {
            Some(p) => p.clone(),
            None => Pix::new(1, 1, PixelDepth::Bit1).unwrap(),
        };

        self.pix.clear();
        self.pix.reserve(count);
        for _ in 0..count {
            self.pix.push(template.clone());
        }

        if let Some(bx) = b {
            self.boxa.clear();
            for _ in 0..count {
                self.boxa.push(*bx);
            }
        }
    }

    /// Get all Pix as a slice
    pub fn pix_slice(&self) -> &[Pix] {
        &self.pix
    }

    /// Get a reference to the Boxa
    pub fn boxa(&self) -> &Boxa {
        &self.boxa
    }

    /// Get a mutable reference to the Boxa
    pub fn boxa_mut(&mut self) -> &mut Boxa {
        &mut self.boxa
    }

    /// Get the number of boxes
    pub fn boxa_count(&self) -> usize {
        self.boxa.len()
    }

    /// Get a box by index
    pub fn get_box(&self, index: usize) -> Option<&Box> {
        self.boxa.get(index)
    }

    /// Set the Boxa, replacing any existing boxes
    pub fn set_boxa(&mut self, boxa: Boxa) {
        self.boxa = boxa;
    }

    /// Add a box for an existing Pix
    ///
    /// The box is added to the internal Boxa.
    pub fn add_box(&mut self, b: Box) {
        self.boxa.push(b);
    }

    /// Verify that all Pix have the same depth
    ///
    /// Returns `Ok((true, depth))` if all have the same depth,
    /// `Ok((false, max_depth))` if depths vary.
    /// Returns an error if the Pixa is empty.
    pub fn verify_depth(&self) -> Result<(bool, PixelDepth)> {
        if self.pix.is_empty() {
            return Err(Error::NullInput("pixa is empty"));
        }

        let first_depth = self.pix[0].depth();
        let mut max_depth = first_depth;
        let mut same = true;

        for pix in &self.pix[1..] {
            let d = pix.depth();
            if d != first_depth {
                same = false;
            }
            if d.bits() > max_depth.bits() {
                max_depth = d;
            }
        }

        Ok((same, max_depth))
    }

    /// Verify that all Pix have the same dimensions
    ///
    /// Returns `Ok(true)` if all have the same width and height.
    pub fn verify_dimensions(&self) -> Result<bool> {
        if self.pix.is_empty() {
            return Err(Error::NullInput("pixa is empty"));
        }

        let first_w = self.pix[0].width();
        let first_h = self.pix[0].height();

        for pix in &self.pix[1..] {
            if pix.width() != first_w || pix.height() != first_h {
                return Ok(false);
            }
        }

        Ok(true)
    }

    // ========================================================================
    // Selection functions
    // ========================================================================

    /// Select Pix images by width and height threshold.
    ///
    /// Returns a new Pixa containing only images whose dimensions satisfy
    /// the given relation against the threshold values. Both width AND height
    /// must satisfy the relation.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaSelectBySize()` in `pixafunc1.c`
    pub fn select_by_size(&self, width: i32, height: i32, relation: SizeRelation) -> Pixa {
        let mut result = Pixa::new();
        for (i, pix) in self.pix.iter().enumerate() {
            let pw = pix.width() as i32;
            let ph = pix.height() as i32;
            if compare_relation(pw, width, relation) && compare_relation(ph, height, relation) {
                result.pix.push(pix.clone());
                if let Some(b) = self.boxa.get(i) {
                    result.boxa.push(*b);
                }
            }
        }
        result
    }

    /// Select Pix images by area threshold.
    ///
    /// Returns a new Pixa containing only images whose area (width * height)
    /// satisfies the given relation against the threshold.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaSelectByArea()` (subset of `pixaSelectBySize`)
    pub fn select_by_area(&self, area: i64, relation: SizeRelation) -> Pixa {
        let mut result = Pixa::new();
        for (i, pix) in self.pix.iter().enumerate() {
            let pix_area = pix.width() as i64 * pix.height() as i64;
            if compare_relation_i64(pix_area, area, relation) {
                result.pix.push(pix.clone());
                if let Some(b) = self.boxa.get(i) {
                    result.boxa.push(*b);
                }
            }
        }
        result
    }

    // ========================================================================
    // Sort functions
    // ========================================================================

    /// Sort Pixa by a specified key, returning a new sorted Pixa.
    ///
    /// Sorts by bounding box properties (x, y, width, height, area, etc.)
    /// or by image dimensions when no boxes are present.
    /// Returns the sorted Pixa and the permutation index array.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaSort()` in `pixafunc1.c`
    pub fn sort(&self, sort_type: PixaSortType, order: SortOrder) -> (Pixa, Vec<usize>) {
        let n = self.pix.len();
        if n == 0 {
            return (Pixa::new(), Vec::new());
        }

        // Extract sort keys
        let keys: Vec<f64> = (0..n)
            .map(|i| {
                let (w, h) = if let Some(b) = self.boxa.get(i) {
                    (b.w as f64, b.h as f64)
                } else {
                    (self.pix[i].width() as f64, self.pix[i].height() as f64)
                };
                let x = self.boxa.get(i).map_or(0.0, |b| b.x as f64);
                let y = self.boxa.get(i).map_or(0.0, |b| b.y as f64);
                match sort_type {
                    PixaSortType::ByX => x,
                    PixaSortType::ByY => y,
                    PixaSortType::ByRight => x + w,
                    PixaSortType::ByBottom => y + h,
                    PixaSortType::ByWidth => w,
                    PixaSortType::ByHeight => h,
                    PixaSortType::ByMinDimension => w.min(h),
                    PixaSortType::ByMaxDimension => w.max(h),
                    PixaSortType::ByPerimeter => 2.0 * (w + h),
                    PixaSortType::ByArea => w * h,
                    PixaSortType::ByAspectRatio => {
                        if h == 0.0 {
                            0.0
                        } else {
                            w / h
                        }
                    }
                }
            })
            .collect();

        // Create index array and sort it
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|&a, &b| {
            let cmp = keys[a]
                .partial_cmp(&keys[b])
                .unwrap_or(std::cmp::Ordering::Equal);
            match order {
                SortOrder::Increasing => cmp,
                SortOrder::Decreasing => cmp.reverse(),
            }
        });

        // Build sorted Pixa
        let sorted = self.sort_by_index(&indices).unwrap_or_default();
        (sorted, indices)
    }

    /// Reorder Pixa by a permutation index array.
    ///
    /// Returns a new Pixa with elements reordered according to the index array.
    /// `indices[i]` gives the index in `self` of the element that should appear
    /// at position `i` in the result.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaSortByIndex()` in `pixafunc1.c`
    pub fn sort_by_index(&self, indices: &[usize]) -> Result<Pixa> {
        let n = self.pix.len();
        for &idx in indices {
            if idx >= n {
                return Err(Error::IndexOutOfBounds { index: idx, len: n });
            }
        }
        let mut result = Pixa::with_capacity(indices.len());
        for &idx in indices {
            result.pix.push(self.pix[idx].clone());
            if let Some(b) = self.boxa.get(idx) {
                result.boxa.push(*b);
            }
        }
        Ok(result)
    }

    // ========================================================================
    // Pixel counting functions
    // ========================================================================

    /// Count ON pixels in each 1 bpp Pix.
    ///
    /// Returns a Numa with one entry per Pix, where each entry is the
    /// count of ON pixels in that image.
    ///
    /// C equivalent: `pixaCountPixels()` in `pix3.c`
    pub fn count_pixels(&self) -> Result<Numa> {
        let mut counts = Numa::with_capacity(self.pix.len());
        for pix in &self.pix {
            if pix.depth() != PixelDepth::Bit1 {
                return Err(Error::UnsupportedDepth(pix.depth().bits()));
            }
            counts.push(pix.count_pixels() as f32);
        }
        Ok(counts)
    }

    /// Extract one column from each 8bpp Pix and write them as rows of `dst`.
    ///
    /// `dst` must be 8bpp with width == `self.len()` and height == the height
    /// of each constituent Pix. All Pix in the Pixa must be 8bpp and identical
    /// in size.
    ///
    /// C equivalent: `pixaExtractColumnFromEachPix()` in `pix4.c`
    pub fn extract_column_from_each(&self, col: u32, dst: &mut PixMut) -> Result<()> {
        todo!("not yet implemented")
    }

    /// Compute pixel-wise statistics over identically-sized 8bpp images.
    ///
    /// Each pixel in the returned `Pix` represents the chosen statistic
    /// (mean, median, mode, or mode count) across the corresponding pixels
    /// in every image in the Pixa.
    ///
    /// All images in the Pixa must be 8bpp and the same size.
    ///
    /// C equivalent: `pixaGetAlignedStats()` in `pix4.c`
    pub fn aligned_stats(&self, stat_type: RowColStatType, nbins: u32, thresh: u32) -> Result<Pix> {
        todo!("not yet implemented")
    }

    // ========================================================================
    // Display / composition functions
    // ========================================================================

    /// Compose all Pix images onto a single canvas.
    ///
    /// Each image is placed at its associated bounding box position.
    /// Images without boxes are placed at (0, 0). If `w` or `h` is 0,
    /// the canvas size is computed from the extent of all boxes and images.
    /// Negative box coordinates are handled: portions outside the canvas
    /// are clipped.
    ///
    /// The canvas depth is taken from the first image. All images should
    /// have the same depth for correct rendering.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaDisplay()` in `pixafunc1.c`
    pub fn display(&self, w: u32, h: u32) -> Result<Pix> {
        if self.pix.is_empty() {
            return Err(Error::NullInput("pixa is empty"));
        }

        // Determine depth from first image
        let depth = self.pix[0].depth();

        // Auto-compute canvas size if w or h is 0.
        // Track min offsets to handle negative box coordinates.
        let (canvas_w, canvas_h) = if w == 0 || h == 0 {
            let mut min_x: i32 = 0;
            let mut min_y: i32 = 0;
            let mut max_right: i32 = 0;
            let mut max_bottom: i32 = 0;
            for (i, pix) in self.pix.iter().enumerate() {
                let (bx, by) = if let Some(b) = self.boxa.get(i) {
                    (b.x, b.y)
                } else {
                    (0, 0)
                };
                min_x = min_x.min(bx);
                min_y = min_y.min(by);
                let right = bx + pix.width() as i32;
                let bottom = by + pix.height() as i32;
                max_right = max_right.max(right);
                max_bottom = max_bottom.max(bottom);
            }
            let computed_w = (max_right - min_x).max(1) as u32;
            let computed_h = (max_bottom - min_y).max(1) as u32;
            (
                if w == 0 { computed_w } else { w },
                if h == 0 { computed_h } else { h },
            )
        } else {
            (w, h)
        };

        let canvas = Pix::new(canvas_w, canvas_h, depth)?;
        let mut canvas_mut = canvas.try_into_mut().unwrap_or_else(|p: Pix| p.to_mut());

        // Paint each image onto the canvas
        for (i, src) in self.pix.iter().enumerate() {
            let (ox, oy) = if let Some(b) = self.boxa.get(i) {
                (b.x, b.y)
            } else {
                (0, 0)
            };
            blit_pix(&mut canvas_mut, src, ox, oy);
        }

        Ok(canvas_mut.into())
    }

    /// Arrange all Pix images in a tiled layout.
    ///
    /// Images are placed left-to-right, wrapping to the next row when
    /// `max_width` is exceeded. If a single image is wider than `max_width`,
    /// it is placed on its own row. Returns the composited image.
    ///
    /// The canvas depth is taken from the first image. All images should
    /// have the same depth for correct rendering.
    ///
    /// # Arguments
    ///
    /// * `max_width` - Maximum width before wrapping to next row
    /// * `background` - Background pixel value (0 for black, 255 for white)
    /// * `spacing` - Pixels of spacing between images
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaDisplayTiled()` in `pixafunc1.c`
    pub fn display_tiled(&self, max_width: u32, background: u32, spacing: u32) -> Result<Pix> {
        if self.pix.is_empty() {
            return Err(Error::NullInput("pixa is empty"));
        }

        let depth = self.pix[0].depth();

        // Compute layout: rows of images
        let mut rows: Vec<Vec<usize>> = Vec::new();
        let mut current_row: Vec<usize> = Vec::new();
        let mut row_x: u32 = 0;

        for i in 0..self.pix.len() {
            let pw = self.pix[i].width();
            let next_x = if current_row.is_empty() {
                pw
            } else {
                row_x + spacing + pw
            };
            if !current_row.is_empty() && next_x > max_width {
                rows.push(std::mem::take(&mut current_row));
                row_x = pw;
                current_row.push(i);
            } else {
                row_x = next_x;
                current_row.push(i);
            }
        }
        if !current_row.is_empty() {
            rows.push(current_row);
        }

        // Compute total dimensions
        let mut total_width: u32 = 0;
        let mut total_height: u32 = 0;
        let mut row_heights: Vec<u32> = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut rw: u32 = 0;
            let mut rh: u32 = 0;
            for (j, &idx) in row.iter().enumerate() {
                if j > 0 {
                    rw += spacing;
                }
                rw += self.pix[idx].width();
                rh = rh.max(self.pix[idx].height());
            }
            total_width = total_width.max(rw);
            row_heights.push(rh);
            total_height += rh;
        }
        // Add spacing between rows
        if rows.len() > 1 {
            total_height += spacing * (rows.len() as u32 - 1);
        }

        let canvas = Pix::new(total_width, total_height, depth)?;
        let mut canvas_mut = canvas.try_into_mut().unwrap_or_else(|p: Pix| p.to_mut());

        // Fill background (per-pixel; row-level fill could be more efficient
        // for large canvases but this is sufficient for typical use)
        if background != 0 {
            for y in 0..total_height {
                for x in 0..total_width {
                    canvas_mut.set_pixel_unchecked(x, y, background);
                }
            }
        }

        // Place images
        let mut cy: u32 = 0;
        for (row_idx, row) in rows.iter().enumerate() {
            let mut cx: u32 = 0;
            for (j, &idx) in row.iter().enumerate() {
                if j > 0 {
                    cx += spacing;
                }
                blit_pix(&mut canvas_mut, &self.pix[idx], cx as i32, cy as i32);
                cx += self.pix[idx].width();
            }
            cy += row_heights[row_idx];
            if row_idx < rows.len() - 1 {
                cy += spacing;
            }
        }

        Ok(canvas_mut.into())
    }

    /// Create a deep copy of this Pixa
    ///
    /// Unlike `clone()` which shares Pix data via Arc, this creates
    /// completely independent copies of all images.
    pub fn deep_clone(&self) -> Self {
        let pix = self.pix.iter().map(|p| p.deep_clone()).collect();
        Self {
            pix,
            boxa: self.boxa.clone(),
        }
    }

    /// Create an iterator over Pix references
    pub fn iter(&self) -> PixaIter<'_> {
        PixaIter {
            pixa: self,
            index: 0,
        }
    }

    /// Create a mutable iterator over Pix references
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pix> {
        self.pix.iter_mut()
    }
}

// ============================================================================
// Helper functions
// ============================================================================

use crate::box_::{compare_relation, compare_relation_i64};

/// Copy pixels from `src` onto `dst` at offset (ox, oy).
///
/// Uses per-pixel get/set; sufficient for small component images.
/// For bulk image operations, row-level memcpy would be more efficient.
///
/// Clips to destination bounds. Handles all pixel depths.
fn blit_pix(dst: &mut PixMut, src: &Pix, ox: i32, oy: i32) {
    let dw = dst.width() as i32;
    let dh = dst.height() as i32;
    let sw = src.width() as i32;
    let sh = src.height() as i32;

    // Compute clipped source region
    let src_x0 = if ox < 0 { -ox } else { 0 };
    let src_y0 = if oy < 0 { -oy } else { 0 };
    let src_x1 = sw.min(dw - ox);
    let src_y1 = sh.min(dh - oy);

    if src_x0 >= src_x1 || src_y0 >= src_y1 {
        return;
    }

    for sy in src_y0..src_y1 {
        let dy = oy + sy;
        for sx in src_x0..src_x1 {
            let dx = ox + sx;
            let val = src.get_pixel(sx as u32, sy as u32).unwrap_or(0);
            dst.set_pixel_unchecked(dx as u32, dy as u32, val);
        }
    }
}

/// Iterator over Pixa Pix references
pub struct PixaIter<'a> {
    pixa: &'a Pixa,
    index: usize,
}

impl<'a> Iterator for PixaIter<'a> {
    type Item = &'a Pix;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.pixa.len() {
            let pix = &self.pixa.pix[self.index];
            self.index += 1;
            Some(pix)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.pixa.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for PixaIter<'_> {}

impl<'a> IntoIterator for &'a Pixa {
    type Item = &'a Pix;
    type IntoIter = PixaIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Pixa {
    type Item = Pix;
    type IntoIter = std::vec::IntoIter<Pix>;

    fn into_iter(self) -> Self::IntoIter {
        self.pix.into_iter()
    }
}

impl FromIterator<Pix> for Pixa {
    fn from_iter<T: IntoIterator<Item = Pix>>(iter: T) -> Self {
        Self {
            pix: iter.into_iter().collect(),
            boxa: Boxa::new(),
        }
    }
}

impl std::ops::Index<usize> for Pixa {
    type Output = Pix;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pix[index]
    }
}

impl std::ops::IndexMut<usize> for Pixa {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.pix[index]
    }
}

// ============================================================================
// Pixaa - Array of Pixa
// ============================================================================

/// Array of Pixa
///
/// `Pixaa` manages a collection of `Pixa` arrays, useful for hierarchical
/// organization of images (e.g., pages containing regions).
#[derive(Debug, Clone, Default)]
pub struct Pixaa {
    pixas: Vec<Pixa>,
}

impl Pixaa {
    /// Create a new empty Pixaa
    pub fn new() -> Self {
        Self { pixas: Vec::new() }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pixas: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of Pixa
    #[inline]
    pub fn len(&self) -> usize {
        self.pixas.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pixas.is_empty()
    }

    /// Get a Pixa by index
    pub fn get(&self, index: usize) -> Option<&Pixa> {
        self.pixas.get(index)
    }

    /// Get a mutable Pixa by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Pixa> {
        self.pixas.get_mut(index)
    }

    /// Add a Pixa
    pub fn push(&mut self, pixa: Pixa) {
        self.pixas.push(pixa);
    }

    /// Remove and return the last Pixa
    pub fn pop(&mut self) -> Option<Pixa> {
        self.pixas.pop()
    }

    /// Replace a Pixa at index
    pub fn replace(&mut self, index: usize, pixa: Pixa) -> Result<Pixa> {
        if index >= self.pixas.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.pixas.len(),
            });
        }
        Ok(std::mem::replace(&mut self.pixas[index], pixa))
    }

    /// Clear all Pixa
    pub fn clear(&mut self) {
        self.pixas.clear();
    }

    /// Get all Pixa as a slice
    pub fn pixas(&self) -> &[Pixa] {
        &self.pixas
    }

    /// Get total number of Pix across all Pixa
    pub fn total_pix(&self) -> usize {
        self.pixas.iter().map(|p| p.len()).sum()
    }

    /// Flatten into a single Pixa
    pub fn flatten(&self) -> Pixa {
        let total = self.total_pix();
        let mut result = Pixa::with_capacity(total);
        for pixa in &self.pixas {
            for pix in pixa.iter() {
                result.push(pix.clone());
            }
            // Also copy boxes
            for b in pixa.boxa().iter() {
                result.add_box(*b);
            }
        }
        result
    }

    /// Get a specific Pix from a Pixa
    ///
    /// Convenience method for accessing `pixaa[pixa_index][pix_index]`.
    pub fn get_pix(&self, pixa_index: usize, pix_index: usize) -> Option<&Pix> {
        self.pixas.get(pixa_index)?.get(pix_index)
    }

    /// Create an iterator over Pixa
    pub fn iter(&self) -> impl Iterator<Item = &Pixa> {
        self.pixas.iter()
    }

    /// Create a mutable iterator over Pixa
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pixa> {
        self.pixas.iter_mut()
    }
}

impl std::ops::Index<usize> for Pixaa {
    type Output = Pixa;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pixas[index]
    }
}

impl std::ops::IndexMut<usize> for Pixaa {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.pixas[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_pix(width: u32, height: u32) -> Pix {
        Pix::new(width, height, PixelDepth::Bit8).unwrap()
    }

    #[test]
    fn test_pixa_creation() {
        let pixa = Pixa::new();
        assert!(pixa.is_empty());
        assert_eq!(pixa.len(), 0);

        let pixa = Pixa::with_capacity(10);
        assert!(pixa.is_empty());
    }

    #[test]
    fn test_pixa_push_and_get() {
        let mut pixa = Pixa::new();
        let pix = make_test_pix(100, 200);

        pixa.push(pix);
        assert_eq!(pixa.len(), 1);

        let retrieved = pixa.get(0).unwrap();
        assert_eq!(retrieved.width(), 100);
        assert_eq!(retrieved.height(), 200);

        assert!(pixa.get(1).is_none());
    }

    #[test]
    fn test_pixa_push_with_box() {
        let mut pixa = Pixa::new();
        let pix = make_test_pix(100, 100);
        let b = Box::new(10, 20, 30, 40).unwrap();

        pixa.push_with_box(pix, b);

        assert_eq!(pixa.len(), 1);
        assert_eq!(pixa.boxa_count(), 1);

        let retrieved_box = pixa.get_box(0).unwrap();
        assert_eq!(retrieved_box.x, 10);
        assert_eq!(retrieved_box.y, 20);
    }

    #[test]
    fn test_pixa_get_cloned() {
        let mut pixa = Pixa::new();
        let pix = make_test_pix(100, 100);
        pixa.push(pix);

        let cloned = pixa.get_cloned(0).unwrap();
        assert_eq!(cloned.width(), 100);

        // Original should still be accessible
        assert_eq!(pixa.get(0).unwrap().width(), 100);
    }

    #[test]
    fn test_pixa_get_dimensions() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 200));
        pixa.push(Pix::new(50, 50, PixelDepth::Bit1).unwrap());

        let (w, h, d) = pixa.get_dimensions(0).unwrap();
        assert_eq!(w, 100);
        assert_eq!(h, 200);
        assert_eq!(d, PixelDepth::Bit8);

        let (w, h, d) = pixa.get_dimensions(1).unwrap();
        assert_eq!(w, 50);
        assert_eq!(h, 50);
        assert_eq!(d, PixelDepth::Bit1);

        assert!(pixa.get_dimensions(2).is_none());
    }

    #[test]
    fn test_pixa_remove() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 100));
        pixa.push(make_test_pix(200, 200));
        pixa.push(make_test_pix(300, 300));

        let removed = pixa.remove(1).unwrap();
        assert_eq!(removed.width(), 200);
        assert_eq!(pixa.len(), 2);
        assert_eq!(pixa.get(1).unwrap().width(), 300);

        assert!(pixa.remove(10).is_err());
    }

    #[test]
    fn test_pixa_insert() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 100));
        pixa.push(make_test_pix(300, 300));

        pixa.insert(1, make_test_pix(200, 200)).unwrap();

        assert_eq!(pixa.len(), 3);
        assert_eq!(pixa.get(1).unwrap().width(), 200);
        assert_eq!(pixa.get(2).unwrap().width(), 300);

        assert!(pixa.insert(10, make_test_pix(1, 1)).is_err());
    }

    #[test]
    fn test_pixa_replace() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 100));

        let old = pixa.replace(0, make_test_pix(200, 200)).unwrap();
        assert_eq!(old.width(), 100);
        assert_eq!(pixa.get(0).unwrap().width(), 200);

        assert!(pixa.replace(10, make_test_pix(1, 1)).is_err());
    }

    #[test]
    fn test_pixa_pop() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 100));
        pixa.push(make_test_pix(200, 200));

        let popped = pixa.pop().unwrap();
        assert_eq!(popped.width(), 200);
        assert_eq!(pixa.len(), 1);

        pixa.pop();
        assert!(pixa.pop().is_none());
    }

    #[test]
    fn test_pixa_clear() {
        let mut pixa = Pixa::new();
        pixa.push_with_box(make_test_pix(100, 100), Box::new_unchecked(0, 0, 10, 10));
        pixa.push_with_box(make_test_pix(200, 200), Box::new_unchecked(0, 0, 20, 20));

        pixa.clear();
        assert!(pixa.is_empty());
        assert_eq!(pixa.boxa_count(), 0);
    }

    #[test]
    fn test_pixa_verify_depth() {
        let mut pixa = Pixa::new();

        // Empty pixa should error
        assert!(pixa.verify_depth().is_err());

        // Same depth
        pixa.push(make_test_pix(100, 100));
        pixa.push(make_test_pix(200, 200));
        let (same, depth) = pixa.verify_depth().unwrap();
        assert!(same);
        assert_eq!(depth, PixelDepth::Bit8);

        // Different depths
        pixa.push(Pix::new(50, 50, PixelDepth::Bit32).unwrap());
        let (same, max_depth) = pixa.verify_depth().unwrap();
        assert!(!same);
        assert_eq!(max_depth, PixelDepth::Bit32);
    }

    #[test]
    fn test_pixa_verify_dimensions() {
        let mut pixa = Pixa::new();

        // Empty pixa should error
        assert!(pixa.verify_dimensions().is_err());

        // Same dimensions
        pixa.push(make_test_pix(100, 100));
        pixa.push(make_test_pix(100, 100));
        assert!(pixa.verify_dimensions().unwrap());

        // Different dimensions
        pixa.push(make_test_pix(200, 200));
        assert!(!pixa.verify_dimensions().unwrap());
    }

    #[test]
    fn test_pixa_iterator() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 100));
        pixa.push(make_test_pix(200, 200));
        pixa.push(make_test_pix(300, 300));

        let widths: Vec<_> = pixa.iter().map(|p| p.width()).collect();
        assert_eq!(widths, vec![100, 200, 300]);

        // Test for loop
        let mut count = 0;
        for pix in &pixa {
            assert!(pix.width() > 0);
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_pixa_into_iterator() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 100));
        pixa.push(make_test_pix(200, 200));

        let collected: Vec<_> = pixa.into_iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].width(), 100);
    }

    #[test]
    fn test_pixa_from_iterator() {
        let pix_list = vec![make_test_pix(100, 100), make_test_pix(200, 200)];

        let pixa: Pixa = pix_list.into_iter().collect();
        assert_eq!(pixa.len(), 2);
    }

    #[test]
    fn test_pixa_indexing() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 100));
        pixa.push(make_test_pix(200, 200));

        assert_eq!(pixa[0].width(), 100);
        assert_eq!(pixa[1].width(), 200);
    }

    #[test]
    fn test_pixa_clone_vs_deep_clone() {
        let mut pixa = Pixa::new();
        let pix = make_test_pix(100, 100);
        pixa.push(pix);

        // Regular clone shares data via Arc
        let cloned = pixa.clone();
        assert_eq!(pixa[0].data().as_ptr(), cloned[0].data().as_ptr());

        // Deep clone creates independent copies
        let deep = pixa.deep_clone();
        assert_ne!(pixa[0].data().as_ptr(), deep[0].data().as_ptr());
    }

    // ========================================================================
    // Pixaa tests
    // ========================================================================

    #[test]
    fn test_pixaa_creation() {
        let pixaa = Pixaa::new();
        assert!(pixaa.is_empty());
        assert_eq!(pixaa.len(), 0);
    }

    #[test]
    fn test_pixaa_push_and_get() {
        let mut pixaa = Pixaa::new();

        let mut pixa1 = Pixa::new();
        pixa1.push(make_test_pix(100, 100));
        pixa1.push(make_test_pix(200, 200));
        pixaa.push(pixa1);

        let mut pixa2 = Pixa::new();
        pixa2.push(make_test_pix(300, 300));
        pixaa.push(pixa2);

        assert_eq!(pixaa.len(), 2);
        assert_eq!(pixaa.get(0).unwrap().len(), 2);
        assert_eq!(pixaa.get(1).unwrap().len(), 1);
    }

    #[test]
    fn test_pixaa_total_pix() {
        let mut pixaa = Pixaa::new();

        let mut pixa1 = Pixa::new();
        pixa1.push(make_test_pix(100, 100));
        pixa1.push(make_test_pix(200, 200));
        pixaa.push(pixa1);

        let mut pixa2 = Pixa::new();
        pixa2.push(make_test_pix(300, 300));
        pixaa.push(pixa2);

        assert_eq!(pixaa.total_pix(), 3);
    }

    #[test]
    fn test_pixaa_flatten() {
        let mut pixaa = Pixaa::new();

        let mut pixa1 = Pixa::new();
        pixa1.push(make_test_pix(100, 100));
        pixa1.push(make_test_pix(200, 200));
        pixaa.push(pixa1);

        let mut pixa2 = Pixa::new();
        pixa2.push(make_test_pix(300, 300));
        pixaa.push(pixa2);

        let flat = pixaa.flatten();
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].width(), 100);
        assert_eq!(flat[1].width(), 200);
        assert_eq!(flat[2].width(), 300);
    }

    #[test]
    fn test_pixaa_get_pix() {
        let mut pixaa = Pixaa::new();

        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 100));
        pixa.push(make_test_pix(200, 200));
        pixaa.push(pixa);

        let pix = pixaa.get_pix(0, 1).unwrap();
        assert_eq!(pix.width(), 200);

        assert!(pixaa.get_pix(0, 10).is_none());
        assert!(pixaa.get_pix(10, 0).is_none());
    }

    #[test]
    fn test_pixaa_indexing() {
        let mut pixaa = Pixaa::new();

        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(100, 100));
        pixaa.push(pixa);

        assert_eq!(pixaa[0].len(), 1);
        assert_eq!(pixaa[0][0].width(), 100);
    }

    #[test]
    fn test_pixaa_replace() {
        let mut pixaa = Pixaa::new();

        let mut pixa1 = Pixa::new();
        pixa1.push(make_test_pix(100, 100));
        pixaa.push(pixa1);

        let mut pixa2 = Pixa::new();
        pixa2.push(make_test_pix(200, 200));
        pixa2.push(make_test_pix(300, 300));

        let old = pixaa.replace(0, pixa2).unwrap();
        assert_eq!(old.len(), 1);
        assert_eq!(pixaa[0].len(), 2);

        assert!(pixaa.replace(10, Pixa::new()).is_err());
    }

    // -- Pixa::count_pixels --

    #[test]
    fn test_pixa_count_pixels() {
        use crate::pix::PixelDepth;

        let mut pixa = Pixa::new();

        // 1bpp image with 3 ON pixels
        let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm1 = pix1.to_mut();
        pm1.set_pixel_unchecked(0, 0, 1);
        pm1.set_pixel_unchecked(5, 5, 1);
        pm1.set_pixel_unchecked(9, 9, 1);
        pixa.push(pm1.into());

        // 1bpp image with 0 ON pixels
        pixa.push(Pix::new(10, 10, PixelDepth::Bit1).unwrap());

        // 1bpp image with 2 ON pixels
        let pix3 = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pm3 = pix3.to_mut();
        pm3.set_pixel_unchecked(0, 0, 1);
        pm3.set_pixel_unchecked(4, 4, 1);
        pixa.push(pm3.into());

        let counts = pixa.count_pixels().unwrap();
        assert_eq!(counts.len(), 3);
        assert_eq!(counts.get_i32(0), Some(3));
        assert_eq!(counts.get_i32(1), Some(0));
        assert_eq!(counts.get_i32(2), Some(2));
    }

    #[test]
    fn test_pixa_count_pixels_empty() {
        let pixa = Pixa::new();
        let counts = pixa.count_pixels().unwrap();
        assert_eq!(counts.len(), 0);
    }

    #[test]
    fn test_pixa_count_pixels_not_1bpp() {
        use crate::pix::PixelDepth;

        let mut pixa = Pixa::new();
        pixa.push(Pix::new(10, 10, PixelDepth::Bit8).unwrap());
        assert!(pixa.count_pixels().is_err());
    }

    // -- Pixa::extract_column_from_each --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_extract_column_from_each_basic() {
        use crate::pix::PixelDepth;
        // 3 images of size 2x3, each with a distinct value in column 0
        // Image 0: col0 = [10, 20, 30]
        // Image 1: col0 = [40, 50, 60]
        // Image 2: col0 = [70, 80, 90]
        let mut pixa = Pixa::new();
        for (i, &vals) in [(10u32, 20u32, 30u32), (40, 50, 60), (70, 80, 90)]
            .iter()
            .enumerate()
        {
            let _ = i;
            let base = Pix::new(2, 3, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            pm.set_pixel_unchecked(0, 0, vals.0);
            pm.set_pixel_unchecked(0, 1, vals.1);
            pm.set_pixel_unchecked(0, 2, vals.2);
            pixa.push(Pix::from(pm));
        }
        // dst: width=3 (= n images), height=3 (= image height)
        let dst_base = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
        let mut dst = dst_base.try_into_mut().unwrap();
        pixa.extract_column_from_each(0, &mut dst).unwrap();
        // Row 0 of dst: pixels from col0,row0 of each image = [10, 40, 70]
        assert_eq!(dst.get_pixel(0, 0), Some(10));
        assert_eq!(dst.get_pixel(1, 0), Some(40));
        assert_eq!(dst.get_pixel(2, 0), Some(70));
        // Row 1: [20, 50, 80]
        assert_eq!(dst.get_pixel(0, 1), Some(20));
        assert_eq!(dst.get_pixel(1, 1), Some(50));
        assert_eq!(dst.get_pixel(2, 1), Some(80));
    }

    // -- Pixa::aligned_stats --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_aligned_stats_mean() {
        use crate::pix::PixelDepth;
        use crate::pix::statistics::RowColStatType;
        // 3 identical 2x2 8bpp images, all pixels = 60
        let mut pixa = Pixa::new();
        for _ in 0..3 {
            let base = Pix::new(2, 2, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for y in 0..2 {
                for x in 0..2 {
                    pm.set_pixel_unchecked(x, y, 60);
                }
            }
            pixa.push(Pix::from(pm));
        }
        let result = pixa
            .aligned_stats(RowColStatType::MeanAbsVal, 0, 0)
            .unwrap();
        assert_eq!(result.width(), 2);
        assert_eq!(result.height(), 2);
        // All pixels should be 60 (mean of [60, 60, 60])
        for y in 0..2 {
            for x in 0..2 {
                let v = result.get_pixel(x, y).unwrap();
                assert!((v as i32 - 60).abs() <= 1, "pixel({x},{y})={v}");
            }
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_aligned_stats_empty_pixa() {
        use crate::pix::statistics::RowColStatType;
        let pixa = Pixa::new();
        assert!(
            pixa.aligned_stats(RowColStatType::MeanAbsVal, 0, 0)
                .is_err()
        );
    }
}
