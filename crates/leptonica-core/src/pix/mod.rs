//! PIX - The main image container
//!
//! The `Pix` structure is the fundamental image type in Leptonica.
//! It supports various pixel depths and optional colormaps.
//!
//! # Pixel layout
//!
//! - Image data is stored in 32-bit words
//! - Every row starts on a 32-bit boundary
//! - Pixels are packed MSB to LSB within each word
//! - For 32-bit images, color order is RGBA (red in MSB)
//!
//! # Ownership model
//!
//! `Pix` uses `Arc` for efficient cloning (shared ownership).
//! To modify pixel data, convert to `PixMut` via [`Pix::try_into_mut`]
//! or [`Pix::to_mut`], then convert back with `Into<Pix>`.
//!
//! # See also
//!
//! - C Leptonica: `pix.h` (struct `Pix`), `pix1.c` (creation/destruction)
//! - Pixel access: `GET_DATA_*` / `SET_DATA_*` macros in `arrayaccess.h`

mod access;
pub mod arith;
pub mod blend;
mod border;
mod clip;
pub mod compare;
pub mod convert;
mod extract;
pub mod graphics;
mod histogram;
mod mask;
mod ops;
mod rgb;
pub mod rop;
pub mod serial;
pub mod statistics;

pub use access::*;
pub use blend::{BlendMode, GrayBlendType, MaskBlendType, blend_with_gray_mask};
pub use clip::ScanDirection;
pub use compare::{CompareResult, CompareType, PixelDiffResult, correlation_binary};
pub use convert::{Convert16To8Type, GrayConversionType, MinMaxType, RemoveColormapTarget};
pub use graphics::{Color, ContourOutput, PixelOp};
pub use histogram::ColorHistogram;
pub use rgb::RgbComponent;
pub use rop::{InColor, RopOp};

use crate::error::{Error, Result};
use std::sync::Arc;

/// Initial color for `Pix::new_with_colormap`.
///
/// # See also
///
/// C Leptonica: `L_SET_BLACK`, `L_SET_WHITE` in `pix.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitColor {
    /// Initialize to black (R=0, G=0, B=0)
    Black,
    /// Initialize to white (R=255, G=255, B=255)
    White,
}

/// Pixel depth (bits per pixel)
///
/// Represents the number of bits used to encode each pixel.
///
/// # See also
///
/// C Leptonica: `pix.h` depth field (1, 2, 4, 8, 16, 32)
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
    /// Create `PixelDepth` from a raw bit count.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidDepth`] if `bits` is not 1, 2, 4, 8, 16, or 32.
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

    /// Get the number of bits per pixel.
    pub fn bits(self) -> u32 {
        self as u32
    }

    /// Check if a colormap is allowed for this depth.
    ///
    /// Colormaps are supported for 1, 2, 4, and 8 bpp images only.
    pub fn colormap_allowed(self) -> bool {
        matches!(
            self,
            PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8
        )
    }

    /// Get the maximum pixel value representable at this depth.
    pub fn max_value(self) -> u32 {
        match self {
            PixelDepth::Bit32 => u32::MAX,
            _ => (1u32 << self.bits()) - 1,
        }
    }
}

/// Image file format
///
/// # See also
///
/// C Leptonica: `imageio.h` (`IFF_*` constants)
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
    /// Get the file extension for this format.
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
    /// Optional colormap for indexed images (1, 2, 4, 8 bpp)
    colormap: Option<crate::PixColormap>,
    /// The image data (packed 32-bit words)
    data: Vec<u32>,
}

