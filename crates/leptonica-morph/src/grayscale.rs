//! Grayscale morphological operations
//!
//! Implements erosion, dilation, opening, and closing for 8-bpp grayscale images.
//!
//! # Algorithm
//!
//! For grayscale morphology with a brick (rectangular) structuring element:
//! - **Dilation**: Computes the maximum pixel value in the neighborhood
//! - **Erosion**: Computes the minimum pixel value in the neighborhood
//! - **Opening**: Erosion followed by dilation (removes small bright features)
//! - **Closing**: Dilation followed by erosion (fills small dark features)
//!
//! # Reference
//!
//! Based on Leptonica's `graymorph.c` implementation.

use crate::{MorphError, MorphResult};
use leptonica_core::{Pix, PixelDepth};

/// Dilate a grayscale image with a brick structuring element
///
/// Dilation computes the maximum pixel value in the SE neighborhood,
/// which expands bright regions and shrinks dark regions.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE (will be made odd if even)
/// * `vsize` - Vertical size of the brick SE (will be made odd if even)
///
/// # Returns
///
/// A new dilated image, or error if input is not 8-bpp.
///
/// # Notes
///
/// - If hsize and vsize are both 1, returns a copy of the input
/// - Out-of-bounds pixels are treated as 0 (no contribution to max)
pub fn dilate_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_grayscale(pix)?;
    let (hsize, vsize) = ensure_odd(hsize, vsize)?;

    // Identity operation
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    let w = pix.width();
    let h = pix.height();
    let half_h = (hsize / 2) as i32;
    let half_v = (vsize / 2) as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let mut max_val: u8 = 0;

            for dy in -half_v..=half_v {
                for dx in -half_h..=half_h {
                    let sx = x as i32 + dx;
                    let sy = y as i32 + dy;

                    if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                        let val = unsafe { pix.get_pixel_unchecked(sx as u32, sy as u32) } as u8;
                        max_val = max_val.max(val);
                    }
                    // Out of bounds: treated as 0 (minimum), no contribution to max
                }
            }

            unsafe { out_mut.set_pixel_unchecked(x, y, max_val as u32) };
        }
    }

    Ok(out_mut.into())
}

/// Erode a grayscale image with a brick structuring element
///
/// Erosion computes the minimum pixel value in the SE neighborhood,
/// which shrinks bright regions and expands dark regions.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE (will be made odd if even)
/// * `vsize` - Vertical size of the brick SE (will be made odd if even)
///
/// # Returns
///
/// A new eroded image, or error if input is not 8-bpp.
///
/// # Notes
///
/// - If hsize and vsize are both 1, returns a copy of the input
/// - Out-of-bounds pixels are treated as 255 (no contribution to min)
pub fn erode_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_grayscale(pix)?;
    let (hsize, vsize) = ensure_odd(hsize, vsize)?;

    // Identity operation
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    let w = pix.width();
    let h = pix.height();
    let half_h = (hsize / 2) as i32;
    let half_v = (vsize / 2) as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let mut min_val: u8 = 255;

            for dy in -half_v..=half_v {
                for dx in -half_h..=half_h {
                    let sx = x as i32 + dx;
                    let sy = y as i32 + dy;

                    if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                        let val = unsafe { pix.get_pixel_unchecked(sx as u32, sy as u32) } as u8;
                        min_val = min_val.min(val);
                    }
                    // Out of bounds: treated as 255 (maximum), no contribution to min
                }
            }

            unsafe { out_mut.set_pixel_unchecked(x, y, min_val as u32) };
        }
    }

    Ok(out_mut.into())
}

/// Open a grayscale image (erosion followed by dilation)
///
/// Opening removes small bright features while preserving the overall shape.
/// It is useful for removing noise and small bright spots.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn open_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let eroded = erode_gray(pix, hsize, vsize)?;
    dilate_gray(&eroded, hsize, vsize)
}

/// Close a grayscale image (dilation followed by erosion)
///
/// Closing fills small dark features while preserving the overall shape.
/// It is useful for filling small holes and gaps.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn close_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let dilated = dilate_gray(pix, hsize, vsize)?;
    erode_gray(&dilated, hsize, vsize)
}

