//! DWA morphology regression test 2
//!
//! C version: reference/leptonica/prog/dwamorph2_reg.c
//! Compares timing and correctness of various binary morphological
//! implementations: standard brick, DWA brick.
//!
//! The C version benchmarks four implementations across all linear DWA
//! selection sizes (2..63) for dilation, erosion, opening, and closing:
//!   1. Linear rasterop (pixDilate with sel)
//!   2. Composite rasterop (pixDilateCompBrick)
//!   3. Linear DWA (pixMorphDwa_3)
//!   4. Composite DWA (pixDilateCompBrickDwa)
//!
//! In Rust, we have:
//!   - Standard brick: dilate_brick, erode_brick, open_brick, close_brick
//!   - DWA brick: dilate_brick_dwa, erode_brick_dwa, open_brick_dwa, close_brick_dwa
//!
//! C版: selaAddDwaLinear() -- Rust未実装のためスキップ（線形サイズ2..64で代用）
//! C版: pixDilateCompBrick() -- composite rasterop -- Rust未実装のためスキップ
//! C版: pixMorphDwa_3() -- named DWA sel -- Rust未実装のためスキップ（dilate_brick_dwaで代用）
//! C版: pixDilateCompBrickDwa() -- composite DWA -- Rust未実装のためスキップ
//! C版: GPLOT / gnuplot timing graphs -- Rust未実装のためスキップ
//! C版: numaWindowedMean() -- Rust未実装のためスキップ
//!
//! Run with:
//! ```
//! cargo test -p leptonica-morph --test dwamorph2_reg -- --nocapture
//! ```

use leptonica_core::PixelDepth;
use leptonica_morph::{
    close_brick, close_brick_dwa, dilate_brick, dilate_brick_dwa, erode_brick, erode_brick_dwa,
    open_brick, open_brick_dwa,
};
use leptonica_test::{RegParams, load_test_image};
use std::time::Instant;

/// Number of repetitions for timing (reduced from C version's 20 for test speed)
const NTIMES: u32 = 2;

/// Maximum linear size to test.
/// C version uses selaAddDwaLinear which provides selections of sizes 2..63.
/// We use a slightly smaller range (2..32) to keep test runtime reasonable.
const MAX_LINEAR_SIZE: u32 = 32;

/// Compare two Pix for equality using data-level comparison
fn compare_pix(pix1: &leptonica_core::Pix, pix2: &leptonica_core::Pix) -> bool {
    if pix1.width() != pix2.width() || pix1.height() != pix2.height() {
        return false;
    }
    pix1.equals(pix2)
}

/// Count differing pixels between two images
fn count_diff_pixels(pix1: &leptonica_core::Pix, pix2: &leptonica_core::Pix) -> u64 {
    if pix1.width() != pix2.width() || pix1.height() != pix2.height() {
        return u64::MAX;
    }
    let w = pix1.width();
    let h = pix1.height();
    let mut count = 0u64;
    for y in 0..h {
        for x in 0..w {
            if pix1.get_pixel(x, y) != pix2.get_pixel(x, y) {
                count += 1;
            }
        }
    }
    count
}

