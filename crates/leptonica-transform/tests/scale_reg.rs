//! Scale regression test
//!
//! C版: reference/leptonica/prog/scale_reg.c
//! 各種スケーリング操作をテスト。

use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{ScaleMethod, scale, scale_by_sampling, scale_to_size};

#[test]
fn scale_reg() {
    let mut rp = RegParams::new("scale");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // --- Test 1: Scale up 2x ---
    let up2 = scale(&pixs, 2.0, 2.0, ScaleMethod::Linear).expect("scale 2x");
    rp.compare_values((w * 2) as f64, up2.width() as f64, 1.0);
    rp.compare_values((h * 2) as f64, up2.height() as f64, 1.0);
    eprintln!("  scale 2x: {}x{}", up2.width(), up2.height());

    // --- Test 2: Scale down 0.5x ---
    let down2 = scale(&pixs, 0.5, 0.5, ScaleMethod::Linear).expect("scale 0.5x");
    rp.compare_values((w / 2) as f64, down2.width() as f64, 1.0);
    rp.compare_values((h / 2) as f64, down2.height() as f64, 1.0);
    eprintln!("  scale 0.5x: {}x{}", down2.width(), down2.height());

    // --- Test 3: Scale to specific size ---
    let target_w = 200u32;
    let target_h = 150u32;
    let sized = scale_to_size(&pixs, target_w, target_h).expect("scale_to_size");
    rp.compare_values(target_w as f64, sized.width() as f64, 0.0);
    rp.compare_values(target_h as f64, sized.height() as f64, 0.0);
    eprintln!(
        "  scale_to_size(200,150): {}x{}",
        sized.width(),
        sized.height()
    );

    // --- Test 4: Scale by sampling ---
    let sampled = scale_by_sampling(&pixs, 2.0, 2.0).expect("scale_by_sampling 2x");
    rp.compare_values((w * 2) as f64, sampled.width() as f64, 1.0);
    rp.compare_values((h * 2) as f64, sampled.height() as f64, 1.0);

    // --- Test 5: Scale 1.0 should preserve dimensions ---
    let s1 = scale(&pixs, 1.0, 1.0, ScaleMethod::Linear).expect("scale 1x");
    rp.compare_values(w as f64, s1.width() as f64, 0.0);
    rp.compare_values(h as f64, s1.height() as f64, 0.0);

    // --- Test 6: Anisotropic scaling ---
    let aniso = scale(&pixs, 2.0, 0.5, ScaleMethod::Linear).expect("aniso scale");
    rp.compare_values((w * 2) as f64, aniso.width() as f64, 1.0);
    rp.compare_values((h / 2) as f64, aniso.height() as f64, 1.0);
    eprintln!(
        "  aniso scale(2.0, 0.5): {}x{}",
        aniso.width(),
        aniso.height()
    );

    // --- Test 7: Scale with different methods ---
    for method in [ScaleMethod::Linear, ScaleMethod::Sampling] {
        let s = scale(&pixs, 1.5, 1.5, method).expect("scale method");
        rp.compare_values(
            1.0,
            if s.width() > 0 && s.height() > 0 {
                1.0
            } else {
                0.0
            },
            0.0,
        );
        eprintln!("  scale {:?} 1.5x: {}x{}", method, s.width(), s.height());
    }

    // --- Test 8: Scale with binary image ---
    let pixb = load_test_image("feyn-fract.tif").expect("load binary");
    let sb = scale(&pixb, 2.0, 2.0, ScaleMethod::Sampling).expect("scale binary");
    rp.compare_values((pixb.width() * 2) as f64, sb.width() as f64, 1.0);
    rp.compare_values((pixb.height() * 2) as f64, sb.height() as f64, 1.0);

    assert!(rp.cleanup(), "scale regression test failed");
}
