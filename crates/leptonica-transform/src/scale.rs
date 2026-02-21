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

/// Mode for grayscale min/max downscaling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrayMinMaxMode {
    /// Select minimum value in block (darkest)
    Min,
    /// Select maximum value in block (lightest)
    Max,
    /// Select max minus min value in block (contrast)
    MaxDiff,
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

// --- Phase 4: 1bpp→8bpp scale-to-gray functions ---

/// Scale a 1bpp binary image to 8bpp grayscale using optimal integer reduction.
///
/// Dispatches to the most appropriate `scale_to_gray_N` function based on `scale_factor`.
/// For scale factors that are not exact powers-of-two-reciprocals, binary pre-scaling
/// is applied before the nearest integer `scale_to_gray_N` step.
///
/// `scale_factor` must be in (0.0, 1.0).
pub fn scale_to_gray(pix: &Pix, scale_factor: f32) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(TransformError::InvalidParameters(
            "scale_to_gray requires 1bpp input".to_string(),
        ));
    }
    if scale_factor <= 0.0 || scale_factor >= 1.0 {
        return Err(TransformError::InvalidParameters(
            "scale_factor must be in (0.0, 1.0)".to_string(),
        ));
    }

    if scale_factor > 0.5 {
        let mag = 2.0 * scale_factor;
        let pre = scale_binary(pix, mag, mag)?;
        scale_to_gray_2(&pre)
    } else if scale_factor == 0.5 {
        scale_to_gray_2(pix)
    } else if scale_factor > 1.0 / 3.0 {
        let mag = 3.0 * scale_factor;
        let pre = scale_binary(pix, mag, mag)?;
        scale_to_gray_3(&pre)
    } else if scale_factor > 0.25 {
        let mag = 4.0 * scale_factor;
        let pre = scale_binary(pix, mag, mag)?;
        scale_to_gray_4(&pre)
    } else if scale_factor == 0.25 {
        scale_to_gray_4(pix)
    } else if scale_factor > 1.0 / 6.0 {
        let mag = 6.0 * scale_factor;
        let pre = scale_binary(pix, mag, mag)?;
        scale_to_gray_6(&pre)
    } else if scale_factor > 0.125 {
        let mag = 8.0 * scale_factor;
        let pre = scale_binary(pix, mag, mag)?;
        scale_to_gray_8(&pre)
    } else if scale_factor == 0.125 {
        scale_to_gray_8(pix)
    } else if scale_factor > 0.0625 {
        let red = 8.0 * scale_factor;
        let pre = scale_binary(pix, red, red)?;
        scale_to_gray_8(&pre)
    } else if scale_factor == 0.0625 {
        scale_to_gray_16(pix)
    } else {
        let red = 16.0 * scale_factor;
        let pre = scale_to_gray_16(pix)?;
        if red < 0.7 {
            scale_smooth(&pre, red, red)
        } else {
            scale_gray_li(&pre, red, red)
        }
    }
}

/// Faster variant of [`scale_to_gray`] for scale factors in [0.0625, 0.5].
///
/// Binary downscaling is applied first, followed by `scale_to_gray_2`.
/// Quality is slightly lower than [`scale_to_gray`] but computation is faster.
pub fn scale_to_gray_fast(pix: &Pix, scale_factor: f32) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(TransformError::InvalidParameters(
            "scale_to_gray_fast requires 1bpp input".to_string(),
        ));
    }
    if scale_factor <= 0.0 || scale_factor >= 1.0 {
        return Err(TransformError::InvalidParameters(
            "scale_factor must be in (0.0, 1.0)".to_string(),
        ));
    }

    let eps = 0.0001;
    // Exact special cases
    if (scale_factor - 0.5).abs() < eps {
        return scale_to_gray_2(pix);
    } else if (scale_factor - 1.0 / 3.0).abs() < eps {
        return scale_to_gray_3(pix);
    } else if (scale_factor - 0.25).abs() < eps {
        return scale_to_gray_4(pix);
    } else if (scale_factor - 1.0 / 6.0).abs() < eps {
        return scale_to_gray_6(pix);
    } else if (scale_factor - 0.125).abs() < eps {
        return scale_to_gray_8(pix);
    } else if (scale_factor - 0.0625).abs() < eps {
        return scale_to_gray_16(pix);
    }

    if scale_factor > 0.0625 {
        let factor = 2.0 * scale_factor;
        let pre = scale_binary(pix, factor, factor)?;
        scale_to_gray_2(&pre)
    } else {
        let factor = 16.0 * scale_factor;
        let pre = scale_to_gray_16(pix)?;
        if factor < 0.7 {
            scale_smooth(&pre, factor, factor)
        } else {
            scale_gray_li(&pre, factor, factor)
        }
    }
}

