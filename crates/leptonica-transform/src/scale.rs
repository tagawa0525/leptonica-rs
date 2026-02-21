//! Image scaling operations
//!
//! Provides various scaling algorithms including:
//! - Linear interpolation (for upscaling)
//! - Sampling (nearest neighbor)
//! - Area mapping (for downscaling with anti-aliasing)

use crate::{TransformError, TransformResult};
use leptonica_core::{Pix, PixelDepth, color};

/// Scaling method to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleMethod {
    /// Nearest-neighbor sampling (fastest, pixelated results)
    Sampling,
    /// Bilinear interpolation (good for upscaling)
    Linear,
    /// Area mapping (best for downscaling, anti-aliased)
    AreaMap,
    /// Automatic selection based on scale factor
    Auto,
}

/// Scale an image by the given factors
///
/// # Arguments
/// * `pix` - Input image
/// * `scale_x` - Horizontal scale factor (e.g., 2.0 = double width)
/// * `scale_y` - Vertical scale factor
/// * `method` - Scaling algorithm to use
pub fn scale(pix: &Pix, scale_x: f32, scale_y: f32, method: ScaleMethod) -> TransformResult<Pix> {
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return Err(TransformError::InvalidScaleFactor(format!(
            "scale factors must be positive: ({}, {})",
            scale_x, scale_y
        )));
    }

    let w = pix.width();
    let h = pix.height();
    let new_w = ((w as f32) * scale_x).round() as u32;
    let new_h = ((h as f32) * scale_y).round() as u32;

    if new_w == 0 || new_h == 0 {
        return Err(TransformError::InvalidScaleFactor(
            "resulting dimensions would be zero".to_string(),
        ));
    }

    let method = match method {
        ScaleMethod::Auto => {
            // Choose method based on scale factors
            let min_scale = scale_x.min(scale_y);
            if min_scale < 0.7 {
                ScaleMethod::AreaMap
            } else {
                ScaleMethod::Linear
            }
        }
        m => m,
    };

    match method {
        ScaleMethod::Sampling => scale_by_sampling_impl(pix, new_w, new_h),
        ScaleMethod::Linear => scale_linear(pix, new_w, new_h),
        ScaleMethod::AreaMap => scale_area_map(pix, scale_x, scale_y, new_w, new_h),
        ScaleMethod::Auto => unreachable!(),
    }
}

/// Scale an image to a specific size
///
/// # Arguments
/// * `pix` - Input image
/// * `width` - Target width (0 to maintain aspect ratio)
/// * `height` - Target height (0 to maintain aspect ratio)
pub fn scale_to_size(pix: &Pix, width: u32, height: u32) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let (target_w, target_h) = match (width, height) {
        (0, 0) => return Ok(pix.deep_clone()),
        (0, h_target) => {
            let scale = h_target as f32 / h as f32;
            ((w as f32 * scale).round() as u32, h_target)
        }
        (w_target, 0) => {
            let scale = w_target as f32 / w as f32;
            (w_target, (h as f32 * scale).round() as u32)
        }
        (w_target, h_target) => (w_target, h_target),
    };

    let scale_x = target_w as f32 / w as f32;
    let scale_y = target_h as f32 / h as f32;

    scale(pix, scale_x, scale_y, ScaleMethod::Auto)
}

/// Scale an image using nearest-neighbor sampling
pub fn scale_by_sampling(pix: &Pix, scale_x: f32, scale_y: f32) -> TransformResult<Pix> {
    scale(pix, scale_x, scale_y, ScaleMethod::Sampling)
}

/// Scale using bilinear (linear) interpolation.
///
/// For 8bpp (grayscale) and 32bpp (color) images.
/// If scale factors are both < 0.7, redirects to [`scale_general`] with area mapping.
pub fn scale_li(pix: &Pix, scale_x: f32, scale_y: f32) -> TransformResult<Pix> {
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return Err(TransformError::InvalidScaleFactor(format!(
            "scale factors must be positive: ({}, {})",
            scale_x, scale_y
        )));
    }
    let max_scale = scale_x.max(scale_y);
    if max_scale < 0.7 {
        return scale_general(pix, scale_x, scale_y, 0.0, 0);
    }
    let depth = pix.depth();
    match depth {
        PixelDepth::Bit32 => scale_color_li(pix, scale_x, scale_y),
        PixelDepth::Bit8 => scale_gray_li(pix, scale_x, scale_y),
        _ => {
            let new_w = ((pix.width() as f32) * scale_x).round() as u32;
            let new_h = ((pix.height() as f32) * scale_y).round() as u32;
            scale_by_sampling_impl(pix, new_w.max(1), new_h.max(1))
        }
    }
}

