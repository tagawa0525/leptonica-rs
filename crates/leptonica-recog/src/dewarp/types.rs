//! Dewarp type definitions

use leptonica_core::FPix;

/// Options for dewarping operations
#[derive(Debug, Clone)]
pub struct DewarpOptions {
    /// Sampling interval for building disparity arrays (default: 30)
    pub sampling: u32,
    /// Reduction factor for input images (1 or 2, default: 1)
    pub reduction_factor: u32,
    /// Minimum number of text lines required (default: 15)
    pub min_lines: u32,
    /// Whether to use both vertical and horizontal disparity (default: true)
    pub use_both: bool,
    /// Maximum line curvature in micro-units (default: 150)
    pub max_line_curvature: i32,
    /// Maximum edge curvature in micro-units (default: 50)
    pub max_edge_curvature: i32,
    /// Maximum edge slope in milli-units (default: 80)
    pub max_edge_slope: i32,
    /// Gray value for pixels brought in from outside (default: 255)
    pub gray_in: u8,
}

impl Default for DewarpOptions {
    fn default() -> Self {
        Self {
            sampling: 30,
            reduction_factor: 1,
            min_lines: 15,
            use_both: true,
            max_line_curvature: 150,
            max_edge_curvature: 50,
            max_edge_slope: 80,
            gray_in: 255,
        }
    }
}

impl DewarpOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sampling interval
    pub fn with_sampling(mut self, sampling: u32) -> Self {
        self.sampling = sampling.max(8);
        self
    }

    /// Set the reduction factor
    pub fn with_reduction_factor(mut self, factor: u32) -> Self {
        self.reduction_factor = if factor == 2 { 2 } else { 1 };
        self
    }

    /// Set the minimum number of lines
    pub fn with_min_lines(mut self, min_lines: u32) -> Self {
        self.min_lines = min_lines.max(4);
        self
    }

    /// Set whether to use both disparity types
    pub fn with_use_both(mut self, use_both: bool) -> Self {
        self.use_both = use_both;
        self
    }

    /// Set the gray value for outside pixels
    pub fn with_gray_in(mut self, gray_in: u8) -> Self {
        self.gray_in = gray_in;
        self
    }
}

/// Dewarp data for a single page
#[derive(Debug)]
#[allow(dead_code)]
pub struct Dewarp {
    pub(crate) page_number: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) nx: u32,
    pub(crate) ny: u32,
    pub(crate) sampling: u32,
    pub(crate) reduction_factor: u32,
    pub(crate) min_lines: u32,
    pub(crate) n_lines: u32,
    pub(crate) sampled_v_disparity: Option<FPix>,
    pub(crate) sampled_h_disparity: Option<FPix>,
    pub(crate) full_v_disparity: Option<FPix>,
    pub(crate) full_h_disparity: Option<FPix>,
    pub(crate) min_curvature: i32,
    pub(crate) max_curvature: i32,
    pub(crate) left_slope: i32,
    pub(crate) right_slope: i32,
    pub(crate) left_curvature: i32,
    pub(crate) right_curvature: i32,
    pub(crate) v_success: bool,
    pub(crate) h_success: bool,
    pub(crate) v_valid: bool,
    pub(crate) h_valid: bool,
}

impl Dewarp {
    /// Create a new Dewarp for an image
    pub fn new(width: u32, height: u32, page_number: u32, options: &DewarpOptions) -> Self {
        let sampling = options.sampling;
        let nx = (width + 2 * sampling - 2) / sampling;
        let ny = (height + 2 * sampling - 2) / sampling;

        Self {
            page_number,
            width,
            height,
            nx,
            ny,
            sampling,
            reduction_factor: options.reduction_factor,
            min_lines: options.min_lines,
            n_lines: 0,
            sampled_v_disparity: None,
            sampled_h_disparity: None,
            full_v_disparity: None,
            full_h_disparity: None,
            min_curvature: 0,
            max_curvature: 0,
            left_slope: 0,
            right_slope: 0,
            left_curvature: 0,
            right_curvature: 0,
            v_success: false,
            h_success: false,
            v_valid: false,
            h_valid: false,
        }
    }

