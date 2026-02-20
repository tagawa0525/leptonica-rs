//! Sorting, binary search, random permutation, and median/deviation functions
//! for Numa arrays.
//!
//! Corresponds to sort-related functions in C Leptonica's `numafunc1.c`.

use super::{Numa, SortOrder};
use crate::error::{Error, Result};

impl Numa {
    /// Sort a Numa of non-negative integers using bin sort.
    ///
    /// C equivalent: `numaBinSort()` in `numafunc1.c`
    pub fn bin_sort(&self, order: SortOrder) -> Result<Numa> {
        let _ = order;
        todo!("Phase 16.2 GREEN")
    }

    /// Return the sort index for a Numa of non-negative integers using bin sort.
    ///
    /// C equivalent: `numaGetBinSortIndex()` in `numafunc1.c`
    pub fn bin_sort_index(&self, order: SortOrder) -> Result<Numa> {
        let _ = order;
        todo!("Phase 16.2 GREEN")
    }

    /// Sort two parallel Numa arrays together, using the first as the key.
    ///
    /// Returns `(sorted_nax, sorted_nay)`.
    ///
    /// C equivalent: `numaSortPair()` in `numafunc1.c`
    pub fn sort_pair(&self, nay: &Numa, order: SortOrder) -> (Numa, Numa) {
        let _ = (nay, order);
        todo!("Phase 16.2 GREEN")
    }

    /// Invert a permutation index array.
    ///
    /// C equivalent: `numaInvertMap()` in `numafunc1.c`
    pub fn invert_map(&self) -> Result<Numa> {
        todo!("Phase 16.2 GREEN")
    }

    /// Insert a value into a sorted Numa, maintaining sort order.
    ///
    /// C equivalent: `numaAddSorted()` in `numafunc1.c`
    pub fn add_sorted(&mut self, val: f32) -> Result<()> {
        let _ = val;
        todo!("Phase 16.2 GREEN")
    }

    /// Find the insertion location for a value in a sorted Numa.
    ///
    /// C equivalent: `numaFindSortedLoc()` in `numafunc1.c`
    pub fn find_sorted_loc(&self, val: f32) -> usize {
        let _ = val;
        todo!("Phase 16.2 GREEN")
    }

    /// Generate a pseudorandom permutation of integers 0..size.
    ///
    /// C equivalent: `numaPseudorandomSequence()` in `numafunc1.c`
    pub fn pseudorandom_sequence(size: usize, seed: u64) -> Numa {
        let _ = (size, seed);
        todo!("Phase 16.2 GREEN")
    }

    /// Randomly permute the elements of this Numa using a deterministic seed.
    ///
    /// C equivalent: `numaRandomPermutation()` in `numafunc1.c`
    pub fn random_permutation(&self, seed: u64) -> Numa {
        let _ = seed;
        todo!("Phase 16.2 GREEN")
    }

    /// Compute the median value rounded to the nearest integer.
    ///
    /// C equivalent: `numaGetBinnedMedian()` in `numafunc1.c`
    pub fn binned_median(&self) -> Result<i32> {
        todo!("Phase 16.2 GREEN")
    }

    /// Compute the mean absolute deviation from a given median value.
    ///
    /// C equivalent: `numaGetMeanDevFromMedian()` in `numafunc1.c`
    pub fn mean_dev_from_median(&self, med: f32) -> Result<f32> {
        let _ = med;
        todo!("Phase 16.2 GREEN")
    }

