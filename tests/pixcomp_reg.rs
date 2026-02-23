//! Compressed Pix (PixComp) regression test
//!
//! Tests compressed pix and compressed pix arrays in memory.
//!
//! The C version tests PixComp creation, round-trip compression,
//! PixAComp array operations, serialization, and PDF generation.
//! PixComp/PixAComp types are not implemented in Rust.
//! This test covers available Pixa operations as a partial substitute.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pixcomp_reg.c`

mod common;
use common::RegParams;
use leptonica::Pixa;

/// Test Pixa array operations as partial substitute for PixAComp (C checks 0-2).
///
/// While PixComp provides in-memory compression, Pixa stores uncompressed images.
/// This test verifies basic Pixa array construction and element access.
#[test]
fn pixcomp_reg_pixa_array() {
    let mut rp = RegParams::new("pixcomp_pixa");

    let images = ["marge.jpg", "weasel4.16c.png", "weasel8.149g.png"];

    let mut pixa = Pixa::new();
    for img in &images {
        let pix = common::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        pixa.push(pix);
    }

    // Verify count
    rp.compare_values(images.len() as f64, pixa.len() as f64, 0.0);

    // Verify element access preserves dimensions
    for (i, img) in images.iter().enumerate() {
        let pix_orig = common::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        let pix_ref = pixa.get(i).unwrap_or_else(|| panic!("get {i}"));
        rp.compare_values(pix_orig.width() as f64, pix_ref.width() as f64, 0.0);
        rp.compare_values(pix_orig.height() as f64, pix_ref.height() as f64, 0.0);
    }

    assert!(rp.cleanup(), "pixcomp pixa array test failed");
}

/// Test PixComp creation and round-trip (C checks 0-5).
///
/// Requires PixComp/pixcompCreateFromPix/pixCreateFromPixcomp which are not available.
#[test]
#[ignore = "not yet implemented: PixComp type not available"]
fn pixcomp_reg_create_roundtrip() {
    // C version:
    // 1. pixcompCreateFromPix(pix, IFF_JFIF_JPEG)
    // 2. pixCreateFromPixcomp(pixc) to decompress
    // 3. Verify round-trip fidelity
}

/// Test PixAComp serialization (C checks 6-8).
///
/// Requires PixAComp type and pixacompWrite/pixacompRead which are not available.
#[test]
#[ignore = "not yet implemented: PixAComp serialization not available"]
fn pixcomp_reg_serialization() {
    // C version:
    // 1. pixacompWrite to file
    // 2. pixacompRead from file
    // 3. Verify round-trip
}

/// Test PixAComp join and PDF generation (C checks 9-11).
///
/// Requires PixAComp join, pixacompConvertToPdfData which are not available.
#[test]
#[ignore = "not yet implemented: PixAComp join/PDF conversion not available"]
fn pixcomp_reg_join_pdf() {
    // C version:
    // 1. pixacompJoin to concatenate arrays
    // 2. pixacompConvertToPdfData for PDF output
    // 3. pixacompFastConvertToPdfData for fast variant
}