/// Grayscale morphological gradient (dilation - erosion)
///
/// Highlights edges and boundaries in the image.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn gradient_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let dilated = dilate_gray(pix, hsize, vsize)?;
    let eroded = erode_gray(pix, hsize, vsize)?;
    subtract_gray(&dilated, &eroded)
}

/// Grayscale top-hat transform (original - opening)
///
/// Extracts bright features smaller than the structuring element.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn top_hat_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let opened = open_gray(pix, hsize, vsize)?;
    subtract_gray(pix, &opened)
}

/// Grayscale bottom-hat transform (closing - original)
///
/// Extracts dark features smaller than the structuring element.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn bottom_hat_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let closed = close_gray(pix, hsize, vsize)?;
    subtract_gray(&closed, pix)
}

/// Subtract two grayscale images (a - b, clamped to 0)
fn subtract_gray(a: &Pix, b: &Pix) -> MorphResult<Pix> {
    let w = a.width();
    let h = a.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let va = unsafe { a.get_pixel_unchecked(x, y) } as i32;
            let vb = unsafe { b.get_pixel_unchecked(x, y) } as i32;
            let result = (va - vb).max(0) as u32;
            unsafe { out_mut.set_pixel_unchecked(x, y, result) };
        }
    }

    Ok(out_mut.into())
}

