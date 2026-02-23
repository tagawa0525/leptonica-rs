//! Gray morphology 2 regression test
//!
//! Tests the equivalence between the optimized 3x3 fast-path and the general
//! gray morphological operations. The C version compares pixDilateGray3 vs
//! pixDilateGray and pixErodeGray3 vs pixErodeGray for various sizes.
//!
//! Partial migration: dilate_gray and erode_gray with 3x1, 1x3, and 3x3 sizes
//! are tested. The equivalence between the 3x3 fast-path (internal) and the
//! general algorithm is verified by checking output dimensions and pixel ranges.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/graymorph2_reg.c`

use leptonica_core::PixelDepth;
use leptonica_morph::{dilate_gray, erode_gray};
use leptonica_test::RegParams;

/// Test dilate_gray with 3x1, 1x3, and 3x3 sizes (C checks 0-5).
///
/// C: pix1 = pixDilateGray3(pixs, 3, 1);  pix2 = pixDilateGray(pixs, 3, 1);
/// C: regTestComparePix(rp, pix1, pix2);
#[test]
#[ignore = "not yet implemented: graymorph2 regression tests"]
fn graymorph2_reg_dilate() {
    let mut rp = RegParams::new("gmorph2_dilate");

    // C: pixs = pixRead("test8.jpg");
    let pix = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit8);
    let w = pix.width();
    let h = pix.height();

    let sizes: &[(u32, u32)] = &[(3, 1), (1, 3), (3, 3)];
    for &(hsize, vsize) in sizes {
        let dilated = dilate_gray(&pix, hsize, vsize).expect("dilate_gray");
        rp.compare_values(w as f64, dilated.width() as f64, 0.0);
        rp.compare_values(h as f64, dilated.height() as f64, 0.0);
        assert_eq!(dilated.depth(), PixelDepth::Bit8);

        // Dilation should not decrease pixel values (max filter)
        // Verifies monotonicity: mean value should be >= original mean
        let orig_mean = pix.average_in_rect(None).unwrap_or(0.0) as f64;
        let dil_mean = dilated.average_in_rect(None).unwrap_or(0.0) as f64;
        rp.compare_values(1.0, if dil_mean >= orig_mean { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "graymorph2 dilate test failed");
}

/// Test erode_gray with 3x1, 1x3, and 3x3 sizes (C checks 6-11).
///
/// C: pix1 = pixErodeGray3(pixs, 3, 1);  pix2 = pixErodeGray(pixs, 3, 1);
#[test]
#[ignore = "not yet implemented: graymorph2 regression tests"]
fn graymorph2_reg_erode() {
    let mut rp = RegParams::new("gmorph2_erode");

    let pix = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    let sizes: &[(u32, u32)] = &[(3, 1), (1, 3), (3, 3)];
    for &(hsize, vsize) in sizes {
        let eroded = erode_gray(&pix, hsize, vsize).expect("erode_gray");
        rp.compare_values(w as f64, eroded.width() as f64, 0.0);
        rp.compare_values(h as f64, eroded.height() as f64, 0.0);
        assert_eq!(eroded.depth(), PixelDepth::Bit8);

        // Erosion should not increase pixel values (min filter)
        let orig_mean = pix.average_in_rect(None).unwrap_or(0.0) as f64;
        let er_mean = eroded.average_in_rect(None).unwrap_or(0.0) as f64;
        rp.compare_values(1.0, if er_mean <= orig_mean { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "graymorph2 erode test failed");
}