/// Scale a 1bpp image to 8bpp by averaging 2×2 pixel blocks.
///
/// Each output pixel represents the fraction of white pixels in a 2×2 source block
/// (0 = all black, 255 = all white).
/// Output dimensions: `(w/2, h/2)`.
pub fn scale_to_gray_2(pix: &Pix) -> TransformResult<Pix> {
    scale_to_gray_n(pix, 2)
}

/// Scale a 1bpp image to 8bpp by averaging 3×3 pixel blocks.
/// Output dimensions: `(w/3, h/3)`.
pub fn scale_to_gray_3(pix: &Pix) -> TransformResult<Pix> {
    scale_to_gray_n(pix, 3)
}

/// Scale a 1bpp image to 8bpp by averaging 4×4 pixel blocks.
/// Output dimensions: `(w/4, h/4)`.
pub fn scale_to_gray_4(pix: &Pix) -> TransformResult<Pix> {
    scale_to_gray_n(pix, 4)
}

/// Scale a 1bpp image to 8bpp by averaging 6×6 pixel blocks.
/// Output dimensions: `(w/6, h/6)`.
pub fn scale_to_gray_6(pix: &Pix) -> TransformResult<Pix> {
    scale_to_gray_n(pix, 6)
}

/// Scale a 1bpp image to 8bpp by averaging 8×8 pixel blocks.
/// Output dimensions: `(w/8, h/8)`.
pub fn scale_to_gray_8(pix: &Pix) -> TransformResult<Pix> {
    scale_to_gray_n(pix, 8)
}

/// Scale a 1bpp image to 8bpp by averaging 16×16 pixel blocks.
/// Output dimensions: `(w/16, h/16)`.
pub fn scale_to_gray_16(pix: &Pix) -> TransformResult<Pix> {
    scale_to_gray_n(pix, 16)
}

/// Generic N×N scale-to-gray for 1bpp input.
/// For each N×N block, counts white (0) pixels and maps to 0-255.
fn scale_to_gray_n(pix: &Pix, n: u32) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(TransformError::InvalidParameters(format!(
            "scale_to_gray_{n} requires 1bpp input"
        )));
    }
    let ws = pix.width();
    let hs = pix.height();
    let wd = ws / n;
    let hd = hs / n;
    if wd == 0 || hd == 0 {
        return Err(TransformError::InvalidParameters(format!(
            "image too small for scale_to_gray_{n}: {ws}x{hs}"
        )));
    }

    let out = Pix::new(wd, hd, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();
    let total = n * n;

    for yd in 0..hd {
        for xd in 0..wd {
            let mut white_count: u32 = 0;
            for dy in 0..n {
                let ys = yd * n + dy;
                for dx in 0..n {
                    let xs = xd * n + dx;
                    if xs < ws && ys < hs {
                        // 1bpp: 0 = white, 1 = black
                        if pix.get_pixel_unchecked(xs, ys) == 0 {
                            white_count += 1;
                        }
                    }
                }
            }
            let gray = ((white_count * 255 + total / 2) / total).min(255);
            out_mut.set_pixel_unchecked(xd, yd, gray);
        }
    }
    Ok(out_mut.into())
}

