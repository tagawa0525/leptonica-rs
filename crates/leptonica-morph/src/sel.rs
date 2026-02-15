//! Structuring Element (SEL) for morphological operations
//!
//! A structuring element defines the neighborhood used in morphological operations.

use crate::{MorphError, MorphResult};

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
    pub fn create_comb_horizontal(_factor1: u32, _factor2: u32) -> MorphResult<Self> {
        todo!("create_comb_horizontal not yet implemented")
    }

    /// Create a vertical comb structuring element
    ///
    /// Same as horizontal comb but oriented vertically.
    pub fn create_comb_vertical(_factor1: u32, _factor2: u32) -> MorphResult<Self> {
        todo!("create_comb_vertical not yet implemented")
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
    #[ignore = "create_comb not yet implemented"]
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
    #[ignore = "create_comb not yet implemented"]
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
    #[ignore = "create_comb not yet implemented"]
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
}
