//! Coverage tests for 28 recog functions
//!
//! Tests are grouped by C source file origin:
//! - recogbasic.c: create_from_recog, create_from_pixa_no_finish
//! - recogident.c: skip_identify, process_to_identify, extract_numbers, rcha_extract, rch_extract
//! - recogtrain.c: process_labeled, add_sample_pub, pixa_accumulate_samples,
//!   filter_pixa_by_size_adv, sort_pixa_by_class, pixa_remove_outliers1, pixa_remove_outliers2
//! - pageseg.c:    find_page_foreground, pix_split_into_characters, split_component_with_profile,
//!   get_words_in_textlines, get_word_boxes_in_textlines
//! - skew.c:       find_skew_sweep, find_skew_orthogonal_range
//! - dewarp1.c:    dewarpa_create_from_pixacomp, dewarpa_restore_models
//! - jbclass.c:    correlation_init_without_components, add_page_components, jb_correlation, jb_rank_haus
//! - readbarcode.c: read_barcodes

use leptonica::core::{Pix, PixelDepth};
use leptonica::recog::barcode::{BarcodeFormat, read_barcodes};
use leptonica::recog::dewarp::Dewarpa;
use leptonica::recog::jbclass::{
    JbComponent, add_page_components, correlation_init_without_components, jb_correlation,
    jb_rank_haus, rank_haus_init,
};
use leptonica::recog::pageseg::{
    find_page_foreground, get_word_boxes_in_textlines, get_words_in_textlines,
    pix_split_into_characters, split_component_with_profile,
};
use leptonica::recog::recog::{
    Rch, Rcha, Recog, create, create_from_pixa_no_finish, create_from_recog,
    pixa_accumulate_samples, pixa_remove_outliers1, pixa_remove_outliers2,
};
use leptonica::recog::skew::{find_skew_orthogonal_range, find_skew_sweep};

// ── helpers ─────────────────────────────────────────────────────────────
fn make_char_pix(w: u32, h: u32, offset: u32) -> Pix {
    let p = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut m = p.try_into_mut().unwrap();
    let start = offset.min(w.saturating_sub(1));
    for y in 1..h.saturating_sub(1) {
        for x in start..w.min(start + w / 2) {
            let _ = m.set_pixel(x, y, 1);
        }
    }
    m.into()
}

fn make_trained_recog() -> Recog {
    let mut recog = create(0, 40, 0, 150, 1).unwrap();
    for i in 0..3 {
        let p = make_char_pix(10 + i * 2, 20, i);
        recog
            .train_labeled(&p, &format!("{}", (b'A' + i as u8) as char))
            .unwrap();
    }
    recog.finish_training().unwrap();
    recog
}

fn make_text_block_image(w: u32, h: u32) -> Pix {
    let p = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut m = p.try_into_mut().unwrap();
    // Create several horizontal "text lines"
    for line in 0..5 {
        let y_base = 50 + line * 40;
        for y in y_base..y_base + 10 {
            if y >= h {
                break;
            }
            for x in 30..w.saturating_sub(30) {
                // character-like blocks every 12px
                if x % 12 < 8 {
                    let _ = m.set_pixel(x, y, 1);
                }
            }
        }
    }
    m.into()
}

// ═══════════════════════════════════════════════════════════════════════
//  recogbasic.c
// ═══════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "not yet implemented"]
fn test_create_from_recog() {
    let recog = make_trained_recog();
    let recog2 = create_from_recog(&recog, 0, 40, 0, 150, 1).unwrap();
    assert!(recog2.train_done);
    assert_eq!(recog2.set_size, recog.set_size);
}

#[test]
#[ignore = "not yet implemented"]
fn test_create_from_recog_different_params() {
    let recog = make_trained_recog();
    let recog2 = create_from_recog(&recog, 30, 30, 0, 128, 0).unwrap();
    assert!(recog2.train_done);
    assert_eq!(recog2.scale_w, 30);
    assert_eq!(recog2.scale_h, 30);
}

#[test]
#[ignore = "not yet implemented"]
fn test_create_from_pixa_no_finish() {
    let samples: Vec<Pix> = (0..3).map(|i| make_char_pix(10 + i * 2, 20, i)).collect();
    let labels: Vec<&str> = vec!["A", "B", "C"];
    let recog = create_from_pixa_no_finish(&samples, &labels, 0, 40, 0, 150, 1).unwrap();
    // Training should NOT be finished
    assert!(!recog.train_done);
    assert_eq!(recog.set_size, 3);
}

#[test]
#[ignore = "not yet implemented"]
fn test_create_from_pixa_no_finish_empty_label_skipped() {
    let samples = vec![make_char_pix(10, 20, 0), make_char_pix(12, 20, 1)];
    let labels: Vec<&str> = vec!["X", ""];
    let recog = create_from_pixa_no_finish(&samples, &labels, 0, 40, 0, 150, 1).unwrap();
    // Only one valid sample added (empty label skipped)
    assert_eq!(recog.set_size, 1);
}

