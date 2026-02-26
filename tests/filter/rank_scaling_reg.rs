//! Rank filter with scaling regression test
//!
//! C version: reference/leptonica/prog/rank_reg.c (extended)
//!
//! Tests accelerated rank filtering via downscaling.
//!
//! C API mapping:
//! - pixRankFilterWithScaling -> rank_filter_with_scaling

use crate::common::{RegParams, load_test_image};
use leptonica::filter::rank_filter_with_scaling;
use leptonica::{Pix, PixelDepth};

/// Test rank filter with scaling on 8bpp and 32bpp images.
#[test]
fn rank_scaling_reg() {
    let mut rp = RegParams::new("rank_scaling");

    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");
    let pixs = pixs.convert_to_8().expect("convert to 8bpp");
    let w = pixs.width();
    let h = pixs.height();

    // Median filter with scaling
    let pixd =
        rank_filter_with_scaling(&pixs, 7, 7, 0.5, 0.5).expect("8bpp rank filter with scaling");
    rp.compare_values(w as f64, pixd.width() as f64, 0.0);
    rp.compare_values(h as f64, pixd.height() as f64, 0.0);
    assert_eq!(pixd.depth(), PixelDepth::Bit8);

    // Min filter (rank=0.0) with scaling
    let pixd_min =
        rank_filter_with_scaling(&pixs, 5, 5, 0.0, 0.3).expect("8bpp min filter with scaling");
    rp.compare_values(w as f64, pixd_min.width() as f64, 0.0);

    // Max filter (rank=1.0) with scaling
    let pixd_max =
        rank_filter_with_scaling(&pixs, 5, 5, 1.0, 0.3).expect("8bpp max filter with scaling");
    rp.compare_values(w as f64, pixd_max.width() as f64, 0.0);

    // 1x1 filter should be a no-op
    let pixd_noop = rank_filter_with_scaling(&pixs, 1, 1, 0.5, 0.5).expect("1x1 no-op");
    rp.compare_values(w as f64, pixd_noop.width() as f64, 0.0);

    // Error: colormap
    // (skip - we'd need to create a colormapped image)

    // Error: invalid rank
    assert!(rank_filter_with_scaling(&pixs, 5, 5, -0.1, 0.5).is_err());
    assert!(rank_filter_with_scaling(&pixs, 5, 5, 1.1, 0.5).is_err());

    // Error: invalid dimensions
    assert!(rank_filter_with_scaling(&pixs, 0, 5, 0.5, 0.5).is_err());

    // Error: unsupported depth
    let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    assert!(rank_filter_with_scaling(&pix1, 3, 3, 0.5, 0.5).is_err());

    assert!(rp.cleanup(), "rank_scaling_reg failed");
}
