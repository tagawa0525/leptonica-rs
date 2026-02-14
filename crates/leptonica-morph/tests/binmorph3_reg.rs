//! Binary morphology regression test 3 - boundary conditions
//!
//! C版: reference/leptonica/prog/binmorph3_reg.c
//! 境界条件でのmorphology操作をテスト。
//! 小さい画像、大きいカーネル、エッジケースを検証。

use leptonica_core::PixelDepth;
use leptonica_morph::{Sel, close_brick, dilate_brick, erode_brick, open_brick};
use leptonica_test::{RegParams, load_test_image};

#[test]
#[ignore = "not yet implemented"]
fn binmorph3_reg() {
    let mut rp = RegParams::new("binmorph3");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    // --- Test 1: Dilation with size 1x1 should be identity ---
    let dil_11 = dilate_brick(&pixs, 1, 1).expect("dilate 1x1");
    let same = pixs.equals(&dil_11);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  dilate(1,1) == identity: {}", same);

    // --- Test 2: Erosion with size 1x1 should be identity ---
    let ero_11 = erode_brick(&pixs, 1, 1).expect("erode 1x1");
    let same = pixs.equals(&ero_11);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  erode(1,1) == identity: {}", same);

    // --- Test 3: Opening with size 1x1 should be identity ---
    let open_11 = open_brick(&pixs, 1, 1).expect("open 1x1");
    let same = pixs.equals(&open_11);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  open(1,1) == identity: {}", same);

    // --- Test 4: Closing with size 1x1 should be identity ---
    let close_11 = close_brick(&pixs, 1, 1).expect("close 1x1");
    let same = pixs.equals(&close_11);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  close(1,1) == identity: {}", same);

    // --- Test 5: Horizontal-only dilation (Nx1) ---
    let dil_h = dilate_brick(&pixs, 21, 1).expect("dilate 21x1");
    let dil_fg = dil_h.count_pixels();
    let orig_fg = pixs.count_pixels();
    rp.compare_values(1.0, if dil_fg >= orig_fg { 1.0 } else { 0.0 }, 0.0);

    // --- Test 6: Vertical-only dilation (1xN) ---
    let dil_v = dilate_brick(&pixs, 1, 21).expect("dilate 1x21");
    let dil_v_fg = dil_v.count_pixels();
    rp.compare_values(1.0, if dil_v_fg >= orig_fg { 1.0 } else { 0.0 }, 0.0);

    // --- Test 7: Separable dilation should equal direct dilation ---
    // dilate(hsize, vsize) should == dilate(hsize, 1) then dilate(1, vsize)
    let dil_direct = dilate_brick(&pixs, 11, 7).expect("dilate 11x7");
    let dil_sep_h = dilate_brick(&pixs, 11, 1).expect("dilate 11x1");
    let dil_sep_hv = dilate_brick(&dil_sep_h, 1, 7).expect("dilate 1x7 after h");
    let same = dil_direct.equals(&dil_sep_hv);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  separable dilation == direct: {}", same);

    // --- Test 8: SEL creation ---
    let sel = Sel::create_brick(5, 5).expect("create sel");
    rp.compare_values(5.0, sel.width() as f64, 0.0);
    rp.compare_values(5.0, sel.height() as f64, 0.0);

    // --- Test 9: Output dimensions should match input ---
    let dil = dilate_brick(&pixs, 15, 15).expect("dilate");
    rp.compare_values(pixs.width() as f64, dil.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, dil.height() as f64, 0.0);

    let ero = erode_brick(&pixs, 15, 15).expect("erode");
    rp.compare_values(pixs.width() as f64, ero.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, ero.height() as f64, 0.0);

    assert!(rp.cleanup(), "binmorph3 regression test failed");
}
