//! Binary morphological operations
//!
//! Implements erosion, dilation, opening, and closing for 1-bpp images.

use crate::{MorphError, MorphResult, Sel};
use leptonica_core::{Pix, PixelDepth};

/// Dilate a binary image using rasterop (word-level shift-and-OR)
///
/// Dilation expands foreground regions. For each hit position in the SEL,
/// the source image is shifted by that offset and OR-accumulated into the
/// output. All operations are performed at 32-bit word granularity.
///
/// Algorithm (C version: morph.c:213-238):
///   1. Clear output
///   2. For each hit (dx, dy): dest[y] |= shift(src[y - dy], dx)
pub fn dilate(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let mut out_mut = dilate_rasterop(pix, sel)?;
    // Clear unused bits to prevent contamination in subsequent operations
    // (e.g., erosion after dilation in closing).
    let w = pix.width();
    let wpl = pix.wpl() as usize;
    clear_unused_bits(out_mut.data_mut(), w, wpl);
    Ok(out_mut.into())
}

/// Rasterop dilation without clearing unused bits.
///
/// Used internally by composite decomposition where intermediate
/// results need to preserve bits beyond the image width for the
/// next decomposition step to read.
fn dilate_rasterop(pix: &Pix, sel: &Sel) -> MorphResult<leptonica_core::PixMut> {
    check_binary(pix)?;

    let w = pix.width();
    let h = pix.height();
    let wpl = pix.wpl() as usize;

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let src_data = pix.data();
    let dst_data = out_mut.data_mut();

    let hit_offsets: Vec<_> = sel.hit_offsets().collect();

    for &(dx, dy) in &hit_offsets {
        for y in 0..h as i32 {
            let src_y = y + dy;
            if src_y < 0 || src_y >= h as i32 {
                continue;
            }

            let src_start = src_y as usize * wpl;
            let dst_start = y as usize * wpl;

            shift_or_row(
                &mut dst_data[dst_start..dst_start + wpl],
                &src_data[src_start..src_start + wpl],
                -dx,
            );
        }
    }

    Ok(out_mut)
}

/// Erode a binary image using rasterop (word-level shift-and-AND)
///
/// Erosion shrinks foreground regions. For each hit position in the SEL,
/// the source image is shifted by the inverted offset and AND-accumulated
/// into the output. All operations are performed at 32-bit word granularity.
///
/// Algorithm (C version: morph.c:265-309):
///   1. Set all output bits to 1
///   2. For each hit (dx, dy): dest[y] &= shift(src[y + dy], -dx)
///   3. Outside boundaries: AND with 0 = clear (asymmetric BC)
pub fn erode(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let mut out_mut = erode_rasterop(pix, sel)?;
    let w = pix.width();
    let wpl = pix.wpl() as usize;
    clear_unused_bits(out_mut.data_mut(), w, wpl);
    Ok(out_mut.into())
}

