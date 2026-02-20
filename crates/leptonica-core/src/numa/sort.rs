//! Sorting, binary search, random permutation, and median/deviation functions
//! for Numa arrays.
//!
//! Corresponds to sort-related functions in C Leptonica's `numafunc1.c`.

use super::{Numa, SortOrder};
use crate::error::{Error, Result};

impl Numa {
    /// Sort a Numa of non-negative integers using bin sort.
    ///
    /// This is faster than shell sort for large arrays of non-negative
    /// integers, but not appropriate for small arrays or very large values.
    ///
    /// C equivalent: `numaBinSort()` in `numafunc1.c`
    pub fn bin_sort(&self, order: SortOrder) -> Result<Numa> {
        if self.is_empty() {
            return Ok(self.clone());
        }
        let naindex = self.bin_sort_index(order)?;
        self.sort_by_index(&naindex)
    }

    /// Return the sort index for a Numa of non-negative integers using bin sort.
    ///
    /// Creates an index array that, when applied to the source array, produces
    /// a sorted array. Requires all values to be non-negative integers.
    ///
    /// C equivalent: `numaGetBinSortIndex()` in `numafunc1.c`
    pub fn bin_sort_index(&self, order: SortOrder) -> Result<Numa> {
        if self.is_empty() {
            return Ok(Numa::new());
        }
        let min_val = self.min_value().ok_or(Error::NullInput("empty Numa"))?;
        if min_val < 0.0 {
            return Err(Error::InvalidParameter(
                "bin_sort_index: values must be non-negative".to_string(),
            ));
        }
        let max_val = self.max_value().ok_or(Error::NullInput("empty Numa"))?;
        let imax = max_val as usize;

        // Create buckets indexed by value; each bucket holds original indices
        let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); imax + 1];
        for (i, &val) in self.as_slice().iter().enumerate() {
            buckets[val as usize].push(i);
        }

        let mut nad = Numa::new();
        match order {
            SortOrder::Increasing => {
                for bucket in &buckets {
                    for &idx in bucket {
                        nad.push(idx as f32);
                    }
                }
            }
            SortOrder::Decreasing => {
                for bucket in buckets.iter().rev() {
                    for &idx in bucket {
                        nad.push(idx as f32);
                    }
                }
            }
        }
        Ok(nad)
    }

    /// Sort two parallel Numa arrays together, using the first as the key.
    ///
    /// Returns `(sorted_nax, sorted_nay)` where `nay` is reordered to match
    /// the sort of `nax`.
    ///
    /// C equivalent: `numaSortPair()` in `numafunc1.c`
    pub fn sort_pair(&self, nay: &Numa, order: SortOrder) -> (Numa, Numa) {
        if self.is_sorted(order) {
            return (self.clone(), nay.clone());
        }
        let naindex = self.sort_index(order);
        let nasx = self
            .sort_by_index(&naindex)
            .unwrap_or_else(|_| self.clone());
        let nasy = nay.sort_by_index(&naindex).unwrap_or_else(|_| nay.clone());
        (nasx, nasy)
    }

    /// Invert a permutation index array.
    ///
    /// Requires that the array contains each integer from 0 to n-1 exactly
    /// once. Returns the inverse permutation.
    ///
    /// C equivalent: `numaInvertMap()` in `numafunc1.c`
    pub fn invert_map(&self) -> Result<Numa> {
        let n = self.len();
        if n == 0 {
            return Ok(self.clone());
        }
        let mut nad = Numa::from_vec(vec![0.0; n]);
        let mut seen = vec![false; n];
        for i in 0..n {
            let val = self.get_i32(i).ok_or(Error::NullInput("empty Numa"))? as usize;
            if val >= n || seen[val] {
                return Err(Error::InvalidParameter(
                    "invert_map: not a valid permutation".to_string(),
                ));
            }
            seen[val] = true;
            nad.set(val, i as f32)?;
        }
        Ok(nad)
    }

    /// Insert a value into a sorted Numa, maintaining sort order.
    ///
    /// C equivalent: `numaAddSorted()` in `numafunc1.c`
    pub fn add_sorted(&mut self, val: f32) -> Result<()> {
        let index = self.find_sorted_loc(val);
        self.insert(index, val)
    }

    /// Find the insertion location for a value in a sorted Numa.
    ///
    /// Returns the index at which `val` should be inserted to maintain
    /// the array's sort order. Uses binary search (O(log n)).
    ///
    /// C equivalent: `numaFindSortedLoc()` in `numafunc1.c`
    pub fn find_sorted_loc(&self, val: f32) -> usize {
        let n = self.len();
        if n == 0 {
            return 0;
        }
        let val0 = self.get(0).unwrap();
        if n == 1 {
            return if val >= val0 { 1 } else { 0 };
        }
        let valn = self.get(n - 1).unwrap();
        let increasing = valn >= val0;

        if increasing {
            if val < val0 {
                return 0;
            }
            if val > valn {
                return n;
            }
        } else {
            if val > val0 {
                return 0;
            }
            if val < valn {
                return n;
            }
        }

        let mut lindex = 0usize;
        let mut rindex = n - 1;
        loop {
            let midindex = (lindex + rindex) / 2;
            if midindex == lindex || midindex == rindex {
                break;
            }
            let valmid = self.get(midindex).unwrap();
            if increasing {
                if val > valmid {
                    lindex = midindex;
                } else {
                    rindex = midindex;
                }
            } else if val > valmid {
                rindex = midindex;
            } else {
                lindex = midindex;
            }
        }
        rindex
    }

    /// Generate a pseudorandom permutation of integers 0..size.
    ///
    /// Uses the Fisher-Yates (Durstenfeld) shuffle with a deterministic
    /// LCG seeded by `seed`.
    ///
    /// C equivalent: `numaPseudorandomSequence()` in `numafunc1.c`
    pub fn pseudorandom_sequence(size: usize, seed: u64) -> Numa {
        if size == 0 {
            return Numa::new();
        }
        let mut array: Vec<usize> = (0..size).collect();
        let mut rng = seed;
        for i in (1..size).rev() {
            // LCG step
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = (rng as usize) % (i + 1);
            array.swap(i, j);
        }
        Numa::from_vec(array.iter().map(|&x| x as f32).collect())
    }

    /// Randomly permute the elements of this Numa using a deterministic seed.
    ///
    /// C equivalent: `numaRandomPermutation()` in `numafunc1.c`
    pub fn random_permutation(&self, seed: u64) -> Numa {
        let size = self.len();
        if size == 0 {
            return self.clone();
        }
        let naindex = Numa::pseudorandom_sequence(size, seed);
        let mut nad = Numa::new();
        for i in 0..size {
            let index = naindex.get_i32(i).unwrap() as usize;
            nad.push(self.get(index).unwrap());
        }
        nad
    }

    /// Compute the median value rounded to the nearest integer.
    ///
    /// Uses `rank_value(0.5)` and rounds the result.
    ///
    /// C equivalent: `numaGetBinnedMedian()` in `numafunc1.c`
    pub fn binned_median(&self) -> Result<i32> {
        let fval = self.rank_value(0.5)?;
        Ok(fval.round() as i32)
    }

    /// Compute the mean absolute deviation from a given median value.
    ///
    /// C equivalent: `numaGetMeanDevFromMedian()` in `numafunc1.c`
    pub fn mean_dev_from_median(&self, med: f32) -> Result<f32> {
        if self.is_empty() {
            return Err(Error::NullInput("empty Numa"));
        }
        let n = self.len();
        let dev: f32 = self
            .as_slice()
            .iter()
            .map(|&v| (v - med).abs())
            .sum::<f32>()
            / n as f32;
        Ok(dev)
    }

    /// Compute the median and median absolute deviation from the median.
    ///
    /// Returns `(median, median_dev)`.
    ///
    /// C equivalent: `numaGetMedianDevFromMedian()` in `numafunc1.c`
    pub fn median_dev_from_median(&self) -> Result<(f32, f32)> {
        if self.is_empty() {
            return Err(Error::NullInput("empty Numa"));
        }
        let med = self.median()?;
        let mut nadev = Numa::new();
        for &val in self.as_slice() {
            nadev.push((val - med).abs());
        }
        let dev = nadev.median()?;
        Ok((med, dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- bin_sort / bin_sort_index --

    #[test]
    fn test_bin_sort_increasing() {
        let na = Numa::from_slice(&[3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0]);
        let sorted = na.bin_sort(SortOrder::Increasing).unwrap();
        assert_eq!(sorted.as_slice(), &[1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 9.0]);
    }

    #[test]
    fn test_bin_sort_decreasing() {
        let na = Numa::from_slice(&[3.0, 1.0, 4.0, 1.0, 5.0]);
        let sorted = na.bin_sort(SortOrder::Decreasing).unwrap();
        assert_eq!(sorted.as_slice(), &[5.0, 4.0, 3.0, 1.0, 1.0]);
    }

    #[test]
    fn test_bin_sort_index_order() {
        // [3, 1, 4] increasing: sorted is [1, 3, 4]
        // index: 1, 0, 2
        let na = Numa::from_slice(&[3.0, 1.0, 4.0]);
        let idx = na.bin_sort_index(SortOrder::Increasing).unwrap();
        // index at position 0 is 1 (value 1.0), at position 1 is 0 (value 3.0), at position 2 is 2 (value 4.0)
        assert_eq!(idx.get_i32(0).unwrap(), 1);
        assert_eq!(idx.get_i32(1).unwrap(), 0);
        assert_eq!(idx.get_i32(2).unwrap(), 2);
    }

    #[test]
    fn test_bin_sort_negative_error() {
        let na = Numa::from_slice(&[3.0, -1.0, 4.0]);
        assert!(na.bin_sort(SortOrder::Increasing).is_err());
    }

    // -- sort_pair --

    #[test]
    fn test_sort_pair_increasing() {
        let nax = Numa::from_slice(&[3.0, 1.0, 2.0]);
        let nay = Numa::from_slice(&[30.0, 10.0, 20.0]);
        let (sx, sy) = nax.sort_pair(&nay, SortOrder::Increasing);
        assert_eq!(sx.as_slice(), &[1.0, 2.0, 3.0]);
        assert_eq!(sy.as_slice(), &[10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_sort_pair_already_sorted() {
        let nax = Numa::from_slice(&[1.0, 2.0, 3.0]);
        let nay = Numa::from_slice(&[10.0, 20.0, 30.0]);
        let (sx, sy) = nax.sort_pair(&nay, SortOrder::Increasing);
        assert_eq!(sx.as_slice(), &[1.0, 2.0, 3.0]);
        assert_eq!(sy.as_slice(), &[10.0, 20.0, 30.0]);
    }

    // -- invert_map --

    #[test]
    fn test_invert_map_basic() {
        // Permutation [2, 0, 1] should invert to [1, 2, 0]
        // nas[0]=2 → nad[2]=0; nas[1]=0 → nad[0]=1; nas[2]=1 → nad[1]=2
        let nas = Numa::from_slice(&[2.0, 0.0, 1.0]);
        let inv = nas.invert_map().unwrap();
        assert_eq!(inv.get_i32(0).unwrap(), 1);
        assert_eq!(inv.get_i32(1).unwrap(), 2);
        assert_eq!(inv.get_i32(2).unwrap(), 0);
    }

    #[test]
    fn test_invert_map_identity() {
        let nas = Numa::from_slice(&[0.0, 1.0, 2.0, 3.0]);
        let inv = nas.invert_map().unwrap();
        assert_eq!(inv.get_i32(0).unwrap(), 0);
        assert_eq!(inv.get_i32(1).unwrap(), 1);
        assert_eq!(inv.get_i32(2).unwrap(), 2);
        assert_eq!(inv.get_i32(3).unwrap(), 3);
    }

    #[test]
    fn test_invert_map_duplicate_error() {
        let nas = Numa::from_slice(&[0.0, 0.0, 1.0]); // duplicate 0
        assert!(nas.invert_map().is_err());
    }

    // -- find_sorted_loc / add_sorted --

    #[test]
    fn test_find_sorted_loc_increasing() {
        let na = Numa::from_slice(&[1.0, 3.0, 5.0, 7.0]);
        assert_eq!(na.find_sorted_loc(0.0), 0); // before all
        assert_eq!(na.find_sorted_loc(4.0), 2); // between 3 and 5
        assert_eq!(na.find_sorted_loc(8.0), 4); // after all
        assert_eq!(na.find_sorted_loc(3.0), 1); // before existing equal value
    }

    #[test]
    fn test_add_sorted_increasing() {
        let mut na = Numa::from_slice(&[1.0, 3.0, 5.0]);
        na.add_sorted(4.0).unwrap();
        assert_eq!(na.as_slice(), &[1.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_add_sorted_at_start() {
        let mut na = Numa::from_slice(&[2.0, 4.0, 6.0]);
        na.add_sorted(0.0).unwrap();
        assert_eq!(na.as_slice(), &[0.0, 2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_add_sorted_at_end() {
        let mut na = Numa::from_slice(&[1.0, 2.0, 3.0]);
        na.add_sorted(10.0).unwrap();
        assert_eq!(na.as_slice(), &[1.0, 2.0, 3.0, 10.0]);
    }

    // -- pseudorandom_sequence / random_permutation --

    #[test]
    fn test_pseudorandom_sequence_length() {
        let na = Numa::pseudorandom_sequence(10, 42);
        assert_eq!(na.len(), 10);
    }

    #[test]
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
    fn test_pseudorandom_sequence_deterministic() {
        let a = Numa::pseudorandom_sequence(10, 42);
        let b = Numa::pseudorandom_sequence(10, 42);
        assert_eq!(a.as_slice(), b.as_slice());
    }

    #[test]
    fn test_pseudorandom_sequence_different_seeds() {
        let a = Numa::pseudorandom_sequence(20, 1);
        let b = Numa::pseudorandom_sequence(20, 2);
        assert_ne!(a.as_slice(), b.as_slice());
    }

    #[test]
    fn test_random_permutation_length() {
        let na = Numa::from_slice(&[10.0, 20.0, 30.0, 40.0]);
        let perm = na.random_permutation(7);
        assert_eq!(perm.len(), 4);
    }

    #[test]
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
    fn test_binned_median_odd() {
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(na.binned_median().unwrap(), 3);
    }

    #[test]
    fn test_binned_median_even() {
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0]);
        // median = (2+3)/2 = 2.5, rounded = 3
        let m = na.binned_median().unwrap();
        assert!(m == 2 || m == 3, "expected 2 or 3, got {m}");
    }

    #[test]
    fn test_binned_median_empty_error() {
        let na = Numa::new();
        assert!(na.binned_median().is_err());
    }

    // -- mean_dev_from_median --

    #[test]
    fn test_mean_dev_from_median_basic() {
        // values [1, 2, 3, 4, 5], med = 3
        // deviations: 2, 1, 0, 1, 2 → mean = 6/5 = 1.2
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let dev = na.mean_dev_from_median(3.0).unwrap();
        assert!((dev - 1.2).abs() < 1e-5, "expected 1.2, got {dev}");
    }

    #[test]
    fn test_mean_dev_from_median_empty_error() {
        let na = Numa::new();
        assert!(na.mean_dev_from_median(0.0).is_err());
    }

    // -- median_dev_from_median --

    #[test]
    fn test_median_dev_from_median_basic() {
        // values [1, 2, 3, 4, 5], median = 3
        // abs deviations: [2, 1, 0, 1, 2], median of those = 1
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let (med, dev) = na.median_dev_from_median().unwrap();
        assert!((med - 3.0).abs() < 1e-5, "expected median=3.0, got {med}");
        assert!((dev - 1.0).abs() < 1e-5, "expected dev=1.0, got {dev}");
    }

    #[test]
    fn test_median_dev_from_median_empty_error() {
        let na = Numa::new();
        assert!(na.median_dev_from_median().is_err());
    }
}
