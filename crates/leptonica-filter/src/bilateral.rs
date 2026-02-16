//! Bilateral filtering (edge-preserving smoothing)
//!
//! Bilateral filtering is a non-linear, edge-preserving smoothing filter.
//! It combines a spatial Gaussian filter with a range (intensity) Gaussian filter.
//!
//! The bilateral filter has the property of smoothing uniform regions while
//! preserving edges.
//!
//! # Algorithm
//!
//! For each pixel, the output is a weighted average of neighboring pixels where:
//! - Spatial weight: Gaussian based on distance from center pixel
//! - Range weight: Gaussian based on intensity difference from center pixel
//!
//! # Example
//!
//! ```ignore
//! use leptonica_filter::bilateral_exact;
//!
//! let smoothed = bilateral_exact(&pix, 2.0, 30.0)?;
//! ```

use crate::{FilterError, FilterResult, Kernel};
use leptonica_core::pix::RgbComponent;
use leptonica_core::{Pix, PixelDepth, color};

/// Create a range kernel for bilateral filtering
///
/// Creates a 256-element array where each element represents the weight
/// for a given intensity difference (0-255).
///
/// # Arguments
/// * `range_stdev` - Standard deviation for the range Gaussian (must be > 0.0)
///
/// # Returns
/// A 256-element array of weights, where index i corresponds to intensity difference i
pub fn make_range_kernel(range_stdev: f32) -> FilterResult<[f32; 256]> {
    if range_stdev <= 0.0 {
        return Err(FilterError::InvalidParameters(
            "range_stdev must be positive".to_string(),
        ));
    }

    let mut kernel = [0.0f32; 256];
    let denom = 2.0 * range_stdev * range_stdev;

    for (i, val) in kernel.iter_mut().enumerate() {
        *val = (-(i as f32 * i as f32) / denom).exp();
    }

    Ok(kernel)
}

/// Apply exact bilateral filter to an 8bpp grayscale image
///
/// This is the slow but exact implementation of bilateral filtering.
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale image
/// * `spatial_kernel` - 2D spatial Gaussian kernel
/// * `range_kernel` - Optional 256-element range kernel. If None, degenerates to regular Gaussian convolution.
///
/// # Returns
/// Filtered 8bpp grayscale image
pub fn bilateral_gray_exact(
    pix: &Pix,
    spatial_kernel: &Kernel,
    range_kernel: Option<&[f32; 256]>,
) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let kw = spatial_kernel.width();
    let kh = spatial_kernel.height();
    let kcx = spatial_kernel.center_x() as i32;
    let kcy = spatial_kernel.center_y() as i32;

    // Check if image is large enough
    if w < kw || h < kh {
        // Return a copy for images too small
        return Ok(pix.deep_clone());
    }

    // If no range kernel, this degenerates to standard convolution
    // For simplicity, we still use bilateral logic with unit range weights
    let unit_range = [1.0f32; 256];
    let range = range_kernel.unwrap_or(&unit_range);

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let center_val = pix.get_pixel_unchecked(x, y) as i32;

            let mut sum = 0.0f32;
            let mut weight_sum = 0.0f32;

            for ky in 0..kh {
                for kx in 0..kw {
                    let sx = x as i32 + (kx as i32 - kcx);
                    let sy = y as i32 + (ky as i32 - kcy);

                    // Clamp to image boundaries (replicate border)
                    let sx = sx.clamp(0, w as i32 - 1) as u32;
                    let sy = sy.clamp(0, h as i32 - 1) as u32;

                    let neighbor_val = pix.get_pixel_unchecked(sx, sy) as i32;
                    let spatial_weight = spatial_kernel.get(kx, ky).unwrap_or(0.0);
                    let intensity_diff = (center_val - neighbor_val).unsigned_abs() as usize;
                    let range_weight = range[intensity_diff.min(255)];

                    let weight = spatial_weight * range_weight;
                    sum += neighbor_val as f32 * weight;
                    weight_sum += weight;
                }
            }

            let result = if weight_sum > 0.0 {
                (sum / weight_sum + 0.5) as u32
            } else {
                center_val as u32
            };

            out_mut.set_pixel_unchecked(x, y, result.min(255));
        }
    }

    Ok(out_mut.into())
}

