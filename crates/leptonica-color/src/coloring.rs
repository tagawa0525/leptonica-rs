//! Coloring functions for grayscale and RGB images
//!
//! This module provides functions to colorize grayscale pixels, snap colors to
//! target values, and perform fractional color shifts.
//!
//! # Overview
//!
//! The coloring functions fall into several categories:
//!
//! 1. **Gray Colorization** ([`pix_color_gray`], [`pix_color_gray_masked`]):
//!    Colorize light or dark pixels while preserving antialiasing
//!
//! 2. **Color Snapping** ([`pix_snap_color`]):
//!    Force colors within a tolerance to a target color
//!
//! 3. **Linear Mapping** ([`pix_linear_map_to_target_color`]):
//!    Piecewise linear color transformation
//!
//! 4. **Component Shift** ([`pix_shift_by_component`]):
//!    Fractional shift toward black or white
//!
//! 5. **Hue-Invariant Mapping** ([`pix_map_with_invariant_hue`]):
//!    Change saturation/brightness while preserving hue
//!
//! # Examples
//!
//! ```no_run
//! use leptonica_color::coloring::{pix_color_gray, ColorGrayOptions, PaintType};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! // Colorize light pixels with red
//! let pix = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
//! let options = ColorGrayOptions {
//!     paint_type: PaintType::Light,
//!     threshold: 0,
//!     target_color: (255, 0, 0),
//! };
//! let colored = pix_color_gray(&pix, None, &options).unwrap();
//! ```

use crate::{ColorError, ColorResult};
use leptonica_core::{Box, Pix, PixelDepth, color};

/// Paint type for gray colorization
///
/// Determines whether to colorize light (non-black) or dark (non-white) pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaintType {
    /// Colorize light (non-black) pixels
    ///
    /// Pixels with average value above threshold are colorized.
    /// Black pixels remain black (preserving antialiasing).
    #[default]
    Light,
    /// Colorize dark (non-white) pixels
    ///
    /// Pixels with average value below threshold are colorized.
    /// White pixels remain white (preserving antialiasing).
    Dark,
}

/// Options for gray colorization operations
#[derive(Debug, Clone)]
pub struct ColorGrayOptions {
    /// Paint type: colorize light or dark pixels
    pub paint_type: PaintType,
    /// Threshold for colorization
    ///
    /// For `PaintType::Light`: pixels with average > threshold are colorized
    /// For `PaintType::Dark`: pixels with average < threshold are colorized
    pub threshold: u8,
    /// Target color (r, g, b)
    pub target_color: (u8, u8, u8),
}

impl Default for ColorGrayOptions {
    fn default() -> Self {
        Self {
            paint_type: PaintType::Light,
            threshold: 0,
            target_color: (255, 0, 0), // Red
        }
    }
}

// =============================================================================
// Pixel-level functions
// =============================================================================

/// Fractional shift of a pixel toward black or white
///
/// This linear transformation shifts each component a fraction toward either
/// black (`fract < 0`) or white (`fract > 0`).
///
/// # Arguments
///
/// * `r`, `g`, `b` - RGB components
/// * `fract` - Fraction in range [-1.0, 1.0]
///   - Negative: shift toward black (increases saturation, decreases brightness)
///   - Positive: shift toward white (decreases saturation, increases brightness)
///   - -1.0: result is black
///   - 1.0: result is white
///
/// # Returns
///
/// The shifted RGB values, or error if fract is out of range.
///
/// # Notes
///
/// This transformation preserves hue while changing saturation and brightness.
pub fn pixel_fractional_shift(r: u8, g: u8, b: u8, fract: f32) -> ColorResult<(u8, u8, u8)> {
    if !(-1.0..=1.0).contains(&fract) {
        return Err(ColorError::InvalidParameters(
            "fraction must be in range [-1.0, 1.0]".to_string(),
        ));
    }

    let (nr, ng, nb) = if fract < 0.0 {
        // Shift toward black
        let factor = 1.0 + fract;
        (
            (factor * r as f32 + 0.5) as u8,
            (factor * g as f32 + 0.5) as u8,
            (factor * b as f32 + 0.5) as u8,
        )
    } else {
        // Shift toward white
        (
            r + (fract * (255.0 - r as f32) + 0.5) as u8,
            g + (fract * (255.0 - g as f32) + 0.5) as u8,
            b + (fract * (255.0 - b as f32) + 0.5) as u8,
        )
    };

    Ok((nr, ng, nb))
}

