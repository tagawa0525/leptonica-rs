//! Shear transformations for images
//!
//! This module provides shear transformation operations including:
//! - Horizontal shear (about an arbitrary horizontal line)
//! - Vertical shear (about an arbitrary vertical line)
//! - Convenience functions for shearing about corners and centers
//! - In-place shear operations
//! - Linear interpolated shear for high-quality results
//!
//! # Shear Transformation
//!
//! A shear transformation skews an image along one axis. The transformation
//! leaves one line (horizontal or vertical) invariant while shifting other
//! pixels proportionally to their distance from that line.
//!
//! ## Horizontal Shear
//!
//! For a horizontal shear about y = yloc:
//! - Pixels at y = yloc remain unchanged
//! - Pixels above yloc shift right (positive angle) or left (negative angle)
//! - Pixels below yloc shift in the opposite direction
//! - The shift amount is `tan(angle) * (yloc - y)`
//!
//! ## Vertical Shear
//!
//! For a vertical shear about x = xloc:
//! - Pixels at x = xloc remain unchanged
//! - Pixels right of xloc shift down (positive angle) or up (negative angle)
//! - Pixels left of xloc shift in the opposite direction
//! - The shift amount is `tan(angle) * (x - xloc)`
//!
//! # Example
//!
//! ```no_run
//! use leptonica_transform::{h_shear, v_shear, ShearFill};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
//!
//! // Horizontal shear about the center line
//! let sheared = h_shear(&pix, 50, 0.1, ShearFill::White).unwrap();
//!
//! // Vertical shear about the left edge
//! let sheared = v_shear(&pix, 0, 0.1, ShearFill::White).unwrap();
//! ```

use crate::{TransformError, TransformResult};
use leptonica_core::{Pix, PixMut, PixelDepth, color};

// ============================================================================
// Constants
// ============================================================================

/// Minimum difference from +-pi/2 for shear angles
/// Shear angle must not get too close to -pi/2 or pi/2 (would require infinite shift)
const MIN_DIFF_FROM_HALF_PI: f32 = 0.04;

// ============================================================================
// Types
// ============================================================================

/// Background fill color for shear transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShearFill {
    /// Fill with white pixels (L_BRING_IN_WHITE in Leptonica)
    #[default]
    White,
    /// Fill with black pixels (L_BRING_IN_BLACK in Leptonica)
    Black,
}

impl ShearFill {
    /// Get the fill value for a specific pixel depth
    pub fn to_value(self, depth: PixelDepth) -> u32 {
        match self {
            ShearFill::White => match depth {
                PixelDepth::Bit1 => 0, // 0 = white for binary (foreground is black)
                PixelDepth::Bit2 => 3,
                PixelDepth::Bit4 => 15,
                PixelDepth::Bit8 => 255,
                PixelDepth::Bit16 => 65535,
                PixelDepth::Bit32 => 0xFFFFFF00,
            },
            ShearFill::Black => match depth {
                PixelDepth::Bit1 => 1, // 1 = black for binary
                PixelDepth::Bit32 => 0x00000000,
                _ => 0,
            },
        }
    }
}

// ============================================================================
// Angle Normalization
// ============================================================================

/// Normalize angle for shear operations
///
/// Brings the angle into the range [-pi/2 + mindif, pi/2 - mindif].
/// Returns None if the angle is effectively zero (no shear needed).
fn normalize_angle_for_shear(mut radang: f32, mindif: f32) -> Option<f32> {
    let pi2 = std::f32::consts::FRAC_PI_2;

    // Bring angle into range [-pi/2, pi/2]
    if radang < -pi2 || radang > pi2 {
        radang -= (radang / pi2).trunc() * pi2;
    }

    // If angle is too close to pi/2 or -pi/2, clamp it
    if radang > pi2 - mindif {
        radang = pi2 - mindif;
    } else if radang < -pi2 + mindif {
        radang = -pi2 + mindif;
    }

    // If angle is effectively zero, return None
    if radang.abs() < 1e-7 || radang.tan().abs() < 1e-7 {
        return None;
    }

    Some(radang)
}

// ============================================================================
// Horizontal Shear
// ============================================================================

