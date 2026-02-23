//! Circle regression test
//!
//! Tests extraction of digits embedded within circular regions using
//! erosion and connected-component counting. The C version reads a
//! Pixa archive of pre-rendered circles, uses seedfill to isolate
//! circle interiors, and counts components at each erosion step.
//!
//! Not yet migrated: the test data file circles.pa (Pixa archive) is
//! not present in the test data directory.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/circle_reg.c`

/// Test circle extraction using erosion and connected components (C checks 0-1).
///
/// Requires circles.pa Pixa archive which is not present in test data.
#[test]
#[ignore = "not yet implemented: circles.pa test data not available"]
fn circle_reg_extract_circles() {
    // C version:
    // 1. pixaRead("circles.pa") to load pre-rendered circle images
    // 2. pixInvert + pixCreateTemplate + pixSetOrClearBorder + pixSeedfillBinary
    //    to isolate circle interior
    // 3. pixAnd + pixCountConnComp to count components in and around circles
    // 4. pixErodeBrick to find boundary transitions
    // 5. regTestCompareValues() for component counts at erosion thresholds
}
