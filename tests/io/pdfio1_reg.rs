//! PDF I/O regression test (part 1)
//!
//! Tests basic PDF generation: single image, multiple compression modes,
//! multi-page documents, and memory output.
//!
//! The C version also tests segmented images, low-level CI data,
//! G4 image masking, and colormap handling which are not available.
//!
//! Rust追加:
//!   write_pix_and_check: PDF変換前の入力画像を golden 化
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pdfio1_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::io::pdf::{PdfCompression, PdfOptions};

/// Test basic PDF output with auto compression (C checks 0-2).
///
/// Writes images of various depths, verifies PDF header and structure.
#[test]
fn pdfio1_reg_auto_compression() {
    let mut rp = RegParams::new("pdfio1_auto");

    let images = [
        ("feyn.tif", "1bpp"),
        ("karen8.jpg", "8bpp"),
        ("marge.jpg", "32bpp"),
    ];

    for (img, _label) in &images {
        let pix = crate::common::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));

        // Golden check: input image before PDF conversion
        let golden_fmt = if pix.depth().bits() == 1 {
            ImageFormat::Tiff
        } else {
            ImageFormat::Png
        };
        rp.write_pix_and_check(&pix, golden_fmt)
            .expect("write PDF input image");

        let opts = PdfOptions::default();
        let data = leptonica::io::pdf::write_pdf_mem(&pix, &opts).expect("write_pdf_mem");
        let header = String::from_utf8_lossy(&data[..8.min(data.len())]);

        // PDF should start with %PDF-
        rp.compare_values(
            1.0,
            if header.starts_with("%PDF-") {
                1.0
            } else {
                0.0
            },
            0.0,
        );

        // Should have non-trivial size
        rp.compare_values(1.0, if data.len() > 100 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "pdfio1 auto compression test failed");
}

/// Test PDF with Flate compression (C checks 3-5).
///
/// Writes images with explicit Flate compression.
#[test]
fn pdfio1_reg_flate() {
    let mut rp = RegParams::new("pdfio1_flate");

    let images = [
        "feyn.tif",   // 1bpp
        "karen8.jpg", // 8bpp
        "marge.jpg",  // 32bpp
    ];

    let opts = PdfOptions::default().compression(PdfCompression::Flate);

    for img in &images {
        let pix = crate::common::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        let data = leptonica::io::pdf::write_pdf_mem(&pix, &opts).expect("write_pdf_mem flate");

        // PDF header
        let header = String::from_utf8_lossy(&data[..8.min(data.len())]);
        rp.compare_values(
            1.0,
            if header.starts_with("%PDF-") {
                1.0
            } else {
                0.0
            },
            0.0,
        );

        // Should have FlateDecode in the stream
        let pdf_str = String::from_utf8_lossy(&data);
        let has_flate = pdf_str.contains("FlateDecode");
        rp.compare_values(1.0, if has_flate { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "pdfio1 flate test failed");
}

/// Test PDF with JPEG compression (C checks 6-8).
///
/// Writes 8bpp and 32bpp images with JPEG compression.
/// Note: 1bpp falls back to Flate since JPEG is unsuitable for binary.
#[test]
fn pdfio1_reg_jpeg() {
    let mut rp = RegParams::new("pdfio1_jpeg");

    let opts = PdfOptions::default().compression(PdfCompression::Jpeg);

    // 8bpp grayscale
    let pix8 = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let data8 = leptonica::io::pdf::write_pdf_mem(&pix8, &opts).expect("write_pdf_mem jpeg 8bpp");
    let pdf_str8 = String::from_utf8_lossy(&data8);

    rp.compare_values(
        1.0,
        if pdf_str8.starts_with("%PDF-") {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    // JPEG compression uses DCTDecode
    let has_dct = pdf_str8.contains("DCTDecode");
    rp.compare_values(1.0, if has_dct { 1.0 } else { 0.0 }, 0.0);

    // 32bpp RGB
    let pix32 = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let data32 =
        leptonica::io::pdf::write_pdf_mem(&pix32, &opts).expect("write_pdf_mem jpeg 32bpp");
    let pdf_str32 = String::from_utf8_lossy(&data32);

    rp.compare_values(
        1.0,
        if pdf_str32.starts_with("%PDF-") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "pdfio1 jpeg test failed");
}

/// Test multi-page PDF generation (C checks 9-10).
///
/// Creates a PDF with multiple images, verifies page count.
#[test]
fn pdfio1_reg_multipage() {
    let mut rp = RegParams::new("pdfio1_multipage");

    let pix1 = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pix2 = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let pix3 = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");

    let images: Vec<&leptonica::io::Pix> = vec![&pix1, &pix2, &pix3];
    let opts = PdfOptions::default();

    let mut buf = Vec::new();
    leptonica::io::pdf::write_pdf_multi(&images, &mut buf, &opts).expect("write_pdf_multi");

    // Should start with PDF header
    let header = String::from_utf8_lossy(&buf[..8.min(buf.len())]);
    rp.compare_values(
        1.0,
        if header.starts_with("%PDF-") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Count /MediaBox entries (one per page, not present in /Pages catalog)
    let pdf_str = String::from_utf8_lossy(&buf);
    let page_count = pdf_str.matches("/MediaBox").count();
    rp.compare_values(3.0, page_count as f64, 0.0);

    // Should have non-trivial size
    rp.compare_values(1.0, if buf.len() > 1000 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pdfio1 multipage test failed");
}

/// Test PDF with custom title (additional).
///
/// Verifies the title metadata appears in the PDF output.
#[test]
fn pdfio1_reg_title() {
    let mut rp = RegParams::new("pdfio1_title");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let opts = PdfOptions::with_title("Test Document");
    let data = leptonica::io::pdf::write_pdf_mem(&pix, &opts).expect("write_pdf_mem with title");
    let pdf_str = String::from_utf8_lossy(&data);

    // Should contain the title
    let has_title = pdf_str.contains("Test Document");
    rp.compare_values(1.0, if has_title { 1.0 } else { 0.0 }, 0.0);

    // Should be valid PDF
    rp.compare_values(
        1.0,
        if pdf_str.starts_with("%PDF-") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "pdfio1 title test failed");
}

/// Test segmented PDF with region masking (C checks 11-19).
///
/// Requires convertToPdfSegmented with boxa and region detection.
#[test]
#[ignore = "not yet implemented: convertToPdfSegmented not available"]
fn pdfio1_reg_segmented() {
    // C version:
    // 1. Detect image/text regions via morphology
    // 2. convertToPdfSegmented() with different encoding per region
    // 3. Verify mixed-raster output
}

/// Test low-level CI data generation (C checks 23-26).
///
/// Requires pixGenerateCIData, cidConvertToPdfData.
#[test]
#[ignore = "not yet implemented: CI data generation not available"]
fn pdfio1_reg_ci_data() {
    // C version:
    // 1. pixGenerateCIData() to create compound image data
    // 2. cidConvertToPdfData() to convert to PDF bytes
    // 3. Verify PDF structure
}

/// Test G4 image masking (C checks 20-22).
///
/// Requires l_pdfSetG4ImageMask state control.
#[test]
#[ignore = "not yet implemented: G4 image mask control not available"]
fn pdfio1_reg_g4_mask() {
    // C version:
    // 1. l_pdfSetG4ImageMask(1) to enable masking
    // 2. Generate PDF with masked G4 images
    // 3. l_pdfSetG4ImageMask(0) to disable
}
