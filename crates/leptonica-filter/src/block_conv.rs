//! Block convolution using integral images (summed area tables)
//!
//! Fast block average filter that runs in O(1) per pixel regardless of
//! kernel size, by precomputing an integral image (accumulator).
//!
//! # See also
//!
//! C Leptonica: `convolve.c` (`pixBlockconv`, `pixBlockconvGray`,
//! `pixBlockconvAccum`, `pixBlockconvGrayUnnormalized`)

use crate::{FilterError, FilterResult};
use leptonica_core::pix::RgbComponent;
use leptonica_core::{Pix, PixelDepth};

/// Validate that the input image is 8 bpp grayscale.
fn check_8bpp(pix: &Pix) -> FilterResult<()> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

/// Create a copy of an 8 bpp image by duplicating all pixel data.
fn copy_pix(pix: &Pix) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, pix.depth())?;
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            out_mut.set_pixel_unchecked(x, y, pix.get_pixel_unchecked(x, y));
        }
    }
    Ok(out_mut.into())
}

/// Build an integral image (summed area table) from an 8 bpp grayscale image.
///
/// Each pixel in the output 32 bpp image contains the sum of all source
/// pixel values in the rectangle from (0,0) to (x,y) inclusive.
///
/// The recursion is: `a(i,j) = v(i,j) + a(i-1,j) + a(i,j-1) - a(i-1,j-1)`
///
/// # See also
///
/// C Leptonica: `pixBlockconvAccum()` in `convolve.c`
pub fn blockconv_accum(pix: &Pix) -> FilterResult<Pix> {
    check_8bpp(pix)?;

    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out.try_into_mut().unwrap();

    // First pixel
    out_mut.set_pixel_unchecked(0, 0, pix.get_pixel_unchecked(0, 0));

    // First row: cumulative sum along x
    for x in 1..w {
        let val = pix.get_pixel_unchecked(x, 0) + out_mut.get_pixel_unchecked(x - 1, 0);
        out_mut.set_pixel_unchecked(x, 0, val);
    }

    // First column: cumulative sum along y
    for y in 1..h {
        let val = pix.get_pixel_unchecked(0, y) + out_mut.get_pixel_unchecked(0, y - 1);
        out_mut.set_pixel_unchecked(0, y, val);
    }

    // Interior: a(i,j) = v(i,j) + a(i-1,j) + a(i,j-1) - a(i-1,j-1)
    for y in 1..h {
        for x in 1..w {
            let val = pix.get_pixel_unchecked(x, y)
                + out_mut.get_pixel_unchecked(x - 1, y)
                + out_mut.get_pixel_unchecked(x, y - 1)
                - out_mut.get_pixel_unchecked(x - 1, y - 1);
            out_mut.set_pixel_unchecked(x, y, val);
        }
    }

    Ok(out_mut.into())
}