/// Scale a 32bpp color image using bilinear interpolation.
///
/// If scale factors are both < 0.7, redirects to area mapping via [`scale_general`].
pub fn scale_color_li(pix: &Pix, scale_x: f32, scale_y: f32) -> TransformResult<Pix> {
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return Err(TransformError::InvalidScaleFactor(format!(
            "scale factors must be positive: ({}, {})",
            scale_x, scale_y
        )));
    }
    if pix.depth() != PixelDepth::Bit32 {
        return Err(TransformError::InvalidParameters(
            "scale_color_li requires 32bpp input".to_string(),
        ));
    }
    let max_scale = scale_x.max(scale_y);
    if max_scale < 0.7 {
        return scale_general(pix, scale_x, scale_y, 0.0, 0);
    }
    let new_w = ((pix.width() as f32) * scale_x).round() as u32;
    let new_h = ((pix.height() as f32) * scale_y).round() as u32;
    scale_linear_color(pix, new_w.max(1), new_h.max(1))
}

/// Scale an 8bpp grayscale image using bilinear interpolation.
///
/// If scale factors are both < 0.7, redirects to area mapping via [`scale_general`].
pub fn scale_gray_li(pix: &Pix, scale_x: f32, scale_y: f32) -> TransformResult<Pix> {
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return Err(TransformError::InvalidScaleFactor(format!(
            "scale factors must be positive: ({}, {})",
            scale_x, scale_y
        )));
    }
    if pix.depth() != PixelDepth::Bit8 {
        return Err(TransformError::InvalidParameters(
            "scale_gray_li requires 8bpp input".to_string(),
        ));
    }
    let max_scale = scale_x.max(scale_y);
    if max_scale < 0.7 {
        return scale_general(pix, scale_x, scale_y, 0.0, 0);
    }
    let new_w = ((pix.width() as f32) * scale_x).round() as u32;
    let new_h = ((pix.height() as f32) * scale_y).round() as u32;
    scale_linear_gray(pix, new_w.max(1), new_h.max(1))
}

/// General-purpose scaling with optional sharpening.
///
/// Dispatches to the appropriate method based on scale factors:
/// - For 1bpp: nearest-neighbor sampling
/// - For `max_scale < 0.7`: smooth (box-filter) or area mapping
/// - Otherwise: linear interpolation (LI)
///
/// **Note**: The `sharpfract` and `sharpwidth` parameters are accepted for API compatibility
/// but sharpening is not applied. Use an external unsharp-masking step if needed.
pub fn scale_general(
    pix: &Pix,
    scale_x: f32,
    scale_y: f32,
    _sharpfract: f32,
    _sharpwidth: i32,
) -> TransformResult<Pix> {
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return Err(TransformError::InvalidScaleFactor(format!(
            "scale factors must be positive: ({}, {})",
            scale_x, scale_y
        )));
    }
    if scale_x == 1.0 && scale_y == 1.0 {
        return Ok(pix.deep_clone());
    }

    let depth = pix.depth();
    // 1bpp: fall through to sampling
    if depth == PixelDepth::Bit1 {
        let new_w = ((pix.width() as f32) * scale_x).round() as u32;
        let new_h = ((pix.height() as f32) * scale_y).round() as u32;
        return scale_by_sampling_impl(pix, new_w.max(1), new_h.max(1));
    }

    let max_scale = scale_x.max(scale_y);
    let min_scale = scale_x.min(scale_y);

    if max_scale < 0.7 {
        if min_scale < 0.02 {
            scale_smooth(pix, scale_x, scale_y)
        } else {
            let new_w = ((pix.width() as f32) * scale_x).round() as u32;
            let new_h = ((pix.height() as f32) * scale_y).round() as u32;
            scale_area_map(pix, scale_x, scale_y, new_w.max(1), new_h.max(1))
        }
    } else {
        // Linear interpolation
        let new_w = ((pix.width() as f32) * scale_x).round() as u32;
        let new_h = ((pix.height() as f32) * scale_y).round() as u32;
        scale_linear(pix, new_w.max(1), new_h.max(1))
    }
}

/// Scale to a target resolution.
///
/// If the image has a known resolution (`xres > 0`), the scale factor is computed as
/// `target / xres`. If the resolution is unknown (0), `assumed` is used instead;
/// passing `assumed == 0.0` returns a copy of the input unchanged.
pub fn scale_to_resolution(pix: &Pix, target: f32, assumed: f32) -> TransformResult<Pix> {
    if target <= 0.0 {
        return Err(TransformError::InvalidParameters(
            "target resolution must be > 0".to_string(),
        ));
    }
    let xres = pix.xres();
    let effective_res = if xres > 0 {
        xres as f32
    } else if assumed <= 0.0 {
        return Ok(pix.deep_clone());
    } else {
        assumed
    };
    let factor = target / effective_res;
    scale(pix, factor, factor, ScaleMethod::Auto)
}

