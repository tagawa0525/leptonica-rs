//! PostScript image format support (write-only)
//!
//! This module provides PostScript (PS) and Encapsulated PostScript (EPS)
//! output functionality for images.
//!
//! # Features
//!
//! - Level 1 PostScript: Uncompressed hex-encoded data
//! - Level 3 PostScript: Flate (zlib) compressed with ASCII85 encoding
//! - EPS output with bounding box for embedding in documents
//!
//! # Example
//!
//! ```no_run
//! use leptonica_io::ps::{write_ps_mem, PsOptions, PsLevel};
//! use leptonica_core::Pix;
//!
//! let pix = Pix::new(100, 100, leptonica_core::PixelDepth::Bit8).unwrap();
//! let options = PsOptions::default();
//! let ps_data = write_ps_mem(&pix, &options).unwrap();
//! ```

mod ascii85;

use crate::{IoError, IoResult};
use leptonica_core::{Pix, PixelDepth, color};
use miniz_oxide::deflate::compress_to_vec_zlib;
use std::io::Write;

/// PostScript language level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PsLevel {
    /// Level 1: Uncompressed, hex-encoded
    /// Most compatible but largest file size
    Level1,
    /// Level 2: DCT (JPEG) compressed with ASCII85 encoding
    ///
    /// Best for photographic images. Requires the `jpeg` feature.
    /// Quality is controlled by `PsOptions::quality` (1-100, default 75).
    /// 1bpp images fall back to Level 3 Flate since JPEG is unsuitable.
    Level2,
    /// Level 3: Flate compressed with ASCII85 encoding
    /// Good compression, widely supported
    #[default]
    Level3,
}

/// PostScript output options
#[derive(Debug, Clone)]
pub struct PsOptions {
    /// PostScript language level (determines compression)
    pub level: PsLevel,
    /// Resolution in PPI (0 to use image's resolution, or 300 as fallback)
    pub resolution: u32,
    /// JPEG quality (1-100, default 75) - used when level is Level2
    pub quality: u8,
    /// Scale factor (0.0 or 1.0 for no scaling)
    pub scale: f32,
    /// Whether to write bounding box (required for EPS)
    pub write_bounding_box: bool,
    /// Document title
    pub title: Option<String>,
    /// Page number (for multi-page documents)
    pub page_number: u32,
}

impl Default for PsOptions {
    fn default() -> Self {
        Self {
            level: PsLevel::Level3,
            resolution: 0,
            quality: 75,
            scale: 1.0,
            write_bounding_box: true,
            title: None,
            page_number: 1,
        }
    }
}

impl PsOptions {
    /// Create options for EPS (Encapsulated PostScript) output
    pub fn eps() -> Self {
        Self {
            write_bounding_box: true,
            ..Default::default()
        }
    }

    /// Create options with a specific title
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..Default::default()
        }
    }

    /// Set the resolution
    pub fn resolution(mut self, res: u32) -> Self {
        self.resolution = res;
        self
    }

    /// Set the PostScript level
    pub fn level(mut self, level: PsLevel) -> Self {
        self.level = level;
        self
    }

    /// Set the scale factor
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Enable or disable bounding box output
    pub fn bounding_box(mut self, enable: bool) -> Self {
        self.write_bounding_box = enable;
        self
    }
}

/// Default resolution when none is specified
const DEFAULT_RESOLUTION: u32 = 300;

/// Points per inch in PostScript coordinates
const POINTS_PER_INCH: f32 = 72.0;

/// US Letter page dimensions in points
const LETTER_WIDTH: f32 = 612.0;
const LETTER_HEIGHT: f32 = 792.0;

/// Write a single image to PostScript bytes
///
/// This is the simplest interface for writing a single image to PostScript.
///
/// # Arguments
///
/// * `pix` - The image to convert
/// * `options` - PostScript output options
///
/// # Returns
///
/// PostScript data as a byte vector
pub fn write_ps_mem(pix: &Pix, options: &PsOptions) -> IoResult<Vec<u8>> {
    let mut buffer = Vec::new();
    write_ps(pix, &mut buffer, options)?;
    Ok(buffer)
}

