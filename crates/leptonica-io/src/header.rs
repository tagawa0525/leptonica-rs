//! Image header reading
//!
//! Provides metadata extraction from image files without decoding pixel data.
//!
//! # See also
//!
//! C Leptonica: `readfile.c` (`pixReadHeader*`), `writefile.c`

use crate::{IoError, IoResult, detect_format_from_bytes};
use leptonica_core::ImageFormat;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Image metadata read without decoding pixel data
///
/// # See also
///
/// C Leptonica: `pixReadHeader()` in `readfile.c`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageHeader {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Bit depth (bits per pixel: 1, 2, 4, 8, 16, or 32)
    pub depth: u32,
    /// Bits per sample
    pub bps: u32,
    /// Samples per pixel (1 for grayscale, 3 for RGB, 4 for RGBA)
    pub spp: u32,
    /// Whether the image has a colormap
    pub has_colormap: bool,
    /// Number of colormap entries (0 if no colormap)
    pub num_colors: u32,
    /// Detected image format
    pub format: ImageFormat,
    /// X resolution in pixels per inch (DPI), if available
    pub x_resolution: Option<u32>,
    /// Y resolution in pixels per inch (DPI), if available
    pub y_resolution: Option<u32>,
}

/// Read image metadata from a file path without decoding pixel data
///
/// # See also
///
/// C Leptonica: `pixReadHeader()` in `readfile.c`
pub fn read_image_header<P: AsRef<Path>>(path: P) -> IoResult<ImageHeader> {
    let path = path.as_ref();
    let file = File::open(path).map_err(IoError::Io)?;
    let mut reader = BufReader::new(file);
    let mut data = Vec::new();
    reader.read_to_end(&mut data).map_err(IoError::Io)?;
    read_image_header_mem(&data)
}

/// Read image metadata from bytes without decoding pixel data
///
/// # See also
///
/// C Leptonica: `pixReadHeaderMem()` in `readfile.c`
pub fn read_image_header_mem(data: &[u8]) -> IoResult<ImageHeader> {
    let format = detect_format_from_bytes(data)?;
    read_header_for_format(data, format)
}

/// Read header for a specific format
fn read_header_for_format(data: &[u8], format: ImageFormat) -> IoResult<ImageHeader> {
    match format {
        #[cfg(feature = "bmp")]
        ImageFormat::Bmp => crate::bmp::read_header_bmp(data),

        #[cfg(feature = "pnm")]
        ImageFormat::Pnm => crate::pnm::read_header_pnm(data),

        #[cfg(feature = "png-format")]
        ImageFormat::Png => crate::png::read_header_png(data),

        #[cfg(feature = "jpeg")]
        ImageFormat::Jpeg => crate::jpeg::read_header_jpeg(data),

        #[cfg(feature = "tiff-format")]
        ImageFormat::Tiff
        | ImageFormat::TiffG3
        | ImageFormat::TiffG4
        | ImageFormat::TiffRle
        | ImageFormat::TiffPackbits
        | ImageFormat::TiffLzw
        | ImageFormat::TiffZip
        | ImageFormat::TiffJpeg => crate::tiff::read_header_tiff(data),

        #[cfg(feature = "gif-format")]
        ImageFormat::Gif => crate::gif::read_header_gif(data),

        #[cfg(feature = "webp-format")]
        ImageFormat::WebP => crate::webp::read_header_webp(data),

        #[cfg(feature = "jp2k-format")]
        ImageFormat::Jp2 => crate::jp2k::read_header_jp2k(data),

        ImageFormat::Spix => crate::spix::read_header_spix(data),

        _ => Err(IoError::UnsupportedFormat(format!("{:?}", format))),
    }
}

/// Choose the best output format for a given image.
///
/// Selection rules (following C Leptonica `pixChooseOutputFormat`):
/// - 1 bpp without colormap → PNG (lossless binary)
/// - 1–8 bpp with colormap → PNG (lossless indexed)
/// - 8 bpp grayscale → PNG
/// - 16 bpp → PNG
/// - 32 bpp (spp=3, no alpha) → JPEG
/// - 32 bpp (spp=4, with alpha) → PNG
///
/// # See also
///
/// C Leptonica: `pixChooseOutputFormat()` in `writefile.c`
pub fn choose_output_format(pix: &leptonica_core::Pix) -> ImageFormat {
    use leptonica_core::PixelDepth;
    match pix.depth() {
        PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 => ImageFormat::Png,
        PixelDepth::Bit8 => ImageFormat::Png,
        PixelDepth::Bit16 => ImageFormat::Png,
        PixelDepth::Bit32 => {
            if pix.spp() == 3 {
                ImageFormat::Jpeg
            } else {
                ImageFormat::Png
            }
        }
    }
}