/// Horizontal shear transformation about an arbitrary horizontal line
///
/// This is equivalent to Leptonica's `pixHShear`.
///
/// # Arguments
/// * `pix` - Input image (any depth)
/// * `yloc` - Y-coordinate of the invariant horizontal line
/// * `radang` - Shear angle in radians (must not be too close to +-pi/2)
/// * `fill` - Background fill color for pixels brought in from outside
///
/// # Returns
/// A new sheared image
///
/// # Notes
/// - Pixels on the line y = yloc remain unchanged
/// - For positive angles, pixels above this line shift right, below shift left
/// - The angle must be within approximately [-1.53, 1.53] radians
///
/// # Example
/// ```no_run
/// use leptonica_transform::{h_shear, ShearFill};
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
/// let sheared = h_shear(&pix, 50, 0.1, ShearFill::White).unwrap();
/// ```
pub fn h_shear(pix: &Pix, yloc: i32, radang: f32, fill: ShearFill) -> TransformResult<Pix> {
    // Normalize angle and check if shear is needed
    let radang = match normalize_angle_for_shear(radang, MIN_DIFF_FROM_HALF_PI) {
        Some(a) => a,
        None => return Ok(pix.deep_clone()),
    };

    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();
    let fill_value = fill.to_value(depth);

    // Create output image
    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Copy colormap if present
    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    // Perform horizontal shear
    let tan_angle = radang.tan();
    let wi = w as i32;
    let hi = h as i32;

    for y in 0..hi {
        // Calculate horizontal shift for this row
        // Positive angle: rows above yloc shift right (positive), below shift left (negative)
        let shift = ((yloc - y) as f32 * tan_angle).round() as i32;

        for x in 0..wi {
            let src_x = x - shift;
            if src_x >= 0 && src_x < wi {
                let val = unsafe { pix.get_pixel_unchecked(src_x as u32, y as u32) };
                unsafe { out_mut.set_pixel_unchecked(x as u32, y as u32, val) };
            }
            // Pixels outside source keep the fill value
        }
    }

    Ok(out_mut.into())
}

/// Horizontal shear about the upper-left corner (y = 0)
///
/// This is equivalent to Leptonica's `pixHShearCorner`.
///
/// # Arguments
/// * `pix` - Input image
/// * `radang` - Shear angle in radians
/// * `fill` - Background fill color
pub fn h_shear_corner(pix: &Pix, radang: f32, fill: ShearFill) -> TransformResult<Pix> {
    h_shear(pix, 0, radang, fill)
}

/// Horizontal shear about the center (y = height/2)
///
/// This is equivalent to Leptonica's `pixHShearCenter`.
///
/// # Arguments
/// * `pix` - Input image
/// * `radang` - Shear angle in radians
/// * `fill` - Background fill color
pub fn h_shear_center(pix: &Pix, radang: f32, fill: ShearFill) -> TransformResult<Pix> {
    let yloc = (pix.height() / 2) as i32;
    h_shear(pix, yloc, radang, fill)
}

// ============================================================================
// Vertical Shear
// ============================================================================

/// Vertical shear transformation about an arbitrary vertical line
///
/// This is equivalent to Leptonica's `pixVShear`.
///
/// # Arguments
/// * `pix` - Input image (any depth)
/// * `xloc` - X-coordinate of the invariant vertical line
/// * `radang` - Shear angle in radians (must not be too close to +-pi/2)
/// * `fill` - Background fill color for pixels brought in from outside
///
/// # Returns
/// A new sheared image
///
/// # Notes
/// - Pixels on the line x = xloc remain unchanged
/// - For positive angles, pixels right of this line shift down, left shift up
/// - The angle must be within approximately [-1.53, 1.53] radians
pub fn v_shear(pix: &Pix, xloc: i32, radang: f32, fill: ShearFill) -> TransformResult<Pix> {
    // Normalize angle and check if shear is needed
    let radang = match normalize_angle_for_shear(radang, MIN_DIFF_FROM_HALF_PI) {
        Some(a) => a,
        None => return Ok(pix.deep_clone()),
    };

    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();
    let fill_value = fill.to_value(depth);

    // Create output image
    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Copy colormap if present
    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    // Perform vertical shear
    let tan_angle = radang.tan();
    let wi = w as i32;
    let hi = h as i32;

    for x in 0..wi {
        // Calculate vertical shift for this column
        // Positive angle: columns right of xloc shift down (positive), left shift up (negative)
        let shift = ((x - xloc) as f32 * tan_angle).round() as i32;

        for y in 0..hi {
            let src_y = y - shift;
            if src_y >= 0 && src_y < hi {
                let val = unsafe { pix.get_pixel_unchecked(x as u32, src_y as u32) };
                unsafe { out_mut.set_pixel_unchecked(x as u32, y as u32, val) };
            }
            // Pixels outside source keep the fill value
        }
    }

    Ok(out_mut.into())
}

