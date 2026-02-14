//! Adaptive normalization regression test
//!
//! C version: reference/leptonica/prog/adaptnorm_reg.c
//!
//! Tests adaptive normalization for two extreme cases:
//!   (1) Variable and low contrast -> pixContrastNorm pipeline
//!   (2) Good contrast but rapidly varying background -> pixBackgroundNormFlex pipeline
//!
//! NOTE: adaptmap_reg.rs tests background_norm and contrast_norm with dreyfus8.png/wet-day.jpg.
//! This test focuses on DIFFERENT images and parameters following C adaptnorm_reg.c:
//!   - lighttext.jpg for contrast normalization
//!   - w91frag.jpg for background normalization
//!
//! Rust API mapping:
//!   - pixContrastNorm(NULL, pixs, 10, 10, 40, 2, 2) -> contrast_norm()
//!   - pixBackgroundNormFlex(pixs, 7, 7, 1, 1, 10)   -> background_norm() (closest)
//!
//! C APIs not yet implemented (skipped):
//!   - pixGammaTRC, pixGammaTRCMasked, pixDitherTo2bpp, pixThresholdTo4bpp
//!   - pixThresholdToBinary, pixLocalExtrema, pixSeedfillGrayBasin, pixScaleSmooth

use leptonica_core::Pix;
use leptonica_filter::{
    BackgroundNormOptions, ContrastNormOptions, background_norm, background_norm_simple,
    contrast_norm, contrast_norm_simple,
};
use leptonica_test::{RegParams, load_test_image};

/// Helper: sample min and max pixel values from an 8bpp image
fn sample_min_max(pix: &Pix) -> (u32, u32) {
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

/// Helper: compute mean pixel value
fn sample_mean(pix: &Pix) -> f64 {
    let w = pix.width();
    let h = pix.height();
    let step = std::cmp::max(1, std::cmp::min(w, h) / 50) as usize;
    let mut sum = 0u64;
    let mut count = 0u64;
    for y in (0..h).step_by(step) {
        for x in (0..w).step_by(step) {
            sum += pix.get_pixel(x, y).unwrap_or(0) as u64;
            count += 1;
        }
    }
    if count > 0 {
        sum as f64 / count as f64
    } else {
        0.0
    }
}

/// Helper: compute standard deviation
fn sample_stddev(pix: &Pix) -> f64 {
    let w = pix.width();
    let h = pix.height();
    let step = std::cmp::max(1, std::cmp::min(w, h) / 50) as usize;
    let mean = sample_mean(pix);
    let mut sum_sq = 0.0f64;
    let mut count = 0u64;
    for y in (0..h).step_by(step) {
        for x in (0..w).step_by(step) {
            let val = pix.get_pixel(x, y).unwrap_or(0) as f64;
            sum_sq += (val - mean) * (val - mean);
            count += 1;
        }
    }
    if count > 1 {
        (sum_sq / (count - 1) as f64).sqrt()
    } else {
        0.0
    }
}

// ============================================================================
// Part 1: Contrast normalization on low-contrast text image (lighttext.jpg)
// ============================================================================

/// Test: Contrast normalization on low-contrast text.
///
/// C test 0-1: pixContrastNorm(NULL, pixs, 10, 10, 40, 2, 2) on lighttext.jpg
#[test]
fn adaptnorm_reg_contrast_norm_lighttext() {
    let mut rp = RegParams::new("adaptnorm_contrast");

    let pixs = load_test_image("lighttext.jpg").expect("load lighttext.jpg");
    let w = pixs.width();
    let h = pixs.height();

    let options = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 40,
        smooth_x: 2,
        smooth_y: 2,
    };
    let pix1 = contrast_norm(&pixs, &options).expect("contrast_norm(10, 10, 40, 2, 2)");
    rp.compare_values(w as f64, pix1.width() as f64, 0.0);
    rp.compare_values(h as f64, pix1.height() as f64, 0.0);
    rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);

    // Verify dynamic range expanded
    let (orig_min, orig_max) = sample_min_max(&pixs);
    let (norm_min, norm_max) = sample_min_max(&pix1);
    let orig_range = orig_max.saturating_sub(orig_min);
    let norm_range = norm_max.saturating_sub(norm_min);
    let range_expanded = norm_range >= orig_range || norm_range >= 200;
    rp.compare_values(1.0, if range_expanded { 1.0 } else { 0.0 }, 0.0);

    // Verify stddev increased
    let orig_std = sample_stddev(&pixs);
    let norm_std = sample_stddev(&pix1);
    let std_increased = norm_std >= orig_std || norm_std > 50.0;
    rp.compare_values(1.0, if std_increased { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "adaptnorm_contrast regression test failed");
}

