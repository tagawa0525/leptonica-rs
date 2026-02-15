//! Windowed statistics using integral images
//!
//! Computes local mean, mean-square, variance and RMS deviation over
//! rectangular sliding windows in O(1) per pixel, using summed area tables.
//!
//! # See also
//!
//! C Leptonica: `convolve.c` (`pixWindowedStats`, `pixWindowedMean`,
//! `pixWindowedMeanSquare`, `pixWindowedVariance`, `pixMeanSquareAccum`)

use crate::block_conv::blockconv_accum;
use crate::{FilterError, FilterResult};
use leptonica_core::{FPix, Pix, PixelDepth};

/// Result of [`windowed_stats`] containing all windowed statistics.
pub struct WindowedStatsResult {
    /// Windowed mean image (8 bpp)
    pub mean: Pix,
    /// Windowed mean-square image (32 bpp)
    pub mean_square: Pix,
    /// Variance image (floating-point)
    pub variance: FPix,
    /// RMS deviation image (floating-point)
    pub rms_deviation: FPix,
}

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

/// Validate that wc and hc are at least 2.
fn check_kernel_size(wc: u32, hc: u32) -> FilterResult<()> {
    if wc < 2 || hc < 2 {
        return Err(FilterError::InvalidParameters(
            "wc and hc must be >= 2".into(),
        ));
    }
    Ok(())
}

/// Internal f64 accumulator for mean-square computations.
///
/// Keeps full f64 precision to avoid rounding errors that accumulate
/// when large sums are stored in f32 (which only has 24 bits of mantissa).
/// The C version uses DPIX (f64) for the same reason.
struct MeanSquareAccumF64 {
    data: Vec<f64>,
    width: u32,
}

impl MeanSquareAccumF64 {
    #[inline]
    fn get(&self, x: u32, y: u32) -> f64 {
        self.data[(y as usize) * (self.width as usize) + (x as usize)]
    }
}

/// Build an f64 integral image of squared pixel values (internal).
fn mean_square_accum_f64(pix: &Pix) -> FilterResult<MeanSquareAccumF64> {
    check_8bpp(pix)?;

    let w = pix.width();
    let h = pix.height();
    let size = (w as usize) * (h as usize);
    let mut acc = vec![0.0f64; size];

    // First pixel
    let v0 = pix.get_pixel_unchecked(0, 0) as f64;
    acc[0] = v0 * v0;

    // First row
    for x in 1..w {
        let v = pix.get_pixel_unchecked(x, 0) as f64;
        acc[x as usize] = acc[(x - 1) as usize] + v * v;
    }

    // First column
    for y in 1..h {
        let v = pix.get_pixel_unchecked(0, y) as f64;
        let idx = (y as usize) * (w as usize);
        let idx_prev = ((y - 1) as usize) * (w as usize);
        acc[idx] = acc[idx_prev] + v * v;
    }

    // Interior
    for y in 1..h {
        for x in 1..w {
            let v = pix.get_pixel_unchecked(x, y) as f64;
            let idx = (y as usize) * (w as usize) + (x as usize);
            let idx_left = idx - 1;
            let idx_above = ((y - 1) as usize) * (w as usize) + (x as usize);
            let idx_diag = idx_above - 1;
            acc[idx] = v * v + acc[idx_left] + acc[idx_above] - acc[idx_diag];
        }
    }

    Ok(MeanSquareAccumF64 {
        data: acc,
        width: w,
    })
}

/// Build an integral image of squared pixel values from an 8 bpp image.
///
/// Each pixel in the output FPix contains the sum of all squared source
/// pixel values in the rectangle from (0,0) to (x,y) inclusive.
/// Uses `f64` precision internally to avoid overflow.
///
/// The recursion is: `a(i,j) = v(i,j)^2 + a(i-1,j) + a(i,j-1) - a(i-1,j-1)`
///
/// # See also
///
/// C Leptonica: `pixMeanSquareAccum()` in `convolve.c`
pub fn mean_square_accum(pix: &Pix) -> FilterResult<FPix> {
    let w = pix.width();
    let h = pix.height();
    let acc64 = mean_square_accum_f64(pix)?;

    let mut fpix = FPix::new(w, h)?;
    for y in 0..h {
        for x in 0..w {
            fpix.set_pixel_unchecked(x, y, acc64.get(x, y) as f32);
        }
    }

    Ok(fpix)
}

