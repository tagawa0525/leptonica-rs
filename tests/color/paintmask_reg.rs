//! Paint mask regression test
//!
//! Tests painting through a mask onto various depth images.
//! The C version creates masks and uses pixClipMasked to clip and paint
//! onto images of depths from 1bpp to 32bpp.
//!
//! Partial migration: paint_through_mask on 32bpp with a clipped mask
//! and median_cut_quant for quantization are tested. pixClipMasked and
//! multi-depth painting operations are not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/paintmask_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::MedianCutOptions;
use leptonica::color::median_cut_quant;

/// Test paint_through_mask on 32bpp with clipped mask (C check 1 setup).
///
/// Creates a mask from rabi.png, clips and inverts it, then paints through
/// the mask onto a 32bpp image.
#[test]
fn paintmask_reg_32bpp() {
    let mut rp = RegParams::new("pmask_32");

    // C: pixs = pixRead("test24.jpg");
    let pixs = crate::common::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pixs.depth(), PixelDepth::Bit32);
    let w = pixs.width();
    let h = pixs.height();

    // C: pixt1 = pixRead("rabi.png");
    //    box = boxCreate(303, 1983, 800, 500);
    //    pixm = pixClipRectangle(pixt1, box, NULL);
    //    pixInvert(pixm, pixm);
    let rabi = crate::common::load_test_image("rabi.png").expect("load rabi.png");
    let mask = rabi.clip_rectangle(303, 1983, 800, 500).expect("clip mask");
    let mask = mask.invert();
    assert_eq!(mask.depth(), PixelDepth::Bit1);

    // C: pixPaintThroughMask(pixt, pixb, box->x, box->y, val32);
    let val = leptonica::core::pixel::compose_rgb(3, 192, 128);
    let mut pixmut = pixs.try_into_mut().expect("try_into_mut");
    pixmut
        .paint_through_mask(&mask, 100, 100, val)
        .expect("paint_through_mask on 32bpp");
    let result: leptonica::Pix = pixmut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "paintmask 32bpp test failed");
}

/// Test median_cut_quant + clip_rectangle workflow (C checks 2-3 setup).
///
/// Quantizes a 32bpp image and clips a region.
#[test]
fn paintmask_reg_quant_clip() {
    let mut rp = RegParams::new("pmask_qclip");

    let pixs = crate::common::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pixs.depth(), PixelDepth::Bit32);

    // C: pixt1 = pixMedianCutQuant(pixs, 0);
    let options = MedianCutOptions::default();
    let quant = median_cut_quant(&pixs, &options).expect("median_cut_quant");
    let qw = quant.width();
    let qh = quant.height();
    rp.compare_values(pixs.width() as f64, qw as f64, 0.0);
    rp.compare_values(pixs.height() as f64, qh as f64, 0.0);

    // C: pixt2 = pixClipRectangle(pixt1, box, NULL); box=(100,100,800,500)
    let clipped = quant
        .clip_rectangle(100, 100, 400, 300)
        .expect("clip quantized");
    rp.compare_values(400.0, clipped.width() as f64, 0.0);
    rp.compare_values(300.0, clipped.height() as f64, 0.0);

    assert!(rp.cleanup(), "paintmask quant+clip test failed");
}

/// Test pixClipMasked on multiple depths (C checks 1-21).
///
/// Requires pixClipMasked which is not available in the Rust API.
/// Also requires pixOctreeQuantNumColors, pixConvertRGBToLuminance,
/// and multi-depth threshold/paint operations.
#[test]
#[ignore = "not yet implemented: pixClipMasked not available"]
fn paintmask_reg_clip_masked() {
    // C version:
    // pixd = pixClipMasked(pixs, pixm, 100, 100, 0x03c08000);  -- 32bpp
    // pixd = pixClipMasked(pixt1, pixm, 100, 100, 0x03c08000);  -- 8bpp cmap
    // pixd = pixClipMasked(pixt1, pixm, 100, 100, 0x03c08000);  -- 4bpp cmap
    // pixd = pixClipMasked(pixs8, pixm, 100, 100, 90);          -- 8bpp gray
    // etc.
}
