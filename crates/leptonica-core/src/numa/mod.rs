//! Numa, Numaa - Numeric arrays
//!
//! Arrays of floating-point numbers, commonly used for histograms,
//! signal processing, and general numeric data storage.

mod histogram;
mod operations;

pub use histogram::HistogramStats;
pub use operations::{HistogramResult, WindowedStats};

use crate::error::{Error, Result};

/// Array of floating-point numbers
///
/// `Numa` manages a dynamic array of `f32` values. It includes optional
/// parameters `startx` and `delx` for representing sampled functions
/// or histograms where values correspond to evenly-spaced x positions.
///
/// # Examples
///
/// ```
/// use leptonica_core::Numa;
///
/// let mut numa = Numa::new();
/// numa.push(1.0);
/// numa.push(2.0);
/// numa.push(3.0);
///
/// assert_eq!(numa.len(), 3);
/// assert_eq!(numa.sum(), Some(6.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Numa {
    /// The numeric values
    data: Vec<f32>,
    /// Starting x value (for histograms/sampled functions)
    startx: f32,
    /// Delta x between samples (for histograms/sampled functions)
    delx: f32,
}

impl Numa {
    /// Create a new empty Numa
    ///
    /// The parameters `startx` and `delx` are initialized to 0.0 and 1.0
    /// respectively.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            startx: 0.0,
            delx: 1.0,
        }
    }

    /// Create a Numa with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            startx: 0.0,
            delx: 1.0,
        }
    }

    /// Create a Numa from a vector of values
    pub fn from_vec(data: Vec<f32>) -> Self {
        Self {
            data,
            startx: 0.0,
            delx: 1.0,
        }
    }

    /// Create a Numa from a slice of f32 values
    pub fn from_slice(data: &[f32]) -> Self {
        Self {
            data: data.to_vec(),
            startx: 0.0,
            delx: 1.0,
        }
    }

    /// Create a Numa from a slice of i32 values
    ///
    /// Each integer is converted to f32.
    pub fn from_i32_slice(data: &[i32]) -> Self {
        Self {
            data: data.iter().map(|&v| v as f32).collect(),
            startx: 0.0,
            delx: 1.0,
        }
    }

    /// Get the number of values
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get a value by index
    pub fn get(&self, index: usize) -> Option<f32> {
        self.data.get(index).copied()
    }

    /// Get a value as i32 (rounded)
    ///
    /// The value is rounded to the nearest integer using standard rounding.
    pub fn get_i32(&self, index: usize) -> Option<i32> {
        self.data.get(index).map(|&v| v.round() as i32)
    }

    /// Add a value to the end
    pub fn push(&mut self, val: f32) {
        self.data.push(val);
    }

    /// Remove and return the last value
    pub fn pop(&mut self) -> Option<f32> {
        self.data.pop()
    }

    /// Set a value at index
    pub fn set(&mut self, index: usize, val: f32) -> Result<()> {
        if index >= self.data.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.data.len(),
            });
        }
        self.data[index] = val;
        Ok(())
    }

    /// Add a delta to the value at index
    pub fn shift(&mut self, index: usize, delta: f32) -> Result<()> {
        if index >= self.data.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.data.len(),
            });
        }
        self.data[index] += delta;
        Ok(())
    }

    /// Insert a value at index
    pub fn insert(&mut self, index: usize, val: f32) -> Result<()> {
        if index > self.data.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.data.len(),
            });
        }
        self.data.insert(index, val);
        Ok(())
    }

    /// Remove a value at index
    pub fn remove(&mut self, index: usize) -> Result<f32> {
        if index >= self.data.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.data.len(),
            });
        }
        Ok(self.data.remove(index))
    }

    /// Replace a value at index, returning the old value
    pub fn replace(&mut self, index: usize, val: f32) -> Result<f32> {
        if index >= self.data.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.data.len(),
            });
        }
        Ok(std::mem::replace(&mut self.data[index], val))
    }

    /// Clear all values
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get the underlying data as a slice
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    /// Get the underlying data as a mutable slice
    pub fn as_slice_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Get the underlying vector (consuming self)
    pub fn into_vec(self) -> Vec<f32> {
        self.data
    }

    /// Get the parameters (startx, delx)
    ///
    /// These parameters are used when the Numa represents a sampled
    /// function or histogram. `startx` is the x-value corresponding
    /// to index 0, and `delx` is the spacing between consecutive
    /// x-values.
    pub fn parameters(&self) -> (f32, f32) {
        (self.startx, self.delx)
    }

    /// Set the parameters (startx, delx)
    pub fn set_parameters(&mut self, startx: f32, delx: f32) {
        self.startx = startx;
        self.delx = delx;
    }

    /// Get the x-value for a given index
    ///
    /// Returns `startx + index * delx`
    pub fn x_value(&self, index: usize) -> f32 {
        self.startx + (index as f32) * self.delx
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get the minimum value and its index
    ///
    /// Returns `None` if the array is empty.
    pub fn min(&self) -> Option<(f32, usize)> {
        if self.data.is_empty() {
            return None;
        }

        let mut min_val = f32::MAX;
        let mut min_idx = 0;

        for (i, &val) in self.data.iter().enumerate() {
            if val < min_val {
                min_val = val;
                min_idx = i;
            }
        }

        Some((min_val, min_idx))
    }

    /// Get the minimum value only
    pub fn min_value(&self) -> Option<f32> {
        self.min().map(|(v, _)| v)
    }

    /// Get the maximum value and its index
    ///
    /// Returns `None` if the array is empty.
    pub fn max(&self) -> Option<(f32, usize)> {
        if self.data.is_empty() {
            return None;
        }

        let mut max_val = f32::MIN;
        let mut max_idx = 0;

        for (i, &val) in self.data.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = i;
            }
        }

        Some((max_val, max_idx))
    }

    /// Get the maximum value only
    pub fn max_value(&self) -> Option<f32> {
        self.max().map(|(v, _)| v)
    }

    /// Get the sum of all values
    ///
    /// Returns `None` if the array is empty.
    pub fn sum(&self) -> Option<f32> {
        if self.data.is_empty() {
            return None;
        }
        Some(self.data.iter().sum())
    }

    /// Get the mean (average) of all values
    ///
    /// Returns `None` if the array is empty.
    pub fn mean(&self) -> Option<f32> {
        if self.data.is_empty() {
            return None;
        }
        Some(self.data.iter().sum::<f32>() / self.data.len() as f32)
    }

    /// Get the sum of values in a range [first, last]
    ///
    /// Both indices are inclusive. Returns `None` if the range is invalid.
    pub fn sum_on_interval(&self, first: usize, last: usize) -> Option<f32> {
        if self.data.is_empty() || first > last || first >= self.data.len() {
            return None;
        }
        let last = last.min(self.data.len() - 1);
        Some(self.data[first..=last].iter().sum())
    }

    /// Get the mean of absolute values
    pub fn mean_absval(&self) -> Option<f32> {
        if self.data.is_empty() {
            return None;
        }
        let sum: f32 = self.data.iter().map(|v| v.abs()).sum();
        Some(sum / self.data.len() as f32)
    }

    /// Check if all values are integers (within tolerance)
    pub fn has_only_integers(&self, tolerance: f32) -> bool {
        self.data
            .iter()
            .all(|&v| (v - v.round()).abs() <= tolerance)
    }

    // ========================================================================
    // Iterator
    // ========================================================================

    /// Create an iterator over values
    pub fn iter(&self) -> NumaIter<'_> {
        NumaIter {
            numa: self,
            index: 0,
        }
    }

    /// Create a mutable iterator over values
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.data.iter_mut()
    }
}

