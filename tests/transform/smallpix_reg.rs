//! Small Pix scaling and rotation regression test
//!
//! Tests quantization behavior of scaling and rotation on very small images.
//! Creates a 9×9 test pattern with cross lines, then applies various
//! scaling/rotation methods at different parameters to verify symmetry
//! and correctness.
//!
//! NOTE: Partial port. The C version also tests pixScaleAreaMap (not public)
//! and pixRotateBySampling (not public), and uses display_tiled_in_columns
//! for golden file comparison. Currently tests exercise the API without
//! golden file comparison.
//!
//! # See also
//!
//! C Leptonica: `prog/smallpix_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::core::pixel;
use leptonica::io::ImageFormat;
use leptonica::transform::{
    RotateFill, expand_replicate, rotate_am_color_corner, rotate_am_corner, scale_by_sampling,
    scale_color_li, scale_li, scale_smooth,
};
use leptonica::{Pix, Pixa};

/// Helper: create the 9×9 cross test pattern used by the C version.
///
/// C version uses generatePtaLineFromPt + pixRenderPta + pixPaintThroughMask
/// to draw a green cross pattern centered at (4,4).
///
/// We create it directly by setting pixels.
fn make_test_pattern() -> Pix {
    let pix = Pix::new(9, 9, PixelDepth::Bit32).expect("create 9x9");
    let green = pixel::compose_rgba(0, 255, 0, 0);

    // Draw horizontal and vertical cross lines through center (4,4)
    let mut pm = pix.try_into_mut().expect("into_mut");
    for i in 0..9u32 {
        pm.set_pixel(i, 4, green).expect("set_pixel horizontal");
        pm.set_pixel(4, i, green).expect("set_pixel vertical");
    }
    pm.into()
}

/// Test pixScaleSmooth at 11 downscale factors (C test check 0)
///
/// C version expands 2x first, then scales at factors 0.30–0.685.
#[test]
fn smallpix_reg_scale_smooth() {
    let mut rp = RegParams::new("smallpix_smooth");
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 2).expect("expand 2x");

    let mut pixa = Pixa::new();
    for i in 0..11 {
        let scale = 0.30 + 0.035 * i as f32;
        let pix2 = scale_smooth(&pix1, scale, scale).expect("scale_smooth");
        assert!(pix2.width() > 0 && pix2.height() > 0);
        let pix3 = expand_replicate(&pix2, 6).expect("expand 6x");
        assert_eq!(pix3.width(), pix2.width() * 6);
        assert_eq!(pix3.height(), pix2.height() * 6);
        pixa.push(pix3);
    }
    let tiled = pixa.display_tiled(400, 0, 4).expect("tiled smooth");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: smallpix smooth tiled");
    assert!(rp.cleanup(), "smallpix smooth test failed");
}

/// Test pixScaleBySampling at 11 downscale factors (C test check 2)
#[test]
fn smallpix_reg_scale_by_sampling() {
    let mut rp = RegParams::new("smallpix_sampling");
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 2).expect("expand 2x");

    let mut pixa = Pixa::new();
    for i in 0..11 {
        let scale = 0.30 + 0.035 * i as f32;
        let pix2 = scale_by_sampling(&pix1, scale, scale).expect("scale_by_sampling");
        assert!(pix2.width() > 0 && pix2.height() > 0);
        let pix3 = expand_replicate(&pix2, 6).expect("expand 6x");
        assert_eq!(pix3.width(), pix2.width() * 6);
        assert_eq!(pix3.height(), pix2.height() * 6);
        pixa.push(pix3);
    }
    let tiled = pixa.display_tiled(400, 0, 4).expect("tiled sampling");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: smallpix sampling tiled");
    assert!(rp.cleanup(), "smallpix sampling test failed");
}

