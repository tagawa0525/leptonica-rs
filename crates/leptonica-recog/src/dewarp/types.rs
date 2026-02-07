//! Dewarp type definitions
//!
//! This module defines the core types for dewarping operations.

use leptonica_core::FPix;

/// Options for dewarping operations
///
/// These control the behavior of the dewarping algorithm.
#[derive(Debug, Clone)]
pub struct DewarpOptions {
    /// Sampling interval for building disparity arrays (default: 30)
    ///
    /// Larger values are faster but less accurate.
    /// Minimum allowed is 8.
    pub sampling: u32,

    /// Reduction factor for input images (1 or 2, default: 1)
    ///
    /// Use 1 for full resolution, 2 for 2x reduced input.
    pub reduction_factor: u32,

    /// Minimum number of text lines required to build a model (default: 15)
    ///
    /// Pages with fewer lines will not be dewarped.
    /// Minimum allowed is 4.
    pub min_lines: u32,

    /// Whether to use both vertical and horizontal disparity (default: true)
    ///
    /// If false, only vertical disparity is applied.
    pub use_both: bool,

    /// Maximum line curvature in micro-units (default: 150)
    ///
    /// Lines with greater curvature are considered invalid.
    pub max_line_curvature: i32,

    /// Maximum edge curvature in micro-units (default: 50)
    pub max_edge_curvature: i32,

    /// Maximum edge slope in milli-units (default: 80)
    pub max_edge_slope: i32,

    /// Gray value for pixels brought in from outside (0-255, default: 255)
    ///
    /// Use 0 for black, 255 for white.
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

    /// Set whether to use both vertical and horizontal disparity
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
///
/// Contains the disparity arrays and metadata needed to dewarp an image.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Dewarp {
    /// Page number (0-indexed)
    pub(crate) page_number: u32,

    /// Original image width
    pub(crate) width: u32,

    /// Original image height
    pub(crate) height: u32,

    /// Sampled width (width / sampling)
    pub(crate) nx: u32,

    /// Sampled height (height / sampling)
    pub(crate) ny: u32,

    /// Sampling interval
    pub(crate) sampling: u32,

    /// Reduction factor
    pub(crate) reduction_factor: u32,

    /// Minimum lines required
    pub(crate) min_lines: u32,

    /// Number of text lines found
    pub(crate) n_lines: u32,

    /// Sampled vertical disparity array
    pub(crate) sampled_v_disparity: Option<FPix>,

    /// Sampled horizontal disparity array
    pub(crate) sampled_h_disparity: Option<FPix>,

    /// Full resolution vertical disparity array
    pub(crate) full_v_disparity: Option<FPix>,

    /// Full resolution horizontal disparity array
    pub(crate) full_h_disparity: Option<FPix>,

    /// Minimum line curvature (micro-units)
    pub(crate) min_curvature: i32,

    /// Maximum line curvature (micro-units)
    pub(crate) max_curvature: i32,

    /// Left edge slope (milli-units)
    pub(crate) left_slope: i32,

    /// Right edge slope (milli-units)
    pub(crate) right_slope: i32,

    /// Left edge curvature (micro-units)
    pub(crate) left_curvature: i32,

    /// Right edge curvature (micro-units)
    pub(crate) right_curvature: i32,

    /// Vertical disparity model successfully built
    pub(crate) v_success: bool,

    /// Horizontal disparity model successfully built
    pub(crate) h_success: bool,

    /// Vertical model is valid for rendering
    pub(crate) v_valid: bool,

    /// Horizontal model is valid for rendering
    pub(crate) h_valid: bool,
}

impl Dewarp {
    /// Create a new Dewarp for an image
    ///
    /// # Arguments
    ///
    /// * `width` - Image width
    /// * `height` - Image height
    /// * `page_number` - Page number (typically 0-indexed)
    /// * `options` - Dewarping options
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

    /// Get a reference to the sampled vertical disparity array
    pub fn sampled_v_disparity(&self) -> Option<&FPix> {
        self.sampled_v_disparity.as_ref()
    }

    /// Get a reference to the sampled horizontal disparity array
    pub fn sampled_h_disparity(&self) -> Option<&FPix> {
        self.sampled_h_disparity.as_ref()
    }

    /// Get a reference to the full resolution vertical disparity array
    pub fn full_v_disparity(&self) -> Option<&FPix> {
        self.full_v_disparity.as_ref()
    }

    /// Get a reference to the full resolution horizontal disparity array
    pub fn full_h_disparity(&self) -> Option<&FPix> {
        self.full_h_disparity.as_ref()
    }

    /// Minimize storage by removing full resolution arrays
    ///
    /// The sampled arrays are kept so full resolution can be regenerated.
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

    /// The dewarp model used (contains disparity arrays)
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
///
/// A text line is represented as a series of (x, y) points
/// tracing the center of the text.
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

        let n = self.points.len();
        Some(self.points[n / 2].1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dewarp_options_default() {
        let opts = DewarpOptions::default();
        assert_eq!(opts.sampling, 30);
        assert_eq!(opts.reduction_factor, 1);
        assert_eq!(opts.min_lines, 15);
        assert!(opts.use_both);
        assert_eq!(opts.gray_in, 255);
    }

    #[test]
    fn test_dewarp_options_builder() {
        let opts = DewarpOptions::new()
            .with_sampling(20)
            .with_reduction_factor(2)
            .with_min_lines(10)
            .with_use_both(false)
            .with_gray_in(128);

        assert_eq!(opts.sampling, 20);
        assert_eq!(opts.reduction_factor, 2);
        assert_eq!(opts.min_lines, 10);
        assert!(!opts.use_both);
        assert_eq!(opts.gray_in, 128);
    }

    #[test]
    fn test_dewarp_options_validation() {
        // Sampling minimum is 8
        let opts = DewarpOptions::new().with_sampling(5);
        assert_eq!(opts.sampling, 8);

        // Min lines minimum is 4
        let opts = DewarpOptions::new().with_min_lines(2);
        assert_eq!(opts.min_lines, 4);

        // Reduction factor must be 1 or 2
        let opts = DewarpOptions::new().with_reduction_factor(3);
        assert_eq!(opts.reduction_factor, 1);
    }

    #[test]
    fn test_dewarp_creation() {
        let opts = DewarpOptions::default();
        let dew = Dewarp::new(800, 600, 0, &opts);

        assert_eq!(dew.page_number(), 0);
        assert_eq!(dew.width(), 800);
        assert_eq!(dew.height(), 600);
        assert!(!dew.v_success());
        assert!(!dew.h_success());
    }

    #[test]
    fn test_text_line() {
        let points = vec![(0.0, 10.0), (50.0, 12.0), (100.0, 11.0)];
        let line = TextLine::new(points);

        assert_eq!(line.len(), 3);
        assert!(!line.is_empty());
        assert_eq!(line.horizontal_extent(), 100.0);
        assert_eq!(line.mid_y(), Some(12.0));
    }

    #[test]
    fn test_text_line_empty() {
        let line = TextLine::new(vec![]);

        assert!(line.is_empty());
        assert_eq!(line.horizontal_extent(), 0.0);
        assert_eq!(line.mid_y(), None);
    }
}
