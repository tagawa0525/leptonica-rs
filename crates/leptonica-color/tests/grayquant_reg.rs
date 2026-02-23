//! Gray quantization regression test
//!
//! Tests gray thresholding to 1, 2, and 4 bpp, plus color quantization.
//! The C version tests various threshold levels with and without colormaps.
//!
//! Partial migration: threshold_to_binary, threshold_to_2bpp, threshold_to_4bpp,
//! median_cut_quant, octree_quant_256, fixed_octcube_quant_256, and
//! octree_quant_by_population are tested. pixThresholdOn8bpp, pixThreshold8,
//! pixThresholdGrayArb, and colormap highlight operations are not available.
//! Test image stampede2.jpg is not available; test8.jpg is used instead.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/grayquant_reg.c`

use leptonica_color::{
    MedianCutOptions, fixed_octcube_quant_256, median_cut_quant, octree_quant_256,
    octree_quant_by_population, threshold_to_2bpp, threshold_to_4bpp, threshold_to_binary,
};
use leptonica_core::PixelDepth;
use leptonica_test::RegParams;

/// Test threshold_to_binary (C check 0: pixThresholdToBinary).
///
/// Converts 8bpp grayscale to 1bpp at a given threshold.
#[test]
fn grayquant_reg_threshold_binary() {
    let mut rp = RegParams::new("gquant_bin");

    let pix = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // C: pix1 = pixThresholdToBinary(pixs, THRESHOLD=130);
    let result = threshold_to_binary(&pix, 130).expect("threshold_to_binary 130");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit1);

    assert!(rp.cleanup(), "grayquant threshold_to_binary test failed");
}

/// Test threshold_to_2bpp and threshold_to_4bpp (C checks 5-12).
///
/// Thresholds 8bpp gray to 2bpp and 4bpp with various levels.
#[test]
fn grayquant_reg_threshold_multi() {
    let mut rp = RegParams::new("gquant_multi");

    let pix = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // C: pix1 = pixThresholdTo2bpp(pixs, 4, 1); -- with colormap
    let result_2bpp_cmap = threshold_to_2bpp(&pix, 4, true).expect("threshold_to_2bpp 4 cmap");
    rp.compare_values(w as f64, result_2bpp_cmap.width() as f64, 0.0);
    rp.compare_values(h as f64, result_2bpp_cmap.height() as f64, 0.0);
    assert_eq!(result_2bpp_cmap.depth(), PixelDepth::Bit2);

    // C: pix2 = pixThresholdTo2bpp(pixs, 4, 0); -- without colormap
    let result_2bpp_no = threshold_to_2bpp(&pix, 4, false).expect("threshold_to_2bpp 4 no cmap");
    rp.compare_values(w as f64, result_2bpp_no.width() as f64, 0.0);
    assert_eq!(result_2bpp_no.depth(), PixelDepth::Bit2);

    // C: pix1 = pixThresholdTo2bpp(pixs, 3, 1);
    let result_2bpp_3 = threshold_to_2bpp(&pix, 3, true).expect("threshold_to_2bpp 3 cmap");
    rp.compare_values(w as f64, result_2bpp_3.width() as f64, 0.0);
    assert_eq!(result_2bpp_3.depth(), PixelDepth::Bit2);

    // C: pix1 = pixThresholdTo4bpp(pixs, 9, 1); -- with colormap
    let result_4bpp_cmap = threshold_to_4bpp(&pix, 9, true).expect("threshold_to_4bpp 9 cmap");
    rp.compare_values(w as f64, result_4bpp_cmap.width() as f64, 0.0);
    rp.compare_values(h as f64, result_4bpp_cmap.height() as f64, 0.0);
    assert_eq!(result_4bpp_cmap.depth(), PixelDepth::Bit4);

    // C: pix2 = pixThresholdTo4bpp(pixs, 9, 0); -- without colormap
    let result_4bpp_no = threshold_to_4bpp(&pix, 9, false).expect("threshold_to_4bpp 9 no cmap");
    rp.compare_values(w as f64, result_4bpp_no.width() as f64, 0.0);
    assert_eq!(result_4bpp_no.depth(), PixelDepth::Bit4);

    assert!(rp.cleanup(), "grayquant threshold multi test failed");
}

/// Test color quantization on 32bpp RGB (related to quantization checks).
///
/// Tests median_cut_quant, octree_quant_256, fixed_octcube_quant_256,
/// and octree_quant_by_population.
#[test]
fn grayquant_reg_color_quant() {
    let mut rp = RegParams::new("gquant_cquant");

    let pix = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Median cut quantization
    let options = MedianCutOptions::default();
    let result = median_cut_quant(&pix, &options).expect("median_cut_quant");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);

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
/// Requires pixThresholdOn8bpp, pixThreshold8, pixThresholdGrayArb,
/// pixSetSelectCmap, pixConvertGrayToColormap which are not available.
/// Test image stampede2.jpg is also not available.
#[test]
#[ignore = "not yet implemented: pixThresholdOn8bpp/pixThreshold8/pixThresholdGrayArb not available"]
fn grayquant_reg_advanced_threshold() {
    // C version:
    // pix1 = pixThresholdOn8bpp(pixs, 9, 1);
    // pix1 = pixThreshold8(pixs, 1, 2, 1);
    // pix1 = pixThresholdGrayArb(pixs, "45 75 115 185", 8, 0, 0, 0);
}