/// PIX - Main image container
///
/// `Pix` is the fundamental image type in Leptonica. It uses reference
/// counting via `Arc` for efficient cloning.
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
///
/// # See also
///
/// C Leptonica: `struct Pix` in `pix.h`, creation via `pixCreate()` in `pix1.c`
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
    /// Returns [`Error::InvalidDimension`] if width or height is 0.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCreate()` in `pix1.c`
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

    /// Compute words per line for given width and depth.
    ///
    /// Uses u64 arithmetic to prevent overflow for large widths.
    ///
    /// # Panics
    ///
    /// Panics if the result would exceed `u32::MAX`.
    #[inline]
    fn compute_wpl(width: u32, depth: PixelDepth) -> u32 {
        let bits_per_line = u64::from(width) * u64::from(depth.bits());
        let wpl = bits_per_line.div_ceil(32);
        u32::try_from(wpl).unwrap_or_else(|_| {
            panic!(
                "image row too large: width={} depth={:?} requires {} words",
                width, depth, wpl
            )
        })
    }

    /// Get the image width in pixels.
    #[inline]
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// Get the image height in pixels.
    #[inline]
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    /// Get the pixel depth.
    #[inline]
    pub fn depth(&self) -> PixelDepth {
        self.inner.depth
    }

    /// Get the samples per pixel.
    #[inline]
    pub fn spp(&self) -> u32 {
        self.inner.spp
    }

    /// Get the words per line.
    #[inline]
    pub fn wpl(&self) -> u32 {
        self.inner.wpl
    }

    /// Get the X resolution (ppi).
    #[inline]
    pub fn xres(&self) -> i32 {
        self.inner.xres
    }

    /// Get the Y resolution (ppi).
    #[inline]
    pub fn yres(&self) -> i32 {
        self.inner.yres
    }

    /// Get the input file format.
    #[inline]
    pub fn informat(&self) -> ImageFormat {
        self.inner.informat
    }

    /// Get the special field.
    #[inline]
    pub fn special(&self) -> i32 {
        self.inner.special
    }

    /// Get the associated text.
    #[inline]
    pub fn text(&self) -> Option<&str> {
        self.inner.text.as_deref()
    }

    /// Check whether this image has a colormap attached.
    #[inline]
    pub fn has_colormap(&self) -> bool {
        self.inner.colormap.is_some()
    }

    /// Get a reference to the image's colormap, if present.
    #[inline]
    pub fn colormap(&self) -> Option<&crate::PixColormap> {
        self.inner.colormap.as_ref()
    }

    /// Get raw access to the image data.
    #[inline]
    pub fn data(&self) -> &[u32] {
        &self.inner.data
    }

    /// Get the number of strong references to this PIX.
    #[inline]
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Get a pointer to the start of a specific row.
    ///
    /// # Panics
    ///
    /// Panics if `y >= height`.
    #[inline]
    pub fn row_data(&self, y: u32) -> &[u32] {
        let start = (y * self.inner.wpl) as usize;
        let end = start + self.inner.wpl as usize;
        &self.inner.data[start..end]
    }

    /// Create a new PIX with the same dimensions and metadata as the source.
    ///
    /// The image data is initialized to zero. Copies resolution, colormap,
    /// text, input format, and spp from the source.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCreateTemplate()` in `pix1.c`
    pub fn create_template(&self) -> Self {
        let wpl = self.inner.wpl;
        let data_size = (wpl as usize) * (self.inner.height as usize);
        let inner = PixData {
            width: self.inner.width,
            height: self.inner.height,
            depth: self.inner.depth,
            spp: self.inner.spp,
            wpl,
            xres: self.inner.xres,
            yres: self.inner.yres,
            informat: self.inner.informat,
            special: 0,
            text: self.inner.text.clone(),
            colormap: self.inner.colormap.clone(),
            data: vec![0u32; data_size],
        };
        Pix {
            inner: Arc::new(inner),
        }
    }

    /// Create a new PIX with a colormap initialized to the given color.
    ///
    /// All pixels are initialized to 0 (the first colormap entry).
    /// The colormap's first entry is set to `init_color`.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `depth` - Pixel depth (must be 2, 4, or 8)
    /// * `init_color` - Initial color for colormap index 0
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCreateWithCmap()` in `pix1.c`
    pub fn new_with_colormap(
        width: u32,
        height: u32,
        depth: PixelDepth,
        init_color: InitColor,
    ) -> Result<Self> {
        if !matches!(
            depth,
            PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8
        ) {
            return Err(Error::InvalidParameter(format!(
                "new_with_colormap requires depth 2, 4, or 8; got {}",
                depth.bits()
            )));
        }
        let pix = Self::new(width, height, depth)?;
        let mut cmap = crate::PixColormap::new(depth.bits())?;
        match init_color {
            InitColor::Black => cmap.add_rgb(0, 0, 0)?,
            InitColor::White => cmap.add_rgb(255, 255, 255)?,
        };
        let mut pix_mut = pix.try_into_mut().unwrap();
        // set_colormap won't fail: depth is already validated
        pix_mut.set_colormap(Some(cmap)).unwrap();
        Ok(pix_mut.into())
    }

    /// Check if two PIX have the same width, height, and depth.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSizesEqual()` in `pix1.c`
    pub fn sizes_equal(&self, other: &Pix) -> bool {
        self.inner.width == other.inner.width
            && self.inner.height == other.inner.height
            && self.inner.depth == other.inner.depth
    }

    /// Get the maximum aspect ratio (>= 1.0).
    ///
    /// Returns `max(w/h, h/w)`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixMaxAspectRatio()` in `pix1.c`
    pub fn max_aspect_ratio(&self) -> f32 {
        let w = self.inner.width as f32;
        let h = self.inner.height as f32;
        f32::max(w / h, h / w)
    }

    /// Write image metadata to a writer (for debugging).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixPrintStreamInfo()` in `pix1.c`
    pub fn print_info(&self, writer: &mut impl std::io::Write, label: Option<&str>) -> Result<()> {
        if let Some(text) = label {
            writeln!(writer, "  Pix Info for {text}:")?;
        }
        writeln!(
            writer,
            "    width = {}, height = {}, depth = {}, spp = {}",
            self.inner.width,
            self.inner.height,
            self.inner.depth.bits(),
            self.inner.spp
        )?;
        writeln!(writer, "    wpl = {}", self.inner.wpl)?;
        writeln!(
            writer,
            "    xres = {}, yres = {}",
            self.inner.xres, self.inner.yres
        )?;
        if let Some(ref cmap) = self.inner.colormap {
            writeln!(writer, "    colormap: {} colors", cmap.len())?;
        } else {
            writeln!(writer, "    no colormap")?;
        }
        writeln!(
            writer,
            "    input format: {} ({})",
            self.inner.informat as i32,
            self.inner.informat.extension()
        )?;
        if let Some(ref text) = self.inner.text {
            writeln!(writer, "    text: {text}")?;
        }
        Ok(())
    }

    /// Create a deep copy of this PIX.
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

    /// Try to get mutable access to the image data.
    ///
    /// Succeeds only if there is exactly one reference to the data.
    /// If successful, returns a [`PixMut`] that allows modification.
    pub fn try_into_mut(self) -> std::result::Result<PixMut, Self> {
        match Arc::try_unwrap(self.inner) {
            Ok(data) => Ok(PixMut { inner: data }),
            Err(arc) => Err(Pix { inner: arc }),
        }
    }

    /// Create a mutable copy of this PIX.
    ///
    /// Always creates a new copy that can be modified.
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
/// Allows modification of image data. Convert back to an immutable
/// [`Pix`] using `Into<Pix>`.
///
/// # See also
///
/// C Leptonica does not have a separate mutable type; it relies on
/// reference counting. This Rust design enforces exclusive access
/// at compile time.
#[derive(Debug)]
pub struct PixMut {
    inner: PixData,
}

