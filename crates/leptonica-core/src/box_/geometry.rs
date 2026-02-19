//! Box geometry and relationship operations
//!
//! Functions for computing distances, intersections, nearest neighbors,
//! and overlap handling between boxes.
//!
//! C Leptonica equivalents: boxfunc1.c

use crate::error::{Error, Result};

use super::{Box, Boxa};

// ---- Types ----

/// Size comparison metric for comparing two boxes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeComparisonType {
    /// Compare by width
    Width,
    /// Compare by height
    Height,
    /// Compare by max(width, height)
    MaxDimension,
    /// Compare by perimeter (w + h)
    Perimeter,
    /// Compare by area (w * h)
    Area,
}

/// Direction for nearest-box search
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Search for boxes to the left
    FromLeft,
    /// Search for boxes to the right
    FromRight,
    /// Search for boxes above
    FromTop,
    /// Search for boxes below
    FromBottom,
}

/// Distance selection filter for nearest-box search
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistSelect {
    /// Only consider non-overlapping boxes (distance >= 0)
    NonNegative,
    /// Consider all boxes including overlapping ones
    All,
}

/// Operation type for handling overlaps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlapOp {
    /// Combine overlapping boxes into their union
    Combine,
    /// Remove the smaller of overlapping boxes
    RemoveSmall,
}

/// Result of a line-box intersection computation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineIntersection {
    /// First intersection point (if any)
    pub p1: Option<(i32, i32)>,
    /// Second intersection point (if any)
    pub p2: Option<(i32, i32)>,
    /// Number of intersection points (0, 1, or 2)
    pub count: usize,
}

/// Clipping parameters for a box within a rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipParams {
    /// Start x (inclusive)
    pub x_start: i32,
    /// Start y (inclusive)
    pub y_start: i32,
    /// End x (exclusive)
    pub x_end: i32,
    /// End y (exclusive)
    pub y_end: i32,
    /// Clipped width
    pub width: i32,
    /// Clipped height
    pub height: i32,
}

/// Result of a nearest-by-direction search for a single box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NearestResult {
    /// Index of the nearest box (None if not found)
    pub index: Option<usize>,
    /// Distance to the nearest box (i32::MAX if not found)
    pub distance: i32,
}

// ---- Box methods ----

impl Box {
    /// Compute horizontal and vertical overlap distances between two boxes.
    ///
    /// Returns `(h_overlap, v_overlap)` where:
    /// - Positive = overlap extent
    /// - Zero = touching
    /// - Negative = gap between boxes
    ///
    /// C Leptonica equivalent: `boxOverlapDistance`
    pub fn overlap_distance(&self, other: &Box) -> (i32, i32) {
        todo!()
    }

    /// Compute horizontal and vertical separation distances between two boxes.
    ///
    /// Returns `(h_sep, v_sep)` where:
    /// - 0 = overlapping in that dimension
    /// - Positive = gap distance + 1
    ///
    /// C Leptonica equivalent: `boxSeparationDistance`
    pub fn separation_distance(&self, other: &Box) -> (i32, i32) {
        todo!()
    }

    /// Compare two boxes by a size metric.
    ///
    /// Returns `Ordering::Greater` if self > other, `Ordering::Less` if self < other,
    /// `Ordering::Equal` if equal.
    ///
    /// C Leptonica equivalent: `boxCompareSize`
    pub fn compare_size(&self, other: &Box, cmp_type: SizeComparisonType) -> std::cmp::Ordering {
        todo!()
    }

    /// Find intersection points of a line with this box's boundary.
    ///
    /// The line is defined by a point `(x, y)` and a `slope`.
    /// Use `slope > 1_000_000.0` for vertical lines.
    ///
    /// C Leptonica equivalent: `boxIntersectByLine`
    pub fn intersect_by_line(&self, x: i32, y: i32, slope: f32) -> Result<LineIntersection> {
        todo!()
    }

