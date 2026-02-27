//! Bitmap Font (BMF) and text rendering operations
//!
//! Provides bitmap fonts for rendering text onto images.
//! Each font is a collection of 1 bpp character glyphs stored as a [`Pixa`].
//!
//! # Text rendering functions
//!
//! - [`Bmf::set_textline`] — render a single line of text
//! - [`Bmf::set_textblock`] — render multi-line text block
//! - [`Bmf::add_textlines`] — add text above/below/left/right of an image
//! - [`Bmf::get_line_strings`] — break text into lines fitting a width
//!
//! # See also
//!
//! C Leptonica: `bmf.c`, `textops.c`

use crate::core::error::{Error, Result};
use crate::core::pix::{Pix, PixelDepth};
use crate::core::pixa::Pixa;

// ────────────────────────────────────────────────────────────────────
//  Font glyph data
// ────────────────────────────────────────────────────────────────────

/// A minimal 5×7 bitmap font definition for ASCII 32–126.
///
/// Each character is represented as a slice of bytes, one per row (MSB-left).
/// Width is stored separately so proportional widths are possible.
mod font_data {
    /// (width, rows) for each ASCII code 32..=126.
    /// Row bytes are MSB-left: bit 7 = leftmost pixel.
    pub(super) fn glyph(ch: u8) -> (u8, [u8; 7]) {
        match ch {
            // space
            32 => (3, [0, 0, 0, 0, 0, 0, 0]),
            // !
            33 => (1, [0x80, 0x80, 0x80, 0x80, 0x80, 0x00, 0x80]),
            // "
            34 => (3, [0xA0, 0xA0, 0, 0, 0, 0, 0]),
            // #
            35 => (5, [0x50, 0xF8, 0x50, 0x50, 0xF8, 0x50, 0]),
            // $
            36 => (5, [0x20, 0x78, 0xA0, 0x70, 0x28, 0xF0, 0x20]),
            // %
            37 => (5, [0xC8, 0xC8, 0x10, 0x20, 0x40, 0x98, 0x98]),
            // &
            38 => (5, [0x40, 0xA0, 0xA0, 0x40, 0xA8, 0x90, 0x68]),
            // '
            39 => (1, [0x80, 0x80, 0, 0, 0, 0, 0]),
            // (
            40 => (2, [0x40, 0x80, 0x80, 0x80, 0x80, 0x80, 0x40]),
            // )
            41 => (2, [0x80, 0x40, 0x40, 0x40, 0x40, 0x40, 0x80]),
            // *
            42 => (5, [0, 0x20, 0xA8, 0x70, 0xA8, 0x20, 0]),
            // +
            43 => (5, [0, 0x20, 0x20, 0xF8, 0x20, 0x20, 0]),
            // ,
            44 => (2, [0, 0, 0, 0, 0, 0x40, 0x80]),
            // -
            45 => (4, [0, 0, 0, 0xF0, 0, 0, 0]),
            // .
            46 => (1, [0, 0, 0, 0, 0, 0, 0x80]),
            // /
            47 => (3, [0x20, 0x20, 0x40, 0x40, 0x40, 0x80, 0x80]),
            // 0
            48 => (4, [0x60, 0x90, 0x90, 0x90, 0x90, 0x90, 0x60]),
            // 1
            49 => (3, [0x20, 0x60, 0x20, 0x20, 0x20, 0x20, 0x70]),
            // 2
            50 => (4, [0x60, 0x90, 0x10, 0x20, 0x40, 0x80, 0xF0]),
            // 3
            51 => (4, [0x60, 0x90, 0x10, 0x60, 0x10, 0x90, 0x60]),
            // 4
            52 => (4, [0x10, 0x30, 0x50, 0x90, 0xF0, 0x10, 0x10]),
            // 5
            53 => (4, [0xF0, 0x80, 0xE0, 0x10, 0x10, 0x90, 0x60]),
            // 6
            54 => (4, [0x60, 0x80, 0xE0, 0x90, 0x90, 0x90, 0x60]),
            // 7
            55 => (4, [0xF0, 0x10, 0x20, 0x20, 0x40, 0x40, 0x40]),
            // 8
            56 => (4, [0x60, 0x90, 0x90, 0x60, 0x90, 0x90, 0x60]),
            // 9
            57 => (4, [0x60, 0x90, 0x90, 0x70, 0x10, 0x10, 0x60]),
            // :
            58 => (1, [0, 0, 0x80, 0, 0, 0x80, 0]),
            // ;
            59 => (2, [0, 0, 0x40, 0, 0, 0x40, 0x80]),
            // <
            60 => (3, [0, 0x20, 0x40, 0x80, 0x40, 0x20, 0]),
            // =
            61 => (4, [0, 0, 0xF0, 0, 0xF0, 0, 0]),
            // >
            62 => (3, [0, 0x80, 0x40, 0x20, 0x40, 0x80, 0]),
            // ?
            63 => (4, [0x60, 0x90, 0x10, 0x20, 0x20, 0, 0x20]),
            // @
            64 => (5, [0x70, 0x88, 0xB8, 0xA8, 0xB8, 0x80, 0x70]),
            // A
            65 => (4, [0x60, 0x90, 0x90, 0xF0, 0x90, 0x90, 0x90]),
            // B
            66 => (4, [0xE0, 0x90, 0x90, 0xE0, 0x90, 0x90, 0xE0]),
            // C
            67 => (4, [0x60, 0x90, 0x80, 0x80, 0x80, 0x90, 0x60]),
            // D
            68 => (4, [0xE0, 0x90, 0x90, 0x90, 0x90, 0x90, 0xE0]),
            // E
            69 => (4, [0xF0, 0x80, 0x80, 0xE0, 0x80, 0x80, 0xF0]),
            // F
            70 => (4, [0xF0, 0x80, 0x80, 0xE0, 0x80, 0x80, 0x80]),
            // G
            71 => (4, [0x60, 0x90, 0x80, 0xB0, 0x90, 0x90, 0x60]),
            // H
            72 => (4, [0x90, 0x90, 0x90, 0xF0, 0x90, 0x90, 0x90]),
            // I
            73 => (3, [0xE0, 0x40, 0x40, 0x40, 0x40, 0x40, 0xE0]),
            // J
            74 => (4, [0x70, 0x10, 0x10, 0x10, 0x10, 0x90, 0x60]),
            // K
            75 => (4, [0x90, 0xA0, 0xC0, 0xC0, 0xA0, 0x90, 0x90]),
            // L
            76 => (4, [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xF0]),
            // M
            77 => (5, [0x88, 0xD8, 0xA8, 0x88, 0x88, 0x88, 0x88]),
            // N
            78 => (4, [0x90, 0xD0, 0xD0, 0xB0, 0xB0, 0x90, 0x90]),
            // O
            79 => (4, [0x60, 0x90, 0x90, 0x90, 0x90, 0x90, 0x60]),
            // P
            80 => (4, [0xE0, 0x90, 0x90, 0xE0, 0x80, 0x80, 0x80]),
            // Q
            81 => (4, [0x60, 0x90, 0x90, 0x90, 0x90, 0xA0, 0x50]),
            // R
            82 => (4, [0xE0, 0x90, 0x90, 0xE0, 0xA0, 0x90, 0x90]),
            // S
            83 => (4, [0x60, 0x90, 0x80, 0x60, 0x10, 0x90, 0x60]),
            // T
            84 => (5, [0xF8, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20]),
            // U
            85 => (4, [0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x60]),
            // V
            86 => (5, [0x88, 0x88, 0x88, 0x50, 0x50, 0x20, 0x20]),
            // W
            87 => (5, [0x88, 0x88, 0x88, 0xA8, 0xA8, 0xD8, 0x88]),
            // X
            88 => (4, [0x90, 0x90, 0x60, 0x60, 0x90, 0x90, 0x90]),
            // Y
            89 => (5, [0x88, 0x88, 0x50, 0x20, 0x20, 0x20, 0x20]),
            // Z
            90 => (4, [0xF0, 0x10, 0x20, 0x40, 0x80, 0x80, 0xF0]),
            // [
            91 => (2, [0xC0, 0x80, 0x80, 0x80, 0x80, 0x80, 0xC0]),
            // backslash
            92 => (3, [0x80, 0x80, 0x40, 0x40, 0x40, 0x20, 0x20]),
            // ]
            93 => (2, [0xC0, 0x40, 0x40, 0x40, 0x40, 0x40, 0xC0]),
            // ^
            94 => (3, [0x40, 0xA0, 0, 0, 0, 0, 0]),
            // _
            95 => (4, [0, 0, 0, 0, 0, 0, 0xF0]),
            // `
            96 => (2, [0x80, 0x40, 0, 0, 0, 0, 0]),
            // a
            97 => (4, [0, 0, 0x60, 0x10, 0x70, 0x90, 0x70]),
            // b
            98 => (4, [0x80, 0x80, 0xE0, 0x90, 0x90, 0x90, 0xE0]),
            // c
            99 => (3, [0, 0, 0x60, 0x80, 0x80, 0x80, 0x60]),
            // d
            100 => (4, [0x10, 0x10, 0x70, 0x90, 0x90, 0x90, 0x70]),
            // e
            101 => (4, [0, 0, 0x60, 0x90, 0xF0, 0x80, 0x60]),
            // f
            102 => (3, [0x20, 0x40, 0xE0, 0x40, 0x40, 0x40, 0x40]),
            // g
            103 => (4, [0, 0, 0x70, 0x90, 0x90, 0x70, 0x10]),
            // h
            104 => (4, [0x80, 0x80, 0xE0, 0x90, 0x90, 0x90, 0x90]),
            // i
            105 => (1, [0x80, 0, 0x80, 0x80, 0x80, 0x80, 0x80]),
            // j
            106 => (2, [0x40, 0, 0x40, 0x40, 0x40, 0x40, 0x80]),
            // k
            107 => (4, [0x80, 0x80, 0x90, 0xA0, 0xC0, 0xA0, 0x90]),
            // l
            108 => (1, [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80]),
            // m
            109 => (5, [0, 0, 0xD0, 0xA8, 0xA8, 0x88, 0x88]),
            // n
            110 => (4, [0, 0, 0xE0, 0x90, 0x90, 0x90, 0x90]),
            // o
            111 => (4, [0, 0, 0x60, 0x90, 0x90, 0x90, 0x60]),
            // p
            112 => (4, [0, 0, 0xE0, 0x90, 0x90, 0xE0, 0x80]),
            // q
            113 => (4, [0, 0, 0x70, 0x90, 0x90, 0x70, 0x10]),
            // r
            114 => (3, [0, 0, 0xA0, 0xC0, 0x80, 0x80, 0x80]),
            // s
            115 => (3, [0, 0, 0x60, 0x80, 0x40, 0x20, 0xC0]),
            // t
            116 => (3, [0x40, 0x40, 0xE0, 0x40, 0x40, 0x40, 0x20]),
            // u
            117 => (4, [0, 0, 0x90, 0x90, 0x90, 0x90, 0x70]),
            // v
            118 => (3, [0, 0, 0xA0, 0xA0, 0xA0, 0x40, 0x40]),
            // w
            119 => (5, [0, 0, 0x88, 0x88, 0xA8, 0xA8, 0x50]),
            // x
            120 => (3, [0, 0, 0xA0, 0x40, 0x40, 0x40, 0xA0]),
            // y
            121 => (4, [0, 0, 0x90, 0x90, 0x90, 0x70, 0x60]),
            // z
            122 => (3, [0, 0, 0xE0, 0x20, 0x40, 0x80, 0xE0]),
            // {
            123 => (3, [0x20, 0x40, 0x40, 0x80, 0x40, 0x40, 0x20]),
            // |
            124 => (1, [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80]),
            // }
            125 => (3, [0x80, 0x40, 0x40, 0x20, 0x40, 0x40, 0x80]),
            // ~
            126 => (4, [0, 0, 0x50, 0xA0, 0, 0, 0]),
            _ => (3, [0, 0, 0, 0, 0, 0, 0]),
        }
    }