/// Write a single image to PostScript
///
/// # Arguments
///
/// * `pix` - The image to convert
/// * `writer` - Output destination
/// * `options` - PostScript output options
pub fn write_ps<W: Write>(pix: &Pix, mut writer: W, options: &PsOptions) -> IoResult<()> {
    let ps_string = generate_ps(pix, options)?;
    writer
        .write_all(ps_string.as_bytes())
        .map_err(IoError::Io)?;
    Ok(())
}

/// Write multiple images to a multi-page PostScript document
///
/// Each image becomes one page in the output PS.
///
/// # Arguments
///
/// * `images` - Slice of images to include
/// * `writer` - Output destination
/// * `options` - PostScript output options (page_number is overridden per page)
pub fn write_ps_multi<W: Write>(images: &[&Pix], writer: W, options: &PsOptions) -> IoResult<()> {
    let _ = (images, writer, options);
    todo!("write_ps_multi not yet implemented")
}

/// Write an image as EPS (Encapsulated PostScript)
///
/// EPS includes bounding box information suitable for embedding in documents.
pub fn write_eps_mem(pix: &Pix, options: &PsOptions) -> IoResult<Vec<u8>> {
    let mut eps_options = options.clone();
    eps_options.write_bounding_box = true;
    write_ps_mem(pix, &eps_options)
}

/// Generate PostScript string for an image
fn generate_ps(pix: &Pix, options: &PsOptions) -> IoResult<String> {
    let width = pix.width();
    let height = pix.height();

    // Determine resolution
    let res = if options.resolution > 0 {
        options.resolution
    } else {
        let xres = pix.xres();
        if xres > 0 {
            xres as u32
        } else {
            DEFAULT_RESOLUTION
        }
    };

    // Determine scale
    let scale = if options.scale <= 0.0 {
        1.0
    } else {
        options.scale
    };

    // Calculate page position (centered on letter page)
    let effective_res = res as f32 / scale;
    let width_pt = width as f32 * POINTS_PER_INCH / effective_res;
    let height_pt = height as f32 * POINTS_PER_INCH / effective_res;
    let x_pt = (LETTER_WIDTH - width_pt) / 2.0;
    let y_pt = (LETTER_HEIGHT - height_pt) / 2.0;

    match options.level {
        PsLevel::Level1 => generate_uncompressed_ps(pix, options, x_pt, y_pt, width_pt, height_pt),
        PsLevel::Level2 => {
            // Level 2 DCT: 1bpp falls back to Level 3 Flate
            if pix.depth() == PixelDepth::Bit1 {
                generate_flate_ps(pix, options, x_pt, y_pt, width_pt, height_pt)
            } else {
                generate_dct_ps(pix, options, x_pt, y_pt, width_pt, height_pt)
            }
        }
        PsLevel::Level3 => generate_flate_ps(pix, options, x_pt, y_pt, width_pt, height_pt),
    }
}