/// Scale using nearest-neighbor sampling with a configurable half-pixel shift.
///
/// `shift_x` / `shift_y` must be either `0.0` or `0.5`.
/// The default (used by [`scale_by_sampling`]) is `0.5`.
/// Using `0.0` minimizes edge effects near image borders.
pub fn scale_by_sampling_with_shift(
    pix: &Pix,
    scale_x: f32,
    scale_y: f32,
    shift_x: f32,
    shift_y: f32,
) -> TransformResult<Pix> {
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return Err(TransformError::InvalidScaleFactor(format!(
            "scale factors must be positive: ({}, {})",
            scale_x, scale_y
        )));
    }
    if (shift_x != 0.0 && shift_x != 0.5) || (shift_y != 0.0 && shift_y != 0.5) {
        return Err(TransformError::InvalidParameters(
            "shift must be 0.0 or 0.5".to_string(),
        ));
    }
    let w = pix.width();
    let h = pix.height();
    let new_w = ((w as f32) * scale_x).round() as u32;
    let new_h = ((h as f32) * scale_y).round() as u32;
    if new_w == 0 || new_h == 0 {
        return Err(TransformError::InvalidScaleFactor(
            "resulting dimensions would be zero".to_string(),
        ));
    }

    let depth = pix.depth();
    let out_pix = Pix::new(new_w, new_h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    let inv_x = w as f32 / new_w as f32;
    let inv_y = h as f32 / new_h as f32;

    for y in 0..new_h {
        let src_y = ((y as f32 + shift_y) * inv_y) as u32;
        let src_y = src_y.min(h - 1);
        for x in 0..new_w {
            let src_x = ((x as f32 + shift_x) * inv_x) as u32;
            let src_x = src_x.min(w - 1);
            out_mut.set_pixel_unchecked(x, y, pix.get_pixel_unchecked(src_x, src_y));
        }
    }

    Ok(out_mut.into())
}

/// Scale by an integer subsampling factor (isotropic downsampling).
///
/// A `factor` of 1 returns a copy. For `factor >= 2`, every `factor`-th pixel
/// is sampled (equivalent to `scale_by_sampling(pix, 1/factor, 1/factor)`).
pub fn scale_by_int_sampling(pix: &Pix, factor: u32) -> TransformResult<Pix> {
    if factor == 0 {
        return Err(TransformError::InvalidParameters(
            "factor must be >= 1".to_string(),
        ));
    }
    if factor == 1 {
        return Ok(pix.deep_clone());
    }
    let scale = 1.0 / factor as f32;
    scale_by_sampling(pix, scale, scale)
}

/// Smooth downscaling using a box filter followed by subsampling.
///
/// Should only be used when both scale factors are < 0.7. For larger factors,
/// this redirects to [`scale_general`] with LI.
///
/// The box filter width is `max(2, round(1 / min_scale))`, ensuring adequate
/// anti-aliasing.
pub fn scale_smooth(pix: &Pix, scale_x: f32, scale_y: f32) -> TransformResult<Pix> {
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return Err(TransformError::InvalidScaleFactor(format!(
            "scale factors must be positive: ({}, {})",
            scale_x, scale_y
        )));
    }
    if scale_x >= 0.7 || scale_y >= 0.7 {
        return scale_general(pix, scale_x, scale_y, 0.0, 0);
    }

    let depth = pix.depth();
    let w = pix.width();
    let h = pix.height();
    let new_w = ((w as f32) * scale_x).round() as u32;
    let new_h = ((h as f32) * scale_y).round() as u32;
    if new_w == 0 || new_h == 0 {
        return Err(TransformError::InvalidScaleFactor(
            "resulting dimensions would be zero".to_string(),
        ));
    }

    let min_scale = scale_x.min(scale_y);
    let isize = (1.0 / min_scale + 0.5).floor() as u32;
    let isize = isize.clamp(2, 10000);

    match depth {
        PixelDepth::Bit8 if pix.colormap().is_none() => {
            scale_smooth_gray(pix, scale_x, scale_y, new_w, new_h, isize)
        }
        PixelDepth::Bit32 => scale_smooth_color(pix, scale_x, scale_y, new_w, new_h, isize),
        _ => scale_area_map(pix, scale_x, scale_y, new_w, new_h),
    }
}

