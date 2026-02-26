//! Structuring element generation from image patterns
//!
//! Functions to generate hit-miss SELs from binary image patterns:
//! - Boundary-based generation
//! - Run-length analysis
//! - Random subsampling
//! - Boundary pixel subsampling
//! - Run center extraction
//!
//! # Reference
//!
//! Based on Leptonica's `selgen.c` implementation.

use crate::core::{Numa, Pix, PixelDepth, Pta};
use crate::morph::binary::{BoundaryType, extract_boundary};
use crate::morph::sel::{Sel, SelElement};
use crate::morph::{MorphError, MorphResult, dilate_brick, erode_brick};

/// Generate a hit-miss Sel from the boundary of a binary image.
///
/// Creates a SEL by extracting inner and outer boundaries at specified
/// distances, then subsampling boundary pixels.
///
/// # Arguments
///
/// * `pix` - 1 bpp binary image
/// * `hit_dist` - Distance for inner boundary erosion (0-4)
/// * `miss_dist` - Distance for outer boundary dilation (0-4)
/// * `hit_skip` - Subsampling interval for hit pixels (0 = all)
/// * `miss_skip` - Subsampling interval for miss pixels (0 = all)
/// * `top_flag` - Whether to expand the top boundary
/// * `bot_flag` - Whether to expand the bottom boundary
/// * `left_flag` - Whether to expand the left boundary
/// * `right_flag` - Whether to expand the right boundary
///
/// Based on C leptonica `pixGenerateSelBoundary`.
#[allow(clippy::too_many_arguments)]
pub fn generate_sel_boundary(
    pix: &Pix,
    hit_dist: u32,
    miss_dist: u32,
    hit_skip: i32,
    miss_skip: i32,
    top_flag: bool,
    bot_flag: bool,
    left_flag: bool,
    right_flag: bool,
) -> MorphResult<Sel> {
    check_binary(pix)?;

    if hit_dist > 4 || miss_dist > 4 {
        return Err(MorphError::InvalidParameters(
            "hit_dist and miss_dist must be 0-4".into(),
        ));
    }
    if hit_skip < 0 && miss_skip < 0 {
        return Err(MorphError::InvalidParameters(
            "at least one of hit_skip or miss_skip must be >= 0".into(),
        ));
    }

    // Expand the image based on flags
    let expanded = expand_image(pix, top_flag, bot_flag, left_flag, right_flag, miss_dist)?;
    let ew = expanded.width();
    let eh = expanded.height();

    // Generate hit boundary: erode by hit_dist, then extract inner boundary
    let hit_pta = if hit_skip >= 0 {
        let eroded = if hit_dist > 0 {
            let size = 2 * hit_dist + 1;
            erode_brick(&expanded, size, size)?
        } else {
            expanded.clone()
        };
        let boundary = extract_boundary(&eroded, BoundaryType::Inner)?;
        subsample_boundary_pixels(&boundary, hit_skip as u32)?
    } else {
        Pta::new()
    };

    // Generate miss boundary: dilate by miss_dist, then extract outer boundary
    let miss_pta = if miss_skip >= 0 {
        let dilated = if miss_dist > 0 {
            let size = 2 * miss_dist + 1;
            dilate_brick(&expanded, size, size)?
        } else {
            expanded.clone()
        };
        let boundary = extract_boundary(&dilated, BoundaryType::Outer)?;
        subsample_boundary_pixels(&boundary, miss_skip as u32)?
    } else {
        Pta::new()
    };

    // Create the SEL from the combined points
    let mut sel = Sel::new(ew, eh)?;
    sel.set_origin(ew / 2, eh / 2)?;

    for i in 0..hit_pta.len() {
        if let Some((x, y)) = hit_pta.get_i_pt(i)
            && x >= 0
            && y >= 0
            && (x as u32) < ew
            && (y as u32) < eh
        {
            sel.set_element(x as u32, y as u32, SelElement::Hit);
        }
    }

    for i in 0..miss_pta.len() {
        if let Some((x, y)) = miss_pta.get_i_pt(i)
            && x >= 0
            && y >= 0
            && (x as u32) < ew
            && (y as u32) < eh
        {
            sel.set_element(x as u32, y as u32, SelElement::Miss);
        }
    }

    Ok(sel)
}

