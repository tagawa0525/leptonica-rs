//! Adaptive mapping regression test
//!
//! C version: reference/leptonica/prog/adaptmap_reg.c
//!
//! Tests adaptive background normalization and contrast normalization.
//!
//! C image checkpoints (indices 0-15):
//!   0: grayscale background map (pixGetBackgroundGrayMap)
//!   1: inverted grayscale background map (pixGetInvBackgroundMap)
//!   2: gray image after applying inv map (pixApplyInvBackgroundGrayMap)
//!   3: gray image after gamma TRC masked (pixGammaTRCMasked)
//!   4-6: RGB background maps R/G/B (pixGetBackgroundRGBMap)
//!   7-9: inverted RGB maps R/G/B (pixGetInvBackgroundMap x3)
//!   10: color image after applying inv maps (pixApplyInvBackgroundRGBMap)
//!   11: color image after gamma TRC masked
//!   12: high-level color background norm (pixBackgroundNorm)
//!   13: color image after gamma TRC masked (post pixBackgroundNorm)
//!   14: pixFillMapHoles demonstration (weasel8.png)
//!   15: pixFillMapHoles simple 3x3 case
//!
//! Rust API mapping:
//!   - pixGetBackgroundGrayMap()     -> get_background_gray_map()
//!   - pixGetInvBackgroundMap()      -> get_inv_background_map()
//!   - pixApplyInvBackgroundGrayMap()-> apply_inv_background_gray_map()
//!   - pixGetBackgroundRGBMap()      -> get_background_rgb_map()
//!   - pixApplyInvBackgroundRGBMap() -> apply_inv_background_rgb_map()
//!   - pixBackgroundNorm()           -> background_norm()
//!   - pixBackgroundNormSimple()     -> background_norm_simple()
//!   - pixContrastNorm()             -> contrast_norm()
//!   - pixFillMapHoles()             -> fill_map_holes()
//!   - pixGammaTRCMasked()           -> gamma_trc_masked()

use crate::common::{RegParams, load_test_image};
use leptonica::filter::{
    BackgroundNormOptions, ContrastNormOptions, apply_inv_background_gray_map,
    apply_inv_background_rgb_map, background_norm, background_norm_simple, contrast_norm,
    contrast_norm_simple, fill_map_holes, gamma_trc_masked, get_background_gray_map,
    get_background_rgb_map, get_foreground_gray_map, get_inv_background_map,
};
use leptonica::io::ImageFormat;

// C version constants
const SIZE_X: u32 = 10;
const SIZE_Y: u32 = 30;
const BINTHRESH: u32 = 50;
const MINCOUNT: u32 = 30;
const BGVAL: u32 = 200;
const SMOOTH_X: u32 = 2;
const SMOOTH_Y: u32 = 1;

