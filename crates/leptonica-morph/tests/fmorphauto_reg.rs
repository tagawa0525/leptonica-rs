//! FMorph auto-generated code regression test
//!
//! Tests dilate and erode operations with various structuring elements.
//! The C version compares auto-generated pixFMorphopGen_1 against pixDilate/
//! pixErode for named sels from a pre-defined library.
//!
//! Partial migration: dilate and erode are tested with brick sels of various
//! sizes. The auto-generated functions (pixFMorphopGen_1) are not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/fmorphauto_reg.c`

use leptonica_core::PixelDepth;
use leptonica_morph::{Sel, dilate, erode};
use leptonica_test::RegParams;

/// Test dilate and erode with various brick sels (C: pixDilate, pixErode).
///
/// C: pixt1 = pixDilate(NULL, pixs, sel);
///    pixt2 = pixFMorphopGen_1(NULL, pixs1, L_MORPH_DILATE, selname);  -- not available
#[test]
fn fmorphauto_reg_dilate_erode() {
    let mut rp = RegParams::new("fmorphauto_ops");

    // C: pixs = pixRead("feyn-fract.tif");
    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let w = pix.width();
    let h = pix.height();

    // Test with various brick sel sizes (subset of C test sel library)
    let sizes: &[(u32, u32)] = &[(1, 1), (3, 3), (5, 5), (7, 1), (1, 7)];
    for &(sw, sh) in sizes {
        let sel = Sel::create_brick(sw, sh).expect("create brick sel");

        let dilated = dilate(&pix, &sel).expect("dilate");
        rp.compare_values(w as f64, dilated.width() as f64, 0.0);
        rp.compare_values(h as f64, dilated.height() as f64, 0.0);
        assert_eq!(dilated.depth(), PixelDepth::Bit1);

        // Dilation is extensive: result contains at least all original pixels
        rp.compare_values(
            1.0,
            if dilated.count_pixels() >= pix.count_pixels() {
                1.0
            } else {
                0.0
            },
            0.0,
        );

        let eroded = erode(&pix, &sel).expect("erode");
        rp.compare_values(w as f64, eroded.width() as f64, 0.0);
        rp.compare_values(h as f64, eroded.height() as f64, 0.0);

        // Erosion is anti-extensive: result is subset of original pixels
        rp.compare_values(
            1.0,
            if eroded.count_pixels() <= pix.count_pixels() {
                1.0
            } else {
                0.0
            },
            0.0,
        );
    }

    assert!(rp.cleanup(), "fmorphauto dilate_erode test failed");
}

/// Test that dilate-erode roundtrip (open) gives fewer pixels than original.
///
/// C: open = erode(dilate(pix)); result should be subset of original
#[test]
fn fmorphauto_reg_open_subset() {
    let mut rp = RegParams::new("fmorphauto_open");

    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let w = pix.width();
    let h = pix.height();
    let orig_count = pix.count_pixels();

    // Opening = erode then dilate (anti-extensive: result <= original)
    let sel = Sel::create_brick(3, 3).expect("create 3x3 sel");
    let eroded = erode(&pix, &sel).expect("erode (open first step)");
    let opened = dilate(&eroded, &sel).expect("dilate (open second step)");

    rp.compare_values(w as f64, opened.width() as f64, 0.0);
    rp.compare_values(h as f64, opened.height() as f64, 0.0);

    // Opening is anti-extensive: opened <= original
    rp.compare_values(
        1.0,
        if opened.count_pixels() <= orig_count {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "fmorphauto open_subset test failed");
}

/// Test auto-generated morphological functions (pixFMorphopGen_1).
///
/// Requires auto-generated code not available in the Rust API.
#[test]
#[ignore = "not yet implemented: auto-generated pixFMorphopGen_1 not available"]
fn fmorphauto_reg_autogen() {
    // C: pixt2 = pixFMorphopGen_1(NULL, pixs1, L_MORPH_DILATE, selname);
    //    pixt3 = pixRemoveBorder(pixt2, 32);
    //    pixt4 = pixXor(NULL, pixt1, pixt3);
    //    pixZero(pixt4, &same);  -- check they are identical
}
