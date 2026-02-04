//! Rotation and flip operations
//!
//! This module provides:
//! - Orthogonal rotations (90/180/270 degrees)
//! - Arbitrary angle rotations (with bilinear interpolation)
//! - Horizontal and vertical flips

use crate::TransformResult;
use leptonica_core::{Pix, PixMut, PixelDepth};

/// Rotate an image by 90-degree increments
///
/// # Arguments
/// * `pix` - Input image
/// * `quads` - Number of 90-degree clockwise rotations (0-3)
///
/// # Returns
/// The rotated image
pub fn rotate_orth(pix: &Pix, quads: u32) -> TransformResult<Pix> {
    match quads % 4 {
        0 => Ok(pix.deep_clone()),
        1 => rotate_90(pix, true),
        2 => rotate_180(pix),
        3 => rotate_90(pix, false),
        _ => unreachable!(),
    }
}

/// Rotate an image 90 degrees
///
/// # Arguments
/// * `pix` - Input image
/// * `clockwise` - If true, rotate clockwise; otherwise counterclockwise
pub fn rotate_90(pix: &Pix, clockwise: bool) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    // Output dimensions are swapped
    let out_pix = Pix::new(h, w, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    rotate_90_impl(pix, &mut out_mut, clockwise, w, h);

    Ok(out_mut.into())
}

/// Internal implementation of 90 degree rotation
fn rotate_90_impl(src: &Pix, dst: &mut PixMut, clockwise: bool, w: u32, h: u32) {
    for y in 0..h {
        for x in 0..w {
            let val = unsafe { src.get_pixel_unchecked(x, y) };
            let (nx, ny) = if clockwise {
                (h - 1 - y, x)
            } else {
                (y, w - 1 - x)
            };
            unsafe { dst.set_pixel_unchecked(nx, ny, val) };
        }
    }
}

/// Rotate an image 180 degrees
pub fn rotate_180(pix: &Pix) -> TransformResult<Pix> {
    // 180 rotation = horizontal flip + vertical flip
    let flipped_h = flip_lr(pix)?;
    flip_tb(&flipped_h)
}

/// Flip an image left-right (horizontal mirror)
pub fn flip_lr(pix: &Pix) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    for y in 0..h {
        for x in 0..w {
            let val = unsafe { pix.get_pixel_unchecked(x, y) };
            let nx = w - 1 - x;
            unsafe { out_mut.set_pixel_unchecked(nx, y, val) };
        }
    }

    Ok(out_mut.into())
}

/// Flip an image top-bottom (vertical mirror)
pub fn flip_tb(pix: &Pix) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    for y in 0..h {
        for x in 0..w {
            let val = unsafe { pix.get_pixel_unchecked(x, y) };
            let ny = h - 1 - y;
            unsafe { out_mut.set_pixel_unchecked(x, ny, val) };
        }
    }

    Ok(out_mut.into())
}

/// Rotate an image in-place by 180 degrees
pub fn rotate_180_in_place(pix: &mut PixMut) -> TransformResult<()> {
    let w = pix.width();
    let h = pix.height();

    // Swap pixels from opposite corners
    let total_pixels = (w as u64) * (h as u64);
    let half = total_pixels / 2;

    for i in 0..half {
        let x1 = (i % (w as u64)) as u32;
        let y1 = (i / (w as u64)) as u32;
        let x2 = w - 1 - x1;
        let y2 = h - 1 - y1;

        let val1 = unsafe { pix.get_pixel_unchecked(x1, y1) };
        let val2 = unsafe { pix.get_pixel_unchecked(x2, y2) };

        unsafe {
            pix.set_pixel_unchecked(x1, y1, val2);
            pix.set_pixel_unchecked(x2, y2, val1);
        }
    }

    Ok(())
}

// ============================================================================
// Arbitrary angle rotation
// ============================================================================

/// Rotate an image by an arbitrary angle in degrees
///
/// Uses bilinear interpolation for smooth results. The output image size
/// is adjusted to contain the entire rotated image.
///
/// # Arguments
/// * `pix` - Input image
/// * `angle` - Rotation angle in degrees (positive = counterclockwise)
///
/// # Returns
/// The rotated image with white background
///
/// # Example
/// ```no_run
/// use leptonica_transform::rotate_by_angle;
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
/// let rotated = rotate_by_angle(&pix, 45.0).unwrap();
/// ```
pub fn rotate_by_angle(pix: &Pix, angle: f32) -> TransformResult<Pix> {
    let fill_value = get_default_fill_value(pix.depth());
    rotate_by_angle_with_options(pix, angle, fill_value)
}