    /// Compute the median and median absolute deviation from the median.
    ///
    /// Returns `(median, median_dev)`.
    ///
    /// C equivalent: `numaGetMedianDevFromMedian()` in `numafunc1.c`
    pub fn median_dev_from_median(&self) -> Result<(f32, f32)> {
        todo!("Phase 16.2 GREEN")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- bin_sort / bin_sort_index --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_bin_sort_increasing() {
        let na = Numa::from_slice(&[3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0]);
        let sorted = na.bin_sort(SortOrder::Increasing).unwrap();
        assert_eq!(sorted.as_slice(), &[1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 9.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_bin_sort_decreasing() {
        let na = Numa::from_slice(&[3.0, 1.0, 4.0, 1.0, 5.0]);
        let sorted = na.bin_sort(SortOrder::Decreasing).unwrap();
        assert_eq!(sorted.as_slice(), &[5.0, 4.0, 3.0, 1.0, 1.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_bin_sort_index_order() {
        // [3, 1, 4] increasing: sorted is [1, 3, 4]
        // index: 1, 0, 2
        let na = Numa::from_slice(&[3.0, 1.0, 4.0]);
        let idx = na.bin_sort_index(SortOrder::Increasing).unwrap();
        assert_eq!(idx.get_i32(0).unwrap(), 1);
        assert_eq!(idx.get_i32(1).unwrap(), 0);
        assert_eq!(idx.get_i32(2).unwrap(), 2);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_bin_sort_negative_error() {
        let na = Numa::from_slice(&[3.0, -1.0, 4.0]);
        assert!(na.bin_sort(SortOrder::Increasing).is_err());
    }

    // -- sort_pair --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sort_pair_increasing() {
        let nax = Numa::from_slice(&[3.0, 1.0, 2.0]);
        let nay = Numa::from_slice(&[30.0, 10.0, 20.0]);
        let (sx, sy) = nax.sort_pair(&nay, SortOrder::Increasing);
        assert_eq!(sx.as_slice(), &[1.0, 2.0, 3.0]);
        assert_eq!(sy.as_slice(), &[10.0, 20.0, 30.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sort_pair_already_sorted() {
        let nax = Numa::from_slice(&[1.0, 2.0, 3.0]);
        let nay = Numa::from_slice(&[10.0, 20.0, 30.0]);
        let (sx, sy) = nax.sort_pair(&nay, SortOrder::Increasing);
        assert_eq!(sx.as_slice(), &[1.0, 2.0, 3.0]);
        assert_eq!(sy.as_slice(), &[10.0, 20.0, 30.0]);
    }

    // -- invert_map --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_invert_map_basic() {
        let nas = Numa::from_slice(&[2.0, 0.0, 1.0]);
        let inv = nas.invert_map().unwrap();
        assert_eq!(inv.get_i32(0).unwrap(), 1);
        assert_eq!(inv.get_i32(1).unwrap(), 2);
        assert_eq!(inv.get_i32(2).unwrap(), 0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_invert_map_identity() {
        let nas = Numa::from_slice(&[0.0, 1.0, 2.0, 3.0]);
        let inv = nas.invert_map().unwrap();
        assert_eq!(inv.get_i32(0).unwrap(), 0);
        assert_eq!(inv.get_i32(1).unwrap(), 1);
        assert_eq!(inv.get_i32(2).unwrap(), 2);
        assert_eq!(inv.get_i32(3).unwrap(), 3);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_invert_map_duplicate_error() {
        let nas = Numa::from_slice(&[0.0, 0.0, 1.0]);
        assert!(nas.invert_map().is_err());
    }

    // -- find_sorted_loc / add_sorted --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_find_sorted_loc_increasing() {
        let na = Numa::from_slice(&[1.0, 3.0, 5.0, 7.0]);
        assert_eq!(na.find_sorted_loc(0.0), 0);
        assert_eq!(na.find_sorted_loc(4.0), 2);
        assert_eq!(na.find_sorted_loc(8.0), 4);
        assert_eq!(na.find_sorted_loc(3.0), 1); // before existing equal value
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_sorted_increasing() {
        let mut na = Numa::from_slice(&[1.0, 3.0, 5.0]);
        na.add_sorted(4.0).unwrap();
        assert_eq!(na.as_slice(), &[1.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_sorted_at_start() {
        let mut na = Numa::from_slice(&[2.0, 4.0, 6.0]);
        na.add_sorted(0.0).unwrap();
        assert_eq!(na.as_slice(), &[0.0, 2.0, 4.0, 6.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_sorted_at_end() {
        let mut na = Numa::from_slice(&[1.0, 2.0, 3.0]);
        na.add_sorted(10.0).unwrap();
        assert_eq!(na.as_slice(), &[1.0, 2.0, 3.0, 10.0]);
    }

    // -- pseudorandom_sequence / random_permutation --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_pseudorandom_sequence_length() {
        let na = Numa::pseudorandom_sequence(10, 42);
        assert_eq!(na.len(), 10);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_pseudorandom_sequence_is_permutation() {
        let size = 20;
        let na = Numa::pseudorandom_sequence(size, 42);
        let mut counts = vec![0usize; size];
        for i in 0..size {
            let v = na.get_i32(i).unwrap() as usize;
            assert!(v < size);
            counts[v] += 1;
        }
        assert!(counts.iter().all(|&c| c == 1), "not a valid permutation");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_pseudorandom_sequence_deterministic() {
        let a = Numa::pseudorandom_sequence(10, 42);
        let b = Numa::pseudorandom_sequence(10, 42);
        assert_eq!(a.as_slice(), b.as_slice());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_pseudorandom_sequence_different_seeds() {
        let a = Numa::pseudorandom_sequence(20, 1);
        let b = Numa::pseudorandom_sequence(20, 2);
        assert_ne!(a.as_slice(), b.as_slice());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_random_permutation_length() {
        let na = Numa::from_slice(&[10.0, 20.0, 30.0, 40.0]);
        let perm = na.random_permutation(7);
        assert_eq!(perm.len(), 4);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_random_permutation_same_values() {
        let na = Numa::from_slice(&[10.0, 20.0, 30.0, 40.0]);
        let perm = na.random_permutation(7);
        let mut orig: Vec<f32> = na.as_slice().to_vec();
        let mut perm_sorted: Vec<f32> = perm.as_slice().to_vec();
        orig.sort_by(|a, b| a.partial_cmp(b).unwrap());
        perm_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(orig, perm_sorted);
    }

    // -- binned_median --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_binned_median_odd() {
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(na.binned_median().unwrap(), 3);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_binned_median_even() {
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0]);
        let m = na.binned_median().unwrap();
        assert!(m == 2 || m == 3, "expected 2 or 3, got {m}");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_binned_median_empty_error() {
        let na = Numa::new();
        assert!(na.binned_median().is_err());
    }

    // -- mean_dev_from_median --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_mean_dev_from_median_basic() {
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let dev = na.mean_dev_from_median(3.0).unwrap();
        assert!((dev - 1.2).abs() < 1e-5, "expected 1.2, got {dev}");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_mean_dev_from_median_empty_error() {
        let na = Numa::new();
        assert!(na.mean_dev_from_median(0.0).is_err());
    }

    // -- median_dev_from_median --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_median_dev_from_median_basic() {
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let (med, dev) = na.median_dev_from_median().unwrap();
        assert!((med - 3.0).abs() < 1e-5, "expected median=3.0, got {med}");
        assert!((dev - 1.0).abs() < 1e-5, "expected dev=1.0, got {dev}");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_median_dev_from_median_empty_error() {
        let na = Numa::new();
        assert!(na.median_dev_from_median().is_err());
    }
}
