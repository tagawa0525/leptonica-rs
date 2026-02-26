//! Region coverage regression tests
//!
//! Tests for 25 region functions corresponding to C Leptonica:
//! - conncomp.c: seedfill_bb, seedfill_4_bb, seedfill_8_bb, seedfill, seedfill_4, seedfill_8
//! - ccbord.c: pix_get_cc_borders, get_outer_border, pix_get_hole_border,
//!   locate_outside_seed_pixel, generate_global_locs, generate_step_chains,
//!   step_chains_to_pix_coords, generate_sp_global_locs, generate_single_path,
//!   get_cut_path_for_hole, write/read file, write/read stream, write_svg file/string
//! - pixlabel.c: pix_conn_comp_incr_init, pix_conn_comp_incr_add, pix_loc_to_color_transform

use leptonica::region::ccbord::{
    ImageBorders, get_all_borders, get_cut_path_for_hole, get_outer_border,
    locate_outside_seed_pixel, pix_get_hole_border,
};
use leptonica::region::conncomp::{
    ConnectivityType, seedfill, seedfill_4, seedfill_4_bb, seedfill_8, seedfill_8_bb, seedfill_bb,
};
use leptonica::region::label::{
    pix_conn_comp_incr_add, pix_conn_comp_incr_init, pix_loc_to_color_transform,
};
use leptonica::{Pix, PixelDepth};

/// Helper: create binary image with specific pixels
fn make_binary(w: u32, h: u32, pixels: &[(u32, u32)]) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for &(x, y) in pixels {
        pm.set_pixel_unchecked(x, y, 1);
    }
    pm.into()
}

/// Helper: create binary rect
fn make_rect(w: u32, h: u32, x0: u32, y0: u32, x1: u32, y1: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in y0..y1 {
        for x in x0..x1 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    pm.into()
}

/// Helper: create ring (hollow square) for hole tests
fn make_ring(w: u32, h: u32, x0: u32, y0: u32, x1: u32, y1: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in y0..y1 {
        for x in x0..x1 {
            if y == y0 || y == y1 - 1 || x == x0 || x == x1 - 1 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
    }
    pm.into()
}

// ============================================================================
// 1. seedfill_bb - Binary seedfill with bounding box (4+8 connectivity)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_bb_4conn() {
    let pix = make_rect(20, 20, 2, 3, 7, 8);
    let mut pm = pix.try_into_mut().unwrap();
    let bbox = seedfill_bb(&mut pm, 4, 5, ConnectivityType::FourWay).unwrap();
    assert_eq!((bbox.x, bbox.y, bbox.w, bbox.h), (2, 3, 5, 5));
    // All component pixels should be cleared
    for y in 3..8 {
        for x in 2..7 {
            assert_eq!(pm.get_pixel(x, y), Some(0));
        }
    }
}

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_bb_8conn() {
    // Diagonal pixels: only connected in 8-way
    let pix = make_binary(10, 10, &[(2, 2), (3, 3), (4, 4)]);
    let mut pm = pix.try_into_mut().unwrap();
    let bbox = seedfill_bb(&mut pm, 2, 2, ConnectivityType::EightWay).unwrap();
    assert_eq!(bbox.w, 3);
    assert_eq!(bbox.h, 3);
    assert_eq!(pm.get_pixel(3, 3), Some(0));
    assert_eq!(pm.get_pixel(4, 4), Some(0));
}

// ============================================================================
// 2. seedfill_4_bb - 4-connected seedfill with bounding box
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_4_bb_basic() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let mut pm = pix.try_into_mut().unwrap();
    let bbox = seedfill_4_bb(&mut pm, 3, 3).unwrap();
    assert_eq!((bbox.x, bbox.y, bbox.w, bbox.h), (1, 1, 4, 4));
}

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_4_bb_diagonal_not_connected() {
    // Diagonal pixels should be separate in 4-connectivity
    let pix = make_binary(10, 10, &[(2, 2), (3, 3)]);
    let mut pm = pix.try_into_mut().unwrap();
    let bbox = seedfill_4_bb(&mut pm, 2, 2).unwrap();
    assert_eq!((bbox.w, bbox.h), (1, 1));
    // (3,3) should still be ON
    assert_eq!(pm.get_pixel(3, 3), Some(1));
}

// ============================================================================
// 3. seedfill_8_bb - 8-connected seedfill with bounding box
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_8_bb_diagonal() {
    let pix = make_binary(10, 10, &[(2, 2), (3, 3)]);
    let mut pm = pix.try_into_mut().unwrap();
    let bbox = seedfill_8_bb(&mut pm, 2, 2).unwrap();
    assert_eq!((bbox.w, bbox.h), (2, 2));
    assert_eq!(pm.get_pixel(3, 3), Some(0));
}

