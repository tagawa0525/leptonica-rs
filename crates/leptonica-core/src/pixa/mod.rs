//! Pixa, Pixaa - Arrays of Pix images
//!
//! These structures manage collections of images, optionally with
//! associated bounding boxes for each image.
//!
//! # See also
//!
//! C Leptonica: `pixabasic.c`, `pixafunc1.c`, `pixafunc2.c`

use crate::box_::{Box, Boxa};
use crate::error::{Error, Result};
use crate::pix::{Pix, PixelDepth};

/// Array of Pix images
///
/// `Pixa` manages a collection of `Pix` images along with optional
/// bounding boxes for each image.
///
/// # See also
///
/// C Leptonica: `struct Pixa` in `environ.h`, `pixaCreate()` in `pixabasic.c`
#[derive(Debug, Clone, Default)]
pub struct Pixa {
    pix: Vec<Pix>,
    boxa: Boxa,
}

impl Pixa {
    /// Create a new empty Pixa
    pub fn new() -> Self {
        todo!()
    }

    /// Create a Pixa with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        todo!()
    }

    /// Get the number of Pix images
    #[inline]
    pub fn len(&self) -> usize {
        todo!()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Get a reference to a Pix by index
    pub fn get(&self, index: usize) -> Option<&Pix> {
        todo!()
    }

    /// Get a mutable reference to a Pix by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Pix> {
        todo!()
    }

    /// Get a cloned Pix by index
    pub fn get_cloned(&self, index: usize) -> Option<Pix> {
        todo!()
    }

    /// Get the dimensions of a Pix by index
    pub fn get_dimensions(&self, index: usize) -> Option<(u32, u32, PixelDepth)> {
        todo!()
    }

    /// Add a Pix to the array
    pub fn push(&mut self, pix: Pix) {
        todo!()
    }

    /// Add a Pix with an associated bounding box
    pub fn push_with_box(&mut self, pix: Pix, b: Box) {
        todo!()
    }

    /// Remove and return the last Pix
    pub fn pop(&mut self) -> Option<Pix> {
        todo!()
    }

    /// Remove a Pix at index
    pub fn remove(&mut self, index: usize) -> Result<Pix> {
        todo!()
    }

    /// Insert a Pix at index
    pub fn insert(&mut self, index: usize, pix: Pix) -> Result<()> {
        todo!()
    }

    /// Replace a Pix at index
    pub fn replace(&mut self, index: usize, pix: Pix) -> Result<Pix> {
        todo!()
    }

    /// Clear all Pix images and boxes
    pub fn clear(&mut self) {
        todo!()
    }

    /// Extend the array to accommodate at least `size` elements
    pub fn extend_to_size(&mut self, size: usize) {
        todo!()
    }

    /// Initialize all slots with copies of the given Pix and optional Box
    pub fn init_full(&mut self, count: usize, pix: Option<&Pix>, b: Option<&Box>) {
        todo!()
    }

    /// Get all Pix as a slice
    pub fn pix_slice(&self) -> &[Pix] {
        todo!()
    }

    /// Get a reference to the Boxa
    pub fn boxa(&self) -> &Boxa {
        todo!()
    }

    /// Get a mutable reference to the Boxa
    pub fn boxa_mut(&mut self) -> &mut Boxa {
        todo!()
    }

    /// Get the number of boxes
    pub fn boxa_count(&self) -> usize {
        todo!()
    }

    /// Get a box by index
    pub fn get_box(&self, index: usize) -> Option<&Box> {
        todo!()
    }

    /// Set the Boxa, replacing any existing boxes
    pub fn set_boxa(&mut self, boxa: Boxa) {
        todo!()
    }

    /// Add a box for an existing Pix
    pub fn add_box(&mut self, b: Box) {
        todo!()
    }

    /// Verify that all Pix have the same depth
    pub fn verify_depth(&self) -> Result<(bool, PixelDepth)> {
        todo!()
    }

    /// Verify that all Pix have the same dimensions
    pub fn verify_dimensions(&self) -> Result<bool> {
        todo!()
    }

    /// Create a deep copy of this Pixa
    pub fn deep_clone(&self) -> Self {
        todo!()
    }

    /// Create an iterator over Pix references
    pub fn iter(&self) -> PixaIter<'_> {
        todo!()
    }

    /// Create a mutable iterator over Pix references
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pix> {
        self.pix.iter_mut()
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
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
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
        todo!()
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
/// # See also
///
/// C Leptonica: `struct Pixaa` in `environ.h`, `pixaaCreate()` in `pixabasic.c`
#[derive(Debug, Clone, Default)]
pub struct Pixaa {
    pixas: Vec<Pixa>,
}

impl Pixaa {
    /// Create a new empty Pixaa
    pub fn new() -> Self {
        todo!()
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        todo!()
    }

    /// Get the number of Pixa
    #[inline]
    pub fn len(&self) -> usize {
        todo!()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Get a Pixa by index
    pub fn get(&self, index: usize) -> Option<&Pixa> {
        todo!()
    }

    /// Get a mutable Pixa by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Pixa> {
        todo!()
    }

    /// Add a Pixa
    pub fn push(&mut self, pixa: Pixa) {
        todo!()
    }

    /// Remove and return the last Pixa
    pub fn pop(&mut self) -> Option<Pixa> {
        todo!()
    }

    /// Replace a Pixa at index
    pub fn replace(&mut self, index: usize, pixa: Pixa) -> Result<Pixa> {
        todo!()
    }

    /// Clear all Pixa
    pub fn clear(&mut self) {
        todo!()
    }

    /// Get all Pixa as a slice
    pub fn pixas(&self) -> &[Pixa] {
        todo!()
    }

    /// Get total number of Pix across all Pixa
    pub fn total_pix(&self) -> usize {
        todo!()
    }

    /// Flatten into a single Pixa
    pub fn flatten(&self) -> Pixa {
        todo!()
    }

    /// Get a specific Pix from a Pixa
    pub fn get_pix(&self, pixa_index: usize, pix_index: usize) -> Option<&Pix> {
        todo!()
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
