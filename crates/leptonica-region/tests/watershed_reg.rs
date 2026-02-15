//! Watershed segmentation regression test
//!
//! C reference: reference/leptonica/prog/watershed_reg.c
//!
//! Verifies:
//! 1. Local minima and maxima detection in synthetic images
//! 2. Watershed segmentation produces labeled regions with boundaries
//! 3. Gradient computation produces meaningful edge responses
//! 4. Error handling for invalid input depths

use leptonica_core::{Pix, PixelDepth};
use leptonica_region::{
    ConnectivityType, WatershedOptions, compute_gradient, find_local_maxima, find_local_minima,
    watershed_segmentation,
};
use leptonica_test::RegParams;

/// Create the synthetic test image used in the C version.
fn create_synthetic_image(variant: u32) -> Pix {
    let size = 500u32;
    let pix = Pix::new(size, size, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    for i in 0..size {
        for j in 0..size {
            let fi = i as f32;
            let fj = j as f32;
            let f = if variant == 0 {
                128.0
                    + 26.3 * (0.0438 * fi).sin()
                    + 33.4 * (0.0712 * fi).cos()
                    + 18.6 * (0.0561 * fj).sin()
                    + 23.6 * (0.0327 * fj).cos()
            } else {
                128.0
                    + 26.3 * (0.0238 * fi).sin()
                    + 33.4 * (0.0312 * fi).cos()
                    + 18.6 * (0.0261 * fj).sin()
                    + 23.6 * (0.0207 * fj).cos()
            };
            let _ = pix_mut.set_pixel(j, i, f as u32);
        }
    }

    pix_mut.into()
}

/// Core watershed test, corresponding to DoWatershed() in C version.
fn do_watershed(rp: &mut RegParams, pixs: &Pix) {
    let w = pixs.width();
    let h = pixs.height();

    rp.compare_values(500.0, w as f64, 0.0);
    rp.compare_values(500.0, h as f64, 0.0);

    let minima =
        find_local_minima(pixs, ConnectivityType::EightWay).expect("find_local_minima failed");
    let maxima =
        find_local_maxima(pixs, ConnectivityType::EightWay).expect("find_local_maxima failed");

    eprintln!(
        "  Local minima: {}, Local maxima: {}",
        minima.len(),
        maxima.len()
    );

    rp.compare_values(1.0, if !minima.is_empty() { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if !maxima.is_empty() { 1.0 } else { 0.0 }, 0.0);

    let seed_count = minima.len();
    rp.compare_values(1.0, if seed_count > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(seed_count as f64, minima.len() as f64, 0.0);

    let options = WatershedOptions::new()
        .with_min_depth(10)
        .with_connectivity(ConnectivityType::EightWay);
    let result = watershed_segmentation(pixs, &options);

    match result {
        Ok(segmented) => {
            rp.compare_values(w as f64, segmented.width() as f64, 0.0);
            rp.compare_values(h as f64, segmented.height() as f64, 0.0);
            rp.compare_values(32.0, segmented.depth().bits() as f64, 0.0);

            let mut labels = std::collections::HashSet::new();
            for y in 0..h {
                for x in 0..w {
                    if let Some(label) = segmented.get_pixel(x, y)
                        && label > 0
                    {
                        labels.insert(label);
                    }
                }
            }
            let num_basins = labels.len();
            eprintln!("  Number of basins: {}", num_basins);
            rp.compare_values(1.0, if num_basins > 1 { 1.0 } else { 0.0 }, 0.0);

            let mut boundary_count = 0u64;
            for y in 0..h {
                for x in 0..w {
                    if let Some(label) = segmented.get_pixel(x, y)
                        && label == 0
                    {
                        boundary_count += 1;
                    }
                }
            }
            eprintln!("  Boundary pixels: {}", boundary_count);

            let total_pixels = (w as u64) * (h as u64);
            rp.compare_values(
                1.0,
                if boundary_count > 0 && boundary_count < total_pixels {
                    1.0
                } else {
                    0.0
                },
                0.0,
            );
        }
        Err(e) => {
            eprintln!("  watershed_segmentation failed: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }
}

#[test]
fn watershed_segmentation_synthetic() {
    let mut rp = RegParams::new("watershed");

    eprintln!("=== Synthetic image 1 ===");
    let pix1 = create_synthetic_image(0);
    do_watershed(&mut rp, &pix1);

    eprintln!("=== Synthetic image 2 ===");
    let pix2 = create_synthetic_image(1);
    do_watershed(&mut rp, &pix2);

    assert!(rp.cleanup(), "watershed regression test failed");
}

#[test]
fn watershed_local_extrema_basic() {
    let mut rp = RegParams::new("watershed_extrema");

    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    for y in 0..10u32 {
        for x in 0..10u32 {
            let _ = pix_mut.set_pixel(x, y, 128);
        }
    }

    let _ = pix_mut.set_pixel(3, 3, 10);
    let _ = pix_mut.set_pixel(7, 7, 250);

    let pix: Pix = pix_mut.into();

    let minima = find_local_minima(&pix, ConnectivityType::EightWay).expect("find minima");
    eprintln!("Minima found: {} (expected >= 1)", minima.len());
    rp.compare_values(1.0, if !minima.is_empty() { 1.0 } else { 0.0 }, 0.0);

    let has_valley = minima.iter().any(|&(x, y)| x == 3 && y == 3);
    rp.compare_values(1.0, if has_valley { 1.0 } else { 0.0 }, 0.0);

    let maxima = find_local_maxima(&pix, ConnectivityType::EightWay).expect("find maxima");
    eprintln!("Maxima found: {} (expected >= 1)", maxima.len());
    rp.compare_values(1.0, if !maxima.is_empty() { 1.0 } else { 0.0 }, 0.0);

    let has_hill = maxima.iter().any(|&(x, y)| x == 7 && y == 7);
    rp.compare_values(1.0, if has_hill { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "watershed extrema test failed");
}

#[test]
fn watershed_gradient() {
    let mut rp = RegParams::new("watershed_gradient");

    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    for y in 0..20u32 {
        for x in 0..20u32 {
            let val = if x < 10 { 50u32 } else { 200u32 };
            let _ = pix_mut.set_pixel(x, y, val);
        }
    }
    let pix: Pix = pix_mut.into();

    let gradient = compute_gradient(&pix).expect("compute gradient");

    rp.compare_values(20.0, gradient.width() as f64, 0.0);
    rp.compare_values(20.0, gradient.height() as f64, 0.0);
    rp.compare_values(8.0, gradient.depth().bits() as f64, 0.0);

    let grad_edge = gradient.get_pixel(9, 10).unwrap_or(0);
    let grad_flat = gradient.get_pixel(5, 10).unwrap_or(0);
    eprintln!("  Gradient at edge: {}, at flat: {}", grad_edge, grad_flat);
    rp.compare_values(1.0, if grad_edge > grad_flat { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "watershed gradient test failed");
}

#[test]
fn watershed_error_handling() {
    let mut rp = RegParams::new("watershed_errors");

    let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let options = WatershedOptions::default();
    let result = watershed_segmentation(&pix1, &options);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    let pix32 = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let result = watershed_segmentation(&pix32, &options);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    let result = find_local_minima(&pix1, ConnectivityType::EightWay);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    let result = find_local_maxima(&pix1, ConnectivityType::EightWay);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    let result = compute_gradient(&pix1);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "watershed error handling test failed");
}
