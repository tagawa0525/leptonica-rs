//! Binarization regression test
//!
//! C版: reference/leptonica/prog/binarize_reg.c
//! Otsu、適応しきい値、Sauvola等の二値化をテスト。

use leptonica_color::{
    AdaptiveThresholdOptions, adaptive_threshold, compute_otsu_threshold, dither_to_binary,
    sauvola_threshold, threshold_otsu, threshold_to_binary,
};
use leptonica_test::{RegParams, load_test_image};

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