/// Shift a pixel by component toward black or white
///
/// For each component separately, this does a linear mapping:
/// - If `dst_val <= src_val`: `val -> (dst_val/src_val) * val` (toward black)
/// - If `dst_val > src_val`: shift toward white proportionally
///
/// # Arguments
///
/// * `r`, `g`, `b` - RGB components
/// * `src_color` - Source color in 0xRRGGBB00 format
/// * `dst_color` - Destination color in 0xRRGGBB00 format
///
/// # Returns
///
/// The shifted RGB values.
pub fn pixel_shift_by_component(
    r: u8,
    g: u8,
    b: u8,
    src_color: u32,
    dst_color: u32,
) -> (u8, u8, u8) {
    let (rs, gs, bs) = extract_rgb_from_color(src_color);
    let (rd, gd, bd) = extract_rgb_from_color(dst_color);

    let nr = shift_component(r, rs, rd);
    let ng = shift_component(g, gs, gd);
    let nb = shift_component(b, bs, bd);

    (nr, ng, nb)
}

/// Helper to shift a single component
#[inline]
fn shift_component(val: u8, src: u8, dst: u8) -> u8 {
    if dst == src {
        val
    } else if dst < src {
        // Shift toward black
        if src == 0 {
            0
        } else {
            ((val as u32 * dst as u32) / src as u32) as u8
        }
    } else {
        // Shift toward white
        if src == 255 {
            val
        } else {
            255 - (((255 - dst) as u32 * (255 - val) as u32) / (255 - src) as u32) as u8
        }
    }
}

/// Linear map a single pixel from source to destination color
///
/// For each component (r, g, b) separately, this does a piecewise linear
/// mapping. If `src` and `dst` are the source and destination components:
/// - Range [0, src] maps to [0, dst]
/// - Range [src, 255] maps to [dst, 255]
///
/// # Arguments
///
/// * `pixel` - Input pixel in 32-bit RGBA format
/// * `src_map` - Source mapping color in 0xRRGGBB00 format
/// * `dst_map` - Destination mapping color in 0xRRGGBB00 format
///
/// # Returns
///
/// The mapped pixel in 32-bit RGBA format.
pub fn pixel_linear_map_to_target_color(pixel: u32, src_map: u32, dst_map: u32) -> u32 {
    let (r, g, b, a) = color::extract_rgba(pixel);
    let (sr, sg, sb) = extract_rgb_from_color(src_map);
    let (dr, dg, db) = extract_rgb_from_color(dst_map);

    // Clamp source values to [1, 254] to avoid division issues
    let sr = sr.clamp(1, 254);
    let sg = sg.clamp(1, 254);
    let sb = sb.clamp(1, 254);

    let nr = linear_map_component(r, sr, dr);
    let ng = linear_map_component(g, sg, dg);
    let nb = linear_map_component(b, sb, db);

    color::compose_rgba(nr, ng, nb, a)
}

/// Helper to perform linear mapping on a single component
#[inline]
fn linear_map_component(val: u8, src: u8, dst: u8) -> u8 {
    if val <= src {
        ((val as u32 * dst as u32) / src as u32) as u8
    } else {
        dst + (((255 - dst) as u32 * (val - src) as u32) / (255 - src) as u32) as u8
    }
}

/// Extract RGB components from 0xRRGGBB00 format color
#[inline]
fn extract_rgb_from_color(color: u32) -> (u8, u8, u8) {
    let r = ((color >> 24) & 0xff) as u8;
    let g = ((color >> 16) & 0xff) as u8;
    let b = ((color >> 8) & 0xff) as u8;
    (r, g, b)
}

