//! Maze generation and solving
//!
//! This module provides functions for generating random mazes and finding
//! the shortest path between two points in a maze. The maze is represented
//! as a binary image where:
//! - ON pixels (1) = walls
//! - OFF pixels (0) = passages
//!
//! # Examples
//!
//! ## Generating a maze
//!
//! ```
//! use leptonica_region::{MazeGenerationOptions, generate_binary_maze};
//!
//! let options = MazeGenerationOptions::new(100, 100)
//!     .with_wall_probability(0.6)
//!     .with_anisotropy(0.3);
//! let maze = generate_binary_maze(&options).unwrap();
//! ```
//!
//! ## Finding a path through a maze
//!
//! ```
//! use leptonica_region::{MazeGenerationOptions, generate_binary_maze, search_binary_maze};
//!
//! let options = MazeGenerationOptions::new(100, 100).with_start(16, 20);
//! let maze = generate_binary_maze(&options).unwrap();
//! // Start from the same point used to generate the maze (guaranteed to be a passage)
//! let (path, _) = search_binary_maze(&maze, (16, 20), (90, 90), false).unwrap();
//! if path.found {
//!     println!("Path found with {} steps", path.len());
//! }
//! ```

use crate::error::{RegionError, RegionResult};
use leptonica_core::{Pix, PixMut, PixelDepth};
use rand::RngExt;
use std::collections::VecDeque;

/// Direction from parent to child in maze traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MazeDirection {
    /// Starting location (no parent)
    #[default]
    Start = 0,
    /// Moving north (up, y decreasing)
    North = 1,
    /// Moving south (down, y increasing)
    South = 2,
    /// Moving west (left, x decreasing)
    West = 3,
    /// Moving east (right, x increasing)
    East = 4,
}

impl MazeDirection {
    /// Get the X offset for this direction
    #[inline]
    fn dx(self) -> i32 {
        match self {
            MazeDirection::Start => 0,
            MazeDirection::North => 0,
            MazeDirection::South => 0,
            MazeDirection::West => -1,
            MazeDirection::East => 1,
        }
    }

    /// Get the Y offset for this direction
    #[inline]
    fn dy(self) -> i32 {
        match self {
            MazeDirection::Start => 0,
            MazeDirection::North => -1,
            MazeDirection::South => 1,
            MazeDirection::West => 0,
            MazeDirection::East => 0,
        }
    }

    /// Get the opposite direction (for backtracking)
    #[inline]
    #[allow(dead_code)]
    fn opposite(self) -> MazeDirection {
        match self {
            MazeDirection::Start => MazeDirection::Start,
            MazeDirection::North => MazeDirection::South,
            MazeDirection::South => MazeDirection::North,
            MazeDirection::West => MazeDirection::East,
            MazeDirection::East => MazeDirection::West,
        }
    }

    /// Convert from u8 value
    fn from_u8(val: u8) -> Option<MazeDirection> {
        match val {
            0 => Some(MazeDirection::Start),
            1 => Some(MazeDirection::North),
            2 => Some(MazeDirection::South),
            3 => Some(MazeDirection::West),
            4 => Some(MazeDirection::East),
            _ => None,
        }
    }
}

/// Minimum maze width (in pixels)
pub const MIN_MAZE_WIDTH: u32 = 50;

/// Minimum maze height (in pixels)
pub const MIN_MAZE_HEIGHT: u32 = 50;

/// Default wall probability
pub const DEFAULT_WALL_PROBABILITY: f32 = 0.65;

/// Default anisotropy ratio
pub const DEFAULT_ANISOTROPY_RATIO: f32 = 0.25;

/// Options for maze generation
#[derive(Debug, Clone)]
pub struct MazeGenerationOptions {
    /// Width of the maze (minimum 50)
    pub width: u32,
    /// Height of the maze (minimum 50)
    pub height: u32,
    /// Initial x position (seed point). If 0 or out of bounds, defaults to width/6
    pub start_x: u32,
    /// Initial y position (seed point). If 0 or out of bounds, defaults to height/5
    pub start_y: u32,
    /// Probability that a side pixel becomes a wall (0.05 - 0.95)
    pub wall_probability: f32,
    /// Ratio of forward vs side wall probability (0.05 - 1.0)
    /// Lower values result in longer straight passages
    pub anisotropy_ratio: f32,
}

