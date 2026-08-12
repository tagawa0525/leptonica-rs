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
//  Font sheet data
// ────────────────────────────────────────────────────────────────────

/// The 9 bitmap-font sheets from C leptonica's `bmfdata.h`, decoded from
/// their base64 form into CCITT G4 TIFF bytes by
/// `scripts/extract_bmfdata.py`. Each sheet holds the 95 printable ASCII
/// characters in 3 rows; [`pixa_generate_font`] cuts it into glyphs at
/// runtime, exactly like C `pixaGenerateFontFromString()`.
fn font_sheet_tiff(fontsize: u32) -> Option<&'static [u8]> {
    Some(match fontsize {
        4 => include_bytes!("fonts/chars-4.tif"),
        6 => include_bytes!("fonts/chars-6.tif"),
        8 => include_bytes!("fonts/chars-8.tif"),
        10 => include_bytes!("fonts/chars-10.tif"),
        12 => include_bytes!("fonts/chars-12.tif"),
        14 => include_bytes!("fonts/chars-14.tif"),
        16 => include_bytes!("fonts/chars-16.tif"),
        18 => include_bytes!("fonts/chars-18.tif"),
        20 => include_bytes!("fonts/chars-20.tif"),
        _ => return None,
    })
}

/// Extra vertical space between text lines, as a fraction of line height.
///
/// C Leptonica: `VertFractSep` in `bmf.c`.
const VERT_FRACT_SEP: f32 = 0.3;

/// Cut a font sheet (95 printable ASCII chars in 3 rows) into the 95
/// glyph images, returning them with the baseline of each row.
///
/// C Leptonica: `pixaGenerateFont()` in `bmf.c`
fn pixa_generate_font(sheet: &Pix) -> Result<(Pixa, [u32; 3])> {
    use crate::region::ConnectivityType;

    // Locate the 3 rows of characters from the row pixel counts.
    let w = sheet.width();
    let na = sheet.count_by_row(None)?;
    let mut rows: Vec<(u32, u32)> = Vec::new(); // (top, height)
    let mut inrow = false;
    let mut top = 0u32;
    for i in 0..sheet.height() {
        let count = na.get(i as usize).unwrap_or(0.0);
        if !inrow && count > 0.0 {
            inrow = true;
            top = i;
        } else if inrow && count == 0.0 {
            inrow = false;
            rows.push((top, i - top));
        }
    }
    if rows.len() != 3 {
        return Err(Error::InvalidParameter(format!(
            "font sheet has {} rows of chars, expected 3",
            rows.len()
        )));
    }

    let mut pixa = Pixa::with_capacity(NUM_CHARS);
    let mut baselines = [0u32; 3];
    for (i, &(top, rowh)) in rows.iter().enumerate() {
        let pixr = sheet.clip_rectangle(0, top, w, rowh)?;
        baselines[i] = text_baseline(&pixr)?;

        // Close with a 1x35 brick so each character becomes one component,
        // then take the components in left-to-right order.
        let pixrc = crate::morph::close_safe_brick(&pixr, 1, 35)
            .map_err(|e| Error::InvalidParameter(format!("close_safe_brick: {e}")))?;
        let comps = crate::region::find_connected_components(&pixrc, ConnectivityType::EightWay)
            .map_err(|e| Error::InvalidParameter(format!("find_connected_components: {e}")))?;
        let mut boxes: Vec<crate::core::Box> = comps.into_iter().map(|c| c.bounds).collect();
        boxes.sort_by_key(|b| b.x);

        if i == 0 {
            // Consolidate the two components of '"' into one box.
            if boxes.len() < 3 {
                return Err(Error::InvalidParameter(
                    "font sheet row 0 has too few components".into(),
                ));
            }
            boxes[1].w = boxes[2].x + boxes[2].w - boxes[1].x;
            boxes.remove(2);
        }

        let h = pixr.height();
        for (j, b) in boxes.iter().enumerate() {
            if b.w <= 2 && b.h == 1 {
                // Skip 1x1 and 2x1 noise components.
                continue;
            }
            let pixc = pixr.clip_rectangle(b.x as u32, 0, b.w as u32, h - 1)?;
            if i == 0 && j == 0 {
                // Placeholder for the space; replaced below.
                pixa.push(pixc.clone());
            }
            if i == 2 && j == 0 {
                // Placeholder for the '\'; replaced below.
                pixa.push(pixc.clone());
            }
            pixa.push(pixc);
        }
    }

    if pixa.len() != NUM_CHARS {
        return Err(Error::InvalidParameter(format!(
            "font sheet produced {} chars, expected {}",
            pixa.len(),
            NUM_CHARS
        )));
    }

    // The space (index 0) has no ON pixels and is about twice as wide
    // as the '!' character whose copy currently sits there.
    let (bang_w, bang_h) = {
        let bang = pixa.get(0).expect("glyph 0");
        (bang.width(), bang.height())
    };
    pixa.replace(0, Pix::new(2 * bang_w, bang_h, PixelDepth::Bit1)?)?;

    // The '\' (index 60) is a left-right flip of the '/' (index 15).
    let backslash = crate::transform::flip_lr(pixa.get(15).expect("glyph 15"))
        .map_err(|e| Error::InvalidParameter(format!("flip_lr: {e}")))?;
    pixa.replace(60, backslash)?;

    Ok((pixa, baselines))
}

