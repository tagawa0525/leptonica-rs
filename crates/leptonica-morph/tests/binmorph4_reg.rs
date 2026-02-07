//! Binary morphology regression test 4
//!
//! C version: reference/leptonica/prog/binmorph4_reg.c
//! Tests DWA brick vs standard morph comparison.
//! Compares DWA brick operations with standard brick operations
//! for various sizes to ensure they produce identical results.
//!
//! Run with:
//! ```
//! cargo test -p leptonica-morph --test binmorph4_reg
//! ```

use leptonica_core::PixelDepth;
use leptonica_morph::{
    close_brick, close_brick_dwa, dilate_brick, dilate_brick_dwa, erode_brick, erode_brick_dwa,
    open_brick, open_brick_dwa,
};
use leptonica_test::{RegParams, load_test_image};

/// C version: PixCompareDwa()
/// Compare three pairs of results (horizontal, vertical, square)
#[allow(clippy::too_many_arguments)]
fn pix_compare_dwa(
    rp: &mut RegParams,
    size: u32,
    op_type: &str,
    pix1: &leptonica_core::Pix,
    pix2: &leptonica_core::Pix,
    pix3: &leptonica_core::Pix,
    pix4: &leptonica_core::Pix,
    pix5: &leptonica_core::Pix,
    pix6: &leptonica_core::Pix,
) {
    let same = pix1.equals(pix2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("{} ({}, 1) not same", op_type, size);
    }
    let same = pix3.equals(pix4);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("{} (1, {}) not same", op_type, size);
    }
    let same = pix5.equals(pix6);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("{} ({}, {}) not same", op_type, size, size);
    }
}

/// Compare DWA brick ops vs standard brick ops for a given size.
/// C version tests 5 comparisons; we implement the ones for which Rust APIs exist.
/// C版: DoComparisonDwa1 -- pixDilateCompBrick vs pixDilateBrick -- composite未実装のためスキップ
/// C版: DoComparisonDwa2 -- pixDilateBrickDwa vs pixDilateCompBrick -- DWA vs brickで代用
/// C版: DoComparisonDwa3 -- pixDilateCompBrickDwa vs pixDilateBrickDwa -- composite DWA未実装のためスキップ
/// C版: DoComparisonDwa4 -- pixDilateCompBrickDwa vs pixDilateCompBrick -- composite未実装のためスキップ
/// C版: DoComparisonDwa5 -- pixDilateCompBrickDwa vs pixDilateBrick -- DWA vs brickで代用
fn do_comparison_dwa_vs_brick(rp: &mut RegParams, pixs: &leptonica_core::Pix, size: u32) {
    eprintln!("..{}..", size);

    // Dilation: DWA vs brick
    let pix1 = dilate_brick_dwa(pixs, size, 1).expect("dilate_brick_dwa h");
    let pix3 = dilate_brick_dwa(pixs, 1, size).expect("dilate_brick_dwa v");
    let pix5 = dilate_brick_dwa(pixs, size, size).expect("dilate_brick_dwa sq");
    let pix2 = dilate_brick(pixs, size, 1).expect("dilate_brick h");
    let pix4 = dilate_brick(pixs, 1, size).expect("dilate_brick v");
    let pix6 = dilate_brick(pixs, size, size).expect("dilate_brick sq");
    pix_compare_dwa(rp, size, "dilate", &pix1, &pix2, &pix3, &pix4, &pix5, &pix6);

    // Erosion: DWA vs brick
    let pix1 = erode_brick_dwa(pixs, size, 1).expect("erode_brick_dwa h");
    let pix3 = erode_brick_dwa(pixs, 1, size).expect("erode_brick_dwa v");
    let pix5 = erode_brick_dwa(pixs, size, size).expect("erode_brick_dwa sq");
    let pix2 = erode_brick(pixs, size, 1).expect("erode_brick h");
    let pix4 = erode_brick(pixs, 1, size).expect("erode_brick v");
    let pix6 = erode_brick(pixs, size, size).expect("erode_brick sq");
    pix_compare_dwa(rp, size, "erode", &pix1, &pix2, &pix3, &pix4, &pix5, &pix6);

    // Opening: DWA vs brick
    let pix1 = open_brick_dwa(pixs, size, 1).expect("open_brick_dwa h");
    let pix3 = open_brick_dwa(pixs, 1, size).expect("open_brick_dwa v");
    let pix5 = open_brick_dwa(pixs, size, size).expect("open_brick_dwa sq");
    let pix2 = open_brick(pixs, size, 1).expect("open_brick h");
    let pix4 = open_brick(pixs, 1, size).expect("open_brick v");
    let pix6 = open_brick(pixs, size, size).expect("open_brick sq");
    pix_compare_dwa(rp, size, "open", &pix1, &pix2, &pix3, &pix4, &pix5, &pix6);

    // Closing: DWA vs brick
    // C版: pixCloseSafeBrick / pixCloseSafeCompBrick -- Rust未実装のためclose_brickで代用
    let pix1 = close_brick_dwa(pixs, size, 1).expect("close_brick_dwa h");
    let pix3 = close_brick_dwa(pixs, 1, size).expect("close_brick_dwa v");
    let pix5 = close_brick_dwa(pixs, size, size).expect("close_brick_dwa sq");
    let pix2 = close_brick(pixs, size, 1).expect("close_brick h");
    let pix4 = close_brick(pixs, 1, size).expect("close_brick v");
    let pix6 = close_brick(pixs, size, size).expect("close_brick sq");
    pix_compare_dwa(rp, size, "close", &pix1, &pix2, &pix3, &pix4, &pix5, &pix6);
}

#[test]
fn binmorph4_reg() {
    let mut rp = RegParams::new("binmorph4");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    eprintln!("Testing with asymmetric boundary conditions");

    // C版: for (i = 2; i < 64; i++) with selectComposableSizes
    // Test sizes 2..16 (full range for DoComparisonDwa2/3 which uses i < 16)
    for size in 2..16 {
        do_comparison_dwa_vs_brick(&mut rp, &pixs, size);
    }

    // Larger representative sizes from DoComparisonDwa4/5 range
    for size in [16, 20, 25, 30].iter() {
        do_comparison_dwa_vs_brick(&mut rp, &pixs, *size);
    }

    // C版: TestAll(rp, pixs, TRUE) -- symmetric boundary conditions
    // C版: resetMorphBoundaryCondition(SYMMETRIC_MORPH_BC) -- Rust未実装のためスキップ
    // C版: pixAddBorder() -- Rust未実装のためスキップ
    // C版: pixTransferAllData() -- Rust未実装のためスキップ

    eprintln!();
    assert!(rp.cleanup(), "binmorph4 regression test failed");
}
