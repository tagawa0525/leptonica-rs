//! Box, Boxa, Boxaa - Rectangle regions
//!
//! These structures represent rectangular regions in an image.

use crate::error::{Error, Result};

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
            self.boxes.sort_by(|a, b| b.area().cmp(&a.area()));
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
