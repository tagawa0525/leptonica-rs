//! Tests for partify module
//!
//! Tests file/image partitioning functions.

use leptonica::io::partify;
use leptonica::{Pix, Pixa, PixelDepth};

/// Test partify_pixac with empty collection
#[test]
fn partify_empty() {
    let pixa = Pixa::new();
    let result = partify::partify_pixac(&pixa, 3, "test").unwrap();
    assert_eq!(result.len(), 3);
    for part in &result {
        assert_eq!(part.len(), 0);
    }
}

/// Test partify_pixac with single image, single part
#[test]
fn partify_single_image_single_part() {
    let mut pixa = Pixa::new();
    let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
    pixa.push(pix);

    let result = partify::partify_pixac(&pixa, 1, "test").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].len(), 1);
}

/// Test partify_pixac with multiple images, multiple parts
#[test]
fn partify_multiple_images() {
    let mut pixa = Pixa::new();
    for _ in 0..6 {
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        pixa.push(pix);
    }

    let result = partify::partify_pixac(&pixa, 3, "test").unwrap();
    assert_eq!(result.len(), 3);
    // Images should be distributed (round-robin since they're not binary/stave images)
    let total: usize = result.iter().map(|p| p.len()).sum();
    assert_eq!(total, 6);
}

/// Test partify_pixac error on n_parts < 1
#[test]
fn partify_invalid_parts() {
    let pixa = Pixa::new();
    assert!(partify::partify_pixac(&pixa, 0, "test").is_err());
}
