//! PIX - The main image container
//!
//! The `Pix` structure is the fundamental image type in Leptonica.
//! It supports various pixel depths and optional colormaps.

mod access;
pub mod arith;
pub mod blend;
pub mod compare;
mod convert;
mod extract;
pub mod graphics;
mod histogram;
mod ops;
pub mod rop;
pub mod statistics;

pub use access::*;
pub use blend::{BlendMode, GrayBlendType, MaskBlendType, blend_with_gray_mask};
pub use compare::{CompareResult, CompareType, correlation_binary};
pub use graphics::{Color, ContourOutput, PixelOp};
pub use histogram::ColorHistogram;
pub use rop::RopOp;

use crate::colormap::PixColormap;
use crate::error::{Error, Result};
use std::sync::Arc;

/// Pixel depth (bits per pixel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PixelDepth {
    /// 1-bit binary image
    Bit1 = 1,
    /// 2-bit image (4 levels)
    Bit2 = 2,
    /// 4-bit image (16 levels)
    Bit4 = 4,
    /// 8-bit grayscale or indexed color
    Bit8 = 8,
    /// 16-bit grayscale
    Bit16 = 16,
    /// 32-bit RGB or RGBA
    Bit32 = 32,
}

impl PixelDepth {
    /// Create PixelDepth from a u32 value
    pub fn from_bits(bits: u32) -> Result<Self> {
        match bits {
            1 => Ok(PixelDepth::Bit1),
            2 => Ok(PixelDepth::Bit2),
            4 => Ok(PixelDepth::Bit4),
            8 => Ok(PixelDepth::Bit8),
            16 => Ok(PixelDepth::Bit16),
            32 => Ok(PixelDepth::Bit32),
            _ => Err(Error::InvalidDepth(bits)),
        }
    }

    /// Get the number of bits
    pub fn bits(self) -> u32 {
        self as u32
    }

    /// Check if colormap is allowed for this depth
    pub fn colormap_allowed(self) -> bool {
        matches!(
            self,
            PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8
        )
    }

    /// Get maximum pixel value for this depth
    pub fn max_value(self) -> u32 {
        (1u32 << self.bits()) - 1
    }
}

/// Image file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum ImageFormat {
    /// Unknown format
    #[default]
    Unknown = 0,
    /// BMP format
    Bmp = 1,
    /// JFIF JPEG format
    Jpeg = 2,
    /// PNG format
    Png = 3,
    /// TIFF format
    Tiff = 4,
    /// TIFF with packbits compression
    TiffPackbits = 5,
    /// TIFF with RLE compression
    TiffRle = 6,
    /// TIFF with G3 fax compression
    TiffG3 = 7,
    /// TIFF with G4 fax compression
    TiffG4 = 8,
    /// TIFF with LZW compression
    TiffLzw = 9,
    /// TIFF with ZIP compression
    TiffZip = 10,
    /// PNM format
    Pnm = 11,
    /// Postscript
    Ps = 12,
    /// GIF format
    Gif = 13,
    /// JPEG 2000
    Jp2 = 14,
    /// WebP format
    WebP = 15,
    /// LPDF format
    Lpdf = 16,
    /// TIFF with JPEG compression
    TiffJpeg = 17,
    /// Default format
    Default = 18,
    /// Serialized PIX format
    Spix = 19,
}

impl ImageFormat {
    /// Get the file extension for this format
    pub fn extension(self) -> &'static str {
        match self {
            Self::Unknown => "dat",
            Self::Bmp => "bmp",
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Tiff
            | Self::TiffPackbits
            | Self::TiffRle
            | Self::TiffG3
            | Self::TiffG4
            | Self::TiffLzw
            | Self::TiffZip
            | Self::TiffJpeg => "tif",
            Self::Pnm => "pnm",
            Self::Ps => "ps",
            Self::Gif => "gif",
            Self::Jp2 => "jp2",
            Self::WebP => "webp",
            Self::Lpdf => "pdf",
            Self::Default => "png",
            Self::Spix => "spix",
        }
    }
}

