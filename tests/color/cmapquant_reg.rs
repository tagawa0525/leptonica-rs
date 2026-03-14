//! Colormap quantization regression test
//!
//! C version: prog/cmapquant_reg.c

use crate::common::{RegParams, load_test_image};
use leptonica::color::{
    MedianCutOptions, OctreeOptions, few_colors_median_cut_quant_mixed, median_cut_quant,
    median_cut_quant_simple, octcube_quant_from_cmap, octcube_quant_mixed_with_gray, octree_quant,
    octree_quant_256, remove_unused_colors, threshold_to_4bpp,
};
use leptonica::core::pixel;
use leptonica::io::ImageFormat;
use leptonica::{Pix, PixColormap, PixelDepth};

fn load_source_image() -> Pix {
    if let Ok(pix) = load_test_image("lucasta.150.jpg") {
        if pix.depth() == PixelDepth::Bit32 {
            return pix;
        }
        if pix.depth() == PixelDepth::Bit8 {
            return convert_gray_to_rgb(&pix);
        }
    }
    if let Ok(pix) = load_test_image("test24.jpg")
        && pix.depth() == PixelDepth::Bit32
    {
        return pix;
    }
    create_synthetic_text_image(300, 200)
}

fn convert_gray_to_rgb(pix: &Pix) -> Pix {
    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let gray = pix.get_pixel(x, y).unwrap_or(0) as u8;
            let pixel = pixel::compose_rgb(gray, gray, gray);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    out_mut.into()
}

fn create_synthetic_text_image(w: u32, h: u32) -> Pix {
    let out = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let gray = if (x / 10 + y / 15) % 3 == 0 {
                (40 + (x % 7) * 5 + (y % 11) * 3) as u8
            } else {
                (200 + ((x * 3 + y * 7) % 56)) as u8
            };
            let pixel = pixel::compose_rgb(gray, gray, gray);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    out_mut.into()
}

