//! Pta, Ptaa - Point arrays
//!
//! Arrays of floating-point coordinate pairs.

use crate::error::{Error, Result};

/// Array of points
#[derive(Debug, Clone, Default)]
pub struct Pta {
    /// X coordinates
    x: Vec<f32>,
    /// Y coordinates
    y: Vec<f32>,
}

impl Pta {
    /// Create a new empty Pta
    pub fn new() -> Self {
        Self {
            x: Vec::new(),
            y: Vec::new(),
        }
    }

    /// Create a Pta with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            x: Vec::with_capacity(capacity),
            y: Vec::with_capacity(capacity),
        }
    }

    /// Create a Pta from coordinate vectors
    pub fn from_vecs(x: Vec<f32>, y: Vec<f32>) -> Result<Self> {
        if x.len() != y.len() {
            return Err(Error::InvalidParameter(format!(
                "x and y vectors must have same length: {} vs {}",
                x.len(),
                y.len()
            )));
        }
        Ok(Self { x, y })
    }

    /// Get the number of points
    #[inline]
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// Get a point by index
    pub fn get(&self, index: usize) -> Option<(f32, f32)> {
        if index < self.x.len() {
            Some((self.x[index], self.y[index]))
        } else {
            None
        }
    }

    /// Get X coordinate by index
    pub fn get_x(&self, index: usize) -> Option<f32> {
        self.x.get(index).copied()
    }

    /// Get Y coordinate by index
    pub fn get_y(&self, index: usize) -> Option<f32> {
        self.y.get(index).copied()
    }

    /// Add a point
    pub fn push(&mut self, x: f32, y: f32) {
        self.x.push(x);
        self.y.push(y);
    }

    /// Remove and return the last point
    pub fn pop(&mut self) -> Option<(f32, f32)> {
        if self.x.is_empty() {
            None
        } else {
            Some((self.x.pop().unwrap(), self.y.pop().unwrap()))
        }
    }

    /// Set a point at index
    pub fn set(&mut self, index: usize, x: f32, y: f32) -> Result<()> {
        if index >= self.x.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.x.len(),
            });
        }
        self.x[index] = x;
        self.y[index] = y;
        Ok(())
    }

    /// Remove a point at index
    pub fn remove(&mut self, index: usize) -> Result<(f32, f32)> {
        if index >= self.x.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.x.len(),
            });
        }
        Ok((self.x.remove(index), self.y.remove(index)))
    }

    /// Insert a point at index
    pub fn insert(&mut self, index: usize, x: f32, y: f32) -> Result<()> {
        if index > self.x.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.x.len(),
            });
        }
        self.x.insert(index, x);
        self.y.insert(index, y);
        Ok(())
    }

    /// Clear all points
    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
    }

    /// Get X coordinates as a slice
    pub fn x_coords(&self) -> &[f32] {
        &self.x
    }

    /// Get Y coordinates as a slice
    pub fn y_coords(&self) -> &[f32] {
        &self.y
    }

    /// Get mutable X coordinates
    pub fn x_coords_mut(&mut self) -> &mut [f32] {
        &mut self.x
    }

    /// Get mutable Y coordinates
    pub fn y_coords_mut(&mut self) -> &mut [f32] {
        &mut self.y
    }

    /// Compute the bounding box
    pub fn bounding_box(&self) -> Option<(f32, f32, f32, f32)> {
        if self.x.is_empty() {
            return None;
        }

        let mut x_min = f32::MAX;
        let mut y_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_max = f32::MIN;

        for i in 0..self.x.len() {
            x_min = x_min.min(self.x[i]);
            y_min = y_min.min(self.y[i]);
            x_max = x_max.max(self.x[i]);
            y_max = y_max.max(self.y[i]);
        }

        Some((x_min, y_min, x_max, y_max))
    }

    /// Compute the centroid
    pub fn centroid(&self) -> Option<(f32, f32)> {
        if self.x.is_empty() {
            return None;
        }

        let n = self.x.len() as f32;
        let sum_x: f32 = self.x.iter().sum();
        let sum_y: f32 = self.y.iter().sum();

        Some((sum_x / n, sum_y / n))
    }

    /// Translate all points
    pub fn translate(&mut self, dx: f32, dy: f32) {
        for x in &mut self.x {
            *x += dx;
        }
        for y in &mut self.y {
            *y += dy;
        }
    }

    /// Scale all points relative to origin
    pub fn scale(&mut self, sx: f32, sy: f32) {
        for x in &mut self.x {
            *x *= sx;
        }
        for y in &mut self.y {
            *y *= sy;
        }
    }

    /// Rotate all points around origin
    pub fn rotate(&mut self, angle: f32) {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        for i in 0..self.x.len() {
            let x = self.x[i];
            let y = self.y[i];
            self.x[i] = x * cos_a - y * sin_a;
            self.y[i] = x * sin_a + y * cos_a;
        }
    }

    /// Create an iterator over points
    pub fn iter(&self) -> PtaIter<'_> {
        PtaIter {
            pta: self,
            index: 0,
        }
    }
}

