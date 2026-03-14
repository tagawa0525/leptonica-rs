//! Paint mask regression test
//!
//! Tests painting through a mask onto various depth images.
//! The C version creates masks and uses pixClipMasked to clip and paint
//! onto images of depths from 1bpp to 32bpp.
//!
//! Partial migration: paint_through_mask on 32bpp with a clipped mask,
//! median_cut_quant for quantization, and clip_masked on 32bpp/8bpp
//! are tested.
//!
//! # See also
//!
//! C Leptonica: `prog/paintmask_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::{MedianCutOptions, median_cut_quant};
use leptonica::io::ImageFormat;

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
    let mut pixmut = pixs.to_mut();
    pixmut
        .paint_through_mask(&mask, 100, 100, val)
        .expect("paint_through_mask on 32bpp");
    let result: leptonica::Pix = pixmut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write painted paint_32bpp");

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
    rp.write_pix_and_check(&clipped, ImageFormat::Png)
        .expect("write clipped quant_clip");

    assert!(rp.cleanup(), "paintmask quant+clip test failed");
}

/// Test pixClipMasked on multiple depths (C checks 1, 3, 5, 7, 9, 11, 17, 20-21).
///
/// Tests clip_masked with a 1bpp mask on images of various depths.
#[test]
fn paintmask_reg_clip_masked() {
    let mut rp = RegParams::new("pmask_clip");

    // 32bpp source
    let pixs = crate::common::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pixs.depth(), PixelDepth::Bit32);

    // Create a 1bpp mask from rabi.png
    let rabi = crate::common::load_test_image("rabi.png").expect("load rabi.png");
    let mask = rabi.clip_rectangle(303, 1983, 800, 500).expect("clip mask");
    let mask = mask.invert();
    assert_eq!(mask.depth(), PixelDepth::Bit1);

    // C check 1: pixClipMasked(pixs, pixm, 100, 100, 0x03c08000) — 32bpp
    let result = pixs
        .clip_masked(&mask, 100, 100, 0x03c0_8000)
        .expect("clip_masked 32bpp");
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.compare_values(mask.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(mask.height() as f64, result.height() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("check: clip_masked 32bpp");

    // C check 9: pixClipMasked on 8bpp grayscale — outval=90
    let pix8 = pixs.convert_to_8().expect("convert to 8bpp");
    let result8 = pix8
        .clip_masked(&mask, 100, 100, 90)
        .expect("clip_masked 8bpp");
    assert_eq!(result8.depth(), PixelDepth::Bit8);
    rp.compare_values(mask.width() as f64, result8.width() as f64, 0.0);
    rp.write_pix_and_check(&result8, ImageFormat::Png)
        .expect("check: clip_masked 8bpp gray");

    // C check 11: pixClipMasked on 4bpp gray — outval=0
    let pix4 =
        leptonica::color::threshold_to_4bpp(&pix8, 9, false).expect("threshold_to_4bpp for clip");
    let result4 = pix4
        .clip_masked(&mask, 100, 100, 0)
        .expect("clip_masked 4bpp");
    assert_eq!(result4.depth(), PixelDepth::Bit4);
    rp.write_pix_and_check(&result4, ImageFormat::Png)
        .expect("check: clip_masked 4bpp gray");

    // C check 17: pixClipMasked on 2bpp gray — outval=1
    let pix2 =
        leptonica::color::threshold_to_2bpp(&pix8, 4, false).expect("threshold_to_2bpp for clip");
    let result2 = pix2
        .clip_masked(&mask, 100, 100, 1)
        .expect("clip_masked 2bpp");
    assert_eq!(result2.depth(), PixelDepth::Bit2);
    rp.write_pix_and_check(&result2, ImageFormat::Png)
        .expect("check: clip_masked 2bpp gray");

    // C check 21: pixClipMasked on 1bpp — outval=1
    let pix1 = leptonica::color::threshold_to_binary(&pix8, 128).expect("threshold for 1bpp");
    let result1 = pix1
        .clip_masked(&mask, 100, 100, 1)
        .expect("clip_masked 1bpp");
    assert_eq!(result1.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&result1, ImageFormat::Png)
        .expect("check: clip_masked 1bpp");

    assert!(rp.cleanup(), "paintmask clip_masked test failed");
}
