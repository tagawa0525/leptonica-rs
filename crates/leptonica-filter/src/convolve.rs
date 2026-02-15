//! Convolution operations
//!
//! Implements image convolution with arbitrary kernels.

use crate::{FilterError, FilterResult, Kernel};
use leptonica_core::{Pix, PixelDepth, color, pix::RgbComponent};

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

/// Separable convolution (sequential application of two kernels)
///
/// Applies two convolution passes sequentially: first with `kernel_x`, then
/// with `kernel_y` on the intermediate result. For true separable convolution,
/// `kernel_x` should be a horizontal 1D kernel (height=1) and `kernel_y` should
/// be a vertical 1D kernel (width=1). However, arbitrary 2D kernels are accepted
/// and will be applied sequentially (this matches C Leptonica behavior).
///
/// # Supported depths
///
/// - 8 bpp grayscale
/// - 32 bpp color
///
/// # See also
///
/// C Leptonica: `pixConvolveSep()` in `convolve.c`
pub fn convolve_sep(pix: &Pix, kernel_x: &Kernel, kernel_y: &Kernel) -> FilterResult<Pix> {
    // Validate input depth
    match pix.depth() {
        PixelDepth::Bit8 | PixelDepth::Bit32 => {}
        _ => {
            return Err(FilterError::UnsupportedDepth {
                expected: "8 or 32 bpp",
                actual: pix.depth().bits(),
            });
        }
    }

    // Apply horizontal convolution first
    let temp = convolve(pix, kernel_x)?;

    // Apply vertical convolution to the intermediate result
    let result = convolve(&temp, kernel_y)?;

    Ok(result)
}