/// Locate the baseline of a row of text as the raster line above the
/// largest drop in the row pixel-count profile.
///
/// C Leptonica: `pixGetTextBaseline()` in `bmf.c`
fn text_baseline(pix: &Pix) -> Result<u32> {
    let na = pix.count_by_row(None)?;
    let h = pix.height() as usize;
    let mut diffmax = 0i32;
    let mut ymax = 0u32;
    for i in 1..h {
        let val1 = na.get(i - 1).unwrap_or(0.0) as i32;
        let val2 = na.get(i).unwrap_or(0.0) as i32;
        let diff = (val1 - val2).max(0);
        if diff > diffmax {
            diffmax = diff;
            ymax = (i - 1) as u32; // upper raster line
        }
    }
    Ok(ymax)
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
    /// Valid sizes: 4, 6, 8, ..., 20 (even sizes only, like C). The glyphs
    /// are cut at runtime from the font sheets embedded from C leptonica's
    /// `bmfdata.h`, so the result is bit-identical to `bmfCreate(NULL, size)`.
    ///
    /// Requires the `tiff-format` feature (on by default) to decode the
    /// embedded G4 TIFF sheets; without it this returns an error, matching
    /// C's behavior when built without libtiff.
    ///
    /// # See also
    ///
    /// C Leptonica: `bmfCreate()` in `bmf.c`
    pub fn new(pointsize: u32) -> Result<Self> {
        let sheet_tiff = font_sheet_tiff(pointsize).ok_or_else(|| {
            Error::InvalidParameter(format!(
                "fontsize must be one of 4, 6, ..., 20, got {pointsize}"
            ))
        })?;
        let sheet = crate::io::read_image_mem(sheet_tiff).map_err(|e| {
            Error::InvalidParameter(format!("cannot decode embedded font sheet: {e}"))
        })?;
        let (pixa, row_baselines) = pixa_generate_font(&sheet)?;

        // bmfMakeAsciiTables: per-char baselines. Chars 32-57 sit on row 0,
        // 58-91 on row 1, 93-126 on row 2; '\' (92) was cut from row 0.
        let mut widths = Vec::with_capacity(NUM_CHARS);
        let mut baselines = Vec::with_capacity(NUM_CHARS);
        for ch in FIRST_CHAR..=LAST_CHAR {
            let idx = (ch - FIRST_CHAR) as usize;
            let pix = pixa
                .get(idx)
                .ok_or_else(|| Error::InvalidParameter(format!("font pixa missing glyph {idx}")))?;
            widths.push(pix.width());
            baselines.push(match ch {
                32..=57 | 92 => row_baselines[0],
                58..=91 => row_baselines[1],
                _ => row_baselines[2],
            });
        }

        // Line height: from the highest ascender to the lowest descender,
        // taken as the max glyph height of ' ' (row 0), ':' (row 1) and
        // ']' (row 2).
        let line_height = b" :]"
            .iter()
            .map(|&ch| {
                pixa.get((ch - FIRST_CHAR) as usize)
                    .map(|p| p.height())
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0);

        // Kern width: 8% of the 'x' width, at least 1.
        let x_width = widths[(b'x' - FIRST_CHAR) as usize];
        let kern_width = ((0.08 * x_width as f32 + 0.5) as u32).max(1);

        let space_width = widths[(b' ' - FIRST_CHAR) as usize];
        let vert_line_sep = (VERT_FRACT_SEP * line_height as f32 + 0.5) as u32;

        Ok(Bmf {
            pixa,
            size: pointsize,
            widths,
            baselines,
            kern_width,
            space_width,
            line_height,
            vert_line_sep,
        })
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

/// Generate and save a bitmap-font `Pixa` for a given point size.
///
/// Writes `chars-{fontsize}.pa` into `outdir` via [`Pixa::write_to_file`].
///
/// `fontsize` must be one of `{4, 6, 8, ..., 20}` (even sizes, like C).
///
/// The C version supports an optional `indir` to extract a font from an
/// image file; this port only generates from the compiled-in font data
/// (matching the `indir == null` path of the C function).
///
/// # See also
///
/// C Leptonica: `pixaSaveFont()` in `bmf.c`.
pub fn pixa_save_font(outdir: impl AsRef<std::path::Path>, fontsize: u32) -> Result<()> {
    let bmf = Bmf::new(fontsize)?;
    let path = outdir.as_ref().join(format!("chars-{fontsize}.pa"));
    bmf.get_font_pixa().write_to_file(&path)
}

/// Placement for [`Bmf::add_single_textblock`].
///
/// Mirrors C `L_ADD_ABOVE` / `L_ADD_AT_TOP` / `L_ADD_AT_BOT` /
/// `L_ADD_BELOW`. Unlike [`TextLocation`], this enum supports rendering the
/// text *inside* the image as well as expanding the image to make room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextblockLocation {
    /// Expand the image upward and render the text in the new border.
    Above,
    /// Render the text inside the image, near the top.
    AtTop,
    /// Render the text inside the image, near the bottom.
    AtBot,
    /// Expand the image downward and render the text in the new border.
    Below,
}

impl Bmf {
    /// Paint a block of text over `pix` at the requested location.
    ///
    /// Returns `(rendered_pix, overflowed)`. `overflowed == true` indicates
    /// that one or more lines were too wide to fit within the available
    /// horizontal extent.
    ///
    /// If `text` is empty, returns `(pix.deep_clone(), false)` (the C
    /// version falls back to `pixGetText(pix)`; this Rust port mirrors
    /// that fallback via [`Pix::text`]).
    ///
    /// # `val` semantics
    ///
    /// `val` is the foreground colour. The accepted range depends on
    /// `pix.depth()`:
    ///
    /// - 1 bpp: `0` or `1`
    /// - 2 bpp: `0..=3`
    /// - 4 bpp: `0..=15`
    /// - 8 bpp: `0..=255`
    /// - 16 bpp: `0..=0xffff`
    /// - 32 bpp: a `0xRRGGBBAA` value (use
    ///   [`crate::core::pixel::compose_rgba`] for clarity)
    ///
    /// Out-of-range values are **clamped to a sensible mid-range substitute**
    /// (e.g. 8 bpp `> 255` becomes `128`, 32 bpp `< 256` becomes mid-grey
    /// `0x80808000`). This matches the C version of
    /// `pixAddSingleTextblock`. Note this differs from
    /// [`Bmf::set_textline`] / [`Bmf::add_textlines`], which delegate to
    /// `paint_through_mask` and therefore *wrap* (bitmask) out-of-range
    /// values rather than clamp. Callers mixing the two APIs should pass a
    /// `val` within the depth's range to get identical behaviour.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddSingleTextblock()` in `textops.c`.
    pub fn add_single_textblock(
        &self,
        pix: &Pix,
        text: &str,
        val: u32,
        location: TextblockLocation,
    ) -> Result<(Pix, bool)> {
        let actual_text = if text.is_empty() {
            pix.text().unwrap_or_default().to_string()
        } else {
            text.to_string()
        };
        if actual_text.is_empty() {
            return Ok((pix.deep_clone(), false));
        }

        let depth = pix.depth();
        // Clamp val to a sensible mid-range substitute when out of range
        // (matches C pixAddSingleTextblock). See the doc comment above for
        // the difference vs set_textline / add_textlines, which wrap.
        let val = match depth {
            PixelDepth::Bit1 if val > 1 => 1,
            PixelDepth::Bit2 if val > 3 => 2,
            PixelDepth::Bit4 if val > 15 => 8,
            PixelDepth::Bit8 if val > 0xff => 128,
            PixelDepth::Bit16 if val > 0xffff => 0x8000,
            PixelDepth::Bit32 if val < 256 => 0x80808000,
            _ => val,
        };

        let w = pix.width();
        let h = pix.height();
        let spacer = 10u32;
        let xstart = (w as f32 * 0.1) as u32;
        let max_text_width = w.saturating_sub(2 * xstart).max(1);
        let (lines, text_h) = self.get_line_strings(&actual_text, max_text_width, 0);
        if lines.is_empty() {
            return Ok((pix.deep_clone(), false));
        }

        let extra = text_h + 2 * spacer;
        let dest = match location {
            TextblockLocation::Above | TextblockLocation::Below => {
                let new_h = h + extra;
                let canvas = Pix::new(w, new_h, depth)?;
                let mut cm = canvas.try_into_mut().unwrap();
                if depth != PixelDepth::Bit1 {
                    let white = depth.max_value();
                    for y in 0..new_h {
                        for x in 0..w {
                            cm.set_pixel_unchecked(x, y, white);
                        }
                    }
                }
                let img_y = if location == TextblockLocation::Above {
                    extra
                } else {
                    0
                };
                for y in 0..h {
                    for x in 0..w {
                        let v = pix.get_pixel_unchecked(x, y);
                        cm.set_pixel_unchecked(x, y + img_y, v);
                    }
                }
                let canvas: Pix = cm.into();
                canvas
            }
            TextblockLocation::AtTop | TextblockLocation::AtBot => pix.deep_clone(),
        };

        // Baseline of 'I' approximates C's baselinetab[93].
        let baseline_y = self
            .get_baseline('I')
            .unwrap_or(self.line_height().saturating_sub(1));

        let ystart = match location {
            TextblockLocation::Above | TextblockLocation::AtTop => baseline_y + spacer,
            TextblockLocation::AtBot => h.saturating_sub(text_h + spacer) + baseline_y,
            TextblockLocation::Below => h + baseline_y + spacer,
        };

        let line_step = self.line_height() + self.vert_line_sep();
        let mut current = dest;
        let mut overflow = false;
        for (i, line) in lines.iter().enumerate() {
            let y = ystart + (i as u32) * line_step;
            let (rendered, line_w) =
                self.set_textline(&current, line, xstart as i32, y as i32, val)?;
            if line_w > max_text_width {
                overflow = true;
            }
            current = rendered;
        }

        Ok((current, overflow))
    }
}
