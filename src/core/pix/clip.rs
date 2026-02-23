//! Rectangle clipping operations for images
//!
//! Functions for extracting rectangular sub-regions from images,
//! foreground detection, mask generation, and line averaging.
//!
//! # See also
//!
//! C Leptonica: `pix2.c`, `pix5.c`

use super::{Pix, PixelDepth};
use crate::Box;
use crate::error::{Error, Result};

/// Direction for scanning an image to find the foreground edge.
///
/// # See also
///
/// C Leptonica: `L_FROM_LEFT`, `L_FROM_RIGHT`, `L_FROM_TOP`, `L_FROM_BOT`
/// in `pix5.c`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDirection {
    /// Scan from left edge toward right
    FromLeft,
    /// Scan from right edge toward left
    FromRight,
    /// Scan from top edge toward bottom
    FromTop,
    /// Scan from bottom edge toward top
    FromBot,
}

impl Pix {
    /// Extract a rectangular sub-region from the image.
    ///
    /// Creates a new image containing the specified rectangle. If the
    /// rectangle extends beyond the image bounds, it is clipped to the
    /// valid region. Returns an error if the rectangle is entirely outside
    /// the image.
    ///
    /// For 32-bit images, the output preserves the samples-per-pixel
    /// value from the source.
    ///
    /// C equivalent: `pixClipRectangle()` in `pix2.c`
    ///
    /// # Arguments
    ///
    /// * `x` - Left edge of the rectangle
    /// * `y` - Top edge of the rectangle
    /// * `w` - Width of the rectangle
    /// * `h` - Height of the rectangle
    ///
    /// # Returns
    ///
    /// A new `Pix` containing the clipped region.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The requested width or height is 0
    /// - The rectangle is entirely outside the image bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
    /// let clipped = pix.clip_rectangle(10, 20, 50, 40).unwrap();
    /// assert_eq!(clipped.width(), 50);
    /// assert_eq!(clipped.height(), 40);
    /// ```
    ///
    /// Regions extending beyond the image are clipped:
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
    /// let clipped = pix.clip_rectangle(80, 60, 50, 50).unwrap();
    /// assert_eq!(clipped.width(), 20);   // clipped: 100 - 80
    /// assert_eq!(clipped.height(), 20);  // clipped: 80 - 60
    /// ```
    pub fn clip_rectangle(&self, x: u32, y: u32, w: u32, h: u32) -> Result<Pix> {
        if w == 0 || h == 0 {
            return Err(Error::InvalidParameter(format!(
                "clip rectangle has zero dimension: {}x{}",
                w, h
            )));
        }

        let src_w = self.width();
        let src_h = self.height();

        // Check if the rectangle is entirely outside the image
        if x >= src_w || y >= src_h {
            return Err(Error::InvalidParameter(format!(
                "clip rectangle origin ({}, {}) is outside image bounds ({}x{})",
                x, y, src_w, src_h
            )));
        }

        // Clip the rectangle to the image bounds
        let clip_w = w.min(src_w - x);
        let clip_h = h.min(src_h - y);

        let depth = self.depth();
        let pixd = Pix::new(clip_w, clip_h, depth)?;
        let mut pixd_mut = pixd.try_into_mut().unwrap();

        // Preserve spp for 32-bit images
        if depth == PixelDepth::Bit32 {
            pixd_mut.set_spp(self.spp());
        }

        // Copy resolution from source
        pixd_mut.set_resolution(self.xres(), self.yres());

        // Copy pixel data
        for dy in 0..clip_h {
            for dx in 0..clip_w {
                let val = self.get_pixel_unchecked(x + dx, y + dy);
                pixd_mut.set_pixel_unchecked(dx, dy, val);
            }
        }

        Ok(pixd_mut.into())
    }
}

// ============================================================================
// Advanced clipping, foreground detection, and mask generation (pix5.c)
// ============================================================================

impl Pix {
    /// Extract a rectangular sub-region with an additional border.
    ///
    /// First intersects the region with the image bounds. Then expands
    /// the clipped region by the border amount (clamped symmetrically
    /// so it does not exceed the distance from the region to the image
    /// edge). The border area contains pixels copied from the source image.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClipRectangleWithBorder()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `region` - The rectangle to extract
    /// * `max_border` - Maximum border width to add around the region
    ///
    /// # Errors
    ///
    /// Returns an error if the region does not intersect the image.
    pub fn clip_rectangle_with_border(&self, region: &Box, max_border: u32) -> Result<(Pix, Box)> {
        let w = self.width() as i32;
        let h = self.height() as i32;

        // Intersect region with image bounds first
        let clipped_box = region.clip(w, h).ok_or_else(|| {
            Error::InvalidParameter("region does not intersect image".to_string())
        })?;
        let (bx, by, bw, bh) = (clipped_box.x, clipped_box.y, clipped_box.w, clipped_box.h);

        // Determine the maximum symmetric border that fits within the image
        let left_margin = bx;
        let top_margin = by;
        let right_margin = w - bx - bw;
        let bottom_margin = h - by - bh;
        let border = (max_border as i32)
            .min(left_margin)
            .min(top_margin)
            .min(right_margin)
            .min(bottom_margin)
            .max(0);

        // Expand by border (which may be 0)
        let ex = bx - border;
        let ey = by - border;
        let ew = bw + 2 * border;
        let eh = bh + 2 * border;
        let result = self.clip_rectangle(ex as u32, ey as u32, ew as u32, eh as u32)?;
        let out_box = Box::new(ex, ey, ew, eh)?;
        Ok((result, out_box))
    }

