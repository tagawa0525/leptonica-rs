//! JBIG2 classification regression test
//!
//! Tests connected component classification using both rank Hausdorff
//! distance and correlation-based methods. The C version processes
//! pageseg1.tif and pageseg4.tif (top halves), classifies components,
//! and renders reconstructed pages from templates.
//!
//! Partial port: Tests rank_haus_init, correlation_init, add_page,
//! get_data, and render_page. Auto-generated template display and
//! PDF output are not tested.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/jbclass_reg.c`

use leptonica_core::PixelDepth;
use leptonica_recog::jbclass::{
    JbComponent, correlation_init, pix_word_boxes_by_dilation, pix_word_mask_by_dilation,
    rank_haus_init,
};
use leptonica_test::RegParams;

/// Test rank_haus_init classification on pageseg images (C checks 0-3).
///
/// C: classer = jbRankHausInit(JB_CONN_COMPS, 0, 0, 2, 0.97)
///    jbAddPages(classer, safiles)
///    data = jbDataSave(classer)
///    pixr = jbDataRender(data, 0)  -- render page 0
///    pixr = jbDataRender(data, 1)  -- render page 1
#[test]
#[ignore = "not yet implemented"]
fn jbclass_reg_rank_haus() {
    let mut rp = RegParams::new("jbclass_haus");

    let pix1 = leptonica_test::load_test_image("pageseg1.tif").expect("load pageseg1.tif");
    let pix4 = leptonica_test::load_test_image("pageseg4.tif").expect("load pageseg4.tif");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);
    assert_eq!(pix4.depth(), PixelDepth::Bit1);

    // C version clips to top half; we use the full image for simplicity
    let mut classer =
        rank_haus_init(JbComponent::ConnComps, 0, 0, 2, 0.97).expect("rank_haus_init");

    classer.add_page(&pix1).expect("add_page pageseg1");
    classer.add_page(&pix4).expect("add_page pageseg4");

    // Should have classified some components
    let n_comps = classer.total_components();
    rp.compare_values(1.0, if n_comps > 0 { 1.0 } else { 0.0 }, 0.0);

    let n_classes = classer.num_classes();
    rp.compare_values(1.0, if n_classes > 0 { 1.0 } else { 0.0 }, 0.0);

    // Classes should be fewer than components (grouping occurred)
    rp.compare_values(1.0, if n_classes <= n_comps { 1.0 } else { 0.0 }, 0.0);

    // Get compressed data and render pages
    let data = classer.get_data().expect("get_data");

    let rendered0 = data.render_page(0).expect("render page 0");
    rp.compare_values(
        1.0,
        if rendered0.width() > 0 && rendered0.height() > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let rendered1 = data.render_page(1).expect("render page 1");
    rp.compare_values(
        1.0,
        if rendered1.width() > 0 && rendered1.height() > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "jbclass rank_haus test failed");
}

/// Test correlation_init classification on pageseg images (C checks 4-7).
///
/// C: classer = jbCorrelationInit(JB_CONN_COMPS, 0, 0, 0.8, 0.6)
///    jbAddPages(classer, safiles)
///    data = jbDataSave(classer)
///    pixr = jbDataRender(data, 0)
#[test]
#[ignore = "not yet implemented"]
fn jbclass_reg_correlation() {
    let mut rp = RegParams::new("jbclass_corr");

    let pix1 = leptonica_test::load_test_image("pageseg1.tif").expect("load pageseg1.tif");
    let pix4 = leptonica_test::load_test_image("pageseg4.tif").expect("load pageseg4.tif");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);
    assert_eq!(pix4.depth(), PixelDepth::Bit1);

    let mut classer =
        correlation_init(JbComponent::ConnComps, 0, 0, 0.8, 0.6).expect("correlation_init");

    classer.add_page(&pix1).expect("add_page pageseg1");
    classer.add_page(&pix4).expect("add_page pageseg4");

    let n_comps = classer.total_components();
    rp.compare_values(1.0, if n_comps > 0 { 1.0 } else { 0.0 }, 0.0);

    let n_classes = classer.num_classes();
    rp.compare_values(1.0, if n_classes > 0 { 1.0 } else { 0.0 }, 0.0);

    // Correlation typically produces fewer classes than rank Hausdorff
    // (tighter matching), but we just verify basic grouping occurred
    rp.compare_values(1.0, if n_classes <= n_comps { 1.0 } else { 0.0 }, 0.0);

    let data = classer.get_data().expect("get_data");
    let rendered = data.render_page(0).expect("render page 0");
    rp.compare_values(
        1.0,
        if rendered.width() > 0 && rendered.height() > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "jbclass correlation test failed");
}

/// Test pix_word_mask_by_dilation on a document image.
///
/// C: pixWordMaskByDilation(pixs, &pix1, &size, pixadb)
///    Returns a mask where words are connected blobs.
#[test]
#[ignore = "not yet implemented"]
fn jbclass_reg_word_mask() {
    let mut rp = RegParams::new("jbclass_wordmask");

    let pix = leptonica_test::load_test_image("pageseg1.tif").expect("load pageseg1.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    let (mask, dil_size) = pix_word_mask_by_dilation(&pix, 20).expect("word_mask_by_dilation");

    // Mask should have same dimensions as input
    rp.compare_values(pix.width() as f64, mask.width() as f64, 0.0);
    rp.compare_values(pix.height() as f64, mask.height() as f64, 0.0);
    assert_eq!(mask.depth(), PixelDepth::Bit1);

    // Dilation size should be reasonable (1-20)
    rp.compare_values(
        1.0,
        if dil_size >= 1 && dil_size <= 20 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "jbclass word_mask test failed");
}

/// Test pix_word_boxes_by_dilation on a document image.
///
/// C: pixWordBoxesByDilation(pixs, 18, 18, 0, 5, NULL, &boxa)
///    Returns bounding boxes of word regions.
#[test]
#[ignore = "not yet implemented"]
fn jbclass_reg_word_boxes() {
    let mut rp = RegParams::new("jbclass_wordboxes");

    let pix = leptonica_test::load_test_image("pageseg1.tif").expect("load pageseg1.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    let boxa = pix_word_boxes_by_dilation(&pix, 20).expect("word_boxes_by_dilation");

    // Should find word regions in a text document
    rp.compare_values(1.0, if boxa.len() > 0 { 1.0 } else { 0.0 }, 0.0);

    // All boxes should have valid dimensions
    let all_valid = (0..boxa.len()).all(|i| {
        if let Some(b) = boxa.get(i) {
            b.w > 0 && b.h > 0
        } else {
            false
        }
    });
    rp.compare_values(1.0, if all_valid { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "jbclass word_boxes test failed");
}