/// Vertical shear about the upper-left corner (x = 0)
///
/// This is equivalent to Leptonica's `pixVShearCorner`.
///
/// # Arguments
/// * `pix` - Input image
/// * `radang` - Shear angle in radians
/// * `fill` - Background fill color
pub fn v_shear_corner(pix: &Pix, radang: f32, fill: ShearFill) -> TransformResult<Pix> {
    v_shear(pix, 0, radang, fill)
}

/// Vertical shear about the center (x = width/2)
///
/// This is equivalent to Leptonica's `pixVShearCenter`.
///
/// # Arguments
/// * `pix` - Input image
/// * `radang` - Shear angle in radians
/// * `fill` - Background fill color
pub fn v_shear_center(pix: &Pix, radang: f32, fill: ShearFill) -> TransformResult<Pix> {
    let xloc = (pix.width() / 2) as i32;
    v_shear(pix, xloc, radang, fill)
}

// ============================================================================
// In-place Shear
// ============================================================================

/// In-place horizontal shear transformation
///
/// This is equivalent to Leptonica's `pixHShearIP`.
///
/// # Arguments
/// * `pix` - Image to shear in place (must not have colormap)
/// * `yloc` - Y-coordinate of the invariant horizontal line
/// * `radang` - Shear angle in radians
/// * `fill` - Background fill color
///
/// # Notes
/// - In-place shear cannot work with colormapped images because we can only
///   blit 0 or 1 bits for the background, not arbitrary colormap indices.
/// - The caller must ensure the image does not have a colormap.
///   Use `h_shear` instead if the image might have a colormap.
pub fn h_shear_ip(
    pix: &mut PixMut,
    yloc: i32,
    radang: f32,
    fill: ShearFill,
) -> TransformResult<()> {
    // Note: PixMut does not provide colormap() method, so we cannot check here.
    // The caller must ensure the image does not have a colormap.
    // If needed, use h_shear (non-in-place) which handles colormaps correctly.

    // Normalize angle and check if shear is needed
    let radang = match normalize_angle_for_shear(radang, MIN_DIFF_FROM_HALF_PI) {
        Some(a) => a,
        None => return Ok(()), // No shear needed
    };

    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();
    let fill_value = fill.to_value(depth);

    let tan_angle = radang.tan();
    let wi = w as i32;
    let hi = h as i32;

    // Process rows in order of shift direction to avoid overwriting
    // For positive tan_angle: rows above yloc shift right
    // We need to process in a way that doesn't overwrite unread pixels

    // Since we're shifting rows, we can process each row independently
    // using a temporary buffer for the row
    let mut row_buffer = vec![fill_value; w as usize];

    for y in 0..hi {
        let shift = ((yloc - y) as f32 * tan_angle).round() as i32;

        // Read original row into buffer
        for x in 0..wi {
            row_buffer[x as usize] = unsafe { pix.get_pixel_unchecked(x as u32, y as u32) };
        }

        // Write shifted row
        for x in 0..wi {
            let src_x = x - shift;
            let val = if src_x >= 0 && src_x < wi {
                row_buffer[src_x as usize]
            } else {
                fill_value
            };
            unsafe { pix.set_pixel_unchecked(x as u32, y as u32, val) };
        }
    }

    Ok(())
}

