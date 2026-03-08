//! PostScript segmented output regression test
//!
//! Tests PostScript generation with mixed-raster encoding where
//! text regions use G4 compression and image regions use JPEG.
//!
//! The C version requires convertSegmentedPagesToPS and
//! pixGetRegionsBinary for segmented output, which are not available
//! in Rust. The composite pipeline (scale→tile→quantize) is
//! implemented using available APIs.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/psioseg_reg.c`

use crate::common::RegParams;
use leptonica::color::{OctreeOptions, octree_quant, octree_quant_num_colors};
use leptonica::io::ImageFormat;
use leptonica::io::ps::{PsLevel, PsOptions};
use leptonica::transform::{ScaleMethod, scale};
use leptonica::{Pix, PixelDepth, RopOp};

/// Test basic PS output of images used in C segmented tests (partial).
///
/// Since segmented PS is not available, verifies that images can be
/// written as PS at different compression levels.
#[test]
fn psioseg_reg_basic_ps_output() {
    let mut rp = RegParams::new("psioseg_basic");

    // Test image that would be segmented in C
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");

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

/// Composite image pipeline: scale→tile→quantize (C checks 0, 2-4).
///
/// C: scale tetons.jpg to page width, tile into full page canvas,
/// then gray conversion and color quantization.
/// Check 1 (combine with halftone mask) and check 5 (convertSegmentedPagesToPS)
/// require pixGetRegionsBinary which is not available.
#[test]
fn psioseg_reg_composite_pipeline() {
    let mut rp = RegParams::new("psioseg_pipeline");

    // Load source images (C: pageseg2.tif = 1bpp, tetons.jpg = 32bpp)
    let pix_page = crate::common::load_test_image("pageseg2.tif").expect("load pageseg2.tif");
    let pix_color = crate::common::load_test_image("tetons.jpg").expect("load tetons.jpg");

    // Ensure 32bpp color
    let pix_color = if pix_color.depth() != PixelDepth::Bit32 {
        pix_color.convert_to_32().expect("convert to 32bpp")
    } else {
        pix_color
    };

    // Scale tetons to match page width (C: scalefactor = w / wc; pixScale)
    let w = pix_page.width();
    let h = pix_page.height();
    let wc = pix_color.width();
    let scalefactor = w as f32 / wc as f32;
    let pix_scaled =
        scale(&pix_color, scalefactor, scalefactor, ScaleMethod::Auto).expect("scale tetons");
    let hc = pix_scaled.height();
    rp.compare_values(w as f64, pix_scaled.width() as f64, 1.0);

    // Create 32bpp canvas with page dimensions, tile the scaled image
    // C: pixcs2 = pixCreate(w, h, 32) + two pixRasterop(PIX_SRC) calls
    let pix_composite = {
        let base = Pix::new(w, h, PixelDepth::Bit32).expect("create canvas");
        let mut canvas = base.try_into_mut().expect("try_into_mut canvas");
        canvas
            .rop_region_inplace(0, 0, w, hc, RopOp::Src, &pix_scaled, 0, 0)
            .expect("rasterop top half");
        canvas
            .rop_region_inplace(0, hc as i32, w, hc, RopOp::Src, &pix_scaled, 0, 0)
            .expect("rasterop bottom half");
        let p: Pix = canvas.into();
        p
    };
    assert_eq!(pix_composite.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&pix_composite, ImageFormat::Jpeg)
        .expect("write composite"); // C check 0

    // Gray conversion: pixConvertRGBToLuminance (C check 2)
    let pix_gray = pix_composite
        .convert_rgb_to_luminance()
        .expect("convert_rgb_to_luminance");
    assert_eq!(pix_gray.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&pix_gray, ImageFormat::Jpeg)
        .expect("write gray"); // C check 2

    // 8bpp colormapped: pixOctreeColorQuant with 240 colors (C check 3)
    let pix_8c =
        octree_quant(&pix_composite, &OctreeOptions { max_colors: 240 }).expect("octree_quant 240");
    assert_eq!(pix_8c.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&pix_8c, ImageFormat::Png)
        .expect("write 8bpp colormapped"); // C check 3

    // 4bpp colormapped: pixOctreeQuantNumColors with 16 colors, subsample=4 (C check 4)
    let pix_4c =
        octree_quant_num_colors(&pix_composite, 16, 4).expect("octree_quant_num_colors 16");
    rp.write_pix_and_check(&pix_4c, ImageFormat::Png)
        .expect("write 4bpp colormapped"); // C check 4

    assert!(rp.cleanup(), "psioseg composite pipeline test failed");
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