    /// Compute clipping parameters for this box within a rectangle `(0,0,w,h)`.
    ///
    /// Returns structured parameters for iterating over the clipped region.
    ///
    /// C Leptonica equivalent: `boxClipToRectangleParams`
    pub fn clip_to_rectangle_params(&self, w: i32, h: i32) -> Result<ClipParams> {
        todo!()
    }
}

// ---- Boxa methods ----

impl Boxa {
    /// Count boxes contained within a given box.
    ///
    /// C Leptonica equivalent: `boxaContainedInBoxCount`
    pub fn contained_in_box_count(&self, container: &Box) -> usize {
        todo!()
    }

    /// Check if every box in `self` is contained in at least one box in `container`.
    ///
    /// C Leptonica equivalent: `boxaContainedInBoxa`
    pub fn all_contained_in(&self, container: &Boxa) -> bool {
        todo!()
    }

    /// Count boxes that intersect with a given box.
    ///
    /// C Leptonica equivalent: `boxaIntersectsBoxCount`
    pub fn intersects_box_count(&self, target: &Box) -> usize {
        todo!()
    }

    /// Combine overlapping boxes between two Boxa arrays.
    ///
    /// Iteratively merges overlapping boxes across the two arrays.
    /// Returns the two resulting Boxa arrays after merging.
    ///
    /// C Leptonica equivalent: `boxaCombineOverlapsInPair`
    pub fn combine_overlaps_in_pair(boxa1: &Boxa, boxa2: &Boxa) -> (Boxa, Boxa) {
        todo!()
    }

    /// Handle overlapping boxes with configurable thresholds.
    ///
    /// For each pair of boxes within `range`, if the overlap ratio exceeds
    /// `min_overlap` and the area ratio is below `max_ratio`, the smaller
    /// box is either combined or removed based on `op`.
    ///
    /// - `range`: search window (0 = search all)
    /// - `min_overlap`: minimum overlap_area/smaller_area ratio to trigger [0..1]
    /// - `max_ratio`: maximum smaller_area/larger_area ratio to trigger [0..1]
    ///
    /// C Leptonica equivalent: `boxaHandleOverlaps`
    pub fn handle_overlaps(
        &self,
        op: OverlapOp,
        range: usize,
        min_overlap: f32,
        max_ratio: f32,
    ) -> Boxa {
        todo!()
    }

    /// Find the box whose centroid is nearest to a point.
    ///
    /// C Leptonica equivalent: `boxaGetNearestToPt`
    pub fn nearest_to_point(&self, x: i32, y: i32) -> Option<Box> {
        todo!()
    }

    /// Find the box whose centroid is nearest to a line.
    ///
    /// Specify a vertical line with `x >= 0, y < 0` or
    /// a horizontal line with `y >= 0, x < 0`.
    ///
    /// C Leptonica equivalent: `boxaGetNearestToLine`
    pub fn nearest_to_line(&self, x: i32, y: i32) -> Result<Option<Box>> {
        todo!()
    }

    /// Find the nearest box in a specific direction from a given box.
    ///
    /// - `box_index`: index of the reference box
    /// - `direction`: search direction
    /// - `dist_select`: distance filter
    /// - `range`: search window (0 = search all)
    ///
    /// C Leptonica equivalent: `boxaGetNearestByDirection`
    pub fn nearest_by_direction(
        &self,
        box_index: usize,
        direction: Direction,
        dist_select: DistSelect,
        range: usize,
    ) -> Result<NearestResult> {
        todo!()
    }

