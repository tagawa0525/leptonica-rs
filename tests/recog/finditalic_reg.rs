//! Tests for finditalic module
//!
//! Tests italic word detection functions.

use leptonica::recog::finditalic;
use leptonica::{Pix, PixelDepth};

/// Test italic_words on empty image
#[test]
fn finditalic_empty_image() {
    let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
    let result = finditalic::italic_words(&pix, None, None).unwrap();
    assert_eq!(result.len(), 0, "empty image should have no italic words");
}

/// Test italic_words error on wrong depth
#[test]
fn finditalic_wrong_depth() {
    let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    assert!(finditalic::italic_words(&pix, None, None).is_err());
}

/// Test italic_words error when both boxaw and pixw given
#[test]
fn finditalic_both_inputs_error() {
    let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
    let boxa = leptonica::core::Boxa::new();
    let pixw = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
    assert!(finditalic::italic_words(&pix, Some(&boxa), Some(&pixw)).is_err());
}

/// Test italic_words with a word mask
#[test]
fn finditalic_with_word_mask() {
    let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
    let mask = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
    let result = finditalic::italic_words(&pix, None, Some(&mask)).unwrap();
    assert_eq!(result.len(), 0, "no italic content in empty images");
}
