//! Pixa, Pixaa - Arrays of Pix images
//!
//! These structures manage collections of images, optionally with
//! associated bounding boxes for each image.

mod properties;

pub use properties::SizeIndicatorAxis;
mod select;
mod serial;
mod transform;

pub use select::{
    ThresholdSelect, pix_add_with_indicator, pix_remove_with_indicator,
    pix_select_by_area_fraction, pix_select_by_perim_size_ratio, pix_select_by_perim_to_area_ratio,
    pix_select_by_width_height_ratio,
};

use crate::core::box_::{Box, Boxa, SizeRelation};
use crate::core::error::{Error, Result};
use crate::core::numa::{Numa, SortOrder};
use crate::core::pix::{Pix, PixMut, PixelDepth, statistics::RowColStatType};

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
/// use leptonica::core::{Pixa, Pix, PixelDepth};
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

    /// Remove the Pix (and its box) at index, discarding the value.
    ///
    /// C Leptonica equivalent: `pixaRemovePix`
    pub fn remove_pix(&mut self, index: usize) -> Result<()> {
        self.remove(index)?;
        Ok(())
    }

    /// Remove and return the Pix at index.
    ///
    /// C Leptonica equivalent: `pixaRemovePixAndSave`
    pub fn remove_pix_and_save(&mut self, index: usize) -> Result<Pix> {
        self.remove(index)
    }

    /// Read a Pixa from a PNG image file and a boxa file.
    ///
    /// Reads the image, reads the boxa, then clips the image by each box
    /// to produce the Pixa.
    ///
    /// C Leptonica equivalent: `pixaReadBoth`
    pub fn read_both(image_path: &std::path::Path, boxa_path: &std::path::Path) -> Result<Pixa> {
        let pix = crate::io::read_image(image_path)
            .map_err(|e| Error::InvalidParameter(format!("failed to read image: {}", e)))?;
        let boxa = Boxa::read_from_file(boxa_path)?;
        pix.clip_rectangles(&boxa)
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

    /// Create a Pixa with `n` clones of `pix`.
    ///
    /// C equivalent: `pixaCreateFromPix()` in `pixabasic.c`
    pub fn create_from_pix(pix: &Pix, n: usize) -> Self {
        let mut pixa = Self::with_capacity(n);
        for _ in 0..n {
            pixa.push(pix.clone());
        }
        pixa
    }

    /// Create a Pixa by clipping `pix` at each box in `boxa`.
    ///
    /// Each resulting Pix is the sub-image defined by the corresponding
    /// bounding box, clamped to the image boundary.
    ///
    /// Boxes that are entirely outside the image or result in zero area
    /// after clamping are **skipped**. The returned Pixa may therefore
    /// have fewer elements than `boxa`.
    ///
    /// C equivalent: `pixaCreateFromBoxa()` in `pixabasic.c`
    pub fn create_from_boxa(pix: &Pix, boxa: &Boxa) -> Self {
        let n = boxa.len();
        let pw = pix.width() as i32;
        let ph = pix.height() as i32;
        let mut pixa = Self::with_capacity(n);
        for i in 0..n {
            if let Some(b) = boxa.get(i) {
                let x = b.x.max(0).min(pw);
                let y = b.y.max(0).min(ph);
                let w = (b.w.min(pw - x)).max(0) as u32;
                let h = (b.h.min(ph - y)).max(0) as u32;
                if w > 0 && h > 0 {
                    let clipped = pix
                        .clip_rectangle(x as u32, y as u32, w, h)
                        .unwrap_or_else(|_| pix.clone());
                    let cb = Box::new(x, y, w as i32, h as i32).unwrap();
                    pixa.push_with_box(clipped, cb);
                }
            }
        }
        pixa
    }

    /// Split `pix` into `nx * ny` tiles, each with optional `delta`-pixel overlap.
    ///
    /// `orig` is added to the origin of each tile box.
    ///
    /// C equivalent: `pixaSplitPix()` in `pixabasic.c`
    pub fn split_pix(pix: &Pix, nx: usize, ny: usize, delta: i32, orig: i32) -> Result<Self> {
        if nx == 0 || ny == 0 {
            return Err(Error::InvalidParameter("nx and ny must be > 0".to_string()));
        }
        let pw = pix.width() as i32;
        let ph = pix.height() as i32;
        let tile_w = (pw + nx as i32 - 1) / nx as i32;
        let tile_h = (ph + ny as i32 - 1) / ny as i32;
        let mut pixa = Self::with_capacity(nx * ny);
        for row in 0..ny {
            for col in 0..nx {
                let x = (col as i32 * tile_w - delta).max(0);
                let y = (row as i32 * tile_h - delta).max(0);
                let x2 = ((col as i32 + 1) * tile_w + delta).min(pw);
                let y2 = ((row as i32 + 1) * tile_h + delta).min(ph);
                let w = (x2 - x).max(0) as u32;
                let h = (y2 - y).max(0) as u32;
                let bx_x = x + orig;
                let bx_y = y + orig;
                if w > 0 && h > 0 {
                    let tile = pix
                        .clip_rectangle(x as u32, y as u32, w, h)
                        .unwrap_or_else(|_| pix.clone());
                    let cb = Box::new(bx_x, bx_y, w as i32, h as i32).unwrap();
                    pixa.push_with_box(tile, cb);
                } else {
                    // Placeholder so that pixa[i] always has a corresponding box
                    let placeholder = Pix::new(1, 1, pix.depth()).unwrap();
                    let cb = Box::new(bx_x, bx_y, 1, 1).unwrap();
                    pixa.push_with_box(placeholder, cb);
                }
            }
        }
        Ok(pixa)
    }

    /// Return `(x, y, w, h)` for the box at `index`, or `None` if out of bounds.
    ///
    /// C equivalent: `pixaGetBoxGeometry()` in `pixabasic.c`
    pub fn get_box_geometry(&self, index: usize) -> Option<(i32, i32, i32, i32)> {
        let b = self.boxa.get(index)?;
        Some((b.x, b.y, b.w, b.h))
    }

    /// Return `true` if all slots contain a non-empty Pix.
    ///
    /// An empty Pixa is considered "full" (vacuously true).
    ///
    /// C equivalent: `pixaIsFull()` in `pixabasic.c`
    pub fn is_full(&self) -> bool {
        self.pix.iter().all(|p| p.width() > 0 && p.height() > 0)
    }

    /// Set the text on every Pix in this Pixa.
    ///
    /// C equivalent: `pixaSetText()` in `pixabasic.c`
    pub fn set_text(&mut self, text: Option<String>) {
        // Take ownership of the entire Vec to avoid Arc-count inflation:
        // iterating &mut self.pix and calling clone() would increment the
        // Arc count, guaranteeing try_into_mut() always fails.
        let pix_vec = std::mem::take(&mut self.pix);
        self.pix = pix_vec
            .into_iter()
            .map(|p| {
                let mut pm = p.try_into_mut().unwrap_or_else(|p| p.to_mut());
                pm.set_text(text.clone());
                pm.into()
            })
            .collect();
    }

    /// Count the number of Pix that have a non-None text string.
    ///
    /// C equivalent: `pixaCountText()` in `pixabasic.c`
    pub fn count_text(&self) -> usize {
        self.pix.iter().filter(|p| p.text().is_some()).count()
    }

    /// Remove the Pix at the selected indices (given as f32 in the Numa).
    ///
    /// Indices are sorted in descending order internally before removal,
    /// so they may be provided in any order.
    ///
    /// C equivalent: `pixaRemoveSelected()` in `pixabasic.c`
    pub fn remove_selected(&mut self, na: &Numa) -> Result<()> {
        let n = na.len();
        let mut indices: Vec<usize> = (0..n)
            .map(|i| {
                na.get_i32(i)
                    .ok_or_else(|| Error::InvalidParameter("invalid index in na".to_string()))
                    .map(|v| v as usize)
            })
            .collect::<Result<Vec<_>>>()?;
        // Sort descending so earlier removals don't shift later indices
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            self.remove(idx)?;
        }
        Ok(())
    }

    /// Append all Pix (and boxes, if present) from `src[istart..=iend]` into `self`.
    ///
    /// `iend = None` means "to the end".
    ///
    /// C equivalent: `pixaJoin()` in `pixabasic.c`
    pub fn join(&mut self, src: &Pixa, istart: usize, iend: Option<usize>) -> Result<()> {
        let n = src.len();
        if n == 0 {
            return Ok(());
        }
        let iend = match iend {
            Some(e) if e < n => e,
            _ => n - 1,
        };
        if istart > iend {
            return Err(Error::InvalidParameter("istart > iend".to_string()));
        }
        for i in istart..=iend {
            let pix = src.get(i).unwrap().clone();
            if let Some(b) = src.get_box(i) {
                self.push_with_box(pix, *b);
            } else {
                self.push(pix);
            }
        }
        Ok(())
    }

    /// Return a new Pixa whose elements alternate between `self` and `other`.
    ///
    /// Both Pixa must have the same length.
    ///
    /// C equivalent: `pixaInterleave()` in `pixabasic.c`
    pub fn interleave(&self, other: &Pixa) -> Result<Pixa> {
        if self.len() != other.len() {
            return Err(Error::InvalidParameter(
                "pixas must have same length".to_string(),
            ));
        }
        let n = self.len();
        let mut ptad = Pixa::with_capacity(2 * n);
        for i in 0..n {
            let p1 = self.get(i).unwrap().clone();
            let p2 = other.get(i).unwrap().clone();
            if let Some(b1) = self.get_box(i) {
                ptad.push_with_box(p1, *b1);
            } else {
                ptad.push(p1);
            }
            if let Some(b2) = other.get_box(i) {
                ptad.push_with_box(p2, *b2);
            } else {
                ptad.push(p2);
            }
        }
        Ok(ptad)
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
        if dst.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(dst.depth().bits()));
        }
        let n = self.pix.len();
        if n == 0 {
            return Err(Error::InvalidParameter("pixa is empty".into()));
        }
        if dst.width() as usize != n {
            return Err(Error::InvalidParameter(
                "dst width must equal pixa length".into(),
            ));
        }
        let h = dst.height();
        for (k, pix) in self.pix.iter().enumerate() {
            if pix.depth() != PixelDepth::Bit8 {
                return Err(Error::UnsupportedDepth(pix.depth().bits()));
            }
            if col >= pix.width() {
                return Err(Error::IndexOutOfBounds {
                    index: col as usize,
                    len: pix.width() as usize,
                });
            }
            if pix.height() != h {
                return Err(Error::InvalidParameter(
                    "all pix heights must match dst height".into(),
                ));
            }
            for i in 0..h {
                let val = pix.get_pixel_unchecked(col, i);
                dst.set_pixel_unchecked(k as u32, i, val);
            }
        }
        Ok(())
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
        let n = self.pix.len();
        if n == 0 {
            return Err(Error::InvalidParameter("pixa is empty".into()));
        }
        let first = &self.pix[0];
        if first.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(first.depth().bits()));
        }
        let w = first.width();
        let h = first.height();

        for pix in &self.pix {
            if pix.depth() != PixelDepth::Bit8 {
                return Err(Error::UnsupportedDepth(pix.depth().bits()));
            }
            if pix.width() != w || pix.height() != h {
                return Err(Error::InvalidParameter(
                    "all pix must have identical dimensions".into(),
                ));
            }
        }

        let pixd_base = Pix::new(w, h, PixelDepth::Bit8)
            .map_err(|e| Error::InvalidParameter(format!("cannot create output pix: {e}")))?;
        let mut pixd = pixd_base.try_into_mut().unwrap();

        for j in 0..w {
            // Build n×h intermediate image: column j from each pix → one row each
            let pixt_base = Pix::new(n as u32, h, PixelDepth::Bit8)
                .map_err(|e| Error::InvalidParameter(format!("cannot create pixt: {e}")))?;
            let mut pixt_mut = pixt_base.try_into_mut().unwrap();
            self.extract_column_from_each(j, &mut pixt_mut)?;

            let pixt: Pix = pixt_mut.into();
            let col_stats = pixt.get_row_stats(stat_type, nbins, thresh)?;

            let values: Vec<f32> = (0..h as usize).filter_map(|i| col_stats.get(i)).collect();
            pixd.set_pixel_column(j, &values)?;
        }

        Ok(pixd.into())
    }

    /// Return Numa arrays of widths and heights for all images in the Pixa.
    ///
    /// Returns `(na_widths, na_heights)`. Returns an error if the Pixa is empty.
    ///
    /// C equivalent: `pixaFindDimensions()` in `pix5.c`
    pub fn find_dimensions(&self) -> Result<(crate::core::Numa, crate::core::Numa)> {
        if self.pix.is_empty() {
            return Err(Error::InvalidParameter("pixa is empty".into()));
        }
        let n = self.pix.len();
        let mut na_w = crate::core::Numa::with_capacity(n);
        let mut na_h = crate::core::Numa::with_capacity(n);
        for pix in &self.pix {
            na_w.push(pix.width() as f32);
            na_h.push(pix.height() as f32);
        }
        Ok((na_w, na_h))
    }

    // ========================================================================
    // Display / composition functions
    // ========================================================================

    /// Compose all Pix images onto a single canvas.
    ///
    /// Each image is placed at its associated bounding box position; an image
    /// with no box is skipped. If either `w` or `h` is 0, both come from the
    /// boxa extent ([`Boxa::get_extent`]), which does not compensate for
    /// negative origins — anything falling outside the canvas is clipped.
    ///
    /// An empty pixa yields an empty 1 bpp canvas of the requested size; both
    /// `w` and `h` must be non-zero for that, since there are no boxes to take
    /// an extent from.
    ///
    /// The canvas depth is taken from the first image. All images should
    /// have the same depth for correct rendering.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaDisplay()` in `pixafunc1.c`
    pub fn display(&self, w: u32, h: u32) -> Result<Pix> {
        // C: with no components and no size there is nothing to build, but a
        // given size still yields an empty 1 bpp canvas.
        if self.pix.is_empty() {
            if w == 0 || h == 0 {
                return Err(Error::NullInput(
                    "pixa is empty; both width and height are needed to size the canvas",
                ));
            }
            return Pix::new(w, h, PixelDepth::Bit1);
        }

        // Determine depth from first image
        let depth = self.pix[0].depth();

        // C: when either dimension is missing, both come from the boxa
        // extent (`boxaGetExtent`), which is the largest x+w / y+h over the
        // valid boxes. Negative origins are not compensated for; the blits
        // below clip instead.
        let (canvas_w, canvas_h) = if w == 0 || h == 0 {
            // C reads the extent unconditionally (an empty boxa yields zeros)
            // and fails when either dimension came out zero.
            let (ext_w, ext_h) = self
                .boxa
                .get_extent()
                .map(|(w, h, _)| (w, h))
                .unwrap_or((0, 0));
            if ext_w <= 0 || ext_h <= 0 {
                return Err(Error::NullInput(
                    "pixa boxa has no positive extent; pass an explicit canvas size",
                ));
            }
            (ext_w as u32, ext_h as u32)
        } else {
            (w, h)
        };

        let canvas = Pix::new(canvas_w, canvas_h, depth)?;
        let mut canvas_mut = canvas.try_into_mut().unwrap_or_else(|p: Pix| p.to_mut());

        // C pixaDisplay: canvases deeper than 1bpp start all-white
        // (pixSetAll), and 1bpp components are composited with PIX_PAINT
        // (OR) so overlapping bounding boxes do not erase earlier fg.
        if depth != PixelDepth::Bit1 {
            canvas_mut.set_all();
        }
        for (i, src) in self.pix.iter().enumerate() {
            // C warns and skips a component with no box rather than placing
            // it at the origin. There is no logging layer here, so this only
            // skips.
            let Some(b) = self.boxa.get(i) else {
                continue;
            };
            let (ox, oy) = (b.x, b.y);
            if depth == PixelDepth::Bit1 {
                blit_pix_or(&mut canvas_mut, src, ox, oy);
            } else {
                blit_pix(&mut canvas_mut, src, ox, oy);
            }
        }

        Ok(canvas_mut.into())
    }

    /// Arrange all Pix images on a regular lattice.
    ///
    /// The lattice cell size is taken from the maximum subimage width and
    /// height; the column count is chosen so the output width does not
    /// exceed `max_width` (but at least one column is used, so a single
    /// oversized image still renders). The full column count is reserved
    /// even when there are fewer images. Images with colormaps are
    /// converted to 32 bpp; all images must otherwise share one depth.
    ///
    /// # Arguments
    ///
    /// * `max_width` - Maximum width of the output image
    /// * `background` - 0 for white, 1 for black. Any other value skips
    ///   the background fill entirely (like C), which leaves the canvas
    ///   white for 1 bpp and black for deeper images — pass only 0 or 1
    /// * `spacing` - Pixels of spacing between lattice cells
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaDisplayTiled()` in `pixafunc2.c`
    pub fn display_tiled(&self, max_width: u32, background: u32, spacing: u32) -> Result<Pix> {
        let n = self.pix.len();
        if n == 0 {
            return Err(Error::NullInput("pixa is empty"));
        }

        // If any pix have colormaps, generate rgb.
        let converted: Vec<Pix> = if self.pix.iter().any(|p| p.has_colormap()) {
            self.pix
                .iter()
                .map(|p| p.convert_to_32())
                .collect::<Result<_>>()?
        } else {
            self.pix.clone()
        };

        let depth = converted[0].depth();
        if converted.iter().any(|p| p.depth() != depth) {
            return Err(Error::InvalidParameter(
                "display_tiled: depths not equal".into(),
            ));
        }

        // Lattice geometry from the max subimage dimensions.
        let wmax = converted.iter().map(|p| p.width()).max().unwrap_or(0);
        let hmax = converted.iter().map(|p| p.height()).max().unwrap_or(0);
        let ncols = (((max_width as f32 - spacing as f32) / (wmax + spacing) as f32) as u32).max(1);
        let nrows = (n as u32).div_ceil(ncols);
        let wd = wmax * ncols + spacing * (ncols + 1);
        let hd = hmax * nrows + spacing * (nrows + 1);

        let canvas = Pix::new(wd, hd, depth)?;
        let mut canvas_mut = canvas.try_into_mut().unwrap_or_else(|p: Pix| p.to_mut());

        // background: 0 = white, 1 = black. White for d > 1 and black for
        // d == 1 both mean pixSetAll.
        if (background == 1 && depth == PixelDepth::Bit1)
            || (background == 0 && depth != PixelDepth::Bit1)
        {
            canvas_mut.set_all();
        }

        // Blit the images to the dest.
        let mut res = 0;
        for (ni, pix) in converted.iter().enumerate() {
            let i = ni as u32 / ncols;
            let j = ni as u32 % ncols;
            if ni == 0 {
                res = pix.xres();
            }
            let xstart = spacing + j * (wmax + spacing);
            let ystart = spacing + i * (hmax + spacing);
            blit_pix(&mut canvas_mut, pix, xstart as i32, ystart as i32);
        }
        canvas_mut.set_resolution(res, res);

        Ok(canvas_mut.into())
    }

    /// Scale each Pix to a target size preserving aspect ratio.
    ///
    /// If `wd` is 0, scale to height `hd` (preserving aspect ratio).
    /// If `hd` is 0, scale to width `wd` (preserving aspect ratio).
    /// If both are non-zero, scale to exactly `wd` × `hd`.
    /// If both are 0, return a deep clone.
    ///
    /// C equivalent: `pixaScaleToSize()` in `pixafunc1.c`
    pub fn scale_to_size(&self, wd: u32, hd: u32) -> Pixa {
        let mut out = Pixa::with_capacity(self.len());
        for pix in &self.pix {
            let scaled = scale_pix_to_size(pix, wd, hd);
            out.push(scaled);
        }
        out
    }

    /// Scale each Pix by adding `delw` pixels to its width and `delh` to its
    /// height. If either resulting dimension is ≤ 0, return a copy of the
    /// original image.
    ///
    /// C equivalent: `pixaScaleToSizeRel()` in `pixafunc1.c`
    pub fn scale_to_size_rel(&self, delw: i32, delh: i32) -> Pixa {
        let mut out = Pixa::with_capacity(self.len());
        for pix in &self.pix {
            let w = pix.width() as i32 + delw;
            let h = pix.height() as i32 + delh;
            if w <= 0 || h <= 0 {
                out.push(pix.clone());
            } else {
                out.push(scale_pix_to_size(pix, w as u32, h as u32));
            }
        }
        out
    }
    /// Paint every 1 bpp component into an 8 bpp image with a random
    /// colormap, one colormap index per component.
    ///
    /// Component `i` is painted with index `1 + (i % 254)` at its stored
    /// bounding box, OR-ed into the destination. Index 0 stays black and
    /// index 255 white, as in C's `pixcmapCreateRandom(8, 1, 1)`.
    ///
    /// `w` and `h` give the canvas size; passing 0 for either uses the extent
    /// of the stored boxes. Every component must have a stored box (as it does
    /// when the pixa comes from a connected-component pass); a missing box is
    /// an error rather than a silently skipped blit.
    ///
    /// C equivalent: `pixaDisplayRandomCmap()` in `pixafunc2.c`
    pub fn display_random_cmap(&self, w: u32, h: u32) -> Result<Pix> {
        use crate::core::pix::{PixelDepth, RopOp};

        if self.pix.is_empty() {
            return Err(Error::NullInput("pixa is empty"));
        }
        if self.pix.iter().any(|p| p.depth() != PixelDepth::Bit1) {
            return Err(Error::InvalidParameter(
                "not all components are 1 bpp".to_string(),
            ));
        }

        let (w, h) = if w == 0 || h == 0 {
            let mut ext_w = 0i32;
            let mut ext_h = 0i32;
            for i in 0..self.len() {
                if let Some((x, y, bw, bh)) = self.get_box_geometry(i) {
                    ext_w = ext_w.max(x + bw);
                    ext_h = ext_h.max(y + bh);
                }
            }
            (ext_w.max(0) as u32, ext_h.max(0) as u32)
        } else {
            (w, h)
        };

        let pixd = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut dm = pixd.try_into_mut().unwrap();
        dm.set_colormap(Some(crate::core::PixColormap::create_random(
            8, true, true,
        )?))?;

        for (i, pixs) in self.pix.iter().enumerate() {
            let index = 1 + (i as u32 % 254);
            let (xb, yb, wb, hb) = self.get_box_geometry(i).ok_or_else(|| {
                Error::InvalidParameter(format!("pixa has no box for component {i}"))
            })?;
            let pix1 = pixs.convert_1_to_8(0, index)?;
            dm.rop_region_inplace(xb, yb, wb as u32, hb as u32, RopOp::Or, &pix1, 0, 0)?;
        }

        Ok(dm.into())
    }

    /// Scale every image to a common tile width and tile them in `ncols`
    /// columns.
    ///
    /// Each image is scaled by `(tile_width - 2 * border) / w`. A 1 bpp
    /// image being reduced into a deeper output goes through
    /// `scale_to_gray`, exactly as C does, so text stays legible; everything
    /// else goes through the regular scaler. The result is then converted to
    /// `outdepth` and optionally bordered.
    ///
    /// `background` selects the fill: C paints white when
    /// `(background == 1 && outdepth == 1) || (background == 0 && outdepth != 1)`,
    /// i.e. **0 means white for 8 and 32 bpp output** and black for 1 bpp.
    ///
    /// A `border` larger than `tile_width / 5` is ignored, as in C.
    ///
    /// C equivalent: `pixaDisplayTiledAndScaled()` in `pixafunc2.c`
    pub fn display_tiled_and_scaled(
        &self,
        outdepth: crate::core::pix::PixelDepth,
        tile_width: u32,
        ncols: u32,
        background: u32,
        spacing: u32,
        border: u32,
    ) -> Result<Pix> {
        use crate::core::pix::PixelDepth;

        if self.pix.is_empty() {
            return Err(Error::NullInput("pixa is empty"));
        }
        if !matches!(
            outdepth,
            PixelDepth::Bit1 | PixelDepth::Bit8 | PixelDepth::Bit32
        ) {
            return Err(Error::InvalidParameter(
                "outdepth must be 1, 8 or 32".to_string(),
            ));
        }
        if ncols == 0 {
            return Err(Error::InvalidParameter("ncols must be > 0".to_string()));
        }
        if tile_width == 0 {
            return Err(Error::InvalidParameter(
                "tile_width must be > 0".to_string(),
            ));
        }
        // C: `if (border < 0 || border > tilewidth / 5) border = 0;`
        let border = if border > tile_width / 5 { 0 } else { border };
        let bordval = if outdepth == PixelDepth::Bit1 { 1 } else { 0 };

        let mut scaled_pix: Vec<Pix> = Vec::with_capacity(self.len());
        for pix in &self.pix {
            let w = pix.width();
            if w == 0 {
                continue;
            }
            let scalefact = (tile_width - 2 * border) as f32 / w as f32;
            // C: 1 bpp reduced into a deeper output goes through scale_to_gray.
            let pix1 = if pix.depth() == PixelDepth::Bit1
                && outdepth != PixelDepth::Bit1
                && scalefact < 1.0
            {
                crate::transform::scale_to_gray(pix, scalefact)
                    .map_err(|e| Error::NotSupported(e.to_string()))?
            } else {
                crate::transform::scale(
                    pix,
                    scalefact,
                    scalefact,
                    crate::transform::ScaleMethod::Auto,
                )
                .map_err(|e| Error::NotSupported(e.to_string()))?
            };
            let pixn = match outdepth {
                PixelDepth::Bit1 => pix1.convert_to_1(128)?,
                PixelDepth::Bit8 => pix1.convert_to_8()?,
                _ => pix1.convert_to_32()?,
            };
            let pixb = if border > 0 {
                pixn.add_border(border, bordval)?
            } else {
                pixn
            };
            scaled_pix.push(pixb);
        }

        if scaled_pix.is_empty() {
            return Err(Error::NullInput("no valid images after scaling"));
        }

        let n = scaled_pix.len();
        let ncols_u = ncols as usize;
        let nrows = n.div_ceil(ncols_u);
        let row_heights: Vec<u32> = (0..nrows)
            .map(|row| {
                (0..ncols_u)
                    .filter_map(|col| scaled_pix.get(row * ncols_u + col))
                    .map(|p| p.height())
                    .max()
                    .unwrap_or(0)
            })
            .collect();

        let canvas_w = tile_width * ncols + spacing * (ncols + 1);
        let canvas_h: u32 = row_heights.iter().sum::<u32>() + spacing * (nrows as u32 + 1);

        let canvas = Pix::new(canvas_w, canvas_h, outdepth)?;
        let mut dst = canvas.try_into_mut().unwrap_or_else(|p: Pix| p.to_mut());

        // C: pixSetAll when (background == 1 && outdepth == 1) ||
        //                  (background == 0 && outdepth != 1)
        let fill = if outdepth == PixelDepth::Bit1 {
            background == 1
        } else {
            background == 0
        };
        if fill {
            dst.set_all();
        }

        let mut cy = spacing as i32;
        for (row, &rh) in row_heights.iter().enumerate() {
            let mut cx = spacing as i32;
            for col in 0..ncols_u {
                let idx = row * ncols_u + col;
                if idx < n {
                    let src = &scaled_pix[idx];
                    dst.rop_region_inplace(
                        cx,
                        cy,
                        src.width(),
                        src.height(),
                        crate::core::pix::RopOp::Src,
                        src,
                        0,
                        0,
                    )?;
                }
                cx += tile_width as i32 + spacing as i32;
            }
            cy += rh as i32 + spacing as i32;
        }

        Ok(dst.into())
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

    /// Compute perimeter/sqrt(area) ratio for each Pix in the collection.
    ///
    /// Each Pix must be 1bpp.
    ///
    /// C equivalent: `pixaFindPerimSizeRatio()` in `pix5.c`
    pub fn find_perim_size_ratio(&self) -> Result<Numa> {
        let mut na = Numa::with_capacity(self.len());
        for pix in self.pix.iter() {
            let ratio = pix.find_perim_size_ratio()?;
            na.push(ratio);
        }
        Ok(na)
    }

    /// Compute fraction of 1-pixels in each Pix that are under `mask`.
    ///
    /// Each Pix and mask must be 1bpp.
    ///
    /// C equivalent: `pixaFindAreaFractionMasked()` in `pix5.c`
    pub fn find_area_fraction_masked(&self, mask: &Pix) -> Result<Numa> {
        let mut na = Numa::with_capacity(self.len());
        for pix in self.pix.iter() {
            let frac = pix.find_area_fraction_masked(mask)?;
            na.push(frac);
        }
        Ok(na)
    }

    /// Width/height ratio for each Pix in the collection.
    ///
    /// C equivalent: `pixaFindWidthHeightRatio()` in `pix5.c`
    pub fn find_width_height_ratio(&self) -> Result<Numa> {
        let mut na = Numa::with_capacity(self.len());
        for pix in self.pix.iter() {
            let h = pix.height();
            if h == 0 {
                na.push(0.0);
            } else {
                na.push(pix.width() as f32 / h as f32);
            }
        }
        Ok(na)
    }

    /// Width * height product for each Pix in the collection.
    ///
    /// C equivalent: `pixaFindWidthHeightProduct()` in `pix5.c`
    pub fn find_width_height_product(&self) -> Result<Numa> {
        let mut na = Numa::with_capacity(self.len());
        for pix in self.pix.iter() {
            na.push((pix.width() as f32) * (pix.height() as f32));
        }
        Ok(na)
    }
}

// ============================================================================
// Helper functions
// ============================================================================

use crate::core::box_::{compare_relation, compare_relation_i64};

/// Copy pixels from `src` onto `dst` at offset (ox, oy).
///
/// Uses per-pixel get/set; sufficient for small component images.
/// For bulk image operations, row-level memcpy would be more efficient.
///
/// Clips to destination bounds. Handles all pixel depths.
impl Pixa {
    /// Tile the images into rows, wrapping at `maxwidth`, exactly as C
    /// `pixaDisplayTiledInRows()`: images are normalized to `outdepth`
    /// (8 or 32; for `outdepth` = 1 the inputs must already be 1bpp — the
    /// C `pixConvertTo1(pix, 128)` depth conversion is not ported),
    /// optionally scaled and bordered, laid out left-to-right with
    /// `spacing` around them, and blitted with PIX_SRC onto a canvas whose
    /// background is set by `background` (1 = black for 1bpp,
    /// 0 = white otherwise). Like C, a first image wider than `maxwidth`
    /// records an empty leading row.
    #[allow(clippy::too_many_arguments)]
    pub fn display_tiled_in_rows(
        &self,
        outdepth: PixelDepth,
        maxwidth: u32,
        scalefactor: f32,
        background: u32,
        spacing: u32,
        border: u32,
    ) -> Result<Pix> {
        if !matches!(
            outdepth,
            PixelDepth::Bit1 | PixelDepth::Bit8 | PixelDepth::Bit32
        ) {
            return Err(Error::InvalidParameter(
                "outdepth must be 1, 8 or 32".to_string(),
            ));
        }
        if self.pix.is_empty() {
            return Err(Error::NullInput("pixa is empty"));
        }
        let scalefactor = if scalefactor <= 0.0 { 1.0 } else { scalefactor };

        // Normalize depths, scale, optionally add border.
        let bordval = if outdepth == PixelDepth::Bit1 { 1 } else { 0 };
        let mut norm: Vec<Pix> = Vec::with_capacity(self.pix.len());
        for pix in &self.pix {
            let pixn = match outdepth {
                PixelDepth::Bit8 => pix.convert_to_8()?,
                PixelDepth::Bit32 => pix.convert_to_32()?,
                _ => {
                    // C uses pixConvertTo1(pix, 128); this port only accepts
                    // inputs that are already 1bpp (a no-op there).
                    if pix.depth() != PixelDepth::Bit1 {
                        return Err(Error::UnsupportedDepth(pix.depth().bits()));
                    }
                    pix.deep_clone()
                }
            };
            let pix1 = if scalefactor != 1.0 {
                // C uses pixScale, i.e. the auto dispatch with sharpening.
                crate::transform::scale(
                    &pixn,
                    scalefactor,
                    scalefactor,
                    crate::transform::ScaleMethod::Auto,
                )
                .map_err(|e| Error::InvalidParameter(e.to_string()))?
            } else {
                pixn
            };
            let pixd = if border > 0 {
                pix1.add_border(border, bordval)?
            } else {
                pix1
            };
            norm.push(pixd);
        }

        // Row layout, exactly as C: accumulate widths until maxwidth.
        let spacing = spacing as i32;
        let maxwidth = maxwidth as i32;
        let mut nainrow: Vec<i32> = Vec::new();
        let mut namaxh: Vec<i32> = Vec::new();
        let mut wmaxrow = 0i32;
        let mut w = spacing;
        let mut h = spacing;
        let mut maxh = 0i32;
        let mut irow = 0i32;
        for pix in &norm {
            let wt = pix.width() as i32;
            let ht = pix.height() as i32;
            let wtry = w + wt + spacing;
            if wtry > maxwidth {
                nainrow.push(irow);
                namaxh.push(maxh);
                wmaxrow = wmaxrow.max(w);
                h += maxh + spacing;
                irow = 0;
                w = wt + 2 * spacing;
                maxh = ht;
            } else {
                w = wtry;
                maxh = maxh.max(ht);
            }
            irow += 1;
        }
        nainrow.push(irow);
        namaxh.push(maxh);
        wmaxrow = wmaxrow.max(w);
        h += maxh + spacing;

        let canvas = Pix::new(wmaxrow.max(1) as u32, h.max(1) as u32, outdepth)?;
        let mut canvas_mut = canvas.try_into_mut().unwrap_or_else(|p: Pix| p.to_mut());
        // C: background = 1 → black for 1bpp; background = 0 → white otherwise.
        if (background == 1 && outdepth == PixelDepth::Bit1)
            || (background == 0 && outdepth != PixelDepth::Bit1)
        {
            canvas_mut.set_all();
        }

        let mut y = spacing;
        let mut index = 0usize;
        for (row, &ninrow) in nainrow.iter().enumerate() {
            let maxh = namaxh[row];
            let mut x = spacing;
            for _ in 0..ninrow {
                let pix = &norm[index];
                blit_pix(&mut canvas_mut, pix, x, y);
                x += pix.width() as i32 + spacing;
                index += 1;
            }
            y += maxh + spacing;
        }

        Ok(canvas_mut.into())
    }

    /// Tile the images into `nx` columns, left to right and top to bottom.
    ///
    /// Unlike [`Pixa::display_tiled_in_rows`], the column count is fixed and
    /// each row is as tall as its tallest member, so images of differing
    /// sizes stay on a per-row baseline. All images are first converted to a
    /// common depth, then optionally scaled and given a border.
    ///
    /// The serialized layout [`Boxa`] is stored in the output's text field,
    /// as in C.
    ///
    /// # Arguments
    ///
    /// * `nx` - Number of columns (must be > 0)
    /// * `scalefactor` - Scale applied to each image (<= 0 means 1.0)
    /// * `spacing` - Pixels between images and around the border
    /// * `border` - Width of a border added to each image
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaDisplayTiledInColumns()` in `pixafunc2.c`
    pub fn display_tiled_in_columns(
        &self,
        nx: u32,
        scalefactor: f32,
        spacing: u32,
        border: u32,
    ) -> Result<Pix> {
        if self.pix.is_empty() {
            return Err(Error::NullInput("pixa is empty"));
        }
        if nx == 0 {
            return Err(Error::InvalidParameter("nx must be > 0".to_string()));
        }
        let scalefactor = if scalefactor <= 0.0 { 1.0 } else { scalefactor };

        // Convert to same depth, then scale and optionally add a border.
        let same = self.convert_to_same_depth()?;
        let maxd = same
            .pix
            .first()
            .map(|p| p.depth())
            .unwrap_or(PixelDepth::Bit1);
        let bordval = if maxd == PixelDepth::Bit1 { 1 } else { 0 };

        let mut norm: Vec<Pix> = Vec::with_capacity(same.pix.len());
        let mut res = 0;
        for (i, pix) in same.pix.iter().enumerate() {
            let pix1 = if scalefactor != 1.0 {
                crate::transform::scale(
                    pix,
                    scalefactor,
                    scalefactor,
                    crate::transform::ScaleMethod::Auto,
                )
                .map_err(|e| Error::InvalidParameter(e.to_string()))?
            } else {
                pix.clone()
            };
            let pix2 = if border > 0 {
                pix1.add_border(border, bordval)?
            } else {
                pix1
            };
            if i == 0 {
                res = pix2.xres();
            }
            norm.push(pix2);
        }

        // Compute the layout and save it as a boxa. Layout coordinates are
        // i32, so reject a spacing that cannot be represented there rather
        // than wrapping into negative box origins.
        let n = norm.len();
        let spacing = i32::try_from(spacing)
            .map_err(|_| Error::InvalidParameter("spacing exceeds i32::MAX".to_string()))?;
        let nrows = n.div_ceil(nx as usize);
        let mut boxa = Boxa::new();
        let mut y = spacing;
        let mut index = 0usize;
        for _ in 0..nrows {
            let mut x = spacing;
            let mut maxh = 0i32;
            for _ in 0..nx {
                if index >= n {
                    break;
                }
                let wb = norm[index].width() as i32;
                let hb = norm[index].height() as i32;
                boxa.push(Box::new(x, y, wb, hb)?);
                maxh = maxh.max(hb + spacing);
                x += wb + spacing;
                index += 1;
            }
            y += maxh;
        }

        // Render through pixaDisplay over the layout extent.
        let (w, h, _) = boxa
            .get_extent()
            .ok_or_else(|| Error::InvalidParameter("empty layout".to_string()))?;
        let mut laid = Pixa::with_capacity(n);
        for (pix, b) in norm.into_iter().zip(boxa.iter()) {
            laid.push_with_box(pix, *b);
        }
        let mut pixd = laid
            .display((w + spacing) as u32, (h + spacing) as u32)?
            .to_mut();
        pixd.set_resolution(res, res);

        // C stores the serialized boxa in the text field.
        if let Ok(data) = boxa.write_to_bytes() {
            pixd.set_text(Some(String::from_utf8_lossy(&data).into_owned()));
        }

        Ok(pixd.into())
    }
}