/// In-place vertical shear transformation
///
/// This is equivalent to Leptonica's `pixVShearIP`.
///
/// # Arguments
/// * `pix` - Image to shear in place (must not have colormap)
/// * `xloc` - X-coordinate of the invariant vertical line
/// * `radang` - Shear angle in radians
/// * `fill` - Background fill color
///
/// # Notes
/// - In-place shear cannot work with colormapped images because we can only
///   blit 0 or 1 bits for the background, not arbitrary colormap indices.
/// - The caller must ensure the image does not have a colormap.
///   Use `v_shear` instead if the image might have a colormap.
pub fn v_shear_ip(
    pix: &mut PixMut,
    xloc: i32,
    radang: f32,
    fill: ShearFill,
) -> TransformResult<()> {
    // Note: PixMut does not provide colormap() method, so we cannot check here.
    // The caller must ensure the image does not have a colormap.
    // If needed, use v_shear (non-in-place) which handles colormaps correctly.

    // Normalize angle and check if shear is needed
    let radang = match normalize_angle_for_shear(radang, MIN_DIFF_FROM_HALF_PI) {
        Some(a) => a,
        None => return Ok(()), // No shear needed
    };

    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();
    let fill_value = fill.to_value(depth);

    let tan_angle = radang.tan();
    let wi = w as i32;
    let hi = h as i32;

    // Process columns using a temporary buffer
    let mut col_buffer = vec![fill_value; h as usize];

    for x in 0..wi {
        let shift = ((x - xloc) as f32 * tan_angle).round() as i32;

        // Read original column into buffer
        for y in 0..hi {
            col_buffer[y as usize] = unsafe { pix.get_pixel_unchecked(x as u32, y as u32) };
        }

        // Write shifted column
        for y in 0..hi {
            let src_y = y - shift;
            let val = if src_y >= 0 && src_y < hi {
                col_buffer[src_y as usize]
            } else {
                fill_value
            };
            unsafe { pix.set_pixel_unchecked(x as u32, y as u32, val) };
        }
    }

    Ok(())
}

// ============================================================================
// Linear Interpolated Shear
// ============================================================================

/// Horizontal shear with linear interpolation
///
/// This is equivalent to Leptonica's `pixHShearLI`.
/// Provides high-quality results using bilinear interpolation.
///
/// # Arguments
/// * `pix` - Input image (8bpp grayscale or 32bpp color, or colormapped)
/// * `yloc` - Y-coordinate of the invariant horizontal line (must be in [0, h-1])
/// * `radang` - Shear angle in radians
/// * `fill` - Background fill color
///
/// # Returns
/// A new sheared image
///
/// # Notes
/// - Works only with 8bpp, 32bpp, or colormapped images
/// - Colormaps are removed and converted based on source
/// - Uses 64-subdivision interpolation for sub-pixel accuracy
///
/// # Errors
/// Returns `TransformError::UnsupportedDepth` for unsupported pixel depths.
/// Returns `TransformError::InvalidParameters` if yloc is out of bounds.
pub fn h_shear_li(pix: &Pix, yloc: i32, radang: f32, fill: ShearFill) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    // Validate yloc
    if yloc < 0 || yloc >= h as i32 {
        return Err(TransformError::InvalidParameters(format!(
            "yloc {} is out of bounds [0, {})",
            yloc, h
        )));
    }

    // Check depth
    if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 && pix.colormap().is_none() {
        return Err(TransformError::UnsupportedDepth(
            "h_shear_li requires 8bpp, 32bpp, or colormapped image".to_string(),
        ));
    }

    // Remove colormap if present
    let src_pix = if pix.colormap().is_some() {
        remove_colormap(pix)?
    } else {
        pix.deep_clone()
    };

    // Normalize angle and check if shear is needed
    let radang = match normalize_angle_for_shear(radang, MIN_DIFF_FROM_HALF_PI) {
        Some(a) => a,
        None => return Ok(src_pix),
    };

    let src_depth = src_pix.depth();
    let fill_value = fill.to_value(src_depth);

    // Create output image
    let out_pix = Pix::new(w, h, src_depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    let tan_angle = radang.tan();
    let wi = w as i32;
    let hi = h as i32;
    let wm = wi - 1;

    match src_depth {
        PixelDepth::Bit8 => {
            h_shear_li_gray(
                &src_pix,
                &mut out_mut,
                yloc,
                tan_angle,
                wi,
                hi,
                wm,
                fill_value as u8,
            );
        }
        PixelDepth::Bit32 => {
            h_shear_li_color(
                &src_pix,
                &mut out_mut,
                yloc,
                tan_angle,
                wi,
                hi,
                wm,
                fill_value,
            );
        }
        _ => unreachable!(),
    }

    Ok(out_mut.into())
}

