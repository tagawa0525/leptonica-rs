//! Morphological sequence regression test
//!
//! C version: reference/leptonica/prog/morphseq_reg.c
//! Tests morph_sequence and gray_morph_sequence interpreters,
//! including display mode and rejection of invalid sequence components.
//!
//! Run with:
//! ```
//! cargo test -p leptonica-morph --test morphseq_reg
//! ```

use leptonica_core::PixelDepth;
use leptonica_morph::{gray_morph_sequence, morph_comp_sequence, morph_sequence};
use leptonica_test::{RegParams, load_test_image};

// C version sequence definitions
const SEQUENCE1: &str = "O1.3 + C3.1";
// C版: "O1.3 + C3.1 + R22 + D2.2 + X4" -- R (rank reduction) と X (expansion) は未実装
const SEQUENCE2: &str = "O2.13 + C5.23";
// C版: "O2.13 + C5.23 + R22 + X4" -- R, X は未実装
const SEQUENCE3: &str = "e3.3 + d3.3 + tw5.5";
const SEQUENCE4: &str = "O3.3 + C3.3";
// C版: SEQUENCE5 = "O5.5 + C5.5" -- pixColorMorphSequence未実装のため直接使用せず
// C版: BAD_SEQUENCE = "O1.+D8 + E2.4 + e.4 + r25 + R + R.5 + X + x5 + y7.3"
const BAD_SEQUENCE: &str = "O1.+D8 + E2.4 + e.4 + r25 + y7.3";

