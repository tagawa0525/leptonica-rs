//! Warper regression test
//!
//! Tests random harmonic warp, stereoscopic warp, and horizontal stretch
//! operations. The C version generates 50 warped variants per parameter set
//! and compares tiled display output. It also tests pixSimpleCaptcha.
//!
//! Partial migration: pixSimpleCaptcha is not available in leptonica-transform.
//! Tests random_harmonic_warp with reproducibility checks, warp_stereoscopic
//! with default and custom parameters, and stretch_horizontal.
//!
//! # See also
//!
//! C Leptonica: `prog/warper_reg.c`

use crate::common::RegParams;
use leptonica::core::Pixa;
use leptonica::io::ImageFormat;
use leptonica::transform::warper::simple_captcha;
use leptonica::transform::{
    StereoscopicParams, WarpDirection, WarpFill, WarpOperation, WarpType, random_harmonic_warp,
    stretch_horizontal, warp_stereoscopic,
};
use leptonica::{Pix, PixelDepth};

/// Test random harmonic warp reproducibility (C checks 0-3).
///
/// Verifies that random_harmonic_warp produces consistent results for
/// a given seed, and dimensions are preserved across parameter sets.
#[test]
fn warper_reg_random_harmonic() {
    let mut rp = RegParams::new("warper_rhw");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit8);

    let w = pix.width();
    let h = pix.height();

    // C test uses 4 parameter sets; verify the first two
    let warped1 = random_harmonic_warp(&pix, 3.0, 5.0, 0.11, 0.11, 4, 4, 0, 255)
        .expect("random_harmonic_warp set0 seed0");
    rp.compare_values(w as f64, warped1.width() as f64, 0.0);
    rp.compare_values(h as f64, warped1.height() as f64, 0.0);
    rp.write_pix_and_check(&warped1, ImageFormat::Png)
        .expect("write warped1");

    let warped2 = random_harmonic_warp(&pix, 4.0, 6.0, 0.10, 0.13, 3, 3, 7, 255)
        .expect("random_harmonic_warp set1 seed7");
    rp.compare_values(w as f64, warped2.width() as f64, 0.0);
    rp.compare_values(h as f64, warped2.height() as f64, 0.0);

    // Same seed should produce the same result (deterministic RNG)
    let warped1b = random_harmonic_warp(&pix, 3.0, 5.0, 0.11, 0.11, 4, 4, 0, 255)
        .expect("random_harmonic_warp set0 seed0 repeat");
    rp.compare_pix(&warped1, &warped1b);

    assert!(rp.cleanup(), "warper random harmonic test failed");
}

/// Test warp_stereoscopic with default and custom parameters.
///
/// Verifies stereoscopic warp produces 32bpp output at original dimensions.
#[test]
fn warper_reg_stereoscopic() {
    let mut rp = RegParams::new("warper_stereo");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();

    // Default params: zbend=20, zshift_top=15, zshift_bottom=-15,
    // ybend_top=30, ybend_bottom=0, red_left=true
    let result =
        warp_stereoscopic(&pix, StereoscopicParams::default()).expect("warp_stereoscopic default");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result warper_stereo");
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Flat params: only horizontal shift, no vertical bending
    let flat_params = StereoscopicParams {
        zbend: 10,
        zshift_top: 0,
        zshift_bottom: 0,
        ybend_top: 0,
        ybend_bottom: 0,
        red_left: false,
    };
    let flat_result = warp_stereoscopic(&pix, flat_params).expect("warp_stereoscopic flat");
    rp.compare_values(w as f64, flat_result.width() as f64, 0.0);
    rp.compare_values(h as f64, flat_result.height() as f64, 0.0);

    assert!(rp.cleanup(), "warper stereoscopic test failed");
}

