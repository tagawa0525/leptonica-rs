//! Run-length analysis for binary images
//!
//! This module provides functions for computing run-length transforms
//! and related operations on binary images.
//!
//! # See also
//! C Leptonica: `runlength.c`

use crate::core::{Numa, Pix, PixelDepth, Result};

/// Direction of run-length computation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunDirection {
    /// Horizontal runs along scanlines
    Horizontal,
    /// Vertical runs along columns
    Vertical,
}

/// Transform a binary image to grayscale showing run lengths
///
/// Each pixel value in the output equals the length of the run it belongs to.
/// Pixels not in the chosen color get value 0.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `color` - 0 for white runs, 1 for black runs
/// * `direction` - horizontal or vertical runs
/// * `depth` - output depth: 8 or 16 bpp
///
/// # See also
/// C Leptonica: `pixRunlengthTransform()` in `runlength.c`
pub fn runlength_transform(
    pix: &Pix,
    color: u32,
    direction: RunDirection,
    depth: PixelDepth,
) -> Result<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(crate::core::Error::UnsupportedDepth(pix.depth().bits()));
    }
    if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit16 {
        return Err(crate::core::Error::InvalidParameter(
            "depth must be 8 or 16 bpp".into(),
        ));
    }

    let w = pix.width();
    let h = pix.height();
    let max_size = match direction {
        RunDirection::Horizontal => 1 + w / 2,
        RunDirection::Vertical => 1 + h / 2,
    } as usize;
    let buf_size = w.max(h) as usize;

    if buf_size > 1_000_000 {
        return Err(crate::core::Error::InvalidParameter(
            "image dimension too large".into(),
        ));
    }

    let pixd = Pix::new(w, h, depth)?;
    let mut pixd = pixd
        .try_into_mut()
        .map_err(|_| crate::core::Error::AllocationFailed)?;

    let mut start = vec![0i32; max_size];
    let mut end = vec![0i32; max_size];
    let mut buffer = vec![0i32; buf_size];

    // Use fg runs: invert if looking for white runs
    let work_pix = if color == 0 {
        pix.invert()
    } else {
        pix.clone()
    };

    match direction {
        RunDirection::Horizontal => {
            for i in 0..h {
                let n = find_horizontal_runs(&work_pix, i, &mut start, &mut end);
                runlength_membership_on_line(&mut buffer, w as usize, depth, &start, &end, n);
                for j in 0..w {
                    pixd.set_pixel(j, i, buffer[j as usize] as u32)?;
                }
            }
        }
        RunDirection::Vertical => {
            for j in 0..w {
                let n = find_vertical_runs(&work_pix, j, &mut start, &mut end);
                runlength_membership_on_line(&mut buffer, h as usize, depth, &start, &end, n);
                for i in 0..h {
                    pixd.set_pixel(j, i, buffer[i as usize] as u32)?;
                }
            }
        }
    }

    Ok(pixd.into())
}

/// Compute run-length membership values for a line
///
/// Each position in the buffer gets the length of the run it belongs to,
/// clipped to max pixel value. Positions not in any run get 0.
///
/// # See also
/// C Leptonica: `runlengthMembershipOnLine()` in `runlength.c`
pub fn runlength_membership_on_line(
    buffer: &mut [i32],
    size: usize,
    depth: PixelDepth,
    start: &[i32],
    end: &[i32],
    n: usize,
) {
    let max_val = match depth {
        PixelDepth::Bit8 => 0xff,
        PixelDepth::Bit16 => 0xffff,
        _ => 0xff,
    };

    for b in buffer.iter_mut().take(size) {
        *b = 0;
    }

    for i in 0..n {
        let first = start[i] as usize;
        let last = end[i] as usize;
        let diff = (last - first + 1).min(max_val);
        let end = (last + 1).min(size);
        for b in buffer[first..end].iter_mut() {
            *b = diff as i32;
        }
    }
}

/// Make MSB location lookup table
///
/// For each byte value (0-255), returns the position of the most significant
/// bit with the specified value. Position 0 is the MSB. Returns 8 if no
/// bit with the specified value is found.
///
/// # See also
/// C Leptonica: `makeMSBitLocTab()` in `runlength.c`
pub fn make_msbit_loc_tab(bitval: u32) -> Vec<i32> {
    let mut tab = vec![0i32; 256];
    for i in 0..256u16 {
        let mut byte = i as u8;
        if bitval == 0 {
            byte = !byte;
        }
        tab[i as usize] = 8;
        let mut mask: u8 = 0x80;
        for j in 0..8 {
            if byte & mask != 0 {
                tab[i as usize] = j;
                break;
            }
            mask >>= 1;
        }
    }
    tab
}

