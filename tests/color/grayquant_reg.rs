//! Gray quantization regression test
//!
//! Tests gray thresholding to 1, 2, and 4 bpp, plus color quantization.
//! The C version tests various threshold levels with and without colormaps.
//!
//! Partial migration: threshold_to_binary, threshold_to_2bpp, threshold_to_4bpp,
//! median_cut_quant, octree_quant_256, fixed_octcube_quant_256,
//! octree_quant_by_population, threshold_on_8bpp, threshold_8, and
//! threshold_gray_arb are tested.
//!
//! # See also
//!
//! C Leptonica: `prog/grayquant_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::{
    MedianCutOptions, dither_to_2bpp, dither_to_2bpp_spec, fixed_octcube_quant_256,
    median_cut_quant, octree_quant_256, octree_quant_by_population, threshold_gray_arb,
    threshold_on_8bpp, threshold_to_2bpp, threshold_to_4bpp, threshold_to_binary,
};
use leptonica::io::ImageFormat;

/// Test threshold_to_binary (C check 0: pixThresholdToBinary).
///
/// Converts 8bpp grayscale to 1bpp at a given threshold.
#[test]
fn grayquant_reg_threshold_binary() {
    let mut rp = RegParams::new("gquant_bin");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // C: pix1 = pixThresholdToBinary(pixs, THRESHOLD=130);
    let result = threshold_to_binary(&pix, 130).expect("threshold_to_binary 130");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&result, ImageFormat::Tiff)
        .expect("write result threshold_binary");

    assert!(rp.cleanup(), "grayquant threshold_to_binary test failed");
}

/// Test dither_to_2bpp (C checks 1-2).
///
/// C: pixDitherTo2bpp(pixs, 1/0) with and without colormap.
#[test]
fn grayquant_reg_dither_2bpp() {
    let mut rp = RegParams::new("gquant_dither");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // C check 1: pixDitherTo2bpp(pixs, 1) — with colormap (Rust doesn't have cmap variant)
    let result = dither_to_2bpp(&pix).expect("dither_to_2bpp");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit2);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("check: dither_to_2bpp");

    // C check 2: pixDitherTo2bpp(pixs, 0) — without colormap
    // Rust dither_to_2bpp_spec with distinct thresholds to exercise a different config
    let result2 = dither_to_2bpp_spec(&pix, 51, 85, 170).expect("dither_to_2bpp_spec");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);
    assert_eq!(result2.depth(), PixelDepth::Bit2);
    rp.write_pix_and_check(&result2, ImageFormat::Png)
        .expect("check: dither_to_2bpp_spec");

    assert!(rp.cleanup(), "grayquant dither_2bpp test failed");
}

/// Test threshold_to_2bpp and threshold_to_4bpp (C checks 5-12).
///
/// Thresholds 8bpp gray to 2bpp and 4bpp with various levels.
#[test]
fn grayquant_reg_threshold_multi() {
    let mut rp = RegParams::new("gquant_multi");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // C check 5: pixThresholdTo2bpp(pixs, 4, 1) — with colormap
    let result_2bpp_cmap = threshold_to_2bpp(&pix, 4, true).expect("threshold_to_2bpp 4 cmap");
    rp.compare_values(w as f64, result_2bpp_cmap.width() as f64, 0.0);
    rp.compare_values(h as f64, result_2bpp_cmap.height() as f64, 0.0);
    assert_eq!(result_2bpp_cmap.depth(), PixelDepth::Bit2);
    rp.write_pix_and_check(&result_2bpp_cmap, ImageFormat::Png)
        .expect("check: threshold_to_2bpp 4 cmap");

    // C check 6: pixThresholdTo2bpp(pixs, 4, 0) — without colormap
    let result_2bpp_no = threshold_to_2bpp(&pix, 4, false).expect("threshold_to_2bpp 4 no cmap");
    rp.compare_values(w as f64, result_2bpp_no.width() as f64, 0.0);
    assert_eq!(result_2bpp_no.depth(), PixelDepth::Bit2);
    rp.write_pix_and_check(&result_2bpp_no, ImageFormat::Png)
        .expect("check: threshold_to_2bpp 4 no cmap");

    // C check 9: pixThresholdTo2bpp(pixs, 3, 1)
    let result_2bpp_3 = threshold_to_2bpp(&pix, 3, true).expect("threshold_to_2bpp 3 cmap");
    rp.compare_values(w as f64, result_2bpp_3.width() as f64, 0.0);
    assert_eq!(result_2bpp_3.depth(), PixelDepth::Bit2);

    // C check 10: pixThresholdTo2bpp(pixs, 3, 0)
    let result_2bpp_3n = threshold_to_2bpp(&pix, 3, false).expect("threshold_to_2bpp 3 no cmap");
    rp.compare_values(w as f64, result_2bpp_3n.width() as f64, 0.0);

    // C check 11: pixThresholdTo4bpp(pixs, 9, 1) — with colormap
    let result_4bpp_cmap = threshold_to_4bpp(&pix, 9, true).expect("threshold_to_4bpp 9 cmap");
    rp.compare_values(w as f64, result_4bpp_cmap.width() as f64, 0.0);
    rp.compare_values(h as f64, result_4bpp_cmap.height() as f64, 0.0);
    assert_eq!(result_4bpp_cmap.depth(), PixelDepth::Bit4);
    rp.write_pix_and_check(&result_4bpp_cmap, ImageFormat::Png)
        .expect("check: threshold_to_4bpp 9 cmap");

    // C check 12: pixThresholdTo4bpp(pixs, 9, 0) — without colormap
    let result_4bpp_no = threshold_to_4bpp(&pix, 9, false).expect("threshold_to_4bpp 9 no cmap");
    rp.compare_values(w as f64, result_4bpp_no.width() as f64, 0.0);
    assert_eq!(result_4bpp_no.depth(), PixelDepth::Bit4);
    rp.write_pix_and_check(&result_4bpp_no, ImageFormat::Png)
        .expect("check: threshold_to_4bpp 9 no cmap");

    assert!(rp.cleanup(), "grayquant threshold multi test failed");
}

