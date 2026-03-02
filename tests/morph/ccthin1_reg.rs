//! Connected-component thinning 1 regression test
//!
//! C version: reference/leptonica/prog/ccthin1_reg.c
//! Tests generation and display of thinning structuring element sets,
//! and applies thinning to a clipped region of feyn.tif.
//!
//! C checkpoint mapping (11 total):
//!   0-2: selaDisplayInPix for 4cc, 8cc, 4and8cc SEL sets (Rust未実装)
//!   3-4: SEL rotation display (Rust未実装)
//!   5:   clipped region of feyn.tif
//!   6:   pixThinConnected(FG, 4, 0)
//!   7:   pixThinConnected(BG, 4, 0)
//!   8:   pixThinConnected(FG, 8, 0)
//!   9:   pixThinConnected(BG, 8, 0)
//!   10:  tiled display (Rust未実装)

use crate::common::{RegParams, load_test_image};
use leptonica::io::ImageFormat;
use leptonica::morph::{
    Connectivity, ThinSelSet, ThinType, make_thin_sels, sels_4and8cc_thin, sels_4cc_thin,
    sels_8cc_thin, thin_connected,
};

/// Test sels_4cc_thin: generate 4-connected component thinning sels (C check 0).
#[test]
fn ccthin1_reg_4cc_sels() {
    let mut rp = RegParams::new("cthin1_4cc");

    let sels = sels_4cc_thin();
    rp.compare_values(9.0, sels.len() as f64, 0.0);

    for sel in &sels {
        rp.compare_values(1.0, if sel.hit_count() > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "ccthin1 4cc sels test failed");
}

/// Test sels_8cc_thin: generate 8-connected component thinning sels (C check 1).
#[test]
fn ccthin1_reg_8cc_sels() {
    let mut rp = RegParams::new("cthin1_8cc");

    let sels = sels_8cc_thin();
    rp.compare_values(9.0, sels.len() as f64, 0.0);

    for sel in &sels {
        rp.compare_values(1.0, if sel.hit_count() > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "ccthin1 8cc sels test failed");
}

/// Test sels_4and8cc_thin: generate 4+8 cc preserving thinning sels (C check 2).
#[test]
fn ccthin1_reg_4and8cc_sels() {
    let mut rp = RegParams::new("cthin1_48cc");

    let sels = sels_4and8cc_thin();
    rp.compare_values(2.0, sels.len() as f64, 0.0);

    for sel in &sels {
        rp.compare_values(1.0, if sel.hit_count() > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "ccthin1 4and8cc sels test failed");
}

/// Test make_thin_sels and sel rotation.
#[test]
fn ccthin1_reg_make_thin_sels() {
    let mut rp = RegParams::new("cthin1_make");

    let sels4cc1 = make_thin_sels(ThinSelSet::Set4cc1);
    rp.compare_values(3.0, sels4cc1.len() as f64, 0.0);

    let sels8cc1 = make_thin_sels(ThinSelSet::Set8cc1);
    rp.compare_values(1.0, if !sels8cc1.is_empty() { 1.0 } else { 0.0 }, 0.0);

    let sels48 = make_thin_sels(ThinSelSet::Set48);
    rp.compare_values(1.0, if !sels48.is_empty() { 1.0 } else { 0.0 }, 0.0);

    let sels4 = sels_4cc_thin();
    if !sels4.is_empty() {
        let rotated = sels4[0].rotate_orth(1);
        rp.compare_values(sels4[0].hit_count() as f64, rotated.hit_count() as f64, 0.0);
    }

    assert!(rp.cleanup(), "ccthin1 make_thin_sels test failed");
}

/// Test thinning on a region of feyn.tif (C checks 5-9).
///
/// C: pix1 = pixClipRectangle(pixs, box1, NULL);  // box(683, 799, 970, 479)
///    pixThinConnected(pix1, L_THIN_FG/BG, 4/8, 0)
#[test]
fn ccthin1_reg_thinning() {
    let mut rp = RegParams::new("cthin1_thin");

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    // C: box1 = boxCreate(683, 799, 970, 479)
    let pix1 = pixs
        .clip_rectangle(683, 799, 970, 479)
        .expect("clip rectangle");

    // C check 6: pixThinConnected(pix1, L_THIN_FG, 4, 0)
    let thin_fg4 =
        thin_connected(&pix1, ThinType::Foreground, Connectivity::Four, 0).expect("thin FG 4-cc");
    rp.write_pix_and_check(&thin_fg4, ImageFormat::Png)
        .expect("write thin FG 4-cc");

    // C check 7: pixThinConnected(pix1, L_THIN_BG, 4, 0)
    let thin_bg4 =
        thin_connected(&pix1, ThinType::Background, Connectivity::Four, 0).expect("thin BG 4-cc");
    rp.write_pix_and_check(&thin_bg4, ImageFormat::Png)
        .expect("write thin BG 4-cc");

    // C check 8: pixThinConnected(pix1, L_THIN_FG, 8, 0)
    let thin_fg8 =
        thin_connected(&pix1, ThinType::Foreground, Connectivity::Eight, 0).expect("thin FG 8-cc");
    rp.write_pix_and_check(&thin_fg8, ImageFormat::Png)
        .expect("write thin FG 8-cc");

    // C check 9: pixThinConnected(pix1, L_THIN_BG, 8, 0)
    let thin_bg8 =
        thin_connected(&pix1, ThinType::Background, Connectivity::Eight, 0).expect("thin BG 8-cc");
    rp.write_pix_and_check(&thin_bg8, ImageFormat::Png)
        .expect("write thin BG 8-cc");

    // Verify anti-extensive property: thinned ⊆ original
    rp.compare_values(
        1.0,
        if thin_fg4.count_pixels() <= pix1.count_pixels() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if thin_fg8.count_pixels() <= pix1.count_pixels() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "ccthin1 thinning test failed");
}

/// C checks 0-4, 10: selaDisplayInPix — Rust未実装
#[test]
#[ignore = "not yet implemented: selaDisplayInPix not available"]
fn ccthin1_reg_sela_display() {
    // C: pix1 = selaDisplayInPix(sela4, 35, 3, 15, 3);
    //    regTestWritePixAndCheck(rp, pix1, IFF_PNG);  /* 0-4 */
}
