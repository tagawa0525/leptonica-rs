//! Border tracing for connected components
//!
//! This module provides functions for tracing the borders (contours) of connected
//! components in binary images. It implements the standard contour-following algorithm
//! where foreground pixels are kept on the right side of the traversal path.
//!
//! - Outer borders are traced clockwise
//! - Hole borders are traced counter-clockwise
//!
//! # Known Limitations
//!
//! **Memory Efficiency Issue in `get_all_borders()`:**
//! The current implementation has O(n_components * image_size) memory complexity.
//! When processing images with many components (e.g., feyn-fract.tif), memory usage
//! can exceed available RAM. This is due to the Vec-based component detection
//! accumulating all components before processing, unlike the C reference implementation
//! which processes components sequentially and releases memory immediately.
//!
//! **Workaround:** For large images with many components, consider:
//! - Processing components iteratively rather than collecting all at once
//! - Or breaking the image into smaller regions before calling `get_all_borders()`
//!
//! See `docs/plans/500_region-full-porting.md` Phase 5 for improvement plan.
//!
//! # Examples
//!
//! ```
//! use leptonica_region::ccbord::{get_outer_border, BorderPoint};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! // Create a 5x5 binary image with a 3x3 square
//! let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
//! let mut pix_mut = pix.try_into_mut().unwrap();
//! for y in 1..4 {
//!     for x in 1..4 {
//!         pix_mut.set_pixel(x, y, 1).unwrap();
//!     }
//! }
//! let pix: Pix = pix_mut.into();
//!
//! // Get the outer border
//! let border = get_outer_border(&pix, None).unwrap();
//! assert!(!border.is_empty());
//! ```

use crate::conncomp::{ConnectivityType, find_connected_components};
use crate::error::{RegionError, RegionResult};
use crate::seedfill::fill_holes;
use leptonica_core::{Box, Pix, PixelDepth};
use std::io::{Read, Write};

/// Direction for chain code representation (8-connectivity)
///
/// Uses the standard 8-direction encoding where directions are numbered
/// clockwise starting from West:
/// ```text
///   1  2  3
///   0  P  4
///   7  6  5
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    /// West (-1, 0)
    West = 0,
    /// Northwest (-1, -1)
    NorthWest = 1,
    /// North (0, -1)
    North = 2,
    /// Northeast (1, -1)
    NorthEast = 3,
    /// East (1, 0)
    East = 4,
    /// Southeast (1, 1)
    SouthEast = 5,
    /// South (0, 1)
    South = 6,
    /// Southwest (-1, 1)
    SouthWest = 7,
}

impl Direction {
    /// Get the x offset for this direction
    #[inline]
    pub fn dx(self) -> i32 {
        XPOSTAB[self as usize]
    }

    /// Get the y offset for this direction
    #[inline]
    pub fn dy(self) -> i32 {
        YPOSTAB[self as usize]
    }

    /// Get direction from x,y offsets
    ///
    /// Returns None if offsets are not valid (both zero or absolute value > 1)
    pub fn from_offset(dx: i32, dy: i32) -> Option<Self> {
        if dx.abs() > 1 || dy.abs() > 1 || (dx == 0 && dy == 0) {
            return None;
        }
        let idx = DIRTAB[(1 + dy) as usize][(1 + dx) as usize];
        if idx < 0 {
            None
        } else {
            Some(Self::from_index(idx as usize))
        }
    }

    /// Create direction from index (0-7)
    #[inline]
    fn from_index(idx: usize) -> Self {
        match idx % 8 {
            0 => Direction::West,
            1 => Direction::NorthWest,
            2 => Direction::North,
            3 => Direction::NorthEast,
            4 => Direction::East,
            5 => Direction::SouthEast,
            6 => Direction::South,
            _ => Direction::SouthWest,
        }
    }

    /// Get all 8 directions in order
    pub fn all() -> [Direction; 8] {
        [
            Direction::West,
            Direction::NorthWest,
            Direction::North,
            Direction::NorthEast,
            Direction::East,
            Direction::SouthEast,
            Direction::South,
            Direction::SouthWest,
        ]
    }
}

/// X offset for each direction (indexed by direction value)
const XPOSTAB: [i32; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];

/// Y offset for each direction (indexed by direction value)
const YPOSTAB: [i32; 8] = [0, -1, -1, -1, 0, 1, 1, 1];

/// New qpos when moving to direction pos
/// This gives the starting direction for the next search after moving in direction `pos`
const QPOSTAB: [usize; 8] = [6, 6, 0, 0, 2, 2, 4, 4];

/// Direction lookup table: DIRTAB[1+dy][1+dx] gives direction index
/// -1 indicates invalid (center point)
const DIRTAB: [[i32; 3]; 3] = [[1, 2, 3], [0, -1, 4], [7, 6, 5]];

/// A point on a border
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BorderPoint {
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
}

impl BorderPoint {
    /// Create a new border point
    #[inline]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Move in the given direction
    #[inline]
    pub fn moved(self, dir: Direction) -> Self {
        Self {
            x: self.x + dir.dx(),
            y: self.y + dir.dy(),
        }
    }

    /// Add offset to create new point
    #[inline]
    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl From<(i32, i32)> for BorderPoint {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}

impl From<(u32, u32)> for BorderPoint {
    fn from((x, y): (u32, u32)) -> Self {
        Self::new(x as i32, y as i32)
    }
}

/// Border type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BorderType {
    /// Outer border (clockwise traversal, foreground on the right)
    #[default]
    Outer,
    /// Hole border (counter-clockwise traversal, foreground on the right)
    Hole,
}