/// Test pixRotateAMCorner at 11 angles (C test check 3)
#[test]
fn smallpix_reg_rotate_am() {
    let mut rp = RegParams::new("smallpix_rotate_am");
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 1).expect("expand 1x");

    let mut pixa = Pixa::new();
    for i in 0..11 {
        let angle = 0.10 + 0.05 * i as f32;
        let pix2 = rotate_am_corner(&pix1, angle, RotateFill::Black).expect("rotate_am_corner");
        assert!(pix2.width() > 0 && pix2.height() > 0);
        let pix3 = expand_replicate(&pix2, 8).expect("expand 8x");
        assert_eq!(pix3.width(), pix2.width() * 8);
        assert_eq!(pix3.height(), pix2.height() * 8);
        pixa.push(pix3);
    }
    let tiled = pixa.display_tiled(600, 0, 4).expect("tiled rotate_am");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: smallpix rotate_am tiled");
    assert!(rp.cleanup(), "smallpix rotate_am test failed");
}

/// Test pixRotateAMColorFast at 11 angles (C test check 6)
#[test]
fn smallpix_reg_rotate_am_color_fast() {
    let mut rp = RegParams::new("smallpix_rotate_color");
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 1).expect("expand 1x");

    let mut pixa = Pixa::new();
    for i in 0..11 {
        let angle = 0.10 + 0.05 * i as f32;
        let pix2 = rotate_am_color_corner(&pix1, angle, RotateFill::Black)
            .expect("rotate_am_color_corner");
        assert!(pix2.width() > 0 && pix2.height() > 0);
        let pix3 = expand_replicate(&pix2, 8).expect("expand 8x");
        assert_eq!(pix3.width(), pix2.width() * 8);
        assert_eq!(pix3.height(), pix2.height() * 8);
        pixa.push(pix3);
    }
    let tiled = pixa.display_tiled(600, 0, 4).expect("tiled rotate_color");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: smallpix rotate_color tiled");
    assert!(rp.cleanup(), "smallpix rotate_color test failed");
}

/// Test pixScaleColorLI at 11 upscale factors (C test check 7)
#[test]
fn smallpix_reg_scale_color_li() {
    let mut rp = RegParams::new("smallpix_color_li");
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 1).expect("expand 1x");

    let mut pixa = Pixa::new();
    for i in 0..11 {
        let scale = 1.0 + 0.2 * i as f32;
        let pix2 = scale_color_li(&pix1, scale, scale).expect("scale_color_li");
        assert!(pix2.width() > 0 && pix2.height() > 0);
        let pix3 = expand_replicate(&pix2, 4).expect("expand 4x");
        assert_eq!(pix3.width(), pix2.width() * 4);
        assert_eq!(pix3.height(), pix2.height() * 4);
        pixa.push(pix3);
    }
    let tiled = pixa.display_tiled(800, 0, 4).expect("tiled color_li");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: smallpix color_li tiled");
    assert!(rp.cleanup(), "smallpix color_li test failed");
}

/// Test pixScaleLI at 11 upscale factors (C test check 8)
#[test]
fn smallpix_reg_scale_li() {
    let mut rp = RegParams::new("smallpix_li");
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 1).expect("expand 1x");

    let mut pixa = Pixa::new();
    for i in 0..11 {
        let scale = 1.0 + 0.2 * i as f32;
        let pix2 = scale_li(&pix1, scale, scale).expect("scale_li");
        assert!(pix2.width() > 0 && pix2.height() > 0);
        let pix3 = expand_replicate(&pix2, 4).expect("expand 4x");
        assert_eq!(pix3.width(), pix2.width() * 4);
        assert_eq!(pix3.height(), pix2.height() * 4);
        pixa.push(pix3);
    }
    let tiled = pixa.display_tiled(800, 0, 4).expect("tiled li");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: smallpix li tiled");
    assert!(rp.cleanup(), "smallpix li test failed");
}