/// Generate uncompressed (Level 1) PostScript
fn generate_uncompressed_ps(
    pix: &Pix,
    options: &PsOptions,
    x_pt: f32,
    y_pt: f32,
    width_pt: f32,
    height_pt: f32,
) -> IoResult<String> {
    let width = pix.width();
    let height = pix.height();

    // Prepare image data and get format info
    let (image_data, samples_per_pixel, bits_per_sample) = prepare_image_data(pix)?;

    // Calculate bytes per line
    let bytes_per_line = if bits_per_sample == 1 {
        width.div_ceil(8)
    } else {
        width * samples_per_pixel
    } as usize;

    // Convert to hex string
    let hex_data = bytes_to_hex(&image_data, bytes_per_line, height as usize);

    let mut ps = String::new();

    // DSC header
    ps.push_str("%!Adobe-PS\n");
    if options.write_bounding_box {
        ps.push_str(&format!(
            "%%BoundingBox: {:.2} {:.2} {:.2} {:.2}\n",
            x_pt,
            y_pt,
            x_pt + width_pt,
            y_pt + height_pt
        ));
    } else {
        ps.push_str("gsave\n");
    }

    // Invert for 1bpp images
    if bits_per_sample == 1 {
        ps.push_str("{1 exch sub} settransfer    %invert binary\n");
    }

    // Setup
    ps.push_str(&format!(
        "/bpl {} string def         %bpl as a string\n",
        bytes_per_line
    ));
    ps.push_str(&format!(
        "{:.2} {:.2} translate         %set image origin in pts\n",
        x_pt, y_pt
    ));
    ps.push_str(&format!(
        "{:.2} {:.2} scale             %set image size in pts\n",
        width_pt, height_pt
    ));
    ps.push_str(&format!(
        "{} {} {}                 %image dimensions in pixels\n",
        width, height, bits_per_sample
    ));
    ps.push_str(&format!(
        "[{} {} {} {} {} {}]     %mapping matrix: [w 0 0 -h 0 h]\n",
        width,
        0,
        0,
        -(height as i32),
        0,
        height
    ));

    // Image command
    if samples_per_pixel == 3 {
        if options.write_bounding_box {
            ps.push_str("{currentfile bpl readhexstring pop} false 3 colorimage\n");
        } else {
            ps.push_str("{currentfile bpl readhexstring pop} bind false 3 colorimage\n");
        }
    } else if options.write_bounding_box {
        ps.push_str("{currentfile bpl readhexstring pop} image\n");
    } else {
        ps.push_str("{currentfile bpl readhexstring pop} bind image\n");
    }

    // Image data
    ps.push_str(&hex_data);

    // Footer
    if options.write_bounding_box {
        ps.push_str("\nshowpage\n");
    } else {
        ps.push_str("\ngrestore\n");
    }

    Ok(ps)
}

/// Generate Flate-compressed (Level 3) PostScript
fn generate_flate_ps(
    pix: &Pix,
    options: &PsOptions,
    x_pt: f32,
    y_pt: f32,
    width_pt: f32,
    height_pt: f32,
) -> IoResult<String> {
    let width = pix.width();
    let height = pix.height();

    // Prepare image data
    let (image_data, samples_per_pixel, bits_per_sample) = prepare_image_data(pix)?;

    // Compress with zlib
    let compressed = compress_to_vec_zlib(&image_data, 6);

    // Encode with ASCII85
    let encoded = ascii85::encode(&compressed);

    let page_no = options.page_number;

    let mut ps = String::new();

    // DSC header
    ps.push_str("%!PS-Adobe-3.0 EPSF-3.0\n");
    ps.push_str("%%Creator: leptonica-rs\n");
    if let Some(ref title) = options.title {
        ps.push_str(&format!("%%Title: {}\n", title));
    } else {
        ps.push_str("%%Title: Flate compressed PS\n");
    }
    ps.push_str("%%DocumentData: Clean7Bit\n");

    if options.write_bounding_box {
        ps.push_str(&format!(
            "%%BoundingBox: {:.2} {:.2} {:.2} {:.2}\n",
            x_pt,
            y_pt,
            x_pt + width_pt,
            y_pt + height_pt
        ));
    }

    ps.push_str("%%LanguageLevel: 3\n");
    ps.push_str("%%EndComments\n");
    ps.push_str(&format!("%%Page: {} {}\n", page_no, page_no));

    ps.push_str("save\n");
    ps.push_str(&format!(
        "{:.2} {:.2} translate         %set image origin in pts\n",
        x_pt, y_pt
    ));
    ps.push_str(&format!(
        "{:.2} {:.2} scale             %set image size in pts\n",
        width_pt, height_pt
    ));

    // Color space
    if samples_per_pixel == 1 {
        ps.push_str("/DeviceGray setcolorspace\n");
    } else {
        ps.push_str("/DeviceRGB setcolorspace\n");
    }

    // Data filter setup
    ps.push_str("/RawData currentfile /ASCII85Decode filter def\n");
    ps.push_str("/Data RawData << >> /FlateDecode filter def\n");

    // Image dictionary
    ps.push_str("{ << /ImageType 1\n");
    ps.push_str(&format!("     /Width {}\n", width));
    ps.push_str(&format!("     /Height {}\n", height));
    ps.push_str(&format!("     /BitsPerComponent {}\n", bits_per_sample));
    ps.push_str(&format!(
        "     /ImageMatrix [ {} 0 0 {} 0 {} ]\n",
        width,
        -(height as i32),
        height
    ));

    // Decode array
    if samples_per_pixel == 1 {
        if bits_per_sample == 1 {
            // miniswhite photometry for 1bpp
            ps.push_str("     /Decode [1 0]\n");
        } else {
            ps.push_str("     /Decode [0 1]\n");
        }
    } else {
        ps.push_str("     /Decode [0 1 0 1 0 1]\n");
    }

    ps.push_str("     /DataSource Data\n");
    ps.push_str("  >> image\n");
    ps.push_str("  Data closefile\n");
    ps.push_str("  RawData flushfile\n");
    ps.push_str("  showpage\n");
    ps.push_str("  restore\n");
    ps.push_str("} exec\n");

    // Encoded data
    ps.push_str(&encoded);
    ps.push('\n');

    Ok(ps)
}

