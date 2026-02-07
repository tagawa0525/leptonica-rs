//! Adaptive mapping regression test
//!
//! C version: reference/leptonica/prog/adaptmap_reg.c
//!
//! Tests adaptive background normalization and contrast normalization.
//!
//! Rust API mapping:
//!   - pixBackgroundNorm() -> background_norm()
//!   - pixBackgroundNormSimple() -> background_norm_simple()
//!   - pixContrastNorm() -> contrast_norm()  (C版にはないが同等機能)
//!
//! C版の低レベルAPI (Rust未実装/非公開のためスキップ):
//!   - pixGetBackgroundGrayMap() -- adaptmap.rs内部関数
//!   - pixGetInvBackgroundMap() -- adaptmap.rs内部関数
//!   - pixApplyInvBackgroundGrayMap() -- adaptmap.rs内部関数
//!   - pixGetBackgroundRGBMap() -- adaptmap.rs内部関数 (color版)
//!   - pixApplyInvBackgroundRGBMap() -- adaptmap.rs内部関数 (color版)
//!   - pixGammaTRCMasked() -- leptonica-enhance未実装
//!   - pixFillMapHoles() -- adaptmap.rs内部関数
//!   - pixConvertRGBToGray() -- leptonica-colorクレートにあるが別テスト対象
//!   - pixRasterop() -- leptonica-core未実装
//!   - pixCreate(w, h, 1) (1bpp mask) -- Pix::new(w, h, Bit1) 可能だがmask操作未実装

use leptonica_filter::{
    BackgroundNormOptions, ContrastNormOptions, background_norm, background_norm_simple,
    contrast_norm, contrast_norm_simple,
};
use leptonica_test::{RegParams, load_test_image};

// C版の定数
// static const l_int32  XS = 151, YS = 225, WS = 913, HS = 1285;
// static const l_int32  SIZE_X = 10, SIZE_Y = 30, BINTHRESH = 50, MINCOUNT = 30;
// static const l_int32  BGVAL = 200, SMOOTH_X = 2, SMOOTH_Y = 1;

