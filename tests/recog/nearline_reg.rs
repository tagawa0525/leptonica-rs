//! Near-line regression test
//!
//! Covers textline word extraction and component profile splitting.
//!
//! # See also
//!
//! C Leptonica: `prog/nearline_reg.c`

use crate::common::RegParams;
use leptonica::recog::pageseg::{
    get_word_boxes_in_textlines, get_words_in_textlines, split_component_with_profile,
};
use leptonica::{Pix, PixelDepth};

fn make_textline_blocks() -> Pix {
    let pix = Pix::new(320, 140, PixelDepth::Bit1).expect("create textline image");
    let mut pm = pix.try_into_mut().expect("mutable textline image");

    // Three lines, three words per line.
    let lines = [18u32, 42u32, 90u32];
    for &y0 in &lines {
        for word in 0..3u32 {
            let x0 = 20 + word * 95;
            for y in y0..(y0 + 12) {
                for x in x0..(x0 + 50) {
                    pm.set_pixel_unchecked(x, y, 1);
                }
            }
        }
    }
    pm.into()
}

fn make_two_touching_blobs() -> Pix {
    let pix = Pix::new(90, 30, PixelDepth::Bit1).expect("create component image");
    let mut pm = pix.try_into_mut().expect("mutable component image");

    for y in 4..26u32 {
        for x in 8..34u32 {
            pm.set_pixel_unchecked(x, y, 1);
        }
        for x in 48..78u32 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    pm.into()
}

#[test]
fn nearline_reg() {
    let mut rp = RegParams::new("nearline");

    let pix = make_textline_blocks();
    let (boxes, images, nai) =
        get_words_in_textlines(&pix, 8, 6, 120, 40).expect("get_words_in_textlines");
    rp.compare_values(1.0, if !boxes.is_empty() { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(boxes.len() as f64, nai.len() as f64, 0.0);
    rp.compare_values(
        1.0,
        if images.as_ref().map(|v| v.len()) == Some(boxes.len()) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let (boxes2, nai2) =
        get_word_boxes_in_textlines(&pix, 8, 6, 120, 40).expect("get_word_boxes_in_textlines");
    rp.compare_values(boxes.len() as f64, boxes2.len() as f64, 0.0);
    rp.compare_values(nai.len() as f64, nai2.len() as f64, 0.0);

    let comp = make_two_touching_blobs();
    let split_boxes = split_component_with_profile(&comp, 3, 3).expect("split component");
    rp.compare_values(1.0, if split_boxes.len() >= 2 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "nearline regression test failed");
}
