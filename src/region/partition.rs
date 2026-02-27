//! White block detection via recursive rectangle partitioning
//!
//! Implements Breuel's algorithm for finding whitespace rectangles
//! and box pruning based on overlap.
//!
//! # See also
//! C Leptonica: `partition.c`

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::core::Box;
use crate::core::Boxa;
use crate::region::{RegionError, RegionResult};

/// Sort criterion for whitespace blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteblockSort {
    /// Sort by width
    ByWidth,
    /// Sort by height
    ByHeight,
    /// Sort by min(width, height)
    ByMinDimension,
    /// Sort by max(width, height)
    ByMaxDimension,
    /// Sort by half-perimeter (w+h)
    ByPerimeter,
    /// Sort by area (w*h)
    ByArea,
}

/// Partition element for the heap
struct Partel {
    size: f64,
    boxx: Box,
    boxa: Boxa,
}

impl PartialEq for Partel {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
    }
}

impl Eq for Partel {}

impl PartialOrd for Partel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Partel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size
            .partial_cmp(&other.size)
            .unwrap_or(Ordering::Equal)
    }
}

fn partel_set_size(partel: &mut Partel, sort_flag: WhiteblockSort) {
    let w = partel.boxx.w as f64;
    let h = partel.boxx.h as f64;
    partel.size = match sort_flag {
        WhiteblockSort::ByWidth => w,
        WhiteblockSort::ByHeight => h,
        WhiteblockSort::ByMinDimension => w.min(h),
        WhiteblockSort::ByMaxDimension => w.max(h),
        WhiteblockSort::ByPerimeter => w + h,
        WhiteblockSort::ByArea => w * h,
    };
}

fn boxa_select_pivot_box(boxx: &Box, boxa: &Boxa, max_perim: i32, fract: f32) -> Box {
    let n = boxa.len();
    let cx = boxx.center_x() as f64;
    let cy = boxx.center_y() as f64;
    let w = boxx.w as f64;
    let h = boxx.h as f64;
    let thresh_dist = (fract as f64) * (w * w + h * h);
    let mut min_dist = f64::MAX;
    let mut min_index = 0;
    let mut small_found = false;

    for i in 0..n {
        if let Some(bt) = boxa.get(i) {
            let perim = bt.w + bt.h;
            if perim > max_perim {
                continue;
            }
            small_found = true;
            let delx = bt.center_x() as f64 - cx;
            let dely = bt.center_y() as f64 - cy;
            let dist = delx * delx + dely * dely;
            if dist <= thresh_dist {
                return *bt;
            }
            if dist < min_dist {
                min_index = i;
                min_dist = dist;
            }
        }
    }

    if small_found {
        return *boxa.get(min_index).unwrap();
    }

    // No small boxes; return smallest of large
    let mut min_size = i32::MAX;
    let mut min_idx = 0;
    for i in 0..n {
        if let Some(bt) = boxa.get(i) {
            let perim = bt.w + bt.h;
            if perim < min_size {
                min_size = perim;
                min_idx = i;
            }
        }
    }
    *boxa.get(min_idx).unwrap()
}

fn boxa_generate_subboxes(boxx: &Box, boxa: &Boxa, max_perim: i32, fract: f32) -> Boxa {
    let pivot = boxa_select_pivot_box(boxx, boxa, max_perim, fract);
    let (x, y, w, h) = (boxx.x, boxx.y, boxx.w, boxx.h);
    let (xp, yp, wp, hp) = (pivot.x, pivot.y, pivot.w, pivot.h);

    let mut boxa4 = Boxa::with_capacity(4);

    // Left sub-box
    if let (true, Ok(b)) = (xp > x, Box::new(x, y, xp - x, h)) {
        boxa4.push(b);
    }
    // Top sub-box
    if let (true, Ok(b)) = (yp > y, Box::new(x, y, w, yp - y)) {
        boxa4.push(b);
    }
    // Right sub-box
    if let (true, Ok(b)) = (xp + wp < x + w, Box::new(xp + wp, y, x + w - xp - wp, h)) {
        boxa4.push(b);
    }
    // Bottom sub-box
    if let (true, Ok(b)) = (yp + hp < y + h, Box::new(x, yp + hp, w, y + h - yp - hp)) {
        boxa4.push(b);
    }

    boxa4
}

fn box_check_if_overlap_is_big(boxx: &Box, boxa: &Boxa, max_overlap: f32) -> bool {
    if boxa.is_empty() || max_overlap >= 1.0 {
        return false;
    }

    for i in 0..boxa.len() {
        if let Some(bt) = boxa.get(i) {
            let fract = bt.overlap_fraction(boxx);
            if fract > max_overlap as f64 {
                return true;
            }
        }
    }
    false
}

