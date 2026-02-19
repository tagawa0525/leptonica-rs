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
        let h_ovl = if other.x >= self.x {
            self.right() - other.x
        } else {
            other.right() - self.x
        };
        let v_ovl = if other.y >= self.y {
            self.bottom() - other.y
        } else {
            other.bottom() - self.y
        };
        (h_ovl, v_ovl)
    }

    /// Compute horizontal and vertical separation distances between two boxes.
    ///
    /// Returns `(h_sep, v_sep)` where:
    /// - 0 = overlapping in that dimension
    /// - Positive = gap distance + 1
    ///
    /// C Leptonica equivalent: `boxSeparationDistance`
    pub fn separation_distance(&self, other: &Box) -> (i32, i32) {
        let (h_ovl, v_ovl) = self.overlap_distance(other);
        let h_sep = if h_ovl > 0 { 0 } else { -h_ovl + 1 };
        let v_sep = if v_ovl > 0 { 0 } else { -v_ovl + 1 };
        (h_sep, v_sep)
    }

    /// Compare two boxes by a size metric.
    ///
    /// Returns `Ordering::Greater` if self > other, `Ordering::Less` if self < other,
    /// `Ordering::Equal` if equal.
    ///
    /// C Leptonica equivalent: `boxCompareSize`
    pub fn compare_size(&self, other: &Box, cmp_type: SizeComparisonType) -> std::cmp::Ordering {
        let (v1, v2): (i64, i64) = match cmp_type {
            SizeComparisonType::Width => (self.w as i64, other.w as i64),
            SizeComparisonType::Height => (self.h as i64, other.h as i64),
            SizeComparisonType::MaxDimension => {
                (self.w.max(self.h) as i64, other.w.max(other.h) as i64)
            }
            SizeComparisonType::Perimeter => ((self.w + self.h) as i64, (other.w + other.h) as i64),
            SizeComparisonType::Area => (self.area(), other.area()),
        };
        v1.cmp(&v2)
    }

    /// Find intersection points of a line with this box's boundary.
    ///
    /// The line is defined by a point `(x, y)` and a `slope`.
    /// Use `slope > 1_000_000.0` for vertical lines.
    ///
    /// C Leptonica equivalent: `boxIntersectByLine`
    pub fn intersect_by_line(&self, x: i32, y: i32, slope: f32) -> Result<LineIntersection> {
        if self.w == 0 || self.h == 0 {
            return Err(Error::InvalidParameter(
                "box must have non-zero dimensions for line intersection".into(),
            ));
        }

        let bx = self.x;
        let by = self.y;
        let bw = self.w;
        let bh = self.h;

        let mut points: Vec<(i32, i32)> = Vec::with_capacity(4);

        if slope.abs() < f32::EPSILON {
            // Horizontal line: y = y
            if y >= by && y < by + bh {
                return Ok(LineIntersection {
                    p1: Some((bx, y)),
                    p2: Some((bx + bw, y)),
                    count: 2,
                });
            }
            return Ok(LineIntersection {
                p1: None,
                p2: None,
                count: 0,
            });
        }

        if slope.abs() > 1_000_000.0 {
            // Vertical line: x = x
            if x >= bx && x < bx + bw {
                return Ok(LineIntersection {
                    p1: Some((x, by)),
                    p2: Some((x, by + bh)),
                    count: 2,
                });
            }
            return Ok(LineIntersection {
                p1: None,
                p2: None,
                count: 0,
            });
        }

        // General case: check all 4 edges (using inclusive bounds like C version)
        let inv_slope = 1.0 / slope as f64;
        let slope_d = slope as f64;

        // Top edge (y = by): x_intersect = x + (1/slope) * (by - y)
        let xp = x as f64 + inv_slope * (by as f64 - y as f64);
        if xp >= bx as f64 && xp <= (bx + bw) as f64 {
            points.push(((xp + 0.5) as i32, by));
        }

        // Bottom edge (y = by + bh): x_intersect = x + (1/slope) * (by + bh - y)
        let xp = x as f64 + inv_slope * ((by + bh) as f64 - y as f64);
        if xp >= bx as f64 && xp <= (bx + bw) as f64 {
            points.push(((xp + 0.5) as i32, by + bh));
        }

        // Left edge (x = bx): y_intersect = y + slope * (bx - x)
        let yp = y as f64 + slope_d * (bx as f64 - x as f64);
        if yp >= by as f64 && yp <= (by + bh) as f64 {
            let pt = (bx, (yp + 0.5) as i32);
            if !points.contains(&pt) {
                points.push(pt);
            }
        }

        // Right edge (x = bx + bw): y_intersect = y + slope * (bx + bw - x)
        let yp = y as f64 + slope_d * ((bx + bw) as f64 - x as f64);
        if yp >= by as f64 && yp <= (by + bh) as f64 {
            let pt = (bx + bw, (yp + 0.5) as i32);
            if !points.contains(&pt) {
                points.push(pt);
            }
        }

        // Deduplicate and limit to 2 points
        points.dedup();
        let count = points.len().min(2);

        Ok(LineIntersection {
            p1: points.first().copied(),
            p2: if count >= 2 {
                points.get(1).copied()
            } else {
                None
            },
            count,
        })
    }

    /// Compute clipping parameters for this box within a rectangle `(0,0,w,h)`.
    ///
    /// Returns structured parameters for iterating over the clipped region.
    ///
    /// C Leptonica equivalent: `boxClipToRectangleParams`
    pub fn clip_to_rectangle_params(&self, w: i32, h: i32) -> Result<ClipParams> {
        let clipped = self.clip(w, h).ok_or_else(|| {
            Error::InvalidParameter("box is entirely outside the rectangle".into())
        })?;
        Ok(ClipParams {
            x_start: clipped.x,
            y_start: clipped.y,
            x_end: clipped.x + clipped.w,
            y_end: clipped.y + clipped.h,
            width: clipped.w,
            height: clipped.h,
        })
    }
}

