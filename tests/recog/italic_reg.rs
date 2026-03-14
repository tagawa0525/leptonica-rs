//! Italic text detection regression test
//!
//! Tests detection of italic (slanted) text in binary images. The C version
//! uses pixItalicWords to find italic text regions, with intermediate steps
//! using word mask dilation and morphological sequences.
//!
//! Partial port: Tests word mask by dilation, morphological sequence
//! processing, and connected component analysis on italic text. The
//! high-level pixItalicWords function is not available in the Rust API.
//!
//! # See also
//!
//! C Leptonica: `prog/italic_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::io::ImageFormat;
use leptonica::morph::morph_sequence;
use leptonica::recog::jbclass::pix_word_mask_by_dilation;
use leptonica::region::{ConnectivityType, conncomp_pixa};

/// Test word mask by dilation on italic text (C test: pixWordMaskByDilation).
///
/// C: pixWordMaskByDilation(pixb, &pixm, &size, NULL)
///    Creates a mask where words are connected blobs.
#[test]
fn italic_reg_word_mask() {
    let mut rp = RegParams::new("italic_wordmask");

    let pix = crate::common::load_test_image("italic.png").expect("load italic.png");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("convert to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };
    assert_eq!(pix_bin.depth(), PixelDepth::Bit1);

    let (mask, dil_size) = pix_word_mask_by_dilation(&pix_bin, 20).expect("word_mask italic");

    rp.compare_values(pix_bin.width() as f64, mask.width() as f64, 0.0);
    rp.compare_values(pix_bin.height() as f64, mask.height() as f64, 0.0);
    assert_eq!(mask.depth(), PixelDepth::Bit1);

    // Dilation size should be reasonable
    rp.compare_values(
        1.0,
        if (1..=20).contains(&dil_size) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    rp.write_pix_and_check(&mask, ImageFormat::Tiff)
        .expect("write mask italic_wordmask");

    assert!(rp.cleanup(), "italic word_mask test failed");
}

/// Test morphological sequence on italic text.
///
/// C: pixMorphSequence(pixm, "d1.5 + c15.1", 0)
///    Dilate vertically then close horizontally to connect italic words.
#[test]
fn italic_reg_morph_sequence() {
    let mut rp = RegParams::new("italic_morph");

    let pix = crate::common::load_test_image("italic.png").expect("load italic.png");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("convert to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    let (mask, _) = pix_word_mask_by_dilation(&pix_bin, 20).expect("word_mask");

    // Apply morphological sequence: dilate vertically, then close horizontally
    let processed = morph_sequence(&mask, "d1.5 + c15.1").expect("morph_sequence");

    rp.compare_values(mask.width() as f64, processed.width() as f64, 0.0);
    rp.compare_values(mask.height() as f64, processed.height() as f64, 0.0);
    assert_eq!(processed.depth(), PixelDepth::Bit1);

    rp.write_pix_and_check(&processed, ImageFormat::Tiff)
        .expect("write processed italic_morph");

    assert!(rp.cleanup(), "italic morph_sequence test failed");
}

/// Test connected component extraction on italic text.
///
/// C: pixConnComp(pixm, &pixa, 8)
///    Extract word-level connected components from the word mask.
#[test]
fn italic_reg_conncomp() {
    let mut rp = RegParams::new("italic_conncomp");

    let pix = crate::common::load_test_image("italic.png").expect("load italic.png");
    let pix_bin = if pix.depth() == PixelDepth::Bit1 {
        pix
    } else {
        let gray = pix.convert_to_8().expect("convert to gray");
        threshold_to_binary(&gray, 128).expect("threshold")
    };

    let (mask, _) = pix_word_mask_by_dilation(&pix_bin, 20).expect("word_mask");

    // Extract connected components (word regions)
    let (boxa, pixa) = conncomp_pixa(&mask, ConnectivityType::EightWay).expect("conncomp_pixa");

    // Should find word regions in the italic text
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(boxa.len() as f64, pixa.len() as f64, 0.0);

    // All boxes should have valid dimensions
    let all_valid = (0..boxa.len()).all(|i| {
        if let Some(b) = boxa.get(i) {
            b.w > 0 && b.h > 0
        } else {
            false
        }
    });
    rp.compare_values(1.0, if all_valid { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "italic conncomp test failed");
}
