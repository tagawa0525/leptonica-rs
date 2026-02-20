//! Pta geometric transformations, filtering, and array operations.
//!
//! Corresponds to functions in C Leptonica's `ptafunc1.c`.

use crate::box_::Box as LBox;
use crate::error::{Error, Result};
use crate::numa::Numa;
use crate::pta::{Pta, Ptaa};

impl Pta {
    /// Return a new Pta by subsampling every `subfactor`-th point.
    ///
    /// C equivalent: `ptaSubsample()` in `ptafunc1.c`
    pub fn subsample(&self, subfactor: usize) -> Result<Pta> {
        if subfactor < 1 {
            return Err(Error::InvalidParameter(
                "subfactor must be >= 1".to_string(),
            ));
        }
        let mut ptad = Pta::new();
        for (i, (x, y)) in self.iter().enumerate() {
            if i % subfactor == 0 {
                ptad.push(x, y);
            }
        }
        Ok(ptad)
    }

    /// Append points from `ptas[istart..=iend]` into `self`.
    ///
    /// `iend = None` means "to the end".
    ///
    /// C equivalent: `ptaJoin()` in `ptafunc1.c`
    pub fn join(&mut self, ptas: &Pta, istart: usize, iend: Option<usize>) -> Result<()> {
        let n = ptas.len();
        let iend = match iend {
            Some(e) if e < n => e,
            _ => {
                if n == 0 {
                    return Ok(());
                }
                n - 1
            }
        };
        if istart > iend {
            return Err(Error::InvalidParameter("istart > iend; no pts".to_string()));
        }
        for i in istart..=iend {
            let (ix, iy) = ptas.get_i_pt(i).map(|(x, y)| (x as f32, y as f32)).unwrap();
            self.push(ix, iy);
        }
        Ok(())
    }

    /// Return a new Pta with points in reversed order.
    ///
    /// C equivalent: `ptaReverse()` in `ptafunc1.c`
    pub fn reverse(&self) -> Pta {
        let n = self.len();
        let mut ptad = Pta::with_capacity(n);
        for i in (0..n).rev() {
            let (x, y) = self.get(i).unwrap();
            ptad.push(x, y);
        }
        ptad
    }

    /// Return a new Pta with x and y coordinates swapped.
    ///
    /// C equivalent: `ptaTranspose()` in `ptafunc1.c`
    pub fn transpose(&self) -> Pta {
        self.iter().map(|(x, y)| (y, x)).collect()
    }

    /// Return a cyclic permutation starting (and ending) at `(xs, ys)`.
    ///
    /// Requires the Pta to be a closed path (first == last point as integers).
    ///
    /// C equivalent: `ptaCyclicPerm()` in `ptafunc1.c`
    pub fn cyclic_perm(&self, xs: i32, ys: i32) -> Result<Pta> {
        let n = self.len();
        if n < 2 {
            return Err(Error::InvalidParameter(
                "pta must have at least 2 points".to_string(),
            ));
        }
        let (x1, y1) = self.get_i_pt(0).unwrap();
        let (x2, y2) = self.get_i_pt(n - 1).unwrap();
        if x1 != x2 || y1 != y2 {
            return Err(Error::InvalidParameter(
                "start and end pts not same".to_string(),
            ));
        }
        // Find the start index
        let start_idx = (0..n)
            .find(|&i| {
                let (ix, iy) = self.get_i_pt(i).unwrap();
                ix == xs && iy == ys
            })
            .ok_or_else(|| Error::InvalidParameter("start pt not in ptas".to_string()))?;

        // Build n-1 interior points starting from start_idx.
        // We must skip index n-1 (the closing duplicate of index 0), so
        // when start_idx + j reaches n-1 we jump to (start_idx+j+1)%n.
        // Using simply (start_idx+j)%n would visit both index n-1 and
        // index 0 (which hold the same coordinates), causing a duplicate
        // and missing one interior point.
        let mut ptad = Pta::with_capacity(n);
        for j in 0..n - 1 {
            let index = if start_idx + j < n - 1 {
                start_idx + j
            } else {
                (start_idx + j + 1) % n
            };
            let (ix, iy) = self.get_i_pt(index).unwrap();
            ptad.push(ix as f32, iy as f32);
        }
        ptad.push(xs as f32, ys as f32);
        Ok(ptad)
    }

