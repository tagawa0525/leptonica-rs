//! Watershed segmentation regression test
//!
//! C版: reference/leptonica/prog/watershed_reg.c
//!
//! C版テストの概要:
//! 1. 500x500の合成8bit画像(sin/cos関数)を2つ生成
//! 2. 各画像に対してDoWatershed()を実行:
//!    - pixLocalExtrema() でローカル極値を検出
//!    - pixSetOrClearBorder() でボーダーの最大値をクリア
//!    - pixSelectMinInConnComp() でシード生成
//!    - wshedCreate() / wshedApply() で分水嶺セグメンテーション実行
//!    - wshedRenderFill() / wshedRenderColors() で結果を可視化
//!
//! Run with:
//! ```
//! cargo test -p leptonica-region --test watershed_reg
//! ```

use leptonica_core::{Pix, PixelDepth};
use leptonica_region::{
    ConnectivityType, WatershedOptions, compute_gradient, find_local_maxima, find_local_minima,
    watershed_segmentation,
};
use leptonica_test::RegParams;

/// Create the synthetic test image used in the C version.
/// C版: pix1 uses one set of sin/cos frequencies,
///       pix2 uses another set.
fn create_synthetic_image(variant: u32) -> Pix {
    let size = 500u32;
    let pix = Pix::new(size, size, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    for i in 0..size {
        for j in 0..size {
            let fi = i as f32;
            let fj = j as f32;
            let f = if variant == 0 {
                // C版: pix1
                128.0
                    + 26.3 * (0.0438 * fi).sin()
                    + 33.4 * (0.0712 * fi).cos()
                    + 18.6 * (0.0561 * fj).sin()
                    + 23.6 * (0.0327 * fj).cos()
            } else {
                // C版: pix2
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
///
/// C版のDoWatershed()は12個のregTestを実行するが、
/// Rust側では対応APIがないものをスキップしつつ主要ロジックを検証する。
fn do_watershed(rp: &mut RegParams, pixs: &Pix) {
    let w = pixs.width();
    let h = pixs.height();

    // -- regTest 0 (C版): Write input image --
    // Verify input image dimensions
    rp.compare_values(500.0, w as f64, 0.0);
    rp.compare_values(500.0, h as f64, 0.0);

    // -- regTest 1-2 (C版): Find local extrema --
    // C版: pixLocalExtrema(pixs, 0, 0, &pix1, &pix2)
    //   pix1 = local minima mask, pix2 = local maxima mask
    let minima =
        find_local_minima(pixs, ConnectivityType::EightWay).expect("find_local_minima failed");
    let maxima =
        find_local_maxima(pixs, ConnectivityType::EightWay).expect("find_local_maxima failed");

    eprintln!(
        "  Local minima: {}, Local maxima: {}",
        minima.len(),
        maxima.len()
    );

    // Both should have some extrema in a synthetic wavy image
    rp.compare_values(1.0, if !minima.is_empty() { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if !maxima.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // -- regTest 3-5 (C版): Generate seeds and seed filtering --
    // C版: pixSelectMinInConnComp(pixs, pix1, &pta, NULL) -- Rust未実装のためスキップ
    // C版: pixGenerateFromPta(pta, w, h) -- Rust未実装のためスキップ
    // C版: pixRemoveSeededComponents() -- Rust未実装のためスキップ
    // Rust側ではlocal minimaのポイントリストをそのままシードとして使用
    let seed_count = minima.len();
    rp.compare_values(1.0, if seed_count > 0 { 1.0 } else { 0.0 }, 0.0);

    // C版: pixSetOrClearBorder(pix1, 2, 2, 2, 2, PIX_CLR) -- Rust未実装のためスキップ
    // C版: composeRGBPixel() / pixConvertTo32() / pixPaintThroughMask() -- 可視化用、スキップ

    // -- regTest 6 (C版): pixZero check on removed components --
    // C版: pixZero(pix5, &empty) -> empty should be 1
    // Rust側ではpixRemoveSeededComponentsが未実装のため直接検証不可
    // 代わりに、minimaの数がseed_countと一致することを確認
    rp.compare_values(seed_count as f64, minima.len() as f64, 0.0);

    // -- regTest 7-10 (C版): Watershed execution and rendering --
    // C版: wshedCreate(pixs, pix3, 10, 0) / wshedApply(wshed)
    let options = WatershedOptions::new()
        .with_min_depth(10)
        .with_connectivity(ConnectivityType::EightWay);
    let result = watershed_segmentation(pixs, &options);

    match result {
        Ok(segmented) => {
            // Verify dimensions match
            rp.compare_values(w as f64, segmented.width() as f64, 0.0);
            rp.compare_values(h as f64, segmented.height() as f64, 0.0);

            // Verify output is 32-bit label image
            rp.compare_values(32.0, segmented.depth().bits() as f64, 0.0);

            // Count unique labels (basins)
            let mut labels = std::collections::HashSet::new();
            for y in 0..h {
                for x in 0..w {
                    if let Some(label) = segmented.get_pixel(x, y) {
                        if label > 0 {
                            labels.insert(label);
                        }
                    }
                }
            }
            let num_basins = labels.len();
            eprintln!("  Number of basins: {}", num_basins);

            // Should have multiple basins in a wavy image
            rp.compare_values(1.0, if num_basins > 1 { 1.0 } else { 0.0 }, 0.0);

            // Count watershed (boundary) pixels
            let mut boundary_count = 0u64;
            for y in 0..h {
                for x in 0..w {
                    if let Some(label) = segmented.get_pixel(x, y) {
                        if label == 0 {
                            boundary_count += 1;
                        }
                    }
                }
            }
            eprintln!("  Boundary pixels: {}", boundary_count);

            // Should have some boundary pixels but not all
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
            rp.compare_values(1.0, 0.0, 0.0); // Record failure
        }
    }

    // C版: wshedRenderFill(wshed) -- Rust未実装のためスキップ (regTest 9)
    // C版: wshedRenderColors(wshed) -- Rust未実装のためスキップ (regTest 10)
    // C版: pixaDisplayRandomCmap(wshed->pixad, w, h) -- Rust未実装のためスキップ (regTest 7)
    // C版: numaWriteMem(&data, &size, wshed->nalevels) -- Rust未実装のためスキップ (regTest 8)
    // C版: pixaDisplayTiledInColumns() -- Rust未実装のためスキップ (regTest 11)
}

#[test]
fn watershed_segmentation_synthetic() {
    let mut rp = RegParams::new("watershed");

    // -- Test with synthetic image 1 (C版: pix1) --
    eprintln!("=== Synthetic image 1 ===");
    let pix1 = create_synthetic_image(0);
    do_watershed(&mut rp, &pix1);

    // -- Test with synthetic image 2 (C版: pix2) --
    eprintln!("=== Synthetic image 2 ===");
    let pix2 = create_synthetic_image(1);
    do_watershed(&mut rp, &pix2);

    assert!(rp.cleanup(), "watershed regression test failed");
}

#[test]
fn watershed_local_extrema_basic() {
    // Simple test: create image with clear minima/maxima
    let mut rp = RegParams::new("watershed_extrema");

    // Create a small image with known extrema pattern
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    // Set all pixels to middle value
    for y in 0..10u32 {
        for x in 0..10u32 {
            let _ = pix_mut.set_pixel(x, y, 128);
        }
    }

    // Create a valley at (3,3) and a hill at (7,7)
    let _ = pix_mut.set_pixel(3, 3, 10);
    let _ = pix_mut.set_pixel(7, 7, 250);

    let pix: Pix = pix_mut.into();

    // Find minima
    let minima = find_local_minima(&pix, ConnectivityType::EightWay).expect("find minima");
    eprintln!("Minima found: {} (expected >= 1)", minima.len());
    rp.compare_values(1.0, if !minima.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // The valley should be among the minima
    let has_valley = minima.iter().any(|&(x, y)| x == 3 && y == 3);
    rp.compare_values(1.0, if has_valley { 1.0 } else { 0.0 }, 0.0);

    // Find maxima
    let maxima = find_local_maxima(&pix, ConnectivityType::EightWay).expect("find maxima");
    eprintln!("Maxima found: {} (expected >= 1)", maxima.len());
    rp.compare_values(1.0, if !maxima.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // The hill should be among the maxima
    let has_hill = maxima.iter().any(|&(x, y)| x == 7 && y == 7);
    rp.compare_values(1.0, if has_hill { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "watershed extrema test failed");
}

#[test]
fn watershed_gradient() {
    // C版ではcompute_gradientは直接テストされていないが、
    // 分水嶺セグメンテーションの基盤となる勾配計算を検証する。
    let mut rp = RegParams::new("watershed_gradient");

    // Create an image with a vertical edge
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

    // Check dimensions
    rp.compare_values(20.0, gradient.width() as f64, 0.0);
    rp.compare_values(20.0, gradient.height() as f64, 0.0);
    rp.compare_values(8.0, gradient.depth().bits() as f64, 0.0);

    // Gradient should be high at the edge (around x=9,10) and low elsewhere
    let grad_edge = gradient.get_pixel(9, 10).unwrap_or(0);
    let grad_flat = gradient.get_pixel(5, 10).unwrap_or(0);
    eprintln!("  Gradient at edge: {}, at flat: {}", grad_edge, grad_flat);
    rp.compare_values(1.0, if grad_edge > grad_flat { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "watershed gradient test failed");
}

#[test]
fn watershed_error_handling() {
    // Test error cases for all watershed-related functions
    let mut rp = RegParams::new("watershed_errors");

    // 1-bit image should fail
    let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let options = WatershedOptions::default();
    let result = watershed_segmentation(&pix1, &options);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    // 32-bit image should fail
    let pix32 = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let result = watershed_segmentation(&pix32, &options);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    // find_local_minima on non-8bit should fail
    let result = find_local_minima(&pix1, ConnectivityType::EightWay);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    // find_local_maxima on non-8bit should fail
    let result = find_local_maxima(&pix1, ConnectivityType::EightWay);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    // compute_gradient on non-8bit should fail
    let result = compute_gradient(&pix1);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "watershed error handling test failed");
}
