//! GIF image format support
//!
//! Supports reading and writing single-frame GIF images.
//! Animated GIFs (multiple frames) are not supported.

use crate::{IoError, IoResult};
use gif::{ColorOutput, DecodeOptions, Encoder, Frame, Repeat};
use leptonica_core::{Pix, PixColormap, PixelDepth, color};
use std::io::{Read, Write};

/// Read a GIF image
///
/// Reads the first frame of a GIF image. Animated GIFs (multiple frames)
/// will return an error.
pub fn read_gif<R: Read>(reader: R) -> IoResult<Pix> {
    let mut options = DecodeOptions::new();
    options.set_color_output(ColorOutput::Indexed);

    let mut decoder = options
        .read_info(reader)
        .map_err(|e| IoError::DecodeError(format!("GIF decode error: {}", e)))?;

    // Read the first frame
    let frame = decoder
        .read_next_frame()
        .map_err(|e| IoError::DecodeError(format!("GIF frame error: {}", e)))?
        .ok_or_else(|| IoError::InvalidData("no frames in GIF".to_string()))?
        .clone();

    // Check for additional frames (animated GIF)
    if decoder
        .read_next_frame()
        .map_err(|e| IoError::DecodeError(format!("GIF frame error: {}", e)))?
        .is_some()
    {
        return Err(IoError::UnsupportedFormat(
            "animated GIF not supported".to_string(),
        ));
    }

    // Get palette - prefer local, fall back to global
    let palette: &[u8] = if let Some(ref local_palette) = frame.palette {
        local_palette
    } else if let Some(global_palette) = decoder.global_palette() {
        global_palette
    } else {
        return Err(IoError::InvalidData("GIF has no color map".to_string()));
    };

    // Validate palette size
    let ncolors = palette.len() / 3;
    if ncolors == 0 || ncolors > 256 {
        return Err(IoError::InvalidData(format!(
            "invalid palette size: {}",
            ncolors
        )));
    }

    // Determine depth based on color count (same as C version)
    let depth = if ncolors <= 2 {
        PixelDepth::Bit1
    } else if ncolors <= 4 {
        PixelDepth::Bit2
    } else if ncolors <= 16 {
        PixelDepth::Bit4
    } else {
        PixelDepth::Bit8
    };

    let width = frame.width as u32;
    let height = frame.height as u32;

    // Create pix with colormap
    let pix = Pix::new(width, height, depth)?;
    let mut pix_mut = pix.try_into_mut().unwrap();

    // Build colormap
    let mut cmap = PixColormap::new(depth.bits()).map_err(IoError::Core)?;
    for chunk in palette.chunks(3) {
        if chunk.len() == 3 {
            cmap.add_rgb(chunk[0], chunk[1], chunk[2])
                .map_err(IoError::Core)?;
        }
    }
    pix_mut.set_colormap(Some(cmap)).map_err(IoError::Core)?;

    // Copy pixel data
    let buffer = &frame.buffer;
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < buffer.len() {
                let val = buffer[idx] as u32;
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }
    }

    Ok(pix_mut.into())
}

