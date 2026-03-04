//! Depth conversion regression test
//!
//! Tests conversions between 1, 2, 4, 8, 16, and 32 bpp images,
//! including colormap operations.
//!
//! The C version uses 9 different source images across all bit depths
//! and performs 32+ regression checks covering pixConvertTo8/32/16,
//! pixThreshold*, pixRemoveColormap, etc.
//! This Rust port tests available depth conversion APIs.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/conversion_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::core::pix::RemoveColormapTarget;
use leptonica::io::ImageFormat;

/// Test 1bpp → various depth conversions (C checks 0-3).
///
/// Converts test1.png (1bpp) to 2, 4, 8, 32 bpp and verifies
/// the resulting depths and that image dimensions are preserved.
#[test]
fn conversion_reg_from_1bpp() {
    let mut rp = RegParams::new("conversion_from_1bpp");

    let pix1 = crate::common::load_test_image("test1.png").expect("load test1.png");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);

    // 1 → 2 bpp (via unpack)
    let pix2 = pix1.convert_to_2().expect("convert 1→2");
    assert_eq!(pix2.depth(), PixelDepth::Bit2);
    rp.compare_values(2.0, pix2.depth().bits() as f64, 0.0);

    // 1 → 4 bpp
    let pix4 = pix1.convert_to_4().expect("convert 1→4");
    assert_eq!(pix4.depth(), PixelDepth::Bit4);
    rp.compare_values(4.0, pix4.depth().bits() as f64, 0.0);

    // 1 → 8 bpp
    let pix8 = pix1.convert_to_8().expect("convert 1→8");
    assert_eq!(pix8.depth(), PixelDepth::Bit8);
    rp.compare_values(8.0, pix8.depth().bits() as f64, 0.0);

    // 1 → 32 bpp
    let pix32 = pix1.convert_to_32().expect("convert 1→32");
    assert_eq!(pix32.depth(), PixelDepth::Bit32);
    rp.compare_values(32.0, pix32.depth().bits() as f64, 0.0);

    // Dimensions should be preserved
    rp.compare_values(pix1.width() as f64, pix2.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, pix2.height() as f64, 0.0);

    rp.write_pix_and_check(&pix8, ImageFormat::Png)
        .expect("write pix8 conversion_from_1bpp");

    assert!(rp.cleanup(), "conversion from 1bpp test failed");
}

/// Test 2bpp colormapped → various depth conversions (C checks 4-7).
#[test]
fn conversion_reg_from_2bpp() {
    let mut rp = RegParams::new("conversion_from_2bpp");

    let pix2 = crate::common::load_test_image("dreyfus2.png").expect("load dreyfus2.png");
    assert_eq!(pix2.depth(), PixelDepth::Bit2);

    // Remove colormap to grayscale
    let pix8 = pix2
        .remove_colormap(RemoveColormapTarget::ToGrayscale)
        .expect("remove to grayscale");
    assert_eq!(pix8.depth(), PixelDepth::Bit8);
    rp.compare_values(8.0, pix8.depth().bits() as f64, 0.0);

    // Remove colormap to full color
    let pix32 = pix2
        .remove_colormap(RemoveColormapTarget::ToFullColor)
        .expect("remove to full color");
    assert_eq!(pix32.depth(), PixelDepth::Bit32);
    rp.compare_values(32.0, pix32.depth().bits() as f64, 0.0);

    // Dimensions preserved
    rp.compare_values(pix2.width() as f64, pix8.width() as f64, 0.0);
    rp.compare_values(pix2.height() as f64, pix8.height() as f64, 0.0);

    rp.write_pix_and_check(&pix8, ImageFormat::Png)
        .expect("write pix8 conversion_from_2bpp");
    rp.write_pix_and_check(&pix32, ImageFormat::Png)
        .expect("write pix32 conversion_from_2bpp");

    assert!(rp.cleanup(), "conversion from 2bpp test failed");
}

/// Test 4bpp colormapped → various depth conversions (C checks 8-11).
#[test]
fn conversion_reg_from_4bpp() {
    let mut rp = RegParams::new("conversion_from_4bpp");

    let pix4 = crate::common::load_test_image("weasel4.16c.png").expect("load weasel4.16c.png");
    assert_eq!(pix4.depth(), PixelDepth::Bit4);

    // Remove colormap based on src
    let pix8 = pix4
        .remove_colormap(RemoveColormapTarget::BasedOnSrc)
        .expect("remove based_on_src");
    rp.compare_values(pix4.width() as f64, pix8.width() as f64, 0.0);
    rp.compare_values(pix4.height() as f64, pix8.height() as f64, 0.0);

    rp.write_pix_and_check(&pix8, ImageFormat::Png)
        .expect("write pix8 conversion_from_4bpp");

    // Convert 4 bpp grayscale image
    let pix4g = crate::common::load_test_image("weasel4.16g.png").expect("load weasel4.16g.png");
    let pix8g = pix4g.convert_4_to_8(false).expect("convert_4_to_8 no cmap");
    assert_eq!(pix8g.depth(), PixelDepth::Bit8);
    rp.compare_values(8.0, pix8g.depth().bits() as f64, 0.0);

    rp.write_pix_and_check(&pix8g, ImageFormat::Png)
        .expect("write pix8g conversion_from_4bpp");

    assert!(rp.cleanup(), "conversion from 4bpp test failed");
}

