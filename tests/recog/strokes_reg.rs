//! Tests for strokes module functions
//!
//! Tests stroke length measurement, stroke width detection,
//! and stroke width modification.

use leptonica::recog::strokes;
use leptonica::{Pix, Pixa, PixelDepth};

/// Test find_stroke_length on a simple rectangle
#[test]
fn strokes_find_stroke_length() {
    let pix = Pix::new(20, 10, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Draw a filled rectangle 10x5
    for y in 2..7 {
        for x in 5..15 {
            pix_mut.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pix_mut.into();

    let length = strokes::find_stroke_length(&pix).unwrap();
    // Should be approximately half the boundary pixels
    assert!(length > 0, "stroke length should be > 0");
}

/// Test find_stroke_width on a simple stroke
#[test]
fn strokes_find_stroke_width() {
    let pix = Pix::new(30, 30, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Draw a horizontal bar of width 4
    for y in 13..17 {
        for x in 5..25 {
            pix_mut.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pix_mut.into();

    let (width, _histo) = strokes::find_stroke_width(&pix, 0.15).unwrap();
    // Width should be roughly 4
    assert!(width > 2.0 && width < 8.0, "width = {width}, expected ~4");
}

/// Test pixa_find_stroke_width with multiple images
#[test]
fn strokes_pixa_find_stroke_width() {
    let mut pixa = Pixa::new();

    // Image 1: thin stroke (width ~2)
    let pix1 = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let mut pm1 = pix1.try_into_mut().unwrap();
    for y in 9..11 {
        for x in 3..17 {
            pm1.set_pixel(x, y, 1).unwrap();
        }
    }
    pixa.push(pm1.into());

    // Image 2: thicker stroke (width ~4)
    let pix2 = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let mut pm2 = pix2.try_into_mut().unwrap();
    for y in 8..12 {
        for x in 3..17 {
            pm2.set_pixel(x, y, 1).unwrap();
        }
    }
    pixa.push(pm2.into());

    let na = strokes::pixa_find_stroke_width(&pixa, 0.15).unwrap();
    assert_eq!(na.len(), 2);
    let w1 = na.get(0).unwrap();
    let w2 = na.get(1).unwrap();
    assert!(
        w2 > w1,
        "thicker stroke ({w2}) should have larger width than thinner ({w1})"
    );
}

/// Test modify_stroke_width (no change when diff=0)
#[test]
fn strokes_modify_stroke_width_no_change() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 8..12 {
        for x in 5..15 {
            pix_mut.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pix_mut.into();

    // Target = current width → should return copy
    let result = strokes::modify_stroke_width(&pix, 4.0, 4.0).unwrap();
    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
}

/// Test set_stroke_width with thinning
#[test]
fn strokes_set_stroke_width() {
    let pix = Pix::new(30, 30, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Draw a cross pattern
    for x in 5..25 {
        pix_mut.set_pixel(x, 15, 1).unwrap();
    }
    for y in 5..25 {
        pix_mut.set_pixel(15, y, 1).unwrap();
    }
    let pix: Pix = pix_mut.into();

    let result =
        strokes::set_stroke_width(&pix, 3, true, leptonica::morph::Connectivity::Eight).unwrap();
    assert!(
        result.count_pixels() > 0,
        "result should have foreground pixels"
    );
}

/// Test pixa_set_stroke_width
#[test]
fn strokes_pixa_set_stroke_width() {
    let mut pixa = Pixa::new();
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 9..11 {
        for x in 3..17 {
            pix_mut.set_pixel(x, y, 1).unwrap();
        }
    }
    pixa.push(pix_mut.into());

    let result =
        strokes::pixa_set_stroke_width(&pixa, 3, true, leptonica::morph::Connectivity::Eight)
            .unwrap();
    assert_eq!(result.len(), 1);
}

/// Test pixa_modify_stroke_width
#[test]
fn strokes_pixa_modify_stroke_width() {
    let mut pixa = Pixa::new();
    let pix = Pix::new(30, 30, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 10..16 {
        for x in 5..25 {
            pix_mut.set_pixel(x, y, 1).unwrap();
        }
    }
    pixa.push(pix_mut.into());

    let result = strokes::pixa_modify_stroke_width(&pixa, 3.0).unwrap();
    assert_eq!(result.len(), 1);
}
