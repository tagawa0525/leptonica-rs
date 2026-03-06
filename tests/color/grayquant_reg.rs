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
//! C Leptonica: `reference/leptonica/prog/grayquant_reg.c`

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