/// Write a GIF image
///
/// Writes a pix as a single-frame GIF.
///
/// # Supported depths
/// - 1/2/4/8 bpp: Written with existing or generated colormap
/// - 16 bpp: Converted to 8bpp grayscale
/// - 32 bpp: Quantized to 8bpp using octree algorithm
pub fn write_gif<W: Write>(pix: &Pix, mut writer: W) -> IoResult<()> {
    // Convert to 8bpp with colormap if needed
    let (write_pix, cmap) = prepare_pix_for_gif(pix)?;

    let width = write_pix.width() as u16;
    let height = write_pix.height() as u16;

    // Build GIF palette (must be power of 2 size)
    let cmap_len = cmap.len();
    let gif_palette_size = cmap_len.next_power_of_two().max(2);

    let mut palette = Vec::with_capacity(gif_palette_size * 3);
    for i in 0..gif_palette_size {
        if i < cmap_len {
            if let Some((r, g, b)) = cmap.get_rgb(i) {
                palette.push(r);
                palette.push(g);
                palette.push(b);
            } else {
                palette.extend_from_slice(&[0, 0, 0]);
            }
        } else {
            palette.extend_from_slice(&[0, 0, 0]);
        }
    }

    // Create encoder
    let mut encoder = Encoder::new(&mut writer, width, height, &palette)
        .map_err(|e| IoError::EncodeError(format!("GIF encoder error: {}", e)))?;

    encoder
        .set_repeat(Repeat::Finite(0))
        .map_err(|e| IoError::EncodeError(format!("GIF repeat error: {}", e)))?;

    // Build frame buffer (always 8-bit indices)
    let mut buffer = Vec::with_capacity((width as usize) * (height as usize));

    for y in 0..(height as u32) {
        for x in 0..(width as u32) {
            let val = write_pix.get_pixel(x, y).unwrap_or(0);
            buffer.push(val as u8);
        }
    }

    // Create and write frame
    let mut frame = Frame::from_indexed_pixels(width, height, buffer, None);
    frame.palette = None; // Use global palette

    encoder
        .write_frame(&frame)
        .map_err(|e| IoError::EncodeError(format!("GIF frame write error: {}", e)))?;

    Ok(())
}

/// Prepare pix for GIF output
///
/// Converts the input pix to a format suitable for GIF encoding.
/// Returns the converted pix and its colormap.
fn prepare_pix_for_gif(pix: &Pix) -> IoResult<(Pix, PixColormap)> {
    match pix.depth() {
        PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8 => {
            if let Some(cmap) = pix.colormap() {
                // Already has colormap, clone the pix
                let new_pix = clone_pix(pix)?;
                Ok((new_pix, cmap.clone()))
            } else {
                // Create grayscale colormap
                let (new_pix, cmap) = create_grayscale_colormapped_pix(pix)?;
                Ok((new_pix, cmap))
            }
        }
        PixelDepth::Bit16 => {
            // Convert 16bpp to 8bpp grayscale
            let (new_pix, cmap) = convert_16bpp_to_8bpp_grayscale(pix)?;
            Ok((new_pix, cmap))
        }
        PixelDepth::Bit32 => {
            // Quantize 32bpp to 8bpp with colormap using median-cut
            let (quantized, cmap) = quantize_32bpp_to_8bpp(pix)?;
            Ok((quantized, cmap))
        }
    }
}

/// Quantize a 32bpp RGB image to 8bpp with colormap using median-cut algorithm
fn quantize_32bpp_to_8bpp(pix: &Pix) -> IoResult<(Pix, PixColormap)> {
    let w = pix.width();
    let h = pix.height();
    let max_colors: usize = 256;

    // Collect a bounded sample of RGB pixels for median-cut.
    // Limit the number of samples to avoid excessive memory usage on large images.
    // This still provides a representative distribution for palette generation.
    let total_pixels = (w as usize) * (h as usize);
    let max_samples: usize = 100_000;
    let sample_count = total_pixels.min(max_samples);
    let sample_stride = if sample_count == 0 {
        1
    } else {
        total_pixels.div_ceil(sample_count)
    };

    let mut pixels: Vec<[u8; 3]> = Vec::with_capacity(sample_count);
    let mut idx: usize = 0;
    for y in 0..h {
        for x in 0..w {
            if idx.is_multiple_of(sample_stride) {
                let val = pix.get_pixel(x, y).unwrap_or(0);
                let (r, g, b) = color::extract_rgb(val);
                pixels.push([r, g, b]);
            }
            idx += 1;
        }
    }

    // Median-cut quantization
    let palette = median_cut(&pixels, max_colors);

    // Build colormap
    let mut cmap = PixColormap::new(8).map_err(IoError::Core)?;
    for &[r, g, b] in &palette {
        cmap.add_rgb(r, g, b).map_err(IoError::Core)?;
    }

    // Create 8bpp image and map each pixel to nearest palette entry
    let new_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut new_mut = new_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel(x, y).unwrap_or(0);
            let (r, g, b) = color::extract_rgb(val);
            let idx = find_nearest_color(&palette, r, g, b);
            new_mut.set_pixel_unchecked(x, y, idx as u32);
        }
    }

    new_mut
        .set_colormap(Some(cmap.clone()))
        .map_err(IoError::Core)?;

    Ok((new_mut.into(), cmap))
}

