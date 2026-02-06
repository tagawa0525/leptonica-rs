//! DWA morphology regression test
//!
//! C版: reference/leptonica/prog/dwamorph1_reg.c
//! DWA (destination word accumulation) 高速morphology操作をテスト。
//! DWA結果が通常のbrick操作と一致することを検証。

use leptonica_core::PixelDepth;
use leptonica_morph::{
    close_brick, close_brick_dwa, dilate_brick, dilate_brick_dwa, erode_brick, erode_brick_dwa,
    open_brick, open_brick_dwa,
};
use leptonica_test::{RegParams, load_test_image};

#[test]
fn dwamorph1_reg() {
    let mut rp = RegParams::new("dwamorph1");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    // Test various kernel sizes
    let sizes = [(3, 3), (5, 5), (7, 7), (3, 1), (1, 3), (11, 1), (1, 11)];

    for &(hsize, vsize) in &sizes {
        eprintln!("Testing DWA vs brick: {}x{}", hsize, vsize);

        // --- Dilation: DWA vs brick ---
        let dil_brick = dilate_brick(&pixs, hsize, vsize).expect("dilate_brick");
        let dil_dwa = dilate_brick_dwa(&pixs, hsize, vsize).expect("dilate_brick_dwa");
        let same = compare_pix(&dil_brick, &dil_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!("  dilate: {}", if same { "MATCH" } else { "DIFFER" });

        // --- Erosion: DWA vs brick ---
        let ero_brick = erode_brick(&pixs, hsize, vsize).expect("erode_brick");
        let ero_dwa = erode_brick_dwa(&pixs, hsize, vsize).expect("erode_brick_dwa");
        let same = compare_pix(&ero_brick, &ero_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!("  erode:  {}", if same { "MATCH" } else { "DIFFER" });

        // --- Opening: DWA vs brick ---
        let open_brick_r = open_brick(&pixs, hsize, vsize).expect("open_brick");
        let open_dwa = open_brick_dwa(&pixs, hsize, vsize).expect("open_brick_dwa");
        let same = compare_pix(&open_brick_r, &open_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!("  open:   {}", if same { "MATCH" } else { "DIFFER" });

        // --- Closing: DWA vs brick ---
        let close_brick_r = close_brick(&pixs, hsize, vsize).expect("close_brick");
        let close_dwa = close_brick_dwa(&pixs, hsize, vsize).expect("close_brick_dwa");
        let same = compare_pix(&close_brick_r, &close_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!("  close:  {}", if same { "MATCH" } else { "DIFFER" });
    }

    assert!(rp.cleanup(), "dwamorph1 regression test failed");
}

fn compare_pix(pix1: &leptonica_core::Pix, pix2: &leptonica_core::Pix) -> bool {
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
