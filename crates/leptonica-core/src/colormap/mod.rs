//! PixColormap - Color palette for indexed images
//!
//! A colormap is used with 1, 2, 4, and 8 bpp images to map
//! pixel values to RGBA colors.

pub mod serial;

use crate::error::{Error, Result};

/// RGBA color entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RgbaQuad {
    /// Blue component (stored first for BMP compatibility)
    pub blue: u8,
    /// Green component
    pub green: u8,
    /// Red component
    pub red: u8,
    /// Alpha component
    pub alpha: u8,
}

impl RgbaQuad {
    /// Create a new RGBA color
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Create an RGB color (alpha = 255)
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::new(red, green, blue, 255)
    }

    /// Create a grayscale color
    pub fn gray(value: u8) -> Self {
        Self::rgb(value, value, value)
    }
}

/// Colormap for indexed images
#[derive(Debug, Clone)]
pub struct PixColormap {
    /// Color entries
    colors: Vec<RgbaQuad>,
    /// Depth of the associated image (1, 2, 4, or 8)
    depth: u32,
}

impl PixColormap {
    /// Create a new colormap for the specified depth
    ///
    /// # Arguments
    ///
    /// * `depth` - Image depth (1, 2, 4, or 8)
    ///
    /// # Errors
    ///
    /// Returns an error if depth is not 1, 2, 4, or 8.
    pub fn new(depth: u32) -> Result<Self> {
        if !matches!(depth, 1 | 2 | 4 | 8) {
            return Err(Error::ColormapNotAllowed(depth));
        }

        let max_entries = 1usize << depth;
        Ok(Self {
            colors: Vec::with_capacity(max_entries),
            depth,
        })
    }

    /// Create a grayscale colormap
    pub fn create_linear(depth: u32, dark_to_light: bool) -> Result<Self> {
        let mut cmap = Self::new(depth)?;
        let n = 1u32 << depth;

        for i in 0..n {
            let val = if dark_to_light {
                (i * 255 / (n - 1)) as u8
            } else {
                ((n - 1 - i) * 255 / (n - 1)) as u8
            };
            cmap.add_color(RgbaQuad::gray(val))?;
        }

        Ok(cmap)
    }

    /// Get the depth
    #[inline]
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Get the number of colors
    #[inline]
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Get maximum number of entries
    #[inline]
    pub fn max_entries(&self) -> usize {
        1 << self.depth
    }

    /// Get a color by index
    pub fn get(&self, index: usize) -> Option<&RgbaQuad> {
        self.colors.get(index)
    }

    /// Get a mutable color by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut RgbaQuad> {
        self.colors.get_mut(index)
    }

    /// Add a color to the colormap
    pub fn add_color(&mut self, color: RgbaQuad) -> Result<usize> {
        if self.colors.len() >= self.max_entries() {
            return Err(Error::InvalidParameter(format!(
                "colormap is full ({} entries)",
                self.max_entries()
            )));
        }
        let index = self.colors.len();
        self.colors.push(color);
        Ok(index)
    }

    /// Add an RGB color
    pub fn add_rgb(&mut self, r: u8, g: u8, b: u8) -> Result<usize> {
        self.add_color(RgbaQuad::rgb(r, g, b))
    }

    /// Add an RGBA color
    pub fn add_rgba(&mut self, r: u8, g: u8, b: u8, a: u8) -> Result<usize> {
        self.add_color(RgbaQuad::new(r, g, b, a))
    }

    /// Set a color at a specific index
    pub fn set_color(&mut self, index: usize, color: RgbaQuad) -> Result<()> {
        if index >= self.colors.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.colors.len(),
            });
        }
        self.colors[index] = color;
        Ok(())
    }

    /// Get RGB values at index
    pub fn get_rgb(&self, index: usize) -> Option<(u8, u8, u8)> {
        self.colors.get(index).map(|c| (c.red, c.green, c.blue))
    }

    /// Get RGBA values at index
    pub fn get_rgba(&self, index: usize) -> Option<(u8, u8, u8, u8)> {
        self.colors
            .get(index)
            .map(|c| (c.red, c.green, c.blue, c.alpha))
    }

    /// Find the nearest color in the colormap
    pub fn find_nearest(&self, r: u8, g: u8, b: u8) -> Option<usize> {
        if self.colors.is_empty() {
            return None;
        }

        let mut min_dist = u32::MAX;
        let mut best_index = 0;

        for (i, color) in self.colors.iter().enumerate() {
            let dr = (r as i32 - color.red as i32).unsigned_abs();
            let dg = (g as i32 - color.green as i32).unsigned_abs();
            let db = (b as i32 - color.blue as i32).unsigned_abs();
            let dist = dr * dr + dg * dg + db * db;

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

    /// Check if the colormap contains only grayscale colors
    pub fn is_grayscale(&self) -> bool {
        self.colors
            .iter()
            .all(|c| c.red == c.green && c.green == c.blue)
    }

    /// Check if all colors in the colormap are fully opaque (alpha = 255)
    pub fn is_opaque(&self) -> bool {
        self.colors.iter().all(|c| c.alpha == 255)
    }

    /// Check if the colormap contains any color (non-grayscale entries)
    pub fn has_color(&self) -> bool {
        self.colors
            .iter()
            .any(|c| c.red != c.green || c.red != c.blue)
    }

    /// Check if the colormap is black and white only (for 1 bpp images)
    ///
    /// Returns true if the colormap has exactly 2 entries and they are
    /// black (0,0,0) and white (255,255,255) in any order.
    pub fn is_black_and_white(&self) -> bool {
        if self.colors.len() != 2 {
            return false;
        }
        let c0 = &self.colors[0];
        let c1 = &self.colors[1];
        let is_black = |c: &RgbaQuad| c.red == 0 && c.green == 0 && c.blue == 0;
        let is_white = |c: &RgbaQuad| c.red == 255 && c.green == 255 && c.blue == 255;
        (is_black(c0) && is_white(c1)) || (is_white(c0) && is_black(c1))
    }

    /// Get all colors as a slice
    pub fn colors(&self) -> &[RgbaQuad] {
        &self.colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colormap_creation() {
        let cmap = PixColormap::new(8).unwrap();
        assert_eq!(cmap.depth(), 8);
        assert_eq!(cmap.max_entries(), 256);
        assert!(cmap.is_empty());

        assert!(PixColormap::new(16).is_err());
    }

    #[test]
    fn test_add_colors() {
        let mut cmap = PixColormap::new(2).unwrap();
        assert_eq!(cmap.max_entries(), 4);

        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        cmap.add_rgb(0, 255, 0).unwrap();
        cmap.add_rgb(0, 0, 255).unwrap();

        assert_eq!(cmap.len(), 4);
        assert!(cmap.add_rgb(255, 255, 255).is_err()); // Full

        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        assert_eq!(cmap.get_rgb(1), Some((255, 0, 0)));
    }

    #[test]
    fn test_linear_colormap() {
        let cmap = PixColormap::create_linear(8, true).unwrap();
        assert_eq!(cmap.len(), 256);
        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        assert_eq!(cmap.get_rgb(255), Some((255, 255, 255)));
        assert!(cmap.is_grayscale());
    }

    #[test]
    fn test_find_nearest() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        cmap.add_rgb(0, 255, 0).unwrap();
        cmap.add_rgb(0, 0, 255).unwrap();

        assert_eq!(cmap.find_nearest(0, 0, 0), Some(0));
        assert_eq!(cmap.find_nearest(200, 50, 50), Some(1)); // Closest to red
        assert_eq!(cmap.find_nearest(50, 200, 50), Some(2)); // Closest to green
    }
}