/// Apply exact bilateral filter to an image
///
/// This is the slow but exact implementation of bilateral filtering.
/// Automatically handles both 8bpp grayscale and 32bpp color images.
///
/// For color images, each channel (R, G, B) is processed independently.
///
/// # Arguments
/// * `pix` - Input 8bpp grayscale or 32bpp color image
/// * `spatial_stdev` - Standard deviation for spatial Gaussian (must be > 0.0)
/// * `range_stdev` - Standard deviation for range Gaussian (must be > 0.0)
///
/// # Returns
/// Filtered image with same depth as input
///
/// # Example
/// ```ignore
/// let smoothed = bilateral_exact(&pix, 2.0, 30.0)?;
/// ```
pub fn bilateral_exact(pix: &Pix, spatial_stdev: f32, range_stdev: f32) -> FilterResult<Pix> {
    // Validate parameters
    if spatial_stdev <= 0.0 {
        return Err(FilterError::InvalidParameters(
            "spatial_stdev must be positive".to_string(),
        ));
    }
    if range_stdev <= 0.0 {
        return Err(FilterError::InvalidParameters(
            "range_stdev must be positive".to_string(),
        ));
    }

    // Create kernels
    let halfwidth = (2.0 * spatial_stdev) as u32;
    let size = 2 * halfwidth + 1;
    let spatial_kernel = Kernel::gaussian(size, spatial_stdev)?;
    let range_kernel = make_range_kernel(range_stdev)?;

    match pix.depth() {
        PixelDepth::Bit8 => bilateral_gray_exact(pix, &spatial_kernel, Some(&range_kernel)),
        PixelDepth::Bit32 => bilateral_color_exact(pix, &spatial_kernel, &range_kernel),
        _ => Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

/// Apply exact bilateral filter to a 32bpp color image
///
/// Each color channel is processed independently.
fn bilateral_color_exact(
    pix: &Pix,
    spatial_kernel: &Kernel,
    range_kernel: &[f32; 256],
) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let kw = spatial_kernel.width();
    let kh = spatial_kernel.height();
    let kcx = spatial_kernel.center_x() as i32;
    let kcy = spatial_kernel.center_y() as i32;

    // Check if image is large enough
    if w < kw || h < kh {
        return Ok(pix.deep_clone());
    }

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    for y in 0..h {
        for x in 0..w {
            let center_pixel = pix.get_pixel_unchecked(x, y);
            let (center_r, center_g, center_b, center_a) = color::extract_rgba(center_pixel);

            let mut sum_r = 0.0f32;
            let mut sum_g = 0.0f32;
            let mut sum_b = 0.0f32;
            let mut sum_a = 0.0f32;
            let mut weight_sum_r = 0.0f32;
            let mut weight_sum_g = 0.0f32;
            let mut weight_sum_b = 0.0f32;
            let mut weight_sum_a = 0.0f32;

            for ky in 0..kh {
                for kx in 0..kw {
                    let sx = x as i32 + (kx as i32 - kcx);
                    let sy = y as i32 + (ky as i32 - kcy);

                    let sx = sx.clamp(0, w as i32 - 1) as u32;
                    let sy = sy.clamp(0, h as i32 - 1) as u32;

                    let neighbor_pixel = pix.get_pixel_unchecked(sx, sy);
                    let (nr, ng, nb, na) = color::extract_rgba(neighbor_pixel);

                    let spatial_weight = spatial_kernel.get(kx, ky).unwrap_or(0.0);

                    // Process each channel independently
                    let diff_r = (center_r as i32 - nr as i32).unsigned_abs() as usize;
                    let diff_g = (center_g as i32 - ng as i32).unsigned_abs() as usize;
                    let diff_b = (center_b as i32 - nb as i32).unsigned_abs() as usize;
                    let diff_a = (center_a as i32 - na as i32).unsigned_abs() as usize;

                    let weight_r = spatial_weight * range_kernel[diff_r.min(255)];
                    let weight_g = spatial_weight * range_kernel[diff_g.min(255)];
                    let weight_b = spatial_weight * range_kernel[diff_b.min(255)];
                    let weight_a = spatial_weight * range_kernel[diff_a.min(255)];

                    sum_r += nr as f32 * weight_r;
                    sum_g += ng as f32 * weight_g;
                    sum_b += nb as f32 * weight_b;
                    sum_a += na as f32 * weight_a;

                    weight_sum_r += weight_r;
                    weight_sum_g += weight_g;
                    weight_sum_b += weight_b;
                    weight_sum_a += weight_a;
                }
            }

            let result_r = if weight_sum_r > 0.0 {
                (sum_r / weight_sum_r + 0.5) as u8
            } else {
                center_r
            };
            let result_g = if weight_sum_g > 0.0 {
                (sum_g / weight_sum_g + 0.5) as u8
            } else {
                center_g
            };
            let result_b = if weight_sum_b > 0.0 {
                (sum_b / weight_sum_b + 0.5) as u8
            } else {
                center_b
            };
            let result_a = if weight_sum_a > 0.0 {
                (sum_a / weight_sum_a + 0.5) as u8
            } else {
                center_a
            };

            let result = color::compose_rgba(result_r, result_g, result_b, result_a);
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Fast approximate separable bilateral filter
///
/// Uses the Principal Bilateral Components (PBC) algorithm from Yang, Tan
/// and Ahuja for fast, separable bilateral filtering. Handles both 8bpp
/// grayscale and 32bpp RGB images. For 32bpp, each channel is processed
/// independently.
///
/// C版: `pixBilateral()` in `bilateral.c`
///
/// # Arguments
/// * `pix` - 8bpp grayscale or 32bpp RGB image (no colormap)
/// * `spatial_stdev` - Spatial Gaussian std dev in pixels (> 0.5)
/// * `range_stdev` - Range Gaussian std dev (> 5.0; typical 50.0)
/// * `ncomps` - Number of PBC images [4..30]
/// * `reduction` - Downscaling factor: 1, 2, or 4
pub fn bilateral(
    pix: &Pix,
    spatial_stdev: f32,
    range_stdev: f32,
    ncomps: u32,
    reduction: u32,
) -> FilterResult<Pix> {
    let d = pix.depth();
    if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: d.bits(),
        });
    }
    // Reject colormapped 8bpp images: pixel values are palette indices, not
    // intensities, so the bilateral filter would produce incorrect results.
    if d == PixelDepth::Bit8 && pix.colormap().is_some() {
        return Err(FilterError::InvalidParameters(
            "8bpp images with a colormap are not supported".into(),
        ));
    }
    validate_bilateral_params(pix, spatial_stdev, range_stdev, ncomps, reduction)?;

    // Check image size
    let filtersize = (2.0 * spatial_stdev + 1.5) as u32;
    let w = pix.width();
    let h = pix.height();
    if w < 2 * filtersize || h < 2 * filtersize {
        return Ok(pix.deep_clone());
    }

    if d == PixelDepth::Bit8 {
        return bilateral_gray(pix, spatial_stdev, range_stdev, ncomps, reduction);
    }

    // 32bpp: process each channel (R, G, B) independently.
    // Alpha channel is not filtered (matches C version behavior).
    let pix_r = pix.get_rgb_component(RgbComponent::Red)?;
    let pix_g = pix.get_rgb_component(RgbComponent::Green)?;
    let pix_b = pix.get_rgb_component(RgbComponent::Blue)?;

    let res_r = bilateral_gray(&pix_r, spatial_stdev, range_stdev, ncomps, reduction)?;
    let res_g = bilateral_gray(&pix_g, spatial_stdev, range_stdev, ncomps, reduction)?;
    let res_b = bilateral_gray(&pix_b, spatial_stdev, range_stdev, ncomps, reduction)?;

    Ok(Pix::create_rgb_image(&res_r, &res_g, &res_b)?)
}

/// Fast approximate separable bilateral filter for 8bpp grayscale
///
/// Uses the Principal Bilateral Components (PBC) algorithm for fast,
/// separable bilateral filtering on grayscale images.
///
/// C版: `pixBilateralGray()` in `bilateral.c`
///
/// # Arguments
/// * `pix` - 8bpp grayscale image (no colormap)
/// * `spatial_stdev` - Spatial Gaussian std dev in pixels (> 0.5)
/// * `range_stdev` - Range Gaussian std dev (> 5.0; typical 50.0)
/// * `ncomps` - Number of PBC images [4..30]
/// * `reduction` - Downscaling factor: 1, 2, or 4
pub fn bilateral_gray(
    pix: &Pix,
    spatial_stdev: f32,
    range_stdev: f32,
    ncomps: u32,
    reduction: u32,
) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8bpp",
            actual: pix.depth().bits(),
        });
    }
    validate_bilateral_params(pix, spatial_stdev, range_stdev, ncomps, reduction)?;

    // Check image size
    let filtersize = (2.0 * spatial_stdev + 1.5) as u32;
    let w = pix.width();
    let h = pix.height();
    if w < 2 * filtersize || h < 2 * filtersize {
        return Ok(pix.deep_clone());
    }

    let bil = BilateralData::create(pix, spatial_stdev, range_stdev, ncomps, reduction)?;
    bilateral_apply(&bil, pix)
}

