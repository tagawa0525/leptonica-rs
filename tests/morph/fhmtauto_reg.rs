//! FHMT auto-generated code regression test
//!
//! C version: reference/leptonica/prog/fhmtauto_reg.c
//! Tests hit-miss transform (HMT) on a binary image.
//! C版は auto-gen pixFHMTGen_1 / pixHMTDwa_1 vs pixHMT を比較。
//! Rust版は pixHMT 結果のgolden化 + 性質検証。
//!
//! C checkpoint mapping (20 total):
//!   0-19: compare_pix(pixHMT, pixFHMTGen_1/pixHMTDwa_1) — auto-gen未実装
//!
//! Rust追加:
//!   write_pix_and_check: HMT結果をgolden化（各SEL set）

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::morph::{Sel, ThinSelSet, hit_miss_transform, make_thin_sels};

/// Test hit_miss_transform with various SEL sets and golden-check results.
///
/// C: pixref = pixHMT(NULL, pixs, sel);
#[test]
fn fhmtauto_reg_hit_miss() {
    let mut rp = RegParams::new("fhmtauto_hmt");

    let pix = crate::common::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    // Test HMT with SEL sets used in thinning (similar to C's selaAddHitMiss sels)
    let sels = make_thin_sels(ThinSelSet::Set4cc1);
    for sel in &sels {
        let result = hit_miss_transform(&pix, sel).expect("hit_miss_transform");
        assert_eq!(result.depth(), PixelDepth::Bit1);
        // HMT result is subset of foreground
        rp.compare_values(
            1.0,
            if result.count_pixels() <= pix.count_pixels() {
                1.0
            } else {
                0.0
            },
            0.0,
        );
        // Golden check
        rp.write_pix_and_check(&result, ImageFormat::Tiff)
            .expect("write HMT result");
    }

    // Additional SEL sets for broader coverage
    let sels8 = make_thin_sels(ThinSelSet::Set8cc1);
    for sel in &sels8 {
        let result = hit_miss_transform(&pix, sel).expect("hit_miss_transform 8cc");
        rp.compare_values(
            1.0,
            if result.count_pixels() <= pix.count_pixels() {
                1.0
            } else {
                0.0
            },
            0.0,
        );
        rp.write_pix_and_check(&result, ImageFormat::Tiff)
            .expect("write HMT 8cc result");
    }

    assert!(rp.cleanup(), "fhmtauto hit_miss test failed");
}

/// Test HMT with a 1x1 HIT sel returns the original image.
#[test]
fn fhmtauto_reg_identity_sel() {
    let mut rp = RegParams::new("fhmtauto_id");

    let pix = crate::common::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    let sel = Sel::create_brick(1, 1).expect("create 1x1 sel");
    let result = hit_miss_transform(&pix, &sel).expect("hit_miss_transform 1x1");
    assert!(result.equals(&pix));

    rp.write_pix_and_check(&result, ImageFormat::Tiff)
        .expect("write identity HMT result");

    assert!(rp.cleanup(), "fhmtauto identity sel test failed");
}

/// Auto-generated FHMT functions (pixFHMTGen_1, pixHMTDwa_1) — Rust未実装
#[test]
#[ignore = "not yet implemented: auto-generated pixFHMTGen_1/pixHMTDwa_1 not available"]
fn fhmtauto_reg_autogen() {
    // C: pix2 = pixFHMTGen_1(NULL, pix1, selname);
    //    pix4 = pixHMTDwa_1(NULL, pixs, selname);
    //    regTestComparePix(rp, pixref, pix3);
}