/// A single border (outer or hole)
#[derive(Debug, Clone, Default)]
pub struct Border {
    /// Type of this border
    pub border_type: BorderType,
    /// Starting point of the border (in local coordinates)
    pub start: BorderPoint,
    /// All points on the border (in traversal order, local coordinates)
    pub points: Vec<BorderPoint>,
    /// Chain code representation (if computed)
    pub chain_code: Option<Vec<Direction>>,
}

impl Border {
    /// Create a new border from points
    pub fn new(border_type: BorderType, points: Vec<BorderPoint>) -> Self {
        let start = points.first().copied().unwrap_or_default();
        Self {
            border_type,
            start,
            points,
            chain_code: None,
        }
    }

    /// Create an empty border
    pub fn empty(border_type: BorderType) -> Self {
        Self {
            border_type,
            start: BorderPoint::default(),
            points: Vec::new(),
            chain_code: None,
        }
    }

    /// Get the number of points in this border
    #[inline]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if the border is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Compute and store chain code representation
    pub fn compute_chain_code(&mut self) {
        self.chain_code = Some(to_chain_code(&self.points));
    }

    /// Get chain code, computing if necessary
    pub fn get_chain_code(&mut self) -> &[Direction] {
        if self.chain_code.is_none() {
            self.compute_chain_code();
        }
        self.chain_code.as_ref().unwrap()
    }

    /// Convert points to global coordinates by adding offset
    pub fn to_global(&self, offset_x: i32, offset_y: i32) -> Border {
        Border {
            border_type: self.border_type,
            start: self.start.offset(offset_x, offset_y),
            points: self
                .points
                .iter()
                .map(|p| p.offset(offset_x, offset_y))
                .collect(),
            chain_code: self.chain_code.clone(),
        }
    }

    /// Get perimeter (number of boundary pixels)
    #[inline]
    pub fn perimeter(&self) -> usize {
        self.points.len()
    }

    /// Get bounding box of this border
    pub fn bounding_box(&self) -> Option<Box> {
        if self.points.is_empty() {
            return None;
        }

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for p in &self.points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Some(Box::new_unchecked(
            min_x,
            min_y,
            max_x - min_x + 1,
            max_y - min_y + 1,
        ))
    }
}

/// Collection of borders for a single connected component
#[derive(Debug, Clone)]
pub struct ComponentBorders {
    /// Bounding box of the component (in global coordinates)
    pub bounds: Box,
    /// The outer border (in local coordinates relative to bounds)
    pub outer: Border,
    /// Hole borders (may be empty, in local coordinates)
    pub holes: Vec<Border>,
    /// Single continuous path (outer + holes connected via cuts, in local coordinates)
    /// Computed by `ImageBorders::generate_single_path()`
    pub single_path: Option<Vec<BorderPoint>>,
}

impl ComponentBorders {
    /// Create new component borders
    pub fn new(bounds: Box, outer: Border) -> Self {
        Self {
            bounds,
            outer,
            holes: Vec::new(),
            single_path: None,
        }
    }

    /// Get total number of borders (outer + holes)
    pub fn border_count(&self) -> usize {
        1 + self.holes.len()
    }

    /// Check if component has holes
    pub fn has_holes(&self) -> bool {
        !self.holes.is_empty()
    }

    /// Get the outer border in global coordinates
    pub fn outer_global(&self) -> Border {
        self.outer.to_global(self.bounds.x, self.bounds.y)
    }

    /// Get all hole borders in global coordinates
    pub fn holes_global(&self) -> Vec<Border> {
        self.holes
            .iter()
            .map(|h| h.to_global(self.bounds.x, self.bounds.y))
            .collect()
    }

    /// Get total perimeter (outer + all holes)
    pub fn total_perimeter(&self) -> usize {
        self.outer.perimeter() + self.holes.iter().map(|h| h.perimeter()).sum::<usize>()
    }
}

/// Collection of all borders in an image
#[derive(Debug, Clone)]
pub struct ImageBorders {
    /// Width of the source image
    pub width: u32,
    /// Height of the source image
    pub height: u32,
    /// Borders for each connected component
    pub components: Vec<ComponentBorders>,
}

impl ImageBorders {
    /// Create new empty image borders
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            components: Vec::new(),
        }
    }

    /// Get total number of connected components
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Get total number of borders (all outer + all holes)
    pub fn total_border_count(&self) -> usize {
        self.components.iter().map(|c| c.border_count()).sum()
    }

    /// Check if any component has holes
    pub fn has_holes(&self) -> bool {
        self.components.iter().any(|c| c.has_holes())
    }
}

impl ImageBorders {
    /// Compute step chain codes for all borders in the image
    ///
    /// Populates `Border::chain_code` for outer and hole borders of each component.
    pub fn generate_step_chains(&mut self) {
        for comp in &mut self.components {
            comp.outer.compute_chain_code();
            for hole in &mut comp.holes {
                hole.compute_chain_code();
            }
        }
    }

    /// Reconstruct pixel coordinates from step chain codes
    ///
    /// For each border with a computed chain code, regenerates `Border::points`
    /// from the start point and chain code directions.
    pub fn step_chains_to_pix_coords(&mut self) -> RegionResult<()> {
        for comp in &mut self.components {
            if let Some(code) = &comp.outer.chain_code {
                let start = comp.outer.start;
                let restored = from_chain_code(start, code);
                comp.outer.points = restored;
            }
            for hole in &mut comp.holes {
                if let Some(code) = &hole.chain_code {
                    let start = hole.start;
                    let restored = from_chain_code(start, code);
                    hole.points = restored;
                }
            }
        }
        Ok(())
    }