/// Test: Contrast normalization with varied parameters on lighttext.
#[test]
fn adaptnorm_reg_contrast_norm_params() {
    let mut rp = RegParams::new("adaptnorm_contrast_params");

    let pixs = load_test_image("lighttext.jpg").expect("load lighttext.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // C exact parameters
    let opts_c = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 40,
        smooth_x: 2,
        smooth_y: 2,
    };
    let result_c = contrast_norm(&pixs, &opts_c).expect("contrast_norm C-params");
    rp.compare_values(w as f64, result_c.width() as f64, 0.0);
    rp.compare_values(h as f64, result_c.height() as f64, 0.0);

    // Smaller tiles
    let opts_small = ContrastNormOptions {
        tile_width: 5,
        tile_height: 5,
        min_diff: 40,
        smooth_x: 1,
        smooth_y: 1,
    };
    let result_small = contrast_norm(&pixs, &opts_small).expect("contrast_norm small tiles");
    rp.compare_values(w as f64, result_small.width() as f64, 0.0);
    rp.compare_values(h as f64, result_small.height() as f64, 0.0);

    // Different params should produce different results
    let mut differ = false;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            if result_c.get_pixel(x, y).unwrap_or(0) != result_small.get_pixel(x, y).unwrap_or(0) {
                differ = true;
                break;
            }
        }
        if differ {
            break;
        }
    }
    rp.compare_values(1.0, if differ { 1.0 } else { 0.0 }, 0.0);

    assert!(
        rp.cleanup(),
        "adaptnorm_contrast_params regression test failed"
    );
}

// ============================================================================
// Part 2: Background normalization on varying-background image (w91frag.jpg)
// ============================================================================

/// Test: Background normalization on rapidly varying background.
///
/// C test 8-9: pixBackgroundNormFlex(pixs, 7, 7, 1, 1, 10) on w91frag.jpg
#[test]
fn adaptnorm_reg_background_norm_w91frag() {
    let mut rp = RegParams::new("adaptnorm_bg");

    let pixs = load_test_image("w91frag.jpg").expect("load w91frag.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // Approximate C's pixBackgroundNormFlex with background_norm and small tiles
    let options = BackgroundNormOptions {
        tile_width: 7,
        tile_height: 7,
        fg_threshold: 60,
        min_count: 10,
        bg_val: 200,
        smooth_x: 1,
        smooth_y: 1,
    };
    let pix1 = background_norm(&pixs, &options).expect("background_norm on w91frag");
    rp.compare_values(w as f64, pix1.width() as f64, 0.0);
    rp.compare_values(h as f64, pix1.height() as f64, 0.0);

    // After normalization, mean should shift toward bg_val (200)
    let norm_mean = sample_mean(&pix1);
    let mean_ok = norm_mean > 100.0;
    rp.compare_values(1.0, if mean_ok { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "adaptnorm_bg regression test failed");
}

/// Test: Background normalization with different tile sizes on w91frag.
#[test]
fn adaptnorm_reg_background_norm_tile_sizes() {
    let mut rp = RegParams::new("adaptnorm_bg_tiles");

    let pixs = load_test_image("w91frag.jpg").expect("load w91frag.jpg");
    let w = pixs.width();
    let h = pixs.height();

    let opts_small = BackgroundNormOptions {
        tile_width: 7,
        tile_height: 7,
        fg_threshold: 40,
        min_count: 8,
        bg_val: 200,
        smooth_x: 1,
        smooth_y: 1,
    };
    let result_small = background_norm(&pixs, &opts_small).expect("small tiles");
    rp.compare_values(w as f64, result_small.width() as f64, 0.0);
    rp.compare_values(h as f64, result_small.height() as f64, 0.0);

    let opts_large = BackgroundNormOptions {
        tile_width: 30,
        tile_height: 30,
        fg_threshold: 40,
        min_count: 40,
        bg_val: 200,
        smooth_x: 3,
        smooth_y: 3,
    };
    let result_large = background_norm(&pixs, &opts_large).expect("large tiles");
    rp.compare_values(w as f64, result_large.width() as f64, 0.0);
    rp.compare_values(h as f64, result_large.height() as f64, 0.0);

    // Different tile sizes produce different results
    let mut differ = false;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            if result_small.get_pixel(x, y).unwrap_or(0)
                != result_large.get_pixel(x, y).unwrap_or(0)
            {
                differ = true;
                break;
            }
        }
        if differ {
            break;
        }
    }
    rp.compare_values(1.0, if differ { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "adaptnorm_bg_tiles regression test failed");
}

// ============================================================================
// Part 3: Combined pipeline
// ============================================================================

/// Test: contrast_norm followed by background_norm (pipeline).
#[test]
fn adaptnorm_reg_pipeline() {
    let mut rp = RegParams::new("adaptnorm_pipeline");

    let pixs = load_test_image("lighttext.jpg").expect("load lighttext.jpg");
    let w = pixs.width();
    let h = pixs.height();

    let contrast_opts = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 40,
        smooth_x: 2,
        smooth_y: 2,
    };
    let pix_contrast = contrast_norm(&pixs, &contrast_opts).expect("contrast_norm");
    rp.compare_values(w as f64, pix_contrast.width() as f64, 0.0);

    let bg_opts = BackgroundNormOptions {
        tile_width: 10,
        tile_height: 15,
        fg_threshold: 60,
        min_count: 40,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let pix_final = background_norm(&pix_contrast, &bg_opts).expect("background_norm");
    rp.compare_values(w as f64, pix_final.width() as f64, 0.0);

    // Pipeline should change the image
    let mut differ = false;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            if pixs.get_pixel(x, y).unwrap_or(0) != pix_final.get_pixel(x, y).unwrap_or(0) {
                differ = true;
                break;
            }
        }
        if differ {
            break;
        }
    }
    rp.compare_values(1.0, if differ { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "adaptnorm_pipeline regression test failed");
}

