//! pixa1_reg - Pixa (image array) operations regression test
//!
//! This test corresponds to pixa1_reg.c in the C version.
//! Tests removal of connected components by size using bounding box filtering.
//!
//! The C test:
//!   1. Loads feyn-fract.tif (binary image)
//!   2. Finds connected components (8-connectivity)
//!   3. Iterates over threshold sizes 2..100 (step 2)
//!   4. For each size, selects components where BOTH width and height >= size (GTE)
//!      and counts remaining components
//!   5. Same with EITHER width or height >= size
//!   6. Same for keeping small (LTE) with BOTH and EITHER
//!   7. Generates gnuplot graphs (skipped in Rust -- no gplot API)
//!
//! Run with:
//! ```
//! cargo test -p leptonica-region --test pixa1_reg -- --nocapture
//! ```

use leptonica_core::{Pix, PixelDepth};
use leptonica_region::{
    ConnectivityType, SizeSelectRelation, SizeSelectType, find_connected_components,
    pix_select_by_size,
};
use leptonica_test::{RegParams, load_test_image};

const CONNECTIVITY: ConnectivityType = ConnectivityType::EightWay;

/// Count connected components in a binary image
fn count_components(pix: &Pix, connectivity: ConnectivityType) -> usize {
    find_connected_components(pix, connectivity)
        .expect("find_connected_components failed")
        .len()
}

