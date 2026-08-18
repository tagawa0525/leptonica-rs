//! Newspaper segmentation regression test
//!
//! Tests article segmentation from newspaper pages using morphological
//! operations at reduced resolution. The C version processes scots-frag.tif
//! through a multi-stage pipeline: scale reduction, line detection and
//! removal (horizontal/vertical), gutter detection, and article region
//! identification.
//!
//! Partial port: Tests the morphological pipeline components: scale_to_gray_4,
//! morph_sequence, seedfill_morph, XOR for line removal, and connected
//! component extraction. The full segmentation pipeline is tested using
//! segment_regions. The C version also uses pixReduceRankBinary2 and
//! pixExpandBinaryPower2 which are not available.
//!
//! # See also
//!
//! C Leptonica: `prog/newspaper_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::io::ImageFormat;
use leptonica::morph::{morph_sequence, seedfill_morph};
use leptonica::recog::pageseg::{PageSegOptions, segment_regions};
use leptonica::region::{ConnectivityType, conncomp_pixa};
use leptonica::transform::scale_to_gray_4;

/// Test scale_to_gray_4 reduction on scots-frag.tif (C test: pixScaleToGray4).
///
/// C: pixScaleToGray4(pixs)
///    Reduce binary image to 1/4 scale grayscale.
#[test]
fn newspaper_reg_scale_reduce() {
    let mut rp = RegParams::new("newspaper_scale");

    let pix = crate::common::load_test_image("scots-frag.tif").expect("load scots-frag.tif");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix.clone()
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };
    assert_eq!(pix_bin.depth(), PixelDepth::Bit1);

    // Reduce to 1/4 grayscale
    let reduced = scale_to_gray_4(&pix_bin).expect("scale_to_gray_4");
    assert_eq!(reduced.depth(), PixelDepth::Bit8);

    // Dimensions should be approximately 1/4
    let expected_w = pix_bin.width() / 4;
    let expected_h = pix_bin.height() / 4;
    rp.compare_values(expected_w as f64, reduced.width() as f64, 1.0);
    rp.compare_values(expected_h as f64, reduced.height() as f64, 1.0);

    rp.write_pix_and_check(&reduced, ImageFormat::Png)
        .expect("write reduced newspaper_scale");

    assert!(rp.cleanup(), "newspaper scale_reduce test failed");
}

/// Test morphological sequence for line detection (C test: morph pipeline).
///
/// C: pixMorphSequence(pixs, "c1.80 + c80.1", 0)
///    Detect horizontal and vertical lines via closing operations.
#[test]
fn newspaper_reg_line_detect() {
    let mut rp = RegParams::new("newspaper_lines");

    let pix = crate::common::load_test_image("scots-frag.tif").expect("load scots-frag.tif");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    // Detect horizontal lines: close with wide horizontal element
    let h_lines = morph_sequence(&pix_bin, "c80.1").expect("horizontal line detect");
    rp.compare_values(pix_bin.width() as f64, h_lines.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, h_lines.height() as f64, 0.0);

    rp.write_pix_and_check(&h_lines, ImageFormat::Tiff)
        .expect("write h_lines newspaper_lines");

    // Detect vertical lines: close with tall vertical element
    let v_lines = morph_sequence(&pix_bin, "c1.80").expect("vertical line detect");
    rp.compare_values(pix_bin.width() as f64, v_lines.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, v_lines.height() as f64, 0.0);

    assert!(rp.cleanup(), "newspaper line_detect test failed");
}

/// Test XOR-based line removal (C test: pixXor for removing lines).
///
/// C: pixXor(pixd, pixd, pix_hlines)  — remove horizontal lines
///    pixXor(pixd, pixd, pix_vlines)  — remove vertical lines
///
/// Rust: pix.xor() to remove detected lines from the document.
#[test]
fn newspaper_reg_line_removal() {
    let mut rp = RegParams::new("newspaper_removal");

    let pix = crate::common::load_test_image("scots-frag.tif").expect("load scots-frag.tif");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    // Detect lines
    let h_lines = morph_sequence(&pix_bin, "c80.1").expect("h_lines");
    let v_lines = morph_sequence(&pix_bin, "c1.80").expect("v_lines");

    // Remove horizontal lines via XOR
    let no_hlines = pix_bin.xor(&h_lines).expect("xor h_lines");
    rp.compare_values(pix_bin.width() as f64, no_hlines.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, no_hlines.height() as f64, 0.0);

    // Remove vertical lines via XOR
    let no_lines = no_hlines.xor(&v_lines).expect("xor v_lines");
    rp.compare_values(pix_bin.width() as f64, no_lines.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, no_lines.height() as f64, 0.0);

    rp.write_pix_and_check(&no_lines, ImageFormat::Tiff)
        .expect("write no_lines newspaper_removal");

    assert!(rp.cleanup(), "newspaper line_removal test failed");
}

