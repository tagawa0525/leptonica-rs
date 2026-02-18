//! Box, Boxa, Boxaa - Rectangle regions
//!
//! These structures represent rectangular regions in an image.

pub mod serial;

use crate::error::{Error, Result};

/// Size comparison relation for selection functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeRelation {
    /// Select if value < threshold
    LessThan,
    /// Select if value <= threshold
    LessThanOrEqual,
    /// Select if value > threshold
    GreaterThan,
    /// Select if value >= threshold
    GreaterThanOrEqual,
}

/// A rectangle region
///
/// Unlike Leptonica's Box which uses reference counting, this is a simple
/// Copy type since it's small and frequently copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Box {
    /// Left x coordinate
    pub x: i32,
    /// Top y coordinate
    pub y: i32,
    /// Width
    pub w: i32,
    /// Height
    pub h: i32,
}

impl Box {
    /// Create a new box
    ///
    /// # Errors
    ///
    /// Returns an error if width or height is negative.
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Result<Self> {
        if w < 0 || h < 0 {
            return Err(Error::InvalidParameter(format!(
                "box dimensions must be non-negative: w={}, h={}",
                w, h
            )));
        }
        Ok(Self { x, y, w, h })
    }

    /// Create a box without validation
    pub const fn new_unchecked(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    /// Create a box from two corner points
    pub fn from_corners(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        let (x, w) = if x1 <= x2 {
            (x1, x2 - x1)
        } else {
            (x2, x1 - x2)
        };
        let (y, h) = if y1 <= y2 {
            (y1, y2 - y1)
        } else {
            (y2, y1 - y2)
        };
        Self { x, y, w, h }
    }

    /// Get the right x coordinate (exclusive)
    #[inline]
    pub fn right(&self) -> i32 {
        self.x + self.w
    }

    /// Get the bottom y coordinate (exclusive)
    #[inline]
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }

    /// Get the center x coordinate
    #[inline]
    pub fn center_x(&self) -> i32 {
        self.x + self.w / 2
    }

    /// Get the center y coordinate
    #[inline]
    pub fn center_y(&self) -> i32 {
        self.y + self.h / 2
    }

    /// Get the area
    #[inline]
    pub fn area(&self) -> i64 {
        self.w as i64 * self.h as i64
    }

    /// Check if the box is valid (non-negative dimensions)
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.w >= 0 && self.h >= 0
    }

    /// Check if the box is empty (zero area)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    /// Check if a point is inside the box
    #[inline]
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Check if this box contains another box
    pub fn contains_box(&self, other: &Box) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// Check if this box overlaps with another
    pub fn overlaps(&self, other: &Box) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Compute the intersection of two boxes
    pub fn intersect(&self, other: &Box) -> Option<Box> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if x < right && y < bottom {
            Some(Box {
                x,
                y,
                w: right - x,
                h: bottom - y,
            })
        } else {
            None
        }
    }

    /// Compute the union (bounding box) of two boxes
    pub fn union(&self, other: &Box) -> Box {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());

        Box {
            x,
            y,
            w: right - x,
            h: bottom - y,
        }
    }

    /// Translate the box by (dx, dy)
    pub fn translate(&self, dx: i32, dy: i32) -> Box {
        Box {
            x: self.x + dx,
            y: self.y + dy,
            w: self.w,
            h: self.h,
        }
    }

    /// Scale the box by a factor
    pub fn scale(&self, factor: f32) -> Box {
        Box {
            x: (self.x as f32 * factor).round() as i32,
            y: (self.y as f32 * factor).round() as i32,
            w: (self.w as f32 * factor).round() as i32,
            h: (self.h as f32 * factor).round() as i32,
        }
    }

    /// Expand the box by a margin on all sides
    pub fn expand(&self, margin: i32) -> Box {
        Box {
            x: self.x - margin,
            y: self.y - margin,
            w: self.w + 2 * margin,
            h: self.h + 2 * margin,
        }
    }

    /// Compute the area of overlap between two boxes
    ///
    /// Returns the area of intersection, or 0 if boxes don't overlap.
    ///
    /// C Leptonica equivalent: `boxOverlapArea`
    pub fn overlap_area(&self, other: &Box) -> i64 {
        match self.intersect(other) {
            Some(inter) => inter.area(),
            None => 0,
        }
    }

    /// Compute the fraction of this box that overlaps with another
    ///
    /// Returns the intersection area divided by this box's area.
    /// Returns 0.0 if this box has zero area or there is no overlap.
    ///
    /// C Leptonica equivalent: `boxOverlapFraction`
    pub fn overlap_fraction(&self, other: &Box) -> f64 {
        let self_area = self.area();
        if self_area == 0 {
            return 0.0;
        }
        self.overlap_area(other) as f64 / self_area as f64
    }

    /// Clip the box to fit within bounds
    pub fn clip(&self, width: i32, height: i32) -> Option<Box> {
        let x = self.x.max(0);
        let y = self.y.max(0);
        let right = self.right().min(width);
        let bottom = self.bottom().min(height);

        if x < right && y < bottom {
            Some(Box {
                x,
                y,
                w: right - x,
                h: bottom - y,
            })
        } else {
            None
        }
    }
}