    /// Baseline position (distance from top of glyph to baseline) in the
    /// base 7-row font.  Most characters sit at row 6; descenders go below.
    pub(super) fn baseline() -> u32 {
        5 // row index 5 in the 7-row grid (0-based)
    }
}

// ────────────────────────────────────────────────────────────────────
//  Bmf struct
// ────────────────────────────────────────────────────────────────────

/// Location for adding text to an image.
///
/// # See also
///
/// C Leptonica: `L_ADD_ABOVE`, `L_ADD_BELOW`, `L_ADD_LEFT`, `L_ADD_RIGHT`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextLocation {
    /// Add text above the image
    Above,
    /// Add text below the image
    Below,
    /// Add text to the left of the image
    Left,
    /// Add text to the right of the image
    Right,
}

/// Bitmap font for text rendering.
///
/// Contains pre-rendered 1 bpp character glyphs for ASCII 32–126,
/// scaled to the requested point size.
///
/// # See also
///
/// C Leptonica: `L_Bmf` struct in `bmf.c`
#[derive(Debug, Clone)]
pub struct Bmf {
    /// Font Pixa — one Pix per ASCII character (index = ch − 32)
    pixa: Pixa,
    /// Point size
    size: u32,
    /// Width of each glyph (index = ch − 32)
    widths: Vec<u32>,
    /// Baseline position (from top) for each glyph
    baselines: Vec<u32>,
    /// Inter-character spacing in pixels
    kern_width: u32,
    /// Width of the space character
    space_width: u32,
    /// Line height (maximum character height)
    line_height: u32,
    /// Vertical separation between lines
    vert_line_sep: u32,
}