/// Test 8bpp colormapped → various depth conversions (C checks 12-15).
#[test]
fn conversion_reg_from_8bpp() {
    let mut rp = RegParams::new("conversion_from_8bpp");

    let pix8c = crate::common::load_test_image("weasel8.240c.png").expect("load weasel8.240c");
    assert_eq!(pix8c.depth(), PixelDepth::Bit8);

    // Remove colormap based on src
    let converted = pix8c
        .remove_colormap(RemoveColormapTarget::BasedOnSrc)
        .expect("remove based_on_src");
    rp.compare_values(pix8c.width() as f64, converted.width() as f64, 0.0);
    rp.compare_values(pix8c.height() as f64, converted.height() as f64, 0.0);

    // Remove to grayscale
    let gray = pix8c
        .remove_colormap(RemoveColormapTarget::ToGrayscale)
        .expect("remove to grayscale");
    assert_eq!(gray.depth(), PixelDepth::Bit8);
    rp.compare_values(8.0, gray.depth().bits() as f64, 0.0);

    rp.write_pix_and_check(&converted, ImageFormat::Png)
        .expect("write converted conversion_from_8bpp");
    rp.write_pix_and_check(&gray, ImageFormat::Png)
        .expect("write gray conversion_from_8bpp");

    assert!(rp.cleanup(), "conversion from 8bpp test failed");
}

/// Test 8bpp grayscale → other depths (C checks 16-19).
#[test]
fn conversion_reg_gray_to_other() {
    let mut rp = RegParams::new("conversion_gray_to_other");

    let pix8 = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    assert_eq!(pix8.depth(), PixelDepth::Bit8);

    // 8 → 16 bpp
    let pix16 = pix8.convert_to_16().expect("convert 8→16");
    assert_eq!(pix16.depth(), PixelDepth::Bit16);
    rp.compare_values(16.0, pix16.depth().bits() as f64, 0.0);

    // 8 → 32 bpp
    let pix32 = pix8.convert_to_32().expect("convert 8→32");
    assert_eq!(pix32.depth(), PixelDepth::Bit32);
    rp.compare_values(32.0, pix32.depth().bits() as f64, 0.0);

    // 8 → gray colormap
    let pix_cmap = pix8.convert_gray_to_colormap().expect("gray→cmap");
    rp.compare_values(pix8.width() as f64, pix_cmap.width() as f64, 0.0);

    assert!(rp.cleanup(), "conversion gray to other test failed");
}

/// Test 16bpp → 8bpp conversions (C checks 20-23).
#[test]
fn conversion_reg_from_16bpp() {
    let mut rp = RegParams::new("conversion_from_16bpp");

    let pix16 = crate::common::load_test_image("test16.tif").expect("load test16.tif");
    assert_eq!(pix16.depth(), PixelDepth::Bit16);

    // 16 → 8 bpp (LS byte)
    let pix8_ls = pix16
        .convert_16_to_8(leptonica::core::pix::Convert16To8Type::LsByte)
        .expect("convert 16→8 ls");
    assert_eq!(pix8_ls.depth(), PixelDepth::Bit8);
    rp.compare_values(8.0, pix8_ls.depth().bits() as f64, 0.0);

    // 16 → 8 bpp (MS byte)
    let pix8_ms = pix16
        .convert_16_to_8(leptonica::core::pix::Convert16To8Type::MsByte)
        .expect("convert 16→8 ms");
    assert_eq!(pix8_ms.depth(), PixelDepth::Bit8);

    // Dimensions preserved
    rp.compare_values(pix16.width() as f64, pix8_ls.width() as f64, 0.0);
    rp.compare_values(pix16.height() as f64, pix8_ls.height() as f64, 0.0);

    rp.write_pix_and_check(&pix8_ls, ImageFormat::Png)
        .expect("write pix8_ls conversion_from_16bpp");
    rp.write_pix_and_check(&pix8_ms, ImageFormat::Png)
        .expect("write pix8_ms conversion_from_16bpp");

    assert!(rp.cleanup(), "conversion from 16bpp test failed");
}

/// Test 32bpp RGB → various depth conversions (C checks 24-31).
#[test]
fn conversion_reg_from_32bpp() {
    let mut rp = RegParams::new("conversion_from_32bpp");

    let pix32 = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    assert_eq!(pix32.depth(), PixelDepth::Bit32);

    // 32 → 8 bpp grayscale
    let pix8 = pix32.convert_to_8().expect("convert 32→8");
    assert_eq!(pix8.depth(), PixelDepth::Bit8);
    rp.compare_values(8.0, pix8.depth().bits() as f64, 0.0);

    // 32 → luminance (grayscale)
    let pix_lum = pix32.convert_rgb_to_luminance().expect("rgb→luminance");
    assert_eq!(pix_lum.depth(), PixelDepth::Bit8);

    // 32 → gray with custom weights
    let pix_gray = pix32.convert_rgb_to_gray(0.3, 0.5, 0.2).expect("rgb→gray");
    assert_eq!(pix_gray.depth(), PixelDepth::Bit8);
    rp.compare_values(8.0, pix_gray.depth().bits() as f64, 0.0);

    // Dimensions preserved
    rp.compare_values(pix32.width() as f64, pix8.width() as f64, 0.0);
    rp.compare_values(pix32.height() as f64, pix8.height() as f64, 0.0);

    // Octcube quantization (not available)
    // TODO: pixOctcubeQuantFromCmap not available

    assert!(rp.cleanup(), "conversion from 32bpp test failed");
}