    /// Crop two images to their overlapping region so they have the same size.
    ///
    /// Both images are cropped to the minimum of their widths and heights,
    /// taken from the upper-left corner.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCropToMatch()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `other` - The second image to match sizes with
    ///
    /// # Errors
    ///
    /// Returns an error if either image has zero dimensions after cropping.
    pub fn crop_to_match(&self, other: &Pix) -> Result<(Pix, Pix)> {
        let w = self.width().min(other.width());
        let h = self.height().min(other.height());

        let r1 = if self.width() == w && self.height() == h {
            self.deep_clone()
        } else {
            self.clip_rectangle(0, 0, w, h)?
        };

        let r2 = if other.width() == w && other.height() == h {
            other.deep_clone()
        } else {
            other.clip_rectangle(0, 0, w, h)?
        };

        Ok((r1, r2))
    }

    /// Clip the image to the bounding box of its foreground pixels.
    ///
    /// Only works on 1bpp images. Foreground pixels have value 1.
    /// Returns `None` if no foreground pixels are found.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClipToForeground()` in `pix5.c`
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1bpp.
    pub fn clip_to_foreground(&self) -> Result<Option<(Pix, Box)>> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let wpl = self.wpl();

        // Mask for the last word in each row to exclude padding bits
        let leftover = w % 32;
        let end_mask: u32 = if leftover == 0 {
            0xFFFF_FFFF
        } else {
            !0u32 << (32 - leftover)
        };

        // Check if a row has any foreground pixels (excluding padding bits)
        let row_has_fg = |row: &[u32]| -> bool {
            if row.is_empty() {
                return false;
            }
            let last = row.len() - 1;
            for (i, &word) in row.iter().enumerate() {
                let masked = if i == last { word & end_mask } else { word };
                if masked != 0 {
                    return true;
                }
            }
            false
        };

        // Find top edge (first row with any foreground)
        let mut miny = None;
        for y in 0..h {
            if row_has_fg(self.row_data(y)) {
                miny = Some(y);
                break;
            }
        }

        let miny = match miny {
            Some(y) => y,
            None => return Ok(None), // no foreground
        };

        // Find bottom edge (last row with any foreground)
        let mut maxy = miny;
        for y in (miny..h).rev() {
            if row_has_fg(self.row_data(y)) {
                maxy = y;
                break;
            }
        }

        // Find left edge (minimum x with foreground across rows miny..=maxy)
        let mut minx = w;
        let data = self.data();
        for y in miny..=maxy {
            let row_start = (y * wpl) as usize;
            for x in 0..minx {
                let word_idx = (x / 32) as usize;
                let bit_pos = 31 - (x % 32);
                if (data[row_start + word_idx] >> bit_pos) & 1 != 0 {
                    minx = x;
                    break;
                }
            }
            if minx == 0 {
                break;
            }
        }

        // Find right edge (maximum x with foreground across rows miny..=maxy)
        let mut maxx = minx;
        for y in miny..=maxy {
            let row_start = (y * wpl) as usize;
            for x in (maxx..w).rev() {
                let word_idx = (x / 32) as usize;
                let bit_pos = 31 - (x % 32);
                if (data[row_start + word_idx] >> bit_pos) & 1 != 0 {
                    if x > maxx {
                        maxx = x;
                    }
                    break;
                }
            }
        }

        let bbox = Box::new(
            minx as i32,
            miny as i32,
            (maxx - minx + 1) as i32,
            (maxy - miny + 1) as i32,
        )?;

        let clipped = self.clip_rectangle(minx, miny, maxx - minx + 1, maxy - miny + 1)?;