const FIRST_CHAR: u8 = 32;
const LAST_CHAR: u8 = 126;
const NUM_CHARS: usize = (LAST_CHAR - FIRST_CHAR + 1) as usize; // 95

impl Bmf {
    /// Create a bitmap font at the given point size.
    ///
    /// Valid sizes: 4, 6, 8, 10, 12, 14, 16, 20.  Other sizes are
    /// clamped to the nearest valid size.
    ///
    /// # See also
    ///
    /// C Leptonica: `bmfCreate()` in `bmf.c`
    pub fn new(pointsize: u32) -> Result<Self> {
        let pointsize = Self::clamp_size(pointsize);
        if pointsize == 0 {
            return Err(Error::InvalidParameter("pointsize must be > 0".into()));
        }

        // Base font is 5-wide × 7-tall.  Scale factor = pointsize / 7
        // (at least 1).
        let scale = (pointsize as f64 / 7.0).max(1.0);

        let base_height = 7u32;
        let scaled_height = (base_height as f64 * scale).round() as u32;
        let base_baseline = font_data::baseline();
        let scaled_baseline = (base_baseline as f64 * scale).round() as u32;

        let mut pixa = Pixa::with_capacity(NUM_CHARS);
        let mut widths = Vec::with_capacity(NUM_CHARS);
        let mut baselines = Vec::with_capacity(NUM_CHARS);

        for ch in FIRST_CHAR..=LAST_CHAR {
            let (base_w, rows) = font_data::glyph(ch);
            let base_w = base_w as u32;
            let scaled_w = ((base_w as f64) * scale).round().max(1.0) as u32;

            // Create 1bpp glyph
            let pix = Pix::new(scaled_w, scaled_height, PixelDepth::Bit1)?;
            let mut pm = pix.try_into_mut().unwrap();

            for (src_y, &row_byte) in rows.iter().enumerate() {
                for src_x in 0..base_w {
                    let bit = (row_byte >> (7 - src_x)) & 1;
                    if bit != 0 {
                        // Scale up: fill the rectangle
                        let dx_start = (src_x as f64 * scale).round() as u32;
                        let dx_end =
                            (((src_x + 1) as f64) * scale).round().min(scaled_w as f64) as u32;
                        let dy_start = (src_y as f64 * scale).round() as u32;
                        let dy_end = (((src_y + 1) as f64) * scale)
                            .round()
                            .min(scaled_height as f64) as u32;
                        for dy in dy_start..dy_end {
                            for dx in dx_start..dx_end {
                                pm.set_pixel_unchecked(dx, dy, 1);
                            }
                        }
                    }
                }
            }

            let pix: Pix = pm.into();
            pixa.push(pix);
            widths.push(scaled_w);
            baselines.push(scaled_baseline);
        }

        // Kern width ≈ 8% of 'x' width (minimum 1)
        let x_idx = (b'x' - FIRST_CHAR) as usize;
        let kern_width = (widths[x_idx] as f64 * 0.08).round().max(1.0) as u32;

        let space_width = widths[(b' ' - FIRST_CHAR) as usize];
        let vert_line_sep = (scaled_height as f64 * 0.3).round().max(1.0) as u32;

        Ok(Bmf {
            pixa,
            size: pointsize,
            widths,
            baselines,
            kern_width,
            space_width,
            line_height: scaled_height,
            vert_line_sep,
        })
    }

