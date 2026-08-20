//! Test Pixa extension functions
//!
//! # See also
//!
//! C Leptonica: `pixafunc1.c`

use leptonica::{Box, Pix, Pixa, PixaSortType, PixelDepth, SizeRelation, SortOrder};

fn make_pix(w: u32, h: u32) -> Pix {
    Pix::new(w, h, PixelDepth::Bit8).unwrap()
}

// ============================================================================
// Pixa::select_by_size
// ============================================================================

#[test]
fn test_select_by_size_greater() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(50, 50));
    pixa.push(make_pix(100, 100));

    let result = pixa.select_by_size(30, 30, SizeRelation::GreaterThan);
    assert_eq!(result.len(), 2); // 50x50 and 100x100
}

#[test]
fn test_select_by_size_less() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(50, 50));
    pixa.push(make_pix(100, 100));

    let result = pixa.select_by_size(60, 60, SizeRelation::LessThan);
    assert_eq!(result.len(), 2); // 10x10 and 50x50
}

#[test]
fn test_select_by_size_empty() {
    let pixa = Pixa::new();
    let result = pixa.select_by_size(10, 10, SizeRelation::GreaterThan);
    assert!(result.is_empty());
}

// ============================================================================
// Pixa::select_by_area
// ============================================================================

#[test]
fn test_select_by_area() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10)); // area 100
    pixa.push(make_pix(20, 20)); // area 400
    pixa.push(make_pix(5, 5)); // area 25

    let result = pixa.select_by_area(100, SizeRelation::GreaterThanOrEqual);
    assert_eq!(result.len(), 2); // 100 and 400
}

// ============================================================================
// Pixa::sort
// ============================================================================

#[test]
fn test_sort_by_width_increasing() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(50, 10));
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(30, 10));

    let (sorted, indices) = pixa.sort(PixaSortType::ByWidth, SortOrder::Increasing);
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].width(), 10);
    assert_eq!(sorted[1].width(), 30);
    assert_eq!(sorted[2].width(), 50);
    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn test_sort_by_height_decreasing() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 20));
    pixa.push(make_pix(10, 50));
    pixa.push(make_pix(10, 10));

    let (sorted, _) = pixa.sort(PixaSortType::ByHeight, SortOrder::Decreasing);
    assert_eq!(sorted[0].height(), 50);
    assert_eq!(sorted[1].height(), 20);
    assert_eq!(sorted[2].height(), 10);
}

#[test]
fn test_sort_by_area() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10)); // area 100
    pixa.push(make_pix(5, 5)); // area 25
    pixa.push(make_pix(20, 3)); // area 60

    let (sorted, _) = pixa.sort(PixaSortType::ByArea, SortOrder::Increasing);
    assert_eq!(sorted[0].width() * sorted[0].height(), 25);
    assert_eq!(sorted[1].width() * sorted[1].height(), 60);
    assert_eq!(sorted[2].width() * sorted[2].height(), 100);
}

#[test]
fn test_sort_by_x_with_boxes() {
    let mut pixa = Pixa::new();
    pixa.push_with_box(make_pix(10, 10), Box::new(50, 0, 10, 10).unwrap());
    pixa.push_with_box(make_pix(10, 10), Box::new(10, 0, 10, 10).unwrap());
    pixa.push_with_box(make_pix(10, 10), Box::new(30, 0, 10, 10).unwrap());

    let (sorted, indices) = pixa.sort(PixaSortType::ByX, SortOrder::Increasing);
    assert_eq!(sorted.get_box(0).unwrap().x, 10);
    assert_eq!(sorted.get_box(1).unwrap().x, 30);
    assert_eq!(sorted.get_box(2).unwrap().x, 50);
    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn test_sort_preserves_boxes() {
    let mut pixa = Pixa::new();
    pixa.push_with_box(make_pix(30, 30), Box::new(0, 0, 30, 30).unwrap());
    pixa.push_with_box(make_pix(10, 10), Box::new(100, 100, 10, 10).unwrap());

    let (sorted, _) = pixa.sort(PixaSortType::ByWidth, SortOrder::Increasing);
    assert_eq!(sorted.boxa_count(), 2);
    assert_eq!(sorted.get_box(0).unwrap().x, 100); // 10x10 moved first
    assert_eq!(sorted.get_box(1).unwrap().x, 0); // 30x30 moved second
}

// ============================================================================
// Pixa::sort_by_index
// ============================================================================

#[test]
fn test_sort_by_index() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(20, 20));
    pixa.push(make_pix(30, 30));

    let reordered = pixa.sort_by_index(&[2, 0, 1]).unwrap();
    assert_eq!(reordered[0].width(), 30);
    assert_eq!(reordered[1].width(), 10);
    assert_eq!(reordered[2].width(), 20);
}