/// Vertical shear with linear interpolation
///
/// This is equivalent to Leptonica's `pixVShearLI`.
/// Provides high-quality results using bilinear interpolation.
///
/// # Arguments
/// * `pix` - Input image (8bpp grayscale or 32bpp color, or colormapped)
/// * `xloc` - X-coordinate of the invariant vertical line (must be in [0, w-1])
/// * `radang` - Shear angle in radians
/// * `fill` - Background fill color
///
/// # Returns
/// A new sheared image
///
/// # Notes
/// - Works only with 8bpp, 32bpp, or colormapped images
/// - Colormaps are removed and converted based on source
/// - Uses 64-subdivision interpolation for sub-pixel accuracy
///
/// # Errors
/// Returns `TransformError::UnsupportedDepth` for unsupported pixel depths.
/// Returns `TransformError::InvalidParameters` if xloc is out of bounds.
pub fn v_shear_li(pix: &Pix, xloc: i32, radang: f32, fill: ShearFill) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    // Validate xloc
    if xloc < 0 || xloc >= w as i32 {
        return Err(TransformError::InvalidParameters(format!(
            "xloc {} is out of bounds [0, {})",
            xloc, w
        )));
    }

    // Check depth
    if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 && pix.colormap().is_none() {
        return Err(TransformError::UnsupportedDepth(
            "v_shear_li requires 8bpp, 32bpp, or colormapped image".to_string(),
        ));
    }

    // Remove colormap if present
    let src_pix = if pix.colormap().is_some() {
        remove_colormap(pix)?
    } else {
        pix.deep_clone()
    };

    // Normalize angle and check if shear is needed
    let radang = match normalize_angle_for_shear(radang, MIN_DIFF_FROM_HALF_PI) {
        Some(a) => a,
        None => return Ok(src_pix),
    };

    let src_depth = src_pix.depth();
    let fill_value = fill.to_value(src_depth);

    // Create output image
    let out_pix = Pix::new(w, h, src_depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    let tan_angle = radang.tan();
    let wi = w as i32;
    let hi = h as i32;
    let hm = hi - 1;

    match src_depth {
        PixelDepth::Bit8 => {
            v_shear_li_gray(
                &src_pix,
                &mut out_mut,
                xloc,
                tan_angle,
                wi,
                hi,
                hm,
                fill_value as u8,
            );
        }
        PixelDepth::Bit32 => {
            v_shear_li_color(
                &src_pix,
                &mut out_mut,
                xloc,
                tan_angle,
                wi,
                hi,
                hm,
                fill_value,
            );
        }
        _ => unreachable!(),
    }

    Ok(out_mut.into())
}

// ============================================================================
// Linear Interpolation Helpers
// ============================================================================

/// Horizontal shear with linear interpolation for 8bpp grayscale
#[allow(clippy::too_many_arguments)]
fn h_shear_li_gray(
    src: &Pix,
    dst: &mut PixMut,
    yloc: i32,
    tan_angle: f32,
    wi: i32,
    hi: i32,
    wm: i32,
    _fill_value: u8,
) {
    for y in 0..hi {
        // xshift is the fractional horizontal shift for this row
        let xshift = (yloc - y) as f32 * tan_angle;

        for jd in 0..wi {
            // Compute sub-pixel source position (scaled by 64)
            let x = (64.0 * (-xshift + jd as f32) + 0.5) as i32;
            let xp = x >> 6; // Integer part
            let xf = x & 63; // Fractional part (0-63)

            if xp < 0 || xp > wm {
                continue; // Keep fill value
            }

            let val = if xp < wm {
                let v0 = unsafe { src.get_pixel_unchecked(xp as u32, y as u32) } as i32;
                let v1 = unsafe { src.get_pixel_unchecked((xp + 1) as u32, y as u32) } as i32;
                ((63 - xf) * v0 + xf * v1 + 31) / 63
            } else {
                // xp == wm, no interpolation needed
                unsafe { src.get_pixel_unchecked(xp as u32, y as u32) as i32 }
            };

            unsafe { dst.set_pixel_unchecked(jd as u32, y as u32, val as u32) };
        }
    }
}

/// Horizontal shear with linear interpolation for 32bpp color
#[allow(clippy::too_many_arguments)]
fn h_shear_li_color(
    src: &Pix,
    dst: &mut PixMut,
    yloc: i32,
    tan_angle: f32,
    wi: i32,
    hi: i32,
    wm: i32,
    _fill_value: u32,
) {
    for y in 0..hi {
        let xshift = (yloc - y) as f32 * tan_angle;

        for jd in 0..wi {
            let x = (64.0 * (-xshift + jd as f32) + 0.5) as i32;
            let xp = x >> 6;
            let xf = x & 63;

            if xp < 0 || xp > wm {
                continue;
            }

            let pixel = if xp < wm {
                let word0 = unsafe { src.get_pixel_unchecked(xp as u32, y as u32) };
                let word1 = unsafe { src.get_pixel_unchecked((xp + 1) as u32, y as u32) };

                let (r0, g0, b0, a0) = color::extract_rgba(word0);
                let (r1, g1, b1, a1) = color::extract_rgba(word1);

                let r = interp_channel(r0, r1, xf);
                let g = interp_channel(g0, g1, xf);
                let b = interp_channel(b0, b1, xf);
                let a = interp_channel(a0, a1, xf);

                color::compose_rgba(r, g, b, a)
            } else {
                unsafe { src.get_pixel_unchecked(xp as u32, y as u32) }
            };

            unsafe { dst.set_pixel_unchecked(jd as u32, y as u32, pixel) };
        }
    }
}

