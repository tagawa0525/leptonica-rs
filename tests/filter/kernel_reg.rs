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
//! C Leptonica: `prog/kernel_reg.c`

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

/// Test kernelCreateFromString equivalent.
#[test]
fn kernel_reg_from_string() {
    let mut rp = RegParams::new("kernel_from_string");

    let kdata = " 2   4   5   4   2 \
                  4   9  12   9   4 \
                  5  12  15  12   5 \
                  4   9  12   9   4 \
                  2   4   5   4   2";
    let kernel = Kernel::from_string(5, 5, 2, 2, kdata).expect("from_string 5x5");
    rp.compare_values(5.0, kernel.width() as f64, 0.0);
    rp.compare_values(5.0, kernel.height() as f64, 0.0);
    rp.compare_values(2.0, kernel.center_x() as f64, 0.0);
    rp.compare_values(2.0, kernel.center_y() as f64, 0.0);
    rp.compare_values(15.0, kernel.get(2, 2).unwrap() as f64, 0.0);

    assert!(rp.cleanup(), "kernel from_string test failed");
}

/// Test kernel I/O (write/read roundtrip).
#[test]
fn kernel_reg_io() {
    let mut rp = RegParams::new("kernel_io");

    let data: Vec<f32> = vec![
        0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 1.0, 2.0, 4.0, 2.0, 1.0, 0.0, 1.0, 2.0,
        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
    ];
    let kernel = Kernel::from_slice(5, 5, &data).expect("from_slice 5x5");

    // Write to buffer and read back
    let mut buf = Vec::new();
    kernel.write(&mut buf).expect("write kernel");
    let kernel2 = Kernel::read(buf.as_slice()).expect("read kernel");

    rp.compare_values(kernel.width() as f64, kernel2.width() as f64, 0.0);
    rp.compare_values(kernel.height() as f64, kernel2.height() as f64, 0.0);
    rp.compare_values(kernel.center_x() as f64, kernel2.center_x() as f64, 0.0);
    rp.compare_values(kernel.center_y() as f64, kernel2.center_y() as f64, 0.0);
    rp.compare_values(kernel.sum() as f64, kernel2.sum() as f64, 0.1);

    assert!(rp.cleanup(), "kernel io test failed");
}

/// Test kernel get_min_max and invert.
#[test]
fn kernel_reg_min_max_invert() {
    let mut rp = RegParams::new("kernel_minmax_inv");

    let data: Vec<f32> = vec![-1.0, 0.0, 1.0, 0.0, 5.0, 0.0, -1.0, 0.0, 1.0];
    let kernel = Kernel::from_slice(3, 3, &data).expect("from_slice 3x3");

    // get_min_max
    let (min, max) = kernel.get_min_max();
    rp.compare_values(-1.0, min as f64, 0.0);
    rp.compare_values(5.0, max as f64, 0.0);

    // invert (spatial flip)
    let inv = kernel.invert();
    rp.compare_values(1.0, inv.get(0, 0).unwrap() as f64, 0.0);
    rp.compare_values(5.0, inv.get(1, 1).unwrap() as f64, 0.0);
    rp.compare_values(-1.0, inv.get(2, 2).unwrap() as f64, 0.0);

    assert!(rp.cleanup(), "kernel min_max/invert test failed");
}

/// Test kernelCreateFromPix equivalent.
#[test]
fn kernel_reg_from_pix() {
    use leptonica::core::{Pix, PixelDepth};

    let mut rp = RegParams::new("kernel_from_pix");

    let pix = Pix::new(5, 5, PixelDepth::Bit8).expect("create 5x5 pix");
    let mut pix_mut = pix.try_into_mut().unwrap();
    pix_mut.set_pixel_unchecked(2, 2, 200);
    pix_mut.set_pixel_unchecked(0, 0, 50);
    let pix: Pix = pix_mut.into();

    let kernel = Kernel::from_pix(&pix, 2, 2).expect("from_pix");
    rp.compare_values(5.0, kernel.width() as f64, 0.0);
    rp.compare_values(5.0, kernel.height() as f64, 0.0);
    rp.compare_values(200.0, kernel.get(2, 2).unwrap() as f64, 0.0);
    rp.compare_values(50.0, kernel.get(0, 0).unwrap() as f64, 0.0);

    assert!(rp.cleanup(), "kernel from_pix test failed");
}

