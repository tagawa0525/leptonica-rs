//! Box selection, permutation, and statistics operations
//!
//! Functions for range selection, indicator-based filtering,
//! permutation, and statistical extraction.
//!
//! C Leptonica equivalents: boxfunc4.c

use crate::error::{Error, Result};
use crate::numa::Numa;

use super::{
    Boxa, Boxaa, SizeRelation, compare_relation, compare_relation_f64, compare_relation_i64,
};

// ---- Types ----

/// Selection mode for size-based indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeSelectType {
    /// Select based on width only
    Width,
    /// Select based on height only
    Height,
    /// Select if either width or height satisfies the relation
    Either,
    /// Select only if both width and height satisfy the relation
    Both,
}

// ---- Boxa methods ----

impl Boxa {
    /// Select a range of boxes by index.
    ///
    /// Returns boxes from index `first` to `last` (inclusive).
    /// If `last` is 0, selects to the end.
    ///
    /// C Leptonica equivalent: `boxaSelectRange`
    pub fn select_range(&self, first: usize, last: usize) -> Boxa {
        let n = self.len();
        if n == 0 || first >= n {
            return Boxa::new();
        }
        let actual_last = if last == 0 { n - 1 } else { last.min(n - 1) };
        if first > actual_last {
            return Boxa::new();
        }
        self.boxes()[first..=actual_last].iter().copied().collect()
    }

    /// Generate a boolean indicator based on box dimensions.
    ///
    /// Each element is `true` if the box satisfies the size criteria.
    ///
    /// C Leptonica equivalent: `boxaMakeSizeIndicator`
    pub fn make_size_indicator(
        &self,
        width: i32,
        height: i32,
        select_type: SizeSelectType,
        relation: SizeRelation,
    ) -> Vec<bool> {
        self.boxes()
            .iter()
            .map(|b| {
                let w_match = compare_relation(b.w, width, relation);
                let h_match = compare_relation(b.h, height, relation);
                match select_type {
                    SizeSelectType::Width => w_match,
                    SizeSelectType::Height => h_match,
                    SizeSelectType::Either => w_match || h_match,
                    SizeSelectType::Both => w_match && h_match,
                }
            })
            .collect()
    }

    /// Generate a boolean indicator based on box area.
    ///
    /// C Leptonica equivalent: `boxaMakeAreaIndicator`
    pub fn make_area_indicator(&self, area: i64, relation: SizeRelation) -> Vec<bool> {
        self.boxes()
            .iter()
            .map(|b| compare_relation_i64(b.area(), area, relation))
            .collect()
    }

    /// Generate a boolean indicator based on width/height ratio.
    ///
    /// C Leptonica equivalent: `boxaMakeWHRatioIndicator`
    pub fn make_wh_ratio_indicator(&self, ratio: f64, relation: SizeRelation) -> Vec<bool> {
        self.boxes()
            .iter()
            .map(|b| {
                if b.h == 0 {
                    return false;
                }
                let r = b.w as f64 / b.h as f64;
                compare_relation_f64(r, ratio, relation)
            })
            .collect()
    }

    /// Filter boxes using a boolean indicator array.
    ///
    /// Returns boxes where the indicator is `true`.
    ///
    /// C Leptonica equivalent: `boxaSelectWithIndicator`
    pub fn select_with_indicator(&self, indicator: &[bool]) -> Boxa {
        self.boxes()
            .iter()
            .zip(indicator.iter())
            .filter(|(_, ind)| **ind)
            .map(|(b, _)| *b)
            .collect()
    }

    /// Swap two boxes at the given indices.
    ///
    /// C Leptonica equivalent: `boxaSwapBoxes`
    pub fn swap_boxes(&mut self, i: usize, j: usize) -> Result<()> {
        let n = self.len();
        if i >= n || j >= n {
            return Err(Error::IndexOutOfBounds {
                index: i.max(j),
                len: n,
            });
        }
        self.boxes_mut().swap(i, j);
        Ok(())
    }

