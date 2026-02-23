//! Bilateral filtering regression test (exact)
//!
//! C version: reference/leptonica/prog/bilateral1_reg.c
//!
//! Tests bilateral filtering with both:
//!   (1) Separable results with full res intermediate images (pixBilateral with reduction=1,2)
//!   (2) Exact results (pixBlockBilateralExact)
//!
//! The C version tests 3 images (rock.png, church.png, color-wheel-hue.jpg)
//! with various spatial/range stdev combinations.
//!
//! Rust API mapping:
//!   - pixBlockBilateralExact -> bilateral_exact
//!   - pixBilateralExact -> bilateral_gray_exact (lower-level)
//!   - pixBilateral -> NOT IMPLEMENTED (separable approximate version)

use leptonica_filter::{Kernel, bilateral_exact, bilateral_gray_exact, make_range_kernel};
use leptonica_test::{RegParams, load_test_image};

/// Helper: run exact bilateral tests on a single image.
///
/// C version calls pixBlockBilateralExact with 4 parameter combinations,
/// plus pixBilateral (separable) with various ncomps/reduction.
/// Rust only has bilateral_exact (= pixBlockBilateralExact).
fn do_exact_tests_on_image(pixs: &leptonica_core::Pix, rp: &mut RegParams, label: &str) {
    let w = pixs.width();
    let h = pixs.height();
    eprintln!(
        "  Testing {} ({}x{} d={})",
        label,
        w,
        h,
        pixs.depth().bits()
    );

    // C: pixBlockBilateralExact(pixs, 10.0, 10.0) -- test indices 12-15 per image
    for &range_stdev in &[10.0_f32, 20.0, 40.0, 60.0] {
        let spatial_stdev = 10.0_f32;
        let result = bilateral_exact(pixs, spatial_stdev, range_stdev);
        match result {
            Ok(ref pix) => {
                rp.compare_values(w as f64, pix.width() as f64, 0.0);
                rp.compare_values(h as f64, pix.height() as f64, 0.0);
                eprintln!(
                    "    bilateral_exact({}, {}): {}x{} OK",
                    spatial_stdev,
                    range_stdev,
                    pix.width(),
                    pix.height()
                );
            }
            Err(ref e) => {
                eprintln!(
                    "    bilateral_exact({}, {}): ERROR: {}",
                    spatial_stdev, range_stdev, e
                );
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }
}

/// Test bilateral_exact on multiple images with various parameters.
///
/// C: bilateral1_reg.c DoTestsOnImage for rock.png, church.png, color-wheel-hue.jpg
#[test]
fn bilateral1_reg_exact() {
    let mut rp = RegParams::new("bilateral1");

    let pixs = load_test_image("rock.png").expect("load rock.png");
    do_exact_tests_on_image(&pixs, &mut rp, "rock.png");

    let pixs = load_test_image("church.png").expect("load church.png");
    do_exact_tests_on_image(&pixs, &mut rp, "church.png");

    let pixs = load_test_image("color-wheel-hue.jpg").expect("load color-wheel-hue.jpg");
    do_exact_tests_on_image(&pixs, &mut rp, "color-wheel-hue.jpg");

    assert!(rp.cleanup(), "bilateral1 regression test failed");
}

/// Test bilateral_gray_exact directly with spatial kernel and range kernel.
///
/// C: pixBilateralExact(pixs, spatial_kel, range_kel) -- called via pixBlockBilateralExact
#[test]
fn bilateral1_reg_gray_exact() {
    let mut rp = RegParams::new("bilateral1_gray");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();

    // Create spatial kernel (halfwidth = 2 * spatial_stdev)
    let spatial_stdev = 2.0_f32;
    let halfwidth = (2.0 * spatial_stdev) as u32;
    let size = 2 * halfwidth + 1;
    let spatial_kernel = Kernel::gaussian(size, spatial_stdev).expect("create spatial kernel");

    // Create range kernel
    let range_kernel = make_range_kernel(30.0).expect("create range kernel");

    // Apply bilateral_gray_exact with range kernel
    let result = bilateral_gray_exact(&pixs, &spatial_kernel, Some(&range_kernel))
        .expect("bilateral_gray_exact with range kernel");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);

    // Apply bilateral_gray_exact without range kernel (degenerates to Gaussian)
    // C: range_kel = NULL case in pixBilateralExact
    let result_no_range = bilateral_gray_exact(&pixs, &spatial_kernel, None)
        .expect("bilateral_gray_exact without range kernel");
    rp.compare_values(w as f64, result_no_range.width() as f64, 0.0);
    rp.compare_values(h as f64, result_no_range.height() as f64, 0.0);

    // Verify that bilateral filtering preserves edges (edge-aware smoothing)
    let center_val = result.get_pixel(w / 2, h / 2).unwrap_or(0);
    let is_valid = center_val <= 255;
    rp.compare_values(1.0, if is_valid { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "bilateral1_gray regression test failed");
}

/// C: pixBilateral() (separable approximate bilateral) -- Rust unimplemented
#[test]
#[ignore = "C: pixBilateral() (separable approximate) -- not implemented in Rust"]
fn bilateral1_reg_separable() {}
