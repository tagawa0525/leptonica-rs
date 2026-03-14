//! Word boxes regression test
//!
//! Tests word detection by progressive dilation on document images.
//! The C version tests pixWordMaskByDilation, pixGetWordsInTextlines,
//! pixGetWordBoxesInTextlines, and pixFindWordAndCharacterBoxes on
//! multiple images at multiple scales.
//!
//! Partial port: Tests pix_word_mask_by_dilation and pix_word_boxes_by_dilation
//! on lucasta.150.jpg and other document images. The higher-level
//! word/character box functions (pixGetWordsInTextlines, etc.) and adjacency
//! finding are not available in the Rust API.
//!
//! # See also
//!
//! C Leptonica: `prog/wordboxes_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::io::ImageFormat;
use leptonica::recog::jbclass::{pix_word_boxes_by_dilation, pix_word_mask_by_dilation};
use leptonica::transform::scale_by_sampling;

/// Test word mask by dilation on lucasta.150.jpg at full scale (C test section 1).
///
/// C: pixWordMaskByDilation(pix1, &pix2, &size, NULL)
///    On lucasta.150.jpg binarized image.
#[test]
fn wordboxes_reg_lucasta_full() {
    let mut rp = RegParams::new("wordboxes_lucasta");

    let pix_orig = crate::common::load_test_image("lucasta.150.jpg").expect("load lucasta");
    // Convert to binary
    let pix_gray = pix_orig.convert_to_8().expect("convert to gray");
    let pix = threshold_to_binary(&pix_gray, 128).expect("threshold");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let w = pix.width();
    let h = pix.height();

    let (mask, dil_size) = pix_word_mask_by_dilation(&pix, 20).expect("word_mask lucasta");

    rp.compare_values(w as f64, mask.width() as f64, 0.0);
    rp.compare_values(h as f64, mask.height() as f64, 0.0);
    assert_eq!(mask.depth(), PixelDepth::Bit1);

    // Should find a reasonable dilation size
    rp.compare_values(
        1.0,
        if (1..=20).contains(&dil_size) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Word boxes
    let boxa = pix_word_boxes_by_dilation(&pix, 20).expect("word_boxes lucasta");
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "wordboxes lucasta full test failed");
}

/// Test word mask by dilation on lucasta.150.jpg at 0.6 scale (C test section 1 variant).
///
/// C: pixScale(pixs, 0.6, 0.6) then pixWordMaskByDilation
#[test]
fn wordboxes_reg_lucasta_scaled() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("wordboxes_lucasta_s");

    let pix_orig = crate::common::load_test_image("lucasta.150.jpg").expect("load lucasta");
    let pix_gray = pix_orig.convert_to_8().expect("convert to gray");
    let pix_bin = threshold_to_binary(&pix_gray, 128).expect("threshold");

    // Scale to 60%
    let pix = scale_by_sampling(&pix_bin, 0.6, 0.6).expect("scale 0.6");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let w = pix.width();
    let h = pix.height();

    let (mask, _) = pix_word_mask_by_dilation(&pix, 20).expect("word_mask scaled");
    rp.compare_values(w as f64, mask.width() as f64, 0.0);
    rp.compare_values(h as f64, mask.height() as f64, 0.0);

    let boxa = pix_word_boxes_by_dilation(&pix, 20).expect("word_boxes scaled");
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "wordboxes lucasta scaled test failed");
}

/// Test word detection on zanotti-78.jpg (C test section 2).
///
/// C: pixWordMaskByDilation on zanotti-78.jpg binarized, full and scaled.
#[test]
fn wordboxes_reg_zanotti() {
    let mut rp = RegParams::new("wordboxes_zanotti");

    let pix_orig = crate::common::load_test_image("zanotti-78.jpg").expect("load zanotti");
    let pix_gray = pix_orig.convert_to_8().expect("convert to gray");
    let pix = threshold_to_binary(&pix_gray, 128).expect("threshold");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    let (mask, _) = pix_word_mask_by_dilation(&pix, 20).expect("word_mask zanotti");
    rp.compare_values(pix.width() as f64, mask.width() as f64, 0.0);
    rp.compare_values(pix.height() as f64, mask.height() as f64, 0.0);

    let boxa = pix_word_boxes_by_dilation(&pix, 20).expect("word_boxes zanotti");
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "wordboxes zanotti test failed");
}

/// Test word detection on words.15.tif (C test section 3).
///
/// C: pixWordMaskByDilation on pre-binarized word images.
#[test]
fn wordboxes_reg_words15() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("wordboxes_w15");

    let pix = crate::common::load_test_image("words.15.tif").expect("load words.15.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let w = pix.width();
    let h = pix.height();

    let (mask, _) = pix_word_mask_by_dilation(&pix, 20).expect("word_mask words.15");
    rp.compare_values(w as f64, mask.width() as f64, 0.0);
    rp.compare_values(h as f64, mask.height() as f64, 0.0);

    rp.write_pix_and_check(&mask, ImageFormat::Tiff)
        .expect("write mask wordboxes_w15");

    let boxa = pix_word_boxes_by_dilation(&pix, 20).expect("word_boxes words.15");
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "wordboxes words15 test failed");
}

/// Test word detection on words.44.tif (C test section 4).
#[test]
fn wordboxes_reg_words44() {
    let mut rp = RegParams::new("wordboxes_w44");

    let pix = crate::common::load_test_image("words.44.tif").expect("load words.44.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let w = pix.width();
    let h = pix.height();

    let (mask, _) = pix_word_mask_by_dilation(&pix, 20).expect("word_mask words.44");
    rp.compare_values(w as f64, mask.width() as f64, 0.0);
    rp.compare_values(h as f64, mask.height() as f64, 0.0);

    rp.write_pix_and_check(&mask, ImageFormat::Tiff)
        .expect("write mask wordboxes_w44");

    let boxa = pix_word_boxes_by_dilation(&pix, 20).expect("word_boxes words.44");
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "wordboxes words44 test failed");
}

/// Test consistency: word_boxes results should be contained within word_mask.
///
/// Each bounding box from pix_word_boxes_by_dilation should correspond to
/// a connected component in the word mask.
#[test]
fn wordboxes_reg_mask_box_consistency() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("wordboxes_consist");

    let pix = crate::common::load_test_image("pageseg1.tif").expect("load pageseg1.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    let (mask, _) = pix_word_mask_by_dilation(&pix, 20).expect("word_mask");
    let boxa = pix_word_boxes_by_dilation(&pix, 20).expect("word_boxes");

    // Each box center should overlap with the mask foreground
    let mut boxes_in_mask = 0;
    for i in 0..boxa.len() {
        if let Some(b) = boxa.get(i) {
            let cx = b.x + b.w / 2;
            let cy = b.y + b.h / 2;
            if cx >= 0
                && cy >= 0
                && (cx as u32) < mask.width()
                && (cy as u32) < mask.height()
                && let Some(val) = mask.get_pixel(cx as u32, cy as u32)
                && val != 0
            {
                boxes_in_mask += 1;
            }
        }
    }

    // Most boxes should overlap with mask
    let ratio = if !boxa.is_empty() {
        boxes_in_mask as f64 / boxa.len() as f64
    } else {
        0.0
    };
    rp.compare_values(1.0, if ratio > 0.8 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "wordboxes mask_box consistency test failed");
}
