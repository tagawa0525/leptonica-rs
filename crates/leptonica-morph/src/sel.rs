//! Structuring Element (SEL) for morphological operations
//!
//! A structuring element defines the neighborhood used in morphological operations.

use crate::{MorphError, MorphResult};
use leptonica_core::{Pix, Pta};
use std::io::{BufRead, Write};
use std::path::Path;

/// Element type in a structuring element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[derive(Default)]
pub enum SelElement {
    /// Don't care - this position is ignored
    #[default]
    DontCare = 0,
    /// Hit - must match foreground (set pixels)
    Hit = 1,
    /// Miss - must match background (unset pixels)
    Miss = 2,
}

/// Structuring Element (SEL)
///
/// Defines the neighborhood pattern for morphological operations.
/// The origin (cx, cy) is the reference point for the operation.
#[derive(Debug, Clone)]
pub struct Sel {
    /// Width of the SEL
    width: u32,
    /// Height of the SEL
    height: u32,
    /// X coordinate of the origin
    cx: u32,
    /// Y coordinate of the origin
    cy: u32,
    /// Element data (row-major order)
    data: Vec<SelElement>,
    /// Optional name for identification
    name: Option<String>,
}

impl Sel {
    /// Create a new empty structuring element
    pub fn new(width: u32, height: u32) -> MorphResult<Self> {
        if width == 0 || height == 0 {
            return Err(MorphError::InvalidSel(
                "width and height must be > 0".to_string(),
            ));
        }

        let size = (width * height) as usize;
        Ok(Sel {
            width,
            height,
            cx: width / 2,
            cy: height / 2,
            data: vec![SelElement::DontCare; size],
            name: None,
        })
    }

    /// Create a rectangular "brick" structuring element with all hits
    ///
    /// # Arguments
    /// * `width` - Width of the brick
    /// * `height` - Height of the brick
    pub fn create_brick(width: u32, height: u32) -> MorphResult<Self> {
        if width == 0 || height == 0 {
            return Err(MorphError::InvalidSel(
                "width and height must be > 0".to_string(),
            ));
        }

        let size = (width * height) as usize;
        Ok(Sel {
            width,
            height,
            cx: width / 2,
            cy: height / 2,
            data: vec![SelElement::Hit; size],
            name: Some(format!("brick_{}x{}", width, height)),
        })
    }

    /// Create a square structuring element with all hits
    pub fn create_square(size: u32) -> MorphResult<Self> {
        Self::create_brick(size, size)
    }

    /// Create a horizontal line structuring element
    pub fn create_horizontal(length: u32) -> MorphResult<Self> {
        Self::create_brick(length, 1)
    }

    /// Create a vertical line structuring element
    pub fn create_vertical(length: u32) -> MorphResult<Self> {
        Self::create_brick(1, length)
    }

    /// Create a horizontal comb structuring element
    ///
    /// A comb SEL has `factor2` hits spaced `factor1` apart in a row
    /// of width `factor1 * factor2`. Used for composite decomposition:
    /// `dilate(brick(f1)) then dilate(comb(f1, f2))` == `dilate(brick(f1*f2))`.
    ///
    /// C version: `sel1.c:452-487` `selCreateComb`
    pub fn create_comb_horizontal(factor1: u32, factor2: u32) -> MorphResult<Self> {
        if factor1 == 0 || factor2 == 0 {
            return Err(MorphError::InvalidSel("factors must be > 0".to_string()));
        }
        let size = factor1 * factor2;
        let mut sel = Self::new(size, 1)?;
        for i in 0..factor2 {
            let z = factor1 / 2 + i * factor1;
            sel.set_element(z, 0, SelElement::Hit);
        }
        sel.name = Some(format!("comb_{}h", size));
        Ok(sel)
    }

    /// Create a vertical comb structuring element
    ///
    /// Same as horizontal comb but oriented vertically.
    pub fn create_comb_vertical(factor1: u32, factor2: u32) -> MorphResult<Self> {
        if factor1 == 0 || factor2 == 0 {
            return Err(MorphError::InvalidSel("factors must be > 0".to_string()));
        }
        let size = factor1 * factor2;
        let mut sel = Self::new(1, size)?;
        for i in 0..factor2 {
            let z = factor1 / 2 + i * factor1;
            sel.set_element(0, z, SelElement::Hit);
        }
        sel.name = Some(format!("comb_{}v", size));
        Ok(sel)
    }

    /// Create a cross (+) structuring element
    pub fn create_cross(size: u32) -> MorphResult<Self> {
        if size == 0 {
            return Err(MorphError::InvalidSel("size must be > 0".to_string()));
        }

        let mut sel = Self::new(size, size)?;
        let center = size / 2;

        // Horizontal line
        for x in 0..size {
            sel.set_element(x, center, SelElement::Hit);
        }

        // Vertical line
        for y in 0..size {
            sel.set_element(center, y, SelElement::Hit);
        }

        sel.name = Some(format!("cross_{}", size));
        Ok(sel)
    }

    /// Create a diamond structuring element
    pub fn create_diamond(radius: u32) -> MorphResult<Self> {
        if radius == 0 {
            return Err(MorphError::InvalidSel("radius must be > 0".to_string()));
        }

        let size = 2 * radius + 1;
        let mut sel = Self::new(size, size)?;
        let center = radius as i32;

        for y in 0..size {
            for x in 0..size {
                let dx = (x as i32 - center).abs();
                let dy = (y as i32 - center).abs();
                if dx + dy <= radius as i32 {
                    sel.set_element(x, y, SelElement::Hit);
                }
            }
        }

        sel.name = Some(format!("diamond_{}", radius));
        Ok(sel)
    }

