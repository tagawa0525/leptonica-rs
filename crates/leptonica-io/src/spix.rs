//! SPIX image format support
//!
//! Reading and writing Leptonica's native SPIX binary format.
//! SPIX is a fast, uncompressed serialization of `Pix` images.
//!
//! # See also
//!
//! C Leptonica: `spixio.c`

use crate::{IoError, IoResult, header::ImageHeader};
use leptonica_core::{ImageFormat, Pix};
use std::io::{Read, Write};

/// Read SPIX header metadata without decoding pixel data
///
/// # See also
///
/// C Leptonica: `sreadHeaderSpix()` in `spixio.c`
pub fn read_header_spix(data: &[u8]) -> IoResult<ImageHeader> {
    let _ = data;
    Err(IoError::UnsupportedFormat(
        "SPIX header reading not yet implemented".to_string(),
    ))
}

/// Read a SPIX image
///
/// # See also
///
/// C Leptonica: `pixReadStreamSpix()` in `spixio.c`
pub fn read_spix<R: Read>(mut reader: R) -> IoResult<Pix> {
    Ok(Pix::read_spix(&mut reader)?)
}

/// Write a Pix as SPIX
///
/// # See also
///
/// C Leptonica: `pixWriteStreamSpix()` in `spixio.c`
pub fn write_spix<W: Write>(pix: &Pix, mut writer: W) -> IoResult<()> {
    Ok(pix.write_spix(&mut writer)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{Pix, PixelDepth};
    use std::io::Cursor;

    #[test]
    fn test_spix_roundtrip_1bpp() {
        let pix = Pix::new(64, 48, PixelDepth::Bit1).unwrap();
        let mut buf = Vec::new();
        write_spix(&pix, &mut buf).unwrap();
        assert!(buf.starts_with(b"spix"));
        let pix2 = read_spix(Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.width(), 64);
        assert_eq!(pix2.height(), 48);
        assert_eq!(pix2.depth(), PixelDepth::Bit1);
        assert_eq!(pix.data(), pix2.data());
    }

    #[test]
    fn test_spix_roundtrip_8bpp() {
        let pix = Pix::new(100, 50, PixelDepth::Bit8).unwrap();
        let mut buf = Vec::new();
        write_spix(&pix, &mut buf).unwrap();
        let pix2 = read_spix(Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.width(), 100);
        assert_eq!(pix2.height(), 50);
        assert_eq!(pix2.depth(), PixelDepth::Bit8);
        assert_eq!(pix.data(), pix2.data());
    }

    #[test]
    fn test_spix_roundtrip_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut buf = Vec::new();
        write_spix(&pix, &mut buf).unwrap();
        let pix2 = read_spix(Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);
        assert_eq!(pix2.depth(), PixelDepth::Bit32);
        assert_eq!(pix.data(), pix2.data());
    }

    #[test]
    fn test_spix_invalid_magic() {
        let data = b"notspix_invalid_data_here_padding";
        assert!(read_spix(Cursor::new(data)).is_err());
    }

    #[test]
    fn test_spix_truncated() {
        let data = b"spix";
        assert!(read_spix(Cursor::new(data)).is_err());
    }
}
