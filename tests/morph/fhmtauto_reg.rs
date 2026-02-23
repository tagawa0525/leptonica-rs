//! FHMT auto-generated code regression test
//!
//! Tests hit-miss transform (HMT) on a binary image.
//! The C version compares auto-generated pixFHMTGen_1 and pixHMTDwa_1
//! against pixHMT for various named sels.
//!
//! Partial migration: hit_miss_transform is tested with sels created via
//! make_thin_sels. The auto-generated functions (pixFHMTGen_1, pixHMTDwa_1)
//! are not available in the Rust API.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/fhmtauto_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::morph::{Sel, ThinSelSet, hit_miss_transform, make_thin_sels};

/// Test hit_miss_transform with various sels (C: pixHMT).
///
/// C: pixref = pixHMT(NULL, pixs, sel);
///    pix2 = pixFHMTGen_1(NULL, pix1, selname);  -- auto-gen version (not available)
#[test]
fn fhmtauto_reg_hit_miss() {
    let mut rp = RegParams::new("fhmtauto_hmt");

    // C: pixs = pixRead("feyn-fract.tif");
    let pix = crate::common::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let w = pix.width();
    let h = pix.height();

    // Test HMT with a small cross pattern (similar to sels from sela4ccThin)
    let sels = make_thin_sels(ThinSelSet::Set4cc1);
    for sel in &sels {
        let result = hit_miss_transform(&pix, sel).expect("hit_miss_transform");
        rp.compare_values(w as f64, result.width() as f64, 0.0);
        rp.compare_values(h as f64, result.height() as f64, 0.0);
        assert_eq!(result.depth(), PixelDepth::Bit1);
        // HMT should return subset of foreground pixels
        rp.compare_values(
            1.0,
            if result.count_pixels() <= pix.count_pixels() {
                1.0
            } else {
                0.0
            },
            0.0,
        );
    }

    assert!(rp.cleanup(), "fhmtauto hit_miss test failed");
}

/// Test hit_miss_transform with a custom brick sel.
///
/// Verifies that HMT with a 1x1 HIT sel returns the original image.
#[test]
fn fhmtauto_reg_identity_sel() {
    let mut rp = RegParams::new("fhmtauto_id");

    let pix = crate::common::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let w = pix.width();
    let h = pix.height();

    // A 1x1 HIT sel: hit-miss transform should return the original image
    let sel = Sel::create_brick(1, 1).expect("create 1x1 sel");
    let result = hit_miss_transform(&pix, &sel).expect("hit_miss_transform 1x1");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit1);
    assert!(result.equals(&pix));

    assert!(rp.cleanup(), "fhmtauto identity sel test failed");
}

/// Test auto-generated FHMT functions (pixFHMTGen_1, pixHMTDwa_1).
///
/// These require auto-generated code that is not available in the Rust API.
#[test]
#[ignore = "not yet implemented: auto-generated pixFHMTGen_1/pixHMTDwa_1 not available"]
fn fhmtauto_reg_autogen() {
    // C: pix2 = pixFHMTGen_1(NULL, pix1, selname);
    //    pix4 = pixHMTDwa_1(NULL, pixs, selname);
    //    regTestComparePix(rp, pixref, pix3);  /* compares with pixHMT */
}
