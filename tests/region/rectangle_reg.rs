//! Rectangle finding regression test
//!
//! Tests finding the largest rectangle in background/foreground of an image,
//! and finding rectangles associated with single connected components.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/rectangle_reg.c`

/// Test finding largest rectangles iteratively in image background.
///
/// C version loads test1.png, calls pixFindLargestRectangle 20 times
/// (polarity=0 for background), fills each found rectangle, and
/// visualizes results with colored hash boxes.
///
/// The core function `pixFindLargestRectangle` is not yet implemented
/// in the Rust version.
#[test]
#[ignore = "not yet implemented: find_largest_rectangle"]
fn rectangle_reg_largest() {
    // TODO: Implement when pixFindLargestRectangle is available
    // let pixs = load_test_image("test1.png").expect("load");
    // for i in 0..20 {
    //     let box1 = find_largest_rectangle(&pixs, Polarity::Background);
    //     set_in_rect(&mut pixs, &box1);
    // }
}

/// Test finding rectangles within connected components.
///
/// C version loads singlecc.tif, extracts connected components,
/// then calls pixFindRectangleInCC with 4 selection modes
/// (GeometricUnion, GeometricIntersection, LargestArea, SmallestArea)
/// and 2 scan directions (Vertical, Horizontal) = 8 tests.
///
/// The core function `pixFindRectangleInCC` is not yet implemented.
#[test]
#[ignore = "not yet implemented: find_rectangle_in_cc"]
fn rectangle_reg_in_cc() {
    // TODO: Implement when pixFindRectangleInCC is available
}