#[test]
fn pixa1_reg() {
    let mut rp = RegParams::new("pixa1");

    // C版: pixs = pixRead("feyn-fract.tif")
    let pixs = match load_test_image("feyn-fract.tif") {
        Ok(pix) => pix,
        Err(e) => {
            panic!("Failed to load test image feyn-fract.tif: {}", e);
        }
    };

    assert_eq!(
        pixs.depth(),
        PixelDepth::Bit1,
        "Test image should be binary (1 bpp)"
    );
    eprintln!(
        "Image size: {}x{}, depth: {:?}",
        pixs.width(),
        pixs.height(),
        pixs.depth()
    );

    // ----------------  Count initial components  ---------------
    // C版: boxa = pixConnComp(pixs, NULL, 8); n0 = boxaGetCount(boxa);
    let n0 = count_components(&pixs, CONNECTIVITY);
    eprintln!("Initial number of 8-connected components: n0 = {}", n0);

    // Sanity check: should have many components in feyn-fract.tif
    assert!(
        n0 > 100,
        "Expected many connected components in feyn-fract.tif, got {}",
        n0
    );

    // ================================================================
    // Part 1: Remove small components (keep large)
    //         pixSelectBySize with L_SELECT_IF_GTE
    // ================================================================

    // --- Select Large if Both (width >= size AND height >= size) ---
    eprintln!("\n Select Large if Both");
    eprintln!("Iter 0: n = {}", n0);
    let mut nay1_gte_both: Vec<usize> = Vec::with_capacity(51);
    nay1_gte_both.push(n0);

    for i in 1..=50 {
        let size = 2 * i;
        let pixd = pix_select_by_size(
            &pixs,
            size,
            size,
            CONNECTIVITY,
            SizeSelectType::IfBoth,
            SizeSelectRelation::Gte,
        )
        .expect("pix_select_by_size failed");
        let n = count_components(&pixd, CONNECTIVITY);
        nay1_gte_both.push(n);
        if i <= 5 || i % 10 == 0 {
            eprintln!("Iter {}: size={}, n = {}", i, size, n);
        }
    }

    // --- Select Large if Either (width >= size OR height >= size) ---
    eprintln!("\n Select Large if Either");
    eprintln!("Iter 0: n = {}", n0);
    let mut nay2_gte_either: Vec<usize> = Vec::with_capacity(51);
    nay2_gte_either.push(n0);

    for i in 1..=50 {
        let size = 2 * i;
        let pixd = pix_select_by_size(
            &pixs,
            size,
            size,
            CONNECTIVITY,
            SizeSelectType::IfEither,
            SizeSelectRelation::Gte,
        )
        .expect("pix_select_by_size failed");
        let n = count_components(&pixd, CONNECTIVITY);
        nay2_gte_either.push(n);
        if i <= 5 || i % 10 == 0 {
            eprintln!("Iter {}: size={}, n = {}", i, size, n);
        }
    }

    // C版: gplotCreate / gplotAddPlot / gplotMakeOutputPix -- Rust未実装のためスキップ
    // C版: regTestWritePixAndCheck(rp, pix1, IFF_PNG) -- gnuplotグラフ出力のためスキップ

    // ================================================================
    // Part 2: Remove large components (keep small)
    //         pixSelectBySize with L_SELECT_IF_LTE
    // ================================================================

    // --- Select Small if Both (width <= size AND height <= size) ---
    eprintln!("\n Select Small if Both");
    eprintln!("Iter 0: n = 0");
    let mut nay1_lte_both: Vec<usize> = Vec::with_capacity(51);
    nay1_lte_both.push(0); // C version starts with 0 at iter 0

    for i in 1..=50 {
        let size = 2 * i;
        let pixd = pix_select_by_size(
            &pixs,
            size,
            size,
            CONNECTIVITY,
            SizeSelectType::IfBoth,
            SizeSelectRelation::Lte,
        )
        .expect("pix_select_by_size failed");
        let n = count_components(&pixd, CONNECTIVITY);
        nay1_lte_both.push(n);
        if i <= 5 || i % 10 == 0 {
            eprintln!("Iter {}: size={}, n = {}", i, size, n);
        }
    }

    // --- Select Small if Either (width <= size OR height <= size) ---
    eprintln!("\n Select Small if Either");
    eprintln!("Iter 0: n = 0");
    let mut nay2_lte_either: Vec<usize> = Vec::with_capacity(51);
    nay2_lte_either.push(0); // C version starts with 0 at iter 0

    for i in 1..=50 {
        let size = 2 * i;
        let pixd = pix_select_by_size(
            &pixs,
            size,
            size,
            CONNECTIVITY,
            SizeSelectType::IfEither,
            SizeSelectRelation::Lte,
        )
        .expect("pix_select_by_size failed");
        let n = count_components(&pixd, CONNECTIVITY);
        nay2_lte_either.push(n);
        if i <= 5 || i % 10 == 0 {
            eprintln!("Iter {}: size={}, n = {}", i, size, n);
        }
    }

    // C版: gplotCreate / gplotAddPlot / gplotMakeOutputPix -- Rust未実装のためスキップ
    // C版: regTestWritePixAndCheck(rp, pix1, IFF_PNG) -- gnuplotグラフ出力のためスキップ
    // C版: pixaDisplayTiledInRows / pixDisplayWithTitle -- Rust未実装のためスキップ

    // ================================================================
    // Verification: Monotonicity and consistency checks
    // ================================================================

    // -- GTE Both: as threshold increases, fewer components should remain
    // (monotonically non-increasing)
    eprintln!("\n--- Verification: GTE Both monotonicity ---");
    for i in 1..nay1_gte_both.len() {
        assert!(
            nay1_gte_both[i] <= nay1_gte_both[i - 1],
            "GTE Both: count at size {} ({}) should be <= count at size {} ({})",
            2 * i,
            nay1_gte_both[i],
            2 * (i - 1),
            nay1_gte_both[i - 1]
        );
    }
    rp.compare_values(1.0, 1.0, 0.0); // monotonicity passed

    // -- GTE Either: same monotonicity
    eprintln!("--- Verification: GTE Either monotonicity ---");
    for i in 1..nay2_gte_either.len() {
        assert!(
            nay2_gte_either[i] <= nay2_gte_either[i - 1],
            "GTE Either: count at size {} ({}) should be <= count at size {} ({})",
            2 * i,
            nay2_gte_either[i],
            2 * (i - 1),
            nay2_gte_either[i - 1]
        );
    }
    rp.compare_values(1.0, 1.0, 0.0); // monotonicity passed

    // -- LTE Both: as threshold increases, more components should be kept
    // (monotonically non-decreasing)
    eprintln!("--- Verification: LTE Both monotonicity ---");
    for i in 1..nay1_lte_both.len() {
        assert!(
            nay1_lte_both[i] >= nay1_lte_both[i - 1],
            "LTE Both: count at size {} ({}) should be >= count at size {} ({})",
            2 * i,
            nay1_lte_both[i],
            2 * (i - 1),
            nay1_lte_both[i - 1]
        );
    }
    rp.compare_values(1.0, 1.0, 0.0); // monotonicity passed

    // -- LTE Either: same monotonicity
    eprintln!("--- Verification: LTE Either monotonicity ---");
    for i in 1..nay2_lte_either.len() {
        assert!(
            nay2_lte_either[i] >= nay2_lte_either[i - 1],
            "LTE Either: count at size {} ({}) should be >= count at size {} ({})",
            2 * i,
            nay2_lte_either[i],
            2 * (i - 1),
            nay2_lte_either[i - 1]
        );
    }
    rp.compare_values(1.0, 1.0, 0.0); // monotonicity passed

    // -- GTE Both should always have >= components than GTE Either
    // Because "Both >= size" is a stricter filter than "Either >= size"
    // Wait: Actually it's the opposite. IfBoth means BOTH must be >= size,
    // which is stricter, so FEWER components are removed (more pass).
    // Actually no: "select if GTE" means we KEEP components where dim >= size.
    // IfBoth: keep if w>=size AND h>=size (stricter -> fewer kept)
    // IfEither: keep if w>=size OR h>=size (looser -> more kept)
    eprintln!("--- Verification: GTE Either >= GTE Both ---");
    for i in 0..nay1_gte_both.len() {
        assert!(
            nay2_gte_either[i] >= nay1_gte_both[i],
            "At size {}: GTE Either ({}) should be >= GTE Both ({})",
            2 * i,
            nay2_gte_either[i],
            nay1_gte_both[i]
        );
    }
    rp.compare_values(1.0, 1.0, 0.0); // relation check passed

    // -- LTE Both should always have <= components than LTE Either
    // IfBoth: keep if w<=size AND h<=size (stricter -> fewer kept)
    // IfEither: keep if w<=size OR h<=size (looser -> more kept)
    eprintln!("--- Verification: LTE Either >= LTE Both ---");
    for i in 0..nay1_lte_both.len() {
        assert!(
            nay2_lte_either[i] >= nay1_lte_both[i],
            "At size {}: LTE Either ({}) should be >= LTE Both ({})",
            2 * i,
            nay2_lte_either[i],
            nay1_lte_both[i]
        );
    }
    rp.compare_values(1.0, 1.0, 0.0); // relation check passed

    // -- Complementarity check: for any size,
    // (GTE Both) + (LTE Both with strict LT) should roughly account for all components
    // More precisely: components kept by GTE(size) + components kept by LT(size) = total
    // But since we use GTE and LTE (with overlap at exactly size), this is not exact.
    // We verify that at the largest threshold, GTE Both should have very few or 0 components
    eprintln!("--- Verification: Large threshold reduces to few components ---");
    let last_gte_both = *nay1_gte_both.last().unwrap();
    eprintln!(
        "GTE Both at size=100: {} components (of {} total)",
        last_gte_both, n0
    );
    assert!(
        last_gte_both < n0,
        "At large threshold, should have fewer components than original"
    );
    rp.compare_values(1.0, 1.0, 0.0); // large threshold check passed

    // -- Verify at size=2, GTE Both should have all components with w>=2 AND h>=2
    // This should be fewer than n0 (some 1-pixel components exist)
    let gte_both_2 = nay1_gte_both[1]; // index 1 corresponds to size=2
    eprintln!("GTE Both at size=2: {} (total: {})", gte_both_2, n0);
    assert!(
        gte_both_2 <= n0,
        "Components with both dimensions >=2 should be <= total"
    );
    rp.compare_values(1.0, 1.0, 0.0);

    // -- At the largest threshold (size=100), LTE Both should have nearly all
    // components (most components in text are smaller than 100x100)
    let last_lte_both = *nay1_lte_both.last().unwrap();
    eprintln!("LTE Both at size=100: {} (total: {})", last_lte_both, n0);
    // Most text components should be smaller than 100x100
    assert!(
        last_lte_both > 0,
        "At size=100, should still have components with both dims <= 100"
    );
    rp.compare_values(1.0, 1.0, 0.0);

    // Print summary tables
    eprintln!("\n--- Summary: component counts by threshold size ---");
    eprintln!(
        "{:>6} {:>12} {:>12} {:>12} {:>12}",
        "size", "GTE Both", "GTE Either", "LTE Both", "LTE Either"
    );
    for i in (0..=50).step_by(5) {
        let size = 2 * i;
        eprintln!(
            "{:>6} {:>12} {:>12} {:>12} {:>12}",
            size, nay1_gte_both[i], nay2_gte_either[i], nay1_lte_both[i], nay2_lte_either[i]
        );
    }

    assert!(rp.cleanup(), "pixa1 regression test failed");
}