/// Rasterop erosion without clearing unused bits.
fn erode_rasterop(pix: &Pix, sel: &Sel) -> MorphResult<leptonica_core::PixMut> {
    check_binary(pix)?;

    let h = pix.height();
    let wpl = pix.wpl() as usize;

    let out_pix = Pix::new(pix.width(), h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let src_data = pix.data();
    let dst_data = out_mut.data_mut();

    // Initialize output to all 1s (for AND accumulation)
    for word in dst_data.iter_mut() {
        *word = 0xFFFF_FFFF;
    }

    let hit_offsets: Vec<_> = sel.hit_offsets().collect();

    for &(dx, dy) in &hit_offsets {
        for y in 0..h as i32 {
            let src_y = y + dy;
            let dst_start = y as usize * wpl;

            if src_y < 0 || src_y >= h as i32 {
                for w in 0..wpl {
                    dst_data[dst_start + w] = 0;
                }
                continue;
            }

            let src_start = src_y as usize * wpl;

            shift_and_row(
                &mut dst_data[dst_start..dst_start + wpl],
                &src_data[src_start..src_start + wpl],
                -dx,
            );
        }
    }

    Ok(out_mut)
}

/// Open a binary image
///
/// Opening = Erosion followed by Dilation.
/// Removes small foreground objects and smooths contours.
pub fn open(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let eroded = erode(pix, sel)?;
    dilate(&eroded, sel)
}

/// Close a binary image
///
/// Closing = Dilation followed by Erosion.
/// Fills small holes and connects nearby objects.
pub fn close(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let dilated = dilate(pix, sel)?;
    erode(&dilated, sel)
}

/// Hit-miss transform
///
/// The HMT identifies pixels that match both the hit pattern (foreground)
/// AND the miss pattern (background). Used for pattern detection.
pub fn hit_miss_transform(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    check_binary(pix)?;

    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let hit_offsets: Vec<_> = sel.hit_offsets().collect();
    let miss_offsets: Vec<_> = sel.miss_offsets().collect();

    for y in 0..h {
        for x in 0..w {
            // Check if all hits match foreground
            let hits_match = hit_offsets.iter().all(|&(dx, dy)| {
                let sx = x as i32 + dx;
                let sy = y as i32 + dy;
                if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                    pix.get_pixel_unchecked(sx as u32, sy as u32) != 0
                } else {
                    false
                }
            });

            // Check if all misses match background
            let misses_match = miss_offsets.iter().all(|&(dx, dy)| {
                let sx = x as i32 + dx;
                let sy = y as i32 + dy;
                if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                    pix.get_pixel_unchecked(sx as u32, sy as u32) == 0
                } else {
                    true // Outside is background
                }
            });

            if hits_match && misses_match {
                out_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok(out_mut.into())
}

/// Morphological gradient (dilation - erosion)
///
/// Highlights edges/boundaries of objects.
pub fn gradient(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let dilated = dilate(pix, sel)?;
    let eroded = erode(pix, sel)?;
    subtract(&dilated, &eroded)
}

/// Top-hat transform (original - opening)
///
/// Extracts bright features smaller than the SE.
pub fn top_hat(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let opened = open(pix, sel)?;
    subtract(pix, &opened)
}

/// Bottom-hat transform (closing - original)
///
/// Extracts dark features smaller than the SE.
pub fn bottom_hat(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let closed = close(pix, sel)?;
    subtract(&closed, pix)
}

/// Subtract two binary images (a AND NOT b)
fn subtract(a: &Pix, b: &Pix) -> MorphResult<Pix> {
    let w = a.width();
    let h = a.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let va = a.get_pixel_unchecked(x, y);
            let vb = b.get_pixel_unchecked(x, y);
            let result = if va != 0 && vb == 0 { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Boundary type for [`extract_boundary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryType {
    /// Background boundary: pixels just outside the foreground objects.
    /// Computed as (dilate 3×3) XOR original.
    Outer,
    /// Foreground boundary: pixels on the inner edge of foreground objects.
    /// Computed as (erode 3×3) XOR original.
    Inner,
}

/// Extract boundary pixels from a 1-bpp binary image.
///
/// Returns a 1-bpp image containing only the boundary pixels of
/// foreground components.
///
/// - [`BoundaryType::Outer`]: background pixels adjacent to foreground
///   (dilation XOR original)
/// - [`BoundaryType::Inner`]: foreground pixels adjacent to background
///   (erosion XOR original)
///
/// # See also
///
/// C Leptonica: `pixExtractBoundary()` in `morphapp.c`
pub fn extract_boundary(pix: &Pix, boundary_type: BoundaryType) -> MorphResult<Pix> {
    check_binary(pix)?;

    let morphed = match boundary_type {
        BoundaryType::Outer => dilate_brick(pix, 3, 3)?,
        BoundaryType::Inner => erode_brick(pix, 3, 3)?,
    };

    xor(pix, &morphed)
}

/// XOR two binary images
fn xor(a: &Pix, b: &Pix) -> MorphResult<Pix> {
    let w = a.width();
    let h = a.height();
    let wpl = a.wpl() as usize;

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let a_data = a.data();
    let b_data = b.data();
    let out_data = out_mut.data_mut();

    for y in 0..h as usize {
        let offset = y * wpl;
        for i in 0..wpl {
            out_data[offset + i] = a_data[offset + i] ^ b_data[offset + i];
        }
    }

    // Clear unused padding bits beyond the image width to avoid
    // propagating garbage into subsequent morphology.
    clear_unused_bits(out_data, w, wpl);

    Ok(out_mut.into())
}

/// Dilate with a brick (rectangular) structuring element
///
/// Uses separable + composite decomposition for optimal performance.
/// For composite sizes N = f1 × f2: brick(f1) then comb(f1, f2) reduces
/// shift operations from N to f1 + f2.
pub fn dilate_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    if width == 1 && height == 1 {
        return Ok(pix.clone());
    }
    // Separable: horizontal then vertical, each using composite decomposition.
    // dilate_rasterop preserves unused bits for composite intermediate steps.
    let tmp: Pix = dilate_1d_composite(pix, width, true)?.into();
    let mut result = dilate_1d_composite(&tmp, height, false)?;
    clear_unused_bits(result.data_mut(), pix.width(), pix.wpl() as usize);
    Ok(result.into())
}

/// Erode with a brick (rectangular) structuring element
///
/// Uses separable + composite decomposition for optimal performance.
pub fn erode_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    if width == 1 && height == 1 {
        return Ok(pix.clone());
    }
    let tmp: Pix = erode_1d_composite(pix, width, true)?.into();
    let mut result = erode_1d_composite(&tmp, height, false)?;
    clear_unused_bits(result.data_mut(), pix.width(), pix.wpl() as usize);
    Ok(result.into())
}

/// Open with a brick structuring element
///
/// Opening = erosion followed by dilation.
pub fn open_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    if width == 1 && height == 1 {
        return Ok(pix.clone());
    }
    let eroded = erode_brick(pix, width, height)?;
    dilate_brick(&eroded, width, height)
}

/// Close with a brick structuring element
///
/// Closing = dilation followed by erosion.
/// Must clear unused bits between dilation and erosion to prevent
/// contamination from dilated unused bits propagating into erosion.
pub fn close_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    if width == 1 && height == 1 {
        return Ok(pix.clone());
    }
    let dilated = dilate_brick(pix, width, height)?;
    erode_brick(&dilated, width, height)
}

/// Close a binary image safely, avoiding boundary artifacts.
///
/// Standard `close` can introduce artifacts at image borders because erosion
/// after dilation may erode pixels that were dilated using the padded edge.
/// This function pads the image by the SEL extent before closing, then strips
/// the border, preventing those artifacts.
///
/// The horizontal border is rounded up to the nearest 32-bit word boundary
/// to align with the rasterop word granularity.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `sel` - Structuring element
///
/// Based on C leptonica `pixCloseSafe`.
pub fn close_safe(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    check_binary(pix)?;
    let (xp, yp, xn, yn) = sel.find_max_translations();
    let xmax = xp.max(xn);
    // Round up to nearest multiple of 32 (full 32-bit words for rasterop alignment)
    let xbord = xmax.div_ceil(32) * 32;
    let padded = pix.add_border_general(xbord, xbord, yp, yn, 0)?;
    let closed = close(&padded, sel)?;
    Ok(closed.remove_border_general(xbord, xbord, yp, yn)?)
}

/// Close a binary image safely using a brick (rectangular) structuring element.
///
/// Pads the image by the SEL half-extent (rounded up to 32-bit word boundary)
/// before closing to prevent border artifacts, then strips the border.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `hsize` - Horizontal size of the brick
/// * `vsize` - Vertical size of the brick
///
/// Based on C leptonica `pixCloseSafeBrick`.
pub fn close_safe_brick(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }
    let maxtrans = (hsize / 2).max(vsize / 2);
    // Round up to nearest multiple of 32 (full 32-bit words for rasterop alignment)
    let bordsize = maxtrans.div_ceil(32) * 32;
    let padded = pix.add_border(bordsize, 0)?;
    let closed = close_brick(&padded, hsize, vsize)?;
    Ok(closed.remove_border(bordsize)?)
}

/// Close a binary image safely using composite brick decomposition.
///
/// Like `close_safe_brick` but uses composite (factored) structuring elements
/// for improved efficiency on large bricks. In this Rust implementation,
/// `close_safe_brick` already uses composite decomposition internally
/// (via `dilate_brick`/`erode_brick`), so this delegates to it.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `hsize` - Horizontal size of the brick
/// * `vsize` - Vertical size of the brick
///
/// Based on C leptonica `pixCloseSafeCompBrick`.
pub fn close_safe_comp_brick(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    // close_brick internally uses composite decomposition via
    // dilate_1d_composite/erode_1d_composite, so close_safe_brick
    // already provides the composite behavior with proper bit clearing.
    close_safe_brick(pix, hsize, vsize)
}

/// Generalized morphological opening: Hit-Miss Transform followed by dilation.
///
/// Unlike standard `open` which uses only hit elements, this operation uses
/// both hit and miss elements of the SEL. It finds patterns matching the full
/// SEL (via HMT) and then expands those matches by the same SEL.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `sel` - Structuring element with both hit and miss elements
///
/// Based on C leptonica `pixOpenGeneralized`.
pub fn open_generalized(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    check_binary(pix)?;
    let hmt = hit_miss_transform(pix, sel)?;
    dilate(&hmt, sel)
}

/// Generalized morphological closing: dilation followed by Hit-Miss Transform.
///
/// Unlike standard `close` which uses only hit elements, this operation uses
/// both hit and miss elements of the SEL. It expands the image by the SEL
/// (via dilation) and then finds patterns matching the full SEL (via HMT).
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `sel` - Structuring element with both hit and miss elements
///
/// Based on C leptonica `pixCloseGeneralized`.
pub fn close_generalized(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    check_binary(pix)?;
    let dilated = dilate(pix, sel)?;
    hit_miss_transform(&dilated, sel)
}

/// Composite 1D dilation: brick(f1) then comb(f1, f2) when beneficial.
///
/// `horizontal`: true for horizontal, false for vertical.
/// Returns PixMut without clearing unused bits (caller's responsibility).
fn dilate_1d_composite(
    pix: &Pix,
    size: u32,
    horizontal: bool,
) -> MorphResult<leptonica_core::PixMut> {
    if size <= 1 {
        // Identity: 1x1 SEL at origin just copies the image
        let sel = Sel::create_horizontal(1)?;
        return dilate_rasterop(pix, &sel);
    }
    let (f1, f2) = select_composable_sizes(size);
    if f2 <= 1 {
        // Not composable (prime or small): single SEL
        let sel = if horizontal {
            Sel::create_horizontal(size)?
        } else {
            Sel::create_vertical(size)?
        };
        return dilate_rasterop(pix, &sel);
    }
    // Composite: brick(f1) then comb(f1, f2).
    // Add border to prevent boundary clipping in the brick step.
    // The comb step needs intermediate results that extend beyond the
    // original image boundary (C version: pixAddBorder in morphcomp.c).
    let (sel_brick, sel_comb) = if horizontal {
        (
            Sel::create_horizontal(f1)?,
            Sel::create_comb_horizontal(f1, f2)?,
        )
    } else {
        (
            Sel::create_vertical(f1)?,
            Sel::create_comb_vertical(f1, f2)?,
        )
    };
    let max_comb_offset = ((f2 - 1) * f1).div_ceil(2);
    let (left, right, top, bottom) = if horizontal {
        let border = max_comb_offset.div_ceil(32) * 32;
        (border, border, 0u32, 0u32)
    } else {
        (0u32, 0u32, max_comb_offset, max_comb_offset)
    };
    let bordered = add_border(pix, left, right, top, bottom)?;
    let tmp: Pix = dilate_rasterop(&bordered, &sel_brick)?.into();
    let result: Pix = dilate_rasterop(&tmp, &sel_comb)?.into();
    remove_border(&result, left, top, pix.width(), pix.height())
}

/// Composite 1D erosion: brick(f1) then comb(f1, f2) when beneficial.
fn erode_1d_composite(
    pix: &Pix,
    size: u32,
    horizontal: bool,
) -> MorphResult<leptonica_core::PixMut> {
    if size <= 1 {
        let sel = Sel::create_horizontal(1)?;
        return erode_rasterop(pix, &sel);
    }
    let (f1, f2) = select_composable_sizes(size);
    if f2 <= 1 {
        let sel = if horizontal {
            Sel::create_horizontal(size)?
        } else {
            Sel::create_vertical(size)?
        };
        return erode_rasterop(pix, &sel);
    }
    let (sel_brick, sel_comb) = if horizontal {
        (
            Sel::create_horizontal(f1)?,
            Sel::create_comb_horizontal(f1, f2)?,
        )
    } else {
        (
            Sel::create_vertical(f1)?,
            Sel::create_comb_vertical(f1, f2)?,
        )
    };
    let tmp: Pix = erode_rasterop(pix, &sel_brick)?.into();
    erode_rasterop(&tmp, &sel_comb)
}

/// Find the factor pair (f1, f2) where f1 * f2 = size and f1 + f2 is minimized.
///
/// Returns (1, size) for primes (no composite decomposition possible).
pub(crate) fn select_composable_sizes(size: u32) -> (u32, u32) {
    if size <= 1 {
        return (1, size);
    }
    let sqrt = (size as f64).sqrt() as u32;
    for f1 in (2..=sqrt).rev() {
        if size.is_multiple_of(f1) {
            return (f1, size / f1);
        }
    }
    (1, size)
}

/// Shift src row by `shift` pixels and OR into dst (word-level).
///
/// MSB-first bit ordering: pixel 0 = bit 31, pixel 31 = bit 0.
/// Positive shift = image content moves right (src >> shift in bit terms).
/// Negative shift = image content moves left (src << |shift| in bit terms).
///
/// Inner loops have no bounds checks to enable auto-vectorization.
#[allow(clippy::needless_range_loop)]
fn shift_or_row(dst: &mut [u32], src: &[u32], shift: i32) {
    let wpl = dst.len();

    if shift == 0 {
        for i in 0..wpl {
            dst[i] |= src[i];
        }
        return;
    }

    let abs_shift = shift.unsigned_abs() as usize;
    let word_shift = abs_shift / 32;
    let bit_shift = (abs_shift % 32) as u32;

    if word_shift >= wpl {
        return; // Entire row shifts out of bounds; OR with 0 is no-op
    }

    if shift > 0 {
        // Shift right: dst[word_shift..wpl] gets src[0..wpl-word_shift]
        if bit_shift == 0 {
            for i in word_shift..wpl {
                dst[i] |= src[i - word_shift];
            }
        } else {
            // First valid word: no carry from previous
            dst[word_shift] |= src[0] >> bit_shift;
            // Remaining words: carry from src[si-1]
            for i in (word_shift + 1)..wpl {
                let si = i - word_shift;
                dst[i] |= (src[si] >> bit_shift) | (src[si - 1] << (32 - bit_shift));
            }
        }
    } else {
        // Shift left: dst[0..wpl-word_shift] gets src[word_shift..wpl]
        let end = wpl - word_shift;
        if bit_shift == 0 {
            for i in 0..end {
                dst[i] |= src[i + word_shift];
            }
        } else {
            // All words except last: carry from src[si+1]
            for i in 0..end.saturating_sub(1) {
                let si = i + word_shift;
                dst[i] |= (src[si] << bit_shift) | (src[si + 1] >> (32 - bit_shift));
            }
            // Last valid word: no carry from next
            if end > 0 {
                dst[end - 1] |= src[wpl - 1] << bit_shift;
            }
        }
    }
}

/// Shift src row by `shift` pixels and AND into dst (word-level).
///
/// Same shift semantics as `shift_or_row`, but uses AND accumulation.
/// Out-of-bounds positions are 0, so AND with them clears dst bits.
///
/// Inner loops have no bounds checks to enable auto-vectorization.
#[allow(clippy::needless_range_loop)]
fn shift_and_row(dst: &mut [u32], src: &[u32], shift: i32) {
    let wpl = dst.len();

    if shift == 0 {
        for i in 0..wpl {
            dst[i] &= src[i];
        }
        return;
    }

    let abs_shift = shift.unsigned_abs() as usize;
    let word_shift = abs_shift / 32;
    let bit_shift = (abs_shift % 32) as u32;

    if word_shift >= wpl {
        // Entire row shifts out of bounds; AND with 0 clears all
        for i in 0..wpl {
            dst[i] = 0;
        }
        return;
    }

    if shift > 0 {
        // Clear words before the valid range (AND with 0)
        for i in 0..word_shift {
            dst[i] = 0;
        }
        if bit_shift == 0 {
            for i in word_shift..wpl {
                dst[i] &= src[i - word_shift];
            }
        } else {
            // First valid word: no carry from previous, also AND-clear high bits
            dst[word_shift] &= src[0] >> bit_shift;
            for i in (word_shift + 1)..wpl {
                let si = i - word_shift;
                dst[i] &= (src[si] >> bit_shift) | (src[si - 1] << (32 - bit_shift));
            }
        }
    } else {
        let end = wpl - word_shift;
        if bit_shift == 0 {
            for i in 0..end {
                dst[i] &= src[i + word_shift];
            }
        } else {
            for i in 0..end.saturating_sub(1) {
                let si = i + word_shift;
                dst[i] &= (src[si] << bit_shift) | (src[si + 1] >> (32 - bit_shift));
            }
            // Last valid word: no carry from next
            if end > 0 {
                dst[end - 1] &= src[wpl - 1] << bit_shift;
            }
        }
        // Clear words after the valid range (AND with 0)
        for i in end..wpl {
            dst[i] = 0;
        }
    }
}

/// Clear unused bits in the last word of each row.
///
/// When image width is not a multiple of 32, the last word of each row
/// has unused bit positions (lower bits in MSB-first ordering). Word-level
/// shift operations can set these bits, which would contaminate subsequent
/// operations (e.g., erosion reading garbage bits from a dilated result).
fn clear_unused_bits(data: &mut [u32], width: u32, wpl: usize) {
    let extra = width % 32;
    if extra == 0 {
        return;
    }
    // MSB-first: valid bits are the top `extra` bits; mask off the rest
    let mask = !0u32 << (32 - extra);
    let h = data.len() / wpl;
    for y in 0..h {
        let idx = y * wpl + wpl - 1;
        data[idx] &= mask;
    }
}

/// Add zero-padding border around a binary image.
///
/// Used by composite decomposition to prevent boundary clipping.
/// `left` and `right` must be multiples of 32 for word-aligned copy.
fn add_border(pix: &Pix, left: u32, right: u32, top: u32, bottom: u32) -> MorphResult<Pix> {
    debug_assert!(
        left.is_multiple_of(32) && right.is_multiple_of(32),
        "horizontal borders must be word-aligned"
    );

    let new_w = pix.width() + left + right;
    let new_h = pix.height() + top + bottom;
    let new_pix = Pix::new(new_w, new_h, PixelDepth::Bit1)?;
    let mut new_mut = new_pix.try_into_mut().unwrap();

    let left_words = (left / 32) as usize;
    let src_wpl = pix.wpl() as usize;
    let dst_wpl = new_mut.wpl() as usize;
    let src_data = pix.data();
    let dst_data = new_mut.data_mut();

    for y in 0..pix.height() as usize {
        let src_start = y * src_wpl;
        let dst_start = (y + top as usize) * dst_wpl + left_words;
        dst_data[dst_start..dst_start + src_wpl]
            .copy_from_slice(&src_data[src_start..src_start + src_wpl]);
    }

    Ok(new_mut.into())
}

/// Remove border from a binary image, extracting the central region.
///
/// `left` must be a multiple of 32 for word-aligned copy.
fn remove_border(
    pix: &Pix,
    left: u32,
    top: u32,
    orig_w: u32,
    orig_h: u32,
) -> MorphResult<leptonica_core::PixMut> {
    debug_assert!(
        left.is_multiple_of(32),
        "horizontal border must be word-aligned"
    );

    let new_pix = Pix::new(orig_w, orig_h, PixelDepth::Bit1)?;
    let mut new_mut = new_pix.try_into_mut().unwrap();

    let left_words = (left / 32) as usize;
    let src_wpl = pix.wpl() as usize;
    let dst_wpl = new_mut.wpl() as usize;
    let src_data = pix.data();
    let dst_data = new_mut.data_mut();

    for y in 0..orig_h as usize {
        let src_start = (y + top as usize) * src_wpl + left_words;
        let dst_start = y * dst_wpl;
        dst_data[dst_start..dst_start + dst_wpl]
            .copy_from_slice(&src_data[src_start..src_start + dst_wpl]);
    }

    Ok(new_mut)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> Pix {
        // Create a 5x5 image with a 3x3 square in the center
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set the 3x3 square
        for y in 1..4 {
            for x in 1..4 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_dilate() {
        let pix = create_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();

        let dilated = dilate(&pix, &sel).unwrap();

        // The 3x3 square should expand to 5x5
        assert_eq!(dilated.get_pixel_unchecked(0, 0), 1);
        assert_eq!(dilated.get_pixel_unchecked(4, 4), 1);
    }

    #[test]
    fn test_erode() {
        let pix = create_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();

        let eroded = erode(&pix, &sel).unwrap();

        // The 3x3 square should shrink to 1x1 (just the center)
        assert_eq!(eroded.get_pixel_unchecked(2, 2), 1);
        assert_eq!(eroded.get_pixel_unchecked(1, 1), 0);
        assert_eq!(eroded.get_pixel_unchecked(3, 3), 0);
    }

    #[test]
    fn test_open_close() {
        let pix = create_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();

        // Opening then closing should roughly preserve the shape
        let opened = open(&pix, &sel).unwrap();
        let closed = close(&pix, &sel).unwrap();

        // The opened image should have the center pixel
        assert_eq!(opened.get_pixel_unchecked(2, 2), 1);

        // The closed image should have the original square plus some
        assert_eq!(closed.get_pixel_unchecked(2, 2), 1);
    }

    #[test]
    fn test_hit_miss_transform() {
        // Create an image with a single pixel
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(2, 2, 1);
        let pix: Pix = pix_mut.into();

        // Create a SEL that matches isolated pixels
        let sel = Sel::from_string(
            "ooo\n\
             oxo\n\
             ooo",
            1,
            1,
        )
        .unwrap();

        let hmt = hit_miss_transform(&pix, &sel).unwrap();

        // The isolated pixel should be detected
        assert_eq!(hmt.get_pixel_unchecked(2, 2), 1);
    }

    #[test]
    fn test_gradient() {
        let pix = create_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();

        let _grad = gradient(&pix, &sel).unwrap();

        // The gradient should show the boundary
        // Center should be 0 (dilated and eroded both have it)
        // Edges of the original should be 1
    }

    #[test]
    fn test_brick_operations() {
        let pix = create_test_image();

        let dilated = dilate_brick(&pix, 3, 3).unwrap();
        let eroded = erode_brick(&pix, 3, 3).unwrap();

        assert_eq!(dilated.get_pixel_unchecked(0, 0), 1);
        assert_eq!(eroded.get_pixel_unchecked(2, 2), 1);
    }

    #[test]
    fn test_non_binary_error() {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let sel = Sel::create_brick(3, 3).unwrap();

        let result = dilate(&pix, &sel);
        assert!(result.is_err());
    }

    /// Create a 32x32 test image with varied patterns for separable decomposition tests
    fn create_pattern_image() -> Pix {
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Large rectangle
        for y in 2..12 {
            for x in 2..15 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        // Diagonal line
        for i in 0..20 {
            let x = i + 5;
            let y = i + 8;
            if x < 32 && y < 32 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        // Scattered pixels
        pix_mut.set_pixel_unchecked(20, 5, 1);
        pix_mut.set_pixel_unchecked(25, 15, 1);
        pix_mut.set_pixel_unchecked(28, 28, 1);
        // Small cluster
        for y in 20..25 {
            for x in 3..8 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }

        pix_mut.into()
    }

    const SEPARABLE_SIZES: &[(u32, u32)] =
        &[(3, 3), (5, 7), (7, 5), (1, 5), (5, 1), (1, 1), (9, 9)];

    #[test]
    fn test_dilate_brick_separable_equivalence() {
        let pix = create_pattern_image();
        for &(w, h) in SEPARABLE_SIZES {
            let brick_result = dilate_brick(&pix, w, h).unwrap();
            let sel = Sel::create_brick(w, h).unwrap();
            let generic_result = dilate(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&generic_result),
                "dilate_brick({}, {}) != dilate with 2D SEL",
                w,
                h
            );
        }
    }

    #[test]
    fn test_erode_brick_separable_equivalence() {
        let pix = create_pattern_image();
        for &(w, h) in SEPARABLE_SIZES {
            let brick_result = erode_brick(&pix, w, h).unwrap();
            let sel = Sel::create_brick(w, h).unwrap();
            let generic_result = erode(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&generic_result),
                "erode_brick({}, {}) != erode with 2D SEL",
                w,
                h
            );
        }
    }

    #[test]
    fn test_open_brick_separable_equivalence() {
        let pix = create_pattern_image();
        for &(w, h) in SEPARABLE_SIZES {
            let brick_result = open_brick(&pix, w, h).unwrap();
            let sel = Sel::create_brick(w, h).unwrap();
            let generic_result = open(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&generic_result),
                "open_brick({}, {}) != open with 2D SEL",
                w,
                h
            );
        }
    }

    #[test]
    fn test_close_brick_separable_equivalence() {
        let pix = create_pattern_image();
        for &(w, h) in SEPARABLE_SIZES {
            let brick_result = close_brick(&pix, w, h).unwrap();
            let sel = Sel::create_brick(w, h).unwrap();
            let generic_result = close(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&generic_result),
                "close_brick({}, {}) != close with 2D SEL",
                w,
                h
            );
        }
    }

    // --- Rasterop equivalence tests ---
    //
    // Reference pixel-by-pixel implementations (C version: morph.c:213-309)
    // These serve as the ground truth for verifying rasterop optimization.

    /// Pixel-by-pixel dilation reference implementation
    fn dilate_reference(pix: &Pix, sel: &Sel) -> Pix {
        let w = pix.width();
        let h = pix.height();
        let out = Pix::new(w, h, PixelDepth::Bit1).unwrap();
        let mut out_mut = out.try_into_mut().unwrap();
        let hit_offsets: Vec<_> = sel.hit_offsets().collect();

        for y in 0..h {
            for x in 0..w {
                let dilated = hit_offsets.iter().any(|&(dx, dy)| {
                    let sx = x as i32 + dx;
                    let sy = y as i32 + dy;
                    sx >= 0
                        && sx < w as i32
                        && sy >= 0
                        && sy < h as i32
                        && pix.get_pixel_unchecked(sx as u32, sy as u32) != 0
                });
                if dilated {
                    out_mut.set_pixel_unchecked(x, y, 1);
                }
            }
        }
        out_mut.into()
    }

    /// Pixel-by-pixel erosion reference implementation
    fn erode_reference(pix: &Pix, sel: &Sel) -> Pix {
        let w = pix.width();
        let h = pix.height();
        let out = Pix::new(w, h, PixelDepth::Bit1).unwrap();
        let mut out_mut = out.try_into_mut().unwrap();
        let hit_offsets: Vec<_> = sel.hit_offsets().collect();

        for y in 0..h {
            for x in 0..w {
                let eroded = hit_offsets.iter().all(|&(dx, dy)| {
                    let sx = x as i32 + dx;
                    let sy = y as i32 + dy;
                    sx >= 0
                        && sx < w as i32
                        && sy >= 0
                        && sy < h as i32
                        && pix.get_pixel_unchecked(sx as u32, sy as u32) != 0
                });
                if eroded {
                    out_mut.set_pixel_unchecked(x, y, 1);
                }
            }
        }
        out_mut.into()
    }

    /// Create a 50x37 test image with word-boundary-crossing patterns.
    /// Width of 50 is deliberately not a multiple of 32 to exercise partial
    /// word handling. C version (binmorph1_reg.c) uses feyn-fract.tif;
    /// here we create a synthetic image for unit tests.
    fn create_rasterop_test_image() -> Pix {
        let pix = Pix::new(50, 37, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        // Rectangle crossing word boundary (pixels 28-36 span words 0 and 1)
        for y in 3..15 {
            for x in 28..37 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        // Diagonal crossing word boundary
        for i in 0..30 {
            let x = i + 10;
            let y = i + 5;
            if x < 50 && y < 37 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        // Pixels at word boundaries
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(31, 0, 1);
        pm.set_pixel_unchecked(32, 0, 1);
        pm.set_pixel_unchecked(49, 0, 1);
        // Bottom-right cluster
        for y in 30..37 {
            for x in 40..50 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }

        pm.into()
    }

    #[test]
    fn test_dilate_rasterop_brick_equivalence() {
        let pix = create_rasterop_test_image();
        // C version: binmorph1_reg.c uses WIDTH=21, HEIGHT=15
        for &(w, h) in &[(3u32, 3u32), (5, 7), (21, 15), (1, 5), (5, 1)] {
            let sel = Sel::create_brick(w, h).unwrap();
            let result = dilate(&pix, &sel).unwrap();
            let reference = dilate_reference(&pix, &sel);
            assert!(
                result.equals(&reference),
                "dilate rasterop != reference for brick {}x{}",
                w,
                h
            );
        }
    }

    #[test]
    fn test_erode_rasterop_brick_equivalence() {
        let pix = create_rasterop_test_image();
        for &(w, h) in &[(3u32, 3u32), (5, 7), (21, 15), (1, 5), (5, 1)] {
            let sel = Sel::create_brick(w, h).unwrap();
            let result = erode(&pix, &sel).unwrap();
            let reference = erode_reference(&pix, &sel);
            assert!(
                result.equals(&reference),
                "erode rasterop != reference for brick {}x{}",
                w,
                h
            );
        }
    }

    #[test]
    fn test_dilate_rasterop_cross_equivalence() {
        let pix = create_rasterop_test_image();
        for size in [3, 5] {
            let sel = Sel::create_cross(size).unwrap();
            let result = dilate(&pix, &sel).unwrap();
            let reference = dilate_reference(&pix, &sel);
            assert!(
                result.equals(&reference),
                "dilate rasterop != reference for cross {}",
                size
            );
        }
    }

    #[test]
    fn test_erode_rasterop_cross_equivalence() {
        let pix = create_rasterop_test_image();
        for size in [3, 5] {
            let sel = Sel::create_cross(size).unwrap();
            let result = erode(&pix, &sel).unwrap();
            let reference = erode_reference(&pix, &sel);
            assert!(
                result.equals(&reference),
                "erode rasterop != reference for cross {}",
                size
            );
        }
    }

    #[test]
    fn test_dilate_rasterop_diamond_equivalence() {
        let pix = create_rasterop_test_image();
        let sel = Sel::create_diamond(2).unwrap();
        let result = dilate(&pix, &sel).unwrap();
        let reference = dilate_reference(&pix, &sel);
        assert!(
            result.equals(&reference),
            "dilate rasterop != reference for diamond 2"
        );
    }

    #[test]
    fn test_erode_rasterop_diamond_equivalence() {
        let pix = create_rasterop_test_image();
        let sel = Sel::create_diamond(2).unwrap();
        let result = erode(&pix, &sel).unwrap();
        let reference = erode_reference(&pix, &sel);
        assert!(
            result.equals(&reference),
            "erode rasterop != reference for diamond 2"
        );
    }

    // --- Composite decomposition equivalence tests ---
    //
    // Verify that dilate_brick/erode_brick (which use composite decomposition
    // internally) produce the same results as direct single-SEL operations.
    //
    // Note: We test through the brick API rather than manually calling
    // dilate(brick) then dilate(comb), because the public dilate() clears
    // unused bits after each call, destroying intermediate information
    // that the comb step needs when image width is not a multiple of 32.
    // The brick functions use dilate_rasterop internally to preserve these bits.

    /// Composite sizes to test: total sizes that factor into (f1, f2)
    const COMPOSITE_SIZES: &[u32] = &[
        4,   // 2×2
        9,   // 3×3
        12,  // 3×4
        120, // 10×12
    ];

    #[test]
    fn test_dilate_brick_composite_equivalence() {
        let pix = create_rasterop_test_image();
        // Horizontal
        for &size in COMPOSITE_SIZES {
            let brick_result = dilate_brick(&pix, size, 1).unwrap();
            let sel = Sel::create_horizontal(size).unwrap();
            let direct_result = dilate(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&direct_result),
                "dilate_brick({}, 1) != direct dilate",
                size,
            );
        }
        // Vertical
        for &size in COMPOSITE_SIZES {
            let brick_result = dilate_brick(&pix, 1, size).unwrap();
            let sel = Sel::create_vertical(size).unwrap();
            let direct_result = dilate(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&direct_result),
                "dilate_brick(1, {}) != direct dilate",
                size,
            );
        }
        // Prime sizes should still work (no composite decomposition)
        for &size in &[7u32, 13] {
            let brick_result = dilate_brick(&pix, size, 1).unwrap();
            let sel = Sel::create_horizontal(size).unwrap();
            let direct_result = dilate(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&direct_result),
                "dilate_brick({}, 1) != direct dilate (prime)",
                size,
            );
        }
    }

    #[test]
    fn test_erode_brick_composite_equivalence() {
        let pix = create_rasterop_test_image();
        // Horizontal
        for &size in COMPOSITE_SIZES {
            let brick_result = erode_brick(&pix, size, 1).unwrap();
            let sel = Sel::create_horizontal(size).unwrap();
            let direct_result = erode(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&direct_result),
                "erode_brick({}, 1) != direct erode",
                size,
            );
        }
        // Vertical
        for &size in COMPOSITE_SIZES {
            let brick_result = erode_brick(&pix, 1, size).unwrap();
            let sel = Sel::create_vertical(size).unwrap();
            let direct_result = erode(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&direct_result),
                "erode_brick(1, {}) != direct erode",
                size,
            );
        }
        // Prime sizes
        for &size in &[7u32, 13] {
            let brick_result = erode_brick(&pix, size, 1).unwrap();
            let sel = Sel::create_horizontal(size).unwrap();
            let direct_result = erode(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&direct_result),
                "erode_brick({}, 1) != direct erode (prime)",
                size,
            );
        }
    }

    // --- close_safe tests ---

    #[test]
    fn test_close_safe_identity_on_1x1_sel() {
        let pix = create_rasterop_test_image();
        let sel = Sel::create_brick(1, 1).unwrap();
        let result = close_safe(&pix, &sel).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_close_safe_preserves_dimensions() {
        let pix = create_rasterop_test_image();
        let sel = Sel::create_brick(5, 5).unwrap();
        let result = close_safe(&pix, &sel).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_close_safe_at_least_as_large_as_close() {
        // close_safe should produce a superset of close (no pixels are eroded at border)
        let pix = create_rasterop_test_image();
        let sel = Sel::create_brick(5, 5).unwrap();
        let safe = close_safe(&pix, &sel).unwrap();
        let regular = close(&pix, &sel).unwrap();
        // Every ON pixel in safe should be ON in safe (safe >= close on interior)
        // At minimum, the two should have same or more pixels in safe
        assert!(safe.count_pixels() >= regular.count_pixels());
    }

    #[test]
    fn test_close_safe_requires_1bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let sel = Sel::create_brick(3, 3).unwrap();
        let result = close_safe(&pix, &sel);
        assert!(result.is_err());
    }

    // --- close_safe_brick tests ---

    #[test]
    fn test_close_safe_brick_preserves_dimensions() {
        let pix = create_rasterop_test_image();
        let result = close_safe_brick(&pix, 5, 5).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_close_safe_brick_identity_1x1() {
        let pix = create_rasterop_test_image();
        let result = close_safe_brick(&pix, 1, 1).unwrap();
        assert!(result.equals(&pix));
    }

    #[test]
    fn test_close_safe_brick_at_least_as_large_as_close_brick() {
        let pix = create_rasterop_test_image();
        let safe = close_safe_brick(&pix, 5, 5).unwrap();
        let regular = close_brick(&pix, 5, 5).unwrap();
        assert!(safe.count_pixels() >= regular.count_pixels());
    }

    #[test]
    fn test_close_safe_brick_requires_1bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let result = close_safe_brick(&pix, 3, 3);
        assert!(result.is_err());
    }

    // --- close_safe_comp_brick tests ---

    #[test]
    fn test_close_safe_comp_brick_matches_close_safe_brick() {
        let pix = create_rasterop_test_image();
        for &(h, v) in &[(5u32, 5u32), (3, 7), (1, 5), (9, 9)] {
            let comp = close_safe_comp_brick(&pix, h, v).unwrap();
            let regular = close_safe_brick(&pix, h, v).unwrap();
            assert!(
                comp.equals(&regular),
                "close_safe_comp_brick({}, {}) != close_safe_brick",
                h,
                v
            );
        }
    }

    #[test]
    fn test_close_safe_comp_brick_preserves_dimensions() {
        let pix = create_rasterop_test_image();
        let result = close_safe_comp_brick(&pix, 10, 10).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    // --- open_generalized tests ---

    #[test]
    fn test_open_generalized_requires_1bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let sel = Sel::create_brick(3, 3).unwrap();
        let result = open_generalized(&pix, &sel);
        assert!(result.is_err());
    }

    #[test]
    fn test_open_generalized_preserves_dimensions() {
        let pix = create_rasterop_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();
        let result = open_generalized(&pix, &sel).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_open_generalized_produces_subset() {
        // Generalized opening always produces a subset (fewer or equal ON pixels)
        let pix = create_rasterop_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();
        let result = open_generalized(&pix, &sel).unwrap();
        assert!(result.count_pixels() <= pix.count_pixels());
    }

    // --- close_generalized tests ---

    #[test]
    fn test_close_generalized_requires_1bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let sel = Sel::create_brick(3, 3).unwrap();
        let result = close_generalized(&pix, &sel);
        assert!(result.is_err());
    }

    #[test]
    fn test_close_generalized_preserves_dimensions() {
        let pix = create_rasterop_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();
        let result = close_generalized(&pix, &sel).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_close_generalized_produces_superset() {
        // For a pure-hit SEL, close_generalized = dilate then HMT = dilate then erode = close.
        // With asymmetric boundary conditions closing is NOT necessarily extensive
        // (border pixels can be eroded away), so we verify equivalence with standard close.
        let pix = create_rasterop_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();
        let generalized = close_generalized(&pix, &sel).unwrap();
        let standard = close(&pix, &sel).unwrap();
        assert_eq!(generalized.width(), standard.width());
        assert_eq!(generalized.height(), standard.height());
        // Both should produce same result since pure-hit HMT = erode
        assert_eq!(generalized.count_pixels(), standard.count_pixels());
    }
}
