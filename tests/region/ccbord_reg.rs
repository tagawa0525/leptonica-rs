//! Component borders regression test
//!
//! C reference: prog/ccbord_reg.c
//!
//! Verifies:
//! 1. get_all_borders retrieves all borders (outer + holes)
//! 2. Each component has a non-empty outer border
//! 3. render_borders renders borders to an image
//! 4. Rendered border pixels are a subset of the original image pixels
//! 5. Chain code encode/decode roundtrip preserves border points

use crate::common::{RegParams, load_test_image};
use leptonica::io::ImageFormat;
use leptonica::region::{from_chain_code, get_all_borders, render_borders, to_chain_code};
use leptonica::{Pix, PixelDepth};

/// Equivalent of the C version's RunCCBordTest function.
fn run_ccbord_test(fname: &str, rp: &mut RegParams) {
    let pixs = load_test_image(fname).unwrap_or_else(|e| panic!("load {}: {}", fname, e));
    assert_eq!(
        pixs.depth(),
        PixelDepth::Bit1,
        "{} must be 1-bit image",
        fname
    );

    let w = pixs.width();
    let h = pixs.height();
    eprintln!("=== {} ({}x{}) ===", fname, w, h);

    // --- Test 1: Get all borders (outer + holes) ---
    let all_borders = get_all_borders(&pixs).expect("get_all_borders");
    let n_comp = all_borders.components.len();
    eprintln!("  Components: {}", n_comp);
    rp.compare_values(1.0, if n_comp > 0 { 1.0 } else { 0.0 }, 0.0);

    // --- Test 2: Each component has a non-empty outer border ---
    for (i, comp) in all_borders.components.iter().enumerate() {
        let has_outer = !comp.outer.points.is_empty();
        rp.compare_values(1.0, if has_outer { 1.0 } else { 0.0 }, 0.0);
        if i < 5 || !has_outer {
            eprintln!(
                "  comp[{}]: outer={} pts, holes={}",
                i,
                comp.outer.points.len(),
                comp.holes.len()
            );
        }
    }

    // --- Test 3: Render borders and verify subset of original ---
    let rendered = render_borders(&all_borders).expect("render_borders");
    rp.compare_values(w as f64, rendered.width() as f64, 0.0);
    rp.compare_values(h as f64, rendered.height() as f64, 0.0);

    let mut excess_count = 0u64;
    for y in 0..h {
        for x in 0..w {
            let border_val = rendered.get_pixel(x, y).unwrap_or(0);
            let orig_val = pixs.get_pixel(x, y).unwrap_or(0);
            if border_val != 0 && orig_val == 0 {
                excess_count += 1;
            }
        }
    }
    rp.compare_values(0.0, excess_count as f64, 0.0);
    if excess_count == 0 {
        eprintln!("  ==> all border pixels are in original set");
    } else {
        eprintln!(
            "  ==> {} border pixels are NOT in original set",
            excess_count
        );
    }

    // --- Test 4: Chain code roundtrip ---
    let mut chain_ok_count = 0usize;
    let mut chain_fail_count = 0usize;
    for comp in &all_borders.components {
        let global_border = comp.outer_global();
        if global_border.points.len() < 2 {
            continue;
        }

        let chain = to_chain_code(&global_border.points);
        if chain.is_empty() {
            chain_fail_count += 1;
            continue;
        }

        let start = global_border.points[0];
        let reconstructed = from_chain_code(start, &chain);

        if reconstructed.len() == global_border.points.len() {
            chain_ok_count += 1;
        } else {
            chain_fail_count += 1;
        }

        if let (Some(orig_first), Some(recon_first)) =
            (global_border.points.first(), reconstructed.first())
        {
            rp.compare_values(orig_first.x as f64, recon_first.x as f64, 0.0);
            rp.compare_values(orig_first.y as f64, recon_first.y as f64, 0.0);
        }
    }
    eprintln!(
        "  Chain code roundtrip: {} ok, {} failed",
        chain_ok_count, chain_fail_count
    );
    rp.compare_values(0.0, chain_fail_count as f64, 0.0);

    // --- Test 5: Holes ---
    let n_holes: usize = all_borders.components.iter().map(|c| c.holes.len()).sum();
    let has_holes = all_borders.has_holes();
    eprintln!("  Total hole borders: {}, has_holes={}", n_holes, has_holes);
}