#[test]
fn test_sort_by_index_with_boxes() {
    let mut pixa = Pixa::new();
    pixa.push_with_box(make_pix(10, 10), Box::new(0, 0, 10, 10).unwrap());
    pixa.push_with_box(make_pix(20, 20), Box::new(50, 50, 20, 20).unwrap());

    let reordered = pixa.sort_by_index(&[1, 0]).unwrap();
    assert_eq!(reordered[0].width(), 20);
    assert_eq!(reordered.get_box(0).unwrap().x, 50);
    assert_eq!(reordered[1].width(), 10);
    assert_eq!(reordered.get_box(1).unwrap().x, 0);
}

#[test]
fn test_sort_by_index_invalid() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));

    assert!(pixa.sort_by_index(&[0, 1]).is_err()); // index 1 out of bounds
}

// ============================================================================
// Pixa::display
// ============================================================================

#[test]
fn test_display_basic() {
    let mut pixa = Pixa::new();
    // Place a 10x10 white block at (5, 5)
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..10 {
        for x in 0..10 {
            pm.set_pixel_unchecked(x, y, 200);
        }
    }
    pixa.push_with_box(pm.into(), Box::new(5, 5, 10, 10).unwrap());

    let canvas = pixa.display(30, 30).unwrap();
    assert_eq!(canvas.width(), 30);
    assert_eq!(canvas.height(), 30);
    // Check pixel at (10, 10) - inside the placed image
    assert_eq!(canvas.get_pixel(10, 10).unwrap(), 200);
    // Check pixel at (0, 0) - outside. C pixaDisplay initializes canvases
    // deeper than 1bpp with pixSetAll, so the background is white (255).
    assert_eq!(canvas.get_pixel(0, 0).unwrap(), 255);
}

#[test]
fn test_display_auto_size() {
    let mut pixa = Pixa::new();
    pixa.push_with_box(make_pix(10, 10), Box::new(0, 0, 10, 10).unwrap());
    pixa.push_with_box(make_pix(10, 10), Box::new(20, 30, 10, 10).unwrap());

    // w=0, h=0 means auto-compute
    let canvas = pixa.display(0, 0).unwrap();
    assert_eq!(canvas.width(), 30); // 20+10
    assert_eq!(canvas.height(), 40); // 30+10
}

#[test]
fn test_display_empty() {
    let pixa = Pixa::new();
    assert!(pixa.display(0, 0).is_err());
}

// ============================================================================
// Pixa::display_tiled
// ============================================================================

#[test]
fn test_display_tiled_single_row() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(10, 10));

    // C lattice: ncols = (1000-5)/(10+5) = 66 columns are reserved even
    // for 3 images -> wd = 10*66 + 5*67 = 995; one row -> hd = 10 + 5*2.
    let result = pixa.display_tiled(1000, 0, 5).unwrap();
    assert_eq!(result.width(), 995);
    assert_eq!(result.height(), 20);
}

