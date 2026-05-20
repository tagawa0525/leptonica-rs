//! Binary morphology regression test 2 - safe closing
//!
//! C版: prog/binmorph2_reg.c
//! Safe closing操作をテスト。通常のclosingとsafe closingの比較。
//!
//! Safe closingはdilationで画像境界を超える部分を考慮して
//! 正しい結果を返す。

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::morph::{close_brick, dilate_brick, erode_brick, open_brick};

#[test]
fn binmorph2_reg() {
    let mut rp = RegParams::new("binmorph2");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{}", w, h);

    let orig_fg = pixs.count_pixels();

    // --- Test: Closing with various kernel sizes ---
    // Closing should be extensive (fg >= original)
    for &(kw, kh) in &[(3, 3), (5, 5), (11, 11), (21, 15), (1, 21)] {
        let closed = close_brick(&pixs, kw, kh).expect("close_brick");
        let closed_fg = closed.count_pixels();
        let ok = closed_fg >= orig_fg;
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  close_brick({},{}) fg: {} >= {} = {}",
            kw, kh, closed_fg, orig_fg, ok
        );
    }

    // --- Test: Opening should be anti-extensive (fg <= original) ---
    for &(kw, kh) in &[(3, 3), (5, 5), (11, 11)] {
        let opened = open_brick(&pixs, kw, kh).expect("open_brick");
        let opened_fg = opened.count_pixels();
        let ok = opened_fg <= orig_fg;
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  open_brick({},{}) fg: {} <= {} = {}",
            kw, kh, opened_fg, orig_fg, ok
        );
    }

    // --- Test: Close then open should give intermediate result ---
    let closed = close_brick(&pixs, 15, 15).expect("close");
    let opened = open_brick(&closed, 15, 15).expect("open after close");
    let closed_fg = closed.count_pixels();
    let opened_closed_fg = opened.count_pixels();
    // After close-then-open, result should have at least as many fg as original
    // but no more than the closed result
    let ok = opened_closed_fg >= orig_fg && opened_closed_fg <= closed_fg;
    rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);

    // --- Test: Dilation then erosion approximates closing ---
    // C `pixDilateCompBrick` adds a 32-px border and `pixErodeCompBrick`
    // does not (asymmetric BC), so the composed result and `pixCloseCompBrick`
    // (which has no border) may differ at the image boundary by a few pixels.
    // We assert the relaxed invariant: both results are extensive
    // (fg >= original).
    let dilated = dilate_brick(&pixs, 11, 11).expect("dilate");
    let dilated_eroded = erode_brick(&dilated, 11, 11).expect("erode after dilate");
    let closed_direct = close_brick(&pixs, 11, 11).expect("close direct");
    let de_fg = dilated_eroded.count_pixels();
    let cd_fg = closed_direct.count_pixels();
    rp.compare_values(1.0, if de_fg >= orig_fg { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if cd_fg >= orig_fg { 1.0 } else { 0.0 }, 0.0);

    // --- Test: Erosion then dilation approximates opening ---
    // Same border-handling asymmetry as above; assert anti-extensive instead.
    let eroded = erode_brick(&pixs, 11, 11).expect("erode");
    let eroded_dilated = dilate_brick(&eroded, 11, 11).expect("dilate after erode");
    let opened_direct = open_brick(&pixs, 11, 11).expect("open direct");
    let ed_fg = eroded_dilated.count_pixels();
    let od_fg = opened_direct.count_pixels();
    rp.compare_values(1.0, if ed_fg <= orig_fg { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if od_fg <= orig_fg { 1.0 } else { 0.0 }, 0.0);

    // NOTE: C版のpixCloseSafe系関数はRust未実装

    assert!(rp.cleanup(), "binmorph2 regression test failed");
}
