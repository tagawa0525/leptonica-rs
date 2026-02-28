//! Checkerboard regression test
//!
//! Tests detection of checkerboard corner points where four squares meet.
//! The C version uses pixFindCheckerboardCorners to locate corner points
//! on two test images and verifies detection counts.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/checkerboard_reg.c`

use crate::common::RegParams;

/// Test checkerboard corner detection (C checks 0-3).
///
/// C version: pixFindCheckerboardCorners(pix, 15, 3, nsels) with
/// nsels=2 for checkerboard1.tif and nsels=4 for checkerboard2.tif.
#[test]
fn checkerboard_reg_find_corners() {
    let _rp = RegParams::new("checkerboard");

    // checkerboard1.tif with nsels=2
    let pix1 = crate::common::load_test_image("checkerboard1.tif").expect("load checkerboard1.tif");
    let (_corner_pix1, pta1) = leptonica::region::find_checkerboard_corners(&pix1, 15, 3, 2)
        .expect("find_checkerboard_corners checkerboard1");
    eprintln!("checkerboard1.tif corners: {}", pta1.len());
    assert!(
        !pta1.is_empty(),
        "should detect corners in checkerboard1.tif"
    );

    // checkerboard2.tif with nsels=4
    let pix2 = crate::common::load_test_image("checkerboard2.tif").expect("load checkerboard2.tif");
    let (_corner_pix2, pta2) = leptonica::region::find_checkerboard_corners(&pix2, 15, 3, 4)
        .expect("find_checkerboard_corners checkerboard2");
    eprintln!("checkerboard2.tif corners: {}", pta2.len());
    assert!(
        !pta2.is_empty(),
        "should detect corners in checkerboard2.tif"
    );
}