/// Compute the windowed mean of an 8 bpp image.
///
/// `wc` and `hc` are the half-width and half-height of the window.
/// The full window size is `(2*wc + 1) x (2*hc + 1)`.
///
/// A border of `(wc+1)` pixels on each side and `(hc+1)` on top/bottom
/// is added internally; the output dimensions match the original image.
///
/// If `normalize` is true, the output is an 8 bpp image with the mean value
/// in each window. If false, the output is a 32 bpp image with the raw sum.
///
/// # See also
///
/// C Leptonica: `pixWindowedMean()` in `convolve.c`
pub fn windowed_mean(pix: &Pix, wc: u32, hc: u32, normalize: bool) -> FilterResult<Pix> {
    check_8bpp(pix)?;
    check_kernel_size(wc, hc)?;

    // Add border
    let pixb = pix.add_border_general(wc + 1, wc + 1, hc + 1, hc + 1, 0)?;

    // Build integral image from bordered image
    let acc = blockconv_accum(&pixb)?;

    let wb = pixb.width();
    let hb = pixb.height();
    let wd = wb - 2 * (wc + 1);
    let hd = hb - 2 * (hc + 1);

    if wd < 2 || hd < 2 {
        return Err(FilterError::InvalidParameters(
            "image too small for the given kernel".into(),
        ));
    }

    let wincr = 2 * wc + 1;
    let hincr = 2 * hc + 1;

    let out_depth = if normalize {
        PixelDepth::Bit8
    } else {
        PixelDepth::Bit32
    };

    let out = Pix::new(wd, hd, out_depth)?;
    let mut out_mut = out.try_into_mut().unwrap();

    let norm: f64 = if normalize {
        1.0 / (wincr as f64 * hincr as f64)
    } else {
        1.0
    };

    for y in 0..hd {
        for x in 0..wd {
            // Four-corner lookup on integral image
            let val = acc.get_pixel_unchecked(x + wincr, y + hincr) as i64
                - acc.get_pixel_unchecked(x, y + hincr) as i64
                - acc.get_pixel_unchecked(x + wincr, y) as i64
                + acc.get_pixel_unchecked(x, y) as i64;

            if normalize {
                let result = (norm * val as f64) as u32;
                let result = result.min(255);
                out_mut.set_pixel_unchecked(x, y, result);
            } else {
                out_mut.set_pixel_unchecked(x, y, val as u32);
            }
        }
    }

    Ok(out_mut.into())
}

/// Compute the windowed mean of squared pixel values for an 8 bpp image.
///
/// `wc` and `hc` are the half-width and half-height of the window.
/// The output is a 32 bpp image where each pixel contains the normalized
/// mean of squared values over the window.
///
/// # See also
///
/// C Leptonica: `pixWindowedMeanSquare()` in `convolve.c`
pub fn windowed_mean_square(pix: &Pix, wc: u32, hc: u32) -> FilterResult<Pix> {
    check_8bpp(pix)?;
    check_kernel_size(wc, hc)?;

    // Add border
    let pixb = pix.add_border_general(wc + 1, wc + 1, hc + 1, hc + 1, 0)?;

    // Build mean-square accumulator from bordered image (using f64 precision)
    let msacc = mean_square_accum_f64(&pixb)?;

    let wb = pixb.width();
    let hb = pixb.height();
    let wd = wb - 2 * (wc + 1);
    let hd = hb - 2 * (hc + 1);

    if wd < 2 || hd < 2 {
        return Err(FilterError::InvalidParameters(
            "image too small for the given kernel".into(),
        ));
    }

    let wincr = 2 * wc + 1;
    let hincr = 2 * hc + 1;
    let norm: f64 = 1.0 / (wincr as f64 * hincr as f64);

    let out = Pix::new(wd, hd, PixelDepth::Bit32)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..hd {
        for x in 0..wd {
            // Four-corner lookup on mean-square accumulator (f64 precision)
            let val =
                msacc.get(x + wincr, y + hincr) - msacc.get(x, y + hincr) - msacc.get(x + wincr, y)
                    + msacc.get(x, y);

            let ival = (norm * val + 0.5) as u32;
            out_mut.set_pixel_unchecked(x, y, ival);
        }
    }

    Ok(out_mut.into())
}