/// Test stretch_horizontal on 8bpp (C warp stretch portion).
///
/// Verifies horizontal stretch preserves image height and produces
/// valid output for different warp types and directions.
#[test]
fn warper_reg_stretch_horizontal() {
    let mut rp = RegParams::new("warper_stretch");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let h = pix.height();

    // Quadratic left stretch with linear interpolation
    let stretched_q = stretch_horizontal(
        &pix,
        WarpDirection::ToLeft,
        WarpType::Quadratic,
        30,
        WarpOperation::Interpolated,
        WarpFill::White,
    )
    .expect("stretch_horizontal quadratic left LI");
    rp.compare_values(h as f64, stretched_q.height() as f64, 0.0);
    rp.compare_values(1.0, if stretched_q.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.write_pix_and_check(&stretched_q, ImageFormat::Png)
        .expect("write stretched_q");

    // Linear right stretch with sampling
    let stretched_l = stretch_horizontal(
        &pix,
        WarpDirection::ToRight,
        WarpType::Linear,
        20,
        WarpOperation::Sampled,
        WarpFill::Black,
    )
    .expect("stretch_horizontal linear right sampled");
    rp.compare_values(h as f64, stretched_l.height() as f64, 0.0);

    assert!(rp.cleanup(), "warper stretch horizontal test failed");
}

/// Test pixSimpleCaptcha.
#[test]
fn warper_reg_captcha() {
    let pix = crate::common::load_test_image("weasel8.149g.png").expect("load test image");

    let result = leptonica::transform::warper::simple_captcha(
        &pix,
        10,
        2,
        42,
        leptonica::core::pixel::compose_rgba(200, 60, 60, 0),
        false,
    );
    assert!(result.is_ok());
    let captcha = result.unwrap();
    assert!(captcha.width() > 0);
    assert!(captcha.height() > 0);
    // Captcha should be at least as large as original + border
    assert!(captcha.width() >= pix.width());
}

/// The colour C picks for each tile: three `rand()` draws, packed R, G, B.
///
/// C writes them as one expression whose evaluation order is unspecified, but
/// the reference build takes them left to right, so the first draw is red.
/// Confirmed against the colours in C's own output.
fn next_tile_color(rng: &mut leptonica::core::GlibcRand) -> u32 {
    use leptonica::core::pixel::compose_rgba;
    let r = ((rng.next_u32() >> 16) & 0xff) as u8;
    let g = ((rng.next_u32() >> 16) & 0xff) as u8;
    let b = ((rng.next_u32() >> 16) & 0xff) as u8;
    compose_rgba(r, g, b, 0)
}

/// Lays out the 50 tiles of one check the way C's `pixaDisplayTiledInColumns`
/// call does.
fn tile_check(rp: &mut RegParams, tiles: Vec<Pix>) {
    let mut pixa = Pixa::with_capacity(tiles.len());
    for tile in tiles {
        pixa.push(tile);
    }
    let tiled = pixa
        .display_tiled_in_columns(10, 1.0, 20, 0)
        .expect("display_tiled_in_columns");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("write tiled");
}

/// C-compatible port of `prog/warper_reg.c`.
///
/// Both loops reseed the generator for every image (`pixRandomHarmonicWarp`
/// calls `srand(seed)`), and the colour draws continue from that same stream:
/// after the warp for checks 0-3, before the captcha for checks 4-7.
#[test]
fn warper_c_compat() {
    use leptonica::core::GlibcRand;

    let mut rp = RegParams::new("warper_c");

    let pixs = crate::common::load_test_image("feyn-word.tif").expect("load feyn-word.tif");
    let pixg = pixs
        .add_border(25, 0)
        .expect("add border")
        .convert_to_8()
        .expect("convert to 8bpp");

    const XMAG: [f32; 4] = [3.0, 4.0, 5.0, 7.0];
    const YMAG: [f32; 4] = [5.0, 6.0, 8.0, 10.0];
    const XFREQ: [f32; 4] = [0.11, 0.10, 0.10, 0.12];
    const YFREQ: [f32; 4] = [0.11, 0.13, 0.13, 0.15];
    const NXY: [u32; 4] = [4, 3, 2, 1];

    // 0-3: warp the page directly.
    for k in 0..4 {
        let mut tiles = Vec::with_capacity(50);
        for i in 0..50u32 {
            let seed = 7 * i;
            let warped = random_harmonic_warp(
                &pixg, XMAG[k], YMAG[k], XFREQ[k], YFREQ[k], NXY[k], NXY[k], seed, 255,
            )
            .expect("random_harmonic_warp");
            // The warp consumed 5 * (nx + ny) draws from this seed; the colour
            // takes the next three.
            let mut rng = GlibcRand::new(seed);
            for _ in 0..5 * (NXY[k] + NXY[k]) {
                rng.next_u32();
            }
            let color = next_tile_color(&mut rng);
            tiles.push(warped.colorize_gray(color, false).expect("colorize_gray"));
        }
        // 0 / 1 / 2 / 3
        tile_check(&mut rp, tiles);
    }

    // 4-7: the captcha wrapper. Here the colour is drawn *before* the warp
    // reseeds, so it continues the stream left by the previous image: the last
    // warp above reseeded with 7 * 49 and took 5 * (nx + ny) draws, then its
    // own colour took three more.
    let mut rng = GlibcRand::new(7 * 49);
    for _ in 0..5 * (NXY[3] + NXY[3]) + 3 {
        rng.next_u32();
    }
    for nterms in 1..=4u32 {
        let mut tiles = Vec::with_capacity(50);
        for i in 0..50u32 {
            let color = next_tile_color(&mut rng);
            let seed = 7 * i;
            tiles.push(
                simple_captcha(&pixs, 25, nterms, seed, color, false).expect("simple_captcha"),
            );
            // `simple_captcha` reseeded the stream and used 5 * 2 * nterms
            // draws; the next colour continues from there.
            rng = GlibcRand::new(seed);
            for _ in 0..10 * nterms {
                rng.next_u32();
            }
        }
        // 4 / 5 / 6 / 7
        tile_check(&mut rp, tiles);
    }

    assert!(rp.cleanup(), "warper c-compat test failed");
}