/// Test basic Pixa operations that the C test uses implicitly
/// (pixaCreate, pixaAddPix, pixaGetCount, etc.)
#[test]
fn pixa1_basic_pixa_ops() {
    use leptonica_core::Pixa;

    let mut rp = RegParams::new("pixa1_basic");

    // C版: pixa = pixaCreate(2)
    let mut pixa = Pixa::with_capacity(2);
    assert_eq!(pixa.len(), 0);

    // Create test images
    let pix1 = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
    let pix2 = Pix::new(200, 200, PixelDepth::Bit1).unwrap();

    // C版: pixaAddPix(pixa, pix1, L_INSERT)
    pixa.push(pix1);
    assert_eq!(pixa.len(), 1);

    pixa.push(pix2);
    assert_eq!(pixa.len(), 2);

    // C版: pixaGetCount(pixa)
    rp.compare_values(2.0, pixa.len() as f64, 0.0);

    // Verify dimensions are accessible
    let (w, h, d) = pixa.get_dimensions(0).unwrap();
    assert_eq!(w, 100);
    assert_eq!(h, 100);
    assert_eq!(d, PixelDepth::Bit1);

    let (w, h, _d) = pixa.get_dimensions(1).unwrap();
    assert_eq!(w, 200);
    assert_eq!(h, 200);

    // Test iteration (used implicitly in component processing)
    let widths: Vec<u32> = pixa.iter().map(|p| p.width()).collect();
    assert_eq!(widths, vec![100, 200]);
    rp.compare_values(2.0, widths.len() as f64, 0.0);

    assert!(rp.cleanup(), "pixa1_basic regression test failed");
}