/// Find nearest color index in palette
fn find_nearest_color(palette: &[[u8; 3]], r: u8, g: u8, b: u8) -> usize {
    let mut best = 0;
    let mut best_dist = u32::MAX;
    for (i, &[pr, pg, pb]) in palette.iter().enumerate() {
        let dr = (r as i32 - pr as i32).unsigned_abs();
        let dg = (g as i32 - pg as i32).unsigned_abs();
        let db = (b as i32 - pb as i32).unsigned_abs();
        let dist = dr * dr + dg * dg + db * db;
        if dist < best_dist {
            best_dist = dist;
            best = i;
            if dist == 0 {
                break;
            }
        }
    }
    best
}

/// Simple median-cut color quantization
fn median_cut(pixels: &[[u8; 3]], max_colors: usize) -> Vec<[u8; 3]> {
    if pixels.is_empty() {
        return vec![[0, 0, 0]];
    }

    let mut boxes: Vec<Vec<[u8; 3]>> = vec![pixels.to_vec()];

    while boxes.len() < max_colors {
        // Find the box with the largest range to split
        let mut best_box_idx = 0;
        let mut best_range = 0u32;

        for (i, b) in boxes.iter().enumerate() {
            if b.len() < 2 {
                continue;
            }
            let range = box_color_range(b);
            if range > best_range {
                best_range = range;
                best_box_idx = i;
            }
        }

        if best_range == 0 {
            break;
        }

        let box_to_split = boxes.remove(best_box_idx);
        let (left, right) = split_box(box_to_split);

        if !left.is_empty() {
            boxes.push(left);
        }
        if !right.is_empty() {
            boxes.push(right);
        }
    }

    // Compute average color for each box
    boxes.iter().map(|b| box_average(b)).collect()
}

/// Compute the color range of a box (max range across R, G, B channels)
fn box_color_range(pixels: &[[u8; 3]]) -> u32 {
    let (mut min_r, mut min_g, mut min_b) = (255u8, 255u8, 255u8);
    let (mut max_r, mut max_g, mut max_b) = (0u8, 0u8, 0u8);

    for &[r, g, b] in pixels {
        min_r = min_r.min(r);
        min_g = min_g.min(g);
        min_b = min_b.min(b);
        max_r = max_r.max(r);
        max_g = max_g.max(g);
        max_b = max_b.max(b);
    }

    let range_r = (max_r - min_r) as u32;
    let range_g = (max_g - min_g) as u32;
    let range_b = (max_b - min_b) as u32;

    range_r.max(range_g).max(range_b)
}

/// Split a box along its widest color axis
fn split_box(mut pixels: Vec<[u8; 3]>) -> (Vec<[u8; 3]>, Vec<[u8; 3]>) {
    let (mut min_r, mut min_g, mut min_b) = (255u8, 255u8, 255u8);
    let (mut max_r, mut max_g, mut max_b) = (0u8, 0u8, 0u8);

    for &[r, g, b] in &pixels {
        min_r = min_r.min(r);
        min_g = min_g.min(g);
        min_b = min_b.min(b);
        max_r = max_r.max(r);
        max_g = max_g.max(g);
        max_b = max_b.max(b);
    }

    let range_r = (max_r - min_r) as u32;
    let range_g = (max_g - min_g) as u32;
    let range_b = (max_b - min_b) as u32;

    // Sort along the widest axis
    if range_r >= range_g && range_r >= range_b {
        pixels.sort_unstable_by_key(|p| p[0]);
    } else if range_g >= range_b {
        pixels.sort_unstable_by_key(|p| p[1]);
    } else {
        pixels.sort_unstable_by_key(|p| p[2]);
    }

    let mid = pixels.len() / 2;
    let right = pixels.split_off(mid);
    (pixels, right)
}

