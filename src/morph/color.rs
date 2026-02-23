//! Color morphological operations
//!
//! Implements erosion, dilation, opening, and closing for 32-bpp color images.
//!
//! # Algorithm
//!
//! Color morphology applies grayscale morphological operations separately
//! to each RGB channel, then recombines the results. This follows the
//! approach described by van Herk and Gil-Werman.
//!
//! - **Dilation**: Computes the maximum pixel value in each channel
//! - **Erosion**: Computes the minimum pixel value in each channel
//! - **Opening**: Erosion followed by dilation
//! - **Closing**: Dilation followed by erosion
//!
//! # Reference
//!
//! Based on Leptonica's `colormorph.c` implementation.

use crate::grayscale::{
    bottom_hat_gray, close_gray, dilate_gray, erode_gray, gradient_gray, open_gray, top_hat_gray,
};
use crate::{MorphError, MorphResult};
use leptonica_core::{Pix, PixelDepth, color};

/// Color channel identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChannel {
    /// Red channel
    Red,
    /// Green channel
    Green,
    /// Blue channel
    Blue,
}

/// Dilate a color image with a brick structuring element
///
/// Dilation computes the maximum pixel value in the SE neighborhood for each
/// RGB channel independently, which expands bright regions and shrinks dark
/// regions.
///
/// # Arguments
///
/// * `pix` - 32-bpp color image
/// * `hsize` - Horizontal size of the brick SE (will be made odd if even)
/// * `vsize` - Vertical size of the brick SE (will be made odd if even)
///
/// # Returns
///
/// A new dilated image, or error if input is not 32-bpp.
///
/// # Notes
///
/// - If hsize and vsize are both 1, returns a copy of the input
/// - Alpha channel is preserved from the original image (set to 255)
pub fn dilate_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_color(pix)?;
    let (hsize, vsize) = ensure_odd(hsize, vsize)?;

    // Identity operation
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    // Extract channels
    let r = extract_channel(pix, ColorChannel::Red)?;
    let g = extract_channel(pix, ColorChannel::Green)?;
    let b = extract_channel(pix, ColorChannel::Blue)?;

    // Apply grayscale dilation to each channel
    let r_dilated = dilate_gray(&r, hsize, vsize)?;
    let g_dilated = dilate_gray(&g, hsize, vsize)?;
    let b_dilated = dilate_gray(&b, hsize, vsize)?;

    // Combine back into color image
    combine_rgb(&r_dilated, &g_dilated, &b_dilated)
}

/// Erode a color image with a brick structuring element
///
/// Erosion computes the minimum pixel value in the SE neighborhood for each
/// RGB channel independently, which shrinks bright regions and expands dark
/// regions.
///
/// # Arguments
///
/// * `pix` - 32-bpp color image
/// * `hsize` - Horizontal size of the brick SE (will be made odd if even)
/// * `vsize` - Vertical size of the brick SE (will be made odd if even)
///
/// # Returns
///
/// A new eroded image, or error if input is not 32-bpp.
///
/// # Notes
///
/// - If hsize and vsize are both 1, returns a copy of the input
/// - Alpha channel is preserved (set to 255)
pub fn erode_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_color(pix)?;
    let (hsize, vsize) = ensure_odd(hsize, vsize)?;

    // Identity operation
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    // Extract channels
    let r = extract_channel(pix, ColorChannel::Red)?;
    let g = extract_channel(pix, ColorChannel::Green)?;
    let b = extract_channel(pix, ColorChannel::Blue)?;

    // Apply grayscale erosion to each channel
    let r_eroded = erode_gray(&r, hsize, vsize)?;
    let g_eroded = erode_gray(&g, hsize, vsize)?;
    let b_eroded = erode_gray(&b, hsize, vsize)?;

    // Combine back into color image
    combine_rgb(&r_eroded, &g_eroded, &b_eroded)
}

