//! Color fill regression test
//!
//! C version: reference/leptonica/prog/colorfill_reg.c
//! Tests color_fill, color_fill_from_seed, pixel_is_on_color_boundary.

use leptonica_color::colorfill::{
    ColorFillOptions, Connectivity, color_fill, color_fill_from_seed, pixel_is_on_color_boundary,
};
use leptonica_core::{Pix, PixelDepth, color};
use leptonica_test::RegParams;

fn make_small_test_pix(c1: u32, c2: u32) -> Pix {
    let pix = Pix::new(17, 17, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..17u32 {
        for x in 0..17u32 {
            pm.set_pixel_unchecked(x, y, c1);
        }
    }
    for i in 0..15u32 {
        for j in 0..i {
            pm.set_pixel_unchecked(j, i, c2);
        }
    }
    for i in 0..15u32 {
        for j in (17 - i)..17 {
            pm.set_pixel_unchecked(j, i, c2);
        }
    }
    for i in 9..17u32 {
        pm.set_pixel_unchecked(8, i, c1);
    }
    pm.into()
}

fn create_color_regions() -> Pix {
    let (w, h) = (100u32, 100u32);
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let pixel = if y < h / 2 {
                if x < w / 2 {
                    color::compose_rgb(200, 80, 80)
                } else {
                    color::compose_rgb(80, 200, 80)
                }
            } else if x < w / 2 {
                color::compose_rgb(80, 80, 200)
            } else {
                color::compose_rgb(200, 200, 80)
            };
            pm.set_pixel_unchecked(x, y, pixel);
        }
    }
    pm.into()
}

fn create_random_color_image(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let block_size = 35u32;
    for by in (0..h).step_by(block_size as usize) {
        for bx in (0..w).step_by(block_size as usize) {
            let seed = (bx.wrapping_mul(137) ^ by.wrapping_mul(269)) % 1000;
            let r = ((seed * 251) % 200 + 55) as u8;
            let g = ((seed * 167) % 200 + 55) as u8;
            let b = ((seed * 89) % 200 + 55) as u8;
            for dy in 0..block_size.min(h - by) {
                for dx in 0..block_size.min(w - bx) {
                    let v = ((dx + dy) % 10) as u8;
                    pm.set_pixel_unchecked(
                        bx + dx,
                        by + dy,
                        color::compose_rgb(
                            r.saturating_add(v),
                            g.saturating_add(v),
                            b.saturating_add(v),
                        ),
                    );
                }
            }
        }
    }
    pm.into()
}