// ---- Boxa methods ----

impl Boxa {
    /// Count boxes contained within a given box.
    ///
    /// C Leptonica equivalent: `boxaContainedInBoxCount`
    pub fn contained_in_box_count(&self, container: &Box) -> usize {
        self.iter().filter(|b| container.contains_box(b)).count()
    }

    /// Check if every box in `self` is contained in at least one box in `container`.
    ///
    /// C Leptonica equivalent: `boxaContainedInBoxa`
    pub fn all_contained_in(&self, container: &Boxa) -> bool {
        self.iter()
            .all(|target| container.iter().any(|c| c.contains_box(target)))
    }

    /// Count boxes that intersect with a given box.
    ///
    /// C Leptonica equivalent: `boxaIntersectsBoxCount`
    pub fn intersects_box_count(&self, target: &Box) -> usize {
        self.iter().filter(|b| b.overlaps(target)).count()
    }

    /// Combine overlapping boxes between two Boxa arrays.
    ///
    /// Iteratively merges overlapping boxes across the two arrays.
    /// Returns the two resulting Boxa arrays after merging.
    ///
    /// C Leptonica equivalent: `boxaCombineOverlapsInPair`
    pub fn combine_overlaps_in_pair(boxa1: &Boxa, boxa2: &Boxa) -> (Boxa, Boxa) {
        let mut boxes1: Vec<Box> = boxa1.iter().copied().collect();
        let mut boxes2: Vec<Box> = boxa2.iter().copied().collect();

        let mut changed = true;
        while changed {
            changed = false;

            // Combine overlaps within each array
            boxes1 = Boxa::from_iter(boxes1)
                .combine_overlaps()
                .into_iter()
                .collect();
            boxes2 = Boxa::from_iter(boxes2)
                .combine_overlaps()
                .into_iter()
                .collect();

            // Merge across arrays
            'outer: for i in 0..boxes1.len() {
                for j in 0..boxes2.len() {
                    if i < boxes1.len() && j < boxes2.len() && boxes1[i].overlaps(&boxes2[j]) {
                        if boxes1[i].area() >= boxes2[j].area() {
                            boxes1[i] = boxes1[i].union(&boxes2[j]);
                            boxes2.remove(j);
                        } else {
                            boxes2[j] = boxes1[i].union(&boxes2[j]);
                            boxes1.remove(i);
                        }
                        changed = true;
                        break 'outer;
                    }
                }
            }
        }

        (boxes1.into_iter().collect(), boxes2.into_iter().collect())
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
        let n = self.len();
        if n == 0 {
            return Boxa::new();
        }

        let mut boxes: Vec<Box> = self.iter().copied().collect();
        let mut eliminated = vec![false; n];