/// OR-composite `src` onto `dst` at (ox, oy) — C PIX_PAINT for 1bpp.
fn blit_pix_or(dst: &mut PixMut, src: &Pix, ox: i32, oy: i32) {
    let dw = dst.width() as i32;
    let dh = dst.height() as i32;
    let sw = src.width() as i32;
    let sh = src.height() as i32;

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
            // Loop bounds already guarantee in-range coordinates.
            if src.get_pixel_unchecked(sx as u32, sy as u32) != 0 {
                let dx = ox + sx;
                dst.set_pixel_unchecked(dx as u32, dy as u32, 1);
            }
        }
    }
}

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

/// Scale a Pix to a target size using nearest-neighbor sampling.
///
/// * `wd` = 0: scale proportionally to height `hd`.
/// * `hd` = 0: scale proportionally to width `wd`.
/// * both 0: deep clone.
fn scale_pix_to_size(src: &Pix, wd: u32, hd: u32) -> Pix {
    let sw = src.width();
    let sh = src.height();
    if sw == 0 || sh == 0 {
        return src.deep_clone();
    }
    let (tw, th) = match (wd, hd) {
        (0, 0) => return src.deep_clone(),
        (0, h) => {
            let scale = h as f32 / sh as f32;
            ((sw as f32 * scale).round() as u32, h)
        }
        (w, 0) => {
            let scale = w as f32 / sw as f32;
            (w, (sh as f32 * scale).round() as u32)
        }
        (w, h) => (w, h),
    };
    let tw = tw.max(1);
    let th = th.max(1);

    let depth = src.depth();
    let dst_pix = Pix::new(tw, th, depth).unwrap_or_else(|_| src.deep_clone());
    let mut dst = dst_pix.try_into_mut().unwrap_or_else(|p: Pix| p.to_mut());

    if let Some(cmap) = src.colormap() {
        let _ = dst.set_colormap(Some(cmap.clone()));
    }

    for dy in 0..th {
        let sy = ((dy as f32 + 0.5) * sh as f32 / th as f32) as u32;
        let sy = sy.min(sh - 1);
        for dx in 0..tw {
            let sx = ((dx as f32 + 0.5) * sw as f32 / tw as f32) as u32;
            let sx = sx.min(sw - 1);
            let val = src.get_pixel_unchecked(sx, sy);
            dst.set_pixel_unchecked(dx, dy, val);
        }
    }
    dst.into()
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

    /// Return `true` if every Pixa in the array is non-empty.
    ///
    /// An empty Pixaa is considered "full" (vacuously true),
    /// consistent with `Pixa::is_full`.
    ///
    /// C equivalent: `pixaaIsFull()` in `pixabasic.c`
    pub fn is_full(&self) -> bool {
        self.pixas.iter().all(|p| !p.is_empty())
    }

    /// Overwrite every Pixa slot with a clone of `pixa`.
    ///
    /// C equivalent: `pixaaInitFull()` in `pixabasic.c`
    pub fn init_full(&mut self, pixa: &Pixa) {
        for slot in &mut self.pixas {
            *slot = pixa.clone();
        }
    }

    /// Append all Pixa from `src` into `self`.
    ///
    /// Returns `Result<()>` for consistency with `Pixa::join` and to allow
    /// potential future validation (e.g., depth checks).
    ///
    /// C equivalent: `pixaaJoin()` in `pixabasic.c`
    pub fn join(&mut self, src: &Pixaa) -> Result<()> {
        for pixa in &src.pixas {
            self.pixas.push(pixa.clone());
        }
        Ok(())
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
    /// display_tiled_in_rows must reproduce the C layout algorithm.
    /// Expected positions hand-computed for two 8bpp images (4x3 and 5x2)
    /// with spacing 2: single-row layout at maxwidth 20 puts them at (2,2)
    /// and (8,2) on a 15x7 white canvas; maxwidth 12 wraps the second image
    /// to a new row at (2,7) on a 9x11 canvas.
    #[test]
    fn test_display_tiled_in_rows_matches_c() {
        use crate::core::{Pix, PixelDepth};

        let mut pixa = super::Pixa::new();
        let a = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut am = a.try_into_mut().unwrap();
        for y in 0..3 {
            for x in 0..4 {
                am.set_pixel(x, y, 10).unwrap();
            }
        }
        pixa.push(am.into());
        let b = Pix::new(5, 2, PixelDepth::Bit8).unwrap();
        let mut bm = b.try_into_mut().unwrap();
        for y in 0..2 {
            for x in 0..5 {
                bm.set_pixel(x, y, 20).unwrap();
            }
        }
        pixa.push(bm.into());

        let one_row = pixa
            .display_tiled_in_rows(PixelDepth::Bit8, 20, 1.0, 0, 2, 0)
            .unwrap();
        assert_eq!((one_row.width(), one_row.height()), (15, 7));
        assert_eq!(one_row.get_pixel(0, 0), Some(255), "white background");
        assert_eq!(one_row.get_pixel(2, 2), Some(10));
        assert_eq!(one_row.get_pixel(5, 4), Some(10));
        assert_eq!(one_row.get_pixel(8, 2), Some(20));
        assert_eq!(one_row.get_pixel(8, 4), Some(255));

        let wrapped = pixa
            .display_tiled_in_rows(PixelDepth::Bit8, 12, 1.0, 0, 2, 0)
            .unwrap();
        assert_eq!((wrapped.width(), wrapped.height()), (9, 11));
        assert_eq!(wrapped.get_pixel(2, 2), Some(10));
        assert_eq!(wrapped.get_pixel(2, 7), Some(20));
    }

    /// display must reproduce C pixaDisplay: 1bpp components are composited
    /// with PIX_PAINT (OR) so overlapping bounding boxes do not erase
    /// previously painted fg pixels, and canvases deeper than 1bpp start
    /// all-white (pixSetAll).
    #[test]
    fn test_display_matches_c_compositing() {
        use crate::core::{Box, Pix, PixelDepth};

        // Two overlapping 1bpp components: a pixel of the first lies inside
        // the second's bounding box but is bg in the second's mask.
        let mut pixa = super::Pixa::new();
        let a = Pix::new(3, 1, PixelDepth::Bit1).unwrap();
        let mut am = a.try_into_mut().unwrap();
        am.set_pixel(0, 0, 1).unwrap();
        am.set_pixel(2, 0, 1).unwrap();
        pixa.push_with_box(am.into(), Box::new_unchecked(0, 0, 3, 1));

        let b = Pix::new(3, 1, PixelDepth::Bit1).unwrap();
        let mut bm = b.try_into_mut().unwrap();
        bm.set_pixel(1, 0, 1).unwrap();
        pixa.push_with_box(bm.into(), Box::new_unchecked(0, 0, 3, 1));

        let disp = pixa.display(3, 1).unwrap();
        assert_eq!(disp.get_pixel(0, 0), Some(1), "OR must keep first fg");
        assert_eq!(disp.get_pixel(1, 0), Some(1));
        assert_eq!(disp.get_pixel(2, 0), Some(1), "OR must keep first fg");

        // 8bpp canvas starts white (255) outside any component.
        let mut pixa8 = super::Pixa::new();
        let g = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        pixa8.push_with_box(g, Box::new_unchecked(0, 0, 1, 1));
        let disp8 = pixa8.display(3, 1).unwrap();
        assert_eq!(disp8.get_pixel(0, 0), Some(0), "component copied as-is");
        assert_eq!(disp8.get_pixel(2, 0), Some(255), "background is white");
    }

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
        use crate::core::pix::PixelDepth;

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
        use crate::core::pix::PixelDepth;

        let mut pixa = Pixa::new();
        pixa.push(Pix::new(10, 10, PixelDepth::Bit8).unwrap());
        assert!(pixa.count_pixels().is_err());
    }

    // -- Pixa::extract_column_from_each --

    #[test]
    fn test_extract_column_from_each_basic() {
        use crate::core::pix::PixelDepth;
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
    fn test_aligned_stats_mean() {
        use crate::core::pix::PixelDepth;
        use crate::core::pix::statistics::RowColStatType;
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
    fn test_aligned_stats_empty_pixa() {
        use crate::core::pix::statistics::RowColStatType;
        let pixa = Pixa::new();
        assert!(
            pixa.aligned_stats(RowColStatType::MeanAbsVal, 0, 0)
                .is_err()
        );
    }

    // -- Pixa::find_dimensions --

    #[test]
    fn test_find_dimensions_basic() {
        use crate::core::pix::{Pix, PixelDepth};
        let mut pixa = Pixa::new();
        pixa.push(Pix::new(10, 20, PixelDepth::Bit8).unwrap());
        pixa.push(Pix::new(30, 40, PixelDepth::Bit8).unwrap());
        pixa.push(Pix::new(5, 15, PixelDepth::Bit8).unwrap());
        let (na_w, na_h) = pixa.find_dimensions().unwrap();
        assert_eq!(na_w.len(), 3);
        assert_eq!(na_h.len(), 3);
        assert_eq!(na_w.get(0).unwrap(), 10.0);
        assert_eq!(na_h.get(0).unwrap(), 20.0);
        assert_eq!(na_w.get(1).unwrap(), 30.0);
        assert_eq!(na_h.get(1).unwrap(), 40.0);
        assert_eq!(na_w.get(2).unwrap(), 5.0);
        assert_eq!(na_h.get(2).unwrap(), 15.0);
    }

    #[test]
    fn test_find_dimensions_empty() {
        let pixa = Pixa::new();
        assert!(pixa.find_dimensions().is_err());
    }

    // -- Phase 16.4 new functions --

    #[test]
    fn test_create_from_pix() {
        let pix = make_test_pix(10, 20);
        let pixa = Pixa::create_from_pix(&pix, 3);
        assert_eq!(pixa.len(), 3);
        assert_eq!(pixa.get(0).unwrap().width(), 10);
        assert_eq!(pixa.get(2).unwrap().height(), 20);
    }

    #[test]
    fn test_create_from_boxa() {
        use crate::core::pix::PixelDepth;
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut boxa = Boxa::new();
        boxa.push(crate::core::box_::Box::new(0, 0, 10, 10).unwrap());
        boxa.push(crate::core::box_::Box::new(20, 20, 15, 15).unwrap());
        let pixa = Pixa::create_from_boxa(&pix, &boxa);
        assert_eq!(pixa.len(), 2);
    }

    #[test]
    fn test_split_pix() {
        use crate::core::pix::PixelDepth;
        let pix = Pix::new(100, 60, PixelDepth::Bit8).unwrap();
        let pixa = Pixa::split_pix(&pix, 2, 3, 0, 0).unwrap();
        assert_eq!(pixa.len(), 6); // 2*3
    }

    #[test]
    fn test_get_box_geometry() {
        let mut pixa = Pixa::new();
        let pix = make_test_pix(10, 10);
        pixa.push_with_box(pix, crate::core::box_::Box::new(5, 10, 20, 30).unwrap());
        let (x, y, w, h) = pixa.get_box_geometry(0).unwrap();
        assert_eq!(x, 5);
        assert_eq!(y, 10);
        assert_eq!(w, 20);
        assert_eq!(h, 30);
    }

    #[test]
    fn test_is_full() {
        // Empty Pixa is vacuously full
        assert!(Pixa::new().is_full());
        let mut pixa = Pixa::new();
        pixa.init_full(3, Some(&make_test_pix(10, 10)), None);
        assert!(pixa.is_full());
    }

    #[test]
    fn test_pixa_set_text_count_text() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(10, 10));
        pixa.push(make_test_pix(10, 10));
        pixa.push(make_test_pix(10, 10));
        pixa.set_text(Some("hello".to_string()));
        assert_eq!(pixa.count_text(), 3);
        pixa.set_text(None);
        assert_eq!(pixa.count_text(), 0);
    }

    #[test]
    fn test_pixa_remove_selected() {
        let mut pixa = Pixa::new();
        pixa.push(make_test_pix(1, 1));
        pixa.push(make_test_pix(2, 2));
        pixa.push(make_test_pix(3, 3));
        pixa.push(make_test_pix(4, 4));
        // Remove indices 1 and 3 (descending order required)
        let na = crate::core::numa::Numa::from_slice(&[3.0, 1.0]);
        pixa.remove_selected(&na).unwrap();
        assert_eq!(pixa.len(), 2);
        assert_eq!(pixa.get(0).unwrap().width(), 1);
        assert_eq!(pixa.get(1).unwrap().width(), 3);
    }

    #[test]
    fn test_pixa_join() {
        let mut pixa1 = Pixa::new();
        pixa1.push(make_test_pix(1, 1));
        pixa1.push(make_test_pix(2, 2));
        let pixa2 = {
            let mut p = Pixa::new();
            p.push(make_test_pix(3, 3));
            p
        };
        pixa1.join(&pixa2, 0, None).unwrap();
        assert_eq!(pixa1.len(), 3);
        assert_eq!(pixa1.get(2).unwrap().width(), 3);
    }

    #[test]
    fn test_pixa_interleave() {
        let mut pixa1 = Pixa::new();
        pixa1.push(make_test_pix(1, 1));
        pixa1.push(make_test_pix(3, 3));
        let mut pixa2 = Pixa::new();
        pixa2.push(make_test_pix(2, 2));
        pixa2.push(make_test_pix(4, 4));
        let merged = pixa1.interleave(&pixa2).unwrap();
        assert_eq!(merged.len(), 4);
        assert_eq!(merged.get(0).unwrap().width(), 1);
        assert_eq!(merged.get(1).unwrap().width(), 2);
        assert_eq!(merged.get(2).unwrap().width(), 3);
        assert_eq!(merged.get(3).unwrap().width(), 4);
    }

    #[test]
    fn test_pixaa_is_full() {
        // Empty Pixaa is vacuously full (consistent with Pixa::is_full)
        assert!(Pixaa::new().is_full());
        let mut pixaa = Pixaa::new();
        let mut p = Pixa::new();
        p.push(make_test_pix(10, 10));
        pixaa.push(p);
        assert!(pixaa.is_full());
        pixaa.push(Pixa::new()); // empty slot
        assert!(!pixaa.is_full());
    }

    #[test]
    fn test_pixaa_init_full() {
        let mut pixaa = Pixaa::new();
        let mut template = Pixa::new();
        template.push(make_test_pix(5, 5));
        for _ in 0..3 {
            pixaa.push(Pixa::new());
        }
        pixaa.init_full(&template);
        for i in 0..3 {
            assert_eq!(pixaa.get(i).unwrap().len(), 1);
        }
    }

    #[test]
    fn test_pixaa_join() {
        let mut pixaa1 = Pixaa::new();
        let mut p = Pixa::new();
        p.push(make_test_pix(1, 1));
        pixaa1.push(p);
        let mut pixaa2 = Pixaa::new();
        let mut q = Pixa::new();
        q.push(make_test_pix(2, 2));
        pixaa2.push(q);
        pixaa1.join(&pixaa2).unwrap();
        assert_eq!(pixaa1.len(), 2);
    }

    // -- Phase 16.5 new functions --

    #[test]
    fn test_scale_to_size() {
        let pix = make_test_pix(10, 20);
        let pixa = Pixa::create_from_pix(&pix, 3);
        let scaled = pixa.scale_to_size(5, 0);
        assert_eq!(scaled.len(), 3);
        assert_eq!(scaled.get(0).unwrap().width(), 5);
        assert_eq!(scaled.get(0).unwrap().height(), 10); // proportional
    }

    #[test]
    fn test_scale_to_size_rel() {
        let pix = make_test_pix(10, 20);
        let pixa = Pixa::create_from_pix(&pix, 2);
        let scaled = pixa.scale_to_size_rel(5, 0);
        assert_eq!(scaled.len(), 2);
        assert_eq!(scaled.get(0).unwrap().width(), 15);
        assert_eq!(scaled.get(0).unwrap().height(), 20);
    }

    #[test]
    fn test_display_tiled_and_scaled() {
        use crate::core::pix::PixelDepth;
        let pix = make_test_pix(10, 10);
        let pixa = Pixa::create_from_pix(&pix, 4);
        let result = pixa.display_tiled_and_scaled(PixelDepth::Bit8, 20, 2, 0, 2, 0);
        assert!(result.is_ok());
        let out = result.unwrap();
        assert!(out.width() > 0 && out.height() > 0);
    }
}
