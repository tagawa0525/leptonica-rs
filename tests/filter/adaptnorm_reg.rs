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
//!   - pixBackgroundNormFlex(pixs, 7, 7, 1, 1, 10)   -> background_norm_flex()
//!
//! C checkpoint indices (0-17) are matched as closely as possible.
//! Checkpoints 7 and 17 (tiled display) and 11-12 (unimplemented seedfill basin)
//! are skipped or stubbed.
//!
//! C APIs not yet implemented (skipped):
//!   - pixSeedfillGrayBasin, pixExpandBinaryReplicate

use crate::common::{RegParams, load_test_image};
use leptonica::Pix;
use leptonica::color::{dither_to_2bpp, threshold_to_4bpp, threshold_to_binary};
use leptonica::filter::{
    BackgroundNormOptions, ContrastNormOptions, FlexNormOptions, apply_inv_background_gray_map,
    background_norm, background_norm_flex, background_norm_simple, contrast_norm,
    contrast_norm_simple, extend_by_replication, gamma_trc_masked, gamma_trc_pix,
    get_inv_background_map,
};
use leptonica::io::ImageFormat;
use leptonica::region::local_extrema;

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
// C checkpoints 0-7
// ============================================================================

/// Test: Contrast normalization on low-contrast text (C checkpoints 0-6).
///
/// Matches C adaptnorm_reg.c steps:
///   0: pixs (lighttext.jpg)
///   1: pix1 = pixContrastNorm(NULL, pixs, 10, 10, 40, 2, 2)
///   2: pix2 = pixGammaTRC(NULL, pix1, 1.5, 50, 235)
///   3: pix3 = pixDitherTo2bpp(pix2, 1)
///   4: pix4 = pixThresholdTo4bpp(pix2, 7, 1)
///   5: pix5 = pixThresholdToBinary(pix1, 180)
///   6: pix6 = pixThresholdToBinary(pix2, 200)
///   7: tiled display -- skipped (no tiling API)
#[test]
fn adaptnorm_reg_contrast_norm_lighttext() {
    let mut rp = RegParams::new("adaptnorm_contrast");

    let pixs = load_test_image("lighttext.jpg").expect("load lighttext.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // C checkpoint 0: original lighttext.jpg
    rp.write_pix_and_check(&pixs, ImageFormat::Jpeg)
        .expect("write_pix_and_check 0 (pixs)");

    let options = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 40,
        smooth_x: 2,
        smooth_y: 2,
    };
    // C checkpoint 1: pixContrastNorm result
    let pix1 = contrast_norm(&pixs, &options).expect("contrast_norm(10, 10, 40, 2, 2)");
    rp.compare_values(w as f64, pix1.width() as f64, 0.0);
    rp.compare_values(h as f64, pix1.height() as f64, 0.0);
    rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix1, ImageFormat::Jpeg)
        .expect("write_pix_and_check 1 (contrast_norm)");

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

    // C checkpoint 2: pix2 = pixGammaTRC(NULL, pix1, 1.5, 50, 235)
    let pix2 = gamma_trc_pix(&pix1, 1.5, 50, 235).expect("gamma_trc_pix(1.5, 50, 235)");
    rp.compare_values(w as f64, pix2.width() as f64, 0.0);
    rp.compare_values(h as f64, pix2.height() as f64, 0.0);
    rp.write_pix_and_check(&pix2, ImageFormat::Jpeg)
        .expect("write_pix_and_check 2 (gamma_trc)");

    // C checkpoint 3: pix3 = pixDitherTo2bpp(pix2, 1)
    let pix3 = dither_to_2bpp(&pix2).expect("dither_to_2bpp");
    rp.compare_values(2.0, pix3.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix3, ImageFormat::Png)
        .expect("write_pix_and_check 3 (dither_to_2bpp)");

    // C checkpoint 4: pix4 = pixThresholdTo4bpp(pix2, 7, 1)
    let pix4 = threshold_to_4bpp(&pix2, 7, true).expect("threshold_to_4bpp(7, with_cmap)");
    rp.compare_values(4.0, pix4.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix4, ImageFormat::Png)
        .expect("write_pix_and_check 4 (threshold_to_4bpp)");

    // C checkpoint 5: pix5 = pixThresholdToBinary(pix1, 180)
    let pix5 = threshold_to_binary(&pix1, 180).expect("threshold_to_binary(pix1, 180)");
    rp.compare_values(1.0, pix5.depth().bits() as f64, 0.0);
    rp.compare_values(w as f64, pix5.width() as f64, 0.0);
    rp.compare_values(h as f64, pix5.height() as f64, 0.0);
    rp.write_pix_and_check(&pix5, ImageFormat::Png)
        .expect("write_pix_and_check 5 (threshold_binary pix1)");

    // C checkpoint 6: pix6 = pixThresholdToBinary(pix2, 200)
    let pix6 = threshold_to_binary(&pix2, 200).expect("threshold_to_binary(pix2, 200)");
    rp.compare_values(1.0, pix6.depth().bits() as f64, 0.0);
    rp.compare_values(w as f64, pix6.width() as f64, 0.0);
    rp.compare_values(h as f64, pix6.height() as f64, 0.0);
    rp.write_pix_and_check(&pix6, ImageFormat::Png)
        .expect("write_pix_and_check 6 (threshold_binary pix2)");

    // C checkpoint 7: tiled display -- skipped (no tiling API)

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
    rp.write_pix_and_check(&result_c, ImageFormat::Jpeg)
        .expect("write_pix_and_check contrast_c");

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
    rp.write_pix_and_check(&result_small, ImageFormat::Jpeg)
        .expect("write_pix_and_check contrast_small");

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
// C checkpoints 8-17
// ============================================================================