/// Test: background_norm_simple on w91frag.
#[test]
fn adaptnorm_reg_background_norm_simple_w91frag() {
    let mut rp = RegParams::new("adaptnorm_bg_simple");

    let pixs = load_test_image("w91frag.jpg").expect("load w91frag.jpg");
    let w = pixs.width();
    let h = pixs.height();

    let result = background_norm_simple(&pixs).expect("background_norm_simple on w91frag");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(
        pixs.depth().bits() as f64,
        result.depth().bits() as f64,
        0.0,
    );

    // Verify result differs from input
    let mut differ = false;
    for y in (0..h).step_by(5) {
        for x in (0..w).step_by(5) {
            if pixs.get_pixel(x, y).unwrap_or(0) != result.get_pixel(x, y).unwrap_or(0) {
                differ = true;
                break;
            }
        }
        if differ {
            break;
        }
    }
    rp.compare_values(1.0, if differ { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "adaptnorm_bg_simple regression test failed");
}

/// Test: contrast_norm_simple on lighttext.
#[test]
fn adaptnorm_reg_contrast_norm_simple_lighttext() {
    let mut rp = RegParams::new("adaptnorm_cn_simple");

    let pixs = load_test_image("lighttext.jpg").expect("load lighttext.jpg");
    let w = pixs.width();
    let h = pixs.height();

    let result = contrast_norm_simple(&pixs).expect("contrast_norm_simple on lighttext");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(8.0, result.depth().bits() as f64, 0.0);

    // Verify result differs from custom-params version
    let custom_opts = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 40,
        smooth_x: 2,
        smooth_y: 2,
    };
    let result_custom = contrast_norm(&pixs, &custom_opts).expect("contrast_norm custom");

    let mut differ = false;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            if result.get_pixel(x, y).unwrap_or(0) != result_custom.get_pixel(x, y).unwrap_or(0) {
                differ = true;
                break;
            }
        }
        if differ {
            break;
        }
    }
    rp.compare_values(1.0, if differ { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "adaptnorm_cn_simple regression test failed");
}

// ============================================================================
// Skipped tests for unimplemented C APIs
// ============================================================================

/// C: pixGammaTRC() -- leptonica-enhance not yet implemented
#[test]
#[ignore = "C: pixGammaTRC() -- leptonica-enhance not yet implemented"]
fn adaptnorm_reg_gamma_trc() {}

/// C: pixDitherTo2bpp(), pixThresholdTo4bpp() -- not yet implemented
#[test]
#[ignore = "C: pixDitherTo2bpp(), pixThresholdTo4bpp() -- not yet implemented"]
fn adaptnorm_reg_quantization() {}

/// C: pixThresholdToBinary() -- leptonica-binarize not yet implemented
#[test]
#[ignore = "C: pixThresholdToBinary() -- leptonica-binarize not yet implemented"]
fn adaptnorm_reg_binarization() {}

/// C: pixLocalExtrema(), pixSeedfillGrayBasin() -- not implemented in Rust
#[test]
#[ignore = "C: pixLocalExtrema(), pixSeedfillGrayBasin() -- not implemented in Rust"]
fn adaptnorm_reg_local_extrema_pipeline() {}

/// C: pixGammaTRCMasked() -- leptonica-enhance not yet implemented
#[test]
#[ignore = "C: pixGammaTRCMasked() -- leptonica-enhance not yet implemented"]
fn adaptnorm_reg_gamma_trc_masked() {}
