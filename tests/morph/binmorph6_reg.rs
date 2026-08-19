//! Binary morphology 6 regression test
//!
//! Tests miscellaneous morphological operations: making a Sel from a Pix,
//! then applying dilate, open, close_safe, and subtract to a binary image.
//!
//! Partial migration: dilate, open, close_safe, subtract are tested.
//! pixRemoveBorder and Pixa display operations are not available here.
//!
//! # See also
//!
//! C Leptonica: `prog/binmorph6_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::morph::{Sel, SelElement, close_safe, dilate, open};

/// Test dilate, open, close_safe with a custom Sel from a Pix (C checks 0-5).
///
/// C: pix3 = pixDilate(NULL, pix1, sel);
///    pix4 = pixOpen(NULL, pix1, sel);
///    pix5 = pixCloseSafe(NULL, pix1, sel);
///    pix6 = pixSubtract(NULL, pix3, pix1);
#[test]
fn binmorph6_reg_custom_sel() {
    let mut rp = RegParams::new("bmorph6_sel");

    // C: pix1 = pixRead("feyn-fract.tif");
    let pix1 = crate::common::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);
    let w = pix1.width();
    let h = pix1.height();

    // C: sel = selCreateBrick(5, 5, 2, 2, SEL_HIT);
    let sel = Sel::create_brick(5, 5).expect("create_brick sel");
    assert_eq!(sel.width(), 5);
    assert_eq!(sel.height(), 5);

    // C: pix3 = pixDilate(NULL, pix1, sel);
    let dilated = dilate(&pix1, &sel).expect("dilate");
    rp.compare_values(w as f64, dilated.width() as f64, 0.0);
    rp.compare_values(h as f64, dilated.height() as f64, 0.0);
    assert_eq!(dilated.depth(), PixelDepth::Bit1);

    // C: pix4 = pixOpen(NULL, pix1, sel);
    let opened = open(&pix1, &sel).expect("open");
    rp.compare_values(w as f64, opened.width() as f64, 0.0);

    // C: pix5 = pixCloseSafe(NULL, pix1, sel);
    let closed = close_safe(&pix1, &sel).expect("close_safe");
    rp.compare_values(w as f64, closed.width() as f64, 0.0);
    assert_eq!(closed.depth(), PixelDepth::Bit1);

    // C: pix6 = pixSubtract(NULL, pix3, pix1);  -- dilated minus original
    let subtracted = dilated.subtract(&pix1).expect("subtract");
    rp.compare_values(w as f64, subtracted.width() as f64, 0.0);

    // C: pix7 = pixSubtract(NULL, pix1, pix5);  -- original minus closed (= empty)
    let subtracted2 = pix1
        .subtract(&closed)
        .expect("subtract original minus closed");
    rp.compare_values(w as f64, subtracted2.width() as f64, 0.0);

    assert!(rp.cleanup(), "binmorph6 custom sel test failed");
}

/// Test Sel::from_string (C: selCreateFromString equivalent).
///
/// Creates a custom structuring element from a pattern string.
#[test]
fn binmorph6_reg_sel_from_string() {
    let mut rp = RegParams::new("bmorph6_strsel");

    // C: pattern like "oXo / XXX / oXo" for a cross
    let cross = Sel::from_string("oXo\nXXX\noXo", 1, 1).expect("from_string cross");
    assert_eq!(cross.width(), 3);
    assert_eq!(cross.height(), 3);
    rp.compare_values(5.0, cross.hit_count() as f64, 0.0);

    // Verify center element
    assert_eq!(cross.get_element(1, 1), Some(SelElement::Hit));
    // Verify corner elements ('o' = Miss in from_string)
    assert_eq!(cross.get_element(0, 0), Some(SelElement::Miss));

    assert!(rp.cleanup(), "binmorph6 sel_from_string test failed");
}

/// C-compat: `prog/binmorph6_reg.c`, all 7 outputs.
///
/// Builds a hit-only sel from a clip of `feyn-fract.tif` and applies it with
/// dilate / open / close-safe. The only input is lossless and C writes every
/// output as PNG, so the whole program is bit-exactly comparable.
#[test]
fn binmorph6_c_compat() {
    use leptonica::core::Pixa;
    use leptonica::io::ImageFormat;
    use leptonica::morph::{Sel, close_safe, dilate, open};

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("binmorph6_c");

    let pix1 = crate::common::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    // C: boxCreate(507, 65, 60, 36) then pixClipRectangle.
    let pix2 = pix1.clip_rectangle(507, 65, 60, 36).expect("clip");
    // C: selCreateFromPix(pix2, 6, 6, "life"). C takes (cy, cx) while this
    // takes (cx, cy); both are 6 here, so the order does not matter.
    let sel = Sel::from_pix(&pix2, 6, 6).expect("sel from pix");

    let pix3 = dilate(&pix1, &sel).expect("dilate");
    let pix4 = open(&pix1, &sel).expect("open");
    let pix5 = close_safe(&pix1, &sel).expect("close safe");
    let pix6 = pix3.subtract(&pix1).expect("dilate - src");
    let pix7 = pix1.subtract(&pix5).expect("src - close");

    for pix in [&pix2, &pix3, &pix4, &pix5, &pix6, &pix7] {
        rp.write_pix_and_check(pix, ImageFormat::Png)
            .expect("check: binmorph6 stage");
    }

    let mut pixa = Pixa::new();
    for pix in [&pix1, &pix3, &pix4, &pix5, &pix6, &pix7] {
        pixa.push(pix.clone());
    }
    let tiled = pixa
        .display_tiled_in_columns(2, 0.75, 20, 2)
        .expect("tile binmorph6");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: binmorph6 summary");

    assert!(rp.cleanup(), "binmorph6 C-compat test failed");
}
