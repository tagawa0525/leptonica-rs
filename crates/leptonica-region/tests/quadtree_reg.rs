//! Quadtree regression test
//!
//! C版: reference/leptonica/prog/quadtree_reg.c
//!
//! C版テストの概要:
//! 1. boxaaQuadtreeRegions() で四分木領域を生成 (regTest 0-1)
//! 2. rabi.pngを読み込み、pixScaleToGray4()で8bitグレースケール化
//! 3. pixQuadtreeMean() / pixQuadtreeVariance() で統計量を計算 (regTest 2-4)
//! 4. pixGetAverageTiled() / pixExpandReplicate() で比較 (regTest 5-6)
//! 5. quadtreeGetParent() / quadtreeGetChildren() で階層アクセス検証 (regTest 7-8)
//!
//! Rust側の対応:
//! - quadtree_regions() -> boxaaQuadtreeRegions() に対応
//! - quadtree_mean() -> pixQuadtreeMean() に対応
//! - quadtree_variance() -> pixQuadtreeVariance() に対応
//! - QuadtreeResult::get_parent/get_children -> quadtreeGetParent/Children に対応
//! - IntegralImage / SquaredIntegralImage -> 内部実装で使用
//!
//! Run with:
//! ```
//! cargo test -p leptonica-region --test quadtree_reg
//! ```

use leptonica_core::{Pix, PixelDepth};
use leptonica_region::{
    IntegralImage, SquaredIntegralImage, mean_in_rectangle, quadtree_max_levels, quadtree_mean,
    quadtree_mean_with_integral, quadtree_regions, quadtree_variance,
    quadtree_variance_with_integral, variance_in_rectangle,
};
use leptonica_test::RegParams;