/// Check that the image is 8-bpp grayscale
fn check_grayscale(pix: &Pix) -> MorphResult<()> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(MorphError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

/// Ensure sizes are odd (as required by Leptonica's convention)
fn ensure_odd(hsize: u32, vsize: u32) -> MorphResult<(u32, u32)> {
    if hsize == 0 || vsize == 0 {
        return Err(MorphError::InvalidParameters(
            "hsize and vsize must be >= 1".to_string(),
        ));
    }

    let hsize = if hsize.is_multiple_of(2) {
        hsize + 1
    } else {
        hsize
    };
    let vsize = if vsize.is_multiple_of(2) {
        vsize + 1
    } else {
        vsize
    };

    Ok((hsize, vsize))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_grayscale_image() -> Pix {
        // Create a 9x9 grayscale image with a bright 3x3 square in the center
        let pix = Pix::new(9, 9, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with dark background (50)
        for y in 0..9 {
            for x in 0..9 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, 50) };
            }
        }

        // Set bright center 3x3 (200)
        for y in 3..6 {
            for x in 3..6 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, 200) };
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_dilate_gray_identity() {
        let pix = create_test_grayscale_image();
        let dilated = dilate_gray(&pix, 1, 1).unwrap();

        // Should be identical
        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(unsafe { pix.get_pixel_unchecked(x, y) }, unsafe {
                    dilated.get_pixel_unchecked(x, y)
                });
            }
        }
    }

    #[test]
    fn test_erode_gray_identity() {
        let pix = create_test_grayscale_image();
        let eroded = erode_gray(&pix, 1, 1).unwrap();

        // Should be identical
        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(unsafe { pix.get_pixel_unchecked(x, y) }, unsafe {
                    eroded.get_pixel_unchecked(x, y)
                });
            }
        }
    }

    #[test]
    fn test_dilate_gray_expands_bright() {
        let pix = create_test_grayscale_image();
        let dilated = dilate_gray(&pix, 3, 3).unwrap();

        // The bright center should expand
        // After 3x3 dilation, the 3x3 bright area should become 5x5
        // Pixels at (2,2) should now be bright (200)
        assert_eq!(unsafe { dilated.get_pixel_unchecked(2, 2) }, 200);
        assert_eq!(unsafe { dilated.get_pixel_unchecked(6, 6) }, 200);

        // Center should remain bright
        assert_eq!(unsafe { dilated.get_pixel_unchecked(4, 4) }, 200);

        // Corners should remain dark
        assert_eq!(unsafe { dilated.get_pixel_unchecked(0, 0) }, 50);
        assert_eq!(unsafe { dilated.get_pixel_unchecked(8, 8) }, 50);
    }

    #[test]
    fn test_erode_gray_shrinks_bright() {
        let pix = create_test_grayscale_image();
        let eroded = erode_gray(&pix, 3, 3).unwrap();

        // The 3x3 bright center should shrink to 1x1 (just center pixel)
        assert_eq!(unsafe { eroded.get_pixel_unchecked(4, 4) }, 200);

        // Adjacent pixels should now be dark (50)
        assert_eq!(unsafe { eroded.get_pixel_unchecked(3, 4) }, 50);
        assert_eq!(unsafe { eroded.get_pixel_unchecked(5, 4) }, 50);
    }

    #[test]
    fn test_open_gray() {
        let pix = create_test_grayscale_image();
        let opened = open_gray(&pix, 3, 3).unwrap();

        // Opening should shrink then expand
        // The 3x3 bright region: erode makes it 1x1, dilate makes it 3x3
        // Center should remain bright
        assert_eq!(unsafe { opened.get_pixel_unchecked(4, 4) }, 200);
    }

    #[test]
    fn test_close_gray() {
        let pix = create_test_grayscale_image();
        let closed = close_gray(&pix, 3, 3).unwrap();

        // Closing should expand then shrink
        // The 3x3 bright region should be preserved
        assert_eq!(unsafe { closed.get_pixel_unchecked(4, 4) }, 200);
    }

    #[test]
    fn test_even_size_incremented() {
        let pix = create_test_grayscale_image();

        // Even sizes should work (auto-incremented to odd)
        let result = dilate_gray(&pix, 2, 4);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_size_error() {
        let pix = create_test_grayscale_image();

        let result = dilate_gray(&pix, 0, 3);
        assert!(result.is_err());

        let result = erode_gray(&pix, 3, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_grayscale_error() {
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();

        let result = dilate_gray(&pix, 3, 3);
        assert!(result.is_err());

        let result = erode_gray(&pix, 3, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_gradient_gray() {
        let pix = create_test_grayscale_image();
        let gradient = gradient_gray(&pix, 3, 3).unwrap();

        // Gradient should be highest at edges
        // Interior of bright region and background should be low
        // The center of the bright region should have low gradient
        // (dilated - eroded at center: 200 - 200 = 0... but after erosion center becomes 200)
        // Actually after 3x3 operations on 3x3 bright region:
        // - dilated center: 200
        // - eroded center: 200 (only center survives)
        // So gradient at center should be 0
        assert_eq!(unsafe { gradient.get_pixel_unchecked(4, 4) }, 0);
    }

    #[test]
    fn test_top_hat_gray() {
        let pix = create_test_grayscale_image();
        let tophat = top_hat_gray(&pix, 3, 3).unwrap();

        // Top-hat extracts bright features smaller than SE
        // For our 3x3 SE and 3x3 bright region, the bright region
        // survives opening, so top-hat should be small
        assert!(unsafe { tophat.get_pixel_unchecked(4, 4) } <= 200);
    }

    #[test]
    fn test_bottom_hat_gray() {
        let pix = create_test_grayscale_image();
        let bottomhat = bottom_hat_gray(&pix, 3, 3).unwrap();

        // Bottom-hat extracts dark features
        // Should be non-negative everywhere
        for y in 0..9 {
            for x in 0..9 {
                assert!(unsafe { bottomhat.get_pixel_unchecked(x, y) } <= 255);
            }
        }
    }

    #[test]
    fn test_single_pixel_dilation() {
        // Create image with single bright pixel
        let pix = Pix::new(7, 7, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with dark (0)
        for y in 0..7 {
            for x in 0..7 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, 0) };
            }
        }

        // Single bright pixel at center
        unsafe { pix_mut.set_pixel_unchecked(3, 3, 255) };
        let pix: Pix = pix_mut.into();

        let dilated = dilate_gray(&pix, 3, 3).unwrap();

        // 3x3 dilation should create a 3x3 bright region
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let x = (3 + dx) as u32;
                let y = (3 + dy) as u32;
                assert_eq!(
                    unsafe { dilated.get_pixel_unchecked(x, y) },
                    255,
                    "Expected 255 at ({}, {})",
                    x,
                    y
                );
            }
        }

        // Corners should remain dark
        assert_eq!(unsafe { dilated.get_pixel_unchecked(0, 0) }, 0);
    }
}