#[allow(clippy::too_many_arguments)]
fn apply_color_to_region(pix: &Pix, x0: u32, y0: u32, w: u32, h: u32, r: u8, g: u8, b: u8) -> Pix {
    let pw = pix.width();
    let ph = pix.height();
    let out = Pix::new(pw, ph, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..ph {
        for x in 0..pw {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (pr, pg, pb) = pixel::extract_rgb(pixel);
            let new_pixel = if x >= x0 && x < x0 + w && y >= y0 && y < y0 + h {
                let avg = ((pr as u32 + pg as u32 + pb as u32) / 3) as u8;
                if avg < 220 {
                    let factor = avg as f32 / 255.0;
                    let nr = (r as f32 * (1.0 - factor) + pr as f32 * factor) as u8;
                    let ng = (g as f32 * (1.0 - factor) + pg as f32 * factor) as u8;
                    let nb = (b as f32 * (1.0 - factor) + pb as f32 * factor) as u8;
                    pixel::compose_rgb(nr, ng, nb)
                } else {
                    pixel
                }
            } else {
                pixel
            };
            out_mut.set_pixel_unchecked(x, y, new_pixel);
        }
    }
    out_mut.into()
}

#[test]
fn cmapquant_reg() {
    let mut rp = RegParams::new("cmapquant");

    let pixs = load_source_image();
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("  Source image: {}x{} depth={}", w, h, pixs.depth().bits());

    let pix_colored = apply_color_to_region(&pixs, 120, 30, 200, 200, 0, 0, 255);
    eprintln!("  Applied color region");

    let pix_rgb = &pix_colored;

    // MedianCut quantization
    {
        let result = median_cut_quant(pix_rgb, &MedianCutOptions::default());
        match result {
            Ok(pix_mc) => {
                rp.compare_values(8.0, pix_mc.depth().bits() as f64, 0.0);
                let has_cmap = pix_mc.colormap().is_some();
                rp.compare_values(1.0, if has_cmap { 1.0 } else { 0.0 }, 0.0);
                if let Some(cmap) = pix_mc.colormap() {
                    rp.compare_values(1.0, if cmap.len() <= 256 { 1.0 } else { 0.0 }, 0.0);
                    eprintln!("  MedianCutQuant default: {} colors", cmap.len());
                }
                rp.compare_values(pix_rgb.width() as f64, pix_mc.width() as f64, 0.0);
                rp.compare_values(pix_rgb.height() as f64, pix_mc.height() as f64, 0.0);
                rp.write_pix_and_check(&pix_mc, ImageFormat::Png)
                    .expect("write pix_mc main_median_cut_quant");
            }
            Err(e) => {
                eprintln!("  MedianCutQuant FAILED: {}", e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // Octree quantization
    {
        let result = octree_quant_256(pix_rgb);
        match result {
            Ok(pix_oct) => {
                rp.compare_values(8.0, pix_oct.depth().bits() as f64, 0.0);
                let has_cmap = pix_oct.colormap().is_some();
                rp.compare_values(1.0, if has_cmap { 1.0 } else { 0.0 }, 0.0);
                if let Some(cmap) = pix_oct.colormap() {
                    rp.compare_values(1.0, if cmap.len() <= 256 { 1.0 } else { 0.0 }, 0.0);
                    eprintln!("  OctreeQuant256: {} colors", cmap.len());
                }
                rp.compare_values(pix_rgb.width() as f64, pix_oct.width() as f64, 0.0);
                rp.compare_values(pix_rgb.height() as f64, pix_oct.height() as f64, 0.0);
            }
            Err(e) => {
                eprintln!("  OctreeQuant256 FAILED: {}", e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    assert!(rp.cleanup(), "cmapquant regression test failed");
}

#[test]
fn cmapquant_requantize_workflow() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("cmapquant_requant");

    let pixs = load_source_image();

    let pix_quant1 = median_cut_quant_simple(&pixs, 32).unwrap();
    let cmap1 = pix_quant1.colormap().expect("should have colormap");
    let n_colors1 = cmap1.len();
    rp.compare_values(1.0, if n_colors1 <= 32 { 1.0 } else { 0.0 }, 0.0);

    for i in 0..n_colors1 {
        let (_r, _g, _b) = cmap1.get_rgb(i).expect("color should exist");
    }

    for y in 0..pix_quant1.height() {
        for x in 0..pix_quant1.width() {
            let idx = pix_quant1.get_pixel(x, y).unwrap_or(0) as usize;
            assert!(idx < n_colors1);
        }
    }

    let pix_rgb = pix_quant1.convert_to_32().unwrap();
    rp.compare_values(32.0, pix_rgb.depth().bits() as f64, 0.0);

    let pix_quant2 = median_cut_quant_simple(&pix_rgb, 64).unwrap();
    let cmap2 = pix_quant2.colormap().expect("should have colormap");
    rp.compare_values(1.0, if cmap2.len() <= 64 { 1.0 } else { 0.0 }, 0.0);

    let pix_quant3 = octree_quant(&pix_rgb, &OctreeOptions { max_colors: 128 }).unwrap();
    let cmap3 = pix_quant3.colormap().expect("should have colormap");
    rp.compare_values(1.0, if cmap3.len() <= 128 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "cmapquant_requant regression test failed");
}

#[test]
fn cmapquant_colormap_size_limits() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("cmapquant_cmap_size");

    let pixs = load_source_image();

    for &max_colors in &[2u32, 4, 8, 16, 32, 64, 128, 256] {
        let result = median_cut_quant_simple(&pixs, max_colors);
        match result {
            Ok(quantized) => {
                let cmap = quantized.colormap().expect("should have colormap");
                let ok = cmap.len() <= max_colors as usize;
                rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
            }
            Err(e) => {
                eprintln!("  median_cut(max={}) error: {}", max_colors, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    for &max_colors in &[4u32, 16, 64, 128, 256] {
        let result = octree_quant(&pixs, &OctreeOptions { max_colors });
        match result {
            Ok(quantized) => {
                let cmap = quantized.colormap().expect("should have colormap");
                let ok = cmap.len() <= max_colors as usize;
                rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
            }
            Err(e) => {
                eprintln!("  octree(max={}) error: {}", max_colors, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    assert!(rp.cleanup(), "cmapquant_cmap_size regression test failed");
}

#[test]
fn cmapquant_colormap_quality() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("cmapquant_cmap_quality");

    let pix_3color = create_3color_image(60, 60);
    let quantized = median_cut_quant_simple(&pix_3color, 16).unwrap();
    let cmap = quantized.colormap().expect("should have colormap");

    let idx_r = cmap.find_nearest(255, 0, 0).unwrap();
    let idx_g = cmap.find_nearest(0, 255, 0).unwrap();
    let idx_b = cmap.find_nearest(0, 0, 255).unwrap();

    rp.compare_values(1.0, if idx_r != idx_g { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if idx_g != idx_b { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if idx_r != idx_b { 1.0 } else { 0.0 }, 0.0);

    let (cr, cg, cb) = cmap.get_rgb(idx_r).unwrap();
    let dist_r = color_distance(cr, cg, cb, 255, 0, 0);
    rp.compare_values(1.0, if dist_r < 100.0 { 1.0 } else { 0.0 }, 0.0);

    let (cr, cg, cb) = cmap.get_rgb(idx_g).unwrap();
    let dist_g = color_distance(cr, cg, cb, 0, 255, 0);
    rp.compare_values(1.0, if dist_g < 100.0 { 1.0 } else { 0.0 }, 0.0);

    let (cr, cg, cb) = cmap.get_rgb(idx_b).unwrap();
    let dist_b = color_distance(cr, cg, cb, 0, 0, 255);
    rp.compare_values(1.0, if dist_b < 100.0 { 1.0 } else { 0.0 }, 0.0);

    rp.compare_values(0.0, if cmap.is_grayscale() { 1.0 } else { 0.0 }, 0.0);

    let pix_gray = create_gray_gradient(60, 60);
    let quant_gray = median_cut_quant_simple(&pix_gray, 16).unwrap();
    let cmap_gray = quant_gray.colormap().expect("should have colormap");
    rp.compare_values(1.0, if cmap_gray.is_grayscale() { 1.0 } else { 0.0 }, 0.0);

    assert!(
        rp.cleanup(),
        "cmapquant_cmap_quality regression test failed"
    );
}

#[test]
fn cmapquant_algorithm_comparison() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("cmapquant_algo_cmp");

    let pixs = load_source_image();
    let max_colors = 64u32;

    let mc_result = median_cut_quant_simple(&pixs, max_colors).unwrap();
    let mc_cmap = mc_result.colormap().unwrap();
    let mc_colors = mc_cmap.len();

    let oct_result = octree_quant(&pixs, &OctreeOptions { max_colors }).unwrap();
    let oct_cmap = oct_result.colormap().unwrap();
    let oct_colors = oct_cmap.len();

    rp.compare_values(
        1.0,
        if mc_colors <= max_colors as usize {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if oct_colors <= max_colors as usize {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    rp.compare_values(pixs.width() as f64, mc_result.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, mc_result.height() as f64, 0.0);
    rp.compare_values(pixs.width() as f64, oct_result.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, oct_result.height() as f64, 0.0);

    rp.compare_values(8.0, mc_result.depth().bits() as f64, 0.0);
    rp.compare_values(8.0, oct_result.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&mc_result, ImageFormat::Png)
        .expect("write mc_result algo_comparison");

    let mc_valid = verify_pixel_indices(&mc_result, mc_colors);
    let oct_valid = verify_pixel_indices(&oct_result, oct_colors);
    rp.compare_values(1.0, if mc_valid { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if oct_valid { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "cmapquant_algo_cmp regression test failed");
}

#[test]
fn cmapquant_threshold_to_4bpp() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("cmapquant_thresh4");

    // threshold_to_4bpp accepts 8bpp or 32bpp (internally converts via ensure_grayscale)
    let pixs = load_source_image(); // 32bpp
    let result = threshold_to_4bpp(&pixs, 8, true).expect("threshold_to_4bpp failed");
    rp.compare_values(4.0, result.depth().bits() as f64, 0.0);
    rp.compare_values(pixs.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, result.height() as f64, 0.0);
    eprintln!(
        "  threshold_to_4bpp: {}x{} depth={}",
        result.width(),
        result.height(),
        result.depth().bits()
    );

    assert!(
        rp.cleanup(),
        "cmapquant_threshold_to_4bpp regression test failed"
    );
}

#[test]
fn cmapquant_octcube_from_cmap() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("cmapquant_octfromcmap");

    let pixs = load_source_image(); // 32bpp
    // Build a small colormap with a few representative colors
    let mut cmap = PixColormap::new(8).expect("colormap creation failed");
    cmap.add_rgba(255, 0, 0, 255).unwrap();
    cmap.add_rgba(0, 255, 0, 255).unwrap();
    cmap.add_rgba(0, 0, 255, 255).unwrap();
    cmap.add_rgba(128, 128, 128, 255).unwrap();
    cmap.add_rgba(255, 255, 255, 255).unwrap();

    let result = octcube_quant_from_cmap(&pixs, &cmap, 4).expect("octcube_quant_from_cmap failed");
    let has_cmap = result.colormap().is_some();
    rp.compare_values(1.0, if has_cmap { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(pixs.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, result.height() as f64, 0.0);
    eprintln!(
        "  octcube_quant_from_cmap: depth={} cmap={}",
        result.depth().bits(),
        has_cmap
    );

    assert!(
        rp.cleanup(),
        "cmapquant_octcube_from_cmap regression test failed"
    );
}

#[test]
fn cmapquant_few_colors_mixed() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("cmapquant_fewmixed");

    let pixs = load_source_image(); // 32bpp
    let result = few_colors_median_cut_quant_mixed(&pixs, 10, 4)
        .expect("few_colors_median_cut_quant_mixed failed");
    rp.compare_values(pixs.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, result.height() as f64, 0.0);
    // Result should have a colormap
    let has_cmap = result.colormap().is_some();
    rp.compare_values(1.0, if has_cmap { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  few_colors_median_cut_quant_mixed: depth={} cmap={}",
        result.depth().bits(),
        has_cmap
    );

    assert!(
        rp.cleanup(),
        "cmapquant_few_colors_mixed regression test failed"
    );
}

#[test]
fn cmapquant_octcube_mixed_gray() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("cmapquant_octmixgray");

    let pixs = load_source_image(); // 32bpp
    let result = octcube_quant_mixed_with_gray(&pixs, 8, 4, 10)
        .expect("octcube_quant_mixed_with_gray failed");
    rp.compare_values(pixs.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, result.height() as f64, 0.0);
    let has_cmap = result.colormap().is_some();
    rp.compare_values(1.0, if has_cmap { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  octcube_quant_mixed_with_gray: depth={} cmap={}",
        result.depth().bits(),
        has_cmap
    );

    assert!(
        rp.cleanup(),
        "cmapquant_octcube_mixed_gray regression test failed"
    );
}

#[test]
fn cmapquant_remove_unused_colors() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("cmapquant_rmunused");

    // Create a colormapped image that doesn't use all colormap entries
    let pixs = load_source_image();
    let quantized = octcube_quant_mixed_with_gray(&pixs, 8, 4, 10)
        .expect("octcube_quant_mixed_with_gray failed for remove test");
    let cmap_before = quantized
        .colormap()
        .expect("quantized should have colormap");
    let count_before = cmap_before.len();

    let result = remove_unused_colors(&quantized).expect("remove_unused_colors failed");
    let cmap_after = result.colormap().expect("result should have colormap");
    let count_after = cmap_after.len();

    // After removal, color count should be <= original
    rp.compare_values(
        1.0,
        if count_after <= count_before {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(pixs.width() as f64, result.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, result.height() as f64, 0.0);
    eprintln!(
        "  remove_unused_colors: {} -> {} colors",
        count_before, count_after
    );

    assert!(
        rp.cleanup(),
        "cmapquant_remove_unused_colors regression test failed"
    );
}

fn create_3color_image(w: u32, h: u32) -> Pix {
    let out = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let pixel = if x < w / 3 {
                pixel::compose_rgb(255, 0, 0)
            } else if x < 2 * w / 3 {
                pixel::compose_rgb(0, 255, 0)
            } else {
                pixel::compose_rgb(0, 0, 255)
            };
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    out_mut.into()
}

fn create_gray_gradient(w: u32, h: u32) -> Pix {
    let out = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let gray = ((x * 255) / w.max(1)) as u8;
            let pixel = pixel::compose_rgb(gray, gray, gray);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    out_mut.into()
}

fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> f64 {
    let dr = r1 as f64 - r2 as f64;
    let dg = g1 as f64 - g2 as f64;
    let db = b1 as f64 - b2 as f64;
    (dr * dr + dg * dg + db * db).sqrt()
}

fn verify_pixel_indices(pix: &Pix, cmap_size: usize) -> bool {
    for y in 0..pix.height() {
        for x in 0..pix.width() {
            let idx = pix.get_pixel(x, y).unwrap_or(0) as usize;
            if idx >= cmap_size {
                return false;
            }
        }
    }
    true
}
