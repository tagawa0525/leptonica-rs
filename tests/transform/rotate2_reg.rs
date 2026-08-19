//! Rotation regression test 2 - advanced arbitrary angle rotation
//!
//! C version: `prog/rotate2_reg.c`
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

use crate::common::{RegParams, load_test_image};
use leptonica::io::ImageFormat;
use leptonica::transform::{rotate_by_angle, rotate_by_radians};

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
    rp.write_pix_and_check(&rotated, ImageFormat::Png)
        .expect("write rotated 5deg");
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
    rp.write_pix_and_check(&rot_rad, ImageFormat::Png)
        .expect("write rot_rad");
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
    rp.write_pix_and_check(&back, ImageFormat::Png)
        .expect("write back");
    eprintln!(
        "  rotate +10 then -10: {}x{} (orig {}x{})",
        back.width(),
        back.height(),
        w,
        h
    );

    assert!(rp.cleanup(), "rotate2 regression test failed");
}

/// C-compat: `prog/rotate2_reg.c`, every PNG output.
///
/// `RotateTest` writes PNG only when the depth is neither 8 nor 32 bpp, so
/// the four lossless 1/2/4 bpp inputs account for all 8 PNG outputs.
#[test]
fn rotate2_c_compat() {
    use leptonica::Pix;
    use leptonica::core::Pixa;
    use leptonica::io::ImageFormat;
    use leptonica::transform::{
        RotateEmbed, RotateFill, RotateMethod, RotateOptions, rotate, scale_to_gray_2,
    };

    if crate::common::is_display_mode() {
        return;
    }

    // C uses the literal 3.14159265, not M_PI.
    #[allow(clippy::approx_constant)]
    const ANGLE1: f32 = (3.141_592_65 / 30.0) as f32;
    #[allow(clippy::approx_constant)]
    const ANGLE2: f32 = (3.141_592_65 / 7.0) as f32;
    const IMAGES: [&str; 4] = [
        "test1.png",
        "weasel2.4c.png",
        "weasel4.11c.png",
        "weasel4.16g.png",
    ];

    let mut rp = RegParams::new("rotate2_c");

    for name in IMAGES {
        let pixs = crate::common::load_test_image(name).expect("load rotate input");
        let (w, h) = (pixs.width(), pixs.height());

        let opts = |method: RotateMethod, fill: RotateFill, embed: RotateEmbed| RotateOptions {
            method,
            fill,
            center_x: None,
            center_y: None,
            embed,
        };
        // C passes either (w, h) — embed — or (0, 0) — no embedding.
        let fit = RotateEmbed::Explicit(w, h);
        let none = RotateEmbed::None;

        // C write #1: eight shear rotations, two angles x two fills x embed/no.
        let mut pixa = Pixa::new();
        for angle in [ANGLE1, ANGLE2] {
            for embed in [fit, none] {
                for fill in [RotateFill::White, RotateFill::Black] {
                    pixa.push(
                        rotate(&pixs, angle, &opts(RotateMethod::Shear, fill, embed))
                            .expect("shear rotate"),
                    );
                }
            }
        }
        let tiled = pixa
            .display_tiled_in_columns(2, 1.0, 20, 0)
            .expect("tile shear");
        rp.write_pix_and_check(&tiled, ImageFormat::Png)
            .expect("check: shear variants");

        // C write #2: four sampling rotations then four area-map ones. For a
        // 1 bpp source the area-map block runs on pixScaleToGray2 output.
        let mut pixa = Pixa::new();
        for embed in [fit, none] {
            for fill in [RotateFill::White, RotateFill::Black] {
                pixa.push(
                    rotate(&pixs, ANGLE2, &opts(RotateMethod::Sampling, fill, embed))
                        .expect("sampling rotate"),
                );
            }
        }
        let am_src: Pix = if pixs.depth() == leptonica::PixelDepth::Bit1 {
            scale_to_gray_2(&pixs).expect("scale to gray 2")
        } else {
            pixs.clone()
        };
        for embed in [fit, none] {
            for fill in [RotateFill::White, RotateFill::Black] {
                pixa.push(
                    rotate(&am_src, ANGLE2, &opts(RotateMethod::AreaMap, fill, embed))
                        .expect("area map rotate"),
                );
            }
        }
        let tiled = pixa
            .display_tiled_in_columns(2, 1.0, 20, 0)
            .expect("tile sampling/areamap");
        rp.write_pix_and_check(&tiled, ImageFormat::Png)
            .expect("check: sampling and area map variants");
    }

    assert!(rp.cleanup(), "rotate2 C-compat test failed");
}