/// Generate DCT (JPEG) compressed (Level 2) PostScript
fn generate_dct_ps(
    pix: &Pix,
    options: &PsOptions,
    x_pt: f32,
    y_pt: f32,
    width_pt: f32,
    height_pt: f32,
) -> IoResult<String> {
    let _ = (pix, options, x_pt, y_pt, width_pt, height_pt);
    todo!("generate_dct_ps not yet implemented")
}

/// Prepare image data for PostScript embedding
///
/// Returns (raw_data, samples_per_pixel, bits_per_sample)
fn prepare_image_data(pix: &Pix) -> IoResult<(Vec<u8>, u32, u32)> {
    let width = pix.width();
    let height = pix.height();

    match pix.depth() {
        PixelDepth::Bit1 => {
            // 1bpp: Pack bits into bytes (8 pixels per byte)
            let bytes_per_line = width.div_ceil(8) as usize;
            let mut data = vec![0u8; bytes_per_line * height as usize];

            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    if val != 0 {
                        let byte_idx = (y as usize * bytes_per_line) + (x / 8) as usize;
                        let bit_idx = 7 - (x % 8);
                        data[byte_idx] |= 1 << bit_idx;
                    }
                }
            }
            Ok((data, 1, 1))
        }
        PixelDepth::Bit2 | PixelDepth::Bit4 => {
            // Low-depth grayscale: expand to 8bpp
            let max_val = match pix.depth() {
                PixelDepth::Bit2 => 3,
                PixelDepth::Bit4 => 15,
                _ => unreachable!(),
            };
            let mut data = Vec::with_capacity((width * height) as usize);
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    let scaled = (val * 255 / max_val) as u8;
                    data.push(scaled);
                }
            }
            Ok((data, 1, 8))
        }
        PixelDepth::Bit8 => {
            if pix.has_colormap() {
                // Indexed color: expand to RGB
                let cmap = pix.colormap().ok_or_else(|| {
                    IoError::InvalidData("colormap expected but not found".to_string())
                })?;
                let mut data = Vec::with_capacity((width * height * 3) as usize);
                for y in 0..height {
                    for x in 0..width {
                        let idx = pix.get_pixel(x, y).unwrap_or(0) as usize;
                        if let Some((r, g, b)) = cmap.get_rgb(idx) {
                            data.push(r);
                            data.push(g);
                            data.push(b);
                        } else {
                            data.push(0);
                            data.push(0);
                            data.push(0);
                        }
                    }
                }
                Ok((data, 3, 8))
            } else {
                // Grayscale
                let mut data = Vec::with_capacity((width * height) as usize);
                for y in 0..height {
                    for x in 0..width {
                        data.push(pix.get_pixel(x, y).unwrap_or(0) as u8);
                    }
                }
                Ok((data, 1, 8))
            }
        }
        PixelDepth::Bit16 => {
            // 16bpp grayscale: convert to 8bpp
            let mut data = Vec::with_capacity((width * height) as usize);
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    data.push((val >> 8) as u8);
                }
            }
            Ok((data, 1, 8))
        }
        PixelDepth::Bit32 => {
            // 32bpp: RGB or RGBA
            let spp = pix.spp();
            let mut data = Vec::with_capacity((width * height * 3) as usize);
            for y in 0..height {
                for x in 0..width {
                    let pixel = pix.get_pixel(x, y).unwrap_or(0);
                    if spp == 4 {
                        let (r, g, b, _a) = color::extract_rgba(pixel);
                        data.push(r);
                        data.push(g);
                        data.push(b);
                    } else {
                        let (r, g, b) = color::extract_rgb(pixel);
                        data.push(r);
                        data.push(g);
                        data.push(b);
                    }
                }
            }
            Ok((data, 3, 8))
        }
    }
}