/// Iterator over Numa values
pub struct NumaIter<'a> {
    numa: &'a Numa,
    index: usize,
}

impl<'a> Iterator for NumaIter<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.numa.len() {
            let val = self.numa.data[self.index];
            self.index += 1;
            Some(val)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.numa.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for NumaIter<'_> {}

impl<'a> IntoIterator for &'a Numa {
    type Item = f32;
    type IntoIter = NumaIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Numa {
    type Item = f32;
    type IntoIter = std::vec::IntoIter<f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl FromIterator<f32> for Numa {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self {
            data: iter.into_iter().collect(),
            startx: 0.0,
            delx: 1.0,
        }
    }
}

impl std::ops::Index<usize> for Numa {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl std::ops::IndexMut<usize> for Numa {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

// ============================================================================
// Numaa - Array of Numa
// ============================================================================

/// Array of Numa
///
/// `Numaa` manages a collection of `Numa` arrays, useful for storing
/// related sets of numeric data.
#[derive(Debug, Clone, Default)]
pub struct Numaa {
    numas: Vec<Numa>,
}

impl Numaa {
    /// Create a new empty Numaa
    pub fn new() -> Self {
        Self { numas: Vec::new() }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            numas: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of Numa
    #[inline]
    pub fn len(&self) -> usize {
        self.numas.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.numas.is_empty()
    }

    /// Get a Numa by index
    pub fn get(&self, index: usize) -> Option<&Numa> {
        self.numas.get(index)
    }

    /// Get a mutable Numa by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Numa> {
        self.numas.get_mut(index)
    }