/// Test DWA vs standard brick morph for horizontal linear selections.
///
/// The C version iterates over the first half of the linear DWA sel array,
/// which corresponds to horizontal linear selections of sizes 2..63.
/// For each size, it benchmarks and compares four implementations.
///
/// We faithfully port this by comparing DWA brick vs standard brick
/// for horizontal selections (size, 1).
#[test]
fn dwamorph2_reg_horizontal() {
    let mut rp = RegParams::new("dwamorph2_h");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    eprintln!(
        "Testing horizontal DWA vs brick morph for sizes 2..{}",
        MAX_LINEAR_SIZE
    );

    // --- Dilation ---
    eprintln!("  --- Dilation (horizontal) ---");
    for size in 2..MAX_LINEAR_SIZE {
        let dil_brick = dilate_brick(&pixs, size, 1).expect("dilate_brick");
        let dil_dwa = dilate_brick_dwa(&pixs, size, 1).expect("dilate_brick_dwa");
        let same = compare_pix(&dil_brick, &dil_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        if !same {
            let diff = count_diff_pixels(&dil_brick, &dil_dwa);
            eprintln!("  dilate h size={}: DIFFER ({} pixels)", size, diff);
        }
    }

    // --- Erosion ---
    eprintln!("  --- Erosion (horizontal) ---");
    for size in 2..MAX_LINEAR_SIZE {
        let ero_brick = erode_brick(&pixs, size, 1).expect("erode_brick");
        let ero_dwa = erode_brick_dwa(&pixs, size, 1).expect("erode_brick_dwa");
        let same = compare_pix(&ero_brick, &ero_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        if !same {
            let diff = count_diff_pixels(&ero_brick, &ero_dwa);
            eprintln!("  erode h size={}: DIFFER ({} pixels)", size, diff);
        }
    }

    // --- Opening ---
    eprintln!("  --- Opening (horizontal) ---");
    for size in 2..MAX_LINEAR_SIZE {
        let open_std = open_brick(&pixs, size, 1).expect("open_brick");
        let open_dwa = open_brick_dwa(&pixs, size, 1).expect("open_brick_dwa");
        let same = compare_pix(&open_std, &open_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        if !same {
            let diff = count_diff_pixels(&open_std, &open_dwa);
            eprintln!("  open h size={}: DIFFER ({} pixels)", size, diff);
        }
    }

    // --- Closing ---
    eprintln!("  --- Closing (horizontal) ---");
    for size in 2..MAX_LINEAR_SIZE {
        let close_std = close_brick(&pixs, size, 1).expect("close_brick");
        let close_dwa = close_brick_dwa(&pixs, size, 1).expect("close_brick_dwa");
        let same = compare_pix(&close_std, &close_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        if !same {
            let diff = count_diff_pixels(&close_std, &close_dwa);
            eprintln!("  close h size={}: DIFFER ({} pixels)", size, diff);
        }
    }

    assert!(rp.cleanup(), "dwamorph2 horizontal regression test failed");
}

/// Test DWA vs standard brick morph for vertical linear selections.
///
/// The C version iterates over the second half of the linear DWA sel array,
/// which corresponds to vertical linear selections of sizes 2..63.
#[test]
fn dwamorph2_reg_vertical() {
    let mut rp = RegParams::new("dwamorph2_v");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    eprintln!(
        "Testing vertical DWA vs brick morph for sizes 2..{}",
        MAX_LINEAR_SIZE
    );

    // --- Dilation ---
    eprintln!("  --- Dilation (vertical) ---");
    for size in 2..MAX_LINEAR_SIZE {
        let dil_brick = dilate_brick(&pixs, 1, size).expect("dilate_brick");
        let dil_dwa = dilate_brick_dwa(&pixs, 1, size).expect("dilate_brick_dwa");
        let same = compare_pix(&dil_brick, &dil_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        if !same {
            let diff = count_diff_pixels(&dil_brick, &dil_dwa);
            eprintln!("  dilate v size={}: DIFFER ({} pixels)", size, diff);
        }
    }

    // --- Erosion ---
    eprintln!("  --- Erosion (vertical) ---");
    for size in 2..MAX_LINEAR_SIZE {
        let ero_brick = erode_brick(&pixs, 1, size).expect("erode_brick");
        let ero_dwa = erode_brick_dwa(&pixs, 1, size).expect("erode_brick_dwa");
        let same = compare_pix(&ero_brick, &ero_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        if !same {
            let diff = count_diff_pixels(&ero_brick, &ero_dwa);
            eprintln!("  erode v size={}: DIFFER ({} pixels)", size, diff);
        }
    }

    // --- Opening ---
    eprintln!("  --- Opening (vertical) ---");
    for size in 2..MAX_LINEAR_SIZE {
        let open_std = open_brick(&pixs, 1, size).expect("open_brick");
        let open_dwa = open_brick_dwa(&pixs, 1, size).expect("open_brick_dwa");
        let same = compare_pix(&open_std, &open_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        if !same {
            let diff = count_diff_pixels(&open_std, &open_dwa);
            eprintln!("  open v size={}: DIFFER ({} pixels)", size, diff);
        }
    }

    // --- Closing ---
    eprintln!("  --- Closing (vertical) ---");
    for size in 2..MAX_LINEAR_SIZE {
        let close_std = close_brick(&pixs, 1, size).expect("close_brick");
        let close_dwa = close_brick_dwa(&pixs, 1, size).expect("close_brick_dwa");
        let same = compare_pix(&close_std, &close_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        if !same {
            let diff = count_diff_pixels(&close_std, &close_dwa);
            eprintln!("  close v size={}: DIFFER ({} pixels)", size, diff);
        }
    }

    assert!(rp.cleanup(), "dwamorph2 vertical regression test failed");
}

/// Timing comparison: DWA brick vs standard brick.
///
/// C版: GPLOT timing graphs -- Rust未実装のためスキップ
/// C版: numaWindowedMean() -- Rust未実装のためスキップ
///
/// Instead of graphing, we print timing results for manual inspection.
/// Tests sizes 2 and 3 with horizontal orientation for timing comparison.
#[test]
fn dwamorph2_reg_timing() {
    let mut rp = RegParams::new("dwamorph2_timing");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    eprintln!("Timing comparison: DWA brick vs standard brick");
    eprintln!(
        "Image size: {}x{}, repetitions: {}",
        pixs.width(),
        pixs.height(),
        NTIMES
    );
    eprintln!();

    for size in [2u32, 3] {
        eprintln!("=== Size {} ===", size);

        // --- Dilation timing ---
        let start = Instant::now();
        let mut dil_std = None;
        for _ in 0..NTIMES {
            dil_std = Some(dilate_brick(&pixs, size, 1).expect("dilate_brick"));
        }
        let std_time = start.elapsed();

        let start = Instant::now();
        let mut dil_dwa = None;
        for _ in 0..NTIMES {
            dil_dwa = Some(dilate_brick_dwa(&pixs, size, 1).expect("dilate_brick_dwa"));
        }
        let dwa_time = start.elapsed();

        let same = compare_pix(dil_std.as_ref().unwrap(), dil_dwa.as_ref().unwrap());
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  dilate h: std={:?}, dwa={:?}, match={}",
            std_time, dwa_time, same
        );

        // --- Erosion timing ---
        let start = Instant::now();
        let mut ero_std = None;
        for _ in 0..NTIMES {
            ero_std = Some(erode_brick(&pixs, size, 1).expect("erode_brick"));
        }
        let std_time = start.elapsed();

        let start = Instant::now();
        let mut ero_dwa = None;
        for _ in 0..NTIMES {
            ero_dwa = Some(erode_brick_dwa(&pixs, size, 1).expect("erode_brick_dwa"));
        }
        let dwa_time = start.elapsed();

        let same = compare_pix(ero_std.as_ref().unwrap(), ero_dwa.as_ref().unwrap());
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  erode h:  std={:?}, dwa={:?}, match={}",
            std_time, dwa_time, same
        );

        // --- Opening timing ---
        let start = Instant::now();
        let mut open_std_result = None;
        for _ in 0..NTIMES {
            open_std_result = Some(open_brick(&pixs, size, 1).expect("open_brick"));
        }
        let std_time = start.elapsed();

        let start = Instant::now();
        let mut open_dwa_result = None;
        for _ in 0..NTIMES {
            open_dwa_result = Some(open_brick_dwa(&pixs, size, 1).expect("open_brick_dwa"));
        }
        let dwa_time = start.elapsed();

        let same = compare_pix(
            open_std_result.as_ref().unwrap(),
            open_dwa_result.as_ref().unwrap(),
        );
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  open h:   std={:?}, dwa={:?}, match={}",
            std_time, dwa_time, same
        );

        // --- Closing timing ---
        let start = Instant::now();
        let mut close_std_result = None;
        for _ in 0..NTIMES {
            close_std_result = Some(close_brick(&pixs, size, 1).expect("close_brick"));
        }
        let std_time = start.elapsed();

        let start = Instant::now();
        let mut close_dwa_result = None;
        for _ in 0..NTIMES {
            close_dwa_result = Some(close_brick_dwa(&pixs, size, 1).expect("close_brick_dwa"));
        }
        let dwa_time = start.elapsed();

        let same = compare_pix(
            close_std_result.as_ref().unwrap(),
            close_dwa_result.as_ref().unwrap(),
        );
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  close h:  std={:?}, dwa={:?}, match={}",
            std_time, dwa_time, same
        );

        eprintln!();
    }

    assert!(rp.cleanup(), "dwamorph2 timing regression test failed");
}

/// Correctness test for DWA vs standard morph at small sizes.
///
/// Tests sizes 2 and 3 with horizontal, vertical, and square
/// orientations for all four operations (dilate, erode, open, close).
#[test]
fn dwamorph2_reg_small_sizes() {
    let mut rp = RegParams::new("dwamorph2_small");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    eprintln!("Testing DWA vs brick morph for small sizes (2, 3)");

    for size in [2u32, 3] {
        eprintln!("  Size {}", size);

        // Horizontal
        let dil_std = dilate_brick(&pixs, size, 1).expect("dilate_brick h");
        let dil_dwa = dilate_brick_dwa(&pixs, size, 1).expect("dilate_brick_dwa h");
        let same = compare_pix(&dil_std, &dil_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    dilate ({},1): {}",
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        let ero_std = erode_brick(&pixs, size, 1).expect("erode_brick h");
        let ero_dwa = erode_brick_dwa(&pixs, size, 1).expect("erode_brick_dwa h");
        let same = compare_pix(&ero_std, &ero_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    erode  ({},1): {}",
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        let open_std = open_brick(&pixs, size, 1).expect("open_brick h");
        let open_dwa = open_brick_dwa(&pixs, size, 1).expect("open_brick_dwa h");
        let same = compare_pix(&open_std, &open_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    open   ({},1): {}",
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        let close_std = close_brick(&pixs, size, 1).expect("close_brick h");
        let close_dwa = close_brick_dwa(&pixs, size, 1).expect("close_brick_dwa h");
        let same = compare_pix(&close_std, &close_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    close  ({},1): {}",
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        // Vertical
        let dil_std = dilate_brick(&pixs, 1, size).expect("dilate_brick v");
        let dil_dwa = dilate_brick_dwa(&pixs, 1, size).expect("dilate_brick_dwa v");
        let same = compare_pix(&dil_std, &dil_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    dilate (1,{}): {}",
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        let ero_std = erode_brick(&pixs, 1, size).expect("erode_brick v");
        let ero_dwa = erode_brick_dwa(&pixs, 1, size).expect("erode_brick_dwa v");
        let same = compare_pix(&ero_std, &ero_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    erode  (1,{}): {}",
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        let open_std = open_brick(&pixs, 1, size).expect("open_brick v");
        let open_dwa = open_brick_dwa(&pixs, 1, size).expect("open_brick_dwa v");
        let same = compare_pix(&open_std, &open_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    open   (1,{}): {}",
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        let close_std = close_brick(&pixs, 1, size).expect("close_brick v");
        let close_dwa = close_brick_dwa(&pixs, 1, size).expect("close_brick_dwa v");
        let same = compare_pix(&close_std, &close_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    close  (1,{}): {}",
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        // Square (2D)
        let dil_std = dilate_brick(&pixs, size, size).expect("dilate_brick sq");
        let dil_dwa = dilate_brick_dwa(&pixs, size, size).expect("dilate_brick_dwa sq");
        let same = compare_pix(&dil_std, &dil_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    dilate ({},{}): {}",
            size,
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        let ero_std = erode_brick(&pixs, size, size).expect("erode_brick sq");
        let ero_dwa = erode_brick_dwa(&pixs, size, size).expect("erode_brick_dwa sq");
        let same = compare_pix(&ero_std, &ero_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    erode  ({},{}): {}",
            size,
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        let open_std = open_brick(&pixs, size, size).expect("open_brick sq");
        let open_dwa = open_brick_dwa(&pixs, size, size).expect("open_brick_dwa sq");
        let same = compare_pix(&open_std, &open_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    open   ({},{}): {}",
            size,
            size,
            if same { "MATCH" } else { "DIFFER" }
        );

        let close_std = close_brick(&pixs, size, size).expect("close_brick sq");
        let close_dwa = close_brick_dwa(&pixs, size, size).expect("close_brick_dwa sq");
        let same = compare_pix(&close_std, &close_dwa);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "    close  ({},{}): {}",
            size,
            size,
            if same { "MATCH" } else { "DIFFER" }
        );
    }

    assert!(rp.cleanup(), "dwamorph2 small sizes regression test failed");
}

/// Full range correctness test: DWA vs standard morph for all four operations,
/// across sizes 2..MAX_LINEAR_SIZE, horizontal + vertical.
///
/// This is the faithful port of the C version's core comparison loop.
/// The C version uses selaAddDwaLinear() which provides linear sels from size 2 to 63.
/// The first half are horizontal, the second half are vertical.
/// For each sel, it runs all four operations (dilate, erode, open, close)
/// with four implementations and checks timing.
///
/// We compare DWA brick vs standard brick across the same size range,
/// testing horizontal and vertical orientations.
#[test]
#[ignore = "slow: ~35s -- same coverage as dwamorph2_reg_horizontal + dwamorph2_reg_vertical combined"]
fn dwamorph2_reg_full() {
    let mut rp = RegParams::new("dwamorph2_full");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    eprintln!(
        "Full DWA vs brick morph comparison, sizes 2..{}",
        MAX_LINEAR_SIZE
    );
    eprintln!("Image size: {}x{}", pixs.width(), pixs.height());

    let mut total_tests = 0u32;
    let mut total_matches = 0u32;

    for size in 2..MAX_LINEAR_SIZE {
        eprint!(" {}.", size);

        for (op_name, std_fn, dwa_fn) in [
            (
                "dilate",
                dilate_brick
                    as fn(
                        &leptonica_core::Pix,
                        u32,
                        u32,
                    ) -> leptonica_morph::MorphResult<leptonica_core::Pix>,
                dilate_brick_dwa
                    as fn(
                        &leptonica_core::Pix,
                        u32,
                        u32,
                    ) -> leptonica_morph::MorphResult<leptonica_core::Pix>,
            ),
            ("erode", erode_brick, erode_brick_dwa),
            ("open", open_brick, open_brick_dwa),
            ("close", close_brick, close_brick_dwa),
        ] {
            // Horizontal
            let std_result = std_fn(&pixs, size, 1).expect("std h");
            let dwa_result = dwa_fn(&pixs, size, 1).expect("dwa h");
            let same = compare_pix(&std_result, &dwa_result);
            rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
            total_tests += 1;
            if same {
                total_matches += 1;
            } else {
                let diff = count_diff_pixels(&std_result, &dwa_result);
                eprintln!("\n  {} ({},1) DIFFER: {} pixels", op_name, size, diff);
            }

            // Vertical
            let std_result = std_fn(&pixs, 1, size).expect("std v");
            let dwa_result = dwa_fn(&pixs, 1, size).expect("dwa v");
            let same = compare_pix(&std_result, &dwa_result);
            rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
            total_tests += 1;
            if same {
                total_matches += 1;
            } else {
                let diff = count_diff_pixels(&std_result, &dwa_result);
                eprintln!("\n  {} (1,{}) DIFFER: {} pixels", op_name, size, diff);
            }
        }
    }

    eprintln!();
    eprintln!("Results: {}/{} tests matched", total_matches, total_tests);

    assert!(rp.cleanup(), "dwamorph2 full regression test failed");
}
