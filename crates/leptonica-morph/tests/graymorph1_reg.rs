//! Gray morphology regression test 1
//!
//! C version: reference/leptonica/prog/graymorph1_reg.c
//! Tests:
//!   (1) Gray morph operations and gray_morph_sequence interpreter
//!   (2) Composite operations: tophat
//!   (3) Duality for grayscale erode/dilate, open/close, and tophat
//!   (4) Closing plus white tophat
//!   (5) Contrast enhancement (not tested: requires pixInitAccumulate, Pixacc -- Rust未実装)
//!   (6) Feynman stamp tophat extraction (not tested: requires pixRemoveColormap etc.)
//!
//! Run with:
//! ```
//! cargo test -p leptonica-morph --test graymorph1_reg
//! ```

use leptonica_core::PixelDepth;
use leptonica_morph::{
    bottom_hat_gray, close_gray, dilate_gray, erode_gray, gray_morph_sequence, open_gray,
    top_hat_gray,
};
use leptonica_test::{RegParams, load_test_image};

const WSIZE: u32 = 7;
const HSIZE: u32 = 7;

/// Compare two Pix for equality
fn compare_pix(pix1: &leptonica_core::Pix, pix2: &leptonica_core::Pix) -> bool {
    if pix1.width() != pix2.width() || pix1.height() != pix2.height() {
        return false;
    }
    pix1.equals(pix2)
}

/// Invert an 8-bpp grayscale image (255 - pixel)
fn invert_gray(pix: &leptonica_core::Pix) -> leptonica_core::Pix {
    pix.invert()
}

