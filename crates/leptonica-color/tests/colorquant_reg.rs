//! Color quantization regression test
//!
//! C version: reference/leptonica/prog/colorquant_reg.c
//! Tests MedianCut and Octree quantization with various parameters.

use leptonica_color::{
    MedianCutOptions, OctreeOptions, median_cut_quant, median_cut_quant_simple, octree_quant,
    octree_quant_256,
};
use leptonica_core::{Pix, PixelDepth, color};
use leptonica_test::{RegParams, load_test_image};

fn scale_to_max_width(pix: &Pix, max_width: u32) -> Pix {
    let w = pix.width();
    if w <= max_width {
        return pix.clone();
    }
    let factor = max_width as f64 / w as f64;
    let new_w = (w as f64 * factor) as u32;
    let new_h = (pix.height() as f64 * factor) as u32;
    let out = Pix::new(new_w, new_h, pix.depth()).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..new_h {
        for x in 0..new_w {
            let src_x = ((x as f64 / factor) as u32).min(w - 1);
            let src_y = ((y as f64 / factor) as u32).min(pix.height() - 1);
            let pixel = pix.get_pixel_unchecked(src_x, src_y);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    out_mut.into()
}

fn create_color_gradient(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 255) / w.max(1)) as u8;
            let g = ((y * 255) / h.max(1)) as u8;
            let b = (128u32.wrapping_add((x + y) * 64 / (w + h).max(1))) as u8;
            let pixel = color::compose_rgb(r, g, b);
            pix_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    pix_mut.into()
}

fn test_image(pix: &Pix, name: &str, rp: &mut RegParams) {
    let pixs = scale_to_max_width(pix, 350);
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("  Testing image '{}': {}x{}", name, w, h);

    for &max_colors in &[16u32, 128, 256] {
        let result = median_cut_quant(
            &pixs,
            &MedianCutOptions {
                max_colors,
                min_box_pixels: 1,
            },
        );
        match result {
            Ok(quantized) => {
                rp.compare_values(8.0, quantized.depth().bits() as f64, 0.0);
                let cmap = quantized.colormap().expect("should have colormap");
                rp.compare_values(
                    1.0,
                    if cmap.len() <= max_colors as usize {
                        1.0
                    } else {
                        0.0
                    },
                    0.0,
                );
                rp.compare_values(w as f64, quantized.width() as f64, 0.0);
                rp.compare_values(h as f64, quantized.height() as f64, 0.0);
            }
            Err(e) => {
                eprintln!("    median_cut(max_colors={}) FAILED: {}", max_colors, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    {
        let result = octree_quant_256(&pixs);
        match result {
            Ok(quantized) => {
                rp.compare_values(8.0, quantized.depth().bits() as f64, 0.0);
                let cmap = quantized.colormap().expect("should have colormap");
                rp.compare_values(1.0, if cmap.len() <= 256 { 1.0 } else { 0.0 }, 0.0);
                rp.compare_values(w as f64, quantized.width() as f64, 0.0);
                rp.compare_values(h as f64, quantized.height() as f64, 0.0);
            }
            Err(e) => {
                eprintln!("    octree_quant_256 FAILED: {}", e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    for &max_colors in &[128u32, 240] {
        let result = octree_quant(
            &pixs,
            &OctreeOptions {
                max_colors: max_colors.min(256),
            },
        );
        match result {
            Ok(quantized) => {
                rp.compare_values(8.0, quantized.depth().bits() as f64, 0.0);
                let cmap = quantized.colormap().expect("should have colormap");
                rp.compare_values(
                    1.0,
                    if cmap.len() <= max_colors as usize {
                        1.0
                    } else {
                        0.0
                    },
                    0.0,
                );
            }
            Err(e) => {
                eprintln!("    octree_quant(max_colors={}) FAILED: {}", max_colors, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }
}

#[test]
fn colorquant_reg() {
    let mut rp = RegParams::new("colorquant");

    let test_images: Vec<(&str, Pix)> = {
        let mut images = Vec::new();
        if let Ok(pix) = load_test_image("marge.jpg") {
            images.push(("marge.jpg", pix));
        }
        if let Ok(pix) = load_test_image("test24.jpg") {
            images.push(("test24.jpg", pix));
        }
        images.push(("synthetic_gradient", create_color_gradient(200, 150)));
        images
    };

    for (name, pix) in &test_images {
        if pix.depth() != PixelDepth::Bit32 {
            continue;
        }
        test_image(pix, name, &mut rp);
    }

    // Edge cases
    {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..50u32 {
            for x in 0..50u32 {
                pm.set_pixel_unchecked(x, y, color::compose_rgb(128, 64, 32));
            }
        }
        let uniform_pix: Pix = pm.into();

        let mc = median_cut_quant_simple(&uniform_pix, 16).unwrap();
        rp.compare_values(8.0, mc.depth().bits() as f64, 0.0);

        let ot = octree_quant_256(&uniform_pix).unwrap();
        rp.compare_values(8.0, ot.depth().bits() as f64, 0.0);
    }

    let pix8 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(median_cut_quant_simple(&pix8, 16).is_err());
    assert!(octree_quant_256(&pix8).is_err());

    let pix32 = create_color_gradient(10, 10);
    assert!(median_cut_quant_simple(&pix32, 0).is_err());
    assert!(median_cut_quant_simple(&pix32, 257).is_err());

    assert!(rp.cleanup(), "colorquant regression test failed");
}
