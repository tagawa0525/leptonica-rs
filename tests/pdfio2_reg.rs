#![cfg(all(feature = "pdf-format", feature = "tiff-format"))]
//! PDF I/O regression test (part 2)
//!
//! Tests advanced PDF operations: multi-page from files,
//! resolution handling, and memory output verification.
//!
//! The C version tests segmented images with region masking,
//! PDF concatenation, batch operations, and corruption recovery
//! which are not available in Rust.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pdfio2_reg.c`

mod common;
use common::RegParams;
use leptonica::io::pdf::PdfOptions;

/// Test PDF from file list (C partial check).
///
/// Writes multiple images from files to a single PDF.
#[test]
fn pdfio2_reg_from_files() {
    let mut rp = RegParams::new("pdfio2_from_files");

    // Write test images to temp files, then create PDF from file list
    let tmpdir = std::path::PathBuf::from(common::regout_dir()).join("pdfio2_from_files");
    std::fs::create_dir_all(&tmpdir).expect("create temp dir");

    let test_images = ["feyn.tif", "karen8.jpg", "marge.jpg"];
    let mut paths = Vec::new();
    for img in &test_images {
        let pix = common::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        let path = tmpdir.join(img);
        let format = match *img {
            s if s.ends_with(".tif") => leptonica::io::ImageFormat::Tiff,
            s if s.ends_with(".jpg") => leptonica::io::ImageFormat::Jpeg,
            _ => leptonica::io::ImageFormat::Png,
        };
        leptonica::io::write_image(&pix, &path, format).expect("write temp image");
        paths.push(path);
    }

    let opts = PdfOptions::with_title("Multi-file PDF");
    let mut buf = Vec::new();
    leptonica::io::pdf::write_pdf_from_files(&paths, &mut buf, &opts)
        .expect("write_pdf_from_files");

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

    // Should have non-trivial size
    rp.compare_values(1.0, if buf.len() > 1000 { 1.0 } else { 0.0 }, 0.0);

    // Clean up
    if let Err(e) = std::fs::remove_dir_all(&tmpdir) {
        eprintln!("Failed to remove temporary directory {tmpdir:?}: {e}");
    }

    assert!(rp.cleanup(), "pdfio2 from files test failed");
}

/// Test PDF memory output round-trip verification (additional).
///
/// Generates PDF to memory and verifies basic structure.
#[test]
fn pdfio2_reg_memory_output() {
    let mut rp = RegParams::new("pdfio2_memory");

    let pix = common::load_test_image("marge.jpg").expect("load marge.jpg");
    let opts = PdfOptions::default().resolution(150);
    let data = leptonica::io::pdf::write_pdf_mem(&pix, &opts).expect("write_pdf_mem");

    // Valid PDF header
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

    // Should end with %%EOF
    let has_eof = pdf_str.trim_end().ends_with("%%EOF");
    rp.compare_values(1.0, if has_eof { 1.0 } else { 0.0 }, 0.0);

    // Should contain xref table
    let has_xref = pdf_str.contains("xref");
    rp.compare_values(1.0, if has_xref { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pdfio2 memory output test failed");
}

/// Test segmented PDF with region detection (C checks 0-18).
///
/// Requires convertToPdfSegmented with boxa, halftone mask generation,
/// and morphological region detection.
#[test]
#[ignore = "not yet implemented: segmented PDF with region masking not available"]
fn pdfio2_reg_segmented() {
    // C version:
    // 1. pixGenerateHalftoneMask() for region detection
    // 2. pixMorphSequence() for cleanup
    // 3. pixConnComp() for bounding boxes
    // 4. convertToPdfSegmented() for mixed-raster PDF
}

/// Test PDF concatenation (C check 19).
///
/// Requires concatenatePdf with pattern matching.
#[test]
#[ignore = "not yet implemented: PDF concatenation not available"]
fn pdfio2_reg_concatenate() {
    // C version:
    // 1. Generate multiple single-page PDFs
    // 2. concatenatePdf() to merge them
    // 3. Verify merged output
}

/// Test PDF batch from directory (C additional checks).
///
/// Requires convertFilesToPdf with directory scanning.
#[test]
#[ignore = "not yet implemented: batch PDF from directory not available"]
fn pdfio2_reg_batch() {
    // C version:
    // 1. convertFilesToPdf() with directory of images
    // 2. Pattern matching for file selection
}

/// Test PDF corruption recovery (C additional checks).
///
/// Requires PDF reader/parser for header inspection and repair.
#[test]
#[ignore = "not yet implemented: PDF corruption recovery not available"]
fn pdfio2_reg_corruption() {
    // C version:
    // 1. Create corrupted PDF (missing headers)
    // 2. Attempt parse and recover
    // 3. Verify recovery success
}