/// Create a synthetic 8-bit grayscale image for testing.
/// The C version uses pixScaleToGray4(rabi.png), but since Rust side does not have
/// scale/convert functions in leptonica-region's dependencies, we create a
/// synthetic gradient image with known properties.
fn create_test_grayscale_image(width: u32, height: u32) -> Pix {
    let pix = Pix::new(width, height, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    for y in 0..height {
        for x in 0..width {
            // Create a pattern with spatial variation for meaningful quadtree stats
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
    // C版: regTest 0-1 -- boxaaQuadtreeRegions()
    let mut rp = RegParams::new("quadtree_regions");

    // --- Test 0 (C版 regTest 0): boxaaQuadtreeRegions(1000, 500, 3) ---
    eprintln!("=== quadtree_regions(1000, 500, 3) ===");
    let baa = quadtree_regions(1000, 500, 3).expect("quadtree_regions(1000,500,3)");

    // Level 0: 1 box covering entire image
    let boxa0 = baa.get(0).expect("level 0");
    rp.compare_values(1.0, boxa0.len() as f64, 0.0);
    let b0 = boxa0.get(0).unwrap();
    rp.compare_values(0.0, b0.x as f64, 0.0);
    rp.compare_values(0.0, b0.y as f64, 0.0);
    // Level 0 box should cover the whole image
    eprintln!("  Level 0: box ({},{},{},{})", b0.x, b0.y, b0.w, b0.h);
    rp.compare_values(1.0, if b0.w > 0 && b0.h > 0 { 1.0 } else { 0.0 }, 0.0);

    // Level 1: 4 boxes
    let boxa1 = baa.get(1).expect("level 1");
    rp.compare_values(4.0, boxa1.len() as f64, 0.0);

    // Level 2: 16 boxes
    let boxa2 = baa.get(2).expect("level 2");
    rp.compare_values(16.0, boxa2.len() as f64, 0.0);

    // Verify all boxes at level 2 have positive dimensions
    for i in 0..boxa2.len() {
        let b = boxa2.get(i).unwrap();
        rp.compare_values(1.0, if b.w > 0 && b.h > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    // --- Test 1 (C版 regTest 1): boxaaQuadtreeRegions(1001, 501, 3) ---
    eprintln!("=== quadtree_regions(1001, 501, 3) ===");
    let baa2 = quadtree_regions(1001, 501, 3).expect("quadtree_regions(1001,501,3)");

    // Same structure: 3 levels
    rp.compare_values(3.0, baa2.len() as f64, 0.0);

    // Level 0: 1 box
    let boxa0_2 = baa2.get(0).expect("level 0");
    rp.compare_values(1.0, boxa0_2.len() as f64, 0.0);

    // Level 1: 4 boxes
    let boxa1_2 = baa2.get(1).expect("level 1");
    rp.compare_values(4.0, boxa1_2.len() as f64, 0.0);

    // Level 2: 16 boxes
    let boxa2_2 = baa2.get(2).expect("level 2");
    rp.compare_values(16.0, boxa2_2.len() as f64, 0.0);

    // Verify all level-2 boxes have positive dimensions for odd-sized image
    for i in 0..boxa2_2.len() {
        let b = boxa2_2.get(i).unwrap();
        rp.compare_values(1.0, if b.w > 0 && b.h > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "quadtree regions test failed");
}

#[test]
fn quadtree_mean_variance() {
    // C版: regTest 2-4 -- pixQuadtreeMean() / pixQuadtreeVariance()
    // C版ではrabi.pngをpixScaleToGray4()で変換した画像を使用するが、
    // Rust側ではpixScaleToGray4() (leptonica-transform) が依存関係にないため
    // 合成画像を使用する。
    // C版: pixScaleToGray4() -- leptonica-regionの依存関係にないためスキップ
    let mut rp = RegParams::new("quadtree_stats");

    // Create a test image (approximately same size as C version: ~632x825)
    // C版: rabi.png is 2528x3300, after pixScaleToGray4 -> ~632x825
    let width = 640u32;
    let height = 832u32;
    let pixg = create_test_grayscale_image(width, height);

    eprintln!(
        "Test image: {}x{} depth={}",
        pixg.width(),
        pixg.height(),
        pixg.depth().bits()
    );

    // --- Compute max levels ---
    let max_levels = quadtree_max_levels(width, height);
    eprintln!("  Max levels for {}x{}: {}", width, height, max_levels);
    rp.compare_values(1.0, if max_levels >= 8 { 1.0 } else { 0.0 }, 0.0);

    // --- regTest 2 (C版): pixQuadtreeMean(pixg, 8, NULL, &fpixam) ---
    eprintln!("=== quadtree_mean with 8 levels ===");
    let nlevels = 8u32;
    let mean_result = quadtree_mean(&pixg, nlevels).expect("quadtree_mean");
    rp.compare_values(nlevels as f64, mean_result.num_levels() as f64, 0.0);

    // Check level dimensions
    for level in 0..nlevels {
        let expected_size = 1u32 << level;
        let fpix = mean_result.get_level(level as usize).unwrap();
        rp.compare_values(expected_size as f64, fpix.width() as f64, 0.0);
        rp.compare_values(expected_size as f64, fpix.height() as f64, 0.0);
    }

    // Level 0 mean should be close to overall image mean
    let level0_mean = mean_result.get_value(0, 0, 0).unwrap();
    eprintln!("  Level 0 mean (whole image): {:.2}", level0_mean);
    // For our synthetic image, the mean should be roughly around 128
    rp.compare_values(
        1.0,
        if level0_mean > 50.0 && level0_mean < 220.0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // C版: fpixaDisplayQuadtree(fpixam, 2, 10) -- 可視化関数、Rust未実装のためスキップ

    // --- regTest 3-4 (C版): pixQuadtreeVariance(pixg, 8, NULL, NULL, &fpixav, &fpixarv) ---
    eprintln!("=== quadtree_variance with 8 levels ===");
    let (var_result, rvar_result) = quadtree_variance(&pixg, nlevels).expect("quadtree_variance");
    rp.compare_values(nlevels as f64, var_result.num_levels() as f64, 0.0);
    rp.compare_values(nlevels as f64, rvar_result.num_levels() as f64, 0.0);

    // Variance at level 0 should be non-negative
    let level0_var = var_result.get_value(0, 0, 0).unwrap();
    let level0_rvar = rvar_result.get_value(0, 0, 0).unwrap();
    eprintln!(
        "  Level 0 variance: {:.2}, root_variance: {:.2}",
        level0_var, level0_rvar
    );
    rp.compare_values(1.0, if level0_var >= 0.0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if level0_rvar >= 0.0 { 1.0 } else { 0.0 }, 0.0);

    // Root variance should be sqrt of variance (approximately)
    let expected_rvar = level0_var.sqrt();
    rp.compare_values(expected_rvar as f64, level0_rvar as f64, 1.0);

    // C版: fpixaDisplayQuadtree(fpixav, 2, 10) -- 可視化関数、Rust未実装のためスキップ
    // C版: fpixaDisplayQuadtree(fpixarv, 2, 10) -- 可視化関数、Rust未実装のためスキップ

    assert!(rp.cleanup(), "quadtree stats test failed");
}

#[test]
fn quadtree_parent_child_access() {
    // C版: regTest 7-8 -- quadtreeGetParent() / quadtreeGetChildren()
    let mut rp = RegParams::new("quadtree_hierarchy");

    // Use the same nlevels=8 as C version, but with a smaller image for speed
    let width = 256u32;
    let height = 256u32;
    let pixg = create_test_grayscale_image(width, height);

    let nlevels = 8u32;
    let mean_result = quadtree_mean(&pixg, nlevels).expect("quadtree_mean");

    // --- regTest 7 (C版): Parent access verification ---
    // C版:
    //   fpixaGetFPixDimensions(fpixam, 4, &w, &h);
    //   for (i = 0; i < w; i += 2)
    //     for (j = 0; j < h; j += 2)
    //       quadtreeGetParent(fpixam, 4, j, i, &val1);
    //       fpixaGetPixel(fpixam, 3, j/2, i/2, &val2);
    //       if (val1 != val2) error = TRUE;
    eprintln!("=== Parent access verification (C版 regTest 7) ===");
    let check_level = 4usize;
    let parent_level = check_level - 1;
    let check_size = 1u32 << check_level; // 16
    let mut parent_error = false;

    for i in (0..check_size).step_by(2) {
        for j in (0..check_size).step_by(2) {
            // get_parent returns the value at (x/2, y/2) in level-1
            let parent_val = mean_result.get_parent(check_level, j, i);
            let direct_val = mean_result.get_value(parent_level, j / 2, i / 2);

            match (parent_val, direct_val) {
                (Some(p), Some(d)) => {
                    if (p - d).abs() > 0.001 {
                        parent_error = true;
                        eprintln!(
                            "  Parent mismatch at level {} ({},{}): parent={}, direct={}",
                            check_level, j, i, p, d
                        );
                    }
                }
                _ => {
                    parent_error = true;
                    eprintln!(
                        "  Parent access failed at level {} ({},{}): parent={:?}, direct={:?}",
                        check_level, j, i, parent_val, direct_val
                    );
                }
            }
        }
    }
    rp.compare_values(0.0, if parent_error { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  Parent access: {}",
        if parent_error { "FAILED" } else { "OK" }
    );

    // --- regTest 8 (C版): Children access verification ---
    // C版:
    //   for (i = 0; i < w; i++)
    //     for (j = 0; j < h; j++)
    //       quadtreeGetChildren(fpixam, 4, j, i, &val00, &val10, &val01, &val11);
    //       fpixaGetPixel(fpixam, 5, 2*j, 2*i, &valc00);
    //       fpixaGetPixel(fpixam, 5, 2*j+1, 2*i, &valc10);
    //       fpixaGetPixel(fpixam, 5, 2*j, 2*i+1, &valc01);
    //       fpixaGetPixel(fpixam, 5, 2*j+1, 2*i+1, &valc11);
    eprintln!("=== Children access verification (C版 regTest 8) ===");
    let child_level = check_level + 1; // 5
    let mut child_error = false;

    for i in 0..check_size {
        for j in 0..check_size {
            let children = mean_result.get_children(check_level, j, i);
            match children {
                Some([val00, val10, val01, val11]) => {
                    // Compare with direct access at child_level
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
                                eprintln!(
                                    "  Children mismatch at level {} ({},{})",
                                    check_level, j, i
                                );
                            }
                        }
                        _ => {
                            child_error = true;
                            eprintln!(
                                "  Direct child access failed at level {} ({},{})",
                                check_level, j, i
                            );
                        }
                    }
                }
                None => {
                    child_error = true;
                    eprintln!(
                        "  get_children failed at level {} ({},{})",
                        check_level, j, i
                    );
                }
            }
        }
    }
    rp.compare_values(0.0, if child_error { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  Children access: {}",
        if child_error { "FAILED" } else { "OK" }
    );

    assert!(rp.cleanup(), "quadtree hierarchy test failed");
}

#[test]
fn quadtree_integral_image() {
    // Test IntegralImage and SquaredIntegralImage,
    // which underlie the quadtree computations.
    let mut rp = RegParams::new("quadtree_integral");

    // --- Test with uniform image ---
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

    // Sum of entire image: 16*16*100 = 25600
    let total_sum = integral.sum_rect(0, 0, 16, 16).expect("total sum");
    rp.compare_values(25600.0, total_sum as f64, 0.0);

    // Sum of 4x4 subregion: 4*4*100 = 1600
    let sub_sum = integral.sum_rect(4, 4, 4, 4).expect("sub sum");
    rp.compare_values(1600.0, sub_sum as f64, 0.0);

    // Mean via mean_in_rectangle
    let rect = leptonica_core::Box::new_unchecked(0, 0, 16, 16);
    let mean = mean_in_rectangle(&rect, &integral).expect("mean_in_rectangle");
    rp.compare_values(100.0, mean as f64, 0.001);

    // --- Test with known values ---
    eprintln!("=== Integral image: known values ===");
    let pix_known = {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Row 0: 1, 2, 3, 4
        // Row 1: 5, 6, 7, 8
        // Row 2: 9, 10, 11, 12
        // Row 3: 13, 14, 15, 16
        for y in 0..4u32 {
            for x in 0..4u32 {
                let _ = pix_mut.set_pixel(x, y, y * 4 + x + 1);
            }
        }
        let p: Pix = pix_mut.into();
        p
    };

    let integral2 = IntegralImage::from_pix(&pix_known).expect("IntegralImage for known");

    // Corner value: I(0,0) = 1
    rp.compare_values(1.0, integral2.get(0, 0).unwrap() as f64, 0.0);

    // I(3,0) = 1+2+3+4 = 10
    rp.compare_values(10.0, integral2.get(3, 0).unwrap() as f64, 0.0);

    // I(0,3) = 1+5+9+13 = 28
    rp.compare_values(28.0, integral2.get(0, 3).unwrap() as f64, 0.0);

    // I(3,3) = sum of all = 136
    let total = integral2.get(3, 3).unwrap();
    rp.compare_values(136.0, total as f64, 0.0);

    // Sum of 2x2 bottom-right corner: 11+12+15+16 = 54
    let corner_sum = integral2.sum_rect(2, 2, 2, 2).expect("corner sum");
    rp.compare_values(54.0, corner_sum as f64, 0.0);

    // --- Test SquaredIntegralImage ---
    eprintln!("=== Squared integral image ===");
    let sq_integral = SquaredIntegralImage::from_pix(&pix_uniform).expect("SquaredIntegralImage");

    // Variance of uniform image should be 0
    let rect_all = leptonica_core::Box::new_unchecked(0, 0, 16, 16);
    let (var, rvar) = variance_in_rectangle(&rect_all, &integral, &sq_integral).expect("variance");
    rp.compare_values(0.0, var as f64, 0.001);
    rp.compare_values(0.0, rvar as f64, 0.001);

    // --- Test with non-uniform for variance ---
    eprintln!("=== Variance: non-uniform ===");
    let pix_nonuniform = {
        let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Values: 0, 0, 10, 10 -> mean=5, variance=25
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
    // Mean=5, E[X^2]=(0+0+100+100)/4=50, Var=50-25=25, RVar=5
    rp.compare_values(25.0, var_nu as f64, 0.1);
    rp.compare_values(5.0, rvar_nu as f64, 0.1);

    assert!(rp.cleanup(), "quadtree integral image test failed");
}

#[test]
fn quadtree_with_integral_precomputed() {
    // Test quadtree_mean_with_integral / quadtree_variance_with_integral
    // (using precomputed integral images for efficiency)
    let mut rp = RegParams::new("quadtree_precomputed");

    let width = 128u32;
    let height = 128u32;
    let pixg = create_test_grayscale_image(width, height);

    let nlevels = 5u32;

    // Compute with automatic integral
    let mean_auto = quadtree_mean(&pixg, nlevels).expect("quadtree_mean auto");
    let (var_auto, rvar_auto) = quadtree_variance(&pixg, nlevels).expect("quadtree_variance auto");

    // Compute with explicit integral
    let integral = IntegralImage::from_pix(&pixg).expect("IntegralImage");
    let sq_integral = SquaredIntegralImage::from_pix(&pixg).expect("SqIntegralImage");

    let mean_pre =
        quadtree_mean_with_integral(&pixg, nlevels, &integral).expect("quadtree_mean pre");
    let (var_pre, rvar_pre) =
        quadtree_variance_with_integral(&pixg, nlevels, &integral, &sq_integral)
            .expect("quadtree_variance pre");

    // Results should be identical
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

    // If we got here without mismatch, record success
    rp.compare_values(1.0, 1.0, 0.0);

    assert!(rp.cleanup(), "quadtree precomputed test failed");
}

#[test]
fn quadtree_max_levels_various() {
    // Test quadtree_max_levels with various image sizes
    let mut rp = RegParams::new("quadtree_maxlevels");

    // C版ではboxaaQuadtreeRegionsの中でmax_levelsを暗黙的に使用
    // ここでは明示的にテスト

    // Small images
    rp.compare_values(0.0, quadtree_max_levels(1, 1) as f64, 0.0);
    rp.compare_values(0.0, quadtree_max_levels(2, 2) as f64, 0.0);
    rp.compare_values(1.0, quadtree_max_levels(4, 4) as f64, 0.0);
    rp.compare_values(2.0, quadtree_max_levels(8, 8) as f64, 0.0);
    rp.compare_values(3.0, quadtree_max_levels(16, 16) as f64, 0.0);

    // Non-square images use minimum dimension
    rp.compare_values(1.0, quadtree_max_levels(16, 4) as f64, 0.0);
    rp.compare_values(1.0, quadtree_max_levels(4, 16) as f64, 0.0);

    // Zero dimensions
    rp.compare_values(0.0, quadtree_max_levels(0, 10) as f64, 0.0);
    rp.compare_values(0.0, quadtree_max_levels(10, 0) as f64, 0.0);

    // Large images (like rabi.png after scaling)
    let max_632 = quadtree_max_levels(632, 825);
    eprintln!("  max_levels for 632x825: {}", max_632);
    rp.compare_values(1.0, if max_632 >= 8 { 1.0 } else { 0.0 }, 0.0);

    // C版で使用: 1000x500, nlevels=3
    let max_1000x500 = quadtree_max_levels(1000, 500);
    eprintln!("  max_levels for 1000x500: {}", max_1000x500);
    rp.compare_values(1.0, if max_1000x500 >= 3 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "quadtree max levels test failed");
}

#[test]
fn quadtree_error_cases() {
    // Test error handling
    let mut rp = RegParams::new("quadtree_errors");

    // quadtree_regions with nlevels=0 should fail
    let result = quadtree_regions(100, 100, 0);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    // quadtree_regions with too many levels should fail
    let result = quadtree_regions(4, 4, 10);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    // quadtree_mean on non-8bit image should fail
    let pix1 = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
    let result = quadtree_mean(&pix1, 2);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    // quadtree_variance on non-8bit image should fail
    let result = quadtree_variance(&pix1, 2);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    // IntegralImage on non-8bit image should fail
    let result = IntegralImage::from_pix(&pix1);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    // SquaredIntegralImage on non-8bit image should fail
    let result = SquaredIntegralImage::from_pix(&pix1);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "quadtree error cases test failed");
}

// C版: pixGetAverageTiled() -- leptonica-regionに未実装のためスキップ (regTest 5-6)
// C版: pixExpandReplicate() -- leptonica-regionに未実装のためスキップ (regTest 5-6)
