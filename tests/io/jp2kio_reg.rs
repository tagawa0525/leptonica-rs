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
//! C Leptonica: `prog/jp2kio_reg.c`

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
#[cfg(feature = "jp2k-format")]
fn jp2kio_reg_cropped_read() {
    // Smoke: full JP2K decode tests require a bundled .jp2 fixture, which is
    // not available in-tree. We verify the error paths regardless.
    use leptonica::core::Box;
    use leptonica::io::IoError;
    use leptonica::io::jp2k::read_jp2k_cropped_mem;

    let bad: &[u8] = b"not a jpeg 2000 file";
    let any_box = Box::new(0, 0, 1, 1).unwrap();
    assert!(read_jp2k_cropped_mem(bad, &any_box).is_err());

    // Degenerate box is rejected before the JP2K parser is touched, so the
    // error must be `InvalidData`. We even pass empty bytes to make sure the
    // box check fires first.
    let bad_box = Box::new(0, 0, 0, 0).unwrap();
    let err = read_jp2k_cropped_mem(b"", &bad_box).unwrap_err();
    assert!(
        matches!(err, IoError::InvalidData(_)),
        "expected InvalidData for degenerate box, got {err:?}",
    );
}

/// Test JP2K scaled read at different reductions (C checks 8-11).
///
/// Requires `pixReadJp2k` with reduction factor parameter — Rust port uses
/// `read_jp2k_scaled_mem(data, scale_denom)` backed by hayro-jpeg2000's
/// `target_resolution` hint.
#[test]
#[cfg(feature = "jp2k-format")]
fn jp2kio_reg_scaled_read() {
    use leptonica::io::jp2k::read_jp2k_scaled_mem;
    // Without sample data we can only verify error paths and the
    // scale_denom == 0 contract.
    let bad: &[u8] = b"not a jpeg 2000 file";
    assert!(read_jp2k_scaled_mem(bad, 2).is_err());
    // scale_denom == 0 is rejected before parsing.
    assert!(read_jp2k_scaled_mem(b"", 0).is_err());
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
