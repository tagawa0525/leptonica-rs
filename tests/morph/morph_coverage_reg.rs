//! Coverage tests for morph functions
//!
//! Tests for:
//! - pixMorphSequenceByComponent / pixMorphSequenceByRegion (morphapp.c)
//! - pixGenerateSelBoundary / pixGenerateSelWithRuns / pixGenerateSelRandom (selgen.c)
//! - pixGetRunCentersOnLine / pixGetRunsOnLine / pixSubsampleBoundaryPixels (selgen.c)
//! - selaCreateFromColorPixa (sel1.c)
//! - pixaThinConnected (ccthin.c)
//!
//! # See also
//!
//! C Leptonica: `morphapp.c`, `selgen.c`, `sel1.c`, `ccthin.c`

use leptonica::morph::morphapp::{morph_sequence_by_component, morph_sequence_by_region};
use leptonica::morph::sel::SelElement;
use leptonica::morph::selgen::{
    generate_sel_boundary, generate_sel_random, generate_sel_with_runs, get_run_centers_on_line,
    get_runs_on_line, subsample_boundary_pixels,
};
use leptonica::morph::thin::pixa_thin_connected;
use leptonica::morph::{Connectivity, ThinType};
use leptonica::{Pix, Pixa, PixelDepth, Sarray};