/// Validate bilateral filter parameters
fn validate_bilateral_params(
    _pix: &Pix,
    spatial_stdev: f32,
    range_stdev: f32,
    ncomps: u32,
    reduction: u32,
) -> FilterResult<()> {
    if reduction != 1 && reduction != 2 && reduction != 4 {
        return Err(FilterError::InvalidParameters(
            "reduction must be 1, 2, or 4".into(),
        ));
    }
    let sstdev = spatial_stdev / reduction as f32;
    if sstdev < 0.5 {
        return Err(FilterError::InvalidParameters(
            "spatial_stdev / reduction must be >= 0.5".into(),
        ));
    }
    if range_stdev <= 5.0 {
        return Err(FilterError::InvalidParameters(
            "range_stdev must be > 5.0".into(),
        ));
    }
    if !(4..=30).contains(&ncomps) {
        return Err(FilterError::InvalidParameters(
            "ncomps must be in [4..30]".into(),
        ));
    }
    if (ncomps as f32) * range_stdev < 100.0 {
        return Err(FilterError::InvalidParameters(
            "ncomps * range_stdev must be >= 100".into(),
        ));
    }
    Ok(())
}

/// Intermediate data for fast separable bilateral filter
struct BilateralData {
    /// Reduction factor
    reduction: u32,
    /// Maps intensity → lower PBC index (size 256)
    kindex: Vec<u32>,
    /// Maps intensity → interpolation fraction (size 256)
    kfract: Vec<f32>,
    /// PBC images at reduced resolution (size ncomps)
    pbc_images: Vec<Pix>,
}

