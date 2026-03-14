//! Binarization regression test
//!
//! C version: prog/binarize_reg.c
//! Tests Otsu, adaptive threshold, Sauvola and other binarization methods.
//!
//! Expanded in Phase 5 to add tiled Sauvola, sauvola_on_contrast_norm,
//! and thresh_on_double_norm.

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::color::{
    AdaptiveThresholdOptions, adaptive_threshold, compute_otsu_threshold, dither_to_binary,
    sauvola_binarize_tiled, sauvola_on_contrast_norm, sauvola_threshold, thresh_on_double_norm,
    threshold_otsu, threshold_to_binary,
};
use leptonica::io::ImageFormat;

#[test]
fn binarize_reg() {
    let mut rp = RegParams::new("binarize");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // --- Test 1: Fixed threshold ---
    eprintln!("=== Fixed threshold ===");
    let bin128 = threshold_to_binary(&pixs, 128).expect("threshold 128");
    rp.compare_values(w as f64, bin128.width() as f64, 0.0);
    rp.compare_values(h as f64, bin128.height() as f64, 0.0);
    rp.compare_values(1.0, bin128.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&bin128, ImageFormat::Tiff)
        .expect("write bin128");
    eprintln!(
        "  threshold(128): {}x{} d={}",
        bin128.width(),
        bin128.height(),
        bin128.depth().bits()
    );

    // Different thresholds should produce valid binary images
    let bin64 = threshold_to_binary(&pixs, 64).expect("threshold 64");
    let bin192 = threshold_to_binary(&pixs, 192).expect("threshold 192");
    rp.compare_values(1.0, bin64.depth().bits() as f64, 0.0);
    rp.compare_values(1.0, bin192.depth().bits() as f64, 0.0);

    // Verify dimensions are preserved
    rp.compare_values(w as f64, bin64.width() as f64, 0.0);
    rp.compare_values(w as f64, bin192.width() as f64, 0.0);

    // --- Test 2: Otsu threshold ---
    eprintln!("=== Otsu threshold ===");
    let otsu_thresh = compute_otsu_threshold(&pixs).expect("compute_otsu");
    rp.compare_values(
        1.0,
        if otsu_thresh > 0 && otsu_thresh < 255 {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    eprintln!("  Otsu threshold: {}", otsu_thresh);

    let otsu_bin = threshold_otsu(&pixs).expect("threshold_otsu");
    rp.compare_values(w as f64, otsu_bin.width() as f64, 0.0);
    rp.compare_values(h as f64, otsu_bin.height() as f64, 0.0);
    rp.compare_values(1.0, otsu_bin.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&otsu_bin, ImageFormat::Tiff)
        .expect("write otsu_bin");

    // --- Test 3: Adaptive threshold ---
    eprintln!("=== Adaptive threshold ===");
    let options = AdaptiveThresholdOptions::default();
    let adaptive = adaptive_threshold(&pixs, &options).expect("adaptive_threshold");
    rp.compare_values(w as f64, adaptive.width() as f64, 0.0);
    rp.compare_values(h as f64, adaptive.height() as f64, 0.0);
    rp.compare_values(1.0, adaptive.depth().bits() as f64, 0.0);
    eprintln!(
        "  adaptive: {}x{} d={}",
        adaptive.width(),
        adaptive.height(),
        adaptive.depth().bits()
    );

    // --- Test 4: Sauvola threshold ---
    eprintln!("=== Sauvola threshold ===");
    let sauvola = sauvola_threshold(&pixs, 15, 0.34, 128.0).expect("sauvola_threshold");
    rp.compare_values(w as f64, sauvola.width() as f64, 0.0);
    rp.compare_values(h as f64, sauvola.height() as f64, 0.0);
    rp.compare_values(1.0, sauvola.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&sauvola, ImageFormat::Tiff)
        .expect("write sauvola");
    eprintln!(
        "  sauvola: {}x{} d={}",
        sauvola.width(),
        sauvola.height(),
        sauvola.depth().bits()
    );

    // --- Test 5: Floyd-Steinberg dithering ---
    eprintln!("=== Dithering ===");
    let dithered = dither_to_binary(&pixs).expect("dither_to_binary");
    rp.compare_values(w as f64, dithered.width() as f64, 0.0);
    rp.compare_values(h as f64, dithered.height() as f64, 0.0);
    rp.compare_values(1.0, dithered.depth().bits() as f64, 0.0);

    // --- Test 6: All binarization methods should produce binary output ---
    for (name, pix) in [
        ("fixed", &bin128),
        ("otsu", &otsu_bin),
        ("adaptive", &adaptive),
        ("sauvola", &sauvola),
        ("dither", &dithered),
    ] {
        rp.compare_values(1.0, pix.depth().bits() as f64, 0.0);
        eprintln!("  {} depth: {}", name, pix.depth().bits());
    }

    assert!(rp.cleanup(), "binarize regression test failed");
}

/// Test tiled Sauvola binarization at various nx×ny grid sizes.
///
/// C: pixSauvolaBinarizeTiled at different tile grids (1×1, 2×2, 3×3, 4×6).
/// Compares tiled vs non-tiled results.
#[test]
fn binarize_reg_tiled_sauvola() {
    let mut rp = RegParams::new("binarize_tiled");

    // Use lucasta.150.jpg (grayscale document image)
    let pixs = load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let pixs = if pixs.depth() == PixelDepth::Bit8 {
        pixs
    } else {
        pixs.convert_to_8().expect("convert to 8bpp")
    };

    // --- Non-tiled (1x1) ---
    let (thresh1, bin1) = sauvola_binarize_tiled(&pixs, 7, 0.34, 1, 1).expect("sauvola 1x1");
    assert_eq!(bin1.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&thresh1, ImageFormat::Png)
        .expect("write thresh 1x1");
    rp.write_pix_and_check(&bin1, ImageFormat::Tiff)
        .expect("write bin 1x1");

    // --- 2x2 tiled ---
    let (thresh2, bin2) = sauvola_binarize_tiled(&pixs, 7, 0.34, 2, 2).expect("sauvola 2x2");
    assert_eq!(bin2.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&thresh2, ImageFormat::Png)
        .expect("write thresh 2x2");
    rp.write_pix_and_check(&bin2, ImageFormat::Tiff)
        .expect("write bin 2x2");

    // --- 3x3 tiled ---
    let (thresh3, bin3) = sauvola_binarize_tiled(&pixs, 7, 0.34, 3, 3).expect("sauvola 3x3");
    assert_eq!(bin3.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&bin3, ImageFormat::Tiff)
        .expect("write bin 3x3");
    drop(thresh3);

    // --- 4x6 tiled (C: uses asymmetric tiling) ---
    let (thresh4, bin4) = sauvola_binarize_tiled(&pixs, 7, 0.34, 4, 6).expect("sauvola 4x6");
    assert_eq!(bin4.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&bin4, ImageFormat::Tiff)
        .expect("write bin 4x6");
    drop(thresh4);

    // Tiled and non-tiled should produce images of same size
    rp.compare_values(bin1.width() as f64, bin2.width() as f64, 0.0);
    rp.compare_values(bin1.height() as f64, bin2.height() as f64, 0.0);
    rp.compare_values(bin1.width() as f64, bin3.width() as f64, 0.0);
    rp.compare_values(bin1.width() as f64, bin4.width() as f64, 0.0);

    assert!(rp.cleanup(), "binarize tiled_sauvola test failed");
}

/// Test Sauvola on contrast-normalized image.
///
/// C: pixSauvolaOnContrastNorm(pix, mindiff=15)
/// Applies contrast normalization first, then Sauvola binarization.
#[test]
fn binarize_reg_sauvola_on_contrast() {
    let mut rp = RegParams::new("binarize_contrast_sauvola");

    let pixs = load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let pixs = if pixs.depth() == PixelDepth::Bit8 {
        pixs
    } else {
        pixs.convert_to_8().expect("convert to 8bpp")
    };

    // Test with different mindiff values (C: uses 15)
    let bin15 = sauvola_on_contrast_norm(&pixs, 15).expect("sauvola_on_contrast_norm mindiff=15");
    assert_eq!(bin15.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&bin15, ImageFormat::Tiff)
        .expect("write sauvola_on_contrast mindiff=15");

    let bin30 = sauvola_on_contrast_norm(&pixs, 30).expect("sauvola_on_contrast_norm mindiff=30");
    assert_eq!(bin30.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&bin30, ImageFormat::Tiff)
        .expect("write sauvola_on_contrast mindiff=30");

    // Both should preserve image dimensions
    rp.compare_values(pixs.width() as f64, bin15.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, bin15.height() as f64, 0.0);
    rp.compare_values(pixs.width() as f64, bin30.width() as f64, 0.0);

    assert!(rp.cleanup(), "binarize sauvola_on_contrast test failed");
}

/// Test threshold on double normalization.
///
/// C: pixThreshOnDoubleNorm(pix, mindiff=15)
/// Applies background normalization + contrast normalization + fixed threshold.
#[test]
fn binarize_reg_thresh_on_double_norm() {
    let mut rp = RegParams::new("binarize_double_norm");

    let pixs = load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");

    let bin = thresh_on_double_norm(&pixs, 15).expect("thresh_on_double_norm");
    assert_eq!(bin.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&bin, ImageFormat::Tiff)
        .expect("write thresh_on_double_norm");

    rp.compare_values(pixs.width() as f64, bin.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, bin.height() as f64, 0.0);

    // Compare with dreyfus8 (original test image)
    let dreyfus = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let bin_dreyfus = thresh_on_double_norm(&dreyfus, 10).expect("thresh_on_double_norm dreyfus");
    assert_eq!(bin_dreyfus.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&bin_dreyfus, ImageFormat::Tiff)
        .expect("write thresh_on_double_norm dreyfus");

    assert!(rp.cleanup(), "binarize thresh_on_double_norm test failed");
}