/// Internal PIX data
#[derive(Debug)]
struct PixData {
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Depth in bits per pixel
    depth: PixelDepth,
    /// Samples per pixel (1 for grayscale, 3 for RGB, 4 for RGBA)
    spp: u32,
    /// 32-bit words per line
    wpl: u32,
    /// X resolution (ppi), 0 if unknown
    xres: i32,
    /// Y resolution (ppi), 0 if unknown
    yres: i32,
    /// Input file format
    informat: ImageFormat,
    /// Special instructions for I/O
    special: i32,
    /// Text string associated with pix
    text: Option<String>,
    /// Colormap (optional)
    colormap: Option<PixColormap>,
    /// The image data (packed 32-bit words)
    data: Vec<u32>,
}

/// PIX - Main image container
///
/// `Pix` is the fundamental image type in Leptonica. It uses reference
/// counting via `Arc` for efficient cloning.
///
/// # Pixel Layout
///
/// - Image data is stored in 32-bit words
/// - Every row starts on a 32-bit boundary
/// - Pixels are packed MSB to LSB within each word
/// - For 32-bit images, color order is RGBA (red in MSB)
///
/// # Examples
///
/// ```
/// use leptonica_core::{Pix, PixelDepth};
///
/// // Create a new 8-bit grayscale image
/// let pix = Pix::new(640, 480, PixelDepth::Bit8).unwrap();
/// assert_eq!(pix.width(), 640);
/// assert_eq!(pix.height(), 480);
/// ```
#[derive(Debug, Clone)]
pub struct Pix {
    inner: Arc<PixData>,
}

