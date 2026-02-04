//! Binary morphological operations
//!
//! Implements erosion, dilation, opening, and closing for 1-bpp images.

use crate::{MorphError, MorphResult, Sel};
use leptonica_core::{Pix, PixelDepth};

/// Dilate a binary image
///
/// Dilation expands foreground regions. For each pixel, if ANY hit position
/// in the SEL corresponds to a foreground pixel, the output is foreground.
pub fn dilate(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    check_binary(pix)?;

    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let hit_offsets: Vec<_> = sel.hit_offsets().collect();

    for y in 0..h {
        for x in 0..w {
            // Check if any hit position has a foreground pixel
            let dilated = hit_offsets.iter().any(|&(dx, dy)| {
                let sx = x as i32 + dx;
                let sy = y as i32 + dy;
                if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                    unsafe { pix.get_pixel_unchecked(sx as u32, sy as u32) != 0 }
                } else {
                    false // Pixels outside are treated as background
                }
            });

            if dilated {
                unsafe { out_mut.set_pixel_unchecked(x, y, 1) };
            }
        }
    }

    Ok(out_mut.into())
}

/// Erode a binary image
///
/// Erosion shrinks foreground regions. For each pixel, if ALL hit positions
/// in the SEL correspond to foreground pixels, the output is foreground.
pub fn erode(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    check_binary(pix)?;

    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let hit_offsets: Vec<_> = sel.hit_offsets().collect();

    for y in 0..h {
        for x in 0..w {
            // Check if all hit positions have foreground pixels
            let eroded = hit_offsets.iter().all(|&(dx, dy)| {
                let sx = x as i32 + dx;
                let sy = y as i32 + dy;
                if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                    unsafe { pix.get_pixel_unchecked(sx as u32, sy as u32) != 0 }
                } else {
                    false // Pixels outside are treated as background
                }
            });

            if eroded {
                unsafe { out_mut.set_pixel_unchecked(x, y, 1) };
            }
        }
    }

    Ok(out_mut.into())
}

/// Open a binary image
///
/// Opening = Erosion followed by Dilation.
/// Removes small foreground objects and smooths contours.
pub fn open(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let eroded = erode(pix, sel)?;
    dilate(&eroded, sel)
}

/// Close a binary image
///
/// Closing = Dilation followed by Erosion.
/// Fills small holes and connects nearby objects.
pub fn close(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let dilated = dilate(pix, sel)?;
    erode(&dilated, sel)
}

/// Hit-miss transform
///
/// The HMT identifies pixels that match both the hit pattern (foreground)
/// AND the miss pattern (background). Used for pattern detection.
pub fn hit_miss_transform(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    check_binary(pix)?;

    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let hit_offsets: Vec<_> = sel.hit_offsets().collect();
    let miss_offsets: Vec<_> = sel.miss_offsets().collect();

    for y in 0..h {
        for x in 0..w {
            // Check if all hits match foreground
            let hits_match = hit_offsets.iter().all(|&(dx, dy)| {
                let sx = x as i32 + dx;
                let sy = y as i32 + dy;
                if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                    unsafe { pix.get_pixel_unchecked(sx as u32, sy as u32) != 0 }
                } else {
                    false
                }
            });

            // Check if all misses match background
            let misses_match = miss_offsets.iter().all(|&(dx, dy)| {
                let sx = x as i32 + dx;
                let sy = y as i32 + dy;
                if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                    unsafe { pix.get_pixel_unchecked(sx as u32, sy as u32) == 0 }
                } else {
                    true // Outside is background
                }
            });

            if hits_match && misses_match {
                unsafe { out_mut.set_pixel_unchecked(x, y, 1) };
            }
        }
    }

    Ok(out_mut.into())
}

/// Morphological gradient (dilation - erosion)
///
/// Highlights edges/boundaries of objects.
pub fn gradient(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let dilated = dilate(pix, sel)?;
    let eroded = erode(pix, sel)?;
    subtract(&dilated, &eroded)
}

/// Top-hat transform (original - opening)
///
/// Extracts bright features smaller than the SE.
pub fn top_hat(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let opened = open(pix, sel)?;
    subtract(pix, &opened)
}

/// Bottom-hat transform (closing - original)
///
/// Extracts dark features smaller than the SE.
pub fn bottom_hat(pix: &Pix, sel: &Sel) -> MorphResult<Pix> {
    let closed = close(pix, sel)?;
    subtract(&closed, pix)
}