impl BilateralData {
    /// Create bilateral filter data including PBC images
    fn create(
        pix: &Pix,
        spatial_stdev: f32,
        range_stdev: f32,
        ncomps: u32,
        reduction: u32,
    ) -> FilterResult<Self> {
        use leptonica_transform::{ScaleMethod, scale};

        // Downscale
        let pix_reduced = if reduction == 1 {
            pix.deep_clone()
        } else {
            let factor = 1.0 / reduction as f32;
            scale(pix, factor, factor, ScaleMethod::AreaMap)?
        };

        let sstdev = spatial_stdev / reduction as f32;
        let border = (2.0 * sstdev + 1.0) as u32;

        // Add mirrored border
        let pixsc = pix_reduced.add_mirrored_border(border, border, border, border)?;

        // Use full 8-bit intensity range for LUT construction.
        // When reduction > 1, the downscaled image may have a narrower
        // intensity range than the original. Using the full [0..255] range
        // ensures that bilateral_apply (which indexes by original pixel
        // values) always finds valid LUT entries.
        let minval: u32 = 0;
        let maxval: u32 = 255;

        // Generate k values
        let nc: Vec<u32> = (0..ncomps as usize)
            .map(|i| minval + (i as u32) * (maxval - minval) / (ncomps - 1))
            .collect();

        // Generate kindex: maps intensity → lower PBC index
        let mut kindex = vec![0u32; 256];
        {
            let mut i = minval as usize;
            let mut k = 0usize;
            while i <= maxval as usize && k < (ncomps - 1) as usize {
                let fval2 = nc[k + 1];
                while (i as u32) < fval2 {
                    kindex[i] = k as u32;
                    i += 1;
                }
                k += 1;
            }
            kindex[maxval as usize] = ncomps - 2;
        }

        // Generate kfract: maps intensity → interpolation fraction
        let mut kfract = vec![0.0f32; 256];
        {
            let mut i = minval as usize;
            let mut k = 0usize;
            while i <= maxval as usize && k < (ncomps - 1) as usize {
                let fval1 = nc[k] as f32;
                let fval2 = nc[k + 1] as f32;
                while (i as f32) < fval2 {
                    kfract[i] = (i as f32 - fval1) / (fval2 - fval1);
                    i += 1;
                }
                k += 1;
            }
            kfract[maxval as usize] = 1.0;
        }

        // Generate 1D kernels
        let spatial_size = (2.0 * sstdev + 1.0) as usize;
        let spatial_denom = 2.0 * sstdev * sstdev;
        let spatial: Vec<f32> = (0..spatial_size)
            .map(|i| (-(i as f32 * i as f32) / spatial_denom).exp())
            .collect();

        let range_denom = 2.0 * range_stdev * range_stdev;
        let range: Vec<f32> = (0..256)
            .map(|i| (-(i as f32 * i as f32) / range_denom).exp())
            .collect();

        // Generate PBC images
        // Use the actual reduced image dimensions (from scale() output),
        // not div_ceil, to match the mirrored border layout.
        let wd = pix_reduced.width();
        let hd = pix_reduced.height();
        let halfwidth = (2.0 * sstdev) as i32;

        let mut pbc_images = Vec::with_capacity(ncomps as usize);

        for &kval_u in &nc {
            let kval = kval_u as i32;

            // Copy pixsc for horizontal convolution
            let pixt = pixsc.deep_clone();
            let mut pixt_mut = pixt.try_into_mut().unwrap();

            // Horizontal separable convolution.
            // Process all rows that the vertical pass will read:
            // rows [border-halfwidth .. border+hd+halfwidth-1] in the bordered image.
            let border_i = border as i32;
            let hw = halfwidth as u32;
            let row_start = border.saturating_sub(hw);
            let row_end = (border + hd + hw).min(pixsc.height());
            for i in row_start..row_end {
                for j in 0..wd {
                    let mut sum = 0.0f32;
                    let mut norm = 0.0f32;
                    for k in -halfwidth..=halfwidth {
                        let sx = (border_i + j as i32 + k) as u32;
                        let nval = pixsc.get_pixel_unchecked(sx, i) as i32;
                        let kern = spatial[k.unsigned_abs() as usize]
                            * range[(kval - nval).unsigned_abs() as usize];
                        sum += kern * nval as f32;
                        norm += kern;
                    }
                    if norm > 0.0 {
                        let dval = (sum / norm + 0.5) as u32;
                        pixt_mut.set_pixel_unchecked(border + j, i, dval.min(255));
                    }
                }
            }

            let pixt: Pix = pixt_mut.into();

            // Vertical separable convolution → output PBC image
            let pixd = Pix::new(wd, hd, PixelDepth::Bit8)?;
            let mut pixd_mut = pixd.try_into_mut().unwrap();

            for i in 0..hd {
                for j in 0..wd {
                    let mut sum = 0.0f32;
                    let mut norm = 0.0f32;
                    for k in -halfwidth..=halfwidth {
                        let sx = (border_i + j as i32) as u32;
                        let sy = (border_i + i as i32 + k) as u32;
                        let nval = pixt.get_pixel_unchecked(sx, sy) as i32;
                        let kern = spatial[k.unsigned_abs() as usize]
                            * range[(kval - nval).unsigned_abs() as usize];
                        sum += kern * nval as f32;
                        norm += kern;
                    }
                    let dval = if norm > 0.0 {
                        (sum / norm + 0.5) as u32
                    } else {
                        pixt.get_pixel_unchecked(border + j, border + i)
                    };
                    pixd_mut.set_pixel_unchecked(j, i, dval.min(255));
                }
            }

            pbc_images.push(pixd_mut.into());
        }

        Ok(Self {
            reduction,
            kindex,
            kfract,
            pbc_images,
        })
    }
}

