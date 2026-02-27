//! Classapp regression tests
//!
//! Tests for word/character box detection, sorted pattern extraction,
//! and image comparison by box patterns.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/classapp_reg.c` (partial)

use crate::common::RegParams;
use leptonica::core::{Box, Boxa, Numa, Numaa};
use leptonica::recog::classapp::{
    boxa_extract_sorted_pattern, find_word_and_character_boxes, numaa_compare_images_by_boxes,
};

/// Test `boxa_extract_sorted_pattern` with a simple two-row layout.
///
/// Constructs character boxes in 2 rows and verifies the extracted pattern
/// has the correct structure: y-center + (xl, xr) pairs per row.
#[test]
fn classapp_extract_sorted_pattern_basic() {
    let mut rp = RegParams::new("classapp_pattern");

    // Create boxes: 3 chars in row 0, 2 chars in row 1
    let mut boxa = Boxa::new();
    boxa.push(Box::new_unchecked(10, 100, 20, 30)); // row 0, char 0
    boxa.push(Box::new_unchecked(40, 100, 25, 30)); // row 0, char 1
    boxa.push(Box::new_unchecked(80, 100, 15, 30)); // row 0, char 2
    boxa.push(Box::new_unchecked(10, 200, 30, 40)); // row 1, char 0
    boxa.push(Box::new_unchecked(50, 200, 20, 40)); // row 1, char 1

    // na: row indices for each box
    let mut na = Numa::new();
    na.push(0.0);
    na.push(0.0);
    na.push(0.0);
    na.push(1.0);
    na.push(1.0);

    let naa = boxa_extract_sorted_pattern(&boxa, &na).expect("extract pattern");

    // Should have 2 rows
    rp.compare_values(2.0, naa.len() as f64, 0.0);

    // Row 0: y-center=115, then (10,29), (40,64), (80,94) => 7 values
    let row0 = naa.get(0).unwrap();
    rp.compare_values(7.0, row0.len() as f64, 0.0);
    rp.compare_values(115.0, row0.get(0).unwrap() as f64, 0.0); // y + h/2 = 100 + 15
    rp.compare_values(10.0, row0.get(1).unwrap() as f64, 0.0); // xl
    rp.compare_values(29.0, row0.get(2).unwrap() as f64, 0.0); // xr = 10+20-1

    // Row 1: y-center=220, then (10,39), (50,69) => 5 values
    let row1 = naa.get(1).unwrap();
    rp.compare_values(5.0, row1.len() as f64, 0.0);
    rp.compare_values(220.0, row1.get(0).unwrap() as f64, 0.0); // y + h/2 = 200 + 20

    assert!(rp.cleanup(), "classapp extract sorted pattern test failed");
}

/// Test `boxa_extract_sorted_pattern` with empty input.
#[test]
fn classapp_extract_sorted_pattern_empty() {
    let boxa = Boxa::new();
    let na = Numa::new();

    let naa = boxa_extract_sorted_pattern(&boxa, &na).expect("extract empty");
    assert_eq!(naa.len(), 0);
}

/// Test `numaa_compare_images_by_boxes` with identical patterns.
///
/// Two identical box patterns should be detected as "same".
#[test]
fn classapp_compare_identical_patterns() {
    let mut rp = RegParams::new("classapp_cmp_ident");

    // Create a pattern with 3 rows, each having 3 boxes
    let naa = build_test_pattern(&[
        (100, &[(10, 30), (40, 70), (80, 100)]),
        (200, &[(10, 30), (40, 70), (80, 100)]),
        (300, &[(10, 30), (40, 70), (80, 100)]),
    ]);

    let same =
        numaa_compare_images_by_boxes(&naa, &naa, 2, 2, 50, 50, 10, 10).expect("compare identical");
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "classapp compare identical test failed");
}

/// Test `numaa_compare_images_by_boxes` with shifted patterns.
///
/// Patterns shifted within tolerance should match.
#[test]
fn classapp_compare_shifted_patterns() {
    let mut rp = RegParams::new("classapp_cmp_shift");

    let naa1 = build_test_pattern(&[
        (100, &[(10, 30), (40, 70), (80, 100)]),
        (200, &[(10, 30), (40, 70), (80, 100)]),
        (300, &[(10, 30), (40, 70), (80, 100)]),
    ]);

    // Shift by (5, 3) — within tolerance
    let naa2 = build_test_pattern(&[
        (103, &[(15, 35), (45, 75), (85, 105)]),
        (203, &[(15, 35), (45, 75), (85, 105)]),
        (303, &[(15, 35), (45, 75), (85, 105)]),
    ]);

    let same =
        numaa_compare_images_by_boxes(&naa1, &naa2, 2, 2, 50, 50, 10, 10).expect("compare shift");
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "classapp compare shifted test failed");
}

