//! Checkerboard regression test
//!
//! Tests detection of checkerboard corner points where four squares meet.
//! The C version uses pixFindCheckerboardCorners to locate corner points
//! on two test images and verifies detection counts.
//!
//! Not yet migrated: pixFindCheckerboardCorners is not available in
//! leptonica-transform, and the required test images (checkerboard1.tif,
//! checkerboard2.tif) are not present in the test data directory.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/checkerboard_reg.c`

/// Test checkerboard corner detection (C checks 0-3).
///
/// Requires pixFindCheckerboardCorners and test images checkerboard1.tif,
/// checkerboard2.tif which are not available.
#[test]
#[ignore = "not yet implemented: pixFindCheckerboardCorners not available; test images missing"]
fn checkerboard_reg_find_corners() {
    // C version:
    // 1. pixRead("checkerboard1.tif") and pixRead("checkerboard2.tif")
    // 2. pixFindCheckerboardCorners(pix1, cornersize=7, maxangle=0.2, nsels=10,
    //    &pix2, &pta1, pixa1) for each nsels in {10, 20}
    // 3. pixGenerateFromPta() + pixDilateBrick() to visualize detected corners
    // 4. regTestWritePixAndCheck() for each result
}