/// Rotate an image by an arbitrary angle in radians
///
/// Uses bilinear interpolation for smooth results.
///
/// # Arguments
/// * `pix` - Input image
/// * `radians` - Rotation angle in radians (positive = counterclockwise)
pub fn rotate_by_radians(pix: &Pix, radians: f32) -> TransformResult<Pix> {
    let angle = radians.to_degrees();
    rotate_by_angle(pix, angle)
}

/// Rotate an image by an arbitrary angle with custom fill value
///
/// Uses bilinear interpolation for 8-bit and 32-bit images,
/// nearest neighbor for 1-bit images.
///
/// # Arguments
/// * `pix` - Input image
/// * `angle` - Rotation angle in degrees (positive = counterclockwise)
/// * `fill_value` - Value to fill pixels outside the original image
pub fn rotate_by_angle_with_options(
    pix: &Pix,
    angle: f32,
    fill_value: u32,
) -> TransformResult<Pix> {
    // Normalize angle to [0, 360)
    let angle = angle % 360.0;
    let angle = if angle < 0.0 { angle + 360.0 } else { angle };

    // Handle special cases for orthogonal rotations
    if (angle - 0.0).abs() < 0.001 || (angle - 360.0).abs() < 0.001 {
        return Ok(pix.deep_clone());
    }
    if (angle - 90.0).abs() < 0.001 {
        return rotate_90(pix, false); // counterclockwise
    }
    if (angle - 180.0).abs() < 0.001 {
        return rotate_180(pix);
    }
    if (angle - 270.0).abs() < 0.001 {
        return rotate_90(pix, true); // clockwise = 270 CCW
    }

    // General rotation
    let radians = angle.to_radians();
    let cos_a = radians.cos();
    let sin_a = radians.sin();

    let w = pix.width() as f32;
    let h = pix.height() as f32;

    // Calculate new dimensions to fit the rotated image
    let (new_w, new_h) = calculate_rotated_bounds(w, h, cos_a, sin_a);
    let new_w = new_w as u32;
    let new_h = new_h as u32;

    let out_pix = Pix::new(new_w, new_h, pix.depth())?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Copy colormap if present
    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    // Center of original and new images
    let cx_src = w / 2.0;
    let cy_src = h / 2.0;
    let cx_dst = new_w as f32 / 2.0;
    let cy_dst = new_h as f32 / 2.0;

    // Apply rotation with appropriate interpolation
    match pix.depth() {
        PixelDepth::Bit1 => {
            rotate_nearest_neighbor(
                pix,
                &mut out_mut,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
            );
        }
        PixelDepth::Bit8 | PixelDepth::Bit16 | PixelDepth::Bit32 => {
            rotate_bilinear(
                pix,
                &mut out_mut,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
            );
        }
        _ => {
            // For 2-bit and 4-bit, use nearest neighbor
            rotate_nearest_neighbor(
                pix,
                &mut out_mut,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
            );
        }
    }

    Ok(out_mut.into())
}

/// Get default fill value (white) for a given pixel depth
fn get_default_fill_value(depth: PixelDepth) -> u32 {
    match depth {
        PixelDepth::Bit1 => 0, // 0 = white for binary (foreground is black)
        PixelDepth::Bit2 => 3,
        PixelDepth::Bit4 => 15,
        PixelDepth::Bit8 => 255,
        PixelDepth::Bit16 => 65535,
        PixelDepth::Bit32 => 0xFFFFFFFF,
    }
}

/// Calculate the bounding box dimensions after rotation
fn calculate_rotated_bounds(w: f32, h: f32, cos_a: f32, sin_a: f32) -> (f32, f32) {
    // Corners of original image (relative to center)
    let corners = [
        (-w / 2.0, -h / 2.0),
        (w / 2.0, -h / 2.0),
        (w / 2.0, h / 2.0),
        (-w / 2.0, h / 2.0),
    ];

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for (x, y) in corners {
        let rx = x * cos_a - y * sin_a;
        let ry = x * sin_a + y * cos_a;
        min_x = min_x.min(rx);
        max_x = max_x.max(rx);
        min_y = min_y.min(ry);
        max_y = max_y.max(ry);
    }

    ((max_x - min_x).ceil(), (max_y - min_y).ceil())
}

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

