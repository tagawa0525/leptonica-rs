//! JPEG 2000 I/O regression test
//!
//! Tests JP2K format detection from magic bytes.
//!
//! The C version tests read/write round-trips, cropped/scaled reading,
//! and J2K codec variants. JP2K writing is not implemented in Rust,
//! and no JP2K test images are bundled, so only format detection is tested.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/jp2kio_reg.c`

use crate::common::RegParams;

/// Test JP2K format detection from magic bytes.
///
/// Verifies that JP2 file signature is correctly identified.
#[test]
fn jp2kio_reg_format_detection() {
    let mut rp = RegParams::new("jp2kio_format");

    // JP2 magic bytes: 0x0000000C 6A502020 0D0A870A
    let jp2_magic: &[u8] = &[
        0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20, 0x0D, 0x0A, 0x87, 0x0A,
    ];
    let format = leptonica::io::detect_format_from_bytes(jp2_magic);
    rp.compare_values(
        1.0,
        if matches!(format, Ok(leptonica::io::ImageFormat::Jp2)) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "jp2kio format detection test failed");
}

/// Test JP2K header reading from memory (C checks 12-13).
///
/// Requires a bundled JP2K test image which is not available.
#[test]
#[ignore = "not yet implemented: no JP2K test image bundled"]
fn jp2kio_reg_header_reading() {
    // Requires test.jp2 or similar JP2K test image in tests/data/images/
}

/// Test JP2K read from memory (C checks 14-15).
///
/// Requires a bundled JP2K test image which is not available.
#[test]
#[ignore = "not yet implemented: no JP2K test image bundled"]
fn jp2kio_reg_memory_read() {
    // Requires test.jp2 or similar JP2K test image in tests/data/images/
}

/// Test JP2K write/read round-trip (C checks 0-1).
///
/// Requires pixWriteJp2k which is not implemented.
#[test]
#[ignore = "not yet implemented: JP2K write not available"]
fn jp2kio_reg_roundtrip() {
    // C version:
    // 1. Read JPEG, scale to 50%
    // 2. pixWriteJp2k() with various quality settings
    // 3. pixReadJp2k() to verify
    // 4. Compare dimensions
}

/// Test JP2K cropped read with bounding box (C checks 2-7).
///
/// Requires pixReadJp2k with box parameter for ROI extraction.
#[test]
#[ignore = "not yet implemented: JP2K cropped read not available"]
fn jp2kio_reg_cropped_read() {
    // C version:
    // 1. pixReadJp2k(fname, reduction, box) with bounding box
    // 2. Verify cropped dimensions
    // 3. Multiple crop regions
}

/// Test JP2K scaled read at different reductions (C checks 8-11).
///
/// Requires pixReadJp2k with reduction factor parameter.
#[test]
#[ignore = "not yet implemented: JP2K scaled read not available"]
fn jp2kio_reg_scaled_read() {
    // C version:
    // 1. pixReadJp2k(fname, 2, NULL) for 2x reduction
    // 2. Verify reduced dimensions
}

/// Test J2K codec variant (C check 16).
///
/// Requires pixWriteStreamJp2k with J2K codec type.
#[test]
#[ignore = "not yet implemented: J2K codec variant not available"]
fn jp2kio_reg_j2k_codec() {
    // C version:
    // 1. pixWriteStreamJp2k() with L_J2K_CODEC
    // 2. Verify J2K codestream output
}
