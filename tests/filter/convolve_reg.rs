//! Convolution regression test
//!
//! C version: reference/leptonica/prog/convolve_reg.c
//!
//! Tests convolution, box blur, Gaussian blur, census transform,
//! blockconv, blockrank, blocksum, and windowed statistics operations.
//!
//! C checkpoint mapping (18 total):
//!  0  pixBlockconvGray on 8 bpp          -> blockconv_gray (write_pix_and_check)
//!  1  pixBlockconv on 8 bpp              -> blockconv (write_pix_and_check)
//!  2-4 pixBlockrank on 1 bpp (x3)        -> blockrank (write_pix_and_check)
//!  5  pixBlocksum on 1 bpp               -> blocksum (write_pix_and_check)
//!  6  pixCensusTransform                 -> census_transform (write_pix_and_check)
//!  7  pixConvolve with kel1              -> convolve (write_pix_and_check)
//!  8  pixConvolve with flat kel          -> convolve (write_pix_and_check)
//!  9  pixBlockconv on 32 bpp             -> blockconv (write_pix_and_check)
//! 10  pixConvolveWithBias non-sep        -> (stub – not implemented)
//! 11  pixConvolveWithBias sep            -> (stub – not implemented)
//! 12  pixWindowedMean                    -> windowed_mean (write_pix_and_check)
//! 13  pixWindowedVariance → pixrv        -> windowed_variance (write_pix_and_check)
//! 14  fpixDisplayMaxDynamicRange(fpixv)  -> (write_pix_and_check via to_pix)
//! 15  fpixDisplayMaxDynamicRange(fpixrv) -> (write_pix_and_check via to_pix)
//! 16  regTestComparePix variance         -> compare_pix
//! 17  regTestComparePix rms deviation    -> compare_pix

use crate::common::{RegParams, load_test_image};
use leptonica::NegativeHandling;
use leptonica::filter::{
    Kernel, blockconv, blockconv_accum, blockconv_gray, blockrank, blocksum, box_blur,
    census_transform, convolve, gaussian_blur, windowed_mean, windowed_mean_square, windowed_stats,
    windowed_variance,
};
use leptonica::io::ImageFormat;

/// Test blockconv_gray on 8 bpp (C checkpoint 0).
///
/// C: pixBlockconvGray(pixs, pixacc, 3, 5) -> regTestWritePixAndCheck /* 0 */
#[test]
fn convolve_blockconv_gray_reg() {
    let mut rp = RegParams::new("convolve_blockconv_gray");

    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");

    let pixacc = blockconv_accum(&pixs).expect("blockconv_accum");
    let pixd = blockconv_gray(&pixs, Some(&pixacc), 3, 5).expect("blockconv_gray");
    rp.write_pix_and_check(&pixd, ImageFormat::Jpeg)
        .expect("write blockconv_gray result");

    assert!(
        rp.cleanup(),
        "convolve_blockconv_gray regression test failed"
    );
}

/// Test blockconv on 8 bpp (C checkpoint 1).
///
/// C: pixBlockconv(pixs, 9, 8) -> regTestWritePixAndCheck /* 1 */
#[test]
fn convolve_blockconv_8bpp_reg() {
    let mut rp = RegParams::new("convolve_blockconv_8bpp");

    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");

    let pixd = blockconv(&pixs, 9, 8).expect("blockconv 8bpp");
    rp.write_pix_and_check(&pixd, ImageFormat::Jpeg)
        .expect("write blockconv 8bpp result");

    assert!(
        rp.cleanup(),
        "convolve_blockconv_8bpp regression test failed"
    );
}

/// Test blockrank on 1 bpp (C checkpoints 2-4).
///
/// C: for i in 0..3: pixBlockrank(pixs, pixacc, 4, 4, 0.25+0.25*i) /* 2-4 */
#[test]
fn convolve_blockrank_reg() {
    let mut rp = RegParams::new("convolve_blockrank");

    let pixs = load_test_image("test1.png").expect("load test1.png");

    for &rank in &[0.25f32, 0.50, 0.75] {
        let pixd = blockrank(&pixs, 4, 4, rank).expect("blockrank");
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("write blockrank result");
    }

    assert!(rp.cleanup(), "convolve_blockrank regression test failed");
}

