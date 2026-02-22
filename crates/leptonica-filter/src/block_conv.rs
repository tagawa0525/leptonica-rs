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

    // Interior: a(y,x) = v(y,x) + a(y-1,x) + a(y,x-1) - a(y-1,x-1)
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
/// If either `wc` or `hc` is 0, returns a copy of the input (no-op).
/// This matches C Leptonica behavior: a 1×N or N×1 kernel is treated as
/// a no-op rather than performing one-dimensional averaging.
///
/// If the kernel is larger than the image, it is automatically reduced.
///
/// An optional pre-computed accumulator (`pixacc`) can be provided to avoid
/// redundant computation when the same image is convolved multiple times.
/// The accumulator must have the same dimensions as `pix` and be 32 bpp.
///
/// # See also
///
/// C Leptonica: `pixBlockconvGray()` in `convolve.c`
pub fn blockconv_gray(pix: &Pix, pixacc: Option<&Pix>, wc: u32, hc: u32) -> FilterResult<Pix> {
    check_8bpp(pix)?;

    let w = pix.width();
    let h = pix.height();

    if wc == 0 || hc == 0 {
        return Ok(pix.deep_clone());
    }

    // Reduce kernel if it exceeds image dimensions
    let wc = wc.min((w - 1) / 2);
    let hc = hc.min((h - 1) / 2);

    // Validate and use provided accumulator, or compute one
    let owned_acc;
    let acc = match pixacc {
        Some(a) => {
            if a.depth() != PixelDepth::Bit32 || a.width() != w || a.height() != h {
                return Err(FilterError::InvalidParameters(
                    "accumulator must be 32 bpp with same dimensions as input".into(),
                ));
            }
            a
        }
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

            // Normalize with boundary correction
            let result = (norm * val as f64 * fwc / wn * fhc / hn + 0.5) as u32;
            let result = result.min(255);
            out_mut.set_pixel_unchecked(x, y, result);
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
/// If either `wc` or `hc` is 0, returns a copy of the input as-is (8 bpp,
/// no-op). This matches C Leptonica behavior: a 1×N or N×1 kernel is
/// treated as a no-op rather than performing one-dimensional summing.
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
        return Ok(pix.deep_clone());
    }

    let w = pix.width();
    let h = pix.height();

    // Reduce kernel if it exceeds image dimensions
    let wc = wc.min((w - 1) / 2);
    let hc = hc.min((h - 1) / 2);

    // Add mirrored border to avoid boundary handling
    let bordered = pix.add_mirrored_border(wc + 1, wc, hc + 1, hc)?;
    let acc = blockconv_accum(&bordered)?;

    let out = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let xmax = x + 2 * wc + 1;
            let val = acc.get_pixel_unchecked(xmax, y + 2 * hc + 1) as i64
                - acc.get_pixel_unchecked(xmax, y) as i64
                - acc.get_pixel_unchecked(x, y + 2 * hc + 1) as i64
                + acc.get_pixel_unchecked(x, y) as i64;
            out_mut.set_pixel_unchecked(x, y, val as u32);
        }
    }

    Ok(out_mut.into())
}