impl PixMut {
    /// Get the image width.
    #[inline]
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// Get the image height.
    #[inline]
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    /// Get the pixel depth.
    #[inline]
    pub fn depth(&self) -> PixelDepth {
        self.inner.depth
    }

    /// Get samples per pixel.
    #[inline]
    pub fn spp(&self) -> u32 {
        self.inner.spp
    }

    /// Get words per line.
    #[inline]
    pub fn wpl(&self) -> u32 {
        self.inner.wpl
    }

    /// Get the X resolution (ppi).
    #[inline]
    pub fn xres(&self) -> i32 {
        self.inner.xres
    }

    /// Get the Y resolution (ppi).
    #[inline]
    pub fn yres(&self) -> i32 {
        self.inner.yres
    }

    /// Get the input file format.
    #[inline]
    pub fn informat(&self) -> ImageFormat {
        self.inner.informat
    }

    /// Get the associated text.
    #[inline]
    pub fn text(&self) -> Option<&str> {
        self.inner.text.as_deref()
    }

    /// Set the X resolution.
    pub fn set_xres(&mut self, xres: i32) {
        self.inner.xres = xres;
    }

    /// Set the Y resolution.
    pub fn set_yres(&mut self, yres: i32) {
        self.inner.yres = yres;
    }

    /// Set both resolutions.
    pub fn set_resolution(&mut self, xres: i32, yres: i32) {
        self.inner.xres = xres;
        self.inner.yres = yres;
    }