/// Test color quantization on 32bpp RGB (related to quantization checks).
///
/// Tests median_cut_quant, octree_quant_256, fixed_octcube_quant_256,
/// and octree_quant_by_population.
#[test]
fn grayquant_reg_color_quant() {
    let mut rp = RegParams::new("gquant_cquant");

    let pix = crate::common::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Median cut quantization
    let options = MedianCutOptions::default();
    let result = median_cut_quant(&pix, &options).expect("median_cut_quant");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result color_quant");

    // Octree quantization to 256 colors
    let result2 = octree_quant_256(&pix).expect("octree_quant_256");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    // Fixed octcube quantization to 256 colors
    let result3 = fixed_octcube_quant_256(&pix).expect("fixed_octcube_quant_256");
    rp.compare_values(w as f64, result3.width() as f64, 0.0);

    // Octree quantization by population
    let result4 = octree_quant_by_population(&pix, 4).expect("octree_quant_by_population");
    rp.compare_values(w as f64, result4.width() as f64, 0.0);

    assert!(rp.cleanup(), "grayquant color quant test failed");
}

/// Test pixThresholdOn8bpp, pixThreshold8, pixThresholdGrayArb (C checks 14-49).
///
/// Tests threshold_on_8bpp, threshold_8 (on Pix), and threshold_gray_arb.
#[test]
fn grayquant_reg_advanced_threshold() {
    let mut rp = RegParams::new("gquant_adv");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let pix8 = pix.convert_to_8().expect("convert to 8bpp");
    let w = pix8.width();
    let h = pix8.height();

    // C check 14: pixThresholdOn8bpp(pixs, 9, 1) — with colormap
    let result = threshold_on_8bpp(&pix8, 9, true).expect("threshold_on_8bpp 9 cmap");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("check: threshold_on_8bpp 9 cmap");

    // C check 15: pixThresholdOn8bpp(pixs, 9, 0) — without colormap
    let result2 = threshold_on_8bpp(&pix8, 9, false).expect("threshold_on_8bpp 9 no cmap");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);
    assert_eq!(result2.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&result2, ImageFormat::Png)
        .expect("check: threshold_on_8bpp 9 no cmap");

    // C check 20: pixThreshold8(pixs, 1, 2, 1)
    let result3 = pix8.threshold_8(1, 2, true).expect("threshold_8 depth=1");
    rp.compare_values(w as f64, result3.width() as f64, 0.0);
    assert_eq!(result3.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&result3, ImageFormat::Tiff)
        .expect("check: threshold_8 1,2 cmap");

    // C check 44: pixThresholdGrayArb(pixs, "45 75 115 185", 8, ...)
    let result4 = threshold_gray_arb(&pix8, "45 75 115 185").expect("threshold_gray_arb 4 bounds");
    rp.compare_values(w as f64, result4.width() as f64, 0.0);
    rp.compare_values(h as f64, result4.height() as f64, 0.0);
    rp.write_pix_and_check(&result4, ImageFormat::Png)
        .expect("check: threshold_gray_arb 4 bounds");

    // C check 45: pixThresholdGrayArb(pixs, "38 65 85 115 160 210", 8, ...)
    let result5 =
        threshold_gray_arb(&pix8, "38 65 85 115 160 210").expect("threshold_gray_arb 6 bounds");
    rp.compare_values(w as f64, result5.width() as f64, 0.0);
    rp.write_pix_and_check(&result5, ImageFormat::Png)
        .expect("check: threshold_gray_arb 6 bounds");

    assert!(rp.cleanup(), "grayquant advanced threshold test failed");
}

