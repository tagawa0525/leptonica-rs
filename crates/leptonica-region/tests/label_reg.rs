//! Labeling regression test
//!
//! C版: reference/leptonica/prog/label_reg.c
//! 連結成分ラベリング、成分数カウント、成分境界をテスト。

use leptonica_core::PixelDepth;
use leptonica_region::{ConnectivityType, find_connected_components, label_connected_components};
use leptonica_test::{RegParams, load_test_image};

#[test]
fn label_reg() {
    let mut rp = RegParams::new("label");

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image: {}x{}", w, h);

    // --- Test 1: 4-connected labeling ---
    eprintln!("=== 4-connected labeling ===");
    let labeled4 =
        label_connected_components(&pixs, ConnectivityType::FourWay).expect("label 4-connected");
    rp.compare_values(w as f64, labeled4.width() as f64, 0.0);
    rp.compare_values(h as f64, labeled4.height() as f64, 0.0);
    eprintln!(
        "  labeled 4-way: {}x{} d={}",
        labeled4.width(),
        labeled4.height(),
        labeled4.depth().bits()
    );

    // --- Test 2: 8-connected labeling ---
    eprintln!("=== 8-connected labeling ===");
    let labeled8 =
        label_connected_components(&pixs, ConnectivityType::EightWay).expect("label 8-connected");
    rp.compare_values(w as f64, labeled8.width() as f64, 0.0);
    rp.compare_values(h as f64, labeled8.height() as f64, 0.0);

    // --- Test 3: Component counting ---
    eprintln!("=== Component counting ===");
    let comps4 =
        find_connected_components(&pixs, ConnectivityType::FourWay).expect("find components 4-way");
    let comps8 = find_connected_components(&pixs, ConnectivityType::EightWay)
        .expect("find components 8-way");

    let n4 = comps4.len();
    let n8 = comps8.len();
    eprintln!("  4-connected components: {}", n4);
    eprintln!("  8-connected components: {}", n8);

    // Should have components
    rp.compare_values(1.0, if n4 > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if n8 > 0 { 1.0 } else { 0.0 }, 0.0);

    // 4-connected should have >= 8-connected components
    rp.compare_values(1.0, if n4 >= n8 { 1.0 } else { 0.0 }, 0.0);

    // --- Test 4: Component properties ---
    eprintln!("=== Component properties ===");
    for (i, comp) in comps8.iter().take(5).enumerate() {
        let area = comp.pixel_count;
        let bounds = &comp.bounds;
        rp.compare_values(1.0, if area > 0 { 1.0 } else { 0.0 }, 0.0);
        rp.compare_values(
            1.0,
            if bounds.w > 0 && bounds.h > 0 {
                1.0
            } else {
                0.0
            },
            0.0,
        );
        eprintln!(
            "  comp[{}]: area={}, bounds=({},{},{},{})",
            i, area, bounds.x, bounds.y, bounds.w, bounds.h
        );
    }

    // --- Test 5: feyn-fract.tif has known component count ---
    let pixf = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let comps_fract = find_connected_components(&pixf, ConnectivityType::EightWay)
        .expect("find components feyn-fract");
    eprintln!("  feyn-fract 8-way components: {}", comps_fract.len());
    rp.compare_values(1.0, if comps_fract.len() > 100 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "label regression test failed");
}
