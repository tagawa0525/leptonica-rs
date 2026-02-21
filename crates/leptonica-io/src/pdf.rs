//! PDF image format support (write-only)
//!
//! This module provides PDF output functionality for images.
//! It uses the `pdf-writer` crate to generate PDF files.
//!
//! # Features
//!
//! - Single image to PDF conversion
//! - Multiple images to multi-page PDF
//! - Configurable compression and quality settings
//!
//! # Example
//!
//! ```no_run
//! use leptonica_io::pdf::{write_pdf_mem, PdfOptions};
//! use leptonica_core::Pix;
//!
//! let pix = Pix::new(100, 100, leptonica_core::PixelDepth::Bit8).unwrap();
//! let options = PdfOptions::default();
//! let pdf_data = write_pdf_mem(&pix, &options).unwrap();
//! ```

use crate::{IoError, IoResult};
use leptonica_core::{Pix, PixelDepth, color};
use miniz_oxide::deflate::compress_to_vec_zlib;
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref, TextStr};
use std::io::Write;

/// Color space type for PDF images
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PdfColorSpace {
    DeviceGray,
    DeviceRgb,
}

/// PDF compression method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfCompression {
    /// Automatic selection based on image depth
    /// - 1bpp: Flate (CCITT G4 is complex to implement)
    /// - 8bpp grayscale: Flate
    /// - 32bpp RGB: Flate
    #[default]
    Auto,
    /// Flate (Deflate/zlib) compression - works for all image types
    Flate,
    /// DCT (JPEG) compression - best for photographic images
    ///
    /// Requires the `jpeg` feature to be enabled. Uses the `jpeg-encoder` crate
    /// to compress image data and embeds it with the DCTDecode filter.
    /// Quality is controlled by `PdfOptions::quality` (1-100, default 75).
    ///
    /// Note: 1bpp images are always written with Flate even when Jpeg is selected,
    /// since JPEG is unsuitable for binary images.
    Jpeg,
}

/// PDF output options
#[derive(Debug, Clone)]
pub struct PdfOptions {
    /// Compression method
    pub compression: PdfCompression,
    /// JPEG quality (1-100, 0 for default 75) - reserved for future DCT support
    pub quality: u8,
    /// Resolution in PPI (0 to use image's resolution, or 300 as fallback)
    pub resolution: u32,
    /// Document title
    pub title: Option<String>,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            compression: PdfCompression::Auto,
            quality: 75,
            resolution: 0,
            title: None,
        }
    }
}

impl PdfOptions {
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

    /// Set the compression method
    pub fn compression(mut self, comp: PdfCompression) -> Self {
        self.compression = comp;
        self
    }
}

/// Default resolution when none is specified
const DEFAULT_RESOLUTION: u32 = 300;

/// Points per inch in PDF coordinates
const POINTS_PER_INCH: f32 = 72.0;

/// Write a single image to PDF bytes
///
/// This is the simplest interface for writing a single image to PDF.
///
/// # Arguments
///
/// * `pix` - The image to convert
/// * `options` - PDF output options
///
/// # Returns
///
/// PDF data as a byte vector
pub fn write_pdf_mem(pix: &Pix, options: &PdfOptions) -> IoResult<Vec<u8>> {
    let mut buffer = Vec::new();
    write_pdf(pix, &mut buffer, options)?;
    Ok(buffer)
}

/// Write a single image to PDF
///
/// # Arguments
///
/// * `pix` - The image to convert
/// * `writer` - Output destination
/// * `options` - PDF output options
pub fn write_pdf<W: Write>(pix: &Pix, mut writer: W, options: &PdfOptions) -> IoResult<()> {
    let pdf_data = generate_pdf(&[pix], options)?;
    writer.write_all(&pdf_data).map_err(IoError::Io)?;
    Ok(())
}

