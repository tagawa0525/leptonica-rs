//! Pta geometric transformations, filtering, and array operations.
//!
//! Corresponds to functions in C Leptonica's `ptafunc1.c`.

use crate::box_::Box as LBox;
use crate::error::{Error, Result};
use crate::numa::{Numa, SortOrder};
use crate::pta::{Pta, Ptaa};

impl Pta {
    /// Return a new Pta by subsampling every `subfactor`-th point.
    ///
    /// C equivalent: `ptaSubsample()` in `ptafunc1.c`
    pub fn subsample(&self, subfactor: usize) -> Result<Pta> {
        todo!("Phase 16.3 GREEN")
    }

    /// Append points from `ptas[istart..=iend]` into `self`.
    ///
    /// `iend < 0` (represented as `None`) means "to the end".
    ///
    /// C equivalent: `ptaJoin()` in `ptafunc1.c`
    pub fn join(&mut self, ptas: &Pta, istart: usize, iend: Option<usize>) -> Result<()> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return a new Pta with points in reversed order.
    ///
    /// C equivalent: `ptaReverse()` in `ptafunc1.c`
    pub fn reverse(&self) -> Pta {
        todo!("Phase 16.3 GREEN")
    }

    /// Return a new Pta with x and y coordinates swapped.
    ///
    /// C equivalent: `ptaTranspose()` in `ptafunc1.c`
    pub fn transpose(&self) -> Pta {
        todo!("Phase 16.3 GREEN")
    }

    /// Return a cyclic permutation starting (and ending) at `(xs, ys)`.
    ///
    /// Requires the Pta to be a closed path (first == last point).
    ///
    /// C equivalent: `ptaCyclicPerm()` in `ptafunc1.c`
    pub fn cyclic_perm(&self, xs: i32, ys: i32) -> Result<Pta> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return a new Pta containing points `[first, last]`.
    ///
    /// `last = None` means "to the end".
    ///
    /// C equivalent: `ptaSelectRange()` in `ptafunc1.c`
    pub fn select_range(&self, first: usize, last: Option<usize>) -> Result<Pta> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return the x/y range: `(minx, maxx, miny, maxy)`.
    ///
    /// C equivalent: `ptaGetRange()` in `ptafunc1.c`
    pub fn get_range(&self) -> Result<(f32, f32, f32, f32)> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return points that fall inside `box_`.
    ///
    /// C equivalent: `ptaGetInsideBox()` in `ptafunc1.c`
    pub fn get_inside_box(&self, box_: &LBox) -> Pta {
        todo!("Phase 16.3 GREEN")
    }

    /// Return `true` if `(x, y)` (rounded to integer) is contained.
    ///
    /// C equivalent: `ptaContainsPt()` in `ptafunc1.c`
    pub fn contains_pt(&self, x: i32, y: i32) -> bool {
        todo!("Phase 16.3 GREEN")
    }

    /// Return `true` if `self` and `other` share at least one integer point.
    ///
    /// C equivalent: `ptaTestIntersection()` in `ptafunc1.c`
    pub fn test_intersection(&self, other: &Pta) -> bool {
        todo!("Phase 16.3 GREEN")
    }

    /// Shift then scale all points: `x = scalex * (x + shiftx)`.
    ///
    /// C equivalent: `ptaTransform()` in `ptafunc1.c`
    pub fn transform_pts(&self, shiftx: i32, shifty: i32, scalex: f32, scaley: f32) -> Pta {
        todo!("Phase 16.3 GREEN")
    }

    /// Return `true` if `(x, y)` is strictly inside the polygon defined by `self`.
    ///
    /// Uses the sum-of-angles method.
    ///
    /// C equivalent: `ptaPtInsidePolygon()` in `ptafunc1.c`
    pub fn pt_inside_polygon(&self, x: f32, y: f32) -> bool {
        todo!("Phase 16.3 GREEN")
    }

    /// Return `true` if the polygon defined by `self` is convex.
    ///
    /// C equivalent: `ptaPolygonIsConvex()` in `ptafunc1.c`
    pub fn polygon_is_convex(&self) -> Result<bool> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return `(xmin, ymin, xmax, ymax)`.
    ///
    /// Returns `None` if empty.
    ///
    /// C equivalent: `ptaGetMinMax()` in `ptafunc1.c`
    pub fn get_min_max(&self) -> Option<(f32, f32, f32, f32)> {
        todo!("Phase 16.3 GREEN")
    }