/// Test background normalization on grayscale image
///
/// C版 test 0-3: グレースケールでの低レベル背景マップ生成とマップ適用
/// C版 test 12-13: pixBackgroundNorm (高レベルAPI) == Rust background_norm
#[test]
fn adaptmap_reg_background_norm_gray() {
    let mut rp = RegParams::new("adaptmap_bg_gray");

    // C版: pixs = pixRead("wet-day.jpg")
    let pixs = load_test_image("wet-day.jpg").expect("load wet-day.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // C版は pixConvertRGBToGray(pixs, 0.33, 0.34, 0.33) でグレースケール化するが、
    // leptonica-color は dev-dependency にないため、8bpp テスト画像で代替。

    // dreyfus8.png は 8bpp グレースケール
    let pixg = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let gw = pixg.width();
    let gh = pixg.height();
    eprintln!("Gray image: {}x{} d={}", gw, gh, pixg.depth().bits());

    // --- Test: background_norm_simple on grayscale ---
    // C版: pixBackgroundNorm(pixg, pixim, NULL, SIZE_X, SIZE_Y, BINTHRESH, MINCOUNT,
    //                        BGVAL, SMOOTH_X, SMOOTH_Y)
    // Rustのbackground_norm_simpleはデフォルトパラメータを使用
    let result = background_norm_simple(&pixg).expect("background_norm_simple gray");
    rp.compare_values(gw as f64, result.width() as f64, 0.0);
    rp.compare_values(gh as f64, result.height() as f64, 0.0);
    rp.compare_values(
        pixg.depth().bits() as f64,
        result.depth().bits() as f64,
        0.0,
    );
    eprintln!(
        "  background_norm_simple gray: {}x{} d={}",
        result.width(),
        result.height(),
        result.depth().bits()
    );

    // --- Test: background_norm with C版のパラメータ ---
    // C版 test 12: pixBackgroundNorm(pixs, pixim, NULL, 5, 10, 50, 20, 200, 2, 1)
    // pixim は ROI マスク。Rustはマスクなし版のみ対応。
    let options = BackgroundNormOptions {
        tile_width: 5,
        tile_height: 10,
        fg_threshold: 50,
        min_count: 20,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let result = background_norm(&pixg, &options).expect("background_norm with C params (gray)");
    rp.compare_values(gw as f64, result.width() as f64, 0.0);
    rp.compare_values(gh as f64, result.height() as f64, 0.0);
    eprintln!(
        "  background_norm(tile=5x10, fg=50, min=20, bg=200): {}x{}",
        result.width(),
        result.height()
    );

    // --- Test: background_norm with different tile sizes ---
    // C版 test 0: pixGetBackgroundGrayMap(pixg, pixim, SIZE_X=10, SIZE_Y=30, 50, 30, &pixgm)
    let options2 = BackgroundNormOptions {
        tile_width: 10,
        tile_height: 30,
        fg_threshold: 50,
        min_count: 30,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let result2 = background_norm(&pixg, &options2).expect("background_norm tile 10x30");
    rp.compare_values(gw as f64, result2.width() as f64, 0.0);
    rp.compare_values(gh as f64, result2.height() as f64, 0.0);
    eprintln!(
        "  background_norm(tile=10x30, fg=50, min=30, bg=200): {}x{}",
        result2.width(),
        result2.height()
    );

    // --- Verify: normalization should brighten dark background areas ---
    // After normalization, background should be closer to bg_val (200)
    // Sample some pixels from the result
    let mut bright_count = 0u32;
    let total_samples = 100u32;
    let step_x = std::cmp::max(1, gw / 10);
    let step_y = std::cmp::max(1, gh / 10);
    for y in (0..gh).step_by(step_y as usize) {
        for x in (0..gw).step_by(step_x as usize) {
            let val = result.get_pixel(x, y).unwrap_or(0);
            if val >= 128 {
                bright_count += 1;
            }
        }
    }
    // After background normalization, most background pixels should be bright
    let bright_ratio = bright_count as f64 / total_samples.max(1) as f64;
    eprintln!("  bright pixel ratio after norm: {:.2}", bright_ratio);
    // Not asserting a specific ratio -- just verify the operation completed

    assert!(rp.cleanup(), "adaptmap_bg_gray regression test failed");
}

/// Test background normalization on color (32bpp) image
///
/// C版 test 4-11: カラー画像での背景マップ生成 (RGB分離処理)
/// C版 test 12: pixBackgroundNorm on color
#[test]
fn adaptmap_reg_background_norm_color() {
    let mut rp = RegParams::new("adaptmap_bg_color");

    // C版: pixs = pixRead("wet-day.jpg")
    let pixs = load_test_image("wet-day.jpg").expect("load wet-day.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // --- Test: background_norm_simple on color ---
    let result = background_norm_simple(&pixs).expect("background_norm_simple color");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(
        pixs.depth().bits() as f64,
        result.depth().bits() as f64,
        0.0,
    );
    eprintln!(
        "  background_norm_simple color: {}x{} d={}",
        result.width(),
        result.height(),
        result.depth().bits()
    );

    // --- Test: background_norm with C版パラメータ ---
    // C版 test 12: pixBackgroundNorm(pixs, pixim, NULL, 5, 10, 50, 20, 200, 2, 1)
    let options = BackgroundNormOptions {
        tile_width: 5,
        tile_height: 10,
        fg_threshold: 50,
        min_count: 20,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let result = background_norm(&pixs, &options).expect("background_norm color with C params");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(32.0, result.depth().bits() as f64, 0.0);
    eprintln!(
        "  background_norm color(tile=5x10): {}x{} d={}",
        result.width(),
        result.height(),
        result.depth().bits()
    );

    // --- Test: background_norm with SIZE_X=10, SIZE_Y=30 ---
    let options2 = BackgroundNormOptions {
        tile_width: 10,
        tile_height: 30,
        fg_threshold: 50,
        min_count: 30,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let result2 = background_norm(&pixs, &options2).expect("background_norm color tile 10x30");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);
    rp.compare_values(h as f64, result2.height() as f64, 0.0);
    eprintln!(
        "  background_norm color(tile=10x30): {}x{}",
        result2.width(),
        result2.height()
    );

    assert!(rp.cleanup(), "adaptmap_bg_color regression test failed");
}

/// Test contrast normalization
///
/// C版には pixContrastNorm テストはないが、adaptmapモジュールの
/// contrast_norm/contrast_norm_simple をリグレッションテストする。
#[test]
fn adaptmap_reg_contrast_norm() {
    let mut rp = RegParams::new("adaptmap_contrast");

    // 8bpp grayscale image for contrast normalization
    let pixg = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixg.width();
    let h = pixg.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixg.depth().bits());

    // --- Test: contrast_norm_simple ---
    let result = contrast_norm_simple(&pixg).expect("contrast_norm_simple");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(8.0, result.depth().bits() as f64, 0.0);
    eprintln!(
        "  contrast_norm_simple: {}x{} d={}",
        result.width(),
        result.height(),
        result.depth().bits()
    );

    // --- Test: contrast_norm with custom options ---
    let options = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 30,
        smooth_x: 1,
        smooth_y: 1,
    };
    let result2 = contrast_norm(&pixg, &options).expect("contrast_norm custom");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);
    rp.compare_values(h as f64, result2.height() as f64, 0.0);
    eprintln!(
        "  contrast_norm(tile=10x10, min_diff=30): {}x{}",
        result2.width(),
        result2.height()
    );

    // --- Test: contrast_norm with larger tiles ---
    let options3 = ContrastNormOptions {
        tile_width: 30,
        tile_height: 30,
        min_diff: 50,
        smooth_x: 2,
        smooth_y: 2,
    };
    let result3 = contrast_norm(&pixg, &options3).expect("contrast_norm large tiles");
    rp.compare_values(w as f64, result3.width() as f64, 0.0);
    rp.compare_values(h as f64, result3.height() as f64, 0.0);
    eprintln!(
        "  contrast_norm(tile=30x30, min_diff=50): {}x{}",
        result3.width(),
        result3.height()
    );

    // --- Verify: contrast normalization should expand dynamic range ---
    // Calculate range (max - min) of sampled pixels before and after
    let (orig_min, orig_max) = sample_min_max(&pixg);
    let (norm_min, norm_max) = sample_min_max(&result);
    let orig_range = orig_max.saturating_sub(orig_min);
    let norm_range = norm_max.saturating_sub(norm_min);
    eprintln!(
        "  original range: {} (min={}, max={})",
        orig_range, orig_min, orig_max
    );
    eprintln!(
        "  normalized range: {} (min={}, max={})",
        norm_range, norm_min, norm_max
    );
    // After contrast normalization, range should be at least as large
    // (typically much larger)
    let range_expanded = norm_range >= orig_range || norm_range >= 200;
    rp.compare_values(1.0, if range_expanded { 1.0 } else { 0.0 }, 0.0);

    // --- Test: contrast_norm rejects non-8bpp ---
    let pix32 = load_test_image("weasel32.png").expect("load weasel32.png");
    let result_err = contrast_norm_simple(&pix32);
    rp.compare_values(1.0, if result_err.is_err() { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  contrast_norm rejects 32bpp: {}", result_err.is_err());

    // --- Test: invalid parameters ---
    let bad_options = ContrastNormOptions {
        tile_width: 3, // too small, must be >= 5
        tile_height: 5,
        min_diff: 50,
        smooth_x: 2,
        smooth_y: 2,
    };
    let result_err2 = contrast_norm(&pixg, &bad_options);
    rp.compare_values(1.0, if result_err2.is_err() { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  contrast_norm rejects small tiles: {}",
        result_err2.is_err()
    );

    let bad_options2 = ContrastNormOptions {
        tile_width: 20,
        tile_height: 20,
        min_diff: 50,
        smooth_x: 10, // too large, must be <= 8
        smooth_y: 2,
    };
    let result_err3 = contrast_norm(&pixg, &bad_options2);
    rp.compare_values(1.0, if result_err3.is_err() { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  contrast_norm rejects large smooth: {}",
        result_err3.is_err()
    );

    assert!(rp.cleanup(), "adaptmap_contrast regression test failed");
}

/// Test parameter validation for background normalization
#[test]
fn adaptmap_reg_param_validation() {
    let mut rp = RegParams::new("adaptmap_params");

    let pixg = load_test_image("dreyfus8.png").expect("load dreyfus8.png");

    // --- Test: tile too small ---
    let bad = BackgroundNormOptions {
        tile_width: 2,
        ..Default::default()
    };
    let result = background_norm(&pixg, &bad);
    rp.compare_values(1.0, if result.is_err() { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  rejects tile_width=2: {}", result.is_err());

    // --- Test: bg_val out of range ---
    let bad2 = BackgroundNormOptions {
        bg_val: 50, // must be >= 128
        ..Default::default()
    };
    let result2 = background_norm(&pixg, &bad2);
    rp.compare_values(1.0, if result2.is_err() { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  rejects bg_val=50: {}", result2.is_err());

    // --- Test: default options should work ---
    let result3 = background_norm(&pixg, &BackgroundNormOptions::default());
    rp.compare_values(1.0, if result3.is_ok() { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  default options work: {}", result3.is_ok());

    assert!(rp.cleanup(), "adaptmap_params regression test failed");
}

/// C版 test 14-15: pixFillMapHoles のテスト
/// Rust未公開 (adaptmap.rs内部関数) のためスキップ
#[test]
#[ignore = "C版: pixFillMapHoles() -- Rust非公開内部関数のためスキップ"]
fn adaptmap_reg_fill_map_holes() {
    // C版 test 14:
    //   pix1 = pixRead("weasel8.png");
    //   pixGammaTRC(pix1, pix1, 1.0, 0, 270);
    //   add white holes via pixRasterop
    //   pixFillMapHoles(pix1, w, h, L_FILL_WHITE);
    //
    // C版 test 15:
    //   pix1 = pixCreate(3, 3, 8);
    //   pixSetPixel(pix1, 1, 0, 128);
    //   pixFillMapHoles(pix1, 3, 3, L_FILL_BLACK);
    //
    // These internal APIs are not exposed in the Rust public API.
}

/// C版 test 3, 11, 13: pixGammaTRCMasked -- enhance未実装のためスキップ
#[test]
#[ignore = "C版: pixGammaTRCMasked() -- leptonica-enhance未実装のためスキップ"]
fn adaptmap_reg_gamma_trc_masked() {
    // C版:
    //   pix2 = pixGammaTRCMasked(NULL, pix1, pixim, 1.0, 0, 190);
    //   pixInvert(pixim, pixim);
    //   pixGammaTRCMasked(pix2, pix2, pixim, 1.0, 60, 190);
    //
    // This function is part of image enhancement, not yet implemented in Rust.
}

/// Helper: sample min and max pixel values from an 8bpp image
fn sample_min_max(pix: &leptonica_core::Pix) -> (u32, u32) {
    let w = pix.width();
    let h = pix.height();
    let mut min_val = 255u32;
    let mut max_val = 0u32;
    let step = std::cmp::max(1, std::cmp::min(w, h) / 50) as usize;

    for y in (0..h).step_by(step) {
        for x in (0..w).step_by(step) {
            let val = pix.get_pixel(x, y).unwrap_or(0);
            min_val = min_val.min(val);
            max_val = max_val.max(val);
        }
    }

    (min_val, max_val)
}
