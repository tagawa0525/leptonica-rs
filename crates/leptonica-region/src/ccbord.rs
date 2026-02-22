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
}

impl ComponentBorders {
    /// Create new component borders
    pub fn new(bounds: Box, outer: Border) -> Self {
        Self {
            bounds,
            outer,
            holes: Vec::new(),
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
