//! Connected-component thinning 1 regression test
//!
//! Tests generation and display of thinning structuring element sets.
//! The C version generates sela4cc, sela8cc, sela4and8cc sel sets
//! and displays them using selaDisplayInPix.
//!
//! Partial migration: sels_4cc_thin, sels_8cc_thin, sels_4and8cc_thin,
//! and make_thin_sels are tested. selaDisplayInPix is not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/ccthin1_reg.c`

use crate::common::RegParams;
use leptonica::morph::{
    ThinSelSet, make_thin_sels, sels_4and8cc_thin, sels_4cc_thin, sels_8cc_thin,
};

/// Test sels_4cc_thin: generate 4-connected component thinning sels (C check 0).
///
/// C: sela4 = sela4ccThin(NULL); pix1 = selaDisplayInPix(sela4, 35, 3, 15, 3);
#[test]
fn ccthin1_reg_4cc_sels() {
    let mut rp = RegParams::new("cthin1_4cc");

    let sels = sels_4cc_thin();
    // There are 9 4-cc thinning sels (SEL_4_1 through SEL_4_9)
    rp.compare_values(9.0, sels.len() as f64, 0.0);

    // Each sel should have at least one hit element
    for sel in &sels {
        rp.compare_values(1.0, if sel.hit_count() > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "ccthin1 4cc sels test failed");
}

/// Test sels_8cc_thin: generate 8-connected component thinning sels (C check 1).
///
/// C: sela8 = sela8ccThin(NULL); pix1 = selaDisplayInPix(sela8, 35, 3, 15, 3);
#[test]
fn ccthin1_reg_8cc_sels() {
    let mut rp = RegParams::new("cthin1_8cc");

    let sels = sels_8cc_thin();
    // There are 9 8-cc thinning sels (SEL_8_1 through SEL_8_9)
    rp.compare_values(9.0, sels.len() as f64, 0.0);

    for sel in &sels {
        rp.compare_values(1.0, if sel.hit_count() > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "ccthin1 8cc sels test failed");
}

/// Test sels_4and8cc_thin: generate 4+8 cc preserving thinning sels (C check 2).
///
/// C: sela48 = sela4and8ccThin(NULL);
#[test]
fn ccthin1_reg_4and8cc_sels() {
    let mut rp = RegParams::new("cthin1_48cc");

    let sels = sels_4and8cc_thin();
    // There are 2 4-and-8-cc preserving sels (SEL_48_1, SEL_48_2)
    rp.compare_values(2.0, sels.len() as f64, 0.0);

    for sel in &sels {
        rp.compare_values(1.0, if sel.hit_count() > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "ccthin1 4and8cc sels test failed");
}

/// Test make_thin_sels: create named sel sets (C: selaMakeThinSets equivalent).
///
/// C: various sel rotation tests from sela4ccThin and selRotateOrth
#[test]
fn ccthin1_reg_make_thin_sels() {
    let mut rp = RegParams::new("cthin1_make");

    // Set4cc1 contains 3 sels
    let sels4cc1 = make_thin_sels(ThinSelSet::Set4cc1);
    rp.compare_values(3.0, sels4cc1.len() as f64, 0.0);

    // Set8cc1 also has sels
    let sels8cc1 = make_thin_sels(ThinSelSet::Set8cc1);
    rp.compare_values(1.0, if !sels8cc1.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // Set48 has sels
    let sels48 = make_thin_sels(ThinSelSet::Set48);
    rp.compare_values(1.0, if !sels48.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // Verify rotate_orth works on a sel (C: selRotateOrth)
    let sels4 = sels_4cc_thin();
    if !sels4.is_empty() {
        let rotated = sels4[0].rotate_orth(1);
        // Rotated sel should have the same element count
        rp.compare_values(sels4[0].hit_count() as f64, rotated.hit_count() as f64, 0.0);
    }

    assert!(rp.cleanup(), "ccthin1 make_thin_sels test failed");
}
