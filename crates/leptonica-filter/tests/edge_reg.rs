//! Edge detection regression test
//!
//! C版: reference/leptonica/prog/edge_reg.c
//! Sobel、Laplacianエッジ検出、シャープン、エンボスをテスト。

use leptonica_filter::{
    EdgeOrientation, emboss, laplacian_edge, sharpen, sobel_edge, unsharp_mask,
};
use leptonica_test::{RegParams, load_test_image};

#[test]
fn edge_reg() {
    let mut rp = RegParams::new("edge");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // --- Test 1: Sobel edge detection ---
    for &orient in &[EdgeOrientation::Horizontal, EdgeOrientation::Vertical] {
        let edges = sobel_edge(&pixs, orient).expect("sobel_edge");
        rp.compare_values(w as f64, edges.width() as f64, 0.0);
        rp.compare_values(h as f64, edges.height() as f64, 0.0);
        eprintln!(
            "  sobel({:?}): {}x{}",
            orient,
            edges.width(),
            edges.height()
        );
    }

    // --- Test 2: Laplacian edge detection ---
    let lap = laplacian_edge(&pixs).expect("laplacian_edge");
    rp.compare_values(w as f64, lap.width() as f64, 0.0);
    rp.compare_values(h as f64, lap.height() as f64, 0.0);
    eprintln!("  laplacian: {}x{}", lap.width(), lap.height());

    // --- Test 3: Sharpen ---
    let sharp = sharpen(&pixs).expect("sharpen");
    rp.compare_values(w as f64, sharp.width() as f64, 0.0);
    rp.compare_values(h as f64, sharp.height() as f64, 0.0);

    // --- Test 4: Unsharp mask ---
    let unsharp = unsharp_mask(&pixs, 2, 1.5).expect("unsharp_mask");
    rp.compare_values(w as f64, unsharp.width() as f64, 0.0);
    rp.compare_values(h as f64, unsharp.height() as f64, 0.0);

    // --- Test 5: Emboss ---
    let emb = emboss(&pixs).expect("emboss");
    rp.compare_values(w as f64, emb.width() as f64, 0.0);
    rp.compare_values(h as f64, emb.height() as f64, 0.0);

    // --- Test 6: Edge detection should produce non-zero output ---
    let edge_fg = lap.count_pixels();
    rp.compare_values(1.0, if edge_fg > 0 { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  laplacian nonzero pixels: {}", edge_fg);

    // --- Test 7: Edge functions require 8bpp, test error on 32bpp ---
    let pix32 = load_test_image("weasel32.png").expect("load 32bpp");
    let result32 = sobel_edge(&pix32, EdgeOrientation::Horizontal);
    rp.compare_values(1.0, if result32.is_err() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "edge regression test failed");
}
