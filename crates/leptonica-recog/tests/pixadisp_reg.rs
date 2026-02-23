//! Pixa display regression test
//!
//! Tests Pixa display arrangement functions: display_tiled and
//! display_tiled_and_scaled. The C version exhaustively tests many display
//! functions including pixaDisplay, pixaDisplayOnLattice, pixaDisplayTiledInRows,
//! and round-trip tiling consistency.
//!
//! Partial port: Tests display_tiled and display_tiled_and_scaled with
//! images loaded from test data and clipped regions. The C version also
//! tests pixaSplitPix, pixaMakeFromTiledPix, pixaDisplayPairTiledInColumns,
//! and PDF output which are not available in the Rust API.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pixadisp_reg.c`

use leptonica_core::{Pixa, PixelDepth};
use leptonica_test::RegParams;
use leptonica_transform::scale_by_sampling;

/// Test display_tiled with clipped regions from feyn.tif (C test section 1).
///
/// C: pixConnComp(pixs, &pixa, 8)
///    pixaSelectBySize(pixa, 6, 6, L_SELECT_IF_BOTH, ...)
///    pixd = pixaDisplay(pixat, w, h)
///
/// Rust: Clip small rectangular regions from the image to form a Pixa,
/// then test display_tiled.
#[test]
fn pixadisp_reg_display_tiled() {
    let mut rp = RegParams::new("pixadisp_tiled");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    // Clip small regions to create a Pixa (simulating connected component extraction)
    let mut pixa = Pixa::new();
    let w = pix.width();
    let h = pix.height();
    let tile_w = 60;
    let tile_h = 30;
    let mut y = 100;
    while y + tile_h < h && pixa.len() < 100 {
        let mut x = 50;
        while x + tile_w < w && pixa.len() < 100 {
            if let Ok(clip) = pix.clip_rectangle(x, y, tile_w, tile_h) {
                pixa.push(clip);
            }
            x += tile_w + 20;
        }
        y += tile_h + 15;
    }

    rp.compare_values(1.0, if pixa.len() > 0 { 1.0 } else { 0.0 }, 0.0);

    // display_tiled: arrange in rows up to max_width=1000, white background, 5px spacing
    let tiled = pixa.display_tiled(1000, 1, 5).expect("display_tiled");
    rp.compare_values(1.0, if tiled.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if tiled.height() > 0 { 1.0 } else { 0.0 }, 0.0);

    // Width should not exceed max_width + tolerance for the last element
    rp.compare_values(1.0, if tiled.width() <= 1200 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pixadisp display_tiled test failed");
}

/// Test display_tiled_and_scaled with 8bpp output (C test section 5).
///
/// C: pixaDisplayTiledAndScaled(pixa, 8, 250, 5, 0, 10, 2)
///    Scale each component to width 250, arrange in 5 columns,
///    with 10px spacing and 2px border.
#[test]
fn pixadisp_reg_display_tiled_and_scaled() {
    let mut rp = RegParams::new("pixadisp_scaled");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    // Clip regions for Pixa
    let mut pixa = Pixa::new();
    for y in (100..600).step_by(80) {
        for x in (50..800).step_by(120) {
            if let Ok(clip) = pix.clip_rectangle(x, y, 80, 40) {
                pixa.push(clip);
            }
            if pixa.len() >= 50 {
                break;
            }
        }
        if pixa.len() >= 50 {
            break;
        }
    }

    rp.compare_values(1.0, if pixa.len() > 0 { 1.0 } else { 0.0 }, 0.0);

    // display_tiled_and_scaled: 8bpp output, tile_width=250, 5 columns, white bg, 10px spacing, 2px border
    let scaled = pixa
        .display_tiled_and_scaled(PixelDepth::Bit8, 250, 5, 1, 10, 2)
        .expect("display_tiled_and_scaled");

    assert_eq!(scaled.depth(), PixelDepth::Bit8);
    rp.compare_values(1.0, if scaled.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if scaled.height() > 0 { 1.0 } else { 0.0 }, 0.0);

    // With 5 columns at ~250px + spacing + border, width should be reasonable
    rp.compare_values(
        1.0,
        if scaled.width() >= 500 && scaled.width() <= 2000 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(
        rp.cleanup(),
        "pixadisp display_tiled_and_scaled test failed"
    );
}

/// Test display_tiled_and_scaled with 32bpp color output.
///
/// C: pixaDisplayTiledAndScaled(pixa, 32, tilewidth, ncols, bg, spacing, border)
#[test]
fn pixadisp_reg_display_tiled_color() {
    let mut rp = RegParams::new("pixadisp_color");

    // Use marge.jpg (color image) for 32bpp testing
    let pix = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");

    // Create pixa with multiple copies at different sizes
    let mut pixa = Pixa::new();
    pixa.push(pix.clone());
    let small = scale_by_sampling(&pix, 0.5, 0.5).expect("scale 0.5");
    pixa.push(small);
    let tiny = scale_by_sampling(&pix, 0.25, 0.25).expect("scale 0.25");
    pixa.push(tiny);
    pixa.push(pix.clone());

    let tiled = pixa
        .display_tiled_and_scaled(PixelDepth::Bit32, 200, 2, 0, 5, 1)
        .expect("display_tiled_and_scaled 32bpp");

    assert_eq!(tiled.depth(), PixelDepth::Bit32);
    rp.compare_values(1.0, if tiled.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if tiled.height() > 0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pixadisp display_tiled_color test failed");
}

/// Test display_tiled with black background.
///
/// C: pixaDisplayTiled(pixa, maxwidth, 0, spacing)
///    background=0 means black background.
#[test]
fn pixadisp_reg_black_bg() {
    let mut rp = RegParams::new("pixadisp_blackbg");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    // Clip small regions
    let mut pixa = Pixa::new();
    for y in (200..500).step_by(60) {
        for x in (100..600).step_by(100) {
            if let Ok(clip) = pix.clip_rectangle(x, y, 50, 25) {
                pixa.push(clip);
            }
        }
    }

    if pixa.len() == 0 {
        rp.compare_values(1.0, 0.0, 0.0);
        assert!(
            rp.cleanup(),
            "pixadisp black_bg test failed (no components)"
        );
        return;
    }

    // Black background (0)
    let tiled = pixa
        .display_tiled(800, 0, 3)
        .expect("display_tiled black bg");
    rp.compare_values(1.0, if tiled.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if tiled.height() > 0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pixadisp black_bg test failed");
}

/// Test display_tiled_and_scaled with 1bpp output.
///
/// C: pixaDisplayTiledAndScaled(pixa, 1, tilewidth, ncols, bg, spacing, border)
///    Binary output preserves thresholded content.
#[test]
fn pixadisp_reg_display_1bpp() {
    let mut rp = RegParams::new("pixadisp_1bpp");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    // Clip larger regions for 1bpp test
    let mut pixa = Pixa::new();
    for y in (100..800).step_by(100) {
        for x in (50..500).step_by(150) {
            if let Ok(clip) = pix.clip_rectangle(x, y, 100, 50) {
                pixa.push(clip);
            }
            if pixa.len() >= 30 {
                break;
            }
        }
        if pixa.len() >= 30 {
            break;
        }
    }

    if pixa.len() == 0 {
        rp.compare_values(1.0, 0.0, 0.0);
        assert!(rp.cleanup(), "pixadisp 1bpp test failed (no components)");
        return;
    }

    let tiled = pixa
        .display_tiled_and_scaled(PixelDepth::Bit1, 100, 10, 1, 5, 0)
        .expect("display_tiled_and_scaled 1bpp");

    assert_eq!(tiled.depth(), PixelDepth::Bit1);
    rp.compare_values(1.0, if tiled.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if tiled.height() > 0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pixadisp 1bpp test failed");
}

/// Test display_tiled with various images from brev series (C test section 6).
///
/// C: pixaDisplayPairTiledInColumns(pixa1, pixa2, ncols, 1.0, 15, ...)
///    Uses brev.*.jpg images for pair display.
///
/// Rust: We load the brev images into a Pixa and test basic tiled display.
#[test]
fn pixadisp_reg_brev_images() {
    let mut rp = RegParams::new("pixadisp_brev");

    let brev_files = [
        "brev.06.75.jpg",
        "brev.10.75.jpg",
        "brev.14.75.jpg",
        "brev.20.75.jpg",
        "brev.36.75.jpg",
        "brev.53.75.jpg",
        "brev.56.75.jpg",
    ];

    let mut pixa = Pixa::new();
    for name in &brev_files {
        match leptonica_test::load_test_image(name) {
            Ok(img) => pixa.push(img),
            Err(e) => eprintln!("Failed to load {}: {}", name, e),
        }
    }

    rp.compare_values(brev_files.len() as f64, pixa.len() as f64, 0.0);

    if pixa.len() > 0 {
        // Display tiled with scaled 32bpp output
        let tiled = pixa
            .display_tiled_and_scaled(PixelDepth::Bit32, 300, 3, 1, 15, 0)
            .expect("display brev tiled");

        assert_eq!(tiled.depth(), PixelDepth::Bit32);
        rp.compare_values(1.0, if tiled.width() > 0 { 1.0 } else { 0.0 }, 0.0);
        rp.compare_values(1.0, if tiled.height() > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "pixadisp brev_images test failed");
}
