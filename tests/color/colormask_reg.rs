//! Color mask regression test
//!
//! Tests HSV-based color region identification and masking.
//! The C version creates an HS histogram, finds peaks, and builds masks
//! covering color regions in HSV space.
//!
//! Not migrated: All HSV mask functions (pixMakeHistoHS, pixFindHistoPeaksHSV,
//! pixMakeRangeMaskHS) are not available in the Rust API.
//! Test image 1555.003.jpg is also not available.
//!
//! # See also
//!
//! C Leptonica: `prog/colormask_reg.c`

/// Test HSV histogram peak detection and mask generation (C checks 0-10).
///
/// Requires pixMakeHistoHS, pixFindHistoPeaksHSV, and pixMakeRangeMaskHS
/// which are not available in the Rust API.
/// Test image 1555.003.jpg is also not available.
#[test]
#[ignore = "not yet implemented: HSV mask functions (pixMakeHistoHS/pixFindHistoPeaksHSV/pixMakeRangeMaskHS) not available"]
fn colormask_reg_hsv_peaks() {
    // C version:
    // pixhsv = pixConvertRGBToHSV(NULL, pixs);
    // pixh = pixMakeHistoHS(pixhsv, 5, &nahue, &nasat);
    // pixFindHistoPeaksHSV(pixh, L_HS_HISTO, 20, 20, 6, 2.0, &ptapk, &napk, &pixapk);
    // pix1 = pixMakeRangeMaskHS(pixr, y, 20, x, 20, L_INCLUDE_REGION);
}
