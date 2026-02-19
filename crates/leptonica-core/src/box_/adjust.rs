//! Box adjustment, comparison, and conversion operations
//!
//! Functions for adjusting box sides, comparing boxes, split/merge operations,
//! and Box/Pta conversions.
//!
//! C Leptonica equivalents: boxfunc1.c, boxfunc4.c

use crate::error::{Error, Result};
use crate::pta::Pta;

use super::geometry::Direction;
use super::{Box, Boxa, Boxaa};

// ---- Types ----

/// Which sides to adjust when targeting a specific dimension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustSide {
    /// Adjust the left/top side only
    Start,
    /// Adjust the right/bottom side only
    End,
    /// Adjust both sides equally
    Both,
}

// ---- Box methods ----

impl Box {
    /// Relocate one side of the box to a specific coordinate.
    ///
    /// Adjusts the position and dimension to accommodate the new side location.
    ///
    /// C Leptonica equivalent: `boxRelocateOneSide`
    pub fn relocate_one_side(&self, side: Direction, loc: i32) -> Box {
        if self.w == 0 || self.h == 0 {
            return *self;
        }
        match side {
            Direction::FromLeft => Box {
                x: loc,
                y: self.y,
                w: self.w + self.x - loc,
                h: self.h,
            },
            Direction::FromRight => Box {
                x: self.x,
                y: self.y,
                w: loc - self.x + 1,
                h: self.h,
            },
            Direction::FromTop => Box {
                x: self.x,
                y: loc,
                w: self.w,
                h: self.h + self.y - loc,
            },
            Direction::FromBottom => Box {
                x: self.x,
                y: self.y,
                w: self.w,
                h: loc - self.y + 1,
            },
        }
    }

    /// Adjust all four sides by specified deltas.
    ///
    /// Returns `None` if the resulting box would have non-positive dimensions.
    /// Coordinates are clamped to >= 0.
    ///
    /// C Leptonica equivalent: `boxAdjustSides`
    pub fn adjust_sides(
        &self,
        del_left: i32,
        del_right: i32,
        del_top: i32,
        del_bot: i32,
    ) -> Option<Box> {
        let xl = (self.x + del_left).max(0);
        let yt = (self.y + del_top).max(0);
        let xr = self.x + self.w + del_right;
        let yb = self.y + self.h + del_bot;
        let w = xr - xl;
        let h = yb - yt;
        if w < 1 || h < 1 {
            return None;
        }
        Some(Box { x: xl, y: yt, w, h })
    }

    /// Set a specific side to an absolute coordinate.
    ///
    /// Only applies the change if the difference exceeds `thresh`.
    /// Use `thresh = 0` to always apply.
    ///
    /// C Leptonica equivalent: `boxSetSide`
    pub fn set_side(&mut self, side: Direction, val: i32, thresh: i32) {
        match side {
            Direction::FromLeft => {
                if (self.x - val).abs() >= thresh {
                    self.w += self.x - val;
                    self.x = val;
                }
            }
            Direction::FromRight => {
                let right = self.x + self.w - 1;
                if (right - val).abs() >= thresh {
                    self.w = val - self.x + 1;
                }
            }
            Direction::FromTop => {
                if (self.y - val).abs() >= thresh {
                    self.h += self.y - val;
                    self.y = val;
                }
            }
            Direction::FromBottom => {
                let bottom = self.y + self.h - 1;
                if (bottom - val).abs() >= thresh {
                    self.h = val - self.y + 1;
                }
            }
        }
    }

    /// Check if two boxes are similar with per-side tolerances.
    ///
    /// Returns true if each side location differs by at most the specified amount.
    ///
    /// C Leptonica equivalent: `boxSimilar`
    pub fn similar_per_side(
        &self,
        other: &Box,
        left_diff: i32,
        right_diff: i32,
        top_diff: i32,
        bot_diff: i32,
    ) -> bool {
        (self.x - other.x).abs() <= left_diff
            && ((self.x + self.w - 1) - (other.x + other.w - 1)).abs() <= right_diff
            && (self.y - other.y).abs() <= top_diff
            && ((self.y + self.h - 1) - (other.y + other.h - 1)).abs() <= bot_diff
    }