/// Fast block convolution on an 8 bpp grayscale image using an integral image.
///
/// `wc` and `hc` are the half-width and half-height of the convolution kernel.
/// The full kernel size is `(2*wc + 1) x (2*hc + 1)`.
///
/// If either `wc` or `hc` is 0, returns a copy of the input.
/// If the kernel is larger than the image, it is automatically reduced.
///
/// An optional pre-computed accumulator (`pixacc`) can be provided to avoid
/// redundant computation when the same image is convolved multiple times.
///
/// # See also
///
/// C Leptonica: `pixBlockconvGray()` in `convolve.c`
pub fn blockconv_gray(pix: &Pix, pixacc: Option<&Pix>, wc: u32, hc: u32) -> FilterResult<Pix> {
    check_8bpp(pix)?;

    let w = pix.width();
    let h = pix.height();

    if wc == 0 || hc == 0 {
        return copy_pix(pix);
    }

    // Reduce kernel if it exceeds image dimensions
    let wc = wc.min((w - 1) / 2);
    let hc = hc.min((h - 1) / 2);

    // Use provided accumulator or compute one
    let owned_acc;
    let acc = match pixacc {
        Some(a) => a,
        None => {
            owned_acc = blockconv_accum(pix)?;
            &owned_acc
        }
    };

    let fwc = (2 * wc + 1) as f64;
    let fhc = (2 * hc + 1) as f64;
    let norm = 1.0 / (fwc * fhc);

    let out = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for i in 0..h {
        let imin = if i > hc { i - hc - 1 } else { 0 };
        let imax = (i + hc).min(h - 1);
        // Actual window height at this row
        let hn = if i > hc {
            (imax - imin) as f64
        } else {
            (imax + 1) as f64
        };

        for j in 0..w {
            let jmin = if j > wc { j - wc - 1 } else { 0 };
            let jmax = (j + wc).min(w - 1);
            // Actual window width at this column
            let wn = if j > wc {
                (jmax - jmin) as f64
            } else {
                (jmax + 1) as f64
            };

            // Four-corner lookup on integral image
            let mut val = acc.get_pixel_unchecked(jmax, imax) as i64;
            if i > hc {
                val -= acc.get_pixel_unchecked(jmax, imin) as i64;
            }
            if j > wc {
                val -= acc.get_pixel_unchecked(jmin, imax) as i64;
            }
            if i > hc && j > wc {
                val += acc.get_pixel_unchecked(jmin, imin) as i64;
            }

            // Normalize with boundary correction
            let result = (norm * val as f64 * fwc / wn * fhc / hn + 0.5) as u32;
            let result = result.min(255);
            out_mut.set_pixel_unchecked(j, i, result);
        }
    }

    Ok(out_mut.into())
}

/// Fast block convolution on an 8 or 32 bpp image.
///
/// For 8 bpp, delegates to [`blockconv_gray`]. For 32 bpp, splits into
/// R/G/B channels, convolves each independently, and recombines.
///
/// `wc` and `hc` are the half-width and half-height of the convolution kernel.
///
/// # See also
///
/// C Leptonica: `pixBlockconv()` in `convolve.c`
pub fn blockconv(pix: &Pix, wc: u32, hc: u32) -> FilterResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit8 => blockconv_gray(pix, None, wc, hc),
        PixelDepth::Bit32 => {
            let pix_r = pix.get_rgb_component(RgbComponent::Red)?;
            let pix_g = pix.get_rgb_component(RgbComponent::Green)?;
            let pix_b = pix.get_rgb_component(RgbComponent::Blue)?;

            let conv_r = blockconv_gray(&pix_r, None, wc, hc)?;
            let conv_g = blockconv_gray(&pix_g, None, wc, hc)?;
            let conv_b = blockconv_gray(&pix_b, None, wc, hc)?;

            Ok(Pix::create_rgb_image(&conv_r, &conv_g, &conv_b)?)
        }
        _ => Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

