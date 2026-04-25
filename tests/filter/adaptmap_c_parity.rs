//! C-parity tests for the adaptmap pipeline.
//!
//! Reference hashes are FNV-1a `pixel_content_hash` values computed from
//! C leptonica outputs produced by `scripts/verify_*.c`. See plan
//! `docs/plans/028_fill-map-holes-c-alignment.md` (`fill_map_holes`) and
//! plan `docs/plans/029_adaptmap-pipeline-c-alignment.md` (the rest of
//! the pipeline). Tests that have not yet been brought into bit-equivalence
//! are marked `#[ignore = "RED: blocked on plan 029 ..."]` so the
//! assertion lives in the commit history and the corresponding GREEN PR
//! flips RED → GREEN by deleting the `#[ignore]`.
use crate::common::{load_test_image, pixel_content_hash};
use leptonica::filter::adaptmap::{fill_map_holes, get_background_gray_map};
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

/// weasel8 (82x73): asserts bit-equivalence with C `pixFillMapHoles` after
/// the column-major rewrite of `fill_map_holes_inner`. Setup mirrors
/// `tests/filter/adaptmap_reg.rs::adaptmap_reg_fill_map_holes_weasel`.
#[test]
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

// ============================================================================
// Plan 029 — adaptmap pipeline C alignment
// ============================================================================

/// FNV-1a pixel_content_hash of `/tmp/c_bg_gray_map_dreyfus.png` produced by
/// `scripts/verify_bg_gray_map.c` on dreyfus8.png with default options
/// (10x15 tiles, fg_thresh=60, min_count=40). Output is 33x27x8.
///
/// Only PNG inputs are used here — JPEG decoders (Rust `jpeg-decoder` vs
/// C `libjpeg-turbo`) produce sub-pixel differences in the decoded
/// samples, which then propagate into the bg map (e.g. lucasta.150.jpg
/// shows ~0.6% pixel divergence with max delta 3 even after the
/// algorithm is C-aligned). Use lossless PNG fixtures for parity tests.
const EXPECTED_C_BG_GRAY_DREYFUS_HASH: u64 = 0x66d045f59e9a84a8;

/// `pixGetBackgroundGrayMap(pixs, NULL, 10, 15, 60, 40)` on dreyfus8.png.
#[test]
fn c_parity_bg_gray_map_dreyfus() {
    let pix = load_test_image("dreyfus8.png").expect("load dreyfus8");
    let map = get_background_gray_map(&pix, None, 10, 15, 60, 40)
        .expect("get_background_gray_map dreyfus");
    assert_eq!(
        pixel_content_hash(&map),
        EXPECTED_C_BG_GRAY_DREYFUS_HASH,
        "Rust get_background_gray_map(dreyfus8) must match C reference",
    );
}
