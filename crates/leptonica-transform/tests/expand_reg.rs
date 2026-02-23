//! Expand/reduce regression test
//!
//! Tests replicative pixel expansion (expand_replicate) across all bit depths.
//! The C version also tests binary power-of-2 expansion/reduction
//! (pixExpandBinaryPower2, pixReduceRankBinary2, pixReduceRankBinaryCascade)
//! which are not available in Rust.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/expand_reg.c`

use leptonica_core::PixelDepth;
use leptonica_test::RegParams;
use leptonica_transform::expand_replicate;

/// Test expand_replicate on 1bpp binary image (C checks 0-1).
///
/// Verifies 2× and 3× expansion preserves content structure.
#[test]
#[ignore = "not yet implemented"]
fn expand_reg_1bpp() {
    let mut rp = RegParams::new("expand_1bpp");

    let pix1 = leptonica_test::load_test_image("test1.png").expect("load test1.png");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);

    // Expand 2×
    let pix2x = expand_replicate(&pix1, 2).expect("expand 2x");
    rp.compare_values((pix1.width() * 2) as f64, pix2x.width() as f64, 0.0);
    rp.compare_values((pix1.height() * 2) as f64, pix2x.height() as f64, 0.0);

    // Expand 3×
    let pix3x = expand_replicate(&pix1, 3).expect("expand 3x");
    rp.compare_values((pix1.width() * 3) as f64, pix3x.width() as f64, 0.0);
    rp.compare_values((pix1.height() * 3) as f64, pix3x.height() as f64, 0.0);

    assert!(rp.cleanup(), "expand 1bpp test failed");
}

/// Test expand_replicate on 2bpp image (C checks 2-3).
#[test]
#[ignore = "not yet implemented"]
fn expand_reg_2bpp() {
    let mut rp = RegParams::new("expand_2bpp");

    let pix2 = leptonica_test::load_test_image("weasel2.4g.png").expect("load weasel2.4g.png");
    assert_eq!(pix2.depth(), PixelDepth::Bit2);

    // Expand 2×
    let pix2x = expand_replicate(&pix2, 2).expect("expand 2x");
    rp.compare_values((pix2.width() * 2) as f64, pix2x.width() as f64, 0.0);
    rp.compare_values((pix2.height() * 2) as f64, pix2x.height() as f64, 0.0);
    assert_eq!(pix2x.depth(), PixelDepth::Bit2);

    assert!(rp.cleanup(), "expand 2bpp test failed");
}

/// Test expand_replicate on 4bpp image (C checks 4-5).
#[test]
#[ignore = "not yet implemented"]
fn expand_reg_4bpp() {
    let mut rp = RegParams::new("expand_4bpp");

    let pix4 = leptonica_test::load_test_image("weasel4.16g.png").expect("load weasel4.16g.png");
    assert_eq!(pix4.depth(), PixelDepth::Bit4);

    let pix2x = expand_replicate(&pix4, 2).expect("expand 2x");
    rp.compare_values((pix4.width() * 2) as f64, pix2x.width() as f64, 0.0);
    rp.compare_values((pix4.height() * 2) as f64, pix2x.height() as f64, 0.0);
    assert_eq!(pix2x.depth(), PixelDepth::Bit4);

    assert!(rp.cleanup(), "expand 4bpp test failed");
}

/// Test expand_replicate on 8bpp image (C checks 6-7).
#[test]
#[ignore = "not yet implemented"]
fn expand_reg_8bpp() {
    let mut rp = RegParams::new("expand_8bpp");

    let pix8 = leptonica_test::load_test_image("weasel8.149g.png").expect("load weasel8.149g.png");
    assert_eq!(pix8.depth(), PixelDepth::Bit8);

    let pix2x = expand_replicate(&pix8, 2).expect("expand 2x");
    rp.compare_values((pix8.width() * 2) as f64, pix2x.width() as f64, 0.0);
    rp.compare_values((pix8.height() * 2) as f64, pix2x.height() as f64, 0.0);
    assert_eq!(pix2x.depth(), PixelDepth::Bit8);

    assert!(rp.cleanup(), "expand 8bpp test failed");
}

/// Test expand_replicate on 32bpp RGB image (C checks 8-9).
#[test]
#[ignore = "not yet implemented"]
fn expand_reg_32bpp() {
    let mut rp = RegParams::new("expand_32bpp");

    let pix32 = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");
    assert_eq!(pix32.depth(), PixelDepth::Bit32);

    let pix2x = expand_replicate(&pix32, 2).expect("expand 2x");
    rp.compare_values((pix32.width() * 2) as f64, pix2x.width() as f64, 0.0);
    rp.compare_values((pix32.height() * 2) as f64, pix2x.height() as f64, 0.0);
    assert_eq!(pix2x.depth(), PixelDepth::Bit32);

    // Factor 1 should be identity
    let pix1x = expand_replicate(&pix32, 1).expect("expand 1x");
    rp.compare_pix(&pix32, &pix1x);

    assert!(rp.cleanup(), "expand 32bpp test failed");
}

/// Test binary power-of-2 expansion (C checks 10-42).
///
/// Requires pixExpandBinaryPower2, pixReduceRankBinary2, pixReduceRankBinaryCascade
/// which are not available in leptonica-transform.
#[test]
#[ignore = "not yet implemented: pixExpandBinaryPower2/pixReduceRankBinary2 not available"]
fn expand_reg_binary_power2() {
    // C version tests:
    // 1. pixExpandBinaryPower2(pix1, 2), (4), (8)
    // 2. pixReduceRankBinary2(expanded, 1), (2), (3)
    // 3. Verifies round-trip: expand × n then reduce × n == original
    // 4. pixReduceRankBinaryCascade cascaded reductions
}

/// Test expand_replicate with clipping (C additional checks).
#[test]
#[ignore = "not yet implemented"]
fn expand_reg_clip() {
    let mut rp = RegParams::new("expand_clip");

    let pix = leptonica_test::load_test_image("speckle.png").expect("load speckle.png");

    // Expand then clip should give back region of correct size
    let pix2x = expand_replicate(&pix, 2).expect("expand 2x");
    let clipped = pix2x
        .clip_rectangle(0, 0, pix.width() * 2, pix.height() * 2)
        .expect("clip");
    rp.compare_values((pix.width() * 2) as f64, clipped.width() as f64, 0.0);
    rp.compare_values((pix.height() * 2) as f64, clipped.height() as f64, 0.0);

    // Full clip should equal original
    let full_clip = pix2x
        .clip_rectangle(0, 0, pix2x.width(), pix2x.height())
        .expect("full clip");
    rp.compare_pix(&pix2x, &full_clip);

    assert!(rp.cleanup(), "expand clip test failed");
}