/// Compose a 0xRRGGBB00 format color from RGB components
#[inline]
fn compose_color_from_rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8)
}

// =============================================================================
// Image-level functions
// =============================================================================

/// Colorize gray pixels in a 32-bit RGB image
///
/// This function colorizes pixels based on their gray level, preserving
/// antialiasing. The algorithm differs based on paint type:
///
/// - **Light**: Non-black pixels are colorized. A pixel with gray level `g`
///   gets color `(target_r * g / 255, target_g * g / 255, target_b * g / 255)`.
///   Black pixels (g=0) remain black.
///
/// - **Dark**: Non-white pixels are colorized. A pixel with gray level `g`
///   gets color `(target_r + (255-target_r)*g/255, ...)`.
///   White pixels (g=255) remain white.
///
/// # Arguments
///
/// * `pix` - Input 32-bit RGB image
/// * `region` - Optional bounding box to restrict operation
/// * `options` - Colorization options
///
/// # Returns
///
/// A new colorized image.
///
/// # Errors
///
/// Returns error if the image is not 32-bit.
pub fn pix_color_gray(
    pix: &Pix,
    region: Option<&Box>,
    options: &ColorGrayOptions,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    // Validate threshold
    match options.paint_type {
        PaintType::Light => {
            if options.threshold == 255 {
                return Err(ColorError::InvalidParameters(
                    "threshold must be < 255 for Light paint type".to_string(),
                ));
            }
        }
        PaintType::Dark => {
            if options.threshold == 0 {
                return Err(ColorError::InvalidParameters(
                    "threshold must be > 0 for Dark paint type".to_string(),
                ));
            }
        }
    }

    let w = pix.width();
    let h = pix.height();

    // Determine region bounds
    let (x1, y1, x2, y2) = if let Some(b) = region {
        let bx = b.x.max(0) as u32;
        let by = b.y.max(0) as u32;
        let bw = b.w as u32;
        let bh = b.h as u32;
        (bx, by, (bx + bw).min(w), (by + bh).min(h))
    } else {
        (0, 0, w, h)
    };

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    // Copy original and then modify region
    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };

            let new_pixel = if x >= x1 && x < x2 && y >= y1 && y < y2 {
                colorize_pixel(pixel, options)
            } else {
                pixel
            };

            unsafe { out_mut.set_pixel_unchecked(x, y, new_pixel) };
        }
    }

    Ok(out_mut.into())
}

/// Colorize gray pixels under a mask
///
/// Similar to [`pix_color_gray`], but only pixels under foreground (1) pixels
/// of the mask are colorized.
///
/// # Arguments
///
/// * `pix` - Input 32-bit RGB image
/// * `mask` - 1-bit mask image
/// * `options` - Colorization options
///
/// # Returns
///
/// A new colorized image.
///
/// # Errors
///
/// Returns error if the image is not 32-bit or mask is not 1-bit.
pub fn pix_color_gray_masked(
    pix: &Pix,
    mask: &Pix,
    options: &ColorGrayOptions,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    if mask.depth() != PixelDepth::Bit1 {
        return Err(ColorError::UnsupportedDepth {
            expected: "1 bpp mask",
            actual: mask.depth().bits(),
        });
    }

    // Validate threshold
    match options.paint_type {
        PaintType::Light => {
            if options.threshold == 255 {
                return Err(ColorError::InvalidParameters(
                    "threshold must be < 255 for Light paint type".to_string(),
                ));
            }
        }
        PaintType::Dark => {
            if options.threshold == 0 {
                return Err(ColorError::InvalidParameters(
                    "threshold must be > 0 for Dark paint type".to_string(),
                ));
            }
        }
    }

    let w = pix.width();
    let h = pix.height();
    let mask_w = mask.width();
    let mask_h = mask.height();

    let w_min = w.min(mask_w);
    let h_min = h.min(mask_h);

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };

            let new_pixel = if x < w_min && y < h_min {
                let mask_val = unsafe { mask.get_pixel_unchecked(x, y) };
                if mask_val != 0 {
                    colorize_pixel(pixel, options)
                } else {
                    pixel
                }
            } else {
                pixel
            };

            unsafe { out_mut.set_pixel_unchecked(x, y, new_pixel) };
        }
    }

    Ok(out_mut.into())
}