/// Find white (empty) rectangular blocks in a box arrangement
///
/// Uses Breuel's recursive rectangle partitioning algorithm.
///
/// # Arguments
/// * `boxas` - bounding boxes of foreground components
/// * `boxx` - initial region (None to compute from boxas)
/// * `sort_flag` - criterion for sorting whitespace blocks
/// * `max_boxes` - maximum number of output whitespace boxes
/// * `max_overlap` - maximum fractional overlap (0.0-1.0)
/// * `max_perim` - max half-perimeter for pivot selection by proximity
/// * `fract` - fraction of diagonal for acceptable pivot distance
/// * `max_pops` - max pops from heap (0 for default=20000)
///
/// # See also
/// C Leptonica: `boxaGetWhiteblocks()` in `partition.c`
#[allow(clippy::too_many_arguments)]
pub fn boxa_get_whiteblocks(
    boxas: &Boxa,
    boxx: Option<&Box>,
    sort_flag: WhiteblockSort,
    max_boxes: usize,
    max_overlap: f32,
    max_perim: i32,
    fract: f32,
    max_pops: usize,
) -> RegionResult<Boxa> {
    if !(0.0..=1.0).contains(&max_overlap) {
        return Err(RegionError::InvalidParameters(
            "max_overlap must be in [0.0, 1.0]".into(),
        ));
    }

    let max_pops = if max_pops == 0 { 20000 } else { max_pops };
    let max_boxes = max_boxes.max(1);

    let initial_box = match boxx {
        Some(b) => *b,
        None => {
            let ext = boxas.get_extent();
            match ext {
                Some((w, h, _)) => Box::new(0, 0, w, h).map_err(RegionError::Core)?,
                None => {
                    return Err(RegionError::InvalidParameters(
                        "empty boxas and no bounding box".into(),
                    ));
                }
            }
        }
    };

    let mut heap = BinaryHeap::new();
    let mut initial = Partel {
        size: 0.0,
        boxx: initial_box,
        boxa: boxas.clone(),
    };
    partel_set_size(&mut initial, sort_flag);
    heap.push(initial);

    let mut boxad = Boxa::new();
    let mut npop = 0usize;

    while let Some(partel) = heap.pop() {
        npop += 1;
        if npop > max_pops {
            break;
        }

        let n = partel.boxa.len();
        if n == 0 {
            // No intersecting boxes — candidate for output
            if !box_check_if_overlap_is_big(&partel.boxx, &boxad, max_overlap) {
                boxad.push(partel.boxx);
            }
            if boxad.len() >= max_boxes {
                break;
            }
            continue;
        }

        // Generate up to 4 subboxes
        let boxa4 = boxa_generate_subboxes(&partel.boxx, &partel.boxa, max_perim, fract);
        for i in 0..boxa4.len() {
            if let Some(boxsub) = boxa4.get(i) {
                let boxasub = partel.boxa.intersects_box(boxsub);
                let mut new_partel = Partel {
                    size: 0.0,
                    boxx: *boxsub,
                    boxa: boxasub,
                };
                partel_set_size(&mut new_partel, sort_flag);
                heap.push(new_partel);
            }
        }
    }

    Ok(boxad)
}

/// Prune sorted boxes that overlap too much
///
/// Removes smaller boxes when they overlap any larger box by more than
/// the threshold fraction.
///
/// # Arguments
/// * `boxas` - sorted by size in decreasing order
/// * `max_overlap` - maximum fractional overlap (0.0-1.0)
///
/// # See also
/// C Leptonica: `boxaPruneSortedOnOverlap()` in `partition.c`
pub fn boxa_prune_sorted_on_overlap(boxas: &Boxa, max_overlap: f32) -> RegionResult<Boxa> {
    if !(0.0..=1.0).contains(&max_overlap) {
        return Err(RegionError::InvalidParameters(
            "max_overlap must be in [0.0, 1.0]".into(),
        ));
    }

    let n = boxas.len();
    if n == 0 || max_overlap >= 1.0 {
        return Ok(boxas.clone());
    }

    let mut boxad = Boxa::new();
    // Always keep the first (largest) box
    if let Some(b) = boxas.get(0) {
        boxad.push(*b);
    }

    for j in 1..n {
        if let Some(box2) = boxas.get(j) {
            let mut remove = false;
            for i in 0..j {
                if let Some(box1) = boxas.get(i) {
                    let fract = box1.overlap_fraction(box2);
                    if fract > max_overlap as f64 {
                        remove = true;
                        break;
                    }
                }
            }
            if !remove {
                boxad.push(*box2);
            }
        }
    }

    Ok(boxad)
}