/// C-compat: `prog/grayquant_reg.c` checks 28-39.
///
/// This is the `feyn.tif` block. Unlike the earlier checks in that program
/// it reads a lossless 1 bpp TIFF rather than `test8.jpg`, so it is free of
/// JPEG decode differences and can be compared bit-exactly against C.
#[test]
#[ignore = "not yet implemented"]
fn grayquant_c_compat() {
    use leptonica::color::paintcmap::pix_set_select_cmap;
    use leptonica::core::pix::RemoveColormapTarget;
    use leptonica::core::pix::RopOp;
    use leptonica::transform::{ScaleMethod, reduce_rank_binary_cascade, scale, scale_to_gray_4};

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("grayquant_c");

    let pixs = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");

    // C 28-30: 8 bpp reduction, rank-binary cascade, and 2 bpp thresholding
    // with one colormap entry repainted inside a box.
    let pix1 = scale_to_gray_4(&pixs).expect("scale to gray 4");
    let pix2 = reduce_rank_binary_cascade(&pixs, &[2, 2]).expect("rank cascade");
    let pix3 = threshold_to_2bpp(&pix1, 3, true).expect("threshold to 2bpp");
    rp.write_pix_and_check(&pix1, ImageFormat::Png)
        .expect("check: scale to gray 4");
    rp.write_pix_and_check(&pix2, ImageFormat::Png)
        .expect("check: rank binary cascade");

    let mut pm = pix3.try_into_mut().unwrap();
    pix_set_select_cmap(
        &mut pm,
        Some(&leptonica::core::Box::new(175, 208, 228, 88).unwrap()),
        2,
        (255, 255, 100),
    )
    .expect("set select cmap 2bpp");
    let pix3: leptonica::Pix = pm.into();
    rp.write_pix_and_check(&pix3, ImageFormat::Png)
        .expect("check: 2bpp with highlight");

    // C 31-32: 4 bpp thresholding with three highlighted boxes.
    const NLEVELS: u32 = 4;
    let pix2 = threshold_to_4bpp(&pix1, NLEVELS, true).expect("threshold to 4bpp");
    let mut pm = pix2.try_into_mut().unwrap();
    for (b, color) in [
        ((175, 208, 228, 83), (255, 255, 100)),
        ((232, 298, 110, 25), (100, 255, 255)),
        ((21, 698, 246, 82), (225, 100, 255)),
    ] {
        pix_set_select_cmap(
            &mut pm,
            Some(&leptonica::core::Box::new(b.0, b.1, b.2, b.3).unwrap()),
            NLEVELS - 1,
            color,
        )
        .expect("set select cmap 4bpp");
    }
    let pix2: leptonica::Pix = pm.into();
    rp.write_pix_and_check(&pix2, ImageFormat::Png)
        .expect("check: 4bpp with highlights");
    let pix3 = reduce_rank_binary_cascade(&pixs, &[2, 2]).expect("rank cascade 2");
    rp.write_pix_and_check(&pix3, ImageFormat::Png)
        .expect("check: rank binary cascade 2");

    // C 33-39: a 6x magnified crop thresholded to 4 bpp at 6, 5, 4, 3 and 2
    // levels, stacked into a single 8 bpp image.
    let crop = pix1.clip_rectangle(25, 202, 136, 37).expect("clip");
    let big = scale(&crop, 6.0, 6.0, ScaleMethod::Auto).expect("scale 6x");
    rp.write_pix_and_check(&big, ImageFormat::Png)
        .expect("check: 6x crop");

    let (w, h) = (big.width(), big.height());
    let stack = leptonica::Pix::new(w, 6 * h, PixelDepth::Bit8).expect("stack");
    let mut sm = stack.try_into_mut().unwrap();
    sm.rop_region_inplace(0, 0, w, h, RopOp::Src, &big, 0, 0)
        .expect("blit original");
    for (k, levels) in [6u32, 5, 4, 3, 2].into_iter().enumerate() {
        let quant = threshold_to_4bpp(&big, levels, true).expect("threshold to 4bpp levels");
        let gray = quant
            .remove_colormap(RemoveColormapTarget::ToGrayscale)
            .expect("remove cmap to gray");
        sm.rop_region_inplace(0, (k as i32 + 1) * h as i32, w, h, RopOp::Src, &gray, 0, 0)
            .expect("blit quantized");
        rp.write_pix_and_check(&quant, ImageFormat::Png)
            .expect("check: 4bpp levels");
    }
    let stack: leptonica::Pix = sm.into();
    rp.write_pix_and_check(&stack, ImageFormat::Png)
        .expect("check: stacked comparison");

    assert!(rp.cleanup(), "grayquant C-compat test failed");
}