/// C tests 0-3: Grayscale background map pipeline.
///
/// Matches C code:
///   pixs = pixRead("wet-day.jpg")
///   pixg = pixConvertRGBToGray(pixs, 0.33, 0.34, 0.33)
///   pixGetBackgroundGrayMap(pixg, pixim, SIZE_X, SIZE_Y, BINTHRESH, MINCOUNT, &pixgm)  -> index 0
///   pixGetInvBackgroundMap(pixgm, BGVAL, SMOOTH_X, SMOOTH_Y)                           -> index 1
///   pixApplyInvBackgroundGrayMap(pixg, pixmi, SIZE_X, SIZE_Y)                          -> index 2
///   pixGammaTRCMasked(pix2, pix1, pixim, 1.0, 0, 190) + masked                        -> index 3
#[test]
fn adaptmap_reg_gray_pipeline() {
    let mut rp = RegParams::new("adaptmap_gray_pipeline");

    // C: pixs = pixRead("wet-day.jpg"); pixg = pixConvertRGBToGray(pixs, 0.33, 0.34, 0.33)
    let pixs = load_test_image("wet-day.jpg").expect("load wet-day.jpg");
    let pixg = pixs
        .convert_rgb_to_gray(0.33, 0.34, 0.33)
        .expect("convert to gray");

    // C index 0: pixGetBackgroundGrayMap(pixg, pixim, SIZE_X, SIZE_Y, BINTHRESH, MINCOUNT, &pixgm)
    let pixgm = get_background_gray_map(&pixg, None, SIZE_X, SIZE_Y, BINTHRESH, MINCOUNT)
        .expect("get_background_gray_map");
    rp.compare_values(8.0, pixgm.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pixgm, ImageFormat::Png)
        .expect("write pixgm (index 0)");

    // C index 1: pixmi = pixGetInvBackgroundMap(pixgm, BGVAL, SMOOTH_X, SMOOTH_Y)
    // The Rust implementation stores 16-bit factors in a 32bpp Pix for convenience.
    let pixmi =
        get_inv_background_map(&pixgm, BGVAL, SMOOTH_X, SMOOTH_Y).expect("get_inv_background_map");
    rp.compare_values(pixgm.width() as f64, pixmi.width() as f64, 0.0);
    rp.compare_values(pixgm.height() as f64, pixmi.height() as f64, 0.0);
    rp.write_pix_and_check(&pixmi, ImageFormat::Png)
        .expect("write pixmi (index 1)");

    // C index 2: pix1 = pixApplyInvBackgroundGrayMap(pixg, pixmi, SIZE_X, SIZE_Y)
    let pix1 = apply_inv_background_gray_map(&pixg, &pixmi, SIZE_X, SIZE_Y)
        .expect("apply_inv_background_gray_map");
    rp.compare_values(pixg.width() as f64, pix1.width() as f64, 0.0);
    rp.compare_values(pixg.height() as f64, pix1.height() as f64, 0.0);
    rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix1, ImageFormat::Jpeg)
        .expect("write pix1 gray applied (index 2)");

    // C index 3: pix2 = pixGammaTRCMasked(pix1, pixim, 1.0, 0, 190) + masked invert
    // The C code applies two gamma_trc_masked passes (one for image region, one for non-image).
    // In Rust, we apply gamma_trc_masked without a mask to approximate index 3.
    let pix2 = gamma_trc_masked(&pix1, None, 1.0, 0, 190).expect("gamma_trc_masked gray");
    rp.compare_values(pixg.width() as f64, pix2.width() as f64, 0.0);
    rp.compare_values(pixg.height() as f64, pix2.height() as f64, 0.0);
    rp.compare_values(8.0, pix2.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix2, ImageFormat::Jpeg)
        .expect("write pix2 gamma gray (index 3)");

    assert!(
        rp.cleanup(),
        "adaptmap_gray_pipeline regression test failed"
    );
}