/// Unnormalized block convolution on an 8 bpp grayscale image.
///
/// Returns a 32 bpp image where each pixel contains the raw sum of source
/// pixel values in the window, without dividing by the window area.
///
/// Uses mirrored border padding to avoid special boundary handling.
/// To get normalized results, divide each pixel value by `(2*wc+1)*(2*hc+1)`.
///
/// # See also
///
/// C Leptonica: `pixBlockconvGrayUnnormalized()` in `convolve.c`
pub fn blockconv_gray_unnormalized(pix: &Pix, wc: u32, hc: u32) -> FilterResult<Pix> {
    check_8bpp(pix)?;

    if wc == 0 || hc == 0 {
        return copy_pix(pix);
    }

    let w = pix.width();
    let h = pix.height();

    // Add mirrored border to avoid boundary handling
    let bordered = pix.add_mirrored_border(wc + 1, wc, hc + 1, hc)?;
    let acc = blockconv_accum(&bordered)?;

    let out = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for i in 0..h {
        for j in 0..w {
            let jmax = j + 2 * wc + 1;
            let val = acc.get_pixel_unchecked(jmax, i + 2 * hc + 1) as i64
                - acc.get_pixel_unchecked(jmax, i) as i64
                - acc.get_pixel_unchecked(j, i + 2 * hc + 1) as i64
                + acc.get_pixel_unchecked(j, i) as i64;
            out_mut.set_pixel_unchecked(j, i, val as u32);
        }
    }

    Ok(out_mut.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{PixelDepth, color};

    fn create_test_gray_image(w: u32, h: u32) -> Pix {
        let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..h {
            for x in 0..w {
                let val = ((x + y * w) % 256) as u32;
                pm.set_pixel_unchecked(x, y, val);
            }
        }
        pm.into()
    }

    fn create_uniform_gray_image(w: u32, h: u32, val: u32) -> Pix {
        let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..h {
            for x in 0..w {
                pm.set_pixel_unchecked(x, y, val);
            }
        }
        pm.into()
    }

    fn create_test_color_image(w: u32, h: u32) -> Pix {
        let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..h {
            for x in 0..w {
                let r = (x * 50 % 256) as u8;
                let g = (y * 50 % 256) as u8;
                let b = 128u8;
                pm.set_pixel_unchecked(x, y, color::compose_rgb(r, g, b));
            }
        }
        pm.into()
    }

    // ---- blockconv_accum tests ----

    #[test]

    fn test_blockconv_accum_basic() {
        // 3x3 image with known values
        let pix = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Row 0: 1, 2, 3
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(1, 0, 2);
        pm.set_pixel_unchecked(2, 0, 3);
        // Row 1: 4, 5, 6
        pm.set_pixel_unchecked(0, 1, 4);
        pm.set_pixel_unchecked(1, 1, 5);
        pm.set_pixel_unchecked(2, 1, 6);
        // Row 2: 7, 8, 9
        pm.set_pixel_unchecked(0, 2, 7);
        pm.set_pixel_unchecked(1, 2, 8);
        pm.set_pixel_unchecked(2, 2, 9);
        let pix: Pix = pm.into();

        let acc = blockconv_accum(&pix).unwrap();
        assert_eq!(acc.depth(), PixelDepth::Bit32);
        assert_eq!(acc.width(), 3);
        assert_eq!(acc.height(), 3);

        // Expected integral image:
        // Row 0: 1, 3, 6
        // Row 1: 5, 12, 21
        // Row 2: 12, 27, 45
        assert_eq!(acc.get_pixel_unchecked(0, 0), 1);
        assert_eq!(acc.get_pixel_unchecked(1, 0), 3);
        assert_eq!(acc.get_pixel_unchecked(2, 0), 6);
        assert_eq!(acc.get_pixel_unchecked(0, 1), 5);
        assert_eq!(acc.get_pixel_unchecked(1, 1), 12);
        assert_eq!(acc.get_pixel_unchecked(2, 1), 21);
        assert_eq!(acc.get_pixel_unchecked(0, 2), 12);
        assert_eq!(acc.get_pixel_unchecked(1, 2), 27);
        assert_eq!(acc.get_pixel_unchecked(2, 2), 45);
    }

    #[test]

    fn test_blockconv_accum_rejects_non_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(blockconv_accum(&pix).is_err());
    }

    // ---- blockconv_gray tests ----

    #[test]

    fn test_blockconv_gray_uniform_image() {
        // A uniform image convolved with any kernel should remain uniform
        let pix = create_uniform_gray_image(20, 20, 100);
        let result = blockconv_gray(&pix, None, 3, 3).unwrap();
        assert_eq!(result.width(), 20);
        assert_eq!(result.height(), 20);
        assert_eq!(result.depth(), PixelDepth::Bit8);

        // All pixels should be close to 100
        for y in 0..20 {
            for x in 0..20 {
                let val = result.get_pixel_unchecked(x, y);
                assert!(
                    (val as i32 - 100).unsigned_abs() <= 1,
                    "pixel ({},{}) = {}, expected ~100",
                    x,
                    y,
                    val
                );
            }
        }
    }

    #[test]

    fn test_blockconv_gray_preserves_dimensions() {
        let pix = create_test_gray_image(30, 25);
        let result = blockconv_gray(&pix, None, 2, 3).unwrap();
        assert_eq!(result.width(), 30);
        assert_eq!(result.height(), 25);
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]

    fn test_blockconv_gray_zero_kernel_returns_copy() {
        let pix = create_test_gray_image(10, 10);
        let result = blockconv_gray(&pix, None, 0, 3).unwrap();
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(
                    pix.get_pixel_unchecked(x, y),
                    result.get_pixel_unchecked(x, y)
                );
            }
        }
    }

    #[test]

    fn test_blockconv_gray_with_precomputed_accum() {
        let pix = create_test_gray_image(20, 20);
        let acc = blockconv_accum(&pix).unwrap();
        let result1 = blockconv_gray(&pix, None, 2, 2).unwrap();
        let result2 = blockconv_gray(&pix, Some(&acc), 2, 2).unwrap();
        for y in 0..20 {
            for x in 0..20 {
                assert_eq!(
                    result1.get_pixel_unchecked(x, y),
                    result2.get_pixel_unchecked(x, y),
                    "mismatch at ({},{})",
                    x,
                    y
                );
            }
        }
    }

    #[test]

    fn test_blockconv_gray_kernel_reduction() {
        // Image is 5x5, kernel half-width 10 would be too large
        let pix = create_uniform_gray_image(5, 5, 50);
        let result = blockconv_gray(&pix, None, 10, 10).unwrap();
        assert_eq!(result.width(), 5);
        assert_eq!(result.height(), 5);
    }

    #[test]

    fn test_blockconv_gray_rejects_non_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(blockconv_gray(&pix, None, 2, 2).is_err());
    }

    // ---- blockconv tests (auto-dispatch) ----

    #[test]

    fn test_blockconv_gray_dispatch() {
        let pix = create_test_gray_image(20, 20);
        let result = blockconv(&pix, 2, 2).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]

    fn test_blockconv_color_dispatch() {
        let pix = create_test_color_image(20, 20);
        let result = blockconv(&pix, 2, 2).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.width(), 20);
        assert_eq!(result.height(), 20);
    }

    #[test]

    fn test_blockconv_color_uniform() {
        // Uniform color image should stay uniform after block conv
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..20 {
            for x in 0..20 {
                pm.set_pixel_unchecked(x, y, color::compose_rgb(100, 150, 200));
            }
        }
        let pix: Pix = pm.into();

        let result = blockconv(&pix, 3, 3).unwrap();
        for y in 0..20 {
            for x in 0..20 {
                let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(x, y));
                assert!(
                    (r as i32 - 100).unsigned_abs() <= 1,
                    "R mismatch at ({},{})",
                    x,
                    y
                );
                assert!(
                    (g as i32 - 150).unsigned_abs() <= 1,
                    "G mismatch at ({},{})",
                    x,
                    y
                );
                assert!(
                    (b as i32 - 200).unsigned_abs() <= 1,
                    "B mismatch at ({},{})",
                    x,
                    y
                );
            }
        }
    }

    #[test]

    fn test_blockconv_rejects_unsupported_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(blockconv(&pix, 2, 2).is_err());
    }

    // ---- blockconv_gray_unnormalized tests ----

    #[test]

    fn test_blockconv_gray_unnormalized_basic() {
        // Uniform image: every window sum should be val * window_area
        let pix = create_uniform_gray_image(20, 20, 10);
        let result = blockconv_gray_unnormalized(&pix, 2, 2).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.width(), 20);
        assert_eq!(result.height(), 20);

        let window_area = (2 * 2 + 1) * (2 * 2 + 1); // 25
        let expected = 10 * window_area;
        for y in 0..20u32 {
            for x in 0..20u32 {
                let val = result.get_pixel_unchecked(x, y);
                assert_eq!(val, expected, "mismatch at ({},{}): got {}", x, y, val);
            }
        }
    }

    #[test]

    fn test_blockconv_gray_unnormalized_zero_kernel_returns_copy() {
        let pix = create_test_gray_image(10, 10);
        let result = blockconv_gray_unnormalized(&pix, 0, 3).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(
                    pix.get_pixel_unchecked(x, y),
                    result.get_pixel_unchecked(x, y)
                );
            }
        }
    }

    #[test]

    fn test_blockconv_gray_unnormalized_rejects_non_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(blockconv_gray_unnormalized(&pix, 2, 2).is_err());
    }
}