impl Default for MazeGenerationOptions {
    fn default() -> Self {
        Self {
            width: MIN_MAZE_WIDTH,
            height: MIN_MAZE_HEIGHT,
            start_x: 0,
            start_y: 0,
            wall_probability: DEFAULT_WALL_PROBABILITY,
            anisotropy_ratio: DEFAULT_ANISOTROPY_RATIO,
        }
    }
}

impl MazeGenerationOptions {
    /// Create options with specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Set the starting position
    pub fn with_start(mut self, x: u32, y: u32) -> Self {
        self.start_x = x;
        self.start_y = y;
        self
    }

    /// Set wall probability (clamped to 0.05 - 0.95)
    pub fn with_wall_probability(mut self, prob: f32) -> Self {
        self.wall_probability = prob.clamp(0.05, 0.95);
        self
    }

    /// Set anisotropy ratio (clamped to 0.05 - 1.0)
    pub fn with_anisotropy(mut self, ratio: f32) -> Self {
        self.anisotropy_ratio = ratio.clamp(0.05, 1.0);
        self
    }

    /// Validate and normalize options
    fn normalize(&self) -> Self {
        let mut opts = self.clone();

        // Ensure minimum dimensions
        if opts.width < MIN_MAZE_WIDTH {
            opts.width = MIN_MAZE_WIDTH;
        }
        if opts.height < MIN_MAZE_HEIGHT {
            opts.height = MIN_MAZE_HEIGHT;
        }

        // Set default start position if invalid
        if opts.start_x == 0 || opts.start_x >= opts.width {
            opts.start_x = opts.width / 6;
        }
        if opts.start_y == 0 || opts.start_y >= opts.height {
            opts.start_y = opts.height / 5;
        }

        // Clamp probabilities
        if opts.wall_probability < 0.05 || opts.wall_probability > 0.95 {
            opts.wall_probability = DEFAULT_WALL_PROBABILITY;
        }
        if opts.anisotropy_ratio < 0.05 || opts.anisotropy_ratio > 1.0 {
            opts.anisotropy_ratio = DEFAULT_ANISOTROPY_RATIO;
        }

        opts
    }
}

/// Result of maze path search
#[derive(Debug, Clone)]
pub struct MazePath {
    /// Sequence of points from end to start (reverse order of traversal)
    pub points: Vec<(u32, u32)>,
    /// Whether a valid path was found
    pub found: bool,
}

impl MazePath {
    /// Create an empty path (no path found)
    pub fn not_found() -> Self {
        Self {
            points: Vec::new(),
            found: false,
        }
    }

    /// Get the length of the path (number of points)
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Get points in order from start to end
    pub fn points_start_to_end(&self) -> Vec<(u32, u32)> {
        let mut points = self.points.clone();
        points.reverse();
        points
    }
}

/// Element in the maze generation/search queue
#[derive(Debug, Clone, Copy)]
struct MazeElement {
    x: u32,
    y: u32,
    dir: MazeDirection,
}

impl MazeElement {
    fn new(x: u32, y: u32, dir: MazeDirection) -> Self {
        Self { x, y, dir }
    }
}

