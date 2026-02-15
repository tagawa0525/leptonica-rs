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
                continue; // Outside = background (0), OR with 0 is no-op
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

    // Clear unused bits in the last word of each row to prevent
    // contamination in subsequent operations (e.g., erosion after dilation
    // in closing). Word-level shifts can set bits beyond the image width.
    clear_unused_bits(dst_data, w, wpl);

    Ok(out_mut.into())
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
            // Erosion uses inverted offsets: src[y + dy] shifted by -dx
            let src_y = y + dy;
            let dst_start = y as usize * wpl;

            if src_y < 0 || src_y >= h as i32 {
                // Outside = background (0), AND with 0 clears the row
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

    // Clear unused bits in the last word of each row.
    // The initial all-1s fill sets unused bits, and AND operations
    // may not clear them if source data also has unused bits set.
    clear_unused_bits(dst_data, pix.width(), wpl);

    Ok(out_mut.into())
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

/// Dilate with a brick (rectangular) structuring element
///
/// Optimized for rectangular SEs using separable decomposition.
/// Complexity: O(W × H × (width + height)) instead of O(W × H × width × height)
pub fn dilate_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    // Identity case
    if width == 1 && height == 1 {
        return Ok(pix.clone());
    }

    // 1D cases
    if width == 1 {
        let sel = Sel::create_vertical(height)?;
        return dilate(pix, &sel);
    }
    if height == 1 {
        let sel = Sel::create_horizontal(width)?;
        return dilate(pix, &sel);
    }

    // Separable decomposition: dilate(pix, horz) then dilate(tmp, vert)
    let sel_h = Sel::create_horizontal(width)?;
    let tmp = dilate(pix, &sel_h)?;
    let sel_v = Sel::create_vertical(height)?;
    dilate(&tmp, &sel_v)
}

/// Erode with a brick (rectangular) structuring element
///
/// Optimized for rectangular SEs using separable decomposition.
/// Complexity: O(W × H × (width + height)) instead of O(W × H × width × height)
pub fn erode_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    // Identity case
    if width == 1 && height == 1 {
        return Ok(pix.clone());
    }

    // 1D cases
    if width == 1 {
        let sel = Sel::create_vertical(height)?;
        return erode(pix, &sel);
    }
    if height == 1 {
        let sel = Sel::create_horizontal(width)?;
        return erode(pix, &sel);
    }

    // Separable decomposition: erode(pix, horz) then erode(tmp, vert)
    let sel_h = Sel::create_horizontal(width)?;
    let tmp = erode(pix, &sel_h)?;
    let sel_v = Sel::create_vertical(height)?;
    erode(&tmp, &sel_v)
}

/// Open with a brick structuring element
///
/// Optimized for rectangular SEs using separable decomposition.
/// Opening = erosion followed by dilation.
/// Separable: erode(horz) → erode(vert) → dilate(horz) → dilate(vert)
pub fn open_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    // Identity case
    if width == 1 && height == 1 {
        return Ok(pix.clone());
    }

    // 1D cases - use non-separable generic version (still fast)
    if width == 1 || height == 1 {
        let sel = Sel::create_brick(width, height)?;
        return open(pix, &sel);
    }

    // Separable decomposition: 4 passes
    // Erode: horizontal then vertical
    let sel_h = Sel::create_horizontal(width)?;
    let step1 = erode(pix, &sel_h)?;
    let sel_v = Sel::create_vertical(height)?;
    let step2 = erode(&step1, &sel_v)?;

    // Dilate: horizontal then vertical
    let step3 = dilate(&step2, &sel_h)?;
    dilate(&step3, &sel_v)
}

/// Close with a brick structuring element
///
/// Optimized for rectangular SEs using separable decomposition.
/// Closing = dilation followed by erosion.
/// Separable: dilate(horz) → dilate(vert) → erode(horz) → erode(vert)
pub fn close_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    // Identity case
    if width == 1 && height == 1 {
        return Ok(pix.clone());
    }

    // 1D cases - use non-separable generic version (still fast)
    if width == 1 || height == 1 {
        let sel = Sel::create_brick(width, height)?;
        return close(pix, &sel);
    }

    // Separable decomposition: 4 passes
    // Dilate: horizontal then vertical
    let sel_h = Sel::create_horizontal(width)?;
    let step1 = dilate(pix, &sel_h)?;
    let sel_v = Sel::create_vertical(height)?;
    let step2 = dilate(&step1, &sel_v)?;

    // Erode: horizontal then vertical
    let step3 = erode(&step2, &sel_h)?;
    erode(&step3, &sel_v)
}