    /// Convert this box to a Pta of corner points.
    ///
    /// - `ncorners = 2`: returns UL (x,y) and LR (x+w-1, y+h-1)
    /// - `ncorners = 4`: returns UL, UR, LL, LR
    ///
    /// C Leptonica equivalent: `boxConvertToPta`
    pub fn to_pta(&self, ncorners: usize) -> Result<Pta> {
        if ncorners != 2 && ncorners != 4 {
            return Err(Error::InvalidParameter(format!(
                "ncorners must be 2 or 4, got {}",
                ncorners
            )));
        }
        let mut pta = Pta::with_capacity(ncorners);
        let x = self.x as f32;
        let y = self.y as f32;
        let xr = (self.x + self.w - 1) as f32;
        let yb = (self.y + self.h - 1) as f32;

        if ncorners == 2 {
            pta.push(x, y); // UL
            pta.push(xr, yb); // LR
        } else {
            pta.push(x, y); // UL
            pta.push(xr, y); // UR
            pta.push(x, yb); // LL
            pta.push(xr, yb); // LR
        }
        Ok(pta)
    }
}

// ---- Boxa methods ----

impl Boxa {
    /// Adjust all four sides of every box by specified deltas.
    ///
    /// Boxes that would have non-positive dimensions become (x, y, 1, 1).
    ///
    /// C Leptonica equivalent: `boxaAdjustSides`
    pub fn adjust_all_sides(
        &self,
        del_left: i32,
        del_right: i32,
        del_top: i32,
        del_bot: i32,
    ) -> Boxa {
        self.iter()
            .map(|b| {
                b.adjust_sides(del_left, del_right, del_top, del_bot)
                    .unwrap_or(Box {
                        x: b.x,
                        y: b.y,
                        w: 1,
                        h: 1,
                    })
            })
            .collect()
    }

    /// Adjust the four sides of a single box in-place.
    ///
    /// C Leptonica equivalent: `boxaAdjustBoxSides`
    pub fn adjust_box_sides(
        &mut self,
        index: usize,
        del_left: i32,
        del_right: i32,
        del_top: i32,
        del_bot: i32,
    ) -> Result<()> {
        let b = self.get(index).ok_or(Error::IndexOutOfBounds {
            index,
            len: self.len(),
        })?;
        let adjusted = b
            .adjust_sides(del_left, del_right, del_top, del_bot)
            .ok_or_else(|| {
                Error::InvalidParameter("adjustment would produce non-positive dimensions".into())
            })?;
        self.replace(index, adjusted)?;
        Ok(())
    }

    /// Set a specific side of all boxes to an absolute coordinate.
    ///
    /// C Leptonica equivalent: `boxaSetSide`
    pub fn set_all_sides(&mut self, side: Direction, val: i32, thresh: i32) {
        for b in self.iter_mut() {
            b.set_side(side, val, thresh);
        }
    }

    /// Adjust width of all boxes to a target value.
    ///
    /// Only adjusts if `|current_width - target| >= thresh`.
    /// `target` must be positive; if not, boxes are returned unchanged.
    ///
    /// C Leptonica equivalent: `boxaAdjustWidthToTarget`
    pub fn adjust_width_to_target(&self, adjust: AdjustSide, target: i32, thresh: i32) -> Boxa {
        if target < 1 {
            return self.clone();
        }
        self.iter()
            .map(|b| {
                let diff = b.w - target;
                if diff.abs() < thresh {
                    return *b;
                }
                match adjust {
                    AdjustSide::Start => Box {
                        x: (b.x + diff).max(0),
                        y: b.y,
                        w: target,
                        h: b.h,
                    },
                    AdjustSide::End => Box {
                        x: b.x,
                        y: b.y,
                        w: target,
                        h: b.h,
                    },
                    AdjustSide::Both => Box {
                        x: (b.x + diff / 2).max(0),
                        y: b.y,
                        w: target,
                        h: b.h,
                    },
                }
            })
            .collect()
    }

