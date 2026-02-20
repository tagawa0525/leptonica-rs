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
    /// Return a sorted copy of `self` plus the sort index Numa.
    ///
    /// C equivalent: `ptaSort()` in `ptafunc2.c`
    pub fn sort_pta(&self, by: SortBy, order: SortOrder) -> Result<(Pta, Numa)> {
        let naindex = self.get_sort_index(by, order)?;
        let ptad = self.sort_by_index(&naindex)?;
        Ok((ptad, naindex))
    }

    /// Return the index array that would sort `self`.
    ///
    /// C equivalent: `ptaGetSortIndex()` in `ptafunc2.c`
    pub fn get_sort_index(&self, by: SortBy, order: SortOrder) -> Result<Numa> {
        let n = self.len();
        let mut na = Numa::with_capacity(n);
        for i in 0..n {
            let (x, y) = self.get(i).unwrap();
            match by {
                SortBy::X => na.push(x),
                SortBy::Y => na.push(y),
            }
        }
        Ok(na.sort_index(order))
    }

    /// Return a new Pta sorted according to an index Numa.
    ///
    /// C equivalent: `ptaSortByIndex()` in `ptafunc2.c`
    pub fn sort_by_index(&self, naindex: &Numa) -> Result<Pta> {
        let n = naindex.len();
        let mut ptad = Pta::with_capacity(n);
        for i in 0..n {
            let index = naindex
                .get_i32(i)
                .ok_or_else(|| Error::InvalidParameter("invalid index".to_string()))?
                as usize;
            let (x, y) = self.get(index).ok_or(Error::IndexOutOfBounds {
                index,
                len: self.len(),
            })?;
            ptad.push(x, y);
        }
        Ok(ptad)
    }

    /// Return the x or y value at fractile `fract` (0.0 = min, 1.0 = max).
    ///
    /// C equivalent: `ptaGetRankValue()` in `ptafunc2.c`
    pub fn get_rank_value(&self, fract: f32, by: SortBy) -> Result<f32> {
        if self.is_empty() {
            return Err(Error::NullInput("empty Pta"));
        }
        if !(0.0..=1.0).contains(&fract) {
            return Err(Error::InvalidParameter(
                "fract must be in [0.0, 1.0]".to_string(),
            ));
        }
        let (sorted, _) = self.sort_pta(by, SortOrder::Increasing)?;
        let n = sorted.len();
        let index = ((fract * (n - 1) as f32) + 0.5) as usize;
        let index = index.min(n - 1);
        let (x, y) = sorted.get(index).unwrap();
        Ok(match by {
            SortBy::X => x,
            SortBy::Y => y,
        })
    }

    /// Return a copy sorted first by x, then by y (lexicographic).
    ///
    /// C equivalent: `ptaSort2d()` in `ptafunc2.c`
    pub fn sort_2d(&self) -> Pta {
        let mut pts: Vec<(f32, f32)> = self.iter().collect();
        pts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        pts.into_iter().collect()
    }

    /// Return `true` if `self` and `other` have the same points in the same order.
    ///
    /// C equivalent: `ptaEqual()` in `ptafunc2.c`
    pub fn equal(&self, other: &Pta) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().zip(other.iter()).all(|((x1, y1), (x2, y2))| {
            (x1 - x2).abs() < f32::EPSILON && (y1 - y2).abs() < f32::EPSILON
        })
    }
}

impl Ptaa {
    /// Return a new Ptaa sorted according to an index Numa.
    ///
    /// C equivalent: `ptaaSortByIndex()` in `ptafunc2.c`
    pub fn sort_by_index(&self, naindex: &Numa) -> Result<Ptaa> {
        let n = self.len();
        if naindex.len() != n {
            return Err(Error::InvalidParameter(
                "numa and ptaa sizes differ".to_string(),
            ));
        }
        let mut ptaad = Ptaa::with_capacity(n);
        for i in 0..n {
            let index = naindex
                .get_i32(i)
                .ok_or_else(|| Error::InvalidParameter("invalid index".to_string()))?
                as usize;
            let pta = self
                .get(index)
                .ok_or(Error::IndexOutOfBounds { index, len: n })?;
            ptaad.push(pta.clone());
        }
        Ok(ptaad)
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
    fn test_sort_pta_by_x_increasing() {
        let p = make_pts();
        let (sorted, _) = p.sort_pta(SortBy::X, SortOrder::Increasing).unwrap();
        assert_eq!(sorted.get(0).map(|(x, _)| x), Some(1.0));
        assert_eq!(sorted.get(1).map(|(x, _)| x), Some(2.0));
        assert_eq!(sorted.get(2).map(|(x, _)| x), Some(3.0));
    }

    #[test]
    fn test_sort_pta_by_y_decreasing() {
        let p = make_pts();
        let (sorted, _) = p.sort_pta(SortBy::Y, SortOrder::Decreasing).unwrap();
        assert_eq!(sorted.get(0).map(|(_, y)| y), Some(4.0));
        assert_eq!(sorted.get(2).map(|(_, y)| y), Some(1.0));
    }

    #[test]
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
    fn test_sort_by_index() {
        let p = make_pts();
        let idx = p.get_sort_index(SortBy::X, SortOrder::Increasing).unwrap();
        let sorted = p.sort_by_index(&idx).unwrap();
        assert_eq!(sorted.get(0).map(|(x, _)| x), Some(1.0));
        assert_eq!(sorted.get(2).map(|(x, _)| x), Some(3.0));
    }

    #[test]
    fn test_get_rank_value() {
        let p = make_pts();
        let v = p.get_rank_value(0.0, SortBy::X).unwrap();
        assert_eq!(v, 1.0);
        let v2 = p.get_rank_value(1.0, SortBy::X).unwrap();
        assert_eq!(v2, 3.0);
    }

    #[test]
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
    fn test_equal() {
        let p1 = make_pts();
        let p2 = make_pts();
        assert!(p1.equal(&p2));
        let mut p3 = make_pts();
        p3.push(99.0, 99.0);
        assert!(!p1.equal(&p3));
    }

    #[test]
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
