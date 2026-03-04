//! Tests for seed spread and select-min-in-conncomp functions
//!
//! # See also
//!
//! C Leptonica: `seedfill.c` — `pixSeedspread`, `pixSelectMinInConnComp`

use leptonica::core::pixel;
use leptonica::io::ImageFormat;
use leptonica::region::ConnectivityType;
use leptonica::{Pix, PixelDepth};

use crate::common::RegParams;

// ============================================================================
// seedspread — C-equivalent regression test (C checks 0–5)
// ============================================================================

/// Paint a 3x3 marker on a 32bpp mutable image at (cx, cy).
///
/// Replaces the C version's `pixRasterop(pixc, x-1, y-1, 3, 3, PIX_SRC, pixsq, 0, 0)`
/// which copies a 3x3 solid-color square onto the destination.
fn paint_marker_3x3(pm: &mut leptonica::PixMut, cx: u32, cy: u32, color: u32) {
    let w = pm.width();
    let h = pm.height();
    let x0 = cx.saturating_sub(1);
    let y0 = cy.saturating_sub(1);
    for dy in 0..3u32 {
        for dx in 0..3u32 {
            let px = x0 + dx;
            let py = y0 + dy;
            if px < w && py < h {
                pm.set_pixel_unchecked(px, py, color);
            }
        }
    }
}

#[test]
fn seedspread_reg() {
    let mut rp = RegParams::new("seedspread");
    // C uses 0x00ff0000 which in leptonica 32bpp RGBA is green
    let marker_color = pixel::compose_rgba(0x00, 0xFF, 0x00, 0x00);

    // --- Moderately dense seeds (100 points) ---
    let pixs = Pix::new(300, 300, PixelDepth::Bit8).unwrap();
    let mut pm = pixs.to_mut();
    let mut seeds_dense: Vec<(u32, u32)> = Vec::new();
    for i in 0u32..100 {
        let x = ((153u64 * (i as u64) * (i as u64) * (i as u64) + 59) % 299) as u32;
        let y = ((117u64 * (i as u64) * (i as u64) * (i as u64) + 241) % 299) as u32;
        let val = (97 * i + 74) % 256;
        pm.set_pixel(x, y, val).unwrap();
        seeds_dense.push((x, y));
    }
    let pixs: Pix = pm.into();

    // Check 0: 4-cc moderately dense
    let pixd = leptonica::region::seedfill::seedspread(&pixs, ConnectivityType::FourWay).unwrap();
    let pixc = pixd.convert_to_32().unwrap();
    let mut pm = pixc.try_into_mut().unwrap();
    for &(x, y) in &seeds_dense {
        paint_marker_3x3(&mut pm, x, y, marker_color);
    }
    let pixc: Pix = pm.into();
    rp.write_pix_and_check(&pixc, ImageFormat::Png)
        .expect("check 0: 4-cc moderately dense");

    // Check 1: 8-cc moderately dense
    let pixd = leptonica::region::seedfill::seedspread(&pixs, ConnectivityType::EightWay).unwrap();
    let pixc = pixd.convert_to_32().unwrap();
    let mut pm = pixc.try_into_mut().unwrap();
    for &(x, y) in &seeds_dense {
        paint_marker_3x3(&mut pm, x, y, marker_color);
    }
    let pixc: Pix = pm.into();
    rp.write_pix_and_check(&pixc, ImageFormat::Png)
        .expect("check 1: 8-cc moderately dense");

    // --- Regular lattice seeds ---
    let pixs = Pix::new(200, 200, PixelDepth::Bit8).unwrap();
    let mut pm = pixs.to_mut();
    let mut seeds_lattice: Vec<(u32, u32)> = Vec::new();
    for i in (5u32..=195).step_by(10) {
        for j in (5u32..=195).step_by(10) {
            pm.set_pixel(i, j, (7 * i + 17 * j) % 255).unwrap();
            seeds_lattice.push((i, j));
        }
    }
    let pixs: Pix = pm.into();

    // Check 2: 4-cc lattice
    // NOTE: C version uses pixRasterop(pixc, j-1, i-1, ...) meaning (x=j, y=i)
    let pixd = leptonica::region::seedfill::seedspread(&pixs, ConnectivityType::FourWay).unwrap();
    let pixc = pixd.convert_to_32().unwrap();
    let mut pm = pixc.try_into_mut().unwrap();
    for &(i, j) in &seeds_lattice {
        paint_marker_3x3(&mut pm, j, i, marker_color);
    }
    let pixc: Pix = pm.into();
    rp.write_pix_and_check(&pixc, ImageFormat::Png)
        .expect("check 2: 4-cc lattice");

    // Check 3: 8-cc lattice
    let pixd = leptonica::region::seedfill::seedspread(&pixs, ConnectivityType::EightWay).unwrap();
    let pixc = pixd.convert_to_32().unwrap();
    let mut pm = pixc.try_into_mut().unwrap();
    for &(i, j) in &seeds_lattice {
        paint_marker_3x3(&mut pm, j, i, marker_color);
    }
    let pixc: Pix = pm.into();
    rp.write_pix_and_check(&pixc, ImageFormat::Png)
        .expect("check 3: 8-cc lattice");

    // --- Very sparse seeds (4 points) ---
    let pixs = Pix::new(200, 200, PixelDepth::Bit8).unwrap();
    let mut pm = pixs.to_mut();
    pm.set_pixel(60, 20, 90).unwrap();
    pm.set_pixel(160, 40, 130).unwrap();
    pm.set_pixel(80, 80, 205).unwrap();
    pm.set_pixel(40, 160, 115).unwrap();
    let pixs: Pix = pm.into();
    let sparse_seeds = [(60u32, 20u32), (160, 40), (80, 80), (40, 160)];

    // Check 4: 4-cc sparse
    let pixd = leptonica::region::seedfill::seedspread(&pixs, ConnectivityType::FourWay).unwrap();
    let pixc = pixd.convert_to_32().unwrap();
    let mut pm = pixc.try_into_mut().unwrap();
    for &(x, y) in &sparse_seeds {
        paint_marker_3x3(&mut pm, x, y, marker_color);
    }
    let pixc: Pix = pm.into();
    rp.write_pix_and_check(&pixc, ImageFormat::Png)
        .expect("check 4: 4-cc sparse");

    // Check 5: 8-cc sparse
    let pixd = leptonica::region::seedfill::seedspread(&pixs, ConnectivityType::EightWay).unwrap();
    let pixc = pixd.convert_to_32().unwrap();
    let mut pm = pixc.try_into_mut().unwrap();
    for &(x, y) in &sparse_seeds {
        paint_marker_3x3(&mut pm, x, y, marker_color);
    }
    let pixc: Pix = pm.into();
    rp.write_pix_and_check(&pixc, ImageFormat::Png)
        .expect("check 5: 8-cc sparse");

    assert!(rp.cleanup(), "seedspread_reg regression test failed");
}