    /// Adjust height of all boxes to a target value.
    ///
    /// Only adjusts if `|current_height - target| >= thresh`.
    /// `target` must be positive; if not, boxes are returned unchanged.
    ///
    /// C Leptonica equivalent: `boxaAdjustHeightToTarget`
    pub fn adjust_height_to_target(&self, adjust: AdjustSide, target: i32, thresh: i32) -> Boxa {
        if target < 1 {
            return self.clone();
        }
        self.iter()
            .map(|b| {
                let diff = b.h - target;
                if diff.abs() < thresh {
                    return *b;
                }
                match adjust {
                    AdjustSide::Start => Box {
                        x: b.x,
                        y: (b.y + diff).max(0),
                        w: b.w,
                        h: target,
                    },
                    AdjustSide::End => Box {
                        x: b.x,
                        y: b.y,
                        w: b.w,
                        h: target,
                    },
                    AdjustSide::Both => Box {
                        x: b.x,
                        y: (b.y + diff / 2).max(0),
                        w: b.w,
                        h: target,
                    },
                }
            })
            .collect()
    }

    /// Check if two Boxas contain the same boxes.
    ///
    /// With `max_dist = 0`, requires exact ordering.
    /// With `max_dist > 0`, allows boxes to differ in position by up to max_dist indices.
    ///
    /// C Leptonica equivalent: `boxaEqual`
    pub fn equal_ordered(&self, other: &Boxa, max_dist: usize) -> bool {
        if self.len() != other.len() {
            return false;
        }
        if max_dist == 0 {
            return self.iter().zip(other.iter()).all(|(a, b)| a == b);
        }
        // With max_dist > 0: each box in self must match some box in other within range
        let n = self.len();
        let mut matched = vec![false; n];
        for (i, a) in self.iter().enumerate() {
            let j_min = i.saturating_sub(max_dist);
            let j_max = (i + max_dist + 1).min(n);
            let mut found = false;
            for (j, m) in matched[j_min..j_max].iter_mut().enumerate() {
                if !*m && a == other.get(j_min + j).unwrap() {
                    *m = true;
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }

    /// Split boxes into even-indexed and odd-indexed arrays.
    ///
    /// If `fill` is true, output arrays have the same length as input,
    /// with (0,0,0,0) placeholder boxes at the other parity positions.
    ///
    /// C Leptonica equivalent: `boxaSplitEvenOdd`
    pub fn split_even_odd(&self, fill: bool) -> (Boxa, Boxa) {
        let n = self.len();
        let placeholder = Box::new_unchecked(0, 0, 0, 0);

        if !fill {
            let even: Boxa = self
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 2 == 0)
                .map(|(_, b)| *b)
                .collect();
            let odd: Boxa = self
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 2 == 1)
                .map(|(_, b)| *b)
                .collect();
            (even, odd)
        } else {
            let mut even = Boxa::with_capacity(n);
            let mut odd = Boxa::with_capacity(n);
            for i in 0..n {
                if i % 2 == 0 {
                    even.push(*self.get(i).unwrap());
                    odd.push(placeholder);
                } else {
                    even.push(placeholder);
                    odd.push(*self.get(i).unwrap());
                }
            }
            (even, odd)
        }
    }

    /// Merge even and odd arrays back into a single Boxa.
    ///
    /// Inverse of `split_even_odd`. `fill` must match the flag used in the split.
    ///
    /// C Leptonica equivalent: `boxaMergeEvenOdd`
    pub fn merge_even_odd(even: &Boxa, odd: &Boxa, fill: bool) -> Result<Boxa> {
        if !fill {
            let ne = even.len();
            let no = odd.len();
            if !(ne >= no && ne <= no + 1) {
                return Err(Error::InvalidParameter(format!(
                    "even ({}) and odd ({}) lengths incompatible for merge",
                    ne, no
                )));
            }
            let total = ne + no;
            let mut result = Boxa::with_capacity(total);
            for i in 0..total {
                if i % 2 == 0 {
                    result.push(*even.get(i / 2).unwrap());
                } else {
                    result.push(*odd.get(i / 2).unwrap());
                }
            }
            Ok(result)
        } else {
            if even.len() != odd.len() {
                return Err(Error::InvalidParameter(format!(
                    "even ({}) and odd ({}) lengths must match when fill=true",
                    even.len(),
                    odd.len()
                )));
            }
            let n = even.len();
            let mut result = Boxa::with_capacity(n);
            for i in 0..n {
                if i % 2 == 0 {
                    result.push(*even.get(i).unwrap());
                } else {
                    result.push(*odd.get(i).unwrap());
                }
            }
            Ok(result)
        }
    }

    /// Convert all boxes to a Pta of corner points.
    ///
    /// Each box contributes `ncorners` points (2 or 4).
    ///
    /// C Leptonica equivalent: `boxaConvertToPta`
    pub fn to_pta(&self, ncorners: usize) -> Result<Pta> {
        if ncorners != 2 && ncorners != 4 {
            return Err(Error::InvalidParameter(format!(
                "ncorners must be 2 or 4, got {}",
                ncorners
            )));
        }
        let mut pta = Pta::with_capacity(self.len() * ncorners);
        for b in self.iter() {
            let box_pta = b.to_pta(ncorners)?;
            for (x, y) in box_pta.iter() {
                pta.push(x, y);
            }
        }
        Ok(pta)
    }
}

// ---- Boxaa methods ----

impl Boxaa {
    /// Append Boxas from another Boxaa in the range `[start, end)`.
    ///
    /// If `end` is 0, appends all from `start` onwards.
    ///
    /// C Leptonica equivalent: `boxaaJoin`
    pub fn join(&mut self, other: &Boxaa, start: usize, end: usize) {
        let actual_end = if end == 0 {
            other.len()
        } else {
            end.min(other.len())
        };
        let actual_start = start.min(actual_end);
        for i in actual_start..actual_end {
            if let Some(boxa) = other.get(i) {
                self.push(boxa.clone());
            }
        }
    }
}

// ---- Pta → Box/Boxa conversions ----

impl Pta {
    /// Convert to a bounding box enclosing all points.
    ///
    /// C Leptonica equivalent: `ptaConvertToBox`
    pub fn to_box(&self) -> Option<Box> {
        let (x_min, y_min, x_max, y_max) = self.bounding_box()?;
        let x = x_min.round() as i32;
        let y = y_min.round() as i32;
        let xr = x_max.round() as i32;
        let yb = y_max.round() as i32;
        Some(Box {
            x,
            y,
            w: xr - x + 1,
            h: yb - y + 1,
        })
    }

