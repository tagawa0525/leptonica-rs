//! C-parity test for `fill_map_holes`.
//!
//! Reference hashes derived from C leptonica master (commit f7082ecd) by
//! running `scripts/verify_fillmapholes.c` and feeding `/tmp/c_fillmapholes_*.png`
//! through `tests::common::pixel_content_hash` (FNV-1a). See plan
//! `docs/plans/028_fill-map-holes-c-alignment.md`.
//!
//! The simple 3x3 case is bit-equivalent today. The weasel8 case asserts
//! against the C-aligned hash, but is currently `#[ignore]`'d as **RED**
//! because Rust's `fill_map_holes_inner` still uses 4-neighbor diffusion
//! (~7.3% pixel divergence vs C). The GREEN PR will reimplement
//! `fill_map_holes_inner` in C's column-major style and remove the
//! `#[ignore]`, turning this into a live regression guard.
use crate::common::{load_test_image, pixel_content_hash};
use leptonica::filter::adaptmap::fill_map_holes;
use leptonica::filter::enhance::gamma_trc_masked;
use leptonica::{Pix, PixMut, PixelDepth};

/// FNV-1a pixel_content_hash of `/tmp/c_fillmapholes_simple.png` produced by
/// `scripts/verify_fillmapholes.c`. The image is 3x3x8 with all pixels = 128.
const EXPECTED_C_SIMPLE_HASH: u64 = 0x9ac41e78c2782bfd;

/// FNV-1a pixel_content_hash of `/tmp/c_fillmapholes_weasel.png` produced by
/// `scripts/verify_fillmapholes.c` on the same gamma+holes input as the Rust
/// test below. The image is 82x73x8.
const EXPECTED_C_WEASEL_HASH: u64 = 0x9b960e39a97d0d8b;

/// 3x3 case: input has pixel (1,0)=128 and zeros elsewhere. C's
/// `pixFillMapHoles(pix, 3, 3, L_FILL_BLACK)` propagates 128 to every cell.
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
    assert_eq!(
        pixel_content_hash(&filled),
        EXPECTED_C_SIMPLE_HASH,
        "Rust 3x3 fill_map_holes hash must match C reference",
    );
}

/// weasel8 (82x73): currently RED. The GREEN PR for plan 028 will remove
/// `#[ignore]`. The setup mirrors
/// `tests/filter/adaptmap_reg.rs::adaptmap_reg_fill_map_holes_weasel`.
#[test]
#[ignore = "RED: blocked on plan 028 GREEN PR (fill_map_holes_inner C alignment)"]
fn c_parity_weasel() {
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
    let filled = fill_map_holes(&pix_with_holes, w, h).expect("fill weasel");
    assert_eq!(
        pixel_content_hash(&filled),
        EXPECTED_C_WEASEL_HASH,
        "Rust fill_map_holes(weasel8) hash must match C reference",
    );
}