    /// Add a Numa
    pub fn push(&mut self, numa: Numa) {
        self.numas.push(numa);
    }

    /// Remove and return the last Numa
    pub fn pop(&mut self) -> Option<Numa> {
        self.numas.pop()
    }

    /// Replace a Numa at index
    pub fn replace(&mut self, index: usize, numa: Numa) -> Result<Numa> {
        if index >= self.numas.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.numas.len(),
            });
        }
        Ok(std::mem::replace(&mut self.numas[index], numa))
    }

    /// Clear all Numa
    pub fn clear(&mut self) {
        self.numas.clear();
    }

    /// Get all Numa as a slice
    pub fn numas(&self) -> &[Numa] {
        &self.numas
    }

    /// Get the total number of values across all Numa
    pub fn total_count(&self) -> usize {
        self.numas.iter().map(|n| n.len()).sum()
    }

    /// Get a value from a specific Numa
    ///
    /// Convenience method for accessing `numaa[numa_index][value_index]`.
    pub fn get_value(&self, numa_index: usize, value_index: usize) -> Option<f32> {
        self.numas.get(numa_index)?.get(value_index)
    }

    /// Add a value to a specific Numa
    ///
    /// Returns an error if `numa_index` is out of bounds.
    pub fn add_value(&mut self, numa_index: usize, val: f32) -> Result<()> {
        let len = self.numas.len();
        let numa = self
            .numas
            .get_mut(numa_index)
            .ok_or(Error::IndexOutOfBounds {
                index: numa_index,
                len,
            })?;
        numa.push(val);
        Ok(())
    }

    /// Flatten into a single Numa
    ///
    /// Concatenates all values from all Numa arrays. The parameters
    /// (startx, delx) of the result are set to defaults (0.0, 1.0).
    pub fn flatten(&self) -> Numa {
        let total = self.total_count();
        let mut result = Numa::with_capacity(total);
        for numa in &self.numas {
            for val in numa.iter() {
                result.push(val);
            }
        }
        result
    }

    /// Create an iterator over Numa
    pub fn iter(&self) -> impl Iterator<Item = &Numa> {
        self.numas.iter()
    }

    /// Create a mutable iterator over Numa
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Numa> {
        self.numas.iter_mut()
    }
}

impl std::ops::Index<usize> for Numaa {
    type Output = Numa;

    fn index(&self, index: usize) -> &Self::Output {
        &self.numas[index]
    }
}

impl std::ops::IndexMut<usize> for Numaa {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.numas[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Numa tests
    // ========================================================================

    #[test]
    fn test_numa_creation() {
        let numa = Numa::new();
        assert!(numa.is_empty());
        assert_eq!(numa.len(), 0);

        let numa = Numa::with_capacity(10);
        assert!(numa.is_empty());

        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        assert_eq!(numa.len(), 3);

        let numa = Numa::from_slice(&[4.0, 5.0, 6.0]);
        assert_eq!(numa.len(), 3);

        let numa = Numa::from_i32_slice(&[1, 2, 3]);
        assert_eq!(numa.len(), 3);
        assert_eq!(numa.get(0), Some(1.0));
    }

    #[test]
    fn test_numa_push_and_get() {
        let mut numa = Numa::new();
        numa.push(1.5);
        numa.push(2.5);
        numa.push(3.5);

        assert_eq!(numa.len(), 3);
        assert_eq!(numa.get(0), Some(1.5));
        assert_eq!(numa.get(1), Some(2.5));
        assert_eq!(numa.get(2), Some(3.5));
        assert_eq!(numa.get(3), None);
    }

    #[test]
    fn test_numa_get_i32() {
        let numa = Numa::from_vec(vec![1.4, 2.5, 3.6, -1.5]);

        assert_eq!(numa.get_i32(0), Some(1)); // 1.4 rounds to 1
        assert_eq!(numa.get_i32(1), Some(3)); // 2.5 rounds to 3 (round half to even might differ)
        assert_eq!(numa.get_i32(2), Some(4)); // 3.6 rounds to 4
        assert_eq!(numa.get_i32(3), Some(-2)); // -1.5 rounds to -2
    }

    #[test]
    fn test_numa_set() {
        let mut numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);

        numa.set(1, 10.0).unwrap();
        assert_eq!(numa.get(1), Some(10.0));

        assert!(numa.set(10, 5.0).is_err());
    }

    #[test]
    fn test_numa_shift() {
        let mut numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);

