//! Rectangle finding regression test
//!
//! Tests finding the largest rectangle in background/foreground of an image,
//! and finding rectangles associated with single connected components.
//!
//! # See also
//!
//! C Leptonica: `prog/rectangle_reg.c`

use leptonica::core::Box;
use leptonica::region::rectangle::{
    Polarity, RectSelect, ScanDirection, find_large_rectangles, find_largest_rectangle,
    find_rectangle_in_cc,
};
use leptonica::{Pix, PixelDepth};

/// Build a `w x h` 1bpp Pix with the given list of foreground points set.
fn make_1bpp(w: u32, h: u32, fg: &[(u32, u32)]) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.to_mut();
    for &(x, y) in fg {
        pm.set_pixel(x, y, 1).unwrap();
    }
    pm.into()
}

/// `pixFindLargestRectangle` background search and round-trip.
#[test]
#[ignore = "RED: find_largest_rectangle not yet implemented (plan 801)"]
fn rectangle_reg_largest() {
    // Empty 10x6 image — the largest background rectangle is the full canvas.
    let pix = make_1bpp(10, 6, &[]);
    let b = find_largest_rectangle(&pix, Polarity::Background).expect("largest bg");
    assert_eq!((b.x, b.y, b.w, b.h), (0, 0, 10, 6));

    // A single fg pixel at (5, 3) splits the bg into 4 quadrants. The largest
    // bg rectangle has area = max(5*6, 4*6, 10*3, 10*2) = 30 (5x6 left half).
    let pix = make_1bpp(10, 6, &[(5, 3)]);
    let b = find_largest_rectangle(&pix, Polarity::Background).expect("largest bg w/ fg");
    assert_eq!(b.w * b.h, 30);
    assert!(
        (b.x == 0 && b.w == 5 && b.h == 6) || (b.x == 6 && b.w == 4 && b.h == 6),
        "expected 5x6 or 4x6 vertical strip, got {b:?}"
    );

    // Foreground polarity: a fully-black image has the whole canvas as the
    // largest fg rectangle.
    let pix = Pix::new(8, 4, PixelDepth::Bit1).unwrap();
    let mut pm = pix.to_mut();
    for y in 0..4 {
        for x in 0..8 {
            pm.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pm.into();
    let b = find_largest_rectangle(&pix, Polarity::Foreground).expect("largest fg");
    assert_eq!((b.x, b.y, b.w, b.h), (0, 0, 8, 4));
}

/// `pixFindLargeRectangles` greedy multiple rectangles.
#[test]
#[ignore = "RED: find_large_rectangles not yet implemented (plan 801)"]
fn rectangle_reg_large_rectangles() {
    // 10x6 background with one fg pixel at (5, 3). Asking for 3 rectangles
    // should return 3 boxes, each smaller than the previous greedy fill.
    let pix = make_1bpp(10, 6, &[(5, 3)]);
    let boxa = find_large_rectangles(&pix, Polarity::Background, 3).expect("large rects");
    assert_eq!(boxa.len(), 3);
    let a0 = boxa.get(0).unwrap();
    let a1 = boxa.get(1).unwrap();
    let a2 = boxa.get(2).unwrap();
    let area0 = a0.w * a0.h;
    let area1 = a1.w * a1.h;
    let area2 = a2.w * a2.h;
    assert!(
        area0 >= area1 && area1 >= area2,
        "{area0} >= {area1} >= {area2}"
    );

    // nrect = 0 returns an empty Boxa.
    let boxa0 = find_large_rectangles(&pix, Polarity::Background, 0).expect("0 rects");
    assert_eq!(boxa0.len(), 0);

    // 1bpp validation.
    let gray = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    assert!(find_large_rectangles(&gray, Polarity::Background, 1).is_err());
}

/// `pixFindRectangleInCC` finds the inner rectangle of a single CC.
#[test]
#[ignore = "RED: find_rectangle_in_cc not yet implemented (plan 801)"]
fn rectangle_reg_in_cc() {
    // Solid 6x4 fg block — the largest rect is the full block.
    let mut fg = Vec::new();
    for y in 0..4 {
        for x in 0..6 {
            fg.push((x as u32, y as u32));
        }
    }
    let pix = make_1bpp(6, 4, &fg);
    let r = find_rectangle_in_cc(
        &pix,
        None,
        0.5,
        ScanDirection::Horizontal,
        RectSelect::GeometricUnion,
    )
    .expect("in_cc")
    .expect("box found");
    assert_eq!((r.x, r.y, r.w, r.h), (0, 0, 6, 4));

    // LargestArea on a fully-fg block must give the same shape.
    let r2 = find_rectangle_in_cc(
        &pix,
        None,
        0.5,
        ScanDirection::Horizontal,
        RectSelect::LargestArea,
    )
    .expect("in_cc largest")
    .expect("box found");
    assert_eq!((r2.x, r2.y, r2.w, r2.h), (0, 0, 6, 4));

    // Vertical scan: rotated internally but result must be in source coords.
    let r3 = find_rectangle_in_cc(
        &pix,
        None,
        0.5,
        ScanDirection::Vertical,
        RectSelect::GeometricUnion,
    )
    .expect("in_cc vert")
    .expect("box found");
    assert_eq!((r3.x, r3.y, r3.w, r3.h), (0, 0, 6, 4));

    // boxs offsets: pass a 1-pixel-shifted box → result coordinates should
    // be in the original (un-shifted) frame.
    let outer = make_1bpp(
        8,
        6,
        &fg.iter().map(|&(x, y)| (x + 1, y + 1)).collect::<Vec<_>>(),
    );
    let boxs = Box::new(1, 1, 6, 4).unwrap();
    let r4 = find_rectangle_in_cc(
        &outer,
        Some(&boxs),
        0.5,
        ScanDirection::Horizontal,
        RectSelect::GeometricUnion,
    )
    .expect("in_cc with boxs")
    .expect("box found");
    assert_eq!((r4.x, r4.y, r4.w, r4.h), (1, 1, 6, 4));

    // Invalid fract is rejected.
    assert!(
        find_rectangle_in_cc(
            &pix,
            None,
            0.0,
            ScanDirection::Horizontal,
            RectSelect::GeometricUnion,
        )
        .is_err(),
    );
}