/// Test connected component finding with bounding box info,
/// which is the foundation of pixSelectBySize
#[test]
fn pixa1_conncomp_with_boxes() {
    let mut rp = RegParams::new("pixa1_conncomp");

    let pixs = match load_test_image("feyn-fract.tif") {
        Ok(pix) => pix,
        Err(e) => {
            panic!("Failed to load test image: {}", e);
        }
    };

    // Find connected components (same as C version uses for pixSelectBySize)
    let components = find_connected_components(&pixs, ConnectivityType::EightWay)
        .expect("find_connected_components failed");

    let n = components.len();
    eprintln!("Number of 8-connected components: {}", n);
    assert!(n > 0, "Should find components in feyn-fract.tif");

    // Verify all components have valid bounding boxes
    let img_w = pixs.width() as i32;
    let img_h = pixs.height() as i32;

    let mut min_w = i32::MAX;
    let mut max_w = 0i32;
    let mut min_h = i32::MAX;
    let mut max_h = 0i32;

    for comp in &components {
        assert!(comp.bounds.w > 0, "Component width must be positive");
        assert!(comp.bounds.h > 0, "Component height must be positive");
        assert!(comp.bounds.x >= 0, "Component x must be non-negative");
        assert!(comp.bounds.y >= 0, "Component y must be non-negative");
        assert!(
            comp.bounds.x + comp.bounds.w <= img_w,
            "Component must fit within image width: x={} w={} img_w={}",
            comp.bounds.x,
            comp.bounds.w,
            img_w
        );
        assert!(
            comp.bounds.y + comp.bounds.h <= img_h,
            "Component must fit within image height: y={} h={} img_h={}",
            comp.bounds.y,
            comp.bounds.h,
            img_h
        );
        assert!(comp.pixel_count > 0, "Component must have at least 1 pixel");

        min_w = min_w.min(comp.bounds.w);
        max_w = max_w.max(comp.bounds.w);
        min_h = min_h.min(comp.bounds.h);
        max_h = max_h.max(comp.bounds.h);
    }

    eprintln!("Bounding box width range: {} - {}", min_w, max_w);
    eprintln!("Bounding box height range: {} - {}", min_h, max_h);

    // There should be a range of component sizes in this text image
    assert!(
        max_w > min_w,
        "Expected varying component widths in text image"
    );
    assert!(
        max_h > min_h,
        "Expected varying component heights in text image"
    );

    rp.compare_values(n as f64, n as f64, 0.0); // self-check
    assert!(rp.cleanup(), "pixa1_conncomp regression test failed");
}