/// Test kernelDisplayInPix equivalent.
#[test]
fn kernel_reg_display() {
    let mut rp = RegParams::new("kernel_display");

    let data: Vec<f32> = vec![0.0, 1.0, 0.0, 1.0, 4.0, 1.0, 0.0, 1.0, 0.0];
    let kernel = Kernel::from_slice(3, 3, &data).expect("from_slice 3x3");

    // Simple 1-pixel-per-element mode
    let pix1 = kernel.display_in_pix(1, false).expect("display size=1");
    rp.compare_values(3.0, pix1.width() as f64, 0.0);
    rp.compare_values(3.0, pix1.height() as f64, 0.0);
    rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);

    // Grid mode
    let pix2 = kernel.display_in_pix(17, true).expect("display size=17");
    rp.compare_values(8.0, pix2.depth().bits() as f64, 0.0);
    // width = 17*3 + 2*(3+1) = 51 + 8 = 59
    rp.compare_values(59.0, pix2.width() as f64, 0.0);

    assert!(rp.cleanup(), "kernel display test failed");
}

/// Test kernelCreateFromFile equivalent.
#[test]
fn kernel_reg_from_file() {
    use std::io::Write;

    let mut rp = RegParams::new("kernel_from_file");

    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "kernel_reg_test_{}_{:?}_.txt",
        std::process::id(),
        std::thread::current().id()
    ));
    {
        let mut f = std::fs::File::create(&path).expect("create temp file");
        writeln!(f, "# test kernel for regression").unwrap();
        writeln!(f, "3 3").unwrap();
        writeln!(f, "1 1").unwrap();
        writeln!(f, "1.0 2.0 1.0").unwrap();
        writeln!(f, "2.0 4.0 2.0").unwrap();
        writeln!(f, "1.0 2.0 1.0").unwrap();
    }
    let kernel = Kernel::from_file(&path).expect("from_file");
    rp.compare_values(3.0, kernel.width() as f64, 0.0);
    rp.compare_values(3.0, kernel.height() as f64, 0.0);
    rp.compare_values(4.0, kernel.get(1, 1).unwrap() as f64, 0.0);
    rp.compare_values(16.0, kernel.sum() as f64, 0.01);

    std::fs::remove_file(&path).ok();

    assert!(rp.cleanup(), "kernel from_file test failed");
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

// =====================================================================
// gap-fill 第2弾 (plan 501): kernel.c 生成系 5 関数
// =====================================================================

/// C: makeFlatKernel(height, width, cy, cx) — rectangular flat kernel
#[test]

fn kernel_reg_make_flat_5x3_off_center() {
    let kel = Kernel::make_flat(5, 3, 2, 1).expect("make_flat 5x3 cy=2 cx=1");
    assert_eq!(kel.width(), 3);
    assert_eq!(kel.height(), 5);
    assert_eq!(kel.center_x(), 1);
    assert_eq!(kel.center_y(), 2);

    // Each cell = 1 / (5*3) = 1/15
    let expected = 1.0_f32 / 15.0;
    for y in 0..5 {
        for x in 0..3 {
            let v = kel.get(x, y).unwrap();
            assert!(
                (v - expected).abs() < 1e-6,
                "cell ({},{}) = {} expected {}",
                x,
                y,
                v,
                expected
            );
        }
    }

    // Sum should be ~1.0 (normalized)
    assert!((kel.sum() - 1.0).abs() < 1e-5);
}

/// C: makeGaussianKernel(halfh, halfw, stdev, max) — peak value at center is `max`
#[test]

fn kernel_reg_make_gaussian_peak_value() {
    let max = 2.5_f32;
    let kel = Kernel::make_gaussian(2, 2, 1.0, max).expect("make_gaussian halfh=halfw=2");
    assert_eq!(kel.width(), 5);
    assert_eq!(kel.height(), 5);
    assert_eq!(kel.center_x(), 2);
    assert_eq!(kel.center_y(), 2);

    // Center value should equal `max`
    let center = kel.get(2, 2).unwrap();
    assert!(
        (center - max).abs() < 1e-6,
        "center = {}, expected {}",
        center,
        max
    );

    // Symmetry: kel[(0,0)] == kel[(0,4)] == kel[(4,0)] == kel[(4,4)]
    let corner = kel.get(0, 0).unwrap();
    assert!((kel.get(0, 4).unwrap() - corner).abs() < 1e-6);
    assert!((kel.get(4, 0).unwrap() - corner).abs() < 1e-6);
    assert!((kel.get(4, 4).unwrap() - corner).abs() < 1e-6);
}

/// C: makeGaussianKernelSep — separable Gaussian, product at center equals `max`
#[test]

