//! DWA (Destination Word Accumulation) - High-speed morphological operations
//!
//! DWA is a technique for accelerating binary morphological operations by
//! operating on aligned words (32 or 64 bits) instead of individual pixels.
//!
//! This module provides optimized implementations for brick (rectangular)
//! structuring elements using word-aligned bit operations.
//!
//! # Performance
//!
//! DWA operations are typically 3-10x faster than pixel-by-pixel implementations,
//! especially for larger structuring elements.
//!
//! # Example
//!
//! ```no_run
//! use leptonica_core::Pix;
//! use leptonica_morph::dwa;
//!
//! # fn example(pix: &Pix) -> leptonica_morph::MorphResult<()> {
//! // Fast dilation with a 5x5 brick
//! let dilated = dwa::dilate_brick_dwa(pix, 5, 5)?;
//!
//! // Fast erosion with a 3x3 brick
//! let eroded = dwa::erode_brick_dwa(pix, 3, 3)?;
//!
//! // Fast opening (erosion followed by dilation)
//! let opened = dwa::open_brick_dwa(pix, 5, 5)?;
//!
//! // Fast closing (dilation followed by erosion)
//! let closed = dwa::close_brick_dwa(pix, 5, 5)?;
//! # Ok(())
//! # }
//! ```

use crate::{MorphError, MorphResult};
use leptonica_core::{Pix, PixelDepth};

/// Check that the image is binary (1-bpp)
fn check_binary(pix: &Pix) -> MorphResult<()> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

/// DWA dilation with a brick (rectangular) structuring element
///
/// Performs fast morphological dilation using word-aligned bit operations.
/// The operation is separable: horizontal dilation followed by vertical dilation.
///
/// # Arguments
///
/// * `pix` - Input binary (1-bpp) image
/// * `hsize` - Horizontal size of the brick (width)
/// * `vsize` - Vertical size of the brick (height)
///
/// # Returns
///
/// Dilated binary image
///
/// # Example
///
/// ```no_run
/// use leptonica_core::Pix;
/// use leptonica_morph::dwa::dilate_brick_dwa;
///
/// # fn example(pix: &Pix) -> leptonica_morph::MorphResult<()> {
/// let dilated = dilate_brick_dwa(pix, 5, 5)?;
/// # Ok(())
/// # }
/// ```
pub fn dilate_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;

    if hsize == 0 || vsize == 0 {
        return Err(MorphError::InvalidSel(
            "hsize and vsize must be > 0".to_string(),
        ));
    }

    // Identity case
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    // Separable: horizontal then vertical
    let mut result = if hsize > 1 {
        dilate_horizontal_dwa(pix, hsize)?
    } else {
        pix.clone()
    };

    if vsize > 1 {
        result = dilate_vertical_dwa(&result, vsize)?;
    }

    Ok(result)
}

/// DWA erosion with a brick (rectangular) structuring element
///
/// Performs fast morphological erosion using word-aligned bit operations.
/// The operation is separable: horizontal erosion followed by vertical erosion.
///
/// # Arguments
///
/// * `pix` - Input binary (1-bpp) image
/// * `hsize` - Horizontal size of the brick (width)
/// * `vsize` - Vertical size of the brick (height)
///
/// # Returns
///
/// Eroded binary image
pub fn erode_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;

    if hsize == 0 || vsize == 0 {
        return Err(MorphError::InvalidSel(
            "hsize and vsize must be > 0".to_string(),
        ));
    }

    // Identity case
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    // Separable: horizontal then vertical
    let mut result = if hsize > 1 {
        erode_horizontal_dwa(pix, hsize)?
    } else {
        pix.clone()
    };

    if vsize > 1 {
        result = erode_vertical_dwa(&result, vsize)?;
    }

    Ok(result)
}

/// DWA opening with a brick (rectangular) structuring element
///
/// Opening = Erosion followed by Dilation.
/// Removes small foreground objects and smooths contours.
///
/// # Arguments
///
/// * `pix` - Input binary (1-bpp) image
/// * `hsize` - Horizontal size of the brick (width)
/// * `vsize` - Vertical size of the brick (height)
///
/// # Returns
///
/// Opened binary image
pub fn open_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let eroded = erode_brick_dwa(pix, hsize, vsize)?;
    dilate_brick_dwa(&eroded, hsize, vsize)
}