    /// Convert to a Boxa by grouping every `ncorners` points into a box.
    ///
    /// The Pta length must be a multiple of `ncorners` (2 or 4).
    ///
    /// C Leptonica equivalent: `ptaConvertToBoxa`
    pub fn to_boxa(&self, ncorners: usize) -> Result<Boxa> {
        if ncorners != 2 && ncorners != 4 {
            return Err(Error::InvalidParameter(format!(
                "ncorners must be 2 or 4, got {}",
                ncorners
            )));
        }
        if !self.len().is_multiple_of(ncorners) {
            return Err(Error::InvalidParameter(format!(
                "pta length {} is not a multiple of ncorners {}",
                self.len(),
                ncorners
            )));
        }

        let n_boxes = self.len() / ncorners;
        let mut boxa = Boxa::with_capacity(n_boxes);

        for i in 0..n_boxes {
            let base = i * ncorners;
            // Create a temporary Pta for this group of points
            let mut sub = Pta::with_capacity(ncorners);
            for j in 0..ncorners {
                let (x, y) = self.get(base + j).unwrap();
                sub.push(x, y);
            }
            let b = sub
                .to_box()
                .ok_or_else(|| Error::InvalidParameter("empty point group in pta".into()))?;
            boxa.push(b);
        }

        Ok(boxa)
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    // -- Box::relocate_one_side --

    #[test]
    fn test_relocate_one_side_left() {
        let b = Box::new(20, 30, 100, 80).unwrap();
        let r = b.relocate_one_side(Direction::FromLeft, 10);
        assert_eq!(r.x, 10);
        assert_eq!(r.w, 110); // w + (20 - 10) = 110
        assert_eq!(r.y, 30);
        assert_eq!(r.h, 80);
    }

    #[test]
    fn test_relocate_one_side_right() {
        let b = Box::new(20, 30, 100, 80).unwrap();
        let r = b.relocate_one_side(Direction::FromRight, 150);
        assert_eq!(r.x, 20);
        assert_eq!(r.w, 131); // 150 - 20 + 1
    }

    #[test]
    fn test_relocate_one_side_top() {
        let b = Box::new(20, 30, 100, 80).unwrap();
        let r = b.relocate_one_side(Direction::FromTop, 10);
        assert_eq!(r.y, 10);
        assert_eq!(r.h, 100); // 80 + (30 - 10) = 100
    }

    #[test]
    fn test_relocate_one_side_bottom() {
        let b = Box::new(20, 30, 100, 80).unwrap();
        let r = b.relocate_one_side(Direction::FromBottom, 150);
        assert_eq!(r.y, 30);
        assert_eq!(r.h, 121); // 150 - 30 + 1
    }

    // -- Box::adjust_sides --

    #[test]
    fn test_adjust_sides_expand() {
        let b = Box::new(20, 30, 100, 80).unwrap();
        let r = b.adjust_sides(-10, 10, -5, 5).unwrap();
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 25);
        assert_eq!(r.w, 120);
        assert_eq!(r.h, 90);
    }