#[test]
#[ignore = "not yet implemented"]
fn morphseq_reg() {
    let mut rp = RegParams::new("morphseq");

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    // ====================================================================
    // Test 1: Binary morph_sequence
    // C版: pixMorphSequence(pixs, SEQUENCE1, -1) -- display separation = -1 (internal display)
    //       pixMorphSequence(pixs, SEQUENCE1, DISPLAY_SEPARATION)
    // Rust: display mode not implemented, just run the sequence
    // ====================================================================
    eprintln!("  Testing binary morph_sequence with SEQUENCE1");
    // C版: SEQUENCE1 = "O1.3 + C3.1 + R22 + D2.2 + X4"
    // R (rank reduction) and X (expansion) are not supported in Rust.
    // We test the supported portion: "O1.3 + C3.1"
    let result = morph_sequence(&pixs, SEQUENCE1);
    match result {
        Ok(pixd) => {
            assert_eq!(pixd.depth(), PixelDepth::Bit1);
            rp.compare_values(1.0, 1.0, 0.0);
            eprintln!("    SEQUENCE1 succeeded");
        }
        Err(e) => {
            eprintln!("    SEQUENCE1 failed: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }

    // ====================================================================
    // Test 2: Binary morph_comp_sequence
    // C版: pixMorphCompSequence(pixs, SEQUENCE2, -2)
    //       pixMorphCompSequence(pixs, SEQUENCE2, DISPLAY_SEPARATION)
    // ====================================================================
    eprintln!("  Testing binary morph_comp_sequence with SEQUENCE2");
    // C版: SEQUENCE2 = "O2.13 + C5.23 + R22 + X4"
    // Supported portion: "O2.13 + C5.23"
    let result = morph_comp_sequence(&pixs, SEQUENCE2);
    match result {
        Ok(pixd) => {
            assert_eq!(pixd.depth(), PixelDepth::Bit1);
            rp.compare_values(1.0, 1.0, 0.0);
            eprintln!("    SEQUENCE2 comp succeeded");
        }
        Err(e) => {
            eprintln!("    SEQUENCE2 comp failed: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }

    // ====================================================================
    // Test 3: Verify morph_sequence == morph_comp_sequence for same input
    // C版では別々のsequenceだが、Rustではmorph_comp_sequenceはmorph_sequenceに委譲するため
    // 同じsequenceで結果が一致することを確認
    // ====================================================================
    eprintln!("  Testing morph_sequence == morph_comp_sequence");
    let seq = "O3.3 + C5.5 + D2.2";
    let pix1 = morph_sequence(&pixs, seq).expect("morph_sequence");
    let pix2 = morph_comp_sequence(&pixs, seq).expect("morph_comp_sequence");
    let same = pix1.equals(&pix2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: morph_sequence != morph_comp_sequence");
    }

    // C版: pixMorphSequenceDwa(pixs, SEQUENCE2, -3) -- Rust未実装 (morph_sequence_dwa)
    // C版: pixMorphCompSequenceDwa(pixs, SEQUENCE2, -4) -- Rust未実装 (morph_comp_sequence_dwa)

    // ====================================================================
    // Test 4: Grayscale morph sequence
    // C版: pixScaleToGray(pixs, 0.25) then pixGrayMorphSequence(pixg, SEQUENCE3, ...)
    // Rust: pixScaleToGray -- 未実装のため、test8.jpgで代用
    // ====================================================================
    eprintln!("  Testing gray morph sequence");
    let pixg = load_test_image("test8.jpg").expect("load test8.jpg");
    // Ensure it's 8-bpp
    assert_eq!(pixg.depth(), PixelDepth::Bit8);

    // C版: SEQUENCE3 = "e3.3 + d3.3 + tw5.5"
    let result = gray_morph_sequence(&pixg, SEQUENCE3);
    match result {
        Ok(pixd) => {
            assert_eq!(pixd.depth(), PixelDepth::Bit8);
            rp.compare_values(1.0, 1.0, 0.0);
            eprintln!("    SEQUENCE3 gray succeeded");
        }
        Err(e) => {
            eprintln!("    SEQUENCE3 gray failed: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }

    // C版: SEQUENCE4 = "O3.3 + C3.3"
    let result = gray_morph_sequence(&pixg, SEQUENCE4);
    match result {
        Ok(pixd) => {
            assert_eq!(pixd.depth(), PixelDepth::Bit8);
            rp.compare_values(1.0, 1.0, 0.0);
            eprintln!("    SEQUENCE4 gray succeeded");
        }
        Err(e) => {
            eprintln!("    SEQUENCE4 gray failed: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }

    // ====================================================================
    // Test 5: Verify gray sequence operations match individual operations
    // ====================================================================
    eprintln!("  Testing gray sequence vs individual ops");
    let pix1 = leptonica_morph::open_gray(&pixg, 3, 3).expect("open_gray");
    let pix2 = leptonica_morph::close_gray(&pix1, 3, 3).expect("close_gray after open");
    let pix3 = gray_morph_sequence(&pixg, "O3.3 + C3.3").expect("gray sequence O3.3+C3.3");
    let same = pix2.equals(&pix3);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: individual ops vs gray_morph_sequence");
    }

    // ====================================================================
    // Test 6: Color morph sequence -- not directly available
    // C版: pixColorMorphSequence(pixc, SEQUENCE5, -7, 150)
    // Rust: pixColorMorphSequence -- 未実装のためスキップ
    // ====================================================================
    // C版: pixRead("wyom.jpg") -> pixColorMorphSequence(pixc, "O5.5 + C5.5") -- Rust未実装

    // ====================================================================
    // Test 7: Bad sequence error handling
    // C版: pixMorphSequence(pixs, BAD_SEQUENCE, 50) -- returns null
    //       pixGrayMorphSequence(pixg, BAD_SEQUENCE, 50, 0) -- returns null
    // ====================================================================
    eprintln!("  Testing error handling for bad sequences");

    let result = morph_sequence(&pixs, BAD_SEQUENCE);
    let bad_binary_rejected = result.is_err();
    rp.compare_values(1.0, if bad_binary_rejected { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "    bad binary sequence: {}",
        if bad_binary_rejected {
            "correctly rejected"
        } else {
            "ERROR: not rejected"
        }
    );

    let result = gray_morph_sequence(&pixg, BAD_SEQUENCE);
    let bad_gray_rejected = result.is_err();
    rp.compare_values(1.0, if bad_gray_rejected { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "    bad gray sequence: {}",
        if bad_gray_rejected {
            "correctly rejected"
        } else {
            "ERROR: not rejected"
        }
    );

    // Additional: test individual bad sequences
    eprintln!("  Testing specific invalid sequences");
    // Empty dimension
    assert!(morph_sequence(&pixs, "O1.").is_err(), "O1. should fail");
    // Missing dimension
    assert!(morph_sequence(&pixs, "D8").is_err(), "D8 should fail");
    // Zero dimension
    assert!(morph_sequence(&pixs, "E0.4").is_err(), "E0.4 should fail");
    // Unknown operation
    assert!(morph_sequence(&pixs, "y7.3").is_err(), "y7.3 should fail");
    rp.compare_values(1.0, 1.0, 0.0); // All invalid sequences correctly rejected

    eprintln!();
    assert!(rp.cleanup(), "morphseq regression test failed");
}