/// Helper to colorize a single pixel
#[inline]
fn colorize_pixel(pixel: u32, options: &ColorGrayOptions) -> u32 {
    let (r, g, b, a) = color::extract_rgba(pixel);
    let (tr, tg, tb) = options.target_color;

    // Calculate average gray value
    let avg = ((r as u32 + g as u32 + b as u32) / 3) as u8;

    match options.paint_type {
        PaintType::Light => {
            if avg <= options.threshold {
                // Skip dark pixels
                return pixel;
            }
            // Colorize: new_color = target_color * avg / 255
            let factor = avg as f32 / 255.0;
            let nr = (tr as f32 * factor) as u8;
            let ng = (tg as f32 * factor) as u8;
            let nb = (tb as f32 * factor) as u8;
            color::compose_rgba(nr, ng, nb, a)
        }
        PaintType::Dark => {
            if avg >= options.threshold {
                // Skip light pixels
                return pixel;
            }
            // Colorize: new_color = target_color + (255 - target_color) * avg / 255
            let factor = avg as f32 / 255.0;
            let nr = tr + ((255.0 - tr as f32) * factor) as u8;
            let ng = tg + ((255.0 - tg as f32) * factor) as u8;
            let nb = tb + ((255.0 - tb as f32) * factor) as u8;
            color::compose_rgba(nr, ng, nb, a)
        }
    }
}

/// Snap colors within a tolerance to a target color
///
/// All pixels whose color is within `diff` of `src_color` (componentwise)
/// are set to `dst_color`.
///
/// # Arguments
///
/// * `pix` - Input 8-bit grayscale or 32-bit RGB image
/// * `src_color` - Source color center in 0xRRGGBB00 format (for 8-bit, only low byte is used)
/// * `dst_color` - Target color in 0xRRGGBB00 format
/// * `diff` - Maximum absolute difference per component
///
/// # Returns
///
/// A new image with snapped colors.
pub fn pix_snap_color(pix: &Pix, src_color: u32, dst_color: u32, diff: u8) -> ColorResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit8 => pix_snap_color_8bpp(pix, src_color, dst_color, diff),
        PixelDepth::Bit32 => pix_snap_color_32bpp(pix, src_color, dst_color, diff),
        _ => Err(ColorError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

fn pix_snap_color_8bpp(pix: &Pix, src_color: u32, dst_color: u32, diff: u8) -> ColorResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let src_val = (src_color & 0xff) as u8;
    let dst_val = (dst_color & 0xff) as u8;

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val = unsafe { pix.get_pixel_unchecked(x, y) } as u8;
            let new_val = if (val as i16 - src_val as i16).unsigned_abs() as u8 <= diff {
                dst_val
            } else {
                val
            };
            unsafe { out_mut.set_pixel_unchecked(x, y, new_val as u32) };
        }
    }

    Ok(out_mut.into())
}

fn pix_snap_color_32bpp(pix: &Pix, src_color: u32, dst_color: u32, diff: u8) -> ColorResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let (sr, sg, sb) = extract_rgb_from_color(src_color);

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
            let (r, g, b, a) = color::extract_rgba(pixel);

            let new_pixel = if (r as i16 - sr as i16).unsigned_abs() as u8 <= diff
                && (g as i16 - sg as i16).unsigned_abs() as u8 <= diff
                && (b as i16 - sb as i16).unsigned_abs() as u8 <= diff
            {
                // Extract RGB from dst_color and compose with original alpha
                let (dr, dg, db) = extract_rgb_from_color(dst_color);
                color::compose_rgba(dr, dg, db, a)
            } else {
                pixel
            };

            unsafe { out_mut.set_pixel_unchecked(x, y, new_pixel) };
        }
    }

    Ok(out_mut.into())
}