/// Compute variance and RMS deviation from precomputed mean and mean-square images.
///
/// * `pixm` - 8 bpp mean image (from [`windowed_mean`] with normalize=true)
/// * `pixms` - 32 bpp mean-square image (from [`windowed_mean_square`])
///
/// Returns `(variance, rms_deviation)` as FPix images.
///
/// The variance is computed as: `<p*p> - <p>*<p>`
/// The RMS deviation is `sqrt(variance)`.
///
/// # See also
///
/// C Leptonica: `pixWindowedVariance()` in `convolve.c`
pub fn windowed_variance(pixm: &Pix, pixms: &Pix) -> FilterResult<(FPix, FPix)> {
    if pixm.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8-bpp grayscale (mean image)",
            actual: pixm.depth().bits(),
        });
    }
    if pixms.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32-bpp (mean-square image)",
            actual: pixms.depth().bits(),
        });
    }

    let w = pixm.width();
    let h = pixm.height();
    if w != pixms.width() || h != pixms.height() {
        return Err(FilterError::InvalidParameters(
            "mean and mean-square images must have the same dimensions".into(),
        ));
    }

    let mut fpixv = FPix::new(w, h)?;
    let mut fpixrv = FPix::new(w, h)?;

    for y in 0..h {
        for x in 0..w {
            let valm = pixm.get_pixel_unchecked(x, y) as f32;
            let valms = pixms.get_pixel_unchecked(x, y) as f32;
            let var = valms - valm * valm;
            fpixv.set_pixel_unchecked(x, y, var);
            fpixrv.set_pixel_unchecked(x, y, var.sqrt());
        }
    }

    Ok((fpixv, fpixrv))
}