/// Test `numaa_compare_images_by_boxes` with very different patterns.
///
/// Completely different patterns should not match.
#[test]
fn classapp_compare_different_patterns() {
    let mut rp = RegParams::new("classapp_cmp_diff");

    let naa1 = build_test_pattern(&[
        (100, &[(10, 30), (40, 70), (80, 100)]),
        (200, &[(10, 30), (40, 70), (80, 100)]),
    ]);

    let naa2 = build_test_pattern(&[
        (500, &[(200, 300), (400, 500), (600, 700)]),
        (700, &[(200, 300), (400, 500), (600, 700)]),
    ]);

    let same = numaa_compare_images_by_boxes(&naa1, &naa2, 2, 2, 50, 50, 10, 10)
        .expect("compare different");
    rp.compare_values(0.0, if same { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "classapp compare different test failed");
}

/// Test `numaa_compare_images_by_boxes` parameter validation.
#[test]
fn classapp_compare_invalid_params() {
    let naa = Numaa::new();
    assert!(numaa_compare_images_by_boxes(&naa, &naa, 0, 1, 50, 50, 10, 10).is_err());
    assert!(numaa_compare_images_by_boxes(&naa, &naa, 1, 0, 50, 50, 10, 10).is_err());
}

/// Test `numaa_compare_images_by_boxes` with insufficient rows.
#[test]
fn classapp_compare_insufficient_rows() {
    let naa1 = build_test_pattern(&[(100, &[(10, 30), (40, 70)])]);
    let naa2 = build_test_pattern(&[(100, &[(10, 30), (40, 70)]), (200, &[(10, 30), (40, 70)])]);

    // Require 2 matching rows, but naa1 only has 1
    let same = numaa_compare_images_by_boxes(&naa1, &naa2, 1, 2, 50, 50, 10, 10)
        .expect("compare insufficient");
    assert!(!same);
}

/// Test `find_word_and_character_boxes` on a document image.
///
/// Loads a real document image and verifies that word and character
/// boxes are found and have reasonable properties.
#[test]
fn classapp_find_word_char_boxes() {
    let mut rp = RegParams::new("classapp_wordchar");

    let pix = crate::common::load_test_image("lucasta.150.jpg").expect("load lucasta");

    let (boxaw, boxaac) =
        find_word_and_character_boxes(&pix, None, 140).expect("find word/char boxes");

    // Should find some words
    rp.compare_values(1.0, if !boxaw.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // Number of word boxes should equal number of char box arrays
    rp.compare_values(boxaw.len() as f64, boxaac.len() as f64, 0.0);

    // Each word should have at least one character
    let mut all_have_chars = true;
    for i in 0..boxaac.len() {
        if let Some(char_boxa) = boxaac.get(i)
            && char_boxa.is_empty()
        {
            all_have_chars = false;
            break;
        }
    }
    rp.compare_values(1.0, if all_have_chars { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "classapp find word/char boxes test failed");
}

/// Test `find_word_and_character_boxes` rejects 1bpp input.
#[test]
fn classapp_find_word_char_boxes_rejects_1bpp() {
    let pix =
        leptonica::core::Pix::new(100, 100, leptonica::core::PixelDepth::Bit1).expect("create");
    assert!(find_word_and_character_boxes(&pix, None, 128).is_err());
}

/// Helper: build a test Numaa pattern from row definitions.
///
/// Each row is (y_center, &[(xl, xr), ...]).
fn build_test_pattern(rows: &[(i32, &[(i32, i32)])]) -> Numaa {
    let mut naa = Numaa::new();
    for (y_center, boxes) in rows {
        let mut na = Numa::new();
        na.push(*y_center as f32);
        for (xl, xr) in *boxes {
            na.push(*xl as f32);
            na.push(*xr as f32);
        }
        naa.push(na);
    }
    naa
}