/// Block convolution on a single pre-padded 8 bpp tile using an integral image.
///
/// The input tile must have at least `wc + 2` extra border pixels on each side.
/// Returns a smaller image containing only the convolved interior region
/// of size `(w - 2*(wc + 2)) × (h - 2*(hc + 2))`, i.e. with `wc + 2` pixels
/// stripped from each side.
///
/// If either `wc` or `hc` is 0, returns a copy of the input (no-op).
///
/// An optional pre-computed accumulator (`pixacc`) can be provided.
///
/// # See also
///
/// C Leptonica: `pixBlockconvGrayTile()` in `convolve.c`
pub fn blockconv_gray_tile(
    pixs: &Pix,
    pixacc: Option<&Pix>,
    wc: u32,
    hc: u32,
) -> FilterResult<Pix> {
    check_8bpp(pixs)?;

    let w = pixs.width();
    let h = pixs.height();

    if wc == 0 || hc == 0 {
        return Ok(pixs.deep_clone());
    }

    // Reduce kernel if it exceeds tile dimensions
    let mut wc = wc;
    let mut hc = hc;
    if w < 2 * wc + 3 || h < 2 * hc + 3 {
        wc = wc.min((w - 1) / 2);
        hc = hc.min((h - 1) / 2);
    }
    if wc == 0 || hc == 0 {
        return Ok(pixs.deep_clone());
    }

    // The output strips (wc+2) border pixels from each side.
    // This corresponds to the interior region that pixTilingPaintTile()
    // would extract from a full-size output in the C implementation.
    let border_w = 2 * (wc + 2);
    let border_h = 2 * (hc + 2);
    let wd = w
        .checked_sub(border_w)
        .ok_or_else(|| FilterError::InvalidParameters("tile width too small for kernel".into()))?;
    let hd = h
        .checked_sub(border_h)
        .ok_or_else(|| FilterError::InvalidParameters("tile height too small for kernel".into()))?;

    // Validate and use provided accumulator, or compute one
    let owned_acc;
    let acc = match pixacc {
        Some(a) => {
            if a.depth() != PixelDepth::Bit32 || a.width() != w || a.height() != h {
                return Err(FilterError::InvalidParameters(
                    "accumulator must be 32 bpp with same dimensions as input".into(),
                ));
            }
            a
        }
        None => {
            owned_acc = blockconv_accum(pixs)?;
            &owned_acc
        }
    };

    let norm = 1.0 / ((2 * wc + 1) as f64 * (2 * hc + 1) as f64);

    let out = Pix::new(wd, hd, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    // Compute convolution for each output pixel. The output pixel (ox, oy)
    // maps to source position (ox + wc + 2, oy + hc + 2), which is always
    // in the clean interior where no boundary clamping is needed.
    for oy in 0..hd {
        let i = oy + hc + 2;
        let imin = i - hc - 1; // always >= 1
        let imax = i + hc; // always <= h - 2

        for ox in 0..wd {
            let j = ox + wc + 2;
            let jmin = j - wc - 1; // always >= 1
            let jmax = j + wc; // always <= w - 2

            // Four-corner lookup on integral image:
            // sum of pixels in window [j-wc, j+wc] × [i-hc, i+hc]
            let val = acc.get_pixel_unchecked(jmax, imax) as i64
                - acc.get_pixel_unchecked(jmin, imax) as i64
                + acc.get_pixel_unchecked(jmin, imin) as i64
                - acc.get_pixel_unchecked(jmax, imin) as i64;

            let result = (norm * val as f64 + 0.5) as u32;
            out_mut.set_pixel_unchecked(ox, oy, result.min(255));
        }
    }

    Ok(out_mut.into())
}

/// Tiled block convolution on an 8 or 32 bpp image.
///
/// Divides the image into `nx × ny` tiles and convolves each tile
/// independently using [`blockconv_gray`].  This reduces peak memory
/// usage compared to [`blockconv`] because the integral-image accumulator
/// is only allocated for one tile at a time.
///
/// `wc` and `hc` are the half-width and half-height of the convolution kernel.
///
/// If `nx ≤ 1` **and** `ny ≤ 1`, delegates to [`blockconv`].
///
/// # See also
///
/// C Leptonica: `pixBlockconvTiled()` in `convolve.c`
pub fn blockconv_tiled(pix: &Pix, wc: u32, hc: u32, nx: u32, ny: u32) -> FilterResult<Pix> {
    if wc == 0 || hc == 0 {
        return Ok(pix.deep_clone());
    }

    // Validate tile counts: must be non-zero to avoid division-by-zero.
    if nx == 0 || ny == 0 {
        return Err(FilterError::InvalidParameters(
            "nx and ny must be non-zero".into(),
        ));
    }

    if nx <= 1 && ny <= 1 {
        return blockconv(pix, wc, hc);
    }

    let w = pix.width();
    let h = pix.height();

    // Reduce kernel if it exceeds image dimensions
    let mut wc = wc;
    let mut hc = hc;
    if w < 2 * wc + 3 || h < 2 * hc + 3 {
        wc = wc.min((w - 1) / 2);
        hc = hc.min((h - 1) / 2);
    }
    if wc == 0 || hc == 0 {
        return Ok(pix.deep_clone());
    }

    let d = pix.depth();
    if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: d.bits(),
        });
    }

    // Adjust nx/ny if tiles would be too small (each tile needs at least
    // (wc+2) × (hc+2) pixels for the overlap region to be meaningful).
    // Clamp to at least 1 to avoid division-by-zero after adjustment.
    let mut nx = nx;
    let mut ny = ny;
    if w / nx < wc + 2 {
        nx = (w / (wc + 2)).max(1);
    }
    if h / ny < hc + 2 {
        ny = (h / (hc + 2)).max(1);
    }

    // After adjustment, tiling may have collapsed to a single tile.
    // Delegate to blockconv() to avoid unnecessary per-tile overhead.
    if nx <= 1 && ny <= 1 {
        return blockconv(pix, wc, hc);
    }

    match d {
        PixelDepth::Bit8 => blockconv_tiled_gray(pix, wc, hc, nx, ny),
        _ => {
            // 32bpp: split into R/G/B channels, convolve each, recombine
            let pix_r = pix.get_rgb_component(RgbComponent::Red)?;
            let pix_g = pix.get_rgb_component(RgbComponent::Green)?;
            let pix_b = pix.get_rgb_component(RgbComponent::Blue)?;

            let conv_r = blockconv_tiled_gray(&pix_r, wc, hc, nx, ny)?;
            let conv_g = blockconv_tiled_gray(&pix_g, wc, hc, nx, ny)?;
            let conv_b = blockconv_tiled_gray(&pix_b, wc, hc, nx, ny)?;

            Ok(Pix::create_rgb_image(&conv_r, &conv_g, &conv_b)?)
        }
    }
}

