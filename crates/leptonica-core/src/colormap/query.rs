//! PixColormap query and info functions
//!
//! Additional query, modification, and info functions for colormaps.
//!
//! # See also
//!
//! C Leptonica: `colormap.c`

use super::{PixColormap, RgbaQuad};
use crate::color;
use crate::error::{Error, Result};

/// Component selector for range value queries.
///
/// # See also
///
/// C Leptonica: `L_SELECT_RED`, `L_SELECT_GREEN`, `L_SELECT_BLUE`,
/// `L_SELECT_AVERAGE` in `colormap.c`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeComponent {
    Red,
    Green,
    Blue,
    Average,
}

/// Information about non-opaque colors in a colormap.
///
/// # See also
///
/// C Leptonica: `pixcmapNonOpaqueColorsInfo()` in `colormap.c`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonOpaqueInfo {
    /// Number of entries with alpha != 255
    pub num_transparent: usize,
    /// Highest index with alpha != 255, or None if all opaque
    pub max_transparent_index: Option<usize>,
    /// Lowest index with alpha == 255, or None if all transparent
    pub min_opaque_index: Option<usize>,
}

/// Result of a range value query.
///
/// # See also
///
/// C Leptonica: `pixcmapGetRangeValues()` in `colormap.c`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeValues {
    pub min_val: i32,
    pub max_val: i32,
    pub min_index: usize,
    pub max_index: usize,
}

impl PixColormap {
    /// Create a colormap with random colors.
    ///
    /// # Arguments
    ///
    /// * `depth` - Image depth (2, 4, or 8)
    /// * `has_black` - Add black (0,0,0) at index 0
    /// * `has_white` - Add white (255,255,255) at last index
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapCreateRandom()` in `colormap.c`
    pub fn create_random(depth: u32, has_black: bool, has_white: bool) -> Result<Self> {
        if !matches!(depth, 2 | 4 | 8) {
            return Err(Error::InvalidParameter(format!(
                "create_random requires depth 2, 4, or 8, got {depth}"
            )));
        }

        let n = 1usize << depth;
        let mut cmap = Self::new(depth)?;

        if has_black {
            cmap.add_color(RgbaQuad::rgb(0, 0, 0))?;
        }

        let random_count = n - usize::from(has_black) - usize::from(has_white);
        // Simple deterministic "random" using a linear congruential generator
        // to avoid depending on rand crate. Seed with depth for variety.
        let mut state: u32 = 1_103_515_245u32.wrapping_mul(depth).wrapping_add(12345);
        for _ in 0..random_count {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let r = ((state >> 16) & 0xff) as u8;
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let g = ((state >> 16) & 0xff) as u8;
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let b = ((state >> 16) & 0xff) as u8;
            cmap.add_color(RgbaQuad::rgb(r, g, b))?;
        }

        if has_white {
            cmap.add_color(RgbaQuad::rgb(255, 255, 255))?;
        }

        Ok(cmap)
    }

    /// Check if the number of colors is valid for the depth.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapIsValid()` in `colormap.c`
    pub fn is_valid(&self) -> bool {
        matches!(self.depth(), 1 | 2 | 4 | 8) && self.len() <= self.max_entries()
    }

    /// Add a color only if it does not already exist.
    ///
    /// Returns `Ok(Some(index))` if the color was added or already exists.
    /// Returns `Ok(None)` if the colormap is full and the color is not present.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapAddNewColor()` in `colormap.c`
    pub fn add_new_color(&mut self, r: u8, g: u8, b: u8) -> Result<Option<usize>> {
        if let Some(idx) = self.get_index(r, g, b) {
            return Ok(Some(idx));
        }
        if self.len() < self.max_entries() {
            let idx = self.add_rgb(r, g, b)?;
            Ok(Some(idx))
        } else {
            Ok(None)
        }
    }

    /// Add a color or return the nearest existing color index.
    ///
    /// Always returns a valid index: exact match, newly added, or nearest.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapAddNearestColor()` in `colormap.c`
    pub fn add_nearest_color(&mut self, r: u8, g: u8, b: u8) -> Result<usize> {
        if let Some(idx) = self.get_index(r, g, b) {
            return Ok(idx);
        }
        if self.len() < self.max_entries() {
            let idx = self.add_rgb(r, g, b)?;
            Ok(idx)
        } else {
            // Colormap full → return nearest
            self.find_nearest(r, g, b)
                .ok_or_else(|| Error::InvalidParameter("colormap is empty".into()))
        }
    }