    /// Clamp to the nearest supported font size.
    fn clamp_size(ps: u32) -> u32 {
        const SIZES: [u32; 8] = [4, 6, 8, 10, 12, 14, 16, 20];
        *SIZES
            .iter()
            .min_by_key(|&&s| (s as i32 - ps as i32).unsigned_abs())
            .unwrap_or(&10)
    }

    /// Return the character index (0-based) for an ASCII character.
    fn char_index(ch: char) -> Option<usize> {
        let code = ch as u32;
        if (FIRST_CHAR as u32..=LAST_CHAR as u32).contains(&code) {
            Some((code - FIRST_CHAR as u32) as usize)
        } else {
            None
        }
    }

    /// Get the 1 bpp glyph Pix for a character.
    ///
    /// Returns `None` for characters outside ASCII 32–126 or newlines.
    ///
    /// # See also
    ///
    /// C Leptonica: `bmfGetPix()` in `bmf.c`
    pub fn get_pix(&self, ch: char) -> Option<Pix> {
        if ch == '\n' {
            return None;
        }
        let idx = Self::char_index(ch)?;
        self.pixa.get_cloned(idx)
    }

    /// Get the pixel width of a character glyph.
    ///
    /// # See also
    ///
    /// C Leptonica: `bmfGetWidth()` in `bmf.c`
    pub fn get_width(&self, ch: char) -> Option<u32> {
        let idx = Self::char_index(ch)?;
        self.widths.get(idx).copied()
    }