    /// Generate a single continuous path for each component (for SVG export)
    ///
    /// For components without holes, copies the outer border points.
    /// For components with holes, builds a single connected path that includes
    /// the outer border and each hole border via straight-line cut paths.
    /// Result is stored in `ComponentBorders::single_path`.
    pub fn generate_single_path(&mut self) -> RegionResult<()> {
        for comp in &mut self.components {
            let mut path = Vec::new();

            if comp.holes.is_empty() {
                // No holes - just use outer border
                path = comp.outer.points.clone();
            } else {
                // With holes - build a single continuous path
                // This is a simplified implementation that concatenates borders via cuts
                path.extend(&comp.outer.points);

                for hole in &comp.holes {
                    // Connect hole to outer border via a cut path.
                    // Return to the same outer-border cut point to keep the path continuous.
                    if !hole.points.is_empty() {
                        let cut_point = path.last().copied().unwrap_or(comp.outer.points[0]);
                        path.push(hole.points[0]);
                        path.extend(&hole.points);
                        path.push(hole.points[0]);
                        path.push(cut_point);
                    }
                }
            }

            comp.single_path = Some(path);
        }
        Ok(())
    }

    /// Serialize borders to binary format
    ///
    /// Format: magic "ccba" + image dimensions + per-component step chains.
    pub fn write<W: Write>(&self, mut writer: W) -> RegionResult<()> {
        // Header: "ccba" (4 bytes) + image dimensions (8 bytes)
        writer
            .write_all(b"ccba")
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

        let width_bytes = (self.width as u32).to_le_bytes();
        let height_bytes = (self.height as u32).to_le_bytes();
        writer
            .write_all(&width_bytes)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
        writer
            .write_all(&height_bytes)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

        // Number of components
        let ncc = (self.components.len() as u32).to_le_bytes();
        writer
            .write_all(&ncc)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

        // Per-component data
        for comp in &self.components {
            let bx = (comp.bounds.x as i32).to_le_bytes();
            let by = (comp.bounds.y as i32).to_le_bytes();
            let bw = (comp.bounds.w as i32).to_le_bytes();
            let bh = (comp.bounds.h as i32).to_le_bytes();

            writer
                .write_all(&bx)
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
            writer
                .write_all(&by)
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
            writer
                .write_all(&bw)
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
            writer
                .write_all(&bh)
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

            // Number of borders (outer + holes)
            let nb: u32 = comp.border_count().try_into().map_err(|_| {
                RegionError::InvalidParameters(format!(
                    "component has too many borders to serialize: {}",
                    comp.border_count()
                ))
            })?;
            writer
                .write_all(&nb.to_le_bytes())
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

            // Outer border
            write_border(&mut writer, &comp.outer)?;

            // Hole borders
            for hole in &comp.holes {
                write_border(&mut writer, hole)?;
            }
        }

        Ok(())
    }

    /// Deserialize borders from binary format
    pub fn read_from<R: Read>(mut reader: R) -> RegionResult<Self> {
        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

        if &magic != b"ccba" {
            return Err(RegionError::InvalidParameters(
                "invalid ccba magic header".to_string(),
            ));
        }

        let mut width_bytes = [0u8; 4];
        let mut height_bytes = [0u8; 4];
        let mut ncc_bytes = [0u8; 4];

        reader
            .read_exact(&mut width_bytes)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
        reader
            .read_exact(&mut height_bytes)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
        reader
            .read_exact(&mut ncc_bytes)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

        let width = u32::from_le_bytes(width_bytes);
        let height = u32::from_le_bytes(height_bytes);
        let ncc = u32::from_le_bytes(ncc_bytes) as usize;

        let mut image_borders = ImageBorders::new(width, height);

        for _ in 0..ncc {
            let mut bx_bytes = [0u8; 4];
            let mut by_bytes = [0u8; 4];
            let mut bw_bytes = [0u8; 4];
            let mut bh_bytes = [0u8; 4];
            let mut nb_bytes = [0u8; 4];

            reader
                .read_exact(&mut bx_bytes)
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
            reader
                .read_exact(&mut by_bytes)
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
            reader
                .read_exact(&mut bw_bytes)
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
            reader
                .read_exact(&mut bh_bytes)
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
            reader
                .read_exact(&mut nb_bytes)
                .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

            let bx = i32::from_le_bytes(bx_bytes);
            let by = i32::from_le_bytes(by_bytes);
            let bw = i32::from_le_bytes(bw_bytes);
            let bh = i32::from_le_bytes(bh_bytes);
            let bounds = Box::new_unchecked(bx, by, bw, bh);
            let nb = u32::from_le_bytes(nb_bytes) as usize;

            if nb == 0 {
                continue;
            }

            let outer = read_border(&mut reader)?;
            let mut comp = ComponentBorders::new(bounds, outer);

            for _ in 1..nb {
                let hole = read_border(&mut reader)?;
                comp.holes.push(hole);
            }

            image_borders.components.push(comp);
        }

        Ok(image_borders)
    }

    /// Generate an SVG polygon string for all component borders
    ///
    /// Uses the single path (if computed via `generate_single_path`) or
    /// falls back to the outer border in global coordinates.
    pub fn to_svg_string(&self) -> RegionResult<String> {
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"iso-8859-1\"?>\n");
        svg.push_str(&format!(
            "<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
            self.width, self.height, self.width, self.height
        ));

        for comp in &self.components {
            // Collect global-coordinate points: single_path is in local coords, so offset by bounds.
            let global_points: Vec<BorderPoint>;
            let points: &[BorderPoint] = if let Some(ref single) = comp.single_path {
                global_points = single
                    .iter()
                    .map(|p| BorderPoint {
                        x: p.x + comp.bounds.x,
                        y: p.y + comp.bounds.y,
                    })
                    .collect();
                &global_points
            } else {
                global_points = comp.outer_global().points;
                &global_points
            };

            if !points.is_empty() {
                svg.push_str("  <polygon style=\"stroke-width:1;stroke:black;\" points=\"");
                use std::fmt::Write as FmtWrite;
                for (i, p) in points.iter().enumerate() {
                    if i > 0 {
                        svg.push(' ');
                    }
                    let _ = write!(svg, "{},{}", p.x, p.y);
                }
                svg.push_str("\" />\n");
            }
        }

