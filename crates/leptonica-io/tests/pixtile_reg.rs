//! Pixel tiling regression test
//!
//! Tests the PIXTILING structure for tile-based image processing
//! with overlap support. The C version creates tiling configurations,
//! extracts tiles, paints them back, and verifies round-trip fidelity.
//!
//! PIXTILING is not implemented in Rust. This file documents the
//! C test structure. Basic image clipping (the foundation of tiling)
//! is tested as a partial substitute.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pixtile_reg.c`

use leptonica_test::RegParams;

/// Test basic image clipping as partial substitute for tiling.
///
/// Extracts non-overlapping sub-regions and verifies dimensions,
/// simulating the simplest tiling operation.
#[test]
fn pixtile_reg_basic_clip() {
    let mut rp = RegParams::new("pixtile_clip");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    let w = pix.width();
    let h = pix.height();

    // Clip top-left quadrant
    let tile_w = w / 2;
    let tile_h = h / 2;
    let tile = pix.clip_rectangle(0, 0, tile_w, tile_h).expect("clip");
    rp.compare_values(tile_w as f64, tile.width() as f64, 0.0);
    rp.compare_values(tile_h as f64, tile.height() as f64, 0.0);

    // Verify pixel values match between source and clipped tile
    let p_orig = pix.get_pixel(0, 0).expect("get_pixel origin");
    let p_tile = tile.get_pixel(0, 0).expect("get_pixel tile origin");
    rp.compare_values(p_orig as f64, p_tile as f64, 0.0);
    // Check a point near the tile boundary
    let mid_x = tile_w.min(tile.width()) - 1;
    let mid_y = tile_h.min(tile.height()) - 1;
    let p_orig_mid = pix.get_pixel(mid_x, mid_y).expect("get_pixel mid");
    let p_tile_mid = tile.get_pixel(mid_x, mid_y).expect("get_pixel tile mid");
    rp.compare_values(p_orig_mid as f64, p_tile_mid as f64, 0.0);

    // Clip bottom-right quadrant
    let tile_br = pix
        .clip_rectangle(tile_w, tile_h, w - tile_w, h - tile_h)
        .expect("clip br");
    rp.compare_values((w - tile_w) as f64, tile_br.width() as f64, 0.0);
    rp.compare_values((h - tile_h) as f64, tile_br.height() as f64, 0.0);

    // Verify pixel at (0,0) of bottom-right tile matches source at (tile_w, tile_h)
    let p_orig_br = pix.get_pixel(tile_w, tile_h).expect("get_pixel br");
    let p_tile_br = tile_br.get_pixel(0, 0).expect("get_pixel tile br");
    rp.compare_values(p_orig_br as f64, p_tile_br as f64, 0.0);

    assert!(rp.cleanup(), "pixtile basic clip test failed");
}

/// Test PIXTILING create and iterate (C checks 0-7).
///
/// Requires pixTilingCreate, pixTilingGetTile, pixTilingPaintTile,
/// pixTilingGetCount, pixTilingGetSize.
#[test]
#[ignore = "not yet implemented: PIXTILING type not available"]
fn pixtile_reg_tiling_roundtrip() {
    // C version TestTiling configurations:
    // 1. (0, 0, 0, 0, 0, 0) - 1x1, no tiling
    // 2. (0, 1, 0, 0, 0, 0) - single column
    // 3. (1, 0, 0, 0, 0, 0) - single row
    // 4. (1, 1, 0, 0, 0, 0) - 1x1 explicit
    // 5. (2, 3, 0, 0, 0, 0) - 2x3 grid, no overlap
    // 6. (7, 9, 0, 0, 0, 0) - 7x9 grid, no overlap
    // 7. (7, 9, 0, 0, 35, 40) - 7x9 grid, with overlap
    // 8. (0, 0, 200, 300, 0, 0) - by tile size
    // For each: extract all tiles, paint back, verify equals original
}

/// Test PIXTILING with overlapping tiles (C additional checks).
///
/// Requires pixTilingCreate with xoverlap/yoverlap parameters.
#[test]
#[ignore = "not yet implemented: PIXTILING overlap support not available"]
fn pixtile_reg_overlap() {
    // C version:
    // 1. Create tiling with 35px x-overlap and 40px y-overlap
    // 2. Extract overlapping tiles
    // 3. Paint back with overlap handling
    // 4. Verify reconstruction equals original
}
