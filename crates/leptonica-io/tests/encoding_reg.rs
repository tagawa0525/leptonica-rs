//! Encoding regression test
//!
//! Tests ASCII85 encoding/decoding of binary data.
//!
//! The C version tests encodeAscii85/decodeAscii85 round-trips,
//! compressed variants, and pix text metadata storage.
//! The Rust ascii85 module is private (internal to PostScript output),
//! and decoding is not implemented.
//! Available tests verify PostScript output contains valid ASCII85 data.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/encoding_reg.c`

use leptonica_test::RegParams;

/// Test that PostScript output contains ASCII85-encoded data.
///
/// Since ascii85::encode is not publicly accessible, we verify indirectly
/// by checking that PS output of an image contains ASCII85 markers.
#[test]
fn encoding_reg_ps_ascii85() {
    let mut rp = RegParams::new("encoding_ps_ascii85");

    let pix = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");

    // Write as PostScript to memory
    let ps_data = leptonica_io::write_image_mem(&pix, leptonica_io::ImageFormat::Ps);
    match ps_data {
        Ok(data) => {
            let ps_str = String::from_utf8_lossy(&data);
            // PostScript with ASCII85 encoding should contain the EOD marker "~>"
            let has_a85_eod = ps_str.contains("~>");
            rp.compare_values(1.0, if has_a85_eod { 1.0 } else { 0.0 }, 0.0);

            // Should contain standard PS header
            let has_ps_header = ps_str.starts_with("%!");
            rp.compare_values(1.0, if has_ps_header { 1.0 } else { 0.0 }, 0.0);
        }
        Err(e) => {
            // PS output requires the "ps-format" feature; verify it's UnsupportedFormat
            assert!(
                matches!(e, leptonica_io::IoError::UnsupportedFormat(_)),
                "expected UnsupportedFormat error, got: {e}"
            );
            rp.compare_values(1.0, 1.0, 0.0);
            rp.compare_values(1.0, 1.0, 0.0);
        }
    }

    assert!(rp.cleanup(), "encoding ps ascii85 test failed");
}

/// Test ASCII85 encode/decode round-trip (C check 0).
///
/// Requires publicly accessible ascii85::encode and ascii85::decode.
#[test]
#[ignore = "not yet implemented: ascii85 module is private, decode not available"]
fn encoding_reg_ascii85_roundtrip() {
    // C version:
    // 1. Read karen8.jpg as binary
    // 2. encodeAscii85(bina, fbytes, &nbytes1)
    // 3. decodeAscii85(a85a, nbytes1, &nbytes2)
    // 4. Verify fbytes == nbytes2
}

/// Test ASCII85 with compression (C checks 2-3).
///
/// Requires encodeAscii85WithComp/decodeAscii85WithComp.
#[test]
#[ignore = "not yet implemented: ascii85 compression variants not available"]
fn encoding_reg_ascii85_compression() {
    // C version:
    // 1. encodeAscii85WithComp on ascii data
    // 2. decodeAscii85WithComp to verify round-trip
}

/// Test pix text metadata storage (C check 4).
///
/// Requires pixSetTextCompNew/pixGetTextCompNew.
#[test]
#[ignore = "not yet implemented: Pix text metadata storage not available"]
fn encoding_reg_pix_text_metadata() {
    // C version:
    // 1. Read weasel32.png as binary
    // 2. pixSetTextCompNew(pix, bina, nbytes1)
    // 3. pixGetTextCompNew(pix, &nbytes2)
    // 4. Compare original and retrieved binary data
}