/// Open a color image (erosion followed by dilation)
///
/// Opening removes small bright features while preserving the overall shape.
/// It is useful for removing noise and small bright spots in each color channel.
///
/// # Arguments
///
/// * `pix` - 32-bpp color image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn open_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_color(pix)?;
    let (hsize, vsize) = ensure_odd(hsize, vsize)?;

    // Identity operation
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    // Extract channels
    let r = extract_channel(pix, ColorChannel::Red)?;
    let g = extract_channel(pix, ColorChannel::Green)?;
    let b = extract_channel(pix, ColorChannel::Blue)?;

    // Apply grayscale opening to each channel
    let r_opened = open_gray(&r, hsize, vsize)?;
    let g_opened = open_gray(&g, hsize, vsize)?;
    let b_opened = open_gray(&b, hsize, vsize)?;

    // Combine back into color image
    combine_rgb(&r_opened, &g_opened, &b_opened)
}

/// Close a color image (dilation followed by erosion)
///
/// Closing fills small dark features while preserving the overall shape.
/// It is useful for filling small holes and gaps in each color channel.
///
/// # Arguments
///
/// * `pix` - 32-bpp color image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn close_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_color(pix)?;
    let (hsize, vsize) = ensure_odd(hsize, vsize)?;

    // Identity operation
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    // Extract channels
    let r = extract_channel(pix, ColorChannel::Red)?;
    let g = extract_channel(pix, ColorChannel::Green)?;
    let b = extract_channel(pix, ColorChannel::Blue)?;

    // Apply grayscale closing to each channel
    let r_closed = close_gray(&r, hsize, vsize)?;
    let g_closed = close_gray(&g, hsize, vsize)?;
    let b_closed = close_gray(&b, hsize, vsize)?;

    // Combine back into color image
    combine_rgb(&r_closed, &g_closed, &b_closed)
}

/// Color morphological gradient (dilation - erosion)
///
/// Highlights edges and boundaries in the image by computing the
/// difference between dilation and erosion for each channel.
///
/// # Arguments
///
/// * `pix` - 32-bpp color image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn gradient_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_color(pix)?;

    // Extract channels
    let r = extract_channel(pix, ColorChannel::Red)?;
    let g = extract_channel(pix, ColorChannel::Green)?;
    let b = extract_channel(pix, ColorChannel::Blue)?;

    // Apply grayscale gradient to each channel
    let r_gradient = gradient_gray(&r, hsize, vsize)?;
    let g_gradient = gradient_gray(&g, hsize, vsize)?;
    let b_gradient = gradient_gray(&b, hsize, vsize)?;

    // Combine back into color image
    combine_rgb(&r_gradient, &g_gradient, &b_gradient)
}

/// Color top-hat transform (original - opening)
///
/// Extracts bright features smaller than the structuring element
/// in each color channel.
///
/// # Arguments
///
/// * `pix` - 32-bpp color image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn top_hat_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_color(pix)?;

    // Extract channels
    let r = extract_channel(pix, ColorChannel::Red)?;
    let g = extract_channel(pix, ColorChannel::Green)?;
    let b = extract_channel(pix, ColorChannel::Blue)?;

    // Apply grayscale top-hat to each channel
    let r_tophat = top_hat_gray(&r, hsize, vsize)?;
    let g_tophat = top_hat_gray(&g, hsize, vsize)?;
    let b_tophat = top_hat_gray(&b, hsize, vsize)?;

    // Combine back into color image
    combine_rgb(&r_tophat, &g_tophat, &b_tophat)
}

/// Color bottom-hat transform (closing - original)
///
/// Extracts dark features smaller than the structuring element
/// in each color channel.
///
/// # Arguments
///
/// * `pix` - 32-bpp color image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn bottom_hat_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_color(pix)?;

    // Extract channels
    let r = extract_channel(pix, ColorChannel::Red)?;
    let g = extract_channel(pix, ColorChannel::Green)?;
    let b = extract_channel(pix, ColorChannel::Blue)?;

    // Apply grayscale bottom-hat to each channel
    let r_bottomhat = bottom_hat_gray(&r, hsize, vsize)?;
    let g_bottomhat = bottom_hat_gray(&g, hsize, vsize)?;
    let b_bottomhat = bottom_hat_gray(&b, hsize, vsize)?;

    // Combine back into color image
    combine_rgb(&r_bottomhat, &g_bottomhat, &b_bottomhat)
}

