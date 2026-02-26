//! Binary image expansion (scaling up by replication)
//!
//! Provides optimized expansion of 1-bpp images using power-of-2 factors
//! and lookup tables for fast bit manipulation.
//!
//! # Reference
//!
//! Based on Leptonica's `binexpand.c`.

use crate::core::{Pix, PixelDepth};
use crate::transform::{TransformError, TransformResult};

/// Expand a 1-bpp image by replication with separate x and y factors.
///
/// Each source pixel is replicated `xfact × yfact` times. For power-of-2
/// factors where xfact == yfact, the optimized [`expand_binary_power2`] is
/// used internally.
///
/// # Arguments
///
/// * `pix` - 1-bpp source image
/// * `xfact` - Horizontal replication factor (≥ 1)
/// * `yfact` - Vertical replication factor (≥ 1)
///
/// # Reference
///
/// C Leptonica: `pixExpandBinaryReplicate()`
pub fn expand_binary_replicate(pix: &Pix, xfact: u32, yfact: u32) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(TransformError::UnsupportedDepth(
            "expand_binary_replicate requires 1-bpp".to_string(),
        ));
    }
    if xfact == 0 || yfact == 0 {
        return Err(TransformError::InvalidParameters(
            "expansion factors must be >= 1".to_string(),
        ));
    }
    if xfact == 1 && yfact == 1 {
        return Ok(pix.clone());
    }

    // Use optimized path for equal power-of-2 factors
    if xfact == yfact && xfact.is_power_of_two() && xfact <= 16 {
        return expand_binary_power2(pix, xfact);
    }

    let src_w = pix.width();
    let src_h = pix.height();
    let dst_w = src_w * xfact;
    let dst_h = src_h * yfact;

    let mut dst = Pix::new(dst_w, dst_h, PixelDepth::Bit1)
        .map_err(TransformError::Core)?
        .to_mut();

    for sy in 0..src_h {
        for sx in 0..src_w {
            if pix.get_pixel(sx, sy) == Some(1) {
                let dx_start = sx * xfact;
                let dy_start = sy * yfact;
                for dy in 0..yfact {
                    for dx in 0..xfact {
                        dst.set_pixel_unchecked(dx_start + dx, dy_start + dy, 1);
                    }
                }
            }
        }
    }

    Ok(dst.into())
}

/// Expand a 1-bpp image by a power-of-2 factor (2, 4, 8, or 16).
///
/// Uses lookup tables for very fast bit expansion.
///
/// # Arguments
///
/// * `pix` - 1-bpp source image
/// * `factor` - Expansion factor: 1, 2, 4, 8, or 16
///
/// # Reference
///
/// C Leptonica: `pixExpandBinaryPower2()`
pub fn expand_binary_power2(pix: &Pix, factor: u32) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(TransformError::UnsupportedDepth(
            "expand_binary_power2 requires 1-bpp".to_string(),
        ));
    }

    match factor {
        1 => Ok(pix.clone()),
        2 => expand_2x(pix),
        4 => expand_4x(pix),
        8 => expand_8x(pix),
        16 => expand_16x(pix),
        _ => Err(TransformError::InvalidParameters(format!(
            "factor must be 1, 2, 4, 8 or 16, got {factor}"
        ))),
    }
}

/// Create a 2x subsampling lookup table for binary reduction.
///
/// Maps each byte (8 pixels) to a nibble (4 pixels) using the specified
/// rank threshold. This is the inverse of expansion.
///
/// # Arguments
///
/// * `level` - Rank threshold (1-4): how many of each 2-pixel pair must be
///   ON for the output pixel to be ON
///
/// # Reference
///
/// C Leptonica: `makeSubsampleTab2x()`
pub fn make_subsample_tab_2x(level: u8) -> TransformResult<Vec<u8>> {
    if !(1..=4).contains(&level) {
        return Err(TransformError::InvalidParameters(format!(
            "level must be 1-4, got {level}"
        )));
    }

    // For each possible byte value (8 source bits), compute the 4-bit output
    let mut tab = vec![0u8; 256];
    for val in 0u16..256 {
        let mut out = 0u8;
        for bit in 0..4 {
            let shift = 6 - bit * 2;
            let pair = ((val >> shift) & 0x03) as u8;
            let count = (pair & 1) + ((pair >> 1) & 1);
            if count >= level {
                out |= 1 << (3 - bit);
            }
        }
        tab[val as usize] = out;
    }

    Ok(tab)
}

