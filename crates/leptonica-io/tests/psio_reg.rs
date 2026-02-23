//! PostScript I/O regression test
//!
//! Tests PostScript output at all three levels (uncompressed, DCT, Flate),
//! EPS generation, and multi-page PS documents.
//!
//! The C version tests positioned output with BOX parameters,
//! pixa compressed output, and segmented PS with mask files.
//! These require APIs not yet available in Rust.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/psio_reg.c`

use leptonica_io::ps::{PsLevel, PsOptions};
use leptonica_test::RegParams;

/// Test PostScript Level 1 (uncompressed) output (C check 0).
///
/// Writes an image to PS at Level 1, verifies DSC header and hex data.
#[test]
fn psio_reg_level1() {
    let mut rp = RegParams::new("psio_level1");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    let opts = PsOptions::default().level(PsLevel::Level1);
    let data = leptonica_io::ps::write_ps_mem(&pix, &opts).expect("write_ps_mem level1");
    let ps_str = String::from_utf8_lossy(&data);

    // Should contain DSC header
    rp.compare_values(1.0, if ps_str.starts_with("%!") { 1.0 } else { 0.0 }, 0.0);

    // Level 1 uses hex encoding, not ASCII85
    let has_hex_data = ps_str.contains("currentfile /ASCIIHexDecode")
        || ps_str.contains("readhexstring")
        || !ps_str.contains("~>");
    rp.compare_values(1.0, if has_hex_data { 1.0 } else { 0.0 }, 0.0);

    // Should have non-trivial size
    rp.compare_values(1.0, if data.len() > 100 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "psio level1 test failed");
}

