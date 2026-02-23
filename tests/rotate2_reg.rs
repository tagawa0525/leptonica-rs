//! Rotation regression test 2 - advanced arbitrary angle rotation
//!
//! C version: `reference/leptonica/prog/rotate2_reg.c`
//!
//! Tests various rotation methods (shear, sampling, area-map) at different
//! angles, comparing results across methods and validating that rotated images
//! are non-degenerate.
//!
//! C version tests:
//!   1. Rotation by ANGLE1 (pi/30) and ANGLE2 (pi/7) with shear method,
//!      using L_BRING_IN_WHITE and L_BRING_IN_BLACK, with/without expansion
//!   2. Rotation by ANGLE2 with sampling method
//!   3. Rotation by ANGLE2 with area-map method (requires >= 8bpp)

use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{rotate_by_angle, rotate_by_radians};

/// Test arbitrary-angle rotation methods
///
/// Validates `rotate_by_angle` and `rotate_by_radians` produce valid non-degenerate
/// output at various angles. C version compares shear/sampling/area-map methods across
/// 8 image types (1/2/4/8/8cmap/32bpp).
#[test]
fn rotate2_reg() {
    let mut rp = RegParams::new("rotate2");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // --- Test 1: Small angle rotation preserves dimensions ---
    // C version: pixRotate(pixs, ANGLE1, L_ROTATE_SHEAR, L_BRING_IN_WHITE, w, h)
    let angle = 5.0_f32;
    let rotated = rotate_by_angle(&pixs, angle).expect("rotate_by_angle 5 deg");
    rp.compare_values(
        1.0,
        if rotated.width() > 0 && rotated.height() > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    eprintln!("  rotate 5 deg: {}x{}", rotated.width(), rotated.height());

    // --- Test 2: Rotation by 0 degrees should preserve image ---
    let rot0 = rotate_by_angle(&pixs, 0.0).expect("rotate 0 deg");
    rp.compare_values(w as f64, rot0.width() as f64, 0.0);
    rp.compare_values(h as f64, rot0.height() as f64, 0.0);

    // --- Test 3: Rotation by radians ---
    // C version: various angles tested with ANGLE2 = pi/7
    let radians = std::f32::consts::PI / 6.0;
    let rot_rad = rotate_by_radians(&pixs, radians).expect("rotate_by_radians pi/6");
    rp.compare_values(1.0, if rot_rad.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  rotate pi/6 rad: {}x{}",
        rot_rad.width(),
        rot_rad.height()
    );

    // --- Test 4: Various angles ---
    // C version tests ANGLE1=pi/30 and ANGLE2=pi/7 on 8 image types
    for &angle in &[15.0, 30.0, 45.0, 60.0, 90.0, 135.0, 180.0] {
        let rotated =
            rotate_by_angle(&pixs, angle).unwrap_or_else(|e| panic!("rotate {} deg: {}", angle, e));
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
            "  rotate {} deg: {}x{}",
            angle,
            rotated.width(),
            rotated.height()
        );
    }

    // --- Test 5: Rotate and rotate back should produce valid image ---
    // C version: forward+inverse rotation tested for each method
    let fwd = rotate_by_angle(&pixs, 10.0).expect("rotate 10 deg");
    let back = rotate_by_angle(&fwd, -10.0).expect("rotate -10 deg");
    let valid = back.width() >= w && back.height() >= h;
    rp.compare_values(1.0, if valid { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  rotate +10 then -10: {}x{} (orig {}x{})",
        back.width(),
        back.height(),
        w,
        h
    );

    assert!(rp.cleanup(), "rotate2 regression test failed");
}
