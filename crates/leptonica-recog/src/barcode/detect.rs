//! Barcode detection and extraction

use crate::RecogResult;
use leptonica_core::{Box as PixBox, Boxa, Pix, Pixa};

/// Minimum barcode image dimensions
pub const MIN_BC_WIDTH: u32 = 50;
/// Minimum barcode image height
pub const MIN_BC_HEIGHT: u32 = 50;

/// Locates potential barcode regions in an image
pub fn locate_barcodes(_pix: &Pix, _threshold: i32) -> RecogResult<(Boxa, Option<Pix>)> {
    todo!("locate_barcodes not yet implemented")
}

/// Extract barcode sub-images from an image
pub fn extract_barcodes(_pix: &Pix) -> RecogResult<Pixa> {
    todo!("extract_barcodes not yet implemented")
}

/// Deskew a barcode region
pub fn deskew_barcode(
    _pix_gray: &Pix,
    _bbox: &PixBox,
    _margin: i32,
) -> RecogResult<(Pix, f32, f32)> {
    todo!("deskew_barcode not yet implemented")
}