/// Vertical shear with linear interpolation for 8bpp grayscale
#[allow(clippy::too_many_arguments)]
fn v_shear_li_gray(
    src: &Pix,
    dst: &mut PixMut,
    xloc: i32,
    tan_angle: f32,
    wi: i32,
    hi: i32,
    hm: i32,
    _fill_value: u8,
) {
    for x in 0..wi {
        let yshift = (x - xloc) as f32 * tan_angle;

        for id in 0..hi {
            let y = (64.0 * (-yshift + id as f32) + 0.5) as i32;
            let yp = y >> 6;
            let yf = y & 63;

            if yp < 0 || yp > hm {
                continue;
            }

            let val = if yp < hm {
                let v0 = unsafe { src.get_pixel_unchecked(x as u32, yp as u32) } as i32;
                let v1 = unsafe { src.get_pixel_unchecked(x as u32, (yp + 1) as u32) } as i32;
                ((63 - yf) * v0 + yf * v1 + 31) / 63
            } else {
                unsafe { src.get_pixel_unchecked(x as u32, yp as u32) as i32 }
            };

            unsafe { dst.set_pixel_unchecked(x as u32, id as u32, val as u32) };
        }
    }
}

/// Vertical shear with linear interpolation for 32bpp color
#[allow(clippy::too_many_arguments)]
fn v_shear_li_color(
    src: &Pix,
    dst: &mut PixMut,
    xloc: i32,
    tan_angle: f32,
    wi: i32,
    hi: i32,
    hm: i32,
    _fill_value: u32,
) {
    for x in 0..wi {
        let yshift = (x - xloc) as f32 * tan_angle;

        for id in 0..hi {
            let y = (64.0 * (-yshift + id as f32) + 0.5) as i32;
            let yp = y >> 6;
            let yf = y & 63;

            if yp < 0 || yp > hm {
                continue;
            }

            let pixel = if yp < hm {
                let word0 = unsafe { src.get_pixel_unchecked(x as u32, yp as u32) };
                let word1 = unsafe { src.get_pixel_unchecked(x as u32, (yp + 1) as u32) };

                let (r0, g0, b0, a0) = color::extract_rgba(word0);
                let (r1, g1, b1, a1) = color::extract_rgba(word1);

                let r = interp_channel(r0, r1, yf);
                let g = interp_channel(g0, g1, yf);
                let b = interp_channel(b0, b1, yf);
                let a = interp_channel(a0, a1, yf);

                color::compose_rgba(r, g, b, a)
            } else {
                unsafe { src.get_pixel_unchecked(x as u32, yp as u32) }
            };

            unsafe { dst.set_pixel_unchecked(x as u32, id as u32, pixel) };
        }
    }
}