/// Piecewise linear color mapping from source to target
///
/// For each component (r, g, b) separately, this does a piecewise linear
/// mapping:
/// - Range [0, src] maps to [0, dst]
/// - Range [src, 255] maps to [dst, 255]
///
/// This mapping will generally change the hue of the pixels. However, if
/// the source and target are related by a fractional shift, the hue is
/// preserved.
///
/// # Arguments
///
/// * `pix` - Input 32-bit RGB image
/// * `src_color` - Source mapping color in 0xRRGGBB00 format
/// * `dst_color` - Target mapping color in 0xRRGGBB00 format
///
/// # Returns
///
/// A new image with mapped colors.
pub fn pix_linear_map_to_target_color(
    pix: &Pix,
    src_color: u32,
    dst_color: u32,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();

    // Build lookup tables for each component
    let (sr, sg, sb) = extract_rgb_from_color(src_color);
    let (dr, dg, db) = extract_rgb_from_color(dst_color);

    // Clamp source values to [1, 254]
    let sr = sr.clamp(1, 254);
    let sg = sg.clamp(1, 254);
    let sb = sb.clamp(1, 254);

    let mut rtab = [0u8; 256];
    let mut gtab = [0u8; 256];
    let mut btab = [0u8; 256];

    for i in 0..256 {
        rtab[i] = linear_map_component(i as u8, sr, dr);
        gtab[i] = linear_map_component(i as u8, sg, dg);
        btab[i] = linear_map_component(i as u8, sb, db);
    }

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
            let (r, g, b, a) = color::extract_rgba(pixel);

            let nr = rtab[r as usize];
            let ng = gtab[g as usize];
            let nb = btab[b as usize];

            let new_pixel = color::compose_rgba(nr, ng, nb, a);
            unsafe { out_mut.set_pixel_unchecked(x, y, new_pixel) };
        }
    }

    Ok(out_mut.into())
}

/// Fractional shift of RGB toward black or white
///
/// For each component separately, this does a linear mapping:
/// - If `dst <= src`: shift toward black (`val -> (dst/src) * val`)
/// - If `dst > src`: shift toward white
///
/// This is essentially a different linear TRC (gamma = 1) for each component.
/// The source and target color inputs define the three fractions.
///
/// # Arguments
///
/// * `pix` - Input 32-bit RGB image
/// * `src_color` - Source color in 0xRRGGBB00 format
/// * `dst_color` - Target color in 0xRRGGBB00 format
///
/// # Returns
///
/// A new image with shifted colors.
///
/// # Example
///
/// To color a light background, use `src_color = 0xffffff00` and pick a
/// target background color for `dst_color`.
pub fn pix_shift_by_component(pix: &Pix, src_color: u32, dst_color: u32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();

    // Build lookup tables for each component
    let (sr, sg, sb) = extract_rgb_from_color(src_color);
    let (dr, dg, db) = extract_rgb_from_color(dst_color);

    let mut rtab = [0u8; 256];
    let mut gtab = [0u8; 256];
    let mut btab = [0u8; 256];

    for i in 0..256 {
        rtab[i] = shift_component(i as u8, sr, dr);
        gtab[i] = shift_component(i as u8, sg, dg);
        btab[i] = shift_component(i as u8, sb, db);
    }

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
            let (r, g, b, a) = color::extract_rgba(pixel);

            let nr = rtab[r as usize];
            let ng = gtab[g as usize];
            let nb = btab[b as usize];

            let new_pixel = color::compose_rgba(nr, ng, nb, a);
            unsafe { out_mut.set_pixel_unchecked(x, y, new_pixel) };
        }
    }

    Ok(out_mut.into())
}

