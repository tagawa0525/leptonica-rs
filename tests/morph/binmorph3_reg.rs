//! Binary morphology regression test 3 - boundary conditions
//!
//! C版: prog/binmorph3_reg.c
//! 境界条件でのmorphology操作をテスト。
//! 1x1 identity, 分離可能性, 次元保持を検証。
//!
//! C版はDWA vs rasterop の等価比較がメイン。Rust版は brick 実装の性質検証 +
//! 分離可能性検証結果のgolden化。
//!
//! Checkpoint mapping:
//!   compare_values 1-13: 性質検証（identity, 単調性, 分離可能性, 次元）
//!   write_pix_and_check 14-16: separable dilation, direct dilation, horizontal dilation

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::morph::{Sel, close_brick, dilate_brick, erode_brick, open_brick};

#[test]
fn binmorph3_reg() {
    let mut rp = RegParams::new("binmorph3");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    // --- Identity tests (1x1 kernel) ---
    let dil_11 = dilate_brick(&pixs, 1, 1).expect("dilate 1x1");
    rp.compare_values(1.0, if pixs.equals(&dil_11) { 1.0 } else { 0.0 }, 0.0);

    let ero_11 = erode_brick(&pixs, 1, 1).expect("erode 1x1");
    rp.compare_values(1.0, if pixs.equals(&ero_11) { 1.0 } else { 0.0 }, 0.0);

    let open_11 = open_brick(&pixs, 1, 1).expect("open 1x1");
    rp.compare_values(1.0, if pixs.equals(&open_11) { 1.0 } else { 0.0 }, 0.0);

    let close_11 = close_brick(&pixs, 1, 1).expect("close 1x1");
    rp.compare_values(1.0, if pixs.equals(&close_11) { 1.0 } else { 0.0 }, 0.0);

    // --- Directional dilation ---
    let orig_fg = pixs.count_pixels();
    let dil_h = dilate_brick(&pixs, 21, 1).expect("dilate 21x1");
    rp.compare_values(
        1.0,
        if dil_h.count_pixels() >= orig_fg {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let dil_v = dilate_brick(&pixs, 1, 21).expect("dilate 1x21");
    rp.compare_values(
        1.0,
        if dil_v.count_pixels() >= orig_fg {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // --- Separable dilation: dilate(11,7) == dilate(11,1) + dilate(1,7) ---
    let dil_direct = dilate_brick(&pixs, 11, 7).expect("dilate 11x7");
    let dil_sep_h = dilate_brick(&pixs, 11, 1).expect("dilate 11x1");
    let dil_sep_hv = dilate_brick(&dil_sep_h, 1, 7).expect("dilate 1x7 after h");
    rp.compare_values(
        1.0,
        if dil_direct.equals(&dil_sep_hv) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // --- SEL creation ---
    let sel = Sel::create_brick(5, 5).expect("create sel");
    rp.compare_values(5.0, sel.width() as f64, 0.0);
    rp.compare_values(5.0, sel.height() as f64, 0.0);

    // --- Dimension preservation ---
    let dil = dilate_brick(&pixs, 15, 15).expect("dilate 15x15");
    rp.compare_values(pixs.width() as f64, dil.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, dil.height() as f64, 0.0);

    let ero = erode_brick(&pixs, 15, 15).expect("erode 15x15");
    rp.compare_values(pixs.width() as f64, ero.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, ero.height() as f64, 0.0);

    // --- Golden checks: separable vs direct dilation ---
    rp.write_pix_and_check(&dil_sep_hv, ImageFormat::Tiff)
        .expect("write separable dilation");
    rp.write_pix_and_check(&dil_direct, ImageFormat::Tiff)
        .expect("write direct dilation");
    rp.write_pix_and_check(&dil_h, ImageFormat::Tiff)
        .expect("write horizontal dilation");

    assert!(rp.cleanup(), "binmorph3 regression test failed");
}