/// Apply pre-computed bilateral filter to produce the output image
fn bilateral_apply(bil: &BilateralData, pix: &Pix) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let reduction = bil.reduction;
    // PBC image dimensions (from actual scale() output)
    let pbc_w = bil.pbc_images[0].width();
    let pbc_h = bil.pbc_images[0].height();

    let pixd = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut pixd_mut = pixd.try_into_mut().unwrap();

    for i in 0..h {
        let ired = (i / reduction).min(pbc_h - 1);
        for j in 0..w {
            let jred = (j / reduction).min(pbc_w - 1);
            let vals = pix.get_pixel_unchecked(j, i);
            let k = bil.kindex[vals as usize] as usize;
            let lowval = bil.pbc_images[k].get_pixel_unchecked(jred, ired) as f32;
            let hival = bil.pbc_images[k + 1].get_pixel_unchecked(jred, ired) as f32;
            let fract = bil.kfract[vals as usize];
            let vald = ((1.0 - fract) * lowval + fract * hival + 0.5) as u32;
            pixd_mut.set_pixel_unchecked(j, i, vald.min(255));
        }
    }

    Ok(pixd_mut.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_gray_image() -> Pix {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create an image with a sharp edge
        for y in 0..20 {
            for x in 0..20 {
                let val = if x < 10 { 50 } else { 200 };
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }

        pix_mut.into()
    }

    fn create_test_color_image() -> Pix {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..20 {
            for x in 0..20 {
                let r = if x < 10 { 50 } else { 200 };
                let g = if y < 10 { 100 } else { 150 };
                let b = 128;
                let pixel = color::compose_rgb(r, g, b);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_make_range_kernel() {
        let kernel = make_range_kernel(30.0).unwrap();

        // Value at 0 should be 1.0 (no difference)
        assert!((kernel[0] - 1.0).abs() < 0.001);

        // Values should decrease with distance
        assert!(kernel[0] > kernel[30]);
        assert!(kernel[30] > kernel[60]);

        // Should be monotonically decreasing
        for i in 1..256 {
            assert!(kernel[i] <= kernel[i - 1]);
        }
    }

    #[test]
    fn test_make_range_kernel_invalid() {
        assert!(make_range_kernel(0.0).is_err());
        assert!(make_range_kernel(-1.0).is_err());
    }

    #[test]
    fn test_bilateral_exact_gray() {
        let pix = create_test_gray_image();
        let result = bilateral_exact(&pix, 2.0, 30.0).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit8);

        // Edge should be preserved (values at x=5 and x=15 should be distinct)
        let val_left = result.get_pixel_unchecked(5, 10);
        let val_right = result.get_pixel_unchecked(15, 10);
        assert!(val_right > val_left + 50); // Edge preserved
    }

    #[test]
    fn test_bilateral_exact_color() {
        let pix = create_test_color_image();
        let result = bilateral_exact(&pix, 2.0, 30.0).unwrap();

        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_bilateral_exact_invalid_params() {
        let pix = create_test_gray_image();

        assert!(bilateral_exact(&pix, 0.0, 30.0).is_err());
        assert!(bilateral_exact(&pix, -1.0, 30.0).is_err());
        assert!(bilateral_exact(&pix, 2.0, 0.0).is_err());
        assert!(bilateral_exact(&pix, 2.0, -1.0).is_err());
    }

    #[test]
    fn test_bilateral_gray_exact_with_spatial_only() {
        let pix = create_test_gray_image();
        let spatial_kernel = Kernel::gaussian(5, 1.0).unwrap();

        // Without range kernel, should behave like Gaussian blur
        let result = bilateral_gray_exact(&pix, &spatial_kernel, None).unwrap();
        assert_eq!(result.width(), pix.width());
    }

    #[test]
    fn test_bilateral_small_image() {
        // Create a very small image
        let pix = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
        let pix_mut = pix.try_into_mut().unwrap();
        let pix = pix_mut.into();

        // Should return a copy without error
        let result = bilateral_exact(&pix, 2.0, 30.0);
        assert!(result.is_ok());
    }
}