/// Array of boxes
#[derive(Debug, Clone, Default)]
pub struct Boxa {
    boxes: Vec<Box>,
}

impl Boxa {
    /// Create a new empty Boxa
    pub fn new() -> Self {
        Self { boxes: Vec::new() }
    }

    /// Create a Boxa with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            boxes: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of boxes
    #[inline]
    pub fn len(&self) -> usize {
        self.boxes.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    /// Get a box by index
    pub fn get(&self, index: usize) -> Option<&Box> {
        self.boxes.get(index)
    }

    /// Get a mutable box by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Box> {
        self.boxes.get_mut(index)
    }

    /// Add a box
    pub fn push(&mut self, b: Box) {
        self.boxes.push(b);
    }

    /// Remove and return the last box
    pub fn pop(&mut self) -> Option<Box> {
        self.boxes.pop()
    }

    /// Remove a box at index
    pub fn remove(&mut self, index: usize) -> Result<Box> {
        if index >= self.boxes.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.boxes.len(),
            });
        }
        Ok(self.boxes.remove(index))
    }

    /// Insert a box at index
    pub fn insert(&mut self, index: usize, b: Box) -> Result<()> {
        if index > self.boxes.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.boxes.len(),
            });
        }
        self.boxes.insert(index, b);
        Ok(())
    }

    /// Replace a box at index
    pub fn replace(&mut self, index: usize, b: Box) -> Result<Box> {
        if index >= self.boxes.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.boxes.len(),
            });
        }
        Ok(std::mem::replace(&mut self.boxes[index], b))
    }

    /// Clear all boxes
    pub fn clear(&mut self) {
        self.boxes.clear();
    }

    /// Get all boxes as a slice
    pub fn boxes(&self) -> &[Box] {
        &self.boxes
    }

    /// Get all boxes as a mutable slice
    pub fn boxes_mut(&mut self) -> &mut [Box] {
        &mut self.boxes
    }

    /// Compute the bounding box of all boxes
    pub fn bounding_box(&self) -> Option<Box> {
        if self.boxes.is_empty() {
            return None;
        }

        let mut x1 = i32::MAX;
        let mut y1 = i32::MAX;
        let mut x2 = i32::MIN;
        let mut y2 = i32::MIN;

        for b in &self.boxes {
            x1 = x1.min(b.x);
            y1 = y1.min(b.y);
            x2 = x2.max(b.right());
            y2 = y2.max(b.bottom());
        }

        Some(Box {
            x: x1,
            y: y1,
            w: x2 - x1,
            h: y2 - y1,
        })
    }

    /// Sort boxes by position (top-to-bottom, left-to-right)
    pub fn sort_by_position(&mut self) {
        self.boxes.sort_by(|a, b| {
            let y_cmp = a.y.cmp(&b.y);
            if y_cmp == std::cmp::Ordering::Equal {
                a.x.cmp(&b.x)
            } else {
                y_cmp
            }
        });
    }

    /// Sort boxes by area
    pub fn sort_by_area(&mut self, ascending: bool) {
        if ascending {
            self.boxes.sort_by_key(|b| b.area());
        } else {
            self.boxes.sort_by_key(|b| std::cmp::Reverse(b.area()));
        }
    }

    /// Create an iterator over boxes
    pub fn iter(&self) -> impl Iterator<Item = &Box> {
        self.boxes.iter()
    }

    /// Create a mutable iterator over boxes
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box> {
        self.boxes.iter_mut()
    }

    /// Filter boxes contained within a given box
    ///
    /// Returns a new Boxa containing only boxes fully inside `container`.
    ///
    /// C Leptonica equivalent: `boxaContainedInBox`
    pub fn contained_in_box(&self, container: &Box) -> Boxa {
        self.boxes
            .iter()
            .filter(|b| container.contains_box(b))
            .copied()
            .collect()
    }

    /// Filter boxes that intersect with a given box
    ///
    /// Returns a new Boxa containing only boxes that overlap with `target`.
    ///
    /// C Leptonica equivalent: `boxaIntersectsBox`
    pub fn intersects_box(&self, target: &Box) -> Boxa {
        self.boxes
            .iter()
            .filter(|b| b.overlaps(target))
            .copied()
            .collect()
    }

    /// Clip all boxes to fit within a given box
    ///
    /// Returns a new Boxa where each box is clipped to the bounds of `clip_box`.
    /// Boxes that don't intersect `clip_box` are omitted.
    ///
    /// C Leptonica equivalent: `boxaClipToBox`
    pub fn clip_to_box(&self, clip_box: &Box) -> Boxa {
        self.boxes
            .iter()
            .filter_map(|b| b.intersect(clip_box))
            .collect()
    }

    /// Combine overlapping boxes into their unions
    ///
    /// Iteratively merges any pair of overlapping boxes until no overlaps remain.
    ///
    /// C Leptonica equivalent: `boxaCombineOverlaps`
    pub fn combine_overlaps(&self) -> Boxa {
        if self.boxes.is_empty() {
            return Boxa::new();
        }

        let mut result: Vec<Box> = self.boxes.clone();
        let mut changed = true;

        while changed {
            changed = false;
            let mut i = 0;
            while i < result.len() {
                let mut j = i + 1;
                while j < result.len() {
                    if result[i].overlaps(&result[j]) {
                        result[i] = result[i].union(&result[j]);
                        result.remove(j);
                        changed = true;
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }

        Boxa { boxes: result }
    }

    /// Select boxes by width and height
    ///
    /// Filters boxes where both width and height satisfy the relation
    /// against the given thresholds.
    ///
    /// C Leptonica equivalent: `boxaSelectBySize`
    pub fn select_by_size(&self, width: i32, height: i32, relation: SizeRelation) -> Boxa {
        self.boxes
            .iter()
            .filter(|b| {
                compare_relation(b.w, width, relation) && compare_relation(b.h, height, relation)
            })
            .copied()
            .collect()
    }

    /// Select boxes by area
    ///
    /// Filters boxes based on an area threshold and comparison relation.
    ///
    /// C Leptonica equivalent: `boxaSelectByArea`
    pub fn select_by_area(&self, area: i64, relation: SizeRelation) -> Boxa {
        self.boxes
            .iter()
            .filter(|b| compare_relation_i64(b.area(), area, relation))
            .copied()
            .collect()
    }

    /// Select boxes by width/height ratio
    ///
    /// Filters boxes based on a w/h ratio threshold and comparison relation.
    /// Boxes with zero height are excluded.
    ///
    /// C Leptonica equivalent: `boxaSelectByWHRatio`
    pub fn select_by_wh_ratio(&self, ratio: f64, relation: SizeRelation) -> Boxa {
        self.boxes
            .iter()
            .filter(|b| {
                if b.h == 0 {
                    return false;
                }
                let r = b.w as f64 / b.h as f64;
                compare_relation_f64(r, ratio, relation)
            })
            .copied()
            .collect()
    }

    /// Get the extent (overall width and height) of all boxes
    ///
    /// Returns `(width, height, bounding_box)` where width and height
    /// are the extent from the origin (0,0) to the furthest right/bottom edge.
    ///
    /// C Leptonica equivalent: `boxaGetExtent`
    pub fn get_extent(&self) -> Option<(i32, i32, Box)> {
        let bb = self.bounding_box()?;
        let max_right = self.boxes.iter().map(|b| b.right()).max().unwrap_or(0);
        let max_bottom = self.boxes.iter().map(|b| b.bottom()).max().unwrap_or(0);
        Some((max_right, max_bottom, bb))
    }

    /// Compute the fractional coverage of boxes within a canvas
    ///
    /// Returns the fraction of the canvas area covered by the union of all boxes.
    /// When `exact` is true, uses per-pixel row scanning for exact coverage.
    /// When false, uses the sum of individual box areas (may overcount overlaps).
    ///
    /// C Leptonica equivalent: `boxaGetCoverage`
    pub fn get_coverage(&self, canvas_w: i32, canvas_h: i32, exact: bool) -> f64 {
        let canvas_area = canvas_w as i64 * canvas_h as i64;
        if canvas_area == 0 || self.boxes.is_empty() {
            return 0.0;
        }

        if !exact {
            // Approximate: sum of individual areas (may overcount)
            let total_area: i64 = self.boxes.iter().map(|b| b.area()).sum();
            return (total_area as f64 / canvas_area as f64).min(1.0);
        }

        // Exact: count covered pixels per row using interval merging
        let mut covered_pixels: i64 = 0;
        for y in 0..canvas_h {
            // Collect all horizontal intervals on this row
            let mut intervals: Vec<(i32, i32)> = Vec::new();
            for b in &self.boxes {
                if y >= b.y && y < b.bottom() {
                    let left = b.x.max(0);
                    let right = b.right().min(canvas_w);
                    if left < right {
                        intervals.push((left, right));
                    }
                }
            }
            if intervals.is_empty() {
                continue;
            }
            // Merge overlapping intervals
            intervals.sort_unstable();
            let mut merged_left = intervals[0].0;
            let mut merged_right = intervals[0].1;
            for &(l, r) in &intervals[1..] {
                if l <= merged_right {
                    merged_right = merged_right.max(r);
                } else {
                    covered_pixels += (merged_right - merged_left) as i64;
                    merged_left = l;
                    merged_right = r;
                }
            }
            covered_pixels += (merged_right - merged_left) as i64;
        }

        covered_pixels as f64 / canvas_area as f64
    }

    /// Get the range of box dimensions
    ///
    /// Returns `(min_w, min_h, max_w, max_h)`.
    ///
    /// C Leptonica equivalent: `boxaSizeRange`
    pub fn size_range(&self) -> Option<(i32, i32, i32, i32)> {
        if self.boxes.is_empty() {
            return None;
        }
        let mut min_w = i32::MAX;
        let mut min_h = i32::MAX;
        let mut max_w = i32::MIN;
        let mut max_h = i32::MIN;
        for b in &self.boxes {
            min_w = min_w.min(b.w);
            min_h = min_h.min(b.h);
            max_w = max_w.max(b.w);
            max_h = max_h.max(b.h);
        }
        Some((min_w, min_h, max_w, max_h))
    }

    /// Check if two Boxa are similar within tolerances
    ///
    /// Two Boxa are similar if they have the same number of boxes and each
    /// corresponding pair differs by no more than the given tolerance in
    /// x, y, w, h respectively.
    ///
    /// C Leptonica equivalent: `boxaSimilar`
    pub fn similar(&self, other: &Boxa, tolerance: i32) -> bool {
        if self.boxes.len() != other.boxes.len() {
            return false;
        }
        self.boxes.iter().zip(other.boxes.iter()).all(|(a, b)| {
            (a.x - b.x).abs() <= tolerance
                && (a.y - b.y).abs() <= tolerance
                && (a.w - b.w).abs() <= tolerance
                && (a.h - b.h).abs() <= tolerance
        })
    }

    /// Append boxes from another Boxa
    ///
    /// Appends boxes from `other` in the range `[start, end)`.
    /// If `end` is 0, appends all boxes from `start` onwards.
    ///
    /// C Leptonica equivalent: `boxaJoin`
    pub fn join(&mut self, other: &Boxa, start: usize, end: usize) {
        let actual_end = if end == 0 {
            other.boxes.len()
        } else {
            end.min(other.boxes.len())
        };
        let actual_start = start.min(actual_end);
        self.boxes
            .extend_from_slice(&other.boxes[actual_start..actual_end]);
    }
}

/// Helper: compare two i32 values using SizeRelation
pub(crate) fn compare_relation(value: i32, threshold: i32, relation: SizeRelation) -> bool {
    match relation {
        SizeRelation::LessThan => value < threshold,
        SizeRelation::LessThanOrEqual => value <= threshold,
        SizeRelation::GreaterThan => value > threshold,
        SizeRelation::GreaterThanOrEqual => value >= threshold,
    }
}

/// Helper: compare two i64 values using SizeRelation
pub(crate) fn compare_relation_i64(value: i64, threshold: i64, relation: SizeRelation) -> bool {
    match relation {
        SizeRelation::LessThan => value < threshold,
        SizeRelation::LessThanOrEqual => value <= threshold,
        SizeRelation::GreaterThan => value > threshold,
        SizeRelation::GreaterThanOrEqual => value >= threshold,
    }
}

/// Helper: compare two f64 values using SizeRelation
fn compare_relation_f64(value: f64, threshold: f64, relation: SizeRelation) -> bool {
    match relation {
        SizeRelation::LessThan => value < threshold,
        SizeRelation::LessThanOrEqual => value <= threshold,
        SizeRelation::GreaterThan => value > threshold,
        SizeRelation::GreaterThanOrEqual => value >= threshold,
    }
}

impl FromIterator<Box> for Boxa {
    fn from_iter<T: IntoIterator<Item = Box>>(iter: T) -> Self {
        Self {
            boxes: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for Boxa {
    type Item = Box;
    type IntoIter = std::vec::IntoIter<Box>;

    fn into_iter(self) -> Self::IntoIter {
        self.boxes.into_iter()
    }
}

impl<'a> IntoIterator for &'a Boxa {
    type Item = &'a Box;
    type IntoIter = std::slice::Iter<'a, Box>;

    fn into_iter(self) -> Self::IntoIter {
        self.boxes.iter()
    }
}

/// Array of Boxa
#[derive(Debug, Clone, Default)]
pub struct Boxaa {
    boxas: Vec<Boxa>,
}

impl Boxaa {
    /// Create a new empty Boxaa
    pub fn new() -> Self {
        Self { boxas: Vec::new() }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            boxas: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of Boxa
    #[inline]
    pub fn len(&self) -> usize {
        self.boxas.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.boxas.is_empty()
    }

    /// Get a Boxa by index
    pub fn get(&self, index: usize) -> Option<&Boxa> {
        self.boxas.get(index)
    }

    /// Get a mutable Boxa by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Boxa> {
        self.boxas.get_mut(index)
    }

    /// Add a Boxa
    pub fn push(&mut self, boxa: Boxa) {
        self.boxas.push(boxa);
    }

    /// Remove and return the last Boxa
    pub fn pop(&mut self) -> Option<Boxa> {
        self.boxas.pop()
    }

    /// Clear all Boxa
    pub fn clear(&mut self) {
        self.boxas.clear();
    }

    /// Get all Boxa as a slice
    pub fn boxas(&self) -> &[Boxa] {
        &self.boxas
    }

    /// Get total number of boxes across all Boxa
    pub fn total_boxes(&self) -> usize {
        self.boxas.iter().map(|b| b.len()).sum()
    }

    /// Flatten into a single Boxa
    pub fn flatten(&self) -> Boxa {
        let total = self.total_boxes();
        let mut result = Boxa::with_capacity(total);
        for boxa in &self.boxas {
            for b in boxa.iter() {
                result.push(*b);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_creation() {
        let b = Box::new(10, 20, 100, 50).unwrap();
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 20);
        assert_eq!(b.w, 100);
        assert_eq!(b.h, 50);
        assert_eq!(b.right(), 110);
        assert_eq!(b.bottom(), 70);
        assert_eq!(b.area(), 5000);

        assert!(Box::new(0, 0, -1, 10).is_err());
    }

    #[test]
    fn test_box_from_corners() {
        let b = Box::from_corners(100, 100, 0, 0);
        assert_eq!(b.x, 0);
        assert_eq!(b.y, 0);
        assert_eq!(b.w, 100);
        assert_eq!(b.h, 100);
    }

    #[test]
    fn test_box_contains() {
        let b = Box::new(10, 10, 100, 100).unwrap();
        assert!(b.contains_point(50, 50));
        assert!(b.contains_point(10, 10));
        assert!(!b.contains_point(110, 110)); // Exclusive boundary
        assert!(!b.contains_point(0, 0));
    }

    #[test]
    fn test_box_intersect() {
        let b1 = Box::new(0, 0, 100, 100).unwrap();
        let b2 = Box::new(50, 50, 100, 100).unwrap();

        let intersection = b1.intersect(&b2).unwrap();
        assert_eq!(intersection.x, 50);
        assert_eq!(intersection.y, 50);
        assert_eq!(intersection.w, 50);
        assert_eq!(intersection.h, 50);

        let b3 = Box::new(200, 200, 10, 10).unwrap();
        assert!(b1.intersect(&b3).is_none());
    }

    #[test]
    fn test_box_union() {
        let b1 = Box::new(0, 0, 50, 50).unwrap();
        let b2 = Box::new(25, 25, 50, 50).unwrap();

        let union = b1.union(&b2);
        assert_eq!(union.x, 0);
        assert_eq!(union.y, 0);
        assert_eq!(union.w, 75);
        assert_eq!(union.h, 75);
    }

    #[test]
    fn test_boxa() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 10, 10).unwrap());
        boxa.push(Box::new(20, 20, 10, 10).unwrap());

        assert_eq!(boxa.len(), 2);

        let bb = boxa.bounding_box().unwrap();
        assert_eq!(bb.x, 0);
        assert_eq!(bb.y, 0);
        assert_eq!(bb.w, 30);
        assert_eq!(bb.h, 30);
    }

    #[test]
    fn test_boxa_sort() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(100, 100, 10, 10).unwrap());
        boxa.push(Box::new(0, 0, 10, 10).unwrap());
        boxa.push(Box::new(50, 0, 10, 10).unwrap());

        boxa.sort_by_position();

        assert_eq!(boxa.get(0).unwrap().x, 0);
        assert_eq!(boxa.get(1).unwrap().x, 50);
        assert_eq!(boxa.get(2).unwrap().x, 100);
    }
}