/// Shift src row by `shift` pixels and OR into dst (word-level).
///
/// MSB-first bit ordering: pixel 0 = bit 31, pixel 31 = bit 0.
/// Positive shift = image content moves right (src >> shift in bit terms).
/// Negative shift = image content moves left (src << |shift| in bit terms).
///
/// Inner loops have no bounds checks to enable auto-vectorization.
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
    // Verify that brick(f1) then comb(f1, f2) == brick(f1*f2)
    // for both dilation and erosion.

    /// Composite sizes to test: (f1, f2) pairs where f1*f2 = size
    const COMPOSITE_SIZES: &[(u32, u32)] = &[
        (2, 2),   // size 4
        (3, 3),   // size 9
        (3, 4),   // size 12
        (10, 12), // size 120
    ];

    #[test]
    #[ignore = "composite decomposition not yet implemented"]
    fn test_dilate_composite_horizontal_equivalence() {
        let pix = create_rasterop_test_image();
        for &(f1, f2) in COMPOSITE_SIZES {
            let size = f1 * f2;
            // Composite: brick(f1) then comb(f1, f2)
            let sel_brick = Sel::create_horizontal(f1).unwrap();
            let sel_comb = Sel::create_comb_horizontal(f1, f2).unwrap();
            let tmp = dilate(&pix, &sel_brick).unwrap();
            let composite_result = dilate(&tmp, &sel_comb).unwrap();
            // Direct: brick(size)
            let sel_full = Sel::create_horizontal(size).unwrap();
            let direct_result = dilate(&pix, &sel_full).unwrap();
            assert!(
                composite_result.equals(&direct_result),
                "dilate composite h ({}x{}) != direct h ({})",
                f1,
                f2,
                size,
            );
        }
    }

    #[test]
    #[ignore = "composite decomposition not yet implemented"]
    fn test_dilate_composite_vertical_equivalence() {
        let pix = create_rasterop_test_image();
        for &(f1, f2) in COMPOSITE_SIZES {
            let size = f1 * f2;
            let sel_brick = Sel::create_vertical(f1).unwrap();
            let sel_comb = Sel::create_comb_vertical(f1, f2).unwrap();
            let tmp = dilate(&pix, &sel_brick).unwrap();
            let composite_result = dilate(&tmp, &sel_comb).unwrap();
            let sel_full = Sel::create_vertical(size).unwrap();
            let direct_result = dilate(&pix, &sel_full).unwrap();
            assert!(
                composite_result.equals(&direct_result),
                "dilate composite v ({}x{}) != direct v ({})",
                f1,
                f2,
                size,
            );
        }
    }

    #[test]
    #[ignore = "composite decomposition not yet implemented"]
    fn test_erode_composite_horizontal_equivalence() {
        let pix = create_rasterop_test_image();
        for &(f1, f2) in COMPOSITE_SIZES {
            let size = f1 * f2;
            let sel_brick = Sel::create_horizontal(f1).unwrap();
            let sel_comb = Sel::create_comb_horizontal(f1, f2).unwrap();
            let tmp = erode(&pix, &sel_brick).unwrap();
            let composite_result = erode(&tmp, &sel_comb).unwrap();
            let sel_full = Sel::create_horizontal(size).unwrap();
            let direct_result = erode(&pix, &sel_full).unwrap();
            assert!(
                composite_result.equals(&direct_result),
                "erode composite h ({}x{}) != direct h ({})",
                f1,
                f2,
                size,
            );
        }
    }

    #[test]
    #[ignore = "composite decomposition not yet implemented"]
    fn test_erode_composite_vertical_equivalence() {
        let pix = create_rasterop_test_image();
        for &(f1, f2) in COMPOSITE_SIZES {
            let size = f1 * f2;
            let sel_brick = Sel::create_vertical(f1).unwrap();
            let sel_comb = Sel::create_comb_vertical(f1, f2).unwrap();
            let tmp = erode(&pix, &sel_brick).unwrap();
            let composite_result = erode(&tmp, &sel_comb).unwrap();
            let sel_full = Sel::create_vertical(size).unwrap();
            let direct_result = erode(&pix, &sel_full).unwrap();
            assert!(
                composite_result.equals(&direct_result),
                "erode composite v ({}x{}) != direct v ({})",
                f1,
                f2,
                size,
            );
        }
    }

    #[test]
    #[ignore = "composite decomposition not yet implemented"]
    fn test_dilate_brick_composite_equivalence() {
        let pix = create_rasterop_test_image();
        // dilate_brick should use composite internally and match direct result
        for &(f1, f2) in COMPOSITE_SIZES {
            let size = f1 * f2;
            let brick_result = dilate_brick(&pix, size, 1).unwrap();
            let sel = Sel::create_horizontal(size).unwrap();
            let direct_result = dilate(&pix, &sel).unwrap();
            assert!(
                brick_result.equals(&direct_result),
                "dilate_brick({}, 1) != direct dilate",
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
}