    /// Get the baseline position (distance from top of glyph to baseline).
    ///
    /// # See also
    ///
    /// C Leptonica: `bmfGetBaseline()` in `bmf.c`
    pub fn get_baseline(&self, ch: char) -> Option<u32> {
        let idx = Self::char_index(ch)?;
        self.baselines.get(idx).copied()
    }

    /// Get a reference to the underlying font [`Pixa`].
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaGetFont()` in `bmf.c`
    pub fn get_font_pixa(&self) -> &Pixa {
        &self.pixa
    }

    /// Get the font point size.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Get the line height in pixels.
    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    /// Get the vertical line separation.
    pub fn vert_line_sep(&self) -> u32 {
        self.vert_line_sep
    }

    /// Get the inter-character kern width.
    pub fn kern_width(&self) -> u32 {
        self.kern_width
    }
}

// ────────────────────────────────────────────────────────────────────
//  Text measurement functions
// ────────────────────────────────────────────────────────────────────

impl Bmf {
    /// Get the pixel width of a string.
    ///
    /// Width = Σ(char_width + kern_width) − kern_width.
    ///
    /// # See also
    ///
    /// C Leptonica: `bmfGetStringWidth()` in `textops.c`
    pub fn get_string_width(&self, text: &str) -> u32 {
        if text.is_empty() {
            return 0;
        }
        let mut w: u32 = 0;
        let mut count = 0u32;
        for ch in text.chars() {
            if let Some(cw) = self.get_width(ch) {
                w += cw + self.kern_width;
                count += 1;
            }
        }
        if count > 0 {
            w.saturating_sub(self.kern_width)
        } else {
            0
        }
    }

    /// Get the pixel width of each word in a text string.
    ///
    /// Words are separated by whitespace.
    ///
    /// # See also
    ///
    /// C Leptonica: `bmfGetWordWidths()` in `textops.c`
    pub fn get_word_widths(&self, text: &str) -> Vec<u32> {
        text.split_whitespace()
            .map(|word| self.get_string_width(word))
            .collect()
    }

    /// Break text into lines that fit within `max_w` pixels.
    ///
    /// Returns a vector of line strings and the total height needed.
    ///
    /// # Arguments
    ///
    /// * `text` — input text (may contain newlines)
    /// * `max_w` — maximum line width in pixels
    /// * `first_indent` — indentation of the first line, in multiples of
    ///   the 'x' character width
    ///
    /// # See also
    ///
    /// C Leptonica: `bmfGetLineStrings()` in `textops.c`
    pub fn get_line_strings(
        &self,
        text: &str,
        max_w: u32,
        first_indent: u32,
    ) -> (Vec<String>, u32) {
        let x_width = self.get_width('x').unwrap_or(self.size.max(1));
        let indent_px = first_indent * x_width;

        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return (vec![], 0);
        }

        let word_widths: Vec<u32> = words.iter().map(|w| self.get_string_width(w)).collect();

        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();
        let mut current_w: u32 = if lines.is_empty() { indent_px } else { 0 };

        for (i, word) in words.iter().enumerate() {
            let ww = word_widths[i];
            let needed = if current_line.is_empty() {
                ww
            } else {
                self.space_width + self.kern_width + ww
            };

            if !current_line.is_empty() && current_w + needed > max_w {
                lines.push(current_line);
                current_line = String::new();
                current_w = 0;
            }

            if !current_line.is_empty() {
                current_line.push(' ');
                current_w += self.space_width + self.kern_width;
            }
            current_line.push_str(word);
            current_w += ww;
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        let nlines = lines.len() as u32;
        let h = if nlines > 0 {
            nlines * self.line_height + (nlines - 1) * self.vert_line_sep
        } else {
            0
        };

        (lines, h)
    }
}

// ────────────────────────────────────────────────────────────────────
//  Text rendering functions
// ────────────────────────────────────────────────────────────────────

impl Bmf {
    /// Render a single line of text onto an image.
    ///
    /// Characters are rendered by painting the 1 bpp glyph mask at
    /// the specified position.
    ///
    /// # Arguments
    ///
    /// * `pix` — source image
    /// * `text` — text to render (single line, newlines ignored)
    /// * `x` — starting x position
    /// * `y` — baseline y position
    /// * `val` — pixel value to paint through the mask
    ///
    /// # Returns
    ///
    /// A new Pix with the text rendered, plus the rendered text width.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetTextline()` in `textops.c`
    pub fn set_textline(
        &self,
        pix: &Pix,
        text: &str,
        x: i32,
        y: i32,
        val: u32,
    ) -> Result<(Pix, u32)> {
        let mut pm = pix.to_mut();
        let mut xpos = x;

        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            if let Some(glyph) = self.get_pix(ch) {
                let baseline = self.get_baseline(ch).unwrap_or(0) as i32;
                let ypos = y - baseline;
                pm.paint_through_mask(&glyph, xpos, ypos, val)?;
                xpos += glyph.width() as i32 + self.kern_width as i32;
            }
        }

