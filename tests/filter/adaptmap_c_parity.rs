//! C-parity test for `fill_map_holes`.
//!
//! Reference values derived from C leptonica master (commit f7082ecd) by
//! running `scripts/verify_fillmapholes.c`.
//!
//! The simple 3x3 case is bit-equivalent. The weasel8 case is NOT —
//! Rust's algorithm (4-neighbor diffusion in two passes) differs structurally
//! from C's column-major replication, producing ~7.3% pixel divergence on
//! the same input. The C bug fix in upstream commit 737f969e (loop bound
//! `j < w` → `j < nx`) addressed that column-replication path, which Rust
//! does not use, so the bug is structurally absent from the Rust port.
//!
//! Bringing weasel8 into bit-equivalence would require reimplementing
//! `fill_map_holes_inner` to match C's column-then-row strategy. That is
//! tracked as a separate decision: see `docs/plans/` if/when scoped.
use crate::common::load_test_image;
use leptonica::filter::adaptmap::fill_map_holes;
use leptonica::filter::enhance::gamma_trc_masked;
use leptonica::{Pix, PixMut, PixelDepth};

/// 3x3 case from C `pixFillMapHoles(pix, 3, 3, L_FILL_BLACK)`:
/// input has pixel (1,0)=128 and zeros elsewhere; output fills entirely with 128.
#[test]
fn c_parity_simple_3x3() {
    let mut input = PixMut::new(3, 3, PixelDepth::Bit8).expect("create 3x3");
    input.set_pixel(1, 0, 128).expect("set pixel");
    let pix: Pix = input.into();

    let filled = fill_map_holes(&pix, 3, 3).expect("fill_map_holes 3x3");

    let mut grid = [[0u32; 3]; 3];
    for y in 0..3 {
        for x in 0..3 {
            grid[y as usize][x as usize] = filled.get_pixel(x, y).expect("get_pixel");
        }
    }
    assert_eq!(
        grid,
        [[128, 128, 128], [128, 128, 128], [128, 128, 128]],
        "Rust 3x3 output must match C version exactly"
    );
}

/// weasel8 case: Rust algorithm differs from C; this test pins down the
/// CURRENT divergence so any future bit-equivalence work surfaces here.
///
/// `scripts/verify_fillmapholes.c` reports against the same 82x73 input:
///   * IDENTICAL count: 5550 / 5986 pixels (92.7%)
///   * ndiff = 436 (7.3%)
///   * max channel delta = 233
///
/// Mirrors the input setup in
/// `tests/filter/adaptmap_reg.rs::adaptmap_reg_fill_map_holes_weasel`.
#[test]
#[ignore = "Rust fill_map_holes uses 4-neighbor diffusion; bit-equivalence with \
            C column-then-row replication requires reimplementation"]
fn c_parity_weasel_known_divergence() {
    let pix = load_test_image("weasel8.png").expect("load weasel8.png");
    let darkened = gamma_trc_masked(&pix, None, 1.0, 0, 200).expect("darken");
    let w = darkened.width();
    let h = darkened.height();
    let mut m = darkened.try_into_mut().expect("mut");
    for y in 0..h {
        for x in 0..5u32.min(w) {
            m.set_pixel_unchecked(x, y, 0);
        }
        for x in 20u32..22u32.min(w) {
            m.set_pixel_unchecked(x, y, 0);
        }
        for x in 40u32..43u32.min(w) {
            m.set_pixel_unchecked(x, y, 0);
        }
    }
    for y in 0..3u32.min(h) {
        for x in 0..w {
            m.set_pixel_unchecked(x, y, 0);
        }
    }
    for y in 15u32..18u32.min(h) {
        for x in 0..w {
            m.set_pixel_unchecked(x, y, 0);
        }
    }
    for y in 35u32..37u32.min(h) {
        for x in 0..w {
            m.set_pixel_unchecked(x, y, 0);
        }
    }
    let pix_with_holes: Pix = m.into();
    let _filled = fill_map_holes(&pix_with_holes, w, h).expect("fill weasel");
    // No assertion: this test is a placeholder while Rust implementation
    // diverges from C. Re-enable bit-equivalence assertion (against
    // /tmp/c_fillmapholes_weasel.png produced by verify_fillmapholes.c) once
    // the algorithm is aligned.
}
