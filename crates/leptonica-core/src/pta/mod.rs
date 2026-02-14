//! Pta, Ptaa - Point arrays
//!
//! Arrays of floating-point coordinate pairs, used for geometric
//! operations, contour representation, and point transformations.
//!
//! # Storage layout
//!
//! Points are stored as separate X and Y vectors (SoA layout),
//! matching C Leptonica's internal representation.
//!
//! # See also
//!
//! - C Leptonica: `pts.h` (struct definitions), `ptabasic.c` (creation/access)
//! - `ptafunc1.c`, `ptafunc2.c` (transformations, sorting)

use crate::error::{Error, Result};

/// Array of points.
///
/// Stores 2D points as parallel x/y coordinate vectors.
///
/// # See also
///
/// C Leptonica: `struct Pta` in `pts.h`
#[derive(Debug, Clone, Default)]
pub struct Pta {
    /// X coordinates
    x: Vec<f32>,
    /// Y coordinates
    y: Vec<f32>,
}

impl Pta {
    /// Create a new empty Pta.
    pub fn new() -> Self {
        Self {
            x: Vec::new(),
            y: Vec::new(),
        }
    }

    /// Create a Pta with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            x: Vec::with_capacity(capacity),
            y: Vec::with_capacity(capacity),
        }
    }

    /// Create a Pta from coordinate vectors.
    ///
    /// # Errors
    ///
    /// Returns an error if `x` and `y` have different lengths.
    pub fn from_vecs(x: Vec<f32>, y: Vec<f32>) -> Result<Self> {
        todo!("Pta::from_vecs")
    }

    /// Get the number of points.
    #[inline]
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// Get a point by index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<(f32, f32)> {
        todo!("Pta::get")
    }

    /// Get X coordinate by index.
    pub fn get_x(&self, index: usize) -> Option<f32> {
        self.x.get(index).copied()
    }

    /// Get Y coordinate by index.
    pub fn get_y(&self, index: usize) -> Option<f32> {
        self.y.get(index).copied()
    }

    /// Add a point.
    pub fn push(&mut self, x: f32, y: f32) {
        self.x.push(x);
        self.y.push(y);
    }

    /// Remove and return the last point.
    pub fn pop(&mut self) -> Option<(f32, f32)> {
        if self.x.is_empty() {
            None
        } else {
            Some((self.x.pop().unwrap(), self.y.pop().unwrap()))
        }
    }

    /// Set a point at index.
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexOutOfBounds`] if the index is out of bounds.
    pub fn set(&mut self, index: usize, x: f32, y: f32) -> Result<()> {
        todo!("Pta::set")
    }

    /// Remove a point at index.
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexOutOfBounds`] if the index is out of bounds.
    pub fn remove(&mut self, index: usize) -> Result<(f32, f32)> {
        todo!("Pta::remove")
    }

    /// Insert a point at index.
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexOutOfBounds`] if the index is beyond the length.
    pub fn insert(&mut self, index: usize, x: f32, y: f32) -> Result<()> {
        todo!("Pta::insert")
    }

    /// Clear all points.
    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
    }

    /// Get X coordinates as a slice.
    pub fn x_coords(&self) -> &[f32] {
        &self.x
    }

    /// Get Y coordinates as a slice.
    pub fn y_coords(&self) -> &[f32] {
        &self.y
    }

    /// Compute the bounding box.
    ///
    /// Returns `(x_min, y_min, x_max, y_max)`, or `None` if empty.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaGetRange()` in `ptafunc1.c`
    pub fn bounding_box(&self) -> Option<(f32, f32, f32, f32)> {
        todo!("Pta::bounding_box")
    }

    /// Compute the centroid.
    ///
    /// Returns `(cx, cy)`, or `None` if empty.
    pub fn centroid(&self) -> Option<(f32, f32)> {
        todo!("Pta::centroid")
    }

    /// Translate all points by (dx, dy).
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaTransform()` in `ptafunc1.c` (with shift only)
    pub fn translate(&mut self, dx: f32, dy: f32) {
        todo!("Pta::translate")
    }

    /// Scale all points relative to origin.
    pub fn scale(&mut self, sx: f32, sy: f32) {
        todo!("Pta::scale")
    }

    /// Rotate all points around origin.
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians (counter-clockwise).
    pub fn rotate(&mut self, angle: f32) {
        todo!("Pta::rotate")
    }

    /// Create an iterator over points.
    pub fn iter(&self) -> PtaIter<'_> {
        PtaIter {
            pta: self,
            index: 0,
        }
    }
}

/// Iterator over Pta points.
pub struct PtaIter<'a> {
    pta: &'a Pta,
    index: usize,
}

impl<'a> Iterator for PtaIter<'a> {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.pta.len() {
            let pt = (self.pta.x[self.index], self.pta.y[self.index]);
            self.index += 1;
            Some(pt)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.pta.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for PtaIter<'_> {}

impl<'a> IntoIterator for &'a Pta {
    type Item = (f32, f32);
    type IntoIter = PtaIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl FromIterator<(f32, f32)> for Pta {
    fn from_iter<T: IntoIterator<Item = (f32, f32)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let capacity = upper.unwrap_or(lower);

        let mut pta = Self::with_capacity(capacity);
        for (x, y) in iter {
            pta.push(x, y);
        }
        pta
    }
}

/// Array of Pta.
///
/// # See also
///
/// C Leptonica: `struct Ptaa` in `pts.h`
#[derive(Debug, Clone, Default)]
pub struct Ptaa {
    ptas: Vec<Pta>,
}

impl Ptaa {
    /// Create a new empty Ptaa.
    pub fn new() -> Self {
        Self { ptas: Vec::new() }
    }

    /// Create with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ptas: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of Pta.
    #[inline]
    pub fn len(&self) -> usize {
        self.ptas.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ptas.is_empty()
    }

    /// Get a Pta by index.
    pub fn get(&self, index: usize) -> Option<&Pta> {
        self.ptas.get(index)
    }

    /// Get a mutable Pta by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Pta> {
        self.ptas.get_mut(index)
    }

    /// Add a Pta.
    pub fn push(&mut self, pta: Pta) {
        self.ptas.push(pta);
    }

    /// Remove and return the last Pta.
    pub fn pop(&mut self) -> Option<Pta> {
        self.ptas.pop()
    }

    /// Clear all Pta.
    pub fn clear(&mut self) {
        self.ptas.clear();
    }

    /// Get all Pta as a slice.
    pub fn ptas(&self) -> &[Pta] {
        &self.ptas
    }

    /// Get total number of points across all Pta.
    pub fn total_points(&self) -> usize {
        self.ptas.iter().map(|p| p.len()).sum()
    }

    /// Flatten into a single Pta.
    pub fn flatten(&self) -> Pta {
        let total = self.total_points();
        let mut result = Pta::with_capacity(total);
        for pta in &self.ptas {
            for (x, y) in pta.iter() {
                result.push(x, y);
            }
        }
        result
    }

    /// Create an iterator over Pta.
    pub fn iter(&self) -> impl Iterator<Item = &Pta> {
        self.ptas.iter()
    }

    /// Create a mutable iterator over Pta.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pta> {
        self.ptas.iter_mut()
    }
}
