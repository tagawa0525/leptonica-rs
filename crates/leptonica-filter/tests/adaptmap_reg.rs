//! Adaptive mapping regression test
//!
//! C version: reference/leptonica/prog/adaptmap_reg.c
//!
//! Tests adaptive background normalization and contrast normalization.
//!
//! Rust API mapping:
//!   - pixBackgroundNorm() -> background_norm()
//!   - pixBackgroundNormSimple() -> background_norm_simple()
//!   - pixContrastNorm() -> contrast_norm()
//!
//! C low-level APIs not exposed in Rust:
//!   - pixGetBackgroundGrayMap, pixGetInvBackgroundMap, pixApplyInvBackgroundGrayMap
//!   - pixGetBackgroundRGBMap, pixApplyInvBackgroundRGBMap, pixFillMapHoles
//!   - pixGammaTRCMasked (leptonica-enhance, not yet implemented)

use leptonica_filter::{
    BackgroundNormOptions, ContrastNormOptions, background_norm, background_norm_simple,
    contrast_norm, contrast_norm_simple,
};
use leptonica_test::{RegParams, load_test_image};

/// Test background normalization on grayscale image.
///
/// C test 0-3: Grayscale low-level background map
/// C test 12-13: pixBackgroundNorm (high-level API)
#[test]
fn adaptmap_reg_background_norm_gray() {
    let mut rp = RegParams::new("adaptmap_bg_gray");

    // Use 8bpp grayscale test image
    let pixg = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let gw = pixg.width();
    let gh = pixg.height();

    // Test: background_norm_simple on grayscale
    let result = background_norm_simple(&pixg).expect("background_norm_simple gray");
    rp.compare_values(gw as f64, result.width() as f64, 0.0);
    rp.compare_values(gh as f64, result.height() as f64, 0.0);
    rp.compare_values(
        pixg.depth().bits() as f64,
        result.depth().bits() as f64,
        0.0,
    );

    // Test: background_norm with C-version parameters
    // C test 12: pixBackgroundNorm(pixs, pixim, NULL, 5, 10, 50, 20, 200, 2, 1)
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

    // Test: background_norm with SIZE_X=10, SIZE_Y=30
    // C test 0: pixGetBackgroundGrayMap(pixg, pixim, 10, 30, 50, 30, &pixgm)
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

    assert!(rp.cleanup(), "adaptmap_bg_gray regression test failed");
}

/// Test background normalization on color (32bpp) image.
///
/// C test 4-11: Color background map generation (RGB separate processing)
/// C test 12: pixBackgroundNorm on color
#[test]
fn adaptmap_reg_background_norm_color() {
    let mut rp = RegParams::new("adaptmap_bg_color");

    // C: pixs = pixRead("wet-day.jpg")
    let pixs = load_test_image("wet-day.jpg").expect("load wet-day.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // Test: background_norm_simple on color
    let result = background_norm_simple(&pixs).expect("background_norm_simple color");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(
        pixs.depth().bits() as f64,
        result.depth().bits() as f64,
        0.0,
    );

    // Test: background_norm with C-version parameters
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

    // Test: with tile 10x30
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

    assert!(rp.cleanup(), "adaptmap_bg_color regression test failed");
}

/// Test contrast normalization.
#[test]
fn adaptmap_reg_contrast_norm() {
    let mut rp = RegParams::new("adaptmap_contrast");

    let pixg = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixg.width();
    let h = pixg.height();

    // Test: contrast_norm_simple
    let result = contrast_norm_simple(&pixg).expect("contrast_norm_simple");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(8.0, result.depth().bits() as f64, 0.0);

    // Test: contrast_norm with custom options
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

    // Test: with larger tiles
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

    // Verify: contrast normalization should expand dynamic range
    let (orig_min, orig_max) = sample_min_max(&pixg);
    let (norm_min, norm_max) = sample_min_max(&result);
    let orig_range = orig_max.saturating_sub(orig_min);
    let norm_range = norm_max.saturating_sub(norm_min);
    let range_expanded = norm_range >= orig_range || norm_range >= 200;
    rp.compare_values(1.0, if range_expanded { 1.0 } else { 0.0 }, 0.0);

    // Test: contrast_norm rejects non-8bpp
    let pix32 = load_test_image("weasel32.png").expect("load weasel32.png");
    rp.compare_values(
        1.0,
        if contrast_norm_simple(&pix32).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Test: invalid parameters (tile too small)
    let bad_options = ContrastNormOptions {
        tile_width: 3,
        tile_height: 5,
        min_diff: 50,
        smooth_x: 2,
        smooth_y: 2,
    };
    rp.compare_values(
        1.0,
        if contrast_norm(&pixg, &bad_options).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Test: smooth too large
    let bad_options2 = ContrastNormOptions {
        tile_width: 20,
        tile_height: 20,
        min_diff: 50,
        smooth_x: 10,
        smooth_y: 2,
    };
    rp.compare_values(
        1.0,
        if contrast_norm(&pixg, &bad_options2).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "adaptmap_contrast regression test failed");
}

/// Test parameter validation for background normalization.
#[test]
fn adaptmap_reg_param_validation() {
    let mut rp = RegParams::new("adaptmap_params");

    let pixg = load_test_image("dreyfus8.png").expect("load dreyfus8.png");

    // Tile too small
    let bad = BackgroundNormOptions {
        tile_width: 2,
        ..Default::default()
    };
    rp.compare_values(
        1.0,
        if background_norm(&pixg, &bad).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // bg_val out of range
    let bad2 = BackgroundNormOptions {
        bg_val: 50,
        ..Default::default()
    };
    rp.compare_values(
        1.0,
        if background_norm(&pixg, &bad2).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Default options should work
    rp.compare_values(
        1.0,
        if background_norm(&pixg, &BackgroundNormOptions::default()).is_ok() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "adaptmap_params regression test failed");
}

/// C test 14-15: pixFillMapHoles -- internal function, not exposed
#[test]
#[ignore = "C: pixFillMapHoles() -- internal function, not exposed in Rust"]
fn adaptmap_reg_fill_map_holes() {}

/// C test 3, 11, 13: pixGammaTRCMasked -- leptonica-enhance not yet implemented
#[test]
#[ignore = "C: pixGammaTRCMasked() -- leptonica-enhance not yet implemented"]
fn adaptmap_reg_gamma_trc_masked() {}

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