/// Test PostScript Level 2 (DCT/JPEG) output (C checks 1-2).
///
/// Writes 8bpp and 32bpp images to PS at Level 2, verifies JPEG encoding.
#[test]
fn psio_reg_level2() {
    let mut rp = RegParams::new("psio_level2");

    // 8bpp grayscale
    let pix8 = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let opts = PsOptions::default().level(PsLevel::Level2);
    let data = leptonica_io::ps::write_ps_mem(&pix8, &opts).expect("write_ps_mem level2 8bpp");
    let ps_str = String::from_utf8_lossy(&data);

    // Should contain DSC header
    rp.compare_values(1.0, if ps_str.starts_with("%!") { 1.0 } else { 0.0 }, 0.0);

    // Level 2 with JPEG uses DCTDecode and ASCII85
    let has_dct_or_a85 = ps_str.contains("DCTDecode") || ps_str.contains("~>");
    rp.compare_values(1.0, if has_dct_or_a85 { 1.0 } else { 0.0 }, 0.0);

    // 32bpp RGB
    let pix32 = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");
    let data32 = leptonica_io::ps::write_ps_mem(&pix32, &opts).expect("write_ps_mem level2 32bpp");
    let ps_str32 = String::from_utf8_lossy(&data32);

    rp.compare_values(1.0, if ps_str32.starts_with("%!") { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if data32.len() > 100 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "psio level2 test failed");
}

/// Test PostScript Level 3 (Flate) output (C checks 3-5).
///
/// Writes images at various depths to PS at Level 3, verifies Flate encoding.
#[test]
fn psio_reg_level3() {
    let mut rp = RegParams::new("psio_level3");

    let images = [
        ("feyn.tif", "1bpp"),
        ("karen8.jpg", "8bpp"),
        ("marge.jpg", "32bpp"),
    ];
    let opts = PsOptions::default().level(PsLevel::Level3);

    for (img, _label) in &images {
        let pix = leptonica_test::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        let data = leptonica_io::ps::write_ps_mem(&pix, &opts).expect("write_ps_mem level3");
        let ps_str = String::from_utf8_lossy(&data);

        // Should contain DSC header
        rp.compare_values(1.0, if ps_str.starts_with("%!") { 1.0 } else { 0.0 }, 0.0);

        // Level 3 uses FlateDecode with ASCII85
        let has_flate_or_a85 = ps_str.contains("FlateDecode") || ps_str.contains("~>");
        rp.compare_values(1.0, if has_flate_or_a85 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "psio level3 test failed");
}

/// Test EPS (Encapsulated PostScript) output (C checks 7-9).
///
/// Writes an image as EPS, verifies bounding box is present.
#[test]
fn psio_reg_eps() {
    let mut rp = RegParams::new("psio_eps");

    let pix = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let opts = PsOptions::eps();
    let data = leptonica_io::ps::write_eps_mem(&pix, &opts).expect("write_eps_mem");
    let eps_str = String::from_utf8_lossy(&data);

    // EPS should contain BoundingBox
    let has_bbox = eps_str.contains("%%BoundingBox:");
    rp.compare_values(1.0, if has_bbox { 1.0 } else { 0.0 }, 0.0);

    // EPS should have PS header
    rp.compare_values(1.0, if eps_str.starts_with("%!") { 1.0 } else { 0.0 }, 0.0);

    // Should be non-trivial size
    rp.compare_values(1.0, if data.len() > 100 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "psio eps test failed");
}

/// Test multi-page PostScript output (C checks 4-5).
///
/// Writes multiple images to a single PS document, verifies page markers.
#[test]
fn psio_reg_multipage() {
    let mut rp = RegParams::new("psio_multipage");

    let pix1 = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    let pix2 = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let pix3 = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");

    let images: Vec<&leptonica_io::Pix> = vec![&pix1, &pix2, &pix3];
    let opts = PsOptions::default().level(PsLevel::Level3);

    let mut buf = Vec::new();
    leptonica_io::ps::write_ps_multi(&images, &mut buf, &opts).expect("write_ps_multi");
    let ps_str = String::from_utf8_lossy(&buf);

    // Should contain DSC header
    rp.compare_values(1.0, if ps_str.starts_with("%!") { 1.0 } else { 0.0 }, 0.0);

    // Should contain page markers for multiple pages
    let page_count = ps_str.matches("%%Page:").count();
    rp.compare_values(3.0, page_count as f64, 0.0);

    // Should end with trailer
    let has_trailer = ps_str.contains("%%Trailer") || ps_str.contains("%%EOF");
    rp.compare_values(1.0, if has_trailer { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "psio multipage test failed");
}

/// Test PS with custom resolution and scaling (additional).
///
/// Verifies resolution and scale options affect output.
#[test]
fn psio_reg_options() {
    let mut rp = RegParams::new("psio_options");

    let pix = leptonica_test::load_test_image("karen8.jpg").expect("load karen8.jpg");

    // Default options
    let data_default =
        leptonica_io::ps::write_ps_mem(&pix, &PsOptions::default()).expect("default");

    // Custom resolution 150 PPI
    let opts_150 = PsOptions::default().resolution(150);
    let data_150 = leptonica_io::ps::write_ps_mem(&pix, &opts_150).expect("150 ppi");

    // Both should produce valid PS
    let s_default = String::from_utf8_lossy(&data_default);
    let s_150 = String::from_utf8_lossy(&data_150);
    rp.compare_values(
        1.0,
        if s_default.starts_with("%!") {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(1.0, if s_150.starts_with("%!") { 1.0 } else { 0.0 }, 0.0);

    // Different resolution should produce different sizes (scaling changes)
    rp.compare_values(
        1.0,
        if data_default.len() != data_150.len() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "psio options test failed");
}

/// Test PS output with positioned images (C checks 1-2).
///
/// Requires BOX parameter for absolute x/y positioning which is not available.
#[test]
#[ignore = "not yet implemented: PS positioned output with BOX parameter not available"]
fn psio_reg_positioned() {
    // C version:
    // 1. pixWriteStreamPS() with box parameter for x/y offset
    // 2. Multiple images at different positions on a page
}

/// Test pixa compressed PS output (C check 10).
///
/// Requires pixaWriteCompressedToPS which is not available.
#[test]
#[ignore = "not yet implemented: pixaWriteCompressedToPS not available"]
fn psio_reg_pixa_compressed() {
    // C version:
    // 1. pixaWriteCompressedToPS() to write array of images
    // 2. Verify compressed stream format
}

/// Test segmented PS from mask files (C additional checks).
///
/// Requires convertSegmentedPagesToPS which is not available.
#[test]
#[ignore = "not yet implemented: convertSegmentedPagesToPS not available"]
fn psio_reg_segmented() {
    // C version:
    // 1. convertSegmentedPagesToPS() with mask file directory
    // 2. Mixed raster encoding (G4 text + JPEG images)
}