/// Create a binary image with two separate rectangles (two components).
fn make_two_rects(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    // Component 1: top-left rectangle
    for y in 2..10 {
        for x in 2..10 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    // Component 2: bottom-right rectangle
    for y in 20..30 {
        for x in 20..30 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    pm.into()
}

/// Create a binary image with a single filled rectangle.
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

/// Create a binary image with alternating foreground/background runs on a row.
fn make_run_pattern() -> Pix {
    // 40 wide, 10 tall. Row 5 has runs: 5 bg, 10 fg, 5 bg, 10 fg, 10 bg
    let pix = Pix::new(40, 10, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for x in 5..15 {
        pm.set_pixel_unchecked(x, 5, 1);
    }
    for x in 20..30 {
        pm.set_pixel_unchecked(x, 5, 1);
    }
    pm.into()
}

// ============================================================================
// morph_sequence_by_component (pixMorphSequenceByComponent)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_morph_sequence_by_component_basic() {
    let pix = make_two_rects(40, 40);
    let result = morph_sequence_by_component(&pix, "d3.3", 0, 0, 4).unwrap();
    // Dilation should grow each component but result is still 1bpp
    assert_eq!(result.depth(), PixelDepth::Bit1);
    assert_eq!(result.width(), 40);
    assert_eq!(result.height(), 40);
    // Should have more pixels than original due to dilation
    assert!(result.count_pixels() > pix.count_pixels());
}

#[test]
#[ignore = "not yet implemented"]
fn test_morph_sequence_by_component_min_size_filter() {
    let pix = make_two_rects(40, 40);
    // Set min size to filter out the smaller component (8x8)
    let result = morph_sequence_by_component(&pix, "d3.3", 9, 9, 4).unwrap();
    // Only the larger component (10x10) should be processed and present
    assert!(result.count_pixels() > 0);
    // Top-left area should have no foreground (small component filtered)
    let top_left_count: u64 = (2..10u32)
        .flat_map(|y| (2..10u32).map(move |x| (x, y)))
        .filter(|&(x, y)| result.get_pixel_unchecked(x, y) != 0)
        .count() as u64;
    assert_eq!(top_left_count, 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_morph_sequence_by_component_requires_1bpp() {
    let pix = Pix::new(40, 40, PixelDepth::Bit8).unwrap();
    assert!(morph_sequence_by_component(&pix, "d3.3", 0, 0, 4).is_err());
}

// ============================================================================
// morph_sequence_by_region (pixMorphSequenceByRegion)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_morph_sequence_by_region_basic() {
    let pix = make_two_rects(40, 40);
    let mask = make_two_rects(40, 40);
    let result = morph_sequence_by_region(&pix, &mask, "d3.3", 4, 0, 0).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit1);
    assert_eq!(result.width(), 40);
    assert_eq!(result.height(), 40);
    assert!(result.count_pixels() > 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_morph_sequence_by_region_requires_1bpp() {
    let pix = Pix::new(40, 40, PixelDepth::Bit8).unwrap();
    let mask = make_two_rects(40, 40);
    assert!(morph_sequence_by_region(&pix, &mask, "d3.3", 4, 0, 0).is_err());
}

// ============================================================================
// generate_sel_boundary (pixGenerateSelBoundary)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_generate_sel_boundary_basic() {
    let pix = make_rect(20, 20, 5, 5, 15, 15);
    let sel = generate_sel_boundary(&pix, 1, 1, 0, 0, true, true, true, true).unwrap();
    // Should produce a hit-miss Sel
    assert!(sel.hit_count() > 0);
    assert!(sel.miss_count() > 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_generate_sel_boundary_skip_hits() {
    let pix = make_rect(20, 20, 5, 5, 15, 15);
    // Skip=2 means subsample boundary pixels
    let sel = generate_sel_boundary(&pix, 1, 1, 2, 2, true, true, true, true).unwrap();
    assert!(sel.hit_count() > 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_generate_sel_boundary_requires_1bpp() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    assert!(generate_sel_boundary(&pix, 1, 1, 0, 0, true, true, true, true).is_err());
}

// ============================================================================
// generate_sel_with_runs (pixGenerateSelWithRuns)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_generate_sel_with_runs_basic() {
    let pix = make_rect(30, 30, 5, 5, 25, 25);
    let sel = generate_sel_with_runs(&pix, 2, 2, 1, 1, 0, 0, 0, 0).unwrap();
    assert!(sel.hit_count() > 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_generate_sel_with_runs_requires_1bpp() {
    let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
    assert!(generate_sel_with_runs(&pix, 2, 2, 1, 1, 0, 0, 0, 0).is_err());
}

// ============================================================================
// generate_sel_random (pixGenerateSelRandom)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_generate_sel_random_basic() {
    let pix = make_rect(30, 30, 5, 5, 25, 25);
    let sel = generate_sel_random(&pix, 0.5, 0.5, 1, 0, 0, 0, 0).unwrap();
    assert!(sel.hit_count() > 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_generate_sel_random_requires_1bpp() {
    let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
    assert!(generate_sel_random(&pix, 0.5, 0.5, 1, 0, 0, 0, 0).is_err());
}

// ============================================================================
// get_run_centers_on_line (pixGetRunCentersOnLine)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_get_run_centers_on_line_horizontal() {
    let pix = make_run_pattern();
    // Horizontal line at y=5: runs are 5bg, 10fg, 5bg, 10fg, 10bg
    let centers = get_run_centers_on_line(&pix, -1, 5, 1).unwrap();
    // Two foreground runs with length >= 1 → 2 centers
    assert_eq!(centers.len(), 2);
    // First run center: pixels 5..15, center ≈ 10
    let c0 = centers.get(0).unwrap() as u32;
    assert!((9..=10).contains(&c0));
    // Second run center: pixels 20..30, center ≈ 25
    let c1 = centers.get(1).unwrap() as u32;
    assert!((24..=25).contains(&c1));
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_run_centers_on_line_vertical() {
    // Vertical: set x=10, scan column
    let pix = make_run_pattern();
    // At x=10, only y=5 is foreground
    let centers = get_run_centers_on_line(&pix, 10, -1, 1).unwrap();
    assert_eq!(centers.len(), 1);
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_run_centers_on_line_min_length_filter() {
    let pix = make_run_pattern();
    // Only runs with length >= 15 (none qualifies since runs are length 10)
    let centers = get_run_centers_on_line(&pix, -1, 5, 15).unwrap();
    assert_eq!(centers.len(), 0);
}

// ============================================================================
// get_runs_on_line (pixGetRunsOnLine)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_get_runs_on_line_horizontal() {
    let pix = make_run_pattern();
    // Horizontal line at y=5
    let runs = get_runs_on_line(&pix, 0, 5, 39, 5).unwrap();
    // Alternating bg/fg: 5bg, 10fg, 5bg, 10fg, 10bg → 5 runs
    assert!(runs.len() >= 5);
    // First run is background (5 pixels)
    assert!((runs.get(0).unwrap() - 5.0).abs() < 0.01);
    // Second run is foreground (10 pixels)
    assert!((runs.get(1).unwrap() - 10.0).abs() < 0.01);
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_runs_on_line_single_pixel_line() {
    let pix = Pix::new(1, 1, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(0, 0, 1);
    let pix: Pix = pm.into();
    let runs = get_runs_on_line(&pix, 0, 0, 0, 0).unwrap();
    // Single foreground pixel: bg_run=0, fg_run=1
    assert!(runs.len() >= 2);
}

// ============================================================================
// subsample_boundary_pixels (pixSubsampleBoundaryPixels)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_subsample_boundary_pixels_skip_0() {
    let pix = make_rect(20, 20, 5, 5, 15, 15);
    let boundary =
        leptonica::morph::extract_boundary(&pix, leptonica::morph::BoundaryType::Inner).unwrap();
    let pta = subsample_boundary_pixels(&boundary, 0).unwrap();
    // skip=0 returns all foreground pixels
    assert!(!pta.is_empty());
}

#[test]
#[ignore = "not yet implemented"]
fn test_subsample_boundary_pixels_skip_2() {
    let pix = make_rect(20, 20, 5, 5, 15, 15);
    let boundary =
        leptonica::morph::extract_boundary(&pix, leptonica::morph::BoundaryType::Inner).unwrap();
    let all = subsample_boundary_pixels(&boundary, 0).unwrap();
    let sampled = subsample_boundary_pixels(&boundary, 2).unwrap();
    // Subsampled should have fewer or equal points
    assert!(sampled.len() <= all.len());
    assert!(!sampled.is_empty());
}

// ============================================================================
// sela_create_from_color_pixa (selaCreateFromColorPixa)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_sela_create_from_color_pixa_basic() {
    use leptonica::morph::sel::sela_create_from_color_pixa;

    // Create a 3x3 color image: green=hit, red=miss, white=don't care
    let pix = Pix::new(3, 3, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    // Fill white
    for y in 0..3u32 {
        for x in 0..3u32 {
            pm.set_pixel_unchecked(x, y, 0xFFFFFF00);
        }
    }
    // Center = green (hit)
    pm.set_pixel_unchecked(1, 1, 0x00FF0000);
    // Corner = red (miss)
    pm.set_pixel_unchecked(0, 0, 0xFF000000);
    let pix: Pix = pm.into();

    let mut pixa = Pixa::new();
    pixa.push(pix);

    let mut sa = Sarray::new();
    sa.push("test_sel");

    let sela = sela_create_from_color_pixa(&pixa, &sa).unwrap();
    assert_eq!(sela.count(), 1);
    let sel = sela.get(0).unwrap();
    assert_eq!(sel.get_element(1, 1), Some(SelElement::Hit));
    assert_eq!(sel.get_element(0, 0), Some(SelElement::Miss));
}

#[test]
#[ignore = "not yet implemented"]
fn test_sela_create_from_color_pixa_mismatched_lengths() {
    use leptonica::morph::sel::sela_create_from_color_pixa;

    let pixa = Pixa::new();
    let mut sa = Sarray::new();
    sa.push("extra_name");
    assert!(sela_create_from_color_pixa(&pixa, &sa).is_err());
}

// ============================================================================
// pixa_thin_connected (pixaThinConnected)
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_pixa_thin_connected_basic() {
    // Create a Pixa with thick rectangles
    let mut pixa = Pixa::new();
    pixa.push(make_rect(20, 20, 2, 5, 18, 15));
    pixa.push(make_rect(30, 30, 3, 3, 27, 27));

    let result = pixa_thin_connected(&pixa, ThinType::Foreground, Connectivity::Four, 0).unwrap();
    assert_eq!(result.len(), 2);
    // Each thinned image should have fewer pixels than original
    for i in 0..result.len() {
        let orig = pixa.get(i).unwrap();
        let thinned = result.get(i).unwrap();
        assert_eq!(thinned.depth(), PixelDepth::Bit1);
        assert!(thinned.count_pixels() <= orig.count_pixels());
        assert!(thinned.count_pixels() > 0);
    }
}

#[test]
#[ignore = "not yet implemented"]
fn test_pixa_thin_connected_requires_1bpp() {
    let mut pixa = Pixa::new();
    pixa.push(Pix::new(10, 10, PixelDepth::Bit8).unwrap());
    assert!(pixa_thin_connected(&pixa, ThinType::Foreground, Connectivity::Four, 0).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn test_pixa_thin_connected_empty_pixa() {
    let pixa = Pixa::new();
    let result = pixa_thin_connected(&pixa, ThinType::Foreground, Connectivity::Four, 0).unwrap();
    assert_eq!(result.len(), 0);
}