    /// Return a new Pta containing points `[first, last]`.
    ///
    /// `last = None` means "to the end".
    ///
    /// C equivalent: `ptaSelectRange()` in `ptafunc1.c`
    pub fn select_range(&self, first: usize, last: Option<usize>) -> Result<Pta> {
        let n = self.len();
        if n == 0 {
            return Ok(self.clone());
        }
        if first >= n {
            return Err(Error::InvalidParameter("invalid first".to_string()));
        }
        let last = match last {
            Some(l) if l < n => l,
            _ => n - 1,
        };
        if first > last {
            return Err(Error::InvalidParameter("first > last".to_string()));
        }
        let mut ptad = Pta::with_capacity(last - first + 1);
        for i in first..=last {
            let (x, y) = self.get(i).unwrap();
            ptad.push(x, y);
        }
        Ok(ptad)
    }

    /// Return the x/y range: `(minx, maxx, miny, maxy)`.
    ///
    /// C equivalent: `ptaGetRange()` in `ptafunc1.c`
    pub fn get_range(&self) -> Result<(f32, f32, f32, f32)> {
        if self.is_empty() {
            return Err(Error::NullInput("no points in pta"));
        }
        let (x0, y0) = self.get(0).unwrap();
        let mut minx = x0;
        let mut maxx = x0;
        let mut miny = y0;
        let mut maxy = y0;
        for (x, y) in self.iter().skip(1) {
            if x < minx {
                minx = x;
            }
            if x > maxx {
                maxx = x;
            }
            if y < miny {
                miny = y;
            }
            if y > maxy {
                maxy = y;
            }
        }
        Ok((minx, maxx, miny, maxy))
    }

    /// Return points that fall inside `box_`.
    ///
    /// C equivalent: `ptaGetInsideBox()` in `ptafunc1.c`
    pub fn get_inside_box(&self, box_: &LBox) -> Pta {
        let (bx, by, bw, bh) = (box_.x, box_.y, box_.w, box_.h);
        let bx = bx as f32;
        let by = by as f32;
        let bw = bw as f32;
        let bh = bh as f32;
        self.iter()
            .filter(|&(x, y)| x >= bx && x < bx + bw && y >= by && y < by + bh)
            .collect()
    }

    /// Return `true` if `(x, y)` (rounded to integer) is contained.
    ///
    /// C equivalent: `ptaContainsPt()` in `ptafunc1.c`
    pub fn contains_pt(&self, x: i32, y: i32) -> bool {
        (0..self.len()).any(|i| {
            let (ix, iy) = self.get_i_pt(i).unwrap();
            ix == x && iy == y
        })
    }

    /// Return `true` if `self` and `other` share at least one integer point.
    ///
    /// C equivalent: `ptaTestIntersection()` in `ptafunc1.c`
    pub fn test_intersection(&self, other: &Pta) -> bool {
        let n1 = self.len();
        let n2 = other.len();
        for i in 0..n1 {
            let (x1, y1) = self.get_i_pt(i).unwrap();
            for j in 0..n2 {
                let (x2, y2) = other.get_i_pt(j).unwrap();
                if x1 == x2 && y1 == y2 {
                    return true;
                }
            }
        }
        false
    }

    /// Shift then scale all points: `x = round(scalex * (x + shiftx))`.
    ///
    /// C equivalent: `ptaTransform()` in `ptafunc1.c`
    pub fn transform_pts(&self, shiftx: i32, shifty: i32, scalex: f32, scaley: f32) -> Pta {
        self.iter()
            .map(|(x, y)| {
                let nx = (scalex * (x + shiftx as f32) + 0.5) as i32 as f32;
                let ny = (scaley * (y + shifty as f32) + 0.5) as i32 as f32;
                (nx, ny)
            })
            .collect()
    }

    /// Return `true` if `(x, y)` is strictly inside the polygon defined by `self`.
    ///
    /// Uses the sum-of-angles method.
    ///
    /// C equivalent: `ptaPtInsidePolygon()` in `ptafunc1.c`
    pub fn pt_inside_polygon(&self, x: f32, y: f32) -> bool {
        let n = self.len();
        if n < 3 {
            return false;
        }
        let sum: f64 = (0..n)
            .map(|i| {
                let (xp1, yp1) = self.get(i).unwrap();
                let (xp2, yp2) = self.get((i + 1) % n).unwrap();
                let x1 = (xp1 - x) as f64;
                let y1 = (yp1 - y) as f64;
                let x2 = (xp2 - x) as f64;
                let y2 = (yp2 - y) as f64;
                angle_between_vectors(x1, y1, x2, y2)
            })
            .sum();
        sum.abs() > std::f64::consts::PI
    }