/// Compute the average color of a box
fn box_average(pixels: &[[u8; 3]]) -> [u8; 3] {
    if pixels.is_empty() {
        return [0, 0, 0];
    }

    let (mut sum_r, mut sum_g, mut sum_b) = (0u64, 0u64, 0u64);
    for &[r, g, b] in pixels {
        sum_r += r as u64;
        sum_g += g as u64;
        sum_b += b as u64;
    }

    let n = pixels.len() as u64;
    [(sum_r / n) as u8, (sum_g / n) as u8, (sum_b / n) as u8]
}

/// Clone a pix
fn clone_pix(pix: &Pix) -> IoResult<Pix> {
    let new_pix = Pix::new(pix.width(), pix.height(), pix.depth())?;
    let mut new_mut = new_pix.try_into_mut().unwrap();

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if let Some(val) = pix.get_pixel(x, y) {
                new_mut.set_pixel_unchecked(x, y, val);
            }
        }
    }

    if let Some(cmap) = pix.colormap() {
        new_mut
            .set_colormap(Some(cmap.clone()))
            .map_err(IoError::Core)?;
    }

    Ok(new_mut.into())
}

/// Create a grayscale-colormapped version of a pix without colormap
fn create_grayscale_colormapped_pix(pix: &Pix) -> IoResult<(Pix, PixColormap)> {
    let depth = pix.depth();
    let max_val = match depth {
        PixelDepth::Bit1 => 1,
        PixelDepth::Bit2 => 3,
        PixelDepth::Bit4 => 15,
        PixelDepth::Bit8 => 255,
        _ => return Err(IoError::UnsupportedFormat("unsupported depth".to_string())),
    };

    // Create grayscale colormap
    let mut cmap = PixColormap::new(depth.bits()).map_err(IoError::Core)?;

    // Add grayscale entries
    let num_entries = max_val + 1;
    for i in 0..num_entries {
        let gray = ((i * 255) / max_val) as u8;
        cmap.add_rgb(gray, gray, gray).map_err(IoError::Core)?;
    }

    // Clone pix with colormap
    let new_pix = Pix::new(pix.width(), pix.height(), depth)?;
    let mut new_mut = new_pix.try_into_mut().unwrap();

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if let Some(val) = pix.get_pixel(x, y) {
                new_mut.set_pixel_unchecked(x, y, val);
            }
        }
    }

    new_mut
        .set_colormap(Some(cmap.clone()))
        .map_err(IoError::Core)?;

    Ok((new_mut.into(), cmap))
}