/// Generate a binary maze using cellular automaton approach
///
/// Creates a maze where ON pixels (1) are walls and OFF pixels (0) are passages.
/// The maze is generated using a queue-based BFS from the starting point,
/// with probabilistic wall placement.
///
/// # Arguments
///
/// * `options` - Maze generation options
///
/// # Returns
///
/// A 1-bit binary image representing the maze
///
/// # Notes
///
/// The `wall_probability` and `anisotropy_ratio` parameters control maze density:
/// - Higher wall_probability = denser walls
/// - Lower anisotropy_ratio = longer straight passages
///
/// Phase transition boundary (approximate):
/// ```text
/// wall_prob  anisotropy
/// 0.35       1.00
/// 0.50       0.50
/// 0.65       0.25
/// 0.80       0.11
/// ```
pub fn generate_binary_maze(options: &MazeGenerationOptions) -> RegionResult<Pix> {
    let opts = options.normalize();
    let width = opts.width;
    let height = opts.height;
    let xi = opts.start_x;
    let yi = opts.start_y;
    let wall_ps = opts.wall_probability;
    let wall_pf = wall_ps * opts.anisotropy_ratio; // Forward wall probability

    // Create output maze (all passages initially)
    let mut pixd = Pix::new(width, height, PixelDepth::Bit1)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    // Create visited marker
    let mut pixm = Pix::new(width, height, PixelDepth::Bit1)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    let mut rng = rand::rng();
    let mut queue: VecDeque<MazeElement> = VecDeque::new();

    // Prime the queue with the first pixel
    let _ = pixm.set_pixel(xi, yi, 1); // Mark visited
    queue.push_back(MazeElement::new(xi, yi, MazeDirection::Start));

    // Process queue
    while let Some(elem) = queue.pop_front() {
        let x = elem.x;
        let y = elem.y;
        let dir = elem.dir;

        // Check West
        if x > 0 && pixm.get_pixel(x - 1, y).unwrap_or(1) == 0 {
            let _ = pixm.set_pixel(x - 1, y, 1); // Mark visited
            let test_p = if dir == MazeDirection::West {
                wall_pf
            } else {
                wall_ps
            };
            if rng.random::<f32>() <= test_p {
                let _ = pixd.set_pixel(x - 1, y, 1); // Make wall
            } else {
                queue.push_back(MazeElement::new(x - 1, y, MazeDirection::West));
            }
        }

        // Check North
        if y > 0 && pixm.get_pixel(x, y - 1).unwrap_or(1) == 0 {
            let _ = pixm.set_pixel(x, y - 1, 1); // Mark visited
            let test_p = if dir == MazeDirection::North {
                wall_pf
            } else {
                wall_ps
            };
            if rng.random::<f32>() <= test_p {
                let _ = pixd.set_pixel(x, y - 1, 1); // Make wall
            } else {
                queue.push_back(MazeElement::new(x, y - 1, MazeDirection::North));
            }
        }

        // Check East
        if x + 1 < width && pixm.get_pixel(x + 1, y).unwrap_or(1) == 0 {
            let _ = pixm.set_pixel(x + 1, y, 1); // Mark visited
            let test_p = if dir == MazeDirection::East {
                wall_pf
            } else {
                wall_ps
            };
            if rng.random::<f32>() <= test_p {
                let _ = pixd.set_pixel(x + 1, y, 1); // Make wall
            } else {
                queue.push_back(MazeElement::new(x + 1, y, MazeDirection::East));
            }
        }

        // Check South
        if y + 1 < height && pixm.get_pixel(x, y + 1).unwrap_or(1) == 0 {
            let _ = pixm.set_pixel(x, y + 1, 1); // Mark visited
            let test_p = if dir == MazeDirection::South {
                wall_pf
            } else {
                wall_ps
            };
            if rng.random::<f32>() <= test_p {
                let _ = pixd.set_pixel(x, y + 1, 1); // Make wall
            } else {
                queue.push_back(MazeElement::new(x, y + 1, MazeDirection::South));
            }
        }
    }

    Ok(pixd.into())
}

/// Search for a background pixel near the given position
///
/// Searches in expanding squares from the given position to find a
/// background (passage) pixel.
///
/// # Arguments
///
/// * `pix` - Binary maze image
/// * `x` - X coordinate (will be modified if a nearby background is found)
/// * `y` - Y coordinate (will be modified if a nearby background is found)
/// * `max_radius` - Maximum search radius
///
/// # Returns
///
/// `true` if a background pixel was found, `false` otherwise
fn local_search_for_background(pix: &Pix, x: &mut u32, y: &mut u32, max_radius: i32) -> bool {
    let width = pix.width() as i32;
    let height = pix.height() as i32;
    let px = *x as i32;
    let py = *y as i32;

    // Check current position first
    if pix.get_pixel(*x, *y).unwrap_or(1) == 0 {
        return true;
    }

    // Search in expanding rings
    for r in 1..max_radius {
        for i in -r..=r {
            let ny = py + i;
            if ny < 0 || ny >= height {
                continue;
            }
            for j in -r..=r {
                let nx = px + j;
                if nx < 0 || nx >= width {
                    continue;
                }
                // Only check pixels on the boundary of the square
                if i.abs() != r && j.abs() != r {
                    continue;
                }
                if pix.get_pixel(nx as u32, ny as u32).unwrap_or(1) == 0 {
                    *x = nx as u32;
                    *y = ny as u32;
                    return true;
                }
            }
        }
    }

    false
}