    #[test]
    fn test_adjust_sides_clamp_to_zero() {
        let b = Box::new(5, 5, 100, 80).unwrap();
        let r = b.adjust_sides(-20, 0, -20, 0).unwrap();
        assert_eq!(r.x, 0); // clamped from -15
        assert_eq!(r.y, 0); // clamped from -15
    }

    #[test]
    fn test_adjust_sides_invalid() {
        let b = Box::new(20, 30, 10, 10).unwrap();
        assert!(b.adjust_sides(20, -20, 0, 0).is_none());
    }

    // -- Box::set_side --

    #[test]
    fn test_set_side_left() {
        let mut b = Box::new(20, 30, 100, 80).unwrap();
        b.set_side(Direction::FromLeft, 10, 0);
        assert_eq!(b.x, 10);
        assert_eq!(b.w, 110);
    }

    #[test]
    fn test_set_side_below_threshold() {
        let mut b = Box::new(20, 30, 100, 80).unwrap();
        b.set_side(Direction::FromLeft, 22, 5);
        // Diff is 2, below threshold 5, so no change
        assert_eq!(b.x, 20);
        assert_eq!(b.w, 100);
    }

    // -- Box::similar_per_side --

    #[test]
    fn test_similar_per_side_true() {
        let b1 = Box::new(10, 20, 100, 80).unwrap();
        let b2 = Box::new(12, 18, 102, 78).unwrap();
        assert!(b1.similar_per_side(&b2, 3, 5, 3, 5));
    }

    #[test]
    fn test_similar_per_side_false() {
        let b1 = Box::new(10, 20, 100, 80).unwrap();
        let b2 = Box::new(20, 20, 100, 80).unwrap();
        assert!(!b1.similar_per_side(&b2, 5, 5, 5, 5)); // left diff = 10
    }

    // -- Box::to_pta --

    #[test]
    fn test_box_to_pta_2_corners() {
        let b = Box::new(10, 20, 100, 80).unwrap();
        let pta = b.to_pta(2).unwrap();
        assert_eq!(pta.len(), 2);
        assert_eq!(pta.get(0), Some((10.0, 20.0)));
        assert_eq!(pta.get(1), Some((109.0, 99.0)));
    }

    #[test]
    fn test_box_to_pta_4_corners() {
        let b = Box::new(10, 20, 100, 80).unwrap();
        let pta = b.to_pta(4).unwrap();
        assert_eq!(pta.len(), 4);
        assert_eq!(pta.get(0), Some((10.0, 20.0))); // UL
        assert_eq!(pta.get(1), Some((109.0, 20.0))); // UR
        assert_eq!(pta.get(2), Some((10.0, 99.0))); // LL
        assert_eq!(pta.get(3), Some((109.0, 99.0))); // LR
    }

    #[test]
    fn test_box_to_pta_invalid() {
        let b = Box::new(10, 20, 100, 80).unwrap();
        assert!(b.to_pta(3).is_err());
    }

    // -- Boxa::adjust_all_sides --

