//! Boxa sort functions
//!
//! Bin sort, sort by index, 2D sort, and related operations.
//!
//! C Leptonica equivalents: boxfunc2.c

use crate::core::error::{Error, Result};
use crate::core::numa::{Numa, SortOrder};

use super::{Box, Boxa, Boxaa};

/// Sort type for box parameters.
///
/// C Leptonica equivalents: `L_SORT_BY_X`, `L_SORT_BY_Y`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxSortType {
    /// Sort by x (left side)
    ByX,
    /// Sort by y (top side)
    ByY,
    /// Sort by right side (x + w)
    ByRight,
    /// Sort by bottom side (y + h)
    ByBottom,
    /// Sort by width
    ByWidth,
    /// Sort by height
    ByHeight,
    /// Sort by min(w, h)
    ByMinDimension,
    /// Sort by max(w, h)
    ByMaxDimension,
    /// Sort by perimeter (w + h)
    ByPerimeter,
    /// Sort by area (w * h)
    ByArea,
    /// Sort by aspect ratio (w / h)
    ByAspectRatio,
}

impl Boxa {
    /// Extract the sort key for a box based on the sort type.
    fn sort_key(b: &Box, sort_type: BoxSortType) -> f32 {
        match sort_type {
            BoxSortType::ByX => b.x as f32,
            BoxSortType::ByY => b.y as f32,
            BoxSortType::ByRight => (b.x + b.w - 1) as f32,
            BoxSortType::ByBottom => (b.y + b.h - 1) as f32,
            BoxSortType::ByWidth => b.w as f32,
            BoxSortType::ByHeight => b.h as f32,
            BoxSortType::ByMinDimension => b.w.min(b.h) as f32,
            BoxSortType::ByMaxDimension => b.w.max(b.h) as f32,
            BoxSortType::ByPerimeter => (b.w + b.h) as f32,
            BoxSortType::ByArea => (b.w * b.h) as f32,
            BoxSortType::ByAspectRatio => {
                if b.h == 0 {
                    0.0
                } else {
                    b.w as f32 / b.h as f32
                }
            }
        }
    }

    /// O(n) bin sort by an integer-valued box parameter.
    ///
    /// Suitable sort types: `ByX`, `ByY`, `ByWidth`, `ByHeight`, `ByPerimeter`.
    /// Returns `(sorted_boxa, sort_index)`.
    ///
    /// C Leptonica equivalent: `boxaBinSort`
    pub fn bin_sort(&self, sort_type: BoxSortType, order: SortOrder) -> Result<(Boxa, Numa)> {
        let n = self.len();
        if n == 0 {
            return Ok((Boxa::new(), Numa::new()));
        }
        match sort_type {
            BoxSortType::ByX
            | BoxSortType::ByY
            | BoxSortType::ByWidth
            | BoxSortType::ByHeight
            | BoxSortType::ByPerimeter => {}
            _ => {
                return Err(Error::InvalidParameter(
                    "bin_sort only supports ByX, ByY, ByWidth, ByHeight, ByPerimeter".into(),
                ));
            }
        }

        let na: Numa = self.iter().map(|b| Self::sort_key(b, sort_type)).collect();

        let naindex = na.bin_sort_index(order)?;
        let boxad = self.sort_by_index(&naindex)?;
        Ok((boxad, naindex))
    }

    /// Sort the boxa according to a given index array.
    ///
    /// `naindex` maps from position in the output to position in the input.
    ///
    /// C Leptonica equivalent: `boxaSortByIndex`
    pub fn sort_by_index(&self, naindex: &Numa) -> Result<Boxa> {
        let n = self.len();
        if n == 0 {
            return Ok(Boxa::new());
        }
        let mut result = Boxa::with_capacity(naindex.len());
        for i in 0..naindex.len() {
            let index = naindex.get_i32(i).ok_or_else(|| Error::IndexOutOfBounds {
                index: i,
                len: naindex.len(),
            })? as usize;
            let b = self
                .get(index)
                .ok_or(Error::IndexOutOfBounds { index, len: n })?;
            result.push(*b);
        }
        Ok(result)
    }