/// Search for the shortest path in a binary maze
///
/// Uses breadth-first search (BFS) to find the shortest Manhattan path
/// between start and end points, avoiding walls (ON pixels).
///
/// # Arguments
///
/// * `maze` - Binary maze image (1-bit, ON = wall, OFF = passage)
/// * `start` - Starting point (xi, yi)
/// * `end` - Ending point (xf, yf)
/// * `visualize` - If true, also returns a visualization image
///
/// # Returns
///
/// A tuple of (MazePath, Option<Pix>):
/// - MazePath contains the path coordinates (or empty if no path)
/// - If visualize=true, returns 32-bit RGB image with:
///   - Red pixel at start
///   - Blue pixel at end
///   - Green pixels along the path
///   - If no path, green marks explored region
///
/// # Errors
///
/// Returns an error if:
/// - The maze is not a 1-bit image
/// - The start point is out of bounds or on a wall
pub fn search_binary_maze(
    maze: &Pix,
    start: (u32, u32),
    end: (u32, u32),
    visualize: bool,
) -> RegionResult<(MazePath, Option<Pix>)> {
    if maze.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: maze.depth().bits(),
        });
    }

    let width = maze.width();
    let height = maze.height();

    let (xi, yi) = start;
    let (mut xf, mut yf) = end;

    // Validate start position
    if xi >= width || yi >= height {
        return Err(RegionError::InvalidParameters(format!(
            "start position ({}, {}) out of bounds for {}x{} image",
            xi, yi, width, height
        )));
    }

    // Check that start is on a passage
    if maze.get_pixel(xi, yi).unwrap_or(1) != 0 {
        return Err(RegionError::InvalidParameters(
            "start position is on a wall, not a passage".to_string(),
        ));
    }

    // Adjust end position if on a wall
    if xf >= width {
        xf = width - 1;
    }
    if yf >= height {
        yf = height - 1;
    }
    let _ = local_search_for_background(maze, &mut xf, &mut yf, 5);

    // Create visited marker
    let mut pixm = Pix::new(width, height, PixelDepth::Bit1)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    // Create parent direction map (8-bit to store direction enum)
    let mut pixp = Pix::new(width, height, PixelDepth::Bit8)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    let mut queue: VecDeque<MazeElement> = VecDeque::new();

    // Prime the queue with start
    let _ = pixm.set_pixel(xi, yi, 1);
    queue.push_back(MazeElement::new(xi, yi, MazeDirection::Start));

    let mut found = false;

    // BFS
    while let Some(elem) = queue.pop_front() {
        let x = elem.x;
        let y = elem.y;

        // Check if we reached the end
        if x == xf && y == yf {
            found = true;
            break;
        }

        // Check West
        if x > 0 && pixm.get_pixel(x - 1, y).unwrap_or(1) == 0 {
            let _ = pixm.set_pixel(x - 1, y, 1); // Mark visited
            if maze.get_pixel(x - 1, y).unwrap_or(1) == 0 {
                // Not a wall
                let _ = pixp.set_pixel(x - 1, y, MazeDirection::East as u32); // Parent is East
                queue.push_back(MazeElement::new(x - 1, y, MazeDirection::West));
            }
        }

        // Check North
        if y > 0 && pixm.get_pixel(x, y - 1).unwrap_or(1) == 0 {
            let _ = pixm.set_pixel(x, y - 1, 1); // Mark visited
            if maze.get_pixel(x, y - 1).unwrap_or(1) == 0 {
                // Not a wall
                let _ = pixp.set_pixel(x, y - 1, MazeDirection::South as u32); // Parent is South
                queue.push_back(MazeElement::new(x, y - 1, MazeDirection::North));
            }
        }

        // Check East
        if x + 1 < width && pixm.get_pixel(x + 1, y).unwrap_or(1) == 0 {
            let _ = pixm.set_pixel(x + 1, y, 1); // Mark visited
            if maze.get_pixel(x + 1, y).unwrap_or(1) == 0 {
                // Not a wall
                let _ = pixp.set_pixel(x + 1, y, MazeDirection::West as u32); // Parent is West
                queue.push_back(MazeElement::new(x + 1, y, MazeDirection::East));
            }
        }

        // Check South
        if y + 1 < height && pixm.get_pixel(x, y + 1).unwrap_or(1) == 0 {
            let _ = pixm.set_pixel(x, y + 1, 1); // Mark visited
            if maze.get_pixel(x, y + 1).unwrap_or(1) == 0 {
                // Not a wall
                let _ = pixp.set_pixel(x, y + 1, MazeDirection::North as u32); // Parent is North
                queue.push_back(MazeElement::new(x, y + 1, MazeDirection::South));
            }
        }
    }

    let path = if found {
        // Backtrack from end to start
        let mut points = Vec::new();
        let mut x = xf;
        let mut y = yf;

        loop {
            points.push((x, y));
            if x == xi && y == yi {
                break;
            }

            let dir_val = pixp.get_pixel(x, y).unwrap_or(0) as u8;
            if let Some(dir) = MazeDirection::from_u8(dir_val) {
                x = (x as i32 + dir.dx()) as u32;
                y = (y as i32 + dir.dy()) as u32;
            } else {
                // Should not happen, but break to avoid infinite loop
                break;
            }
        }

        MazePath {
            points,
            found: true,
        }
    } else {
        MazePath::not_found()
    };

    // Create visualization if requested
    let vis = if visualize {
        Some(create_visualization(
            maze,
            &path,
            &pixp,
            &pixm,
            (xi, yi),
            (xf, yf),
        )?)
    } else {
        None
    };

    Ok((path, vis))
}