/// DWA closing with a brick (rectangular) structuring element
///
/// Closing = Dilation followed by Erosion.
/// Fills small holes and connects nearby objects.
///
/// # Arguments
///
/// * `pix` - Input binary (1-bpp) image
/// * `hsize` - Horizontal size of the brick (width)
/// * `vsize` - Vertical size of the brick (height)
///
/// # Returns
///
/// Closed binary image
pub fn close_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let dilated = dilate_brick_dwa(pix, hsize, vsize)?;
    erode_brick_dwa(&dilated, hsize, vsize)
}

/// Shift a row of words by `shift` bit positions.
///
/// Positive shift = left (toward higher bit numbers / lower pixel indices).
/// Negative shift = right (toward lower bit numbers / higher pixel indices).
/// Out-of-range words are treated as 0 (asymmetric boundary condition).
fn shift_row(row: &[u32], wpl: usize, shift: i32, out: &mut [u32]) {
    let abs_shift = shift.unsigned_abs() as usize;
    let word_offset = abs_shift / 32;
    let bit_shift = (abs_shift % 32) as u32;

    for i in 0..wpl {
        if shift > 0 {
            // Left shift: source is further right in the row
            let src_idx = i + word_offset;
            let hi = if src_idx < wpl { row[src_idx] } else { 0 };
            let lo = if src_idx + 1 < wpl {
                row[src_idx + 1]
            } else {
                0
            };
            out[i] = if bit_shift == 0 {
                hi
            } else {
                (hi << bit_shift) | (lo >> (32 - bit_shift))
            };
        } else if shift < 0 {
            // Right shift: source is further left in the row
            let hi = if i > word_offset {
                row[i - word_offset - 1]
            } else {
                0
            };
            let lo = if i >= word_offset {
                row[i - word_offset]
            } else {
                0
            };
            out[i] = if bit_shift == 0 {
                lo
            } else {
                (lo >> bit_shift) | (hi << (32 - bit_shift))
            };
        } else {
            out[i] = row[i];
        }
    }
}

/// Compute the mask to clear padding bits in the last word of a row.
///
/// For images whose width is not a multiple of 32, the last word contains
/// padding bits (LSBs) that must remain 0 to avoid corrupting subsequent
/// operations (e.g., close = dilate then erode).
fn last_word_mask(width: u32) -> u32 {
    let rem = width % 32;
    if rem == 0 { !0 } else { !0u32 << (32 - rem) }
}

/// Horizontal dilation using word-aligned shift operations
///
/// For each pixel, if ANY pixel in the horizontal neighborhood is set,
/// the output pixel is set. Operates on whole 32-bit words, giving
/// ~32x speedup over the per-bit approach.
fn dilate_horizontal_dwa(pix: &Pix, hsize: u32) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let wpl = pix.wpl() as usize;

    let origin = (hsize / 2) as i32;
    let left = -origin;
    let right = hsize as i32 - 1 - origin;

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let src_data = pix.data();
    let dst_data = out_mut.data_mut();

    let mut shifted = vec![0u32; wpl];
    let mask = last_word_mask(w);

    for y in 0..h as usize {
        let src_row = &src_data[y * wpl..(y + 1) * wpl];
        let dst_row = &mut dst_data[y * wpl..(y + 1) * wpl];

        // Initialize accumulator to 0 (OR identity)
        for dst_word in dst_row.iter_mut() {
            *dst_word = 0;
        }

        for d in left..=right {
            shift_row(src_row, wpl, d, &mut shifted);
            for i in 0..wpl {
                dst_row[i] |= shifted[i];
            }
        }

        // Clear padding bits in the last word
        dst_row[wpl - 1] &= mask;
    }

    Ok(out_mut.into())
}