/// Replicate each pixel `factor` times in each direction.
///
/// All pixel depths (1, 2, 4, 8, 16, 32 bpp) are supported.
/// A factor of 1 returns a copy.
pub fn expand_replicate(pix: &Pix, factor: u32) -> TransformResult<Pix> {
    if factor == 0 {
        return Err(TransformError::InvalidParameters(
            "factor must be >= 1".to_string(),
        ));
    }
    if factor == 1 {
        return Ok(pix.deep_clone());
    }
    let w = pix.width();
    let h = pix.height();
    let new_w = w.checked_mul(factor).ok_or_else(|| {
        TransformError::InvalidParameters("scaled width would overflow u32".to_string())
    })?;
    let new_h = h.checked_mul(factor).ok_or_else(|| {
        TransformError::InvalidParameters("scaled height would overflow u32".to_string())
    })?;
    let depth = pix.depth();

    let out = Pix::new(new_w, new_h, depth)?;
    let mut out_mut = out.try_into_mut().unwrap();
    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }
    out_mut.set_spp(pix.spp());

    for ys in 0..h {
        for xs in 0..w {
            let val = pix.get_pixel_unchecked(xs, ys);
            for dy in 0..factor {
                for dx in 0..factor {
                    out_mut.set_pixel_unchecked(xs * factor + dx, ys * factor + dy, val);
                }
            }
        }
    }
    Ok(out_mut.into())
}

/// Scale a 1bpp binary image using nearest-neighbor sampling.
///
/// This is equivalent to [`scale_by_sampling_with_shift`] with `shift = 0.5`
/// applied to a 1bpp image.
pub fn scale_binary(pix: &Pix, scale_x: f32, scale_y: f32) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(TransformError::InvalidParameters(
            "scale_binary requires 1bpp input".to_string(),
        ));
    }
    scale_by_sampling_with_shift(pix, scale_x, scale_y, 0.5, 0.5)
}

// ────────────────────────────────────────────────────────────────────────────
// Phase 5: Special scaling operations
// ────────────────────────────────────────────────────────────────────────────

/// 2× upscale of a 32bpp color image using linear interpolation.
///
/// Equivalent to `scale_color_li(pix, 2.0, 2.0)`.
pub fn scale_color_2x_li(pix: &Pix) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    scale_color_li(pix, 2.0, 2.0)
}

/// 4× upscale of a 32bpp color image using linear interpolation.
///
/// Equivalent to `scale_color_li(pix, 4.0, 4.0)`.
pub fn scale_color_4x_li(pix: &Pix) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    scale_color_li(pix, 4.0, 4.0)
}

/// 2× upscale of an 8bpp grayscale image using linear interpolation.
///
/// Equivalent to `scale_gray_li(pix, 2.0, 2.0)`.
pub fn scale_gray_2x_li(pix: &Pix) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    if pix.colormap().is_some() {
        return Err(TransformError::InvalidParameters(
            "cmapped input not supported".to_string(),
        ));
    }
    scale_gray_li(pix, 2.0, 2.0)
}

/// 4× upscale of an 8bpp grayscale image using linear interpolation.
///
/// Equivalent to `scale_gray_li(pix, 4.0, 4.0)`.
pub fn scale_gray_4x_li(pix: &Pix) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    if pix.colormap().is_some() {
        return Err(TransformError::InvalidParameters(
            "cmapped input not supported".to_string(),
        ));
    }
    scale_gray_li(pix, 4.0, 4.0)
}

/// 2× upscale of an 8bpp image with LI, then threshold to 1bpp.
///
/// `thresh` must be in `[0, 256]`.  Pixels with value `< thresh` become
/// black (1) and pixels `>= thresh` become white (0).
pub fn scale_gray_2x_li_thresh(pix: &Pix, thresh: i32) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    if pix.colormap().is_some() {
        return Err(TransformError::InvalidParameters(
            "cmapped input not supported".to_string(),
        ));
    }
    if !(0..=256).contains(&thresh) {
        return Err(TransformError::InvalidParameters(format!(
            "thresh must be in [0, 256], got {thresh}"
        )));
    }
    let gray = scale_gray_2x_li(pix)?;
    gray_threshold_to_binary(&gray, thresh)
}

/// 4× upscale of an 8bpp image with LI, then threshold to 1bpp.
///
/// `thresh` must be in `[0, 256]`.  Pixels with value `< thresh` become
/// black (1) and pixels `>= thresh` become white (0).
pub fn scale_gray_4x_li_thresh(pix: &Pix, thresh: i32) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    if pix.colormap().is_some() {
        return Err(TransformError::InvalidParameters(
            "cmapped input not supported".to_string(),
        ));
    }
    if !(0..=256).contains(&thresh) {
        return Err(TransformError::InvalidParameters(format!(
            "thresh must be in [0, 256], got {thresh}"
        )));
    }
    let gray = scale_gray_4x_li(pix)?;
    gray_threshold_to_binary(&gray, thresh)
}

