//! Component borders regression test
//!
//! C版: reference/leptonica/prog/ccbord_reg.c
//! 連結成分の境界追跡、チェインコードをテスト。

use leptonica_core::{Pix, PixelDepth};
use leptonica_region::{
    BorderPoint, from_chain_code, get_all_borders, get_outer_borders, to_chain_code,
};
use leptonica_test::RegParams;

/// Create a small test image with known shapes for border testing
fn create_test_shapes() -> Pix {
    // 40x30 image with a few distinct shapes
    let pix = Pix::new(40, 30, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    // Shape 1: 5x5 filled square at (2,2)
    for y in 2..7 {
        for x in 2..7 {
            let _ = pix_mut.set_pixel(x, y, 1);
        }
    }

    // Shape 2: 3x3 filled square at (15,5)
    for y in 5..8 {
        for x in 15..18 {
            let _ = pix_mut.set_pixel(x, y, 1);
        }
    }

    // Shape 3: Ring (square with hole) at (25,2)
    // 7x7 outer, 3x3 hole
    for y in 2..9 {
        for x in 25..32 {
            if y == 2 || y == 8 || x == 25 || x == 31 {
                let _ = pix_mut.set_pixel(x, y, 1);
            } else if y >= 4 && y <= 6 && x >= 27 && x <= 29 {
                // hole: leave as 0
            } else {
                let _ = pix_mut.set_pixel(x, y, 1);
            }
        }
    }

    // Shape 4: Small line at (5, 20)
    for x in 5..12 {
        let _ = pix_mut.set_pixel(x, 20, 1);
    }

    pix_mut.into()
}

#[test]
fn ccbord_reg() {
    let mut rp = RegParams::new("ccbord");

    let pixs = create_test_shapes();
    assert_eq!(pixs.depth(), PixelDepth::Bit1);
    eprintln!("Image: {}x{}", pixs.width(), pixs.height());

    // --- Test 1: Get outer borders ---
    eprintln!("=== Outer borders ===");
    let borders = get_outer_borders(&pixs).expect("get_outer_borders");
    let n_borders = borders.len();
    eprintln!("  Number of outer borders: {}", n_borders);
    rp.compare_values(1.0, if n_borders > 0 { 1.0 } else { 0.0 }, 0.0);
    // We created 4 shapes, so should have 4 borders
    rp.compare_values(4.0, n_borders as f64, 0.0);

    // --- Test 2: Border properties ---
    for (i, border) in borders.iter().enumerate() {
        let n_pts = border.points.len();
        rp.compare_values(1.0, if n_pts > 0 { 1.0 } else { 0.0 }, 0.0);
        eprintln!("  border[{}]: {} points", i, n_pts);
    }

    // --- Test 3: Chain code roundtrip ---
    eprintln!("=== Chain code ===");
    if let Some(border) = borders.first() {
        let chain = to_chain_code(&border.points);
        rp.compare_values(1.0, if !chain.is_empty() { 1.0 } else { 0.0 }, 0.0);
        eprintln!("  chain code length: {}", chain.len());

        // Reconstruct points from chain code
        let start = border
            .points
            .first()
            .copied()
            .unwrap_or(BorderPoint::new(0, 0));
        let reconstructed = from_chain_code(start, &chain);
        rp.compare_values(border.points.len() as f64, reconstructed.len() as f64, 0.0);
        eprintln!("  reconstructed points: {}", reconstructed.len());

        // First point should match
        if !border.points.is_empty() && !reconstructed.is_empty() {
            let op = &border.points[0];
            let rp2 = &reconstructed[0];
            rp.compare_values(op.x as f64, rp2.x as f64, 0.0);
            rp.compare_values(op.y as f64, rp2.y as f64, 0.0);
        }
    }

    // --- Test 4: Get all borders (outer + holes) ---
    eprintln!("=== All borders ===");
    let all = get_all_borders(&pixs).expect("get_all_borders");
    let n_all = all.components.len();
    eprintln!("  Components with borders: {}", n_all);
    rp.compare_values(1.0, if n_all > 0 { 1.0 } else { 0.0 }, 0.0);

    // Each component should have at least an outer border
    for (i, comp) in all.components.iter().enumerate() {
        let has_outer = !comp.outer.points.is_empty();
        rp.compare_values(1.0, if has_outer { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  comp[{}]: outer={} pts, holes={} borders",
            i,
            comp.outer.points.len(),
            comp.holes.len()
        );
    }

    // The ring shape should have a hole
    let has_any_holes = all.components.iter().any(|c| !c.holes.is_empty());
    rp.compare_values(1.0, if has_any_holes { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  Has components with holes: {}", has_any_holes);

    assert!(rp.cleanup(), "ccbord regression test failed");
}