/// Test blocksum on 1 bpp (C checkpoint 5).
///
/// C: pixBlocksum(pixs, pixacc, 16, 16) -> regTestWritePixAndCheck /* 5 */
#[test]
fn convolve_blocksum_reg() {
    let mut rp = RegParams::new("convolve_blocksum");

    let pixs = load_test_image("test1.png").expect("load test1.png");

    let pixd = blocksum(&pixs, 16, 16).expect("blocksum");
    rp.write_pix_and_check(&pixd, ImageFormat::Jpeg)
        .expect("write blocksum result");

    assert!(rp.cleanup(), "convolve_blocksum regression test failed");
}

/// Test census_transform (C checkpoint 6).
///
/// C: pixCensusTransform(pixg, 10, NULL) -> regTestWritePixAndCheck /* 6 */
/// where pixg is the green channel of test24.jpg scaled 0.5x.
#[test]
fn convolve_census_transform_reg() {
    let mut rp = RegParams::new("convolve_census_transform");

    // C uses pixScaleRGBToGrayFast(pixs, 2, COLOR_GREEN) on test24.jpg.
    // Rust: load the 24bpp image, convert to 8bpp grayscale.
    let pixs = load_test_image("test24.jpg").expect("load test24.jpg");
    let pixg = pixs.convert_to_8().expect("convert to 8bpp");

    let pixd = census_transform(&pixg, 10).expect("census_transform");
    rp.write_pix_and_check(&pixd, ImageFormat::Png)
        .expect("write census_transform result");

    assert!(
        rp.cleanup(),
        "convolve_census_transform regression test failed"
    );
}

/// Test convolve with custom kernel (C checkpoints 7-8).
///
/// C checkpoint 7: pixConvolve(pixg, kel1, 8, 1) where kel1 is a 5x5 Gaussian-like kernel.
/// C checkpoint 8: pixConvolve(pixg, kel2, 8, 1) where kel2 is a flat 11x11 kernel.
#[test]
fn convolve_custom_kernel_reg() {
    let mut rp = RegParams::new("convolve_custom_kernel");

    let pixs = load_test_image("test24.jpg").expect("load test24.jpg");
    let pixg = pixs.convert_to_8().expect("convert to 8bpp");

    // --- C checkpoint 7: kel1str – 5x5 Gaussian-like kernel (unnormalized) ---
    // The C kernel values are large (sum ~2760), so we normalize them here.
    #[rustfmt::skip]
    let kel1_data: Vec<f32> = vec![
         20.0,  50.0,  80.0,  50.0,  20.0,
         50.0, 100.0, 140.0, 100.0,  50.0,
         90.0, 160.0, 200.0, 160.0,  90.0,
         50.0, 100.0, 140.0, 100.0,  50.0,
         20.0,  50.0,  80.0,  50.0,  20.0,
    ];
    let sum: f32 = kel1_data.iter().sum();
    let kel1_norm: Vec<f32> = kel1_data.iter().map(|&v| v / sum).collect();
    let kel1 = Kernel::from_slice(5, 5, &kel1_norm).expect("create kel1");
    let pixd7 = convolve(&pixg, &kel1).expect("convolve kel1");
    rp.write_pix_and_check(&pixd7, ImageFormat::Jpeg)
        .expect("write convolve kel1 result");

    // --- C checkpoint 8: kel2 – flat 11x11 kernel (box blur equivalent) ---
    let n: usize = 11 * 11;
    let kel2_norm: Vec<f32> = vec![1.0 / n as f32; n];
    let kel2 = Kernel::from_slice(11, 11, &kel2_norm).expect("create kel2");
    let pixd8 = convolve(&pixg, &kel2).expect("convolve kel2");
    rp.write_pix_and_check(&pixd8, ImageFormat::Jpeg)
        .expect("write convolve kel2 result");

    assert!(
        rp.cleanup(),
        "convolve_custom_kernel regression test failed"
    );
}