/// Compute all windowed statistics for an 8 bpp image.
///
/// This is a convenience function that computes mean, mean-square,
/// variance and RMS deviation in one call.
///
/// # See also
///
/// C Leptonica: `pixWindowedStats()` in `convolve.c`
pub fn windowed_stats(pix: &Pix, wc: u32, hc: u32) -> FilterResult<WindowedStatsResult> {
    check_8bpp(pix)?;
    check_kernel_size(wc, hc)?;

    let mean = windowed_mean(pix, wc, hc, true)?;
    let mean_square = windowed_mean_square(pix, wc, hc)?;
    let (variance, rms_deviation) = windowed_variance(&mean, &mean_square)?;

    Ok(WindowedStatsResult {
        mean,
        mean_square,
        variance,
        rms_deviation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a uniform 8 bpp gray image filled with `val`.
    fn create_uniform_gray(w: u32, h: u32, val: u32) -> Pix {
        let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..h {
            for x in 0..w {
                pm.set_pixel_unchecked(x, y, val);
            }
        }
        pm.into()
    }

    /// Create a small 3x3 image with known pixel values 1..9
    fn create_3x3() -> Pix {
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
        pm.into()
    }

    // ---- mean_square_accum tests ----

    #[test]
    fn test_mean_square_accum_3x3() {
        let pix = create_3x3();
        let acc = mean_square_accum(&pix).unwrap();

        assert_eq!(acc.width(), 3);
        assert_eq!(acc.height(), 3);

        // Expected squared values: 1, 4, 9, 16, 25, 36, 49, 64, 81
        // Integral image of squares:
        // Row 0: 1, 1+4=5, 5+9=14
        // Row 1: 1+16=17, 17+4+25-1=45, 45+9+36-5=85 = 1+4+9+16+25+36=91
        //   Actually: a(0,1) = 1+16=17, a(1,1) = 4+17+5-1=25, wait...
        // Let me recompute:
        //   a(0,0) = 1^2 = 1
        //   a(1,0) = 1 + 2^2 = 5
        //   a(2,0) = 5 + 3^2 = 14
        //   a(0,1) = 1 + 4^2 = 17
        //   a(1,1) = 5^2 + a(0,1) + a(1,0) - a(0,0) = 25 + 17 + 5 - 1 = 46
        //   a(2,1) = 6^2 + a(1,1) + a(2,0) - a(1,0) = 36 + 46 + 14 - 5 = 91
        //   a(0,2) = 7^2 + a(0,1) = 49 + 17 = 66
        //   a(1,2) = 8^2 + a(0,2) + a(1,1) - a(0,1) = 64 + 66 + 46 - 17 = 159
        //   a(2,2) = 9^2 + a(1,2) + a(2,1) - a(1,1) = 81 + 159 + 91 - 46 = 285
        assert_eq!(acc.get_pixel_unchecked(0, 0), 1.0);
        assert_eq!(acc.get_pixel_unchecked(1, 0), 5.0);
        assert_eq!(acc.get_pixel_unchecked(2, 0), 14.0);
        assert_eq!(acc.get_pixel_unchecked(0, 1), 17.0);
        assert_eq!(acc.get_pixel_unchecked(1, 1), 46.0);
        assert_eq!(acc.get_pixel_unchecked(2, 1), 91.0);
        assert_eq!(acc.get_pixel_unchecked(0, 2), 66.0);
        assert_eq!(acc.get_pixel_unchecked(1, 2), 159.0);
        assert_eq!(acc.get_pixel_unchecked(2, 2), 285.0);
    }

    #[test]
    fn test_mean_square_accum_uniform() {
        let pix = create_uniform_gray(10, 10, 5);
        let acc = mean_square_accum(&pix).unwrap();

        // For uniform value 5, each squared value is 25.
        // The integral at (x,y) should be 25 * (x+1) * (y+1).
        for y in 0..10u32 {
            for x in 0..10u32 {
                let expected = 25.0 * (x + 1) as f32 * (y + 1) as f32;
                assert!(
                    (acc.get_pixel_unchecked(x, y) - expected).abs() < 0.01,
                    "mismatch at ({},{}): got {}, expected {}",
                    x,
                    y,
                    acc.get_pixel_unchecked(x, y),
                    expected
                );
            }
        }
    }

    #[test]
    fn test_mean_square_accum_rejects_non_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(mean_square_accum(&pix).is_err());
    }

    // ---- windowed_mean tests ----

    #[test]
    fn test_windowed_mean_uniform() {
        // For a uniform image with zero-padded border, only interior pixels
        // (where the window lies entirely within the original data) have the
        // correct mean. The border region sees zeros, which lowers the mean.
        let pix = create_uniform_gray(30, 30, 100);
        let wc = 3u32;
        let hc = 3u32;
        let result = windowed_mean(&pix, wc, hc, true).unwrap();

        assert_eq!(result.width(), 30);
        assert_eq!(result.height(), 30);
        assert_eq!(result.depth(), PixelDepth::Bit8);

        // Interior pixels where the full window is within the original image
        for y in wc..30 - wc {
            for x in wc..30 - wc {
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

        // Border pixels should be less than 100 (window extends into zeros)
        assert!(result.get_pixel_unchecked(0, 0) < 100);
    }

    #[test]
    fn test_windowed_mean_unnormalized_uniform() {
        let pix = create_uniform_gray(30, 30, 10);
        let wc = 3u32;
        let hc = 3u32;
        let result = windowed_mean(&pix, wc, hc, false).unwrap();

        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.width(), 30);
        assert_eq!(result.height(), 30);

        let window_area = (2 * wc + 1) * (2 * hc + 1); // 49
        let expected = 10 * window_area;

        // Only interior pixels where full window is inside original image
        for y in wc..30 - wc {
            for x in wc..30 - wc {
                let val = result.get_pixel_unchecked(x, y);
                assert_eq!(val, expected, "mismatch at ({},{})", x, y);
            }
        }
    }

    #[test]
    fn test_windowed_mean_preserves_dimensions() {
        let pix = create_uniform_gray(50, 40, 128);
        let result = windowed_mean(&pix, 5, 3, true).unwrap();
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 40);
    }

    #[test]
    fn test_windowed_mean_rejects_non_8bpp() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        assert!(windowed_mean(&pix, 3, 3, true).is_err());
    }

    #[test]
    fn test_windowed_mean_rejects_small_kernel() {
        let pix = create_uniform_gray(30, 30, 100);
        assert!(windowed_mean(&pix, 1, 3, true).is_err());
        assert!(windowed_mean(&pix, 3, 1, true).is_err());
    }

    // ---- windowed_mean_square tests ----

    #[test]
    fn test_windowed_mean_square_uniform() {
        // For uniform value v, mean-square in the interior = v*v
        let pix = create_uniform_gray(30, 30, 10);
        let wc = 3u32;
        let hc = 3u32;
        let result = windowed_mean_square(&pix, wc, hc).unwrap();

        assert_eq!(result.width(), 30);
        assert_eq!(result.height(), 30);
        assert_eq!(result.depth(), PixelDepth::Bit32);

        let expected = 10u32 * 10; // 100
        // Only interior pixels where full window is within original image
        for y in wc..30 - wc {
            for x in wc..30 - wc {
                let val = result.get_pixel_unchecked(x, y);
                assert!(
                    (val as i64 - expected as i64).unsigned_abs() <= 1,
                    "pixel ({},{}) = {}, expected ~{}",
                    x,
                    y,
                    val,
                    expected
                );
            }
        }
    }

    #[test]
    fn test_windowed_mean_square_rejects_non_8bpp() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        assert!(windowed_mean_square(&pix, 3, 3).is_err());
    }

    #[test]
    fn test_windowed_mean_square_rejects_small_kernel() {
        let pix = create_uniform_gray(30, 30, 100);
        assert!(windowed_mean_square(&pix, 1, 3).is_err());
        assert!(windowed_mean_square(&pix, 3, 0).is_err());
    }

    // ---- windowed_variance tests ----

    #[test]
    fn test_windowed_variance_uniform() {
        // For a uniform image, the interior variance should theoretically be 0,
        // but the algorithm produces a small positive artifact because the
        // mean is stored as 8bpp (integer truncation). For example, with
        // value=100, the float mean may truncate from 100.0 to 99, giving
        // variance = 10000 - 99*99 = 199. This matches C Leptonica behavior.
        //
        // We verify that variance is "small" relative to the mean-square
        // value, and that the rms = sqrt(variance) relationship holds.
        let pix = create_uniform_gray(30, 30, 100);
        let wc = 3u32;
        let hc = 3u32;
        let mean = windowed_mean(&pix, wc, hc, true).unwrap();
        let mean_sq = windowed_mean_square(&pix, wc, hc).unwrap();
        let (variance, rms) = windowed_variance(&mean, &mean_sq).unwrap();

        assert_eq!(variance.width(), 30);
        assert_eq!(variance.height(), 30);

        // Interior pixels: variance should be small relative to mean_square.
        // With integer truncation the maximum artifact for value v is
        // approximately 2*v (when mean truncates by 1).
        for y in wc..30 - wc {
            for x in wc..30 - wc {
                let var = variance.get_pixel_unchecked(x, y);
                let rms_val = rms.get_pixel_unchecked(x, y);

                // Variance artifact is at most ~2*v for value v=100 -> ~200
                assert!(
                    var < 250.0,
                    "variance at ({},{}) = {} is too large for a uniform image",
                    x,
                    y,
                    var
                );

                // RMS should be sqrt(variance) when variance >= 0
                if var >= 0.0 {
                    let expected_rms = var.sqrt();
                    assert!(
                        (rms_val - expected_rms).abs() < 0.01,
                        "rms at ({},{}) = {}, expected sqrt({}) = {}",
                        x,
                        y,
                        rms_val,
                        var,
                        expected_rms
                    );
                }
            }
        }
    }

    #[test]
    fn test_windowed_variance_relationship() {
        // Variance = <p*p> - <p>*<p>
        // For any input, variance >= 0 (within floating point tolerance)
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..30u32 {
            for x in 0..30u32 {
                pm.set_pixel_unchecked(x, y, (x * 8 + y * 3) % 256);
            }
        }
        let pix: Pix = pm.into();

        let mean = windowed_mean(&pix, 3, 3, true).unwrap();
        let mean_sq = windowed_mean_square(&pix, 3, 3).unwrap();
        let (variance, rms) = windowed_variance(&mean, &mean_sq).unwrap();

        for y in 0..30u32 {
            for x in 0..30u32 {
                let var = variance.get_pixel_unchecked(x, y);
                let rms_val = rms.get_pixel_unchecked(x, y);

                // Variance can be slightly negative due to integer rounding
                // of mean and mean-square, but should not be very negative.
                assert!(
                    var >= -2.0,
                    "variance at ({},{}) = {} is too negative",
                    x,
                    y,
                    var
                );

                // RMS should be sqrt of variance (or 0 if variance is negative)
                if var >= 0.0 {
                    let expected_rms = var.sqrt();
                    assert!(
                        (rms_val - expected_rms).abs() < 0.01,
                        "rms at ({},{}) = {}, expected {}",
                        x,
                        y,
                        rms_val,
                        expected_rms
                    );
                }
            }
        }
    }

    #[test]
    fn test_windowed_variance_rejects_wrong_depth() {
        let pix8 = create_uniform_gray(10, 10, 100);
        let pix32 = Pix::new(10, 10, PixelDepth::Bit32).unwrap();

        // mean must be 8bpp
        assert!(windowed_variance(&pix32, &pix32).is_err());
        // mean_square must be 32bpp
        assert!(windowed_variance(&pix8, &pix8).is_err());
    }

    #[test]
    fn test_windowed_variance_rejects_size_mismatch() {
        let pixm = create_uniform_gray(10, 10, 100);
        let pixms = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        assert!(windowed_variance(&pixm, &pixms).is_err());
    }

    // ---- windowed_stats tests ----

    #[test]
    fn test_windowed_stats_uniform() {
        let pix = create_uniform_gray(30, 30, 150);
        let wc = 3u32;
        let hc = 3u32;
        let stats = windowed_stats(&pix, wc, hc).unwrap();

        assert_eq!(stats.mean.width(), 30);
        assert_eq!(stats.mean.height(), 30);
        assert_eq!(stats.mean.depth(), PixelDepth::Bit8);

        assert_eq!(stats.mean_square.width(), 30);
        assert_eq!(stats.mean_square.depth(), PixelDepth::Bit32);

        assert_eq!(stats.variance.width(), 30);
        assert_eq!(stats.rms_deviation.width(), 30);

        // For uniform image, interior mean should be ~150
        for y in wc..30 - wc {
            for x in wc..30 - wc {
                let m = stats.mean.get_pixel_unchecked(x, y);
                assert!(
                    (m as i32 - 150).unsigned_abs() <= 1,
                    "mean at ({},{}) = {}, expected ~150",
                    x,
                    y,
                    m
                );
            }
        }
    }

    #[test]
    fn test_windowed_stats_rejects_non_8bpp() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        assert!(windowed_stats(&pix, 3, 3).is_err());
    }

    #[test]
    fn test_windowed_stats_rejects_small_kernel() {
        let pix = create_uniform_gray(30, 30, 100);
        assert!(windowed_stats(&pix, 1, 3).is_err());
    }
}
