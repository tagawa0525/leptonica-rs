//! Pixa, Pixaa - Arrays of Pix images
//!
//! These structures manage collections of images, optionally with
//! associated bounding boxes for each image.

use crate::box_::{Box, Boxa};
use crate::error::{Error, Result};
use crate::pix::{Pix, PixelDepth};

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
}
