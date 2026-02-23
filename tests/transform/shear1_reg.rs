//! Shear regression test (part 1)
//!
//! Tests horizontal and vertical shear operations at various image depths.
//! The C version tests shear with white/black fill, in-place variants,
//! and linear interpolation, across 1bpp, 2bpp, 4bpp, 8bpp, cmap, and 32bpp.
//!
//! Partial migration: colormap shear tests require pixOctreeColorQuant
//! which is not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/shear1_reg.c`

use crate::common::RegParams;
use leptonica::transform::{
    ShearFill, h_shear, h_shear_center, h_shear_corner, v_shear, v_shear_center, v_shear_corner,
};

const ANGLE1: f32 = std::f32::consts::PI / 12.0;

/// Test horizontal and vertical shear on 8bpp grayscale (C checks 4).
///
/// Verifies shear about corner, center, and arbitrary line with both fills.
#[test]
fn shear1_reg_grayscale_8bpp() {
    let mut rp = RegParams::new("shear1_gray8");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // Horizontal shear about corner (y=0) with white fill
    let sheared = h_shear_corner(&pix, ANGLE1, ShearFill::White).expect("h_shear_corner white");
    rp.compare_values(h as f64, sheared.height() as f64, 0.0);
    assert!(sheared.width() > 0, "sheared width must be positive");

    // Horizontal shear about center with black fill
    let sheared_c = h_shear_center(&pix, ANGLE1, ShearFill::Black).expect("h_shear_center black");
    rp.compare_values(h as f64, sheared_c.height() as f64, 0.0);

    // Vertical shear about corner (x=0) with white fill
    let vsheared = v_shear_corner(&pix, ANGLE1, ShearFill::White).expect("v_shear_corner white");
    rp.compare_values(w as f64, vsheared.width() as f64, 0.0);
    assert!(vsheared.height() > 0, "v-sheared height must be positive");

    // Vertical shear about center with black fill
    let vsheared_c = v_shear_center(&pix, ANGLE1, ShearFill::Black).expect("v_shear_center black");
    rp.compare_values(w as f64, vsheared_c.width() as f64, 0.0);

    // Arbitrary line shear: h_shear at y=h/2
    let arb = h_shear(&pix, (h / 2) as i32, ANGLE1, ShearFill::White).expect("h_shear arb");
    rp.compare_values(h as f64, arb.height() as f64, 0.0);

    // Arbitrary line shear: v_shear at x=w/2
    let varb = v_shear(&pix, (w / 2) as i32, ANGLE1, ShearFill::White).expect("v_shear arb");
    rp.compare_values(w as f64, varb.width() as f64, 0.0);

    assert!(rp.cleanup(), "shear1 grayscale 8bpp test failed");
}

/// Test horizontal and vertical shear on 1bpp binary (C check 0).
///
/// Verifies shear operations preserve binary depth and produce valid output.
#[test]
fn shear1_reg_binary() {
    let mut rp = RegParams::new("shear1_binary");

    let pix = crate::common::load_test_image("test1.png").expect("load test1.png");
    let h = pix.height();
    let w = pix.width();
    assert_eq!(pix.depth(), leptonica::PixelDepth::Bit1);

    // Horizontal shear with white and black fill
    let hw = h_shear_corner(&pix, ANGLE1, ShearFill::White).expect("h_shear_corner white");
    let hb = h_shear_corner(&pix, ANGLE1, ShearFill::Black).expect("h_shear_corner black");
    rp.compare_values(h as f64, hw.height() as f64, 0.0);
    rp.compare_values(h as f64, hb.height() as f64, 0.0);
    assert_eq!(hw.depth(), leptonica::PixelDepth::Bit1);

    // Vertical shear
    let vw = v_shear_corner(&pix, ANGLE1, ShearFill::White).expect("v_shear_corner white");
    let vb = v_shear_corner(&pix, ANGLE1, ShearFill::Black).expect("v_shear_corner black");
    rp.compare_values(w as f64, vw.width() as f64, 0.0);
    rp.compare_values(w as f64, vb.width() as f64, 0.0);
    assert_eq!(vw.depth(), leptonica::PixelDepth::Bit1);

    assert!(rp.cleanup(), "shear1 binary test failed");
}

/// Test in-place shear operations (C checks 8-11).
///
/// Verifies in-place h_shear_ip and v_shear_ip produce the same result
/// as the allocating variants.
#[test]
fn shear1_reg_in_place() {
    let mut rp = RegParams::new("shear1_inplace");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let h = pix.height();
    let w = pix.width();

    // Compare h_shear vs h_shear_ip at center
    let expected = h_shear(&pix, (h / 2) as i32, ANGLE1, ShearFill::White).expect("h_shear");
    let mut pix_mut = pix.to_mut();
    leptonica::transform::h_shear_ip(&mut pix_mut, (h / 2) as i32, ANGLE1, ShearFill::White)
        .expect("h_shear_ip");
    let actual: leptonica::Pix = pix_mut.into();
    rp.compare_pix(&expected, &actual);

    // Compare v_shear vs v_shear_ip at center
    let expected_v = v_shear(&pix, (w / 2) as i32, ANGLE1, ShearFill::Black).expect("v_shear");
    let mut pix_mut_v = pix.to_mut();
    leptonica::transform::v_shear_ip(&mut pix_mut_v, (w / 2) as i32, ANGLE1, ShearFill::Black)
        .expect("v_shear_ip");
    let actual_v: leptonica::Pix = pix_mut_v.into();
    rp.compare_pix(&expected_v, &actual_v);

    assert!(rp.cleanup(), "shear1 in-place test failed");
}

/// Test linear interpolated shear on 8bpp and 32bpp (C checks 4-7 LI portion).
///
/// Verifies h_shear_li and v_shear_li produce valid output at 8bpp and 32bpp.
#[test]
fn shear1_reg_interpolated() {
    let mut rp = RegParams::new("shear1_interp");

    // 8bpp
    let pix8 = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let hli = leptonica::transform::h_shear_li(&pix8, 0, ANGLE1, ShearFill::White)
        .expect("h_shear_li 8bpp");
    rp.compare_values(pix8.height() as f64, hli.height() as f64, 0.0);
    let vli = leptonica::transform::v_shear_li(&pix8, 0, ANGLE1, ShearFill::White)
        .expect("v_shear_li 8bpp");
    rp.compare_values(pix8.width() as f64, vli.width() as f64, 0.0);

    // 32bpp
    let pix32 = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let hli32 = leptonica::transform::h_shear_li(&pix32, 0, ANGLE1, ShearFill::Black)
        .expect("h_shear_li 32bpp");
    rp.compare_values(pix32.height() as f64, hli32.height() as f64, 0.0);
    let vli32 = leptonica::transform::v_shear_li(&pix32, 0, ANGLE1, ShearFill::Black)
        .expect("v_shear_li 32bpp");
    rp.compare_values(pix32.width() as f64, vli32.width() as f64, 0.0);

    assert!(rp.cleanup(), "shear1 interpolated test failed");
}

/// Test shear on colormapped images (C checks 1-3, 6).
///
/// Requires pixOctreeColorQuant for full C test coverage.
#[test]
#[ignore = "not yet implemented: pixOctreeColorQuant not available for colormap shear test"]
fn shear1_reg_colormap() {
    // C version:
    // 1. Read 2bpp cmapped image (weasel2.4c.png), modify cmap, shear
    // 2. Read 4bpp cmapped images (weasel4.11c.png, weasel4.16g.png), shear
    // 3. Read 8bpp color cmap via pixOctreeColorQuant, shear
    // All require colormap handling in shear functions
}
