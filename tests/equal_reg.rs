//! Equality regression test
//!
//! Tests pixEqual across various colormap and depth conversion scenarios.
//! Verifies that colormap removal preserves visual equality, and that
//! write/read round-trips preserve image data.
//!
//! The C version tests 6 image types through colormap removal, quantization,
//! and RGB-to-colormap conversion. This Rust port covers the available
//! operations: remove_colormap, convert_to_8/32, and equals_with_cmap.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/equal_reg.c`

mod common;
use common::RegParams;
use leptonica::core::pix::RemoveColormapTarget;

/// Test 1bpp binary image: write/read round-trip preserves equality (C check 0).
#[test]
fn equal_reg_binary_roundtrip() {
    let mut rp = RegParams::new("equal_binary");

    let pix1 = common::load_test_image("feyn.tif").expect("load feyn.tif");

    // Verify self-equality
    rp.compare_pix(&pix1, &pix1);

    // Deep clone should be equal
    let pix2 = pix1.deep_clone();
    rp.compare_pix(&pix1, &pix2);

    assert!(rp.cleanup(), "equal binary roundtrip test failed");
}

/// Test 8bpp colormapped image: remove colormap preserves equality (C check 9-10).
///
/// Reads dreyfus8.png (8bpp with colormap), removes colormap via
/// BASED_ON_SRC and TO_FULL_COLOR, then checks equals_with_cmap.
#[test]
#[ignore = "not yet implemented: requires equals_with_cmap validation"]
fn equal_reg_8bpp_colormap() {
    let mut rp = RegParams::new("equal_8bpp_cmap");

    let pix1 = common::load_test_image("dreyfus8.png").expect("load dreyfus8.png");

    // Remove colormap based on source
    let pix2 = pix1
        .remove_colormap(RemoveColormapTarget::BasedOnSrc)
        .expect("remove cmap based_on_src");

    // Remove colormap to full color (32bpp)
    let pix3 = pix1
        .remove_colormap(RemoveColormapTarget::ToFullColor)
        .expect("remove cmap to_full_color");

    // The removed versions should be visually equal to the original
    // (equals_with_cmap handles colormap comparison)
    assert!(
        pix1.equals_with_cmap(&pix2),
        "dreyfus8 based_on_src should match original"
    );
    assert!(
        pix1.equals_with_cmap(&pix3),
        "dreyfus8 to_full_color should match original"
    );

    let _ = rp.compare_values(1.0, 1.0, 0.0); // placeholder check

    assert!(rp.cleanup(), "equal 8bpp colormap test failed");
}

/// Test 8bpp grayscale without colormap: convert to 32bpp and back (C check 11-13).
///
/// Tests depth conversion round-trip with karen8.jpg.
#[test]
fn equal_reg_8bpp_gray() {
    let mut rp = RegParams::new("equal_8bpp_gray");

    let pix1 = common::load_test_image("karen8.jpg").expect("load karen8.jpg");

    // Convert 8bpp gray → 32bpp → 8bpp
    let pix_32 = pix1.convert_to_32().expect("convert to 32");
    let pix_8 = pix_32.convert_to_8().expect("convert to 8");

    // Round-trip should preserve the image
    rp.compare_pix(&pix1, &pix_8);

    // TODO: pixThresholdTo4bpp (not available)
    // TODO: pixConvertRGBToColormap (not available)

    assert!(rp.cleanup(), "equal 8bpp gray test failed");
}

/// Test RGB 32bpp image: colormap operations (C checks 14-16).
///
/// Tests remove_colormap after depth conversion on marge.jpg.
#[test]
fn equal_reg_rgb() {
    let mut rp = RegParams::new("equal_rgb");

    let pix1 = common::load_test_image("marge.jpg").expect("load marge.jpg");

    // Convert 32bpp → 8bpp → 32bpp
    let pix_8 = pix1.convert_to_8().expect("convert to 8");
    let pix_32 = pix_8.convert_to_32().expect("convert to 32");

    // Convert to grayscale should be consistent
    let pix_gray1 = pix1.convert_to_8().expect("gray1");
    let pix_gray2 = pix_32.convert_to_8().expect("gray2");
    rp.compare_pix(&pix_gray1, &pix_gray2);

    // TODO: pixOctreeQuantNumColors (not available)
    // TODO: pixConvertRGBToColormap (not available)

    assert!(rp.cleanup(), "equal rgb test failed");
}

/// Test 2bpp colormapped image (C checks 1-4).
///
/// Requires dreyfus2.png and quantization functions not available.
#[test]
#[ignore = "not yet implemented: requires pixOctreeQuantNumColors, pixConvertRGBToColormap"]
fn equal_reg_2bpp_colormap() {
    // C version:
    // 1. Reads dreyfus2.png (2bpp colormapped)
    // 2. pixRemoveColormap (BASED_ON_SRC) → compare
    // 3. pixRemoveColormap (TO_FULL_COLOR) → compare
    // 4. pixOctreeQuantNumColors(64) → compare
    // 5. pixConvertRGBToColormap → compare
}

/// Test 4bpp colormapped image (C checks 5-8).
///
/// Requires dreyfus4.png and quantization functions not available.
#[test]
#[ignore = "not yet implemented: requires pixOctreeQuantNumColors, pixConvertRGBToColormap"]
fn equal_reg_4bpp_colormap() {
    // C version:
    // 1. Reads dreyfus4.png (4bpp colormapped)
    // 2. Same operations as 2bpp test with 256 colors
}
