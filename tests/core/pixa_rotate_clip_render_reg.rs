//! Regression tests for plan 123 (Pixa rotate / clip / render 3 関数).

use leptonica::core::Box;
use leptonica::core::pixa::Pixa;
use leptonica::transform::{RotateMethod, RotateOptions};
use leptonica::{Pix, PixMut, PixelDepth};

fn small_pixa() -> Pixa {
    let mut pa = Pixa::new();
    let mut p1 = PixMut::new(8, 6, PixelDepth::Bit8).unwrap();
    for y in 0..6 {
        for x in 0..8 {
            p1.set_pixel(x, y, (x + y) * 10).unwrap();
        }
    }
    pa.push_with_box(p1.into(), Box::new(0, 0, 8, 6).unwrap());
    let mut p2 = PixMut::new(4, 4, PixelDepth::Bit8).unwrap();
    for y in 0..4 {
        for x in 0..4 {
            p2.set_pixel(x, y, x * 16 + y * 32).unwrap();
        }
    }
    pa.push_with_box(p2.into(), Box::new(8, 0, 4, 4).unwrap());
    pa
}

// -- rotate ------------------------------------------------------------

#[test]

fn pixa_rotate_small_angle_is_clone() {
    let pa = small_pixa();
    let opts = RotateOptions::default();
    let out = pa.rotate(0.0001, &opts).unwrap();
    assert_eq!(out.pix_slice().len(), 2);
    assert_eq!(out.get(0).unwrap().width(), 8);
    assert_eq!(out.get(1).unwrap().width(), 4);
}

#[test]

fn pixa_rotate_45deg_runs() {
    let pa = small_pixa();
    let opts = RotateOptions {
        method: RotateMethod::Sampling,
        expand: true,
        ..RotateOptions::default()
    };
    // 45 degrees, large enough to actually rotate
    let out = pa.rotate(std::f32::consts::FRAC_PI_4, &opts).unwrap();
    assert_eq!(out.pix_slice().len(), 2);
    // Expanded output should be larger than the input
    assert!(out.get(0).unwrap().width() >= 8);
}

#[test]

fn pixa_rotate_empty() {
    let pa = Pixa::new();
    let out = pa.rotate(0.5, &RotateOptions::default()).unwrap();
    assert_eq!(out.pix_slice().len(), 0);
}

// -- clip_to_pix -------------------------------------------------------

#[test]

fn pixa_clip_to_pix_1bpp_and() {
    // 16x16 pixs, all foreground (1bpp). Mask each Pix in pixa is also all 1s
    // and the box covers the full area -> AND should give all 1s.
    let mut pixs = PixMut::new(16, 16, PixelDepth::Bit1).unwrap();
    for y in 0..16 {
        for x in 0..16 {
            pixs.set_pixel(x, y, 1).unwrap();
        }
    }
    let pixs: Pix = pixs.into();

    let mut pa = Pixa::new();
    let mut m = PixMut::new(8, 8, PixelDepth::Bit1).unwrap();
    for y in 0..8 {
        for x in 0..8 {
            m.set_pixel(x, y, 1).unwrap();
        }
    }
    pa.push_with_box(m.into(), Box::new(2, 2, 8, 8).unwrap());

    let out = pa.clip_to_pix(&pixs).unwrap();
    assert_eq!(out.pix_slice().len(), 1);
    let result = out.get(0).unwrap();
    assert_eq!(result.width(), 8);
    assert_eq!(result.height(), 8);
    // Result should still be all 1s (1 AND 1 = 1)
    for y in 0..8 {
        for x in 0..8 {
            assert_eq!(result.get_pixel(x, y), Some(1));
        }
    }
}

#[test]

