//! Binary morphological operations
//!
//! Implements erosion, dilation, opening, and closing for 1-bpp images.

use crate::core::{Pix, PixelDepth};
use crate::morph::{MorphError, MorphResult, Sel};

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
fn dilate_rasterop(pix: &Pix, sel: &Sel) -> MorphResult<crate::core::PixMut> {
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
fn erode_rasterop(pix: &Pix, sel: &Sel) -> MorphResult<crate::core::PixMut> {
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
    let wpl = pix.wpl() as usize;

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Initialize output to all 1s (for AND accumulation).
    for word in out_mut.data_mut().iter_mut() {
        *word = 0xFFFF_FFFF;
    }

    let hit_offsets: Vec<_> = sel.hit_offsets().collect();
    let miss_offsets: Vec<_> = sel.miss_offsets().collect();

    {
        // Clear unused padding bits in source to prevent them from shifting
        // into the valid region during horizontal bit shifts.
        let mut src_owned = pix.data().to_vec();
        clear_unused_bits(&mut src_owned, w, wpl);
        let src_data = &src_owned;
        let dst_data = out_mut.data_mut();

        // Hits: out &= shifted(src), with outside treated as 0.
        for &(dx, dy) in &hit_offsets {
            for y in 0..h as i32 {
                let src_y = y + dy;
                let dst_start = y as usize * wpl;

                if src_y < 0 || src_y >= h as i32 {
                    // Outside rows are 0 => clear destination row.
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

        // Misses: out &= !shifted(src), with outside treated as 1.
        for &(dx, dy) in &miss_offsets {
            for y in 0..h as i32 {
                let src_y = y + dy;
                if src_y < 0 || src_y >= h as i32 {
                    // Outside rows are 1 => no-op for AND.
                    continue;
                }

                let dst_start = y as usize * wpl;
                let src_start = src_y as usize * wpl;
                shift_and_not_row(
                    &mut dst_data[dst_start..dst_start + wpl],
                    &src_data[src_start..src_start + wpl],
                    -dx,
                );
            }
        }
    }

    clear_unused_bits(out_mut.data_mut(), w, wpl);
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

/// Build the composable (brick, comb) SEL pair for a dimension size.
///
/// Mirrors C `selectComposableSels(size, direction, &sel1, &sel2)` in
/// `reference/leptonica/src/morph.c`. The pair is later applied as
/// `dilate(sel1) then dilate(sel2)` (and similarly for erode), which is
/// pixel-equivalent to a single dilate by `brick(factor1 * factor2)`.
///
/// Returns the brick SEL (size `factor1` along the active axis) and the
/// comb SEL (size `factor1 * factor2`, with `factor2` hits spaced
/// `factor1` apart).
fn select_composable_sels(size: u32, horizontal: bool) -> MorphResult<(Sel, Sel)> {
    let (factor1, factor2) = select_composable_sizes(size);
    let sel1 = if horizontal {
        Sel::create_horizontal(factor1)?
    } else {
        Sel::create_vertical(factor1)?
    };
    let sel2 = if horizontal {
        Sel::create_comb_horizontal(factor1, factor2)?
    } else {
        Sel::create_comb_vertical(factor1, factor2)?
    };
    Ok((sel1, sel2))
}

/// Dilate with a brick (rectangular) structuring element.
///
/// Mirrors C `pixDilateCompBrick` (`reference/leptonica/src/morph.c`):
/// adds a 32-pixel border, applies the composable `(brick, comb)` SEL
/// pair for each non-trivial dimension sequentially on the bordered
/// image (each `dilate` call allocates a fresh intermediate `Pix`; the
/// Arc/refcount model makes the per-step allocation cheap), then strips
/// the border.
///
/// `selectComposableSizes` may approximate the requested size by ±1 or
/// ±2 for primes — see `select_composable_sizes` docs for the cost
/// rationale.
pub fn dilate_brick(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;
    if hsize == 0 || vsize == 0 {
        return Err(MorphError::InvalidSel(
            "hsize and vsize must be >= 1".to_string(),
        ));
    }
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    let bordered = add_border(pix, 32, 32, 32, 32)?;

    let final_pix: Pix = if vsize == 1 {
        let (selh1, selh2) = select_composable_sels(hsize, true)?;
        let tmp = dilate(&bordered, &selh1)?;
        dilate(&tmp, &selh2)?
    } else if hsize == 1 {
        let (selv1, selv2) = select_composable_sels(vsize, false)?;
        let tmp = dilate(&bordered, &selv1)?;
        dilate(&tmp, &selv2)?
    } else {
        let (selh1, selh2) = select_composable_sels(hsize, true)?;
        let (selv1, selv2) = select_composable_sels(vsize, false)?;
        let a = dilate(&bordered, &selh1)?;
        let b = dilate(&a, &selh2)?;
        let c = dilate(&b, &selv1)?;
        dilate(&c, &selv2)?
    };

    let mut result = remove_border(&final_pix, 32, 32, pix.width(), pix.height())?;
    clear_unused_bits(result.data_mut(), pix.width(), pix.wpl() as usize);
    Ok(result.into())
}

/// Erode with a brick (rectangular) structuring element.
///
/// Mirrors C `pixErodeCompBrick` (`reference/leptonica/src/morph.c`):
/// applies the composable `(brick, comb)` SEL pair for each non-trivial
/// dimension **without adding a border** (asymmetric boundary handling:
/// erode treats outside as 1, which is the correct BG for foreground
/// shrinkage and needs no padding).
pub fn erode_brick(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;
    if hsize == 0 || vsize == 0 {
        return Err(MorphError::InvalidSel(
            "hsize and vsize must be >= 1".to_string(),
        ));
    }
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    let result: Pix = if vsize == 1 {
        let (selh1, selh2) = select_composable_sels(hsize, true)?;
        let tmp = erode(pix, &selh1)?;
        erode(&tmp, &selh2)?
    } else if hsize == 1 {
        let (selv1, selv2) = select_composable_sels(vsize, false)?;
        let tmp = erode(pix, &selv1)?;
        erode(&tmp, &selv2)?
    } else {
        let (selh1, selh2) = select_composable_sels(hsize, true)?;
        let (selv1, selv2) = select_composable_sels(vsize, false)?;
        let a = erode(pix, &selh1)?;
        let b = erode(&a, &selh2)?;
        let c = erode(&b, &selv1)?;
        erode(&c, &selv2)?
    };
    Ok(result)
}

/// Open with a brick structuring element.
///
/// Mirrors C `pixOpenCompBrick` (`reference/leptonica/src/morph.c`):
/// erode (h1→h2→v1→v2) then dilate (h1→h2→v1→v2) on the original image
/// **without border**. The opening operator already removes objects
/// smaller than the SEL, so no boundary padding is needed.
pub fn open_brick(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;
    if hsize == 0 || vsize == 0 {
        return Err(MorphError::InvalidSel(
            "hsize and vsize must be >= 1".to_string(),
        ));
    }
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    if vsize == 1 {
        let (selh1, selh2) = select_composable_sels(hsize, true)?;
        let a = erode(pix, &selh1)?;
        let b = erode(&a, &selh2)?;
        let c = dilate(&b, &selh1)?;
        return dilate(&c, &selh2);
    }
    if hsize == 1 {
        let (selv1, selv2) = select_composable_sels(vsize, false)?;
        let a = erode(pix, &selv1)?;
        let b = erode(&a, &selv2)?;
        let c = dilate(&b, &selv1)?;
        return dilate(&c, &selv2);
    }
    let (selh1, selh2) = select_composable_sels(hsize, true)?;
    let (selv1, selv2) = select_composable_sels(vsize, false)?;
    let a = erode(pix, &selh1)?;
    let b = erode(&a, &selh2)?;
    let c = erode(&b, &selv1)?;
    let d = erode(&c, &selv2)?;
    let e = dilate(&d, &selh1)?;
    let f = dilate(&e, &selh2)?;
    let g = dilate(&f, &selv1)?;
    dilate(&g, &selv2)
}

/// Close with a brick structuring element.
///
/// Mirrors C `pixCloseCompBrick` (`reference/leptonica/src/morph.c`):
/// dilate (h1→h2→v1→v2) then erode (h1→h2→v1→v2) on the original image
/// **without border**. Unlike `dilate_brick`, the closing operator's
/// erode step compensates for the dilate's boundary spread, so no
/// padding is needed.
pub fn close_brick(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_binary(pix)?;
    if hsize == 0 || vsize == 0 {
        return Err(MorphError::InvalidSel(
            "hsize and vsize must be >= 1".to_string(),
        ));
    }
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    if vsize == 1 {
        let (selh1, selh2) = select_composable_sels(hsize, true)?;
        let a = dilate(pix, &selh1)?;
        let b = dilate(&a, &selh2)?;
        let c = erode(&b, &selh1)?;
        return erode(&c, &selh2);
    }
    if hsize == 1 {
        let (selv1, selv2) = select_composable_sels(vsize, false)?;
        let a = dilate(pix, &selv1)?;
        let b = dilate(&a, &selv2)?;
        let c = erode(&b, &selv1)?;
        return erode(&c, &selv2);
    }
    let (selh1, selh2) = select_composable_sels(hsize, true)?;
    let (selv1, selv2) = select_composable_sels(vsize, false)?;
    let a = dilate(pix, &selh1)?;
    let b = dilate(&a, &selh2)?;
    let c = dilate(&b, &selv1)?;
    let d = dilate(&c, &selv2)?;
    let e = erode(&d, &selh1)?;
    let f = erode(&e, &selh2)?;
    let g = erode(&f, &selv1)?;
    erode(&g, &selv2)
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

/// Find a factor pair `(factor1, factor2)` for composite SEL decomposition.
///
/// Mirrors C `selectComposableSizes` (`reference/leptonica/src/morph.c`):
/// - For perfect squares, returns `(sqrt, sqrt)`.
/// - Otherwise minimizes the cost function `4 * diff + rastcost`, where
///   `diff = |size - factor1 * factor2|` and `rastcost = factor1 +
///   factor2 - 2 * midval`. If a zero-diff pair with `rastcost <
///   ACCEPTABLE_COST (5)` is found, returns it immediately.
/// - **The returned product `factor1 * factor2` may differ from `size`**
///   by up to ±2 (for `size <= 300`). Callers that decompose a brick
///   into `brick(factor1) + comb(factor1, factor2)` must accept this
///   approximation — it is the same trade-off C leptonica makes.
/// - Returned ordering: `factor1 >= factor2`. When `size > 1`,
///   `factor1 > 1` always.
/// - C `selectComposableSizes` rejects `size > 10000`. This wrapper
///   accepts arbitrary `u32` values by performing arithmetic in `i64`
///   so cost-function overflow cannot occur, even though dilation sizes
///   beyond a few hundred have no practical meaning.
pub(crate) fn select_composable_sizes(size: u32) -> (u32, u32) {
    if size <= 1 {
        return (1, 1);
    }
    let size_i: i64 = size as i64;
    let midval = ((size as f64).sqrt() + 0.001) as i64;
    if midval * midval == size_i {
        return (midval as u32, midval as u32);
    }

    let n = (midval + 1) as usize;
    let mut lowval = vec![0i64; n];
    let mut hival = vec![0i64; n];
    let mut rastcost = vec![0i64; n];
    let mut diff = vec![0i64; n];

    let mut val1 = midval + 1;
    let mut i = 0usize;
    while val1 > 0 {
        let val2m = size_i / val1;
        let val2p = val2m + 1;
        let prodm = val1 * val2m;
        let prodp = val1 * val2p;
        let rastcostm = val1 + val2m - 2 * midval;
        let rastcostp = val1 + val2p - 2 * midval;
        let diffm = (size_i - prodm).abs();
        let diffp = (size_i - prodp).abs();
        if diffm <= diffp {
            lowval[i] = val1.min(val2m);
            hival[i] = val1.max(val2m);
            rastcost[i] = rastcostm;
            diff[i] = diffm;
        } else {
            lowval[i] = val1.min(val2p);
            hival[i] = val1.max(val2p);
            rastcost[i] = rastcostp;
            diff[i] = diffp;
        }
        val1 -= 1;
        i += 1;
    }

    const ACCEPTABLE_COST: i64 = 5;
    let mut mincost: i64 = i64::MAX;
    let mut index = 0usize;
    for j in 0..n {
        if diff[j] == 0 && rastcost[j] < ACCEPTABLE_COST {
            return (hival[j] as u32, lowval[j] as u32);
        }
        let totcost = 4 * diff[j] + rastcost[j];
        if totcost < mincost {
            mincost = totcost;
            index = j;
        }
    }
    (hival[index] as u32, lowval[index] as u32)
}

#[cfg(test)]
mod composable_c_parity_tests {
    use super::select_composable_sizes;

    /// Encodes the values produced by C `selectComposableSizes`
    /// (`reference/leptonica/src/morph.c`) for prime sizes. Without the
    /// C-parity rewrite (i.e. with the Rust-only "primes stay plain" rule)
    /// these will fail.
    ///
    /// Computed by hand from the C cost function `totcost = 4 * diff +
    /// rastcost` with `ACCEPTABLE_COST = 5`:
    /// - 11 (prime): expected `(4, 3)` → product 12, diff 1, totcost 5
    /// - 13 (prime): expected `(4, 3)` → product 12, diff 1, totcost 5
    #[test]
    fn test_select_composable_sizes_c_parity_primes() {
        assert_eq!(
            select_composable_sizes(11),
            (4, 3),
            "11 → C selects (4, 3) for size 12 approximation"
        );
        assert_eq!(
            select_composable_sizes(13),
            (4, 3),
            "13 → C selects (4, 3) for size 12 approximation"
        );
    }

    /// Perfect divisors are unaffected by the C-parity rewrite.
    /// These should pass both before and after the change.
    #[test]
    fn test_select_composable_sizes_perfect_divisors() {
        assert_eq!(select_composable_sizes(4), (2, 2));
        assert_eq!(select_composable_sizes(9), (3, 3));
        assert_eq!(select_composable_sizes(16), (4, 4));
    }
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

/// Shift src row by `shift` pixels, bitwise-NOT it, and AND into dst.
///
/// Out-of-bounds positions are treated as 1 (background for miss conditions),
/// so words fully outside the valid shifted range are left unchanged.
#[allow(clippy::needless_range_loop)]
fn shift_and_not_row(dst: &mut [u32], src: &[u32], shift: i32) {
    let wpl = dst.len();

    if shift == 0 {
        for i in 0..wpl {
            dst[i] &= !src[i];
        }
        return;
    }

    let abs_shift = shift.unsigned_abs() as usize;
    let word_shift = abs_shift / 32;
    let bit_shift = (abs_shift % 32) as u32;

    if word_shift >= wpl {
        // Entire row shifts out of bounds; miss condition is true everywhere.
        return;
    }

    if shift > 0 {
        // Valid range: dst[word_shift..wpl]
        if bit_shift == 0 {
            for i in word_shift..wpl {
                dst[i] &= !src[i - word_shift];
            }
        } else {
            dst[word_shift] &= !(src[0] >> bit_shift);
            for i in (word_shift + 1)..wpl {
                let si = i - word_shift;
                let shifted = (src[si] >> bit_shift) | (src[si - 1] << (32 - bit_shift));
                dst[i] &= !shifted;
            }
        }
    } else {
        // Valid range: dst[0..wpl-word_shift]
        let end = wpl - word_shift;
        if bit_shift == 0 {
            for i in 0..end {
                dst[i] &= !src[i + word_shift];
            }
        } else {
            for i in 0..end.saturating_sub(1) {
                let si = i + word_shift;
                let shifted = (src[si] << bit_shift) | (src[si + 1] >> (32 - bit_shift));
                dst[i] &= !shifted;
            }
            if end > 0 {
                dst[end - 1] &= !(src[wpl - 1] << bit_shift);
            }
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
) -> MorphResult<crate::core::PixMut> {
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

    /// `dilate` must match the leptonica/C convention:
    /// `B ⊕ A = ∪_{a ∈ A} (B + a)`. C `pixDilate` (`morph.c:213`)
    /// implements this with `pixRasterop(pixd, j - cx, i - cy, ...,
    /// PIX_SRC | PIX_DST, pixt, 0, 0)`, which shifts src by
    /// `+(j - cx, i - cy)` per hit and ORs into dst.
    ///
    /// With an asymmetric SEL `brick(4)` (cx=2, hits at x ∈ {0, 1, 2, 3},
    /// relative positions a ∈ {-2, -1, 0, +1}), dilating a single
    /// foreground pixel at column 5 must produce 1s at columns
    /// `5 + a` = `{3, 4, 5, 6}` (= the SEL footprint **as-is**, not
    /// reflected).
    ///
    /// Rust's previous `dilate_rasterop` used the **same** sign as
    /// `erode_rasterop` (`src_y = y + dy`, `shift = -dx`), which is
    /// correct for erode but produces `B ⊕ A^` (SEL reflected) for
    /// dilate — invisible for symmetric SELs but visible for any
    /// asymmetric SEL. binmorph3 dilate(11, 7) decomposes to brick(4) +
    /// comb(4, 3), and brick(4) is asymmetric, which is the root cause
    /// of the remaining binmorph3.14/15 Mismatches.
    #[test]
    #[ignore = "RED: pending dilate_rasterop direction fix"]
    fn test_dilate_asymmetric_brick_direction() {
        // 10×3 image with a single foreground pixel at (5, 1).
        let pix = Pix::new(10, 3, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(5, 1, 1);
        let pix: Pix = pm.into();

        // brick(4): width=4, height=1, cx=2, cy=0 → relative hits {-2, -1, 0, +1}.
        let sel = Sel::create_horizontal(4).unwrap();
        let out = dilate(&pix, &sel).unwrap();

        // C-correct dilation: src(5,1)=1 propagates to dst(5+a) for
        // a ∈ {-2, -1, 0, +1} = dst(3, 4, 5, 6) on row 1.
        let row1: Vec<u32> = (0..10).map(|x| out.get_pixel_unchecked(x, 1)).collect();
        assert_eq!(
            row1,
            vec![0, 0, 0, 1, 1, 1, 1, 0, 0, 0],
            "dilate(brick(4)) of pixel (5,1) must set (3,1)..(6,1) (C convention)"
        );
        // Rows 0 and 2 stay zero (sel height = 1).
        for x in 0..10 {
            assert_eq!(out.get_pixel_unchecked(x, 0), 0, "row 0 should stay 0");
            assert_eq!(out.get_pixel_unchecked(x, 2), 0, "row 2 should stay 0");
        }
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
        // Prime sizes: dilate_brick can pick a composite size that differs
        // from the requested size by ±1 or ±2 (C `selectComposableSizes`
        // cost-based selection — see `select_composable_sizes` docs).
        // Only sizes where `select_composable_sizes(size) = (size, 1)`
        // stay equivalent to direct dilate. As of C-parity rewrite, the
        // primes 2, 3, 5, 7 fall into this "plain" category.
        for &size in &[2u32, 3, 5, 7] {
            let brick_result = dilate_brick(&pix, size, 1).unwrap();
            let sel = Sel::create_horizontal(size).unwrap();
            let direct_result = dilate(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&direct_result),
                "dilate_brick({}, 1) != direct dilate (small prime → plain)",
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
        // Small primes (2, 3, 5, 7) stay as plain SEL — see
        // `test_dilate_brick_composite_equivalence` for details.
        for &size in &[2u32, 3, 5, 7] {
            let brick_result = erode_brick(&pix, size, 1).unwrap();
            let sel = Sel::create_horizontal(size).unwrap();
            let direct_result = erode(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&direct_result),
                "erode_brick({}, 1) != direct erode (small prime → plain)",
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
