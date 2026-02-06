//! Apply disparity arrays to images
//!
//! This module provides functions to apply the built disparity models
//! to dewarp images.

use crate::{RecogError, RecogResult};
use leptonica_core::{FPix, Pix, PixelDepth};

use super::types::Dewarp;

/// Apply vertical disparity to an image
///
/// This shifts each pixel vertically according to the disparity array
/// to straighten text lines.
///
/// # Arguments
///
/// * `pix` - Input image (1, 8, or 32 bpp)
/// * `v_disparity` - Full resolution vertical disparity array
/// * `gray_in` - Gray value for pixels brought in from outside (0-255)
///
/// # Returns
///
/// The dewarped image
pub fn apply_vertical_disparity(pix: &Pix, v_disparity: &FPix, gray_in: u8) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    // Check dimensions
    let (dw, dh) = v_disparity.dimensions();
    if dw < w || dh < h {
        return Err(RecogError::InvalidParameter(format!(
            "disparity array too small: {}x{} for image {}x{}",
            dw, dh, w, h
        )));
    }

    // Create output image
    let result = Pix::new(w, h, depth)?;
    let mut result_mut = result.try_into_mut().unwrap();

    // Set all pixels to gray_in initially
    match depth {
        PixelDepth::Bit1 => {
            // For binary, gray_in > 127 means white (0), otherwise black (1)
            if gray_in > 127 {
                // Leave as is (default is 0 = white in binary)
            } else {
                // Set all to 1 (black)
                for y in 0..h {
                    for x in 0..w {
                        unsafe { result_mut.set_pixel_unchecked(x, y, 1) };
                    }
                }
            }
        }
        PixelDepth::Bit8 => {
            // Set all pixels to gray_in
            for y in 0..h {
                for x in 0..w {
                    unsafe { result_mut.set_pixel_unchecked(x, y, gray_in as u32) };
                }
            }
        }
        PixelDepth::Bit32 => {
            // Set all pixels to gray RGB
            let gray_val =
                ((gray_in as u32) << 24) | ((gray_in as u32) << 16) | ((gray_in as u32) << 8) | 255;
            for y in 0..h {
                for x in 0..w {
                    unsafe { result_mut.set_pixel_unchecked(x, y, gray_val) };
                }
            }
        }
        _ => {
            return Err(RecogError::UnsupportedDepth {
                expected: "1, 8, or 32 bpp",
                actual: depth.bits(),
            });
        }
    }

    // Apply disparity
    for y in 0..h {
        for x in 0..w {
            // Get disparity at this position
            let disparity = v_disparity.get_pixel(x, y).unwrap_or(0.0);

            // Source y position (shift in opposite direction)
            let src_y_f = y as f32 - disparity;
            let src_y = (src_y_f + 0.5) as i32;

            // Check bounds
            if src_y >= 0 && src_y < h as i32 {
                let src_y = src_y as u32;
                let val = unsafe { pix.get_pixel_unchecked(x, src_y) };
                unsafe { result_mut.set_pixel_unchecked(x, y, val) };
            }
            // If out of bounds, pixel stays at gray_in
        }
    }

    Ok(result_mut.into())
}

/// Apply horizontal disparity to an image
///
/// This shifts each pixel horizontally according to the disparity array
/// to correct perspective distortion.
///
/// # Arguments
///
/// * `pix` - Input image (1, 8, or 32 bpp)
/// * `h_disparity` - Full resolution horizontal disparity array
/// * `gray_in` - Gray value for pixels brought in from outside (0-255)
///
/// # Returns
///
/// The dewarped image
pub fn apply_horizontal_disparity(pix: &Pix, h_disparity: &FPix, gray_in: u8) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    // Check dimensions
    let (dw, dh) = h_disparity.dimensions();
    if dw < w || dh < h {
        return Err(RecogError::InvalidParameter(format!(
            "disparity array too small: {}x{} for image {}x{}",
            dw, dh, w, h
        )));
    }

    // Create output image
    let result = Pix::new(w, h, depth)?;
    let mut result_mut = result.try_into_mut().unwrap();

    // Set all pixels to gray_in initially
    match depth {
        PixelDepth::Bit1 => {
            if gray_in <= 127 {
                for y in 0..h {
                    for x in 0..w {
                        unsafe { result_mut.set_pixel_unchecked(x, y, 1) };
                    }
                }
            }
        }
        PixelDepth::Bit8 => {
            for y in 0..h {
                for x in 0..w {
                    unsafe { result_mut.set_pixel_unchecked(x, y, gray_in as u32) };
                }
            }
        }
        PixelDepth::Bit32 => {
            let gray_val =
                ((gray_in as u32) << 24) | ((gray_in as u32) << 16) | ((gray_in as u32) << 8) | 255;
            for y in 0..h {
                for x in 0..w {
                    unsafe { result_mut.set_pixel_unchecked(x, y, gray_val) };
                }
            }
        }
        _ => {
            return Err(RecogError::UnsupportedDepth {
                expected: "1, 8, or 32 bpp",
                actual: depth.bits(),
            });
        }
    }

    // Apply disparity
    for y in 0..h {
        for x in 0..w {
            // Get disparity at this position
            let disparity = h_disparity.get_pixel(x, y).unwrap_or(0.0);

            // Source x position (shift in opposite direction)
            let src_x_f = x as f32 - disparity;
            let src_x = (src_x_f + 0.5) as i32;

            // Check bounds
            if src_x >= 0 && src_x < w as i32 {
                let src_x = src_x as u32;
                let val = unsafe { pix.get_pixel_unchecked(src_x, y) };
                unsafe { result_mut.set_pixel_unchecked(x, y, val) };
            }
            // If out of bounds, pixel stays at gray_in
        }
    }

    Ok(result_mut.into())
}