        Ok(Some((clipped, bbox)))
    }

    /// Scan from the specified direction to find the first foreground pixel.
    ///
    /// Only works on 1bpp images. Scans within the given region.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixScanForForeground()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `region` - The region to scan within
    /// * `direction` - The direction to scan from
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1bpp or no foreground is found.
    pub fn scan_for_foreground(&self, region: &Box, direction: ScanDirection) -> Result<u32> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();

        // Clip box to image bounds
        let clipped = region.clip(w as i32, h as i32).ok_or_else(|| {
            Error::InvalidParameter("scan region does not intersect image".to_string())
        })?;

        let bx = clipped.x as u32;
        let by = clipped.y as u32;
        let bw = clipped.w as u32;
        let bh = clipped.h as u32;

        let data = self.data();
        let wpl = self.wpl();

        match direction {
            ScanDirection::FromLeft => {
                for x in bx..bx + bw {
                    let word_idx = (x / 32) as usize;
                    let bit_pos = 31 - (x % 32);
                    for y in by..by + bh {
                        let row_start = (y * wpl) as usize;
                        if (data[row_start + word_idx] >> bit_pos) & 1 != 0 {
                            return Ok(x);
                        }
                    }
                }
            }
            ScanDirection::FromRight => {
                for x in (bx..bx + bw).rev() {
                    let word_idx = (x / 32) as usize;
                    let bit_pos = 31 - (x % 32);
                    for y in by..by + bh {
                        let row_start = (y * wpl) as usize;
                        if (data[row_start + word_idx] >> bit_pos) & 1 != 0 {
                            return Ok(x);
                        }
                    }
                }
            }
            ScanDirection::FromTop => {
                for y in by..by + bh {
                    let row_start = (y * wpl) as usize;
                    for x in bx..bx + bw {
                        let word_idx = (x / 32) as usize;
                        let bit_pos = 31 - (x % 32);
                        if (data[row_start + word_idx] >> bit_pos) & 1 != 0 {
                            return Ok(y);
                        }
                    }
                }
            }
            ScanDirection::FromBot => {
                for y in (by..by + bh).rev() {
                    let row_start = (y * wpl) as usize;
                    for x in bx..bx + bw {
                        let word_idx = (x / 32) as usize;
                        let bit_pos = 31 - (x % 32);
                        if (data[row_start + word_idx] >> bit_pos) & 1 != 0 {
                            return Ok(y);
                        }
                    }
                }
            }
        }

        Err(Error::InvalidParameter(
            "no foreground pixel found in scan region".to_string(),
        ))
    }

    /// Clip a box to the foreground region of a 1bpp image.
    ///
    /// If `input_box` is `None`, the entire image is used. Returns the
    /// clipped image and its bounding box. Returns `None` if no foreground
    /// is found in the region.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClipBoxToForeground()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `input_box` - Optional region to search within; `None` for entire image
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1bpp.
    pub fn clip_box_to_foreground(&self, input_box: Option<&Box>) -> Result<Option<(Pix, Box)>> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let search_box = match input_box {
            Some(b) => {
                // Validate that the box intersects the image
                b.clip(w, h).ok_or_else(|| {
                    Error::InvalidParameter("input box does not intersect image".to_string())
                })?
            }
            None => Box::new(0, 0, w, h)?,
        };

        // Scan from all four directions.
        // "No foreground found" maps to Ok(None); other errors propagate.
        let minx = match self.scan_for_foreground(&search_box, ScanDirection::FromLeft) {
            Ok(v) => v,
            Err(Error::InvalidParameter(msg)) if msg.contains("no foreground") => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        };
        let maxx = self.scan_for_foreground(&search_box, ScanDirection::FromRight)?;
        let miny = self.scan_for_foreground(&search_box, ScanDirection::FromTop)?;
        let maxy = self.scan_for_foreground(&search_box, ScanDirection::FromBot)?;

        let bbox = Box::new(
            minx as i32,
            miny as i32,
            (maxx - minx + 1) as i32,
            (maxy - miny + 1) as i32,
        )?;

        let clipped = self.clip_rectangle(minx, miny, maxx - minx + 1, maxy - miny + 1)?;

        Ok(Some((clipped, bbox)))
    }

    /// Create a 1bpp frame mask with an annular ring of ON pixels.
    ///
    /// The mask has a rectangular ring of foreground (ON) pixels,
    /// with inner and outer boundaries specified as fractions of
    /// the image dimensions.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixMakeFrameMask()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `w` - Width of the output mask
    /// * `h` - Height of the output mask
    /// * `hf1` - Horizontal fraction for outer left/right boundary
    /// * `hf2` - Horizontal fraction for inner left/right boundary
    /// * `vf1` - Vertical fraction for outer top/bottom boundary
    /// * `vf2` - Vertical fraction for inner top/bottom boundary
    ///
    /// # Errors
    ///
    /// Returns an error if any fraction is not in [0.0, 1.0] or
    /// if outer fractions exceed inner fractions.
    pub fn make_frame_mask(w: u32, h: u32, hf1: f32, hf2: f32, vf1: f32, vf2: f32) -> Result<Pix> {
        if !(0.0..=1.0).contains(&hf1)
            || !(0.0..=1.0).contains(&hf2)
            || !(0.0..=1.0).contains(&vf1)
            || !(0.0..=1.0).contains(&vf2)
        {
            return Err(Error::InvalidParameter(
                "fractions must be in [0.0, 1.0]".to_string(),
            ));
        }
        if hf1 > hf2 || vf1 > vf2 {
            return Err(Error::InvalidParameter(
                "outer fractions must not exceed inner fractions".to_string(),
            ));
        }

        let pix = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut pm = pix.try_into_mut().unwrap();

        // Outer boundary (in pixels from edge)
        let h1 = (0.5 * hf1 * w as f32) as u32;
        let v1 = (0.5 * vf1 * h as f32) as u32;
        // Inner boundary (in pixels from edge)
        let h2 = (0.5 * hf2 * w as f32) as u32;
        let v2 = (0.5 * vf2 * h as f32) as u32;

        // Fill the outer rectangle
        let outer_x = h1;
        let outer_y = v1;
        let outer_w = w.saturating_sub(2 * h1);
        let outer_h = h.saturating_sub(2 * v1);

        for y in outer_y..outer_y + outer_h {
            for x in outer_x..outer_x + outer_w {
                if x < w && y < h {
                    pm.set_pixel_unchecked(x, y, 1);
                }
            }
        }

        // Clear the inner rectangle (hole) if it exists
        if hf2 < 1.0 && vf2 < 1.0 {
            let inner_x = h2;
            let inner_y = v2;
            let inner_w = w.saturating_sub(2 * h2);
            let inner_h = h.saturating_sub(2 * v2);

            for y in inner_y..inner_y + inner_h {
                for x in inner_x..inner_x + inner_w {
                    if x < w && y < h {
                        pm.set_pixel_unchecked(x, y, 0);
                    }
                }
            }
        }

        Ok(pm.into())
    }

    /// Compute the fraction of foreground pixels in the source that
    /// are also foreground in the mask.
    ///
    /// Both images must be 1bpp and the same size.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixFractionFgInMask()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `mask` - The mask image (1bpp)
    ///
    /// # Errors
    ///
    /// Returns an error if either image is not 1bpp or sizes differ.
    pub fn fraction_fg_in_mask(&self, mask: &Pix) -> Result<f32> {
        if self.depth() != PixelDepth::Bit1 || mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(
                if self.depth() != PixelDepth::Bit1 {
                    self.depth().bits()
                } else {
                    mask.depth().bits()
                },
            ));
        }
        if self.width() != mask.width() || self.height() != mask.height() {
            return Err(Error::InvalidParameter(format!(
                "image sizes differ: {}x{} vs {}x{}",
                self.width(),
                self.height(),
                mask.width(),
                mask.height()
            )));
        }

        let data1 = self.data();
        let data2 = mask.data();
        let width = self.width() as usize;
        let height = self.height() as usize;

        if height == 0 {
            return Ok(0.0);
        }

        let wpl = self.wpl() as usize;
        let leftover_bits = width % 32;
        // Mask to exclude padding bits in the last word of each row.
        // In 1bpp images, the leftmost pixel is in the MSB.
        let end_mask: u32 = if leftover_bits == 0 {
            0xFFFF_FFFF
        } else {
            !0u32 << (32 - leftover_bits)
        };

        // Count foreground in self and in intersection (self AND mask),
        // excluding per-row padding bits.
        let mut count_self: u64 = 0;
        let mut count_and: u64 = 0;

        for y in 0..height {
            let row_start = y * wpl;
            for x in 0..wpl {
                let mut w1 = data1[row_start + x];
                let mut w2 = data2[row_start + x];
                if leftover_bits != 0 && x == wpl - 1 {
                    w1 &= end_mask;
                    w2 &= end_mask;
                }
                count_self += w1.count_ones() as u64;
                count_and += (w1 & w2).count_ones() as u64;
            }
        }

        if count_self == 0 {
            return Ok(0.0);
        }

        Ok(count_and as f32 / count_self as f32)
    }

    /// Compute the average pixel value along a line.
    ///
    /// Works on 1bpp and 8bpp images. The line must be strictly
    /// horizontal or vertical.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAverageOnLine()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `x1`, `y1` - One endpoint of the line
    /// * `x2`, `y2` - Other endpoint of the line
    /// * `factor` - Sampling factor (>= 1)
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 1 or 8bpp, the line
    /// is neither horizontal nor vertical, or factor < 1.
    pub fn average_on_line(&self, x1: i32, y1: i32, x2: i32, y2: i32, factor: i32) -> Result<f32> {
        let d = self.depth();
        if d != PixelDepth::Bit1 && d != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if self.has_colormap() {
            return Err(Error::NotSupported(
                "average_on_line does not support colormapped images".to_string(),
            ));
        }
        if factor < 1 {
            return Err(Error::InvalidParameter(format!(
                "factor must be >= 1, got {}",
                factor
            )));
        }
        if x1 != x2 && y1 != y2 {
            return Err(Error::InvalidParameter(
                "line must be horizontal or vertical".to_string(),
            ));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let mut sum: f64 = 0.0;
        let mut count: u32 = 0;

        if y1 == y2 {
            // Horizontal line
            let y = y1.clamp(0, h - 1) as u32;
            let xmin = x1.min(x2).clamp(0, w - 1);
            let xmax = x1.max(x2).clamp(0, w - 1);
            let mut x = xmin;
            while x <= xmax {
                sum += self.get_pixel_unchecked(x as u32, y) as f64;
                count += 1;
                x += factor;
            }
        } else {
            // Vertical line
            let x = x1.clamp(0, w - 1) as u32;
            let ymin = y1.min(y2).clamp(0, h - 1);
            let ymax = y1.max(y2).clamp(0, h - 1);
            let mut y = ymin;
            while y <= ymax {
                sum += self.get_pixel_unchecked(x, y as u32) as f64;
                count += 1;
                y += factor;
            }
        }

        if count == 0 {
            return Ok(0.0);
        }

        Ok((sum / count as f64) as f32)
    }

    /// Scan for a sharp edge within a box region of an 8bpp image.
    ///
    /// Computes the sum of pixel values along each column or row
    /// (depending on scan direction), then uses the C-style
    /// `foundmin`/`loc` algorithm to detect edges:
    ///
    /// 1. Skip positions where the sum is below `lowthresh`
    /// 2. When the sum first reaches `lowthresh`, record the position
    /// 3. When it reaches `highthresh` within `maxwidth` of the start,
    ///    return the start position
    ///
    /// # Arguments
    ///
    /// * `region` - Box defining the scan region
    /// * `lowthresh` - Low threshold for column/row pixel sum
    /// * `highthresh` - High threshold for column/row pixel sum
    /// * `maxwidth` - Maximum width of edge transition zone
    /// * `factor` - Subsampling factor for averaging (>= 1)
    /// * `direction` - Direction to scan from
    ///
    /// # See also
    ///
    /// C Leptonica: `pixScanForEdge()` in `pix5.c`
    pub fn scan_for_edge(
        &self,
        region: &Box,
        lowthresh: i32,
        highthresh: i32,
        maxwidth: i32,
        factor: i32,
        direction: ScanDirection,
    ) -> Result<u32> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if factor < 1 {
            return Err(Error::InvalidParameter(format!(
                "factor must be >= 1, got {factor}"
            )));
        }
        if lowthresh >= highthresh {
            return Err(Error::InvalidParameter(
                "lowthresh must be < highthresh".to_string(),
            ));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;
        let clipped = region.clip(w, h).ok_or_else(|| {
            Error::InvalidParameter("scan region does not intersect image".to_string())
        })?;
        let rx = clipped.x as u32;
        let ry = clipped.y as u32;
        let rw = clipped.w as u32;
        let rh = clipped.h as u32;
        let factor = factor as u32;

        match direction {
            ScanDirection::FromLeft => {
                let mut foundmin = false;
                let mut loc = 0u32;
                for x in rx..rx + rw {
                    let avg = self.column_sum_in_region(x, ry, rh, factor);
                    if !foundmin && avg < lowthresh {
                        continue;
                    }
                    if !foundmin {
                        foundmin = true;
                        loc = x;
                    }
                    if avg >= highthresh {
                        if (x - loc) <= maxwidth as u32 {
                            return Ok(loc);
                        }
                        return Err(Error::InvalidParameter(
                            "edge transition too wide".to_string(),
                        ));
                    }
                }
            }
            ScanDirection::FromRight => {
                let mut foundmin = false;
                let mut loc = 0u32;
                for x in (rx..rx + rw).rev() {
                    let avg = self.column_sum_in_region(x, ry, rh, factor);
                    if !foundmin && avg < lowthresh {
                        continue;
                    }
                    if !foundmin {
                        foundmin = true;
                        loc = x;
                    }
                    if avg >= highthresh {
                        if (loc - x) <= maxwidth as u32 {
                            return Ok(loc);
                        }
                        return Err(Error::InvalidParameter(
                            "edge transition too wide".to_string(),
                        ));
                    }
                }
            }
            ScanDirection::FromTop => {
                let mut foundmin = false;
                let mut loc = 0u32;
                for y in ry..ry + rh {
                    let avg = self.row_sum_in_region(y, rx, rw, factor);
                    if !foundmin && avg < lowthresh {
                        continue;
                    }
                    if !foundmin {
                        foundmin = true;
                        loc = y;
                    }
                    if avg >= highthresh {
                        if (y - loc) <= maxwidth as u32 {
                            return Ok(loc);
                        }
                        return Err(Error::InvalidParameter(
                            "edge transition too wide".to_string(),
                        ));
                    }
                }
            }
            ScanDirection::FromBot => {
                let mut foundmin = false;
                let mut loc = 0u32;
                for y in (ry..ry + rh).rev() {
                    let avg = self.row_sum_in_region(y, rx, rw, factor);
                    if !foundmin && avg < lowthresh {
                        continue;
                    }
                    if !foundmin {
                        foundmin = true;
                        loc = y;
                    }
                    if avg >= highthresh {
                        if (loc - y) <= maxwidth as u32 {
                            return Ok(loc);
                        }
                        return Err(Error::InvalidParameter(
                            "edge transition too wide".to_string(),
                        ));
                    }
                }
            }
        }

        Err(Error::InvalidParameter(
            "no edge found in scan region".to_string(),
        ))
    }

    /// Clip a box to the edges of content in an 8bpp image.
    ///
    /// Iteratively scans from all four sides using [`scan_for_edge`]
    /// to find sharp edges and narrows the box to the content area.
    ///
    /// # Arguments
    ///
    /// * `input_box` - Initial bounding box
    /// * `lowthresh` - Low threshold for edge detection
    /// * `highthresh` - High threshold for edge detection
    /// * `maxwidth` - Maximum width for edge transition
    /// * `factor` - Subsampling factor
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClipBoxToEdges()` in `pix5.c`
    pub fn clip_box_to_edges(
        &self,
        input_box: &Box,
        lowthresh: i32,
        highthresh: i32,
        maxwidth: i32,
        factor: i32,
    ) -> Result<(Pix, Box)> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;
        let clipped = input_box.clip(w, h).ok_or_else(|| {
            Error::InvalidParameter("input box does not intersect image".to_string())
        })?;

        let mut bx = clipped.x;
        let mut by = clipped.y;
        let mut bw = clipped.w;
        let mut bh = clipped.h;

        let mut lfound = false;
        let mut rfound = false;
        let mut tfound = false;
        let mut bfound = false;
        let mut left = bx;
        let mut right = bx + bw - 1;
        let mut top = by;
        let mut bottom = by + bh - 1;

        while !lfound || !rfound || !tfound || !bfound {
            let mut change = false;

            if !lfound
                && let Ok(loc) = self.scan_for_edge(
                    &Box::new(bx, by, bw, bh)?,
                    lowthresh,
                    highthresh,
                    maxwidth,
                    factor,
                    ScanDirection::FromLeft,
                )
            {
                lfound = true;
                change = true;
                left = loc as i32;
                bw -= left - bx;
                bx = left;
            }
            if !rfound
                && let Ok(loc) = self.scan_for_edge(
                    &Box::new(bx, by, bw, bh)?,
                    lowthresh,
                    highthresh,
                    maxwidth,
                    factor,
                    ScanDirection::FromRight,
                )
            {
                rfound = true;
                change = true;
                right = loc as i32;
                bw = right - bx + 1;
            }
            if !tfound
                && let Ok(loc) = self.scan_for_edge(
                    &Box::new(bx, by, bw, bh)?,
                    lowthresh,
                    highthresh,
                    maxwidth,
                    factor,
                    ScanDirection::FromTop,
                )
            {
                tfound = true;
                change = true;
                top = loc as i32;
                bh -= top - by;
                by = top;
            }
            if !bfound
                && let Ok(loc) = self.scan_for_edge(
                    &Box::new(bx, by, bw, bh)?,
                    lowthresh,
                    highthresh,
                    maxwidth,
                    factor,
                    ScanDirection::FromBot,
                )
            {
                bfound = true;
                change = true;
                bottom = loc as i32;
                bh = bottom - by + 1;
            }

            if !change {
                break;
            }
        }

        if !lfound || !rfound || !tfound || !bfound {
            return Err(Error::InvalidParameter("not all edges found".to_string()));
        }

        let result_box = Box::new(left, top, right - left + 1, bottom - top + 1)?;
        let clipped_pix = self.clip_rectangle(
            left as u32,
            top as u32,
            (right - left + 1) as u32,
            (bottom - top + 1) as u32,
        )?;

        Ok((clipped_pix, result_box))
    }

    /// Clip a source image using a 1bpp mask.
    ///
    /// Extracts the region under the mask and fills unmasked pixels
    /// with the specified output value.
    ///
    /// # Arguments
    ///
    /// * `mask` - 1bpp mask image
    /// * `x`, `y` - Position of mask on source image
    /// * `outval` - Value for pixels outside the mask
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClipMasked()` in `pix5.c`
    pub fn clip_masked(&self, mask: &Pix, x: i32, y: i32, outval: u32) -> Result<Pix> {
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }

        let wm = mask.width();
        let hm = mask.height();

        // Compute the offset into the mask when the position is negative
        let mask_ox = (-x).max(0) as u32;
        let mask_oy = (-y).max(0) as u32;

        // Clip source at mask position (clamped to image bounds)
        let sx = x.max(0) as u32;
        let sy = y.max(0) as u32;
        let clip_w = wm.saturating_sub(mask_ox);
        let clip_h = hm.saturating_sub(mask_oy);
        let pixd = self.clip_rectangle(sx, sy, clip_w, clip_h)?;
        let mut pm = pixd.try_into_mut().unwrap();
        let dw = pm.width();
        let dh = pm.height();

        // Paint outval through pixels NOT in the mask
        for dy in 0..dh {
            for dx in 0..dw {
                let bit = mask.get_pixel_unchecked(dx + mask_ox, dy + mask_oy);
                if bit == 0 {
                    pm.set_pixel_unchecked(dx, dy, outval);
                }
            }
        }

        Ok(pm.into())
    }

    /// Create a 1bpp mask with horizontal and vertical symmetry.
    ///
    /// Generates either a filled inner rectangle or an outer frame mask
    /// by delegating to [`make_frame_mask`].
    ///
    /// - Inner (`inner = true`): solid rectangle in the center, size
    ///   controlled by `hf` and `vf`
    /// - Outer (`inner = false`): frame (annular ring) around the edges
    ///
    /// # Arguments
    ///
    /// * `w`, `h` - Dimensions of the mask
    /// * `hf` - Horizontal fraction (0.0-1.0) defining the mask boundary
    /// * `vf` - Vertical fraction (0.0-1.0) defining the mask boundary
    /// * `inner` - If true, creates solid inner rectangle; if false, outer frame
    ///
    /// # See also
    ///
    /// C Leptonica: `pixMakeSymmetricMask()` in `pix5.c`
    pub fn make_symmetric_mask(w: u32, h: u32, hf: f32, vf: f32, inner: bool) -> Result<Pix> {
        if !(0.0..=1.0).contains(&hf) {
            return Err(Error::InvalidParameter(
                "hf must be in [0.0, 1.0]".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&vf) {
            return Err(Error::InvalidParameter(
                "vf must be in [0.0, 1.0]".to_string(),
            ));
        }

        if inner {
            Self::make_frame_mask(w, h, hf, 1.0, vf, 1.0)
        } else {
            Self::make_frame_mask(w, h, 0.0, hf, 0.0, vf)
        }
    }

    /// Compute average foreground and background values by thresholding.
    ///
    /// Pixels with values below `thresh` are considered foreground;
    /// pixels at or above `thresh` are background. Returns the average
    /// value of each group.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (>= 1)
    /// * `thresh` - Threshold value for fg/bg separation
    ///
    /// # See also
    ///
    /// C Leptonica: `pixThresholdForFgBg()` in `pix4.c`
    pub fn threshold_for_fg_bg(&self, factor: u32, thresh: u32) -> Result<(u32, u32)> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let factor = factor.max(1);

        let w = self.width();
        let h = self.height();
        let mut fg_sum: u64 = 0;
        let mut fg_count: u64 = 0;
        let mut bg_sum: u64 = 0;
        let mut bg_count: u64 = 0;

        let mut y = 0u32;
        while y < h {
            let mut x = 0u32;
            while x < w {
                let val = self.get_pixel_unchecked(x, y);
                if val < thresh {
                    fg_sum += val as u64;
                    fg_count += 1;
                } else {
                    bg_sum += val as u64;
                    bg_count += 1;
                }
                x += factor;
            }
            y += factor;
        }

        let fg_val = if fg_count > 0 {
            ((fg_sum as f64 / fg_count as f64) + 0.5) as u32
        } else {
            0
        };
        let bg_val = if bg_count > 0 {
            ((bg_sum as f64 / bg_count as f64) + 0.5) as u32
        } else {
            0
        };

        Ok((fg_val, bg_val))
    }

    /// Clip an image to each box in a `Boxa`, returning a `Pixa`.
    ///
    /// Each element of the returned `Pixa` is the sub-image clipped by the
    /// corresponding box, and the associated box is the actual clipped region.
    /// Boxes that do not intersect the image are skipped.
    ///
    /// C equivalent: `pixClipRectangles()` in `pix5.c`
    pub fn clip_rectangles(&self, boxes: &crate::Boxa) -> Result<crate::pixa::Pixa> {
        let mut pixa = crate::pixa::Pixa::new();
        let w = self.width() as i32;
        let h = self.height() as i32;
        for i in 0..boxes.len() {
            let b = boxes
                .get(i)
                .ok_or_else(|| Error::InvalidParameter("box index out of range".into()))?;
            let x0 = b.x.max(0);
            let y0 = b.y.max(0);
            let x1 = (b.x + b.w).min(w);
            let y1 = (b.y + b.h).min(h);
            if x1 <= x0 || y1 <= y0 {
                continue;
            }
            let clipped =
                self.clip_rectangle(x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32)?;
            let box_c = Box::new(x0, y0, x1 - x0, y1 - y0)
                .map_err(|e| Error::InvalidParameter(format!("invalid clipped box: {e}")))?;
            pixa.push_with_box(clipped, box_c);
        }
        Ok(pixa)
    }

    /// Crop to at most `w × h`, returning a clone if already within bounds.
    ///
    /// If either dimension is already ≤ the limit, that dimension is not cropped.
    ///
    /// C equivalent: `pixCropToSize()` in `pix5.c`
    pub fn crop_to_size(&self, w: u32, h: u32) -> Result<Pix> {
        let ws = self.width();
        let hs = self.height();
        if ws <= w && hs <= h {
            return Ok(self.clone());
        }
        let wd = ws.min(w);
        let hd = hs.min(h);
        self.clip_rectangle(0, 0, wd, hd)
    }

    /// Resize to exactly `w × h` without scaling.
    ///
    /// Crops if larger; replicates the last row/column if smaller.
    /// If `pixt` is provided its dimensions override `w` and `h`.
    ///
    /// C equivalent: `pixResizeToMatch()` in `pix5.c`
    pub fn resize_to_match(&self, pixt: Option<&Pix>, w: u32, h: u32) -> Result<Pix> {
        let (tw, th) = if let Some(t) = pixt {
            (t.width(), t.height())
        } else {
            if w == 0 || h == 0 {
                return Err(Error::InvalidParameter("w and h must be > 0".into()));
            }
            (w, h)
        };
        let ws = self.width();
        let hs = self.height();
        if ws == tw && hs == th {
            return Ok(self.clone());
        }
        let depth = self.depth();
        let pixd_base = Pix::new(tw, th, depth)
            .map_err(|e| Error::InvalidParameter(format!("cannot create pixd: {e}")))?;
        let mut pixd = pixd_base.try_into_mut().unwrap();
        pixd.set_resolution(self.xres(), self.yres());
        if depth == PixelDepth::Bit32 {
            pixd.set_spp(self.spp());
        }
        if self.colormap().is_some() {
            pixd.copy_colormap_from(self);
        }
        // Copy source pixels (clipped to min dimensions)
        let copy_w = ws.min(tw);
        let copy_h = hs.min(th);
        for y in 0..copy_h {
            for x in 0..copy_w {
                let val = self.get_pixel_unchecked(x, y);
                pixd.set_pixel_unchecked(x, y, val);
            }
        }
        // Replicate last column if width needs expanding
        if ws < tw {
            for y in 0..copy_h {
                let last_val = self.get_pixel_unchecked(ws - 1, y);
                for x in ws..tw {
                    pixd.set_pixel_unchecked(x, y, last_val);
                }
            }
        }
        // Replicate last row if height needs expanding
        if hs < th {
            for i in hs..th {
                for x in 0..tw {
                    let val = pixd.get_pixel_unchecked(x, hs - 1);
                    pixd.set_pixel_unchecked(x, i, val);
                }
            }
        }
        Ok(pixd.into())
    }

    /// Test whether the image can be further clipped without losing foreground.
    ///
    /// Returns `true` if at least one edge has no foreground pixels (i.e. a
    /// crop would not remove any ON pixels). Returns `false` if fg touches
    /// all four edges.
    ///
    /// Only valid for 1bpp images.
    ///
    /// C equivalent: `pixTestClipToForeground()` in `pix5.c`
    pub fn test_clip_to_foreground(&self) -> Result<bool> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width();
        let h = self.height();
        // top row
        if !(0..w).any(|x| self.get_pixel_unchecked(x, 0) != 0) {
            return Ok(true);
        }
        // bottom row
        if !(0..w).any(|x| self.get_pixel_unchecked(x, h - 1) != 0) {
            return Ok(true);
        }
        // left column
        if !(0..h).any(|y| self.get_pixel_unchecked(0, y) != 0) {
            return Ok(true);
        }
        // right column
        if !(0..h).any(|y| self.get_pixel_unchecked(w - 1, y) != 0) {
            return Ok(true);
        }
        Ok(false)
    }

    /// Compute the sum of pixel values along a column within a region.
    ///
    /// For 8bpp images, the maximum sum is 255 * height. With typical
    /// image dimensions (height <= 8_000_000), this fits within i32.
    fn column_sum_in_region(&self, x: u32, y_start: u32, height: u32, factor: u32) -> i32 {
        let mut sum = 0i64;
        let mut y = y_start;
        while y < y_start + height {
            sum += self.get_pixel_unchecked(x, y) as i64;
            y += factor;
        }
        sum as i32
    }

    /// Compute the sum of pixel values along a row within a region.
    ///
    /// For 8bpp images, the maximum sum is 255 * width. With typical
    /// image dimensions (width <= 8_000_000), this fits within i32.
    fn row_sum_in_region(&self, y: u32, x_start: u32, width: u32, factor: u32) -> i32 {
        let mut sum = 0i64;
        let mut x = x_start;
        while x < x_start + width {
            sum += self.get_pixel_unchecked(x, y) as i64;
            x += factor;
        }
        sum as i32
    }
}