    /// Get the page number
    pub fn page_number(&self) -> u32 {
        self.page_number
    }
    /// Get the original image width
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Get the original image height
    pub fn height(&self) -> u32 {
        self.height
    }
    /// Get the number of text lines found
    pub fn n_lines(&self) -> u32 {
        self.n_lines
    }
    /// Check if vertical disparity model was built successfully
    pub fn v_success(&self) -> bool {
        self.v_success
    }
    /// Check if horizontal disparity model was built successfully
    pub fn h_success(&self) -> bool {
        self.h_success
    }
    /// Check if vertical model is valid for rendering
    pub fn v_valid(&self) -> bool {
        self.v_valid
    }
    /// Check if horizontal model is valid for rendering
    pub fn h_valid(&self) -> bool {
        self.h_valid
    }
    /// Get the minimum line curvature in micro-units
    pub fn min_curvature(&self) -> i32 {
        self.min_curvature
    }
    /// Get the maximum line curvature in micro-units
    pub fn max_curvature(&self) -> i32 {
        self.max_curvature
    }
    /// Get sampled vertical disparity
    pub fn sampled_v_disparity(&self) -> Option<&FPix> {
        self.sampled_v_disparity.as_ref()
    }
    /// Get sampled horizontal disparity
    pub fn sampled_h_disparity(&self) -> Option<&FPix> {
        self.sampled_h_disparity.as_ref()
    }
    /// Get full resolution vertical disparity
    pub fn full_v_disparity(&self) -> Option<&FPix> {
        self.full_v_disparity.as_ref()
    }
    /// Get full resolution horizontal disparity
    pub fn full_h_disparity(&self) -> Option<&FPix> {
        self.full_h_disparity.as_ref()
    }

    /// Minimize storage by removing full resolution arrays
    pub fn minimize(&mut self) {
        self.full_v_disparity = None;
        self.full_h_disparity = None;
    }
}

/// Result of dewarping operation
#[derive(Debug)]
pub struct DewarpResult {
    /// The dewarped image
    pub pix: leptonica_core::Pix,
    /// The dewarp model used
    pub dewarp: Dewarp,
    /// Whether vertical correction was applied
    pub v_applied: bool,
    /// Whether horizontal correction was applied
    pub h_applied: bool,
}

impl DewarpResult {
    /// Create a new dewarp result
    pub fn new(pix: leptonica_core::Pix, dewarp: Dewarp, v_applied: bool, h_applied: bool) -> Self {
        Self {
            pix,
            dewarp,
            v_applied,
            h_applied,
        }
    }

    /// Check if any correction was applied
    pub fn was_corrected(&self) -> bool {
        self.v_applied || self.h_applied
    }
}

/// Text line representation
#[derive(Debug, Clone)]
pub struct TextLine {
    /// Points along the center of the text line
    pub points: Vec<(f32, f32)>,
}

impl TextLine {
    /// Create a new text line from points
    pub fn new(points: Vec<(f32, f32)>) -> Self {
        Self { points }
    }
    /// Get the number of points
    pub fn len(&self) -> usize {
        self.points.len()
    }
    /// Check if the line is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Get the horizontal extent (max_x - min_x)
    pub fn horizontal_extent(&self) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        let min_x = self.points.iter().map(|(x, _)| *x).fold(f32::MAX, f32::min);
        let max_x = self.points.iter().map(|(x, _)| *x).fold(f32::MIN, f32::max);
        max_x - min_x
    }

    /// Get the y-coordinate at the middle x-position
    pub fn mid_y(&self) -> Option<f32> {
        if self.points.is_empty() {
            return None;
        }
        Some(self.points[self.points.len() / 2].1)
    }
}