/// Vertical dilation using word-aligned operations
///
/// For each pixel, if ANY pixel in the vertical neighborhood is set,
/// the output pixel is set.
fn dilate_vertical_dwa(pix: &Pix, vsize: u32) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let wpl = pix.wpl();

    // Match Sel::create_brick origin convention: origin at vsize/2
    let origin = (vsize / 2) as i32;
    let top = -origin;
    let bottom = vsize as i32 - 1 - origin;

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let src_data = pix.data();
    let dst_data = out_mut.data_mut();

    // Process each column
    for word_idx in 0..wpl as usize {
        for bit in 0..32 {
            let x = word_idx * 32 + bit;
            if x >= w as usize {
                break;
            }

            let bit_mask = 1u32 << (31 - bit);

            for y in 0..h as i32 {
                // Check if any pixel in the vertical neighborhood is set
                let mut dilated = false;
                for dy in top..=bottom {
                    let sy = y + dy;
                    if sy >= 0 && sy < h as i32 {
                        let src_word = src_data[(sy as u32 * wpl) as usize + word_idx];
                        if src_word & bit_mask != 0 {
                            dilated = true;
                            break;
                        }
                    }
                }

                if dilated {
                    dst_data[(y as u32 * wpl) as usize + word_idx] |= bit_mask;
                }
            }
        }
    }

    Ok(out_mut.into())
}

/// Horizontal erosion using word-aligned shift operations
///
/// For each pixel, if ALL pixels in the horizontal neighborhood are set,
/// the output pixel is set. Operates on whole 32-bit words, giving
/// ~32x speedup over the per-bit approach.
fn erode_horizontal_dwa(pix: &Pix, hsize: u32) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let wpl = pix.wpl() as usize;

    let origin = (hsize / 2) as i32;
    let left = -origin;
    let right = hsize as i32 - 1 - origin;

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let src_data = pix.data();
    let dst_data = out_mut.data_mut();

    let mut shifted = vec![0u32; wpl];
    let mask = last_word_mask(w);

    for y in 0..h as usize {
        let src_row = &src_data[y * wpl..(y + 1) * wpl];
        let dst_row = &mut dst_data[y * wpl..(y + 1) * wpl];

        // Initialize accumulator to all-1s (AND identity)
        for dst_word in dst_row.iter_mut() {
            *dst_word = !0;
        }

        for d in left..=right {
            shift_row(src_row, wpl, d, &mut shifted);
            for i in 0..wpl {
                dst_row[i] &= shifted[i];
            }
        }

        // Clear padding bits in the last word
        dst_row[wpl - 1] &= mask;
    }

    Ok(out_mut.into())
}

/// Vertical erosion using word-aligned operations
///
/// For each pixel, if ALL pixels in the vertical neighborhood are set,
/// the output pixel is set.
fn erode_vertical_dwa(pix: &Pix, vsize: u32) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let wpl = pix.wpl();

    // Match Sel::create_brick origin convention: origin at vsize/2
    let origin = (vsize / 2) as i32;
    let top = -origin;
    let bottom = vsize as i32 - 1 - origin;

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let src_data = pix.data();
    let dst_data = out_mut.data_mut();

    // Process each column
    for word_idx in 0..wpl as usize {
        for bit in 0..32 {
            let x = word_idx * 32 + bit;
            if x >= w as usize {
                break;
            }

            let bit_mask = 1u32 << (31 - bit);

            for y in 0..h as i32 {
                // Check if all pixels in the vertical neighborhood are set
                let mut eroded = true;
                for dy in top..=bottom {
                    let sy = y + dy;
                    if sy < 0 || sy >= h as i32 {
                        // Outside boundary - treat as background (0) for asymmetric b.c.
                        eroded = false;
                        break;
                    }
                    let src_word = src_data[(sy as u32 * wpl) as usize + word_idx];
                    if src_word & bit_mask == 0 {
                        eroded = false;
                        break;
                    }
                }

                if eroded {
                    dst_data[(y as u32 * wpl) as usize + word_idx] |= bit_mask;
                }
            }
        }
    }

    Ok(out_mut.into())
}

// ---------------------------------------------------------------------------
// Composite and Extended DWA (Phase 5)
// ---------------------------------------------------------------------------

/// Compute the extended composite parameters for DWA operations on sizes > 63.
///
/// Decomposes a linear SE size into `n` passes of size 63 plus one pass of
/// size `extra`.  For size > 63 the formula is:
///   `size = 63 + (n - 1) * 62 + (extra - 1)`
///
/// Returns `(n, extra)` where extra is in 1..=63.
///
/// Reference: Leptonica `getExtendedCompositeParameters()` in morphdwa.c
pub fn get_extended_composite_parameters(size: u32) -> (u32, u32) {
    if size <= 63 {
        return (0, size.max(1));
    }
    let n = 1 + (size - 63) / 62;
    let extra = size - 63 - (n - 1) * 62 + 1;
    (n, extra)
}

