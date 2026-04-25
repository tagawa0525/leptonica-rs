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
use leptonica::filter::adaptmap::{
    BackgroundNormOptions, background_norm, fill_map_holes, get_background_gray_map,
    get_background_rgb_map, get_inv_background_map,
};
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

/// FNV-1a pixel_content_hash of `/tmp/c_inv_bg_map_dreyfus.png` produced by
/// `scripts/verify_inv_bg_map.c` from the C bg gray map (default
/// BackgroundNormOptions: bg_val=200, smooth_x=2, smooth_y=2).
/// Output is 16bpp 33x27.
const EXPECTED_C_INV_BG_DREYFUS_HASH: u64 = 0x8958471f3a062425;

/// `pixGetInvBackgroundMap(c_bg_map, 200, 2, 2)` on the C-aligned bg map
/// for dreyfus8.
#[test]
fn c_parity_inv_bg_map_dreyfus() {
    // Source the bg map by running the same pipeline as Rust does internally.
    let pix = load_test_image("dreyfus8.png").expect("load dreyfus8");
    let bg_map = get_background_gray_map(&pix, None, 10, 15, 60, 40).expect("bg map dreyfus");
    let inv = get_inv_background_map(&bg_map, 200, 2, 2).expect("get_inv_background_map dreyfus");
    assert_eq!(
        pixel_content_hash(&inv),
        EXPECTED_C_INV_BG_DREYFUS_HASH,
        "Rust get_inv_background_map(dreyfus8) must match C reference",
    );
}

/// FNV-1a pixel_content_hash of `/tmp/c_bg_norm_dreyfus.png` produced by
/// `scripts/verify_apply_inv_bg.c` running C `pixBackgroundNorm` end-to-end
/// with C/Rust default options (10x15 tiles, fg_thresh=60, min_count=40,
/// bg_val=200, smooth_x=2, smooth_y=1). Output is 329x400x8.
const EXPECTED_C_BG_NORM_DREYFUS_HASH: u64 = 0xd2b87b21855eca7a;

/// `pixBackgroundNorm(dreyfus8)` end-to-end. With PR1, PR2, and the
/// already-C-shaped `apply_inv_background_gray_map_inner` loop in
/// place, the full pipeline output should be bit-identical to C.
#[test]
fn c_parity_background_norm_dreyfus() {
    let pix = load_test_image("dreyfus8.png").expect("load dreyfus8");
    let normed =
        background_norm(&pix, &BackgroundNormOptions::default()).expect("background_norm dreyfus");
    assert_eq!(
        pixel_content_hash(&normed),
        EXPECTED_C_BG_NORM_DREYFUS_HASH,
        "Rust pixBackgroundNorm(dreyfus8) must match C reference",
    );
}

/// FNV-1a hashes of `/tmp/c_bg_rgb_map_{r,g,b}_church.png` produced by
/// `scripts/verify_bg_rgb_map.c` running C `pixGetBackgroundRGBMap` on
/// church.png with default options (10x15 tiles, fg_thresh=60,
/// min_count=40). All three maps are 32x17x8.
const EXPECTED_C_BG_RGB_R_CHURCH_HASH: u64 = 0x96ee670141955de6;
const EXPECTED_C_BG_RGB_G_CHURCH_HASH: u64 = 0x51f0f5dcea7a058f;
const EXPECTED_C_BG_RGB_B_CHURCH_HASH: u64 = 0x9f6b7fa22faaf8f4;

/// FNV-1a of `/tmp/c_bg_norm_church.png` from the same verifier:
/// full `pixBackgroundNorm` end-to-end on a 32 bpp RGB input
/// (314x255x32, smooth_x=2, smooth_y=1).
const EXPECTED_C_BG_NORM_CHURCH_HASH: u64 = 0x8618c323b6187384;

/// `pixGetBackgroundRGBMap(church.png, NULL, NULL, 10, 15, 60, 40)`.
/// Asserts bit-equivalence with C — Rust now builds a single shared
/// fg mask from `pixConvertRGBToGrayFast`, matching adaptmap.c:1071.
#[test]
fn c_parity_bg_rgb_map_church() {
    let pix = load_test_image("church.png").expect("load church");
    let (map_r, map_g, map_b) =
        get_background_rgb_map(&pix, None, None, 10, 15, 60, 40).expect("bg rgb map church");
    assert_eq!(
        pixel_content_hash(&map_r),
        EXPECTED_C_BG_RGB_R_CHURCH_HASH,
        "Rust bg_rgb_map(church) R must match C reference",
    );
    assert_eq!(
        pixel_content_hash(&map_g),
        EXPECTED_C_BG_RGB_G_CHURCH_HASH,
        "Rust bg_rgb_map(church) G must match C reference",
    );
    assert_eq!(
        pixel_content_hash(&map_b),
        EXPECTED_C_BG_RGB_B_CHURCH_HASH,
        "Rust bg_rgb_map(church) B must match C reference",
    );
}

/// `pixBackgroundNorm(church.png)` end-to-end on a 32 bpp RGB input.
#[test]
fn c_parity_background_norm_rgb_church() {
    let pix = load_test_image("church.png").expect("load church");
    let normed =
        background_norm(&pix, &BackgroundNormOptions::default()).expect("background_norm church");
    assert_eq!(
        pixel_content_hash(&normed),
        EXPECTED_C_BG_NORM_CHURCH_HASH,
        "Rust pixBackgroundNorm(church) must match C reference",
    );
}

/// Regression: passing a colormapped 8 bpp Pix must produce the same map
/// as passing the same Pix with the colormap already removed via
/// `Pix::remove_colormap(ToGrayscale)`. Dreyfus8 ships colormapped, so
/// loading it directly exercises the auto-decode path inside
/// `get_background_gray_map`. Without that path, palette indices would
/// be treated as gray values and the resulting map would diverge silently.
#[test]
fn bg_gray_map_auto_decodes_colormap() {
    use leptonica::core::pix::RemoveColormapTarget;

    let cmapped = load_test_image("dreyfus8.png").expect("load dreyfus8");
    assert!(
        cmapped.colormap().is_some(),
        "fixture must remain colormapped to exercise the auto-decode path"
    );
    let decoded = cmapped
        .remove_colormap(RemoveColormapTarget::ToGrayscale)
        .expect("remove_colormap");

    let map_via_auto_decode =
        get_background_gray_map(&cmapped, None, 10, 15, 60, 40).expect("auto-decode path");
    let map_via_manual_decode =
        get_background_gray_map(&decoded, None, 10, 15, 60, 40).expect("manual-decode path");

    assert_eq!(
        pixel_content_hash(&map_via_auto_decode),
        pixel_content_hash(&map_via_manual_decode),
        "auto-decode of colormap must yield the same map as pre-decoded input",
    );
}