/// Test blockconv on 32 bpp (C checkpoint 9).
///
/// C: pixt = pixScaleBySampling(pixs, 0.5, 0.5); pixBlockconv(pixt, 4, 6) /* 9 */
#[test]
fn convolve_blockconv_32bpp_reg() {
    let mut rp = RegParams::new("convolve_blockconv_32bpp");

    // Use weasel32.png as a readily available 32bpp image.
    let pix32 = load_test_image("weasel32.png").expect("load weasel32.png");
    let pixd = blockconv(&pix32, 4, 6).expect("blockconv 32bpp");
    rp.write_pix_and_check(&pixd, ImageFormat::Jpeg)
        .expect("write blockconv 32bpp result");

    assert!(
        rp.cleanup(),
        "convolve_blockconv_32bpp regression test failed"
    );
}

/// Test windowed_mean and windowed_mean_square (C checkpoints 12-15).
///
/// C checkpoints 12-15: pixWindowedMean, pixWindowedVariance,
/// fpixDisplayMaxDynamicRange for variance and RMS deviation.
///
/// C checkpoint 16-17: pixWindowedStats and regTestComparePix (compare_pix).
#[test]
fn convolve_windowed_stats_reg() {
    let mut rp = RegParams::new("convolve_windowed_stats");

    // C uses feyn-fract2.tif (1bpp), converts to 8bpp with pixConvertTo8.
    // We use feyn-fract.tif (1bpp) and convert to 8bpp.
    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let pixg = pixs.convert_to_8().expect("convert to 8bpp");

    let wc: u32 = 5;
    let hc: u32 = 20;

    // --- C checkpoint 12: pixWindowedMean ---
    // windowed_mean handles border addition internally.
    let pixm = windowed_mean(&pixg, wc, hc, true).expect("windowed_mean");
    rp.write_pix_and_check(&pixm, ImageFormat::Jpeg)
        .expect("write windowed_mean result");

    // --- C checkpoint 13: pixWindowedVariance → pixrv (RMS deviation as 8bpp) ---
    let pixms = windowed_mean_square(&pixg, wc, hc).expect("windowed_mean_square");
    let (fpixv, fpixrv) = windowed_variance(&pixm, &pixms).expect("windowed_variance");
    let pixrv = fpixrv
        .to_pix(8, NegativeHandling::ClipToZero)
        .expect("fpixrv to_pix");
    rp.write_pix_and_check(&pixrv, ImageFormat::Jpeg)
        .expect("write windowed_variance RMS result");

    // --- C checkpoint 14: fpixDisplayMaxDynamicRange(fpixv) ---
    // In C this normalizes the FPix for display. We use to_pix with auto depth.
    let pix1 = fpixv
        .to_pix(8, NegativeHandling::ClipToZero)
        .expect("fpixv to_pix");
    rp.write_pix_and_check(&pix1, ImageFormat::Jpeg)
        .expect("write variance display");

    // --- C checkpoint 15: fpixDisplayMaxDynamicRange(fpixrv) ---
    let pix2 = fpixrv
        .to_pix(8, NegativeHandling::ClipToZero)
        .expect("fpixrv to_pix for display");
    rp.write_pix_and_check(&pix2, ImageFormat::Jpeg)
        .expect("write rms display");

    // --- C checkpoints 16-17: pixWindowedStats + regTestComparePix ---
    // windowed_stats is the all-in-one interface; results must match the
    // step-by-step path above.
    let stats = windowed_stats(&pixg, wc, hc).expect("windowed_stats");
    let pix3 = stats
        .variance
        .to_pix(8, NegativeHandling::ClipToZero)
        .expect("stats.variance to_pix");
    let pix4 = stats
        .rms_deviation
        .to_pix(8, NegativeHandling::ClipToZero)
        .expect("stats.rms_deviation to_pix");
    rp.compare_pix(&pix1, &pix3); // C checkpoint 16
    rp.compare_pix(&pix2, &pix4); // C checkpoint 17

    assert!(
        rp.cleanup(),
        "convolve_windowed_stats regression test failed"
    );
}