    /// Return points selected by value comparison on x, y, or both.
    ///
    /// C equivalent: `ptaSelectByValue()` in `ptafunc1.c`
    pub fn select_by_value(
        &self,
        xth: f32,
        yth: f32,
        select: SelectCoord,
        relation: SelectRelation,
    ) -> Pta {
        todo!("Phase 16.3 GREEN")
    }

    /// Create a Pta from a single Numa, using the Numa's x-parameters for x.
    ///
    /// C equivalent: `numaConvertToPta1()` in `ptafunc1.c`
    pub fn from_numa(na: &Numa) -> Pta {
        todo!("Phase 16.3 GREEN")
    }

    /// Create a Pta from two Numa arrays (x, y).
    ///
    /// C equivalent: `numaConvertToPta2()` in `ptafunc1.c`
    pub fn from_numa2(nax: &Numa, nay: &Numa) -> Pta {
        todo!("Phase 16.3 GREEN")
    }

    /// Convert to (nax, nay) Numa arrays.
    ///
    /// C equivalent: `ptaConvertToNuma()` in `ptafunc1.c`
    pub fn to_numa(&self) -> (Numa, Numa) {
        todo!("Phase 16.3 GREEN")
    }
}

impl Ptaa {
    /// Append Pta elements from `ptaas[istart..=iend]` into `self`.
    ///
    /// C equivalent: `ptaaJoin()` in `ptafunc1.c`
    pub fn join(&mut self, ptaas: &Ptaa, istart: usize, iend: Option<usize>) -> Result<()> {
        todo!("Phase 16.3 GREEN")
    }
}

/// Which coordinate(s) to select on
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectCoord {
    /// Select on X value
    X,
    /// Select on Y value
    Y,
    /// Select if either X or Y matches
    Either,
    /// Select if both X and Y match
    Both,
}