        for i in 0..n {
            if eliminated[i] {
                continue;
            }
            let j_max = if range == 0 {
                n
            } else {
                (i + 1 + range).min(n)
            };
            for j in (i + 1)..j_max {
                if eliminated[j] {
                    continue;
                }
                let overlap_area = boxes[i].overlap_area(&boxes[j]);
                if overlap_area == 0 {
                    continue;
                }

                let area_i = boxes[i].area();
                let area_j = boxes[j].area();
                let (smaller_area, larger_area, smaller_idx, larger_idx) = if area_i <= area_j {
                    (area_i, area_j, i, j)
                } else {
                    (area_j, area_i, j, i)
                };

                if smaller_area == 0 {
                    eliminated[smaller_idx] = true;
                    continue;
                }

                let overlap_ratio = overlap_area as f32 / smaller_area as f32;
                let area_ratio = smaller_area as f32 / larger_area as f32;

                if overlap_ratio >= min_overlap && area_ratio <= max_ratio {
                    match op {
                        OverlapOp::Combine => {
                            boxes[larger_idx] = boxes[larger_idx].union(&boxes[smaller_idx]);
                            eliminated[smaller_idx] = true;
                        }
                        OverlapOp::RemoveSmall => {
                            eliminated[smaller_idx] = true;
                        }
                    }
                }
            }
        }

        boxes
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| !eliminated[*idx])
            .map(|(_, b)| b)
            .collect()
    }

    /// Find the box whose centroid is nearest to a point.
    ///
    /// C Leptonica equivalent: `boxaGetNearestToPt`
    pub fn nearest_to_point(&self, x: i32, y: i32) -> Option<Box> {
        let mut min_dist = i64::MAX;
        let mut nearest = None;
        for b in self.iter() {
            let dx = b.center_x() as i64 - x as i64;
            let dy = b.center_y() as i64 - y as i64;
            let dist = dx * dx + dy * dy;
            if dist < min_dist {
                min_dist = dist;
                nearest = Some(*b);
            }
        }
        nearest
    }

    /// Find the box whose centroid is nearest to a line.
    ///
    /// Specify a vertical line with `x >= 0, y < 0` or
    /// a horizontal line with `y >= 0, x < 0`.
    ///
    /// C Leptonica equivalent: `boxaGetNearestToLine`
    pub fn nearest_to_line(&self, x: i32, y: i32) -> Result<Option<Box>> {
        let vertical = x >= 0 && y < 0;
        let horizontal = y >= 0 && x < 0;
        if !vertical && !horizontal {
            return Err(Error::InvalidParameter(
                "exactly one of x,y must be negative to specify line orientation".into(),
            ));
        }

        let mut min_dist = i32::MAX;
        let mut nearest = None;
        for b in self.iter() {
            let dist = if vertical {
                (b.center_x() - x).abs()
            } else {
                (b.center_y() - y).abs()
            };
            if dist < min_dist {
                min_dist = dist;
                nearest = Some(*b);
            }
        }
        Ok(nearest)
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
        let n = self.len();
        if box_index >= n {
            return Err(Error::IndexOutOfBounds {
                index: box_index,
                len: n,
            });
        }

        let ref_box = self.boxes()[box_index];
        let j_min = if range == 0 {
            0
        } else {
            box_index.saturating_sub(range)
        };
        let j_max = if range == 0 {
            n
        } else {
            (box_index + 1 + range).min(n)
        };

        let mut best_dist = i32::MAX;
        let mut best_index = None;

        for j in j_min..j_max {
            if j == box_index {
                continue;
            }
            let candidate = self.boxes()[j];

            let (has_overlap, dist) = match direction {
                Direction::FromLeft | Direction::FromRight => {
                    let overlap =
                        has_overlap_in_range(ref_box.y, ref_box.h, candidate.y, candidate.h);
                    let d = distance_in_range(ref_box.x, ref_box.w, candidate.x, candidate.w);
                    (overlap, d)
                }
                Direction::FromTop | Direction::FromBottom => {
                    let overlap =
                        has_overlap_in_range(ref_box.x, ref_box.w, candidate.x, candidate.w);
                    let d = distance_in_range(ref_box.y, ref_box.h, candidate.y, candidate.h);
                    (overlap, d)
                }
            };

            if !has_overlap {
                continue;
            }

            // Check directional constraint
            let valid_direction = match direction {
                Direction::FromLeft => candidate.x < ref_box.x || candidate.right() <= ref_box.x,
                Direction::FromRight => {
                    candidate.x >= ref_box.x || candidate.right() > ref_box.right()
                }
                Direction::FromTop => candidate.y < ref_box.y || candidate.bottom() <= ref_box.y,
                Direction::FromBottom => {
                    candidate.y >= ref_box.y || candidate.bottom() > ref_box.bottom()
                }
            };

            if !valid_direction {
                continue;
            }

            if dist_select == DistSelect::NonNegative && dist < 0 {
                continue;
            }

            let abs_dist = dist.abs();
            if abs_dist < best_dist {
                best_dist = abs_dist;
                best_index = Some(j);
            }
        }

        Ok(NearestResult {
            index: best_index,
            distance: if best_index.is_some() {
                best_dist
            } else {
                i32::MAX
            },
        })
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
        let n = self.len();
        let mut results = Vec::with_capacity(n);
        let directions = [
            Direction::FromLeft,
            Direction::FromRight,
            Direction::FromTop,
            Direction::FromBottom,
        ];

        for i in 0..n {
            let mut entry = [NearestResult {
                index: None,
                distance: i32::MAX,
            }; 4];
            for (d, dir) in directions.iter().enumerate() {
                entry[d] = self.nearest_by_direction(i, *dir, dist_select, range)?;
            }
            results.push(entry);
        }

        Ok(results)
    }
}