/// Test seedfill morphological reconstruction (C test: pixSeedfillBinary).
///
/// C: pixSeedfillBinary(NULL, seed, mask, 8)
///    Reconstruct text blocks using seed + mask.
#[test]
fn newspaper_reg_seedfill() {
    let mut rp = RegParams::new("newspaper_seedfill");

    let pix = crate::common::load_test_image("scots-frag.tif").expect("load scots-frag.tif");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    // Create seed from dilated image (text blocks)
    let seed = morph_sequence(&pix_bin, "d5.5").expect("dilate seed");

    // Use original as mask — seedfill reconstructs within mask
    let filled = seedfill_morph(&seed, &pix_bin, 100, 8).expect("seedfill_morph");

    rp.compare_values(pix_bin.width() as f64, filled.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, filled.height() as f64, 0.0);
    assert_eq!(filled.depth(), PixelDepth::Bit1);

    rp.write_pix_and_check(&filled, ImageFormat::Tiff)
        .expect("write filled newspaper_seedfill");

    assert!(rp.cleanup(), "newspaper seedfill test failed");
}

/// Test article region detection via connected components.
///
/// C: pixConnComp(pixd, &pixa, 8) after segmentation
///    pixaDisplayRandomCmap(pixa, w, h)
///
/// Rust: segment_regions + conncomp_pixa for region detection.
#[test]
fn newspaper_reg_article_regions() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("newspaper_regions");

    let pix = crate::common::load_test_image("scots-frag.tif").expect("load scots-frag.tif");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    // Use segment_regions for page segmentation
    let opts = PageSegOptions::default();
    let seg = segment_regions(&pix_bin, &opts).expect("segment_regions");

    // Should produce a text block mask
    rp.compare_values(
        1.0,
        if seg.textblock_mask.width() > 0 && seg.textblock_mask.height() > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Extract connected components from the text block mask
    let (boxa, _pixa) =
        conncomp_pixa(&seg.textblock_mask, ConnectivityType::EightWay).expect("conncomp blocks");

    // Should find text regions in a newspaper page
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "newspaper article_regions test failed");
}

/// Test full pipeline: line removal + block detection on scots-frag.tif.
///
/// C: Full newspaper segmentation pipeline combining all steps.
///
/// Rust: Combine morph_sequence, XOR, and conncomp for text block extraction.
#[test]
fn newspaper_reg_full_pipeline() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("newspaper_pipeline");

    let pix = crate::common::load_test_image("scots-frag.tif").expect("load scots-frag.tif");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    // Step 1: Remove horizontal and vertical lines
    let h_lines = morph_sequence(&pix_bin, "c80.1").expect("detect h lines");
    let v_lines = morph_sequence(&pix_bin, "c1.80").expect("detect v lines");
    let no_lines = pix_bin
        .xor(&h_lines)
        .expect("remove h")
        .xor(&v_lines)
        .expect("remove v");

    // Step 2: Dilate to form text blocks
    let blocks = morph_sequence(&no_lines, "d10.10").expect("dilate blocks");

    // Step 3: Extract block regions
    let (boxa, _) = conncomp_pixa(&blocks, ConnectivityType::EightWay).expect("conncomp blocks");

    // Should find text block regions
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // Blocks should have reasonable sizes (> 10x10 pixels)
    let large_blocks: Vec<_> = (0..boxa.len())
        .filter_map(|i| boxa.get(i))
        .filter(|b| b.w > 10 && b.h > 10)
        .collect();
    rp.compare_values(1.0, if !large_blocks.is_empty() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "newspaper full_pipeline test failed");
}