/// Test that pix_select_by_size produces valid binary output
/// and preserves only the correct components
#[test]
fn pixa1_select_by_size_validation() {
    let mut rp = RegParams::new("pixa1_selsize");

    let pixs = match load_test_image("feyn-fract.tif") {
        Ok(pix) => pix,
        Err(e) => {
            panic!("Failed to load test image: {}", e);
        }
    };

    // Select components where both w>=10 and h>=10
    let pixd = pix_select_by_size(
        &pixs,
        10,
        10,
        CONNECTIVITY,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("pix_select_by_size failed");

    // Output should be same dimensions as input
    assert_eq!(pixd.width(), pixs.width());
    assert_eq!(pixd.height(), pixs.height());
    assert_eq!(pixd.depth(), PixelDepth::Bit1);

    // All components in the result should have both dimensions >= 10
    let result_components = find_connected_components(&pixd, CONNECTIVITY)
        .expect("find_connected_components on filtered result failed");

    for comp in &result_components {
        assert!(
            comp.bounds.w >= 10 && comp.bounds.h >= 10,
            "Filtered component should have w>=10 and h>=10, got w={} h={}",
            comp.bounds.w,
            comp.bounds.h
        );
    }
    eprintln!(
        "After GTE Both (10,10): {} components (all with w>=10 and h>=10)",
        result_components.len()
    );
    rp.compare_values(1.0, 1.0, 0.0); // validation passed

    // Select components where both w<=5 and h<=5
    let pixd_small = pix_select_by_size(
        &pixs,
        5,
        5,
        CONNECTIVITY,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Lte,
    )
    .expect("pix_select_by_size failed");

    let small_components = find_connected_components(&pixd_small, CONNECTIVITY)
        .expect("find_connected_components on small filtered result failed");

    for comp in &small_components {
        assert!(
            comp.bounds.w <= 5 && comp.bounds.h <= 5,
            "Filtered component should have w<=5 and h<=5, got w={} h={}",
            comp.bounds.w,
            comp.bounds.h
        );
    }
    eprintln!(
        "After LTE Both (5,5): {} components (all with w<=5 and h<=5)",
        small_components.len()
    );
    rp.compare_values(1.0, 1.0, 0.0); // validation passed

    // The filtered result should be a subset of the original:
    // every ON pixel in pixd should also be ON in pixs
    let w = pixs.width();
    let h = pixs.height();
    for y in 0..h {
        for x in 0..w {
            let filtered_val = pixd.get_pixel(x, y).unwrap_or(0);
            if filtered_val != 0 {
                let orig_val = pixs.get_pixel(x, y).unwrap_or(0);
                assert!(
                    orig_val != 0,
                    "Filtered pixel at ({},{}) is ON but original is OFF",
                    x,
                    y
                );
            }
        }
    }
    rp.compare_values(1.0, 1.0, 0.0); // subset check passed

    assert!(rp.cleanup(), "pixa1_selsize regression test failed");
}