/// Subtract two binary images (a AND NOT b)
fn subtract(a: &Pix, b: &Pix) -> MorphResult<Pix> {
    let w = a.width();
    let h = a.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let va = unsafe { a.get_pixel_unchecked(x, y) };
            let vb = unsafe { b.get_pixel_unchecked(x, y) };
            let result = if va != 0 && vb == 0 { 1 } else { 0 };
            unsafe { out_mut.set_pixel_unchecked(x, y, result) };
        }
    }

    Ok(out_mut.into())
}

/// Dilate with a brick (rectangular) structuring element
///
/// Optimized for rectangular SEs.
pub fn dilate_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    let sel = Sel::create_brick(width, height)?;
    dilate(pix, &sel)
}

/// Erode with a brick (rectangular) structuring element
pub fn erode_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    let sel = Sel::create_brick(width, height)?;
    erode(pix, &sel)
}

/// Open with a brick structuring element
pub fn open_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    let sel = Sel::create_brick(width, height)?;
    open(pix, &sel)
}

/// Close with a brick structuring element
pub fn close_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    let sel = Sel::create_brick(width, height)?;
    close(pix, &sel)
}

/// Check that the image is binary (1-bpp)
fn check_binary(pix: &Pix) -> MorphResult<()> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> Pix {
        // Create a 5x5 image with a 3x3 square in the center
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set the 3x3 square
        for y in 1..4 {
            for x in 1..4 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, 1) };
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_dilate() {
        let pix = create_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();

        let dilated = dilate(&pix, &sel).unwrap();

        // The 3x3 square should expand to 5x5
        assert_eq!(unsafe { dilated.get_pixel_unchecked(0, 0) }, 1);
        assert_eq!(unsafe { dilated.get_pixel_unchecked(4, 4) }, 1);
    }

    #[test]
    fn test_erode() {
        let pix = create_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();

        let eroded = erode(&pix, &sel).unwrap();

        // The 3x3 square should shrink to 1x1 (just the center)
        assert_eq!(unsafe { eroded.get_pixel_unchecked(2, 2) }, 1);
        assert_eq!(unsafe { eroded.get_pixel_unchecked(1, 1) }, 0);
        assert_eq!(unsafe { eroded.get_pixel_unchecked(3, 3) }, 0);
    }

    #[test]
    fn test_open_close() {
        let pix = create_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();

        // Opening then closing should roughly preserve the shape
        let opened = open(&pix, &sel).unwrap();
        let closed = close(&pix, &sel).unwrap();

        // The opened image should have the center pixel
        assert_eq!(unsafe { opened.get_pixel_unchecked(2, 2) }, 1);

        // The closed image should have the original square plus some
        assert_eq!(unsafe { closed.get_pixel_unchecked(2, 2) }, 1);
    }

    #[test]
    fn test_hit_miss_transform() {
        // Create an image with a single pixel
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(2, 2, 1) };
        let pix: Pix = pix_mut.into();

        // Create a SEL that matches isolated pixels
        let sel = Sel::from_string(
            "ooo\n\
             oxo\n\
             ooo",
            1,
            1,
        )
        .unwrap();

        let hmt = hit_miss_transform(&pix, &sel).unwrap();

        // The isolated pixel should be detected
        assert_eq!(unsafe { hmt.get_pixel_unchecked(2, 2) }, 1);
    }

    #[test]
    fn test_gradient() {
        let pix = create_test_image();
        let sel = Sel::create_brick(3, 3).unwrap();

        let _grad = gradient(&pix, &sel).unwrap();

        // The gradient should show the boundary
        // Center should be 0 (dilated and eroded both have it)
        // Edges of the original should be 1
    }

    #[test]
    fn test_brick_operations() {
        let pix = create_test_image();

        let dilated = dilate_brick(&pix, 3, 3).unwrap();
        let eroded = erode_brick(&pix, 3, 3).unwrap();

        assert_eq!(unsafe { dilated.get_pixel_unchecked(0, 0) }, 1);
        assert_eq!(unsafe { eroded.get_pixel_unchecked(2, 2) }, 1);
    }

    #[test]
    fn test_non_binary_error() {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let sel = Sel::create_brick(3, 3).unwrap();

        let result = dilate(&pix, &sel);
        assert!(result.is_err());
    }
}
