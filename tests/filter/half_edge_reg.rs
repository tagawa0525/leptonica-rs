//! Half-edge by bandpass regression test
//!
//! C version: reference/leptonica/prog/enhance_reg.c (extended)
//!
//! Tests half-edge detection using bandpass filtering.
//!
//! C API mapping:
//! - pixHalfEdgeByBandpass -> half_edge_by_bandpass

use crate::common::{RegParams, load_test_image};
use leptonica::filter::half_edge_by_bandpass;
use leptonica::{Pix, PixelDepth};

/// Test half-edge bandpass filter on 8bpp and 32bpp images.
#[test]
fn half_edge_reg() {
    let mut rp = RegParams::new("half_edge");

    let pixs8 = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs8.width();
    let h = pixs8.height();

    // Test with 8bpp, sm1=(1,1), sm2=(3,3)
    let pixd = half_edge_by_bandpass(&pixs8, 1, 1, 3, 3).expect("8bpp bandpass");
    rp.compare_values(w as f64, pixd.width() as f64, 0.0);
    rp.compare_values(h as f64, pixd.height() as f64, 0.0);
    assert_eq!(pixd.depth(), PixelDepth::Bit8);

    // Edge pixels should exist
    let fg = pixd.count_pixels();
    rp.compare_values(1.0, if fg > 0 { 1.0 } else { 0.0 }, 0.0);

    // Test with 32bpp
    let pixs32 = load_test_image("weasel32.png").expect("load weasel32.png");
    let pixd32 = half_edge_by_bandpass(&pixs32, 2, 2, 4, 4).expect("32bpp bandpass");
    rp.compare_values(pixs32.width() as f64, pixd32.width() as f64, 0.0);
    rp.compare_values(pixs32.height() as f64, pixd32.height() as f64, 0.0);
    assert_eq!(pixd32.depth(), PixelDepth::Bit8);

    // Error: sm1 == sm2
    assert!(half_edge_by_bandpass(&pixs8, 2, 2, 2, 2).is_err());

    // Error: reject non-8/32 bpp
    let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    assert!(half_edge_by_bandpass(&pix1, 1, 1, 2, 2).is_err());

    assert!(rp.cleanup(), "half_edge_reg failed");
}