/// Write multiple images to a multi-page PDF
///
/// Each image becomes one page in the output PDF.
///
/// # Arguments
///
/// * `images` - Slice of images to include
/// * `writer` - Output destination
/// * `options` - PDF output options
pub fn write_pdf_multi<W: Write>(
    images: &[&Pix],
    mut writer: W,
    options: &PdfOptions,
) -> IoResult<()> {
    let pdf_data = generate_pdf(images, options)?;
    writer.write_all(&pdf_data).map_err(IoError::Io)?;
    Ok(())
}

/// Write multiple image files to a multi-page PDF
///
/// Reads each file, detects its format, and adds it as a page in the PDF.
///
/// # Arguments
///
/// * `paths` - Slice of file paths to include
/// * `writer` - Output destination
/// * `options` - PDF output options
pub fn write_pdf_from_files<W: Write>(
    paths: &[impl AsRef<std::path::Path>],
    mut writer: W,
    options: &PdfOptions,
) -> IoResult<()> {
    if paths.is_empty() {
        return Err(IoError::InvalidData("no files provided".to_string()));
    }

    let images: Vec<Pix> = paths
        .iter()
        .map(crate::read_image)
        .collect::<IoResult<Vec<_>>>()?;

    let image_refs: Vec<&Pix> = images.iter().collect();
    let pdf_data = generate_pdf(&image_refs, options)?;
    writer.write_all(&pdf_data).map_err(IoError::Io)?;
    Ok(())
}

/// Generate PDF data from images
fn generate_pdf(images: &[&Pix], options: &PdfOptions) -> IoResult<Vec<u8>> {
    if images.is_empty() {
        return Err(IoError::InvalidData("no images provided".to_string()));
    }

    let mut pdf = Pdf::new();

    // Object reference allocation
    // Structure: Catalog(1), Pages(2), [Page(3+i*3), Contents(4+i*3), XObject(5+i*3)]...
    let catalog_id = Ref::new(1);
    let pages_id = Ref::new(2);

    // Calculate page refs
    let page_refs: Vec<Ref> = (0..images.len())
        .map(|i| Ref::new((3 + i * 3) as i32))
        .collect();

    // Write catalog
    pdf.catalog(catalog_id).pages(pages_id);

    // Write document info if title is provided
    if let Some(ref title) = options.title {
        let info_id = Ref::new((3 + images.len() * 3) as i32);
        pdf.document_info(info_id).title(TextStr(title));
    }

    // Write pages object
    pdf.pages(pages_id)
        .kids(page_refs.iter().copied())
        .count(images.len() as i32);

    // Write each page
    for (i, pix) in images.iter().enumerate() {
        let page_id = Ref::new((3 + i * 3) as i32);
        let contents_id = Ref::new((4 + i * 3) as i32);
        let image_id = Ref::new((5 + i * 3) as i32);

        write_page(
            &mut pdf,
            pix,
            page_id,
            pages_id,
            contents_id,
            image_id,
            options,
        )?;
    }

    Ok(pdf.finish())
}