#[cfg(test)]
mod tests {
    use super::super::PixelDepth;
    use super::*;

    #[test]
    fn test_clip_rectangle_basic() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel(50, 40, 128).unwrap();
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(40, 30, 20, 20).unwrap();
        assert_eq!(clipped.width(), 20);
        assert_eq!(clipped.height(), 20);
        // Original pixel at (50,40) should now be at (10,10) in clipped image
        assert_eq!(clipped.get_pixel(10, 10), Some(128));
    }

    #[test]
    fn test_clip_rectangle_full_image() {
        let pix = Pix::new(50, 30, PixelDepth::Bit8).unwrap();
        let clipped = pix.clip_rectangle(0, 0, 50, 30).unwrap();
        assert_eq!(clipped.width(), 50);
        assert_eq!(clipped.height(), 30);
    }

    #[test]
    fn test_clip_rectangle_clips_to_bounds() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        // Request extends beyond the right and bottom edges
        let clipped = pix.clip_rectangle(80, 60, 50, 50).unwrap();
        assert_eq!(clipped.width(), 20); // 100 - 80
        assert_eq!(clipped.height(), 20); // 80 - 60
    }

    #[test]
    fn test_clip_rectangle_entirely_outside() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        assert!(pix.clip_rectangle(100, 0, 10, 10).is_err());
        assert!(pix.clip_rectangle(0, 80, 10, 10).is_err());
        assert!(pix.clip_rectangle(200, 200, 10, 10).is_err());
    }

    #[test]
    fn test_clip_rectangle_zero_size() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        assert!(pix.clip_rectangle(0, 0, 0, 10).is_err());
        assert!(pix.clip_rectangle(0, 0, 10, 0).is_err());
    }

    #[test]
    fn test_clip_rectangle_1bpp() {
        let pix = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel(32, 32, 1).unwrap();
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(20, 20, 30, 30).unwrap();
        assert_eq!(clipped.width(), 30);
        assert_eq!(clipped.height(), 30);
        // Pixel at (32,32) in source -> (12,12) in clipped
        assert_eq!(clipped.get_pixel(12, 12), Some(1));
        assert_eq!(clipped.get_pixel(0, 0), Some(0));
    }

    #[test]
    fn test_clip_rectangle_32bpp() {
        use crate::color::compose_rgb;

        let pix = Pix::new(100, 80, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut
            .set_pixel(50, 40, compose_rgb(200, 100, 50))
            .unwrap();
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(40, 30, 20, 20).unwrap();
        assert_eq!(clipped.width(), 20);
        assert_eq!(clipped.height(), 20);
        assert_eq!(clipped.depth(), PixelDepth::Bit32);
        assert_eq!(clipped.spp(), pix.spp());

        let (r, g, b) = clipped.get_rgb(10, 10).unwrap();
        assert_eq!((r, g, b), (200, 100, 50));
    }

    #[test]
    fn test_clip_rectangle_preserves_resolution() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_resolution(300, 300);
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(10, 10, 50, 50).unwrap();
        assert_eq!(clipped.xres(), 300);
        assert_eq!(clipped.yres(), 300);
    }

    #[test]
    fn test_clip_rectangle_pixel_values() {
        // Verify that all pixels are correctly copied
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..20u32 {
            for x in 0..20u32 {
                pix_mut.set_pixel(x, y, (x + y * 20) % 256).unwrap();
            }
        }
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(5, 5, 10, 10).unwrap();
        for y in 0..10u32 {
            for x in 0..10u32 {
                let expected = ((x + 5) + (y + 5) * 20) % 256;
                assert_eq!(clipped.get_pixel(x, y), Some(expected));
            }
        }
    }

    // -- Pix::clip_rectangles --

    #[test]
    fn test_clip_rectangles_basic() {
        use crate::Boxa;
        use crate::box_::Box as LepBox;
        let pix = {
            let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for y in 0..10u32 {
                for x in 0..10u32 {
                    pm.set_pixel_unchecked(x, y, x + y * 10);
                }
            }
            Pix::from(pm)
        };
        let mut boxa = Boxa::new();
        boxa.push(LepBox::new(0, 0, 4, 4).unwrap());
        boxa.push(LepBox::new(5, 5, 5, 5).unwrap());
        let pixa = pix.clip_rectangles(&boxa).unwrap();
        assert_eq!(pixa.len(), 2);
        assert_eq!(pixa.get(0).unwrap().width(), 4);
        assert_eq!(pixa.get(0).unwrap().height(), 4);
        assert_eq!(pixa.get(1).unwrap().width(), 5);
        assert_eq!(pixa.get(1).unwrap().height(), 5);
    }

    // -- Pix::crop_to_size --

    #[test]
    fn test_crop_to_size_smaller() {
        let pix = {
            let base = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            pm.set_pixel_unchecked(5, 5, 200);
            Pix::from(pm)
        };
        let cropped = pix.crop_to_size(10, 10).unwrap();
        assert_eq!(cropped.width(), 10);
        assert_eq!(cropped.height(), 10);
        assert_eq!(cropped.get_pixel(5, 5), Some(200));
    }

    #[test]
    fn test_crop_to_size_already_fits() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let result = pix.crop_to_size(10, 10).unwrap();
        // no cropping needed - should return same size
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    // -- Pix::resize_to_match --

    #[test]
    fn test_resize_to_match_crop() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let resized = pix.resize_to_match(None, 10, 10).unwrap();
        assert_eq!(resized.width(), 10);
        assert_eq!(resized.height(), 10);
    }

    #[test]
    fn test_resize_to_match_expand() {
        // Expand by replicating last row/col
        let pix = {
            let base = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for y in 0..4u32 {
                pm.set_pixel_unchecked(3, y, 99); // last column = 99
            }
            for x in 0..4u32 {
                pm.set_pixel_unchecked(x, 3, 77); // last row = 77
            }
            Pix::from(pm)
        };
        let resized = pix.resize_to_match(None, 6, 6).unwrap();
        assert_eq!(resized.width(), 6);
        assert_eq!(resized.height(), 6);
        // Expanded columns should replicate last column value
        assert_eq!(resized.get_pixel(4, 0), Some(99));
        assert_eq!(resized.get_pixel(5, 0), Some(99));
        // Expanded rows should replicate last row value
        assert_eq!(resized.get_pixel(0, 4), Some(77));
        assert_eq!(resized.get_pixel(0, 5), Some(77));
    }

    // -- Pix::test_clip_to_foreground --

    #[test]
    fn test_test_clip_to_foreground_can_clip() {
        // Image with fg only in center - can be clipped
        let pix = {
            let base = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            pm.set_pixel_unchecked(5, 5, 1);
            Pix::from(pm)
        };
        assert!(pix.test_clip_to_foreground().unwrap());
    }

    #[test]
    fn test_test_clip_to_foreground_cannot_clip() {
        // Image with fg touching all 4 edges - cannot be clipped
        let pix = {
            let base = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            pm.set_pixel_unchecked(0, 5, 1); // left edge
            pm.set_pixel_unchecked(9, 5, 1); // right edge
            pm.set_pixel_unchecked(5, 0, 1); // top edge
            pm.set_pixel_unchecked(5, 9, 1); // bottom edge
            Pix::from(pm)
        };
        assert!(!pix.test_clip_to_foreground().unwrap());
    }
}