/// Find foreground horizontal runs on a single scanline
///
/// Returns the number of runs found.
///
/// # See also
/// C Leptonica: `pixFindHorizontalRuns()` in `runlength.c`
pub fn find_horizontal_runs(pix: &Pix, y: u32, start: &mut [i32], end: &mut [i32]) -> usize {
    let w = pix.width();
    let mut in_run = false;
    let mut index = 0;

    for j in 0..w {
        let val = pix.get_pixel(j, y).unwrap_or(0);
        if !in_run {
            if val != 0 {
                start[index] = j as i32;
                in_run = true;
            }
        } else if val == 0 {
            end[index] = j as i32 - 1;
            index += 1;
            in_run = false;
        }
    }

    if in_run {
        end[index] = w as i32 - 1;
        index += 1;
    }

    index
}

/// Find foreground vertical runs on a single column
///
/// Returns the number of runs found.
///
/// # See also
/// C Leptonica: `pixFindVerticalRuns()` in `runlength.c`
pub fn find_vertical_runs(pix: &Pix, x: u32, start: &mut [i32], end: &mut [i32]) -> usize {
    let h = pix.height();
    let mut in_run = false;
    let mut index = 0;

    for i in 0..h {
        let val = pix.get_pixel(x, i).unwrap_or(0);
        if !in_run {
            if val != 0 {
                start[index] = i as i32;
                in_run = true;
            }
        } else if val == 0 {
            end[index] = i as i32 - 1;
            index += 1;
            in_run = false;
        }
    }

    if in_run {
        end[index] = h as i32 - 1;
        index += 1;
    }

    index
}

/// Find the longest foreground horizontal run on a scanline
///
/// Returns (start_position, run_length).
///
/// # See also
/// C Leptonica: `pixFindMaxHorizontalRunOnLine()` in `runlength.c`
pub fn find_max_horizontal_run_on_line(pix: &Pix, y: u32) -> (u32, u32) {
    let w = pix.width();
    let mut in_run = false;
    let mut start = 0u32;
    let mut max_start = 0u32;
    let mut max_size = 0u32;

    for j in 0..w {
        let val = pix.get_pixel(j, y).unwrap_or(0);
        if !in_run {
            if val != 0 {
                start = j;
                in_run = true;
            }
        } else if val == 0 {
            let length = j - start;
            if length > max_size {
                max_size = length;
                max_start = start;
            }
            in_run = false;
        }
    }

    if in_run {
        let length = w - start;
        if length > max_size {
            max_size = length;
            max_start = start;
        }
    }

    (max_start, max_size)
}

/// Find the longest foreground vertical run on a column
///
/// Returns (start_position, run_length).
///
/// # See also
/// C Leptonica: `pixFindMaxVerticalRunOnLine()` in `runlength.c`
pub fn find_max_vertical_run_on_line(pix: &Pix, x: u32) -> (u32, u32) {
    let h = pix.height();
    let mut in_run = false;
    let mut start = 0u32;
    let mut max_start = 0u32;
    let mut max_size = 0u32;

    for i in 0..h {
        let val = pix.get_pixel(x, i).unwrap_or(0);
        if !in_run {
            if val != 0 {
                start = i;
                in_run = true;
            }
        } else if val == 0 {
            let length = i - start;
            if length > max_size {
                max_size = length;
                max_start = start;
            }
            in_run = false;
        }
    }

    if in_run {
        let length = h - start;
        if length > max_size {
            max_size = length;
            max_start = start;
        }
    }

    (max_start, max_size)
}

/// Find maximum run lengths for all rows or columns
///
/// Returns a Numa of max run lengths (one per row or column).
///
/// # See also
/// C Leptonica: `pixFindMaxRuns()` in `runlength.c`
pub fn find_max_runs(pix: &Pix, direction: RunDirection) -> Result<(Numa, Numa)> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(crate::core::Error::UnsupportedDepth(pix.depth().bits()));
    }

    let w = pix.width();
    let h = pix.height();

    let mut na_size = Numa::new();
    let mut na_start = Numa::new();

    match direction {
        RunDirection::Horizontal => {
            for i in 0..h {
                let (start, size) = find_max_horizontal_run_on_line(pix, i);
                na_size.push(size as f32);
                na_start.push(start as f32);
            }
        }
        RunDirection::Vertical => {
            for j in 0..w {
                let (start, size) = find_max_vertical_run_on_line(pix, j);
                na_size.push(size as f32);
                na_start.push(start as f32);
            }
        }
    }

    Ok((na_size, na_start))
}