/// Linear interpolation helper for a single channel
#[inline]
fn interp_channel(v0: u8, v1: u8, f: i32) -> u8 {
    (((63 - f) * v0 as i32 + f * v1 as i32 + 31) / 63).clamp(0, 255) as u8
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Fill an image with a constant value
fn fill_image(pix: &mut PixMut, value: u32) {
    let w = pix.width();
    let h = pix.height();
    for y in 0..h {
        for x in 0..w {
            unsafe { pix.set_pixel_unchecked(x, y, value) };
        }
    }
}

/// Remove colormap from an image (simplified version)
/// Converts to grayscale for 8bpp or RGB for 32bpp based on colormap content
fn remove_colormap(pix: &Pix) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let cmap = pix
        .colormap()
        .ok_or_else(|| TransformError::InvalidParameters("image has no colormap".to_string()))?;

    // Determine if colormap is grayscale or color
    let is_gray = cmap
        .colors()
        .iter()
        .all(|c| c.red == c.green && c.green == c.blue);

    if is_gray {
        // Convert to 8bpp grayscale
        let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut out_mut = out_pix.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let idx = unsafe { pix.get_pixel_unchecked(x, y) } as usize;
                let gray = if idx < cmap.len() {
                    cmap.colors()[idx].red
                } else {
                    0
                };
                unsafe { out_mut.set_pixel_unchecked(x, y, gray as u32) };
            }
        }

        Ok(out_mut.into())
    } else {
        // Convert to 32bpp color
        let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
        let mut out_mut = out_pix.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let idx = unsafe { pix.get_pixel_unchecked(x, y) } as usize;
                let pixel = if idx < cmap.len() {
                    let c = &cmap.colors()[idx];
                    color::compose_rgba(c.red, c.green, c.blue, 255)
                } else {
                    0
                };
                unsafe { out_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        Ok(out_mut.into())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ShearFill tests
    // ========================================================================

    #[test]
    fn test_shear_fill_values() {
        assert_eq!(ShearFill::White.to_value(PixelDepth::Bit1), 0);
        assert_eq!(ShearFill::Black.to_value(PixelDepth::Bit1), 1);
        assert_eq!(ShearFill::White.to_value(PixelDepth::Bit8), 255);
        assert_eq!(ShearFill::Black.to_value(PixelDepth::Bit8), 0);
        assert_eq!(ShearFill::White.to_value(PixelDepth::Bit32), 0xFFFFFF00);
        assert_eq!(ShearFill::Black.to_value(PixelDepth::Bit32), 0);
    }

    // ========================================================================
    // Angle normalization tests
    // ========================================================================

    #[test]
    fn test_normalize_angle_zero() {
        let result = normalize_angle_for_shear(0.0, MIN_DIFF_FROM_HALF_PI);
        assert!(result.is_none());
    }

    #[test]
    fn test_normalize_angle_small() {
        let result = normalize_angle_for_shear(0.1, MIN_DIFF_FROM_HALF_PI);
        assert!(result.is_some());
        assert!((result.unwrap() - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_angle_near_pi_half() {
        let pi2 = std::f32::consts::FRAC_PI_2;
        let result = normalize_angle_for_shear(pi2 - 0.01, MIN_DIFF_FROM_HALF_PI);
        assert!(result.is_some());
        // Should be clamped
        assert!(result.unwrap() <= pi2 - MIN_DIFF_FROM_HALF_PI);
    }

    // ========================================================================
    // Horizontal shear tests
    // ========================================================================

    #[test]
    fn test_h_shear_zero_angle() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(5, 5, 100) };
        let pix: Pix = pix_mut.into();

        let result = h_shear(&pix, 5, 0.0, ShearFill::White).unwrap();
        // Zero angle should return copy
        assert_eq!(unsafe { result.get_pixel_unchecked(5, 5) }, 100);
    }

    #[test]
    fn test_h_shear_positive_angle() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Set a marker at (10, 5) - above center
        unsafe { pix_mut.set_pixel_unchecked(10, 5, 100) };
        let pix: Pix = pix_mut.into();

        let result = h_shear(&pix, 10, 0.1, ShearFill::White).unwrap();
        // With positive angle and yloc=10, pixels above (y=5) should shift right
        // The marker should have moved to the right
        let mut found_marker = false;
        for x in 10..20 {
            if unsafe { result.get_pixel_unchecked(x, 5) } == 100 {
                found_marker = true;
                break;
            }
        }
        assert!(found_marker, "marker should have shifted right");
    }

    #[test]
    fn test_h_shear_center() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = h_shear_center(&pix, 0.1, ShearFill::White);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().width(), 20);
    }

    #[test]
    fn test_h_shear_corner() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = h_shear_corner(&pix, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Vertical shear tests
    // ========================================================================

    #[test]
    fn test_v_shear_zero_angle() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(5, 5, 100) };
        let pix: Pix = pix_mut.into();

        let result = v_shear(&pix, 5, 0.0, ShearFill::White).unwrap();
        assert_eq!(unsafe { result.get_pixel_unchecked(5, 5) }, 100);
    }

    #[test]
    fn test_v_shear_positive_angle() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Set a marker at (15, 10) - right of center
        unsafe { pix_mut.set_pixel_unchecked(15, 10, 100) };
        let pix: Pix = pix_mut.into();

        let result = v_shear(&pix, 10, 0.1, ShearFill::White).unwrap();
        // With positive angle and xloc=10, pixels right (x=15) should shift down
        let mut found_marker = false;
        for y in 10..20 {
            if unsafe { result.get_pixel_unchecked(15, y) } == 100 {
                found_marker = true;
                break;
            }
        }
        assert!(found_marker, "marker should have shifted down");
    }

    #[test]
    fn test_v_shear_center() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = v_shear_center(&pix, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_v_shear_corner() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = v_shear_corner(&pix, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    // ========================================================================
    // In-place shear tests
    // ========================================================================

    #[test]
    fn test_h_shear_ip_zero_angle() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(5, 5, 100) };

        let result = h_shear_ip(&mut pix_mut, 5, 0.0, ShearFill::White);
        assert!(result.is_ok());
        assert_eq!(unsafe { pix_mut.get_pixel_unchecked(5, 5) }, 100);
    }

    #[test]
    fn test_h_shear_ip_basic() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(10, 5, 100) };

        let result = h_shear_ip(&mut pix_mut, 10, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_v_shear_ip_basic() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(15, 10, 100) };

        let result = v_shear_ip(&mut pix_mut, 10, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_shear_ip_without_colormap() {
        // Test that in-place shear works on images without colormap
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(10, 5, 100) };

        let result = h_shear_ip(&mut pix_mut, 10, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Linear interpolated shear tests
    // ========================================================================

    #[test]
    fn test_h_shear_li_8bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = h_shear_li(&pix, 10, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_h_shear_li_32bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let result = h_shear_li(&pix, 10, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_h_shear_li_invalid_yloc() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = h_shear_li(&pix, 30, 0.1, ShearFill::White);
        assert!(matches!(result, Err(TransformError::InvalidParameters(_))));
    }

    #[test]
    fn test_h_shear_li_unsupported_depth() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let result = h_shear_li(&pix, 10, 0.1, ShearFill::White);
        assert!(matches!(result, Err(TransformError::UnsupportedDepth(_))));
    }

    #[test]
    fn test_v_shear_li_8bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = v_shear_li(&pix, 10, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_v_shear_li_32bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let result = v_shear_li(&pix, 10, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_v_shear_li_invalid_xloc() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = v_shear_li(&pix, 30, 0.1, ShearFill::White);
        assert!(matches!(result, Err(TransformError::InvalidParameters(_))));
    }

    // ========================================================================
    // Different pixel depth tests
    // ========================================================================

    #[test]
    fn test_h_shear_1bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let result = h_shear(&pix, 10, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_v_shear_1bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let result = v_shear(&pix, 10, 0.1, ShearFill::Black);
        assert!(result.is_ok());
    }

    #[test]
    fn test_h_shear_32bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        let red = color::compose_rgb(255, 0, 0);
        unsafe { pix_mut.set_pixel_unchecked(10, 5, red) };
        let pix: Pix = pix_mut.into();

        let result = h_shear(&pix, 10, 0.1, ShearFill::White);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Fill color tests
    // ========================================================================

    #[test]
    fn test_h_shear_fill_black() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = h_shear(&pix, 0, 0.3, ShearFill::Black).unwrap();
        // Corner should be filled with black (0 for 8bpp)
        assert_eq!(unsafe { result.get_pixel_unchecked(0, 19) }, 0);
    }

    #[test]
    fn test_v_shear_fill_black() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = v_shear(&pix, 0, 0.3, ShearFill::Black).unwrap();
        // Corner should be filled with black
        assert_eq!(unsafe { result.get_pixel_unchecked(19, 0) }, 0);
    }

    // ========================================================================
    // Colormap tests
    // ========================================================================

    #[test]
    fn test_h_shear_preserves_colormap() {
        use leptonica_core::PixColormap;

        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        cmap.add_rgb(0, 255, 0).unwrap();
        let _ = pix_mut.set_colormap(Some(cmap));

        let pix: Pix = pix_mut.into();
        let result = h_shear(&pix, 10, 0.1, ShearFill::White).unwrap();

        // Colormap should be preserved for non-LI operations
        assert!(result.colormap().is_some());
    }

    #[test]
    fn test_h_shear_li_removes_colormap() {
        use leptonica_core::PixColormap;

        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(128, 128, 128).unwrap(); // Grayscale color
        let _ = pix_mut.set_colormap(Some(cmap));

        let pix: Pix = pix_mut.into();
        let result = h_shear_li(&pix, 10, 0.1, ShearFill::White).unwrap();

        // Colormap should be removed for LI operations
        assert!(result.colormap().is_none());
    }
}
