//! Checkerboard corner detection
//!
//! Find corners where four squares meet in a checkerboard pattern
//! using hit-miss morphology.
//!
//! # See also
//! C Leptonica: `checkerboard.c`

use crate::core::{Boxa, Pix, Pta};
use crate::morph;
use crate::morph::{MorphOpType, Sel};
use crate::region;
use crate::region::{ConnectivityType, RegionError, RegionResult};

/// Find corners in a checkerboard pattern
///
/// Uses hit-miss transforms with diagonal structuring elements to
/// detect the points where four squares meet.
///
/// # Arguments
/// * `pix` - checkerboard image (any depth, converted to 1 bpp internally)
/// * `size` - size of HMT sel (>= 7, default 7)
/// * `dilation` - dilation size for hit/miss squares (1-5, typically 1 or 3)
/// * `nsels` - number of sels to use (2 or 4; use 4 for >20° rotation)
///
/// # Returns
/// `(corner_pix, corner_points)` - 1 bpp image of corners and their coordinates
///
/// # See also
/// C Leptonica: `pixFindCheckerboardCorners()` in `checkerboard.c`
pub fn find_checkerboard_corners(
    pix: &Pix,
    size: u32,
    dilation: u32,
    nsels: u32,
) -> RegionResult<(Pix, Pta)> {
    let size = if size == 0 { 7 } else { size };
    if size < 7 {
        return Err(RegionError::InvalidParameters("size must be >= 7".into()));
    }
    if !(1..=5).contains(&dilation) {
        return Err(RegionError::InvalidParameters(
            "dilation must be in [1..5]".into(),
        ));
    }
    if nsels != 2 && nsels != 4 {
        return Err(RegionError::InvalidParameters(
            "nsels must be 2 or 4".into(),
        ));
    }

    // Generate hit-miss sels for corners
    let sels = make_checkerboard_corner_sels(size, dilation, nsels)?;

    // Do HMT to find corner locations
    let pix1 = morph::union_of_morph_ops(pix, &sels, MorphOpType::HitMiss)
        .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))?;

    // Remove large noise CCs
    let pix2 = region::pix_select_by_size(
        &pix1,
        size as i32,
        size as i32,
        ConnectivityType::EightWay,
        region::SizeSelectType::IfBoth,
        region::SizeSelectRelation::Lte,
    )?;

    // Thin remaining CCs to single pixels
    let pix3 = morph::thin_connected(
        &pix2,
        morph::ThinType::Foreground,
        morph::Connectivity::Eight,
        0,
    )
    .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))?;

    // Extract center coordinates of each CC
    let (boxa, _) = region::conncomp_pixa(&pix3, ConnectivityType::EightWay)?;
    let pta = boxa_extract_centers(&boxa);

    Ok((pix3, pta))
}

/// Extract center points from bounding boxes
fn boxa_extract_centers(boxa: &Boxa) -> Pta {
    let mut pta = Pta::new();
    for i in 0..boxa.len() {
        if let Some(b) = boxa.get(i) {
            pta.push(b.center_x() as f32, b.center_y() as f32);
        }
    }
    pta
}

/// Generate diagonal hit-miss structuring elements for corner detection
fn make_checkerboard_corner_sels(size: u32, dilation: u32, nsels: u32) -> RegionResult<Vec<Sel>> {
    let mut sels = Vec::with_capacity(nsels as usize);
    let s = size as usize;
    let half = s / 2;

    // Sel 1: diagonal pattern (top-left hit, bottom-right hit; top-right miss, bottom-left miss)
    let sel1 = make_diagonal_sel(s, half, dilation, false)?;
    sels.push(sel1);

    // Sel 2: opposite diagonal pattern
    let sel2 = make_diagonal_sel(s, half, dilation, true)?;
    sels.push(sel2);

    if nsels == 4 {
        // Sel 3: cross pattern variant 1
        let sel3 = make_cross_sel(s, half, dilation, false)?;
        sels.push(sel3);

        // Sel 4: cross pattern variant 2
        let sel4 = make_cross_sel(s, half, dilation, true)?;
        sels.push(sel4);
    }

    Ok(sels)
}

/// Make a diagonal corner sel
fn make_diagonal_sel(size: usize, half: usize, dilation: u32, flipped: bool) -> RegionResult<Sel> {
    let mut pattern = String::new();
    let d = dilation as usize;

    for y in 0..size {
        for x in 0..size {
            if y == half && x == half {
                pattern.push('C');
            } else {
                let is_hit;
                let is_miss;

                if !flipped {
                    // Top-left and bottom-right are hits; top-right and bottom-left are misses
                    is_hit = (x < half.saturating_sub(d) && y < half.saturating_sub(d))
                        || (x > half + d && y > half + d);
                    is_miss = (x > half + d && y < half.saturating_sub(d))
                        || (x < half.saturating_sub(d) && y > half + d);
                } else {
                    // Top-right and bottom-left are hits; top-left and bottom-right are misses
                    is_hit = (x > half + d && y < half.saturating_sub(d))
                        || (x < half.saturating_sub(d) && y > half + d);
                    is_miss = (x < half.saturating_sub(d) && y < half.saturating_sub(d))
                        || (x > half + d && y > half + d);
                }

                if is_hit {
                    pattern.push('x');
                } else if is_miss {
                    pattern.push('o');
                } else {
                    pattern.push(' ');
                }
            }
        }
        if y < size - 1 {
            pattern.push('\n');
        }
    }

    Sel::from_string(&pattern, half as u32, half as u32)
        .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))
}

/// Make a cross corner sel
fn make_cross_sel(size: usize, half: usize, dilation: u32, flipped: bool) -> RegionResult<Sel> {
    let mut pattern = String::new();
    let d = dilation as usize;

    for y in 0..size {
        for x in 0..size {
            if y == half && x == half {
                pattern.push('C');
            } else {
                let is_hit;
                let is_miss;

                if !flipped {
                    // Top and bottom center are hits; left and right center are misses
                    is_hit = (x >= half.saturating_sub(d)
                        && x <= half + d
                        && y < half.saturating_sub(d))
                        || (x >= half.saturating_sub(d) && x <= half + d && y > half + d);
                    is_miss = (y >= half.saturating_sub(d)
                        && y <= half + d
                        && x < half.saturating_sub(d))
                        || (y >= half.saturating_sub(d) && y <= half + d && x > half + d);
                } else {
                    // Left and right center are hits; top and bottom center are misses
                    is_hit = (y >= half.saturating_sub(d)
                        && y <= half + d
                        && x < half.saturating_sub(d))
                        || (y >= half.saturating_sub(d) && y <= half + d && x > half + d);
                    is_miss = (x >= half.saturating_sub(d)
                        && x <= half + d
                        && y < half.saturating_sub(d))
                        || (x >= half.saturating_sub(d) && x <= half + d && y > half + d);
                }

                if is_hit {
                    pattern.push('x');
                } else if is_miss {
                    pattern.push('o');
                } else {
                    pattern.push(' ');
                }
            }
        }
        if y < size - 1 {
            pattern.push('\n');
        }
    }

    Sel::from_string(&pattern, half as u32, half as u32)
        .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))
}
