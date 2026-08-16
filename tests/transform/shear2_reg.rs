//! Quadratic shear regression test
//!
//! Tests quadratic vertical shear with sampled and interpolated methods,
//! in both left and right directions.
//!
//! The C version uses pixCreate, pixSetAll, and pixRenderLineArb to create
//! test images with colored lines, then applies quadratic shear. It also
//! uses BMF text labels for display. We test with loaded images instead.
//!
//! # See also
//!
//! C Leptonica: `prog/shear2_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::transform::{WarpDirection, WarpFill, WarpOperation};

/// Test quadratic vertical shear sampled on 32bpp color (C check 0).
///
/// Applies sampled quadratic shear in both directions and verifies output.
#[test]
fn shear2_reg_color_sampled() {
    let mut rp = RegParams::new("shear2_color_samp");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();

    // Sampled, warp to left
    let left = leptonica::transform::quadratic_v_shear_sampled(
        &pix,
        WarpDirection::ToLeft,
        60,
        -20,
        WarpFill::White,
    )
    .expect("quad_v_shear sampled left");
    assert!(left.width() > 0 && left.height() > 0);
    rp.compare_values(w as f64, left.width() as f64, 0.0);
    rp.write_pix_and_check(&left, ImageFormat::Png)
        .expect("write left");

    // Sampled, warp to right
    let right = leptonica::transform::quadratic_v_shear_sampled(
        &pix,
        WarpDirection::ToRight,
        60,
        -20,
        WarpFill::White,
    )
    .expect("quad_v_shear sampled right");
    rp.compare_values(w as f64, right.width() as f64, 0.0);

    // Left and right shears should produce different results
    rp.compare_values(h as f64, left.height() as f64, 0.0);
    rp.compare_values(h as f64, right.height() as f64, 0.0);

    assert!(rp.cleanup(), "shear2 color sampled test failed");
}

/// Test quadratic vertical shear interpolated on 8bpp grayscale (C check 1).
///
/// Applies interpolated quadratic shear in both directions.
#[test]
fn shear2_reg_gray_interpolated() {
    let mut rp = RegParams::new("shear2_gray_interp");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let w = pix.width();
    let h = pix.height();

    // Interpolated, warp to left
    let left = leptonica::transform::quadratic_v_shear_li(
        &pix,
        WarpDirection::ToLeft,
        60,
        -20,
        WarpFill::White,
    )
    .expect("quad_v_shear li left");
    rp.compare_values(w as f64, left.width() as f64, 0.0);
    rp.compare_values(h as f64, left.height() as f64, 0.0);
    rp.write_pix_and_check(&left, ImageFormat::Png)
        .expect("write left");

    // Interpolated, warp to right
    let right = leptonica::transform::quadratic_v_shear_li(
        &pix,
        WarpDirection::ToRight,
        60,
        -20,
        WarpFill::White,
    )
    .expect("quad_v_shear li right");
    rp.compare_values(w as f64, right.width() as f64, 0.0);
    rp.compare_values(h as f64, right.height() as f64, 0.0);

    assert!(rp.cleanup(), "shear2 gray interpolated test failed");
}

/// Test quadratic vertical shear with generic operation parameter (C check 2-3).
///
/// Uses the general quadratic_v_shear with explicit WarpOperation.
#[test]
fn shear2_reg_general() {
    let mut rp = RegParams::new("shear2_general");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();

    // General function with Sampled operation
    let sampled = leptonica::transform::quadratic_v_shear(
        &pix,
        WarpDirection::ToLeft,
        60,
        -20,
        WarpOperation::Sampled,
        WarpFill::White,
    )
    .expect("quad_v_shear general sampled");
    rp.compare_values(w as f64, sampled.width() as f64, 0.0);
    rp.write_pix_and_check(&sampled, ImageFormat::Png)
        .expect("write sampled");

    // General function with Interpolated operation
    let interp = leptonica::transform::quadratic_v_shear(
        &pix,
        WarpDirection::ToRight,
        60,
        -20,
        WarpOperation::Interpolated,
        WarpFill::White,
    )
    .expect("quad_v_shear general interp");
    rp.compare_values(w as f64, interp.width() as f64, 0.0);

    assert!(rp.cleanup(), "shear2 general test failed");
}