    /// Check if a color is usable (can be added or already exists).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapUsableColor()` in `colormap.c`
    pub fn is_usable_color(&self, r: u8, g: u8, b: u8) -> bool {
        if self.len() < self.max_entries() {
            return true;
        }
        self.get_index(r, g, b).is_some()
    }

    /// Add black or white, falling back to nearest intensity if full.
    ///
    /// # Arguments
    ///
    /// * `black` - true for black (0,0,0), false for white (255,255,255)
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapAddBlackOrWhite()` in `colormap.c`
    pub fn add_black_or_white(&mut self, black: bool) -> Result<usize> {
        let (r, g, b) = if black { (0, 0, 0) } else { (255, 255, 255) };

        if let Some(idx) = self.add_new_color(r, g, b)? {
            return Ok(idx);
        }

        // Colormap full and no exact match → find by rank intensity
        let rank = if black { 0.0 } else { 1.0 };
        self.get_rank_intensity(rank)
    }

    /// Set the darkest color to pure black and/or the lightest to pure white.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapSetBlackAndWhite()` in `colormap.c`
    pub fn set_black_and_white(&mut self, set_black: bool, set_white: bool) -> Result<()> {
        if set_black {
            let idx = self.get_rank_intensity(0.0)?;
            self.reset_color(idx, 0, 0, 0)?;
        }
        if set_white {
            let idx = self.get_rank_intensity(1.0)?;
            self.reset_color(idx, 255, 255, 255)?;
        }
        Ok(())
    }

    /// Get the number of unused (free) entries.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGetFreeCount()` in `colormap.c`
    pub fn free_count(&self) -> usize {
        self.max_entries() - self.len()
    }

    /// Get the minimum depth needed for the current color count.
    ///
    /// Returns 2, 4, or 8.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGetMinDepth()` in `colormap.c`
    pub fn min_depth(&self) -> u32 {
        let n = self.len();
        if n <= 4 {
            2
        } else if n <= 16 {
            4
        } else {
            8
        }
    }

    /// Remove all colors (set count to 0).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapClear()` in `colormap.c`
    pub fn clear(&mut self) {
        self.colors.clear();
    }

    /// Get a packed 32-bit RGB value at index (alpha = 255).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGetColor32()` in `colormap.c`
    pub fn get_color32(&self, index: usize) -> Option<u32> {
        self.get_rgb(index)
            .map(|(r, g, b)| color::compose_rgb(r, g, b))
    }

    /// Get a packed 32-bit RGBA value at index.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGetRGBA32()` in `colormap.c`
    pub fn get_rgba32(&self, index: usize) -> Option<u32> {
        self.get_rgba(index)
            .map(|(r, g, b, a)| color::compose_rgba(r, g, b, a))
    }

    /// Reset the RGB color at an existing index (alpha set to 255).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapResetColor()` in `colormap.c`
    pub fn reset_color(&mut self, index: usize, r: u8, g: u8, b: u8) -> Result<()> {
        self.set_color(index, RgbaQuad::rgb(r, g, b))
    }

    /// Set the alpha value at an existing index.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapSetAlpha()` in `colormap.c`
    pub fn set_alpha(&mut self, index: usize, alpha: u8) -> Result<()> {
        let len = self.len();
        let entry = self
            .get_mut(index)
            .ok_or(Error::IndexOutOfBounds { index, len })?;
        entry.alpha = alpha;
        Ok(())
    }

    /// Find the index of an exact RGB color match.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGetIndex()` in `colormap.c`
    pub fn get_index(&self, r: u8, g: u8, b: u8) -> Option<usize> {
        self.colors()
            .iter()
            .position(|c| c.red == r && c.green == g && c.blue == b)
    }

    /// Get information about non-opaque colors.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapNonOpaqueColorsInfo()` in `colormap.c`
    pub fn non_opaque_info(&self) -> NonOpaqueInfo {
        let mut num_transparent = 0;
        let mut max_transparent_index = None;
        let mut min_opaque_index = None;

        for (i, c) in self.colors().iter().enumerate() {
            if c.alpha != 255 {
                num_transparent += 1;
                max_transparent_index = Some(i);
            } else if min_opaque_index.is_none() {
                min_opaque_index = Some(i);
            }
        }

        NonOpaqueInfo {
            num_transparent,
            max_transparent_index,
            min_opaque_index,
        }
    }

    /// Count unique gray colors (entries where r == g == b).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapCountGrayColors()` in `colormap.c`
    pub fn count_gray_colors(&self) -> usize {
        let mut seen = [false; 256];
        let mut count = 0;
        for c in self.colors() {
            if c.red == c.green && c.green == c.blue && !seen[c.red as usize] {
                seen[c.red as usize] = true;
                count += 1;
            }
        }
        count
    }