        svg.push_str("</svg>\n");
        Ok(svg)
    }

    /// Write SVG representation to a writer
    pub fn write_svg<W: Write>(&self, mut writer: W) -> RegionResult<()> {
        let svg = self.to_svg_string()?;
        writer
            .write_all(svg.as_bytes())
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
        Ok(())
    }
}

/// Helper: write a border to the stream
fn write_border<W: Write>(writer: &mut W, border: &Border) -> RegionResult<()> {
    // Border type (1 byte: 0 = Outer, 1 = Hole)
    let border_type = match border.border_type {
        BorderType::Outer => 0u8,
        BorderType::Hole => 1u8,
    };
    writer
        .write_all(&[border_type])
        .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

    // Start point
    let sx = (border.start.x as i32).to_le_bytes();
    let sy = (border.start.y as i32).to_le_bytes();
    writer
        .write_all(&sx)
        .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
    writer
        .write_all(&sy)
        .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

    // Number of points
    let np: u32 = border.points.len().try_into().map_err(|_| {
        RegionError::InvalidParameters(format!(
            "border has too many points to serialize: {}",
            border.points.len()
        ))
    })?;
    writer
        .write_all(&np.to_le_bytes())
        .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

    // Points
    for p in &border.points {
        let px = (p.x as i32).to_le_bytes();
        let py = (p.y as i32).to_le_bytes();
        writer
            .write_all(&px)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
        writer
            .write_all(&py)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
    }

    Ok(())
}

/// Helper: read a border from the stream
fn read_border<R: Read>(reader: &mut R) -> RegionResult<Border> {
    let mut type_byte = [0u8; 1];
    let mut sx_bytes = [0u8; 4];
    let mut sy_bytes = [0u8; 4];
    let mut np_bytes = [0u8; 4];

    reader
        .read_exact(&mut type_byte)
        .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
    reader
        .read_exact(&mut sx_bytes)
        .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
    reader
        .read_exact(&mut sy_bytes)
        .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
    reader
        .read_exact(&mut np_bytes)
        .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

    let border_type = match type_byte[0] {
        0 => BorderType::Outer,
        1 => BorderType::Hole,
        b => {
            return Err(RegionError::InvalidParameters(format!(
                "invalid border type byte: {b}"
            )));
        }
    };
    let sx = i32::from_le_bytes(sx_bytes);
    let sy = i32::from_le_bytes(sy_bytes);
    let np = u32::from_le_bytes(np_bytes) as usize;

    let start = BorderPoint::new(sx, sy);
    let mut points = Vec::with_capacity(np);

    for _ in 0..np {
        let mut px_bytes = [0u8; 4];
        let mut py_bytes = [0u8; 4];
        reader
            .read_exact(&mut px_bytes)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;
        reader
            .read_exact(&mut py_bytes)
            .map_err(|e| RegionError::InvalidParameters(e.to_string()))?;

        let px = i32::from_le_bytes(px_bytes);
        let py = i32::from_le_bytes(py_bytes);
        points.push(BorderPoint::new(px, py));
    }

    let mut border = Border::new(border_type, points);
    border.start = start;
    Ok(border)
}

/// Find the next border pixel by searching clockwise from the current search position
///
/// Returns (new_point, new_qpos) or None if no neighbor is found
fn find_next_border_pixel(
    pix: &Pix,
    width: u32,
    height: u32,
    px: i32,
    py: i32,
    qpos: usize,
) -> Option<(BorderPoint, usize)> {
    for i in 1..8 {
        let pos = (qpos + i) % 8;
        let npx = px + XPOSTAB[pos];
        let npy = py + YPOSTAB[pos];

        // Check bounds
        if npx < 0 || npx >= width as i32 || npy < 0 || npy >= height as i32 {
            continue;
        }

        // Check if pixel is ON
        if let Some(val) = pix.get_pixel(npx as u32, npy as u32)
            && val != 0
        {
            return Some((BorderPoint::new(npx, npy), QPOSTAB[pos]));
        }
    }
    None
}

/// Find the first ON pixel by raster scan
fn find_first_on_pixel(pix: &Pix) -> Option<BorderPoint> {
    let width = pix.width();
    let height = pix.height();

    for y in 0..height {
        for x in 0..width {
            if let Some(val) = pix.get_pixel(x, y)
                && val != 0
            {
                return Some(BorderPoint::new(x as i32, y as i32));
            }
        }
    }
    None
}