/// Iterator over Pta points
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

/// Array of Pta
#[derive(Debug, Clone, Default)]
pub struct Ptaa {
    ptas: Vec<Pta>,
}

impl Ptaa {
    /// Create a new empty Ptaa
    pub fn new() -> Self {
        Self { ptas: Vec::new() }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ptas: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of Pta
    #[inline]
    pub fn len(&self) -> usize {
        self.ptas.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ptas.is_empty()
    }

    /// Get a Pta by index
    pub fn get(&self, index: usize) -> Option<&Pta> {
        self.ptas.get(index)
    }

    /// Get a mutable Pta by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Pta> {
        self.ptas.get_mut(index)
    }

    /// Add a Pta
    pub fn push(&mut self, pta: Pta) {
        self.ptas.push(pta);
    }

    /// Remove and return the last Pta
    pub fn pop(&mut self) -> Option<Pta> {
        self.ptas.pop()
    }

    /// Clear all Pta
    pub fn clear(&mut self) {
        self.ptas.clear();
    }

    /// Get all Pta as a slice
    pub fn ptas(&self) -> &[Pta] {
        &self.ptas
    }

    /// Get total number of points across all Pta
    pub fn total_points(&self) -> usize {
        self.ptas.iter().map(|p| p.len()).sum()
    }

    /// Flatten into a single Pta
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

    /// Create an iterator over Pta
    pub fn iter(&self) -> impl Iterator<Item = &Pta> {
        self.ptas.iter()
    }

    /// Create a mutable iterator over Pta
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pta> {
        self.ptas.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pta_creation() {
        let mut pta = Pta::new();
        pta.push(1.0, 2.0);
        pta.push(3.0, 4.0);

        assert_eq!(pta.len(), 2);
        assert_eq!(pta.get(0), Some((1.0, 2.0)));
        assert_eq!(pta.get(1), Some((3.0, 4.0)));
        assert_eq!(pta.get(2), None);
    }

    #[test]
    fn test_pta_from_vecs() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![4.0, 5.0, 6.0];
        let pta = Pta::from_vecs(x, y).unwrap();

        assert_eq!(pta.len(), 3);
        assert_eq!(pta.get(1), Some((2.0, 5.0)));

        // Mismatched lengths should fail
        let x = vec![1.0, 2.0];
        let y = vec![4.0, 5.0, 6.0];
        assert!(Pta::from_vecs(x, y).is_err());
    }

    #[test]
    fn test_pta_bounding_box() {
        let mut pta = Pta::new();
        pta.push(10.0, 20.0);
        pta.push(30.0, 5.0);
        pta.push(15.0, 40.0);

        let (x_min, y_min, x_max, y_max) = pta.bounding_box().unwrap();
        assert_eq!(x_min, 10.0);
        assert_eq!(y_min, 5.0);
        assert_eq!(x_max, 30.0);
        assert_eq!(y_max, 40.0);
    }

    #[test]
    fn test_pta_centroid() {
        let mut pta = Pta::new();
        pta.push(0.0, 0.0);
        pta.push(10.0, 0.0);
        pta.push(10.0, 10.0);
        pta.push(0.0, 10.0);

        let (cx, cy) = pta.centroid().unwrap();
        assert_eq!(cx, 5.0);
        assert_eq!(cy, 5.0);
    }

    #[test]
    fn test_pta_transform() {
        let mut pta = Pta::new();
        pta.push(1.0, 2.0);

        pta.translate(10.0, 20.0);
        assert_eq!(pta.get(0), Some((11.0, 22.0)));

        pta.scale(2.0, 2.0);
        assert_eq!(pta.get(0), Some((22.0, 44.0)));
    }

    #[test]
    fn test_pta_iterator() {
        let pta: Pta = [(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)].into_iter().collect();
        assert_eq!(pta.len(), 3);

        let points: Vec<_> = pta.iter().collect();
        assert_eq!(points, vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)]);
    }

    #[test]
    fn test_ptaa() {
        let mut ptaa = Ptaa::new();

        let mut pta1 = Pta::new();
        pta1.push(0.0, 0.0);
        pta1.push(1.0, 1.0);
        ptaa.push(pta1);

        let mut pta2 = Pta::new();
        pta2.push(2.0, 2.0);
        ptaa.push(pta2);

        assert_eq!(ptaa.len(), 2);
        assert_eq!(ptaa.total_points(), 3);

        let flat = ptaa.flatten();
        assert_eq!(flat.len(), 3);
    }
}