/// Tiled block convolution on an 8 bpp image.
///
/// For each tile, clips a region from the source with `wc + 1` overlap on
/// each side (bounded by image dimensions), runs [`blockconv_gray`] on the
/// clipped region, and copies the relevant output pixels.  This uses the
/// same boundary normalization correction as the non-tiled version, ensuring
/// pixel-identical results while reducing peak memory usage.
fn blockconv_tiled_gray(pix: &Pix, wc: u32, hc: u32, nx: u32, ny: u32) -> FilterResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let wt = w / nx;
    let ht = h / ny;

    let out = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for iy in 0..ny {
        for jx in 0..nx {
            // Output region for this tile in source coordinates
            let out_x = jx * wt;
            let out_y = iy * ht;
            let out_w = if jx == nx - 1 { w - out_x } else { wt };
            let out_h = if iy == ny - 1 { h - out_y } else { ht };

            // Input region: extend by wc+1 overlap, clipped to image bounds.
            // The extra pixel beyond wc ensures blockconv_gray's integral
            // image has the correct xmin/ymin offsets for interior pixels.
            let clip_x = out_x.saturating_sub(wc + 1);
            let clip_y = out_y.saturating_sub(hc + 1);
            let clip_right = (out_x + out_w + wc).min(w - 1);
            let clip_bottom = (out_y + out_h + hc).min(h - 1);
            let clip_w = clip_right - clip_x + 1;
            let clip_h = clip_bottom - clip_y + 1;

            let tile = pix.clip_rectangle(clip_x, clip_y, clip_w, clip_h)?;
            let convolved = blockconv_gray(&tile, None, wc, hc)?;

            // Copy output pixels from the convolved tile.
            // The offset translates from source coords to tile coords.
            let off_x = out_x - clip_x;
            let off_y = out_y - clip_y;
            for dy in 0..out_h {
                for dx in 0..out_w {
                    let val = convolved.get_pixel_unchecked(off_x + dx, off_y + dy);
                    out_mut.set_pixel_unchecked(out_x + dx, out_y + dy, val);
                }
            }
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
                let val = (x + y * w) % 256;
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
        let pix = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(1, 0, 2);
        pm.set_pixel_unchecked(2, 0, 3);
        pm.set_pixel_unchecked(0, 1, 4);
        pm.set_pixel_unchecked(1, 1, 5);
        pm.set_pixel_unchecked(2, 1, 6);
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
        let pix = create_uniform_gray_image(20, 20, 100);
        let result = blockconv_gray(&pix, None, 3, 3).unwrap();
        assert_eq!(result.width(), 20);
        assert_eq!(result.height(), 20);
        assert_eq!(result.depth(), PixelDepth::Bit8);

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

    #[test]
    fn test_blockconv_gray_rejects_mismatched_accum() {
        let pix = create_test_gray_image(20, 20);
        // Wrong dimensions
        let bad_acc = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(blockconv_gray(&pix, Some(&bad_acc), 2, 2).is_err());
        // Wrong depth
        let bad_acc = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        assert!(blockconv_gray(&pix, Some(&bad_acc), 2, 2).is_err());
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
        // Zero kernel returns an 8 bpp copy (no-op, matching C Leptonica)
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

    // ---- blockconv_gray_tile tests ----

    #[test]
    fn test_blockconv_gray_tile_basic() {
        // Create a uniform 8bpp image with padding (wc=2, so border = wc+1 = 3)
        let pix = create_uniform_gray_image(20, 20, 100);
        let bordered = pix.add_mirrored_border(3, 3, 3, 3).unwrap();
        assert_eq!(bordered.width(), 26);
        assert_eq!(bordered.height(), 26);

        let result = blockconv_gray_tile(&bordered, None, 2, 2).unwrap();
        // Output: (26 - 2*2 - 2) × (26 - 2*2 - 2) = 18×18
        assert_eq!(result.width(), 18);
        assert_eq!(result.height(), 18);

        // Uniform input → all output pixels should be ~100
        for y in 0..result.height() {
            for x in 0..result.width() {
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
    fn test_blockconv_gray_tile_noop_zero_kernel() {
        let pix = create_test_gray_image(10, 10);
        let result = blockconv_gray_tile(&pix, None, 0, 2).unwrap();
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(
                    pix.get_pixel_unchecked(x, y),
                    result.get_pixel_unchecked(x, y)
                );
            }
        }
    }

    // ---- blockconv_tiled tests ----

    #[test]
    fn test_blockconv_tiled_matches_non_tiled() {
        let pix = create_test_gray_image(40, 40);
        let non_tiled = blockconv(&pix, 3, 3).unwrap();
        let tiled = blockconv_tiled(&pix, 3, 3, 2, 2).unwrap();

        assert_eq!(tiled.width(), 40);
        assert_eq!(tiled.height(), 40);

        for y in 0..40u32 {
            for x in 0..40u32 {
                let v1 = non_tiled.get_pixel_unchecked(x, y) as i32;
                let v2 = tiled.get_pixel_unchecked(x, y) as i32;
                assert!(
                    (v1 - v2).unsigned_abs() <= 1,
                    "mismatch at ({},{}): non_tiled={}, tiled={}",
                    x,
                    y,
                    v1,
                    v2
                );
            }
        }
    }

    #[test]
    fn test_blockconv_tiled_single_tile_delegates() {
        let pix = create_uniform_gray_image(20, 20, 128);
        let result = blockconv_tiled(&pix, 2, 2, 1, 1).unwrap();
        let expected = blockconv(&pix, 2, 2).unwrap();

        for y in 0..20u32 {
            for x in 0..20u32 {
                assert_eq!(
                    expected.get_pixel_unchecked(x, y),
                    result.get_pixel_unchecked(x, y),
                    "mismatch at ({},{})",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_blockconv_tiled_color() {
        let pix = create_test_color_image(30, 30);
        let non_tiled = blockconv(&pix, 2, 2).unwrap();
        let tiled = blockconv_tiled(&pix, 2, 2, 3, 3).unwrap();

        assert_eq!(tiled.depth(), PixelDepth::Bit32);

        for y in 0..30u32 {
            for x in 0..30u32 {
                let (r1, g1, b1) = color::extract_rgb(non_tiled.get_pixel_unchecked(x, y));
                let (r2, g2, b2) = color::extract_rgb(tiled.get_pixel_unchecked(x, y));
                assert!(
                    (r1 as i32 - r2 as i32).unsigned_abs() <= 1
                        && (g1 as i32 - g2 as i32).unsigned_abs() <= 1
                        && (b1 as i32 - b2 as i32).unsigned_abs() <= 1,
                    "mismatch at ({},{}): non_tiled=({},{},{}), tiled=({},{},{})",
                    x,
                    y,
                    r1,
                    g1,
                    b1,
                    r2,
                    g2,
                    b2
                );
            }
        }
    }
}