/// Get the outer border of a single connected component
///
/// The border is traced clockwise with foreground pixels on the right side
/// of the traversal path.
///
/// # Arguments
///
/// * `pix` - Binary image (1-bit depth) containing the component
/// * `bounds` - Optional bounding box for global coordinate calculation
///
/// # Returns
///
/// Border with points in local coordinates (relative to pix origin).
/// If bounds is provided, use `border.to_global()` to convert to global coordinates.
///
/// # Errors
///
/// Returns an error if:
/// - The image is not 1-bit depth
/// - The image is empty (all zeros)
pub fn get_outer_border(pix: &Pix, bounds: Option<&Box>) -> RegionResult<Border> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    if width == 0 || height == 0 {
        return Err(RegionError::EmptyImage);
    }

    // Add 1-pixel border around image for edge handling
    // We do this by working with coordinates offset by 1
    let bordered = add_border(pix, 1)?;
    let bwidth = bordered.width();
    let bheight = bordered.height();

    // Find start pixel (first ON pixel by raster scan)
    let start = match find_first_on_pixel(&bordered) {
        Some(p) => p,
        None => return Err(RegionError::EmptyImage),
    };

    let mut points = Vec::new();
    // Store point with border offset removed
    points.push(BorderPoint::new(start.x - 1, start.y - 1));

    let (fpx, fpy) = (start.x, start.y);
    let mut qpos = 0usize;

    // Get second point
    let second = match find_next_border_pixel(&bordered, bwidth, bheight, fpx, fpy, qpos) {
        Some((p, q)) => {
            qpos = q;
            p
        }
        None => {
            // Single pixel component
            let mut border = Border::new(BorderType::Outer, points);
            if let Some(b) = bounds {
                border = border.to_global(b.x, b.y);
            }
            return Ok(border);
        }
    };

    let (spx, spy) = (second.x, second.y);
    points.push(BorderPoint::new(spx - 1, spy - 1));

    let (mut px, mut py) = (spx, spy);

    // Trace the border
    while let Some((next, new_qpos)) =
        find_next_border_pixel(&bordered, bwidth, bheight, px, py, qpos)
    {
        // Check if we've completed the loop
        if px == fpx && py == fpy && next.x == spx && next.y == spy {
            break;
        }

        points.push(BorderPoint::new(next.x - 1, next.y - 1));
        px = next.x;
        py = next.y;
        qpos = new_qpos;
    }

    let border = Border::new(BorderType::Outer, points);

    // Convert to global coordinates if bounds provided
    if let Some(b) = bounds {
        Ok(border.to_global(b.x, b.y))
    } else {
        Ok(border)
    }
}

/// Add a border of zeros around the image
fn add_border(pix: &Pix, border_size: u32) -> RegionResult<Pix> {
    let width = pix.width();
    let height = pix.height();
    let new_width = width + 2 * border_size;
    let new_height = height + 2 * border_size;

    let mut output = Pix::new(new_width, new_height, pix.depth())
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    // Copy original image to center
    for y in 0..height {
        for x in 0..width {
            if let Some(val) = pix.get_pixel(x, y) {
                let _ = output.set_pixel(x + border_size, y + border_size, val);
            }
        }
    }

    Ok(output.into())
}

/// Get all outer borders from a binary image
///
/// This finds all 8-connected components and traces their outer borders.
///
/// # Arguments
///
/// * `pix` - Binary image (1-bit depth)
///
/// # Returns
///
/// Vector of borders in global coordinates, one for each connected component.
///
/// # Errors
///
/// Returns an error if the image is not 1-bit depth.
pub fn get_outer_borders(pix: &Pix) -> RegionResult<Vec<Border>> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    if width == 0 || height == 0 {
        return Ok(Vec::new());
    }

    // Find connected components
    let components = find_connected_components(pix, ConnectivityType::EightWay)?;

    if components.is_empty() {
        return Ok(Vec::new());
    }

    let mut borders = Vec::with_capacity(components.len());

    for comp in &components {
        // Extract the component image
        let comp_pix = extract_component_image(pix, &comp.bounds)?;

        // Get its outer border
        match get_outer_border(&comp_pix, Some(&comp.bounds)) {
            Ok(border) => borders.push(border),
            Err(RegionError::EmptyImage) => continue,
            Err(e) => return Err(e),
        }
    }

    Ok(borders)
}

/// Extract a sub-image for a component based on its bounding box
fn extract_component_image(pix: &Pix, bounds: &Box) -> RegionResult<Pix> {
    let mut output = Pix::new(bounds.w as u32, bounds.h as u32, pix.depth())
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    for y in 0..bounds.h {
        for x in 0..bounds.w {
            let src_x = (bounds.x + x) as u32;
            let src_y = (bounds.y + y) as u32;
            if let Some(val) = pix.get_pixel(src_x, src_y) {
                let _ = output.set_pixel(x as u32, y as u32, val);
            }
        }
    }

    Ok(output.into())
}

/// Get all borders (outer and holes) for a connected component
///
/// # Arguments
///
/// * `pix` - Binary image containing exactly one 8-connected component
/// * `bounds` - Bounding box of the component in global coordinates
///
/// # Returns
///
/// ComponentBorders containing the outer border and any hole borders.
/// All borders are in local coordinates relative to the component's bounding box.
///
/// # Errors
///
/// Returns an error if the image is not 1-bit depth or is empty.
pub fn get_component_borders(pix: &Pix, bounds: Box) -> RegionResult<ComponentBorders> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    // Get outer border (in local coords)
    let outer = get_outer_border(pix, None)?;

    // Find holes by filling the component and XORing
    let filled = fill_holes(pix, ConnectivityType::FourWay)?;

    // XOR to get hole pixels
    let width = pix.width();
    let height = pix.height();

    let mut holes_pix = Pix::new(width, height, PixelDepth::Bit1)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    let mut has_holes = false;
    for y in 0..height {
        for x in 0..width {
            let orig = pix.get_pixel(x, y).unwrap_or(0);
            let fill = filled.get_pixel(x, y).unwrap_or(0);
            let hole_pixel = fill ^ orig;
            if hole_pixel != 0 {
                has_holes = true;
                let _ = holes_pix.set_pixel(x, y, 1);
            }
        }
    }

    let mut component_borders = ComponentBorders::new(bounds, outer);

    if !has_holes {
        return Ok(component_borders);
    }

    let holes_pix: Pix = holes_pix.into();

    // Find connected components in holes
    let hole_components = find_connected_components(&holes_pix, ConnectivityType::FourWay)?;

    for hole_comp in &hole_components {
        // Find a starting pixel for the hole border
        // We need to find a pixel on the component boundary adjacent to this hole
        if let Some(start) = find_hole_start_pixel(pix, &hole_comp.bounds) {
            // Trace the hole border
            match trace_hole_border(pix, start, &hole_comp.bounds) {
                Ok(border) => component_borders.holes.push(border),
                Err(_) => continue,
            }
        }
    }

    Ok(component_borders)
}