        numa.shift(1, 5.0).unwrap();
        assert_eq!(numa.get(1), Some(7.0));

        numa.shift(1, -2.0).unwrap();
        assert_eq!(numa.get(1), Some(5.0));

        assert!(numa.shift(10, 1.0).is_err());
    }

    #[test]
    fn test_numa_insert_remove() {
        let mut numa = Numa::from_vec(vec![1.0, 3.0]);

        numa.insert(1, 2.0).unwrap();
        assert_eq!(numa.as_slice(), &[1.0, 2.0, 3.0]);

        let removed = numa.remove(1).unwrap();
        assert_eq!(removed, 2.0);
        assert_eq!(numa.as_slice(), &[1.0, 3.0]);

        assert!(numa.insert(10, 5.0).is_err());
        assert!(numa.remove(10).is_err());
    }

    #[test]
    fn test_numa_pop_clear() {
        let mut numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);

        assert_eq!(numa.pop(), Some(3.0));
        assert_eq!(numa.len(), 2);

        numa.clear();
        assert!(numa.is_empty());
        assert_eq!(numa.pop(), None);
    }

    #[test]
    fn test_numa_parameters() {
        let mut numa = Numa::new();
        assert_eq!(numa.parameters(), (0.0, 1.0));

        numa.set_parameters(10.0, 0.5);
        assert_eq!(numa.parameters(), (10.0, 0.5));

        // Test x_value
        assert_eq!(numa.x_value(0), 10.0);
        assert_eq!(numa.x_value(1), 10.5);
        assert_eq!(numa.x_value(4), 12.0);
    }

    #[test]
    fn test_numa_min() {
        let numa = Numa::from_vec(vec![3.0, 1.0, 4.0, 1.0, 5.0]);
        let (min_val, min_idx) = numa.min().unwrap();
        assert_eq!(min_val, 1.0);
        assert_eq!(min_idx, 1); // First occurrence

        assert_eq!(numa.min_value(), Some(1.0));

        let empty = Numa::new();
        assert!(empty.min().is_none());
    }

    #[test]
    fn test_numa_max() {
        let numa = Numa::from_vec(vec![3.0, 1.0, 5.0, 1.0, 5.0]);
        let (max_val, max_idx) = numa.max().unwrap();
        assert_eq!(max_val, 5.0);
        assert_eq!(max_idx, 2); // First occurrence

        assert_eq!(numa.max_value(), Some(5.0));

        let empty = Numa::new();
        assert!(empty.max().is_none());
    }

    #[test]
    fn test_numa_sum() {
        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(numa.sum(), Some(15.0));

        let empty = Numa::new();
        assert!(empty.sum().is_none());
    }

    #[test]
    fn test_numa_mean() {
        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(numa.mean(), Some(3.0));

        let empty = Numa::new();
        assert!(empty.mean().is_none());
    }

    #[test]
    fn test_numa_sum_on_interval() {
        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        assert_eq!(numa.sum_on_interval(1, 3), Some(9.0)); // 2 + 3 + 4
        assert_eq!(numa.sum_on_interval(0, 4), Some(15.0)); // all
        assert_eq!(numa.sum_on_interval(3, 10), Some(9.0)); // 4 + 5 (clamped)

        assert!(numa.sum_on_interval(5, 6).is_none()); // out of range
        assert!(numa.sum_on_interval(3, 1).is_none()); // invalid range
    }

    #[test]
    fn test_numa_mean_absval() {
        let numa = Numa::from_vec(vec![-2.0, 1.0, -3.0, 4.0]);
        assert_eq!(numa.mean_absval(), Some(2.5)); // (2 + 1 + 3 + 4) / 4

        let empty = Numa::new();
        assert!(empty.mean_absval().is_none());
    }

    #[test]
    fn test_numa_has_only_integers() {
        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        assert!(numa.has_only_integers(0.0001));

        let numa = Numa::from_vec(vec![1.0, 2.5, 3.0]);
        assert!(!numa.has_only_integers(0.0001));

        let numa = Numa::from_vec(vec![1.0001, 2.0, 3.0]);
        assert!(!numa.has_only_integers(0.00001));
        assert!(numa.has_only_integers(0.001));
    }

    #[test]
    fn test_numa_iterator() {
        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);

        let collected: Vec<_> = numa.iter().collect();
        assert_eq!(collected, vec![1.0, 2.0, 3.0]);

        // Test for loop
        let mut sum = 0.0;
        for val in &numa {
            sum += val;
        }
        assert_eq!(sum, 6.0);
    }

    #[test]
    fn test_numa_into_iterator() {
        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);

        let collected: Vec<_> = numa.into_iter().collect();
        assert_eq!(collected, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_numa_from_iterator() {
        let numa: Numa = vec![1.0, 2.0, 3.0].into_iter().collect();
        assert_eq!(numa.len(), 3);
        assert_eq!(numa.as_slice(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_numa_indexing() {
        let mut numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);

        assert_eq!(numa[0], 1.0);
        assert_eq!(numa[1], 2.0);

        numa[1] = 10.0;
        assert_eq!(numa[1], 10.0);
    }

    #[test]
    fn test_numa_into_vec() {
        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        let vec = numa.into_vec();
        assert_eq!(vec, vec![1.0, 2.0, 3.0]);
    }

    // ========================================================================
    // Numaa tests
    // ========================================================================

    #[test]
    fn test_numaa_creation() {
        let numaa = Numaa::new();
        assert!(numaa.is_empty());
        assert_eq!(numaa.len(), 0);

        let numaa = Numaa::with_capacity(10);
        assert!(numaa.is_empty());
    }

    #[test]
    fn test_numaa_push_and_get() {
        let mut numaa = Numaa::new();

        numaa.push(Numa::from_vec(vec![1.0, 2.0]));
        numaa.push(Numa::from_vec(vec![3.0, 4.0, 5.0]));

        assert_eq!(numaa.len(), 2);
        assert_eq!(numaa.get(0).unwrap().len(), 2);
        assert_eq!(numaa.get(1).unwrap().len(), 3);
        assert!(numaa.get(2).is_none());
    }

    #[test]
    fn test_numaa_total_count() {
        let mut numaa = Numaa::new();

        numaa.push(Numa::from_vec(vec![1.0, 2.0]));
        numaa.push(Numa::from_vec(vec![3.0, 4.0, 5.0]));

        assert_eq!(numaa.total_count(), 5);
    }

    #[test]
    fn test_numaa_get_value() {
        let mut numaa = Numaa::new();

        numaa.push(Numa::from_vec(vec![1.0, 2.0]));
        numaa.push(Numa::from_vec(vec![3.0, 4.0, 5.0]));

        assert_eq!(numaa.get_value(0, 1), Some(2.0));
        assert_eq!(numaa.get_value(1, 2), Some(5.0));
        assert!(numaa.get_value(0, 5).is_none());
        assert!(numaa.get_value(5, 0).is_none());
    }

    #[test]
    fn test_numaa_add_value() {
        let mut numaa = Numaa::new();
        numaa.push(Numa::new());
        numaa.push(Numa::new());

        numaa.add_value(0, 1.0).unwrap();
        numaa.add_value(0, 2.0).unwrap();
        numaa.add_value(1, 3.0).unwrap();

        assert_eq!(numaa[0].len(), 2);
        assert_eq!(numaa[1].len(), 1);
        assert_eq!(numaa.get_value(0, 1), Some(2.0));

        assert!(numaa.add_value(5, 1.0).is_err());
    }

    #[test]
    fn test_numaa_flatten() {
        let mut numaa = Numaa::new();

        numaa.push(Numa::from_vec(vec![1.0, 2.0]));
        numaa.push(Numa::from_vec(vec![3.0, 4.0, 5.0]));

        let flat = numaa.flatten();
        assert_eq!(flat.len(), 5);
        assert_eq!(flat.as_slice(), &[1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_numaa_replace() {
        let mut numaa = Numaa::new();
        numaa.push(Numa::from_vec(vec![1.0, 2.0]));

        let old = numaa
            .replace(0, Numa::from_vec(vec![3.0, 4.0, 5.0]))
            .unwrap();
        assert_eq!(old.len(), 2);
        assert_eq!(numaa[0].len(), 3);

        assert!(numaa.replace(10, Numa::new()).is_err());
    }

    #[test]
    fn test_numaa_indexing() {
        let mut numaa = Numaa::new();
        numaa.push(Numa::from_vec(vec![1.0, 2.0]));

        assert_eq!(numaa[0].len(), 2);
        assert_eq!(numaa[0][0], 1.0);

        numaa[0][0] = 10.0;
        assert_eq!(numaa[0][0], 10.0);
    }

    #[test]
    fn test_numaa_pop_clear() {
        let mut numaa = Numaa::new();
        numaa.push(Numa::from_vec(vec![1.0]));
        numaa.push(Numa::from_vec(vec![2.0]));

        let popped = numaa.pop().unwrap();
        assert_eq!(popped[0], 2.0);
        assert_eq!(numaa.len(), 1);

        numaa.clear();
        assert!(numaa.is_empty());
    }
}