/// Test: Background normalization (flex) on rapidly varying background.
///
/// Matches C adaptnorm_reg.c steps:
///   8:  pixs (w91frag.jpg)
///   9:  pix1 = pixBackgroundNormFlex(pixs, 7, 7, 1, 1, 10)
///   10: pix3 = pixScale(pix2, 7.0, 7.0) -- scale_smooth result upscaled
///   11: pix4 = pixExpandBinaryReplicate(pixmin, 7, 7) -- skipped (no impl)
///   12: pix6 = pixExtendByReplication after SeedfillGrayBasin -- skipped
///   13: pix8 = pixApplyInvBackgroundGrayMap(pixs, pix7, 7, 7)
///   14: pix9 = pixGammaTRCMasked(NULL, pix1, NULL, 1.0, 100, 175)
///   15: pix10 = pixThresholdTo4bpp(pix9, 10, 1)
///   16: pix11 = pixThresholdToBinary(pix9, 190)
///   17: tiled display -- skipped
#[test]
fn adaptnorm_reg_background_norm_w91frag() {
    let mut rp = RegParams::new("adaptnorm_bg");

    let pixs = load_test_image("w91frag.jpg").expect("load w91frag.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // C checkpoint 8: original w91frag.jpg
    rp.write_pix_and_check(&pixs, ImageFormat::Jpeg)
        .expect("write_pix_and_check 8 (pixs w91frag)");

    // C checkpoint 9: pix1 = pixBackgroundNormFlex(pixs, 7, 7, 1, 1, 10)
    // The C signature: pixBackgroundNormFlex(pixs, sx=7, sy=7, smoothx=1, smoothy=1, delta=10)
    // Our FlexNormOptions: tile_width/height=7, smooth_x/y=1, delta (must be 0 -- not supported)
    // delta=10 (basin filling) is not yet implemented; we use delta=0 as closest approximation
    let flex_opts = FlexNormOptions {
        tile_width: 7,
        tile_height: 7,
        smooth_x: 1,
        smooth_y: 1,
        delta: 0, // delta=10 (basin fill) not yet implemented
    };
    let pix1 = background_norm_flex(&pixs, &flex_opts).expect("background_norm_flex(7,7,1,1)");
    rp.compare_values(w as f64, pix1.width() as f64, 0.0);
    rp.compare_values(h as f64, pix1.height() as f64, 0.0);
    rp.write_pix_and_check(&pix1, ImageFormat::Jpeg)
        .expect("write_pix_and_check 9 (background_norm_flex)");

    // After normalization, mean should shift toward bg_val (200)
    let norm_mean = sample_mean(&pix1);
    let mean_ok = norm_mean > 100.0;
    rp.compare_values(1.0, if mean_ok { 1.0 } else { 0.0 }, 0.0);

    // C checkpoint 10: C does scale_smooth(pixs, 1/7, 1/7) then scale up 7x.
    // We reproduce: extend_by_replication of the scaled version (the background map).
    // The low-res background map extended by 1 pixel is what C checkpoint 10 shows.
    // We write the extend_by_replication result as our checkpoint 10 approximation.
    {
        use leptonica::transform::scale::scale_smooth;
        let scaled = scale_smooth(&pixs, 1.0 / 7.0, 1.0 / 7.0).expect("scale_smooth(1/7, 1/7)");
        let extended = extend_by_replication(&scaled, 1, 1).expect("extend_by_replication");
        rp.write_pix_and_check(&extended, ImageFormat::Jpeg)
            .expect("write_pix_and_check 10 (scale_smooth extended)");
    }

    // C checkpoint 11: pixExpandBinaryReplicate(pixmin, 7, 7)
    // Depends on pixLocalExtrema + pixExpandBinaryReplicate, both unimplemented in this context.
    // We produce the local_extrema min mask (1bpp) as the nearest available checkpoint.
    {
        use leptonica::transform::scale::scale_smooth;
        let scaled =
            scale_smooth(&pixs, 1.0 / 7.0, 1.0 / 7.0).expect("scale_smooth(1/7, 1/7) for extrema");
        // C uses 0, 0 which defaults to 15, 0 internally
        let (pixmin, _pixmax) = local_extrema(&scaled, 15, 0).expect("local_extrema(15, 0)");
        rp.compare_values(1.0, pixmin.depth().bits() as f64, 0.0);
        // Write the min-extrema mask (approximation of C checkpoint 11)
        rp.write_pix_and_check(&pixmin, ImageFormat::Png)
            .expect("write_pix_and_check 11 (local_extrema min mask)");
    }

    // C checkpoint 12: pix6 = pixExtendByReplication(pix5, 1, 1)
    // where pix5 = pixSeedfillGrayBasin(pixmin, pix2, 10, 4) -- NOT IMPLEMENTED.
    // We write the extend_by_replication of the background map as the closest alternative.
    {
        use leptonica::transform::scale::scale_smooth;
        let scaled =
            scale_smooth(&pixs, 1.0 / 7.0, 1.0 / 7.0).expect("scale_smooth(1/7, 1/7) for ext");
        let extended =
            extend_by_replication(&scaled, 1, 1).expect("extend_by_replication checkpoint 12");
        rp.write_pix_and_check(&extended, ImageFormat::Jpeg)
            .expect("write_pix_and_check 12 (extend_by_replication approx)");
    }

    // C checkpoint 13: pix8 = pixApplyInvBackgroundGrayMap(pixs, pix7, 7, 7)
    // where pix7 = pixGetInvBackgroundMap(pix6, 200, 1, 1)
    // We replicate this manually using our public API.
    {
        use leptonica::transform::scale::scale_smooth;
        let scaled =
            scale_smooth(&pixs, 1.0 / 7.0, 1.0 / 7.0).expect("scale_smooth(1/7, 1/7) for inv map");
        let extended =
            extend_by_replication(&scaled, 1, 1).expect("extend_by_replication for inv map");
        let inv_map = get_inv_background_map(&extended, 200, 1, 1)
            .expect("get_inv_background_map(200, 1, 1)");
        let pix8 = apply_inv_background_gray_map(&pixs, &inv_map, 7, 7)
            .expect("apply_inv_background_gray_map(7, 7)");
        rp.compare_values(w as f64, pix8.width() as f64, 0.0);
        rp.compare_values(h as f64, pix8.height() as f64, 0.0);
        rp.write_pix_and_check(&pix8, ImageFormat::Jpeg)
            .expect("write_pix_and_check 13 (apply_inv_background_gray_map)");
    }

    // C checkpoint 14: pix9 = pixGammaTRCMasked(NULL, pix1, NULL, 1.0, 100, 175)
    let pix9 =
        gamma_trc_masked(&pix1, None, 1.0, 100, 175).expect("gamma_trc_masked(1.0, 100, 175)");
    rp.compare_values(w as f64, pix9.width() as f64, 0.0);
    rp.compare_values(h as f64, pix9.height() as f64, 0.0);
    rp.compare_values(8.0, pix9.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix9, ImageFormat::Jpeg)
        .expect("write_pix_and_check 14 (gamma_trc_masked)");

    // C checkpoint 15: pix10 = pixThresholdTo4bpp(pix9, 10, 1)
    let pix10 = threshold_to_4bpp(&pix9, 10, true).expect("threshold_to_4bpp(10, with_cmap)");
    rp.compare_values(4.0, pix10.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix10, ImageFormat::Jpeg)
        .expect("write_pix_and_check 15 (threshold_to_4bpp)");

    // C checkpoint 16: pix11 = pixThresholdToBinary(pix9, 190)
    let pix11 = threshold_to_binary(&pix9, 190).expect("threshold_to_binary(190)");
    rp.compare_values(1.0, pix11.depth().bits() as f64, 0.0);
    rp.compare_values(w as f64, pix11.width() as f64, 0.0);
    rp.compare_values(h as f64, pix11.height() as f64, 0.0);
    rp.write_pix_and_check(&pix11, ImageFormat::Png)
        .expect("write_pix_and_check 16 (threshold_to_binary)");

    // C checkpoint 17: tiled display -- skipped (no tiling API)

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
    rp.write_pix_and_check(&result_small, ImageFormat::Jpeg)
        .expect("write_pix_and_check small tiles");

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
    rp.write_pix_and_check(&result_large, ImageFormat::Jpeg)
        .expect("write_pix_and_check large tiles");

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
    rp.write_pix_and_check(&pix_contrast, ImageFormat::Jpeg)
        .expect("write_pix_and_check pipeline contrast_norm");

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
    rp.write_pix_and_check(&pix_final, ImageFormat::Jpeg)
        .expect("write_pix_and_check pipeline background_norm");

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
    rp.write_pix_and_check(&result, ImageFormat::Jpeg)
        .expect("write_pix_and_check background_norm_simple");

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
    rp.write_pix_and_check(&result, ImageFormat::Jpeg)
        .expect("write_pix_and_check contrast_norm_simple");

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
// Additional filter tests for supporting functions
// ============================================================================

/// C: pixGammaTRC() -- gamma correction on 8bpp image
#[test]
fn adaptnorm_reg_gamma_trc() {
    let mut rp = RegParams::new("adaptnorm_gamma_trc");

    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");
    let gray = pixs.convert_to_8().expect("convert to 8bpp");
    let w = gray.width();
    let h = gray.height();

    let result = gamma_trc_pix(&gray, 1.5, 30, 230).expect("gamma_trc_pix");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(8.0, result.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Jpeg)
        .expect("write_pix_and_check gamma_trc");

    assert!(rp.cleanup(), "adaptnorm_gamma_trc regression test failed");
}

/// C: pixDitherTo2bpp(), pixThresholdTo4bpp() -- quantization
#[test]
fn adaptnorm_reg_quantization() {
    let mut rp = RegParams::new("adaptnorm_quantization");

    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");
    let gray = pixs.convert_to_8().expect("convert to 8bpp");

    let pix2 = dither_to_2bpp(&gray).expect("dither_to_2bpp");
    rp.compare_values(2.0, pix2.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix2, ImageFormat::Png)
        .expect("write_pix_and_check dither_to_2bpp");

    let pix4 = threshold_to_4bpp(&gray, 16, false).expect("threshold_to_4bpp");
    rp.compare_values(4.0, pix4.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix4, ImageFormat::Png)
        .expect("write_pix_and_check threshold_to_4bpp");

    assert!(
        rp.cleanup(),
        "adaptnorm_quantization regression test failed"
    );
}

/// C: pixThresholdToBinary() -- binarization at threshold 128
#[test]
fn adaptnorm_reg_binarization() {
    let mut rp = RegParams::new("adaptnorm_binarization");

    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");
    let gray = pixs.convert_to_8().expect("convert to 8bpp");

    let pix1 = threshold_to_binary(&gray, 128).expect("threshold_to_binary");
    rp.compare_values(1.0, pix1.depth().bits() as f64, 0.0);
    rp.compare_values(gray.width() as f64, pix1.width() as f64, 0.0);
    rp.compare_values(gray.height() as f64, pix1.height() as f64, 0.0);
    rp.write_pix_and_check(&pix1, ImageFormat::Png)
        .expect("write_pix_and_check threshold_to_binary");

    assert!(
        rp.cleanup(),
        "adaptnorm_binarization regression test failed"
    );
}

/// C: pixLocalExtrema() -- local min/max masks
#[test]
fn adaptnorm_reg_local_extrema_pipeline() {
    let mut rp = RegParams::new("adaptnorm_local_extrema");

    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");
    let gray = pixs.convert_to_8().expect("convert to 8bpp");
    let w = gray.width();
    let h = gray.height();

    let (min_mask, max_mask) = local_extrema(&gray, 3, 5).expect("local_extrema");
    rp.compare_values(w as f64, min_mask.width() as f64, 0.0);
    rp.compare_values(h as f64, min_mask.height() as f64, 0.0);
    rp.compare_values(1.0, min_mask.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&min_mask, ImageFormat::Png)
        .expect("write_pix_and_check local_extrema min");

    rp.compare_values(w as f64, max_mask.width() as f64, 0.0);
    rp.compare_values(h as f64, max_mask.height() as f64, 0.0);
    rp.compare_values(1.0, max_mask.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&max_mask, ImageFormat::Png)
        .expect("write_pix_and_check local_extrema max");

    assert!(
        rp.cleanup(),
        "adaptnorm_local_extrema regression test failed"
    );
}

/// C: pixGammaTRCMasked() -- masked gamma correction
#[test]
fn adaptnorm_reg_gamma_trc_masked() {
    let mut rp = RegParams::new("adaptnorm_gamma_trc_masked");

    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");
    let gray = pixs.convert_to_8().expect("convert to 8bpp");
    let w = gray.width();
    let h = gray.height();

    let result = gamma_trc_masked(&gray, None, 1.5, 30, 230).expect("gamma_trc_masked");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(8.0, result.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Jpeg)
        .expect("write_pix_and_check gamma_trc_masked");

    assert!(
        rp.cleanup(),
        "adaptnorm_gamma_trc_masked regression test failed"
    );
}

/// Stub: pixSeedfillGrayBasin -- not yet implemented
#[test]
#[ignore = "pixSeedfillGrayBasin not implemented"]
fn adaptnorm_reg_seedfill_gray_basin() {
    // C checkpoint 12 depends on pixSeedfillGrayBasin(pixmin, pix2, 10, 4).
    // This function fills gray-level basins seeded at local minima.
    // When implemented, the test should:
    //   1. Load w91frag.jpg
    //   2. scale_smooth by 1/7
    //   3. local_extrema to get pixmin
    //   4. seedfill_gray_basin(pixmin, scaled, 10, ConnectivityType::FourWay)
    //   5. extend_by_replication(result, 1, 1)
    //   6. write_pix_and_check(extended, ImageFormat::Jpeg)
}
