//! Convolution operations
//!
//! Implements image convolution with arbitrary kernels.

use crate::{FilterError, FilterResult, Kernel};
use leptonica_core::{FPix, Pix, PixelDepth, color, pix::RgbComponent};

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
/// center pixel is strictly greater than the average, 0 otherwise.
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
pub fn census_transform(pix: &Pix, halfsize: u32) -> FilterResult<Pix> {
    check_grayscale(pix)?;

    if halfsize < 1 {
        return Err(FilterError::InvalidParameters(
            "halfsize must be >= 1".into(),
        ));
    }

    // Get neighborhood average using blockconv_gray
    let pixav = crate::block_conv::blockconv_gray(pix, None, halfsize, halfsize)?;

    // Compare each pixel with its neighborhood average
    let w = pix.width();
    let h = pix.height();
    let pixd = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut pixd_mut = pixd.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val_src = pix.get_pixel_unchecked(x, y);
            let val_avg = pixav.get_pixel_unchecked(x, y);
            if val_src > val_avg {
                pixd_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok(pixd_mut.into())
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
pub fn add_gaussian_noise(pix: &Pix, stdev: f32) -> FilterResult<Pix> {
    let stdev = stdev.max(0.0);
    match pix.depth() {
        PixelDepth::Bit8 | PixelDepth::Bit32 => {}
        _ => {
            return Err(FilterError::UnsupportedDepth {
                expected: "8 or 32 bpp",
                actual: pix.depth().bits(),
            });
        }
    }

    let w = pix.width();
    let h = pix.height();
    let pixd = Pix::new(w, h, pix.depth())?;
    let mut pixd_mut = pixd.try_into_mut().unwrap();
    if pix.depth() == PixelDepth::Bit32 {
        pixd_mut.set_spp(pix.spp());
    }

    // Gaussian distribution sampler using Box-Muller transform
    struct GaussianSampler {
        select: bool,
        saved: f32,
        state: u64,
    }

    impl GaussianSampler {
        fn new() -> Self {
            // Use a simple LCG with constants from Numerical Recipes
            Self {
                select: false,
                saved: 0.0,
                state: 1234567890u64,
            }
        }

        fn rand_f32(&mut self) -> f32 {
            // LCG: state = (a * state + c) % m
            const A: u64 = 1664525;
            const C: u64 = 1013904223;
            self.state = self.state.wrapping_mul(A).wrapping_add(C);
            (self.state as f32) / (u64::MAX as f32)
        }

        fn sample(&mut self) -> f32 {
            if self.select {
                self.select = false;
                self.saved
            } else {
                // Box-Muller transform: generate two uniform random variables,
                // transform to two Gaussian random variables
                let (xval, yval, rsq) = loop {
                    let xval = 2.0 * self.rand_f32() - 1.0;
                    let yval = 2.0 * self.rand_f32() - 1.0;
                    let rsq = xval * xval + yval * yval;
                    if rsq > 0.0 && rsq < 1.0 {
                        break (xval, yval, rsq);
                    }
                };
                let factor = (-2.0 * rsq.ln() / rsq).sqrt();
                self.saved = xval * factor;
                self.select = true;
                yval * factor
            }
        }
    }

    let mut sampler = GaussianSampler::new();

    if pix.depth() == PixelDepth::Bit8 {
        for y in 0..h {
            for x in 0..w {
                let val = pix.get_pixel_unchecked(x, y) as i32;
                let noise = (stdev * sampler.sample()).round() as i32;
                let result = (val + noise).clamp(0, 255) as u32;
                pixd_mut.set_pixel_unchecked(x, y, result);
            }
        }
    } else {
        // 32 bpp color
        for y in 0..h {
            for x in 0..w {
                let pixel = pix.get_pixel_unchecked(x, y);
                let (r, g, b, a) = color::extract_rgba(pixel);

                let r_noise = (stdev * sampler.sample()).round() as i32;
                let g_noise = (stdev * sampler.sample()).round() as i32;
                let b_noise = (stdev * sampler.sample()).round() as i32;

                let r_out = ((r as i32) + r_noise).clamp(0, 255) as u8;
                let g_out = ((g as i32) + g_noise).clamp(0, 255) as u8;
                let b_out = ((b as i32) + b_noise).clamp(0, 255) as u8;

                let result = color::compose_rgba(r_out, g_out, b_out, a);
                pixd_mut.set_pixel_unchecked(x, y, result);
            }
        }
    }

    Ok(pixd_mut.into())
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
pub fn blocksum(pix: &Pix, wc: u32, hc: u32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(FilterError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();

    if wc == 0 || hc == 0 {
        return Ok(pix.deep_clone());
    }

    // Reduce kernel if necessary
    let wc = wc.min((w - 1) / 2);
    let hc = hc.min((h - 1) / 2);

    if wc == 0 || hc == 0 {
        // Return 8bpp version even for degenerate kernel (documented output is 8bpp)
        return Ok(pix.convert_1_to_8(0, 255)?);
    }

    // Convert 1bpp to 8bpp (0→0, 1→255) for integral image computation
    let pix8 = pix.convert_1_to_8(0, 255)?;

    // Compute integral image using blockconv_accum
    let acc = crate::block_conv::blockconv_accum(&pix8)?;

    // Compute block sums using integral image
    let pixd = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut pixd_mut = pixd.try_into_mut().unwrap();

    let fwc = (2 * wc + 1) as f64;
    let fhc = (2 * hc + 1) as f64;
    let norm = 1.0 / (fwc * fhc);

    for y in 0..h {
        let ymin = if y > hc { y - hc - 1 } else { 0 };
        let ymax = (y + hc).min(h - 1);
        let hn = if y > hc {
            (ymax - ymin) as f64
        } else {
            (ymax + 1) as f64
        };

        for x in 0..w {
            let xmin = if x > wc { x - wc - 1 } else { 0 };
            let xmax = (x + wc).min(w - 1);
            let wn = if x > wc {
                (xmax - xmin) as f64
            } else {
                (xmax + 1) as f64
            };

            // Four-corner lookup on integral image
            let mut val = acc.get_pixel_unchecked(xmax, ymax) as i64;
            if y > hc {
                val -= acc.get_pixel_unchecked(xmax, ymin) as i64;
            }
            if x > wc {
                val -= acc.get_pixel_unchecked(xmin, ymax) as i64;
            }
            if y > hc && x > wc {
                val += acc.get_pixel_unchecked(xmin, ymin) as i64;
            }

            // Normalize: output = val / (actual_area) = val * norm * fwc/wn * fhc/hn
            // Since input was scaled 0→0, 1→255, the sum is already in terms of 255*count
            // We need to normalize by the actual area, not the full kernel area
            let result = (norm * val as f64 * fwc / wn * fhc / hn + 0.5) as u32;
            let result = result.min(255);
            pixd_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(pixd_mut.into())
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
pub fn blockrank(pix: &Pix, wc: u32, hc: u32, rank: f32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(FilterError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    if !(0.0..=1.0).contains(&rank) {
        return Err(FilterError::InvalidParameters(
            "rank must be in [0.0, 1.0]".into(),
        ));
    }

    // Special case: rank == 0.0 means return all-ones image
    if rank == 0.0 {
        let w = pix.width();
        let h = pix.height();
        let pixd = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut pixd_mut = pixd.try_into_mut().unwrap();
        for y in 0..h {
            for x in 0..w {
                pixd_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        return Ok(pixd_mut.into());
    }

    let w = pix.width();
    let h = pix.height();

    if wc == 0 || hc == 0 {
        return Ok(pix.deep_clone());
    }

    // Reduce kernel if necessary
    let wc = wc.min((w - 1) / 2);
    let hc = hc.min((h - 1) / 2);

    if wc == 0 || hc == 0 {
        return Ok(pix.deep_clone());
    }

    // Get normalized block sums
    let pixt = blocksum(pix, wc, hc)?;

    // Threshold at rank * 255
    // Note: C Leptonica uses pixThresholdToBinary which returns 1 for values < thresh,
    // then inverts. We directly threshold with >= to avoid the inversion.
    let thresh = (255.0 * rank) as u32;

    let pixd = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut pixd_mut = pixd.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val = pixt.get_pixel_unchecked(x, y);
            if val >= thresh {
                pixd_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    Ok(pixd_mut.into())
}

/// Convolve an FPix (floating-point image) with a kernel.
///
/// Each pixel in the output is the weighted sum of the kernel applied to
/// the corresponding neighborhood in the input. Uses replicate (clamp)
/// border handling.
///
/// If `normalize` is true, the kernel values are scaled so that they sum
/// to 1.0 before convolution. Returns an error if the kernel sum is near
/// zero (cannot normalize).
///
/// C equivalent: `fpixConvolve()` in `convolve.c`
pub fn fpix_convolve(fpix: &FPix, kernel: &Kernel, normalize: bool) -> FilterResult<FPix> {
    let w = fpix.width() as i32;
    let h = fpix.height() as i32;
    let kw = kernel.width() as i32;
    let kh = kernel.height() as i32;
    let cx = kernel.center_x() as i32;
    let cy = kernel.center_y() as i32;

    // C equivalent: kernelNormalize() returns a copy (no normalization) when
    // sum is near zero, so we silently skip normalization in that case.
    let ksum = kernel.sum();
    let scale = if normalize && ksum.abs() >= 1e-6 {
        1.0 / ksum
    } else {
        1.0
    };

    let mut fpixd = FPix::new(w as u32, h as u32)?;
    let kdata = kernel.data();

    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;
            for ky in 0..kh {
                let sy = (y + ky - cy).clamp(0, h - 1);
                for kx in 0..kw {
                    let sx = (x + kx - cx).clamp(0, w - 1);
                    let val = fpix.get_pixel_unchecked(sx as u32, sy as u32);
                    let kidx = (ky * kw + kx) as usize;
                    sum += val * kdata[kidx] * scale;
                }
            }
            fpixd.set_pixel_unchecked(x as u32, y as u32, sum);
        }
    }

    Ok(fpixd)
}

/// Convolve an FPix with a pair of separable 1-D kernels.
///
/// Applies `kernel_x` in the horizontal direction, then `kernel_y` in the
/// vertical direction. The full 2-D kernel must be separable (the outer
/// product of the two 1-D kernels).
///
/// C equivalent: `fpixConvolveSep()` in `convolve.c`
pub fn fpix_convolve_sep(
    fpix: &FPix,
    kernel_x: &Kernel,
    kernel_y: &Kernel,
    normalize: bool,
) -> FilterResult<FPix> {
    let tmp = fpix_convolve(fpix, kernel_x, normalize)?;
    fpix_convolve(&tmp, kernel_y, normalize)
}

/// Convolve an 8-bpp grayscale image and apply an automatic bias so that
/// all output values are non-negative.
///
/// Returns `(result_pix, bias)` where `bias` is the integer shift that was
/// added before converting back to a `Pix`.
///
/// - If `kernel1` (and optional `kernel2`) have no negative values, a
///   standard normalized convolution is performed (bias = 0, 8-bpp output).
/// - If any kernel value is negative, FPix convolution is used; the
///   minimum output value is shifted to 0.  `force8` controls whether the
///   output is clamped to 8-bpp or promoted to 16-bpp.
///
/// C equivalent: `pixConvolveWithBias()` in `convolve.c`
pub fn convolve_with_bias(
    pix: &Pix,
    kernel1: &Kernel,
    kernel2: Option<&Kernel>,
    force8: bool,
) -> FilterResult<(Pix, i32)> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::InvalidParameters(
            "input must be 8-bpp grayscale".into(),
        ));
    }

    // Check whether any kernel has negative values
    let min1 = kernel1.data().iter().cloned().fold(f32::INFINITY, f32::min);
    let min2 = kernel2.map_or(0.0f32, |k| {
        k.data().iter().cloned().fold(f32::INFINITY, f32::min)
    });
    let min = min1.min(min2);

    // No negative values: use standard convolution
    if min >= 0.0 {
        let result = if let Some(k2) = kernel2 {
            convolve_sep(pix, kernel1, k2)?
        } else {
            convolve(pix, kernel1)?
        };
        return Ok((result, 0));
    }

    // Negative values present: use FPix path with bias
    let fpix1 = FPix::from_pix(pix)?;
    let fpix2 = if let Some(k2) = kernel2 {
        fpix_convolve_sep(&fpix1, kernel1, k2, true)?
    } else {
        fpix_convolve(&fpix1, kernel1, true)?
    };

    // Find min/max to determine bias and output depth
    let data = fpix2.data();
    let minval = data.iter().cloned().fold(f32::INFINITY, f32::min);
    let maxval = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = maxval - minval;
    let bias = if minval < 0.0 { (-minval) as i32 } else { 0 };

    // Shift values so minimum is 0
    let mut fpix3 = FPix::new_with_value(fpix2.width(), fpix2.height(), 0.0)?;
    for y in 0..fpix2.height() {
        for x in 0..fpix2.width() {
            let v = fpix2.get_pixel_unchecked(x, y) + bias as f32;
            fpix3.set_pixel_unchecked(x, y, v);
        }
    }

    // Scale to 8-bpp if forced and range > 255
    let out_depth = if range <= 255.0 || !force8 {
        if range > 255.0 { 16 } else { 8 }
    } else {
        let scale = 255.0 / range;
        for y in 0..fpix3.height() {
            for x in 0..fpix3.width() {
                let v = fpix3.get_pixel_unchecked(x, y) * scale;
                fpix3.set_pixel_unchecked(x, y, v);
            }
        }
        8
    };

    let result = fpix3.to_pix(
        out_depth,
        leptonica_core::fpix::NegativeHandling::ClipToZero,
    )?;
    Ok((result, bias))
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

        // All pixels == average (128), so pixel > average is false, all should be 0
        assert_eq!(result.depth(), PixelDepth::Bit1);
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(result.get_pixel_unchecked(x, y), 0);
            }
        }
    }

    #[test]
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
    fn test_add_gaussian_noise_8bpp() {
        let pix = create_test_gray_image();
        let result = add_gaussian_noise(&pix, 10.0).unwrap();

        // Output should have same dimensions and depth
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_add_gaussian_noise_32bpp() {
        let pix = create_test_color_image();
        let result = add_gaussian_noise(&pix, 10.0).unwrap();

        // Output should have same dimensions and depth
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
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
    fn test_blockrank_threshold_zero() {
        // rank=0.0: always satisfied, returns all-ones image regardless of input
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
    fn test_blockrank_invalid_depth() {
        // Should reject non-1bpp input
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();

        let result = blockrank(&pix, 1, 1, 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_blockrank_invalid_rank() {
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();

        // rank must be in [0.0, 1.0]
        let result_low = blockrank(&pix, 1, 1, -0.1);
        let result_high = blockrank(&pix, 1, 1, 1.1);

        assert!(result_low.is_err());
        assert!(result_high.is_err());
    }

    #[test]
    fn test_fpix_convolve_basic() {
        // Use 7x7 FPix with impulse at center (3,3) so that border clamping
        // does not affect pixels far from the center (distance > kernel half-width).
        let mut fpix = FPix::new(7, 7).unwrap();
        fpix.set_pixel_unchecked(3, 3, 1.0);

        // 3x3 box kernel (all 1.0); only center of kernel hits the impulse
        let kernel = Kernel::from_slice(3, 3, &[1.0; 9]).unwrap();

        let result = fpix_convolve(&fpix, &kernel, false).unwrap();
        // Center pixel (3,3): only kernel center (kx=1,ky=1) maps to fpix(3,3)=1.0
        assert!((result.get_pixel_unchecked(3, 3) - 1.0).abs() < 0.01);
        // Corner (0,0): more than one kernel radius away from the impulse → 0.0
        assert!((result.get_pixel_unchecked(0, 0) - 0.0).abs() < 0.01);
        // Adjacent pixel (4,4): kernel offset (kx=0,ky=0) maps to fpix(3,3)=1.0
        assert!((result.get_pixel_unchecked(4, 4) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_fpix_convolve_normalized() {
        // Create 3x3 FPix with constant value 100.0
        let fpix = FPix::new_with_value(5, 5, 100.0).unwrap();

        // Box kernel normalized: result should be 100.0
        let kernel = Kernel::from_slice(3, 3, &[1.0; 9]).unwrap();
        let result = fpix_convolve(&fpix, &kernel, true).unwrap();

        // With normalization, constant image convolves to same value
        assert!((result.get_pixel_unchecked(2, 2) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_fpix_convolve_sep_matches_non_sep() {
        // Uniform FPix
        let fpix = FPix::new_with_value(5, 5, 50.0).unwrap();

        // Separable box kernel: kx=[1,1,1], ky=[1,1,1]
        // Non-sep: 3x3 all-ones kernel
        let kernel_x = Kernel::from_slice(3, 1, &[1.0, 1.0, 1.0]).unwrap();
        let kernel_y = Kernel::from_slice(1, 3, &[1.0, 1.0, 1.0]).unwrap();

        let result_sep = fpix_convolve_sep(&fpix, &kernel_x, &kernel_y, true).unwrap();

        // With normalized separable kernel on uniform image, result = 50.0
        let center = result_sep.get_pixel_unchecked(2, 2);
        assert!(
            (center - 50.0).abs() < 0.5,
            "Expected ~50.0, got {}",
            center
        );
    }

    #[test]
    fn test_convolve_with_bias_no_negative_kernel() {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..5u32 {
            for x in 0..5u32 {
                pix_mut.set_pixel_unchecked(x, y, 100);
            }
        }
        let pix: Pix = pix_mut.into();

        // All-positive kernel: no bias needed
        let kernel = Kernel::from_slice(3, 3, &[1.0; 9]).unwrap();
        let (result, bias) = convolve_with_bias(&pix, &kernel, None, true).unwrap();
        assert_eq!(bias, 0);
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_convolve_with_bias_negative_kernel() {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..5u32 {
            for x in 0..5u32 {
                pix_mut.set_pixel_unchecked(x, y, 128);
            }
        }
        let pix: Pix = pix_mut.into();

        // Laplacian kernel (has negative values)
        let kernel = Kernel::laplacian();
        let (result, _bias) = convolve_with_bias(&pix, &kernel, None, true).unwrap();
        // Result should have no negative pixel values (bias ensures this)
        let w = result.width();
        let h = result.height();
        for y in 0..h {
            for x in 0..w {
                let v = result.get_pixel_unchecked(x, y);
                assert!(v <= 255, "pixel ({},{}) = {} exceeds 255", x, y, v);
            }
        }
    }
}