#[test]
fn test_display_tiled_multi_row() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(20, 10));
    pixa.push(make_pix(20, 15));
    pixa.push(make_pix(20, 10));

    // C lattice: wmax=20, hmax=15, ncols = (50-5)/(20+5) = 1, nrows = 3
    // -> wd = 20 + 5*2 = 30, hd = 15*3 + 5*4 = 65.
    let result = pixa.display_tiled(50, 0, 5).unwrap();
    assert_eq!(result.width(), 30);
    assert_eq!(result.height(), 65);
}

#[test]
fn test_display_tiled_empty() {
    let pixa = Pixa::new();
    assert!(pixa.display_tiled(100, 0, 0).is_err());
}

#[test]
fn test_display_tiled_single_image_exceeds_max_width() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(200, 20));

    // C lattice: ncols clamps to 1 even when the image exceeds max_width
    // -> wd = 200 + 5*2 = 210, hd = 20 + 5*2 = 30.
    let result = pixa.display_tiled(100, 0, 5).unwrap();
    assert_eq!(result.width(), 210);
    assert_eq!(result.height(), 30);
}

/// display_tiled must reproduce C pixaDisplayTiled: a regular lattice
/// sized from the max subimage dimensions, with the full column count
/// reserved even when there are fewer images (plan 902 PR 12).
///
/// C: ncols = max(1, (maxwidth - spacing) / (wmax + spacing)),
///    wd = wmax*ncols + spacing*(ncols+1), hd likewise with rows;
///    background = 0 paints white for d > 1 (pixSetAll).
#[test]
fn test_display_tiled_c_lattice() {
    // 3 images of 10x10, maxwidth=1000, spacing=5:
    // ncols = 995/15 = 66, nrows = 1 -> wd = 10*66 + 5*67 = 995, hd = 20.
    let mut pixa = Pixa::new();
    for _ in 0..3 {
        pixa.push(make_pix(10, 10));
    }
    let result = pixa.display_tiled(1000, 0, 5).unwrap();
    assert_eq!((result.width(), result.height()), (995, 20));
    // background=0 on 8bpp paints white
    assert_eq!(result.get_pixel(994, 19).unwrap(), 255);

    // 20x10 + 20x15 + 20x10, maxwidth=50, spacing=5:
    // wmax=20, hmax=15, ncols = 45/25 = 1, nrows = 3
    // -> wd = 20 + 5*2 = 30, hd = 15*3 + 5*4 = 65.
    let mut pixa = Pixa::new();
    pixa.push(make_pix(20, 10));
    pixa.push(make_pix(20, 15));
    pixa.push(make_pix(20, 10));
    let result = pixa.display_tiled(50, 0, 5).unwrap();
    assert_eq!((result.width(), result.height()), (30, 65));

    // Single 200x20 image with maxwidth=100: ncols clamps to 1
    // -> wd = 200 + 5*2 = 210, hd = 20 + 5*2 = 30.
    let mut pixa = Pixa::new();
    pixa.push(make_pix(200, 20));
    let result = pixa.display_tiled(100, 0, 5).unwrap();
    assert_eq!((result.width(), result.height()), (210, 30));
}

#[test]
fn test_sort_by_aspect_ratio() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 20)); // ratio 0.5
    pixa.push(make_pix(20, 10)); // ratio 2.0
    pixa.push(make_pix(10, 10)); // ratio 1.0

    let (sorted, _) = pixa.sort(PixaSortType::ByAspectRatio, SortOrder::Increasing);
    assert_eq!(sorted.len(), 3);
    // 0.5, 1.0, 2.0
    assert_eq!(sorted[0].width(), 10);
    assert_eq!(sorted[0].height(), 20);
    assert_eq!(sorted[2].width(), 20);
    assert_eq!(sorted[2].height(), 10);
}