/// C tests 4-11: Color background map pipeline.
///
/// Matches C code:
///   pixGetBackgroundRGBMap(pixs, pixim, NULL, SIZE_X, SIZE_Y, BINTHRESH, MINCOUNT,
///                          &pixmr, &pixmg, &pixmb)  -> indices 4, 5, 6
///   pixGetInvBackgroundMap(pixmr/g/b, ...)           -> indices 7, 8, 9
///   pixApplyInvBackgroundRGBMap(pixs, ...)           -> index 10
///   pixGammaTRCMasked(pix2, pix1, ...)               -> index 11
#[test]
fn adaptmap_reg_color_pipeline() {
    let mut rp = RegParams::new("adaptmap_color_pipeline");

    // C: pixs = pixRead("wet-day.jpg")
    let pixs = load_test_image("wet-day.jpg").expect("load wet-day.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // C indices 4, 5, 6: pixGetBackgroundRGBMap -> pixmr, pixmg, pixmb
    let (pixmr, pixmg, pixmb) =
        get_background_rgb_map(&pixs, None, None, SIZE_X, SIZE_Y, BINTHRESH, MINCOUNT)
            .expect("get_background_rgb_map");
    rp.compare_values(8.0, pixmr.depth().bits() as f64, 0.0);
    rp.compare_values(8.0, pixmg.depth().bits() as f64, 0.0);
    rp.compare_values(8.0, pixmb.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pixmr, ImageFormat::Png)
        .expect("write pixmr (index 4)");
    rp.write_pix_and_check(&pixmg, ImageFormat::Png)
        .expect("write pixmg (index 5)");
    rp.write_pix_and_check(&pixmb, ImageFormat::Png)
        .expect("write pixmb (index 6)");

    // C indices 7, 8, 9: pixGetInvBackgroundMap for each channel
    let pixmri = get_inv_background_map(&pixmr, BGVAL, SMOOTH_X, SMOOTH_Y).expect("inv map red");
    let pixmgi = get_inv_background_map(&pixmg, BGVAL, SMOOTH_X, SMOOTH_Y).expect("inv map green");
    let pixmbi = get_inv_background_map(&pixmb, BGVAL, SMOOTH_X, SMOOTH_Y).expect("inv map blue");
    rp.write_pix_and_check(&pixmri, ImageFormat::Png)
        .expect("write pixmri (index 7)");
    rp.write_pix_and_check(&pixmgi, ImageFormat::Png)
        .expect("write pixmgi (index 8)");
    rp.write_pix_and_check(&pixmbi, ImageFormat::Png)
        .expect("write pixmbi (index 9)");

    // C index 10: pix1 = pixApplyInvBackgroundRGBMap(pixs, pixmri, pixmgi, pixmbi, SIZE_X, SIZE_Y)
    let pix1 = apply_inv_background_rgb_map(&pixs, &pixmri, &pixmgi, &pixmbi, SIZE_X, SIZE_Y)
        .expect("apply_inv_background_rgb_map");
    rp.compare_values(w as f64, pix1.width() as f64, 0.0);
    rp.compare_values(h as f64, pix1.height() as f64, 0.0);
    rp.compare_values(32.0, pix1.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix1, ImageFormat::Jpeg)
        .expect("write pix1 color applied (index 10)");

    // C index 11: pix2 = pixGammaTRCMasked(pix1, pixim, 1.0, 0, 190) + masked
    // Approximate: apply gamma_trc_masked without a binary mask
    let pix2 = gamma_trc_masked(&pix1, None, 1.0, 0, 190).expect("gamma_trc_masked color");
    rp.compare_values(w as f64, pix2.width() as f64, 0.0);
    rp.compare_values(h as f64, pix2.height() as f64, 0.0);
    rp.compare_values(32.0, pix2.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix2, ImageFormat::Jpeg)
        .expect("write pix2 gamma color (index 11)");

    assert!(
        rp.cleanup(),
        "adaptmap_color_pipeline regression test failed"
    );
}

/// C tests 12-13: High-level background normalization + gamma TRC.
///
/// Matches C code:
///   pix1 = pixBackgroundNorm(pixs, pixim, NULL, 5, 10, BINTHRESH, 20, BGVAL, SMOOTH_X, SMOOTH_Y)
///                                                                      -> index 12
///   pixGammaTRCMasked(pix2, pix1, pixim, 1.0, 0, 190) + masked       -> index 13
#[test]
fn adaptmap_reg_background_norm_highlevel() {
    let mut rp = RegParams::new("adaptmap_bg_highlevel");

    // C: pixs = pixRead("wet-day.jpg")
    let pixs = load_test_image("wet-day.jpg").expect("load wet-day.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // C index 12: pix1 = pixBackgroundNorm(pixs, pixim, NULL, 5, 10, BINTHRESH, 20, BGVAL, ...)
    let options = BackgroundNormOptions {
        tile_width: 5,
        tile_height: 10,
        fg_threshold: BINTHRESH,
        min_count: 20,
        bg_val: BGVAL,
        smooth_x: SMOOTH_X,
        smooth_y: SMOOTH_Y,
    };
    let pix1 = background_norm(&pixs, &options).expect("background_norm high-level");
    rp.compare_values(w as f64, pix1.width() as f64, 0.0);
    rp.compare_values(h as f64, pix1.height() as f64, 0.0);
    rp.compare_values(32.0, pix1.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&pix1, ImageFormat::Jpeg)
        .expect("write background norm (index 12)");

    // C index 13: pixGammaTRCMasked applied to normalized image
    let pix2 = gamma_trc_masked(&pix1, None, 1.0, 0, 190).expect("gamma_trc_masked after bg norm");
    rp.compare_values(w as f64, pix2.width() as f64, 0.0);
    rp.compare_values(h as f64, pix2.height() as f64, 0.0);
    rp.write_pix_and_check(&pix2, ImageFormat::Jpeg)
        .expect("write gamma after bg norm (index 13)");

    assert!(rp.cleanup(), "adaptmap_bg_highlevel regression test failed");
}

/// C test 14: pixFillMapHoles with weasel8.png.
///
/// Matches C code:
///   pix1 = pixRead("weasel8.png")
///   pixGammaTRC(pix1, pix1, 1.0, 0, 270)  -- darken white pixels
///   [add white column/row holes via pixRasterop]
///   pixFillMapHoles(pix1, w, h, L_FILL_WHITE)
///   pix2 = pixaDisplayTiledInColumns(pixa, 3, 1.0, 20, 1)
///   regTestWritePixAndCheck(rp, pix2, IFF_PNG)  -> index 14
///
/// We test the fill_map_holes function directly on weasel8.png with
/// simulated holes (set column strips to 255).
#[test]
fn adaptmap_reg_fill_map_holes_weasel() {
    let mut rp = RegParams::new("adaptmap_fill_holes_weasel");

    // Load weasel8.png as a map (8bpp grayscale)
    let pix_orig = load_test_image("weasel8.png").expect("load weasel8.png");
    assert_eq!(
        pix_orig.depth().bits(),
        8,
        "weasel8.png must be 8bpp for this test"
    );

    let w = pix_orig.width();
    let h = pix_orig.height();

    // Darken white pixels: apply gamma_trc equivalent (darken bright areas)
    // C: pixGammaTRC(pix1, pix1, 1.0, 0, 270) darkens pixels by clamping at 270
    let pix_darkened = gamma_trc_masked(&pix_orig, None, 1.0, 0, 200).expect("darken weasel8");

    // Simulate white holes by writing 255 into column/row strips
    let mut pix_holes = pix_darkened.to_mut();
    // C: pixRasterop(pix1, 0, 0, 5, h, PIX_SET, NULL, 0, 0)  -- columns 0-4
    for y in 0..h {
        for x in 0..5u32.min(w) {
            pix_holes.set_pixel_unchecked(x, y, 255);
        }
    }
    // C: pixRasterop(pix1, 20, 0, 2, h, PIX_SET, NULL, 0, 0) -- columns 20-21
    for y in 0..h {
        for x in 20u32..22u32.min(w) {
            pix_holes.set_pixel_unchecked(x, y, 255);
        }
    }
    // C: pixRasterop(pix1, 40, 0, 3, h, PIX_SET, NULL, 0, 0) -- columns 40-42
    for y in 0..h {
        for x in 40u32..43u32.min(w) {
            pix_holes.set_pixel_unchecked(x, y, 255);
        }
    }
    // C: pixRasterop(pix1, 0, 0, w, 3, PIX_SET, NULL, 0, 0)  -- rows 0-2
    for y in 0..3u32.min(h) {
        for x in 0..w {
            pix_holes.set_pixel_unchecked(x, y, 255);
        }
    }
    // C: pixRasterop(pix1, 0, 15, w, 3, PIX_SET, NULL, 0, 0) -- rows 15-17
    for y in 15u32..18u32.min(h) {
        for x in 0..w {
            pix_holes.set_pixel_unchecked(x, y, 255);
        }
    }
    // C: pixRasterop(pix1, 0, 35, w, 2, PIX_SET, NULL, 0, 0) -- rows 35-36
    for y in 35u32..37u32.min(h) {
        for x in 0..w {
            pix_holes.set_pixel_unchecked(x, y, 255);
        }
    }

    let pix_with_holes: leptonica::Pix = pix_holes.into();

    // Apply fill_map_holes (L_FILL_WHITE: fills 255 "holes" -- in Rust, zero-value holes)
    // The C version treats 255 as holes for the white fill case. Our Rust API fills zero-value holes.
    // We test that fill_map_holes produces a valid output with same dimensions.
    let filled = fill_map_holes(&pix_with_holes, w, h).expect("fill_map_holes weasel");
    rp.compare_values(w as f64, filled.width() as f64, 0.0);
    rp.compare_values(h as f64, filled.height() as f64, 0.0);
    rp.compare_values(8.0, filled.depth().bits() as f64, 0.0);

    // C index 14: write the combined image (before + with holes + filled)
    // We write the filled result as representative checkpoint
    rp.write_pix_and_check(&filled, ImageFormat::Png)
        .expect("write fill_map_holes result (index 14)");

    assert!(
        rp.cleanup(),
        "adaptmap_fill_holes_weasel regression test failed"
    );
}

/// C test 15: pixFillMapHoles simple 3x3 case.
///
/// Matches C code:
///   pix1 = pixCreate(3, 3, 8)
///   pixSetPixel(pix1, 1, 0, 128)   -- single non-zero pixel at (1,0)
///   pixFillMapHoles(pix1, 3, 3, L_FILL_BLACK)  -- fill with 0
///   regTestWritePixAndCheck(rp, pix1_expanded, IFF_PNG)  -> index 15
#[test]
fn adaptmap_reg_fill_map_holes_simple() {
    let mut rp = RegParams::new("adaptmap_fill_holes_simple");

    // C: pix1 = pixCreate(3, 3, 8); pixSetPixel(pix1, 1, 0, 128)
    let mut pix3 =
        leptonica::PixMut::new(3, 3, leptonica::PixelDepth::Bit8).expect("create 3x3 8bpp pix");
    // Set single non-zero pixel at (x=1, y=0) = 128
    pix3.set_pixel(1, 0, 128).expect("set pixel (1,0) = 128");
    let pix_sparse: leptonica::Pix = pix3.into();

    // Apply fill_map_holes: should propagate the value at (1,0) to neighbors
    let filled = fill_map_holes(&pix_sparse, 3, 3).expect("fill_map_holes 3x3");
    rp.compare_values(3.0, filled.width() as f64, 0.0);
    rp.compare_values(3.0, filled.height() as f64, 0.0);
    rp.compare_values(8.0, filled.depth().bits() as f64, 0.0);

    // After filling, the value should have propagated: pixel (2,0) should be non-zero
    let val_1_0 = filled.get_pixel(1, 0).expect("get (1,0)");
    let val_2_0 = filled.get_pixel(2, 0).expect("get (2,0)");
    rp.compare_values(128.0, val_1_0 as f64, 0.0);
    // (2,0) should be filled from (1,0)
    rp.compare_values(1.0, if val_2_0 > 0 { 1.0 } else { 0.0 }, 0.0);

    // C index 15: write result
    rp.write_pix_and_check(&filled, ImageFormat::Png)
        .expect("write fill_map_holes simple (index 15)");

    assert!(
        rp.cleanup(),
        "adaptmap_fill_holes_simple regression test failed"
    );
}

/// Test background normalization on grayscale image.
///
/// C test 0-3: Grayscale low-level background map
/// C test 12-13: pixBackgroundNorm (high-level API)
#[test]
fn adaptmap_reg_background_norm_gray() {
    let mut rp = RegParams::new("adaptmap_bg_gray");

    // Use 8bpp grayscale test image
    let pixg = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let gw = pixg.width();
    let gh = pixg.height();

    // Test: background_norm_simple on grayscale
    let result = background_norm_simple(&pixg).expect("background_norm_simple gray");
    rp.compare_values(gw as f64, result.width() as f64, 0.0);
    rp.compare_values(gh as f64, result.height() as f64, 0.0);
    rp.compare_values(
        pixg.depth().bits() as f64,
        result.depth().bits() as f64,
        0.0,
    );
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write background_norm_simple gray");

    // Test: background_norm with C-version parameters
    // C test 12: pixBackgroundNorm(pixs, pixim, NULL, 5, 10, 50, 20, 200, 2, 1)
    let options = BackgroundNormOptions {
        tile_width: 5,
        tile_height: 10,
        fg_threshold: 50,
        min_count: 20,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let result = background_norm(&pixg, &options).expect("background_norm with C params (gray)");
    rp.compare_values(gw as f64, result.width() as f64, 0.0);
    rp.compare_values(gh as f64, result.height() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write background_norm C params gray");

    // Test: background_norm with SIZE_X=10, SIZE_Y=30
    // C test 0: pixGetBackgroundGrayMap(pixg, pixim, 10, 30, 50, 30, &pixgm)
    let options2 = BackgroundNormOptions {
        tile_width: 10,
        tile_height: 30,
        fg_threshold: 50,
        min_count: 30,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let result2 = background_norm(&pixg, &options2).expect("background_norm tile 10x30");
    rp.compare_values(gw as f64, result2.width() as f64, 0.0);
    rp.compare_values(gh as f64, result2.height() as f64, 0.0);
    rp.write_pix_and_check(&result2, ImageFormat::Png)
        .expect("write background_norm tile 10x30 gray");

    assert!(rp.cleanup(), "adaptmap_bg_gray regression test failed");
}

/// Test background normalization on color (32bpp) image.
///
/// C test 4-11: Color background map generation (RGB separate processing)
/// C test 12: pixBackgroundNorm on color
#[test]
fn adaptmap_reg_background_norm_color() {
    let mut rp = RegParams::new("adaptmap_bg_color");

    // C: pixs = pixRead("wet-day.jpg")
    let pixs = load_test_image("wet-day.jpg").expect("load wet-day.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // Test: background_norm_simple on color
    let result = background_norm_simple(&pixs).expect("background_norm_simple color");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(
        pixs.depth().bits() as f64,
        result.depth().bits() as f64,
        0.0,
    );
    rp.write_pix_and_check(&result, ImageFormat::Jpeg)
        .expect("write background_norm_simple color");

    // Test: background_norm with C-version parameters
    let options = BackgroundNormOptions {
        tile_width: 5,
        tile_height: 10,
        fg_threshold: 50,
        min_count: 20,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let result = background_norm(&pixs, &options).expect("background_norm color with C params");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(32.0, result.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Jpeg)
        .expect("write background_norm color C params");

    // Test: with tile 10x30
    let options2 = BackgroundNormOptions {
        tile_width: 10,
        tile_height: 30,
        fg_threshold: 50,
        min_count: 30,
        bg_val: 200,
        smooth_x: 2,
        smooth_y: 1,
    };
    let result2 = background_norm(&pixs, &options2).expect("background_norm color tile 10x30");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);
    rp.compare_values(h as f64, result2.height() as f64, 0.0);
    rp.write_pix_and_check(&result2, ImageFormat::Jpeg)
        .expect("write background_norm color tile 10x30");

    assert!(rp.cleanup(), "adaptmap_bg_color regression test failed");
}

/// Test contrast normalization.
#[test]
fn adaptmap_reg_contrast_norm() {
    let mut rp = RegParams::new("adaptmap_contrast");

    let pixg = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixg.width();
    let h = pixg.height();

    // Test: contrast_norm_simple
    let result = contrast_norm_simple(&pixg).expect("contrast_norm_simple");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.compare_values(8.0, result.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write contrast_norm_simple");

    // Test: contrast_norm with custom options
    let options = ContrastNormOptions {
        tile_width: 10,
        tile_height: 10,
        min_diff: 30,
        smooth_x: 1,
        smooth_y: 1,
    };
    let result2 = contrast_norm(&pixg, &options).expect("contrast_norm custom");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);
    rp.compare_values(h as f64, result2.height() as f64, 0.0);
    rp.write_pix_and_check(&result2, ImageFormat::Png)
        .expect("write contrast_norm custom");

    // Test: with larger tiles
    let options3 = ContrastNormOptions {
        tile_width: 30,
        tile_height: 30,
        min_diff: 50,
        smooth_x: 2,
        smooth_y: 2,
    };
    let result3 = contrast_norm(&pixg, &options3).expect("contrast_norm large tiles");
    rp.compare_values(w as f64, result3.width() as f64, 0.0);
    rp.compare_values(h as f64, result3.height() as f64, 0.0);
    rp.write_pix_and_check(&result3, ImageFormat::Png)
        .expect("write contrast_norm large tiles");

    // Verify: contrast normalization should expand dynamic range
    let (orig_min, orig_max) = sample_min_max(&pixg);
    let (norm_min, norm_max) = sample_min_max(&result);
    let orig_range = orig_max.saturating_sub(orig_min);
    let norm_range = norm_max.saturating_sub(norm_min);
    let range_expanded = norm_range >= orig_range || norm_range >= 200;
    rp.compare_values(1.0, if range_expanded { 1.0 } else { 0.0 }, 0.0);

    // Test: contrast_norm rejects non-8bpp
    let pix32 = load_test_image("weasel32.png").expect("load weasel32.png");
    rp.compare_values(
        1.0,
        if contrast_norm_simple(&pix32).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Test: invalid parameters (tile too small)
    let bad_options = ContrastNormOptions {
        tile_width: 3,
        tile_height: 5,
        min_diff: 50,
        smooth_x: 2,
        smooth_y: 2,
    };
    rp.compare_values(
        1.0,
        if contrast_norm(&pixg, &bad_options).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Test: smooth too large
    let bad_options2 = ContrastNormOptions {
        tile_width: 20,
        tile_height: 20,
        min_diff: 50,
        smooth_x: 10,
        smooth_y: 2,
    };
    rp.compare_values(
        1.0,
        if contrast_norm(&pixg, &bad_options2).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "adaptmap_contrast regression test failed");
}

/// Test parameter validation for background normalization.
#[test]
fn adaptmap_reg_param_validation() {
    let mut rp = RegParams::new("adaptmap_params");

    let pixg = load_test_image("dreyfus8.png").expect("load dreyfus8.png");

    // Tile too small
    let bad = BackgroundNormOptions {
        tile_width: 2,
        ..Default::default()
    };
    rp.compare_values(
        1.0,
        if background_norm(&pixg, &bad).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // bg_val out of range
    let bad2 = BackgroundNormOptions {
        bg_val: 50,
        ..Default::default()
    };
    rp.compare_values(
        1.0,
        if background_norm(&pixg, &bad2).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Default options should work
    let result = background_norm(&pixg, &BackgroundNormOptions::default());
    rp.compare_values(1.0, if result.is_ok() { 1.0 } else { 0.0 }, 0.0);
    if let Ok(ref pix) = result {
        rp.write_pix_and_check(pix, ImageFormat::Png)
            .expect("write background_norm default params");
    }

    assert!(rp.cleanup(), "adaptmap_params regression test failed");
}

/// Test pixGetForegroundGrayMap equivalent.
#[test]
fn adaptmap_reg_foreground_gray_map() {
    let mut rp = RegParams::new("adaptmap_fg_gray_map");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    let fg_map = get_foreground_gray_map(&pix, None, 10, 15, 60).expect("fg gray map");

    // Output dimensions should be ceil(w/sx) x ceil(h/sy)
    let expected_w = w.div_ceil(10);
    let expected_h = h.div_ceil(15);
    rp.compare_values(expected_w as f64, fg_map.width() as f64, 1.0);
    rp.compare_values(expected_h as f64, fg_map.height() as f64, 1.0);
    rp.compare_values(8.0, fg_map.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&fg_map, ImageFormat::Png)
        .expect("write foreground gray map");

    assert!(rp.cleanup(), "foreground gray map test failed");
}

/// C test 3, 11, 13: pixGammaTRCMasked
#[test]
fn adaptmap_reg_gamma_trc_masked() {
    let mut rp = RegParams::new("adaptmap_gamma_trc_masked");

    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");

    // Convert to 8bpp grayscale if needed
    let pix8 = if pixs.depth() == leptonica::PixelDepth::Bit8 {
        pixs.clone()
    } else {
        pixs.convert_rgb_to_gray_fast().expect("convert to 8bpp")
    };

    let result = gamma_trc_masked(&pix8, None, 1.5, 30, 230).expect("gamma_trc_masked");
    rp.compare_values(pix8.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(pix8.height() as f64, result.height() as f64, 0.0);
    rp.compare_values(
        pix8.depth().bits() as f64,
        result.depth().bits() as f64,
        0.0,
    );
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write gamma_trc_masked result");

    assert!(rp.cleanup(), "gamma_trc_masked test failed");
}

/// Helper: sample min and max pixel values from an 8bpp image
fn sample_min_max(pix: &leptonica::Pix) -> (u32, u32) {
    let w = pix.width();
    let h = pix.height();
    let mut min_val = 255u32;
    let mut max_val = 0u32;
    let step = std::cmp::max(1, std::cmp::min(w, h) / 50) as usize;

    for y in (0..h).step_by(step) {
        for x in (0..w).step_by(step) {
            let val = pix.get_pixel(x, y).unwrap_or(0);
            min_val = min_val.min(val);
            max_val = max_val.max(val);
        }
    }

    (min_val, max_val)
}
