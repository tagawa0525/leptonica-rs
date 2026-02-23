//! Binary image rank reduction
//!
//! Provides 2x downscaling of 1-bpp images using rank (threshold) filtering.
//! Each 2×2 pixel block in the source is mapped to one pixel in the destination;
//! the output pixel is ON if at least `level` of the 4 source pixels are ON.
//!
//! | level | condition                         | logical equivalent |
//! |-------|-----------------------------------|--------------------|
//! | 1     | any 1 of 4 pixels ON              | OR                 |
//! | 2     | at least 2 of 4 pixels ON         | –                  |
//! | 3     | at least 3 of 4 pixels ON         | –                  |
//! | 4     | all 4 pixels ON                   | AND                |
//!
//! # Reference
//!
//! Based on Leptonica's `binreduce.c`: `pixReduceRankBinary2` and
//! `pixReduceRankBinaryCascade`.

use crate::{TransformError, TransformResult};
use leptonica_core::{Pix, PixelDepth};

/// Reduce a 1-bpp image to half size using rank (threshold) filtering.
///
/// Each 2×2 block of source pixels is collapsed to one output pixel.
/// The output pixel is ON (1) if at least `level` of the 4 source pixels
/// are ON.  Pixels in the last row / column when the dimensions are odd
/// are ignored (output size is `floor(w/2) × floor(h/2)`).
///
/// # Arguments
///
/// * `pix` — 1-bpp source image
/// * `level` — rank threshold, must be 1, 2, 3, or 4
///
/// # Errors
///
/// Returns an error when `pix` is not 1-bpp or `level` is outside `[1, 4]`.
///
/// # Examples
///
/// ```
/// use leptonica_core::{Pix, PixelDepth};
/// use leptonica_transform::binreduce::reduce_rank_binary_2;
///
/// let src = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
/// let dst = reduce_rank_binary_2(&src, 1).unwrap();
/// assert_eq!(dst.width(), 4);
/// assert_eq!(dst.height(), 4);
/// ```
pub fn reduce_rank_binary_2(pix: &Pix, level: u8) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(TransformError::InvalidParameters(
            "reduce_rank_binary_2 requires a 1-bpp image".to_string(),
        ));
    }
    if !(1..=4).contains(&level) {
        return Err(TransformError::InvalidParameters(format!(
            "level must be 1–4, got {level}"
        )));
    }

    let src_w = pix.width();
    let src_h = pix.height();
    let dst_w = src_w / 2;
    let dst_h = src_h / 2;

    let mut dst = Pix::new(dst_w, dst_h, PixelDepth::Bit1)
        .map_err(TransformError::Core)?
        .to_mut();

    for oy in 0..dst_h {
        for ox in 0..dst_w {
            let sx = ox * 2;
            let sy = oy * 2;

            // 2x2 ブロックの 4 ピクセルを取得（境界では 0 扱い）
            let p00 = pix.get_pixel(sx, sy).unwrap_or(0);
            let p10 = pix.get_pixel(sx + 1, sy).unwrap_or(0);
            let p01 = pix.get_pixel(sx, sy + 1).unwrap_or(0);
            let p11 = pix.get_pixel(sx + 1, sy + 1).unwrap_or(0);
            let count = p00 + p10 + p01 + p11;

            if count >= level as u32 {
                dst.set_pixel_unchecked(ox, oy, 1);
            }
        }
    }

    Ok(dst.into())
}

