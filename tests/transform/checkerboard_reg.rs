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
//! C Leptonica: `reference/leptonica/prog/checkerboard_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::morph::dilate_brick;
use leptonica::{Pix, PixelDepth};

/// Generate a 1bpp image from Pta coordinates.
///
/// Equivalent to C's `pixGenerateFromPta(pta, w, h)`.
fn generate_pix_from_pta(pta: &leptonica::Pta, w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).expect("create pix for pta");
    let mut pm = pix.try_into_mut().expect("mutable pix for pta");
    for i in 0..pta.len() {
        let (x, y) = pta.get(i).expect("get pta point");
        let xi = x.round() as u32;
        let yi = y.round() as u32;
        if xi < w && yi < h {
            pm.set_pixel_unchecked(xi, yi, 1);
        }
    }
    pm.into()
}

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
    let pta_pix = generate_pix_from_pta(&pta, w, h);
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
