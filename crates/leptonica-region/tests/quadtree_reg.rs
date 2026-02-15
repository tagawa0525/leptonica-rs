//! Quadtree regression test
//!
//! C reference: reference/leptonica/prog/quadtree_reg.c
//!
//! Verifies:
//! 1. quadtree_regions generates correct box hierarchies (C regTest 0-1)
//! 2. quadtree_mean computes level statistics (C regTest 2)
//! 3. quadtree_variance computes variance and root variance (C regTest 3-4)
//! 4. Parent/child hierarchy navigation (C regTest 7-8)
//! 5. Integral image arithmetic correctness
//! 6. Precomputed integral results match automatic computation
//! 7. Max levels computation for various image sizes
//! 8. Error handling for invalid inputs

use leptonica_core::{Pix, PixelDepth};
use leptonica_region::{
    IntegralImage, SquaredIntegralImage, mean_in_rectangle, quadtree_max_levels, quadtree_mean,
    quadtree_mean_with_integral, quadtree_regions, quadtree_variance,
    quadtree_variance_with_integral, variance_in_rectangle,
};
use leptonica_test::RegParams;

/// Create a synthetic 8-bit grayscale image for testing.
fn create_test_grayscale_image(width: u32, height: u32) -> Pix {
    let pix = Pix::new(width, height, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    for y in 0..height {
        for x in 0..width {
            let fx = x as f32 / width as f32;
            let fy = y as f32 / height as f32;
            let val = (128.0
                + 60.0 * (fx * std::f32::consts::TAU).sin()
                + 40.0 * (fy * 2.0 * std::f32::consts::TAU).cos()
                + 20.0 * ((fx + fy) * 9.42).sin()) as u32;
            let _ = pix_mut.set_pixel(x, y, val.min(255));
        }
    }

    pix_mut.into()
}

#[test]

fn quadtree_regions_generation() {
    let mut rp = RegParams::new("quadtree_regions");

    // --- C regTest 0: boxaaQuadtreeRegions(1000, 500, 3) ---
    eprintln!("=== quadtree_regions(1000, 500, 3) ===");
    let baa = quadtree_regions(1000, 500, 3).expect("quadtree_regions(1000,500,3)");

    let boxa0 = baa.get(0).expect("level 0");
    rp.compare_values(1.0, boxa0.len() as f64, 0.0);
    let b0 = boxa0.get(0).unwrap();
    rp.compare_values(0.0, b0.x as f64, 0.0);
    rp.compare_values(0.0, b0.y as f64, 0.0);
    eprintln!("  Level 0: box ({},{},{},{})", b0.x, b0.y, b0.w, b0.h);
    rp.compare_values(1.0, if b0.w > 0 && b0.h > 0 { 1.0 } else { 0.0 }, 0.0);

    let boxa1 = baa.get(1).expect("level 1");
    rp.compare_values(4.0, boxa1.len() as f64, 0.0);

    let boxa2 = baa.get(2).expect("level 2");
    rp.compare_values(16.0, boxa2.len() as f64, 0.0);

    for i in 0..boxa2.len() {
        let b = boxa2.get(i).unwrap();
        rp.compare_values(1.0, if b.w > 0 && b.h > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    // --- C regTest 1: boxaaQuadtreeRegions(1001, 501, 3) ---
    eprintln!("=== quadtree_regions(1001, 501, 3) ===");
    let baa2 = quadtree_regions(1001, 501, 3).expect("quadtree_regions(1001,501,3)");

    rp.compare_values(3.0, baa2.len() as f64, 0.0);

    let boxa0_2 = baa2.get(0).expect("level 0");
    rp.compare_values(1.0, boxa0_2.len() as f64, 0.0);

    let boxa1_2 = baa2.get(1).expect("level 1");
    rp.compare_values(4.0, boxa1_2.len() as f64, 0.0);

    let boxa2_2 = baa2.get(2).expect("level 2");
    rp.compare_values(16.0, boxa2_2.len() as f64, 0.0);

    for i in 0..boxa2_2.len() {
        let b = boxa2_2.get(i).unwrap();
        rp.compare_values(1.0, if b.w > 0 && b.h > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "quadtree regions test failed");
}

#[test]

fn quadtree_mean_variance() {
    let mut rp = RegParams::new("quadtree_stats");

    let width = 640u32;
    let height = 832u32;
    let pixg = create_test_grayscale_image(width, height);

    let max_levels = quadtree_max_levels(width, height);
    eprintln!("  Max levels for {}x{}: {}", width, height, max_levels);
    rp.compare_values(1.0, if max_levels >= 8 { 1.0 } else { 0.0 }, 0.0);

    // --- C regTest 2: pixQuadtreeMean ---
    eprintln!("=== quadtree_mean with 8 levels ===");
    let nlevels = 8u32;
    let mean_result = quadtree_mean(&pixg, nlevels).expect("quadtree_mean");
    rp.compare_values(nlevels as f64, mean_result.num_levels() as f64, 0.0);

    for level in 0..nlevels {
        let expected_size = 1u32 << level;
        let fpix = mean_result.get_level(level as usize).unwrap();
        rp.compare_values(expected_size as f64, fpix.width() as f64, 0.0);
        rp.compare_values(expected_size as f64, fpix.height() as f64, 0.0);
    }

    let level0_mean = mean_result.get_value(0, 0, 0).unwrap();
    eprintln!("  Level 0 mean (whole image): {:.2}", level0_mean);
    rp.compare_values(
        1.0,
        if level0_mean > 50.0 && level0_mean < 220.0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // --- C regTest 3-4: pixQuadtreeVariance ---
    eprintln!("=== quadtree_variance with 8 levels ===");
    let (var_result, rvar_result) = quadtree_variance(&pixg, nlevels).expect("quadtree_variance");
    rp.compare_values(nlevels as f64, var_result.num_levels() as f64, 0.0);
    rp.compare_values(nlevels as f64, rvar_result.num_levels() as f64, 0.0);

    let level0_var = var_result.get_value(0, 0, 0).unwrap();
    let level0_rvar = rvar_result.get_value(0, 0, 0).unwrap();
    eprintln!(
        "  Level 0 variance: {:.2}, root_variance: {:.2}",
        level0_var, level0_rvar
    );
    rp.compare_values(1.0, if level0_var >= 0.0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if level0_rvar >= 0.0 { 1.0 } else { 0.0 }, 0.0);

    let expected_rvar = level0_var.sqrt();
    rp.compare_values(expected_rvar as f64, level0_rvar as f64, 1.0);

    assert!(rp.cleanup(), "quadtree stats test failed");
}

#[test]

fn quadtree_parent_child_access() {
    let mut rp = RegParams::new("quadtree_hierarchy");

    let width = 256u32;
    let height = 256u32;
    let pixg = create_test_grayscale_image(width, height);

    let nlevels = 8u32;
    let mean_result = quadtree_mean(&pixg, nlevels).expect("quadtree_mean");

    // --- C regTest 7: Parent access verification ---
    eprintln!("=== Parent access verification ===");
    let check_level = 4usize;
    let parent_level = check_level - 1;
    let check_size = 1u32 << check_level;
    let mut parent_error = false;

    for i in (0..check_size).step_by(2) {
        for j in (0..check_size).step_by(2) {
            let parent_val = mean_result.get_parent(check_level, j, i);
            let direct_val = mean_result.get_value(parent_level, j / 2, i / 2);

            match (parent_val, direct_val) {
                (Some(p), Some(d)) => {
                    if (p - d).abs() > 0.001 {
                        parent_error = true;
                    }
                }
                _ => {
                    parent_error = true;
                }
            }
        }
    }
    rp.compare_values(0.0, if parent_error { 1.0 } else { 0.0 }, 0.0);

    // --- C regTest 8: Children access verification ---
    eprintln!("=== Children access verification ===");
    let child_level = check_level + 1;
    let mut child_error = false;

    for i in 0..check_size {
        for j in 0..check_size {
            let children = mean_result.get_children(check_level, j, i);
            match children {
                Some([val00, val10, val01, val11]) => {
                    let child_x = j * 2;
                    let child_y = i * 2;

                    let valc00 = mean_result.get_value(child_level, child_x, child_y);
                    let valc10 = mean_result.get_value(child_level, child_x + 1, child_y);
                    let valc01 = mean_result.get_value(child_level, child_x, child_y + 1);
                    let valc11 = mean_result.get_value(child_level, child_x + 1, child_y + 1);

                    match (valc00, valc10, valc01, valc11) {
                        (Some(c00), Some(c10), Some(c01), Some(c11)) => {
                            if (val00 - c00).abs() > 0.001
                                || (val10 - c10).abs() > 0.001
                                || (val01 - c01).abs() > 0.001
                                || (val11 - c11).abs() > 0.001
                            {
                                child_error = true;
                            }
                        }
                        _ => {
                            child_error = true;
                        }
                    }
                }
                None => {
                    child_error = true;
                }
            }
        }
    }
    rp.compare_values(0.0, if child_error { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "quadtree hierarchy test failed");
}

#[test]

fn quadtree_integral_image() {
    let mut rp = RegParams::new("quadtree_integral");

    // --- Uniform image ---
    eprintln!("=== Integral image: uniform ===");
    let pix_uniform = {
        let pix = Pix::new(16, 16, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..16u32 {
            for x in 0..16u32 {
                let _ = pix_mut.set_pixel(x, y, 100);
            }
        }
        let p: Pix = pix_mut.into();
        p
    };

    let integral = IntegralImage::from_pix(&pix_uniform).expect("IntegralImage::from_pix");
    rp.compare_values(16.0, integral.width() as f64, 0.0);
    rp.compare_values(16.0, integral.height() as f64, 0.0);

    let total_sum = integral.sum_rect(0, 0, 16, 16).expect("total sum");
    rp.compare_values(25600.0, total_sum as f64, 0.0);

    let sub_sum = integral.sum_rect(4, 4, 4, 4).expect("sub sum");
    rp.compare_values(1600.0, sub_sum as f64, 0.0);

    let rect = leptonica_core::Box::new_unchecked(0, 0, 16, 16);
    let mean = mean_in_rectangle(&rect, &integral).expect("mean_in_rectangle");
    rp.compare_values(100.0, mean as f64, 0.001);

    // --- Known values ---
    eprintln!("=== Integral image: known values ===");
    let pix_known = {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..4u32 {
            for x in 0..4u32 {
                let _ = pix_mut.set_pixel(x, y, y * 4 + x + 1);
            }
        }
        let p: Pix = pix_mut.into();
        p
    };

    let integral2 = IntegralImage::from_pix(&pix_known).expect("IntegralImage for known");

    rp.compare_values(1.0, integral2.get(0, 0).unwrap() as f64, 0.0);
    rp.compare_values(10.0, integral2.get(3, 0).unwrap() as f64, 0.0);
    rp.compare_values(28.0, integral2.get(0, 3).unwrap() as f64, 0.0);

    let total = integral2.get(3, 3).unwrap();
    rp.compare_values(136.0, total as f64, 0.0);

    let corner_sum = integral2.sum_rect(2, 2, 2, 2).expect("corner sum");
    rp.compare_values(54.0, corner_sum as f64, 0.0);

    // --- Squared integral / variance ---
    eprintln!("=== Squared integral image ===");
    let sq_integral = SquaredIntegralImage::from_pix(&pix_uniform).expect("SquaredIntegralImage");

    let rect_all = leptonica_core::Box::new_unchecked(0, 0, 16, 16);
    let (var, rvar) = variance_in_rectangle(&rect_all, &integral, &sq_integral).expect("variance");
    rp.compare_values(0.0, var as f64, 0.001);
    rp.compare_values(0.0, rvar as f64, 0.001);

    // --- Non-uniform variance ---
    eprintln!("=== Variance: non-uniform ===");
    let pix_nonuniform = {
        let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        let _ = pix_mut.set_pixel(0, 0, 0);
        let _ = pix_mut.set_pixel(1, 0, 0);
        let _ = pix_mut.set_pixel(2, 0, 10);
        let _ = pix_mut.set_pixel(3, 0, 10);
        let p: Pix = pix_mut.into();
        p
    };
    let int_nu = IntegralImage::from_pix(&pix_nonuniform).expect("IntegralImage nonuniform");
    let sq_nu =
        SquaredIntegralImage::from_pix(&pix_nonuniform).expect("SqIntegralImage nonuniform");
    let rect_nu = leptonica_core::Box::new_unchecked(0, 0, 4, 1);
    let (var_nu, rvar_nu) =
        variance_in_rectangle(&rect_nu, &int_nu, &sq_nu).expect("variance nonuniform");
    rp.compare_values(25.0, var_nu as f64, 0.1);
    rp.compare_values(5.0, rvar_nu as f64, 0.1);

    assert!(rp.cleanup(), "quadtree integral image test failed");
}

#[test]

fn quadtree_with_integral_precomputed() {
    let mut rp = RegParams::new("quadtree_precomputed");

    let width = 128u32;
    let height = 128u32;
    let pixg = create_test_grayscale_image(width, height);

    let nlevels = 5u32;

    let mean_auto = quadtree_mean(&pixg, nlevels).expect("quadtree_mean auto");
    let (var_auto, rvar_auto) = quadtree_variance(&pixg, nlevels).expect("quadtree_variance auto");

    let integral = IntegralImage::from_pix(&pixg).expect("IntegralImage");
    let sq_integral = SquaredIntegralImage::from_pix(&pixg).expect("SqIntegralImage");

    let mean_pre =
        quadtree_mean_with_integral(&pixg, nlevels, &integral).expect("quadtree_mean pre");
    let (var_pre, rvar_pre) =
        quadtree_variance_with_integral(&pixg, nlevels, &integral, &sq_integral)
            .expect("quadtree_variance pre");

    for level in 0..nlevels as usize {
        let size = 1u32 << level;
        for y in 0..size {
            for x in 0..size {
                let m_auto = mean_auto.get_value(level, x, y).unwrap();
                let m_pre = mean_pre.get_value(level, x, y).unwrap();
                if (m_auto - m_pre).abs() > 0.001 {
                    rp.compare_values(m_auto as f64, m_pre as f64, 0.001);
                }

                let v_auto = var_auto.get_value(level, x, y).unwrap();
                let v_pre = var_pre.get_value(level, x, y).unwrap();
                if (v_auto - v_pre).abs() > 0.001 {
                    rp.compare_values(v_auto as f64, v_pre as f64, 0.001);
                }

                let rv_auto = rvar_auto.get_value(level, x, y).unwrap();
                let rv_pre = rvar_pre.get_value(level, x, y).unwrap();
                if (rv_auto - rv_pre).abs() > 0.001 {
                    rp.compare_values(rv_auto as f64, rv_pre as f64, 0.001);
                }
            }
        }
    }

    rp.compare_values(1.0, 1.0, 0.0);

    assert!(rp.cleanup(), "quadtree precomputed test failed");
}

#[test]

fn quadtree_max_levels_various() {
    let mut rp = RegParams::new("quadtree_maxlevels");

    rp.compare_values(0.0, quadtree_max_levels(1, 1) as f64, 0.0);
    rp.compare_values(0.0, quadtree_max_levels(2, 2) as f64, 0.0);
    rp.compare_values(1.0, quadtree_max_levels(4, 4) as f64, 0.0);
    rp.compare_values(2.0, quadtree_max_levels(8, 8) as f64, 0.0);
    rp.compare_values(3.0, quadtree_max_levels(16, 16) as f64, 0.0);

    rp.compare_values(1.0, quadtree_max_levels(16, 4) as f64, 0.0);
    rp.compare_values(1.0, quadtree_max_levels(4, 16) as f64, 0.0);

    rp.compare_values(0.0, quadtree_max_levels(0, 10) as f64, 0.0);
    rp.compare_values(0.0, quadtree_max_levels(10, 0) as f64, 0.0);

    let max_632 = quadtree_max_levels(632, 825);
    eprintln!("  max_levels for 632x825: {}", max_632);
    rp.compare_values(1.0, if max_632 >= 8 { 1.0 } else { 0.0 }, 0.0);

    let max_1000x500 = quadtree_max_levels(1000, 500);
    eprintln!("  max_levels for 1000x500: {}", max_1000x500);
    rp.compare_values(1.0, if max_1000x500 >= 3 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "quadtree max levels test failed");
}

#[test]

fn quadtree_error_cases() {
    let mut rp = RegParams::new("quadtree_errors");

    let result = quadtree_regions(100, 100, 0);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    let result = quadtree_regions(4, 4, 10);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    let pix1 = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
    let result = quadtree_mean(&pix1, 2);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    let result = quadtree_variance(&pix1, 2);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    let result = IntegralImage::from_pix(&pix1);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    let result = SquaredIntegralImage::from_pix(&pix1);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "quadtree error cases test failed");
}