    /// Return `true` if the polygon defined by `self` is convex.
    ///
    /// C equivalent: `ptaPolygonIsConvex()` in `ptafunc1.c`
    pub fn polygon_is_convex(&self) -> Result<bool> {
        let n = self.len();
        if n < 3 {
            return Err(Error::InvalidParameter("pta has < 3 pts".to_string()));
        }
        for i in 0..n {
            let (x0, y0) = self.get(i).unwrap();
            let (x1, y1) = self.get((i + 1) % n).unwrap();
            let (x2, y2) = self.get((i + 2) % n).unwrap();
            let cprod = (x1 - x0) as f64 * (y2 - y1) as f64 - (y1 - y0) as f64 * (x2 - x1) as f64;
            if cprod < -0.0001 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Return `(xmin, ymin, xmax, ymax)`.
    ///
    /// Returns `None` if empty.
    ///
    /// C equivalent: `ptaGetMinMax()` in `ptafunc1.c`
    pub fn get_min_max(&self) -> Option<(f32, f32, f32, f32)> {
        if self.is_empty() {
            return None;
        }
        let mut xmin = f32::MAX;
        let mut ymin = f32::MAX;
        let mut xmax = f32::MIN;
        let mut ymax = f32::MIN;
        for (x, y) in self.iter() {
            if x < xmin {
                xmin = x;
            }
            if y < ymin {
                ymin = y;
            }
            if x > xmax {
                xmax = x;
            }
            if y > ymax {
                ymax = y;
            }
        }
        Some((xmin, ymin, xmax, ymax))
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
        let matches = |v: f32, th: f32| match relation {
            SelectRelation::Lt => v < th,
            SelectRelation::Gt => v > th,
            SelectRelation::Lte => v <= th,
            SelectRelation::Gte => v >= th,
        };
        self.iter()
            .filter(|&(x, y)| match select {
                SelectCoord::X => matches(x, xth),
                SelectCoord::Y => matches(y, yth),
                SelectCoord::Either => matches(x, xth) || matches(y, yth),
                SelectCoord::Both => matches(x, xth) && matches(y, yth),
            })
            .collect()
    }

    /// Create a Pta from a single Numa, using the Numa's x-parameters for x.
    ///
    /// C equivalent: `numaConvertToPta1()` in `ptafunc1.c`
    pub fn from_numa(na: &Numa) -> Pta {
        let (startx, delx) = na.parameters();
        na.as_slice()
            .iter()
            .enumerate()
            .map(|(i, &val)| (startx + i as f32 * delx, val))
            .collect()
    }

    /// Create a Pta from two Numa arrays (x, y).
    ///
    /// C equivalent: `numaConvertToPta2()` in `ptafunc1.c`
    pub fn from_numa2(nax: &Numa, nay: &Numa) -> Pta {
        let n = nax.len().min(nay.len());
        (0..n)
            .map(|i| (nax.get(i).unwrap(), nay.get(i).unwrap()))
            .collect()
    }

    /// Convert to (nax, nay) Numa arrays.
    ///
    /// C equivalent: `ptaConvertToNuma()` in `ptafunc1.c`
    pub fn to_numa(&self) -> (Numa, Numa) {
        let mut nax = Numa::with_capacity(self.len());
        let mut nay = Numa::with_capacity(self.len());
        for (x, y) in self.iter() {
            nax.push(x);
            nay.push(y);
        }
        (nax, nay)
    }
}

impl Ptaa {
    /// Append Pta elements from `ptaas[istart..=iend]` into `self`.
    ///
    /// C equivalent: `ptaaJoin()` in `ptafunc1.c`
    pub fn join(&mut self, ptaas: &Ptaa, istart: usize, iend: Option<usize>) -> Result<()> {
        let n = ptaas.len();
        if n == 0 {
            return Ok(());
        }
        let iend = match iend {
            Some(e) if e < n => e,
            _ => n - 1,
        };
        if istart > iend {
            return Err(Error::InvalidParameter("istart > iend; no pts".to_string()));
        }
        for i in istart..=iend {
            self.push(ptaas.get(i).unwrap().clone());
        }
        Ok(())
    }
}

/// Angle from vector (x1,y1) to vector (x2,y2), folded into (-π, π].
fn angle_between_vectors(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let ang = y2.atan2(x2) - y1.atan2(x1);
    if ang > std::f64::consts::PI {
        ang - 2.0 * std::f64::consts::PI
    } else if ang < -std::f64::consts::PI {
        ang + 2.0 * std::f64::consts::PI
    } else {
        ang
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
    fn test_subsample_basic() {
        let p = make_square();
        let s = p.subsample(2).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s.get(0), Some((0.0, 0.0)));
        assert_eq!(s.get(1), Some((10.0, 10.0)));
    }

    #[test]
    fn test_subsample_invalid() {
        let p = make_square();
        assert!(p.subsample(0).is_err());
    }

    #[test]
    fn test_join_basic() {
        let mut p1 = Pta::new();
        p1.push(1.0, 2.0);
        let p2 = make_square();
        p1.join(&p2, 0, None).unwrap();
        assert_eq!(p1.len(), 5);
    }

    #[test]
    fn test_reverse() {
        let p = make_square();
        let r = p.reverse();
        assert_eq!(r.len(), 4);
        assert_eq!(r.get(0), Some((0.0, 10.0)));
        assert_eq!(r.get(3), Some((0.0, 0.0)));
    }

    #[test]
    fn test_transpose() {
        let mut p = Pta::new();
        p.push(1.0, 2.0);
        p.push(3.0, 4.0);
        let t = p.transpose();
        assert_eq!(t.get(0), Some((2.0, 1.0)));
        assert_eq!(t.get(1), Some((4.0, 3.0)));
    }

    #[test]
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
    fn test_select_range() {
        let p = make_square();
        let s = p.select_range(1, Some(2)).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s.get(0), Some((10.0, 0.0)));
        assert_eq!(s.get(1), Some((10.0, 10.0)));
    }

    #[test]
    fn test_get_range() {
        let p = make_square();
        let (minx, maxx, miny, maxy) = p.get_range().unwrap();
        assert_eq!(minx, 0.0);
        assert_eq!(maxx, 10.0);
        assert_eq!(miny, 0.0);
        assert_eq!(maxy, 10.0);
    }

    #[test]
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
    fn test_contains_pt() {
        let p = make_square();
        assert!(p.contains_pt(0, 0));
        assert!(p.contains_pt(10, 10));
        assert!(!p.contains_pt(5, 5));
    }

    #[test]
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
    fn test_transform_pts() {
        let mut p = Pta::new();
        p.push(2.0, 3.0);
        let t = p.transform_pts(1, 2, 2.0, 3.0);
        // x = round(2.0 * (2+1)) = 6, y = round(3.0 * (3+2)) = 15
        assert_eq!(t.get(0), Some((6.0, 15.0)));
    }

    #[test]
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
    fn test_get_min_max() {
        let p = make_square();
        let (xmin, ymin, xmax, ymax) = p.get_min_max().unwrap();
        assert_eq!(xmin, 0.0);
        assert_eq!(ymin, 0.0);
        assert_eq!(xmax, 10.0);
        assert_eq!(ymax, 10.0);
    }

    #[test]
    fn test_select_by_value() {
        let p = make_square();
        let sel = p.select_by_value(5.0, 0.0, SelectCoord::X, SelectRelation::Gte);
        assert_eq!(sel.len(), 2); // (10,0) and (10,10)
    }

    #[test]
    fn test_from_numa() {
        let na = Numa::from_slice(&[10.0, 20.0, 30.0]);
        let p = Pta::from_numa(&na);
        assert_eq!(p.len(), 3);
        // x = startx + i * delx = 0 + i * 1
        assert_eq!(p.get(0), Some((0.0, 10.0)));
        assert_eq!(p.get(2), Some((2.0, 30.0)));
    }

    #[test]
    fn test_from_numa2() {
        let nax = Numa::from_slice(&[1.0, 2.0, 3.0]);
        let nay = Numa::from_slice(&[4.0, 5.0, 6.0]);
        let p = Pta::from_numa2(&nax, &nay);
        assert_eq!(p.len(), 3);
        assert_eq!(p.get(0), Some((1.0, 4.0)));
        assert_eq!(p.get(2), Some((3.0, 6.0)));
    }

    #[test]
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