/// Generate a Sel using run-length patterns from a binary image.
///
/// Scans horizontal and vertical sample lines, finding foreground run
/// centers to use as hit/miss points.
///
/// # Arguments
///
/// * `pix` - 1 bpp binary image
/// * `nhlines` - Number of horizontal sample lines
/// * `nvlines` - Number of vertical sample lines
/// * `distance` - Erosion/dilation distance for safe FG/BG identification
/// * `min_length` - Minimum run length to include
/// * `toppix`, `botpix`, `leftpix`, `rightpix` - Expansion pixels
///
/// Based on C leptonica `pixGenerateSelWithRuns`.
#[allow(clippy::too_many_arguments)]
pub fn generate_sel_with_runs(
    pix: &Pix,
    nhlines: u32,
    nvlines: u32,
    distance: u32,
    min_length: u32,
    toppix: u32,
    botpix: u32,
    leftpix: u32,
    rightpix: u32,
) -> MorphResult<Sel> {
    check_binary(pix)?;

    let expanded = expand_image_by_pixels(pix, toppix, botpix, leftpix, rightpix)?;
    let ew = expanded.width();
    let eh = expanded.height();

    // Get safe FG pixels by erosion
    let safe_fg = if distance > 0 {
        let size = 2 * distance + 1;
        erode_brick(&expanded, size, size)?
    } else {
        expanded.clone()
    };

    // Get safe BG pixels: dilate, then invert
    let safe_bg = if distance > 0 {
        let size = 2 * distance + 1;
        dilate_brick(&expanded, size, size)?.invert()
    } else {
        expanded.invert()
    };

    let mut sel = Sel::new(ew, eh)?;
    sel.set_origin(ew / 2, eh / 2)?;

    // Sample horizontal lines
    if nhlines > 0 && eh > 0 {
        let step = eh / (nhlines + 1);
        for i in 1..=nhlines {
            let y = (i * step).min(eh - 1);
            let centers = get_run_centers_on_line(&expanded, -1, y as i32, min_length)?;
            for j in 0..centers.len() {
                let x = centers.get(j).unwrap() as u32;
                if x < ew && y < eh && safe_fg.get_pixel_unchecked(x, y) != 0 {
                    sel.set_element(x, y, SelElement::Hit);
                }
            }
            // Also add miss points from BG runs
            let bg_centers = get_bg_run_centers(&expanded, -1, y as i32, min_length)?;
            for j in 0..bg_centers.len() {
                let x = bg_centers.get(j).unwrap() as u32;
                if x < ew && y < eh && safe_bg.get_pixel_unchecked(x, y) != 0 {
                    sel.set_element(x, y, SelElement::Miss);
                }
            }
        }
    }

    // Sample vertical lines
    if nvlines > 0 && ew > 0 {
        let step = ew / (nvlines + 1);
        for i in 1..=nvlines {
            let x = (i * step).min(ew - 1);
            let centers = get_run_centers_on_line(&expanded, x as i32, -1, min_length)?;
            for j in 0..centers.len() {
                let y = centers.get(j).unwrap() as u32;
                if x < ew && y < eh && safe_fg.get_pixel_unchecked(x, y) != 0 {
                    sel.set_element(x, y, SelElement::Hit);
                }
            }
            let bg_centers = get_bg_run_centers(&expanded, x as i32, -1, min_length)?;
            for j in 0..bg_centers.len() {
                let y = bg_centers.get(j).unwrap() as u32;
                if x < ew && y < eh && safe_bg.get_pixel_unchecked(x, y) != 0 {
                    sel.set_element(x, y, SelElement::Miss);
                }
            }
        }
    }

    Ok(sel)
}