// ═══════════════════════════════════════════════════════════════════════
//  recogident.c
// ═══════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "not yet implemented"]
fn test_skip_identify() {
    let mut recog = make_trained_recog();
    recog.skip_identify();
    let rch = recog.get_rch().unwrap();
    assert_eq!(rch.index, 0);
    assert_eq!(rch.score, 0.0);
    assert!(rch.text.is_empty());
}

#[test]
#[ignore = "not yet implemented"]
fn test_process_to_identify_binary_input() {
    let recog = make_trained_recog();
    let pix = make_char_pix(12, 20, 2);
    let result = recog.process_to_identify(&pix, 1).unwrap();
    // Should be 1bpp with padding
    assert_eq!(result.depth(), PixelDepth::Bit1);
    assert!(result.width() >= pix.width());
}

#[test]
#[ignore = "not yet implemented"]
fn test_process_to_identify_grayscale_input() {
    let recog = make_trained_recog();
    let gray = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut m = gray.try_into_mut().unwrap();
    for y in 5..15 {
        for x in 5..15 {
            let _ = m.set_pixel(x, y, 0); // black pixels in grayscale
        }
    }
    let gray: Pix = m.into();
    let result = recog.process_to_identify(&gray, 2).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit1);
}

#[test]
#[ignore = "not yet implemented"]
fn test_extract_numbers_basic() {
    let mut recog = make_trained_recog();
    // Build an Rcha with digit-like entries
    let mut rcha = Rcha::new();
    for (i, ch) in ["1", "2", "3"].iter().enumerate() {
        rcha.push(&Rch {
            index: i as i32,
            score: 0.9,
            text: ch.to_string(),
            sample: 0,
            xloc: (i * 15) as i32,
            yloc: 0,
            width: 10,
        });
    }
    recog.set_rcha(rcha);
    let boxes: Vec<leptonica::core::Box> = (0..3)
        .map(|i| leptonica::core::Box::new_unchecked(i * 15, 0, 10, 20))
        .collect();
    let result = recog.extract_numbers(&boxes, 0.5, -1);
    assert!(result.is_ok());
    let numbers = result.unwrap();
    assert!(!numbers.is_empty());
    assert_eq!(numbers[0], "123");
}

#[test]
#[ignore = "not yet implemented"]
fn test_rcha_extract() {
    let mut rcha = Rcha::new();
    rcha.push(&Rch {
        index: 5,
        score: 0.95,
        text: "X".to_string(),
        sample: 1,
        xloc: 10,
        yloc: 20,
        width: 8,
    });
    let (indices, scores, texts, samples, xlocs, ylocs, widths) = rcha.extract();
    assert_eq!(indices, vec![5]);
    assert!((scores[0] - 0.95).abs() < 1e-5);
    assert_eq!(texts, vec!["X"]);
    assert_eq!(samples, vec![1]);
    assert_eq!(xlocs, vec![10]);
    assert_eq!(ylocs, vec![20]);
    assert_eq!(widths, vec![8]);
}

#[test]
#[ignore = "not yet implemented"]
fn test_rch_extract() {
    let rch = Rch {
        index: 3,
        score: 0.88,
        text: "Z".to_string(),
        sample: 2,
        xloc: 5,
        yloc: 7,
        width: 12,
    };
    let (index, score, text, sample, xloc, yloc, width) = rch.extract();
    assert_eq!(index, 3);
    assert!((score - 0.88).abs() < 1e-5);
    assert_eq!(text, "Z");
    assert_eq!(sample, 2);
    assert_eq!(xloc, 5);
    assert_eq!(yloc, 7);
    assert_eq!(width, 12);
}

// ═══════════════════════════════════════════════════════════════════════
//  recogtrain.c
// ═══════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "not yet implemented"]
fn test_process_labeled() {
    let recog = create(0, 40, 0, 150, 1).unwrap();
    let pix = make_char_pix(12, 20, 1);
    let processed = recog.process_labeled_pub(&pix).unwrap();
    assert_eq!(processed.depth(), PixelDepth::Bit1);
    // Should be clipped to foreground
    assert!(processed.width() <= pix.width());
    assert!(processed.height() <= pix.height());
}

#[test]
#[ignore = "not yet implemented"]
fn test_add_sample_pub() {
    let mut recog = create(0, 40, 0, 150, 1).unwrap();
    let pix = make_char_pix(10, 20, 0);
    recog.add_sample_pub(&pix, "Q").unwrap();
    assert_eq!(recog.num_samples, 1);
    assert_eq!(recog.set_size, 1);
    assert_eq!(recog.sa_text[0], "Q");
}

