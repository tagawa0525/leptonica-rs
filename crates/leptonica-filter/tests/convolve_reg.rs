//! Convolution regression test
//!
//! C version: reference/leptonica/prog/convolve_reg.c
//!
//! Tests convolution, box blur, and Gaussian blur operations.
//!
//! C API mapping:
//! - pixBlockconv -> box_blur
//! - pixConvolve -> convolve
//! - Custom Gaussian -> gaussian_blur

use leptonica_filter::{Kernel, box_blur, convolve, gaussian_blur};
use leptonica_test::{RegParams, load_test_image};

/// Test convolution operations.
///
/// C tests: pixBlockconv (8bpp/32bpp), pixConvolve with custom/flat kernels,
/// pixBlockrank, pixBlocksum, pixCensusTransform, pixConvolveWithBias,
/// pixWindowedMean/pixWindowedMeanSquare/pixWindowedVariance.
///
/// Rust: Tests box_blur, gaussian_blur, and convolve with custom kernel.
#[test]
#[ignore = "not yet implemented"]
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
    let identity_data = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
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

fn pixel_variance(pix: &leptonica_core::Pix) -> f64 {
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
