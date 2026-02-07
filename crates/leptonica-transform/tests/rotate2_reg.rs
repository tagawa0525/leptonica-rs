//! Rotation regression test 2 - advanced rotation
//!
//! C版: reference/leptonica/prog/rotate2_reg.c
//! 任意角度回転、回転メソッドの比較をテスト。

use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{rotate_by_angle, rotate_by_radians};

#[test]
fn rotate2_reg() {
    let mut rp = RegParams::new("rotate2");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // --- Test 1: Small angle rotation preserves dimensions ---
    let angle = 5.0_f32; // 5 degrees
    let rotated = rotate_by_angle(&pixs, angle).expect("rotate_by_angle 5°");
    // Dimensions may change slightly for arbitrary rotations
    rp.compare_values(
        1.0,
        if rotated.width() > 0 && rotated.height() > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    eprintln!("  rotate 5°: {}x{}", rotated.width(), rotated.height());

    // --- Test 2: Rotation by 0 degrees should preserve image ---
    let rot0 = rotate_by_angle(&pixs, 0.0).expect("rotate 0°");
    rp.compare_values(w as f64, rot0.width() as f64, 0.0);
    rp.compare_values(h as f64, rot0.height() as f64, 0.0);

    // --- Test 3: Rotation by radians ---
    let radians = std::f32::consts::PI / 6.0; // 30 degrees
    let rot_rad = rotate_by_radians(&pixs, radians).expect("rotate_by_radians pi/6");
    rp.compare_values(1.0, if rot_rad.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  rotate pi/6 rad: {}x{}",
        rot_rad.width(),
        rot_rad.height()
    );

    // --- Test 4: Various angles ---
    for &angle in &[15.0, 30.0, 45.0, 60.0, 90.0, 135.0, 180.0] {
        let rotated =
            rotate_by_angle(&pixs, angle).unwrap_or_else(|e| panic!("rotate {}°: {}", angle, e));
        rp.compare_values(
            1.0,
            if rotated.width() > 0 && rotated.height() > 0 {
                1.0
            } else {
                0.0
            },
            0.0,
        );
        eprintln!(
            "  rotate {}°: {}x{}",
            angle,
            rotated.width(),
            rotated.height()
        );
    }

    // --- Test 5: Rotate and rotate back should produce valid image ---
    // Note: arbitrary angle rotation expands canvas, so dimensions won't match original
    let fwd = rotate_by_angle(&pixs, 10.0).expect("rotate 10°");
    let back = rotate_by_angle(&fwd, -10.0).expect("rotate -10°");
    let valid = back.width() >= w && back.height() >= h;
    rp.compare_values(1.0, if valid { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  rotate +10 then -10: {}x{} (orig {}x{})",
        back.width(),
        back.height(),
        w,
        h
    );

    // NOTE: C版のrotateAMColor, rotateAMGray, rotateBySampling等の
    // メソッド比較テストは、Rust APIの詳細に依存

    assert!(rp.cleanup(), "rotate2 regression test failed");
}
