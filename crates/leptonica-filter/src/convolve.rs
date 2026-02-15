//! Convolution operations
//!
//! Implements image convolution with arbitrary kernels.

use crate::{FilterError, FilterResult, Kernel};
use leptonica_core::{Pix, PixelDepth, color};

/// Convolve an 8-bit grayscale image with a kernel
///
/// Uses replicate (clamp) border handling: pixels outside the image boundary
/// are treated as having the same value as the nearest edge pixel.
pub fn convolve_gray(pix: &Pix, kernel: &Kernel) -> FilterResult<Pix> {
    check_grayscale(pix)?;

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

                    // Clamp to image boundaries (replicate border)
                    let sx = sx.clamp(0, w as i32 - 1) as u32;
                    let sy = sy.clamp(0, h as i32 - 1) as u32;

                    let pixel = pix.get_pixel_unchecked(sx, sy) as f32;
                    let k = kernel.get(kx, ky).unwrap_or(0.0);
                    sum += pixel * k;
                }
            }

            let result = sum.round().clamp(0.0, 255.0) as u32;
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Convolve a 32-bit color image with a kernel
///
/// Uses replicate (clamp) border handling: pixels outside the image boundary
/// are treated as having the same value as the nearest edge pixel.
pub fn convolve_color(pix: &Pix, kernel: &Kernel) -> FilterResult<Pix> {
    check_color(pix)?;

    let w = pix.width();
    let h = pix.height();
    let kw = kernel.width();
    let kh = kernel.height();
    let kcx = kernel.center_x() as i32;
    let kcy = kernel.center_y() as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    for y in 0..h {
        for x in 0..w {
            let mut sum_r = 0.0f32;
            let mut sum_g = 0.0f32;
            let mut sum_b = 0.0f32;
            let mut sum_a = 0.0f32;

            for ky in 0..kh {
                for kx in 0..kw {
                    let sx = x as i32 + (kx as i32 - kcx);
                    let sy = y as i32 + (ky as i32 - kcy);

                    let sx = sx.clamp(0, w as i32 - 1) as u32;
                    let sy = sy.clamp(0, h as i32 - 1) as u32;

                    let pixel = pix.get_pixel_unchecked(sx, sy);
                    let (r, g, b, a) = color::extract_rgba(pixel);
                    let k = kernel.get(kx, ky).unwrap_or(0.0);

                    sum_r += r as f32 * k;
                    sum_g += g as f32 * k;
                    sum_b += b as f32 * k;
                    sum_a += a as f32 * k;
                }
            }

            let r = sum_r.round().clamp(0.0, 255.0) as u8;
            let g = sum_g.round().clamp(0.0, 255.0) as u8;
            let b = sum_b.round().clamp(0.0, 255.0) as u8;
            let a = sum_a.round().clamp(0.0, 255.0) as u8;

            let result = color::compose_rgba(r, g, b, a);
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Convolve an image (auto-dispatch based on depth)
pub fn convolve(pix: &Pix, kernel: &Kernel) -> FilterResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit8 => convolve_gray(pix, kernel),
        PixelDepth::Bit32 => convolve_color(pix, kernel),
        _ => Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

/// Apply box (average) blur
pub fn box_blur(pix: &Pix, radius: u32) -> FilterResult<Pix> {
    let size = 2 * radius + 1;
    let kernel = Kernel::box_kernel(size)?;
    convolve(pix, &kernel)
}

/// Apply Gaussian blur
pub fn gaussian_blur(pix: &Pix, radius: u32, sigma: f32) -> FilterResult<Pix> {
    let size = 2 * radius + 1;
    let kernel = Kernel::gaussian(size, sigma)?;
    convolve(pix, &kernel)
}

/// Apply Gaussian blur with automatic sigma calculation
pub fn gaussian_blur_auto(pix: &Pix, radius: u32) -> FilterResult<Pix> {
    // Use sigma = radius (minimum 0.5) for a reasonable default
    let sigma = (radius as f32).max(0.5);
    gaussian_blur(pix, radius, sigma)
}

/// Separable convolution (2D decomposed into 1D x and y)
///
/// Performs 2D convolution as a sequence of 1D convolutions in x and y directions.
/// This is faster than full 2D convolution when the kernel is separable.
pub fn convolve_sep(pix: &Pix, kernel_x: &Kernel, kernel_y: &Kernel) -> FilterResult<Pix> {
    todo!("convolve_sep not yet implemented")
}

/// Separable convolution for RGB images
///
/// Applies separable convolution to each color channel independently.
pub fn convolve_rgb_sep(pix: &Pix, kernel_x: &Kernel, kernel_y: &Kernel) -> FilterResult<Pix> {
    todo!("convolve_rgb_sep not yet implemented")
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

fn check_color(pix: &Pix) -> FilterResult<()> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32-bpp color",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_gray_image() -> Pix {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a pattern
        for y in 0..5 {
            for x in 0..5 {
                let val = x * 50 + y * 10;
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }

        pix_mut.into()
    }

    fn create_test_color_image() -> Pix {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..5 {
            for x in 0..5 {
                let r = (x * 50) as u8;
                let g = (y * 50) as u8;
                let b = 128;
                let pixel = color::compose_rgb(r, g, b);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_convolve_gray_identity() {
        let pix = create_test_gray_image();

        // Identity kernel
        let kernel = Kernel::from_slice(1, 1, &[1.0]).unwrap();
        let result = convolve_gray(&pix, &kernel).unwrap();

        // Should be identical
        for y in 0..5 {
            for x in 0..5 {
                let orig = pix.get_pixel_unchecked(x, y);
                let conv = result.get_pixel_unchecked(x, y);
                assert_eq!(orig, conv);
            }
        }
    }

    #[test]
    fn test_box_blur_gray() {
        let pix = create_test_gray_image();
        let blurred = box_blur(&pix, 1).unwrap();

        assert_eq!(blurred.width(), pix.width());
        assert_eq!(blurred.height(), pix.height());
    }

    #[test]
    fn test_gaussian_blur_gray() {
        let pix = create_test_gray_image();
        let blurred = gaussian_blur(&pix, 1, 1.0).unwrap();

        assert_eq!(blurred.width(), pix.width());
        assert_eq!(blurred.height(), pix.height());
    }

    #[test]
    fn test_convolve_color() {
        let pix = create_test_color_image();
        let kernel = Kernel::box_kernel(3).unwrap();
        let result = convolve_color(&pix, &kernel).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_convolve_auto_dispatch() {
        let gray = create_test_gray_image();
        let color = create_test_color_image();
        let kernel = Kernel::box_kernel(3).unwrap();

        let result_gray = convolve(&gray, &kernel).unwrap();
        let result_color = convolve(&color, &kernel).unwrap();

        assert_eq!(result_gray.depth(), PixelDepth::Bit8);
        assert_eq!(result_color.depth(), PixelDepth::Bit32);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convolve_sep_identity() {
        // Separable 1D identity kernels should produce same output as input
        let pix = create_test_gray_image();
        let kernel_1d = Kernel::from_slice(1, 1, &[1.0]).unwrap();

        let result = convolve_sep(&pix, &kernel_1d, &kernel_1d).unwrap();

        for y in 0..5 {
            for x in 0..5 {
                let orig = pix.get_pixel_unchecked(x, y);
                let conv = result.get_pixel_unchecked(x, y);
                assert_eq!(orig, conv);
            }
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convolve_sep_horizontal_vertical() {
        // Separable convolution should decompose correctly
        let pix = create_test_gray_image();

        // Horizontal 3x1 box kernel
        let kernel_h = Kernel::from_slice(3, 1, &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]).unwrap();
        // Vertical 1x3 box kernel
        let kernel_v = Kernel::from_slice(1, 3, &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]).unwrap();

        let result_sep = convolve_sep(&pix, &kernel_h, &kernel_v).unwrap();

        // Should be equivalent to full 3x3 box blur
        let kernel_full = Kernel::box_kernel(3).unwrap();
        let result_full = convolve(&pix, &kernel_full).unwrap();

        // Results should be very close (allowing for small rounding differences)
        for y in 0..pix.height() {
            for x in 0..pix.width() {
                let sep_val = result_sep.get_pixel_unchecked(x, y);
                let full_val = result_full.get_pixel_unchecked(x, y);
                let diff = (sep_val as i32 - full_val as i32).abs();
                assert!(
                    diff <= 1,
                    "Difference too large at ({}, {}): {} vs {}",
                    x,
                    y,
                    sep_val,
                    full_val
                );
            }
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convolve_sep_sobel_x() {
        // Sobel-X can be decomposed into separable kernels
        let pix = create_test_gray_image();

        // Sobel-X = [-1, 0, 1] (horizontal) * [1, 2, 1] (vertical)
        let kernel_h = Kernel::from_slice(3, 1, &[-1.0, 0.0, 1.0]).unwrap();
        let kernel_v = Kernel::from_slice(1, 3, &[1.0, 2.0, 1.0]).unwrap();

        let result = convolve_sep(&pix, &kernel_h, &kernel_v).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convolve_rgb_sep_identity() {
        let pix = create_test_color_image();
        let kernel_1d = Kernel::from_slice(1, 1, &[1.0]).unwrap();

        let result = convolve_rgb_sep(&pix, &kernel_1d, &kernel_1d).unwrap();

        for y in 0..5 {
            for x in 0..5 {
                let orig = pix.get_pixel_unchecked(x, y);
                let conv = result.get_pixel_unchecked(x, y);
                assert_eq!(orig, conv);
            }
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_convolve_rgb_sep_box_blur() {
        let pix = create_test_color_image();

        // 3x3 box blur as separable 1D kernels
        let kernel_1d = Kernel::from_slice(3, 1, &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]).unwrap();

        let result = convolve_rgb_sep(&pix, &kernel_1d, &kernel_1d).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }
}
