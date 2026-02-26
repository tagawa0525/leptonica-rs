//! Image tiling for processing large images in pieces
//!
//! `PixTiling` provides a way to split a large image into overlapping or
//! non-overlapping tiles, process each tile independently, and reassemble
//! the results. This is essential for memory-efficient processing of
//! very large images.
//!
//! # Reference
//!
//! Based on Leptonica's `pixtiling.c`.

use crate::core::{Error, Pix, PixMut, Result};

/// Image tiling configuration for tile-based processing.
///
/// Tiles can optionally overlap to avoid boundary artifacts. When painting
/// tiles back, the overlap regions are stripped by default.
pub struct PixTiling {
    /// Source image (cloned reference)
    pix: Pix,
    /// Number of tiles horizontally
    nx: u32,
    /// Number of tiles vertically
    ny: u32,
    /// Tile width (without overlap)
    w: u32,
    /// Tile height (without overlap)
    h: u32,
    /// Horizontal overlap in pixels
    xoverlap: u32,
    /// Vertical overlap in pixels
    yoverlap: u32,
    /// Whether to strip overlap when painting (default: true)
    strip: bool,
}

impl PixTiling {
    /// Create a tiling configuration for the given image.
    ///
    /// Specify either tile counts (nx, ny) or tile sizes (w, h). If both
    /// nx/ny and w/h are non-zero, tile counts take precedence.
    /// If nx or ny is 0 and w or h is also 0, the full dimension is used.
    ///
    /// # Arguments
    ///
    /// * `pix` - Source image
    /// * `nx` - Number of horizontal tiles (0 to compute from w)
    /// * `ny` - Number of vertical tiles (0 to compute from h)
    /// * `w` - Tile width (0 to compute from nx)
    /// * `h` - Tile height (0 to compute from ny)
    /// * `xoverlap` - Horizontal overlap in pixels
    /// * `yoverlap` - Vertical overlap in pixels
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixTilingCreate()`
    pub fn create(
        pix: &Pix,
        nx: u32,
        ny: u32,
        w: u32,
        h: u32,
        xoverlap: u32,
        yoverlap: u32,
    ) -> Result<Self> {
        let img_w = pix.width();
        let img_h = pix.height();

        // Determine tile counts and sizes
        let (nx, tw) = if nx > 0 {
            let tw = img_w.div_ceil(nx);
            (nx, tw)
        } else if w > 0 {
            let nx = img_w.div_ceil(w);
            (nx, w)
        } else {
            (1, img_w)
        };

        let (ny, th) = if ny > 0 {
            let th = img_h.div_ceil(ny);
            (ny, th)
        } else if h > 0 {
            let ny = img_h.div_ceil(h);
            (ny, h)
        } else {
            (1, img_h)
        };

        if nx == 0 || ny == 0 {
            return Err(Error::InvalidParameter(
                "tiling must have at least 1 tile in each dimension".to_string(),
            ));
        }

        Ok(Self {
            pix: pix.clone(),
            nx,
            ny,
            w: tw,
            h: th,
            xoverlap,
            yoverlap,
            strip: true,
        })
    }

    /// Get the number of tiles in each dimension.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixTilingGetCount()`
    pub fn get_count(&self) -> (u32, u32) {
        (self.nx, self.ny)
    }

    /// Get the tile dimensions (without overlap).
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixTilingGetSize()`
    pub fn get_size(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    /// Disable overlap stripping when painting tiles back.
    pub fn no_strip_on_paint(&mut self) {
        self.strip = false;
    }

    /// Extract a tile from the source image.
    ///
    /// The tile at position (i, j) is extracted with the configured overlap.
    /// Tiles at edges are clipped to the image boundary.
    ///
    /// # Arguments
    ///
    /// * `i` - Horizontal tile index (0-based)
    /// * `j` - Vertical tile index (0-based)
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixTilingGetTile()`
    pub fn get_tile(&self, i: u32, j: u32) -> Result<Pix> {
        if i >= self.nx || j >= self.ny {
            return Err(Error::IndexOutOfBounds {
                index: (j * self.nx + i) as usize,
                len: (self.nx * self.ny) as usize,
            });
        }

        let img_w = self.pix.width();
        let img_h = self.pix.height();

        // Compute top-left of this tile (with overlap)
        let x0 = (i * self.w).saturating_sub(self.xoverlap);
        let y0 = (j * self.h).saturating_sub(self.yoverlap);

        // Compute bottom-right (with overlap)
        let x1 = ((i + 1) * self.w + self.xoverlap).min(img_w);
        let y1 = ((j + 1) * self.h + self.yoverlap).min(img_h);

        let tw = x1.saturating_sub(x0);
        let th = y1.saturating_sub(y0);

        if tw == 0 || th == 0 {
            return Err(Error::InvalidParameter(
                "tile has zero dimension".to_string(),
            ));
        }

        self.pix.clip_rectangle(x0, y0, tw, th)
    }

    /// Paint a processed tile back into a destination image.
    ///
    /// If strip mode is on (default), the overlap is removed and only the
    /// core tile region is painted.
    ///
    /// # Arguments
    ///
    /// * `dst` - Destination image to paint into
    /// * `i` - Horizontal tile index
    /// * `j` - Vertical tile index
    /// * `tile` - Processed tile to paint
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixTilingPaintTile()`
    pub fn paint_tile(&self, dst: &mut PixMut, i: u32, j: u32, tile: &Pix) -> Result<()> {
        if i >= self.nx || j >= self.ny {
            return Err(Error::IndexOutOfBounds {
                index: (j * self.nx + i) as usize,
                len: (self.nx * self.ny) as usize,
            });
        }

        let dst_x = i * self.w;
        let dst_y = j * self.h;

        // Source offset within tile (strip overlap if applicable)
        let (sx, sy) = if self.strip {
            let sx = if i > 0 { self.xoverlap } else { 0 };
            let sy = if j > 0 { self.yoverlap } else { 0 };
            (sx, sy)
        } else {
            (0, 0)
        };

        let copy_w = self.w.min(dst.width().saturating_sub(dst_x));
        let copy_h = self.h.min(dst.height().saturating_sub(dst_y));

        for dy in 0..copy_h {
            for dx in 0..copy_w {
                let tx = sx + dx;
                let ty = sy + dy;
                if let Some(val) = tile.get_pixel(tx, ty) {
                    let ox = dst_x + dx;
                    let oy = dst_y + dy;
                    if ox < dst.width() && oy < dst.height() {
                        dst.set_pixel_unchecked(ox, oy, val);
                    }
                }
            }
        }

        Ok(())
    }
}