/// Convert bytes to hex string for Level 1 PostScript
fn bytes_to_hex(data: &[u8], bytes_per_line: usize, num_lines: usize) -> String {
    let hex_chars: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
    ];

    let mut result = String::with_capacity(data.len() * 2 + num_lines);

    for (i, &byte) in data.iter().enumerate() {
        result.push(hex_chars[(byte >> 4) as usize]);
        result.push(hex_chars[(byte & 0x0f) as usize]);

        // Add newline after each row
        if (i + 1) % bytes_per_line == 0 {
            result.push('\n');
        }
    }

    result
}

/// Calculate resolution to fit image on US Letter page
#[allow(dead_code)]
pub fn get_res_letter_page(width: u32, height: u32, fill_fraction: f32) -> u32 {
    let fill = if fill_fraction <= 0.0 {
        0.95
    } else {
        fill_fraction as f64
    };

    let res_w = (width as f64 * 72.0) / (LETTER_WIDTH as f64 * fill);
    let res_h = (height as f64 * 72.0) / (LETTER_HEIGHT as f64 * fill);

    res_w.max(res_h) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::PixelDepth;

    #[test]
    fn test_write_ps_grayscale_level1() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..50 {
            for x in 0..50 {
                pix_mut.set_pixel(x, y, (x + y) % 256).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();
        let options = PsOptions::default().level(PsLevel::Level1);
        let ps_data = write_ps_mem(&pix, &options).unwrap();

        // Verify PS header
        let ps_str = String::from_utf8_lossy(&ps_data);
        assert!(ps_str.starts_with("%!Adobe-PS"));
        assert!(ps_str.contains("BoundingBox"));
        assert!(ps_str.contains("image"));
    }

    #[test]
    fn test_write_ps_grayscale_level3() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let options = PsOptions::default().level(PsLevel::Level3);
        let ps_data = write_ps_mem(&pix, &options).unwrap();

        let ps_str = String::from_utf8_lossy(&ps_data);
        assert!(ps_str.starts_with("%!PS-Adobe-3.0"));
        assert!(ps_str.contains("LanguageLevel: 3"));
        assert!(ps_str.contains("FlateDecode"));
        assert!(ps_str.contains("ASCII85Decode"));
        assert!(ps_str.contains("~>")); // ASCII85 terminator
    }

    #[test]
    fn test_write_ps_rgb() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..30 {
            for x in 0..30 {
                let color = color::compose_rgb((x * 8) as u8, (y * 8) as u8, 128);
                pix_mut.set_pixel(x, y, color).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();
        let options = PsOptions::with_title("RGB Test");
        let ps_data = write_ps_mem(&pix, &options).unwrap();

        let ps_str = String::from_utf8_lossy(&ps_data);
        assert!(ps_str.contains("DeviceRGB"));
        assert!(ps_str.contains("Title: RGB Test"));
    }

    #[test]
    fn test_write_ps_1bpp() {
        let pix = Pix::new(80, 80, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create checkerboard
        for y in 0..80 {
            for x in 0..80 {
                let val = ((x / 10) + (y / 10)) % 2;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();
        let options = PsOptions::default();
        let ps_data = write_ps_mem(&pix, &options).unwrap();

        let ps_str = String::from_utf8_lossy(&ps_data);
        assert!(ps_str.contains("DeviceGray"));
    }

    #[test]
    fn test_write_eps() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let options = PsOptions::eps();
        let eps_data = write_eps_mem(&pix, &options).unwrap();

        let eps_str = String::from_utf8_lossy(&eps_data);
        assert!(eps_str.contains("EPSF-3.0"));
        assert!(eps_str.contains("BoundingBox"));
    }

    #[test]
    fn test_ps_options() {
        let opts = PsOptions::default();
        assert_eq!(opts.level, PsLevel::Level3);
        assert_eq!(opts.resolution, 0);
        assert!(opts.write_bounding_box);
        assert_eq!(opts.scale, 1.0);

        let opts = PsOptions::with_title("Test")
            .resolution(150)
            .level(PsLevel::Level1)
            .bounding_box(false);
        assert_eq!(opts.title, Some("Test".to_string()));
        assert_eq!(opts.resolution, 150);
        assert_eq!(opts.level, PsLevel::Level1);
        assert!(!opts.write_bounding_box);
    }

    #[test]
    fn test_get_res_letter_page() {
        // A 612x792 pixel image at 0.95 fill should be ~72 ppi
        let res = get_res_letter_page(612, 792, 0.95);
        assert!(res > 70 && res < 80);

        // A 2550x3300 pixel image (300 ppi letter size) at full fill
        let res = get_res_letter_page(2550, 3300, 0.95);
        assert!(res > 310 && res < 320);
    }

    #[test]
    fn test_bytes_to_hex() {
        let data = vec![0x00, 0xFF, 0xAB, 0xCD];
        let hex = bytes_to_hex(&data, 2, 2);
        assert!(hex.contains("00ff"));
        assert!(hex.contains("abcd"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_write_ps_multi_pages() {
        let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(200, 150, PixelDepth::Bit32).unwrap();
        let pix3 = Pix::new(50, 50, PixelDepth::Bit1).unwrap();

        let images: Vec<&Pix> = vec![&pix1, &pix2, &pix3];
        let options = PsOptions::with_title("Multi-page Test");

        let mut buffer = Vec::new();
        write_ps_multi(&images, &mut buffer, &options).unwrap();

        let ps_str = String::from_utf8_lossy(&buffer);
        assert!(ps_str.starts_with("%!PS-Adobe"));
        // Should contain page markers for each page
        assert!(ps_str.contains("%%Page: 1 1"));
        assert!(ps_str.contains("%%Page: 2 2"));
        assert!(ps_str.contains("%%Page: 3 3"));
        assert!(ps_str.contains("%%Pages: 3"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_write_ps_level2_rgb() {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..50 {
            for x in 0..50 {
                let c = color::compose_rgb(x as u8 * 5, y as u8 * 5, 128);
                pix_mut.set_pixel(x, y, c).unwrap();
            }
        }
        let pix: Pix = pix_mut.into();

        let options = PsOptions::default().level(PsLevel::Level2);
        let ps_data = write_ps_mem(&pix, &options).unwrap();

        let ps_str = String::from_utf8_lossy(&ps_data);
        assert!(ps_str.contains("DCTDecode"));
        assert!(ps_str.contains("LanguageLevel: 2"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_write_ps_level2_grayscale() {
        let pix = Pix::new(60, 60, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..60 {
            for x in 0..60 {
                pix_mut.set_pixel(x, y, ((x + y) * 3) % 256).unwrap();
            }
        }
        let pix: Pix = pix_mut.into();

        let options = PsOptions::default().level(PsLevel::Level2);
        let ps_data = write_ps_mem(&pix, &options).unwrap();

        let ps_str = String::from_utf8_lossy(&ps_data);
        assert!(ps_str.contains("DCTDecode"));
        assert!(ps_str.contains("DeviceGray"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_write_ps_level2_1bpp_fallback() {
        // 1bpp should fall back to Level 3 Flate
        let pix = Pix::new(80, 80, PixelDepth::Bit1).unwrap();
        let options = PsOptions::default().level(PsLevel::Level2);
        let ps_data = write_ps_mem(&pix, &options).unwrap();

        let ps_str = String::from_utf8_lossy(&ps_data);
        assert!(ps_str.contains("FlateDecode"));
        assert!(!ps_str.contains("DCTDecode"));
    }
}