    /// 2D sort: sort boxes into rows (top-to-bottom) then columns (left-to-right).
    ///
    /// - `delta1`: vertical overlap tolerance for pass 1 (taller boxes)
    /// - `delta2`: vertical overlap tolerance for pass 2 (shorter boxes)
    /// - `minh1`: minimum height to start a new row in pass 1
    ///
    /// C Leptonica equivalent: `boxaSort2d`
    pub fn sort_2d(&self, delta1: i32, delta2: i32, minh1: i32) -> Result<Boxaa> {
        let n = self.len();
        if n == 0 {
            return Err(Error::InvalidParameter("boxa is empty".into()));
        }

        // Sort from left to right
        let na_keys: Numa = self.iter().map(|b| b.x as f32).collect();
        let naindex = na_keys.sort_index(SortOrder::Increasing);
        let boxa = self.sort_by_index(&naindex)?;

        // Pass 1: assign taller boxes to rows
        let mut baa = Boxaa::new();
        let mut small_boxes: Vec<(Box, usize)> = Vec::new(); // (box, orig_index)

        for i in 0..boxa.len() {
            let b = *boxa.get(i).unwrap();
            if b.h < minh1 {
                small_boxes.push((b, i));
            } else {
                let index = baa.align_box(&b, delta1);
                let n_rows = baa.len();
                if index < n_rows {
                    baa.get_mut(index).unwrap().push(b);
                } else {
                    let mut new_boxa = Boxa::new();
                    new_boxa.push(b);
                    baa.push(new_boxa);
                }
            }
        }

        // Pass 2: feed in small height boxes
        for (b, _orig_idx) in &small_boxes {
            let index = baa.align_box(b, delta2);
            let n_rows = baa.len();
            if index < n_rows {
                baa.get_mut(index).unwrap().push(*b);
            } else {
                let mut new_boxa = Boxa::new();
                new_boxa.push(*b);
                baa.push(new_boxa);
            }
        }

        // Sort boxes in each row horizontally (by x)
        for boxa_row in baa.boxas_mut() {
            boxa_row.boxes_mut().sort_by(|a, b| a.x.cmp(&b.x));
        }

        // Sort rows vertically (by y of first box)
        let mut row_indices: Vec<usize> = (0..baa.len()).collect();
        row_indices.sort_by_key(|&i| {
            baa.get(i)
                .and_then(|b| b.get(0))
                .map(|b| b.y)
                .unwrap_or(i32::MAX)
        });

        let mut result = Boxaa::with_capacity(baa.len());
        for &idx in &row_indices {
            result.push(baa.get(idx).unwrap().clone());
        }

        Ok(result)
    }

    /// 2D sort by pre-computed index arrays.
    ///
    /// `naa` is a Numaa-like structure represented as `Vec<Vec<usize>>` where each
    /// inner vec maps from position in a row to position in the input boxa.
    ///
    /// C Leptonica equivalent: `boxaSort2dByIndex`
    pub fn sort_2d_by_index(&self, indices: &[Vec<usize>]) -> Result<Boxaa> {
        let n = self.len();
        if n == 0 {
            return Err(Error::InvalidParameter("boxa is empty".into()));
        }

        let total: usize = indices.iter().map(|v| v.len()).sum();
        if total != n {
            return Err(Error::InvalidParameter(format!(
                "index count {total} != boxa count {n}"
            )));
        }

        let mut baa = Boxaa::with_capacity(indices.len());
        for row in indices {
            let mut boxa = Boxa::with_capacity(row.len());
            for &idx in row {
                let b = self
                    .get(idx)
                    .ok_or(Error::IndexOutOfBounds { index: idx, len: n })?;
                boxa.push(*b);
            }
            baa.push(boxa);
        }
        Ok(baa)
    }

