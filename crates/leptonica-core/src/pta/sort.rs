//! Pta sorting and statistical functions.
//!
//! Corresponds to functions in C Leptonica's `ptafunc2.c`.

use crate::error::{Error, Result};
use crate::numa::{Numa, SortOrder};
use crate::pta::{Pta, Ptaa};

/// Which coordinate to sort by
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    /// Sort by x coordinate
    X,
    /// Sort by y coordinate
    Y,
}

impl Pta {
    /// Return a sorted copy of `self`, optionally also returning the index Numa.
    ///
    /// C equivalent: `ptaSort()` in `ptafunc2.c`
    pub fn sort_pta(&self, by: SortBy, order: SortOrder) -> Result<(Pta, Numa)> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return the index array that would sort `self`.
    ///
    /// C equivalent: `ptaGetSortIndex()` in `ptafunc2.c`
    pub fn get_sort_index(&self, by: SortBy, order: SortOrder) -> Result<Numa> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return a new Pta sorted according to an index Numa.
    ///
    /// C equivalent: `ptaSortByIndex()` in `ptafunc2.c`
    pub fn sort_by_index(&self, naindex: &Numa) -> Result<Pta> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return the x or y value at fractile `fract` (0.0 = min, 1.0 = max).
    ///
    /// C equivalent: `ptaGetRankValue()` in `ptafunc2.c`
    pub fn get_rank_value(&self, fract: f32, by: SortBy) -> Result<f32> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return a copy sorted first by x, then by y.
    ///
    /// C equivalent: `ptaSort2d()` in `ptafunc2.c`
    pub fn sort_2d(&self) -> Pta {
        todo!("Phase 16.3 GREEN")
    }

    /// Return `true` if `self` and `other` have the same points.
    ///
    /// C equivalent: `ptaEqual()` in `ptafunc2.c`
    pub fn equal(&self, other: &Pta) -> bool {
        todo!("Phase 16.3 GREEN")
    }
}

impl Ptaa {
    /// Return a new Ptaa sorted according to an index Numa.
    ///
    /// C equivalent: `ptaaSortByIndex()` in `ptafunc2.c`
    pub fn sort_by_index(&self, naindex: &Numa) -> Result<Ptaa> {
        todo!("Phase 16.3 GREEN")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pts() -> Pta {
        let mut p = Pta::new();
        p.push(3.0, 1.0);
        p.push(1.0, 4.0);
        p.push(2.0, 2.0);
        p
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sort_pta_by_x_increasing() {
        let p = make_pts();
        let (sorted, _) = p.sort_pta(SortBy::X, SortOrder::Increasing).unwrap();
        assert_eq!(sorted.get(0).map(|(x, _)| x), Some(1.0));
        assert_eq!(sorted.get(1).map(|(x, _)| x), Some(2.0));
        assert_eq!(sorted.get(2).map(|(x, _)| x), Some(3.0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sort_pta_by_y_decreasing() {
        let p = make_pts();
        let (sorted, _) = p.sort_pta(SortBy::Y, SortOrder::Decreasing).unwrap();
        assert_eq!(sorted.get(0).map(|(_, y)| y), Some(4.0));
        assert_eq!(sorted.get(2).map(|(_, y)| y), Some(1.0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_sort_index() {
        let p = make_pts();
        let idx = p.get_sort_index(SortBy::X, SortOrder::Increasing).unwrap();
        assert_eq!(idx.len(), 3);
        // x values: [3,1,2] → sorted order index: [1,2,0]
        assert_eq!(idx.get_i32(0), Some(1));
        assert_eq!(idx.get_i32(1), Some(2));
        assert_eq!(idx.get_i32(2), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sort_by_index() {
        let p = make_pts();
        let idx = p.get_sort_index(SortBy::X, SortOrder::Increasing).unwrap();
        let sorted = p.sort_by_index(&idx).unwrap();
        assert_eq!(sorted.get(0).map(|(x, _)| x), Some(1.0));
        assert_eq!(sorted.get(2).map(|(x, _)| x), Some(3.0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_rank_value() {
        let p = make_pts();
        let v = p.get_rank_value(0.0, SortBy::X).unwrap();
        assert_eq!(v, 1.0);
        let v2 = p.get_rank_value(1.0, SortBy::X).unwrap();
        assert_eq!(v2, 3.0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sort_2d() {
        let mut p = Pta::new();
        p.push(2.0, 1.0);
        p.push(1.0, 3.0);
        p.push(1.0, 1.0);
        let s = p.sort_2d();
        assert_eq!(s.get(0), Some((1.0, 1.0)));
        assert_eq!(s.get(1), Some((1.0, 3.0)));
        assert_eq!(s.get(2), Some((2.0, 1.0)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_equal() {
        let p1 = make_pts();
        let p2 = make_pts();
        assert!(p1.equal(&p2));
        let mut p3 = make_pts();
        p3.push(99.0, 99.0);
        assert!(!p1.equal(&p3));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_ptaa_sort_by_index() {
        let mut ptaa = Ptaa::new();
        let mut a = Pta::new();
        a.push(1.0, 1.0);
        let mut b = Pta::new();
        b.push(2.0, 2.0);
        let mut c = Pta::new();
        c.push(3.0, 3.0);
        ptaa.push(a);
        ptaa.push(b);
        ptaa.push(c);
        // Index: [2, 0, 1]
        let na = Numa::from_slice(&[2.0, 0.0, 1.0]);
        let sorted = ptaa.sort_by_index(&na).unwrap();
        assert_eq!(sorted.get(0).unwrap().get(0), Some((3.0, 3.0)));
        assert_eq!(sorted.get(1).unwrap().get(0), Some((1.0, 1.0)));
    }
}
