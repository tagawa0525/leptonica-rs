//! Edge detection and enhancement operations

use crate::{FilterError, FilterResult, Kernel, blockconv_gray, convolve_gray};
use leptonica_core::{Pix, PixelDepth, pix::RgbComponent};

/// Edge detection orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeOrientation {
    /// Detect horizontal edges
    Horizontal,
    /// Detect vertical edges
    Vertical,
    /// Detect all edges
    All,
}

/// Apply Sobel edge detection
///
/// # Arguments
/// * `pix` - Input 8-bit grayscale image
/// * `orientation` - Which edges to detect
pub fn sobel_edge(pix: &Pix, orientation: EdgeOrientation) -> FilterResult<Pix> {
    check_grayscale(pix)?;

    match orientation {
        EdgeOrientation::Horizontal => {
            let kernel = Kernel::sobel_horizontal();
            convolve_and_abs(pix, &kernel)
        }
        EdgeOrientation::Vertical => {
            let kernel = Kernel::sobel_vertical();
            convolve_and_abs(pix, &kernel)
        }
        EdgeOrientation::All => {
            let h_kernel = Kernel::sobel_horizontal();
            let v_kernel = Kernel::sobel_vertical();
            sobel_combined(pix, &h_kernel, &v_kernel)
        }
    }
}

/// Apply Laplacian edge detection
pub fn laplacian_edge(pix: &Pix) -> FilterResult<Pix> {
    check_grayscale(pix)?;
    let kernel = Kernel::laplacian();
    convolve_and_abs(pix, &kernel)
}

/// Convolve and take absolute value (for edge detection)
fn convolve_and_abs(pix: &Pix, kernel: &Kernel) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let kw = kernel.width();
    let kh = kernel.height();
    let kcx = kernel.center_x() as i32;
    let kcy = kernel.center_y() as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;

            for ky in 0..kh {
                for kx in 0..kw {
                    let sx = x as i32 + (kx as i32 - kcx);
                    let sy = y as i32 + (ky as i32 - kcy);

                    let sx = sx.clamp(0, w as i32 - 1) as u32;
                    let sy = sy.clamp(0, h as i32 - 1) as u32;

                    let pixel = pix.get_pixel_unchecked(sx, sy) as f32;
                    let k = kernel.get(kx, ky).unwrap_or(0.0);
                    sum += pixel * k;
                }
            }

            let result = sum.abs().clamp(0.0, 255.0) as u32;
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Combined Sobel (magnitude of both directions)
fn sobel_combined(pix: &Pix, h_kernel: &Kernel, v_kernel: &Kernel) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let kw = h_kernel.width();
    let kh = h_kernel.height();
    let kcx = h_kernel.center_x() as i32;
    let kcy = h_kernel.center_y() as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let mut sum_h = 0.0f32;
            let mut sum_v = 0.0f32;

            for ky in 0..kh {
                for kx in 0..kw {
                    let sx = x as i32 + (kx as i32 - kcx);
                    let sy = y as i32 + (ky as i32 - kcy);

                    let sx = sx.clamp(0, w as i32 - 1) as u32;
                    let sy = sy.clamp(0, h as i32 - 1) as u32;

                    let pixel = pix.get_pixel_unchecked(sx, sy) as f32;
                    sum_h += pixel * h_kernel.get(kx, ky).unwrap_or(0.0);
                    sum_v += pixel * v_kernel.get(kx, ky).unwrap_or(0.0);
                }
            }

            // Magnitude using fast L1 (Manhattan) norm |h| + |v|.
            // For a more standard Sobel magnitude, use Euclidean sqrt(h*h + v*v)
            // instead, at the cost of an extra sqrt per pixel.
            let magnitude = sum_h.abs() + sum_v.abs();
            let result = magnitude.clamp(0.0, 255.0) as u32;
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Apply sharpening filter
pub fn sharpen(pix: &Pix) -> FilterResult<Pix> {
    check_grayscale(pix)?;
    let kernel = Kernel::sharpen();
    convolve_gray(pix, &kernel)
}