/// 2× upscale of an 8bpp image with LI, then Floyd-Steinberg dither to 1bpp.
pub fn scale_gray_2x_li_dither(pix: &Pix) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    if pix.colormap().is_some() {
        return Err(TransformError::InvalidParameters(
            "cmapped input not supported".to_string(),
        ));
    }
    let gray = scale_gray_2x_li(pix)?;
    gray_floyd_steinberg_dither(&gray)
}

/// 4× upscale of an 8bpp image with LI, then Floyd-Steinberg dither to 1bpp.
pub fn scale_gray_4x_li_dither(pix: &Pix) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    if pix.colormap().is_some() {
        return Err(TransformError::InvalidParameters(
            "cmapped input not supported".to_string(),
        ));
    }
    let gray = scale_gray_4x_li(pix)?;
    gray_floyd_steinberg_dither(&gray)
}

/// Downscale an 8bpp image by selecting the min, max, or max-minus-min value
/// from each `xfact × yfact` block.
///
/// The output dimensions are `(w / xfact, h / yfact)`.
/// Use [`GrayMinMaxMode`] to select which aggregate to compute.
pub fn scale_gray_min_max(
    pix: &Pix,
    xfact: u32,
    yfact: u32,
    mode: GrayMinMaxMode,
) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    if pix.colormap().is_some() {
        return Err(TransformError::InvalidParameters(
            "cmapped input not supported".to_string(),
        ));
    }
    if xfact == 0 || yfact == 0 {
        return Err(TransformError::InvalidParameters(
            "xfact and yfact must be >= 1".to_string(),
        ));
    }

    let ws = pix.width();
    let hs = pix.height();
    let wd = (ws / xfact).max(1);
    let hd = (hs / yfact).max(1);
    let xfact = if ws / xfact == 0 { ws } else { xfact };
    let yfact = if hs / yfact == 0 { hs } else { yfact };

    let out = Pix::new(wd, hd, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for i in 0..hd {
        for j in 0..wd {
            let mut minval: u32 = 255;
            let mut maxval: u32 = 0;
            let compute_min = mode == GrayMinMaxMode::Min || mode == GrayMinMaxMode::MaxDiff;
            let compute_max = mode == GrayMinMaxMode::Max || mode == GrayMinMaxMode::MaxDiff;

            for k in 0..yfact {
                let ys = (yfact * i + k).min(hs - 1);
                for m in 0..xfact {
                    let xs = (xfact * j + m).min(ws - 1);
                    let val = pix.get_pixel_unchecked(xs, ys);
                    if compute_min && val < minval {
                        minval = val;
                    }
                    if compute_max && val > maxval {
                        maxval = val;
                    }
                }
            }

            let out_val = match mode {
                GrayMinMaxMode::Min => minval,
                GrayMinMaxMode::Max => maxval,
                GrayMinMaxMode::MaxDiff => maxval.saturating_sub(minval),
            };
            out_mut.set_pixel_unchecked(j, i, out_val);
        }
    }
    Ok(out_mut.into())
}

/// Apply up to four cascaded 2× rank downscales to an 8bpp image.
///
/// Each level selects the rank-th value (1=darkest, 4=lightest) from a 2×2
/// block.  Use `level = 0` to stop the cascade early.  `level1` must be > 0.
///
/// The maximum reduction is 16× (four stages of 2×).
pub fn scale_gray_rank_cascade(
    pix: &Pix,
    level1: i32,
    level2: i32,
    level3: i32,
    level4: i32,
) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    if pix.colormap().is_some() {
        return Err(TransformError::InvalidParameters(
            "cmapped input not supported".to_string(),
        ));
    }
    if level1 > 4 {
        return Err(TransformError::InvalidParameters(format!(
            "level1 must be in [1, 4], got {level1}"
        )));
    }
    for &lvl in &[level2, level3, level4] {
        if !(0..=4).contains(&lvl) {
            return Err(TransformError::InvalidParameters(format!(
                "levels must be in [0, 4], got {lvl}"
            )));
        }
    }
    if level1 <= 0 {
        return Ok(pix.deep_clone());
    }

    let t1 = scale_gray_rank2(pix, level1)?;
    if level2 <= 0 {
        return Ok(t1);
    }
    let t2 = scale_gray_rank2(&t1, level2)?;
    if level3 <= 0 {
        return Ok(t2);
    }
    let t3 = scale_gray_rank2(&t2, level3)?;
    if level4 <= 0 {
        return Ok(t3);
    }
    scale_gray_rank2(&t3, level4)
}