    /// Find nearest boxes in all 4 directions for every box.
    ///
    /// Returns a Vec where each element contains 4 `NearestResult` values
    /// in order: [FromLeft, FromRight, FromTop, FromBottom].
    ///
    /// C Leptonica equivalent: `boxaFindNearestBoxes`
    pub fn find_nearest_boxes(
        &self,
        dist_select: DistSelect,
        range: usize,
    ) -> Result<Vec<[NearestResult; 4]>> {
        todo!()
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    // -- Box::overlap_distance --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_overlap_distance_overlapping() {
        let b1 = Box::new(0, 0, 100, 100).unwrap();
        let b2 = Box::new(60, 40, 100, 100).unwrap();
        let (h, v) = b1.overlap_distance(&b2);
        assert_eq!(h, 40); // b1.right(100) - b2.x(60) = 40
        assert_eq!(v, 60); // b1.bottom(100) - b2.y(40) = 60
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_overlap_distance_separated() {
        let b1 = Box::new(0, 0, 50, 50).unwrap();
        let b2 = Box::new(80, 90, 50, 50).unwrap();
        let (h, v) = b1.overlap_distance(&b2);
        assert_eq!(h, -30); // 50 - 80 = -30 (gap of 30)
        assert_eq!(v, -40); // 50 - 90 = -40 (gap of 40)
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_overlap_distance_touching() {
        let b1 = Box::new(0, 0, 50, 50).unwrap();
        let b2 = Box::new(50, 50, 50, 50).unwrap();
        let (h, v) = b1.overlap_distance(&b2);
        assert_eq!(h, 0);
        assert_eq!(v, 0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_overlap_distance_reversed_order() {
        let b1 = Box::new(80, 90, 50, 50).unwrap();
        let b2 = Box::new(0, 0, 50, 50).unwrap();
        let (h, v) = b1.overlap_distance(&b2);
        assert_eq!(h, -30);
        assert_eq!(v, -40);
    }

    // -- Box::separation_distance --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_separation_distance_separated() {
        let b1 = Box::new(0, 0, 50, 50).unwrap();
        let b2 = Box::new(80, 90, 50, 50).unwrap();
        let (h, v) = b1.separation_distance(&b2);
        assert_eq!(h, 31); // gap(30) + 1
        assert_eq!(v, 41); // gap(40) + 1
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_separation_distance_overlapping() {
        let b1 = Box::new(0, 0, 100, 100).unwrap();
        let b2 = Box::new(50, 50, 100, 100).unwrap();
        let (h, v) = b1.separation_distance(&b2);
        assert_eq!(h, 0);
        assert_eq!(v, 0);
    }

    // -- Box::compare_size --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_compare_size_by_width() {
        let b1 = Box::new(0, 0, 100, 50).unwrap();
        let b2 = Box::new(0, 0, 80, 50).unwrap();
        assert_eq!(
            b1.compare_size(&b2, SizeComparisonType::Width),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_compare_size_by_area() {
        let b1 = Box::new(0, 0, 10, 10).unwrap();
        let b2 = Box::new(0, 0, 10, 10).unwrap();
        assert_eq!(
            b1.compare_size(&b2, SizeComparisonType::Area),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_compare_size_by_max_dimension() {
        let b1 = Box::new(0, 0, 50, 200).unwrap();
        let b2 = Box::new(0, 0, 100, 100).unwrap();
        assert_eq!(
            b1.compare_size(&b2, SizeComparisonType::MaxDimension),
            std::cmp::Ordering::Greater
        );
    }

    // -- Box::intersect_by_line --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_intersect_by_line_horizontal() {
        let b = Box::new(10, 10, 80, 60).unwrap();
        let result = b.intersect_by_line(0, 40, 0.0).unwrap();
        assert_eq!(result.count, 2);
        assert_eq!(result.p1, Some((10, 40)));
        assert_eq!(result.p2, Some((90, 40)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_intersect_by_line_vertical() {
        let b = Box::new(10, 10, 80, 60).unwrap();
        let result = b.intersect_by_line(50, 0, 2_000_000.0).unwrap();
        assert_eq!(result.count, 2);
        assert_eq!(result.p1, Some((50, 10)));
        assert_eq!(result.p2, Some((50, 70)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_intersect_by_line_no_intersection() {
        let b = Box::new(10, 10, 80, 60).unwrap();
        let result = b.intersect_by_line(0, 5, 0.0).unwrap();
        assert_eq!(result.count, 0);
        assert_eq!(result.p1, None);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_intersect_by_line_diagonal() {
        let b = Box::new(0, 0, 100, 100).unwrap();
        // Line through (0,0) with slope 1.0: y = x
        let result = b.intersect_by_line(0, 0, 1.0).unwrap();
        assert_eq!(result.count, 2);
    }

    // -- Box::clip_to_rectangle_params --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_clip_to_rectangle_params_inside() {
        let b = Box::new(10, 20, 30, 40).unwrap();
        let params = b.clip_to_rectangle_params(100, 100).unwrap();
        assert_eq!(params.x_start, 10);
        assert_eq!(params.y_start, 20);
        assert_eq!(params.x_end, 40);
        assert_eq!(params.y_end, 60);
        assert_eq!(params.width, 30);
        assert_eq!(params.height, 40);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_clip_to_rectangle_params_clipped() {
        let b = Box::new(-5, -10, 30, 40).unwrap();
        let params = b.clip_to_rectangle_params(100, 100).unwrap();
        assert_eq!(params.x_start, 0);
        assert_eq!(params.y_start, 0);
        assert_eq!(params.width, 25);
        assert_eq!(params.height, 30);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_clip_to_rectangle_params_outside() {
        let b = Box::new(200, 200, 30, 40).unwrap();
        assert!(b.clip_to_rectangle_params(100, 100).is_err());
    }

    // -- Boxa::contained_in_box_count --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_contained_in_box_count() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 10, 20, 20).unwrap()); // inside
        boxa.push(Box::new(50, 50, 20, 20).unwrap()); // inside
        boxa.push(Box::new(90, 90, 20, 20).unwrap()); // partially outside
        let container = Box::new(0, 0, 100, 100).unwrap();
        assert_eq!(boxa.contained_in_box_count(&container), 2);
    }

    // -- Boxa::all_contained_in --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_all_contained_in_true() {
        let mut containers = Boxa::new();
        containers.push(Box::new(0, 0, 50, 50).unwrap());
        containers.push(Box::new(50, 50, 50, 50).unwrap());

        let mut targets = Boxa::new();
        targets.push(Box::new(10, 10, 20, 20).unwrap());
        targets.push(Box::new(60, 60, 20, 20).unwrap());

        assert!(targets.all_contained_in(&containers));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_all_contained_in_false() {
        let mut containers = Boxa::new();
        containers.push(Box::new(0, 0, 50, 50).unwrap());

        let mut targets = Boxa::new();
        targets.push(Box::new(10, 10, 20, 20).unwrap());
        targets.push(Box::new(60, 60, 20, 20).unwrap()); // not contained

        assert!(!targets.all_contained_in(&containers));
    }

    // -- Boxa::intersects_box_count --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_intersects_box_count() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 10, 20, 20).unwrap()); // overlaps
        boxa.push(Box::new(40, 40, 20, 20).unwrap()); // overlaps
        boxa.push(Box::new(200, 200, 20, 20).unwrap()); // no overlap
        let target = Box::new(0, 0, 50, 50).unwrap();
        assert_eq!(boxa.intersects_box_count(&target), 2);
    }

    // -- Boxa::combine_overlaps_in_pair --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_combine_overlaps_in_pair() {
        let mut boxa1 = Boxa::new();
        boxa1.push(Box::new(0, 0, 60, 60).unwrap());

        let mut boxa2 = Boxa::new();
        boxa2.push(Box::new(40, 40, 60, 60).unwrap());

        let (r1, r2) = Boxa::combine_overlaps_in_pair(&boxa1, &boxa2);
        // After merging, one of the arrays should contain the union
        assert_eq!(r1.len() + r2.len(), 1);
    }

    // -- Boxa::handle_overlaps --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_handle_overlaps_combine() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 100, 100).unwrap());
        boxa.push(Box::new(10, 10, 50, 50).unwrap()); // mostly inside first

        let result = boxa.handle_overlaps(OverlapOp::Combine, 0, 0.5, 0.5);
        assert_eq!(result.len(), 1);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_handle_overlaps_remove_small() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 100, 100).unwrap());
        boxa.push(Box::new(10, 10, 30, 30).unwrap());

        let result = boxa.handle_overlaps(OverlapOp::RemoveSmall, 0, 0.5, 0.5);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().w, 100); // larger box remains
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_handle_overlaps_no_overlap() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 50, 50).unwrap());
        boxa.push(Box::new(100, 100, 50, 50).unwrap());

        let result = boxa.handle_overlaps(OverlapOp::Combine, 0, 0.5, 0.5);
        assert_eq!(result.len(), 2);
    }

    // -- Boxa::nearest_to_point --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_nearest_to_point() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 20, 20).unwrap()); // center (10, 10)
        boxa.push(Box::new(100, 100, 20, 20).unwrap()); // center (110, 110)
        boxa.push(Box::new(50, 50, 20, 20).unwrap()); // center (60, 60)

        let nearest = boxa.nearest_to_point(55, 55).unwrap();
        assert_eq!(nearest.x, 50);
        assert_eq!(nearest.y, 50);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_nearest_to_point_empty() {
        let boxa = Boxa::new();
        assert!(boxa.nearest_to_point(0, 0).is_none());
    }

    // -- Boxa::nearest_to_line --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_nearest_to_line_vertical() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 20, 20).unwrap()); // center_x = 10
        boxa.push(Box::new(40, 0, 20, 20).unwrap()); // center_x = 50
        boxa.push(Box::new(100, 0, 20, 20).unwrap()); // center_x = 110

        // Vertical line at x=45
        let nearest = boxa.nearest_to_line(45, -1).unwrap().unwrap();
        assert_eq!(nearest.x, 40);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_nearest_to_line_horizontal() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 20, 20).unwrap()); // center_y = 10
        boxa.push(Box::new(0, 80, 20, 20).unwrap()); // center_y = 90

        // Horizontal line at y=15
        let nearest = boxa.nearest_to_line(-1, 15).unwrap().unwrap();
        assert_eq!(nearest.y, 0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_nearest_to_line_invalid_params() {
        let boxa = Boxa::new();
        assert!(boxa.nearest_to_line(10, 10).is_err()); // both non-negative
        assert!(boxa.nearest_to_line(-1, -1).is_err()); // both negative
    }

    // -- Boxa::nearest_by_direction --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_nearest_by_direction_from_right() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 20, 40).unwrap());
        boxa.push(Box::new(50, 0, 20, 40).unwrap());
        boxa.push(Box::new(100, 0, 20, 40).unwrap());

        // From box 0, look right
        let result = boxa
            .nearest_by_direction(0, Direction::FromRight, DistSelect::All, 0)
            .unwrap();
        assert_eq!(result.index, Some(1));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_nearest_by_direction_from_left() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 20, 40).unwrap());
        boxa.push(Box::new(50, 0, 20, 40).unwrap());
        boxa.push(Box::new(100, 0, 20, 40).unwrap());

        // From box 2, look left
        let result = boxa
            .nearest_by_direction(2, Direction::FromLeft, DistSelect::All, 0)
            .unwrap();
        assert_eq!(result.index, Some(1));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_nearest_by_direction_no_overlap_in_y() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 20, 20).unwrap());
        boxa.push(Box::new(50, 100, 20, 20).unwrap()); // no Y overlap with box 0

        let result = boxa
            .nearest_by_direction(0, Direction::FromRight, DistSelect::All, 0)
            .unwrap();
        assert!(result.index.is_none());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_nearest_by_direction_invalid_index() {
        let boxa = Boxa::new();
        assert!(
            boxa.nearest_by_direction(0, Direction::FromRight, DistSelect::All, 0)
                .is_err()
        );
    }

    // -- Boxa::find_nearest_boxes --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_find_nearest_boxes() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 20, 40).unwrap());
        boxa.push(Box::new(50, 0, 20, 40).unwrap());
        boxa.push(Box::new(100, 0, 20, 40).unwrap());

        let results = boxa.find_nearest_boxes(DistSelect::All, 0).unwrap();
        assert_eq!(results.len(), 3);

        // Box 0: right neighbor is box 1
        assert_eq!(results[0][1].index, Some(1)); // FromRight
        assert!(results[0][0].index.is_none()); // FromLeft (nothing)
    }
}