    #[test]
    fn test_adjust_all_sides() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 10, 50, 50).unwrap());
        boxa.push(Box::new(100, 100, 50, 50).unwrap());

        let result = boxa.adjust_all_sides(-5, 5, -5, 5);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap().x, 5);
        assert_eq!(result.get(0).unwrap().w, 60);
    }

    // -- Boxa::adjust_box_sides --

    #[test]
    fn test_adjust_box_sides() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 10, 50, 50).unwrap());
        boxa.push(Box::new(100, 100, 50, 50).unwrap());

        boxa.adjust_box_sides(0, -5, 5, -5, 5).unwrap();
        assert_eq!(boxa.get(0).unwrap().x, 5);
        assert_eq!(boxa.get(0).unwrap().w, 60);
        // Box 1 unchanged
        assert_eq!(boxa.get(1).unwrap().x, 100);
    }

    // -- Boxa::set_all_sides --

    #[test]
    fn test_set_all_sides() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 20, 100, 80).unwrap());
        boxa.push(Box::new(30, 40, 100, 80).unwrap());

        boxa.set_all_sides(Direction::FromLeft, 0, 0);
        assert_eq!(boxa.get(0).unwrap().x, 0);
        assert_eq!(boxa.get(0).unwrap().w, 110);
        assert_eq!(boxa.get(1).unwrap().x, 0);
        assert_eq!(boxa.get(1).unwrap().w, 130);
    }

    // -- Boxa::adjust_width_to_target --

    #[test]
    fn test_adjust_width_to_target_end() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 10, 80, 50).unwrap());
        boxa.push(Box::new(20, 20, 120, 50).unwrap());

        let result = boxa.adjust_width_to_target(AdjustSide::End, 100, 0);
        assert_eq!(result.get(0).unwrap().x, 10);
        assert_eq!(result.get(0).unwrap().w, 100);
        assert_eq!(result.get(1).unwrap().x, 20);
        assert_eq!(result.get(1).unwrap().w, 100);
    }

    #[test]
    fn test_adjust_width_to_target_below_threshold() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 10, 98, 50).unwrap());

        let result = boxa.adjust_width_to_target(AdjustSide::End, 100, 5);
        // Diff is 2, below threshold 5 → no change
        assert_eq!(result.get(0).unwrap().w, 98);
    }

    // -- Boxa::adjust_height_to_target --

    #[test]
    fn test_adjust_height_to_target_both() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 10, 50, 80).unwrap());

        let result = boxa.adjust_height_to_target(AdjustSide::Both, 100, 0);
        let b = result.get(0).unwrap();
        assert_eq!(b.h, 100);
        // diff = 80 - 100 = -20, shift = -20/2 = -10, y = max(0, 10 + (-10)) = 0
        assert_eq!(b.y, 0);
    }

    // -- Boxa::equal_ordered --

    #[test]
    fn test_equal_ordered_exact() {
        let mut boxa1 = Boxa::new();
        boxa1.push(Box::new(0, 0, 10, 10).unwrap());
        boxa1.push(Box::new(20, 20, 10, 10).unwrap());

        let boxa2 = boxa1.clone();
        assert!(boxa1.equal_ordered(&boxa2, 0));
    }

    #[test]
    fn test_equal_ordered_different() {
        let mut boxa1 = Boxa::new();
        boxa1.push(Box::new(0, 0, 10, 10).unwrap());

        let mut boxa2 = Boxa::new();
        boxa2.push(Box::new(0, 0, 20, 20).unwrap());

        assert!(!boxa1.equal_ordered(&boxa2, 0));
    }

    // -- Boxa::split_even_odd --

    #[test]
    fn test_split_even_odd_no_fill() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 10, 10).unwrap()); // even
        boxa.push(Box::new(10, 10, 10, 10).unwrap()); // odd
        boxa.push(Box::new(20, 20, 10, 10).unwrap()); // even
        boxa.push(Box::new(30, 30, 10, 10).unwrap()); // odd

        let (even, odd) = boxa.split_even_odd(false);
        assert_eq!(even.len(), 2);
        assert_eq!(odd.len(), 2);
        assert_eq!(even.get(0).unwrap().x, 0);
        assert_eq!(even.get(1).unwrap().x, 20);
        assert_eq!(odd.get(0).unwrap().x, 10);
        assert_eq!(odd.get(1).unwrap().x, 30);
    }

    #[test]
    fn test_split_even_odd_with_fill() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 10, 10).unwrap());
        boxa.push(Box::new(10, 10, 10, 10).unwrap());
        boxa.push(Box::new(20, 20, 10, 10).unwrap());

        let (even, odd) = boxa.split_even_odd(true);
        assert_eq!(even.len(), 3);
        assert_eq!(odd.len(), 3);
        // Even: [box0, placeholder, box2]
        assert_eq!(even.get(0).unwrap().x, 0);
        assert_eq!(even.get(1).unwrap().w, 0); // placeholder
        assert_eq!(even.get(2).unwrap().x, 20);
    }

    // -- Boxa::merge_even_odd --

    #[test]
    fn test_merge_even_odd_no_fill() {
        let mut even = Boxa::new();
        even.push(Box::new_unchecked(0, 0, 10, 10));
        even.push(Box::new_unchecked(20, 20, 10, 10));

        let mut odd = Boxa::new();
        odd.push(Box::new_unchecked(10, 10, 10, 10));
        odd.push(Box::new_unchecked(30, 30, 10, 10));

        let merged = Boxa::merge_even_odd(&even, &odd, false).unwrap();
        assert_eq!(merged.len(), 4);
        assert_eq!(merged.get(0).unwrap().x, 0);
        assert_eq!(merged.get(1).unwrap().x, 10);
        assert_eq!(merged.get(2).unwrap().x, 20);
        assert_eq!(merged.get(3).unwrap().x, 30);
    }

    // -- Boxa::to_pta --

    #[test]
    fn test_boxa_to_pta() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 20, 100, 80).unwrap());
        boxa.push(Box::new(50, 60, 30, 40).unwrap());

        let pta = boxa.to_pta(2).unwrap();
        assert_eq!(pta.len(), 4); // 2 boxes × 2 corners
    }

    // -- Boxaa::join --

    #[test]
    fn test_boxaa_join() {
        let mut baa1 = Boxaa::new();
        baa1.push(Boxa::new());

        let mut baa2 = Boxaa::new();
        let mut b = Boxa::new();
        b.push(Box::new_unchecked(0, 0, 10, 10));
        baa2.push(b);

        baa1.join(&baa2, 0, 0);
        assert_eq!(baa1.len(), 2);
    }

    // -- Pta::to_box --

    #[test]
    fn test_pta_to_box() {
        let mut pta = Pta::new();
        pta.push(10.0, 20.0);
        pta.push(109.0, 99.0);

        let b = pta.to_box().unwrap();
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 20);
        assert_eq!(b.w, 100); // 109 - 10 + 1
        assert_eq!(b.h, 80); // 99 - 20 + 1
    }

    #[test]
    fn test_pta_to_box_empty() {
        let pta = Pta::new();
        assert!(pta.to_box().is_none());
    }

    // -- Pta::to_boxa --

    #[test]
    fn test_pta_to_boxa() {
        let mut pta = Pta::new();
        // Box 1: UL(10,20) LR(109,99)
        pta.push(10.0, 20.0);
        pta.push(109.0, 99.0);
        // Box 2: UL(50,60) LR(79,99)
        pta.push(50.0, 60.0);
        pta.push(79.0, 99.0);

        let boxa = pta.to_boxa(2).unwrap();
        assert_eq!(boxa.len(), 2);
        assert_eq!(boxa.get(0).unwrap().x, 10);
        assert_eq!(boxa.get(0).unwrap().w, 100);
        assert_eq!(boxa.get(1).unwrap().x, 50);
        assert_eq!(boxa.get(1).unwrap().w, 30);
    }

    #[test]
    fn test_pta_to_boxa_invalid_count() {
        let mut pta = Pta::new();
        pta.push(10.0, 20.0);
        pta.push(109.0, 99.0);
        pta.push(50.0, 60.0);

        assert!(pta.to_boxa(2).is_err()); // 3 points not divisible by 2
    }
}