/// Apply unsharp masking
///
/// # Arguments
/// * `pix` - Input image
/// * `radius` - Blur radius
/// * `amount` - Sharpening strength (0.0-1.0 typical, can be higher)
pub fn unsharp_mask(pix: &Pix, radius: u32, amount: f32) -> FilterResult<Pix> {
    check_grayscale(pix)?;

    let w = pix.width();
    let h = pix.height();

    // 1. Create blurred version
    let size = 2 * radius + 1;
    let blur_kernel = Kernel::gaussian(size, radius as f32)?;
    let blurred = convolve_gray(pix, &blur_kernel)?;

    // 2. Compute: result = original + amount * (original - blurred)
    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let orig = pix.get_pixel_unchecked(x, y) as f32;
            let blur = blurred.get_pixel_unchecked(x, y) as f32;

            let diff = orig - blur;
            let result = orig + amount * diff;
            let result = result.round().clamp(0.0, 255.0) as u32;

            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Fast unsharp masking using block convolution
///
/// Uses block convolution for faster blurring instead of Gaussian convolution.
/// Only supports halfwidth=1 (3x3) or halfwidth=2 (5x5) for speed.
///
/// # Arguments
/// * `pix` - Input image (8 bpp grayscale for grayfast, 32 bpp for color)
/// * `halfwidth` - Kernel half-width (1 or 2; kernel size = 2*halfwidth+1)
/// * `amount` - Sharpening strength (typically 0.2-0.7)
pub fn unsharp_masking_fast(pix: &Pix, halfwidth: u32, amount: f32) -> FilterResult<Pix> {
    let depth = pix.depth();

    // Check supported depths
    if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: depth.bits(),
        });
    }

    // If 8bpp, dispatch to gray_fast
    if depth == PixelDepth::Bit8 {
        return unsharp_masking_gray_fast(pix, halfwidth, amount);
    }

    // 32bpp: extract R, G, B channels, apply gray_fast to each, recombine
    let pix_r = pix.get_rgb_component(RgbComponent::Red)?;
    let pix_g = pix.get_rgb_component(RgbComponent::Green)?;
    let pix_b = pix.get_rgb_component(RgbComponent::Blue)?;

    let pix_rs = unsharp_masking_gray_fast(&pix_r, halfwidth, amount)?;
    let pix_gs = unsharp_masking_gray_fast(&pix_g, halfwidth, amount)?;
    let pix_bs = unsharp_masking_gray_fast(&pix_b, halfwidth, amount)?;

    let mut result = Pix::create_rgb_image(&pix_rs, &pix_gs, &pix_bs)?;

    // If the original had alpha channel (spp=4), copy it
    if pix.spp() == 4 {
        let pix_a = pix.get_rgb_component(RgbComponent::Alpha)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_rgb_component(&pix_a, RgbComponent::Alpha)?;
        result = result_mut.into();
    }

    Ok(result)
}

/// Fast unsharp masking for grayscale images
///
/// Uses block convolution for faster blurring.
pub fn unsharp_masking_gray_fast(pix: &Pix, halfwidth: u32, amount: f32) -> FilterResult<Pix> {
    // Validate input
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }

    // If amount <= 0.0, return a clone (no sharpening)
    if amount <= 0.0 {
        return Ok(pix.clone());
    }

    // Validate halfwidth
    if halfwidth != 1 && halfwidth != 2 {
        return Err(FilterError::InvalidParameters(
            "halfwidth must be 1 or 2".to_string(),
        ));
    }

    let w = pix.width();
    let h = pix.height();

    // Use block convolution for fast blurring
    // blockconv_gray(pix, None, wc, hc) where kernel size = 2*halfwidth+1
    let blurred = blockconv_gray(pix, None, halfwidth, halfwidth)?;

    // Compute: result = original + amount * (original - blurred)
    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let orig = pix.get_pixel_unchecked(x, y) as f32;
            let blur = blurred.get_pixel_unchecked(x, y) as f32;

            let diff = orig - blur;
            let result = orig + amount * diff;
            let result = result.round().clamp(0.0, 255.0) as u32;

            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Apply emboss effect