/// C-compat: `prog/newspaper_reg.c` checks 1-12.
///
/// The whole program runs off the lossless `scots-frag.tif`, so every output
/// except check 0 (which C writes as JPEG) is bit-exactly comparable.
#[test]
fn newspaper_c_compat() {
    use leptonica::Pix;
    use leptonica::core::Pixa;
    use leptonica::morph::seedfill_morph;
    use leptonica::transform::{
        ScaleMethod, expand_binary_power2, reduce_rank_binary_2, reduce_rank_binary_cascade, scale,
    };

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("newspaper_c");
    let half = |p: &Pix| scale(p, 0.5, 0.5, ScaleMethod::Auto).expect("scale 0.5");

    let pixs = crate::common::load_test_image("scots-frag.tif").expect("load scots-frag.tif");

    // C 0 is a JPEG of pixScaleToGray4; skipped here (see the exclude rule).
    // Rank reduce 2x.
    let pix1 = reduce_rank_binary_2(&pixs, 2).expect("rank reduce 2");

    // C 1: open out the vertical lines.
    let pix2 = morph_sequence(&pix1, "o1.50").expect("open vertical");
    rp.write_pix_and_check(&half(&pix2), ImageFormat::Tiff)
        .expect("check: open vertical lines");

    // C 2: seedfill back to recover the full lines.
    let pix3 = seedfill_morph(&pix2, &pix1, 0, 8).expect("seedfill vertical");
    rp.write_pix_and_check(&half(&pix3), ImageFormat::Tiff)
        .expect("check: seedfill vertical");

    // C 3: remove the vertical lines.
    let pix2 = pix1.xor(&pix3).expect("xor vertical");
    rp.write_pix_and_check(&half(&pix2), ImageFormat::Tiff)
        .expect("check: remove vertical lines");

    // C 4: open out the horizontal lines, then seedfill them back.
    let pix4 = morph_sequence(&pix2, "o50.1").expect("open horizontal");
    let pix5 = seedfill_morph(&pix4, &pix2, 0, 8).expect("seedfill horizontal");
    rp.write_pix_and_check(&half(&pix5), ImageFormat::Tiff)
        .expect("check: seedfill horizontal");

    // C 5: remove the horizontal lines.
    let pix4 = pix2.xor(&pix5).expect("xor horizontal");
    rp.write_pix_and_check(&half(&pix4), ImageFormat::Tiff)
        .expect("check: remove horizontal lines");

    // C 6-8: invert and find the vertical gutters between text columns.
    let pix6 = reduce_rank_binary_cascade(&pix4, &[1, 1])
        .expect("rank cascade")
        .invert();
    let up2 = |p: &Pix| scale(p, 2.0, 2.0, ScaleMethod::Auto).expect("scale 2.0");
    rp.write_pix_and_check(&up2(&pix6), ImageFormat::Tiff)
        .expect("check: inverted gutters");
    let pix7 = morph_sequence(&pix6, "o1.50").expect("open gutters");
    rp.write_pix_and_check(&up2(&pix7), ImageFormat::Tiff)
        .expect("check: vertical gutters");
    let pix8 = expand_binary_power2(&pix7, 4).expect("expand gutter mask");
    rp.write_pix_and_check(&pix8, ImageFormat::Tiff)
        .expect("check: gutter mask");

    // C 9: solidify the text blocks, preserving the gutters.
    let pix9 = morph_sequence(&pix4, "c50.1 + c1.10").expect("close text");
    let pix9 = pix9.subtract(&pix8).expect("preserve gutter");
    let pix10 = morph_sequence(&pix9, "d3.3").expect("dilate text");
    rp.write_pix_and_check(&half(&pix10), ImageFormat::Tiff)
        .expect("check: solidified text");

    // C 10: show what lies under the mask, one random colormap index per
    // connected component.
    let (w, h) = (pix10.width(), pix10.height());
    let (_, pixa2) = conncomp_pixa(&pix10, ConnectivityType::EightWay).expect("conn comp");
    let pix11 = pixa2.display_random_cmap(w, h).expect("random cmap");
    let mut pm = pix11.to_mut();
    pm.paint_through_mask(&pix4, 0, 0, 0)
        .expect("paint through");
    let pix11: Pix = pm.into();
    rp.write_pix_and_check(&half(&pix11), ImageFormat::Png)
        .expect("check: stuff under mask 1");

    // C 11-12: paint the background white by resetting colormap entry 0.
    let mut pm = pix11.to_mut();
    pm.colormap_mut()
        .expect("colormapped")
        .reset_color(0, 255, 255, 255)
        .expect("reset entry 0");
    let pix11: Pix = pm.into();
    rp.write_pix_and_check(&pix11, ImageFormat::Png)
        .expect("check: background white");
    rp.write_pix_and_check(&half(&pix11), ImageFormat::Png)
        .expect("check: stuff under mask 2");

    let _ = Pixa::new();
    assert!(rp.cleanup(), "newspaper C-compat test failed");
}