    /// Group boxes into a Boxaa, putting `num` consecutive boxes into each Boxa.
    ///
    /// C Leptonica equivalent: `boxaEncapsulateAligned`
    pub fn encapsulate_aligned(&self, num: usize) -> Boxaa {
        let n = self.len();
        let nbaa = n / num;
        let mut baa = Boxaa::with_capacity(nbaa);
        let mut index = 0;
        for _ in 0..nbaa {
            let mut boxa = Boxa::with_capacity(num);
            for _ in 0..num {
                if index < n {
                    boxa.push(*self.get(index).unwrap());
                    index += 1;
                }
            }
            baa.push(boxa);
        }
        baa
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::box_::Box;

    fn sample_boxa() -> Boxa {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(50, 10, 30, 40));
        boxa.push(Box::new_unchecked(10, 20, 20, 30));
        boxa.push(Box::new_unchecked(80, 5, 40, 50));
        boxa.push(Box::new_unchecked(30, 15, 25, 35));
        boxa
    }

    #[test]
    fn test_bin_sort_by_x() {
        let boxa = sample_boxa();
        let (sorted, idx) = boxa
            .bin_sort(BoxSortType::ByX, SortOrder::Increasing)
            .unwrap();
        assert_eq!(sorted.get(0).unwrap().x, 10);
        assert_eq!(sorted.get(1).unwrap().x, 30);
        assert_eq!(sorted.get(2).unwrap().x, 50);
        assert_eq!(sorted.get(3).unwrap().x, 80);
        assert_eq!(idx.len(), 4);
    }

    #[test]
    fn test_bin_sort_by_width_decreasing() {
        let boxa = sample_boxa();
        let (sorted, _) = boxa
            .bin_sort(BoxSortType::ByWidth, SortOrder::Decreasing)
            .unwrap();
        assert_eq!(sorted.get(0).unwrap().w, 40);
        assert_eq!(sorted.get(3).unwrap().w, 20);
    }

    #[test]
    fn test_bin_sort_invalid_type() {
        let boxa = sample_boxa();
        assert!(
            boxa.bin_sort(BoxSortType::ByArea, SortOrder::Increasing)
                .is_err()
        );
    }

    #[test]
    fn test_sort_by_index() {
        let boxa = sample_boxa();
        let idx = Numa::from_vec(vec![2.0, 0.0, 3.0, 1.0]);
        let sorted = boxa.sort_by_index(&idx).unwrap();
        assert_eq!(sorted.get(0).unwrap().x, 80);
        assert_eq!(sorted.get(1).unwrap().x, 50);
    }

    #[test]
    fn test_sort_by_index_out_of_bounds() {
        let boxa = sample_boxa();
        let idx = Numa::from_vec(vec![0.0, 10.0]);
        assert!(boxa.sort_by_index(&idx).is_err());
    }

    #[test]
    fn test_sort_2d() {
        let mut boxa = Boxa::new();
        // Row 1 boxes (y ~ 10)
        boxa.push(Box::new_unchecked(50, 10, 30, 20));
        boxa.push(Box::new_unchecked(10, 12, 20, 18));
        // Row 2 boxes (y ~ 50)
        boxa.push(Box::new_unchecked(20, 50, 25, 22));
        boxa.push(Box::new_unchecked(60, 48, 30, 20));

        let baa = boxa.sort_2d(5, 5, 5).unwrap();
        assert!(baa.len() >= 2);
    }

    #[test]
    fn test_sort_2d_by_index() {
        let boxa = sample_boxa();
        let indices = vec![vec![0, 1], vec![2, 3]];
        let baa = boxa.sort_2d_by_index(&indices).unwrap();
        assert_eq!(baa.len(), 2);
        assert_eq!(baa.get(0).unwrap().len(), 2);
        assert_eq!(baa.get(1).unwrap().len(), 2);
    }

    #[test]
    fn test_encapsulate_aligned() {
        let boxa = sample_boxa();
        let baa = boxa.encapsulate_aligned(2);
        assert_eq!(baa.len(), 2);
        assert_eq!(baa.get(0).unwrap().len(), 2);
        assert_eq!(baa.get(1).unwrap().len(), 2);
    }

    #[test]
    fn test_encapsulate_aligned_uneven() {
        let mut boxa = Boxa::new();
        for i in 0..5 {
            boxa.push(Box::new_unchecked(i * 10, 0, 10, 10));
        }
        let baa = boxa.encapsulate_aligned(2);
        assert_eq!(baa.len(), 2); // 5/2 = 2, last box is dropped
    }
}
