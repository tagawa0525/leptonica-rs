//! Color segmentation regression test
//!
//! C version: reference/leptonica/prog/colorseg_reg.c
//! Tests color_segment, color_segment_simple, color_segment_cluster.

use leptonica_color::{
    ColorSegmentOptions, color_segment, color_segment_cluster, color_segment_simple,
};
use leptonica_core::{Pix, PixelDepth, color};
use leptonica_test::{RegParams, load_test_image};

fn create_test_image() -> Pix {
    let w = 120u32;
    let h = 90u32;
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let pixel = if y < h / 3 {
                if x < w / 3 {
                    color::compose_rgb(200, 50, 50)
                } else if x < 2 * w / 3 {
                    color::compose_rgb(50, 200, 50)
                } else {
                    color::compose_rgb(50, 50, 200)
                }
            } else if y < 2 * h / 3 {
                if x < w / 2 {
                    color::compose_rgb(200, 200, 50)
                } else {
                    color::compose_rgb(200, 50, 200)
                }
            } else if x < w / 2 {
                color::compose_rgb(50, 200, 200)
            } else {
                color::compose_rgb(180, 180, 180)
            };
            pix_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    pix_mut.into()
}

fn create_gradient_image() -> Pix {
    let w = 100u32;
    let h = 80u32;
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 255) / w) as u8;
            let g = ((y * 255) / h) as u8;
            pix_mut.set_pixel_unchecked(x, y, color::compose_rgb(r, g, 100));
        }
    }
    pix_mut.into()
}

#[test]
fn colorseg_reg() {
    let mut rp = RegParams::new("colorseg");

    let pixs = match load_test_image("tetons.jpg") {
        Ok(pix) => pix,
        Err(_) => create_test_image(),
    };

    let w = pixs.width();
    let h = pixs.height();

    let configs = [(4u32, 4u32), (8, 8), (16, 16)];

    for &(max_colors, final_colors) in &configs {
        let representative_params = [(40u32, 0u32), (100, 0), (60, 4), (180, 6)];

        for &(maxdist, _selsize) in &representative_params {
            let options = ColorSegmentOptions {
                max_dist: maxdist,
                max_colors,
                sel_size: 0,
                final_colors,
            };

            match color_segment(&pixs, &options) {
                Ok(result) => {
                    rp.compare_values(8.0, result.depth().bits() as f64, 0.0);
                    let cmap = result
                        .colormap()
                        .expect("segmented image must have colormap");
                    rp.compare_values(
                        1.0,
                        if cmap.len() <= final_colors as usize {
                            1.0
                        } else {
                            0.0
                        },
                        0.0,
                    );
                    rp.compare_values(w as f64, result.width() as f64, 0.0);
                    rp.compare_values(h as f64, result.height() as f64, 0.0);
                }
                Err(e) => {
                    eprintln!("    color_segment(maxdist={}) FAILED: {}", maxdist, e);
                    rp.compare_values(1.0, 0.0, 0.0);
                }
            }
        }
    }

    // color_segment_simple
    for &final_c in &[3u32, 5, 6, 8] {
        match color_segment_simple(&pixs, final_c) {
            Ok(result) => {
                rp.compare_values(8.0, result.depth().bits() as f64, 0.0);
                let cmap = result.colormap().unwrap();
                rp.compare_values(
                    1.0,
                    if cmap.len() <= final_c as usize {
                        1.0
                    } else {
                        0.0
                    },
                    0.0,
                );
            }
            Err(e) => {
                eprintln!("  color_segment_simple({}) FAILED: {}", final_c, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // Phase 1 only: cluster
    match color_segment_cluster(&pixs, 75, 10) {
        Ok(result) => {
            rp.compare_values(8.0, result.depth().bits() as f64, 0.0);
            let cmap = result.colormap().unwrap();
            rp.compare_values(1.0, if cmap.len() <= 10 { 1.0 } else { 0.0 }, 0.0);
        }
        Err(e) => {
            eprintln!("  cluster(75, 10) FAILED: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }

    // Second image
    let pixs2 = match load_test_image("wyom.jpg") {
        Ok(pix) => pix,
        Err(_) => create_gradient_image(),
    };

    match color_segment_simple(&pixs2, 6) {
        Ok(result) => {
            rp.compare_values(8.0, result.depth().bits() as f64, 0.0);
            rp.compare_values(pixs2.width() as f64, result.width() as f64, 0.0);
            rp.compare_values(pixs2.height() as f64, result.height() as f64, 0.0);
        }
        Err(e) => {
            eprintln!("  segment(6) FAILED: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }

    // Error cases
    let pix8 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(color_segment_simple(&pix8, 5).is_err());
    assert!(color_segment_cluster(&pix8, 75, 10).is_err());
    assert!(color_segment_cluster(&pixs, 75, 0).is_err());
    assert!(color_segment_cluster(&pixs, 75, 257).is_err());

    assert!(rp.cleanup(), "colorseg regression test failed");
}