/// Nearest neighbor rotation (for 1-bit images)
fn rotate_nearest_neighbor(
    src: &Pix,
    dst: &mut PixMut,
    cos_a: f32,
    sin_a: f32,
    cx_src: f32,
    cy_src: f32,
    cx_dst: f32,
    cy_dst: f32,
) {
    let src_w = src.width() as i32;
    let src_h = src.height() as i32;
    let dst_w = dst.width();
    let dst_h = dst.height();

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            // Transform destination coordinates to source
            let x_rel = dx as f32 - cx_dst;
            let y_rel = dy as f32 - cy_dst;

            // Inverse rotation
            let sx = x_rel * cos_a + y_rel * sin_a + cx_src;
            let sy = -x_rel * sin_a + y_rel * cos_a + cy_src;

            // Nearest neighbor
            let sx_i = sx.round() as i32;
            let sy_i = sy.round() as i32;

            if sx_i >= 0 && sx_i < src_w && sy_i >= 0 && sy_i < src_h {
                let val = unsafe { src.get_pixel_unchecked(sx_i as u32, sy_i as u32) };
                unsafe { dst.set_pixel_unchecked(dx, dy, val) };
            }
        }
    }
}

/// Bilinear interpolation rotation (for 8-bit and 32-bit images)
fn rotate_bilinear(
    src: &Pix,
    dst: &mut PixMut,
    cos_a: f32,
    sin_a: f32,
    cx_src: f32,
    cy_src: f32,
    cx_dst: f32,
    cy_dst: f32,
) {
    let src_w = src.width() as i32;
    let src_h = src.height() as i32;
    let dst_w = dst.width();
    let dst_h = dst.height();
    let depth = src.depth();

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            // Transform destination coordinates to source
            let x_rel = dx as f32 - cx_dst;
            let y_rel = dy as f32 - cy_dst;

            // Inverse rotation
            let sx = x_rel * cos_a + y_rel * sin_a + cx_src;
            let sy = -x_rel * sin_a + y_rel * cos_a + cy_src;

            // Bilinear interpolation
            let x0 = sx.floor() as i32;
            let y0 = sy.floor() as i32;
            let x1 = x0 + 1;
            let y1 = y0 + 1;

            if x0 >= 0 && x1 < src_w && y0 >= 0 && y1 < src_h {
                let fx = sx - x0 as f32;
                let fy = sy - y0 as f32;

                let val = interpolate_pixel(
                    src, depth, x0 as u32, y0 as u32, x1 as u32, y1 as u32, fx, fy,
                );
                unsafe { dst.set_pixel_unchecked(dx, dy, val) };
            } else if x0 >= -1 && x1 <= src_w && y0 >= -1 && y1 <= src_h {
                // Edge case: partially outside, use available pixels
                let val = interpolate_edge_pixel(src, depth, x0, y0, x1, y1, sx, sy, src_w, src_h);
                unsafe { dst.set_pixel_unchecked(dx, dy, val) };
            }
        }
    }
}

/// Bilinear interpolation of a single pixel
fn interpolate_pixel(
    src: &Pix,
    depth: PixelDepth,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    fx: f32,
    fy: f32,
) -> u32 {
    let p00 = unsafe { src.get_pixel_unchecked(x0, y0) };
    let p10 = unsafe { src.get_pixel_unchecked(x1, y0) };
    let p01 = unsafe { src.get_pixel_unchecked(x0, y1) };
    let p11 = unsafe { src.get_pixel_unchecked(x1, y1) };

    match depth {
        PixelDepth::Bit32 => {
            // Interpolate each channel separately
            let r = interpolate_channel(
                (p00 >> 24) & 0xFF,
                (p10 >> 24) & 0xFF,
                (p01 >> 24) & 0xFF,
                (p11 >> 24) & 0xFF,
                fx,
                fy,
            );
            let g = interpolate_channel(
                (p00 >> 16) & 0xFF,
                (p10 >> 16) & 0xFF,
                (p01 >> 16) & 0xFF,
                (p11 >> 16) & 0xFF,
                fx,
                fy,
            );
            let b = interpolate_channel(
                (p00 >> 8) & 0xFF,
                (p10 >> 8) & 0xFF,
                (p01 >> 8) & 0xFF,
                (p11 >> 8) & 0xFF,
                fx,
                fy,
            );
            let a = interpolate_channel(p00 & 0xFF, p10 & 0xFF, p01 & 0xFF, p11 & 0xFF, fx, fy);
            (r << 24) | (g << 16) | (b << 8) | a
        }
        _ => {
            // Single channel interpolation
            interpolate_channel(p00, p10, p01, p11, fx, fy)
        }
    }
}