/// Composite DWA dilation (≤ 63 per dimension, delegates to extend for larger).
///
/// Decomposes each dimension into two factors via `select_composable_sizes`,
/// then applies brick DWA followed by comb DWA.  The decomposition may
/// approximate the requested size for primes (e.g., 37 → 6×6 = 36).
pub fn dilate_comp_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;
    validate_sizes(hsize, vsize)?;
    if hsize > 63 || vsize > 63 {
        return dilate_comp_brick_extend_dwa(pix, hsize, vsize);
    }
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }
    composite_dwa_op(pix, hsize, vsize, DwaOp::Dilate)
}

/// Composite DWA erosion (≤ 63 per dimension, delegates to extend for larger).
pub fn erode_comp_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;
    validate_sizes(hsize, vsize)?;
    if hsize > 63 || vsize > 63 {
        return erode_comp_brick_extend_dwa(pix, hsize, vsize);
    }
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }
    composite_dwa_op(pix, hsize, vsize, DwaOp::Erode)
}

/// Composite DWA opening (erosion then dilation).
pub fn open_comp_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let eroded = erode_comp_brick_dwa(pix, hsize, vsize)?;
    dilate_comp_brick_dwa(&eroded, hsize, vsize)
}

/// Composite DWA closing (dilation then erosion).
pub fn close_comp_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let dilated = dilate_comp_brick_dwa(pix, hsize, vsize)?;
    erode_comp_brick_dwa(&dilated, hsize, vsize)
}

/// Extended composite DWA dilation (arbitrary size, > 63 supported).
///
/// Chains multiple 63-pixel DWA passes plus one residual pass.
/// Called automatically by `dilate_comp_brick_dwa` when a dimension > 63.
pub fn dilate_comp_brick_extend_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;
    validate_sizes(hsize, vsize)?;
    if hsize < 64 && vsize < 64 {
        return dilate_comp_brick_dwa(pix, hsize, vsize);
    }
    let result = extend_dwa_1d(pix, hsize, true, DwaOp::Dilate)?;
    extend_dwa_1d(&result, vsize, false, DwaOp::Dilate)
}

/// Extended composite DWA erosion (arbitrary size, > 63 supported).
pub fn erode_comp_brick_extend_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;
    validate_sizes(hsize, vsize)?;
    if hsize < 64 && vsize < 64 {
        return erode_comp_brick_dwa(pix, hsize, vsize);
    }
    let result = extend_dwa_1d(pix, hsize, true, DwaOp::Erode)?;
    extend_dwa_1d(&result, vsize, false, DwaOp::Erode)
}

/// Extended composite DWA opening.
pub fn open_comp_brick_extend_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let eroded = erode_comp_brick_extend_dwa(pix, hsize, vsize)?;
    dilate_comp_brick_extend_dwa(&eroded, hsize, vsize)
}

/// Extended composite DWA closing.
pub fn close_comp_brick_extend_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let dilated = dilate_comp_brick_extend_dwa(pix, hsize, vsize)?;
    erode_comp_brick_extend_dwa(&dilated, hsize, vsize)
}

// ---------------------------------------------------------------------------
// Internal helpers for composite / extended DWA
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum DwaOp {
    Dilate,
    Erode,
}

fn validate_sizes(hsize: u32, vsize: u32) -> MorphResult<()> {
    if hsize == 0 || vsize == 0 {
        return Err(MorphError::InvalidSel(
            "hsize and vsize must be > 0".to_string(),
        ));
    }
    Ok(())
}

/// Apply a single 1-D brick DWA (horizontal or vertical).
fn apply_1d_dwa(pix: &Pix, size: u32, horizontal: bool, op: DwaOp) -> MorphResult<Pix> {
    let (h, v) = if horizontal { (size, 1) } else { (1, size) };
    match op {
        DwaOp::Dilate => dilate_brick_dwa(pix, h, v),
        DwaOp::Erode => erode_brick_dwa(pix, h, v),
    }
}

