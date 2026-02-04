//! Orthogonal rotation and flip operations
//!
//! This module provides 90/180/270 degree rotations and horizontal/vertical flips.

use crate::TransformResult;
use leptonica_core::{Pix, PixMut};

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
}