        let width = if xpos > x {
            (xpos - x) as u32 - self.kern_width
        } else {
            0
        };

        Ok((pm.into(), width))
    }

    /// Render a multi-line text block onto an image.
    ///
    /// # Arguments
    ///
    /// * `pix` — source image
    /// * `text` — text to render (will be line-wrapped)
    /// * `val` — pixel value
    /// * `x0` — starting x position
    /// * `y0` — starting y position (baseline of first line)
    /// * `wtext` — maximum text width in pixels
    /// * `first_indent` — first-line indentation in x-widths
    ///
    /// # Returns
    ///
    /// A new Pix with the text block rendered.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetTextblock()` in `textops.c`
    #[allow(clippy::too_many_arguments)]
    pub fn set_textblock(
        &self,
        pix: &Pix,
        text: &str,
        val: u32,
        x0: i32,
        y0: i32,
        wtext: u32,
        first_indent: u32,
    ) -> Result<Pix> {
        let (lines, _h) = self.get_line_strings(text, wtext, first_indent);
        if lines.is_empty() {
            return Ok(pix.deep_clone());
        }

        let x_width = self.get_width('x').unwrap_or(self.size.max(1));
        let indent_px = first_indent * x_width;

        let mut current = pix.deep_clone();
        let line_step = (self.line_height + self.vert_line_sep) as i32;

        for (i, line) in lines.iter().enumerate() {
            let x = if i == 0 { x0 + indent_px as i32 } else { x0 };
            let y = y0 + (i as i32) * line_step;
            let (rendered, _) = self.set_textline(&current, line, x, y, val)?;
            current = rendered;
        }

        Ok(current)
    }

    /// Add text lines above, below, left, or right of an image.
    ///
    /// Creates a new expanded image with the text in the specified location.
    ///
    /// # Arguments
    ///
    /// * `pix` — source image
    /// * `text` — text to add (if empty, uses the image's embedded text)
    /// * `val` — pixel value for text
    /// * `location` — where to place the text
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddTextlines()` in `textops.c`
    pub fn add_textlines(
        &self,
        pix: &Pix,
        text: &str,
        val: u32,
        location: TextLocation,
    ) -> Result<Pix> {
        let actual_text = if text.is_empty() {
            pix.text().unwrap_or_default().to_string()
        } else {
            text.to_string()
        };
        if actual_text.is_empty() {
            return Ok(pix.deep_clone());
        }

        let pw = pix.width();
        let ph = pix.height();
        let depth = pix.depth();

        match location {
            TextLocation::Above | TextLocation::Below => {
                // Break text into lines that fit the image width
                let (lines, text_h) = self.get_line_strings(&actual_text, pw, 0);
                if lines.is_empty() {
                    return Ok(pix.deep_clone());
                }
                let margin = self.vert_line_sep;
                let new_h = ph + text_h + margin;
                let dest = Pix::new(pw, new_h, depth)?;
                let mut dm = dest.try_into_mut().unwrap();

                // Fill with white for non-1bpp images
                if depth != PixelDepth::Bit1 {
                    let white = depth.max_value();
                    for y in 0..new_h {
                        for x in 0..pw {
                            dm.set_pixel_unchecked(x, y, white);
                        }
                    }
                }

                let (img_y, text_y) = match location {
                    TextLocation::Above => (text_h + margin, 0u32),
                    _ => (0, ph + margin),
                };

                // Copy original image
                for y in 0..ph {
                    for x in 0..pw {
                        let v = pix.get_pixel_unchecked(x, y);
                        dm.set_pixel_unchecked(x, y + img_y, v);
                    }
                }

                let result: Pix = dm.into();

                // Render text lines
                let baseline_y = text_y as i32 + self.baselines[0] as i32;
                let mut current = result;
                let line_step = (self.line_height + self.vert_line_sep) as i32;
                for (i, line) in lines.iter().enumerate() {
                    let y = baseline_y + (i as i32) * line_step;
                    let (rendered, _) = self.set_textline(&current, line, 0, y, val)?;
                    current = rendered;
                }
                Ok(current)
            }
            TextLocation::Left | TextLocation::Right => {
                // For left/right, render text vertically (one line)
                let text_w = self.get_string_width(&actual_text);
                let margin = self.kern_width * 2;
                let new_w = pw + text_w + margin;
                let new_h = ph.max(self.line_height);
                let dest = Pix::new(new_w, new_h, depth)?;
                let mut dm = dest.try_into_mut().unwrap();

                if depth != PixelDepth::Bit1 {
                    let white = depth.max_value();
                    for y in 0..new_h {
                        for x in 0..new_w {
                            dm.set_pixel_unchecked(x, y, white);
                        }
                    }
                }

                let (img_x, text_x) = match location {
                    TextLocation::Left => (text_w + margin, 0i32),
                    _ => (0, (pw + margin) as i32),
                };

                // Copy original image
                for y in 0..ph {
                    for x in 0..pw {
                        let v = pix.get_pixel_unchecked(x, y);
                        dm.set_pixel_unchecked(x + img_x, y, v);
                    }
                }

                let result: Pix = dm.into();
                let baseline_y = self.baselines[0] as i32;
                let (rendered, _) =
                    self.set_textline(&result, &actual_text, text_x, baseline_y, val)?;
                Ok(rendered)
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Pixa text operations
// ────────────────────────────────────────────────────────────────────

impl Bmf {
    /// Add a sequential index number to each Pix in a Pixa.
    ///
    /// Returns a new Pixa where each image has its index number
    /// rendered at the specified location.
    ///
    /// # Arguments
    ///
    /// * `pixa` — input Pixa
    /// * `numbers` — optional custom numbers; if `None`, uses 0..n
    /// * `val` — pixel value for text
    /// * `location` — where to place the number
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaAddTextNumber()` in `textops.c`
    pub fn pixa_add_text_number(
        &self,
        pixa: &Pixa,
        numbers: Option<&[i32]>,
        val: u32,
        location: TextLocation,
    ) -> Result<Pixa> {
        let n = pixa.len();
        let mut result = Pixa::with_capacity(n);

        for i in 0..n {
            let pix = pixa
                .get(i)
                .ok_or(Error::IndexOutOfBounds { index: i, len: n })?;
            let num = match numbers {
                Some(nums) => {
                    if i < nums.len() {
                        nums[i]
                    } else {
                        i as i32
                    }
                }
                None => i as i32,
            };
            let text = num.to_string();
            let labeled = self.add_textlines(pix, &text, val, location)?;
            result.push(labeled);
        }

        Ok(result)
    }

    /// Add text lines to each Pix in a Pixa.
    ///
    /// # Arguments
    ///
    /// * `pixa` — input Pixa
    /// * `texts` — text strings for each Pix; if `None`, uses each
    ///   image's embedded text
    /// * `val` — pixel value for text
    /// * `location` — where to place the text
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaAddTextlines()` in `textops.c`
    pub fn pixa_add_textlines(
        &self,
        pixa: &Pixa,
        texts: Option<&[String]>,
        val: u32,
        location: TextLocation,
    ) -> Result<Pixa> {
        let n = pixa.len();
        let mut result = Pixa::with_capacity(n);

        for i in 0..n {
            let pix = pixa
                .get(i)
                .ok_or(Error::IndexOutOfBounds { index: i, len: n })?;
            let text = match texts {
                Some(t) => {
                    if i < t.len() {
                        t[i].as_str()
                    } else {
                        ""
                    }
                }
                None => pix.text().unwrap_or(""),
            };
            let labeled = self.add_textlines(pix, text, val, location)?;
            result.push(labeled);
        }

        Ok(result)
    }

    /// Add a Pix with a text label to a Pixa.
    ///
    /// The image is optionally reduced (subsampled), converted to
    /// the target depth if needed, and the text label is rendered at
    /// the specified location.
    ///
    /// # Arguments
    ///
    /// * `pixa` — destination Pixa (mutated in place)
    /// * `pix` — image to add
    /// * `reduction` — subsampling factor (1 = no reduction)
    /// * `text` — text label (if empty, uses the image's embedded text)
    /// * `val` — pixel value for text
    /// * `location` — where to place the text
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaAddPixWithText()` in `textops.c`
    pub fn pixa_add_pix_with_text(
        &self,
        pixa: &mut Pixa,
        pix: &Pix,
        reduction: u32,
        text: &str,
        val: u32,
        location: TextLocation,
    ) -> Result<()> {
        let reduction = reduction.max(1);

        // Apply reduction if needed
        let reduced = if reduction > 1 {
            let new_w = (pix.width() / reduction).max(1);
            let new_h = (pix.height() / reduction).max(1);
            scale_simple(pix, new_w, new_h)?
        } else {
            pix.deep_clone()
        };

        let labeled = self.add_textlines(&reduced, text, val, location)?;
        pixa.push(labeled);
        Ok(())
    }
}