/// Composite DWA: decompose size into brick × comb for each dimension.
fn composite_dwa_op(pix: &Pix, hsize: u32, vsize: u32, op: DwaOp) -> MorphResult<Pix> {
    let mut result = pix.clone();

    if hsize > 1 {
        let (s1, s2) = crate::binary::select_composable_sizes(hsize);
        result = apply_1d_dwa(&result, s1, true, op)?;
        if s2 > 1 {
            result = comb_dwa(&result, s2, s1, true, op)?;
        }
    }
    if vsize > 1 {
        let (s1, s2) = crate::binary::select_composable_sizes(vsize);
        result = apply_1d_dwa(&result, s1, false, op)?;
        if s2 > 1 {
            result = comb_dwa(&result, s2, s1, false, op)?;
        }
    }
    Ok(result)
}

/// Comb DWA: hits at 0, spacing, 2·spacing, …, (n−1)·spacing.
fn comb_dwa(pix: &Pix, n: u32, spacing: u32, horizontal: bool, op: DwaOp) -> MorphResult<Pix> {
    if n <= 1 {
        return Ok(pix.clone());
    }
    let origin = ((n - 1) * spacing) as i32 / 2;
    if horizontal {
        comb_horizontal_dwa(pix, n, spacing, origin, op)
    } else {
        comb_vertical_dwa(pix, n, spacing, origin, op)
    }
}

/// Horizontal comb DWA via word-aligned shift-and-accumulate.
fn comb_horizontal_dwa(
    pix: &Pix,
    n: u32,
    spacing: u32,
    origin: i32,
    op: DwaOp,
) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let wpl = pix.wpl() as usize;
    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();
    let src = pix.data();
    let dst = out_mut.data_mut();
    let mut shifted = vec![0u32; wpl];
    let mask = last_word_mask(w);
    let init = if op == DwaOp::Dilate { 0u32 } else { !0u32 };

    for y in 0..h as usize {
        let src_row = &src[y * wpl..(y + 1) * wpl];
        let dst_row = &mut dst[y * wpl..(y + 1) * wpl];
        dst_row.fill(init);
        for k in 0..n {
            let d = (k * spacing) as i32 - origin;
            shift_row(src_row, wpl, d, &mut shifted);
            for i in 0..wpl {
                if op == DwaOp::Dilate {
                    dst_row[i] |= shifted[i];
                } else {
                    dst_row[i] &= shifted[i];
                }
            }
        }
        dst_row[wpl - 1] &= mask;
    }
    Ok(out_mut.into())
}

/// Vertical comb DWA: per-pixel accumulate at comb positions.
fn comb_vertical_dwa(pix: &Pix, n: u32, spacing: u32, origin: i32, op: DwaOp) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let wpl = pix.wpl();
    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();
    let src = pix.data();
    let dst = out_mut.data_mut();

    for wi in 0..wpl as usize {
        for bit in 0..32u32 {
            let x = wi * 32 + bit as usize;
            if x >= w as usize {
                break;
            }
            let bm = 1u32 << (31 - bit);
            for y in 0..h as i32 {
                let mut acc = op == DwaOp::Erode; // dilate→false, erode→true
                for k in 0..n {
                    let sy = y + (k * spacing) as i32 - origin;
                    if sy >= 0 && sy < h as i32 {
                        let hit = src[(sy as u32 * wpl) as usize + wi] & bm != 0;
                        if op == DwaOp::Dilate {
                            if hit {
                                acc = true;
                                break;
                            }
                        } else if !hit {
                            acc = false;
                            break;
                        }
                    } else if op == DwaOp::Erode {
                        acc = false;
                        break;
                    }
                }
                if acc {
                    dst[(y as u32 * wpl) as usize + wi] |= bm;
                }
            }
        }
    }
    Ok(out_mut.into())
}