// ============================================================================
// seedspread — unit tests
// ============================================================================

#[test]
fn test_seedspread_single_seed_4conn() {
    // 8bpp 20x20, single seed at (10, 10) with value 100
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel(10, 10, 100).unwrap();
    let pix: Pix = pm.into();

    let result = leptonica::region::seedfill::seedspread(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(result.width(), 20);
    assert_eq!(result.height(), 20);
    assert_eq!(result.depth(), PixelDepth::Bit8);
    // All pixels should be filled with the seed value 100
    assert_eq!(result.get_pixel(0, 0).unwrap(), 100);
    assert_eq!(result.get_pixel(19, 19).unwrap(), 100);
    assert_eq!(result.get_pixel(10, 10).unwrap(), 100);
}

#[test]
fn test_seedspread_single_seed_8conn() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel(10, 10, 200).unwrap();
    let pix: Pix = pm.into();

    let result = leptonica::region::seedfill::seedspread(&pix, ConnectivityType::EightWay).unwrap();
    // All pixels should be 200
    assert_eq!(result.get_pixel(0, 0).unwrap(), 200);
    assert_eq!(result.get_pixel(19, 19).unwrap(), 200);
}

#[test]
fn test_seedspread_two_seeds_voronoi() {
    // Two seeds: left side (5, 10) = 50, right side (15, 10) = 150
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel(5, 10, 50).unwrap();
    pm.set_pixel(15, 10, 150).unwrap();
    let pix: Pix = pm.into();

    let result = leptonica::region::seedfill::seedspread(&pix, ConnectivityType::FourWay).unwrap();
    // Pixels near left seed should have value 50
    assert_eq!(result.get_pixel(0, 10).unwrap(), 50);
    assert_eq!(result.get_pixel(5, 10).unwrap(), 50);
    // Pixels near right seed should have value 150
    assert_eq!(result.get_pixel(19, 10).unwrap(), 150);
    assert_eq!(result.get_pixel(15, 10).unwrap(), 150);
    // The boundary between them should be somewhere around x=10
    // (exact position depends on algorithm tie-breaking)
}