    /// Get the range of box positions (upper-left corners).
    ///
    /// Returns `(min_x, min_y, max_x, max_y)`.
    ///
    /// C Leptonica equivalent: `boxaLocationRange`
    pub fn location_range(&self) -> Option<(i32, i32, i32, i32)> {
        if self.is_empty() {
            return None;
        }
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        for b in self.boxes() {
            min_x = min_x.min(b.x);
            min_y = min_y.min(b.y);
            max_x = max_x.max(b.x);
            max_y = max_y.max(b.y);
        }
        Some((min_x, min_y, max_x, max_y))
    }

    /// Extract widths and heights as separate Numa arrays.
    ///
    /// Returns `(widths, heights)`.
    ///
    /// C Leptonica equivalent: `boxaGetSizes`
    pub fn get_sizes(&self) -> (Numa, Numa) {
        let mut widths = Numa::with_capacity(self.len());
        let mut heights = Numa::with_capacity(self.len());
        for b in self.boxes() {
            widths.push(b.w as f32);
            heights.push(b.h as f32);
        }
        (widths, heights)
    }

    /// Compute the total area of all boxes (sum of w*h).
    ///
    /// Does not account for overlaps.
    ///
    /// C Leptonica equivalent: `boxaGetArea`
    pub fn get_total_area(&self) -> i64 {
        self.boxes().iter().map(|b| b.area()).sum()
    }
}

// ---- Boxaa methods ----

impl Boxaa {
    /// Select a range of Boxa by index.
    ///
    /// Returns Boxas from index `first` to `last` (inclusive).
    /// If `last` is 0, selects to the end.
    ///
    /// C Leptonica equivalent: `boxaaSelectRange`
    pub fn select_range(&self, first: usize, last: usize) -> Boxaa {
        let n = self.len();
        if n == 0 || first >= n {
            return Boxaa::new();
        }
        let actual_last = if last == 0 { n - 1 } else { last.min(n - 1) };
        if first > actual_last {
            return Boxaa::new();
        }
        let mut result = Boxaa::with_capacity(actual_last - first + 1);
        for boxa in &self.boxas()[first..=actual_last] {
            result.push(boxa.clone());
        }
        result
    }

