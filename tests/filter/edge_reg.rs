//! Edge detection regression test
//!
//! C version: prog/edge_reg.c
//!
//! Tests Sobel edge filter, Laplacian edge detection, sharpening,
//! unsharp masking, and emboss operations.
//!
//! C API mapping:
//! - pixSobelEdgeFilter(pixs, L_HORIZONTAL_EDGES) -> sobel_edge(Horizontal)
//! - pixSobelEdgeFilter(pixs, L_VERTICAL_EDGES) -> sobel_edge(Vertical)
//! - pixThresholdToBinary -> threshold_to_binary
//! - pixOr -> Pix::or
//! - Custom Laplacian convolution -> laplacian_edge
//! - Sharpening kernel -> sharpen
//! - Unsharp mask -> unsharp_mask
//! - Emboss kernel -> emboss

use crate::common::{RegParams, load_test_image};
use leptonica::color::threshold_to_binary;
use leptonica::filter::{
    EdgeOrientation, emboss, laplacian_edge, sharpen, sobel_edge, unsharp_mask,
};
use leptonica::io::ImageFormat;

/// Test edge detection and enhancement operations.
///
/// C: edge_reg.c tests Sobel horizontal/vertical edges, combined edges,
/// both 1bpp (thresholded) and 8bpp output.
#[test]
fn edge_reg() {
    let mut rp = RegParams::new("edge");

    // C version uses test8.jpg
    let pixs = load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // --- C test 0: Horizontal Sobel → threshold → invert → write ---
    let pix_h = sobel_edge(&pixs, EdgeOrientation::Horizontal).expect("sobel horiz");
    let pix_h_bin = threshold_to_binary(&pix_h, 60).expect("threshold horiz");
    let pix_h_inv = pix_h_bin.invert();
    rp.write_pix_and_check(&pix_h_inv, ImageFormat::Png)
        .expect("write horiz edges");

    // --- C test 1: Vertical Sobel → threshold → invert → write ---
    let pix_v = sobel_edge(&pixs, EdgeOrientation::Vertical).expect("sobel vert");
    let pix_v_bin = threshold_to_binary(&pix_v, 60).expect("threshold vert");
    let pix_v_inv = pix_v_bin.invert();
    rp.write_pix_and_check(&pix_v_inv, ImageFormat::Png)
        .expect("write vert edges");

    // --- C test 2: OR of horizontal and vertical edges ---
    let pix_combined = pix_v_inv.or(&pix_h_inv).expect("or edges");
    rp.write_pix_and_check(&pix_combined, ImageFormat::Png)
        .expect("write combined edges");

    // --- C test 3: 8bpp max of horizontal and vertical Sobel → invert ---
    // C uses pixMinOrMax(pix1, pix1, pix3, L_CHOOSE_MAX) then pixInvert.
    // Rust: compute per-pixel max of the two 8bpp Sobel edge images, then invert.
    let pix_8bpp_max = pixel_max(&pix_h, &pix_v);
    let pix_8bpp_inv = pix_8bpp_max.invert();
    rp.write_pix_and_check(&pix_8bpp_inv, ImageFormat::Jpeg)
        .expect("write 8bpp edges");

    // --- Additional Rust tests: operations beyond C version ---

    // Laplacian edge detection
    let lap = laplacian_edge(&pixs).expect("laplacian_edge");
    rp.compare_values(w as f64, lap.width() as f64, 0.0);
    rp.compare_values(h as f64, lap.height() as f64, 0.0);

    // Sharpen
    let sharp = sharpen(&pixs).expect("sharpen");
    rp.compare_values(w as f64, sharp.width() as f64, 0.0);
    rp.compare_values(h as f64, sharp.height() as f64, 0.0);

    // Unsharp mask
    let unsharp = unsharp_mask(&pixs, 2, 1.5).expect("unsharp_mask");
    rp.compare_values(w as f64, unsharp.width() as f64, 0.0);
    rp.compare_values(h as f64, unsharp.height() as f64, 0.0);

    // Emboss
    let emb = emboss(&pixs).expect("emboss");
    rp.compare_values(w as f64, emb.width() as f64, 0.0);
    rp.compare_values(h as f64, emb.height() as f64, 0.0);

    // Edge detection should produce non-zero output
    let edge_fg = lap.count_pixels();
    rp.compare_values(1.0, if edge_fg > 0 { 1.0 } else { 0.0 }, 0.0);

    // Edge functions require 8bpp, test error on 32bpp
    let pix32 = load_test_image("weasel32.png").expect("load 32bpp");
    let result32 = sobel_edge(&pix32, EdgeOrientation::Horizontal);
    rp.compare_values(1.0, if result32.is_err() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "edge regression test failed");
}

/// Compute per-pixel maximum of two 8bpp images.
/// Equivalent to C: pixMinOrMax(NULL, pix1, pix2, L_CHOOSE_MAX)
fn pixel_max(pix1: &leptonica::Pix, pix2: &leptonica::Pix) -> leptonica::Pix {
    let w = pix1.width();
    let h = pix1.height();
    let out = leptonica::Pix::new(w, h, leptonica::PixelDepth::Bit8).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let v1 = pix1.get_pixel_unchecked(x, y);
            let v2 = pix2.get_pixel_unchecked(x, y);
            out_mut.set_pixel_unchecked(x, y, v1.max(v2));
        }
    }
    out_mut.into()
}