#[test]
fn test_seedspread_multiple_seeds() {
    // 4 seeds in corners
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel(2, 2, 10).unwrap();
    pm.set_pixel(17, 2, 20).unwrap();
    pm.set_pixel(2, 17, 30).unwrap();
    pm.set_pixel(17, 17, 40).unwrap();
    let pix: Pix = pm.into();

    let result = leptonica::region::seedfill::seedspread(&pix, ConnectivityType::EightWay).unwrap();
    // Each corner should retain its seed value
    assert_eq!(result.get_pixel(2, 2).unwrap(), 10);
    assert_eq!(result.get_pixel(17, 2).unwrap(), 20);
    assert_eq!(result.get_pixel(2, 17).unwrap(), 30);
    assert_eq!(result.get_pixel(17, 17).unwrap(), 40);
    // Every pixel should be non-zero (all filled)
    for y in 0..20 {
        for x in 0..20 {
            assert_ne!(result.get_pixel(x, y).unwrap(), 0);
        }
    }
}

#[test]
fn test_seedspread_rejects_non_8bpp() {
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    assert!(leptonica::region::seedfill::seedspread(&pix, ConnectivityType::FourWay).is_err());
}

// ============================================================================
// select_min_in_conncomp
// ============================================================================

#[test]
fn test_select_min_in_conncomp_basic() {
    // 8bpp gradient image 20x20
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for y in 0..20u32 {
        for x in 0..20u32 {
            pm.set_pixel_unchecked(x, y, 100 + x + y);
        }
    }
    let pixs: Pix = pm.into();

    // 1bpp mask: two separate components
    // Component 1: 5x5 block at (2,2)
    // Component 2: 5x5 block at (12,12)
    let mask = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let mut mm = mask.to_mut();
    for y in 2..7u32 {
        for x in 2..7u32 {
            mm.set_pixel_unchecked(x, y, 1);
        }
    }
    for y in 12..17u32 {
        for x in 12..17u32 {
            mm.set_pixel_unchecked(x, y, 1);
        }
    }
    let mask: Pix = mm.into();

    let (pta, values) = leptonica::region::seedfill::select_min_in_conncomp(&pixs, &mask).unwrap();

    // Should find 2 components
    assert_eq!(pta.len(), 2);
    assert_eq!(values.len(), 2);

    // Component 1 min at (2,2) = 100+2+2 = 104
    // Component 2 min at (12,12) = 100+12+12 = 124
    // Values should be 104 and 124 (order may vary)
    let mut vals: Vec<f32> = (0..values.len()).map(|i| values.get(i).unwrap()).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(vals[0] as u32, 104);
    assert_eq!(vals[1] as u32, 124);
}

#[test]
fn test_select_min_in_conncomp_single_pixel() {
    // Uniform 8bpp image
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for y in 0..10u32 {
        for x in 0..10u32 {
            pm.set_pixel_unchecked(x, y, 200);
        }
    }
    let pixs: Pix = pm.into();

    // Single pixel component
    let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let mut mm = mask.to_mut();
    mm.set_pixel_unchecked(5, 5, 1);
    let mask: Pix = mm.into();

    let (pta, values) = leptonica::region::seedfill::select_min_in_conncomp(&pixs, &mask).unwrap();
    assert_eq!(pta.len(), 1);
    let (px, py) = pta.get(0).unwrap();
    assert_eq!(px as u32, 5);
    assert_eq!(py as u32, 5);
    assert_eq!(values.get(0).unwrap() as u32, 200);
}

#[test]
fn test_select_min_in_conncomp_dimension_mismatch() {
    let pixs = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let pixm = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    assert!(leptonica::region::seedfill::select_min_in_conncomp(&pixs, &pixm).is_err());
}