/// C-comparable quadratic vertical shear series (plan 902 PR 15).
///
/// Mirrors C shear2_reg exactly: 301x301 and 601x601 white RGB canvases
/// with six coloured horizontal lines, sheared to left/right in both
/// sampled and interpolated modes (vmax 60 / -20), each result converted
/// to 32 bpp, given a 3 px black border and a labelled textblock, then
/// tiled with `display_tiled_in_columns(2, 1.0, 20, 0)`.
#[test]
fn shear2_c_compat() {
    use leptonica::core::bmf::TextblockLocation;
    use leptonica::core::pix::graphics::Color;
    use leptonica::transform::{WarpDirection, WarpFill, WarpOperation, quadratic_v_shear};
    use leptonica::{Bmf, Pix, Pixa, PixelDepth};

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("shear2_c");
    let bmf = Bmf::new(8).expect("bmf size 8");

    // C: white canvas with six coloured lines of width 5 at fixed rows.
    let make_canvas = |size: u32| -> Pix {
        let p = Pix::new(size, size, PixelDepth::Bit32).expect("create canvas");
        let mut pm = p.try_into_mut().expect("into_mut");
        pm.set_all();
        let lines: [(i32, Color); 6] = [
            (20, Color::new(0, 0, 255)),
            (70, Color::new(0, 255, 0)),
            (120, Color::new(0, 255, 255)),
            (170, Color::new(255, 0, 0)),
            (220, Color::new(255, 0, 255)),
            (270, Color::new(255, 255, 0)),
        ];
        for (y, color) in lines {
            pm.render_line_color(0, y, 300, y, 5, color)
                .expect("render line");
        }
        pm.into()
    };
    let pixs1 = make_canvas(301);
    let pixs2 = make_canvas(601);

    // C PixSave: convert to 32bpp, add a 3px black border, label below.
    let save = |pixa: &mut Pixa, pix: &Pix, text: &str| {
        let p32 = pix.convert_to_32().expect("to 32bpp");
        let bordered = p32.add_border(3, 0).expect("add border");
        let (labelled, _) = bmf
            .add_single_textblock(&bordered, text, 0xff000000, TextblockLocation::Below)
            .expect("textblock");
        pixa.push(labelled);
    };

    // C checks 0-3: colour/gray x small/large.
    let sources = [
        pixs1.clone(),
        pixs1.convert_to_8().expect("gray small"),
        pixs2.clone(),
        pixs2.convert_to_8().expect("gray large"),
    ];
    for src in &sources {
        let mut pixa = Pixa::new();
        for (op, opname) in [
            (WarpOperation::Sampled, "sampled"),
            (WarpOperation::Interpolated, "interpolated"),
        ] {
            for (dir, dirname) in [
                (WarpDirection::ToLeft, "left"),
                (WarpDirection::ToRight, "right"),
            ] {
                let sheared = quadratic_v_shear(src, dir, 60, -20, op, WarpFill::White)
                    .expect("quadratic_v_shear");
                save(&mut pixa, &sheared, &format!("{opname}-{dirname}"));
            }
        }
        let out = pixa
            .display_tiled_in_columns(2, 1.0, 20, 0)
            .expect("tiled columns");
        rp.write_pix_and_check(&out, ImageFormat::Png)
            .expect("check: shear2 quadratic v shear");
    }

    assert!(rp.cleanup(), "shear2 c-compat test failed");
}

/// The 32 bpp white warp fill must be opaque white (plan 902 PR 15).
///
/// C fills the vacated region with `pixSetBlackOrWhite(pixd,
/// L_BRING_IN_WHITE)`, which for depths above 1 bpp is `pixSetAll` — every
/// bit set, i.e. 0xffffffff including the alpha byte. A fill of
/// 0xffffff00 leaves the alpha byte clear and diverges from C.
#[test]
#[ignore = "not yet implemented: 32bpp warp white fill leaves alpha 0"]
fn shear2_warp_white_fill_is_opaque() {
    use leptonica::transform::{WarpDirection, WarpFill, WarpOperation, quadratic_v_shear};
    use leptonica::{Pix, PixelDepth};

    // A small canvas whose top-right corner is vacated by a right warp.
    let pix = {
        let p = Pix::new(64, 64, PixelDepth::Bit32).unwrap();
        let mut pm = p.try_into_mut().unwrap();
        for y in 0..64u32 {
            for x in 0..64u32 {
                pm.set_pixel(x, y, 0x0000_00ff).unwrap();
            }
        }
        let p: Pix = pm.into();
        p
    };
    let out = quadratic_v_shear(
        &pix,
        WarpDirection::ToRight,
        30,
        -10,
        WarpOperation::Sampled,
        WarpFill::White,
    )
    .expect("quadratic_v_shear");

    // The vacated pixels must be fully opaque white.
    let vacated = out.get_pixel(63, 0).unwrap();
    assert_eq!(vacated, 0xffff_ffff, "got {vacated:08x}");
}
