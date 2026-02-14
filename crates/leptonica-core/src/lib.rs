// Stub: minimal type definitions for workspace compilation.
// These will be replaced with full implementations in later phases.

/// Pixel depth (bits per pixel)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelDepth {
    Bit1 = 1,
    Bit2 = 2,
    Bit4 = 4,
    Bit8 = 8,
    Bit16 = 16,
    Bit32 = 32,
}

/// Image file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFormat {
    #[default]
    Unknown,
    Bmp,
    Jpeg,
    Png,
    Tiff,
    Pnm,
    Gif,
    WebP,
}

impl ImageFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Bmp => "bmp",
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Tiff => "tif",
            Self::Pnm => "pnm",
            Self::Gif => "gif",
            Self::WebP => "webp",
            Self::Unknown => "dat",
        }
    }
}

/// The main image container (stub).
#[derive(Debug, Clone)]
pub struct Pix {
    width: u32,
    height: u32,
    depth: PixelDepth,
}

impl Pix {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn depth(&self) -> u32 {
        self.depth as u32
    }

    pub fn get_pixel(&self, _x: u32, _y: u32) -> u32 {
        0
    }
}