/// Interpolate a single channel value
fn interpolate_channel(p00: u32, p10: u32, p01: u32, p11: u32, fx: f32, fy: f32) -> u32 {
    let top = p00 as f32 * (1.0 - fx) + p10 as f32 * fx;
    let bottom = p01 as f32 * (1.0 - fx) + p11 as f32 * fx;
    let result = top * (1.0 - fy) + bottom * fy;
    result.round() as u32
}

/// Handle edge pixels with partial interpolation
fn interpolate_edge_pixel(
    src: &Pix,
    depth: PixelDepth,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    sx: f32,
    sy: f32,
    src_w: i32,
    src_h: i32,
) -> u32 {
    // Get available pixels, using nearest valid pixel for out-of-bounds
    let clamp_x = |x: i32| x.clamp(0, src_w - 1) as u32;
    let clamp_y = |y: i32| y.clamp(0, src_h - 1) as u32;

    let p00 = unsafe { src.get_pixel_unchecked(clamp_x(x0), clamp_y(y0)) };
    let p10 = unsafe { src.get_pixel_unchecked(clamp_x(x1), clamp_y(y0)) };
    let p01 = unsafe { src.get_pixel_unchecked(clamp_x(x0), clamp_y(y1)) };
    let p11 = unsafe { src.get_pixel_unchecked(clamp_x(x1), clamp_y(y1)) };

    let fx = sx - x0 as f32;
    let fy = sy - y0 as f32;

    match depth {
        PixelDepth::Bit32 => {
            let r = interpolate_channel(
                (p00 >> 24) & 0xFF,
                (p10 >> 24) & 0xFF,
                (p01 >> 24) & 0xFF,
                (p11 >> 24) & 0xFF,
                fx,
                fy,
            );
            let g = interpolate_channel(
                (p00 >> 16) & 0xFF,
                (p10 >> 16) & 0xFF,
                (p01 >> 16) & 0xFF,
                (p11 >> 16) & 0xFF,
                fx,
                fy,
            );
            let b = interpolate_channel(
                (p00 >> 8) & 0xFF,
                (p10 >> 8) & 0xFF,
                (p01 >> 8) & 0xFF,
                (p11 >> 8) & 0xFF,
                fx,
                fy,
            );
            let a = interpolate_channel(p00 & 0xFF, p10 & 0xFF, p01 & 0xFF, p11 & 0xFF, fx, fy);
            (r << 24) | (g << 16) | (b << 8) | a
        }
        _ => interpolate_channel(p00, p10, p01, p11, fx, fy),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{PixelDepth, color};

    #[test]
    fn test_rotate_90_clockwise() {
        // 2x3 grayscale image
        let pix = Pix::new(2, 3, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set pattern:
        // [1, 2]
        // [3, 4]
        // [5, 6]
        unsafe {
            pix_mut.set_pixel_unchecked(0, 0, 1);
            pix_mut.set_pixel_unchecked(1, 0, 2);
            pix_mut.set_pixel_unchecked(0, 1, 3);
            pix_mut.set_pixel_unchecked(1, 1, 4);
            pix_mut.set_pixel_unchecked(0, 2, 5);
            pix_mut.set_pixel_unchecked(1, 2, 6);
        }

        let pix: Pix = pix_mut.into();
        let rotated = rotate_90(&pix, true).unwrap();

        // After 90 CW rotation: 3x2
        // [5, 3, 1]
        // [6, 4, 2]
        assert_eq!((rotated.width(), rotated.height()), (3, 2));
        assert_eq!(unsafe { rotated.get_pixel_unchecked(0, 0) }, 5);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(1, 0) }, 3);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(2, 0) }, 1);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(0, 1) }, 6);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(1, 1) }, 4);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(2, 1) }, 2);
    }

    #[test]
    fn test_rotate_90_counterclockwise() {
        // 2x3 grayscale image
        let pix = Pix::new(2, 3, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        unsafe {
            pix_mut.set_pixel_unchecked(0, 0, 1);
            pix_mut.set_pixel_unchecked(1, 0, 2);
            pix_mut.set_pixel_unchecked(0, 1, 3);
            pix_mut.set_pixel_unchecked(1, 1, 4);
            pix_mut.set_pixel_unchecked(0, 2, 5);
            pix_mut.set_pixel_unchecked(1, 2, 6);
        }

        let pix: Pix = pix_mut.into();
        let rotated = rotate_90(&pix, false).unwrap();

        // After 90 CCW rotation: 3x2
        // [2, 4, 6]
        // [1, 3, 5]
        assert_eq!((rotated.width(), rotated.height()), (3, 2));
        assert_eq!(unsafe { rotated.get_pixel_unchecked(0, 0) }, 2);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(1, 0) }, 4);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(2, 0) }, 6);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(0, 1) }, 1);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(1, 1) }, 3);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(2, 1) }, 5);
    }

    #[test]
    fn test_rotate_180() {
        let pix = Pix::new(2, 2, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // [1, 2]
        // [3, 4]
        unsafe {
            pix_mut.set_pixel_unchecked(0, 0, 1);
            pix_mut.set_pixel_unchecked(1, 0, 2);
            pix_mut.set_pixel_unchecked(0, 1, 3);
            pix_mut.set_pixel_unchecked(1, 1, 4);
        }

        let pix: Pix = pix_mut.into();
        let rotated = rotate_180(&pix).unwrap();

        // After 180 rotation:
        // [4, 3]
        // [2, 1]
        assert_eq!(unsafe { rotated.get_pixel_unchecked(0, 0) }, 4);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(1, 0) }, 3);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(0, 1) }, 2);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(1, 1) }, 1);
    }

    #[test]
    fn test_flip_lr() {
        let pix = Pix::new(3, 2, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // [1, 2, 3]
        // [4, 5, 6]
        unsafe {
            pix_mut.set_pixel_unchecked(0, 0, 1);
            pix_mut.set_pixel_unchecked(1, 0, 2);
            pix_mut.set_pixel_unchecked(2, 0, 3);
            pix_mut.set_pixel_unchecked(0, 1, 4);
            pix_mut.set_pixel_unchecked(1, 1, 5);
            pix_mut.set_pixel_unchecked(2, 1, 6);
        }

        let pix: Pix = pix_mut.into();
        let flipped = flip_lr(&pix).unwrap();

        // After LR flip:
        // [3, 2, 1]
        // [6, 5, 4]
        assert_eq!(unsafe { flipped.get_pixel_unchecked(0, 0) }, 3);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(1, 0) }, 2);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(2, 0) }, 1);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(0, 1) }, 6);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(1, 1) }, 5);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(2, 1) }, 4);
    }

    #[test]
    fn test_flip_tb() {
        let pix = Pix::new(2, 3, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // [1, 2]
        // [3, 4]
        // [5, 6]
        unsafe {
            pix_mut.set_pixel_unchecked(0, 0, 1);
            pix_mut.set_pixel_unchecked(1, 0, 2);
            pix_mut.set_pixel_unchecked(0, 1, 3);
            pix_mut.set_pixel_unchecked(1, 1, 4);
            pix_mut.set_pixel_unchecked(0, 2, 5);
            pix_mut.set_pixel_unchecked(1, 2, 6);
        }

        let pix: Pix = pix_mut.into();
        let flipped = flip_tb(&pix).unwrap();

        // After TB flip:
        // [5, 6]
        // [3, 4]
        // [1, 2]
        assert_eq!(unsafe { flipped.get_pixel_unchecked(0, 0) }, 5);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(1, 0) }, 6);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(0, 1) }, 3);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(1, 1) }, 4);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(0, 2) }, 1);
        assert_eq!(unsafe { flipped.get_pixel_unchecked(1, 2) }, 2);
    }

    #[test]
    fn test_rotate_orth_all_quadrants() {
        let pix = Pix::new(2, 2, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        unsafe {
            pix_mut.set_pixel_unchecked(0, 0, 1);
            pix_mut.set_pixel_unchecked(1, 0, 2);
            pix_mut.set_pixel_unchecked(0, 1, 3);
            pix_mut.set_pixel_unchecked(1, 1, 4);
        }

        let pix: Pix = pix_mut.into();

        // 0 quads = no rotation
        let r0 = rotate_orth(&pix, 0).unwrap();
        assert_eq!(unsafe { r0.get_pixel_unchecked(0, 0) }, 1);

        // 4 quads = back to original (mod 4)
        let r4 = rotate_orth(&pix, 4).unwrap();
        assert_eq!(unsafe { r4.get_pixel_unchecked(0, 0) }, 1);
    }

    #[test]
    fn test_rotate_32bpp() {
        let pix = Pix::new(2, 2, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let red = color::compose_rgb(255, 0, 0);
        let green = color::compose_rgb(0, 255, 0);
        let blue = color::compose_rgb(0, 0, 255);
        let white = color::compose_rgb(255, 255, 255);

        // [R, G]
        // [B, W]
        unsafe {
            pix_mut.set_pixel_unchecked(0, 0, red);
            pix_mut.set_pixel_unchecked(1, 0, green);
            pix_mut.set_pixel_unchecked(0, 1, blue);
            pix_mut.set_pixel_unchecked(1, 1, white);
        }

        let pix: Pix = pix_mut.into();
        let rotated = rotate_90(&pix, true).unwrap();

        // After 90 CW:
        // [B, R]
        // [W, G]
        assert_eq!(unsafe { rotated.get_pixel_unchecked(0, 0) }, blue);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(1, 0) }, red);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(0, 1) }, white);
        assert_eq!(unsafe { rotated.get_pixel_unchecked(1, 1) }, green);
    }

    // ========================================================================
    // Arbitrary angle rotation tests
    // ========================================================================

    #[test]
    fn test_rotate_by_angle_zero() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let rotated = rotate_by_angle(&pix, 0.0).unwrap();
        assert_eq!(rotated.width(), 10);
        assert_eq!(rotated.height(), 10);
    }

    #[test]
    fn test_rotate_by_angle_90() {
        let pix = Pix::new(20, 10, PixelDepth::Bit8).unwrap();
        let rotated = rotate_by_angle(&pix, 90.0).unwrap();
        // 90 degree rotation swaps dimensions
        assert_eq!(rotated.width(), 10);
        assert_eq!(rotated.height(), 20);
    }

    #[test]
    fn test_rotate_by_angle_180() {
        let pix = Pix::new(20, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(0, 0, 100) };
        let pix: Pix = pix_mut.into();

        let rotated = rotate_by_angle(&pix, 180.0).unwrap();
        assert_eq!(rotated.width(), 20);
        assert_eq!(rotated.height(), 10);
        // Top-left becomes bottom-right
        assert_eq!(unsafe { rotated.get_pixel_unchecked(19, 9) }, 100);
    }

    #[test]
    fn test_rotate_by_angle_45() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let rotated = rotate_by_angle(&pix, 45.0).unwrap();

        // 45 degree rotation should expand dimensions by ~sqrt(2)
        let expected_size = (100.0 * 2.0_f32.sqrt()).ceil() as u32;
        assert!(rotated.width() >= expected_size - 2 && rotated.width() <= expected_size + 2);
        assert!(rotated.height() >= expected_size - 2 && rotated.height() <= expected_size + 2);
    }

    #[test]
    fn test_rotate_by_radians() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let rotated_deg = rotate_by_angle(&pix, 45.0).unwrap();
        let rotated_rad = rotate_by_radians(&pix, std::f32::consts::FRAC_PI_4).unwrap();

        // Both should produce same dimensions
        assert_eq!(rotated_deg.width(), rotated_rad.width());
        assert_eq!(rotated_deg.height(), rotated_rad.height());
    }

    #[test]
    fn test_rotate_negative_angle() {
        let pix = Pix::new(100, 50, PixelDepth::Bit8).unwrap();
        let rotated_pos = rotate_by_angle(&pix, 30.0).unwrap();
        let rotated_neg = rotate_by_angle(&pix, -330.0).unwrap(); // -330 = +30

        assert_eq!(rotated_pos.width(), rotated_neg.width());
        assert_eq!(rotated_pos.height(), rotated_neg.height());
    }

    #[test]
    fn test_rotate_1bpp() {
        // 1-bit image uses nearest neighbor
        let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Set some pixels (1 = black in 1bpp)
        for i in 10..40 {
            unsafe { pix_mut.set_pixel_unchecked(i, 25, 1) };
        }
        let pix: Pix = pix_mut.into();

        let rotated = rotate_by_angle(&pix, 15.0).unwrap();
        assert!(rotated.width() > 50);
        assert!(rotated.height() > 50);
    }

    #[test]
    fn test_calculate_rotated_bounds() {
        // 45 degree rotation of a square
        let cos_45 = std::f32::consts::FRAC_PI_4.cos();
        let sin_45 = std::f32::consts::FRAC_PI_4.sin();
        let (w, h) = calculate_rotated_bounds(100.0, 100.0, cos_45, sin_45);

        // Diagonal of 100x100 square is ~141.4
        let expected = 100.0 * 2.0_f32.sqrt();
        assert!((w - expected).abs() < 2.0);
        assert!((h - expected).abs() < 2.0);
    }
}