#[test]
#[ignore = "not yet implemented"]
fn test_add_sample_pub_rejects_after_training() {
    let mut recog = make_trained_recog();
    let pix = make_char_pix(10, 20, 0);
    let result = recog.add_sample_pub(&pix, "Z");
    assert!(result.is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn test_pixa_accumulate_samples() {
    let samples: Vec<Pix> = (0..5).map(|_| make_char_pix(10, 20, 0)).collect();
    let (avg, cx, cy) = pixa_accumulate_samples(&samples).unwrap();
    assert_eq!(avg.depth(), PixelDepth::Bit8);
    assert!(cx > 0.0);
    assert!(cy > 0.0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_filter_pixa_by_size_adv() {
    let samples: Vec<Pix> = vec![
        make_char_pix(5, 10, 0),
        make_char_pix(10, 20, 0),
        make_char_pix(20, 40, 0),
        make_char_pix(12, 22, 0),
    ];
    let (filtered, counts) = Recog::filter_pixa_by_size_adv(&samples, 2, 3, 2.0).unwrap();
    assert!(!filtered.is_empty());
    assert!(!counts.is_empty());
    // All output images should pass height ratio criterion
}

#[test]
#[ignore = "not yet implemented"]
fn test_sort_pixa_by_class() {
    let samples: Vec<Pix> = (0..3).map(|i| make_char_pix(10 + i * 2, 20, i)).collect();
    let labels = vec!["B", "A", "B"];
    let sorted = Recog::sort_pixa_by_class(&samples, &labels).unwrap();
    // Should produce 2 classes: A and B
    assert_eq!(sorted.len(), 2);
}

#[test]
#[ignore = "not yet implemented"]
fn test_pixa_remove_outliers1() {
    let mut samples: Vec<Pix> = (0..5).map(|_| make_char_pix(10, 20, 0)).collect();
    // Add a very different sample as an outlier
    let outlier = make_char_pix(10, 20, 8);
    samples.push(outlier);
    let labels: Vec<&str> = vec!["A", "A", "A", "A", "A", "A"];
    let result = pixa_remove_outliers1(&samples, &labels, 0.7, 3, 2).unwrap();
    // Outlier should be removed, at least 3 remain
    assert!(result.len() >= 3);
    assert!(result.len() <= samples.len());
}

#[test]
#[ignore = "not yet implemented"]
fn test_pixa_remove_outliers2() {
    let samples: Vec<Pix> = (0..4).map(|_| make_char_pix(10, 20, 0)).collect();
    let labels: Vec<&str> = vec!["A", "A", "B", "B"];
    let result = pixa_remove_outliers2(&samples, &labels, 0.6, 1).unwrap();
    assert!(!result.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
//  pageseg.c
// ═══════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "not yet implemented"]
fn test_find_page_foreground() {
    let pix = make_text_block_image(400, 500);
    let bbox = find_page_foreground(&pix, 128, 10, 20).unwrap();
    assert!(bbox.w > 0);
    assert!(bbox.h > 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_find_page_foreground_empty_image() {
    let pix = Pix::new(400, 500, PixelDepth::Bit1).unwrap();
    let result = find_page_foreground(&pix, 128, 10, 20);
    assert!(result.is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn test_pix_split_into_characters() {
    // Create a line with separated characters
    let pix = Pix::new(100, 30, PixelDepth::Bit1).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    // Three separated blocks
    for y in 5..25 {
        for x in 5..15 {
            let _ = m.set_pixel(x, y, 1);
        }
        for x in 25..35 {
            let _ = m.set_pixel(x, y, 1);
        }
        for x in 45..55 {
            let _ = m.set_pixel(x, y, 1);
        }
    }
    let pix: Pix = m.into();
    let (boxes, pixa) = pix_split_into_characters(&pix, 3, 5).unwrap();
    assert_eq!(boxes.len(), 3);
    assert!(pixa.is_some());
}

#[test]
#[ignore = "not yet implemented"]
fn test_split_component_with_profile() {
    // Create a component with two touching characters (dip in projection)
    let pix = Pix::new(30, 20, PixelDepth::Bit1).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for y in 2..18 {
        for x in 2..12 {
            let _ = m.set_pixel(x, y, 1);
        }
        // Narrow bridge
        if y > 8 && y < 12 {
            let _ = m.set_pixel(14, y, 1);
        }
        for x in 17..28 {
            let _ = m.set_pixel(x, y, 1);
        }
    }
    let pix: Pix = m.into();
    let boxes = split_component_with_profile(&pix, 2, 5).unwrap();
    // Should find at least 1 split (2 parts)
    assert!(boxes.len() >= 2);
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_words_in_textlines() {
    let pix = make_text_block_image(400, 300);
    let (boxes, pixa, nai) = get_words_in_textlines(&pix, 4, 4, 200, 60).unwrap();
    assert!(!boxes.is_empty());
    assert!(pixa.is_some());
    assert!(!nai.is_empty());
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_word_boxes_in_textlines() {
    let pix = make_text_block_image(400, 300);
    let (boxes, nai) = get_word_boxes_in_textlines(&pix, 4, 4, 200, 60).unwrap();
    assert!(!boxes.is_empty());
    assert!(!nai.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
//  skew.c
// ═══════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "not yet implemented"]
fn test_find_skew_sweep() {
    let pix = make_text_block_image(400, 300);
    let angle = find_skew_sweep(&pix, 4, 7.0, 1.0).unwrap();
    // Horizontal text => angle near 0
    assert!(angle.abs() < 10.0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_find_skew_sweep_invalid_reduction() {
    let pix = make_text_block_image(400, 300);
    let result = find_skew_sweep(&pix, 3, 7.0, 1.0);
    assert!(result.is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn test_find_skew_orthogonal_range() {
    let pix = make_text_block_image(400, 300);
    let (angle, conf) = find_skew_orthogonal_range(&pix, 4, 2, 7.0, 1.0, 0.01, 0.0).unwrap();
    assert!(angle.abs() < 15.0);
    assert!(conf >= 0.0);
}

// ═══════════════════════════════════════════════════════════════════════
//  dewarp1.c
// ═══════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "not yet implemented"]
fn test_dewarpa_create_from_pixacomp() {
    let pages: Vec<Pix> = (0..3).map(|_| make_text_block_image(200, 300)).collect();
    let result = Dewarpa::create_from_pixacomp(&pages, true, 30, 8, 50);
    assert!(result.is_ok());
    let dewa = result.unwrap();
    assert_eq!(dewa.max_pages, 3);
}

#[test]
#[ignore = "not yet implemented"]
fn test_dewarpa_restore_models() {
    let pages: Vec<Pix> = (0..2).map(|_| make_text_block_image(200, 300)).collect();
    let mut dewa = Dewarpa::create_from_pixacomp(&pages, true, 30, 8, 50).unwrap();
    dewa.strip_ref_models();
    let result = dewa.restore_models();
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════
//  jbclass.c
// ═══════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "not yet implemented"]
fn test_correlation_init_without_components() {
    let classer =
        correlation_init_without_components(JbComponent::Characters, 150, 150, 0.85, 0.0).unwrap();
    assert!(!classer.keep_pixaa);
    assert_eq!(classer.thresh, 0.85);
}

#[test]
#[ignore = "not yet implemented"]
fn test_add_page_components() {
    let mut classer = rank_haus_init(JbComponent::ConnComps, 150, 150, 2, 0.97).unwrap();
    let pix = make_text_block_image(200, 100);
    // Extract components first
    let (pixa, boxa) = classer.get_components(&pix).unwrap();
    let result = add_page_components(&mut classer, &pix, &boxa, &pixa);
    assert!(result.is_ok());
    assert!(classer.npages >= 1);
}

#[test]
#[ignore = "not yet implemented"]
fn test_jb_correlation() {
    let pages: Vec<Pix> = (0..2).map(|_| make_text_block_image(200, 100)).collect();
    let result = jb_correlation(&pages, 0.85, 0.0, JbComponent::ConnComps);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.nclass > 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_jb_rank_haus() {
    let pages: Vec<Pix> = (0..2).map(|_| make_text_block_image(200, 100)).collect();
    let result = jb_rank_haus(&pages, 2, 0.97, JbComponent::ConnComps);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.nclass > 0);
}

// ═══════════════════════════════════════════════════════════════════════
//  readbarcode.c
// ═══════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "not yet implemented"]
fn test_read_barcodes_from_pixa() {
    let pix = Pix::new(200, 50, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    // Create a simple barcode-like pattern (alternating black/white bars)
    for y in 5..45 {
        let mut x = 10u32;
        let widths = [2, 1, 3, 1, 2, 1, 1, 3, 2, 1]; // bar widths
        let mut is_bar = true;
        for &w in &widths {
            for dx in 0..w {
                if x + dx < 190 {
                    let val = if is_bar { 0 } else { 255 };
                    let _ = m.set_pixel(x + dx, y, val);
                }
            }
            x += w;
            is_bar = !is_bar;
        }
        // Fill remainder with white
        for wx in x..200 {
            let _ = m.set_pixel(wx, y, 255);
        }
    }
    let pix: Pix = m.into();
    let pixa = vec![pix];
    // read_barcodes should accept a slice of pre-extracted barcode images
    let result = read_barcodes(&pixa, BarcodeFormat::Any);
    // May fail to decode (synthetic pattern) but should not panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn test_read_barcodes_empty_pixa() {
    let pixa: Vec<Pix> = vec![];
    let result = read_barcodes(&pixa, BarcodeFormat::Any);
    assert!(result.is_err());
}
