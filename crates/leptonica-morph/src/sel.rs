//! Structuring Element (SEL) for morphological operations
//!
//! A structuring element defines the neighborhood used in morphological operations.

use crate::{MorphError, MorphResult};
use leptonica_core::{Pix, Pta};
use std::io::{BufRead, Write};

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

        let (x_min, y_min, x_max, y_max) = pta
            .bounding_box()
            .ok_or_else(|| MorphError::InvalidParameters("PTA is empty".into()))?;

        if x_min < 0.0 || y_min < 0.0 {
            return Err(MorphError::InvalidParameters(
                "PTA points must have non-negative coordinates".into(),
            ));
        }

        let w = x_max as u32 + 1;
        let h = y_max as u32 + 1;
        let mut sel = Sel::new(w, h)?;
        sel.set_origin(cx, cy)?;
        if let Some(n) = name {
            sel.set_name(n);
        }

        for i in 0..pta.len() {
            let (x, y) = pta
                .get_i_pt(i)
                .ok_or_else(|| MorphError::InvalidParameters("PTA index out of bounds".into()))?;
            if x < 0 || y < 0 {
                return Err(MorphError::InvalidParameters(
                    "PTA point coordinates must be non-negative".into(),
                ));
            }
            sel.set_element(x as u32, y as u32, SelElement::Hit);
        }

        Ok(sel)
    }
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
}