/// Write a single page to the PDF
fn write_page(
    pdf: &mut Pdf,
    pix: &Pix,
    page_id: Ref,
    pages_id: Ref,
    contents_id: Ref,
    image_id: Ref,
    options: &PdfOptions,
) -> IoResult<()> {
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

    // Calculate page size in points
    let width_pt = width as f32 * POINTS_PER_INCH / res as f32;
    let height_pt = height as f32 * POINTS_PER_INCH / res as f32;

    // Prepare image data
    let (image_data, color_space, bits_per_component) = prepare_image_data(pix)?;

    // Determine whether to use JPEG compression
    // Only PdfCompression::Jpeg triggers DCT encoding; Auto and Flate use FlateDecode.
    // 1bpp/2bpp/4bpp always use Flate since JPEG is unsuitable for low-depth images.
    #[cfg(feature = "jpeg")]
    let use_jpeg = matches!(options.compression, PdfCompression::Jpeg)
        && pix.depth() != PixelDepth::Bit1
        && pix.depth() != PixelDepth::Bit2
        && pix.depth() != PixelDepth::Bit4;

    #[cfg(not(feature = "jpeg"))]
    let use_jpeg = false;

    // Compress image data
    let (compressed_data, filter) = if use_jpeg {
        #[cfg(feature = "jpeg")]
        {
            let jpeg_data = encode_jpeg_for_pdf(&image_data, width, height, color_space, options)?;
            (jpeg_data, Filter::DctDecode)
        }
        #[cfg(not(feature = "jpeg"))]
        {
            unreachable!()
        }
    } else {
        (compress_to_vec_zlib(&image_data, 6), Filter::FlateDecode)
    };

    // Write image XObject
    let mut image = pdf.image_xobject(image_id, &compressed_data);
    image.filter(filter);
    image.width(width as i32);
    image.height(height as i32);
    match color_space {
        PdfColorSpace::DeviceGray => image.color_space().device_gray(),
        PdfColorSpace::DeviceRgb => image.color_space().device_rgb(),
    }
    image.bits_per_component(bits_per_component);
    image.finish();

    // Write page contents (drawing commands)
    let mut content = Content::new();
    content.save_state();
    // Transform: scale and position the image
    // PDF coordinate system: origin at bottom-left, Y increases upward
    // We need to flip Y and scale to page size
    content.transform([width_pt, 0.0, 0.0, height_pt, 0.0, 0.0]);
    content.x_object(Name(b"Im0"));
    content.restore_state();
    let content_data = content.finish();

    // Write contents stream
    pdf.stream(contents_id, &content_data);

    // Write page object
    let mut page = pdf.page(page_id);
    page.parent(pages_id);
    page.media_box(Rect::new(0.0, 0.0, width_pt, height_pt));
    page.contents(contents_id);

    // Add image resource
    page.resources().x_objects().pair(Name(b"Im0"), image_id);

    page.finish();

    Ok(())
}

/// Prepare image data for PDF embedding
///
/// Returns (raw_data, color_space, bits_per_component)
fn prepare_image_data(pix: &Pix) -> IoResult<(Vec<u8>, PdfColorSpace, i32)> {
    let width = pix.width();
    let height = pix.height();

    match pix.depth() {
        PixelDepth::Bit1 => {
            // 1bpp: Convert to 8bpp grayscale for PDF.
            // Leptonica uses 0 = white, 1 = black; PDF grayscale uses 0 = black, 255 = white.
            let mut data = Vec::with_capacity((width * height) as usize);
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    // Map Leptonica 0 (white) -> 255 (white), 1 (black) -> 0 (black)
                    data.push(if val == 0 { 255 } else { 0 });
                }
            }
            Ok((data, PdfColorSpace::DeviceGray, 8))
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
                    // Scale to 0-255
                    let scaled = (val * 255 / max_val) as u8;
                    data.push(scaled);
                }
            }
            Ok((data, PdfColorSpace::DeviceGray, 8))
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
                            // Default to black if index out of range
                            data.push(0);
                            data.push(0);
                            data.push(0);
                        }
                    }
                }
                Ok((data, PdfColorSpace::DeviceRgb, 8))
            } else {
                // Grayscale
                let mut data = Vec::with_capacity((width * height) as usize);
                for y in 0..height {
                    for x in 0..width {
                        data.push(pix.get_pixel(x, y).unwrap_or(0) as u8);
                    }
                }
                Ok((data, PdfColorSpace::DeviceGray, 8))
            }
        }
        PixelDepth::Bit16 => {
            // 16bpp grayscale: convert to 8bpp
            let mut data = Vec::with_capacity((width * height) as usize);
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    // Take high byte
                    data.push((val >> 8) as u8);
                }
            }
            Ok((data, PdfColorSpace::DeviceGray, 8))
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
                        // Alpha is ignored in PDF (no transparency support in this impl)
                    } else {
                        let (r, g, b) = color::extract_rgb(pixel);
                        data.push(r);
                        data.push(g);
                        data.push(b);
                    }
                }
            }
            Ok((data, PdfColorSpace::DeviceRgb, 8))
        }
    }
}