/// Comparison relation for `select_by_value`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectRelation {
    /// Less than
    Lt,
    /// Greater than
    Gt,
    /// Less than or equal
    Lte,
    /// Greater than or equal
    Gte,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_square() -> Pta {
        let mut p = Pta::new();
        p.push(0.0, 0.0);
        p.push(10.0, 0.0);
        p.push(10.0, 10.0);
        p.push(0.0, 10.0);
        p
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_subsample_basic() {
        let p = make_square();
        let s = p.subsample(2).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s.get(0), Some((0.0, 0.0)));
        assert_eq!(s.get(1), Some((10.0, 10.0)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_subsample_invalid() {
        let p = make_square();
        assert!(p.subsample(0).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_join_basic() {
        let mut p1 = Pta::new();
        p1.push(1.0, 2.0);
        let p2 = make_square();
        p1.join(&p2, 0, None).unwrap();
        assert_eq!(p1.len(), 5);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_reverse() {
        let p = make_square();
        let r = p.reverse();
        assert_eq!(r.len(), 4);
        assert_eq!(r.get(0), Some((0.0, 10.0)));
        assert_eq!(r.get(3), Some((0.0, 0.0)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_transpose() {
        let mut p = Pta::new();
        p.push(1.0, 2.0);
        p.push(3.0, 4.0);
        let t = p.transpose();
        assert_eq!(t.get(0), Some((2.0, 1.0)));
        assert_eq!(t.get(1), Some((4.0, 3.0)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_cyclic_perm() {
        let mut p = Pta::new();
        p.push(0.0, 0.0);
        p.push(1.0, 0.0);
        p.push(1.0, 1.0);
        p.push(0.0, 1.0);
        p.push(0.0, 0.0); // closed
        let c = p.cyclic_perm(1, 0).unwrap();
        assert_eq!(c.len(), 5);
        assert_eq!(c.get(0).map(|(x, y)| (x as i32, y as i32)), Some((1, 0)));
        assert_eq!(c.get(4).map(|(x, y)| (x as i32, y as i32)), Some((1, 0)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_select_range() {
        let p = make_square();
        let s = p.select_range(1, Some(2)).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s.get(0), Some((10.0, 0.0)));
        assert_eq!(s.get(1), Some((10.0, 10.0)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_range() {
        let p = make_square();
        let (minx, maxx, miny, maxy) = p.get_range().unwrap();
        assert_eq!(minx, 0.0);
        assert_eq!(maxx, 10.0);
        assert_eq!(miny, 0.0);
        assert_eq!(maxy, 10.0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_inside_box() {
        let p = make_square();
        let b = LBox::new(0, 0, 11, 11).unwrap();
        let inside = p.get_inside_box(&b);
        assert_eq!(inside.len(), 4);
        let b2 = LBox::new(0, 0, 5, 5).unwrap();
        let inside2 = p.get_inside_box(&b2);
        assert_eq!(inside2.len(), 1); // only (0,0)
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_contains_pt() {
        let p = make_square();
        assert!(p.contains_pt(0, 0));
        assert!(p.contains_pt(10, 10));
        assert!(!p.contains_pt(5, 5));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_test_intersection() {
        let p1 = make_square();
        let mut p2 = Pta::new();
        p2.push(10.0, 10.0);
        p2.push(20.0, 20.0);
        assert!(p1.test_intersection(&p2));
        let mut p3 = Pta::new();
        p3.push(99.0, 99.0);
        assert!(!p1.test_intersection(&p3));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_transform_pts() {
        let mut p = Pta::new();
        p.push(2.0, 3.0);
        let t = p.transform_pts(1, 2, 2.0, 3.0);
        // x = round(2.0 * (2+1)) = 6, y = round(3.0 * (3+2)) = 15
        assert_eq!(t.get(0), Some((6.0, 15.0)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_pt_inside_polygon() {
        // Square polygon
        let mut p = Pta::new();
        p.push(0.0, 0.0);
        p.push(10.0, 0.0);
        p.push(10.0, 10.0);
        p.push(0.0, 10.0);
        assert!(p.pt_inside_polygon(5.0, 5.0));
        assert!(!p.pt_inside_polygon(15.0, 15.0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_polygon_is_convex() {
        let p = make_square();
        assert!(p.polygon_is_convex().unwrap());
        // Non-convex polygon (concave)
        let mut nc = Pta::new();
        nc.push(0.0, 0.0);
        nc.push(10.0, 0.0);
        nc.push(5.0, 3.0); // indent
        nc.push(10.0, 10.0);
        nc.push(0.0, 10.0);
        assert!(!nc.polygon_is_convex().unwrap());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_min_max() {
        let p = make_square();
        let (xmin, ymin, xmax, ymax) = p.get_min_max().unwrap();
        assert_eq!(xmin, 0.0);
        assert_eq!(ymin, 0.0);
        assert_eq!(xmax, 10.0);
        assert_eq!(ymax, 10.0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_select_by_value() {
        let p = make_square();
        let sel = p.select_by_value(5.0, 0.0, SelectCoord::X, SelectRelation::Gte);
        assert_eq!(sel.len(), 2); // (10,0) and (10,10)
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_from_numa() {
        let na = Numa::from_slice(&[10.0, 20.0, 30.0]);
        let p = Pta::from_numa(&na);
        assert_eq!(p.len(), 3);
        // x = startx + i * delx = 0 + i * 1
        assert_eq!(p.get(0), Some((0.0, 10.0)));
        assert_eq!(p.get(2), Some((2.0, 30.0)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_from_numa2() {
        let nax = Numa::from_slice(&[1.0, 2.0, 3.0]);
        let nay = Numa::from_slice(&[4.0, 5.0, 6.0]);
        let p = Pta::from_numa2(&nax, &nay);
        assert_eq!(p.len(), 3);
        assert_eq!(p.get(0), Some((1.0, 4.0)));
        assert_eq!(p.get(2), Some((3.0, 6.0)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_to_numa() {
        let mut p = Pta::new();
        p.push(1.0, 4.0);
        p.push(2.0, 5.0);
        let (nax, nay) = p.to_numa();
        assert_eq!(nax.len(), 2);
        assert_eq!(nay.len(), 2);
        assert_eq!(nax.get(0), Some(1.0));
        assert_eq!(nay.get(1), Some(5.0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_ptaa_join() {
        let mut ptaa1 = Ptaa::new();
        ptaa1.push(make_square());
        let mut ptaa2 = Ptaa::new();
        ptaa2.push(Pta::new());
        ptaa2.push(make_square());
        ptaa1.join(&ptaa2, 0, None).unwrap();
        assert_eq!(ptaa1.len(), 3);
    }
}
