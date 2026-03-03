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

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::transform::expand_replicate;

/// Test expand_replicate on 1bpp binary image (C checks 0-1).
///
/// Verifies 2× and 3× expansion preserves content structure.
#[test]
fn expand_reg_1bpp() {
    let mut rp = RegParams::new("expand_1bpp");

    let pix1 = crate::common::load_test_image("test1.png").expect("load test1.png");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);

    // Expand 2×
    let pix2x = expand_replicate(&pix1, 2).expect("expand 2x");
    rp.compare_values((pix1.width() * 2) as f64, pix2x.width() as f64, 0.0);
    rp.compare_values((pix1.height() * 2) as f64, pix2x.height() as f64, 0.0);
    rp.write_pix_and_check(&pix2x, ImageFormat::Tiff)
        .expect("write pix2x");

    // Expand 3×
    let pix3x = expand_replicate(&pix1, 3).expect("expand 3x");
    rp.compare_values((pix1.width() * 3) as f64, pix3x.width() as f64, 0.0);
    rp.compare_values((pix1.height() * 3) as f64, pix3x.height() as f64, 0.0);

    assert!(rp.cleanup(), "expand 1bpp test failed");
}

/// Test expand_replicate on 2bpp image (C checks 2-3).
#[test]
fn expand_reg_2bpp() {
    let mut rp = RegParams::new("expand_2bpp");

    let pix2 = crate::common::load_test_image("weasel2.4g.png").expect("load weasel2.4g.png");
    assert_eq!(pix2.depth(), PixelDepth::Bit2);

    // Expand 2×
    let pix2x = expand_replicate(&pix2, 2).expect("expand 2x");
    rp.compare_values((pix2.width() * 2) as f64, pix2x.width() as f64, 0.0);
    rp.compare_values((pix2.height() * 2) as f64, pix2x.height() as f64, 0.0);
    rp.write_pix_and_check(&pix2x, ImageFormat::Png)
        .expect("write pix2x");
    assert_eq!(pix2x.depth(), PixelDepth::Bit2);

    assert!(rp.cleanup(), "expand 2bpp test failed");
}

/// Test expand_replicate on 4bpp image (C checks 4-5).
#[test]
fn expand_reg_4bpp() {
    let mut rp = RegParams::new("expand_4bpp");

    let pix4 = crate::common::load_test_image("weasel4.16g.png").expect("load weasel4.16g.png");
    assert_eq!(pix4.depth(), PixelDepth::Bit4);

    let pix2x = expand_replicate(&pix4, 2).expect("expand 2x");
    rp.compare_values((pix4.width() * 2) as f64, pix2x.width() as f64, 0.0);
    rp.compare_values((pix4.height() * 2) as f64, pix2x.height() as f64, 0.0);
    rp.write_pix_and_check(&pix2x, ImageFormat::Png)
        .expect("write pix2x");
    assert_eq!(pix2x.depth(), PixelDepth::Bit4);

    assert!(rp.cleanup(), "expand 4bpp test failed");
}

/// Test expand_replicate on 8bpp image (C checks 6-7).
#[test]
fn expand_reg_8bpp() {
    let mut rp = RegParams::new("expand_8bpp");

    let pix8 = crate::common::load_test_image("weasel8.149g.png").expect("load weasel8.149g.png");
    assert_eq!(pix8.depth(), PixelDepth::Bit8);

    let pix2x = expand_replicate(&pix8, 2).expect("expand 2x");
    rp.compare_values((pix8.width() * 2) as f64, pix2x.width() as f64, 0.0);
    rp.compare_values((pix8.height() * 2) as f64, pix2x.height() as f64, 0.0);
    rp.write_pix_and_check(&pix2x, ImageFormat::Png)
        .expect("write pix2x");
    assert_eq!(pix2x.depth(), PixelDepth::Bit8);

    assert!(rp.cleanup(), "expand 8bpp test failed");
}

/// Test expand_replicate on 32bpp RGB image (C checks 8-9).
#[test]
fn expand_reg_32bpp() {
    let mut rp = RegParams::new("expand_32bpp");

    let pix32 = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
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

/// Test binary power-of-2 expansion and reduce round-trip.
#[test]
fn expand_reg_binary_power2() {
    use leptonica::transform::binexpand::{expand_binary_power2, expand_binary_replicate};
    use leptonica::transform::binreduce::reduce_rank_binary_2;

    let pix1 = crate::common::load_test_image("test1.png").expect("load test1.png");

    // Expand 2x then reduce 2x
    let expanded = expand_binary_power2(&pix1, 2).unwrap();
    assert_eq!(expanded.width(), pix1.width() * 2);
    assert_eq!(expanded.height(), pix1.height() * 2);

    let reduced = reduce_rank_binary_2(&expanded, 2).unwrap();
    assert_eq!(reduced.width(), pix1.width());
    assert_eq!(reduced.height(), pix1.height());

    // Expand 4x
    let expanded4 = expand_binary_power2(&pix1, 4).unwrap();
    assert_eq!(expanded4.width(), pix1.width() * 4);

    // Expand binary replicate with different x/y factors
    let repl = expand_binary_replicate(&pix1, 3, 2).unwrap();
    assert_eq!(repl.width(), pix1.width() * 3);
    assert_eq!(repl.height(), pix1.height() * 2);
}

/// Test make_subsample_tab_2x.
#[test]
fn expand_reg_subsample_tab() {
    use leptonica::transform::binexpand::make_subsample_tab_2x;

    let tab = make_subsample_tab_2x(1).unwrap();
    assert_eq!(tab.len(), 256);

    // All zeros → all zeros
    assert_eq!(tab[0], 0);
    // All ones (0xFF) → level 1 should give all ones output (0x0F = 4 bits)
    assert_eq!(tab[0xFF], 0x0F);

    // Invalid level
    assert!(make_subsample_tab_2x(0).is_err());
    assert!(make_subsample_tab_2x(5).is_err());
}

/// Test expand_replicate with clipping (C additional checks).
#[test]
fn expand_reg_clip() {
    let mut rp = RegParams::new("expand_clip");

    let pix = crate::common::load_test_image("speckle.png").expect("load speckle.png");

    // Expand then clip a sub-region (top-left quadrant)
    let pix2x = expand_replicate(&pix, 2).expect("expand 2x");
    let clip_w = pix.width(); // half the expanded width
    let clip_h = pix.height(); // half the expanded height
    let clipped = pix2x.clip_rectangle(0, 0, clip_w, clip_h).expect("clip");
    rp.compare_values(clip_w as f64, clipped.width() as f64, 0.0);
    rp.compare_values(clip_h as f64, clipped.height() as f64, 0.0);

    // Full clip should equal original
    let full_clip = pix2x
        .clip_rectangle(0, 0, pix2x.width(), pix2x.height())
        .expect("full clip");
    rp.compare_pix(&pix2x, &full_clip);

    assert!(rp.cleanup(), "expand clip test failed");
}