    /// Set the input format.
    pub fn set_informat(&mut self, format: ImageFormat) {
        self.inner.informat = format;
    }

    /// Set the special field.
    pub fn set_special(&mut self, special: i32) {
        self.inner.special = special;
    }

    /// Set the text.
    pub fn set_text(&mut self, text: Option<String>) {
        self.inner.text = text;
    }

    /// Set samples per pixel.
    pub fn set_spp(&mut self, spp: u32) {
        self.inner.spp = spp;
    }

    /// Get raw access to the image data.
    #[inline]
    pub fn data(&self) -> &[u32] {
        &self.inner.data
    }

    /// Get mutable access to the image data.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [u32] {
        &mut self.inner.data
    }

    /// Get mutable access to a specific row.
    ///
    /// # Panics
    ///
    /// Panics if `y >= height`.
    #[inline]
    pub fn row_data_mut(&mut self, y: u32) -> &mut [u32] {
        let start = (y * self.inner.wpl) as usize;
        let end = start + self.inner.wpl as usize;
        &mut self.inner.data[start..end]
    }

    /// Check whether this image has a colormap attached.
    #[inline]
    pub fn has_colormap(&self) -> bool {
        self.inner.colormap.is_some()
    }

    /// Get a reference to the image's colormap, if present.
    #[inline]
    pub fn colormap(&self) -> Option<&crate::PixColormap> {
        self.inner.colormap.as_ref()
    }

    /// Set or remove the colormap.
    ///
    /// Colormaps are only valid for 1, 2, 4, and 8 bpp images.
    pub fn set_colormap(&mut self, cmap: Option<crate::PixColormap>) -> Result<()> {
        if let Some(ref cm) = cmap {
            if !self.inner.depth.colormap_allowed() {
                return Err(Error::ColormapNotAllowed(self.inner.depth.bits()));
            }
            if cm.depth() != self.inner.depth.bits() {
                return Err(Error::InvalidParameter(format!(
                    "colormap depth {} does not match image depth {}",
                    cm.depth(),
                    self.inner.depth.bits()
                )));
            }
        }
        self.inner.colormap = cmap;
        Ok(())
    }

    /// Copy colormap from another PIX.
    ///
    /// If the source has no colormap, the destination's colormap is removed.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCopyColormap()` in `pix1.c`
    pub fn copy_colormap_from(&mut self, src: &Pix) {
        self.inner.colormap = src.inner.colormap.clone();
    }

    /// Copy resolution (xres, yres) from another PIX.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCopyResolution()` in `pix1.c`
    pub fn copy_resolution_from(&mut self, src: &Pix) {
        self.inner.xres = src.inner.xres;
        self.inner.yres = src.inner.yres;
    }

    /// Scale the resolution by the given factors.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixScaleResolution()` in `pix1.c`
    pub fn scale_resolution(&mut self, xscale: f32, yscale: f32) {
        self.inner.xres = (self.inner.xres as f32 * xscale) as i32;
        self.inner.yres = (self.inner.yres as f32 * yscale) as i32;
    }

    /// Copy the input format from another PIX.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCopyInputFormat()` in `pix1.c`
    pub fn copy_input_format_from(&mut self, src: &Pix) {
        self.inner.informat = src.inner.informat;
    }

    /// Append text to existing text.
    ///
    /// If no existing text, just sets the text.
    /// If `text` is None, this is a no-op.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddText()` in `pix1.c`
    pub fn add_text(&mut self, text: Option<&str>) {
        let Some(new_text) = text else {
            return;
        };
        match self.inner.text {
            Some(ref mut existing) => existing.push_str(new_text),
            None => self.inner.text = Some(new_text.to_string()),
        }
    }

    /// Copy text from another PIX.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCopyText()` in `pix1.c`
    pub fn copy_text_from(&mut self, src: &Pix) {
        self.inner.text = src.inner.text.clone();
    }

    /// Set all pixels to a gray value (0-255).
    ///
    /// For 32 bpp, sets R=G=B=grayval, alpha unchanged.
    /// For colormapped images, finds/adds nearest gray.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetAllGray()` in `pix2.c`
    pub fn set_all_gray(&mut self, _grayval: u8) -> Result<()> {
        todo!()
    }

    /// Set all pixels to an arbitrary value.
    ///
    /// The value is depth-dependent:
    /// - 1 bpp: 0 or 1
    /// - 2 bpp: 0-3
    /// - 4 bpp: 0-15
    /// - 8 bpp: 0-255
    /// - 16 bpp: 0-65535
    /// - 32 bpp: full 32-bit value
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetAllArbitrary()` in `pix2.c`
    pub fn set_all_arbitrary(&mut self, _val: u32) -> Result<()> {
        todo!()
    }

    /// Set all pixels to black or white.
    ///
    /// Handles all depths including colormapped images.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetBlackOrWhite()` in `pix2.c`
    pub fn set_black_or_white(&mut self, _color: InitColor) {
        todo!()
    }

    /// Get the pixel value for black or white.
    ///
    /// For non-colormapped images, returns 0 for black and max for white
    /// (with depth=1 reversed: 0=white, 1=black).
    ///
    /// For colormapped images, returns the colormap index.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetBlackOrWhiteVal()` in `pix2.c`
    pub fn get_black_or_white_val(_pix: &Pix, _color: InitColor) -> u32 {
        todo!()
    }

    /// Clear a single pixel (set to 0).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClearPixel()` in `pix2.c`
    pub fn clear_pixel(&mut self, _x: u32, _y: u32) -> Result<()> {
        todo!()
    }

    /// Flip a single pixel (toggle between 0 and max value).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixFlipPixel()` in `pix2.c`
    pub fn flip_pixel(&mut self, _x: u32, _y: u32) -> Result<()> {
        todo!()
    }

    /// Clear pixels in a rectangular region (set to 0).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClearInRect()` in `pix2.c`
    pub fn clear_in_rect(&mut self, _rect: &crate::Box) -> Result<()> {
        todo!()
    }

    /// Set pixels in a rectangular region to max value.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetInRect()` in `pix2.c`
    pub fn set_in_rect(&mut self, _rect: &crate::Box) -> Result<()> {
        todo!()
    }

    /// Set pixels in a rectangular region to an arbitrary value.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetInRectArbitrary()` in `pix2.c`
    pub fn set_in_rect_arbitrary(&mut self, _rect: &crate::Box, _val: u32) -> Result<()> {
        todo!()
    }

    /// Set padding bits at the end of each scanline.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetPadBits()` in `pix2.c`
    pub fn set_pad_bits(&mut self, _val: u32) {
        todo!()
    }

    /// Set padding bits in a horizontal band.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetPadBitsBand()` in `pix2.c`
    pub fn set_pad_bits_band(&mut self, _by: u32, _bh: u32, _val: u32) {
        todo!()
    }

    /// Set or clear border pixels using raster operations.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetOrClearBorder()` in `pix2.c`
    pub fn set_or_clear_border(
        &mut self,
        _left: u32,
        _right: u32,
        _top: u32,
        _bot: u32,
        _color: InitColor,
    ) {
        todo!()
    }

    /// Clear all pixels to zero.
    pub fn clear(&mut self) {
        self.inner.data.fill(0);
    }

    /// Set all pixels to one (all bits set).
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

        assert_eq!(pix1.ref_count(), 2);
        assert_eq!(pix2.ref_count(), 2);
        assert_eq!(pix1.data().as_ptr(), pix2.data().as_ptr());
    }

    #[test]
    fn test_pix_deep_clone() {
        let pix1 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let pix2 = pix1.deep_clone();

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

    // ================================================================
    // Phase 11.1: Pix creation / template tests
    // ================================================================

    #[test]
    fn test_create_template() {
        let src = Pix::new(100, 200, PixelDepth::Bit8).unwrap();
        let mut src_mut = src.try_into_mut().unwrap();
        src_mut.set_xres(300);
        src_mut.set_yres(150);
        src_mut.set_informat(ImageFormat::Png);
        src_mut.set_text(Some("hello".to_string()));
        // Set a pixel so we can verify template is zeroed
        src_mut.set_pixel(50, 100, 42).unwrap();
        let src: Pix = src_mut.into();

        let tmpl = src.create_template();

        // Same dimensions and metadata
        assert_eq!(tmpl.width(), 100);
        assert_eq!(tmpl.height(), 200);
        assert_eq!(tmpl.depth(), PixelDepth::Bit8);
        assert_eq!(tmpl.spp(), 1);
        assert_eq!(tmpl.xres(), 300);
        assert_eq!(tmpl.yres(), 150);
        assert_eq!(tmpl.informat(), ImageFormat::Png);
        assert_eq!(tmpl.text(), Some("hello"));
        // Data should be zeroed
        assert_eq!(tmpl.get_pixel(50, 100), Some(0));
        assert!(tmpl.data().iter().all(|&w| w == 0));
    }

    #[test]
    fn test_create_template_with_colormap() {
        let mut cmap = crate::PixColormap::new(8).unwrap();
        cmap.add_rgba(255, 0, 0, 255).unwrap();
        cmap.add_rgba(0, 255, 0, 255).unwrap();

        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_colormap(Some(cmap)).unwrap();
        let src: Pix = pix_mut.into();

        let tmpl = src.create_template();

        // Colormap should be copied
        assert!(tmpl.has_colormap());
        let cm = tmpl.colormap().unwrap();
        assert_eq!(cm.len(), 2);
        assert_eq!(cm.get_rgba(0), Some((255, 0, 0, 255)));
    }

    #[test]
    fn test_new_with_colormap_black() {
        let pix = Pix::new_with_colormap(100, 100, PixelDepth::Bit8, InitColor::Black).unwrap();

        assert_eq!(pix.width(), 100);
        assert_eq!(pix.height(), 100);
        assert_eq!(pix.depth(), PixelDepth::Bit8);
        assert!(pix.has_colormap());

        let cm = pix.colormap().unwrap();
        assert_eq!(cm.len(), 1);
        assert_eq!(cm.get_rgba(0), Some((0, 0, 0, 255)));
    }

    #[test]
    fn test_new_with_colormap_white() {
        let pix = Pix::new_with_colormap(100, 100, PixelDepth::Bit4, InitColor::White).unwrap();

        assert!(pix.has_colormap());
        let cm = pix.colormap().unwrap();
        assert_eq!(cm.get_rgba(0), Some((255, 255, 255, 255)));
    }

    #[test]
    fn test_new_with_colormap_invalid_depth() {
        // Following C Leptonica, new_with_colormap only supports depths 2, 4, and 8.
        assert!(Pix::new_with_colormap(100, 100, PixelDepth::Bit1, InitColor::Black,).is_err());
        assert!(Pix::new_with_colormap(100, 100, PixelDepth::Bit32, InitColor::Black,).is_err());
    }

    #[test]
    fn test_sizes_equal() {
        let pix1 = Pix::new(100, 200, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(100, 200, PixelDepth::Bit8).unwrap();
        let pix3 = Pix::new(100, 200, PixelDepth::Bit32).unwrap();
        let pix4 = Pix::new(50, 200, PixelDepth::Bit8).unwrap();

        assert!(pix1.sizes_equal(&pix2));
        assert!(!pix1.sizes_equal(&pix3)); // different depth
        assert!(!pix1.sizes_equal(&pix4)); // different width
    }

    #[test]
    fn test_max_aspect_ratio() {
        let pix1 = Pix::new(100, 200, PixelDepth::Bit8).unwrap();
        assert!((pix1.max_aspect_ratio() - 2.0).abs() < 0.001);

        let pix2 = Pix::new(200, 100, PixelDepth::Bit8).unwrap();
        assert!((pix2.max_aspect_ratio() - 2.0).abs() < 0.001);

        let pix3 = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        assert!((pix3.max_aspect_ratio() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_copy_colormap_from() {
        let mut cmap = crate::PixColormap::new(8).unwrap();
        cmap.add_rgb(10, 20, 30).unwrap();

        let src = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut src_mut = src.try_into_mut().unwrap();
        src_mut.set_colormap(Some(cmap)).unwrap();
        let src: Pix = src_mut.into();

        let dst = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut dst_mut = dst.try_into_mut().unwrap();
        assert!(!dst_mut.has_colormap());

        dst_mut.copy_colormap_from(&src);
        assert!(dst_mut.has_colormap());
        let cm = dst_mut.colormap().unwrap();
        assert_eq!(cm.get_rgb(0), Some((10, 20, 30)));
    }

    #[test]
    fn test_copy_colormap_from_none() {
        // Copying from a pix with no colormap should remove existing colormap
        let mut cmap = crate::PixColormap::new(8).unwrap();
        cmap.add_rgb(10, 20, 30).unwrap();

        let dst = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut dst_mut = dst.try_into_mut().unwrap();
        dst_mut.set_colormap(Some(cmap)).unwrap();
        assert!(dst_mut.has_colormap());

        let src_no_cmap = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        dst_mut.copy_colormap_from(&src_no_cmap);
        assert!(!dst_mut.has_colormap());
    }

    #[test]
    fn test_copy_resolution_from() {
        let src = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut src_mut = src.try_into_mut().unwrap();
        src_mut.set_resolution(300, 600);
        let src: Pix = src_mut.into();

        let dst = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut dst_mut = dst.try_into_mut().unwrap();
        dst_mut.copy_resolution_from(&src);

        assert_eq!(dst_mut.xres(), 300);
        assert_eq!(dst_mut.yres(), 600);
    }

    #[test]
    fn test_scale_resolution() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_resolution(300, 600);
        pix_mut.scale_resolution(0.5, 2.0);

        assert_eq!(pix_mut.xres(), 150);
        assert_eq!(pix_mut.yres(), 1200);
    }

    #[test]
    fn test_copy_input_format_from() {
        let src = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut src_mut = src.try_into_mut().unwrap();
        src_mut.set_informat(ImageFormat::Tiff);
        let src: Pix = src_mut.into();

        let dst = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut dst_mut = dst.try_into_mut().unwrap();
        dst_mut.copy_input_format_from(&src);

        assert_eq!(dst_mut.informat(), ImageFormat::Tiff);
    }

    #[test]
    fn test_add_text() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Add to empty text
        pix_mut.add_text(Some("hello"));
        assert_eq!(pix_mut.text(), Some("hello"));

        // Append to existing text
        pix_mut.add_text(Some(" world"));
        assert_eq!(pix_mut.text(), Some("hello world"));

        // None is a no-op
        pix_mut.add_text(None);
        assert_eq!(pix_mut.text(), Some("hello world"));
    }

    #[test]
    fn test_copy_text_from() {
        let src = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut src_mut = src.try_into_mut().unwrap();
        src_mut.set_text(Some("source text".to_string()));
        let src: Pix = src_mut.into();

        let dst = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut dst_mut = dst.try_into_mut().unwrap();
        dst_mut.copy_text_from(&src);

        assert_eq!(dst_mut.text(), Some("source text"));
    }

    #[test]
    fn test_print_info() {
        let pix = Pix::new(100, 200, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_resolution(300, 300);
        pix_mut.set_text(Some("test image".to_string()));
        let pix: Pix = pix_mut.into();

        let mut buf = Vec::new();
        pix.print_info(&mut buf, Some("myimage")).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("myimage"));
        assert!(output.contains("100"));
        assert!(output.contains("200"));
        assert!(output.contains("300"));
        assert!(output.contains("test image"));
    }

    // ================================================================
    // Phase 11.2: Pixel setters / border / rect tests
    // ================================================================

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_all_gray_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_all_gray(128).unwrap();
        assert_eq!(pm.get_pixel(0, 0), Some(128));
        assert_eq!(pm.get_pixel(9, 9), Some(128));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_all_gray_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_all_gray(200).unwrap();
        // 32 bpp: R=G=B=200, alpha=0 → 0xC8C8C800
        let expected = (200u32 << 24) | (200u32 << 16) | (200u32 << 8);
        assert_eq!(pm.get_pixel(5, 5), Some(expected));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_all_arbitrary_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_all_arbitrary(42).unwrap();
        assert_eq!(pm.get_pixel(5, 5), Some(42));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_all_arbitrary_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_all_arbitrary(1).unwrap();
        assert_eq!(pm.get_pixel(0, 0), Some(1));
        assert_eq!(pm.get_pixel(9, 9), Some(1));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_black_or_white_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_black_or_white(InitColor::White);
        assert_eq!(pm.get_pixel(5, 5), Some(255));

        pm.set_black_or_white(InitColor::Black);
        assert_eq!(pm.get_pixel(5, 5), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_black_or_white_1bpp() {
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // 1 bpp: white=0, black=1
        pm.set_black_or_white(InitColor::Black);
        assert_eq!(pm.get_pixel(0, 0), Some(1));

        pm.set_black_or_white(InitColor::White);
        assert_eq!(pm.get_pixel(0, 0), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_black_or_white_val() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert_eq!(PixMut::get_black_or_white_val(&pix, InitColor::Black), 0);
        assert_eq!(PixMut::get_black_or_white_val(&pix, InitColor::White), 255);

        // 1 bpp: inverted
        let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert_eq!(PixMut::get_black_or_white_val(&pix1, InitColor::Black), 1);
        assert_eq!(PixMut::get_black_or_white_val(&pix1, InitColor::White), 0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_clear_pixel() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel(5, 5, 200).unwrap();
        assert_eq!(pm.get_pixel(5, 5), Some(200));

        pm.clear_pixel(5, 5).unwrap();
        assert_eq!(pm.get_pixel(5, 5), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_flip_pixel() {
        // 1 bpp flip
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        assert_eq!(pm.get_pixel(0, 0), Some(0));
        pm.flip_pixel(0, 0).unwrap();
        assert_eq!(pm.get_pixel(0, 0), Some(1));
        pm.flip_pixel(0, 0).unwrap();
        assert_eq!(pm.get_pixel(0, 0), Some(0));

        // 8 bpp flip
        let pix8 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm8 = pix8.try_into_mut().unwrap();
        pm8.set_pixel(5, 5, 100).unwrap();
        pm8.flip_pixel(5, 5).unwrap();
        assert_eq!(pm8.get_pixel(5, 5), Some(100 ^ 255));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_clear_in_rect() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Fill all with 200
        pm.set_all_arbitrary(200).unwrap();

        let rect = crate::Box::new(5, 5, 10, 10).unwrap();
        pm.clear_in_rect(&rect).unwrap();

        // Inside rect should be 0
        assert_eq!(pm.get_pixel(10, 10), Some(0));
        // Outside rect should still be 200
        assert_eq!(pm.get_pixel(0, 0), Some(200));
        assert_eq!(pm.get_pixel(19, 19), Some(200));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_in_rect() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        let rect = crate::Box::new(5, 5, 10, 10).unwrap();
        pm.set_in_rect(&rect).unwrap();

        // Inside rect should be 255 (max for 8bpp)
        assert_eq!(pm.get_pixel(10, 10), Some(255));
        // Outside rect should still be 0
        assert_eq!(pm.get_pixel(0, 0), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_in_rect_arbitrary() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        let rect = crate::Box::new(5, 5, 10, 10).unwrap();
        pm.set_in_rect_arbitrary(&rect, 42).unwrap();

        // Inside rect should be 42
        assert_eq!(pm.get_pixel(10, 10), Some(42));
        // Outside rect should still be 0
        assert_eq!(pm.get_pixel(0, 0), Some(0));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_pad_bits() {
        // 1 bpp, width=10, wpl=1 (32 bits per row, 22 pad bits)
        let pix = Pix::new(10, 2, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_all(); // set all bits to 1
        pm.set_pad_bits(0); // clear pad bits

        // First 10 bits of each row should be 1, rest should be 0
        let row0 = pm.data()[0];
        // Top 10 bits set: 0b1111_1111_1100_0000_0000_0000_0000_0000 = 0xFFC00000
        assert_eq!(row0, 0xFFC00000);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_pad_bits_band() {
        let pix = Pix::new(10, 4, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_all();
        pm.set_pad_bits_band(1, 2, 0); // clear pad bits for rows 1-2

        // Row 0 should still have all bits set
        assert_eq!(pm.data()[0], 0xFFFFFFFF);
        // Rows 1-2 should have pad bits cleared
        assert_eq!(pm.data()[1], 0xFFC00000);
        assert_eq!(pm.data()[2], 0xFFC00000);
        // Row 3 should still have all bits set
        assert_eq!(pm.data()[3], 0xFFFFFFFF);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_or_clear_border() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        pm.set_or_clear_border(2, 2, 2, 2, InitColor::White);
        // Border pixels should be 255
        assert_eq!(pm.get_pixel(0, 0), Some(255));
        assert_eq!(pm.get_pixel(1, 0), Some(255));
        // Interior should remain 0
        assert_eq!(pm.get_pixel(10, 10), Some(0));

        pm.set_or_clear_border(2, 2, 2, 2, InitColor::Black);
        // Border pixels should be 0 again
        assert_eq!(pm.get_pixel(0, 0), Some(0));
    }
}