/// Generate a random Sel from a binary image by random subsampling.
///
/// # Arguments
///
/// * `pix` - 1 bpp binary image
/// * `hit_fract` - Fraction of safe FG pixels to include as hits (0.0-1.0)
/// * `miss_fract` - Fraction of safe BG pixels to include as misses (0.0-1.0)
/// * `distance` - Erosion/dilation distance for safe pixel identification
/// * `toppix`, `botpix`, `leftpix`, `rightpix` - Expansion pixels
///
/// Based on C leptonica `pixGenerateSelRandom`.
#[allow(clippy::too_many_arguments)]
pub fn generate_sel_random(
    pix: &Pix,
    hit_fract: f32,
    miss_fract: f32,
    distance: u32,
    toppix: u32,
    botpix: u32,
    leftpix: u32,
    rightpix: u32,
) -> MorphResult<Sel> {
    check_binary(pix)?;

    if !(0.0..=1.0).contains(&hit_fract) || !(0.0..=1.0).contains(&miss_fract) {
        return Err(MorphError::InvalidParameters(
            "hit_fract and miss_fract must be in [0.0, 1.0]".into(),
        ));
    }

    let expanded = expand_image_by_pixels(pix, toppix, botpix, leftpix, rightpix)?;
    let ew = expanded.width();
    let eh = expanded.height();

    // Safe FG and BG
    let safe_fg = if distance > 0 {
        let size = 2 * distance + 1;
        erode_brick(&expanded, size, size)?
    } else {
        expanded.clone()
    };

    let safe_bg = if distance > 0 {
        let size = 2 * distance + 1;
        dilate_brick(&expanded, size, size)?.invert()
    } else {
        expanded.invert()
    };

    let mut sel = Sel::new(ew, eh)?;
    sel.set_origin(ew / 2, eh / 2)?;

    // Simple deterministic hash-based selection for reproducibility
    let mut counter: u64 = 0;
    for y in 0..eh {
        for x in 0..ew {
            counter += 1;
            let hash = simple_hash(counter);
            let norm = (hash % 10000) as f32 / 10000.0;

            if safe_fg.get_pixel_unchecked(x, y) != 0 && norm < hit_fract {
                sel.set_element(x, y, SelElement::Hit);
            } else if safe_bg.get_pixel_unchecked(x, y) != 0 && norm < miss_fract {
                sel.set_element(x, y, SelElement::Miss);
            }
        }
    }

    Ok(sel)
}

/// Get centers of foreground runs on a specified line.
///
/// # Arguments
///
/// * `pix` - 1 bpp binary image
/// * `x` - X coordinate for vertical line (-1 for horizontal)
/// * `y` - Y coordinate for horizontal line (-1 for vertical)
/// * `min_length` - Minimum foreground run length to include
///
/// Returns a Numa of center coordinates of qualifying runs.
///
/// Based on C leptonica `pixGetRunCentersOnLine`.
pub fn get_run_centers_on_line(pix: &Pix, x: i32, y: i32, min_length: u32) -> MorphResult<Numa> {
    check_binary(pix)?;

    let runs = if x < 0 {
        // Horizontal line at row y
        if y < 0 || y as u32 >= pix.height() {
            return Err(MorphError::InvalidParameters("y out of range".into()));
        }
        get_runs_on_line(pix, 0, y, pix.width() as i32 - 1, y)?
    } else if y < 0 {
        // Vertical line at column x
        if x as u32 >= pix.width() {
            return Err(MorphError::InvalidParameters("x out of range".into()));
        }
        get_runs_on_line(pix, x, 0, x, pix.height() as i32 - 1)?
    } else {
        return Err(MorphError::InvalidParameters(
            "exactly one of x or y must be -1".into(),
        ));
    };

    let mut centers = Numa::new();
    // Runs alternate: bg, fg, bg, fg, ...
    // First run is always background
    let mut pos: f32 = 0.0;
    for i in 0..runs.len() {
        let run_len = runs.get(i).unwrap();
        if i % 2 == 1 {
            // Foreground run
            if run_len as u32 >= min_length {
                let center = pos + run_len / 2.0;
                centers.push(center);
            }
        }
        pos += run_len;
    }

    Ok(centers)
}

