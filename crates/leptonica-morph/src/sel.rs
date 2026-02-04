//! Structuring Element (SEL) for morphological operations
//!
//! A structuring element defines the neighborhood used in morphological operations.

use crate::{MorphError, MorphResult};

/// Element type in a structuring element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SelElement {
    /// Don't care - this position is ignored
    DontCare = 0,
    /// Hit - must match foreground (set pixels)
    Hit = 1,
    /// Miss - must match background (unset pixels)
    Miss = 2,
}

impl Default for SelElement {
    fn default() -> Self {
        SelElement::DontCare
    }
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
}