/// Simple nearest-neighbor scale for reduction.
fn scale_simple(pix: &Pix, new_w: u32, new_h: u32) -> Result<Pix> {
    let src_w = pix.width();
    let src_h = pix.height();
    let dest = Pix::new(new_w, new_h, pix.depth())?;
    let mut dm = dest.try_into_mut().unwrap();

    for dy in 0..new_h {
        let sy = (dy as u64 * src_h as u64 / new_h as u64) as u32;
        for dx in 0..new_w {
            let sx = (dx as u64 * src_w as u64 / new_w as u64) as u32;
            let v = pix.get_pixel_unchecked(sx.min(src_w - 1), sy.min(src_h - 1));
            dm.set_pixel_unchecked(dx, dy, v);
        }
    }

    Ok(dm.into())
}

// ────────────────────────────────────────────────────────────────────
//  Free-standing convenience functions (matching C API signatures)
// ────────────────────────────────────────────────────────────────────

/// Get the pixel width of a string using a bitmap font.
///
/// # See also
///
/// C Leptonica: `bmfGetStringWidth()` in `textops.c`
pub fn bmf_get_string_width(bmf: &Bmf, text: &str) -> u32 {
    bmf.get_string_width(text)
}

/// Get the pixel width of each word in a text string.
///
/// # See also
///
/// C Leptonica: `bmfGetWordWidths()` in `textops.c`
pub fn bmf_get_word_widths(bmf: &Bmf, text: &str) -> Vec<u32> {
    bmf.get_word_widths(text)
}

/// Break text into lines that fit within a maximum width.
///
/// # See also
///
/// C Leptonica: `bmfGetLineStrings()` in `textops.c`
pub fn bmf_get_line_strings(text: &str, max_w: u32, first_indent: u32, bmf: &Bmf) -> Vec<String> {
    let (lines, _) = bmf.get_line_strings(text, max_w, first_indent);
    lines
}
