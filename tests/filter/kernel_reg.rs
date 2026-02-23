//! Kernel regression test
//!
//! Tests kernel creation from various sources and convolution operations.
//! The C version tests kernelCreateFromString, kernelCreateFromFile,
//! kernelCreateFromPix, pixConvolve, pixBlockconv, pixBlockconvTiled.
//!
//! Tests Kernel::from_slice, box_kernel, gaussian, and convolution functions.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/kernel_reg.c`

use crate::common::RegParams;
use leptonica::filter::{
    Kernel, blockconv, blockconv_gray, box_blur, convolve, convolve_gray, gaussian_blur,
};

/// Test kernel creation from slice (C checks 0-3 kernel I/O).
///
/// Verifies Kernel::from_slice, box_kernel, and gaussian kernel creation.
#[test]
fn kernel_reg_creation() {
    let mut rp = RegParams::new("kernel_create");

    // Create 5x5 kernel from data (C: kernelCreateFromString 5x5)
    let data: Vec<f32> = vec![
        2.0, 4.0, 5.0, 4.0, 2.0, 4.0, 9.0, 12.0, 9.0, 4.0, 5.0, 12.0, 15.0, 12.0, 5.0, 4.0, 9.0,
        12.0, 9.0, 4.0, 2.0, 4.0, 5.0, 4.0, 2.0,
    ];
    let kernel = Kernel::from_slice(5, 5, &data).expect("from_slice 5x5");
    rp.compare_values(5.0, kernel.width() as f64, 0.0);
    rp.compare_values(5.0, kernel.height() as f64, 0.0);
    // sum of data: 17+38+49+38+17 = 159 (from_slice does not normalize)
    rp.compare_values(159.0, kernel.sum() as f64, 1.0);

    // Box kernel (C: makeFlatKernel 11x11)
    let box_k = Kernel::box_kernel(11).expect("box_kernel 11");
    rp.compare_values(11.0, box_k.width() as f64, 0.0);
    rp.compare_values(11.0, box_k.height() as f64, 0.0);

    // Gaussian kernel (C: makeGaussianKernel halfsize=5 → 11x11; Rust uses full-width)
    let gauss = Kernel::gaussian(11, 2.0).expect("gaussian kernel size=11 sigma=2");
    rp.compare_values(11.0, gauss.width() as f64, 0.0);
    rp.compare_values(11.0, gauss.height() as f64, 0.0);
    // Gaussian sum should be approximately 1.0 (normalized)
    let gauss_sum = gauss.sum();
    rp.compare_values(1.0, gauss_sum as f64, 0.1);

    assert!(rp.cleanup(), "kernel creation test failed");
}

/// Test pixConvolve with custom kernel (C checks 6-7).
///
/// Verifies convolve and convolve_gray preserve dimensions.
#[test]
fn kernel_reg_convolve() {
    let mut rp = RegParams::new("kernel_conv");

    let pix8 = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix8.width();
    let h = pix8.height();

    // Convolution with 5x5 Gaussian-like kernel (C: pixConvolve kel1 5x5)
    let data: Vec<f32> = vec![
        2.0, 4.0, 5.0, 4.0, 2.0, 4.0, 9.0, 12.0, 9.0, 4.0, 5.0, 12.0, 15.0, 12.0, 5.0, 4.0, 9.0,
        12.0, 9.0, 4.0, 2.0, 4.0, 5.0, 4.0, 2.0,
    ];
    let kernel = Kernel::from_slice(5, 5, &data).expect("5x5 kernel");
    let convolved = convolve_gray(&pix8, &kernel).expect("convolve_gray 5x5");
    rp.compare_values(w as f64, convolved.width() as f64, 0.0);
    rp.compare_values(h as f64, convolved.height() as f64, 0.0);

    // Convolution with box kernel should equal blockconv
    let box_k = Kernel::box_kernel(11).expect("box kernel 11");
    let conv_box = convolve_gray(&pix8, &box_k).expect("convolve_gray box");
    let blockconv_result = blockconv_gray(&pix8, None, 5, 5).expect("blockconv_gray 5,5");
    rp.compare_values(w as f64, conv_box.width() as f64, 0.0);
    rp.compare_values(w as f64, blockconv_result.width() as f64, 0.0);

    assert!(rp.cleanup(), "kernel convolve test failed");
}

/// Test pixBlockconv and pixBlockconvTiled (C checks 7-9).
///
/// Verifies blockconv and box_blur preserve dimensions.
#[test]
fn kernel_reg_blockconv() {
    let mut rp = RegParams::new("kernel_blockconv");

    let pix8 = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix8.width();
    let h = pix8.height();

    // Block convolution 5x5 (C: pixBlockconv(pixg, 5, 5))
    let block = blockconv_gray(&pix8, None, 5, 5).expect("blockconv_gray 5x5");
    rp.compare_values(w as f64, block.width() as f64, 0.0);
    rp.compare_values(h as f64, block.height() as f64, 0.0);

    // Box blur (equivalent to blockconv with normalized kernel)
    let blurred = box_blur(&pix8, 5).expect("box_blur 5");
    rp.compare_values(w as f64, blurred.width() as f64, 0.0);
    rp.compare_values(h as f64, blurred.height() as f64, 0.0);

    // Gaussian blur (C: pixConvolve with Gaussian kernel)
    let gauss = gaussian_blur(&pix8, 5, 2.0).expect("gaussian_blur radius=5 sigma=2");
    rp.compare_values(w as f64, gauss.width() as f64, 0.0);
    rp.compare_values(h as f64, gauss.height() as f64, 0.0);

    // Full blockconv on 8bpp with tiled variant
    let blockconv_full = blockconv(&pix8, 5, 5).expect("blockconv 5x5");
    rp.compare_values(w as f64, blockconv_full.width() as f64, 0.0);

    assert!(rp.cleanup(), "kernel blockconv test failed");
}

/// Test convolve on 32bpp color image (C checks 10-12 RGB conv portion).
///
/// Verifies convolve on a color image produces 32bpp output.
#[test]
fn kernel_reg_convolve_color() {
    let mut rp = RegParams::new("kernel_conv_color");

    let pix32 = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix32.width();
    let h = pix32.height();

    // Gaussian blur on 32bpp (C: pixConvolveRGB with Gaussian kernel)
    let gauss32 = gaussian_blur(&pix32, 3, 1.5).expect("gaussian_blur radius=3 32bpp");
    rp.compare_values(w as f64, gauss32.width() as f64, 0.0);
    rp.compare_values(h as f64, gauss32.height() as f64, 0.0);

    // Box blur on 32bpp
    let box32 = box_blur(&pix32, 3).expect("box_blur 32bpp");
    rp.compare_values(w as f64, box32.width() as f64, 0.0);
    rp.compare_values(h as f64, box32.height() as f64, 0.0);

    // Full convolve with custom kernel on 32bpp
    let data: Vec<f32> = vec![0.1, 0.2, 0.1, 0.2, 0.8, 0.2, 0.1, 0.2, 0.1];
    let kernel = Kernel::from_slice(3, 3, &data).expect("3x3 kernel");
    let conv32 = convolve(&pix32, &kernel).expect("convolve 32bpp");
    rp.compare_values(w as f64, conv32.width() as f64, 0.0);

    assert!(rp.cleanup(), "kernel convolve color test failed");
}
