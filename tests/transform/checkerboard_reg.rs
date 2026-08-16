//! Checkerboard regression test
//!
//! Tests detection of checkerboard corner points where four squares meet.
//! The C version uses pixFindCheckerboardCorners to locate corner points
//! on two test images and verifies detection counts.
//!
//! C version has 6 checks (0-5), 3 per image:
//! - Check 0/3: corner pix from find_checkerboard_corners (WPAC)
//! - Check 1/4: pixaDisplayTiledInColumns of intermediate HMT images
//! - Check 2/5: pixGenerateFromPta + dilate to visualize corners (WPAC)
//!
//! # See also
//!
//! C Leptonica: `prog/checkerboard_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::morph::dilate_brick;

/// Helper to run checkerboard corner detection and register results.
///
/// Mirrors C's `LocateCheckerboardCorners(rp, fname, nsels)`.
fn locate_checkerboard_corners(rp: &mut RegParams, fname: &str, nsels: u32) {
    let pix1 = crate::common::load_test_image(fname).unwrap_or_else(|_| {
        panic!("load {fname}");
    });

    let (corner_pix, pta) = leptonica::region::find_checkerboard_corners(&pix1, 15, 3, nsels)
        .unwrap_or_else(|_| {
            panic!("find_checkerboard_corners {fname}");
        });

    assert!(!pta.is_empty(), "should detect corners in {fname}");
    eprintln!("{fname}: {} corners detected", pta.len());

    // C check 0/3: corner pix (WPAC)
    rp.write_pix_and_check(&corner_pix, ImageFormat::Png)
        .unwrap_or_else(|_| {
            panic!("write corner_pix for {fname}");
        });

    // C check 2/5: generate image from Pta + dilate 5x5 (WPAC)
    let (w, h) = (pix1.width(), pix1.height());
    let pta_pix =
        leptonica::core::pta::graphics::pix_generate_from_pta(&pta, w, h).expect("pta -> pix");
    let dilated = dilate_brick(&pta_pix, 5, 5).unwrap_or_else(|_| {
        panic!("dilate pta_pix for {fname}");
    });
    rp.write_pix_and_check(&dilated, ImageFormat::Png)
        .unwrap_or_else(|_| {
            panic!("write dilated pta_pix for {fname}");
        });
}

#[test]
fn checkerboard_reg() {
    let mut rp = RegParams::new("checkerboard");

    locate_checkerboard_corners(&mut rp, "checkerboard1.tif", 2);
    locate_checkerboard_corners(&mut rp, "checkerboard2.tif", 4);

    assert!(rp.cleanup(), "checkerboard_reg regression test failed");
}

/// Intermediate HMT tiled display for checkerboard1.tif (C check 1).
///
/// C version collects intermediate pixa from pixFindCheckerboardCorners
/// and displays them tiled. The Rust API does not return intermediate images.
#[test]
#[ignore = "pixFindCheckerboardCorners intermediate pixa not available in Rust API"]
fn checkerboard_reg_intermediate_display_1() {
    // C: pixaDisplayTiledInColumns(pixa1, 1, 1.0, 20, 2) for checkerboard1.tif
}

/// Intermediate HMT tiled display for checkerboard2.tif (C check 4).
///
/// C version collects intermediate pixa from pixFindCheckerboardCorners
/// and displays them tiled. The Rust API does not return intermediate images.
#[test]
#[ignore = "pixFindCheckerboardCorners intermediate pixa not available in Rust API"]
fn checkerboard_reg_intermediate_display_2() {
    // C: pixaDisplayTiledInColumns(pixa1, 1, 1.0, 20, 2) for checkerboard2.tif
}
