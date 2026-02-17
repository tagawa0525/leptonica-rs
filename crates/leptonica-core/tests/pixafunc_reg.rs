//! Test Pixa extension functions
//!
//! # See also
//!
//! C Leptonica: `pixafunc1.c`

use leptonica_core::{Box, Pix, Pixa, PixaSortType, PixelDepth, SizeRelation, SortOrder};

fn make_pix(w: u32, h: u32) -> Pix {
    Pix::new(w, h, PixelDepth::Bit8).unwrap()
}

// ============================================================================
// Pixa::select_by_size
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_select_by_size_greater() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(50, 50));
    pixa.push(make_pix(100, 100));

    let result = pixa.select_by_size(30, 30, SizeRelation::GreaterThan);
    assert_eq!(result.len(), 2); // 50x50 and 100x100
}

#[test]
#[ignore = "not yet implemented"]
fn test_select_by_size_less() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(50, 50));
    pixa.push(make_pix(100, 100));

    let result = pixa.select_by_size(60, 60, SizeRelation::LessThan);
    assert_eq!(result.len(), 2); // 10x10 and 50x50
}

#[test]
#[ignore = "not yet implemented"]
fn test_select_by_size_empty() {
    let pixa = Pixa::new();
    let result = pixa.select_by_size(10, 10, SizeRelation::GreaterThan);
    assert!(result.is_empty());
}

// ============================================================================
// Pixa::select_by_area
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
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
#[ignore = "not yet implemented"]
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
#[ignore = "not yet implemented"]
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
#[ignore = "not yet implemented"]
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
#[ignore = "not yet implemented"]
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
#[ignore = "not yet implemented"]
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
#[ignore = "not yet implemented"]
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
#[ignore = "not yet implemented"]
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
#[ignore = "not yet implemented"]
fn test_sort_by_index_invalid() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));

    assert!(pixa.sort_by_index(&[0, 1]).is_err()); // index 1 out of bounds
}

// ============================================================================
// Pixa::display
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
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
    // Check pixel at (0, 0) - outside, should be 0 (background)
    assert_eq!(canvas.get_pixel(0, 0).unwrap(), 0);
}

#[test]
#[ignore = "not yet implemented"]
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
#[ignore = "not yet implemented"]
fn test_display_empty() {
    let pixa = Pixa::new();
    assert!(pixa.display(0, 0).is_err());
}

// ============================================================================
// Pixa::display_tiled
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_display_tiled_single_row() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(10, 10));
    pixa.push(make_pix(10, 10));

    // max_width=1000, all fit in one row
    let result = pixa.display_tiled(1000, 0, 5).unwrap();
    // 3 images of 10px + 2 gaps of 5px = 40px
    assert_eq!(result.width(), 40);
    assert_eq!(result.height(), 10);
}

#[test]
#[ignore = "not yet implemented"]
fn test_display_tiled_multi_row() {
    let mut pixa = Pixa::new();
    pixa.push(make_pix(20, 10));
    pixa.push(make_pix(20, 15));
    pixa.push(make_pix(20, 10));

    // max_width=50, first two images (20+5+20=45) fit, third wraps
    let result = pixa.display_tiled(50, 0, 5).unwrap();
    assert_eq!(result.width(), 45); // first row width
    // height = 15 (first row max) + 5 (spacing) + 10 (second row)
    assert_eq!(result.height(), 30);
}

#[test]
#[ignore = "not yet implemented"]
fn test_display_tiled_empty() {
    let pixa = Pixa::new();
    assert!(pixa.display_tiled(100, 0, 0).is_err());
}