/// Find a starting pixel for tracing a hole border
///
/// We look for a foreground pixel adjacent to the hole
fn find_hole_start_pixel(pix: &Pix, hole_bounds: &Box) -> Option<BorderPoint> {
    let width = pix.width() as i32;

    // Start from the top of the hole bounding box and scan right
    let ys = hole_bounds.y;

    for x in hole_bounds.x..width {
        if let Some(val) = pix.get_pixel(x as u32, ys as u32)
            && val != 0
        {
            return Some(BorderPoint::new(x, ys));
        }
    }

    None
}

/// Trace a hole border starting from a given pixel
fn trace_hole_border(pix: &Pix, start: BorderPoint, _hole_bounds: &Box) -> RegionResult<Border> {
    let width = pix.width();
    let height = pix.height();

    let mut points = Vec::new();
    points.push(start);

    let (fpx, fpy) = (start.x, start.y);
    let mut qpos = 0usize;

    // Get second point
    let second = match find_next_border_pixel(pix, width, height, fpx, fpy, qpos) {
        Some((p, q)) => {
            qpos = q;
            p
        }
        None => {
            // Single pixel - not a valid hole border
            return Err(RegionError::InvalidParameters(
                "hole has only single pixel".to_string(),
            ));
        }
    };

    let (spx, spy) = (second.x, second.y);
    points.push(second);

    let (mut px, mut py) = (spx, spy);

    // Trace the border
    while let Some((next, new_qpos)) = find_next_border_pixel(pix, width, height, px, py, qpos) {
        // Check if we've completed the loop
        if px == fpx && py == fpy && next.x == spx && next.y == spy {
            break;
        }

        points.push(next);
        px = next.x;
        py = next.y;
        qpos = new_qpos;
    }

    Ok(Border::new(BorderType::Hole, points))
}

/// Get all borders from a binary image
///
/// This finds all connected components and traces both outer borders and
/// hole borders for each component.
///
/// # Arguments
///
/// * `pix` - Binary image (1-bit depth)
///
/// # Returns
///
/// ImageBorders containing borders for all connected components.
///
/// # Errors
///
/// Returns an error if the image is not 1-bit depth.
///
/// # Performance Warning
///
/// This function uses O(n_components * image_size) memory. For images with many
/// components (especially large ones), memory usage can be prohibitive. See the
/// module documentation for known limitations and workarounds.
pub fn get_all_borders(pix: &Pix) -> RegionResult<ImageBorders> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: pix.depth().bits(),
        });
    }

    let width = pix.width();
    let height = pix.height();

    let mut image_borders = ImageBorders::new(width, height);

    if width == 0 || height == 0 {
        return Ok(image_borders);
    }

    // Find connected components
    let components = find_connected_components(pix, ConnectivityType::EightWay)?;

    for comp in &components {
        // Extract the component image
        let comp_pix = extract_component_image(pix, &comp.bounds)?;

        // Get all borders for this component
        match get_component_borders(&comp_pix, comp.bounds) {
            Ok(borders) => image_borders.components.push(borders),
            Err(RegionError::EmptyImage) => continue,
            Err(e) => return Err(e),
        }
    }

    Ok(image_borders)
}

/// Convert border points to chain code
///
/// Creates a direction sequence representing the path from point to point.
/// For a closed border, the chain code has one fewer element than points
/// (or same number if first point is repeated at end).
///
/// # Arguments
///
/// * `points` - Sequence of adjacent border points
///
/// # Returns
///
/// Vector of directions representing the chain code
pub fn to_chain_code(points: &[BorderPoint]) -> Vec<Direction> {
    if points.len() < 2 {
        return Vec::new();
    }

    let mut chain = Vec::with_capacity(points.len() - 1);

    for i in 0..points.len() - 1 {
        let p1 = points[i];
        let p2 = points[i + 1];
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;

        if let Some(dir) = Direction::from_offset(dx, dy) {
            chain.push(dir);
        }
    }

    chain
}

/// Convert chain code back to border points
///
/// # Arguments
///
/// * `start` - Starting point
/// * `chain` - Chain code directions
///
/// # Returns
///
/// Vector of border points including the start point
pub fn from_chain_code(start: BorderPoint, chain: &[Direction]) -> Vec<BorderPoint> {
    let mut points = Vec::with_capacity(chain.len() + 1);
    points.push(start);

    let mut current = start;
    for &dir in chain {
        current = current.moved(dir);
        points.push(current);
    }

    points
}