/// Create a visualization of the maze search result
fn create_visualization(
    maze: &Pix,
    path: &MazePath,
    _pixp: &PixMut,
    pixm: &PixMut,
    start: (u32, u32),
    end: (u32, u32),
) -> RegionResult<Pix> {
    let width = maze.width();
    let height = maze.height();

    // Create 32-bit RGB image
    let mut pixd = Pix::new(width, height, PixelDepth::Bit32)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    // RGB color values (packed as 0xRRGGBB00 or similar depending on format)
    let white = 0xFFFFFF00u32;
    let black = 0x00000000u32;
    let red = 0xFF000000u32;
    let green = 0x00FF0000u32;
    let blue = 0x0000FF00u32;

    // Copy maze: walls are black, passages are white
    for y in 0..height {
        for x in 0..width {
            let val = maze.get_pixel(x, y).unwrap_or(0);
            let color = if val == 0 { white } else { black };
            let _ = pixd.set_pixel(x, y, color);
        }
    }

    // If path found, draw it in green
    if path.found {
        for &(px, py) in &path.points {
            let _ = pixd.set_pixel(px, py, green);
        }
    } else {
        // No path found - mark explored region in green
        for y in 0..height {
            for x in 0..width {
                if pixm.get_pixel(x, y).unwrap_or(0) != 0 && maze.get_pixel(x, y).unwrap_or(1) == 0
                {
                    let _ = pixd.set_pixel(x, y, green);
                }
            }
        }
    }

    // Mark start and end points
    let (xi, yi) = start;
    let (xf, yf) = end;
    let _ = pixd.set_pixel(xi, yi, red);
    let _ = pixd.set_pixel(xf, yf, blue);

    Ok(pixd.into())
}