// ============================================================================
// 4. seedfill - Binary seedfill in-place (no bounding box)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_clears_component() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let mut pm = pix.try_into_mut().unwrap();
    seedfill(&mut pm, 3, 3, ConnectivityType::FourWay).unwrap();
    // All component pixels should be cleared
    for y in 1..5 {
        for x in 1..5 {
            assert_eq!(pm.get_pixel(x, y), Some(0), "pixel ({x},{y}) not cleared");
        }
    }
}

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_preserves_other_components() {
    // Two separate rects
    let pix = make_binary(
        20,
        10,
        &[
            (1, 1),
            (2, 1),
            (1, 2),
            (2, 2), // component A
            (10, 1),
            (11, 1),
            (10, 2),
            (11, 2), // component B
        ],
    );
    let mut pm = pix.try_into_mut().unwrap();
    seedfill(&mut pm, 1, 1, ConnectivityType::FourWay).unwrap();
    // A is cleared
    assert_eq!(pm.get_pixel(1, 1), Some(0));
    assert_eq!(pm.get_pixel(2, 2), Some(0));
    // B is preserved
    assert_eq!(pm.get_pixel(10, 1), Some(1));
    assert_eq!(pm.get_pixel(11, 2), Some(1));
}

// ============================================================================
// 5. seedfill_4 - 4-connected in-place seedfill
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_4_basic() {
    let pix = make_rect(10, 10, 2, 2, 6, 6);
    let mut pm = pix.try_into_mut().unwrap();
    seedfill_4(&mut pm, 4, 4).unwrap();
    for y in 2..6 {
        for x in 2..6 {
            assert_eq!(pm.get_pixel(x, y), Some(0));
        }
    }
}

// ============================================================================
// 6. seedfill_8 - 8-connected in-place seedfill
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_8_basic() {
    let pix = make_binary(10, 10, &[(2, 2), (3, 3), (4, 4)]);
    let mut pm = pix.try_into_mut().unwrap();
    seedfill_8(&mut pm, 2, 2).unwrap();
    assert_eq!(pm.get_pixel(2, 2), Some(0));
    assert_eq!(pm.get_pixel(3, 3), Some(0));
    assert_eq!(pm.get_pixel(4, 4), Some(0));
}

#[test]
#[ignore = "not yet implemented"]
fn test_seedfill_8_does_not_clear_4conn_separate() {
    // Diagonal: seedfill_4 should NOT clear, seedfill_8 should
    let pix = make_binary(10, 10, &[(2, 2), (3, 3)]);
    let mut pm4 = pix.clone().try_into_mut().unwrap();
    seedfill_4(&mut pm4, 2, 2).unwrap();
    assert_eq!(pm4.get_pixel(3, 3), Some(1)); // not cleared by 4-conn

    let mut pm8 = pix.try_into_mut().unwrap();
    seedfill_8(&mut pm8, 2, 2).unwrap();
    assert_eq!(pm8.get_pixel(3, 3), Some(0)); // cleared by 8-conn
}

// ============================================================================
// 7. pixGetCCBorders - get_all_borders (already exists, needs test coverage)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_get_all_borders_two_components() {
    // Two separate rects
    let mut pixels = Vec::new();
    for y in 1..4 {
        for x in 1..4 {
            pixels.push((x, y));
        }
    }
    for y in 6..9 {
        for x in 6..9 {
            pixels.push((x, y));
        }
    }
    let pix = make_binary(12, 12, &pixels);
    let borders = get_all_borders(&pix).unwrap();
    assert_eq!(borders.component_count(), 2);
    for comp in &borders.components {
        assert!(!comp.outer.is_empty());
    }
}

// ============================================================================
// 8. pixGetOuterBorder - get_outer_border (already exists, needs test coverage)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_get_outer_border_l_shape() {
    // L-shaped component
    let pix = make_binary(10, 10, &[(1, 1), (1, 2), (1, 3), (2, 3), (3, 3)]);
    let border = get_outer_border(&pix, None).unwrap();
    assert!(!border.is_empty());
    assert!(border.len() >= 5);
}

// ============================================================================
// 9. pixGetHoleBorder - trace hole border
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_pix_get_hole_border() {
    // Ring shape: 7x7 frame
    let pix = make_ring(10, 10, 1, 1, 8, 8);
    // Get hole interior bounds: (2,2)-(6,6)
    let hole_bounds = leptonica::core::Box::new_unchecked(2, 2, 5, 5);
    let border = pix_get_hole_border(&pix, &hole_bounds).unwrap();
    assert!(!border.is_empty());
}

// ============================================================================
// 10. locateOutsideSeedPixel
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_locate_outside_seed_pixel() {
    let pix = make_ring(10, 10, 1, 1, 8, 8);
    let hole_bounds = leptonica::core::Box::new_unchecked(2, 2, 5, 5);
    let seed = locate_outside_seed_pixel(&pix, &hole_bounds);
    assert!(seed.is_some());
    let pt = seed.unwrap();
    // Seed should be a foreground pixel adjacent to the hole
    assert!(pt.x >= 0 && pt.y >= 0);
}

