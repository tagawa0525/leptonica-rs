//! PDF segmented output regression test
//!
//! Tests PDF generation with mixed-raster encoding where text regions
//! use higher resolution and image regions use JPEG compression.
//!
//! The C version requires convertSegmentedFilesToPdf, pixConvertTo1,
//! pixExpandBinaryPower2, pixGenerateHalftoneMask, and related APIs
//! which are not available in Rust. This file documents the C test
//! structure with available partial tests using basic PDF output.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pdfseg_reg.c`

use leptonica_test::RegParams;

/// Test basic PDF output of images that would be segmented in C (partial).
///
/// Since segmented PDF is not available, verifies that individual images
/// can be written to PDF (the prerequisite for segmented output).
#[test]
fn pdfseg_reg_basic_pdf_output() {
    let mut rp = RegParams::new("pdfseg_basic");

    // Test that the images used in C segmented tests can be read and written
    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    let opts = leptonica_io::pdf::PdfOptions::default();
    let data = leptonica_io::pdf::write_pdf_mem(&pix, &opts).expect("write_pdf_mem");
    let pdf_str = String::from_utf8_lossy(&data);

    rp.compare_values(
        1.0,
        if pdf_str.starts_with("%PDF-") {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(1.0, if data.len() > 100 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pdfseg basic pdf output test failed");
}

/// Test segmented PDF with region detection (C checks 0-7).
///
/// Requires convertSegmentedFilesToPdf, pixConvertTo1,
/// pixExpandBinaryPower2, pixGenerateHalftoneMask, pixConnComp.
#[test]
#[ignore = "not yet implemented: convertSegmentedFilesToPdf not available"]
fn pdfseg_reg_segmented_output() {
    // C version:
    // 1. pixScaleToSize() to normalize widths
    // 2. pixConvertTo1() for binary conversion
    // 3. pixMorphSequence() for region detection
    // 4. pixExpandBinaryPower2() for mask scaling
    // 5. convertSegmentedFilesToPdf() for mixed-raster output
}

/// Test PDF segmentation with color quantization (C additional checks).
///
/// Requires pixOctreeColorQuant, pixBackgroundNormSimple.
#[test]
#[ignore = "not yet implemented: color quantization for PDF segmentation not available"]
fn pdfseg_reg_color_quantized() {
    // C version:
    // 1. pixBackgroundNormSimple() for normalization
    // 2. pixOctreeColorQuant() for color reduction
    // 3. convertToPdfSegmented() with quantized regions
}
