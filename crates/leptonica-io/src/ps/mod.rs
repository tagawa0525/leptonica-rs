//! PostScript output support
//!
//! Generates Encapsulated PostScript (EPS) and full-page PostScript
//! from `Pix` images.  Image data is Deflate-compressed and then
//! ASCII85-encoded for safe embedding in text-based PS files.
//!
//! # See also
//! C version: `pixWriteStringPS()`, `convertToPSEmbed()` in `psio1.c` / `psio2.c`

pub mod ascii85;

use crate::IoResult;
use leptonica_core::Pix;
use std::io::Write;

/// PostScript language level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsLevel {
    /// Level 1 (no filters)
    Level1,
    /// Level 2 (ASCII85 + Deflate)
    Level2,
    /// Level 3 (same as Level 2 for our purposes)
    Level3,
}

/// Options controlling PostScript output.
#[derive(Debug, Clone)]
pub struct PsOptions {
    /// Document title
    pub title: String,
    /// Output resolution in DPI
    pub resolution: u32,
    /// PS language level
    pub level: PsLevel,
    /// Scaling factor (1.0 = no scaling)
    pub scale: f32,
    /// Whether to emit a `%%BoundingBox` comment
    pub bounding_box: bool,
}

impl PsOptions {
    /// Create options suitable for EPS output.
    pub fn eps() -> Self {
        todo!()
    }

    /// Create options with a title.
    pub fn with_title(title: impl Into<String>) -> Self {
        todo!()
    }

    /// Set the output resolution.
    pub fn resolution(mut self, res: u32) -> Self {
        self.resolution = res;
        self
    }

    /// Set the PS language level.
    pub fn level(mut self, level: PsLevel) -> Self {
        self.level = level;
        self
    }

    /// Set the scale factor.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Enable or disable bounding-box emission.
    pub fn bounding_box(mut self, enable: bool) -> Self {
        self.bounding_box = enable;
        self
    }
}

/// Write a `Pix` as PostScript to a writer.
///
/// # Arguments
/// * `pix`     - The image to encode
/// * `writer`  - Destination writer
/// * `options` - PS output options
pub fn write_ps<W: Write>(pix: &Pix, writer: W, options: &PsOptions) -> IoResult<()> {
    todo!()
}

/// Write a `Pix` as PostScript, returning a byte vector.
pub fn write_ps_mem(pix: &Pix, options: &PsOptions) -> IoResult<Vec<u8>> {
    todo!()
}

/// Write a `Pix` as Encapsulated PostScript, returning a byte vector.
pub fn write_eps_mem(pix: &Pix, options: &PsOptions) -> IoResult<Vec<u8>> {
    todo!()
}

/// Compute the resolution needed to fit an image on a US Letter page.
///
/// # Arguments
/// * `width`         - Image width in pixels
/// * `height`        - Image height in pixels
/// * `fill_fraction` - Fraction of the page to fill (0.0 .. 1.0)
///
/// # Returns
/// Resolution in DPI.
pub fn get_res_letter_page(width: u32, height: u32, fill_fraction: f32) -> u32 {
    todo!()
}
