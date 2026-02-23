//! SPIX serialization regression test
//!
//! Tests fast (uncompressed) serialization of Pix to memory bytes
//! and deserialization back to Pix, across all supported bit depths.
//!
//! The C version serializes 10 image types to/from memory,
//! writes SPIX files for clipped sub-images, and reads headers.
//! This Rust port tests memory serialization, header reading,
//! and file round-trips using available APIs.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pixserial_reg.c`

use leptonica_core::Pix;
use leptonica_test::RegParams;

/// Images from the C test (all bit depths).
const IMAGES: &[&str] = &[
    "feyn.tif",         // 1 bpp
    "dreyfus2.png",     // 2 bpp cmapped
    "dreyfus4.png",     // 4 bpp cmapped
    "weasel4.16c.png",  // 4 bpp cmapped
    "dreyfus8.png",     // 8 bpp cmapped
    "weasel8.240c.png", // 8 bpp cmapped
    "karen8.jpg",       // 8 bpp, not cmapped
    "test16.tif",       // 16 bpp
    "marge.jpg",        // rgb
    "test24.jpg",       // rgb
];

/// Test SPIX memory serialization/deserialization round-trip (C checks 0-9).
///
/// For each image: serialize to bytes, deserialize, compare pixels.
#[test]
#[ignore = "not yet implemented"]
fn pixserial_reg_memory_roundtrip() {
    let mut rp = RegParams::new("pixserial_memory");

    for img in IMAGES {
        let pix = leptonica_test::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));

        // Serialize to bytes
        let data = pix.write_spix_to_bytes().expect("write_spix_to_bytes");
        assert!(!data.is_empty(), "{img}: serialized data is empty");

        // Deserialize
        let pix2 = Pix::read_spix_from_bytes(&data).expect("read_spix_from_bytes");

        // Compare dimensions and depth
        rp.compare_values(pix.width() as f64, pix2.width() as f64, 0.0);
        rp.compare_values(pix.height() as f64, pix2.height() as f64, 0.0);
        rp.compare_values(pix.depth().bits() as f64, pix2.depth().bits() as f64, 0.0);

        // Compare pixel data
        rp.compare_pix(&pix, &pix2);
    }

    assert!(rp.cleanup(), "pixserial memory roundtrip test failed");
}

/// Test SPIX file write/read round-trip for clipped sub-images (C checks 10-29).
///
/// For each image: clip a 150×150 region, write to SPIX file, read back, compare.
#[test]
#[ignore = "not yet implemented"]
fn pixserial_reg_file_roundtrip() {
    let mut rp = RegParams::new("pixserial_file");

    let tmpdir = std::env::temp_dir().join("lept_pixserial");
    std::fs::create_dir_all(&tmpdir).ok();

    for (i, img) in IMAGES.iter().enumerate() {
        let pix = leptonica_test::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));

        // Clip to at most 150×150 from top-left
        let clip_w = pix.width().min(150);
        let clip_h = pix.height().min(150);
        let clipped = pix.clip_rectangle(0, 0, clip_w, clip_h).expect("clip");

        // Write SPIX to file
        let path = tmpdir.join(format!("pixs.{i}.spix"));
        clipped
            .write_spix_to_file(&path)
            .expect("write_spix_to_file");

        // Read back
        let pix2 = Pix::read_spix_from_file(&path).expect("read_spix_from_file");

        rp.compare_values(clipped.width() as f64, pix2.width() as f64, 0.0);
        rp.compare_values(clipped.height() as f64, pix2.height() as f64, 0.0);
        rp.compare_pix(&clipped, &pix2);
    }

    // Clean up temp files
    std::fs::remove_dir_all(&tmpdir).ok();

    assert!(rp.cleanup(), "pixserial file roundtrip test failed");
}

/// Test SPIX header reading (C checks 30-39).
///
/// For each image: serialize to SPIX bytes, read header, verify dimensions match.
#[test]
#[ignore = "not yet implemented"]
fn pixserial_reg_header() {
    let mut rp = RegParams::new("pixserial_header");

    for img in IMAGES {
        let pix = leptonica_test::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));

        // Serialize to bytes
        let data = pix.write_spix_to_bytes().expect("write_spix_to_bytes");

        // Read header
        let header = Pix::read_spix_header(&data).expect("read_spix_header");

        rp.compare_values(pix.width() as f64, header.width as f64, 0.0);
        rp.compare_values(pix.height() as f64, header.height as f64, 0.0);
        rp.compare_values(pix.depth().bits() as f64, header.depth as f64, 0.0);
    }

    assert!(rp.cleanup(), "pixserial header test failed");
}