    /// Create a disk (approximate circle) structuring element
    pub fn create_disk(radius: u32) -> MorphResult<Self> {
        if radius == 0 {
            return Err(MorphError::InvalidSel("radius must be > 0".to_string()));
        }

        let size = 2 * radius + 1;
        let mut sel = Self::new(size, size)?;
        let center = radius as f32;
        let r_sq = (radius as f32 + 0.5).powi(2);

        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                if dx * dx + dy * dy <= r_sq {
                    sel.set_element(x, y, SelElement::Hit);
                }
            }
        }

        sel.name = Some(format!("disk_{}", radius));
        Ok(sel)
    }

    /// Create a structuring element from a string pattern
    ///
    /// # Arguments
    /// * `pattern` - String with 'x' for hit, 'o' for miss, '.' for don't care
    /// * `origin_x` - X coordinate of origin
    /// * `origin_y` - Y coordinate of origin
    ///
    /// # Example
    /// ```
    /// use leptonica_morph::Sel;
    ///
    /// let sel = Sel::from_string(
    ///     "xxx\n\
    ///      xox\n\
    ///      xxx",
    ///     1, 1
    /// ).unwrap();
    /// ```
    pub fn from_string(pattern: &str, origin_x: u32, origin_y: u32) -> MorphResult<Self> {
        let lines: Vec<&str> = pattern.lines().collect();
        if lines.is_empty() {
            return Err(MorphError::InvalidSel("empty pattern".to_string()));
        }

        let height = lines.len() as u32;
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u32;

        if width == 0 {
            return Err(MorphError::InvalidSel("empty pattern".to_string()));
        }

        let mut sel = Self::new(width, height)?;

        // Validate that the origin lies within the SEL bounds
        if origin_x >= width || origin_y >= height {
            return Err(MorphError::InvalidSel(format!(
                "origin ({}, {}) out of bounds for {}x{} SEL",
                origin_x, origin_y, width, height
            )));
        }

        sel.cx = origin_x;
        sel.cy = origin_y;

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let elem = match ch {
                    'x' | 'X' | '1' => SelElement::Hit,
                    'o' | 'O' | '0' => SelElement::Miss,
                    '.' | ' ' | '-' => SelElement::DontCare,
                    _ => continue,
                };
                sel.set_element(x as u32, y as u32, elem);
            }
        }

        Ok(sel)
    }

    /// Get the width
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the origin x coordinate
    #[inline]
    pub fn origin_x(&self) -> u32 {
        self.cx
    }

    /// Get the origin y coordinate
    #[inline]
    pub fn origin_y(&self) -> u32 {
        self.cy
    }

    /// Set the origin
    pub fn set_origin(&mut self, cx: u32, cy: u32) -> MorphResult<()> {
        if cx >= self.width || cy >= self.height {
            return Err(MorphError::InvalidSel(
                "origin must be within SEL bounds".to_string(),
            ));
        }
        self.cx = cx;
        self.cy = cy;
        Ok(())
    }

    /// Get the name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Get an element at (x, y)
    #[inline]
    pub fn get_element(&self, x: u32, y: u32) -> Option<SelElement> {
        if x < self.width && y < self.height {
            Some(self.data[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Set an element at (x, y)
    #[inline]
    pub fn set_element(&mut self, x: u32, y: u32, elem: SelElement) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = elem;
        }
    }

    /// Get raw element data
    pub fn data(&self) -> &[SelElement] {
        &self.data
    }

    /// Count the number of hit elements
    pub fn hit_count(&self) -> usize {
        self.data.iter().filter(|&&e| e == SelElement::Hit).count()
    }

    /// Count the number of miss elements
    pub fn miss_count(&self) -> usize {
        self.data.iter().filter(|&&e| e == SelElement::Miss).count()
    }

    /// Create the reflected (180-degree rotated) SEL
    ///
    /// Used for correlation-based operations.
    pub fn reflect(&self) -> Self {
        let mut reflected = Sel {
            width: self.width,
            height: self.height,
            cx: self.width - 1 - self.cx,
            cy: self.height - 1 - self.cy,
            data: vec![SelElement::DontCare; self.data.len()],
            name: self.name.as_ref().map(|n| format!("{}_reflected", n)),
        };

        for y in 0..self.height {
            for x in 0..self.width {
                let src = self.data[(y * self.width + x) as usize];
                let dst_x = self.width - 1 - x;
                let dst_y = self.height - 1 - y;
                reflected.data[(dst_y * self.width + dst_x) as usize] = src;
            }
        }

        reflected
    }

    /// Rotate the SEL by 90 degrees orthogonally
    ///
    /// # Arguments
    /// * `rotation` - Number of 90-degree rotations (0-3)
    ///   - 0: no rotation
    ///   - 1: 90 degrees clockwise
    ///   - 2: 180 degrees
    ///   - 3: 270 degrees clockwise (90 degrees counter-clockwise)
    pub fn rotate_orth(&self, rotation: u32) -> Self {
        let rotation = rotation % 4;

        if rotation == 0 {
            return self.clone();
        }

        if rotation == 2 {
            return self.reflect();
        }

        // For 90 and 270 degree rotations, width and height swap
        let (new_width, new_height) = (self.height, self.width);

        let (new_cx, new_cy) = match rotation {
            1 => {
                // 90 degrees clockwise: (x, y) -> (height-1-y, x)
                // Origin: (cx, cy) -> (height-1-cy, cx)
                (self.height - 1 - self.cy, self.cx)
            }
            3 => {
                // 270 degrees clockwise: (x, y) -> (y, width-1-x)
                // Origin: (cx, cy) -> (cy, width-1-cx)
                (self.cy, self.width - 1 - self.cx)
            }
            _ => unreachable!(),
        };

        let mut rotated = Sel {
            width: new_width,
            height: new_height,
            cx: new_cx,
            cy: new_cy,
            data: vec![SelElement::DontCare; self.data.len()],
            name: self.name.as_ref().map(|n| format!("{}_rot{}", n, rotation)),
        };

        for y in 0..self.height {
            for x in 0..self.width {
                let src = self.data[(y * self.width + x) as usize];
                let (dst_x, dst_y) = match rotation {
                    1 => {
                        // 90 degrees clockwise
                        (self.height - 1 - y, x)
                    }
                    3 => {
                        // 270 degrees clockwise
                        (y, self.width - 1 - x)
                    }
                    _ => unreachable!(),
                };
                rotated.data[(dst_y * new_width + dst_x) as usize] = src;
            }
        }

        rotated
    }

    /// Iterate over hit positions relative to origin
    pub fn hit_offsets(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let cx = self.cx as i32;
        let cy = self.cy as i32;
        let width = self.width;

        self.data
            .iter()
            .enumerate()
            .filter_map(move |(idx, &elem)| {
                if elem == SelElement::Hit {
                    let x = (idx as u32 % width) as i32;
                    let y = (idx as u32 / width) as i32;
                    Some((x - cx, y - cy))
                } else {
                    None
                }
            })
    }

    /// Iterate over miss positions relative to origin
    pub fn miss_offsets(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let cx = self.cx as i32;
        let cy = self.cy as i32;
        let width = self.width;

        self.data
            .iter()
            .enumerate()
            .filter_map(move |(idx, &elem)| {
                if elem == SelElement::Miss {
                    let x = (idx as u32 % width) as i32;
                    let y = (idx as u32 / width) as i32;
                    Some((x - cx, y - cy))
                } else {
                    None
                }
            })
    }

    /// Find the maximum translations of hit elements relative to the origin.
    ///
    /// Returns `(xp, yp, xn, yn)` where each value is a non-negative
    /// translation *magnitude*:
    /// - `xp`: maximum positive x-translation (+x) required due to hits located
    ///   to the left of the origin (`cx - x`)
    /// - `yp`: maximum positive y-translation (+y) required due to hits located
    ///   above the origin (`cy - y`)
    /// - `xn`: maximum negative x-translation (−x) required due to hits located
    ///   to the right of the origin (`x - cx`)
    /// - `yn`: maximum negative y-translation (−y) required due to hits located
    ///   below the origin (`y - cy`)
    ///
    /// Used to determine the safe erosion/HMT application region.
    ///
    /// # See also
    ///
    /// C Leptonica: `selFindMaxTranslations()` in `sel1.c`
    pub fn find_max_translations(&self) -> (u32, u32, u32, u32) {
        let cx = self.cx as i32;
        let cy = self.cy as i32;
        let mut xp: i32 = 0;
        let mut yp: i32 = 0;
        let mut xn: i32 = 0;
        let mut yn: i32 = 0;

        for (idx, &elem) in self.data.iter().enumerate() {
            if elem == SelElement::Hit {
                let x = (idx as u32 % self.width) as i32;
                let y = (idx as u32 / self.width) as i32;
                xp = xp.max(cx - x);
                yp = yp.max(cy - y);
                xn = xn.max(x - cx);
                yn = yn.max(y - cy);
            }
        }

        (xp as u32, yp as u32, xn as u32, yn as u32)
    }

    /// Create a structuring element from a 1-bpp image.
    ///
    /// Each foreground pixel in the image becomes a Hit element.
    /// All other pixels become DontCare.
    ///
    /// # See also
    ///
    /// C Leptonica: `selCreateFromPix()` in `sel1.c`
    pub fn from_pix(pix: &Pix, cx: u32, cy: u32) -> MorphResult<Self> {
        if pix.depth() != leptonica_core::PixelDepth::Bit1 {
            return Err(MorphError::UnsupportedDepth {
                expected: "1 bpp",
                actual: pix.depth() as u32,
            });
        }

        let w = pix.width();
        let h = pix.height();
        let mut sel = Sel::new(w, h)?;
        sel.set_origin(cx, cy)?;

        for y in 0..h {
            for x in 0..w {
                if pix.get_pixel_unchecked(x, y) != 0 {
                    sel.set_element(x, y, SelElement::Hit);
                }
            }
        }

        Ok(sel)
    }

    /// Return SEL dimensions and origin as a tuple: (height, width, cy, cx).
    ///
    /// Based on C leptonica `selGetParameters`.
    pub fn get_parameters(&self) -> (u32, u32, u32, u32) {
        (self.height, self.width, self.cy, self.cx)
    }

    /// Return a human-readable string representation of the SEL.
    ///
    /// Each element is encoded as:
    /// - `x` / `X`: Hit (uppercase if at origin)
    /// - `o` / `O`: Miss (uppercase if at origin)
    /// - ` ` / `C`: DontCare (uppercase `C` if at origin)
    ///
    /// Based on C leptonica `selPrintToString`.
    pub fn print_to_string(&self) -> String {
        let mut s = String::with_capacity((self.height as usize) * (self.width as usize + 1));
        for y in 0..self.height {
            for x in 0..self.width {
                let is_center = x == self.cx && y == self.cy;
                let ch = match self.data[(y * self.width + x) as usize] {
                    SelElement::Hit => {
                        if is_center {
                            'X'
                        } else {
                            'x'
                        }
                    }
                    SelElement::Miss => {
                        if is_center {
                            'O'
                        } else {
                            'o'
                        }
                    }
                    SelElement::DontCare => {
                        if is_center {
                            'C'
                        } else {
                            ' '
                        }
                    }
                };
                s.push(ch);
            }
            s.push('\n');
        }
        s
    }

    /// Serialize the SEL to the Leptonica binary SEL file format (Version 1).
    ///
    /// Format:
    /// ```text
    ///   Sel Version 1
    ///   ------  <name>  ------
    ///   sy = <h>, sx = <w>, cy = <cy>, cx = <cx>
    ///     <row data: 0=dont_care, 1=hit, 2=miss>
    ///     ...
    /// ```
    ///
    /// Based on C leptonica `selWriteStream`.
    pub fn write_to_writer<W: Write>(&self, writer: &mut W) -> MorphResult<()> {
        let name = self.name.as_deref().unwrap_or("");
        let map_io = |e: std::io::Error| MorphError::InvalidParameters(e.to_string());
        writeln!(writer, "  Sel Version 1").map_err(map_io)?;
        writeln!(writer, "  ------  {}  ------", name).map_err(map_io)?;
        writeln!(
            writer,
            "  sy = {}, sx = {}, cy = {}, cx = {}",
            self.height, self.width, self.cy, self.cx
        )
        .map_err(map_io)?;
        for y in 0..self.height {
            write!(writer, "    ").map_err(map_io)?;
            for x in 0..self.width {
                let val = self.data[(y * self.width + x) as usize] as u8;
                write!(writer, "{}", val).map_err(map_io)?;
            }
            writeln!(writer).map_err(map_io)?;
        }
        writeln!(writer).map_err(map_io)?;
        Ok(())
    }

    /// Deserialize a SEL from the Leptonica binary SEL file format (Version 1).
    ///
    /// Based on C leptonica `selReadStream`.
    pub fn read_from_reader<R: BufRead>(mut reader: R) -> MorphResult<Self> {
        let map_io = |e: std::io::Error| MorphError::InvalidParameters(e.to_string());

        // Line 1: "  Sel Version 1"
        let mut line = String::new();
        reader.read_line(&mut line).map_err(map_io)?;
        let version: u32 = line
            .trim()
            .strip_prefix("Sel Version")
            .ok_or_else(|| MorphError::InvalidParameters("not a sel file".into()))?
            .trim()
            .parse()
            .map_err(|_| MorphError::InvalidParameters("invalid version number".into()))?;
        if version != 1 {
            return Err(MorphError::InvalidParameters(format!(
                "invalid sel version: {}",
                version
            )));
        }

        // Line 2: "  ------  <name>  ------"
        line.clear();
        reader.read_line(&mut line).map_err(map_io)?;
        let name = {
            let trimmed = line.trim();
            let inner = trimmed
                .strip_prefix("------")
                .ok_or_else(|| MorphError::InvalidParameters("bad sel name line".into()))?
                .trim();
            inner
                .strip_suffix("------")
                .ok_or_else(|| MorphError::InvalidParameters("bad sel name line".into()))?
                .trim()
                .to_string()
        };

        // Line 3: "  sy = <h>, sx = <w>, cy = <cy>, cx = <cx>"
        line.clear();
        reader.read_line(&mut line).map_err(map_io)?;
        let (sy, sx, cy, cx) = Self::parse_dimensions(line.trim())?;

        let mut sel = Sel::new(sx, sy)?;
        sel.set_origin(cx, cy)?;
        if !name.is_empty() {
            sel.set_name(&name);
        }

        // Read sy rows of pixel data
        for y in 0..sy {
            line.clear();
            reader.read_line(&mut line).map_err(map_io)?;
            let row_data = line.trim();
            if row_data.len() != sx as usize {
                return Err(MorphError::InvalidParameters(format!(
                    "row {} length {} != expected {}",
                    y,
                    row_data.len(),
                    sx
                )));
            }
            for (x, ch) in row_data.chars().enumerate() {
                let elem = match ch {
                    '0' => SelElement::DontCare,
                    '1' => SelElement::Hit,
                    '2' => SelElement::Miss,
                    other => {
                        return Err(MorphError::InvalidParameters(format!(
                            "invalid sel element: {}",
                            other
                        )));
                    }
                };
                sel.set_element(x as u32, y, elem);
            }
        }

        Ok(sel)
    }

    fn parse_dimensions(line: &str) -> MorphResult<(u32, u32, u32, u32)> {
        let parse_kv = |s: &str, key: &str| -> MorphResult<u32> {
            s.trim()
                .strip_prefix(key)
                .ok_or_else(|| MorphError::InvalidParameters(format!("expected key '{}'", key)))?
                .trim()
                .parse()
                .map_err(|_| MorphError::InvalidParameters("invalid dimension value".into()))
        };
        let parts: Vec<&str> = line.splitn(4, ',').collect();
        if parts.len() != 4 {
            return Err(MorphError::InvalidParameters("bad dimensions line".into()));
        }
        let sy = parse_kv(parts[0], "sy =")?;
        let sx = parse_kv(parts[1], "sx =")?;
        let cy = parse_kv(parts[2], "cy =")?;
        let cx = parse_kv(parts[3], "cx =")?;
        Ok((sy, sx, cy, cx))
    }

    /// Create a SEL from a 32-bpp color image.
    ///
    /// Pixel color encoding:
    /// - Pure green (R=0, G>0, B=0): Hit
    /// - Pure red (R>0, G=0, B=0): Miss
    /// - White (R>0, G>0, B>0): DontCare
    /// - Any non-white, non-primary pixel: error
    ///
    /// The origin is set to the first non-white pixel found.
    /// Based on C leptonica `selCreateFromColorPix`.
    pub fn from_color_image(pix: &Pix, name: Option<&str>) -> MorphResult<Self> {
        use leptonica_core::PixelDepth;
        if pix.depth() != PixelDepth::Bit32 {
            return Err(MorphError::UnsupportedDepth {
                expected: "32-bpp color",
                actual: pix.depth().bits(),
            });
        }

        let w = pix.width();
        let h = pix.height();
        let mut sel = Sel::new(w, h)?;
        sel.set_origin(w / 2, h / 2)?;
        if let Some(n) = name {
            sel.set_name(n);
        }

        let mut num_origins = 0u32;
        let mut has_hits = false;

        for y in 0..h {
            for x in 0..w {
                let pixel = pix.get_pixel_unchecked(x, y);
                let (r, g, b, _) = leptonica_core::color::extract_rgba(pixel);

                // Non-white pixel (white is exactly 255,255,255) = first one sets the origin
                if !(r == 255 && g == 255 && b == 255) {
                    num_origins += 1;
                    if num_origins == 1 {
                        sel.set_origin(x, y)?;
                    }
                }

                if r == 0 && g > 0 && b == 0 {
                    has_hits = true;
                    sel.set_element(x, y, SelElement::Hit);
                } else if r > 0 && g == 0 && b == 0 {
                    sel.set_element(x, y, SelElement::Miss);
                } else if r > 0 && g > 0 && b > 0 {
                    sel.set_element(x, y, SelElement::DontCare);
                } else {
                    return Err(MorphError::InvalidParameters(format!(
                        "invalid pixel color at ({}, {}): r={}, g={}, b={}",
                        x, y, r, g, b
                    )));
                }
            }
        }

        if !has_hits {
            return Err(MorphError::InvalidParameters(
                "no hits found in color image".into(),
            ));
        }

        Ok(sel)
    }

    /// Create a SEL from a point array (PTA).
    ///
    /// Each point in the PTA becomes a Hit element.
    /// The SEL is sized to enclose all points (using their bounding box).
    ///
    /// # Arguments
    /// * `pta` - point array; all points must have non-negative coordinates
    /// * `cy` - y coordinate of the origin
    /// * `cx` - x coordinate of the origin
    /// * `name` - optional name for the SEL
    ///
    /// Based on C leptonica `selCreateFromPta`.
    pub fn from_pta(pta: &Pta, cy: u32, cx: u32, name: Option<&str>) -> MorphResult<Self> {
        if pta.is_empty() {
            return Err(MorphError::InvalidParameters("PTA is empty".into()));
        }

        // First pass: determine SEL width/height from the integer coordinates
        // returned by get_i_pt (which rounds, not truncates), so size and element
        // placement are consistent.
        let mut max_x: i32 = 0;
        let mut max_y: i32 = 0;
        for i in 0..pta.len() {
            let (x, y) = pta
                .get_i_pt(i)
                .ok_or_else(|| MorphError::InvalidParameters("PTA index out of bounds".into()))?;
            if x < 0 || y < 0 {
                return Err(MorphError::InvalidParameters(
                    "PTA point coordinates must be non-negative".into(),
                ));
            }
            if x > max_x {
                max_x = x;
            }
            if y > max_y {
                max_y = y;
            }
        }

        let w = max_x as u32 + 1;
        let h = max_y as u32 + 1;
        let mut sel = Sel::new(w, h)?;
        sel.set_origin(cx, cy)?;
        if let Some(n) = name {
            sel.set_name(n);
        }

        // Second pass: set hit elements using the same integer coordinates.
        for i in 0..pta.len() {
            let (x, y) = pta
                .get_i_pt(i)
                .ok_or_else(|| MorphError::InvalidParameters("PTA index out of bounds".into()))?;
            sel.set_element(x as u32, y as u32, SelElement::Hit);
        }

        Ok(sel)
    }
}

// ── Phase 6: Sela – ordered collection of Sel instances ────────────────────

/// An ordered, named collection of [`Sel`] structuring elements.
///
/// `Sela` mirrors the C leptonica `SELA` type and supports:
/// - Building up a set incrementally with [`add`](Sela::add)
/// - Indexed and name-based lookup
/// - Serialization/deserialization using the Leptonica text format
#[derive(Debug, Clone, Default)]
pub struct Sela {
    sels: Vec<Sel>,
}

impl Sela {
    /// Create an empty Sela.
    pub fn new() -> Self {
        unimplemented!()
    }

    /// Return the number of SELs in the collection.
    ///
    /// Based on C leptonica `selaGetCount`.
    pub fn count(&self) -> usize {
        unimplemented!()
    }

    /// Add a [`Sel`] to the collection.
    ///
    /// The SEL must have a name (see [`Sel::set_name`]).
    ///
    /// Based on C leptonica `selaAddSel`.
    pub fn add(&mut self, sel: Sel) -> MorphResult<()> {
        unimplemented!()
    }

    /// Return a reference to the SEL at the given index.
    ///
    /// Returns `None` if `index >= self.count()`.
    ///
    /// Based on C leptonica `selaGetSel`.
    pub fn get(&self, index: usize) -> Option<&Sel> {
        unimplemented!()
    }

    /// Find a SEL by its name.
    ///
    /// Returns `None` if no SEL with the given name exists.
    ///
    /// Based on C leptonica `selaFindSelByName`.
    pub fn find_by_name(&self, name: &str) -> Option<&Sel> {
        unimplemented!()
    }

    /// Deserialize a `Sela` from the Leptonica text format.
    ///
    /// File format (produced by `selaWrite` / [`write`](Sela::write)):
    /// ```text
    /// \nSela Version 1\nNumber of Sels = N\n\n
    /// <SEL 0>
    /// ...
    /// <SEL N-1>
    /// ```
    ///
    /// Based on C leptonica `selaRead` / `selaReadStream`.
    pub fn read<P: AsRef<Path>>(path: P) -> MorphResult<Self> {
        unimplemented!()
    }

    /// Serialize this `Sela` to the Leptonica text format.
    ///
    /// Based on C leptonica `selaWrite` / `selaWriteStream`.
    pub fn write<P: AsRef<Path>>(&self, path: P) -> MorphResult<()> {
        unimplemented!()
    }
}

// ── Phase 4: SEL-set generation free functions ─────────────────────────────

const BASIC_LINEAR: &[u32] = &[
    2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 20, 21, 25, 30, 31, 35, 40, 41, 45, 50, 51,
];

/// Build the "basic" SEL library (horizontal/vertical bricks, square bricks,
/// diagonal SELs).
///
/// Returns the same set as C leptonica `selaAddBasic`.
pub fn sela_add_basic() -> Vec<Sel> {
    let mut sels = Vec::new();

    // Linear horizontal bricks: 1×N with origin at (N/2, 0)
    for &size in BASIC_LINEAR {
        if let Ok(mut sel) = Sel::create_brick(size, 1) {
            sel.set_origin(size / 2, 0)
                .expect("horizontal SEL origin is within bounds");
            sel.set_name(format!("sel_{}h", size));
            sels.push(sel);
        }
    }

    // Linear vertical bricks: N×1 with origin at (0, N/2)
    for &size in BASIC_LINEAR {
        if let Ok(mut sel) = Sel::create_brick(1, size) {
            sel.set_origin(0, size / 2)
                .expect("vertical SEL origin is within bounds");
            sel.set_name(format!("sel_{}v", size));
            sels.push(sel);
        }
    }

    // Square 2-D bricks 2×2 – 5×5
    for i in 2u32..=5 {
        if let Ok(mut sel) = Sel::create_brick(i, i) {
            sel.set_origin(i / 2, i / 2)
                .expect("square SEL origin is within bounds");
            sel.set_name(format!("sel_{}", i));
            sels.push(sel);
        }
    }

    // Diagonal sel_2dp: 2×2, hits at (0,1) and (1,0), origin (0,0)
    // Pattern: . x / x .  (DontCare on diagonal, Hit off-diagonal)
    if let Ok(mut sel) = Sel::create_brick(2, 2) {
        sel.set_origin(0, 0)
            .expect("sel_2dp origin is within bounds");
        sel.set_element(0, 0, SelElement::DontCare);
        sel.set_element(1, 1, SelElement::DontCare);
        sel.set_name("sel_2dp");
        sels.push(sel);
    }

    // Diagonal sel_2dm: 2×2, hits at (0,0) and (1,1), origin (0,0)
    // Pattern: x . / . x
    if let Ok(mut sel) = Sel::create_brick(2, 2) {
        sel.set_origin(0, 0)
            .expect("sel_2dm origin is within bounds");
        sel.set_element(0, 1, SelElement::DontCare);
        sel.set_element(1, 0, SelElement::DontCare);
        sel.set_name("sel_2dm");
        sels.push(sel);
    }

    // sel_5dp: 5×5, positive-slope diagonal (top-right to bottom-left)
    // Hit positions: (0,4),(1,3),(2,2),(3,1),(4,0) with origin (2,2)
    if let Ok(mut sel) = Sel::new(5, 5) {
        sel.set_origin(2, 2)
            .expect("diagonal SEL origin is within bounds");
        for i in 0u32..5 {
            sel.set_element(4 - i, i, SelElement::Hit);
        }
        sel.set_name("sel_5dp");
        sels.push(sel);
    }

    // sel_5dm: 5×5, negative-slope diagonal (top-left to bottom-right)
    // Hit positions: (0,0),(1,1),(2,2),(3,3),(4,4) with origin (2,2)
    if let Ok(mut sel) = Sel::new(5, 5) {
        sel.set_origin(2, 2)
            .expect("diagonal SEL origin is within bounds");
        for i in 0u32..5 {
            sel.set_element(i, i, SelElement::Hit);
        }
        sel.set_name("sel_5dm");
        sels.push(sel);
    }

    sels
}

/// Build the hit-miss SEL library (isolated pixel, edge, corner, slanted-edge).
///
/// Returns the same set as C leptonica `selaAddHitMiss`.
pub fn sela_add_hit_miss() -> Vec<Sel> {
    let mut sels = Vec::new();

    // sel_3hm: isolated foreground pixel (3×3 all miss, center hit)
    if let Ok(mut sel) = Sel::create_brick(3, 3) {
        for y in 0u32..3 {
            for x in 0u32..3 {
                sel.set_element(x, y, SelElement::Miss);
            }
        }
        sel.set_element(1, 1, SelElement::Hit);
        sel.set_name("sel_3hm");
        sels.push(sel);
    }

    // sel_3de: top edge detector (3×2, origin cx=1 cy=0)
    // C: selCreateBrick(sy=2, sx=3, cy=0, cx=1, SEL_HIT)
    // Row 0: all Hit; Row 1: all Miss
    if let Ok(mut sel) = Sel::create_brick(3, 2) {
        sel.set_origin(1, 0)
            .expect("sel_3de origin is within bounds");
        sel.set_element(0, 1, SelElement::Miss);
        sel.set_element(1, 1, SelElement::Miss);
        sel.set_element(2, 1, SelElement::Miss);
        sel.set_name("sel_3de");
        sels.push(sel);
    }

    // sel_3ue: bottom edge detector (3×2, origin cx=1 cy=1)
    // C: selCreateBrick(sy=2, sx=3, cy=1, cx=1, SEL_HIT)
    // Row 0: all Miss; Row 1: all Hit
    if let Ok(mut sel) = Sel::create_brick(3, 2) {
        sel.set_origin(1, 1)
            .expect("sel_3ue origin is within bounds");
        sel.set_element(0, 0, SelElement::Miss);
        sel.set_element(1, 0, SelElement::Miss);
        sel.set_element(2, 0, SelElement::Miss);
        sel.set_name("sel_3ue");
        sels.push(sel);
    }

    // sel_3re: right edge detector (2×3, origin cx=0 cy=1)
    // C: selCreateBrick(sy=3, sx=2, cy=1, cx=0, SEL_HIT)
    // Col 0: all Hit; Col 1: all Miss
    if let Ok(mut sel) = Sel::create_brick(2, 3) {
        sel.set_origin(0, 1)
            .expect("sel_3re origin is within bounds");
        sel.set_element(1, 0, SelElement::Miss);
        sel.set_element(1, 1, SelElement::Miss);
        sel.set_element(1, 2, SelElement::Miss);
        sel.set_name("sel_3re");
        sels.push(sel);
    }

    // sel_3le: left edge detector (2×3, origin cx=1 cy=1)
    // C: selCreateBrick(sy=3, sx=2, cy=1, cx=1, SEL_HIT)
    // Col 0: all Miss; Col 1: all Hit
    if let Ok(mut sel) = Sel::create_brick(2, 3) {
        sel.set_origin(1, 1)
            .expect("sel_3le origin is within bounds");
        sel.set_element(0, 0, SelElement::Miss);
        sel.set_element(0, 1, SelElement::Miss);
        sel.set_element(0, 2, SelElement::Miss);
        sel.set_name("sel_3le");
        sels.push(sel);
    }

    // sel_sl1: slanted edge (width=6, height=13, origin cx=2 cy=6)
    // C: selCreateBrick(sy=13, sx=6, cy=6, cx=2, SEL_DONT_CARE)
    if let Ok(mut sel) = Sel::new(6, 13) {
        sel.set_origin(2, 6)
            .expect("sel_sl1 origin is within bounds");
        sel.set_element(3, 0, SelElement::Miss);
        sel.set_element(5, 0, SelElement::Hit);
        sel.set_element(2, 4, SelElement::Miss);
        sel.set_element(4, 4, SelElement::Hit);
        sel.set_element(1, 8, SelElement::Miss);
        sel.set_element(3, 8, SelElement::Hit);
        sel.set_element(0, 12, SelElement::Miss);
        sel.set_element(2, 12, SelElement::Hit);
        sel.set_name("sel_sl1");
        sels.push(sel);
    }

    // Corner detectors: 4×4 with mix of Hit/Miss/DontCare

    // sel_ulc: upper-left corner
    if let Ok(mut sel) = Sel::create_brick(4, 4) {
        sel.set_origin(1, 1)
            .expect("sel_ulc origin is within bounds");
        for y in 0u32..4 {
            for x in 0u32..4 {
                sel.set_element(x, y, SelElement::Miss);
            }
        }
        sel.set_element(1, 1, SelElement::DontCare);
        sel.set_element(2, 1, SelElement::DontCare);
        sel.set_element(1, 2, SelElement::DontCare);
        sel.set_element(3, 1, SelElement::Hit);
        sel.set_element(2, 2, SelElement::Hit);
        sel.set_element(3, 2, SelElement::Hit);
        sel.set_element(1, 3, SelElement::Hit);
        sel.set_element(2, 3, SelElement::Hit);
        sel.set_element(3, 3, SelElement::Hit);
        sel.set_name("sel_ulc");
        sels.push(sel);
    }

    // sel_urc: upper-right corner
    // C: selCreateBrick(sy=4, sx=4, cy=1, cx=2, SEL_MISS)
    if let Ok(mut sel) = Sel::create_brick(4, 4) {
        sel.set_origin(2, 1)
            .expect("sel_urc origin is within bounds");
        for y in 0u32..4 {
            for x in 0u32..4 {
                sel.set_element(x, y, SelElement::Miss);
            }
        }
        sel.set_element(1, 1, SelElement::DontCare);
        sel.set_element(2, 1, SelElement::DontCare);
        sel.set_element(2, 2, SelElement::DontCare);
        sel.set_element(0, 1, SelElement::Hit);
        sel.set_element(0, 2, SelElement::Hit);
        sel.set_element(1, 2, SelElement::Hit);
        sel.set_element(0, 3, SelElement::Hit);
        sel.set_element(1, 3, SelElement::Hit);
        sel.set_element(2, 3, SelElement::Hit);
        sel.set_name("sel_urc");
        sels.push(sel);
    }

    // sel_llc: lower-left corner
    // C: selCreateBrick(sy=4, sx=4, cy=2, cx=1, SEL_MISS)
    // Hits at (row,col): (0,1)(0,2)(0,3)(1,2)(1,3)(2,3) [C notation]
    // → set_element(col, row, val) in Rust API
    if let Ok(mut sel) = Sel::create_brick(4, 4) {
        sel.set_origin(1, 2)
            .expect("sel_llc origin is within bounds");
        for y in 0u32..4 {
            for x in 0u32..4 {
                sel.set_element(x, y, SelElement::Miss);
            }
        }
        sel.set_element(1, 1, SelElement::DontCare);
        sel.set_element(1, 2, SelElement::DontCare);
        sel.set_element(2, 2, SelElement::DontCare);
        sel.set_element(1, 0, SelElement::Hit);
        sel.set_element(2, 0, SelElement::Hit);
        sel.set_element(3, 0, SelElement::Hit);
        sel.set_element(2, 1, SelElement::Hit);
        sel.set_element(3, 1, SelElement::Hit);
        sel.set_element(3, 2, SelElement::Hit);
        sel.set_name("sel_llc");
        sels.push(sel);
    }

    // sel_lrc: lower-right corner
    if let Ok(mut sel) = Sel::create_brick(4, 4) {
        sel.set_origin(2, 2)
            .expect("sel_lrc origin is within bounds");
        for y in 0u32..4 {
            for x in 0u32..4 {
                sel.set_element(x, y, SelElement::Miss);
            }
        }
        sel.set_element(2, 1, SelElement::DontCare);
        sel.set_element(1, 2, SelElement::DontCare);
        sel.set_element(2, 2, SelElement::DontCare);
        sel.set_element(0, 0, SelElement::Hit);
        sel.set_element(1, 0, SelElement::Hit);
        sel.set_element(2, 0, SelElement::Hit);
        sel.set_element(0, 1, SelElement::Hit);
        sel.set_element(1, 1, SelElement::Hit);
        sel.set_element(0, 2, SelElement::Hit);
        sel.set_name("sel_lrc");
        sels.push(sel);
    }

    sels
}

/// Build linear SELs for sizes 2–63 (horizontal and vertical).
///
/// Returns the same set as C leptonica `selaAddDwaLinear`.
pub fn sela_add_dwa_linear() -> Vec<Sel> {
    let mut sels = Vec::new();

    for i in 2u32..64 {
        if let Ok(mut sel) = Sel::create_brick(i, 1) {
            sel.set_origin(i / 2, 0)
                .expect("horizontal SEL origin is within bounds");
            sel.set_name(format!("sel_{}h", i));
            sels.push(sel);
        }
    }
    for i in 2u32..64 {
        if let Ok(mut sel) = Sel::create_brick(1, i) {
            sel.set_origin(0, i / 2)
                .expect("vertical SEL origin is within bounds");
            sel.set_name(format!("sel_{}v", i));
            sels.push(sel);
        }
    }

    sels
}

/// Build comb SELs for DWA composite operations (sizes 4–63).
///
/// For each composable size that can be factored as `f1 * f2`,
/// generates a horizontal comb and a vertical comb SEL.
///
/// Returns the same set as C leptonica `selaAddDwaCombs`.
pub fn sela_add_dwa_combs() -> Vec<Sel> {
    use crate::binary::select_composable_sizes;

    let mut sels = Vec::new();
    let mut prev_size = 0u32;

    for i in 4u32..64 {
        let (f1, f2) = select_composable_sizes(i);
        let size = f1 * f2;
        if size == prev_size {
            continue;
        }
        if let Ok(mut sel) = Sel::create_comb_horizontal(f1, f2) {
            sel.set_name(format!("sel_comb_{}h", size));
            sels.push(sel);
        }
        if let Ok(mut sel) = Sel::create_comb_vertical(f1, f2) {
            sel.set_name(format!("sel_comb_{}v", size));
            sels.push(sel);
        }
        prev_size = size;
    }

    sels
}

/// Generate pixel positions along a line from a center point.
///
/// Starting from `(xc, yc)`, extends `length` pixels in direction `radang` (radians).
fn line_points_from_pt(xc: i32, yc: i32, length: f64, radang: f64) -> Vec<(i32, i32)> {
    let n = length.ceil() as i32;
    (1..=n)
        .map(|k| {
            let x = xc + (k as f64 * radang.cos()).round() as i32;
            let y = yc + (k as f64 * radang.sin()).round() as i32;
            (x, y)
        })
        .collect()
}

/// Build hit-miss SELs for detecting cross (X) junctions of two lines.
///
/// # Arguments
/// * `hlsize` - Length of each hit arm from the origin
/// * `mdist`  - Distance of miss elements from the origin
/// * `norient` - Number of orientations (1–8); generates `norient` SELs
///
/// Based on C leptonica `selaAddCrossJunctions`.
pub fn sela_add_cross_junctions(hlsize: f32, mdist: f32, norient: u32) -> MorphResult<Vec<Sel>> {
    if hlsize <= 0.0 {
        return Err(MorphError::InvalidParameters(
            "hlsize must be > 0".to_string(),
        ));
    }
    if mdist <= 0.0 {
        return Err(MorphError::InvalidParameters(
            "mdist must be > 0".to_string(),
        ));
    }
    if norient == 0 || norient > 8 {
        return Err(MorphError::InvalidParameters(
            "norient must be in [1, 8]".to_string(),
        ));
    }

    let half_pi = std::f64::consts::FRAC_PI_2;
    let rad_incr = half_pi / norient as f64;
    let w_f = 2.2 * (hlsize.max(mdist) as f64 + 0.5);
    let w = if (w_f as u32).is_multiple_of(2) {
        w_f as u32 + 1
    } else {
        w_f as u32
    };
    let xc = (w / 2) as i32;
    let yc = (w / 2) as i32;

    let mut sels = Vec::with_capacity(norient as usize);
    for i in 0..norient {
        let mut sel = Sel::new(w, w)?;
        sel.set_origin(xc as u32, yc as u32)?;
        let rad = i as f64 * rad_incr;

        // Four arms of hits (cross shape)
        for &angle_off in &[0.0, half_pi, std::f64::consts::PI, 3.0 * half_pi] {
            for (px, py) in line_points_from_pt(xc, yc, (hlsize + 1.0) as f64, rad + angle_off) {
                if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < w {
                    sel.set_element(px as u32, py as u32, SelElement::Hit);
                }
            }
        }
        sel.set_element(xc as u32, yc as u32, SelElement::Hit); // origin itself

        // Four miss elements between arms
        for j in 0..4i32 {
            let angle = rad + (j as f64 - 0.5) * half_pi;
            let mx = xc + (mdist as f64 * angle.cos()).round() as i32;
            let my = yc + (mdist as f64 * angle.sin()).round() as i32;
            if mx >= 0 && my >= 0 && (mx as u32) < w && (my as u32) < w {
                // Only set to Miss if not already Hit
                if sel.get_element(mx as u32, my as u32) != Some(SelElement::Hit) {
                    sel.set_element(mx as u32, my as u32, SelElement::Miss);
                }
            }
        }

        sel.set_name(format!("sel_cross_{}", i));
        sels.push(sel);
    }

    Ok(sels)
}

/// Build hit-miss SELs for detecting T-junctions of two lines.
///
/// # Arguments
/// * `hlsize` - Length of each hit arm from the origin
/// * `mdist`  - Distance of miss elements from the origin
/// * `norient` - Number of orientations (1–8); generates `4 * norient` SELs
///
/// Based on C leptonica `selaAddTJunctions`.
pub fn sela_add_t_junctions(hlsize: f32, mdist: f32, norient: u32) -> MorphResult<Vec<Sel>> {
    if hlsize <= 2.0 {
        return Err(MorphError::InvalidParameters(
            "hlsize must be > 2".to_string(),
        ));
    }
    if mdist <= 0.0 {
        return Err(MorphError::InvalidParameters(
            "mdist must be > 0".to_string(),
        ));
    }
    if norient == 0 || norient > 8 {
        return Err(MorphError::InvalidParameters(
            "norient must be in [1, 8]".to_string(),
        ));
    }

    let half_pi = std::f64::consts::FRAC_PI_2;
    let rad_incr = half_pi / norient as f64;
    let w_f = 2.4 * (hlsize.max(mdist) as f64 + 0.5);
    let w = if (w_f as u32).is_multiple_of(2) {
        w_f as u32 + 1
    } else {
        w_f as u32
    };
    let xc = (w / 2) as i32;
    let yc = (w / 2) as i32;

    let mut sels = Vec::with_capacity(4 * norient as usize);
    for i in 0..norient {
        let rad = i as f64 * rad_incr;
        for j in 0..4u32 {
            let j_ang = j as f64 * half_pi;
            let mut sel = Sel::new(w, w)?;
            sel.set_origin(xc as u32, yc as u32)?;

            // Three arms of hits (T shape): straight + two half-perpendiculars
            for (arm_rad, len_mult) in [
                (j_ang + rad, hlsize as f64 + 1.0),
                (j_ang + rad + half_pi, hlsize as f64 / 2.0 + 1.0),
                (j_ang + rad - half_pi, hlsize as f64 / 2.0 + 1.0),
            ] {
                for (px, py) in line_points_from_pt(xc, yc, len_mult, arm_rad) {
                    if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < w {
                        sel.set_element(px as u32, py as u32, SelElement::Hit);
                    }
                }
            }
            sel.set_element(xc as u32, yc as u32, SelElement::Hit); // origin

            // Three miss elements (opposite to arms)
            for (k, &dist) in [mdist as f64, mdist as f64 / 2.0, mdist as f64 / 2.0]
                .iter()
                .enumerate()
            {
                let angle = j_ang + rad + std::f64::consts::PI + (k as f64 - 1.0) * half_pi;
                let mx = xc + (dist * angle.cos()).round() as i32;
                let my = yc + (dist * angle.sin()).round() as i32;
                if mx >= 0
                    && my >= 0
                    && (mx as u32) < w
                    && (my as u32) < w
                    && sel.get_element(mx as u32, my as u32) != Some(SelElement::Hit)
                {
                    sel.set_element(mx as u32, my as u32, SelElement::Miss);
                }
            }

            sel.set_name(format!("sel_t_{}_{}", i, j));
            sels.push(sel);
        }
    }

    Ok(sels)
}

/// Create a plus-sign (+) SEL of the given size and line width.
///
/// # Arguments
/// * `size`      - Side of the bounding square (must be >= 3)
/// * `linewidth` - Width of the horizontal/vertical arms
///
/// Based on C leptonica `selMakePlusSign`.
pub fn sel_make_plus_sign(size: u32, linewidth: u32) -> MorphResult<Sel> {
    if size < 3 {
        return Err(MorphError::InvalidParameters(
            "size must be >= 3".to_string(),
        ));
    }
    if linewidth == 0 || linewidth > size {
        return Err(MorphError::InvalidParameters(
            "linewidth must be between 1 and size".to_string(),
        ));
    }

    let mut sel = Sel::new(size, size)?;
    sel.set_origin(size / 2, size / 2)?;
    sel.set_name("plus_sign");

    let cx = size / 2;
    let cy = size / 2;
    let half = linewidth / 2;

    // Horizontal arm
    for x in 0..size {
        for dy in 0..linewidth {
            let y = cy.saturating_sub(half) + dy;
            if y < size {
                sel.set_element(x, y, SelElement::Hit);
            }
        }
    }
    // Vertical arm
    for y in 0..size {
        for dx in 0..linewidth {
            let x = cx.saturating_sub(half) + dx;
            if x < size {
                sel.set_element(x, y, SelElement::Hit);
            }
        }
    }

    Ok(sel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_brick() {
        let sel = Sel::create_brick(3, 3).unwrap();
        assert_eq!(sel.width(), 3);
        assert_eq!(sel.height(), 3);
        assert_eq!(sel.origin_x(), 1);
        assert_eq!(sel.origin_y(), 1);
        assert_eq!(sel.hit_count(), 9);
    }

    #[test]
    fn test_create_cross() {
        let sel = Sel::create_cross(3).unwrap();
        assert_eq!(sel.hit_count(), 5); // 3 horizontal + 3 vertical - 1 center
    }

    #[test]
    fn test_create_diamond() {
        let sel = Sel::create_diamond(1).unwrap();
        // Diamond with radius 1: 3x3 with 5 hits
        //  .x.
        //  xxx
        //  .x.
        assert_eq!(sel.hit_count(), 5);
    }

    #[test]
    fn test_from_string() {
        let sel = Sel::from_string(
            "xxx\n\
             xox\n\
             xxx",
            1,
            1,
        )
        .unwrap();

        assert_eq!(sel.width(), 3);
        assert_eq!(sel.height(), 3);
        assert_eq!(sel.hit_count(), 8);
        assert_eq!(sel.miss_count(), 1);
        assert_eq!(sel.get_element(1, 1), Some(SelElement::Miss));
    }

    #[test]
    fn test_reflect() {
        let sel = Sel::from_string("xx.\n...", 0, 0).unwrap();

        let reflected = sel.reflect();
        assert_eq!(reflected.get_element(0, 0), Some(SelElement::DontCare));
        assert_eq!(reflected.get_element(2, 1), Some(SelElement::Hit));
        assert_eq!(reflected.get_element(1, 1), Some(SelElement::Hit));
    }

    #[test]
    fn test_hit_offsets() {
        let sel = Sel::create_brick(3, 3).unwrap();
        let offsets: Vec<_> = sel.hit_offsets().collect();

        assert_eq!(offsets.len(), 9);
        assert!(offsets.contains(&(-1, -1)));
        assert!(offsets.contains(&(0, 0)));
        assert!(offsets.contains(&(1, 1)));
    }

    #[test]
    fn test_rotate_orth_identity() {
        let sel = Sel::from_string("xx.\n...", 0, 0).unwrap();
        let rotated = sel.rotate_orth(0);

        assert_eq!(rotated.width(), sel.width());
        assert_eq!(rotated.height(), sel.height());
        assert_eq!(rotated.origin_x(), sel.origin_x());
        assert_eq!(rotated.origin_y(), sel.origin_y());
    }

    #[test]
    fn test_rotate_orth_90() {
        // Original (3x2):
        // xx.   (y=0)
        // ...   (y=1)
        // Origin at (0, 0)
        let sel = Sel::from_string("xx.\n...", 0, 0).unwrap();
        let rotated = sel.rotate_orth(1);

        // After 90 degree rotation (2x3):
        // Width and height swap
        assert_eq!(rotated.width(), 2);
        assert_eq!(rotated.height(), 3);

        // Check the pattern:
        // 90 clockwise: (x, y) -> (height-1-y, x)
        // (0,0) -> (1, 0) - hit
        // (1,0) -> (1, 1) - hit
        // (2,0) -> (1, 2) - don't care
        // (0,1) -> (0, 0) - don't care
        // (1,1) -> (0, 1) - don't care
        // (2,1) -> (0, 2) - don't care
        assert_eq!(rotated.get_element(1, 0), Some(SelElement::Hit));
        assert_eq!(rotated.get_element(1, 1), Some(SelElement::Hit));
        assert_eq!(rotated.get_element(1, 2), Some(SelElement::DontCare));
        assert_eq!(rotated.get_element(0, 0), Some(SelElement::DontCare));
    }

    #[test]
    fn test_rotate_orth_180() {
        let sel = Sel::from_string("xx.\n...", 0, 0).unwrap();
        let rotated = sel.rotate_orth(2);

        // 180 degrees is same as reflect
        let reflected = sel.reflect();
        assert_eq!(rotated.width(), reflected.width());
        assert_eq!(rotated.height(), reflected.height());
        for y in 0..rotated.height() {
            for x in 0..rotated.width() {
                assert_eq!(rotated.get_element(x, y), reflected.get_element(x, y));
            }
        }
    }

    #[test]
    fn test_rotate_orth_360() {
        let sel = Sel::from_string("xx.\n...", 0, 0).unwrap();

        // Four rotations should return to original
        let rotated = sel.rotate_orth(4);
        assert_eq!(rotated.width(), sel.width());
        assert_eq!(rotated.height(), sel.height());
        for y in 0..sel.height() {
            for x in 0..sel.width() {
                assert_eq!(rotated.get_element(x, y), sel.get_element(x, y));
            }
        }
    }

    #[test]
    fn test_create_comb_horizontal_3x3() {
        // comb(f1=3, f2=3): size=9, hits at columns 1, 4, 7
        let sel = Sel::create_comb_horizontal(3, 3).unwrap();
        assert_eq!(sel.width(), 9);
        assert_eq!(sel.height(), 1);
        assert_eq!(sel.origin_x(), 4); // 9 / 2
        assert_eq!(sel.origin_y(), 0);
        assert_eq!(sel.hit_count(), 3);
        // Verify hit positions: f1/2 + i*f1 for i in 0..f2
        assert_eq!(sel.get_element(1, 0), Some(SelElement::Hit));
        assert_eq!(sel.get_element(4, 0), Some(SelElement::Hit));
        assert_eq!(sel.get_element(7, 0), Some(SelElement::Hit));
        // Non-hit positions should be DontCare
        assert_eq!(sel.get_element(0, 0), Some(SelElement::DontCare));
        assert_eq!(sel.get_element(2, 0), Some(SelElement::DontCare));
        assert_eq!(sel.get_element(3, 0), Some(SelElement::DontCare));
    }

    #[test]
    fn test_create_comb_horizontal_2x2() {
        // comb(f1=2, f2=2): size=4, hits at columns 1, 3
        let sel = Sel::create_comb_horizontal(2, 2).unwrap();
        assert_eq!(sel.width(), 4);
        assert_eq!(sel.height(), 1);
        assert_eq!(sel.hit_count(), 2);
        assert_eq!(sel.get_element(1, 0), Some(SelElement::Hit));
        assert_eq!(sel.get_element(3, 0), Some(SelElement::Hit));
    }

    #[test]
    fn test_create_comb_vertical_3x4() {
        // comb(f1=3, f2=4): size=12, hits at rows 1, 4, 7, 10
        let sel = Sel::create_comb_vertical(3, 4).unwrap();
        assert_eq!(sel.width(), 1);
        assert_eq!(sel.height(), 12);
        assert_eq!(sel.origin_x(), 0);
        assert_eq!(sel.origin_y(), 6); // 12 / 2
        assert_eq!(sel.hit_count(), 4);
        assert_eq!(sel.get_element(0, 1), Some(SelElement::Hit));
        assert_eq!(sel.get_element(0, 4), Some(SelElement::Hit));
        assert_eq!(sel.get_element(0, 7), Some(SelElement::Hit));
        assert_eq!(sel.get_element(0, 10), Some(SelElement::Hit));
    }

    // --- Phase 3: SEL I/O and extended creation ---

    fn make_hit_miss_sel() -> Sel {
        // 3x3: center=Hit, corners=Miss, edges=DontCare
        let mut sel = Sel::new(3, 3).unwrap();
        sel.set_origin(1, 1).unwrap();
        sel.set_element(1, 1, SelElement::Hit);
        sel.set_element(0, 0, SelElement::Miss);
        sel.set_element(2, 0, SelElement::Miss);
        sel.set_element(0, 2, SelElement::Miss);
        sel.set_element(2, 2, SelElement::Miss);
        sel.set_name("test_sel");
        sel
    }

    #[test]
    fn test_get_parameters_returns_height_width_cy_cx() {
        let sel = make_hit_miss_sel();
        let (sy, sx, cy, cx) = sel.get_parameters();
        assert_eq!(sy, 3);
        assert_eq!(sx, 3);
        assert_eq!(cy, 1);
        assert_eq!(cx, 1);
    }

    #[test]
    fn test_get_parameters_brick() {
        let sel = Sel::create_brick(5, 3).unwrap();
        let (sy, sx, cy, cx) = sel.get_parameters();
        assert_eq!(sy, 3); // height
        assert_eq!(sx, 5); // width
        assert_eq!(cy, 1); // height/2
        assert_eq!(cx, 2); // width/2
    }

    #[test]
    fn test_print_to_string_encodes_elements() {
        let sel = make_hit_miss_sel();
        let s = sel.print_to_string();
        // Should have height rows each ending with '\n'
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 3);
        // Center (1,1) is the origin, element is Hit → 'X'
        assert!(lines[1].contains('X'));
        // Corners are Miss → 'o'
        assert!(lines[0].starts_with('o'));
    }

    #[test]
    fn test_print_to_string_dont_care_is_space() {
        let sel = Sel::create_brick(1, 1).unwrap();
        // 1x1 with Hit at origin
        let s = sel.print_to_string();
        // Single cell, hit at origin → 'X'
        assert_eq!(s.trim(), "X");
    }

    #[test]
    fn test_write_read_roundtrip() {
        let original = make_hit_miss_sel();
        let mut buf = Vec::new();
        original.write_to_writer(&mut buf).unwrap();
        let text = std::str::from_utf8(&buf).unwrap();
        assert!(text.contains("Sel Version 1"));
        assert!(text.contains("test_sel"));

        let restored = Sel::read_from_reader(std::io::BufReader::new(buf.as_slice())).unwrap();
        assert_eq!(restored.width(), original.width());
        assert_eq!(restored.height(), original.height());
        assert_eq!(restored.origin_x(), original.origin_x());
        assert_eq!(restored.origin_y(), original.origin_y());
        assert_eq!(restored.name(), original.name());
        for y in 0..original.height() {
            for x in 0..original.width() {
                assert_eq!(restored.get_element(x, y), original.get_element(x, y));
            }
        }
    }

    #[test]
    fn test_write_format_matches_leptonica() {
        let sel = Sel::create_brick(3, 3).unwrap();
        let mut buf = Vec::new();
        sel.write_to_writer(&mut buf).unwrap();
        let text = std::str::from_utf8(&buf).unwrap();
        // Check format header
        assert!(text.contains("  Sel Version 1\n"));
        assert!(text.contains("  sy = 3, sx = 3, cy = 1, cx = 1\n"));
        // Brick: all 1 (Hit), each row "    111\n"
        assert!(text.contains("    111\n"));
    }

    #[test]
    fn test_read_from_reader_invalid_version_errors() {
        let data =
            b"  Sel Version 99\n  ------  foo  ------\n  sy = 1, sx = 1, cy = 0, cx = 0\n    1\n\n";
        let result = Sel::read_from_reader(std::io::BufReader::new(data.as_slice()));
        assert!(result.is_err());
    }

    #[test]
    fn test_from_color_image_green_is_hit_red_is_miss() {
        use leptonica_core::{Pix, PixelDepth};
        // 3x1 image: green, red, white
        let pix = Pix::new(3, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Green pixel at (0,0) → Hit
        pm.set_pixel_unchecked(0, 0, leptonica_core::color::compose_rgba(0, 255, 0, 0));
        // Red pixel at (1,0) → Miss
        pm.set_pixel_unchecked(1, 0, leptonica_core::color::compose_rgba(255, 0, 0, 0));
        // White pixel at (2,0) → DontCare
        pm.set_pixel_unchecked(2, 0, leptonica_core::color::compose_rgba(255, 255, 255, 0));
        let pix: Pix = pm.into();

        let sel = Sel::from_color_image(&pix, Some("test")).unwrap();
        assert_eq!(sel.get_element(0, 0), Some(SelElement::Hit));
        assert_eq!(sel.get_element(1, 0), Some(SelElement::Miss));
        assert_eq!(sel.get_element(2, 0), Some(SelElement::DontCare));
    }

    #[test]
    fn test_from_color_image_requires_32bpp() {
        use leptonica_core::{Pix, PixelDepth};
        let pix = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
        assert!(Sel::from_color_image(&pix, None).is_err());
    }

    #[test]
    fn test_from_pta_creates_hit_at_each_point() {
        use leptonica_core::Pta;
        let mut pta = Pta::new();
        pta.push(2.0, 1.0); // (x=2, y=1)
        pta.push(3.0, 2.0); // (x=3, y=2)
        let sel = Sel::from_pta(&pta, 0, 0, Some("pta_test")).unwrap();
        assert_eq!(sel.get_element(2, 1), Some(SelElement::Hit));
        assert_eq!(sel.get_element(3, 2), Some(SelElement::Hit));
        assert_eq!(sel.name(), Some("pta_test"));
    }

    #[test]
    fn test_from_pta_origin_and_dimensions() {
        use leptonica_core::Pta;
        let mut pta = Pta::new();
        pta.push(1.0, 0.0);
        pta.push(2.0, 1.0);
        // bounding box: x in [1,2], y in [0,1] → w=2, h=2 → sel is (x+w=4) x (y+h=2)
        let sel = Sel::from_pta(&pta, 1, 2, None).unwrap();
        assert_eq!(sel.origin_y(), 1);
        assert_eq!(sel.origin_x(), 2);
    }

    // ── Phase 4: SEL-set generation tests ────────────────────────────────

    #[test]
    fn test_sela_add_basic_count() {
        // 25 horizontal + 25 vertical + 4 squares (2..=5) + 4 diagonals = 58
        let sels = sela_add_basic();
        assert_eq!(sels.len(), 58);
    }

    #[test]
    fn test_sela_add_basic_has_expected_names() {
        let sels = sela_add_basic();
        let names: Vec<_> = sels.iter().filter_map(|s| s.name()).collect();
        assert!(names.iter().any(|n| *n == "sel_2h"), "missing sel_2h");
        assert!(names.iter().any(|n| *n == "sel_51v"), "missing sel_51v");
        assert!(
            names.iter().any(|n| *n == "sel_4"),
            "missing sel_4 (2D square)"
        );
        assert!(names.iter().any(|n| *n == "sel_2dp"), "missing sel_2dp");
        assert!(names.iter().any(|n| *n == "sel_5dm"), "missing sel_5dm");
    }

    #[test]
    fn test_sela_add_basic_linear_sel_dimensions() {
        let sels = sela_add_basic();
        // sel_5h: 1 row, 5 columns, origin (0, 2)
        let sel_5h = sels.iter().find(|s| s.name() == Some("sel_5h")).unwrap();
        assert_eq!(sel_5h.height(), 1);
        assert_eq!(sel_5h.width(), 5);
        assert_eq!(sel_5h.origin_y(), 0);
        assert_eq!(sel_5h.origin_x(), 2);
    }

    #[test]
    fn test_sela_add_hit_miss_count() {
        // 1 isolated + 4 edge + 1 slanted + 4 corner = 10
        let sels = sela_add_hit_miss();
        assert_eq!(sels.len(), 10);
    }

    #[test]
    fn test_sela_add_hit_miss_isolated_pixel() {
        let sels = sela_add_hit_miss();
        let sel = sels.iter().find(|s| s.name() == Some("sel_3hm")).unwrap();
        // 3x3, all miss except center hit
        assert_eq!(sel.width(), 3);
        assert_eq!(sel.height(), 3);
        assert_eq!(sel.hit_count(), 1);
        assert_eq!(sel.get_element(1, 1), Some(SelElement::Hit));
        assert_eq!(sel.get_element(0, 0), Some(SelElement::Miss));
    }

    #[test]
    fn test_sela_add_dwa_linear_count() {
        // sizes 2..=63 horizontal + vertical = 62 * 2 = 124
        let sels = sela_add_dwa_linear();
        assert_eq!(sels.len(), 124);
    }

    #[test]
    fn test_sela_add_dwa_linear_sizes() {
        let sels = sela_add_dwa_linear();
        // first SEL should be sel_2h: 1×2
        let first = &sels[0];
        assert_eq!(first.name(), Some("sel_2h"));
        assert_eq!(first.width(), 2);
        assert_eq!(first.height(), 1);
        // last should be sel_63v: 63×1
        let last = &sels[sels.len() - 1];
        assert_eq!(last.name(), Some("sel_63v"));
        assert_eq!(last.height(), 63);
        assert_eq!(last.width(), 1);
    }

    #[test]
    fn test_sela_add_dwa_combs_nonempty() {
        let sels = sela_add_dwa_combs();
        assert!(!sels.is_empty());
        // Each comb SEL should be either 1×N (horizontal) or N×1 (vertical)
        for s in &sels {
            let is_h = s.height() == 1 && s.width() > 1;
            let is_v = s.width() == 1 && s.height() > 1;
            assert!(is_h || is_v, "comb sel {:?} has unexpected shape", s.name());
        }
    }

    #[test]
    fn test_sela_add_dwa_combs_hit_spacing() {
        let sels = sela_add_dwa_combs();
        // sel_comb_4h should have factor1=2, factor2=2, width=4, 2 hits
        let sel = sels
            .iter()
            .find(|s| s.name() == Some("sel_comb_4h"))
            .unwrap();
        assert_eq!(sel.width(), 4);
        assert_eq!(sel.height(), 1);
        assert_eq!(sel.hit_count(), 2);
    }

    #[test]
    fn test_sela_add_cross_junctions_count() {
        let sels = sela_add_cross_junctions(6.0, 5.0, 2).unwrap();
        assert_eq!(sels.len(), 2); // norient SELs
    }

    #[test]
    fn test_sela_add_cross_junctions_has_hits_and_misses() {
        let sels = sela_add_cross_junctions(6.0, 5.0, 1).unwrap();
        let sel = &sels[0];
        assert!(sel.hit_count() > 0, "cross junction sel should have hits");
        assert!(
            sel.miss_count() > 0,
            "cross junction sel should have misses"
        );
    }

    #[test]
    fn test_sela_add_t_junctions_count() {
        let sels = sela_add_t_junctions(6.0, 5.0, 2).unwrap();
        assert_eq!(sels.len(), 8); // 4 * norient SELs
    }

    #[test]
    fn test_sel_make_plus_sign_dimensions() {
        let sel = sel_make_plus_sign(5, 1).unwrap();
        assert_eq!(sel.width(), 5);
        assert_eq!(sel.height(), 5);
        assert_eq!(sel.origin_x(), 2);
        assert_eq!(sel.origin_y(), 2);
    }

    #[test]
    fn test_sel_make_plus_sign_hit_count() {
        // size=5, linewidth=1: horizontal 5 hits + vertical 5 hits - center 1 = 9 hits
        let sel = sel_make_plus_sign(5, 1).unwrap();
        assert_eq!(sel.hit_count(), 9);
    }

    #[test]
    fn test_sel_make_plus_sign_invalid() {
        assert!(sel_make_plus_sign(2, 1).is_err(), "size < 3 should fail");
        assert!(
            sel_make_plus_sign(5, 6).is_err(),
            "linewidth > size should fail"
        );
    }

    // ── Phase 6: Sela tests ─────────────────────────────────────────────────

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sela_new_is_empty() {
        let sela = Sela::new();
        assert_eq!(sela.count(), 0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sela_add_and_count() {
        let mut sela = Sela::new();
        let mut sel = Sel::new(3, 3).unwrap();
        sel.set_name("test_sel");
        sela.add(sel).unwrap();
        assert_eq!(sela.count(), 1);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sela_add_unnamed_fails() {
        let mut sela = Sela::new();
        let sel = Sel::new(3, 3).unwrap(); // no name
        assert!(sela.add(sel).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sela_get_by_index() {
        let mut sela = Sela::new();
        let mut sel = Sel::new(3, 3).unwrap();
        sel.set_name("my_sel");
        sela.add(sel).unwrap();
        let retrieved = sela.get(0).unwrap();
        assert_eq!(retrieved.name(), Some("my_sel"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sela_get_out_of_bounds_returns_none() {
        let sela = Sela::new();
        assert!(sela.get(0).is_none());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sela_find_by_name() {
        let mut sela = Sela::new();
        let mut sel = Sel::new(3, 3).unwrap();
        sel.set_name("target");
        sela.add(sel).unwrap();
        assert!(sela.find_by_name("target").is_some());
        assert!(sela.find_by_name("missing").is_none());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_sela_write_and_read_roundtrip() {
        use std::io::BufReader;

        let mut sela = Sela::new();
        let mut sel1 = Sel::new(3, 3).unwrap();
        sel1.set_name("sel_a");
        sel1.set_element(1, 1, SelElement::Hit);
        sela.add(sel1).unwrap();

        let mut sel2 = Sel::new(5, 1).unwrap();
        sel2.set_name("sel_b");
        sel2.set_origin(2, 0).unwrap();
        for x in 0..5 {
            sel2.set_element(x, 0, SelElement::Hit);
        }
        sela.add(sel2).unwrap();

        let tmp = std::env::temp_dir().join("test_sela_roundtrip.sel");
        sela.write(&tmp).unwrap();

        let loaded = Sela::read(&tmp).unwrap();
        assert_eq!(loaded.count(), 2);
        assert!(loaded.find_by_name("sel_a").is_some());
        assert!(loaded.find_by_name("sel_b").is_some());

        let _ = std::fs::remove_file(&tmp);
    }
}