// ============================================================================
// 11. ccbaGenerateGlobalLocs
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_generate_global_locs() {
    let pix = make_rect(20, 20, 5, 5, 10, 10);
    let mut borders = get_all_borders(&pix).unwrap();
    borders.generate_global_locs();
    let comp = &borders.components[0];
    // Global outer should have points offset by bounding box
    let global = comp.global_outer.as_ref().unwrap();
    assert!(!global.is_empty());
    // All global points should be within image bounds
    for p in &global.points {
        assert!(p.x >= 0 && p.x < 20);
        assert!(p.y >= 0 && p.y < 20);
    }
}

// ============================================================================
// 12. ccbaGenerateStepChains (already exists)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_generate_step_chains_coverage() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let mut borders = get_all_borders(&pix).unwrap();
    borders.generate_step_chains();
    let comp = &borders.components[0];
    assert!(comp.outer.chain_code.is_some());
    let chain = comp.outer.chain_code.as_ref().unwrap();
    assert!(!chain.is_empty());
}

// ============================================================================
// 13. ccbaStepChainsToPixCoords (already exists)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_step_chains_to_pix_coords_coverage() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let mut borders = get_all_borders(&pix).unwrap();
    let original = borders.components[0].outer.points.clone();
    borders.generate_step_chains();
    borders.step_chains_to_pix_coords().unwrap();
    assert_eq!(borders.components[0].outer.points, original);
}

// ============================================================================
// 14. ccbaGenerateSPGlobalLocs
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_generate_sp_global_locs() {
    let pix = make_rect(20, 20, 5, 5, 10, 10);
    let mut borders = get_all_borders(&pix).unwrap();
    borders.generate_single_path().unwrap();
    borders.generate_sp_global_locs();
    let comp = &borders.components[0];
    let sp_global = comp.global_single_path.as_ref().unwrap();
    assert!(!sp_global.is_empty());
    for p in sp_global {
        assert!(p.x >= 0 && p.x < 20);
        assert!(p.y >= 0 && p.y < 20);
    }
}

// ============================================================================
// 15. ccbaGenerateSinglePath (already exists)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_generate_single_path_coverage() {
    let pix = make_ring(10, 10, 0, 0, 7, 7);
    let mut borders = get_all_borders(&pix).unwrap();
    borders.generate_single_path().unwrap();
    let comp = &borders.components[0];
    assert!(comp.single_path.is_some());
    let path = comp.single_path.as_ref().unwrap();
    assert!(path.len() >= comp.outer.len());
}

// ============================================================================
// 16. getCutPathForHole
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_get_cut_path_for_hole() {
    // Ring shape
    let pix = make_ring(10, 10, 0, 0, 7, 7);
    let borders = get_all_borders(&pix).unwrap();
    let comp = &borders.components[0];
    assert!(comp.has_holes());
    let hole = &comp.holes[0];
    let cut = get_cut_path_for_hole(&comp.outer, hole, &comp.bounds);
    assert!(!cut.is_empty());
}

// ============================================================================
// 17. ccbaWrite (file wrapper)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_write_to_file() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let borders = get_all_borders(&pix).unwrap();
    let path = std::env::temp_dir().join("test_ccba_write.bin");
    borders.write_to_file(&path).unwrap();
    assert!(path.exists());
    std::fs::remove_file(&path).ok();
}

// ============================================================================
// 18. ccbaWriteStream (already exists as write method)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_write_stream() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let borders = get_all_borders(&pix).unwrap();
    let mut buf = Vec::new();
    borders.write(&mut buf).unwrap();
    assert!(!buf.is_empty());
    // Check magic header
    assert_eq!(&buf[0..4], b"ccba");
}

// ============================================================================
// 19. ccbaRead (file wrapper)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_read_from_file() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let borders = get_all_borders(&pix).unwrap();
    let path = std::env::temp_dir().join("test_ccba_read.bin");
    borders.write_to_file(&path).unwrap();

    let read_back = ImageBorders::read_from_file(&path).unwrap();
    assert_eq!(read_back.width, borders.width);
    assert_eq!(read_back.height, borders.height);
    assert_eq!(read_back.component_count(), borders.component_count());
    std::fs::remove_file(&path).ok();
}

// ============================================================================
// 20. ccbaReadStream (already exists as read_from method)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_read_stream() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let borders = get_all_borders(&pix).unwrap();
    let mut buf = Vec::new();
    borders.write(&mut buf).unwrap();

    let read_back = ImageBorders::read_from(std::io::Cursor::new(&buf)).unwrap();
    assert_eq!(read_back.component_count(), borders.component_count());
}