/// Map colors with invariant hue
///
/// This transformation changes saturation and brightness while preserving hue.
/// The combination of `src_color` and `fract` defines the linear transformation.
///
/// # Arguments
///
/// * `pix` - Input 32-bit RGB image
/// * `src_color` - Reference source color in 0xRRGGBB00 format
/// * `fract` - Fraction in range [-1.0, 1.0]
///   - Negative: increase saturation, decrease brightness
///   - Positive: decrease saturation, increase brightness
///   - -1.0: `src_color` maps to black
///   - 1.0: `src_color` maps to white
///
/// # Returns
///
/// A new image with mapped colors.
pub fn pix_map_with_invariant_hue(pix: &Pix, src_color: u32, fract: f32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    if !(-1.0..=1.0).contains(&fract) {
        return Err(ColorError::InvalidParameters(
            "fraction must be in range [-1.0, 1.0]".to_string(),
        ));
    }

    // Generate the destination color that is fract toward white from src_color
    let (r, g, b) = extract_rgb_from_color(src_color);
    let (dr, dg, db) = pixel_fractional_shift(r, g, b, fract)?;
    let dst_color = compose_color_from_rgb(dr, dg, db);

    // Use the (src_color, dst_color) pair to define the linear transform
    pix_linear_map_to_target_color(pix, src_color, dst_color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_fractional_shift_to_black() {
        // fract = -1.0 should give black
        let (r, g, b) = pixel_fractional_shift(200, 150, 100, -1.0).unwrap();
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn test_pixel_fractional_shift_to_white() {
        // fract = 1.0 should give white
        let (r, g, b) = pixel_fractional_shift(100, 150, 200, 1.0).unwrap();
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_pixel_fractional_shift_no_change() {
        // fract = 0 should give no change
        let (r, g, b) = pixel_fractional_shift(100, 150, 200, 0.0).unwrap();
        assert_eq!((r, g, b), (100, 150, 200));
    }

    #[test]
    fn test_pixel_fractional_shift_invalid_fract() {
        assert!(pixel_fractional_shift(100, 150, 200, 1.5).is_err());
        assert!(pixel_fractional_shift(100, 150, 200, -1.5).is_err());
    }

    #[test]
    fn test_shift_component() {
        // dst == src: no change
        assert_eq!(shift_component(100, 128, 128), 100);

        // dst < src: shift toward black
        assert_eq!(shift_component(200, 255, 128), 100); // 200 * 128 / 255 ≈ 100

        // dst > src: shift toward white
        // 255 - ((255-200) * (255-200)) / (255-128) = 255 - 55*55/127 ≈ 231
        let result = shift_component(200, 128, 200);
        assert!(result > 200); // Should be shifted toward white
    }

    #[test]
    fn test_linear_map_component() {
        // Map 128 with src=128, dst=64 -> 64
        assert_eq!(linear_map_component(128, 128, 64), 64);

        // Map 0 should stay 0
        assert_eq!(linear_map_component(0, 128, 200), 0);

        // Map 255 should stay 255
        assert_eq!(linear_map_component(255, 128, 200), 255);
    }

    #[test]
    fn test_extract_rgb_from_color() {
        // 0xRRGGBB00 format
        let (r, g, b) = extract_rgb_from_color(0xFF804000);
        assert_eq!((r, g, b), (255, 128, 64));
    }

    #[test]
    fn test_compose_color_from_rgb() {
        let color = compose_color_from_rgb(255, 128, 64);
        assert_eq!(color, 0xFF804000);
    }

    #[test]
    fn test_pix_color_gray_light() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with gray
        for y in 0..10 {
            for x in 0..10 {
                let pixel = color::compose_rgb(128, 128, 128);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        let options = ColorGrayOptions {
            paint_type: PaintType::Light,
            threshold: 0,
            target_color: (255, 0, 0), // Red
        };

        let result = pix_color_gray(&pix_mut.into(), None, &options).unwrap();

        // Check that pixels are now reddish
        let pixel = unsafe { result.get_pixel_unchecked(5, 5) };
        let (r, g, b, _) = color::extract_rgba(pixel);

        // Red should be dominant
        assert!(r > g);
        assert!(r > b);
        // Should be about 128 (half intensity red)
        assert!((r as i32 - 128).abs() < 10);
    }

    #[test]
    fn test_pix_color_gray_dark() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with dark gray
        for y in 0..10 {
            for x in 0..10 {
                let pixel = color::compose_rgb(64, 64, 64);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        let options = ColorGrayOptions {
            paint_type: PaintType::Dark,
            threshold: 255,
            target_color: (0, 0, 255), // Blue
        };

        let result = pix_color_gray(&pix_mut.into(), None, &options).unwrap();

        // Check that pixels are now bluish
        let pixel = unsafe { result.get_pixel_unchecked(5, 5) };
        let (r, g, b, _) = color::extract_rgba(pixel);

        // Blue should be dominant
        assert!(b > r);
        assert!(b > g);
    }

    #[test]
    fn test_pix_snap_color_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with near-white
        for y in 0..10 {
            for x in 0..10 {
                let pixel = color::compose_rgb(250, 252, 248);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        // Snap near-white to pure white
        let result = pix_snap_color(&pix_mut.into(), 0xFFFFFF00, 0xFFFFFF00, 10).unwrap();

        let pixel = unsafe { result.get_pixel_unchecked(5, 5) };
        let (r, g, b, _) = color::extract_rgba(pixel);

        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_pix_snap_color_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with value 250
        for y in 0..10 {
            for x in 0..10 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, 250) };
            }
        }

        // Snap 250 to 255 (within diff=10)
        let result = pix_snap_color(&pix_mut.into(), 0x000000FF, 0x000000FF, 10).unwrap();

        let val = unsafe { result.get_pixel_unchecked(5, 5) } as u8;
        assert_eq!(val, 255);
    }

    #[test]
    fn test_pix_linear_map_to_target_color() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with mid-gray
        for y in 0..10 {
            for x in 0..10 {
                let pixel = color::compose_rgb(128, 128, 128);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        // Map mid-gray (128) to darker (64)
        let result =
            pix_linear_map_to_target_color(&pix_mut.into(), 0x80808000, 0x40404000).unwrap();

        let pixel = unsafe { result.get_pixel_unchecked(5, 5) };
        let (r, g, b, _) = color::extract_rgba(pixel);

        // Should be around 64
        assert_eq!(r, 64);
        assert_eq!(g, 64);
        assert_eq!(b, 64);
    }

    #[test]
    fn test_pix_shift_by_component() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with white
        for y in 0..10 {
            for x in 0..10 {
                let pixel = color::compose_rgb(255, 255, 255);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        // Shift white toward a pink color
        let result = pix_shift_by_component(&pix_mut.into(), 0xFFFFFF00, 0xFF808000).unwrap();

        let pixel = unsafe { result.get_pixel_unchecked(5, 5) };
        let (r, g, b, _) = color::extract_rgba(pixel);

        // Red should stay 255, green and blue should be shifted down
        assert_eq!(r, 255);
        assert!(g < 255);
        assert!(b < 255);
    }

    #[test]
    fn test_pix_map_with_invariant_hue() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with a color
        for y in 0..10 {
            for x in 0..10 {
                let pixel = color::compose_rgb(200, 100, 50);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        // Map toward white (positive fract)
        let result = pix_map_with_invariant_hue(&pix_mut.into(), 0xC8643200, 0.5).unwrap();

        let pixel = unsafe { result.get_pixel_unchecked(5, 5) };
        let (r, g, b, _) = color::extract_rgba(pixel);

        // All components should be higher (shifted toward white)
        assert!(r > 200 || (r == 255));
        assert!(g > 100);
        assert!(b > 50);
    }

    #[test]
    fn test_color_gray_threshold_validation() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();

        // Light paint type with threshold >= 255 should fail
        let options = ColorGrayOptions {
            paint_type: PaintType::Light,
            threshold: 255,
            target_color: (255, 0, 0),
        };
        assert!(pix_color_gray(&pix, None, &options).is_err());

        // Dark paint type with threshold == 0 should fail
        let options = ColorGrayOptions {
            paint_type: PaintType::Dark,
            threshold: 0,
            target_color: (255, 0, 0),
        };
        assert!(pix_color_gray(&pix, None, &options).is_err());
    }
}