/// Write an image to a file, selecting the format from the file extension.
///
/// Falls back to `choose_output_format` if the extension is not recognized.
///
/// # See also
///
/// C Leptonica: `pixWrite()` in `writefile.c`
pub fn write_image_auto<P: AsRef<Path>>(pix: &leptonica_core::Pix, path: P) -> IoResult<()> {
    let path = path.as_ref();
    let format = ImageFormat::from_path(path).unwrap_or_else(|| choose_output_format(pix));
    crate::write_image(pix, path, format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{ImageFormat, Pix, PixelDepth};

    // --- from_extension tests ---

    #[test]
    fn test_from_extension_bmp() {
        assert_eq!(ImageFormat::from_extension("bmp"), Some(ImageFormat::Bmp));
        assert_eq!(ImageFormat::from_extension("BMP"), Some(ImageFormat::Bmp));
    }

    #[test]
    fn test_from_extension_jpeg() {
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("JPG"), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn test_from_extension_png() {
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
    }

    #[test]
    fn test_from_extension_tiff() {
        assert_eq!(ImageFormat::from_extension("tif"), Some(ImageFormat::Tiff));
        assert_eq!(ImageFormat::from_extension("tiff"), Some(ImageFormat::Tiff));
    }

    #[test]
    fn test_from_extension_pnm() {
        assert_eq!(ImageFormat::from_extension("pnm"), Some(ImageFormat::Pnm));
        assert_eq!(ImageFormat::from_extension("pbm"), Some(ImageFormat::Pnm));
        assert_eq!(ImageFormat::from_extension("pgm"), Some(ImageFormat::Pnm));
        assert_eq!(ImageFormat::from_extension("ppm"), Some(ImageFormat::Pnm));
    }

    #[test]
    fn test_from_extension_gif() {
        assert_eq!(ImageFormat::from_extension("gif"), Some(ImageFormat::Gif));
    }

    #[test]
    fn test_from_extension_jp2k() {
        assert_eq!(ImageFormat::from_extension("jp2"), Some(ImageFormat::Jp2));
        assert_eq!(ImageFormat::from_extension("j2k"), Some(ImageFormat::Jp2));
    }

    #[test]
    fn test_from_extension_ps() {
        assert_eq!(ImageFormat::from_extension("ps"), Some(ImageFormat::Ps));
    }

    #[test]
    fn test_from_extension_pdf() {
        assert_eq!(ImageFormat::from_extension("pdf"), Some(ImageFormat::Lpdf));
    }

    #[test]
    fn test_from_extension_webp() {
        assert_eq!(ImageFormat::from_extension("webp"), Some(ImageFormat::WebP));
    }

    #[test]
    fn test_from_extension_spix() {
        assert_eq!(ImageFormat::from_extension("spix"), Some(ImageFormat::Spix));
    }

    #[test]
    fn test_from_extension_unknown() {
        assert_eq!(ImageFormat::from_extension("xyz"), None);
        assert_eq!(ImageFormat::from_extension(""), None);
    }

    #[test]
    fn test_from_path() {
        let path = std::path::Path::new("image.png");
        assert_eq!(ImageFormat::from_path(path), Some(ImageFormat::Png));

        let path = std::path::Path::new("/tmp/test.jpg");
        assert_eq!(ImageFormat::from_path(path), Some(ImageFormat::Jpeg));

        let path = std::path::Path::new("noext");
        assert_eq!(ImageFormat::from_path(path), None);
    }

    // --- choose_output_format tests ---

    #[test]
    fn test_choose_output_format_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert_eq!(choose_output_format(&pix), ImageFormat::Png);
    }

    #[test]
    fn test_choose_output_format_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert_eq!(choose_output_format(&pix), ImageFormat::Png);
    }

    #[test]
    fn test_choose_output_format_32bpp_rgb() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_spp(3);
        let pix: Pix = pix_mut.into();
        assert_eq!(choose_output_format(&pix), ImageFormat::Jpeg);
    }

    #[test]
    fn test_choose_output_format_32bpp_rgba() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_spp(4);
        let pix: Pix = pix_mut.into();
        assert_eq!(choose_output_format(&pix), ImageFormat::Png);
    }

    // --- read_image_header tests (require format implementations) ---

    #[test]
    #[ignore = "not yet implemented"]
    fn test_read_header_png() {
        use leptonica_core::PixelDepth;
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        let data = crate::write_image_mem(&pix, ImageFormat::Png).unwrap();
        let header = read_image_header_mem(&data).unwrap();
        assert_eq!(header.width, 100);
        assert_eq!(header.height, 80);
        assert_eq!(header.depth, 8);
        assert_eq!(header.format, ImageFormat::Png);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_read_header_jpeg() {
        use leptonica_core::PixelDepth;
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        let data = crate::write_image_mem(&pix, ImageFormat::Jpeg).unwrap();
        let header = read_image_header_mem(&data).unwrap();
        assert_eq!(header.width, 100);
        assert_eq!(header.height, 80);
        assert_eq!(header.format, ImageFormat::Jpeg);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_read_header_spix() {
        use leptonica_core::PixelDepth;
        let pix = Pix::new(50, 40, PixelDepth::Bit8).unwrap();
        let data = crate::write_image_mem(&pix, ImageFormat::Spix).unwrap();
        let header = read_image_header_mem(&data).unwrap();
        assert_eq!(header.width, 50);
        assert_eq!(header.height, 40);
        assert_eq!(header.depth, 8);
        assert_eq!(header.format, ImageFormat::Spix);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_read_header_bmp() {
        use leptonica_core::PixelDepth;
        let pix = Pix::new(30, 20, PixelDepth::Bit8).unwrap();
        let data = crate::write_image_mem(&pix, ImageFormat::Bmp).unwrap();
        let header = read_image_header_mem(&data).unwrap();
        assert_eq!(header.width, 30);
        assert_eq!(header.height, 20);
        assert_eq!(header.format, ImageFormat::Bmp);
    }
}