impl Pix {
    /// Create a new PIX with the specified dimensions and depth.
    ///
    /// The image data is initialized to zero.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels (must be > 0)
    /// * `height` - Height in pixels (must be > 0)
    /// * `depth` - Pixel depth
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidDimension` if width or height is 0.
    pub fn new(width: u32, height: u32, depth: PixelDepth) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimension { width, height });
        }

        let wpl = Self::compute_wpl(width, depth);
        let data_size = (wpl as usize) * (height as usize);
        let data = vec![0u32; data_size];

        let spp = match depth {
            PixelDepth::Bit32 => 3, // Default to RGB
            _ => 1,
        };

        let inner = PixData {
            width,
            height,
            depth,
            spp,
            wpl,
            xres: 0,
            yres: 0,
            informat: ImageFormat::Unknown,
            special: 0,
            text: None,
            colormap: None,
            data,
        };

        Ok(Pix {
            inner: Arc::new(inner),
        })
    }

    /// Create a new PIX without initializing the data.
    ///
    /// This is faster than `new()` when you plan to set all pixels.
    ///
    /// # Safety
    ///
    /// The image data contains uninitialized values. You must set all
    /// pixels before reading them.
    pub fn new_uninit(width: u32, height: u32, depth: PixelDepth) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimension { width, height });
        }

        let wpl = Self::compute_wpl(width, depth);
        let data_size = (wpl as usize) * (height as usize);

        // Initialize data with zeros
        let data = vec![0u32; data_size];

        let spp = match depth {
            PixelDepth::Bit32 => 3,
            _ => 1,
        };

        let inner = PixData {
            width,
            height,
            depth,
            spp,
            wpl,
            xres: 0,
            yres: 0,
            informat: ImageFormat::Unknown,
            special: 0,
            text: None,
            colormap: None,
            data,
        };

        Ok(Pix {
            inner: Arc::new(inner),
        })
    }

    /// Compute words per line for given width and depth
    #[inline]
    fn compute_wpl(width: u32, depth: PixelDepth) -> u32 {
        let bits_per_line = width * depth.bits();
        bits_per_line.div_ceil(32)
    }

    /// Get the image width in pixels
    #[inline]
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// Get the image height in pixels
    #[inline]
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    /// Get the pixel depth
    #[inline]
    pub fn depth(&self) -> PixelDepth {
        self.inner.depth
    }

    /// Get the samples per pixel
    #[inline]
    pub fn spp(&self) -> u32 {
        self.inner.spp
    }

    /// Get the words per line
    #[inline]
    pub fn wpl(&self) -> u32 {
        self.inner.wpl
    }

    /// Get the X resolution (ppi)
    #[inline]
    pub fn xres(&self) -> i32 {
        self.inner.xres
    }

    /// Get the Y resolution (ppi)
    #[inline]
    pub fn yres(&self) -> i32 {
        self.inner.yres
    }

    /// Get the input file format
    #[inline]
    pub fn informat(&self) -> ImageFormat {
        self.inner.informat
    }

    /// Get the special field
    #[inline]
    pub fn special(&self) -> i32 {
        self.inner.special
    }

    /// Get the associated text
    #[inline]
    pub fn text(&self) -> Option<&str> {
        self.inner.text.as_deref()
    }

    /// Get the colormap (if any)
    #[inline]
    pub fn colormap(&self) -> Option<&PixColormap> {
        self.inner.colormap.as_ref()
    }

    /// Check if the image has a colormap
    #[inline]
    pub fn has_colormap(&self) -> bool {
        self.inner.colormap.is_some()
    }

    /// Get raw access to the image data
    #[inline]
    pub fn data(&self) -> &[u32] {
        &self.inner.data
    }

    /// Get the number of strong references to this PIX
    #[inline]
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Get a pointer to the start of a specific row
    ///
    /// # Arguments
    ///
    /// * `y` - Row index (0-based)
    ///
    /// # Panics
    ///
    /// Panics if `y >= height`
    #[inline]
    pub fn row_data(&self, y: u32) -> &[u32] {
        let start = (y * self.inner.wpl) as usize;
        let end = start + self.inner.wpl as usize;
        &self.inner.data[start..end]
    }

    /// Create a deep copy of this PIX
    ///
    /// Unlike `clone()` which shares data via Arc, this creates
    /// a completely independent copy.
    pub fn deep_clone(&self) -> Self {
        let inner = PixData {
            width: self.inner.width,
            height: self.inner.height,
            depth: self.inner.depth,
            spp: self.inner.spp,
            wpl: self.inner.wpl,
            xres: self.inner.xres,
            yres: self.inner.yres,
            informat: self.inner.informat,
            special: self.inner.special,
            text: self.inner.text.clone(),
            colormap: self.inner.colormap.clone(),
            data: self.inner.data.clone(),
        };

        Pix {
            inner: Arc::new(inner),
        }
    }

    /// Try to get mutable access to the image data
    ///
    /// This succeeds only if there is exactly one reference to the data.
    /// If successful, returns a `PixMut` that allows modification.
    pub fn try_into_mut(self) -> std::result::Result<PixMut, Self> {
        match Arc::try_unwrap(self.inner) {
            Ok(data) => Ok(PixMut { inner: data }),
            Err(arc) => Err(Pix { inner: arc }),
        }
    }

    /// Create a mutable copy of this PIX
    ///
    /// This always creates a new copy that can be modified.
    pub fn to_mut(&self) -> PixMut {
        let inner = PixData {
            width: self.inner.width,
            height: self.inner.height,
            depth: self.inner.depth,
            spp: self.inner.spp,
            wpl: self.inner.wpl,
            xres: self.inner.xres,
            yres: self.inner.yres,
            informat: self.inner.informat,
            special: self.inner.special,
            text: self.inner.text.clone(),
            colormap: self.inner.colormap.clone(),
            data: self.inner.data.clone(),
        };

        PixMut { inner }
    }
}

/// Mutable PIX
///
/// This type allows modification of image data. It can be converted
/// back to an immutable `Pix` using `into()`.
#[derive(Debug)]
pub struct PixMut {
    inner: PixData,
}

impl PixMut {
    /// Get the image width
    #[inline]
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// Get the image height
    #[inline]
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    /// Get the pixel depth
    #[inline]
    pub fn depth(&self) -> PixelDepth {
        self.inner.depth
    }

    /// Get samples per pixel
    #[inline]
    pub fn spp(&self) -> u32 {
        self.inner.spp
    }

    /// Get words per line
    #[inline]
    pub fn wpl(&self) -> u32 {
        self.inner.wpl
    }

    /// Set the X resolution
    pub fn set_xres(&mut self, xres: i32) {
        self.inner.xres = xres;
    }

    /// Set the Y resolution
    pub fn set_yres(&mut self, yres: i32) {
        self.inner.yres = yres;
    }

    /// Set both resolutions
    pub fn set_resolution(&mut self, xres: i32, yres: i32) {
        self.inner.xres = xres;
        self.inner.yres = yres;
    }

    /// Set the input format
    pub fn set_informat(&mut self, format: ImageFormat) {
        self.inner.informat = format;
    }

    /// Set the special field
    pub fn set_special(&mut self, special: i32) {
        self.inner.special = special;
    }

    /// Set the text
    pub fn set_text(&mut self, text: Option<String>) {
        self.inner.text = text;
    }

    /// Set samples per pixel
    pub fn set_spp(&mut self, spp: u32) {
        self.inner.spp = spp;
    }

    /// Set the colormap
    pub fn set_colormap(&mut self, colormap: Option<PixColormap>) -> Result<()> {
        if colormap.is_some() && !self.inner.depth.colormap_allowed() {
            return Err(Error::ColormapNotAllowed(self.inner.depth.bits()));
        }
        self.inner.colormap = colormap;
        Ok(())
    }

    /// Get raw access to the image data
    #[inline]
    pub fn data(&self) -> &[u32] {
        &self.inner.data
    }

    /// Get mutable access to the image data
    #[inline]
    pub fn data_mut(&mut self) -> &mut [u32] {
        &mut self.inner.data
    }

    /// Get mutable access to a specific row
    #[inline]
    pub fn row_data_mut(&mut self, y: u32) -> &mut [u32] {
        let start = (y * self.inner.wpl) as usize;
        let end = start + self.inner.wpl as usize;
        &mut self.inner.data[start..end]
    }

    /// Clear all pixels to zero
    pub fn clear(&mut self) {
        self.inner.data.fill(0);
    }

    /// Set all pixels to one (all bits set)
    pub fn set_all(&mut self) {
        self.inner.data.fill(0xFFFFFFFF);
    }
}

impl From<PixMut> for Pix {
    fn from(pix_mut: PixMut) -> Self {
        Pix {
            inner: Arc::new(pix_mut.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_depth() {
        assert_eq!(PixelDepth::from_bits(1).unwrap(), PixelDepth::Bit1);
        assert_eq!(PixelDepth::from_bits(8).unwrap(), PixelDepth::Bit8);
        assert_eq!(PixelDepth::from_bits(32).unwrap(), PixelDepth::Bit32);
        assert!(PixelDepth::from_bits(3).is_err());

        assert_eq!(PixelDepth::Bit8.bits(), 8);
        assert_eq!(PixelDepth::Bit8.max_value(), 255);
        assert!(PixelDepth::Bit8.colormap_allowed());
        assert!(!PixelDepth::Bit32.colormap_allowed());
    }

    #[test]
    fn test_pix_creation() {
        let pix = Pix::new(100, 200, PixelDepth::Bit8).unwrap();
        assert_eq!(pix.width(), 100);
        assert_eq!(pix.height(), 200);
        assert_eq!(pix.depth(), PixelDepth::Bit8);
        assert_eq!(pix.spp(), 1);

        // Check wpl calculation: 100 * 8 = 800 bits = 25 words
        assert_eq!(pix.wpl(), 25);
    }

    #[test]
    fn test_pix_creation_invalid() {
        assert!(Pix::new(0, 100, PixelDepth::Bit8).is_err());
        assert!(Pix::new(100, 0, PixelDepth::Bit8).is_err());
    }

    #[test]
    fn test_pix_clone_shares_data() {
        let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let pix2 = pix1.clone();

        // Both should point to the same data
        assert_eq!(pix1.ref_count(), 2);
        assert_eq!(pix2.ref_count(), 2);
        assert_eq!(pix1.data().as_ptr(), pix2.data().as_ptr());
    }

    #[test]
    fn test_pix_deep_clone() {
        let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let pix2 = pix1.deep_clone();

        // Should have separate data
        assert_eq!(pix1.ref_count(), 1);
        assert_eq!(pix2.ref_count(), 1);
        assert_ne!(pix1.data().as_ptr(), pix2.data().as_ptr());
    }

    #[test]
    fn test_pix_mut() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        pix_mut.set_xres(300);
        pix_mut.set_yres(300);
        pix_mut.set_text(Some("test".to_string()));

        let pix: Pix = pix_mut.into();
        assert_eq!(pix.xres(), 300);
        assert_eq!(pix.yres(), 300);
        assert_eq!(pix.text(), Some("test"));
    }

    #[test]
    fn test_wpl_calculation() {
        // 1-bit: 32 pixels fit in 1 word
        let pix = Pix::new(32, 1, PixelDepth::Bit1).unwrap();
        assert_eq!(pix.wpl(), 1);

        let pix = Pix::new(33, 1, PixelDepth::Bit1).unwrap();
        assert_eq!(pix.wpl(), 2);

        // 32-bit: 1 pixel per word
        let pix = Pix::new(10, 1, PixelDepth::Bit32).unwrap();
        assert_eq!(pix.wpl(), 10);
    }
}