/// Scale a 1bpp image to 8bpp grayscale using mipmap-based interpolation.
///
/// `scale_factor` must be in `(0, 1)`.  Two nearby scale-to-gray reductions
/// are computed and linearly interpolated to produce the output.
///
/// Note: this method can produce noticeable aliasing; prefer [`scale_to_gray`]
/// for production use.
pub fn scale_to_gray_mipmap(pix: &Pix, scale_factor: f32) -> TransformResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pix.depth()
        )));
    }
    if scale_factor <= 0.0 || scale_factor >= 1.0 {
        return Err(TransformError::InvalidScaleFactor(format!(
            "scale_factor must be in (0, 1), got {scale_factor}"
        )));
    }
    let (w, h) = (pix.width(), pix.height());
    let min_src = w.min(h);
    let min_dst = (min_src as f32 * scale_factor) as u32;
    if min_dst < 2 {
        return Err(TransformError::InvalidScaleFactor(
            "scale_factor too small: destination would be < 2 pixels".to_string(),
        ));
    }

    if scale_factor > 0.5 {
        // Interpolate between full 1→8 and 2× reduced
        let s1 = pix.convert_1_to_8(255, 0)?;
        let s2 = scale_to_gray_2(pix)?;
        scale_mipmap(&s1, &s2, scale_factor)
    } else if scale_factor == 0.5 {
        scale_to_gray_2(pix)
    } else if scale_factor > 0.25 {
        let s1 = scale_to_gray_2(pix)?;
        let s2 = scale_to_gray_4(pix)?;
        scale_mipmap(&s1, &s2, 2.0 * scale_factor)
    } else if scale_factor == 0.25 {
        scale_to_gray_4(pix)
    } else if scale_factor > 0.125 {
        let s1 = scale_to_gray_4(pix)?;
        let s2 = scale_to_gray_8(pix)?;
        scale_mipmap(&s1, &s2, 4.0 * scale_factor)
    } else if scale_factor == 0.125 {
        scale_to_gray_8(pix)
    } else if scale_factor > 0.0625 {
        let s1 = scale_to_gray_8(pix)?;
        let s2 = scale_to_gray_16(pix)?;
        scale_mipmap(&s1, &s2, 8.0 * scale_factor)
    } else if scale_factor == 0.0625 {
        scale_to_gray_16(pix)
    } else {
        // Beyond the pyramid: scale from 16× reduced
        let red = 16.0 * scale_factor;
        let t = scale_to_gray_16(pix)?;
        if red < 0.7 {
            scale_smooth(&t, red, red)
        } else {
            scale_gray_li(&t, red, red)
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Private helpers for Phase 5
// ────────────────────────────────────────────────────────────────────────────

/// Threshold an 8bpp image to 1bpp.
/// Pixels with value `< thresh` → black (1); pixels `>= thresh` → white (0).
/// `thresh = 256` means all pixels become black.
fn gray_threshold_to_binary(pix: &Pix, thresh: i32) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y) as i32;
            let binary: u32 = if val < thresh { 1 } else { 0 };
            out_mut.set_pixel_unchecked(x, y, binary);
        }
    }
    Ok(out_mut.into())
}

/// Floyd-Steinberg error-diffusion dithering from 8bpp to 1bpp.
///
/// In the output 1bpp image: 0 = white, 1 = black.
fn gray_floyd_steinberg_dither(pix: &Pix) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    // Use an f32 buffer to accumulate diffused errors.
    let mut buf: Vec<f32> = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            buf.push(pix.get_pixel_unchecked(x, y) as f32);
        }
    }

    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let old_val = buf[idx].clamp(0.0, 255.0);
            // Quantize: >= 128 → white (0 in 1bpp), < 128 → black (1 in 1bpp)
            let (new_val, binary) = if old_val >= 128.0 {
                (255.0_f32, 0_u32)
            } else {
                (0.0_f32, 1_u32)
            };
            out_mut.set_pixel_unchecked(x, y, binary);
            let error = old_val - new_val;

            // Distribute error to neighbours (Floyd-Steinberg weights)
            if x + 1 < w {
                buf[idx + 1] += error * 7.0 / 16.0;
            }
            if y + 1 < h {
                if x > 0 {
                    buf[((y + 1) * w + x - 1) as usize] += error * 3.0 / 16.0;
                }
                buf[((y + 1) * w + x) as usize] += error * 5.0 / 16.0;
                if x + 1 < w {
                    buf[((y + 1) * w + x + 1) as usize] += error * 1.0 / 16.0;
                }
            }
        }
    }
    Ok(out_mut.into())
}