// ---- Helper functions ----

/// Check if two 1D ranges [c1, c1+s1) and [c2, c2+s2) overlap.
///
/// C equivalent: `boxHasOverlapInXorY`
fn has_overlap_in_range(c1: i32, s1: i32, c2: i32, s2: i32) -> bool {
    c1 < c2 + s2 && c2 < c1 + s1
}

/// Compute signed distance between two 1D ranges [c1, c1+s1) and [c2, c2+s2).
///
/// Returns negative if overlapping, 0 if touching, positive if separated.
///
/// C equivalent: `boxGetDistanceInXorY`
fn distance_in_range(c1: i32, s1: i32, c2: i32, s2: i32) -> i32 {
    if c2 >= c1 {
        c2 - (c1 + s1)
    } else {
        c1 - (c2 + s2)
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    // -- Box::overlap_distance --

    #[test]
    fn test_overlap_distance_overlapping() {
        let b1 = Box::new(0, 0, 100, 100).unwrap();
        let b2 = Box::new(60, 40, 100, 100).unwrap();
        let (h, v) = b1.overlap_distance(&b2);
        assert_eq!(h, 40); // b1.right(100) - b2.x(60) = 40
        assert_eq!(v, 60); // b1.bottom(100) - b2.y(40) = 60
    }

    #[test]
    fn test_overlap_distance_separated() {
        let b1 = Box::new(0, 0, 50, 50).unwrap();
        let b2 = Box::new(80, 90, 50, 50).unwrap();
        let (h, v) = b1.overlap_distance(&b2);
        assert_eq!(h, -30); // 50 - 80 = -30 (gap of 30)
        assert_eq!(v, -40); // 50 - 90 = -40 (gap of 40)
    }

    #[test]
    fn test_overlap_distance_touching() {
        let b1 = Box::new(0, 0, 50, 50).unwrap();
        let b2 = Box::new(50, 50, 50, 50).unwrap();
        let (h, v) = b1.overlap_distance(&b2);
        assert_eq!(h, 0);
        assert_eq!(v, 0);
    }

    #[test]
    fn test_overlap_distance_reversed_order() {
        let b1 = Box::new(80, 90, 50, 50).unwrap();
        let b2 = Box::new(0, 0, 50, 50).unwrap();
        let (h, v) = b1.overlap_distance(&b2);
        assert_eq!(h, -30);
        assert_eq!(v, -40);
    }

    // -- Box::separation_distance --

    #[test]
    fn test_separation_distance_separated() {
        let b1 = Box::new(0, 0, 50, 50).unwrap();
        let b2 = Box::new(80, 90, 50, 50).unwrap();
        let (h, v) = b1.separation_distance(&b2);
        assert_eq!(h, 31); // gap(30) + 1
        assert_eq!(v, 41); // gap(40) + 1
    }

    #[test]
    fn test_separation_distance_overlapping() {
        let b1 = Box::new(0, 0, 100, 100).unwrap();
        let b2 = Box::new(50, 50, 100, 100).unwrap();
        let (h, v) = b1.separation_distance(&b2);
        assert_eq!(h, 0);
        assert_eq!(v, 0);
    }

    // -- Box::compare_size --

    #[test]
    fn test_compare_size_by_width() {
        let b1 = Box::new(0, 0, 100, 50).unwrap();
        let b2 = Box::new(0, 0, 80, 50).unwrap();
        assert_eq!(
            b1.compare_size(&b2, SizeComparisonType::Width),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_compare_size_by_area() {
        let b1 = Box::new(0, 0, 10, 10).unwrap();
        let b2 = Box::new(0, 0, 10, 10).unwrap();
        assert_eq!(
            b1.compare_size(&b2, SizeComparisonType::Area),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
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
    fn test_intersect_by_line_horizontal() {
        let b = Box::new(10, 10, 80, 60).unwrap();
        let result = b.intersect_by_line(0, 40, 0.0).unwrap();
        assert_eq!(result.count, 2);
        assert_eq!(result.p1, Some((10, 40)));
        assert_eq!(result.p2, Some((90, 40)));
    }

    #[test]
    fn test_intersect_by_line_vertical() {
        let b = Box::new(10, 10, 80, 60).unwrap();
        let result = b.intersect_by_line(50, 0, 2_000_000.0).unwrap();
        assert_eq!(result.count, 2);
        assert_eq!(result.p1, Some((50, 10)));
        assert_eq!(result.p2, Some((50, 70)));
    }

    #[test]
    fn test_intersect_by_line_no_intersection() {
        let b = Box::new(10, 10, 80, 60).unwrap();
        let result = b.intersect_by_line(0, 5, 0.0).unwrap();
        assert_eq!(result.count, 0);
        assert_eq!(result.p1, None);
    }

    #[test]
    fn test_intersect_by_line_diagonal() {
        let b = Box::new(0, 0, 100, 100).unwrap();
        // Line through (0,0) with slope 1.0: y = x
        let result = b.intersect_by_line(0, 0, 1.0).unwrap();
        assert_eq!(result.count, 2);
    }

    // -- Box::clip_to_rectangle_params --

    #[test]
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
    fn test_clip_to_rectangle_params_clipped() {
        let b = Box::new(-5, -10, 30, 40).unwrap();
        let params = b.clip_to_rectangle_params(100, 100).unwrap();
        assert_eq!(params.x_start, 0);
        assert_eq!(params.y_start, 0);
        assert_eq!(params.width, 25);
        assert_eq!(params.height, 30);
    }

    #[test]
    fn test_clip_to_rectangle_params_outside() {
        let b = Box::new(200, 200, 30, 40).unwrap();
        assert!(b.clip_to_rectangle_params(100, 100).is_err());
    }

    // -- Boxa::contained_in_box_count --

    #[test]
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
    fn test_handle_overlaps_combine() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 100, 100).unwrap());
        boxa.push(Box::new(10, 10, 50, 50).unwrap()); // mostly inside first

        let result = boxa.handle_overlaps(OverlapOp::Combine, 0, 0.5, 0.5);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_handle_overlaps_remove_small() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 100, 100).unwrap());
        boxa.push(Box::new(10, 10, 30, 30).unwrap());

        let result = boxa.handle_overlaps(OverlapOp::RemoveSmall, 0, 0.5, 0.5);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().w, 100); // larger box remains
    }

    #[test]
    fn test_handle_overlaps_no_overlap() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 50, 50).unwrap());
        boxa.push(Box::new(100, 100, 50, 50).unwrap());

        let result = boxa.handle_overlaps(OverlapOp::Combine, 0, 0.5, 0.5);
        assert_eq!(result.len(), 2);
    }

    // -- Boxa::nearest_to_point --

    #[test]
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
    fn test_nearest_to_point_empty() {
        let boxa = Boxa::new();
        assert!(boxa.nearest_to_point(0, 0).is_none());
    }

    // -- Boxa::nearest_to_line --

    #[test]
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
    fn test_nearest_to_line_horizontal() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 20, 20).unwrap()); // center_y = 10
        boxa.push(Box::new(0, 80, 20, 20).unwrap()); // center_y = 90

        // Horizontal line at y=15
        let nearest = boxa.nearest_to_line(-1, 15).unwrap().unwrap();
        assert_eq!(nearest.y, 0);
    }

    #[test]
    fn test_nearest_to_line_invalid_params() {
        let boxa = Boxa::new();
        assert!(boxa.nearest_to_line(10, 10).is_err()); // both non-negative
        assert!(boxa.nearest_to_line(-1, -1).is_err()); // both negative
    }

    // -- Boxa::nearest_by_direction --

    #[test]
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
    fn test_nearest_by_direction_invalid_index() {
        let boxa = Boxa::new();
        assert!(
            boxa.nearest_by_direction(0, Direction::FromRight, DistSelect::All, 0)
                .is_err()
        );
    }

    // -- Boxa::find_nearest_boxes --

    #[test]
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
