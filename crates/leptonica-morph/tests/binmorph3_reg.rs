//! Binary morphology regression test 3 - boundary conditions
//!
//! C版: reference/leptonica/prog/binmorph3_reg.c
//! 境界条件でのmorphology操作をテスト。
//! 小さい画像、大きいカーネル、エッジケースを検証。

use leptonica_core::{Pix, PixelDepth};
use leptonica_morph::{Sel, close_brick, dilate_brick, erode_brick, open_brick};
use leptonica_test::{RegParams, load_test_image};

#[test]
fn binmorph3_reg() {
    let mut rp = RegParams::new("binmorph3");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    // --- Test 1: Dilation with size 1x1 should be identity ---
    let dil_11 = dilate_brick(&pixs, 1, 1).expect("dilate 1x1");
    let same = compare_pix(&pixs, &dil_11);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  dilate(1,1) == identity: {}", same);

    // --- Test 2: Erosion with size 1x1 should be identity ---
    let ero_11 = erode_brick(&pixs, 1, 1).expect("erode 1x1");
    let same = compare_pix(&pixs, &ero_11);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  erode(1,1) == identity: {}", same);

    // --- Test 3: Opening with size 1x1 should be identity ---
    let open_11 = open_brick(&pixs, 1, 1).expect("open 1x1");
    let same = compare_pix(&pixs, &open_11);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  open(1,1) == identity: {}", same);

    // --- Test 4: Closing with size 1x1 should be identity ---
    let close_11 = close_brick(&pixs, 1, 1).expect("close 1x1");
    let same = compare_pix(&pixs, &close_11);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  close(1,1) == identity: {}", same);

    // --- Test 5: Horizontal-only dilation (Nx1) ---
    let dil_h = dilate_brick(&pixs, 21, 1).expect("dilate 21x1");
    let dil_fg = count_foreground(&dil_h);
    let orig_fg = count_foreground(&pixs);
    rp.compare_values(1.0, if dil_fg >= orig_fg { 1.0 } else { 0.0 }, 0.0);

    // --- Test 6: Vertical-only dilation (1xN) ---
    let dil_v = dilate_brick(&pixs, 1, 21).expect("dilate 1x21");
    let dil_v_fg = count_foreground(&dil_v);
    rp.compare_values(1.0, if dil_v_fg >= orig_fg { 1.0 } else { 0.0 }, 0.0);

    // --- Test 7: Separable dilation should equal direct dilation ---
    // dilate(hsize, vsize) should == dilate(hsize, 1) then dilate(1, vsize)
    let dil_direct = dilate_brick(&pixs, 11, 7).expect("dilate 11x7");
    let dil_sep_h = dilate_brick(&pixs, 11, 1).expect("dilate 11x1");
    let dil_sep_hv = dilate_brick(&dil_sep_h, 1, 7).expect("dilate 1x7 after h");
    let same = compare_pix(&dil_direct, &dil_sep_hv);
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

fn count_foreground(pix: &Pix) -> u64 {
    let mut count = 0u64;
    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if pix.get_pixel(x, y).unwrap_or(0) != 0 {
                count += 1;
            }
        }
    }
    count
}

fn compare_pix(pix1: &Pix, pix2: &Pix) -> bool {
    if pix1.width() != pix2.width() || pix1.height() != pix2.height() {
        return false;
    }
    for y in 0..pix1.height() {
        for x in 0..pix1.width() {
            if pix1.get_pixel(x, y) != pix2.get_pixel(x, y) {
                return false;
            }
        }
    }
    true
}
