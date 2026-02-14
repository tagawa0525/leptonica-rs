//! Image format detection
//!
//! Detects image formats by examining magic numbers in the file header.

use crate::{IoError, IoResult};
use leptonica_core::ImageFormat;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Magic numbers for image format detection
mod magic {
    /// BMP: "BM"
    pub const BMP: &[u8] = b"BM";

    /// PNG: 89 50 4E 47 0D 0A 1A 0A
    pub const PNG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    /// JPEG: FF D8 FF
    pub const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF];

    /// GIF87a
    pub const GIF87A: &[u8] = b"GIF87a";

    /// GIF89a
    pub const GIF89A: &[u8] = b"GIF89a";

    /// TIFF little-endian: II 2A 00
    pub const TIFF_LE: &[u8] = &[0x49, 0x49, 0x2A, 0x00];

    /// TIFF big-endian: MM 00 2A
    pub const TIFF_BE: &[u8] = &[0x4D, 0x4D, 0x00, 0x2A];

    /// WebP: RIFF....WEBP
    pub const RIFF: &[u8] = b"RIFF";
    pub const WEBP: &[u8] = b"WEBP";

    /// PNM formats
    pub const PBM_ASCII: &[u8] = b"P1";
    pub const PGM_ASCII: &[u8] = b"P2";
    pub const PPM_ASCII: &[u8] = b"P3";
    pub const PBM_BINARY: &[u8] = b"P4";
    pub const PGM_BINARY: &[u8] = b"P5";
    pub const PPM_BINARY: &[u8] = b"P6";

    /// JPEG 2000 Part 1 (JP2) signature box
    /// Starts with: 00 00 00 0C 6A 50 20 20 0D 0A 87 0A
    pub const JP2_SIGNATURE: &[u8] = &[0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20];

    /// JPEG 2000 codestream (J2K) signature
    /// Starts with: FF 4F FF 51
    pub const J2K_SIGNATURE: &[u8] = &[0xFF, 0x4F, 0xFF, 0x51];
}

/// Detect image format from a file path
pub fn detect_format<P: AsRef<Path>>(path: P) -> IoResult<ImageFormat> {
    let mut file = File::open(path).map_err(IoError::Io)?;
    let mut header = [0u8; 12];
    let bytes_read = file.read(&mut header).map_err(IoError::Io)?;
    detect_format_from_bytes(&header[..bytes_read])
}

/// Detect image format from bytes
pub fn detect_format_from_bytes(data: &[u8]) -> IoResult<ImageFormat> {
    if data.len() < 2 {
        return Err(IoError::InvalidData(
            "not enough data to detect format".to_string(),
        ));
    }

    // Check BMP
    if data.starts_with(magic::BMP) {
        return Ok(ImageFormat::Bmp);
    }

    // Check PNG (needs 8 bytes)
    if data.len() >= 8 && data.starts_with(magic::PNG) {
        return Ok(ImageFormat::Png);
    }

    // Check JPEG
    if data.len() >= 3 && data.starts_with(magic::JPEG) {
        return Ok(ImageFormat::Jpeg);
    }

    // Check GIF
    if data.len() >= 6 && (data.starts_with(magic::GIF87A) || data.starts_with(magic::GIF89A)) {
        return Ok(ImageFormat::Gif);
    }

    // Check TIFF
    if data.len() >= 4 && (data.starts_with(magic::TIFF_LE) || data.starts_with(magic::TIFF_BE)) {
        return Ok(ImageFormat::Tiff);
    }

    // Check WebP (RIFF....WEBP)
    if data.len() >= 12 && data.starts_with(magic::RIFF) && &data[8..12] == magic::WEBP {
        return Ok(ImageFormat::WebP);
    }

    // Check JPEG 2000 (JP2 container or J2K codestream)
    if data.len() >= 8 && data.starts_with(magic::JP2_SIGNATURE) {
        return Ok(ImageFormat::Jp2);
    }
    if data.len() >= 4 && data.starts_with(magic::J2K_SIGNATURE) {
        return Ok(ImageFormat::Jp2);
    }

    // Check PNM formats
    if data.len() >= 2 {
        let first_two = &data[..2];
        if first_two == magic::PBM_ASCII
            || first_two == magic::PGM_ASCII
            || first_two == magic::PPM_ASCII
            || first_two == magic::PBM_BINARY
            || first_two == magic::PGM_BINARY
            || first_two == magic::PPM_BINARY
        {
            return Ok(ImageFormat::Pnm);
        }
    }

    Err(IoError::UnsupportedFormat(
        "unknown image format".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_bmp() {
        let data = b"BM\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(detect_format_from_bytes(data).unwrap(), ImageFormat::Bmp);
    }

    #[test]
    fn test_detect_png() {
        let data = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(detect_format_from_bytes(&data).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn test_detect_jpeg() {
        let data = [
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        ];
        assert_eq!(detect_format_from_bytes(&data).unwrap(), ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_gif() {
        let data = b"GIF89a\x00\x00\x00\x00\x00\x00";
        assert_eq!(detect_format_from_bytes(data).unwrap(), ImageFormat::Gif);
    }

    #[test]
    fn test_detect_tiff_le() {
        let data = [
            0x49, 0x49, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(detect_format_from_bytes(&data).unwrap(), ImageFormat::Tiff);
    }

    #[test]
    fn test_detect_pnm() {
        assert_eq!(
            detect_format_from_bytes(b"P5\n100 100\n255\n").unwrap(),
            ImageFormat::Pnm
        );
        assert_eq!(
            detect_format_from_bytes(b"P6\n100 100\n255\n").unwrap(),
            ImageFormat::Pnm
        );
    }

    #[test]
    fn test_detect_unknown() {
        let data = b"UNKNOWN_FORMAT";
        assert!(detect_format_from_bytes(data).is_err());
    }

    #[test]
    fn test_detect_jp2() {
        // JP2 signature box
        let data = [
            0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20, 0x0D, 0x0A, 0x87, 0x0A,
        ];
        assert_eq!(detect_format_from_bytes(&data).unwrap(), ImageFormat::Jp2);
    }

    #[test]
    fn test_detect_j2k() {
        // J2K codestream signature
        let data = [0xFF, 0x4F, 0xFF, 0x51, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(detect_format_from_bytes(&data).unwrap(), ImageFormat::Jp2);
    }
}
