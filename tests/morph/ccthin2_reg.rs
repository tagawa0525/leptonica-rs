//! Connected-component thinning 2 regression test
//!
//! C version: reference/leptonica/prog/ccthin2_reg.c
//! Tests thin_connected_by_set with various SEL sets on a clipped region
//! of feyn.tif.
//!
//! C checkpoint mapping (19 total):
//!   0-8:   pixThinConnectedBySet with sets 1-9 (L_THIN_FG)
//!   9-10:  pixThinConnectedBySet with sets 10,11 (L_THIN_BG, 5 iterations)
//!   11:    tiled display (Rust未実装)
//!   12-18: stroke width normalization (Rust未実装)

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::morph::{ThinSelSet, ThinType, make_thin_sels, thin_connected_by_set};

/// Test thin_connected_by_set with SEL sets 1-9 on clipped feyn.tif (C checks 0-8).
///
/// C: pix1 = pixClipRectangle(pixs, box1, NULL);
///    pixThinConnectedBySet(pix2, L_THIN_FG, sela, 0) for sets 1-9
#[test]
fn ccthin2_reg_thin_by_set() {
    let mut rp = RegParams::new("cthin2_set");

    // C: pixs = pixRead("feyn.tif");
    //    box1 = boxCreate(683, 799, 970, 479);
    //    pix1 = pixClipRectangle(pixs, box1, NULL);
    //    pix2 = pixClipToForeground(pix1, NULL, NULL);
    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    let pix1 = pixs
        .clip_rectangle(683, 799, 970, 479)
        .expect("clip rectangle");
    // C: pix2 = pixClipToForeground(pix1, NULL, NULL)
    let (pix2, _box) = pix1
        .clip_to_foreground()
        .expect("clip to foreground")
        .expect("foreground region found");
    assert_eq!(pix2.depth(), PixelDepth::Bit1);

    // C checks 0-8: thin with sel sets 1-9 (L_THIN_FG, max_iters=0)
    let fg_sets = [
        ThinSelSet::Set4cc1,
        ThinSelSet::Set4cc2,
        ThinSelSet::Set4cc3,
        ThinSelSet::Set48,
        ThinSelSet::Set8cc1,
        ThinSelSet::Set8cc2,
        ThinSelSet::Set8cc3,
        ThinSelSet::Set8cc4,
        ThinSelSet::Set8cc5,
    ];

    for set in fg_sets {
        let sels = make_thin_sels(set);
        let thinned = thin_connected_by_set(&pix2, ThinType::Foreground, &sels, 0)
            .expect("thin_connected_by_set FG");
        rp.write_pix_and_check(&thinned, ImageFormat::Png)
            .expect("write thinned FG");
    }

    // C checks 9-10: thin with sel sets 10,11 (L_THIN_BG, 5 iterations)
    let bg_sets = [ThinSelSet::Thicken4cc, ThinSelSet::Thicken8cc];
    for set in bg_sets {
        let sels = make_thin_sels(set);
        let thinned = thin_connected_by_set(&pix2, ThinType::Background, &sels, 5)
            .expect("thin_connected_by_set BG");
        rp.write_pix_and_check(&thinned, ImageFormat::Png)
            .expect("write thinned BG");
    }

    assert!(rp.cleanup(), "ccthin2 thin_by_set test failed");
}

/// C checks 11-18: tiled display and stroke width normalization — Rust未実装
#[test]
#[ignore = "not yet implemented: pixaDisplayTiledAndScaled / pixaSetStrokeWidth not available"]
fn ccthin2_reg_stroke_width() {
    // C: pixaDisplayTiledAndScaled(pixa, 8, 500, 1, 0, 25, 2)
    //    pixaSetStrokeWidth(pixa3, 5, 1, 8)
}