// ============================================================================
// 21. ccbaWriteSVG (file wrapper)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_write_svg_to_file() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let mut borders = get_all_borders(&pix).unwrap();
    borders.generate_single_path().unwrap();
    let path = std::env::temp_dir().join("test_ccba.svg");
    borders.write_svg_to_file(&path).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
    std::fs::remove_file(&path).ok();
}

// ============================================================================
// 22. ccbaWriteSVGString (already exists as to_svg_string)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_to_svg_string_coverage() {
    let pix = make_rect(10, 10, 1, 1, 5, 5);
    let mut borders = get_all_borders(&pix).unwrap();
    borders.generate_single_path().unwrap();
    let svg = borders.to_svg_string().unwrap();
    assert!(svg.contains("<svg "));
    assert!(svg.contains("polygon"));
    assert!(svg.contains("</svg>"));
}

// ============================================================================
// 23. pixConnCompIncrInit
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_pix_conn_comp_incr_init_empty() {
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let (labeler, ncc) = pix_conn_comp_incr_init(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(ncc, 0);
    let result = labeler.finish();
    assert_eq!(result.depth(), PixelDepth::Bit32);
}

#[test]
#[ignore = "not yet implemented"]
fn test_pix_conn_comp_incr_init_with_components() {
    let pix = make_binary(10, 10, &[(1, 1), (2, 1), (5, 5), (6, 5)]);
    let (labeler, ncc) = pix_conn_comp_incr_init(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(ncc, 2);
    let result = labeler.finish();
    let l1 = result.get_pixel(1, 1).unwrap();
    let l2 = result.get_pixel(5, 5).unwrap();
    assert!(l1 > 0);
    assert!(l2 > 0);
    assert_ne!(l1, l2);
}

// ============================================================================
// 24. pixConnCompIncrAdd
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_pix_conn_comp_incr_add_new_component() {
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let (mut labeler, mut ncc) = pix_conn_comp_incr_init(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(ncc, 0);

    // Add a pixel to empty image -> new component
    let result = pix_conn_comp_incr_add(&mut labeler, &mut ncc, 5, 5);
    assert!(result.is_ok());
    assert_eq!(ncc, 1);
}

#[test]
#[ignore = "not yet implemented"]
fn test_pix_conn_comp_incr_add_join() {
    // Create two separate components, then connect them
    let pix = make_binary(10, 10, &[(1, 1), (3, 1)]);
    let (mut labeler, mut ncc) = pix_conn_comp_incr_init(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(ncc, 2);

    // Add pixel at (2,1) to connect them
    let result = pix_conn_comp_incr_add(&mut labeler, &mut ncc, 2, 1);
    assert!(result.is_ok());
    assert_eq!(ncc, 1); // merged into one component
}

#[test]
#[ignore = "not yet implemented"]
fn test_pix_conn_comp_incr_add_extend() {
    let pix = make_binary(10, 10, &[(1, 1), (2, 1)]);
    let (mut labeler, mut ncc) = pix_conn_comp_incr_init(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(ncc, 1);

    // Add adjacent pixel -> same component
    let result = pix_conn_comp_incr_add(&mut labeler, &mut ncc, 3, 1);
    assert!(result.is_ok());
    assert_eq!(ncc, 1); // still one component
}

// ============================================================================
// 25. pixLocToColorTransform
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_pix_loc_to_color_transform_basic() {
    let pix = make_rect(20, 20, 5, 5, 15, 15);
    let colored = pix_loc_to_color_transform(&pix).unwrap();
    assert_eq!(colored.depth(), PixelDepth::Bit32);
    assert_eq!(colored.width(), 20);
    assert_eq!(colored.height(), 20);

    // Background pixels should be black (0x000000FF or 0x00000000)
    let bg = colored.get_pixel(0, 0).unwrap();
    assert_eq!(bg, 0);

    // Foreground pixels should have non-zero color
    let fg = colored.get_pixel(10, 10).unwrap();
    assert_ne!(fg, 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_pix_loc_to_color_transform_empty() {
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let colored = pix_loc_to_color_transform(&pix).unwrap();
    // All pixels should be black
    for y in 0..10 {
        for x in 0..10 {
            assert_eq!(colored.get_pixel(x, y), Some(0));
        }
    }
}

#[test]
#[ignore = "not yet implemented"]
fn test_pix_loc_to_color_transform_spatial_coding() {
    // Pixels at different locations should get different colors
    let pix = make_binary(100, 100, &[(10, 10), (90, 90)]);
    let colored = pix_loc_to_color_transform(&pix).unwrap();
    let c1 = colored.get_pixel(10, 10).unwrap();
    let c2 = colored.get_pixel(90, 90).unwrap();
    // Different locations should yield different color encodings
    assert_ne!(c1, c2);
}
