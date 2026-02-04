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

            let val = unsafe { pix.get_pixel_unchecked(src_x, src_y) };
            unsafe { out_mut.set_pixel_unchecked(x, y, val) };
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
            let p00 = unsafe { pix.get_pixel_unchecked(x0, y0) };
            let p10 = unsafe { pix.get_pixel_unchecked(x1, y0) };
            let p01 = unsafe { pix.get_pixel_unchecked(x0, y1) };
            let p11 = unsafe { pix.get_pixel_unchecked(x1, y1) };

            // Interpolate each channel
            let result = interpolate_color(p00, p10, p01, p11, fx, fy);
            unsafe { out_mut.set_pixel_unchecked(x, y, result) };
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
            let p00 = unsafe { pix.get_pixel_unchecked(x0, y0) } as f32;
            let p10 = unsafe { pix.get_pixel_unchecked(x1, y0) } as f32;
            let p01 = unsafe { pix.get_pixel_unchecked(x0, y1) } as f32;
            let p11 = unsafe { pix.get_pixel_unchecked(x1, y1) } as f32;

            // Bilinear interpolation
            let top = p00 * (1.0 - fx) + p10 * fx;
            let bottom = p01 * (1.0 - fx) + p11 * fx;
            let result = (top * (1.0 - fy) + bottom * fy).round() as u32;

            unsafe { out_mut.set_pixel_unchecked(x, y, result) };
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
            unsafe { out_mut.set_pixel_unchecked(x, y, pixel) };
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
            unsafe { out_mut.set_pixel_unchecked(x, y, val as u32) };
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
            let pixel = unsafe { pix.get_pixel_unchecked(sx, sy) };
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
            let val = unsafe { pix.get_pixel_unchecked(sx, sy) };
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
        unsafe {
            pix_mut.set_pixel_unchecked(0, 0, 10);
            pix_mut.set_pixel_unchecked(1, 0, 20);
            pix_mut.set_pixel_unchecked(0, 1, 30);
            pix_mut.set_pixel_unchecked(1, 1, 40);
        }

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
                unsafe { pix_mut.set_pixel_unchecked(x, y, x + y * 4) };
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

        unsafe {
            pix_mut.set_pixel_unchecked(0, 0, black);
            pix_mut.set_pixel_unchecked(1, 0, white);
            pix_mut.set_pixel_unchecked(0, 1, white);
            pix_mut.set_pixel_unchecked(1, 1, black);
        }

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
                unsafe { pix_mut.set_pixel_unchecked(x, y, val) };
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
                unsafe { pix_mut.set_pixel_unchecked(x, y, val) };
            }
        }

        let pix: Pix = pix_mut.into();
        let scaled = scale(&pix, 2.0, 2.0, ScaleMethod::Sampling).unwrap();

        assert_eq!((scaled.width(), scaled.height()), (8, 8));
        assert_eq!(scaled.depth(), PixelDepth::Bit1);
    }
}