#[test]
fn graymorph1_reg() {
    let mut rp = RegParams::new("graymorph1");

    let pixs = load_test_image("aneurisms8.jpg").expect("load aneurisms8.jpg");
    assert_eq!(
        pixs.depth(),
        PixelDepth::Bit8,
        "Test image should be 8-bpp grayscale"
    );

    // ====================================================================
    // Test 1: Gray morph operations vs gray_morph_sequence interpreter
    // ====================================================================

    // C版: Test 0,1 -- Dilation
    eprintln!("  Testing gray dilation vs sequence");
    let pix1 = dilate_gray(&pixs, WSIZE, HSIZE).expect("dilate_gray");
    let seq = format!("D{}.{}", WSIZE, HSIZE);
    let pix2 = gray_morph_sequence(&pixs, &seq).expect("gray_morph_sequence D");
    let same = compare_pix(&pix1, &pix2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!(
            "    DIFFER: dilate_gray vs gray_morph_sequence(\"{}\")",
            seq
        );
    }

    // C版: Test 2,3 -- Erosion
    eprintln!("  Testing gray erosion vs sequence");
    let pix1 = erode_gray(&pixs, WSIZE, HSIZE).expect("erode_gray");
    let seq = format!("E{}.{}", WSIZE, HSIZE);
    let pix2 = gray_morph_sequence(&pixs, &seq).expect("gray_morph_sequence E");
    let same = compare_pix(&pix1, &pix2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: erode_gray vs gray_morph_sequence(\"{}\")", seq);
    }

    // C版: Test 4,5 -- Opening
    eprintln!("  Testing gray opening vs sequence");
    let pix1 = open_gray(&pixs, WSIZE, HSIZE).expect("open_gray");
    let seq = format!("O{}.{}", WSIZE, HSIZE);
    let pix2 = gray_morph_sequence(&pixs, &seq).expect("gray_morph_sequence O");
    let same = compare_pix(&pix1, &pix2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: open_gray vs gray_morph_sequence(\"{}\")", seq);
    }

    // C版: Test 6,7 -- Closing
    eprintln!("  Testing gray closing vs sequence");
    let pix1 = close_gray(&pixs, WSIZE, HSIZE).expect("close_gray");
    let seq = format!("C{}.{}", WSIZE, HSIZE);
    let pix2 = gray_morph_sequence(&pixs, &seq).expect("gray_morph_sequence C");
    let same = compare_pix(&pix1, &pix2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: close_gray vs gray_morph_sequence(\"{}\")", seq);
    }

    // C版: Test 8,9 -- White tophat
    eprintln!("  Testing white tophat vs sequence");
    let pix1 = top_hat_gray(&pixs, WSIZE, HSIZE).expect("top_hat_gray");
    let seq = format!("Tw{}.{}", WSIZE, HSIZE);
    let pix2 = gray_morph_sequence(&pixs, &seq).expect("gray_morph_sequence Tw");
    let same = compare_pix(&pix1, &pix2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!(
            "    DIFFER: top_hat_gray vs gray_morph_sequence(\"{}\")",
            seq
        );
    }

    // C版: Test 10,11 -- Black tophat
    eprintln!("  Testing black tophat vs sequence");
    let pix1 = bottom_hat_gray(&pixs, WSIZE, HSIZE).expect("bottom_hat_gray");
    let seq = format!("Tb{}.{}", WSIZE, HSIZE);
    let pix2 = gray_morph_sequence(&pixs, &seq).expect("gray_morph_sequence Tb");
    let same = compare_pix(&pix1, &pix2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!(
            "    DIFFER: bottom_hat_gray vs gray_morph_sequence(\"{}\")",
            seq
        );
    }

    // ====================================================================
    // Test 2: Erode/dilate duality
    // C版: Test 12,13 -- pixDilateGray(pixs) == pixInvert(pixErodeGray(pixInvert(pixs)))
    // ====================================================================
    eprintln!("  Testing erode/dilate duality");
    let pix1 = dilate_gray(&pixs, WSIZE, HSIZE).expect("dilate_gray");
    let pix2 = invert_gray(&pixs);
    let pix3 = erode_gray(&pix2, WSIZE, HSIZE).expect("erode_gray on inverted");
    let pix3_inv = invert_gray(&pix3);
    let same = compare_pix(&pix1, &pix3_inv);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: dilate != invert(erode(invert))");
    }

    // ====================================================================
    // Test 3: Open/close duality
    // C版: Test 14,15 -- pixOpenGray(pixs) == pixInvert(pixCloseGray(pixInvert(pixs)))
    // ====================================================================
    eprintln!("  Testing open/close duality");
    let pix1 = open_gray(&pixs, WSIZE, HSIZE).expect("open_gray");
    let pix2 = invert_gray(&pixs);
    let pix3 = close_gray(&pix2, WSIZE, HSIZE).expect("close_gray on inverted");
    let pix3_inv = invert_gray(&pix3);
    let same = compare_pix(&pix1, &pix3_inv);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: open != invert(close(invert))");
    }

    // ====================================================================
    // Test 4: Tophat duality
    // C版: Test 16,17 -- pixTophat(pixs, WHITE) == pixTophat(pixInvert(pixs), BLACK)
    // ====================================================================
    eprintln!("  Testing tophat duality");
    let pix1 = top_hat_gray(&pixs, WSIZE, HSIZE).expect("top_hat white");
    let pix2 = invert_gray(&pixs);
    let pix3 = bottom_hat_gray(&pix2, WSIZE, HSIZE).expect("bottom_hat on inverted");
    let same = compare_pix(&pix1, &pix3);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: white_tophat(pixs) != black_tophat(invert(pixs))");
    }

    // C版: Test 18,19 -- Same duality via gray_morph_sequence
    eprintln!("  Testing tophat duality via sequence");
    let pix1 = gray_morph_sequence(&pixs, "Tw9.5").expect("Tw9.5");
    let pix2 = invert_gray(&pixs);
    let pix3 = gray_morph_sequence(&pix2, "Tb9.5").expect("Tb9.5");
    let same = compare_pix(&pix1, &pix3);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: Tw9.5 != Tb9.5(inverted)");
    }

    // ====================================================================
    // Test 5: Large sel opening/closing sequences
    // C版: Test 20 -- pixGrayMorphSequence(pixs, "C9.9 + C19.19 + C29.29 + C39.39 + C49.49")
    // ====================================================================
    eprintln!("  Testing large sel sequences");
    let pix1 = gray_morph_sequence(&pixs, "C9.9 + C19.19 + C29.29 + C39.39 + C49.49")
        .expect("close sequence");
    assert_eq!(pix1.depth(), PixelDepth::Bit8);
    rp.compare_values(1.0, 1.0, 0.0); // Just verify it runs successfully

    // C版: Test 21 -- pixGrayMorphSequence(pixs, "O9.9 + O19.19 + O29.29 + O39.39 + O49.49")
    let pix1 = gray_morph_sequence(&pixs, "O9.9 + O19.19 + O29.29 + O39.39 + O49.49")
        .expect("open sequence");
    assert_eq!(pix1.depth(), PixelDepth::Bit8);
    rp.compare_values(1.0, 1.0, 0.0);

    // ====================================================================
    // Test 6: Closing plus white tophat
    // C版: Test 23,24 -- pixCloseGray(9,9) then pixTophat(9,9) == pixGrayMorphSequence("C9.9 + TW9.9")
    // ====================================================================
    eprintln!("  Testing closing + white tophat");
    let pix_closed = close_gray(&pixs, 9, 9).expect("close_gray 9x9");
    let pix_tophat = top_hat_gray(&pix_closed, 9, 9).expect("tophat on closed");
    let pix_seq = gray_morph_sequence(&pixs, "C9.9 + Tw9.9").expect("C9.9+Tw9.9");
    let same = compare_pix(&pix_tophat, &pix_seq);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: close+tophat vs sequence C9.9+Tw9.9");
    }

    // Same with 29x29
    let pix_closed = close_gray(&pixs, 29, 29).expect("close_gray 29x29");
    let pix_tophat = top_hat_gray(&pix_closed, 29, 29).expect("tophat on closed 29");
    let pix_seq = gray_morph_sequence(&pixs, "C29.29 + Tw29.29").expect("C29.29+Tw29.29");
    let same = compare_pix(&pix_tophat, &pix_seq);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: close+tophat vs sequence C29.29+Tw29.29");
    }

    // C版: Test 29,30 -- pixHDome(pixs, 100, 4) -- Rust未実装のためスキップ
    // C版: Test 31-35 -- pixInitAccumulate / pixAccumulate / Pixacc -- Rust未実装のためスキップ
    // C版: Test 37-42 -- feynman-stamp tophat extraction -- pixRemoveColormap / pixConvertRGBToGray / pixConvertTo32 / pixRasterop / pixGammaTRC / pixThresholdToBinary / pixMaxDynamicRange -- Rust未実装のためスキップ

    eprintln!();
    assert!(rp.cleanup(), "graymorph1 regression test failed");
}