/// Render the maze path onto an image
///
/// # Arguments
///
/// * `maze` - Original binary maze image
/// * `path` - Path to render
/// * `start` - Start point (marked in red)
/// * `end` - End point (marked in blue)
///
/// # Returns
///
/// 32-bit RGB image with the path visualized
pub fn render_maze_path(
    maze: &Pix,
    path: &MazePath,
    start: (u32, u32),
    end: (u32, u32),
) -> RegionResult<Pix> {
    if maze.depth() != PixelDepth::Bit1 {
        return Err(RegionError::UnsupportedDepth {
            expected: "1-bit",
            actual: maze.depth().bits(),
        });
    }

    let width = maze.width();
    let height = maze.height();

    // Create 32-bit RGB image
    let mut pixd = Pix::new(width, height, PixelDepth::Bit32)
        .map_err(RegionError::Core)?
        .try_into_mut()
        .unwrap_or_else(|p| p.to_mut());

    let white = 0xFFFFFF00u32;
    let black = 0x00000000u32;
    let red = 0xFF000000u32;
    let green = 0x00FF0000u32;
    let blue = 0x0000FF00u32;

    // Copy maze
    for y in 0..height {
        for x in 0..width {
            let val = maze.get_pixel(x, y).unwrap_or(0);
            let color = if val == 0 { white } else { black };
            let _ = pixd.set_pixel(x, y, color);
        }
    }

    // Draw path
    for &(px, py) in &path.points {
        let _ = pixd.set_pixel(px, py, green);
    }

    // Mark start and end
    let (xi, yi) = start;
    let (xf, yf) = end;
    let _ = pixd.set_pixel(xi, yi, red);
    let _ = pixd.set_pixel(xf, yf, blue);

    Ok(pixd.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_direction_dx_dy() {
        assert_eq!(MazeDirection::North.dx(), 0);
        assert_eq!(MazeDirection::North.dy(), -1);
        assert_eq!(MazeDirection::South.dx(), 0);
        assert_eq!(MazeDirection::South.dy(), 1);
        assert_eq!(MazeDirection::West.dx(), -1);
        assert_eq!(MazeDirection::West.dy(), 0);
        assert_eq!(MazeDirection::East.dx(), 1);
        assert_eq!(MazeDirection::East.dy(), 0);
    }

    #[test]
    fn test_maze_direction_opposite() {
        assert_eq!(MazeDirection::North.opposite(), MazeDirection::South);
        assert_eq!(MazeDirection::South.opposite(), MazeDirection::North);
        assert_eq!(MazeDirection::West.opposite(), MazeDirection::East);
        assert_eq!(MazeDirection::East.opposite(), MazeDirection::West);
    }

    #[test]
    fn test_maze_generation_options_default() {
        let opts = MazeGenerationOptions::default();
        assert_eq!(opts.width, MIN_MAZE_WIDTH);
        assert_eq!(opts.height, MIN_MAZE_HEIGHT);
        assert!((opts.wall_probability - DEFAULT_WALL_PROBABILITY).abs() < 0.001);
        assert!((opts.anisotropy_ratio - DEFAULT_ANISOTROPY_RATIO).abs() < 0.001);
    }

    #[test]
    fn test_maze_generation_options_normalize() {
        let opts = MazeGenerationOptions {
            width: 10, // Too small
            height: 20,
            start_x: 0,
            start_y: 0,
            wall_probability: 0.01, // Too low
            anisotropy_ratio: 2.0,  // Too high
        };
        let normalized = opts.normalize();
        assert_eq!(normalized.width, MIN_MAZE_WIDTH);
        assert_eq!(normalized.height, MIN_MAZE_HEIGHT);
        assert!((normalized.wall_probability - DEFAULT_WALL_PROBABILITY).abs() < 0.001);
        assert!((normalized.anisotropy_ratio - DEFAULT_ANISOTROPY_RATIO).abs() < 0.001);
    }

    #[test]
    fn test_generate_binary_maze() {
        let opts = MazeGenerationOptions::new(60, 60);
        let maze = generate_binary_maze(&opts).unwrap();

        assert_eq!(maze.width(), 60);
        assert_eq!(maze.height(), 60);
        assert_eq!(maze.depth(), PixelDepth::Bit1);

        // The starting position should be a passage
        let start_x = 60 / 6; // default start_x
        let start_y = 60 / 5; // default start_y
        assert_eq!(maze.get_pixel(start_x, start_y), Some(0));
    }

    #[test]
    fn test_generate_maze_with_custom_options() {
        let opts = MazeGenerationOptions::new(100, 100)
            .with_start(20, 20)
            .with_wall_probability(0.5)
            .with_anisotropy(0.5);

        let maze = generate_binary_maze(&opts).unwrap();

        assert_eq!(maze.width(), 100);
        assert_eq!(maze.height(), 100);
        // Custom start should be a passage
        assert_eq!(maze.get_pixel(20, 20), Some(0));
    }

    #[test]
    fn test_search_simple_path() {
        // Create a simple maze with a clear path
        let mut maze = Pix::new(10, 10, PixelDepth::Bit1)
            .unwrap()
            .try_into_mut()
            .unwrap();

        // Leave a horizontal corridor at y=5
        // Everything else is walls
        for y in 0..10 {
            for x in 0..10 {
                if y != 5 {
                    let _ = maze.set_pixel(x, y, 1); // wall
                }
            }
        }

        let maze: Pix = maze.into();

        let (path, _) = search_binary_maze(&maze, (0, 5), (9, 5), false).unwrap();

        assert!(path.found);
        assert_eq!(path.len(), 10); // 10 points from (0,5) to (9,5)
    }

    #[test]
    fn test_search_no_path() {
        // Create a maze with start and end completely separated
        let mut maze = Pix::new(10, 10, PixelDepth::Bit1)
            .unwrap()
            .try_into_mut()
            .unwrap();

        // Wall dividing the maze vertically
        for y in 0..10 {
            let _ = maze.set_pixel(5, y, 1);
        }

        let maze: Pix = maze.into();

        let (path, _) = search_binary_maze(&maze, (2, 5), (8, 5), false).unwrap();

        assert!(!path.found);
        assert!(path.is_empty());
    }

    #[test]
    fn test_search_start_equals_end() {
        let maze = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        let (path, _) = search_binary_maze(&maze, (5, 5), (5, 5), false).unwrap();

        assert!(path.found);
        assert_eq!(path.len(), 1);
        assert_eq!(path.points[0], (5, 5));
    }

    #[test]
    fn test_search_with_visualization() {
        let maze = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        let (path, vis) = search_binary_maze(&maze, (0, 0), (9, 9), true).unwrap();

        assert!(path.found);
        assert!(vis.is_some());

        let vis = vis.unwrap();
        assert_eq!(vis.width(), 10);
        assert_eq!(vis.height(), 10);
        assert_eq!(vis.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_search_invalid_start_position() {
        let maze = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        let result = search_binary_maze(&maze, (20, 20), (5, 5), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_start_on_wall() {
        let mut maze = Pix::new(10, 10, PixelDepth::Bit1)
            .unwrap()
            .try_into_mut()
            .unwrap();

        let _ = maze.set_pixel(5, 5, 1); // wall at start position

        let maze: Pix = maze.into();

        let result = search_binary_maze(&maze, (5, 5), (8, 8), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_maze_path_not_found() {
        let path = MazePath::not_found();
        assert!(!path.found);
        assert!(path.is_empty());
        assert_eq!(path.len(), 0);
    }

    #[test]
    fn test_maze_path_points_start_to_end() {
        let path = MazePath {
            points: vec![(3, 3), (2, 3), (1, 3), (0, 3)],
            found: true,
        };

        let reversed = path.points_start_to_end();
        assert_eq!(reversed, vec![(0, 3), (1, 3), (2, 3), (3, 3)]);
    }

    #[test]
    fn test_render_maze_path() {
        let maze = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        let path = MazePath {
            points: vec![(5, 5), (4, 5), (3, 5)],
            found: true,
        };

        let vis = render_maze_path(&maze, &path, (3, 5), (5, 5)).unwrap();

        assert_eq!(vis.width(), 10);
        assert_eq!(vis.height(), 10);
        assert_eq!(vis.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_local_search_for_background() {
        let mut pix = Pix::new(10, 10, PixelDepth::Bit1)
            .unwrap()
            .try_into_mut()
            .unwrap();

        // Make center 3x3 area walls
        for y in 4..7 {
            for x in 4..7 {
                let _ = pix.set_pixel(x, y, 1);
            }
        }

        let pix: Pix = pix.into();

        // Search from center (which is a wall)
        let mut x = 5u32;
        let mut y = 5u32;
        let found = local_search_for_background(&pix, &mut x, &mut y, 5);

        assert!(found);
        // Should find a background pixel nearby
        assert!(pix.get_pixel(x, y).unwrap_or(1) == 0);
    }
}