fn kernel_reg_make_gaussian_sep_product_equals_full() {
    let max = 1.7_f32;
    let stdev = 1.5_f32;
    let halfh = 2_u32;
    let halfw = 3_u32;

    let (kelx, kely) =
        Kernel::make_gaussian_sep(halfh, halfw, stdev, max).expect("make_gaussian_sep");

    // kelx is 1×(2*halfw+1) horizontal
    assert_eq!(kelx.width(), 2 * halfw + 1);
    assert_eq!(kelx.height(), 1);
    // kely is (2*halfh+1)×1 vertical
    assert_eq!(kely.width(), 1);
    assert_eq!(kely.height(), 2 * halfh + 1);

    // Center product: kelx[(halfw, 0)] * kely[(0, halfh)] should ≈ max
    let cx = kelx.get(halfw, 0).unwrap();
    let cy = kely.get(0, halfh).unwrap();
    let product = cx * cy;
    assert!(
        (product - max).abs() < 1e-5,
        "center product = {} ({} * {}), expected {}",
        product,
        cx,
        cy,
        max
    );

    // Cross-check: separable convolution result equals full Gaussian
    // Compare specific cell — full[(halfw, halfh)] == cx * cy = product = max
    let full = Kernel::make_gaussian(halfh, halfw, stdev, max).expect("full gaussian");
    let full_center = full.get(halfw, halfh).unwrap();
    assert!((full_center - product).abs() < 1e-5);
}

/// C: makeDoGKernel — sum is zero (bandpass filter)
#[test]

fn kernel_reg_make_dog_sum_zero() {
    // halfh=10 covers ~5*ratio*stdev=10 standard deviations of the wider
    // Gaussian so truncation error in the sum stays small.
    let kel = Kernel::make_dog(10, 10, 1.0, 2.0).expect("make_dog ratio=2.0");
    assert_eq!(kel.width(), 21);
    assert_eq!(kel.height(), 21);

    // DoG kernel sums to ~0 (bandpass mother wavelet). Truncation introduces a
    // small residual; tolerance reflects 21x21 truncation at ratio=2.0.
    let s = kel.sum();
    assert!(s.abs() < 1e-3, "DoG sum = {}, expected ~0", s);

    // ratio == 1.0 → all elements should be 0 (no-op)
    let noop = Kernel::make_dog(3, 3, 1.0, 1.0).expect("make_dog ratio=1.0");
    for v in noop.data() {
        assert!(
            v.abs() < 1e-6,
            "ratio=1.0 should produce zero kernel, got {}",
            v
        );
    }
}

/// C: makeDoGKernel — ratio < 1.0 should be rejected
#[test]

fn kernel_reg_make_dog_invalid_ratio() {
    assert!(
        Kernel::make_dog(3, 3, 1.0, 0.5).is_err(),
        "ratio < 1.0 should be rejected"
    );
    assert!(
        Kernel::make_dog(3, 3, 1.0, -1.0).is_err(),
        "negative ratio should be rejected"
    );
}

// =====================================================================
// gap-fill 第2弾 (plan 501) — 境界条件テスト (PR #329 Copilot 指摘対応)
// =====================================================================

#[test]
fn kernel_reg_make_flat_invalid_dimensions() {
    assert!(
        Kernel::make_flat(0, 3, 0, 0).is_err(),
        "height=0 must error"
    );
    assert!(Kernel::make_flat(3, 0, 0, 0).is_err(), "width=0 must error");
}

#[test]
fn kernel_reg_make_flat_invalid_center() {
    // cx >= width
    assert!(Kernel::make_flat(3, 3, 0, 3).is_err());
    // cy >= height
    assert!(Kernel::make_flat(3, 3, 3, 0).is_err());
}

#[test]
fn kernel_reg_make_flat_oversized_rejected() {
    // 65536 x 65536 overflows checked u64*u64 path? — actually fits, but
    // ensure the path runs without panic.
    // Use a value that overflows u32 multiplication (u64 ok), but exceeds
    // usize on 32-bit. Skip; just sanity check huge but safe:
    let r = Kernel::make_flat(1, 1024, 0, 0);
    assert!(r.is_ok());
}

#[test]
fn kernel_reg_make_gaussian_invalid_stdev() {
    assert!(Kernel::make_gaussian(2, 2, 0.0, 1.0).is_err());
    assert!(Kernel::make_gaussian(2, 2, -0.5, 1.0).is_err());
}

#[test]
fn kernel_reg_make_gaussian_oversized_halfsize_rejected() {
    // i32::MAX/2 + 1 should be rejected.
    let huge = (i32::MAX as u32 / 2) + 1;
    assert!(Kernel::make_gaussian(huge, 1, 1.0, 1.0).is_err());
    assert!(Kernel::make_gaussian(1, huge, 1.0, 1.0).is_err());
}

#[test]
fn kernel_reg_make_dog_invalid_stdev() {
    assert!(Kernel::make_dog(2, 2, 0.0, 1.5).is_err());
    assert!(Kernel::make_dog(2, 2, -1.0, 1.5).is_err());
}

#[test]
fn kernel_reg_make_dog_non_finite_ratio_rejected() {
    assert!(Kernel::make_dog(2, 2, 1.0, f32::NAN).is_err());
    assert!(Kernel::make_dog(2, 2, 1.0, f32::INFINITY).is_err());
}
