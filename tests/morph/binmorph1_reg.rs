//! Binary morphology regression test
//!
//! C version: reference/leptonica/prog/binmorph1_reg.c
//! Tests dilation, erosion, opening, and closing operations.
//!
//! C版は複数アルゴリズム（rasterop, dwa, sequence）間の等価比較がメイン。
//! Rust版は brick 実装の性質検証 + 代表結果のgolden化。
//!
//! Checkpoint mapping:
//!   compare_values 1-8: 性質検証（単調性・冪等性・SEL等価性）
//!   write_pix_and_check 9-12: dilate/erode/open/close 結果画像

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::morph::{Sel, close_brick, dilate_brick, erode_brick, open_brick};

// Brick sel dimensions (matching C version)
const WIDTH: u32 = 21;
const HEIGHT: u32 = 15;

#[test]
fn binmorph1_reg() {
    let mut rp = RegParams::new("binmorph1");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    let orig_count = pixs.count_pixels();

    // --- Property checks (compare_values 1-8) ---

    // Dilation should increase foreground pixels
    let dilated = dilate_brick(&pixs, WIDTH, HEIGHT).expect("dilate_brick");
    let dilated_count = dilated.count_pixels();
    rp.compare_values(
        1.0,
        if dilated_count >= orig_count {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Erosion should decrease foreground pixels
    let eroded = erode_brick(&pixs, WIDTH, HEIGHT).expect("erode_brick");
    let eroded_count = eroded.count_pixels();
    rp.compare_values(1.0, if eroded_count <= orig_count { 1.0 } else { 0.0 }, 0.0);

    // Opening should be anti-extensive
    let opened = open_brick(&pixs, WIDTH, HEIGHT).expect("open_brick");
    let opened_count = opened.count_pixels();
    rp.compare_values(1.0, if opened_count <= orig_count { 1.0 } else { 0.0 }, 0.0);

    // Closing should be extensive
    let closed = close_brick(&pixs, WIDTH, HEIGHT).expect("close_brick");
    let closed_count = closed.count_pixels();
    rp.compare_values(1.0, if closed_count >= orig_count { 1.0 } else { 0.0 }, 0.0);

    // Opening idempotence
    let opened2 = open_brick(&opened, WIDTH, HEIGHT).expect("open_brick twice");
    rp.compare_values(1.0, if opened.equals(&opened2) { 1.0 } else { 0.0 }, 0.0);

    // Closing idempotence
    let closed2 = close_brick(&closed, WIDTH, HEIGHT).expect("close_brick twice");
    rp.compare_values(1.0, if closed.equals(&closed2) { 1.0 } else { 0.0 }, 0.0);

    // SEL-based operations match brick operations
    let sel = Sel::create_brick(WIDTH, HEIGHT).expect("create_brick SEL");
    let dilated_sel = leptonica::morph::dilate(&pixs, &sel).expect("dilate with SEL");
    rp.compare_values(
        1.0,
        if dilated.equals(&dilated_sel) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let eroded_sel = leptonica::morph::erode(&pixs, &sel).expect("erode with SEL");
    rp.compare_values(1.0, if eroded.equals(&eroded_sel) { 1.0 } else { 0.0 }, 0.0);

    // --- Golden checks (write_pix_and_check 9-12) ---
    rp.write_pix_and_check(&dilated, ImageFormat::Tiff)
        .expect("write dilate result");
    rp.write_pix_and_check(&eroded, ImageFormat::Tiff)
        .expect("write erode result");
    rp.write_pix_and_check(&opened, ImageFormat::Tiff)
        .expect("write open result");
    rp.write_pix_and_check(&closed, ImageFormat::Tiff)
        .expect("write close result");

    assert!(rp.cleanup(), "binmorph1 regression test failed");
}