/// Apply cascaded 2x rank reductions to a 1-bpp image.
///
/// Each entry in `levels` specifies the rank threshold for one reduction step.
/// The image is halved in both dimensions at each step, so `levels.len()`
/// reductions produce an image `2^n` times smaller in each dimension.
///
/// An empty `levels` slice returns a clone of the input.
///
/// # Arguments
///
/// * `pix` — 1-bpp source image
/// * `levels` — slice of rank thresholds (each 1–4), applied in order
///
/// # Errors
///
/// Propagates any error from [`reduce_rank_binary_2`].
///
/// # Examples
///
/// ```
/// use leptonica_core::{Pix, PixelDepth};
/// use leptonica_transform::binreduce::reduce_rank_binary_cascade;
///
/// let src = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
/// // "r11" in morph sequence: two rank-1 reductions → 4x smaller
/// let dst = reduce_rank_binary_cascade(&src, &[1, 1]).unwrap();
/// assert_eq!(dst.width(), 4);
/// assert_eq!(dst.height(), 4);
/// ```
pub fn reduce_rank_binary_cascade(pix: &Pix, levels: &[u8]) -> TransformResult<Pix> {
    if levels.is_empty() {
        return Ok(pix.clone());
    }
    let mut current = reduce_rank_binary_2(pix, levels[0])?;
    for &lvl in &levels[1..] {
        current = reduce_rank_binary_2(&current, lvl)?;
    }
    Ok(current)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::PixelDepth;

    // ── helper: create a 1bpp image from a row-major bit pattern ──────────
    fn make_1bpp(width: u32, height: u32, pixels: &[u8]) -> Pix {
        assert_eq!(pixels.len(), (width * height) as usize);
        let mut pix = Pix::new(width, height, PixelDepth::Bit1).unwrap().to_mut();
        for y in 0..height {
            for x in 0..width {
                let v = pixels[(y * width + x) as usize];
                pix.set_pixel(x, y, v as u32).unwrap();
            }
        }
        pix.into()
    }

    // ── size checks ───────────────────────────────────────────────────────

    #[test]
    fn test_reduce_rank_binary_2_output_size_even() {
        let src = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
        let dst = reduce_rank_binary_2(&src, 1).unwrap();
        assert_eq!(dst.width(), 4);
        assert_eq!(dst.height(), 4);
    }

    #[test]
    fn test_reduce_rank_binary_2_output_size_odd() {
        // 奇数サイズは floor(n/2) に切り捨て
        let src = Pix::new(9, 7, PixelDepth::Bit1).unwrap();
        let dst = reduce_rank_binary_2(&src, 1).unwrap();
        assert_eq!(dst.width(), 4);
        assert_eq!(dst.height(), 3);
    }

    // ── level 1: any ON pixel in 2x2 block → OR ───────────────────────────

    #[test]
    fn test_reduce_rank_binary_2_level1_all_off() {
        #[rustfmt::skip]
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let src = make_1bpp(4, 4, &pixels);
        let dst = reduce_rank_binary_2(&src, 1).unwrap();
        assert_eq!(dst.get_pixel(0, 0), Some(0));
        assert_eq!(dst.get_pixel(1, 0), Some(0));
        assert_eq!(dst.get_pixel(0, 1), Some(0));
        assert_eq!(dst.get_pixel(1, 1), Some(0));
    }

    #[test]
    fn test_reduce_rank_binary_2_level1_one_on_per_block() {
        // 各 2x2 ブロックにちょうど 1 ピクセルが ON → レベル 1 では出力 ON
        #[rustfmt::skip]
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 1,
            0, 0, 0, 0,
        ];
        let src = make_1bpp(4, 4, &pixels);
        let dst = reduce_rank_binary_2(&src, 1).unwrap();
        assert_eq!(dst.get_pixel(0, 0), Some(1)); // ブロック(0,0): 1個ON
        assert_eq!(dst.get_pixel(1, 0), Some(0)); // ブロック(1,0): 0個ON
        assert_eq!(dst.get_pixel(0, 1), Some(0)); // ブロック(0,1): 0個ON
        assert_eq!(dst.get_pixel(1, 1), Some(1)); // ブロック(1,1): 1個ON
    }

    // ── level 4: all ON in 2x2 block → AND ───────────────────────────────

    #[test]
    fn test_reduce_rank_binary_2_level4_partial() {
        // 3 ピクセル ON の 2x2 ブロック → レベル 4 では出力 OFF
        #[rustfmt::skip]
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 0, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ];
        let src = make_1bpp(4, 4, &pixels);
        let dst = reduce_rank_binary_2(&src, 4).unwrap();
        assert_eq!(dst.get_pixel(0, 0), Some(0)); // 3個ON → レベル4でOFF
        assert_eq!(dst.get_pixel(1, 0), Some(1)); // 4個ON → ON
        assert_eq!(dst.get_pixel(0, 1), Some(1)); // 4個ON → ON
        assert_eq!(dst.get_pixel(1, 1), Some(1)); // 4個ON → ON
    }

    // ── level 2/3 中間値 ───────────────────────────────────────────────────

    #[test]
    fn test_reduce_rank_binary_2_level2() {
        // 2個ONのブロック: レベル2でON, レベル3でOFF
        #[rustfmt::skip]
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0,
            0, 0, 0, 0,
        ];
        let src = make_1bpp(4, 2, &pixels);
        let dst2 = reduce_rank_binary_2(&src, 2).unwrap();
        let dst3 = reduce_rank_binary_2(&src, 3).unwrap();
        assert_eq!(dst2.get_pixel(0, 0), Some(1)); // 2個ON ≥ 2 → ON
        assert_eq!(dst2.get_pixel(1, 0), Some(0)); // 0個ON → OFF
        assert_eq!(dst3.get_pixel(0, 0), Some(0)); // 2個ON < 3 → OFF
    }

    // ── エラーケース ──────────────────────────────────────────────────────

    #[test]
    fn test_reduce_rank_binary_2_invalid_depth() {
        let src = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        assert!(reduce_rank_binary_2(&src, 1).is_err());
    }

    #[test]
    fn test_reduce_rank_binary_2_invalid_level() {
        let src = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
        assert!(reduce_rank_binary_2(&src, 0).is_err());
        assert!(reduce_rank_binary_2(&src, 5).is_err());
    }

    // ── cascade ────────────────────────────────────────────────────────────

    #[test]
    fn test_cascade_r11_size() {
        // "r11": 2段階 level-1 縮小 → 16x16 → 4x4
        let src = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
        let dst = reduce_rank_binary_cascade(&src, &[1, 1]).unwrap();
        assert_eq!(dst.width(), 4);
        assert_eq!(dst.height(), 4);
    }

    #[test]
    fn test_cascade_r1143_size() {
        // "r1143": 4段階縮小 → 64x64 → 4x4
        let src = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let dst = reduce_rank_binary_cascade(&src, &[1, 1, 4, 3]).unwrap();
        assert_eq!(dst.width(), 4);
        assert_eq!(dst.height(), 4);
    }

    #[test]
    fn test_cascade_empty_levels() {
        // 空のlevels → 入力をclone
        let mut src = Pix::new(8, 8, PixelDepth::Bit1).unwrap().to_mut();
        src.set_pixel(2, 3, 1).unwrap();
        let src: Pix = src.into();
        let dst = reduce_rank_binary_cascade(&src, &[]).unwrap();
        assert_eq!(dst.width(), 8);
        assert_eq!(dst.height(), 8);
        assert_eq!(dst.get_pixel(2, 3), Some(1));
    }
}