    /// Get the index of the color at a given intensity rank.
    ///
    /// `rank` is in [0.0, 1.0] where 0.0 = darkest, 1.0 = lightest.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGetRankIntensity()` in `colormap.c`
    pub fn get_rank_intensity(&self, rank: f32) -> Result<usize> {
        if !(0.0..=1.0).contains(&rank) {
            return Err(Error::InvalidParameter(format!(
                "rank must be in [0.0, 1.0], got {rank}"
            )));
        }
        let n = self.len();
        if n == 0 {
            return Err(Error::InvalidParameter("colormap is empty".into()));
        }

        // Compute intensity (r + g + b) for each color and sort indices
        let mut indexed_intensities: Vec<(usize, i32)> = self
            .colors()
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.red as i32 + c.green as i32 + c.blue as i32))
            .collect();
        indexed_intensities.sort_by_key(|&(_, intensity)| intensity);

        let rank_index = (rank * (n - 1) as f32 + 0.5) as usize;
        let rank_index = rank_index.min(n - 1);
        Ok(indexed_intensities[rank_index].0)
    }

    /// Find the nearest color using only the green channel.
    ///
    /// Optimized for grayscale colormaps where r == g == b.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGetNearestGrayIndex()` in `colormap.c`
    pub fn find_nearest_gray(&self, val: u8) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        let mut min_dist = 256i32;
        let mut best_index = 0;

        for (i, c) in self.colors().iter().enumerate() {
            let dist = (c.green as i32 - val as i32).abs();
            if dist < min_dist {
                min_dist = dist;
                best_index = i;
                if dist == 0 {
                    break;
                }
            }
        }

        Some(best_index)
    }

    /// Get the squared L2 distance between a colormap entry and a target color.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGetDistanceToColor()` in `colormap.c`
    pub fn distance_to_color(&self, index: usize, r: u8, g: u8, b: u8) -> Option<u32> {
        let c = self.get(index)?;
        let dr = c.red as i32 - r as i32;
        let dg = c.green as i32 - g as i32;
        let db = c.blue as i32 - b as i32;
        Some((dr * dr + dg * dg + db * db) as u32)
    }

    /// Get the min/max values and indices for a color component.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGetRangeValues()` in `colormap.c`
    pub fn get_range_values(&self, component: RangeComponent) -> Option<RangeValues> {
        if self.is_empty() {
            return None;
        }

        let mut min_val = i32::MAX;
        let mut max_val = i32::MIN;
        let mut min_index = 0;
        let mut max_index = 0;

        for (i, c) in self.colors().iter().enumerate() {
            let val = match component {
                RangeComponent::Red => c.red as i32,
                RangeComponent::Green => c.green as i32,
                RangeComponent::Blue => c.blue as i32,
                RangeComponent::Average => (c.red as i32 + c.green as i32 + c.blue as i32) / 3,
            };
            if val < min_val {
                min_val = val;
                min_index = i;
            }
            if val > max_val {
                max_val = val;
                max_index = i;
            }
        }

        Some(RangeValues {
            min_val,
            max_val,
            min_index,
            max_index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_random() {
        let cmap = PixColormap::create_random(8, true, true).unwrap();
        assert_eq!(cmap.len(), 256);
        // First color should be black
        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        // Last color should be white
        assert_eq!(cmap.get_rgb(255), Some((255, 255, 255)));
    }

    #[test]
    fn test_create_random_no_black_white() {
        let cmap = PixColormap::create_random(4, false, false).unwrap();
        assert_eq!(cmap.len(), 16);
    }

    #[test]
    fn test_create_random_invalid_depth() {
        assert!(PixColormap::create_random(1, false, false).is_err());
        assert!(PixColormap::create_random(8, false, false).is_ok());
    }

    #[test]
    fn test_is_valid() {
        let cmap = PixColormap::create_linear(8, true).unwrap();
        assert!(cmap.is_valid());

        let empty = PixColormap::new(8).unwrap();
        assert!(empty.is_valid());
    }

    #[test]
    fn test_add_new_color() {
        let mut cmap = PixColormap::new(8).unwrap();
        // First add
        assert_eq!(cmap.add_new_color(255, 0, 0).unwrap(), Some(0));
        // Duplicate should return existing index
        assert_eq!(cmap.add_new_color(255, 0, 0).unwrap(), Some(0));
        // Different color
        assert_eq!(cmap.add_new_color(0, 255, 0).unwrap(), Some(1));
        assert_eq!(cmap.len(), 2);
    }

    #[test]
    fn test_add_new_color_full() {
        let mut cmap = PixColormap::new(2).unwrap(); // max 4
        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        cmap.add_rgb(0, 255, 0).unwrap();
        cmap.add_rgb(0, 0, 255).unwrap();
        // Full + color doesn't exist → None
        assert_eq!(cmap.add_new_color(128, 128, 128).unwrap(), None);
        // Full but color exists → Some(index)
        assert_eq!(cmap.add_new_color(255, 0, 0).unwrap(), Some(1));
    }

    #[test]
    fn test_add_nearest_color() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        cmap.add_rgb(0, 255, 0).unwrap();
        cmap.add_rgb(0, 0, 255).unwrap();
        // Full → returns nearest
        let idx = cmap.add_nearest_color(200, 50, 50).unwrap();
        assert_eq!(idx, 1); // nearest to red
    }

    #[test]
    fn test_is_usable_color() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        // Room available → usable
        assert!(cmap.is_usable_color(0, 0, 255));
        // Fill up
        cmap.add_rgb(0, 255, 0).unwrap();
        cmap.add_rgb(0, 0, 255).unwrap();
        cmap.add_rgb(128, 128, 128).unwrap();
        // Full but exists → usable
        assert!(cmap.is_usable_color(255, 0, 0));
        // Full and not exists → not usable
        assert!(!cmap.is_usable_color(1, 2, 3));
    }

    #[test]
    fn test_add_black_or_white() {
        let mut cmap = PixColormap::new(8).unwrap();
        let idx = cmap.add_black_or_white(true).unwrap();
        assert_eq!(cmap.get_rgb(idx), Some((0, 0, 0)));
        let idx = cmap.add_black_or_white(false).unwrap();
        assert_eq!(cmap.get_rgb(idx), Some((255, 255, 255)));
    }

    #[test]
    fn test_add_black_or_white_full() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(50, 50, 50).unwrap(); // darkish
        cmap.add_rgb(100, 100, 100).unwrap();
        cmap.add_rgb(150, 150, 150).unwrap();
        cmap.add_rgb(200, 200, 200).unwrap(); // lightish
        // Full → returns nearest intensity
        let idx = cmap.add_black_or_white(true).unwrap(); // black
        assert_eq!(idx, 0); // darkest
        let idx = cmap.add_black_or_white(false).unwrap(); // white
        assert_eq!(idx, 3); // lightest
    }

    #[test]
    fn test_set_black_and_white() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(10, 10, 10).unwrap(); // darkest
        cmap.add_rgb(128, 128, 128).unwrap();
        cmap.add_rgb(245, 245, 245).unwrap(); // lightest
        cmap.set_black_and_white(true, true).unwrap();
        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        assert_eq!(cmap.get_rgb(2), Some((255, 255, 255)));
    }

    #[test]
    fn test_free_count() {
        let mut cmap = PixColormap::new(2).unwrap();
        assert_eq!(cmap.free_count(), 4);
        cmap.add_rgb(0, 0, 0).unwrap();
        assert_eq!(cmap.free_count(), 3);
    }

    #[test]
    fn test_min_depth() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        assert_eq!(cmap.min_depth(), 2); // 2 colors → needs 2-bit

        for i in 2..5 {
            cmap.add_rgb(i, i, i).unwrap();
        }
        assert_eq!(cmap.min_depth(), 4); // 5 colors → needs 4-bit

        for i in 5..17 {
            cmap.add_rgb(i, i, i).unwrap();
        }
        assert_eq!(cmap.min_depth(), 8); // 17 colors → needs 8-bit
    }

    #[test]
    fn test_clear() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        assert_eq!(cmap.len(), 2);
        cmap.clear();
        assert_eq!(cmap.len(), 0);
        assert!(cmap.is_empty());
    }

    #[test]
    fn test_get_color32() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(0xAA, 0xBB, 0xCC).unwrap();
        // 0xRRGGBBAA with alpha=255 → 0xAABBCCFF
        assert_eq!(cmap.get_color32(0), Some(0xAABBCCFF));
        assert_eq!(cmap.get_color32(1), None);
    }

    #[test]
    fn test_get_rgba32() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(0xAA, 0xBB, 0xCC, 0x80).unwrap();
        // 0xRRGGBBAA → 0xAABBCC80
        assert_eq!(cmap.get_rgba32(0), Some(0xAABBCC80));
    }

    #[test]
    fn test_reset_color() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(100, 100, 100, 128).unwrap();
        cmap.reset_color(0, 200, 200, 200).unwrap();
        // Alpha should be reset to 255
        assert_eq!(cmap.get_rgba(0), Some((200, 200, 200, 255)));
        assert!(cmap.reset_color(1, 0, 0, 0).is_err());
    }

    #[test]
    fn test_set_alpha() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(100, 100, 100).unwrap();
        assert_eq!(cmap.get_rgba(0), Some((100, 100, 100, 255)));
        cmap.set_alpha(0, 128).unwrap();
        assert_eq!(cmap.get_rgba(0), Some((100, 100, 100, 128)));
        assert!(cmap.set_alpha(1, 0).is_err());
    }

    #[test]
    fn test_get_index() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        cmap.add_rgb(0, 255, 0).unwrap();
        assert_eq!(cmap.get_index(255, 0, 0), Some(0));
        assert_eq!(cmap.get_index(0, 255, 0), Some(1));
        assert_eq!(cmap.get_index(0, 0, 255), None);
    }

    #[test]
    fn test_non_opaque_info_all_opaque() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(255, 255, 255).unwrap();
        let info = cmap.non_opaque_info();
        assert_eq!(info.num_transparent, 0);
        assert_eq!(info.max_transparent_index, None);
        assert_eq!(info.min_opaque_index, Some(0));
    }

    #[test]
    fn test_non_opaque_info_mixed() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(0, 0, 0, 255).unwrap(); // opaque
        cmap.add_rgba(100, 0, 0, 128).unwrap(); // transparent
        cmap.add_rgba(200, 0, 0, 0).unwrap(); // transparent
        cmap.add_rgba(255, 255, 255, 255).unwrap(); // opaque
        let info = cmap.non_opaque_info();
        assert_eq!(info.num_transparent, 2);
        assert_eq!(info.max_transparent_index, Some(2));
        assert_eq!(info.min_opaque_index, Some(0));
    }

    #[test]
    fn test_count_gray_colors() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap(); // gray
        cmap.add_rgb(128, 128, 128).unwrap(); // gray
        cmap.add_rgb(128, 128, 128).unwrap(); // duplicate gray
        cmap.add_rgb(255, 0, 0).unwrap(); // not gray
        assert_eq!(cmap.count_gray_colors(), 2); // 0 and 128
    }

    #[test]
    fn test_get_rank_intensity() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(200, 200, 200).unwrap(); // intensity 600
        cmap.add_rgb(10, 10, 10).unwrap(); // intensity 30  (darkest)
        cmap.add_rgb(100, 100, 100).unwrap(); // intensity 300
        let darkest = cmap.get_rank_intensity(0.0).unwrap();
        assert_eq!(darkest, 1); // index 1 has lowest intensity
        let lightest = cmap.get_rank_intensity(1.0).unwrap();
        assert_eq!(lightest, 0); // index 0 has highest intensity
    }

    #[test]
    fn test_find_nearest_gray() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(128, 128, 128).unwrap();
        cmap.add_rgb(255, 255, 255).unwrap();
        assert_eq!(cmap.find_nearest_gray(0), Some(0));
        assert_eq!(cmap.find_nearest_gray(100), Some(1)); // closer to 128
        assert_eq!(cmap.find_nearest_gray(200), Some(2)); // closer to 255
    }

    #[test]
    fn test_distance_to_color() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap();
        // distance from (0,0,0) to (10,20,30) = 10²+20²+30² = 100+400+900 = 1400
        assert_eq!(cmap.distance_to_color(0, 10, 20, 30), Some(1400));
        assert_eq!(cmap.distance_to_color(0, 0, 0, 0), Some(0));
        assert_eq!(cmap.distance_to_color(1, 0, 0, 0), None); // out of bounds
    }

    #[test]
    fn test_get_range_values() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(100, 50, 200).unwrap();
        cmap.add_rgb(50, 150, 100).unwrap();
        cmap.add_rgb(200, 100, 50).unwrap();

        let rv = cmap.get_range_values(RangeComponent::Red).unwrap();
        assert_eq!(rv.min_val, 50);
        assert_eq!(rv.max_val, 200);
        assert_eq!(rv.min_index, 1);
        assert_eq!(rv.max_index, 2);
    }

    #[test]
    fn test_get_range_values_average() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(90, 90, 90).unwrap(); // avg 90
        cmap.add_rgb(30, 30, 30).unwrap(); // avg 30
        cmap.add_rgb(200, 200, 200).unwrap(); // avg 200

        let rv = cmap.get_range_values(RangeComponent::Average).unwrap();
        assert_eq!(rv.min_val, 30);
        assert_eq!(rv.max_val, 200);
        assert_eq!(rv.min_index, 1);
        assert_eq!(rv.max_index, 2);
    }
}