/// Get all run lengths on a specified line.
///
/// Returns alternating background/foreground run lengths. The first
/// run is always background (may be 0 if line starts with foreground).
///
/// # Arguments
///
/// * `pix` - 1 bpp binary image
/// * `x1`, `y1` - Start point
/// * `x2`, `y2` - End point
///
/// Based on C leptonica `pixGetRunsOnLine`.
pub fn get_runs_on_line(pix: &Pix, x1: i32, y1: i32, x2: i32, y2: i32) -> MorphResult<Numa> {
    check_binary(pix)?;

    let w = pix.width() as i32;
    let h = pix.height() as i32;

    // Generate line pixels using Bresenham
    let points = generate_line_points(x1, y1, x2, y2);

    if points.is_empty() {
        return Ok(Numa::new());
    }

    let mut runs = Numa::new();

    // Get first pixel value
    let (fx, fy) = points[0];
    let first_val = if fx >= 0 && fy >= 0 && fx < w && fy < h {
        pix.get_pixel_unchecked(fx as u32, fy as u32)
    } else {
        0
    };

    // If first pixel is foreground, prepend a 0-length background run
    if first_val != 0 {
        runs.push(0.0);
    }

    let mut current_fg = first_val != 0;
    let mut run_len: f32 = 1.0;

    for &(px, py) in &points[1..] {
        let val = if px >= 0 && py >= 0 && px < w && py < h {
            pix.get_pixel_unchecked(px as u32, py as u32)
        } else {
            0
        };
        let is_fg = val != 0;

        if is_fg == current_fg {
            run_len += 1.0;
        } else {
            runs.push(run_len);
            run_len = 1.0;
            current_fg = is_fg;
        }
    }
    runs.push(run_len);

    Ok(runs)
}

/// Subsample boundary pixels from a binary image at regular intervals.
///
/// If skip=0, returns all foreground pixels. Otherwise, traces boundaries
/// and takes every (skip+1)th pixel.
///
/// # Arguments
///
/// * `pix` - 1 bpp binary image containing boundary pixels
/// * `skip` - Subsampling interval (0 = return all)
///
/// Based on C leptonica `pixSubsampleBoundaryPixels`.
pub fn subsample_boundary_pixels(pix: &Pix, skip: u32) -> MorphResult<Pta> {
    check_binary(pix)?;

    let w = pix.width();
    let h = pix.height();

    if skip == 0 {
        // Return all foreground pixels
        let mut pta = Pta::new();
        for y in 0..h {
            for x in 0..w {
                if pix.get_pixel_unchecked(x, y) != 0 {
                    pta.push(x as f32, y as f32);
                }
            }
        }
        return Ok(pta);
    }

    // Trace boundaries with subsampling
    let mut pta = Pta::new();
    let mut visited = vec![false; (w * h) as usize];

    for y in 0..h {
        for x in 0..w {
            if pix.get_pixel_unchecked(x, y) != 0 && !visited[(y * w + x) as usize] {
                // Start tracing this boundary
                trace_boundary_subsampled(pix, x, y, skip, &mut visited, &mut pta);
            }
        }
    }

    Ok(pta)
}

// ── Helper functions ────────────────────────────────────────────────────────

fn check_binary(pix: &Pix) -> MorphResult<()> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