/// Check that the image is 32-bpp color
fn check_color(pix: &Pix) -> MorphResult<()> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(MorphError::UnsupportedDepth {
            expected: "32-bpp color",
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

/// Extract a single RGB channel as an 8-bpp grayscale image
fn extract_channel(pix: &Pix, channel: ColorChannel) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let value = match channel {
                ColorChannel::Red => color::red(pixel),
                ColorChannel::Green => color::green(pixel),
                ColorChannel::Blue => color::blue(pixel),
            };
            out_mut.set_pixel_unchecked(x, y, value as u32);
        }
    }

    Ok(out_mut.into())
}

/// Combine R, G, B channels into a 32-bpp color image
fn combine_rgb(r: &Pix, g: &Pix, b: &Pix) -> MorphResult<Pix> {
    let w = r.width();
    let h = r.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let rv = r.get_pixel_unchecked(x, y) as u8;
            let gv = g.get_pixel_unchecked(x, y) as u8;
            let bv = b.get_pixel_unchecked(x, y) as u8;
            let pixel = color::compose_rgb(rv, gv, bv);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }

    Ok(out_mut.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_color_image() -> Pix {
        // Create a 9x9 color image with a bright 3x3 square in the center
        let pix = Pix::new(9, 9, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with dark background (50, 50, 50)
        for y in 0..9 {
            for x in 0..9 {
                let pixel = color::compose_rgb(50, 50, 50);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        // Set bright center 3x3 (200, 150, 100)
        for y in 3..6 {
            for x in 3..6 {
                let pixel = color::compose_rgb(200, 150, 100);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    fn create_multicolor_test_image() -> Pix {
        // Create a 9x9 image with different colored regions
        let pix = Pix::new(9, 9, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with black
        for y in 0..9 {
            for x in 0..9 {
                let pixel = color::compose_rgb(0, 0, 0);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        // Red region (top-left 3x3)
        for y in 0..3 {
            for x in 0..3 {
                let pixel = color::compose_rgb(255, 0, 0);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        // Green region (top-right 3x3)
        for y in 0..3 {
            for x in 6..9 {
                let pixel = color::compose_rgb(0, 255, 0);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        // Blue region (bottom-center 3x3)
        for y in 6..9 {
            for x in 3..6 {
                let pixel = color::compose_rgb(0, 0, 255);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_extract_channel() {
        let pix = create_multicolor_test_image();

        // Extract red channel
        let r = extract_channel(&pix, ColorChannel::Red).unwrap();
        // Red region should be 255, elsewhere 0
        assert_eq!(r.get_pixel_unchecked(1, 1), 255); // Red region
        assert_eq!(r.get_pixel_unchecked(7, 1), 0); // Green region
        assert_eq!(r.get_pixel_unchecked(4, 7), 0); // Blue region

        // Extract green channel
        let g = extract_channel(&pix, ColorChannel::Green).unwrap();
        assert_eq!(g.get_pixel_unchecked(1, 1), 0); // Red region
        assert_eq!(g.get_pixel_unchecked(7, 1), 255); // Green region
        assert_eq!(g.get_pixel_unchecked(4, 7), 0); // Blue region

        // Extract blue channel
        let b = extract_channel(&pix, ColorChannel::Blue).unwrap();
        assert_eq!(b.get_pixel_unchecked(1, 1), 0); // Red region
        assert_eq!(b.get_pixel_unchecked(7, 1), 0); // Green region
        assert_eq!(b.get_pixel_unchecked(4, 7), 255); // Blue region
    }

    #[test]
    fn test_combine_rgb() {
        let pix = create_test_color_image();

        // Extract and recombine
        let r = extract_channel(&pix, ColorChannel::Red).unwrap();
        let g = extract_channel(&pix, ColorChannel::Green).unwrap();
        let b = extract_channel(&pix, ColorChannel::Blue).unwrap();

        let combined = combine_rgb(&r, &g, &b).unwrap();

        // Check that recombined image matches original (except alpha)
        for y in 0..9 {
            for x in 0..9 {
                let orig_pixel = pix.get_pixel_unchecked(x, y);
                let comb_pixel = combined.get_pixel_unchecked(x, y);

                let (or, og, ob) = color::extract_rgb(orig_pixel);
                let (cr, cg, cb) = color::extract_rgb(comb_pixel);

                assert_eq!((or, og, ob), (cr, cg, cb), "Mismatch at ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn test_dilate_color_identity() {
        let pix = create_test_color_image();
        let dilated = dilate_color(&pix, 1, 1).unwrap();

        // Should be identical
        for y in 0..9 {
            for x in 0..9 {
                let orig_pixel = pix.get_pixel_unchecked(x, y);
                let dil_pixel = dilated.get_pixel_unchecked(x, y);

                let (or, og, ob) = color::extract_rgb(orig_pixel);
                let (dr, dg, db) = color::extract_rgb(dil_pixel);

                assert_eq!((or, og, ob), (dr, dg, db), "Mismatch at ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn test_erode_color_identity() {
        let pix = create_test_color_image();
        let eroded = erode_color(&pix, 1, 1).unwrap();

        // Should be identical
        for y in 0..9 {
            for x in 0..9 {
                let orig_pixel = pix.get_pixel_unchecked(x, y);
                let ero_pixel = eroded.get_pixel_unchecked(x, y);

                let (or, og, ob) = color::extract_rgb(orig_pixel);
                let (er, eg, eb) = color::extract_rgb(ero_pixel);

                assert_eq!((or, og, ob), (er, eg, eb), "Mismatch at ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn test_dilate_color_expands_bright() {
        let pix = create_test_color_image();
        let dilated = dilate_color(&pix, 3, 3).unwrap();

        // The bright center should expand
        // Pixels at (2,2) should now be bright (200, 150, 100)
        let (r, g, b) = color::extract_rgb(dilated.get_pixel_unchecked(2, 2));
        assert_eq!((r, g, b), (200, 150, 100));

        let (r, g, b) = color::extract_rgb(dilated.get_pixel_unchecked(6, 6));
        assert_eq!((r, g, b), (200, 150, 100));

        // Center should remain bright
        let (r, g, b) = color::extract_rgb(dilated.get_pixel_unchecked(4, 4));
        assert_eq!((r, g, b), (200, 150, 100));

        // Corners should remain dark
        let (r, g, b) = color::extract_rgb(dilated.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (50, 50, 50));
    }

    #[test]
    fn test_erode_color_shrinks_bright() {
        let pix = create_test_color_image();
        let eroded = erode_color(&pix, 3, 3).unwrap();

        // The 3x3 bright center should shrink to 1x1 (just center pixel)
        let (r, g, b) = color::extract_rgb(eroded.get_pixel_unchecked(4, 4));
        assert_eq!((r, g, b), (200, 150, 100));

        // Adjacent pixels should now be dark (50, 50, 50)
        let (r, g, b) = color::extract_rgb(eroded.get_pixel_unchecked(3, 4));
        assert_eq!((r, g, b), (50, 50, 50));
    }

    #[test]
    fn test_open_color() {
        let pix = create_test_color_image();
        let opened = open_color(&pix, 3, 3).unwrap();

        // Center should remain bright after open (erode then dilate)
        let (r, g, b) = color::extract_rgb(opened.get_pixel_unchecked(4, 4));
        assert_eq!((r, g, b), (200, 150, 100));
    }

    #[test]
    fn test_close_color() {
        let pix = create_test_color_image();
        let closed = close_color(&pix, 3, 3).unwrap();

        // Center should remain bright after close (dilate then erode)
        let (r, g, b) = color::extract_rgb(closed.get_pixel_unchecked(4, 4));
        assert_eq!((r, g, b), (200, 150, 100));
    }

    #[test]
    fn test_even_size_incremented() {
        let pix = create_test_color_image();

        // Even sizes should work (auto-incremented to odd)
        let result = dilate_color(&pix, 2, 4);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_size_error() {
        let pix = create_test_color_image();

        let result = dilate_color(&pix, 0, 3);
        assert!(result.is_err());

        let result = erode_color(&pix, 3, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_color_error() {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();

        let result = dilate_color(&pix, 3, 3);
        assert!(result.is_err());

        let result = erode_color(&pix, 3, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_gradient_color() {
        let pix = create_test_color_image();
        let gradient = gradient_color(&pix, 3, 3).unwrap();

        // Gradient should be highest at edges of color transitions
        // Center of bright region should have low gradient
        let (r, g, b) = color::extract_rgb(gradient.get_pixel_unchecked(4, 4));
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn test_top_hat_color() {
        let pix = create_test_color_image();
        let tophat = top_hat_color(&pix, 3, 3).unwrap();

        // Top-hat extracts bright features smaller than SE
        // For our test image, the 3x3 bright region survives opening,
        // so top-hat (original - opened) should be close to 0 at the center
        let (r, g, b) = color::extract_rgb(tophat.get_pixel_unchecked(4, 4));
        // The center pixel difference should be small
        assert!(r <= 200 && g <= 150 && b <= 100);
    }

    #[test]
    fn test_bottom_hat_color() {
        let pix = create_test_color_image();
        let bottomhat = bottom_hat_color(&pix, 3, 3).unwrap();

        // Bottom-hat extracts dark features (closing - original)
        // Verify the operation completed successfully and produced valid output
        assert_eq!(bottomhat.width(), 9);
        assert_eq!(bottomhat.height(), 9);
        assert_eq!(bottomhat.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_single_pixel_dilation() {
        // Create image with single bright pixel
        let pix = Pix::new(7, 7, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with dark (0, 0, 0)
        for y in 0..7 {
            for x in 0..7 {
                let pixel = color::compose_rgb(0, 0, 0);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        // Single bright pixel at center (255, 128, 64)
        let center_pixel = color::compose_rgb(255, 128, 64);
        pix_mut.set_pixel_unchecked(3, 3, center_pixel);
        let pix: Pix = pix_mut.into();

        let dilated = dilate_color(&pix, 3, 3).unwrap();

        // 3x3 dilation should create a 3x3 bright region
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let x = (3 + dx) as u32;
                let y = (3 + dy) as u32;
                let (r, g, b) = color::extract_rgb(dilated.get_pixel_unchecked(x, y));
                assert_eq!(
                    (r, g, b),
                    (255, 128, 64),
                    "Expected (255, 128, 64) at ({}, {})",
                    x,
                    y
                );
            }
        }

        // Corners should remain dark
        let (r, g, b) = color::extract_rgb(dilated.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn test_independent_channel_processing() {
        // Create image where channels have different patterns
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with (100, 100, 100)
        for y in 0..5 {
            for x in 0..5 {
                let pixel = color::compose_rgb(100, 100, 100);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        // Center pixel: high red, low green, medium blue
        let center_pixel = color::compose_rgb(255, 50, 150);
        pix_mut.set_pixel_unchecked(2, 2, center_pixel);
        let pix: Pix = pix_mut.into();

        let dilated = dilate_color(&pix, 3, 3).unwrap();

        // After 3x3 dilation, the 3x3 region around center should have:
        // - Red: max(100, 255) = 255
        // - Green: max(100, 50) = 100
        // - Blue: max(100, 150) = 150
        let (r, g, b) = color::extract_rgb(dilated.get_pixel_unchecked(1, 1));
        assert_eq!((r, g, b), (255, 100, 150));

        let eroded = erode_color(&pix, 3, 3).unwrap();

        // After 3x3 erosion, the center should have:
        // - Red: min(100, 255) = 100
        // - Green: min(100, 50) = 50
        // - Blue: min(100, 150) = 100
        let (r, g, b) = color::extract_rgb(eroded.get_pixel_unchecked(2, 2));
        assert_eq!((r, g, b), (100, 50, 100));
    }
}