/// Render borders back to a binary image
///
/// Creates a new image with only the border pixels set.
///
/// # Arguments
///
/// * `borders` - Image borders to render
///
/// # Returns
///
/// A new binary image with border pixels set to 1
pub fn render_borders(borders: &ImageBorders) -> RegionResult<Pix> {
    let mut output = Pix::new(borders.width, borders.height, PixelDepth::Bit1)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    for comp in &borders.components {
        // Render outer border in global coordinates
        let outer_global = comp.outer_global();
        for p in &outer_global.points {
            if p.x >= 0 && p.y >= 0 && (p.x as u32) < borders.width && (p.y as u32) < borders.height
            {
                let _ = output.set_pixel(p.x as u32, p.y as u32, 1);
            }
        }

        // Render hole borders
        for hole in comp.holes_global() {
            for p in &hole.points {
                if p.x >= 0
                    && p.y >= 0
                    && (p.x as u32) < borders.width
                    && (p.y as u32) < borders.height
                {
                    let _ = output.set_pixel(p.x as u32, p.y as u32, 1);
                }
            }
        }
    }

    Ok(output.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32, pixels: &[(u32, u32)]) -> Pix {
        let pix = Pix::new(width, height, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for &(x, y) in pixels {
            let _ = pix_mut.set_pixel(x, y, 1);
        }

        pix_mut.into()
    }

    #[test]
    fn test_direction_offsets() {
        assert_eq!(Direction::West.dx(), -1);
        assert_eq!(Direction::West.dy(), 0);
        assert_eq!(Direction::East.dx(), 1);
        assert_eq!(Direction::East.dy(), 0);
        assert_eq!(Direction::North.dy(), -1);
        assert_eq!(Direction::South.dy(), 1);
    }

    #[test]
    fn test_direction_from_offset() {
        assert_eq!(Direction::from_offset(-1, 0), Some(Direction::West));
        assert_eq!(Direction::from_offset(1, 0), Some(Direction::East));
        assert_eq!(Direction::from_offset(0, -1), Some(Direction::North));
        assert_eq!(Direction::from_offset(0, 1), Some(Direction::South));
        assert_eq!(Direction::from_offset(1, 1), Some(Direction::SouthEast));
        assert_eq!(Direction::from_offset(0, 0), None);
        assert_eq!(Direction::from_offset(2, 0), None);
    }

    #[test]
    fn test_border_point_moved() {
        let p = BorderPoint::new(5, 5);
        assert_eq!(p.moved(Direction::East), BorderPoint::new(6, 5));
        assert_eq!(p.moved(Direction::South), BorderPoint::new(5, 6));
        assert_eq!(p.moved(Direction::NorthWest), BorderPoint::new(4, 4));
    }

    #[test]
    fn test_single_pixel_border() {
        let pix = create_test_image(5, 5, &[(2, 2)]);
        let border = get_outer_border(&pix, None).unwrap();

        assert_eq!(border.len(), 1);
        assert_eq!(border.points[0], BorderPoint::new(2, 2));
        assert_eq!(border.border_type, BorderType::Outer);
    }

    #[test]
    fn test_horizontal_line_border() {
        let pix = create_test_image(10, 5, &[(2, 2), (3, 2), (4, 2)]);
        let border = get_outer_border(&pix, None).unwrap();

        assert!(!border.is_empty());
        // Should trace around the horizontal line
        assert!(border.len() >= 3);
    }

    #[test]
    fn test_square_border() {
        // 3x3 square
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(6, 6, &pixels);
        let border = get_outer_border(&pix, None).unwrap();

        assert!(!border.is_empty());
        // A 3x3 filled square has 8 boundary pixels (9 total - 1 center)
        // Border tracing may include some points twice depending on traversal
        // The perimeter should be at least 8 and not more than 12
        assert!(
            border.len() >= 8 && border.len() <= 12,
            "got {}",
            border.len()
        );
    }

    #[test]
    fn test_outer_borders_multiple() {
        // Two separate squares
        let mut pixels = Vec::new();
        // First square at (1,1)
        for y in 1..3 {
            for x in 1..3 {
                pixels.push((x, y));
            }
        }
        // Second square at (5,1)
        for y in 1..3 {
            for x in 5..7 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(10, 5, &pixels);
        let borders = get_outer_borders(&pix).unwrap();

        assert_eq!(borders.len(), 2);
    }

    #[test]
    fn test_chain_code_conversion() {
        let points = vec![
            BorderPoint::new(0, 0),
            BorderPoint::new(1, 0),
            BorderPoint::new(2, 0),
            BorderPoint::new(2, 1),
        ];

        let chain = to_chain_code(&points);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0], Direction::East);
        assert_eq!(chain[1], Direction::East);
        assert_eq!(chain[2], Direction::South);

        // Convert back
        let restored = from_chain_code(points[0], &chain);
        assert_eq!(restored, points);
    }

    #[test]
    fn test_chain_code_empty() {
        let chain = to_chain_code(&[]);
        assert!(chain.is_empty());

        let chain = to_chain_code(&[BorderPoint::new(0, 0)]);
        assert!(chain.is_empty());
    }

    #[test]
    fn test_border_bounding_box() {
        let border = Border::new(
            BorderType::Outer,
            vec![
                BorderPoint::new(1, 1),
                BorderPoint::new(3, 1),
                BorderPoint::new(3, 4),
                BorderPoint::new(1, 4),
            ],
        );

        let bbox = border.bounding_box().unwrap();
        assert_eq!(bbox.x, 1);
        assert_eq!(bbox.y, 1);
        assert_eq!(bbox.w, 3);
        assert_eq!(bbox.h, 4);
    }

    #[test]
    fn test_border_to_global() {
        let border = Border::new(
            BorderType::Outer,
            vec![BorderPoint::new(0, 0), BorderPoint::new(1, 0)],
        );

        let global = border.to_global(10, 20);
        assert_eq!(global.points[0], BorderPoint::new(10, 20));
        assert_eq!(global.points[1], BorderPoint::new(11, 20));
    }

    #[test]
    fn test_empty_image() {
        let pix = create_test_image(5, 5, &[]);
        let result = get_outer_border(&pix, None);
        assert!(matches!(result, Err(RegionError::EmptyImage)));
    }

    #[test]
    fn test_unsupported_depth() {
        let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let result = get_outer_border(&pix, None);
        assert!(matches!(result, Err(RegionError::UnsupportedDepth { .. })));
    }

    #[test]
    fn test_get_all_borders() {
        // Create a simple square
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(6, 6, &pixels);
        let borders = get_all_borders(&pix).unwrap();

        assert_eq!(borders.component_count(), 1);
        assert!(!borders.components[0].outer.is_empty());
    }

    #[test]
    fn test_component_with_hole() {
        // Create a ring (square with hole)
        // 11111
        // 10001
        // 10001
        // 10001
        // 11111
        let mut pixels = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                if y == 0 || y == 4 || x == 0 || x == 4 {
                    pixels.push((x, y));
                }
            }
        }
        let pix = create_test_image(5, 5, &pixels);
        let borders = get_all_borders(&pix).unwrap();

        assert_eq!(borders.component_count(), 1);
        let comp = &borders.components[0];
        assert!(!comp.outer.is_empty());
        // The component should have a hole
        assert!(comp.has_holes());
    }

    // --- Phase 5: CCBord拡張テスト ---

    #[test]
    fn test_generate_step_chains() {
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(6, 6, &pixels);
        let mut borders = get_all_borders(&pix).unwrap();

        borders.generate_step_chains();

        let comp = &borders.components[0];
        assert!(comp.outer.chain_code.is_some());
        let chain = comp.outer.chain_code.as_ref().unwrap();
        // 8 border pixels → 7 steps (n-1 chain codes)
        assert!(!chain.is_empty());
    }

    #[test]

    fn test_step_chains_to_pix_coords_roundtrip() {
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(6, 6, &pixels);
        let mut borders = get_all_borders(&pix).unwrap();

        let original_points = borders.components[0].outer.points.clone();
        borders.generate_step_chains();
        borders.step_chains_to_pix_coords().unwrap();

        assert_eq!(borders.components[0].outer.points, original_points);
    }

    #[test]

    fn test_generate_single_path_no_holes() {
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(6, 6, &pixels);
        let mut borders = get_all_borders(&pix).unwrap();

        borders.generate_single_path().unwrap();

        let comp = &borders.components[0];
        assert!(comp.single_path.is_some());
        let path = comp.single_path.as_ref().unwrap();
        // For no-hole component, single path == outer border
        assert_eq!(*path, comp.outer.points);
    }

    #[test]

    fn test_generate_single_path_with_holes() {
        // Ring: 7x7 frame (hollow square)
        let mut pixels = Vec::new();
        for y in 0..7u32 {
            for x in 0..7u32 {
                if y == 0 || y == 6 || x == 0 || x == 6 {
                    pixels.push((x, y));
                }
            }
        }
        let pix = create_test_image(8, 8, &pixels);
        let mut borders = get_all_borders(&pix).unwrap();

        borders.generate_single_path().unwrap();

        let comp = &borders.components[0];
        assert!(comp.single_path.is_some());
        let path = comp.single_path.as_ref().unwrap();
        // Path must be non-empty and longer than outer border alone
        assert!(!path.is_empty());
        assert!(path.len() >= comp.outer.points.len());
    }

    #[test]

    fn test_write_read_roundtrip() {
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(6, 6, &pixels);
        let mut borders = get_all_borders(&pix).unwrap();
        borders.generate_step_chains();

        let mut buf = Vec::new();
        borders.write(&mut buf).unwrap();

        let read_back = ImageBorders::read_from(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(read_back.width, borders.width);
        assert_eq!(read_back.height, borders.height);
        assert_eq!(read_back.component_count(), borders.component_count());
        assert_eq!(
            read_back.components[0].outer.points,
            borders.components[0].outer.points
        );
    }

    #[test]

    fn test_to_svg_string() {
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(6, 6, &pixels);
        let mut borders = get_all_borders(&pix).unwrap();
        borders.generate_single_path().unwrap();

        let svg = borders.to_svg_string().unwrap();
        assert!(svg.contains("<svg "));
        assert!(svg.contains("polygon"));
        assert!(svg.contains("</svg>"));
    }

    #[test]

    fn test_write_svg() {
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(6, 6, &pixels);
        let mut borders = get_all_borders(&pix).unwrap();
        borders.generate_single_path().unwrap();

        let mut buf = Vec::new();
        borders.write_svg(&mut buf).unwrap();
        let svg_str = String::from_utf8(buf).unwrap();
        assert!(svg_str.contains("<svg "));
        assert!(svg_str.contains("</svg>"));
    }

    #[test]

    fn test_write_read_with_holes() {
        // Ring shape
        let mut pixels = Vec::new();
        for y in 0..5u32 {
            for x in 0..5u32 {
                if y == 0 || y == 4 || x == 0 || x == 4 {
                    pixels.push((x, y));
                }
            }
        }
        let pix = create_test_image(5, 5, &pixels);
        let mut borders = get_all_borders(&pix).unwrap();
        borders.generate_step_chains();

        let mut buf = Vec::new();
        borders.write(&mut buf).unwrap();

        let read_back = ImageBorders::read_from(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(read_back.component_count(), 1);
        // Outer + hole borders
        assert_eq!(
            read_back.components[0].holes.len(),
            borders.components[0].holes.len()
        );
    }

    #[test]

    fn test_write_read_empty() {
        let borders = ImageBorders::new(100, 200);
        let mut buf = Vec::new();
        borders.write(&mut buf).unwrap();

        let read_back = ImageBorders::read_from(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(read_back.width, 100);
        assert_eq!(read_back.height, 200);
        assert_eq!(read_back.component_count(), 0);
    }

    #[test]
    fn test_render_borders() {
        let mut pixels = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                pixels.push((x, y));
            }
        }
        let pix = create_test_image(6, 6, &pixels);
        let borders = get_all_borders(&pix).unwrap();
        let rendered = render_borders(&borders).unwrap();

        assert_eq!(rendered.width(), 6);
        assert_eq!(rendered.height(), 6);
        // Border pixels should be set
        assert_eq!(rendered.get_pixel(1, 1), Some(1));
    }
}