// ---------------------------------------------------------------------------
// Optimized expansion helpers using lookup tables
// ---------------------------------------------------------------------------

fn expand_2x(pix: &Pix) -> TransformResult<Pix> {
    let tab = make_expand_tab_2x();
    let src_w = pix.width();
    let src_h = pix.height();
    let dst_w = src_w * 2;
    let dst_h = src_h * 2;

    let mut dst = Pix::new(dst_w, dst_h, PixelDepth::Bit1)
        .map_err(TransformError::Core)?
        .to_mut();

    // Process row by row using word data
    let _src_wpl = pix.wpl();
    let dst_wpl = dst.wpl();

    for sy in 0..src_h {
        let src_row = pix.row_data(sy);
        let dy = sy * 2;

        // Build destination row from source words
        let mut dst_row = vec![0u32; dst_wpl as usize];
        for (sw_idx, &src_word) in src_row.iter().enumerate() {
            // Each source word has 32 bits → 64 destination bits → 2 destination words
            let dw_idx = sw_idx * 2;

            // Upper 16 bits → first dest word
            let byte3 = ((src_word >> 24) & 0xFF) as usize;
            let byte2 = ((src_word >> 16) & 0xFF) as usize;
            if dw_idx < dst_wpl as usize {
                dst_row[dw_idx] = (tab[byte3] as u32) << 16 | (tab[byte2] as u32);
            }

            // Lower 16 bits → second dest word
            let byte1 = ((src_word >> 8) & 0xFF) as usize;
            let byte0 = (src_word & 0xFF) as usize;
            if dw_idx + 1 < dst_wpl as usize {
                dst_row[dw_idx + 1] = (tab[byte1] as u32) << 16 | (tab[byte0] as u32);
            }
        }

        // Write both destination rows (they're identical for 2x vertical)
        let dst_data = dst.data_mut();
        let row_start_0 = (dy * dst_wpl) as usize;
        let row_start_1 = ((dy + 1) * dst_wpl) as usize;
        for (i, &val) in dst_row.iter().enumerate() {
            if row_start_0 + i < dst_data.len() {
                dst_data[row_start_0 + i] = val;
            }
            if row_start_1 + i < dst_data.len() {
                dst_data[row_start_1 + i] = val;
            }
        }
    }

    Ok(dst.into())
}

/// Create 2x expansion lookup table: each byte → 16 bits (u16)
fn make_expand_tab_2x() -> Vec<u16> {
    let mut tab = vec![0u16; 256];
    for val in 0u16..256 {
        let mut expanded = 0u16;
        for bit in 0..8 {
            if (val >> (7 - bit)) & 1 == 1 {
                expanded |= 0x03 << ((7 - bit) * 2);
            }
        }
        tab[val as usize] = expanded;
    }
    tab
}

fn expand_4x(pix: &Pix) -> TransformResult<Pix> {
    // 4x = 2x then 2x
    let tmp = expand_2x(pix)?;
    expand_2x(&tmp)
}

fn expand_8x(pix: &Pix) -> TransformResult<Pix> {
    // 8x = 4x then 2x
    let tmp = expand_4x(pix)?;
    expand_2x(&tmp)
}

fn expand_16x(pix: &Pix) -> TransformResult<Pix> {
    // 16x = 8x then 2x
    let tmp = expand_8x(pix)?;
    expand_2x(&tmp)
}
