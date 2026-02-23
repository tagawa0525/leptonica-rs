//! Connected-component thinning 2 regression test
//!
//! Tests thin_connected and thin_connected_by_set on binary images.
//! The C version applies pixThinConnectedBySet with various sel sets
//! to document images with 4-cc and 8-cc preservation.
//!
//! Partial migration: thin_connected and thin_connected_by_set are tested.
//! Display operations are not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/ccthin2_reg.c`

use leptonica_core::PixelDepth;
use leptonica_morph::{
    Connectivity, ThinSelSet, ThinType, make_thin_sels, thin_connected, thin_connected_by_set,
};
use leptonica_test::RegParams;

/// Test thin_connected 4-way on a binary image.
///
/// C: pix1 = pixThinConnected(pixs, L_THIN_FG, 4, 0);
#[test]
fn ccthin2_reg_thin_4cc() {
    let mut rp = RegParams::new("cthin2_4cc");

    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let w = pix.width();
    let h = pix.height();

    // C: pix1 = pixThinConnected(pixs, L_THIN_FG, 4, 0);
    let thinned = thin_connected(&pix, ThinType::Foreground, Connectivity::Four, 0)
        .expect("thin_connected 4-way");
    rp.compare_values(w as f64, thinned.width() as f64, 0.0);
    rp.compare_values(h as f64, thinned.height() as f64, 0.0);
    assert_eq!(thinned.depth(), PixelDepth::Bit1);

    // Thinning should not add pixels (result is subset of original)
    // Count: thinned should have fewer or equal foreground pixels
    rp.compare_values(
        1.0,
        if thinned.count_pixels() <= pix.count_pixels() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "ccthin2 thin_4cc test failed");
}

/// Test thin_connected 8-way on a binary image.
///
/// C: pix1 = pixThinConnected(pixs, L_THIN_FG, 8, 0);
#[test]
fn ccthin2_reg_thin_8cc() {
    let mut rp = RegParams::new("cthin2_8cc");

    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let w = pix.width();
    let h = pix.height();

    let thinned = thin_connected(&pix, ThinType::Foreground, Connectivity::Eight, 0)
        .expect("thin_connected 8-way");
    rp.compare_values(w as f64, thinned.width() as f64, 0.0);
    rp.compare_values(h as f64, thinned.height() as f64, 0.0);

    assert!(rp.cleanup(), "ccthin2 thin_8cc test failed");
}

/// Test thin_connected_by_set with multiple sel sets (C: pixThinConnectedBySet).
///
/// C: pix1 = pixThinConnectedBySet(pixs, L_THIN_FG, sela, 0);
/// Tests Set4cc1, Set4cc2, Set8cc1, Set48.
#[test]
fn ccthin2_reg_thin_by_set() {
    let mut rp = RegParams::new("cthin2_set");

    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let w = pix.width();
    let h = pix.height();

    let sets = [
        ThinSelSet::Set4cc1,
        ThinSelSet::Set4cc2,
        ThinSelSet::Set4cc3,
        ThinSelSet::Set8cc1,
        ThinSelSet::Set48,
    ];

    for set in sets {
        let sels = make_thin_sels(set);
        let thinned = thin_connected_by_set(&pix, ThinType::Foreground, &sels, 0)
            .expect("thin_connected_by_set");
        rp.compare_values(w as f64, thinned.width() as f64, 0.0);
        rp.compare_values(h as f64, thinned.height() as f64, 0.0);
        // Thinning preserves connectivity — must not add pixels
        rp.compare_values(
            1.0,
            if thinned.count_pixels() <= pix.count_pixels() {
                1.0
            } else {
                0.0
            },
            0.0,
        );
    }

    assert!(rp.cleanup(), "ccthin2 thin_by_set test failed");
}

/// Test background thinning (L_THIN_BG equivalent).
///
/// C: pix1 = pixThinConnectedBySet(pixs, L_THIN_BG, sela, 5);
#[test]
fn ccthin2_reg_thin_bg() {
    let mut rp = RegParams::new("cthin2_bg");

    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let w = pix.width();
    let h = pix.height();

    let sels = make_thin_sels(ThinSelSet::Set4cc1);
    let thinned = thin_connected_by_set(&pix, ThinType::Background, &sels, 5)
        .expect("thin_connected_by_set BG");
    rp.compare_values(w as f64, thinned.width() as f64, 0.0);
    rp.compare_values(h as f64, thinned.height() as f64, 0.0);

    assert!(rp.cleanup(), "ccthin2 thin_bg test failed");
}