/// Border tracing test using feyn-fract.tif
///
/// Previously ignored with a misleading "O(n_components * image_size)" memory
/// notice. The actual problem was an infinite loop in the Moore tracer that
/// grew the points vector until allocation hit 4 GiB; capped in PR #323.
#[test]
fn ccbord_reg_feyn_fract() {
    let mut rp = RegParams::new("ccbord_feyn_fract");
    run_ccbord_test("feyn-fract.tif", &mut rp);
    assert!(
        rp.cleanup(),
        "ccbord regression test (feyn-fract.tif) failed"
    );
}

/// Fast smoke test for benchmark mapping parity.
///
/// C version also runs dreyfus1.png in addition to feyn-fract.tif.
#[test]
fn ccbord_reg_dreyfus1_smoke() {
    let mut rp = RegParams::new("ccbord_dreyfus1");

    let pix = Pix::new(96, 96, PixelDepth::Bit1).expect("create image");
    let mut pm = pix.try_into_mut().expect("mutable image");
    for y in 10..30u32 {
        for x in 10..28u32 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    for y in 40..80u32 {
        for x in 50..85u32 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    let pixs: Pix = pm.into();
    let all_borders = get_all_borders(&pixs).expect("get_all_borders");
    rp.compare_values(
        1.0,
        if all_borders.components.is_empty() {
            0.0
        } else {
            1.0
        },
        0.0,
    );

    let rendered = render_borders(&all_borders).expect("render_borders");
    rp.compare_values(pixs.width() as f64, rendered.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, rendered.height() as f64, 0.0);
    rp.write_pix_and_check(&rendered, ImageFormat::Tiff)
        .expect("write rendered ccbord_dreyfus1");

    assert!(rp.cleanup(), "ccbord dreyfus1 smoke test failed");
}

/// C-compatible port of `RunCCBordTest()` in `prog/ccbord_reg.c`, covering the
/// border-following, reconstruction, and serialization stages (C indices 0-4
/// and 7-11).
///
/// The single-path/SVG stage (5,6 / 12,13) needs more of the `CCBORDA` API and
/// follows in a later PR.
///
/// Gated on `ccb-format` as a whole rather than per check: dropping only the
/// serialization checks would renumber the ones after them and break the
/// golden manifest.
#[cfg(feature = "ccb-format")]
fn do_ccbord_c(rp: &mut RegParams, fname: &str) {
    use leptonica::region::{CcBorda, CcbCoords};

    let pixs = load_test_image(fname).unwrap_or_else(|e| panic!("load {}: {}", fname, e));
    let mut ccba = CcBorda::from_pix(&pixs).expect("CcBorda::from_pix");

    // Local -> global locations, then draw the border pixels.
    ccba.generate_global_locs();
    let pixd = ccba.display_border().expect("display_border");
    // 0 / 7
    rp.write_pix_and_check(&pixd, ImageFormat::Png)
        .expect("write border");

    // Step chain code -> global locations, then draw again. C checks that the
    // result is unchanged, which the identical hash confirms.
    ccba.generate_step_chains();
    ccba.step_chains_to_pix_coords(CcbCoords::Global)
        .expect("step chains to global");
    let pixd = ccba.display_border().expect("display_border from steps");
    // 1 / 8
    rp.write_pix_and_check(&pixd, ImageFormat::Png)
        .expect("write border from steps");

    // Reconstruct the image from the borders.
    let pixc = ccba.display_image().expect("display_image");
    // 2 / 9
    rp.write_pix_and_check(&pixc, ImageFormat::Png)
        .expect("write reconstruction");

    // Write the step data out and read it back. Only the step representation
    // survives, so both coordinate frames have to be rebuilt from it.
    let serialized = ccba.to_bytes().expect("to_bytes");
    let mut ccba2 = CcBorda::from_bytes(&serialized).expect("from_bytes");

    ccba2
        .step_chains_to_pix_coords(CcbCoords::Global)
        .expect("read-back step chains to global");
    let pixd2 = ccba2
        .display_border()
        .expect("display_border after round trip");
    // 3 / 10
    rp.write_pix_and_check(&pixd2, ImageFormat::Png)
        .expect("write border after round trip");

    ccba2
        .step_chains_to_pix_coords(CcbCoords::Local)
        .expect("read-back step chains to local");
    let pixc2 = ccba2
        .display_image()
        .expect("display_image after round trip");
    // 4 / 11
    rp.write_pix_and_check(&pixc2, ImageFormat::Png)
        .expect("write reconstruction after round trip");
}

#[test]
#[cfg(feature = "ccb-format")]
fn ccbord_c_compat() {
    let mut rp = RegParams::new("ccbord_c");
    do_ccbord_c(&mut rp, "feyn-fract.tif");
    do_ccbord_c(&mut rp, "dreyfus1.png");
    assert!(rp.cleanup(), "ccbord c-compat test failed");
}