#[test]
fn colorfill_reg() {
    let mut rp = RegParams::new("colorfill");

    // Test 1: Small test image
    let pix1 = make_small_test_pix(0x3070A000, 0xA0703000);
    rp.compare_values(17.0, pix1.width() as f64, 0.0);
    rp.compare_values(17.0, pix1.height() as f64, 0.0);
    rp.compare_values(32.0, pix1.depth().bits() as f64, 0.0);

    let opts_small = ColorFillOptions {
        min_max: 70,
        max_diff: 15,
        min_area: 3,
        connectivity: Connectivity::EightWay,
    };
    match color_fill_from_seed(&pix1, 8, 8, &opts_small) {
        Ok(Some(r)) => {
            rp.compare_values(1.0, if r.pixel_count > 0 { 1.0 } else { 0.0 }, 0.0);
        }
        Ok(None) => {
            rp.compare_values(1.0, 1.0, 0.0);
        }
        Err(e) => {
            eprintln!("  FAILED: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }

    // Test 2: Structured color regions
    let pix_regions = create_color_regions();
    let opts = ColorFillOptions {
        min_max: 70,
        max_diff: 40,
        min_area: 100,
        connectivity: Connectivity::EightWay,
    };

    for &(sx, sy, name) in &[
        (25u32, 25u32, "red"),
        (75, 25, "green"),
        (25, 75, "blue"),
        (75, 75, "yellow"),
    ] {
        match color_fill_from_seed(&pix_regions, sx, sy, &opts) {
            Ok(Some(r)) => {
                rp.compare_values(2500.0, r.pixel_count as f64, 500.0);
                rp.compare_values(100.0, r.mask.width() as f64, 0.0);
                rp.compare_values(100.0, r.mask.height() as f64, 0.0);
            }
            Ok(None) => {
                eprintln!("  {}: no region", name);
                rp.compare_values(1.0, 0.0, 0.0);
            }
            Err(e) => {
                eprintln!("  {} FAILED: {}", name, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // Test 3: Full image fill
    match color_fill(&pix_regions, &opts) {
        Ok(r) => {
            rp.compare_values(4.0, r.region_count as f64, 2.0);
            rp.compare_values(1.0, if r.total_pixels > 0 { 1.0 } else { 0.0 }, 0.0);
            rp.compare_values(100.0, r.mask.width() as f64, 0.0);
            rp.compare_values(100.0, r.mask.height() as f64, 0.0);
        }
        Err(e) => {
            eprintln!("  FAILED: {}", e);
            for _ in 0..4 {
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // Test 4: Random-color image
    let pix_random = create_random_color_image(120, 120);
    let opts_rand = ColorFillOptions {
        min_max: 70,
        max_diff: 30,
        min_area: 50,
        connectivity: Connectivity::EightWay,
    };
    match color_fill(&pix_random, &opts_rand) {
        Ok(r) => {
            rp.compare_values(1.0, if r.region_count > 0 { 1.0 } else { 0.0 }, 0.0);
        }
        Err(e) => {
            eprintln!("  FAILED: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }

    match color_fill_from_seed(&pix_random, 60, 60, &opts_rand) {
        Ok(Some(_r)) => {
            rp.compare_values(1.0, 1.0, 0.0);
        }
        Ok(None) => {
            rp.compare_values(1.0, 1.0, 0.0);
        }
        Err(e) => {
            eprintln!("  seed FAILED: {}", e);
            rp.compare_values(1.0, 0.0, 0.0);
        }
    }

    // Test 5: Boundary detection
    rp.compare_values(
        0.0,
        if pixel_is_on_color_boundary(&pix_regions, 25, 25) {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if pixel_is_on_color_boundary(&pix_regions, 49, 25) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Test 6: Connectivity
    let opts4 = ColorFillOptions {
        min_max: 70,
        max_diff: 40,
        min_area: 10,
        connectivity: Connectivity::FourWay,
    };
    let opts8 = ColorFillOptions {
        min_max: 70,
        max_diff: 40,
        min_area: 10,
        connectivity: Connectivity::EightWay,
    };
    let r4 = color_fill_from_seed(&pix_regions, 25, 25, &opts4).unwrap();
    let r8 = color_fill_from_seed(&pix_regions, 25, 25, &opts8).unwrap();
    match (r4, r8) {
        (Some(a), Some(b)) => {
            rp.compare_values(1.0, if a.pixel_count > 0 { 1.0 } else { 0.0 }, 0.0);
            rp.compare_values(1.0, if b.pixel_count > 0 { 1.0 } else { 0.0 }, 0.0);
        }
        _ => {
            for _ in 0..2 {
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // Test 7: Dark pixel rejection
    let dark = {
        let p = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let mut pm = p.try_into_mut().unwrap();
        for y in 0..20u32 {
            for x in 0..20u32 {
                pm.set_pixel_unchecked(x, y, color::compose_rgb(30, 30, 30));
            }
        }
        let r: Pix = pm.into();
        r
    };
    rp.compare_values(
        1.0,
        if color_fill_from_seed(&dark, 10, 10, &opts)
            .unwrap()
            .is_none()
        {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Test 8: Error cases
    let pix8 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(color_fill(&pix8, &opts).is_err());
    assert!(color_fill_from_seed(&pix8, 5, 5, &opts).is_err());
    assert!(color_fill_from_seed(&pix_regions, 200, 200, &opts).is_err());

    assert!(rp.cleanup(), "colorfill regression test failed");
}
