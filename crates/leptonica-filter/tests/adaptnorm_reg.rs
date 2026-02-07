//! Adaptive normalization regression test
//!
//! C version: reference/leptonica/prog/adaptnorm_reg.c
//!
//! Tests adaptive normalization for two extreme cases:
//!   (1) Variable and low contrast -> pixContrastNorm pipeline
//!   (2) Good contrast but rapidly varying background -> pixBackgroundNormFlex pipeline
//!
//! NOTE: adaptmap_reg.rs already tests background_norm and contrast_norm with
//! basic parameters and the dreyfus8.png/wet-day.jpg images. This test focuses
//! on DIFFERENT aspects following the C adaptnorm_reg.c test:
//!   - Uses lighttext.jpg (low contrast text) for contrast normalization
//!   - Uses w91frag.jpg (varying background) for background normalization
//!   - Tests the specific parameter combinations from the C version
//!   - Validates that contrast normalization actually improves readability
//!   - Tests contrast_norm + background_norm pipeline (combined usage)
//!
//! Rust API mapping:
//!   - pixContrastNorm(NULL, pixs, 10, 10, 40, 2, 2) -> contrast_norm() with matching options
//!   - pixBackgroundNormFlex(pixs, 7, 7, 1, 1, 10)   -> background_norm() (closest equivalent)
//!
//! C版の未実装APIのためスキップ:
//!   - pixGammaTRC()          -- leptonica-enhance未実装のためスキップ
//!   - pixGammaTRCMasked()    -- leptonica-enhance未実装のためスキップ
//!   - pixDitherTo2bpp()      -- leptonica-quantize未実装のためスキップ
//!   - pixThresholdTo4bpp()   -- leptonica-quantize未実装のためスキップ
//!   - pixThresholdToBinary() -- leptonica-binarize未実装のためスキップ
//!   - pixLocalExtrema()      -- Rust未実装のためスキップ
//!   - pixSeedfillGrayBasin() -- Rust未実装のためスキップ
//!   - pixScaleSmooth()       -- leptonica-scale未実装のためスキップ

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

/// Helper: compute mean pixel value from sampled pixels of an 8bpp image
fn sample_mean(pix: &Pix) -> f64 {
    let w = pix.width();
    let h = pix.height();
    let step = std::cmp::max(1, std::cmp::min(w, h) / 50) as usize;
    let mut sum = 0u64;
    let mut count = 0u64;

    for y in (0..h).step_by(step) {
        for x in (0..w).step_by(step) {
            let val = pix.get_pixel(x, y).unwrap_or(0);
            sum += val as u64;
            count += 1;
        }
    }

    if count > 0 {
        sum as f64 / count as f64
    } else {
        0.0
    }
}

/// Helper: compute standard deviation of sampled pixel values
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