/// Expand image with border padding based on flags.
fn expand_image(
    pix: &Pix,
    top: bool,
    bot: bool,
    left: bool,
    right: bool,
    dist: u32,
) -> MorphResult<Pix> {
    let pad_top = if top { dist } else { 0 };
    let pad_bot = if bot { dist } else { 0 };
    let pad_left = if left { dist } else { 0 };
    let pad_right = if right { dist } else { 0 };
    expand_image_by_pixels(pix, pad_top, pad_bot, pad_left, pad_right)
}

/// Expand image by adding border pixels.
fn expand_image_by_pixels(
    pix: &Pix,
    top: u32,
    bot: u32,
    left: u32,
    right: u32,
) -> MorphResult<Pix> {
    if top == 0 && bot == 0 && left == 0 && right == 0 {
        return Ok(pix.clone());
    }

    let ow = pix.width();
    let oh = pix.height();
    let nw = ow + left + right;
    let nh = oh + top + bot;

    let out = Pix::new(nw, nh, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..oh {
        for x in 0..ow {
            let val = pix.get_pixel_unchecked(x, y);
            if val != 0 {
                out_mut.set_pixel_unchecked(x + left, y + top, 1);
            }
        }
    }

    Ok(out_mut.into())
}

/// Get background run centers on a line (complement of foreground).
fn get_bg_run_centers(pix: &Pix, x: i32, y: i32, min_length: u32) -> MorphResult<Numa> {
    let runs = if x < 0 {
        get_runs_on_line(pix, 0, y, pix.width() as i32 - 1, y)?
    } else {
        get_runs_on_line(pix, x, 0, x, pix.height() as i32 - 1)?
    };

    let mut centers = Numa::new();
    let mut pos: f32 = 0.0;
    for i in 0..runs.len() {
        let run_len = runs.get(i).unwrap();
        if i % 2 == 0 {
            // Background run
            if run_len as u32 >= min_length {
                let center = pos + run_len / 2.0;
                centers.push(center);
            }
        }
        pos += run_len;
    }

    Ok(centers)
}

/// Generate line points using Bresenham's algorithm.
fn generate_line_points(x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x1;
    let mut cy = y1;

    loop {
        points.push((cx, cy));
        if cx == x2 && cy == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
    }

    points
}

/// Trace a single boundary with subsampling.
fn trace_boundary_subsampled(
    pix: &Pix,
    start_x: u32,
    start_y: u32,
    skip: u32,
    visited: &mut [bool],
    pta: &mut Pta,
) {
    let w = pix.width();
    let h = pix.height();
    let interval = skip + 1;
    let mut count = 0u32;

    // Use a simple contour tracing: follow connected foreground pixels
    let mut x = start_x;
    let mut y = start_y;

    loop {
        let idx = (y * w + x) as usize;
        if visited[idx] {
            break;
        }
        visited[idx] = true;

        if count.is_multiple_of(interval) {
            pta.push(x as f32, y as f32);
        }
        count += 1;

        // Find next unvisited neighbor (4-connected first, then diagonal)
        if let Some((nx, ny)) = find_next_boundary_pixel(pix, x, y, w, h, visited) {
            x = nx;
            y = ny;
        } else {
            break;
        }
    }
}

/// Find next unvisited foreground neighbor pixel.
fn find_next_boundary_pixel(
    pix: &Pix,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    visited: &[bool],
) -> Option<(u32, u32)> {
    // 4-connected neighbors first, then diagonals
    let neighbors: [(i32, i32); 8] = [
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
        (1, 1),
        (-1, 1),
        (1, -1),
        (-1, -1),
    ];

    for &(dx, dy) in &neighbors {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h {
            let nidx = (ny as u32 * w + nx as u32) as usize;
            if !visited[nidx] && pix.get_pixel_unchecked(nx as u32, ny as u32) != 0 {
                return Some((nx as u32, ny as u32));
            }
        }
    }
    None
}

/// Simple deterministic hash for reproducible random selection.
fn simple_hash(x: u64) -> u64 {
    let mut h = x;
    h = h.wrapping_mul(6364136223846793005);
    h = h.wrapping_add(1442695040888963407);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h
}