/// Test convolution operations (original Rust-centric test, retained).
///
/// C tests: pixBlockconv (8bpp/32bpp), pixConvolve with custom/flat kernels,
/// pixBlockrank, pixBlocksum, pixCensusTransform, pixConvolveWithBias,
/// pixWindowedMean/pixWindowedMeanSquare/pixWindowedVariance.
///
/// Rust: Tests box_blur, gaussian_blur, and convolve with custom kernel.
#[test]
fn convolve_reg() {
    let mut rp = RegParams::new("convolve");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();

    // --- Test 1: Box blur ---
    // C: pixBlockconv(pixs, 9, 8) -- test 1
    for &radius in &[1, 2, 3, 5] {
        let blurred =
            box_blur(&pixs, radius).unwrap_or_else(|e| panic!("box_blur r={}: {}", radius, e));
        rp.compare_values(w as f64, blurred.width() as f64, 0.0);
        rp.compare_values(h as f64, blurred.height() as f64, 0.0);
    }

    // --- Test 2: Gaussian blur ---
    // C: uses custom Gaussian kernels
    for &(radius, sigma) in &[(2, 1.0), (3, 1.5), (5, 2.0)] {
        let blurred = gaussian_blur(&pixs, radius, sigma).expect("gaussian_blur");
        rp.compare_values(w as f64, blurred.width() as f64, 0.0);
        rp.compare_values(h as f64, blurred.height() as f64, 0.0);
    }

    // --- Test 3: Custom kernel convolution ---
    // C: kernelCreateFromString(5, 5, 2, 2, kel1str); pixConvolve(pixg, kel1, 8, 1)
    // Test with a 3x3 identity kernel (should produce near-identical output)
    let identity_data: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
    let kernel = Kernel::from_slice(3, 3, &identity_data).expect("create identity kernel");
    let conv = convolve(&pixs, &kernel).expect("convolve identity");
    rp.compare_values(w as f64, conv.width() as f64, 0.0);
    rp.compare_values(h as f64, conv.height() as f64, 0.0);

    // --- Test 4: Blur should reduce variance ---
    let blurred_strong = box_blur(&pixs, 5).expect("strong blur");
    let orig_var = pixel_variance(&pixs);
    let blur_var = pixel_variance(&blurred_strong);
    let var_reduced = blur_var <= orig_var;
    rp.compare_values(1.0, if var_reduced { 1.0 } else { 0.0 }, 0.0);

    // --- Test 5: Test with 32bpp color image ---
    // C: pixBlockconv on 32bpp (test 9)
    let pix32 = load_test_image("weasel32.png").expect("load 32bpp");
    let blur32 = box_blur(&pix32, 2).expect("box_blur 32bpp");
    rp.compare_values(pix32.width() as f64, blur32.width() as f64, 0.0);
    rp.compare_values(pix32.height() as f64, blur32.height() as f64, 0.0);

    assert!(rp.cleanup(), "convolve regression test failed");
}

fn pixel_variance(pix: &leptonica::Pix) -> f64 {
    let mut sum = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    let mut n = 0u64;
    let step = std::cmp::max(1, std::cmp::min(pix.width(), pix.height()) / 50) as usize;
    for y in (0..pix.height()).step_by(step) {
        for x in (0..pix.width()).step_by(step) {
            let v = pix.get_pixel(x, y).unwrap_or(0) as f64;
            sum += v;
            sum_sq += v * v;
            n += 1;
        }
    }
    if n == 0 {
        return 0.0;
    }
    let mean = sum / n as f64;
    sum_sq / n as f64 - mean * mean
}
