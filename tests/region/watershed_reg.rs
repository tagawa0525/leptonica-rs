//! Watershed segmentation regression test
//!
//! C reference: prog/watershed_reg.c
//!
//! Verifies:
//! 1. Local minima and maxima detection in synthetic images
//! 2. Watershed segmentation produces labeled regions with boundaries
//! 3. Gradient computation produces meaningful edge responses
//! 4. Error handling for invalid input depths

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::region::{
    ConnectivityType, WatershedOptions, compute_gradient, find_local_maxima, find_local_minima,
    watershed_render_colors, watershed_render_fill, watershed_segmentation, watershed_with_basins,
};
use leptonica::{Pix, PixelDepth};

/// Create the synthetic test image used in the C version.
///
/// C accumulates into an `l_float32 f` while every term is evaluated in double
/// precision (the literals are doubles and `sin`/`cos` return doubles), so each
/// `+=` rounds the running sum back to f32.  Evaluating the whole expression in
/// f32 gives a visibly different image, so mirror C's per-term rounding.
fn create_synthetic_image(variant: u32) -> Pix {
    let size = 500u32;
    let pix = Pix::new(size, size, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    let (ca, cb, cc, cd) = if variant == 0 {
        (0.0438f64, 0.0712f64, 0.0561f64, 0.0327f64)
    } else {
        (0.0238f64, 0.0312f64, 0.0261f64, 0.0207f64)
    };

    for i in 0..size {
        for j in 0..size {
            let fi = i as f64;
            let fj = j as f64;
            let mut f = (128.0 + 26.3 * (ca * fi).sin()) as f32;
            f = (f as f64 + 33.4 * (cb * fi).cos()) as f32;
            f = (f as f64 + 18.6 * (cc * fj).sin()) as f32;
            f = (f as f64 + 23.6 * (cd * fj).cos()) as f32;
            // C: pixSetPixel(pix, j, i, (l_int32)f) - truncation toward zero.
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
            rp.write_pix_and_check(&segmented, ImageFormat::Png)
                .expect("write segmented watershed");

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
    rp.write_pix_and_check(&gradient, ImageFormat::Png)
        .expect("write gradient watershed_gradient");

    // C checks 9-10: render_fill and render_colors on small image
    let options = WatershedOptions::new()
        .with_min_depth(5)
        .with_connectivity(ConnectivityType::EightWay);
    let ws_result =
        watershed_with_basins(&pix, &options).expect("watershed_with_basins on small image");
    let filled = watershed_render_fill(&ws_result).expect("watershed_render_fill on small image");
    rp.compare_values(20.0, filled.width() as f64, 0.0);
    rp.compare_values(20.0, filled.height() as f64, 0.0);
    rp.compare_values(8.0, filled.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&filled, ImageFormat::Png)
        .expect("check: watershed render_fill");
    let colored =
        watershed_render_colors(&ws_result).expect("watershed_render_colors on small image");
    rp.compare_values(20.0, colored.width() as f64, 0.0);
    rp.compare_values(20.0, colored.height() as f64, 0.0);
    rp.compare_values(32.0, colored.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&colored, ImageFormat::Png)
        .expect("check: watershed render_colors");

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

/// C-compatible port of `DoWatershed()` in `prog/watershed_reg.c`, restricted
/// to the local-extrema/seed stage (C indices 0-6 and 12-18).
///
/// The watershed stage proper (C indices 7-11, 19-23) needs the `L_WSHED`
/// priority-queue machinery, which is not ported yet.
fn do_watershed_c(rp: &mut crate::common::RegParams, pixs: &Pix) {
    use leptonica::core::pix::InitColor;
    use leptonica::core::pixel::compose_rgba;
    use leptonica::region::{local_extrema, remove_seeded_components, select_min_in_conncomp};

    let w = pixs.width();
    let h = pixs.height();

    // 0
    rp.write_pix_and_check(pixs, ImageFormat::Png)
        .expect("write pixs");

    let (pix1, pix2) = local_extrema(pixs, 0, 0).expect("local_extrema");
    let pix1 = {
        let mut m = pix1.try_into_mut().expect("into mut");
        m.set_or_clear_border(2, 2, 2, 2, InitColor::White);
        Pix::from(m)
    };

    // C `composeRGBPixel` leaves the alpha byte at 0, and `convert_to_32`
    // already matches that (see `pixConvert8To32`). Using `compose_rgb`,
    // which forces alpha = 255, would paint a different alpha than the
    // surrounding pixels and diverge from C's in-memory 32bpp image.
    let redval = compose_rgba(255, 0, 0, 0);
    let greenval = compose_rgba(0, 255, 0, 0);

    let pixc = pixs.convert_to_32().expect("convert_to_32");
    let pixc = {
        let mut m = pixc.try_into_mut().expect("into mut");
        m.paint_through_mask(&pix2, 0, 0, greenval)
            .expect("paint maxima");
        m.paint_through_mask(&pix1, 0, 0, redval)
            .expect("paint minima");
        Pix::from(m)
    };
    // 1
    rp.write_pix_and_check(&pixc, ImageFormat::Png)
        .expect("write pixc");
    // 2
    rp.write_pix_and_check(&pix1, ImageFormat::Png)
        .expect("write minima");

    let (pta, _) = select_min_in_conncomp(pixs, &pix1).expect("select_min_in_conncomp");
    let pix3 =
        leptonica::core::pta::pix_generate_from_pta(&pta, w, h).expect("pix_generate_from_pta");
    // 3
    rp.write_pix_and_check(&pix3, ImageFormat::Png)
        .expect("write seeds");

    let pix4 = pixs.convert_to_32().expect("convert_to_32");
    let pix4 = {
        let mut m = pix4.try_into_mut().expect("into mut");
        m.paint_through_mask(&pix3, 0, 0, greenval)
            .expect("paint seeds");
        Pix::from(m)
    };
    // 4
    rp.write_pix_and_check(&pix4, ImageFormat::Png)
        .expect("write painted seeds");

    let pix5 = remove_seeded_components(&pix3, &pix1, ConnectivityType::EightWay)
        .expect("remove_seeded_components");
    // 5
    rp.write_pix_and_check(&pix5, ImageFormat::Png)
        .expect("write leftovers");
    // 6
    let empty = if pix5.is_zero() { 1.0 } else { 0.0 };
    rp.compare_values(1.0, empty, 0.0);
}

#[test]
fn watershed_c_compat() {
    let mut rp = crate::common::RegParams::new("watershed_c");

    let pix1 = create_synthetic_image(0);
    do_watershed_c(&mut rp, &pix1);
    let pix2 = create_synthetic_image(1);
    do_watershed_c(&mut rp, &pix2);

    assert!(rp.cleanup(), "watershed c-compat test failed");
}