/// Extended DWA: chain multiple 63-pixel passes for one dimension.
///
/// Follows the C algorithm in `pixDilateCompBrickExtendDwa`.
fn extend_dwa_1d(pix: &Pix, size: u32, horizontal: bool, op: DwaOp) -> MorphResult<Pix> {
    if size == 1 {
        return Ok(pix.clone());
    }
    if size < 64 {
        return apply_1d_dwa(pix, size, horizontal, op);
    }
    if size == 64 {
        // Approximate: use 63 (same as C)
        return apply_1d_dwa(pix, 63, horizontal, op);
    }

    let (n, extra) = get_extended_composite_parameters(size);
    let nops = if extra < 3 { n } else { n + 1 };

    let mut result;
    let mut temp;

    if nops & 1 == 1 {
        // Odd number of ops
        result = if extra > 2 {
            apply_1d_dwa(pix, extra, horizontal, op)?
        } else {
            apply_1d_dwa(pix, 63, horizontal, op)?
        };
        for _ in 0..nops / 2 {
            temp = apply_1d_dwa(&result, 63, horizontal, op)?;
            result = apply_1d_dwa(&temp, 63, horizontal, op)?;
        }
    } else {
        // Even number of ops
        temp = if extra > 2 {
            apply_1d_dwa(pix, extra, horizontal, op)?
        } else {
            apply_1d_dwa(pix, 63, horizontal, op)?
        };
        result = apply_1d_dwa(&temp, 63, horizontal, op)?;
        for _ in 0..nops / 2 - 1 {
            temp = apply_1d_dwa(&result, 63, horizontal, op)?;
            result = apply_1d_dwa(&temp, 63, horizontal, op)?;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> Pix {
        // Create a 10x10 image with a 4x4 square in the center
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set the 4x4 square (pixels 3-6 in each dimension)
        for y in 3..7 {
            for x in 3..7 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }

        pix_mut.into()
    }

    fn count_foreground_pixels(pix: &Pix) -> u32 {
        let mut count = 0;
        for y in 0..pix.height() {
            for x in 0..pix.width() {
                if pix.get_pixel_unchecked(x, y) != 0 {
                    count += 1;
                }
            }
        }
        count
    }

    #[test]
    fn test_dilate_identity() {
        let pix = create_test_image();
        let original_count = count_foreground_pixels(&pix);

        // 1x1 dilation should be identity
        let dilated = dilate_brick_dwa(&pix, 1, 1).unwrap();
        let dilated_count = count_foreground_pixels(&dilated);

        assert_eq!(original_count, dilated_count);
    }

    #[test]
    fn test_erode_identity() {
        let pix = create_test_image();
        let original_count = count_foreground_pixels(&pix);

        // 1x1 erosion should be identity
        let eroded = erode_brick_dwa(&pix, 1, 1).unwrap();
        let eroded_count = count_foreground_pixels(&eroded);

        assert_eq!(original_count, eroded_count);
    }

    #[test]
    fn test_dilate_increases_foreground() {
        let pix = create_test_image();
        let original_count = count_foreground_pixels(&pix);

        let dilated = dilate_brick_dwa(&pix, 3, 3).unwrap();
        let dilated_count = count_foreground_pixels(&dilated);

        // Dilation should increase (or maintain) foreground
        assert!(dilated_count >= original_count);
    }

    #[test]
    fn test_erode_decreases_foreground() {
        let pix = create_test_image();
        let original_count = count_foreground_pixels(&pix);

        let eroded = erode_brick_dwa(&pix, 3, 3).unwrap();
        let eroded_count = count_foreground_pixels(&eroded);

        // Erosion should decrease (or maintain) foreground
        assert!(eroded_count <= original_count);
    }

    #[test]
    fn test_open_removes_small_objects() {
        // Create image with a small 1-pixel object
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Large object
        for y in 2..8 {
            for x in 2..8 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        // Small isolated pixel
        pix_mut.set_pixel_unchecked(0, 0, 1);

        let pix: Pix = pix_mut.into();
        let original_count = count_foreground_pixels(&pix);

        // Opening with 3x3 should remove the isolated pixel
        let opened = open_brick_dwa(&pix, 3, 3).unwrap();
        let opened_count = count_foreground_pixels(&opened);

        // The isolated pixel should be removed
        assert!(opened_count < original_count);
        assert_eq!(opened.get_pixel_unchecked(0, 0), 0);
    }

    #[test]
    fn test_close_fills_holes() {
        // Create image with a hole inside
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a ring (square with hole)
        for y in 2..8 {
            for x in 2..8 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        // Create hole
        for y in 4..6 {
            for x in 4..6 {
                pix_mut.set_pixel_unchecked(x, y, 0);
            }
        }

        let pix: Pix = pix_mut.into();
        let original_count = count_foreground_pixels(&pix);

        // Closing should fill the small hole
        let closed = close_brick_dwa(&pix, 3, 3).unwrap();
        let closed_count = count_foreground_pixels(&closed);

        // After closing, we should have more foreground
        assert!(closed_count >= original_count);
    }

    #[test]
    fn test_horizontal_only_dilate() {
        let pix = create_test_image();

        // Horizontal-only dilation (5x1)
        let dilated = dilate_brick_dwa(&pix, 5, 1).unwrap();

        // Check that the center row expanded horizontally but not vertically
        // Original: pixels 3-6 in x
        // After horizontal dilation by 5: pixels 1-8 should be set
        for x in 1..9 {
            // Center rows should be dilated
            assert_eq!(
                dilated.get_pixel_unchecked(x, 5),
                1,
                "Expected pixel at ({}, 5) to be set",
                x
            );
        }
    }

    #[test]
    fn test_vertical_only_dilate() {
        let pix = create_test_image();

        // Vertical-only dilation (1x5)
        let dilated = dilate_brick_dwa(&pix, 1, 5).unwrap();

        // Check that the center column expanded vertically
        for y in 1..9 {
            assert_eq!(
                dilated.get_pixel_unchecked(5, y),
                1,
                "Expected pixel at (5, {}) to be set",
                y
            );
        }
    }

    #[test]
    fn test_invalid_size() {
        let pix = create_test_image();

        // Size 0 should fail
        assert!(dilate_brick_dwa(&pix, 0, 1).is_err());
        assert!(dilate_brick_dwa(&pix, 1, 0).is_err());
        assert!(erode_brick_dwa(&pix, 0, 1).is_err());
        assert!(erode_brick_dwa(&pix, 1, 0).is_err());
    }

    #[test]
    fn test_non_binary_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();

        assert!(dilate_brick_dwa(&pix, 3, 3).is_err());
        assert!(erode_brick_dwa(&pix, 3, 3).is_err());
    }

    #[test]
    fn test_comparison_with_regular_morph() {
        use crate::binary::{dilate_brick, erode_brick};

        let pix = create_test_image();

        // Compare DWA with regular morphology
        let dwa_dilated = dilate_brick_dwa(&pix, 3, 3).unwrap();
        let regular_dilated = dilate_brick(&pix, 3, 3).unwrap();

        // Results should be identical
        for y in 0..pix.height() {
            for x in 0..pix.width() {
                let dwa_val = dwa_dilated.get_pixel_unchecked(x, y);
                let regular_val = regular_dilated.get_pixel_unchecked(x, y);
                assert_eq!(dwa_val, regular_val, "Dilation mismatch at ({}, {})", x, y);
            }
        }

        let dwa_eroded = erode_brick_dwa(&pix, 3, 3).unwrap();
        let regular_eroded = erode_brick(&pix, 3, 3).unwrap();

        // Results should be identical
        for y in 0..pix.height() {
            for x in 0..pix.width() {
                let dwa_val = dwa_eroded.get_pixel_unchecked(x, y);
                let regular_val = regular_eroded.get_pixel_unchecked(x, y);
                assert_eq!(dwa_val, regular_val, "Erosion mismatch at ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn test_large_structuring_element() {
        let pix = create_test_image();

        // Test with a larger SE (7x7)
        let dilated = dilate_brick_dwa(&pix, 7, 7).unwrap();
        let eroded = erode_brick_dwa(&pix, 7, 7).unwrap();

        // Dilated should be larger
        assert!(count_foreground_pixels(&dilated) > count_foreground_pixels(&pix));

        // 4x4 object eroded by 7x7 should be very small or empty
        // (since the object is smaller than the SE)
        assert!(count_foreground_pixels(&eroded) < count_foreground_pixels(&pix));
    }

    // -----------------------------------------------------------------------
    // Phase 5: Composite DWA tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_extended_composite_parameters() {
        // For size <= 63, n=0 and extra=size
        assert_eq!(get_extended_composite_parameters(1), (0, 1));
        assert_eq!(get_extended_composite_parameters(63), (0, 63));
        // For size=64: approximate (n=1, extra=2 → use 63)
        assert_eq!(get_extended_composite_parameters(64), (1, 2));
        // For size=65: n=1, extra=3
        assert_eq!(get_extended_composite_parameters(65), (1, 3));
        // For size=125: n=2, extra=1 → just n passes of 63
        assert_eq!(get_extended_composite_parameters(125), (2, 1));
        // For size=200: n=3, extra=14
        assert_eq!(get_extended_composite_parameters(200), (3, 14));
    }

    #[test]
    fn test_comp_dilate_identity() {
        let pix = create_test_image();
        let dilated = dilate_comp_brick_dwa(&pix, 1, 1).unwrap();
        assert_eq!(
            count_foreground_pixels(&dilated),
            count_foreground_pixels(&pix)
        );
    }

    #[test]
    fn test_comp_erode_identity() {
        let pix = create_test_image();
        let eroded = erode_comp_brick_dwa(&pix, 1, 1).unwrap();
        assert_eq!(
            count_foreground_pixels(&eroded),
            count_foreground_pixels(&pix)
        );
    }

    #[test]
    fn test_comp_dilate_increases_foreground() {
        let pix = create_test_image();
        let original = count_foreground_pixels(&pix);
        let dilated = dilate_comp_brick_dwa(&pix, 3, 3).unwrap();
        assert!(count_foreground_pixels(&dilated) >= original);
    }

    #[test]
    fn test_comp_erode_decreases_foreground() {
        let pix = create_test_image();
        let original = count_foreground_pixels(&pix);
        let eroded = erode_comp_brick_dwa(&pix, 3, 3).unwrap();
        assert!(count_foreground_pixels(&eroded) <= original);
    }

    #[test]
    fn test_comp_open_removes_small_objects() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 2..8 {
            for x in 2..8 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        pm.set_pixel_unchecked(0, 0, 1);
        let pix: Pix = pm.into();
        let opened = open_comp_brick_dwa(&pix, 3, 3).unwrap();
        assert_eq!(opened.get_pixel_unchecked(0, 0), 0);
    }

    #[test]
    fn test_comp_close_fills_holes() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 2..8 {
            for x in 2..8 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        for y in 4..6 {
            for x in 4..6 {
                pm.set_pixel_unchecked(x, y, 0);
            }
        }
        let pix: Pix = pm.into();
        let original = count_foreground_pixels(&pix);
        let closed = close_comp_brick_dwa(&pix, 3, 3).unwrap();
        assert!(count_foreground_pixels(&closed) >= original);
    }

    #[test]
    fn test_extend_dilate_large_se() {
        // Use a larger image to test with SE > 63
        let pix = Pix::new(200, 200, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 90..110 {
            for x in 90..110 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix: Pix = pm.into();
        let original = count_foreground_pixels(&pix);
        let dilated = dilate_comp_brick_extend_dwa(&pix, 70, 70).unwrap();
        assert!(count_foreground_pixels(&dilated) > original);
    }

    #[test]
    fn test_extend_erode_large_se() {
        let pix = Pix::new(200, 200, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 10..190 {
            for x in 10..190 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix: Pix = pm.into();
        let original = count_foreground_pixels(&pix);
        let eroded = erode_comp_brick_extend_dwa(&pix, 70, 70).unwrap();
        assert!(count_foreground_pixels(&eroded) < original);
    }

    #[test]
    fn test_comp_dilate_delegates_to_extend_for_large_se() {
        let pix = Pix::new(200, 200, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 90..110 {
            for x in 90..110 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix: Pix = pm.into();
        // comp_brick_dwa with large SE should succeed (delegates to extend)
        let result = dilate_comp_brick_dwa(&pix, 100, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extend_open_large_se() {
        let pix = Pix::new(200, 200, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 10..190 {
            for x in 10..190 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        pm.set_pixel_unchecked(0, 0, 1);
        let pix: Pix = pm.into();
        let opened = open_comp_brick_extend_dwa(&pix, 70, 70).unwrap();
        assert_eq!(opened.get_pixel_unchecked(0, 0), 0);
    }

    #[test]
    fn test_extend_close_large_se() {
        let pix = Pix::new(200, 200, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 10..190 {
            for x in 10..190 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        // Create a small hole that the SE can fill
        for y in 95..105 {
            for x in 95..105 {
                pm.set_pixel_unchecked(x, y, 0);
            }
        }
        let pix: Pix = pm.into();
        // Close should fill the hole (SE 70x70 is much larger than 10x10 hole)
        let closed = close_comp_brick_extend_dwa(&pix, 70, 70).unwrap();
        // Verify the hole center pixel is now filled
        assert_eq!(closed.get_pixel_unchecked(100, 100), 1);
    }
}
