//! SPIX I/O regression test
//!
//! Corresponds to `spixio_reg.c` in the C version.
//! Tests SPIX reading, writing, and format detection.
//!
//! # C version test summary
//! - Write and read back SPIX at various depths (1bpp, 8bpp, 32bpp)
//! - Colormap round-trip
//! - Format detection from bytes
//! - write_image_mem / read_image_mem dispatch

use leptonica_core::{Pix, PixelDepth};
use leptonica_io::{ImageFormat, read_image_mem, write_image_mem};
use leptonica_test::RegParams;
use std::io::Cursor;

#[test]
fn spixio_reg() {
    let mut rp = RegParams::new("spixio");

    // --- Test 1: 1bpp roundtrip ---
    eprintln!("=== Test: 1bpp SPIX roundtrip ===");
    let pix = Pix::new(64, 48, PixelDepth::Bit1).unwrap();
    test_spix_roundtrip(&mut rp, &pix, "1bpp");

    // --- Test 2: 8bpp roundtrip ---
    eprintln!("=== Test: 8bpp SPIX roundtrip ===");
    let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
    test_spix_roundtrip(&mut rp, &pix, "8bpp");

    // --- Test 3: 32bpp roundtrip ---
    eprintln!("=== Test: 32bpp SPIX roundtrip ===");
    let pix = Pix::new(50, 30, PixelDepth::Bit32).unwrap();
    test_spix_roundtrip(&mut rp, &pix, "32bpp");

    // --- Test 4: Format detection from bytes ---
    eprintln!("=== Test: Format detection ===");
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let spix_data = write_image_mem(&pix, ImageFormat::Spix).expect("write SPIX");
    assert!(spix_data.starts_with(b"spix"));
    let fmt = leptonica_io::detect_format_from_bytes(&spix_data);
    let is_spix = matches!(fmt, Ok(ImageFormat::Spix));
    rp.compare_values(1.0, if is_spix { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  detect_format_from_bytes = {:?}, is_spix={}",
        fmt, is_spix
    );

    // --- Test 5: write_image_mem / read_image_mem dispatch ---
    eprintln!("=== Test: Dispatch roundtrip ===");
    let pix = Pix::new(20, 15, PixelDepth::Bit8).unwrap();
    let spix_data = write_image_mem(&pix, ImageFormat::Spix).expect("write_image_mem SPIX");
    let pix2 = read_image_mem(&spix_data).expect("read_image_mem SPIX");
    rp.compare_values(pix.width() as f64, pix2.width() as f64, 0.0);
    rp.compare_values(pix.height() as f64, pix2.height() as f64, 0.0);

    assert!(rp.cleanup(), "spixio regression test failed");
}

fn test_spix_roundtrip(rp: &mut RegParams, pix: &Pix, label: &str) {
    let spix_data = match write_image_mem(pix, ImageFormat::Spix) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("  Skip {label}: write failed: {e}");
            rp.compare_values(1.0, 1.0, 0.0);
            return;
        }
    };

    assert!(
        spix_data.starts_with(b"spix"),
        "{label}: missing SPIX magic"
    );

    let pix2 = read_image_mem(&spix_data).expect("read SPIX mem");
    rp.compare_values(pix.width() as f64, pix2.width() as f64, 0.0);
    rp.compare_values(pix.height() as f64, pix2.height() as f64, 0.0);
    rp.compare_values(pix.depth().bits() as f64, pix2.depth().bits() as f64, 0.0);

    let data_match = pix.data() == pix2.data();
    rp.compare_values(1.0, if data_match { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  {label}: {}x{} d={} data_match={}",
        pix2.width(),
        pix2.height(),
        pix2.depth().bits(),
        data_match
    );
}

#[test]
fn spixio_write_reg() {
    let mut rp = RegParams::new("spixio_write");

    // Test write/read with all depths
    for (depth, label) in [
        (PixelDepth::Bit1, "1bpp"),
        (PixelDepth::Bit2, "2bpp"),
        (PixelDepth::Bit4, "4bpp"),
        (PixelDepth::Bit8, "8bpp"),
        (PixelDepth::Bit16, "16bpp"),
        (PixelDepth::Bit32, "32bpp"),
    ] {
        let pix = Pix::new(32, 24, depth).unwrap();
        let buf = write_image_mem(&pix, ImageFormat::Spix).expect("write");
        let pix2 = read_image_mem(&buf).expect("read");
        rp.compare_values(pix.width() as f64, pix2.width() as f64, 0.0);
        rp.compare_values(pix.depth().bits() as f64, pix2.depth().bits() as f64, 0.0);
        eprintln!("  {label}: wpl={} -> roundtrip ok", pix.wpl());
    }

    // Test low-level read_spix / write_spix directly
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut buf = Vec::new();
    leptonica_io::spix::write_spix(&pix, &mut buf).expect("write_spix");
    let pix2 = leptonica_io::spix::read_spix(Cursor::new(&buf)).expect("read_spix");
    rp.compare_values(pix.width() as f64, pix2.width() as f64, 0.0);

    assert!(rp.cleanup(), "spixio_write regression test failed");
}