    /// Get the range of box dimensions across all Boxa.
    ///
    /// Returns `(min_w, max_w, min_h, max_h)`.
    ///
    /// C Leptonica equivalent: `boxaaSizeRange`
    pub fn size_range(&self) -> Option<(i32, i32, i32, i32)> {
        let mut min_w = i32::MAX;
        let mut max_w = i32::MIN;
        let mut min_h = i32::MAX;
        let mut max_h = i32::MIN;
        let mut found = false;
        for boxa in self.boxas() {
            for b in boxa.boxes() {
                found = true;
                min_w = min_w.min(b.w);
                max_w = max_w.max(b.w);
                min_h = min_h.min(b.h);
                max_h = max_h.max(b.h);
            }
        }
        if found {
            Some((min_w, max_w, min_h, max_h))
        } else {
            None
        }
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_::Box;

    fn sample_boxa() -> Boxa {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 20, 100, 50).unwrap());
        boxa.push(Box::new(30, 40, 200, 80).unwrap());
        boxa.push(Box::new(50, 60, 50, 150).unwrap());
        boxa.push(Box::new(70, 80, 300, 200).unwrap());
        boxa
    }

    // -- Boxa::select_range --

    #[test]
    fn test_select_range() {
        let boxa = sample_boxa();
        let result = boxa.select_range(1, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap().x, 30);
        assert_eq!(result.get(1).unwrap().x, 50);
    }

    #[test]
    fn test_select_range_to_end() {
        let boxa = sample_boxa();
        let result = boxa.select_range(2, 0);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap().x, 50);
    }

    // -- Boxa::make_size_indicator --

    #[test]
    fn test_make_size_indicator_width_gt() {
        let boxa = sample_boxa();
        let ind =
            boxa.make_size_indicator(100, 0, SizeSelectType::Width, SizeRelation::GreaterThan);
        assert_eq!(ind, vec![false, true, false, true]);
    }

    #[test]
    fn test_make_size_indicator_both_gt() {
        let boxa = sample_boxa();
        let ind =
            boxa.make_size_indicator(100, 100, SizeSelectType::Both, SizeRelation::GreaterThan);
        // Only box 3 (300x200) has both > 100
        assert_eq!(ind, vec![false, false, false, true]);
    }

    #[test]
    fn test_make_size_indicator_either_gt() {
        let boxa = sample_boxa();
        let ind =
            boxa.make_size_indicator(100, 100, SizeSelectType::Either, SizeRelation::GreaterThan);
        // Box 1 (200x80): w>100, Box 2 (50x150): h>100, Box 3 (300x200): both
        assert_eq!(ind, vec![false, true, true, true]);
    }

    // -- Boxa::make_area_indicator --

    #[test]
    fn test_make_area_indicator() {
        let boxa = sample_boxa();
        let ind = boxa.make_area_indicator(10000, SizeRelation::GreaterThan);
        // Areas: 5000, 16000, 7500, 60000
        assert_eq!(ind, vec![false, true, false, true]);
    }

    // -- Boxa::make_wh_ratio_indicator --

    #[test]
    fn test_make_wh_ratio_indicator() {
        let boxa = sample_boxa();
        let ind = boxa.make_wh_ratio_indicator(1.0, SizeRelation::GreaterThan);
        // Ratios: 2.0, 2.5, 0.33, 1.5
        assert_eq!(ind, vec![true, true, false, true]);
    }

    // -- Boxa::select_with_indicator --

    #[test]
    fn test_select_with_indicator() {
        let boxa = sample_boxa();
        let ind = vec![true, false, true, false];
        let result = boxa.select_with_indicator(&ind);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap().x, 10);
        assert_eq!(result.get(1).unwrap().x, 50);
    }

    // -- Boxa::swap_boxes --

    #[test]
    fn test_swap_boxes() {
        let mut boxa = sample_boxa();
        boxa.swap_boxes(0, 3).unwrap();
        assert_eq!(boxa.get(0).unwrap().x, 70);
        assert_eq!(boxa.get(3).unwrap().x, 10);
    }

    #[test]
    fn test_swap_boxes_invalid() {
        let mut boxa = sample_boxa();
        assert!(boxa.swap_boxes(0, 10).is_err());
    }

    // -- Boxa::location_range --

    #[test]
    fn test_location_range() {
        let boxa = sample_boxa();
        let (min_x, min_y, max_x, max_y) = boxa.location_range().unwrap();
        assert_eq!(min_x, 10);
        assert_eq!(min_y, 20);
        assert_eq!(max_x, 70);
        assert_eq!(max_y, 80);
    }

    #[test]
    fn test_location_range_empty() {
        let boxa = Boxa::new();
        assert!(boxa.location_range().is_none());
    }

    // -- Boxa::get_sizes --

    #[test]
    fn test_get_sizes() {
        let boxa = sample_boxa();
        let (widths, heights) = boxa.get_sizes();
        assert_eq!(widths.len(), 4);
        assert_eq!(widths.get_i32(0), Some(100));
        assert_eq!(widths.get_i32(1), Some(200));
        assert_eq!(heights.get_i32(0), Some(50));
        assert_eq!(heights.get_i32(3), Some(200));
    }

    // -- Boxa::get_total_area --

    #[test]
    fn test_get_total_area() {
        let boxa = sample_boxa();
        // 5000 + 16000 + 7500 + 60000 = 88500
        assert_eq!(boxa.get_total_area(), 88500);
    }

    // -- Boxaa::select_range --

    #[test]
    fn test_boxaa_select_range() {
        let mut baa = Boxaa::new();
        baa.push(Boxa::new());
        baa.push(Boxa::new());
        baa.push(Boxa::new());

        let result = baa.select_range(0, 1);
        assert_eq!(result.len(), 2);
    }

    // -- Boxaa::size_range --

    #[test]
    fn test_boxaa_size_range() {
        let mut baa = Boxaa::new();
        let mut b1 = Boxa::new();
        b1.push(Box::new(0, 0, 10, 20).unwrap());
        let mut b2 = Boxa::new();
        b2.push(Box::new(0, 0, 30, 5).unwrap());
        baa.push(b1);
        baa.push(b2);

        let (min_w, max_w, min_h, max_h) = baa.size_range().unwrap();
        assert_eq!(min_w, 10);
        assert_eq!(max_w, 30);
        assert_eq!(min_h, 5);
        assert_eq!(max_h, 20);
    }
}