///
/// Output values are centered around 128 (flat regions produce ~128,
/// edges produce values above or below 128 depending on gradient direction).
/// This differs from edge detection functions which output absolute magnitudes.
pub fn emboss(pix: &Pix) -> FilterResult<Pix> {
    check_grayscale(pix)?;

    let kernel = Kernel::emboss();
    let w = pix.width();
    let h = pix.height();
    let kw = kernel.width();
    let kh = kernel.height();
    let kcx = kernel.center_x() as i32;
    let kcy = kernel.center_y() as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;

            for ky in 0..kh {
                for kx in 0..kw {
                    let sx = x as i32 + (kx as i32 - kcx);
                    let sy = y as i32 + (ky as i32 - kcy);

                    let sx = sx.clamp(0, w as i32 - 1) as u32;
                    let sy = sy.clamp(0, h as i32 - 1) as u32;

                    let pixel = pix.get_pixel_unchecked(sx, sy) as f32;
                    let k = kernel.get(kx, ky).unwrap_or(0.0);
                    sum += pixel * k;
                }
            }

            // Add 128 to center the emboss effect
            let result = (sum + 128.0).round().clamp(0.0, 255.0) as u32;
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

fn check_grayscale(pix: &Pix) -> FilterResult<()> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> Pix {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a pattern with edges
        for y in 0..10 {
            for x in 0..10 {
                let val = if x < 5 { 50 } else { 200 };
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_sobel_vertical() {
        let pix = create_test_image();
        let edges = sobel_edge(&pix, EdgeOrientation::Vertical).unwrap();

        // Vertical edge at x=4-5 should be detected
        let edge_val = edges.get_pixel_unchecked(4, 5);
        let non_edge_val = edges.get_pixel_unchecked(1, 5);

        assert!(edge_val > non_edge_val);
    }

    #[test]
    fn test_sobel_all() {
        let pix = create_test_image();
        let edges = sobel_edge(&pix, EdgeOrientation::All).unwrap();

        assert_eq!(edges.width(), pix.width());
        assert_eq!(edges.height(), pix.height());
    }

    #[test]
    fn test_laplacian() {
        let pix = create_test_image();
        let edges = laplacian_edge(&pix).unwrap();

        assert_eq!(edges.width(), pix.width());
    }

    #[test]
    fn test_sharpen() {
        let pix = create_test_image();
        let sharpened = sharpen(&pix).unwrap();

        assert_eq!(sharpened.width(), pix.width());
    }

    #[test]
    fn test_unsharp_mask() {
        let pix = create_test_image();
        let sharpened = unsharp_mask(&pix, 1, 0.5).unwrap();

        assert_eq!(sharpened.width(), pix.width());
    }

    #[test]
    fn test_emboss() {
        let pix = create_test_image();
        let embossed = emboss(&pix).unwrap();

        assert_eq!(embossed.width(), pix.width());
    }

    #[test]
    fn test_unsharp_masking_gray_fast_basic() {
        let pix = create_test_image();

        // Test with halfwidth=1, amount=0.5
        let result = unsharp_masking_gray_fast(&pix, 1, 0.5).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_unsharp_masking_gray_fast_halfwidth2() {
        let pix = create_test_image();

        // Test with halfwidth=2, amount=0.7
        let result = unsharp_masking_gray_fast(&pix, 2, 0.7).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_unsharp_masking_gray_fast_no_op() {
        let pix = create_test_image();

        // Test with amount=0.0 (should return clone)
        let result = unsharp_masking_gray_fast(&pix, 1, 0.0).unwrap();

        // Should be a clone of original
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_unsharp_masking_gray_fast_rejects_non_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();

        let result = unsharp_masking_gray_fast(&pix, 1, 0.5);

        assert!(result.is_err());
    }

    #[test]
    fn test_unsharp_masking_fast_8bpp() {
        let pix = create_test_image();

        // Test with 8bpp image
        let result = unsharp_masking_fast(&pix, 1, 0.5).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_unsharp_masking_fast_32bpp_rgb() {
        // Create a 32bpp RGB test image
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                // Create RGB pattern
                let r = if x < 5 { 50 } else { 200 };
                let g = if y < 5 { 60 } else { 180 };
                let b = 128;
                let pixel = (r << 24) | (g << 16) | (b << 8) | 0xFF;
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }
        let pix = pix_mut.into();

        // Test with 32bpp image
        let result = unsharp_masking_fast(&pix, 1, 0.5).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }
}