/// Convert 16bpp grayscale to 8bpp with grayscale colormap
fn convert_16bpp_to_8bpp_grayscale(pix: &Pix) -> IoResult<(Pix, PixColormap)> {
    // Create 8bpp grayscale colormap
    let mut cmap = PixColormap::new(8).map_err(IoError::Core)?;
    for i in 0..=255u8 {
        cmap.add_rgb(i, i, i).map_err(IoError::Core)?;
    }

    // Create new 8bpp pix
    let new_pix = Pix::new(pix.width(), pix.height(), PixelDepth::Bit8)?;
    let mut new_mut = new_pix.try_into_mut().unwrap();

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if let Some(val16) = pix.get_pixel(x, y) {
                // Scale 16-bit to 8-bit
                let val8 = val16 >> 8;
                new_mut.set_pixel_unchecked(x, y, val8);
            }
        }
    }

    new_mut
        .set_colormap(Some(cmap.clone()))
        .map_err(IoError::Core)?;

    Ok((new_mut.into(), cmap))
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::color;
    use std::io::Cursor;

    fn create_paletted_pix() -> Pix {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a colormap with a few colors
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap(); // 0: Red
        cmap.add_rgb(0, 255, 0).unwrap(); // 1: Green
        cmap.add_rgb(0, 0, 255).unwrap(); // 2: Blue
        cmap.add_rgb(255, 255, 0).unwrap(); // 3: Yellow
        pix_mut.set_colormap(Some(cmap)).unwrap();

        // Fill with pattern
        for y in 0..10 {
            for x in 0..10 {
                let val = (x + y) % 4;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_gif_roundtrip_paletted() {
        let pix = create_paletted_pix();

        let mut buffer = Vec::new();
        write_gif(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_gif(cursor).unwrap();

        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);
        assert!(pix2.has_colormap());

        // Check pixel values match
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(
                    pix2.get_pixel(x, y),
                    pix.get_pixel(x, y),
                    "mismatch at ({}, {})",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_gif_roundtrip_1bpp() {
        let pix = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create 1bpp colormap
        let mut cmap = PixColormap::new(1).unwrap();
        cmap.add_rgb(255, 255, 255).unwrap(); // 0: White
        cmap.add_rgb(0, 0, 0).unwrap(); // 1: Black
        pix_mut.set_colormap(Some(cmap)).unwrap();

        // Checkerboard pattern
        for y in 0..16 {
            for x in 0..16 {
                let val = (x + y) % 2;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_gif(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_gif(cursor).unwrap();

        // Should be read as 1bpp (2 colors)
        assert_eq!(pix2.depth(), PixelDepth::Bit1);

        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(pix2.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_gif_roundtrip_2bpp() {
        let pix = Pix::new(8, 8, PixelDepth::Bit2).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(85, 85, 85).unwrap();
        cmap.add_rgb(170, 170, 170).unwrap();
        cmap.add_rgb(255, 255, 255).unwrap();
        pix_mut.set_colormap(Some(cmap)).unwrap();

        for y in 0..8 {
            for x in 0..8 {
                let val = (x + y) % 4;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_gif(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_gif(cursor).unwrap();

        assert_eq!(pix2.depth(), PixelDepth::Bit2);

        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(pix2.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_gif_roundtrip_4bpp() {
        let pix = Pix::new(8, 8, PixelDepth::Bit4).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let mut cmap = PixColormap::new(4).unwrap();
        for i in 0..16 {
            let gray = (i * 17) as u8;
            cmap.add_rgb(gray, gray, gray).unwrap();
        }
        pix_mut.set_colormap(Some(cmap)).unwrap();

        for y in 0..8 {
            for x in 0..8 {
                let val = (x + y) % 16;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_gif(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_gif(cursor).unwrap();

        assert_eq!(pix2.depth(), PixelDepth::Bit4);

        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(pix2.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_gif_grayscale_without_colormap() {
        // Test 8bpp without colormap - should add grayscale colormap
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..8 {
            for x in 0..8 {
                let val = (x + y) * 16;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_gif(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_gif(cursor).unwrap();

        assert!(pix2.has_colormap());
    }

    #[test]
    fn test_gif_16bpp_conversion() {
        let pix = Pix::new(4, 4, PixelDepth::Bit16).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..4 {
            for x in 0..4 {
                let val = (x + y) * 16384; // 16-bit values
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_gif(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_gif(cursor).unwrap();

        // Should be 8bpp after conversion
        assert_eq!(pix2.depth(), PixelDepth::Bit8);
        assert!(pix2.has_colormap());
    }

    #[test]
    fn test_gif_32bpp_quantization() {
        let pix = Pix::new(16, 16, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a gradient image
        for y in 0..16 {
            for x in 0..16 {
                let r = (x * 16) as u8;
                let g = (y * 16) as u8;
                let b = 128u8;
                let pixel = color::compose_rgb(r, g, b);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_gif(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_gif(cursor).unwrap();

        // Should be 8bpp after quantization
        assert_eq!(pix2.depth(), PixelDepth::Bit8);
        assert!(pix2.has_colormap());
    }

    #[test]
    fn test_gif_colormap_preservation() {
        let pix = create_paletted_pix();
        let original_cmap = pix.colormap().unwrap();

        let mut buffer = Vec::new();
        write_gif(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_gif(cursor).unwrap();

        let read_cmap = pix2.colormap().unwrap();

        // Check first 4 colors match
        for i in 0..4 {
            let orig = original_cmap.get_rgb(i);
            let read = read_cmap.get_rgb(i);
            assert_eq!(orig, read, "colormap mismatch at index {}", i);
        }
    }
}