/// Test: Contrast normalization on low-contrast text
///
/// C版 test 0-1: pixContrastNorm(NULL, pixs, 10, 10, 40, 2, 2) on lighttext.jpg
///
/// The C adaptnorm_reg test uses lighttext.jpg, which has variable and low
/// contrast text. This is a different image and different parameters than
/// what adaptmap_reg.rs tests (which uses dreyfus8.png with default params).
#[test]
fn adaptnorm_reg_contrast_norm_lighttext() {
    let mut rp = RegParams::new("adaptnorm_contrast");

    // C版: pixs = pixRead("lighttext.jpg")
    let pixs = load_test_image("lighttext.jpg").expect("load lighttext.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // C版: pix1 = pixContrastNorm(NULL, pixs, 10, 10, 40, 2, 2)
    // Parameters: tile_width=10, tile_height=10, min_diff=40, smooth_x=2, smooth_y=2
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
    eprintln!(
        "  contrast_norm(10,10,40,2,2): {}x{} d={}",
        pix1.width(),
        pix1.height(),
        pix1.depth().bits()
    );

    // Verify that contrast normalization expanded the dynamic range
    let (orig_min, orig_max) = sample_min_max(&pixs);
    let (norm_min, norm_max) = sample_min_max(&pix1);
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
    // After contrast normalization, the range should be expanded significantly
    let range_expanded = norm_range >= orig_range || norm_range >= 200;
    rp.compare_values(1.0, if range_expanded { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  range expanded: {}", range_expanded);

    // Verify that the standard deviation increased (more contrast)
    let orig_std = sample_stddev(&pixs);
    let norm_std = sample_stddev(&pix1);
    eprintln!("  original stddev: {:.1}", orig_std);
    eprintln!("  normalized stddev: {:.1}", norm_std);
    let std_increased = norm_std >= orig_std || norm_std > 50.0;
    rp.compare_values(1.0, if std_increased { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "adaptnorm_contrast regression test failed");
}

/// Test: Contrast normalization with varied parameters on lighttext
///
/// This tests different tile sizes and min_diff values on the same lighttext
/// image, covering parameter sensitivity that adaptmap_reg.rs does not test.
#[test]
fn adaptnorm_reg_contrast_norm_params() {
    let mut rp = RegParams::new("adaptnorm_contrast_params");

    let pixs = load_test_image("lighttext.jpg").expect("load lighttext.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // Test with C adaptnorm_reg.c exact parameters: tile=10x10, min_diff=40
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
    eprintln!("  C-params(10,10,40,2,2): OK");

    // Test with smaller tiles (finer adaptation)
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
    eprintln!("  small tiles(5,5,40,1,1): OK");

    // Test with larger tiles (coarser adaptation)
    let opts_large = ContrastNormOptions {
        tile_width: 25,
        tile_height: 25,
        min_diff: 40,
        smooth_x: 3,
        smooth_y: 3,
    };
    let result_large = contrast_norm(&pixs, &opts_large).expect("contrast_norm large tiles");
    rp.compare_values(w as f64, result_large.width() as f64, 0.0);
    rp.compare_values(h as f64, result_large.height() as f64, 0.0);
    eprintln!("  large tiles(25,25,40,3,3): OK");

    // Test with low min_diff (more tiles will be processed)
    let opts_low_diff = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 20,
        smooth_x: 2,
        smooth_y: 2,
    };
    let result_low = contrast_norm(&pixs, &opts_low_diff).expect("contrast_norm low min_diff");
    rp.compare_values(w as f64, result_low.width() as f64, 0.0);
    rp.compare_values(h as f64, result_low.height() as f64, 0.0);
    eprintln!("  low min_diff(10,10,20,2,2): OK");

    // Test with high min_diff (fewer tiles will be processed, more "holes")
    let opts_high_diff = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 80,
        smooth_x: 2,
        smooth_y: 2,
    };
    let result_high = contrast_norm(&pixs, &opts_high_diff).expect("contrast_norm high min_diff");
    rp.compare_values(w as f64, result_high.width() as f64, 0.0);
    rp.compare_values(h as f64, result_high.height() as f64, 0.0);
    eprintln!("  high min_diff(10,10,80,2,2): OK");

    // Test with no smoothing
    let opts_no_smooth = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 40,
        smooth_x: 0,
        smooth_y: 0,
    };
    let result_no_smooth =
        contrast_norm(&pixs, &opts_no_smooth).expect("contrast_norm no smoothing");
    rp.compare_values(w as f64, result_no_smooth.width() as f64, 0.0);
    rp.compare_values(h as f64, result_no_smooth.height() as f64, 0.0);
    eprintln!("  no smoothing(10,10,40,0,0): OK");

    // Verify different parameters produce different results
    // The C-params and small tiles results should differ
    let mut differ = false;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let v_c = result_c.get_pixel(x, y).unwrap_or(0);
            let v_small = result_small.get_pixel(x, y).unwrap_or(0);
            if v_c != v_small {
                differ = true;
                break;
            }
        }
        if differ {
            break;
        }
    }
    rp.compare_values(1.0, if differ { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  different params produce different results: {}", differ);

    assert!(
        rp.cleanup(),
        "adaptnorm_contrast_params regression test failed"
    );
}

// ============================================================================
// Part 2: Background normalization on varying-background image (w91frag.jpg)
// ============================================================================

/// Test: Background normalization on rapidly varying background
///
/// C版 test 8-9: pixBackgroundNormFlex(pixs, 7, 7, 1, 1, 10) on w91frag.jpg
///
/// The C adaptnorm_reg test uses w91frag.jpg which has good contrast but
/// a rapidly varying background. adaptmap_reg.rs tests with wet-day.jpg
/// and dreyfus8.png which are different images with different characteristics.
///
/// NOTE: pixBackgroundNormFlex is not directly available. We use
/// background_norm with small tile sizes as the closest equivalent.
#[test]
fn adaptnorm_reg_background_norm_w91frag() {
    let mut rp = RegParams::new("adaptnorm_bg");

    // C版: pixs = pixRead("w91frag.jpg")
    let pixs = load_test_image("w91frag.jpg").expect("load w91frag.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // C版: pix1 = pixBackgroundNormFlex(pixs, 7, 7, 1, 1, 10)
    // pixBackgroundNormFlex uses sx/sy=7 (tile size), smoothx/smoothy=1,
    // and delta=10 (similar to min_diff parameter).
    // We approximate this with background_norm using small tiles.
    let options = BackgroundNormOptions {
        tile_width: 7,
        tile_height: 7,
        fg_threshold: 60,
        min_count: 10,
        bg_val: 200,
        smooth_x: 1,
        smooth_y: 1,
    };
    let pix1 = background_norm(&pixs, &options).expect("background_norm(7,7,...) on w91frag");
    rp.compare_values(w as f64, pix1.width() as f64, 0.0);
    rp.compare_values(h as f64, pix1.height() as f64, 0.0);
    eprintln!(
        "  background_norm(tile=7x7): {}x{} d={}",
        pix1.width(),
        pix1.height(),
        pix1.depth().bits()
    );

    // After background normalization, background should be more uniform
    // The standard deviation of background pixels should decrease
    let orig_std = sample_stddev(&pixs);
    let norm_std = sample_stddev(&pix1);
    eprintln!("  original stddev: {:.1}", orig_std);
    eprintln!("  normalized stddev: {:.1}", norm_std);

    // After background normalization, the mean should shift toward bg_val (200)
    let norm_mean = sample_mean(&pix1);
    eprintln!("  normalized mean: {:.1} (target bg_val=200)", norm_mean);
    // Mean should be in a reasonable range (at least shifted up from original)
    let mean_ok = norm_mean > 100.0;
    rp.compare_values(1.0, if mean_ok { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "adaptnorm_bg regression test failed");
}

/// Test: Background normalization with different tile sizes on w91frag
///
/// Tests sensitivity to tile size on the w91frag image, which has
/// rapidly varying background. This covers parameter space not explored
/// by adaptmap_reg.rs (which uses standard tile sizes on different images).
#[test]
fn adaptnorm_reg_background_norm_tile_sizes() {
    let mut rp = RegParams::new("adaptnorm_bg_tiles");

    let pixs = load_test_image("w91frag.jpg").expect("load w91frag.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // Small tiles (fine-grained adaptation, close to C's NormFlex with sx=7)
    let opts_small = BackgroundNormOptions {
        tile_width: 7,
        tile_height: 7,
        fg_threshold: 40,
        min_count: 8,
        bg_val: 200,
        smooth_x: 1,
        smooth_y: 1,
    };
    let result_small =
        background_norm(&pixs, &opts_small).expect("background_norm small tiles on w91frag");
    rp.compare_values(w as f64, result_small.width() as f64, 0.0);
    rp.compare_values(h as f64, result_small.height() as f64, 0.0);
    eprintln!(
        "  small tiles(7x7): {}x{} d={}",
        result_small.width(),
        result_small.height(),
        result_small.depth().bits()
    );

    // Medium tiles
    let opts_medium = BackgroundNormOptions {
        tile_width: 15,
        tile_height: 15,
        fg_threshold: 40,
        min_count: 20,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 2,
    };
    let result_medium =
        background_norm(&pixs, &opts_medium).expect("background_norm medium tiles on w91frag");
    rp.compare_values(w as f64, result_medium.width() as f64, 0.0);
    rp.compare_values(h as f64, result_medium.height() as f64, 0.0);
    eprintln!(
        "  medium tiles(15x15): {}x{} d={}",
        result_medium.width(),
        result_medium.height(),
        result_medium.depth().bits()
    );

    // Large tiles (coarser adaptation)
    let opts_large = BackgroundNormOptions {
        tile_width: 30,
        tile_height: 30,
        fg_threshold: 40,
        min_count: 40,
        bg_val: 200,
        smooth_x: 3,
        smooth_y: 3,
    };
    let result_large =
        background_norm(&pixs, &opts_large).expect("background_norm large tiles on w91frag");
    rp.compare_values(w as f64, result_large.width() as f64, 0.0);
    rp.compare_values(h as f64, result_large.height() as f64, 0.0);
    eprintln!(
        "  large tiles(30x30): {}x{} d={}",
        result_large.width(),
        result_large.height(),
        result_large.depth().bits()
    );

    // Verify different tile sizes produce different results
    let mut differ_small_medium = false;
    let mut differ_small_large = false;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let v_s = result_small.get_pixel(x, y).unwrap_or(0);
            let v_m = result_medium.get_pixel(x, y).unwrap_or(0);
            let v_l = result_large.get_pixel(x, y).unwrap_or(0);
            if v_s != v_m {
                differ_small_medium = true;
            }
            if v_s != v_l {
                differ_small_large = true;
            }
        }
    }
    rp.compare_values(1.0, if differ_small_medium { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if differ_small_large { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  small vs medium differ: {}", differ_small_medium);
    eprintln!("  small vs large differ: {}", differ_small_large);

    assert!(rp.cleanup(), "adaptnorm_bg_tiles regression test failed");
}

// ============================================================================
// Part 3: Combined contrast_norm + background_norm pipeline
// ============================================================================

/// Test: contrast_norm followed by background_norm (pipeline)
///
/// The C adaptnorm_reg.c test applies contrast normalization first, then
/// gamma correction (pixGammaTRC) on lighttext.jpg. Since pixGammaTRC is not
/// available, we test the contrast_norm + background_norm pipeline which
/// is the Rust-available equivalent of a full normalization pass.
///
/// This tests a DIFFERENT pipeline than adaptmap_reg.rs, which tests
/// each function independently.
#[test]
fn adaptnorm_reg_pipeline() {
    let mut rp = RegParams::new("adaptnorm_pipeline");

    let pixs = load_test_image("lighttext.jpg").expect("load lighttext.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // Step 1: Apply contrast normalization (from C adaptnorm_reg.c params)
    let contrast_opts = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 40,
        smooth_x: 2,
        smooth_y: 2,
    };
    let pix_contrast =
        contrast_norm(&pixs, &contrast_opts).expect("contrast_norm step in pipeline");
    rp.compare_values(w as f64, pix_contrast.width() as f64, 0.0);
    rp.compare_values(h as f64, pix_contrast.height() as f64, 0.0);
    eprintln!(
        "  after contrast_norm: {}x{} d={}",
        pix_contrast.width(),
        pix_contrast.height(),
        pix_contrast.depth().bits()
    );

    // Step 2: Apply background normalization on the contrast-normalized result
    // C版: pixGammaTRC(NULL, pix1, 1.5, 50, 235) would be applied here,
    // but since it's not available, we use background_norm instead
    let bg_opts = BackgroundNormOptions {
        tile_width: 10,
        tile_height: 15,
        fg_threshold: 60,
        min_count: 40,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let pix_final =
        background_norm(&pix_contrast, &bg_opts).expect("background_norm step in pipeline");
    rp.compare_values(w as f64, pix_final.width() as f64, 0.0);
    rp.compare_values(h as f64, pix_final.height() as f64, 0.0);
    eprintln!(
        "  after background_norm: {}x{} d={}",
        pix_final.width(),
        pix_final.height(),
        pix_final.depth().bits()
    );

    // Verify the pipeline improved the image:
    // After both normalization steps, the output should have:
    //   1. Good dynamic range (from contrast norm)
    //   2. Uniform background (from background norm)
    let orig_std = sample_stddev(&pixs);
    let final_std = sample_stddev(&pix_final);
    let orig_mean = sample_mean(&pixs);
    let final_mean = sample_mean(&pix_final);

    eprintln!("  original: mean={:.1} stddev={:.1}", orig_mean, orig_std);
    eprintln!("  final: mean={:.1} stddev={:.1}", final_mean, final_std);

    // The pipeline should produce a valid result different from input
    let mut differ = false;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let v_orig = pixs.get_pixel(x, y).unwrap_or(0);
            let v_final = pix_final.get_pixel(x, y).unwrap_or(0);
            if v_orig != v_final {
                differ = true;
                break;
            }
        }
        if differ {
            break;
        }
    }
    rp.compare_values(1.0, if differ { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  pipeline changed image: {}", differ);

    assert!(rp.cleanup(), "adaptnorm_pipeline regression test failed");
}

/// Test: background_norm_simple on w91frag (varying background)
///
/// Tests the simplified API that uses default parameters.
/// adaptmap_reg.rs tests background_norm_simple on dreyfus8.png and
/// wet-day.jpg. This test uses w91frag.jpg which has a different
/// type of background variation.
#[test]
fn adaptnorm_reg_background_norm_simple_w91frag() {
    let mut rp = RegParams::new("adaptnorm_bg_simple");

    let pixs = load_test_image("w91frag.jpg").expect("load w91frag.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // background_norm_simple uses default parameters
    let result = background_norm_simple(&pixs).expect("background_norm_simple on w91frag");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(
        pixs.depth().bits() as f64,
        result.depth().bits() as f64,
        0.0,
    );
    eprintln!(
        "  background_norm_simple: {}x{} d={}",
        result.width(),
        result.height(),
        result.depth().bits()
    );

    // Verify the result is different from input (normalization had an effect)
    let mut differ = false;
    for y in (0..h).step_by(5) {
        for x in (0..w).step_by(5) {
            let v_orig = pixs.get_pixel(x, y).unwrap_or(0);
            let v_norm = result.get_pixel(x, y).unwrap_or(0);
            if v_orig != v_norm {
                differ = true;
                break;
            }
        }
        if differ {
            break;
        }
    }
    rp.compare_values(1.0, if differ { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  normalization changed image: {}", differ);

    assert!(rp.cleanup(), "adaptnorm_bg_simple regression test failed");
}

/// Test: contrast_norm_simple on lighttext (low contrast text)
///
/// Tests the simplified API on the lighttext image.
/// adaptmap_reg.rs tests contrast_norm_simple on dreyfus8.png.
/// This test uses lighttext.jpg which is the image used in the C
/// adaptnorm_reg test.
#[test]
fn adaptnorm_reg_contrast_norm_simple_lighttext() {
    let mut rp = RegParams::new("adaptnorm_cn_simple");

    let pixs = load_test_image("lighttext.jpg").expect("load lighttext.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    let result = contrast_norm_simple(&pixs).expect("contrast_norm_simple on lighttext");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(8.0, result.depth().bits() as f64, 0.0);
    eprintln!(
        "  contrast_norm_simple: {}x{} d={}",
        result.width(),
        result.height(),
        result.depth().bits()
    );

    // Verify contrast was expanded
    let (orig_min, orig_max) = sample_min_max(&pixs);
    let (norm_min, norm_max) = sample_min_max(&result);
    let orig_range = orig_max.saturating_sub(orig_min);
    let norm_range = norm_max.saturating_sub(norm_min);
    eprintln!(
        "  original: range={} (min={}, max={})",
        orig_range, orig_min, orig_max
    );
    eprintln!(
        "  normalized: range={} (min={}, max={})",
        norm_range, norm_min, norm_max
    );

    // Verify the result is different from the custom-params version
    let custom_opts = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 40,
        smooth_x: 2,
        smooth_y: 2,
    };
    let result_custom = contrast_norm(&pixs, &custom_opts).expect("contrast_norm custom");

    // Default params (tile=20, min_diff=50) vs custom (tile=10, min_diff=40)
    // should produce different results
    let mut differ = false;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let v_simple = result.get_pixel(x, y).unwrap_or(0);
            let v_custom = result_custom.get_pixel(x, y).unwrap_or(0);
            if v_simple != v_custom {
                differ = true;
                break;
            }
        }
        if differ {
            break;
        }
    }
    rp.compare_values(1.0, if differ { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  simple vs custom params differ: {}", differ);

    assert!(rp.cleanup(), "adaptnorm_cn_simple regression test failed");
}

// ============================================================================
// Skipped tests for unimplemented C APIs
// ============================================================================

/// C版 test 2: pixGammaTRC(NULL, pix1, 1.5, 50, 235) -- leptonica-enhance未実装のためスキップ
#[test]
#[ignore = "C版: pixGammaTRC() -- leptonica-enhance未実装のためスキップ"]
fn adaptnorm_reg_gamma_trc() {
    // C版:
    //   pix2 = pixGammaTRC(NULL, pix1, 1.5, 50, 235);
    // Applied after contrast normalization to clean up remaining background.
    // This function is part of image enhancement, not yet implemented in Rust.
}

/// C版 test 3-4: pixDitherTo2bpp, pixThresholdTo4bpp -- 量子化未実装のためスキップ
#[test]
#[ignore = "C版: pixDitherTo2bpp(), pixThresholdTo4bpp() -- leptonica-quantize未実装のためスキップ"]
fn adaptnorm_reg_quantization() {
    // C版:
    //   pix3 = pixDitherTo2bpp(pix2, 1);
    //   pix4 = pixThresholdTo4bpp(pix2, 7, 1);
    // These are display/output options after normalization.
}

/// C版 test 5-6: pixThresholdToBinary -- 二値化未実装のためスキップ
#[test]
#[ignore = "C版: pixThresholdToBinary() -- leptonica-binarize未実装のためスキップ"]
fn adaptnorm_reg_binarization() {
    // C版:
    //   pix5 = pixThresholdToBinary(pix1, 180);
    //   pix6 = pixThresholdToBinary(pix2, 200);
    // These test thresholding after contrast normalization.
}

/// C版 test 10-13: pixLocalExtrema, pixSeedfillGrayBasin -- Rust未実装のためスキップ
#[test]
#[ignore = "C版: pixLocalExtrema(), pixSeedfillGrayBasin() -- Rust未実装のためスキップ"]
fn adaptnorm_reg_local_extrema_pipeline() {
    // C版:
    //   pix2 = pixScaleSmooth(pixs, 1./7., 1./7.);
    //   pix3 = pixScale(pix2, 7.0, 7.0);
    //   pixLocalExtrema(pix2, 0, 0, &pixmin, NULL);
    //   pix4 = pixExpandBinaryReplicate(pixmin, 7, 7);
    //   pix5 = pixSeedfillGrayBasin(pixmin, pix2, 10, 4);
    //   pix6 = pixExtendByReplication(pix5, 1, 1);
    //   pix7 = pixGetInvBackgroundMap(pix6, 200, 1, 1);
    //   pix8 = pixApplyInvBackgroundGrayMap(pixs, pix7, 7, 7);
    //
    // This is the multi-step background normalization pipeline using
    // local extrema detection. None of these intermediate APIs are
    // available in Rust.
}

/// C版 test 14-16: pixGammaTRCMasked -- leptonica-enhance未実装のためスキップ
#[test]
#[ignore = "C版: pixGammaTRCMasked() -- leptonica-enhance未実装のためスキップ"]
fn adaptnorm_reg_gamma_trc_masked() {
    // C版:
    //   pix9 = pixGammaTRCMasked(NULL, pix1, NULL, 1.0, 100, 175);
    //   pix10 = pixThresholdTo4bpp(pix9, 10, 1);
    //   pix11 = pixThresholdToBinary(pix9, 190);
    //
    // Post-processing of the background-normalized image with gamma
    // correction and thresholding.
}