/// Single-stage 2× grayscale rank reduction.
///
/// `rank` in `[1, 4]`: 1 = darkest, 4 = lightest.
fn scale_gray_rank2(pix: &Pix, rank: i32) -> TransformResult<Pix> {
    if !(1..=4).contains(&rank) {
        return Err(TransformError::InvalidParameters(format!(
            "rank must be in [1, 4], got {rank}"
        )));
    }

    let ws = pix.width();
    let hs = pix.height();
    if ws < 2 || hs < 2 {
        return Err(TransformError::InvalidParameters(
            "image too small for 2x rank reduction".to_string(),
        ));
    }
    let wd = ws / 2;
    let hd = hs / 2;

    let out = Pix::new(wd, hd, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for i in 0..hd {
        for j in 0..wd {
            let mut vals = [
                pix.get_pixel_unchecked(2 * j, 2 * i),
                pix.get_pixel_unchecked(2 * j + 1, 2 * i),
                pix.get_pixel_unchecked(2 * j, 2 * i + 1),
                pix.get_pixel_unchecked(2 * j + 1, 2 * i + 1),
            ];
            vals.sort_unstable();
            // rank 1 = darkest (index 0), rank 4 = lightest (index 3)
            let out_val = vals[(rank - 1) as usize];
            out_mut.set_pixel_unchecked(j, i, out_val);
        }
    }
    Ok(out_mut.into())
}

/// Mipmap interpolation between two 8bpp images at adjacent pyramid levels.
///
/// `pixs1` is the higher-resolution image; `pixs2` is 2× lower resolution.
/// `scale` (in `[0.5, 1.0]`) is the reduction factor relative to `pixs1`.
fn scale_mipmap(pixs1: &Pix, pixs2: &Pix, scale: f32) -> TransformResult<Pix> {
    if pixs1.depth() != PixelDepth::Bit8 || pixs2.depth() != PixelDepth::Bit8 {
        return Err(TransformError::UnsupportedDepth(format!(
            "{:?}",
            pixs1.depth()
        )));
    }
    let ws2 = pixs2.width();
    let hs2 = pixs2.height();
    let wd = (2.0 * scale * ws2 as f32) as u32;
    let hd = (2.0 * scale * hs2 as f32) as u32;
    if wd == 0 || hd == 0 {
        return Err(TransformError::InvalidScaleFactor(
            "mipmap output dimensions would be zero".to_string(),
        ));
    }

    let out = Pix::new(wd, hd, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    // For each output pixel, linearly interpolate between the two source levels.
    // A weight of `scale - 0.5` blends from the high-res to the low-res image.
    let frac = (scale - 0.5) * 2.0; // 0.0 at scale=0.5, 1.0 at scale=1.0

    let ws1 = pixs1.width();
    let hs1 = pixs1.height();

    for y in 0..hd {
        let src_y1 = ((y as f32 / hd as f32) * hs1 as f32) as u32;
        let src_y2 = ((y as f32 / hd as f32) * hs2 as f32) as u32;
        let src_y1 = src_y1.min(hs1 - 1);
        let src_y2 = src_y2.min(hs2 - 1);
        for x in 0..wd {
            let src_x1 = ((x as f32 / wd as f32) * ws1 as f32) as u32;
            let src_x2 = ((x as f32 / wd as f32) * ws2 as f32) as u32;
            let src_x1 = src_x1.min(ws1 - 1);
            let src_x2 = src_x2.min(ws2 - 1);
            let v1 = pixs1.get_pixel_unchecked(src_x1, src_y1) as f32;
            let v2 = pixs2.get_pixel_unchecked(src_x2, src_y2) as f32;
            let val = (v1 * (1.0 - frac) + v2 * frac).round() as u32;
            out_mut.set_pixel_unchecked(x, y, val.min(255));
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

    // --- Phase 4: Scale拡張 - 1bpp→8bpp変換 ---

    fn make_1bpp(w: u32, h: u32, vals: &[(u32, u32, u32)]) -> Pix {
        let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for &(x, y, v) in vals {
            pix_mut.set_pixel_unchecked(x, y, v);
        }
        pix_mut.into()
    }

    #[test]
    fn test_scale_to_gray_2_dims() {
        let pix = make_1bpp(8, 8, &[]);
        let out = scale_to_gray_2(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_to_gray_2_all_white() {
        // 1bpp: all 0 (white) → 8bpp: all 255
        let pix = make_1bpp(8, 8, &[]);
        let out = scale_to_gray_2(&pix).unwrap();
        assert_eq!(out.get_pixel(0, 0).unwrap(), 255);
        assert_eq!(out.get_pixel(3, 3).unwrap(), 255);
    }

    #[test]
    fn test_scale_to_gray_2_all_black() {
        // 1bpp: all 1 (black) → 8bpp: all 0
        let vals: Vec<(u32, u32, u32)> = (0..8)
            .flat_map(|y| (0..8u32).map(move |x| (x, y, 1)))
            .collect();
        let pix = make_1bpp(8, 8, &vals);
        let out = scale_to_gray_2(&pix).unwrap();
        assert_eq!(out.get_pixel(0, 0).unwrap(), 0);
    }

    #[test]
    fn test_scale_to_gray_2_half_black() {
        // Top row of each 2x2 block is black → half pixels black → gray ≈ 127-128
        let vals: Vec<(u32, u32, u32)> = (0..8u32)
            .flat_map(|y| (0..8u32).map(move |x| (x, y, if y % 2 == 0 { 1 } else { 0 })))
            .collect();
        let pix = make_1bpp(8, 8, &vals);
        let out = scale_to_gray_2(&pix).unwrap();
        let v = out.get_pixel(0, 0).unwrap();
        assert!(v == 127 || v == 128, "expected ~128 but got {v}");
    }

    #[test]
    fn test_scale_to_gray_4_dims() {
        let pix = make_1bpp(16, 16, &[]);
        let out = scale_to_gray_4(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_to_gray_8_dims() {
        let pix = make_1bpp(32, 32, &[]);
        let out = scale_to_gray_8(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_to_gray_3_dims() {
        let pix = make_1bpp(12, 12, &[]);
        let out = scale_to_gray_3(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_to_gray_6_dims() {
        let pix = make_1bpp(24, 24, &[]);
        let out = scale_to_gray_6(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_to_gray_16_dims() {
        let pix = make_1bpp(64, 64, &[]);
        let out = scale_to_gray_16(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_to_gray_general_half() {
        // scale=0.5 should give same as scale_to_gray_2
        let pix = make_1bpp(8, 8, &[]);
        let out = scale_to_gray(&pix, 0.5).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_to_gray_general_quarter() {
        let pix = make_1bpp(16, 16, &[]);
        let out = scale_to_gray(&pix, 0.25).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_to_gray_fast_half() {
        let pix = make_1bpp(8, 8, &[]);
        let out = scale_to_gray_fast(&pix, 0.5).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_expand_replicate_8bpp() {
        let pix = Pix::new(2, 2, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 100);
        pix_mut.set_pixel_unchecked(1, 0, 200);
        pix_mut.set_pixel_unchecked(0, 1, 50);
        pix_mut.set_pixel_unchecked(1, 1, 150);
        let pix: Pix = pix_mut.into();
        let out = expand_replicate(&pix, 3).unwrap();
        assert_eq!((out.width(), out.height()), (6, 6));
        // Each 3x3 block must have the same value
        assert_eq!(out.get_pixel(0, 0).unwrap(), 100);
        assert_eq!(out.get_pixel(2, 2).unwrap(), 100);
        assert_eq!(out.get_pixel(3, 0).unwrap(), 200);
        assert_eq!(out.get_pixel(5, 0).unwrap(), 200);
        assert_eq!(out.get_pixel(0, 3).unwrap(), 50);
        assert_eq!(out.get_pixel(3, 3).unwrap(), 150);
    }

    #[test]
    fn test_expand_replicate_factor1() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let out = expand_replicate(&pix, 1).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
    }

    #[test]
    fn test_scale_binary_upscale() {
        let pix = make_1bpp(4, 4, &[(1, 1, 1)]);
        let out = scale_binary(&pix, 2.0, 2.0).unwrap();
        assert_eq!((out.width(), out.height()), (8, 8));
        assert_eq!(out.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_scale_binary_downscale() {
        let pix = make_1bpp(8, 8, &[(1, 1, 1), (4, 4, 1)]);
        let out = scale_binary(&pix, 0.5, 0.5).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit1);
    }

    // --- Phase 5: Scale拡張 - 特殊 ---

    #[test]
    fn test_scale_color_2x_li_dims() {
        let pix = Pix::new(4, 3, PixelDepth::Bit32).unwrap();
        let out = scale_color_2x_li(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (8, 6));
        assert_eq!(out.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_scale_color_4x_li_dims() {
        let pix = Pix::new(4, 3, PixelDepth::Bit32).unwrap();
        let out = scale_color_4x_li(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (16, 12));
        assert_eq!(out.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_scale_gray_2x_li_dims() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let out = scale_gray_2x_li(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (8, 6));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_gray_4x_li_dims() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let out = scale_gray_4x_li(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (16, 12));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_gray_2x_li_thresh_dims() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let out = scale_gray_2x_li_thresh(&pix, 128).unwrap();
        assert_eq!((out.width(), out.height()), (8, 6));
        assert_eq!(out.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_scale_gray_4x_li_thresh_dims() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let out = scale_gray_4x_li_thresh(&pix, 128).unwrap();
        assert_eq!((out.width(), out.height()), (16, 12));
        assert_eq!(out.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_scale_gray_2x_li_dither_dims() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let out = scale_gray_2x_li_dither(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (8, 6));
        assert_eq!(out.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_scale_gray_4x_li_dither_dims() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let out = scale_gray_4x_li_dither(&pix).unwrap();
        assert_eq!((out.width(), out.height()), (16, 12));
        assert_eq!(out.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_scale_gray_min_max_2x_min() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 50);
        pm.set_pixel_unchecked(1, 0, 100);
        pm.set_pixel_unchecked(0, 1, 200);
        pm.set_pixel_unchecked(1, 1, 150);
        let pix: Pix = pm.into();
        let out = scale_gray_min_max(&pix, 2, 2, GrayMinMaxMode::Min).unwrap();
        assert_eq!((out.width(), out.height()), (2, 2));
        assert_eq!(out.get_pixel(0, 0).unwrap(), 50);
    }

    #[test]
    fn test_scale_gray_min_max_2x_max() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 50);
        pm.set_pixel_unchecked(1, 0, 100);
        pm.set_pixel_unchecked(0, 1, 200);
        pm.set_pixel_unchecked(1, 1, 150);
        let pix: Pix = pm.into();
        let out = scale_gray_min_max(&pix, 2, 2, GrayMinMaxMode::Max).unwrap();
        assert_eq!((out.width(), out.height()), (2, 2));
        assert_eq!(out.get_pixel(0, 0).unwrap(), 200);
    }

    #[test]
    fn test_scale_gray_rank_cascade_single() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let out = scale_gray_rank_cascade(&pix, 1, 0, 0, 0).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
    }

    #[test]
    fn test_scale_gray_rank_cascade_double() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let out = scale_gray_rank_cascade(&pix, 1, 1, 0, 0).unwrap();
        assert_eq!((out.width(), out.height()), (2, 2));
    }

    #[test]
    fn test_scale_to_gray_mipmap_half() {
        let pix = make_1bpp(16, 16, &[]);
        let out = scale_to_gray_mipmap(&pix, 0.5).unwrap();
        assert_eq!((out.width(), out.height()), (8, 8));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_scale_to_gray_mipmap_quarter() {
        let pix = make_1bpp(16, 16, &[]);
        let out = scale_to_gray_mipmap(&pix, 0.25).unwrap();
        assert_eq!((out.width(), out.height()), (4, 4));
        assert_eq!(out.depth(), PixelDepth::Bit8);
    }
}