fn pixa_clip_to_pix_partial_overlap() {
    // pixs has only the top half set; clipping with a box covering the
    // whole 16x16 should leave only the top half as 1s after AND.
    let mut pixs = PixMut::new(16, 16, PixelDepth::Bit1).unwrap();
    for y in 0..8 {
        for x in 0..16 {
            pixs.set_pixel(x, y, 1).unwrap();
        }
    }
    let pixs: Pix = pixs.into();

    let mut pa = Pixa::new();
    let mut m = PixMut::new(16, 16, PixelDepth::Bit1).unwrap();
    for y in 0..16 {
        for x in 0..16 {
            m.set_pixel(x, y, 1).unwrap();
        }
    }
    pa.push_with_box(m.into(), Box::new(0, 0, 16, 16).unwrap());

    let out = pa.clip_to_pix(&pixs).unwrap();
    let result = out.get(0).unwrap();
    let mut top_ones = 0;
    let mut bottom_ones = 0;
    for y in 0..16 {
        for x in 0..16 {
            let v = result.get_pixel(x, y).unwrap();
            if y < 8 {
                top_ones += v as i32;
            } else {
                bottom_ones += v as i32;
            }
        }
    }
    assert_eq!(top_ones, 16 * 8);
    assert_eq!(bottom_ones, 0);
}

// -- render_component --------------------------------------------------

#[test]

fn pixa_render_component_into_existing() {
    // pixs is a 16x16 1bpp zero image. Pixa has one 4x4 all-ones Pix
    // at box (2,3). After rendering, the (2..6, 3..7) region should be 1.
    let pixs: Pix = PixMut::new(16, 16, PixelDepth::Bit1).unwrap().into();

    let mut pa = Pixa::new();
    let mut comp = PixMut::new(4, 4, PixelDepth::Bit1).unwrap();
    for y in 0..4 {
        for x in 0..4 {
            comp.set_pixel(x, y, 1).unwrap();
        }
    }
    pa.push_with_box(comp.into(), Box::new(2, 3, 4, 4).unwrap());

    let out = pa.render_component(Some(&pixs), 0).unwrap();
    for y in 0..16 {
        for x in 0..16 {
            let expected = if (2..6).contains(&x) && (3..7).contains(&y) {
                1
            } else {
                0
            };
            assert_eq!(out.get_pixel(x, y), Some(expected), "x={x} y={y}");
        }
    }
}

#[test]

fn pixa_render_component_creates_canvas_when_none() {
    let mut pa = Pixa::new();
    let mut comp = PixMut::new(3, 3, PixelDepth::Bit1).unwrap();
    for y in 0..3 {
        for x in 0..3 {
            comp.set_pixel(x, y, 1).unwrap();
        }
    }
    pa.push_with_box(comp.into(), Box::new(5, 7, 3, 3).unwrap());
    // Extra component with a box that extends the extent
    let comp2: Pix = PixMut::new(4, 5, PixelDepth::Bit1).unwrap().into();
    pa.push_with_box(comp2, Box::new(10, 12, 4, 5).unwrap());

    let out = pa.render_component(None, 0).unwrap();
    // Canvas size should encompass the union of both boxes:
    // x: 5..14, y: 7..17 -> width 14, height 17 from origin (0,0).
    assert_eq!(out.width(), 14);
    assert_eq!(out.height(), 17);
    // The first component at (5,7) should appear as 1s
    for y in 7..10 {
        for x in 5..8 {
            assert_eq!(out.get_pixel(x, y), Some(1));
        }
    }
    // Region outside the rendered component must remain 0
    assert_eq!(out.get_pixel(0, 0), Some(0));
    assert_eq!(out.get_pixel(13, 16), Some(0));
}

#[test]

fn pixa_render_component_index_out_of_range_errors() {
    let pa = Pixa::new();
    assert!(pa.render_component(None, 0).is_err());
}

#[test]

fn pixa_render_component_rejects_non_1bpp() {
    let mut pa = Pixa::new();
    let comp: Pix = PixMut::new(4, 4, PixelDepth::Bit8).unwrap().into();
    pa.push_with_box(comp, Box::new(0, 0, 4, 4).unwrap());
    assert!(pa.render_component(None, 0).is_err());
}