/// Encode image data as JPEG for PDF embedding
#[cfg(feature = "jpeg")]
fn encode_jpeg_for_pdf(
    image_data: &[u8],
    width: u32,
    height: u32,
    color_space: PdfColorSpace,
    options: &PdfOptions,
) -> IoResult<Vec<u8>> {
    let quality = if options.quality == 0 {
        75
    } else {
        options.quality
    };

    let color_type = match color_space {
        PdfColorSpace::DeviceGray => jpeg_encoder::ColorType::Luma,
        PdfColorSpace::DeviceRgb => jpeg_encoder::ColorType::Rgb,
    };

    let mut jpeg_buf = Vec::new();
    let encoder = jpeg_encoder::Encoder::new(&mut jpeg_buf, quality);
    encoder
        .encode(image_data, width as u16, height as u16, color_type)
        .map_err(|e| IoError::EncodeError(format!("JPEG encode for PDF error: {}", e)))?;
    Ok(jpeg_buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::PixelDepth;

    #[test]
    fn test_write_pdf_grayscale() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a simple gradient
        for y in 0..100 {
            for x in 0..100 {
                pix_mut.set_pixel(x, y, (x + y) % 256).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();
        let options = PdfOptions::default();
        let pdf_data = write_pdf_mem(&pix, &options).unwrap();

        // Verify PDF header
        assert!(pdf_data.starts_with(b"%PDF-"));
        assert!(pdf_data.len() > 100);
    }

    #[test]
    fn test_write_pdf_rgb() {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a red/green/blue pattern
        for y in 0..50 {
            for x in 0..50 {
                let color = if x < 17 {
                    color::compose_rgb(255, 0, 0) // Red
                } else if x < 34 {
                    color::compose_rgb(0, 255, 0) // Green
                } else {
                    color::compose_rgb(0, 0, 255) // Blue
                };
                pix_mut.set_pixel(x, y, color).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();
        let options = PdfOptions::with_title("Test RGB Image");
        let pdf_data = write_pdf_mem(&pix, &options).unwrap();

        // Verify PDF header
        assert!(pdf_data.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_write_pdf_1bpp() {
        let pix = Pix::new(80, 80, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a checkerboard pattern
        for y in 0..80 {
            for x in 0..80 {
                let val = ((x / 10) + (y / 10)) % 2;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();
        let options = PdfOptions::default().resolution(150);
        let pdf_data = write_pdf_mem(&pix, &options).unwrap();

        assert!(pdf_data.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_write_pdf_multi() {
        let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(200, 150, PixelDepth::Bit8).unwrap();

        let images: Vec<&Pix> = vec![&pix1, &pix2];
        let options = PdfOptions::with_title("Multi-page Test");

        let mut buffer = Vec::new();
        write_pdf_multi(&images, &mut buffer, &options).unwrap();

        assert!(buffer.starts_with(b"%PDF-"));
        // Should contain references to multiple pages
        assert!(buffer.len() > 200);
    }

    #[test]
    fn test_pdf_options() {
        let opts = PdfOptions::default();
        assert_eq!(opts.compression, PdfCompression::Auto);
        assert_eq!(opts.quality, 75);
        assert_eq!(opts.resolution, 0);
        assert!(opts.title.is_none());

        let opts = PdfOptions::with_title("Test")
            .resolution(150)
            .compression(PdfCompression::Flate);
        assert_eq!(opts.title, Some("Test".to_string()));
        assert_eq!(opts.resolution, 150);
        assert_eq!(opts.compression, PdfCompression::Flate);
    }

    #[test]
    fn test_write_pdf_jpeg_compression() {
        let pix = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..100 {
            for x in 0..100 {
                pix_mut
                    .set_pixel(x, y, color::compose_rgb(x as u8, y as u8, 128))
                    .unwrap();
            }
        }
        let pix: Pix = pix_mut.into();

        let options = PdfOptions {
            compression: PdfCompression::Jpeg,
            quality: 75,
            ..Default::default()
        };
        let pdf_data = write_pdf_mem(&pix, &options).unwrap();

        // Verify PDF header
        assert!(pdf_data.starts_with(b"%PDF-"));
        // DCTDecode filter should be present in the PDF
        let pdf_str = String::from_utf8_lossy(&pdf_data);
        assert!(
            pdf_str.contains("DCTDecode"),
            "PDF should contain DCTDecode filter"
        );
    }

    #[test]
    fn test_write_pdf_jpeg_smaller_than_flate() {
        // For photographic images, JPEG should produce smaller output
        let pix = Pix::new(200, 200, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..200 {
            for x in 0..200 {
                pix_mut
                    .set_pixel(x, y, color::compose_rgb(x as u8, (y / 2) as u8, 100))
                    .unwrap();
            }
        }
        let pix: Pix = pix_mut.into();

        let flate_options = PdfOptions {
            compression: PdfCompression::Flate,
            ..Default::default()
        };
        let jpeg_options = PdfOptions {
            compression: PdfCompression::Jpeg,
            quality: 75,
            ..Default::default()
        };

        let flate_data = write_pdf_mem(&pix, &flate_options).unwrap();
        let jpeg_data = write_pdf_mem(&pix, &jpeg_options).unwrap();

        assert!(
            jpeg_data.len() < flate_data.len(),
            "JPEG ({}) should be smaller than Flate ({}) for photographic content",
            jpeg_data.len(),
            flate_data.len()
        );
    }

    #[test]
    fn test_write_pdf_jpeg_1bpp_fallback() {
        // 1bpp images should fall back to Flate even when Jpeg is selected
        let pix = Pix::new(80, 80, PixelDepth::Bit1).unwrap();
        let options = PdfOptions {
            compression: PdfCompression::Jpeg,
            ..Default::default()
        };
        let pdf_data = write_pdf_mem(&pix, &options).unwrap();

        // Should not contain DCTDecode since 1bpp falls back to Flate
        assert!(pdf_data.starts_with(b"%PDF-"));
        let pdf_str = String::from_utf8_lossy(&pdf_data);
        assert!(
            pdf_str.contains("FlateDecode"),
            "1bpp should use FlateDecode even with Jpeg option"
        );
    }

    #[test]
    fn test_write_pdf_jpeg_grayscale() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..100 {
            for x in 0..100 {
                pix_mut.set_pixel(x, y, ((x + y) * 2) % 256).unwrap();
            }
        }
        let pix: Pix = pix_mut.into();

        let options = PdfOptions {
            compression: PdfCompression::Jpeg,
            quality: 85,
            ..Default::default()
        };
        let pdf_data = write_pdf_mem(&pix, &options).unwrap();

        assert!(pdf_data.starts_with(b"%PDF-"));
        let pdf_str = String::from_utf8_lossy(&pdf_data);
        assert!(
            pdf_str.contains("DCTDecode"),
            "8bpp PDF should contain DCTDecode"
        );
    }

    #[test]
    fn test_write_pdf_from_files() {
        // Create temporary test images
        let outdir = std::env::temp_dir().join("leptonica_pdf_test");
        std::fs::create_dir_all(&outdir).unwrap();

        let pix1 = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();

        let path1 = outdir.join("test1.pnm");
        let path2 = outdir.join("test2.pnm");
        crate::write_image(&pix1, &path1, leptonica_core::ImageFormat::Pnm).unwrap();
        crate::write_image(&pix2, &path2, leptonica_core::ImageFormat::Pnm).unwrap();

        let options = PdfOptions::with_title("From Files Test");
        let mut buffer = Vec::new();
        write_pdf_from_files(&[&path1, &path2], &mut buffer, &options).unwrap();

        assert!(buffer.starts_with(b"%PDF-"));
        assert!(buffer.len() > 200);

        // Cleanup
        let _ = std::fs::remove_dir_all(&outdir);
    }
}