/// Apply both vertical and horizontal disparity to an image
///
/// This is the main function for dewarping. It applies vertical disparity
/// first (to straighten text lines), then horizontal disparity (to correct
/// perspective).
///
/// # Arguments
///
/// * `pix` - Input image (1, 8, or 32 bpp)
/// * `dewarp` - Dewarp model with populated full resolution arrays
/// * `gray_in` - Gray value for pixels brought in from outside
///
/// # Returns
///
/// The dewarped image
pub fn apply_disparity(pix: &Pix, dewarp: &Dewarp, gray_in: u8) -> RecogResult<Pix> {
    // Apply vertical disparity if available
    let pix_v = if let Some(ref v_disp) = dewarp.full_v_disparity {
        if dewarp.v_valid {
            apply_vertical_disparity(pix, v_disp, gray_in)?
        } else {
            pix.deep_clone()
        }
    } else {
        return Err(RecogError::InvalidParameter(
            "no vertical disparity available".to_string(),
        ));
    };

    // Apply horizontal disparity if available and valid
    if let Some(ref h_disp) = dewarp.full_h_disparity {
        if dewarp.h_valid {
            apply_horizontal_disparity(&pix_v, h_disp, gray_in)
        } else {
            Ok(pix_v)
        }
    } else {
        Ok(pix_v)
    }
}

/// Estimate the required disparity from text line analysis
///
/// This provides a quick estimate without building a full model.
pub fn estimate_disparity_magnitude(lines: &[super::types::TextLine]) -> f32 {
    if lines.is_empty() {
        return 0.0;
    }

    let mut max_deviation = 0.0f32;

    for line in lines {
        if line.points.len() < 3 {
            continue;
        }

        // Find the line's y-span (max - min y)
        let min_y = line.points.iter().map(|(_, y)| *y).fold(f32::MAX, f32::min);
        let max_y = line.points.iter().map(|(_, y)| *y).fold(f32::MIN, f32::max);

        let deviation = max_y - min_y;
        if deviation > max_deviation {
            max_deviation = deviation;
        }
    }

    max_deviation
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::FPix;

    #[test]
    fn test_apply_vertical_disparity_no_shift() {
        // Create a simple 8-bit image
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        for y in 0..10 {
            for x in 0..10 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, x + y * 10) };
            }
        }
        let pix: Pix = pix_mut.into();

        // Create zero disparity
        let v_disparity = FPix::new(10, 10).unwrap();

        let result = apply_vertical_disparity(&pix, &v_disparity, 255).unwrap();

        // Should be identical to input
        for y in 0..10 {
            for x in 0..10 {
                let orig = unsafe { pix.get_pixel_unchecked(x, y) };
                let dewarped = unsafe { result.get_pixel_unchecked(x, y) };
                assert_eq!(orig, dewarped);
            }
        }
    }

    #[test]
    fn test_apply_vertical_disparity_uniform_shift() {
        // Create a simple 8-bit image with horizontal stripes
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        for y in 0..10 {
            for x in 0..10 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, y * 25) };
            }
        }
        let pix: Pix = pix_mut.into();

        // Create uniform disparity of 1 pixel down
        let mut v_disparity = FPix::new(10, 10).unwrap();
        v_disparity.set_all(1.0); // Shift source up by 1

        let result = apply_vertical_disparity(&pix, &v_disparity, 255).unwrap();

        // Row y in result should have value from row y-1 in source
        // (shifted up means src_y = y - 1)
        for y in 1..10 {
            for x in 0..10 {
                let expected = (y - 1) * 25;
                let actual = unsafe { result.get_pixel_unchecked(x, y) };
                assert_eq!(expected, actual, "Mismatch at ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn test_apply_horizontal_disparity_no_shift() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();
        for y in 0..10 {
            for x in 0..10 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, x * 25) };
            }
        }
        let pix: Pix = pix_mut.into();

        let h_disparity = FPix::new(10, 10).unwrap();
        let result = apply_horizontal_disparity(&pix, &h_disparity, 255).unwrap();

        for y in 0..10 {
            for x in 0..10 {
                let orig = unsafe { pix.get_pixel_unchecked(x, y) };
                let dewarped = unsafe { result.get_pixel_unchecked(x, y) };
                assert_eq!(orig, dewarped);
            }
        }
    }

    #[test]
    fn test_apply_disparity_dimension_check() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let small_disparity = FPix::new(50, 50).unwrap();

        let result = apply_vertical_disparity(&pix, &small_disparity, 255);
        assert!(result.is_err());
    }

    #[test]
    fn test_estimate_disparity_magnitude() {
        use super::super::types::TextLine;

        // Straight line
        let straight = TextLine::new(vec![(0.0, 50.0), (50.0, 50.0), (100.0, 50.0)]);
        assert_eq!(estimate_disparity_magnitude(&[straight]), 0.0);

        // Curved line
        let curved = TextLine::new(vec![(0.0, 50.0), (50.0, 55.0), (100.0, 50.0)]);
        assert!((estimate_disparity_magnitude(&[curved]) - 5.0).abs() < 0.01);
    }
}
