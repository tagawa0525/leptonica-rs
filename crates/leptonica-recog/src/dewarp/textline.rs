//! Text line detection for dewarping

use super::types::TextLine;
use crate::RecogResult;
use leptonica_core::Pix;

/// Find text line centers in an image
pub fn find_textline_centers(_pix: &Pix) -> RecogResult<Vec<TextLine>> {
    todo!("find_textline_centers not yet implemented")
}

/// Remove lines shorter than a fraction of the longest line
pub fn remove_short_lines(lines: Vec<TextLine>, _min_fraction: f32) -> Vec<TextLine> {
    lines // stub: return unchanged
}

/// Check if line coverage is valid for dewarping
pub fn is_line_coverage_valid(_lines: &[TextLine], _image_height: u32, _min_lines: u32) -> bool {
    false // stub
}

/// Sort text lines by y-coordinate
pub fn sort_lines_by_y(lines: &mut [TextLine]) {
    lines.sort_by(|a, b| {
        let ay = a.mid_y().unwrap_or(0.0);
        let by = b.mid_y().unwrap_or(0.0);
        ay.partial_cmp(&by).unwrap_or(std::cmp::Ordering::Equal)
    });
}