/// Separable convolution for RGB images
///
/// Applies separable convolution to each color channel independently.
/// Only R, G, B channels are processed; alpha is not preserved.
/// The output always has `spp=3`. If you need alpha handling,
/// process channels individually with [`convolve_sep`].
///
/// # Algorithm
///
/// 1. Extract R, G, B channels into separate 8-bit images
/// 2. Apply separable convolution to each channel
/// 3. Recombine channels into 32-bit RGB image (spp=3)
///
/// # See also
///
/// C Leptonica: `pixConvolveRGBSep()` in `convolve.c`
pub fn convolve_rgb_sep(pix: &Pix, kernel_x: &Kernel, kernel_y: &Kernel) -> FilterResult<Pix> {
    check_color(pix)?;

    // Extract RGB channels using core API
    let pix_r = pix.get_rgb_component(RgbComponent::Red)?;
    let pix_g = pix.get_rgb_component(RgbComponent::Green)?;
    let pix_b = pix.get_rgb_component(RgbComponent::Blue)?;

    // Apply separable convolution to each channel
    let result_r = convolve_sep(&pix_r, kernel_x, kernel_y)?;
    let result_g = convolve_sep(&pix_g, kernel_x, kernel_y)?;
    let result_b = convolve_sep(&pix_b, kernel_x, kernel_y)?;

    // Recombine channels using core API
    let result = Pix::create_rgb_image(&result_r, &result_g, &result_b)?;

    Ok(result)
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

/// Census transform: compare each pixel against neighborhood average
///
/// For each pixel in an 8-bit grayscale image, computes the average of pixels
/// in a (2*halfsize+1) × (2*halfsize+1) neighborhood and outputs 1 if the
/// center pixel is >= average, 0 otherwise.
///
/// # Arguments
///
/// * `pix` - Input 8-bit grayscale image
/// * `halfsize` - Half-size of neighborhood (e.g., halfsize=1 → 3×3 window)
///
/// # Returns
///
/// 1-bit binary image where 1 = pixel >= neighborhood average
///
/// # See also
///
/// C Leptonica: `pixCensusTransform()` in `convolve.c`
pub fn census_transform(_pix: &Pix, _halfsize: u32) -> FilterResult<Pix> {
    todo!("census_transform not yet implemented")
}

/// Add Gaussian noise to an image
///
/// Adds random noise from a Gaussian distribution with mean=0 and the specified
/// standard deviation to each pixel. Works with 8-bit grayscale and 32-bit color.
///
/// # Arguments
///
/// * `pix` - Input 8-bit or 32-bit image
/// * `stdev` - Standard deviation of Gaussian noise
///
/// # Returns
///
/// Image with added noise (same depth as input)
///
/// # See also
///
/// C Leptonica: `pixAddGaussianNoise()` in `convolve.c`
pub fn add_gaussian_noise(_pix: &Pix, _stdev: f32) -> FilterResult<Pix> {
    todo!("add_gaussian_noise not yet implemented")
}

/// Block sum for binary images
///
/// Computes the sum of ON pixels in (2*wc+1) × (2*hc+1) blocks centered at
/// each pixel of a 1-bit binary image. The output is an 8-bit image where
/// each pixel value is normalized to 0-255 range based on block size.
///
/// # Arguments
///
/// * `pix` - Input 1-bit binary image
/// * `wc` - Half-width of block
/// * `hc` - Half-height of block
///
/// # Returns
///
/// 8-bit image with normalized block sums
///
/// # See also
///
/// C Leptonica: `pixBlocksum()` in `convolve.c`
pub fn blocksum(_pix: &Pix, _wc: u32, _hc: u32) -> FilterResult<Pix> {
    todo!("blocksum not yet implemented")
}

/// Block rank for binary images
///
/// Computes the rank filter for (2*wc+1) × (2*hc+1) blocks centered at each
/// pixel of a 1-bit binary image. For each block, if the fraction of ON pixels
/// >= rank threshold, output pixel is 1; otherwise 0.
///
/// # Arguments
///
/// * `pix` - Input 1-bit binary image
/// * `wc` - Half-width of block
/// * `hc` - Half-height of block
/// * `rank` - Threshold fraction in [0.0, 1.0] (e.g., 0.5 for median)
///
/// # Returns
///
/// 1-bit binary image with rank filter applied
///
/// # See also
///
/// C Leptonica: `pixBlockrank()` in `convolve.c`
pub fn blockrank(_pix: &Pix, _wc: u32, _hc: u32, _rank: f32) -> FilterResult<Pix> {
    todo!("blockrank not yet implemented")
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
    fn test_convolve_sep_color() {
        // Test convolve_sep directly on 32 bpp color image
        let pix = create_test_color_image();

        // 3x1 horizontal and 1x3 vertical box kernels
        let kernel_h = Kernel::from_slice(3, 1, &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]).unwrap();
        let kernel_v = Kernel::from_slice(1, 3, &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]).unwrap();

        let result = convolve_sep(&pix, &kernel_h, &kernel_v).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit32);

        // Verify it matches the full 2D color convolution
        let kernel_full = Kernel::box_kernel(3).unwrap();
        let result_full = convolve_color(&pix, &kernel_full).unwrap();

        for y in 0..pix.height() {
            for x in 0..pix.width() {
                let sep_px = result.get_pixel_unchecked(x, y);
                let full_px = result_full.get_pixel_unchecked(x, y);
                let (sr, sg, sb, sa) = color::extract_rgba(sep_px);
                let (fr, fg, fb, fa) = color::extract_rgba(full_px);
                assert!(
                    (sr as i32 - fr as i32).abs() <= 1
                        && (sg as i32 - fg as i32).abs() <= 1
                        && (sb as i32 - fb as i32).abs() <= 1
                        && (sa as i32 - fa as i32).abs() <= 1,
                    "Mismatch at ({}, {})",
                    x,
                    y
                );
            }
        }
    }

    #[test]
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
    fn test_convolve_rgb_sep_box_blur() {
        let pix = create_test_color_image();

        // Separable 3x3 box blur
        let kernel_h = Kernel::from_slice(3, 1, &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]).unwrap();
        let kernel_v = Kernel::from_slice(1, 3, &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]).unwrap();
        let result_sep = convolve_rgb_sep(&pix, &kernel_h, &kernel_v).unwrap();

        // Full 2D box blur for reference
        let kernel_full = Kernel::box_kernel(3).unwrap();
        let result_full = convolve_color(&pix, &kernel_full).unwrap();

        assert_eq!(result_sep.width(), pix.width());
        assert_eq!(result_sep.height(), pix.height());
        assert_eq!(result_sep.depth(), PixelDepth::Bit32);

        // Pixel-wise comparison between separable and full 2D convolution
        for y in 0..pix.height() {
            for x in 0..pix.width() {
                let sep_px = result_sep.get_pixel_unchecked(x, y);
                let full_px = result_full.get_pixel_unchecked(x, y);
                let (sr, sg, sb, _) = color::extract_rgba(sep_px);
                let (fr, fg, fb, _) = color::extract_rgba(full_px);
                // Allow ±1 rounding tolerance per channel
                assert!(
                    (sr as i32 - fr as i32).abs() <= 1
                        && (sg as i32 - fg as i32).abs() <= 1
                        && (sb as i32 - fb as i32).abs() <= 1,
                    "Mismatch at ({}, {}): sep=({},{},{}) vs full=({},{},{})",
                    x,
                    y,
                    sr,
                    sg,
                    sb,
                    fr,
                    fg,
                    fb
                );
            }
        }
    }

    // ========================================================================
    // Census transform tests
    // ========================================================================

    #[test]
    #[ignore = "not yet implemented"]
    fn test_census_transform_basic() {
        // Create a simple 8bpp test image
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a gradient: center pixel will be above/below average
        for y in 0..5 {
            for x in 0..5 {
                let val = x * 20 + y * 20;
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }
        let pix = pix_mut.into();

        let result = census_transform(&pix, 1).unwrap();

        // Output should be 1bpp
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_census_transform_uniform() {
        // Uniform image: all pixels equal
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel_unchecked(x, y, 128);
            }
        }
        let pix = pix_mut.into();

        let result = census_transform(&pix, 1).unwrap();

        // All pixels should be >= average (which is 128), so all should be 1
        assert_eq!(result.depth(), PixelDepth::Bit1);
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(result.get_pixel_unchecked(x, y), 1);
            }
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_census_transform_invalid_depth() {
        // Census transform requires 8bpp input
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();

        let result = census_transform(&pix, 1);
        assert!(result.is_err());
    }

    // ========================================================================
    // Gaussian noise tests
    // ========================================================================

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_gaussian_noise_8bpp() {
        let pix = create_test_gray_image();
        let result = add_gaussian_noise(&pix, 10.0).unwrap();

        // Output should have same dimensions and depth
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_gaussian_noise_32bpp() {
        let pix = create_test_color_image();
        let result = add_gaussian_noise(&pix, 10.0).unwrap();

        // Output should have same dimensions and depth
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_gaussian_noise_statistical_properties() {
        // Create a uniform 8bpp image
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let original_value = 128u32;
        for y in 0..100 {
            for x in 0..100 {
                pix_mut.set_pixel_unchecked(x, y, original_value);
            }
        }
        let pix = pix_mut.into();

        // Add noise with stdev=20
        let result = add_gaussian_noise(&pix, 20.0).unwrap();

        // Compute mean of noisy image
        let mut sum = 0u64;
        for y in 0..100 {
            for x in 0..100 {
                sum += result.get_pixel_unchecked(x, y) as u64;
            }
        }
        let mean = sum / (100 * 100);

        // Mean should be close to original value (within a few pixels)
        // Gaussian noise has mean=0, so E[original + noise] = original
        let diff = (mean as i64 - original_value as i64).abs();
        assert!(
            diff < 5,
            "Mean {} too far from original {}",
            mean,
            original_value
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_gaussian_noise_invalid_depth() {
        // Should reject 1bpp input
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();

        let result = add_gaussian_noise(&pix, 10.0);
        assert!(result.is_err());
    }

    // ========================================================================
    // Block sum tests
    // ========================================================================

    #[test]
    #[ignore = "not yet implemented"]
    fn test_blocksum_basic() {
        // Create a simple 1bpp image
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set some pixels to 1
        for y in 3..7 {
            for x in 3..7 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix = pix_mut.into();

        let result = blocksum(&pix, 1, 1).unwrap();

        // Output should be 8bpp
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());

        // Center of filled region should have high value
        let center_val = result.get_pixel_unchecked(5, 5);
        assert!(
            center_val > 200,
            "Center value {} should be near 255",
            center_val
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_blocksum_all_zero() {
        // All-zero image should produce all-zero output
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        let result = blocksum(&pix, 1, 1).unwrap();

        assert_eq!(result.depth(), PixelDepth::Bit8);
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(result.get_pixel_unchecked(x, y), 0);
            }
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_blocksum_all_one() {
        // All-one image should produce all-255 output
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix = pix_mut.into();

        let result = blocksum(&pix, 1, 1).unwrap();

        assert_eq!(result.depth(), PixelDepth::Bit8);
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(result.get_pixel_unchecked(x, y), 255);
            }
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_blocksum_invalid_depth() {
        // Should reject non-1bpp input
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();

        let result = blocksum(&pix, 1, 1);
        assert!(result.is_err());
    }

    // ========================================================================
    // Block rank tests
    // ========================================================================

    #[test]
    #[ignore = "not yet implemented"]
    fn test_blockrank_basic() {
        // Create a simple 1bpp image
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set half the pixels to 1 in a checkerboard pattern
        for y in 0..10 {
            for x in 0..10 {
                if (x + y) % 2 == 0 {
                    pix_mut.set_pixel_unchecked(x, y, 1);
                }
            }
        }
        let pix = pix_mut.into();

        // Median filter (rank=0.5)
        let result = blockrank(&pix, 1, 1, 0.5).unwrap();

        // Output should be 1bpp
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_blockrank_threshold_zero() {
        // rank=0.0 means any ON pixel in block sets output to 1 (dilation-like)
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Single pixel ON at center
        pix_mut.set_pixel_unchecked(5, 5, 1);
        let pix = pix_mut.into();

        let result = blockrank(&pix, 1, 1, 0.0).unwrap();

        // Neighbors of center should also be 1 (dilation effect)
        assert_eq!(result.get_pixel_unchecked(5, 5), 1);
        assert_eq!(result.get_pixel_unchecked(4, 5), 1);
        assert_eq!(result.get_pixel_unchecked(6, 5), 1);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_blockrank_threshold_one() {
        // rank=1.0 means all pixels in block must be ON (erosion-like)
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill entire image
        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix = pix_mut.into();

        let result = blockrank(&pix, 1, 1, 1.0).unwrap();

        // Interior should remain 1, edges might be 0 due to border handling
        assert_eq!(result.get_pixel_unchecked(5, 5), 1);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_blockrank_invalid_depth() {
        // Should reject non-1bpp input
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();

        let result = blockrank(&pix, 1, 1, 0.5);
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_blockrank_invalid_rank() {
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();

        // rank must be in [0.0, 1.0]
        let result_low = blockrank(&pix, 1, 1, -0.1);
        let result_high = blockrank(&pix, 1, 1, 1.1);

        assert!(result_low.is_err());
        assert!(result_high.is_err());
    }
}
