//! PostScript segmented output regression test
//!
//! Tests PostScript generation with mixed-raster encoding where
//! text regions use G4 compression and image regions use JPEG.
//!
//! The C version requires convertSegmentedPagesToPS,
//! pixGetRegionsBinary, pixOctreeColorQuant, and related APIs
//! which are not available in Rust. This file documents the C test
//! structure with available partial tests using basic PS output.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/psioseg_reg.c`

mod common;
use common::RegParams;
use leptonica::io::ps::{PsLevel, PsOptions};

/// Test basic PS output of images used in C segmented tests (partial).
///
/// Since segmented PS is not available, verifies that images can be
/// written as PS at different compression levels.
#[test]
fn psioseg_reg_basic_ps_output() {
    let mut rp = RegParams::new("psioseg_basic");

    // Test image that would be segmented in C
    let pix = common::load_test_image("feyn.tif").expect("load feyn.tif");

    // Level 1: uncompressed hex-encoded PostScript baseline
    let opts_l1 = PsOptions::default().level(PsLevel::Level1);
    let data_l1 = leptonica::io::ps::write_ps_mem(&pix, &opts_l1).expect("write_ps_mem level1");
    let ps_l1 = String::from_utf8_lossy(&data_l1);
    rp.compare_values(1.0, if ps_l1.starts_with("%!") { 1.0 } else { 0.0 }, 0.0);
    // Level 1 uses ASCIIHexDecode
    let has_hex = ps_l1.contains("ASCIIHexDecode") || ps_l1.contains("readhexstring");
    rp.compare_values(1.0, if has_hex { 1.0 } else { 0.0 }, 0.0);

    // Level 3: Flate compressed with ASCII85 encoding
    let opts_l3 = PsOptions::default().level(PsLevel::Level3);
    let data_l3 = leptonica::io::ps::write_ps_mem(&pix, &opts_l3).expect("write_ps_mem level3");
    let ps_l3 = String::from_utf8_lossy(&data_l3);
    rp.compare_values(1.0, if ps_l3.starts_with("%!") { 1.0 } else { 0.0 }, 0.0);
    // Level 3 uses FlateDecode or ASCII85 EOD marker
    let has_flate = ps_l3.contains("FlateDecode") || ps_l3.contains("~>");
    rp.compare_values(1.0, if has_flate { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "psioseg basic ps output test failed");
}

/// Test segmented PS with mixed raster encoding (C checks 0-5).
///
/// Requires convertSegmentedPagesToPS, pixGetRegionsBinary.
#[test]
#[ignore = "not yet implemented: convertSegmentedPagesToPS not available"]
fn psioseg_reg_segmented_output() {
    // C version:
    // 1. pixGetRegionsBinary() for text/image region detection
    // 2. Build composite images with different regions
    // 3. convertSegmentedPagesToPS() with G4 text + JPEG images
}

/// Test PS output with color quantized images (C additional checks).
///
/// Requires pixOctreeColorQuant, pixOctreeQuantNumColors.
#[test]
#[ignore = "not yet implemented: color quantization for PS segmentation not available"]
fn psioseg_reg_color_quantized() {
    // C version:
    // 1. pixOctreeColorQuant() for 240-color quantization
    // 2. pixOctreeQuantNumColors() for 16-color quantization
    // 3. Write quantized images to PS
}

/// Test PS with intermediate format conversion (C additional checks).
///
/// Requires pixSubtract, region-based pixRasterop.
#[test]
#[ignore = "not yet implemented: pixSubtract and region rasterop not available"]
fn psioseg_reg_format_conversion() {
    // C version:
    // 1. pixConvertTo32() for compositing
    // 2. pixSubtract() for mask operations
    // 3. pixCombineMasked() for region merging
    // 4. Write intermediate TIFF/JPEG/PNG files
}