/// display_tiled_in_columns lays out a fixed column count with per-row
/// heights, matching C pixaDisplayTiledInColumns (plan 902 PR 14).
#[test]
fn test_display_tiled_in_columns_layout() {
    // 5 images of 10x10 into 2 columns, spacing 5, no border:
    // 3 rows; each row is 5 + 10 + 5 + 10 = 30 wide (extent = 35 right
    // edge of the second box), 3 rows of (10 + 5).
    // C: boxes at x = 5, 20; y = 5, 20, 35. Extent = (30, 45).
    // Canvas = extent + spacing = (35, 50).
    let mut pixa = Pixa::new();
    for _ in 0..5 {
        pixa.push(make_pix(10, 10));
    }
    let out = pixa.display_tiled_in_columns(2, 1.0, 5, 0).unwrap();
    assert_eq!((out.width(), out.height()), (35, 50));

    // Mixed heights: a taller image sets its row's height.
    // Row 0: 10x10 and 10x30 -> maxh = 35; row 1 starts at y = 5 + 35 = 40.
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(10, 30));
    pixa.push(make_pix(10, 10));
    let out = pixa.display_tiled_in_columns(2, 1.0, 5, 0).unwrap();
    // Extent: right = 20 + 10 = 30, bottom = 40 + 10 = 50 -> +spacing.
    assert_eq!((out.width(), out.height()), (35, 55));

    // A border widens every cell by 2 * border.
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(10, 10));
    let out = pixa.display_tiled_in_columns(2, 1.0, 5, 3).unwrap();
    // Each cell is 16x16: right = 5 + 16 + 5 + 16 = 42, bottom = 5 + 16.
    assert_eq!((out.width(), out.height()), (47, 26));

    // nx = 0 is rejected; an empty pixa errors.
    assert!(pixa.display_tiled_in_columns(0, 1.0, 5, 0).is_err());
    assert!(Pixa::new().display_tiled_in_columns(2, 1.0, 5, 0).is_err());
}

// ============================================================================
// Pixa::display / Boxa::extent (C pixaDisplay / boxaGetExtent)
// ============================================================================

/// C `boxaGetExtent` skips boxes with a non-positive width or height, both
/// for the `w`/`h` outputs and for the enclosing box.
#[test]
#[ignore = "not yet implemented"]
fn test_boxa_get_extent_skips_invalid() {
    use leptonica::core::{Box, Boxa};

    let mut boxa = Boxa::new();
    boxa.push(Box::new(10, 20, 30, 40).expect("b1"));
    boxa.push(Box::new(5, 5, 100, 10).expect("b2"));
    // A degenerate box must not drag the extent out to (999, 999).
    boxa.push(Box::new_unchecked(999, 999, 0, 0));

    let (w, h, bb) = boxa.get_extent().expect("extent");
    assert_eq!((w, h), (105, 60));
    assert_eq!((bb.x, bb.y, bb.w, bb.h), (5, 5, 100, 55));

    // C returns 0 for an empty boxa rather than failing.
    let (w, h, bb) = Boxa::new().get_extent().expect("empty extent");
    assert_eq!((w, h), (0, 0));
    assert_eq!((bb.x, bb.y, bb.w, bb.h), (0, 0, 0, 0));
}

/// C: pixaDisplay on an empty pixa returns an empty 1 bpp pix of the given
/// size, and only errors when no size is given either.
#[test]
#[ignore = "not yet implemented"]
fn test_pixa_display_empty() {
    use leptonica::PixelDepth;
    use leptonica::core::Pixa;

    let pixa = Pixa::new();
    let pix = pixa.display(40, 30).expect("empty display");
    assert_eq!((pix.width(), pix.height()), (40, 30));
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    assert!(pixa.display(0, 0).is_err());
}

/// C: pixaDisplay sizes the canvas from the boxa extent when a dimension is
/// missing, without compensating for negative origins.
#[test]
#[ignore = "not yet implemented"]
fn test_pixa_display_extent_sizing() {
    use leptonica::core::{Box, Pixa};
    use leptonica::{Pix, PixelDepth};

    let mut pixa = Pixa::new();
    pixa.push(Pix::new(10, 10, PixelDepth::Bit1).expect("p1"));
    pixa.add_box(Box::new(20, 30, 10, 10).expect("b1"));

    let pix = pixa.display(0, 0).expect("extent display");
    assert_eq!((pix.width(), pix.height()), (30, 40));
}