/// Box-filter smooth downscale for 8bpp grayscale
fn scale_smooth_gray(
    pix: &Pix,
    scale_x: f32,
    scale_y: f32,
    new_w: u32,
    new_h: u32,
    isize: u32,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let out_pix = Pix::new(new_w, new_h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let half = isize / 2;
    for yd in 0..new_h {
        let ys_center = (yd as f32 / scale_y + 0.5) as i32;
        for xd in 0..new_w {
            let xs_center = (xd as f32 / scale_x + 0.5) as i32;
            let mut sum: u32 = 0;
            let mut count: u32 = 0;
            for dy in 0..isize {
                let ys = ys_center - half as i32 + dy as i32;
                if ys < 0 || ys >= h as i32 {
                    continue;
                }
                for dx in 0..isize {
                    let xs = xs_center - half as i32 + dx as i32;
                    if xs < 0 || xs >= w as i32 {
                        continue;
                    }
                    sum += pix.get_pixel_unchecked(xs as u32, ys as u32);
                    count += 1;
                }
            }
            let val = if count > 0 {
                ((sum as f32 / count as f32) + 0.5) as u32
            } else {
                0
            };
            out_mut.set_pixel_unchecked(xd, yd, val.min(255));
        }
    }
    Ok(out_mut.into())
}

/// Box-filter smooth downscale for 32bpp color
fn scale_smooth_color(
    pix: &Pix,
    scale_x: f32,
    scale_y: f32,
    new_w: u32,
    new_h: u32,
    isize: u32,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let out_pix = Pix::new(new_w, new_h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    let half = isize / 2;
    for yd in 0..new_h {
        let ys_center = (yd as f32 / scale_y + 0.5) as i32;
        for xd in 0..new_w {
            let xs_center = (xd as f32 / scale_x + 0.5) as i32;
            let mut sum_r: u32 = 0;
            let mut sum_g: u32 = 0;
            let mut sum_b: u32 = 0;
            let mut sum_a: u32 = 0;
            let mut count: u32 = 0;
            for dy in 0..isize {
                let ys = ys_center - half as i32 + dy as i32;
                if ys < 0 || ys >= h as i32 {
                    continue;
                }
                for dx in 0..isize {
                    let xs = xs_center - half as i32 + dx as i32;
                    if xs < 0 || xs >= w as i32 {
                        continue;
                    }
                    let (r, g, b, a) =
                        color::extract_rgba(pix.get_pixel_unchecked(xs as u32, ys as u32));
                    sum_r += r as u32;
                    sum_g += g as u32;
                    sum_b += b as u32;
                    sum_a += a as u32;
                    count += 1;
                }
            }
            let pixel = if count > 0 {
                let r = ((sum_r as f32 / count as f32) + 0.5) as u8;
                let g = ((sum_g as f32 / count as f32) + 0.5) as u8;
                let b = ((sum_b as f32 / count as f32) + 0.5) as u8;
                let a = ((sum_a as f32 / count as f32) + 0.5) as u8;
                color::compose_rgba(r, g, b, a)
            } else {
                0
            };
            out_mut.set_pixel_unchecked(xd, yd, pixel);
        }
    }
    Ok(out_mut.into())
}

/// Internal sampling implementation
fn scale_by_sampling_impl(pix: &Pix, new_w: u32, new_h: u32) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    let out_pix = Pix::new(new_w, new_h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    let scale_x = w as f32 / new_w as f32;
    let scale_y = h as f32 / new_h as f32;

    for y in 0..new_h {
        let src_y = ((y as f32 + 0.5) * scale_y) as u32;
        let src_y = src_y.min(h - 1);

        for x in 0..new_w {
            let src_x = ((x as f32 + 0.5) * scale_x) as u32;
            let src_x = src_x.min(w - 1);

            let val = pix.get_pixel_unchecked(src_x, src_y);
            out_mut.set_pixel_unchecked(x, y, val);
        }
    }

    Ok(out_mut.into())
}

/// Scale using bilinear interpolation
fn scale_linear(pix: &Pix, new_w: u32, new_h: u32) -> TransformResult<Pix> {
    let depth = pix.depth();

    match depth {
        PixelDepth::Bit32 => scale_linear_color(pix, new_w, new_h),
        PixelDepth::Bit8 => scale_linear_gray(pix, new_w, new_h),
        _ => {
            // For other depths, fall back to sampling
            scale_by_sampling_impl(pix, new_w, new_h)
        }
    }
}

/// Bilinear interpolation for 32bpp color images
fn scale_linear_color(pix: &Pix, new_w: u32, new_h: u32) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(new_w, new_h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    let scale_x = (w as f32 - 1.0) / (new_w as f32 - 1.0).max(1.0);
    let scale_y = (h as f32 - 1.0) / (new_h as f32 - 1.0).max(1.0);

    for y in 0..new_h {
        let src_y = y as f32 * scale_y;
        let y0 = src_y.floor() as u32;
        let y1 = (y0 + 1).min(h - 1);
        let fy = src_y - y0 as f32;

        for x in 0..new_w {
            let src_x = x as f32 * scale_x;
            let x0 = src_x.floor() as u32;
            let x1 = (x0 + 1).min(w - 1);
            let fx = src_x - x0 as f32;

            // Get 4 corner pixels
            let p00 = pix.get_pixel_unchecked(x0, y0);
            let p10 = pix.get_pixel_unchecked(x1, y0);
            let p01 = pix.get_pixel_unchecked(x0, y1);
            let p11 = pix.get_pixel_unchecked(x1, y1);

            // Interpolate each channel
            let result = interpolate_color(p00, p10, p01, p11, fx, fy);
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Bilinear interpolation for 8bpp grayscale images
fn scale_linear_gray(pix: &Pix, new_w: u32, new_h: u32) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    // If has colormap, convert values
    if pix.colormap().is_some() {
        return scale_by_sampling_impl(pix, new_w, new_h);
    }

    let out_pix = Pix::new(new_w, new_h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let scale_x = (w as f32 - 1.0) / (new_w as f32 - 1.0).max(1.0);
    let scale_y = (h as f32 - 1.0) / (new_h as f32 - 1.0).max(1.0);

    for y in 0..new_h {
        let src_y = y as f32 * scale_y;
        let y0 = src_y.floor() as u32;
        let y1 = (y0 + 1).min(h - 1);
        let fy = src_y - y0 as f32;

        for x in 0..new_w {
            let src_x = x as f32 * scale_x;
            let x0 = src_x.floor() as u32;
            let x1 = (x0 + 1).min(w - 1);
            let fx = src_x - x0 as f32;

            // Get 4 corner pixels
            let p00 = pix.get_pixel_unchecked(x0, y0) as f32;
            let p10 = pix.get_pixel_unchecked(x1, y0) as f32;
            let p01 = pix.get_pixel_unchecked(x0, y1) as f32;
            let p11 = pix.get_pixel_unchecked(x1, y1) as f32;

            // Bilinear interpolation
            let top = p00 * (1.0 - fx) + p10 * fx;
            let bottom = p01 * (1.0 - fx) + p11 * fx;
            let result = (top * (1.0 - fy) + bottom * fy).round() as u32;

            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Interpolate between 4 color pixels using bilinear weights
fn interpolate_color(p00: u32, p10: u32, p01: u32, p11: u32, fx: f32, fy: f32) -> u32 {
    let (r00, g00, b00, a00) = color::extract_rgba(p00);
    let (r10, g10, b10, a10) = color::extract_rgba(p10);
    let (r01, g01, b01, a01) = color::extract_rgba(p01);
    let (r11, g11, b11, a11) = color::extract_rgba(p11);

    let r = interpolate_channel(r00, r10, r01, r11, fx, fy);
    let g = interpolate_channel(g00, g10, g01, g11, fx, fy);
    let b = interpolate_channel(b00, b10, b01, b11, fx, fy);
    let a = interpolate_channel(a00, a10, a01, a11, fx, fy);

    color::compose_rgba(r, g, b, a)
}

/// Bilinear interpolation for a single channel
fn interpolate_channel(c00: u8, c10: u8, c01: u8, c11: u8, fx: f32, fy: f32) -> u8 {
    let c00 = c00 as f32;
    let c10 = c10 as f32;
    let c01 = c01 as f32;
    let c11 = c11 as f32;

    let top = c00 * (1.0 - fx) + c10 * fx;
    let bottom = c01 * (1.0 - fx) + c11 * fx;
    let result = top * (1.0 - fy) + bottom * fy;

    result.round().clamp(0.0, 255.0) as u8
}

/// Scale using area mapping (for downscaling with anti-aliasing)
fn scale_area_map(
    pix: &Pix,
    scale_x: f32,
    scale_y: f32,
    new_w: u32,
    new_h: u32,
) -> TransformResult<Pix> {
    let depth = pix.depth();

    match depth {
        PixelDepth::Bit32 => scale_area_map_color(pix, scale_x, scale_y, new_w, new_h),
        PixelDepth::Bit8 if pix.colormap().is_none() => {
            scale_area_map_gray(pix, scale_x, scale_y, new_w, new_h)
        }
        _ => {
            // For other depths, fall back to sampling
            scale_by_sampling_impl(pix, new_w, new_h)
        }
    }
}

/// Area mapping for color images
fn scale_area_map_color(
    pix: &Pix,
    scale_x: f32,
    scale_y: f32,
    new_w: u32,
    new_h: u32,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(new_w, new_h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    let inv_scale_x = 1.0 / scale_x;
    let inv_scale_y = 1.0 / scale_y;

    for y in 0..new_h {
        let src_y_start = y as f32 * inv_scale_y;
        let src_y_end = ((y + 1) as f32 * inv_scale_y).min(h as f32);

        for x in 0..new_w {
            let src_x_start = x as f32 * inv_scale_x;
            let src_x_end = ((x + 1) as f32 * inv_scale_x).min(w as f32);

            let pixel = area_average_color(pix, src_x_start, src_y_start, src_x_end, src_y_end);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }

    Ok(out_mut.into())
}

/// Area mapping for grayscale images
fn scale_area_map_gray(
    pix: &Pix,
    scale_x: f32,
    scale_y: f32,
    new_w: u32,
    new_h: u32,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(new_w, new_h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let inv_scale_x = 1.0 / scale_x;
    let inv_scale_y = 1.0 / scale_y;

    for y in 0..new_h {
        let src_y_start = y as f32 * inv_scale_y;
        let src_y_end = ((y + 1) as f32 * inv_scale_y).min(h as f32);

        for x in 0..new_w {
            let src_x_start = x as f32 * inv_scale_x;
            let src_x_end = ((x + 1) as f32 * inv_scale_x).min(w as f32);

            let val = area_average_gray(pix, src_x_start, src_y_start, src_x_end, src_y_end);
            out_mut.set_pixel_unchecked(x, y, val as u32);
        }
    }

    Ok(out_mut.into())
}

/// Calculate area-weighted average for color pixels
fn area_average_color(pix: &Pix, x0: f32, y0: f32, x1: f32, y1: f32) -> u32 {
    let w = pix.width();
    let h = pix.height();

    let ix0 = x0.floor() as u32;
    let iy0 = y0.floor() as u32;
    let ix1 = (x1.ceil() as u32).min(w);
    let iy1 = (y1.ceil() as u32).min(h);

    let mut sum_r = 0.0f32;
    let mut sum_g = 0.0f32;
    let mut sum_b = 0.0f32;
    let mut sum_a = 0.0f32;
    let mut total_weight = 0.0f32;

    for sy in iy0..iy1 {
        // Calculate vertical weight
        let y_weight = if sy == iy0 {
            1.0 - (y0 - sy as f32)
        } else if sy + 1 >= iy1 {
            y1 - sy as f32
        } else {
            1.0
        };

        for sx in ix0..ix1 {
            // Calculate horizontal weight
            let x_weight = if sx == ix0 {
                1.0 - (x0 - sx as f32)
            } else if sx + 1 >= ix1 {
                x1 - sx as f32
            } else {
                1.0
            };

            let weight = x_weight * y_weight;
            let pixel = pix.get_pixel_unchecked(sx, sy);
            let (r, g, b, a) = color::extract_rgba(pixel);

            sum_r += r as f32 * weight;
            sum_g += g as f32 * weight;
            sum_b += b as f32 * weight;
            sum_a += a as f32 * weight;
            total_weight += weight;
        }
    }

    if total_weight > 0.0 {
        let r = (sum_r / total_weight).round().clamp(0.0, 255.0) as u8;
        let g = (sum_g / total_weight).round().clamp(0.0, 255.0) as u8;
        let b = (sum_b / total_weight).round().clamp(0.0, 255.0) as u8;
        let a = (sum_a / total_weight).round().clamp(0.0, 255.0) as u8;
        color::compose_rgba(r, g, b, a)
    } else {
        0
    }
}

/// Calculate area-weighted average for grayscale pixels
fn area_average_gray(pix: &Pix, x0: f32, y0: f32, x1: f32, y1: f32) -> u8 {
    let w = pix.width();
    let h = pix.height();

    let ix0 = x0.floor() as u32;
    let iy0 = y0.floor() as u32;
    let ix1 = (x1.ceil() as u32).min(w);
    let iy1 = (y1.ceil() as u32).min(h);

    let mut sum = 0.0f32;
    let mut total_weight = 0.0f32;

    for sy in iy0..iy1 {
        let y_weight = if sy == iy0 {
            1.0 - (y0 - sy as f32)
        } else if sy + 1 >= iy1 {
            y1 - sy as f32
        } else {
            1.0
        };

        for sx in ix0..ix1 {
            let x_weight = if sx == ix0 {
                1.0 - (x0 - sx as f32)
            } else if sx + 1 >= ix1 {
                x1 - sx as f32
            } else {
                1.0
            };

            let weight = x_weight * y_weight;
            let val = pix.get_pixel_unchecked(sx, sy);
            sum += val as f32 * weight;
            total_weight += weight;
        }
    }

    if total_weight > 0.0 {
        (sum / total_weight).round().clamp(0.0, 255.0) as u8
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_sampling_up() {
        let pix = Pix::new(2, 2, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // [10, 20]
        // [30, 40]
        pix_mut.set_pixel_unchecked(0, 0, 10);
        pix_mut.set_pixel_unchecked(1, 0, 20);
        pix_mut.set_pixel_unchecked(0, 1, 30);
        pix_mut.set_pixel_unchecked(1, 1, 40);

        let pix: Pix = pix_mut.into();
        let scaled = scale(&pix, 2.0, 2.0, ScaleMethod::Sampling).unwrap();

        assert_eq!((scaled.width(), scaled.height()), (4, 4));
    }

    #[test]
    fn test_scale_sampling_down() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..4 {
            for x in 0..4 {
                pix_mut.set_pixel_unchecked(x, y, x + y * 4);
            }
        }

        let pix: Pix = pix_mut.into();
        let scaled = scale(&pix, 0.5, 0.5, ScaleMethod::Sampling).unwrap();

        assert_eq!((scaled.width(), scaled.height()), (2, 2));
    }

    #[test]
    fn test_scale_to_size_aspect_ratio() {
        let pix = Pix::new(100, 200, PixelDepth::Bit8).unwrap();

        // Scale to width 50, maintaining aspect ratio
        let scaled = scale_to_size(&pix, 50, 0).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (50, 100));

        // Scale to height 100, maintaining aspect ratio
        let scaled = scale_to_size(&pix, 0, 100).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (50, 100));
    }

    #[test]
    fn test_scale_linear_color() {
        let pix = Pix::new(2, 2, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let black = color::compose_rgb(0, 0, 0);
        let white = color::compose_rgb(255, 255, 255);

        pix_mut.set_pixel_unchecked(0, 0, black);
        pix_mut.set_pixel_unchecked(1, 0, white);
        pix_mut.set_pixel_unchecked(0, 1, white);
        pix_mut.set_pixel_unchecked(1, 1, black);

        let pix: Pix = pix_mut.into();
        let scaled = scale(&pix, 2.0, 2.0, ScaleMethod::Linear).unwrap();

        assert_eq!((scaled.width(), scaled.height()), (4, 4));
        // Center pixels should be interpolated (grayish)
    }

    #[test]
    fn test_scale_area_map_gray() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with values 0-255
        for y in 0..4 {
            for x in 0..4 {
                let val = (x + y * 4) * 16;
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }

        let pix: Pix = pix_mut.into();
        let scaled = scale(&pix, 0.5, 0.5, ScaleMethod::AreaMap).unwrap();

        assert_eq!((scaled.width(), scaled.height()), (2, 2));
        // Values should be averaged from 2x2 blocks
    }

    #[test]
    fn test_scale_invalid_factor() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();

        let result = scale(&pix, 0.0, 1.0, ScaleMethod::Sampling);
        assert!(result.is_err());

        let result = scale(&pix, -1.0, 1.0, ScaleMethod::Sampling);
        assert!(result.is_err());
    }

    #[test]
    fn test_scale_auto_method() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();

        // Upscaling should use linear
        let scaled = scale(&pix, 2.0, 2.0, ScaleMethod::Auto).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (200, 200));

        // Downscaling should use area map (but 8bpp without colormap works)
        let scaled = scale(&pix, 0.5, 0.5, ScaleMethod::Auto).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (50, 50));
    }

    #[test]
    fn test_scale_1bpp_sampling() {
        let pix = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Checkerboard pattern
        for y in 0..4 {
            for x in 0..4 {
                let val = (x + y) % 2;
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }

        let pix: Pix = pix_mut.into();
        let scaled = scale(&pix, 2.0, 2.0, ScaleMethod::Sampling).unwrap();

        assert_eq!((scaled.width(), scaled.height()), (8, 8));
        assert_eq!(scaled.depth(), PixelDepth::Bit1);
    }

    // --- Phase 3: Scale拡張 - 基本 ---

    #[test]
    fn test_scale_li_gray_upscale() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..4u32 {
            for x in 0..4u32 {
                pix_mut.set_pixel_unchecked(x, y, x * 30 + y * 30);
            }
        }
        let pix: Pix = pix_mut.into();
        let scaled = scale_li(&pix, 2.0, 2.0).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (8, 8));
        assert_eq!(scaled.depth(), PixelDepth::Bit8);
        // top-left corner must stay the same
        assert_eq!(scaled.get_pixel(0, 0).unwrap(), 0);
        // bottom-right output corner (7,7) maps exactly to source (3,3) = 3*30+3*30 = 180
        assert_eq!(scaled.get_pixel(7, 7).unwrap(), 180);
    }

    #[test]
    fn test_scale_li_color_upscale() {
        let pix = Pix::new(4, 4, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, color::compose_rgb(0, 0, 0));
        pix_mut.set_pixel_unchecked(3, 3, color::compose_rgb(255, 255, 255));
        let pix: Pix = pix_mut.into();
        let scaled = scale_li(&pix, 2.0, 2.0).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (8, 8));
        assert_eq!(scaled.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_scale_color_li_basic() {
        let pix = Pix::new(3, 3, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, color::compose_rgb(100, 0, 0));
        pix_mut.set_pixel_unchecked(2, 0, color::compose_rgb(0, 100, 0));
        let pix: Pix = pix_mut.into();
        let scaled = scale_color_li(&pix, 2.0, 2.0).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (6, 6));
        assert_eq!(scaled.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_scale_gray_li_basic() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 50);
        pix_mut.set_pixel_unchecked(3, 3, 200);
        let pix: Pix = pix_mut.into();
        let scaled = scale_gray_li(&pix, 1.5, 1.5).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (6, 6));
        assert_eq!(scaled.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_general_no_sharpening_gray() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let scaled = scale_general(&pix, 0.5, 0.5, 0.0, 1).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (50, 50));
        assert_eq!(scaled.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_general_upscale_color() {
        let pix = Pix::new(4, 4, PixelDepth::Bit32).unwrap();
        let scaled = scale_general(&pix, 2.0, 2.0, 0.0, 1).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (8, 8));
        assert_eq!(scaled.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_scale_to_resolution_known_res() {
        let mut pix = Pix::new(100, 100, PixelDepth::Bit8)
            .unwrap()
            .try_into_mut()
            .unwrap();
        pix.set_xres(300);
        pix.set_yres(300);
        let pix: Pix = pix.into();
        // Scale from 300 DPI to 150 DPI → factor = 0.5
        let scaled = scale_to_resolution(&pix, 150.0, 300.0).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (50, 50));
    }

    #[test]
    fn test_scale_to_resolution_unknown_res() {
        // No xres set → uses assumed value
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let scaled = scale_to_resolution(&pix, 150.0, 300.0).unwrap();
        // assumed=300 DPI → target/assumed = 150/300 = 0.5 → 50x50
        assert_eq!((scaled.width(), scaled.height()), (50, 50));
    }

    #[test]
    fn test_scale_by_sampling_with_shift_zero() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..8u32 {
            for x in 0..8u32 {
                pix_mut.set_pixel_unchecked(x, y, x * 10);
            }
        }
        let pix: Pix = pix_mut.into();
        let scaled = scale_by_sampling_with_shift(&pix, 0.5, 0.5, 0.0, 0.0).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (4, 4));
    }

    #[test]
    fn test_scale_by_sampling_with_shift_half() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let scaled = scale_by_sampling_with_shift(&pix, 0.5, 0.5, 0.5, 0.5).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (4, 4));
    }

    #[test]
    fn test_scale_by_int_sampling_factor2() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..8u32 {
            for x in 0..8u32 {
                pix_mut.set_pixel_unchecked(x, y, x * 10);
            }
        }
        let pix: Pix = pix_mut.into();
        let scaled = scale_by_int_sampling(&pix, 2).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (4, 4));
        // With factor=2 and shift=0.5: src_x = (0 + 0.5) * 2 = 1 → pixel[1,0] = 10
        assert_eq!(scaled.get_pixel(0, 0).unwrap(), 10);
    }

    #[test]
    fn test_scale_by_int_sampling_factor1() {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let scaled = scale_by_int_sampling(&pix, 1).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (5, 5));
    }

    #[test]
    fn test_scale_smooth_gray() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Alternating checkerboard
        for y in 0..20u32 {
            for x in 0..20u32 {
                pix_mut.set_pixel_unchecked(x, y, ((x + y) % 2) * 255);
            }
        }
        let pix: Pix = pix_mut.into();
        let scaled = scale_smooth(&pix, 0.5, 0.5).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (10, 10));
        assert_eq!(scaled.depth(), PixelDepth::Bit8);
        // The smoothed result should be approximately 127 (average of 0 and 255)
        let center_val = scaled.get_pixel(5, 5).unwrap();
        assert!(
            center_val > 100 && center_val < 160,
            "expected ~127 but got {center_val}"
        );
    }

    #[test]
    fn test_scale_smooth_color() {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..20u32 {
            for x in 0..20u32 {
                let v = ((x + y) % 2) as u8 * 255;
                pix_mut.set_pixel_unchecked(x, y, color::compose_rgb(v, v, v));
            }
        }
        let pix: Pix = pix_mut.into();
        let scaled = scale_smooth(&pix, 0.5, 0.5).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (10, 10));
        assert_eq!(scaled.depth(), PixelDepth::Bit32);
    }
}
