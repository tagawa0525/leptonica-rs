//! Small Pix scaling and rotation regression test
//!
//! Tests quantization behavior of scaling and rotation on very small images.
//! Creates a 9×9 test pattern with cross lines, then applies various
//! scaling/rotation methods at different parameters to verify symmetry
//! and correctness.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/smallpix_reg.c`

use leptonica_core::Pix;
use leptonica_core::PixelDepth;
use leptonica_core::color;
use leptonica_transform::{
    RotateFill, expand_replicate, rotate_am_color_corner, rotate_am_corner, scale_by_sampling,
    scale_color_li, scale_li, scale_smooth,
};

/// Helper: create the 9×9 cross test pattern used by the C version.
///
/// C version uses generatePtaLineFromPt + pixRenderPta + pixPaintThroughMask
/// to draw a green cross pattern centered at (4,4).
///
/// We create it directly by setting pixels.
fn make_test_pattern() -> Pix {
    let pix = Pix::new(9, 9, PixelDepth::Bit32).expect("create 9x9");
    let green = color::compose_rgba(0, 255, 0, 0);

    // Draw horizontal and vertical cross lines through center (4,4)
    let mut pm = pix.try_into_mut().expect("into_mut");
    for i in 0..9u32 {
        let _ = pm.set_pixel(i, 4, green); // horizontal
        let _ = pm.set_pixel(4, i, green); // vertical
    }
    pm.into()
}

/// Test pixScaleSmooth at 11 downscale factors (C test check 0)
///
/// C version expands 2x first, then scales at factors 0.30–0.685.
#[test]
#[ignore = "not yet implemented"]
fn smallpix_reg_scale_smooth() {
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 2).expect("expand 2x");

    for i in 0..11 {
        let scale = 0.30 + 0.035 * i as f32;
        let pix2 = scale_smooth(&pix1, scale, scale).expect("scale_smooth");
        let _pix3 = expand_replicate(&pix2, 6).expect("expand 6x");
    }

    // TODO: display_tiled_in_columns + write_pix_and_check
}

/// Test pixScaleBySampling at 11 downscale factors (C test check 2)
#[test]
#[ignore = "not yet implemented"]
fn smallpix_reg_scale_by_sampling() {
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 2).expect("expand 2x");

    for i in 0..11 {
        let scale = 0.30 + 0.035 * i as f32;
        let pix2 = scale_by_sampling(&pix1, scale, scale).expect("scale_by_sampling");
        let _pix3 = expand_replicate(&pix2, 6).expect("expand 6x");
    }
}

/// Test pixRotateAMCorner at 11 angles (C test check 3)
#[test]
#[ignore = "not yet implemented"]
fn smallpix_reg_rotate_am() {
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 1).expect("expand 1x");

    for i in 0..11 {
        let angle = 0.10 + 0.05 * i as f32;
        let pix2 = rotate_am_corner(&pix1, angle, RotateFill::Black).expect("rotate_am_corner");
        let _pix3 = expand_replicate(&pix2, 8).expect("expand 8x");
    }
}

/// Test pixRotateAMColorFast at 11 angles (C test check 6)
#[test]
#[ignore = "not yet implemented"]
fn smallpix_reg_rotate_am_color_fast() {
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 1).expect("expand 1x");

    for i in 0..11 {
        let angle = 0.10 + 0.05 * i as f32;
        let pix2 = rotate_am_color_corner(&pix1, angle, RotateFill::Black)
            .expect("rotate_am_color_corner");
        let _pix3 = expand_replicate(&pix2, 8).expect("expand 8x");
    }
}

/// Test pixScaleColorLI at 11 upscale factors (C test check 7)
#[test]
#[ignore = "not yet implemented"]
fn smallpix_reg_scale_color_li() {
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 1).expect("expand 1x");

    for i in 0..11 {
        let scale = 1.0 + 0.2 * i as f32;
        let pix2 = scale_color_li(&pix1, scale, scale).expect("scale_color_li");
        let _pix3 = expand_replicate(&pix2, 4).expect("expand 4x");
    }
}

/// Test pixScaleLI at 11 upscale factors (C test check 8)
#[test]
#[ignore = "not yet implemented"]
fn smallpix_reg_scale_li() {
    let pixc = make_test_pattern();
    let pix1 = expand_replicate(&pixc, 1).expect("expand 1x");

    for i in 0..11 {
        let scale = 1.0 + 0.2 * i as f32;
        let pix2 = scale_li(&pix1, scale, scale).expect("scale_li");
        let _pix3 = expand_replicate(&pix2, 4).expect("expand 4x");
    }
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