/// Test pixScaleAreaMap (C test check 1) and pixRotateBySampling (C test check 4)
///
/// These functions are not publicly available in the Rust version.
#[test]
#[ignore = "not yet implemented: scale_area_map and rotate_by_sampling not public"]
fn smallpix_reg_missing_methods() {
    // pixScaleAreaMap: not publicly exported
    // pixRotateBySampling: only private implementation
    // pixRotateAMCorner: same as rotate_am_corner above (C test check 5)
}

// ==========================================================================
// C-compatible transform entry points (plan 902 PR 14)
// ==========================================================================

/// The C-named rotate/scale entry points must follow C's dispatch rules.
#[test]
fn smallpix_c_entry_points_follow_c_dispatch() {
    use leptonica::transform::{
        rotate_am, rotate_am_color_fast, rotate_by_sampling, scale_area_map,
    };

    let pix32 = make_test_pattern();

    // pixRotateAM rejects 1 bpp and returns a copy below the angle threshold.
    let pix1 = Pix::new(9, 9, PixelDepth::Bit1).unwrap();
    assert!(rotate_am(&pix1, 0.2, RotateFill::Black).is_err());
    let same = rotate_am(&pix32, 0.0, RotateFill::Black).unwrap();
    assert_eq!(same.get_pixel(4, 4), pix32.get_pixel(4, 4));

    // pixRotateAM promotes sub-8bpp to 8bpp.
    let pix4 = Pix::new(9, 9, PixelDepth::Bit4).unwrap();
    let out = rotate_am(&pix4, 0.2, RotateFill::Black).unwrap();
    assert_eq!(out.depth(), PixelDepth::Bit8);
    assert_eq!((out.width(), out.height()), (9, 9));

    // pixRotateBySampling works on 1 bpp (no interpolation) and keeps depth.
    let out = rotate_by_sampling(&pix1, 4, 4, 0.2, RotateFill::White).unwrap();
    assert_eq!(out.depth(), PixelDepth::Bit1);
    // A half-turn about the centre maps the cross onto itself.
    let half_turn =
        rotate_by_sampling(&pix32, 4, 4, std::f32::consts::PI, RotateFill::Black).unwrap();
    for y in 0..9 {
        for x in 0..9 {
            assert_eq!(
                half_turn.get_pixel(x, y),
                pix32.get_pixel(8 - x, 8 - y),
                "half turn at ({x}, {y})"
            );
        }
    }

    // pixRotateAMColorFast is 32bpp-only and leaves the alpha byte 0 on
    // interpolated pixels.
    assert!(rotate_am_color_fast(&pix4, 0.2, RotateFill::Black).is_err());
    let fast = rotate_am_color_fast(&pix32, 0.2, RotateFill::Black).unwrap();
    assert_eq!((fast.width(), fast.height()), (9, 9));
    for y in 0..9 {
        for x in 0..9 {
            assert_eq!(
                fast.get_pixel(x, y).unwrap() & 0xff,
                0,
                "alpha byte must stay 0 at ({x}, {y})"
            );
        }
    }

    // pixScaleAreaMap dispatch: >= 0.7 falls through to regular scaling,
    // exact 1/2 powers use repeated 2x reduction, 1bpp is rejected.
    assert!(scale_area_map(&pix1, 0.5, 0.5).is_err());
    let big = expand_replicate(&pix32, 8).expect("expand 8x"); // 72x72
    let half = scale_area_map(&big, 0.5, 0.5).unwrap();
    assert_eq!((half.width(), half.height()), (36, 36));
    let quarter = scale_area_map(&big, 0.25, 0.25).unwrap();
    assert_eq!((quarter.width(), quarter.height()), (18, 18));
    // 0.3 is the general path: wd = (int)(0.3 * 72 + 0.5) = 22.
    let general = scale_area_map(&big, 0.3, 0.3).unwrap();
    assert_eq!((general.width(), general.height()), (22, 22));
    // >= 0.7 delegates to regular scaling, which rounds instead.
    let large = scale_area_map(&big, 0.8, 0.8).unwrap();
    assert_eq!((large.width(), large.height()), (58, 58));
}
