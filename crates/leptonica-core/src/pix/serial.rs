//! SPIX serialization - Leptonica's native binary image format
//!
//! Provides fast, uncompressed serialization of `Pix` images.
//! The format stores image metadata, an optional colormap, and raw raster data.
//!
//! # Format layout
//!
//! ```text
//! Offset       Size          Field
//! ------       ----          -----
//! 0            4             "spix" magic bytes
//! 4            4             width (u32)
//! 8            4             height (u32)
//! 12           4             depth (u32)
//! 16           4             wpl (u32, words per line)
//! 20           4             ncolors (u32, 0 if no colormap)
//! 24           4 * ncolors   colormap data (RGBA, 4 bytes per entry)
//! 24+4*n       4             raster data size (= 4 * wpl * h)
//! 28+4*n       rdatasize     raw raster data
//! ```
//!
//! # See also
//!
//! C Leptonica: `spixio.c` (`pixSerializeToMemory`, `pixDeserializeFromMemory`)

use crate::error::Result;
use crate::pix::{Pix, PixMut, PixelDepth};
use std::io::{Read, Write};
use std::path::Path;

/// SPIX header information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpixHeader {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub wpl: u32,
    pub ncolors: u32,
}

impl Pix {
    /// Deserialize a `Pix` from SPIX binary format via a reader.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixReadStreamSpix()` in `spixio.c`
    pub fn read_spix(_reader: &mut impl Read) -> Result<Self> {
        todo!()
    }

    /// Deserialize a `Pix` from SPIX bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixReadMemSpix()` / `pixDeserializeFromMemory()` in `spixio.c`
    pub fn read_spix_from_bytes(_data: &[u8]) -> Result<Self> {
        todo!()
    }

    /// Deserialize a `Pix` from an SPIX file.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixReadStreamSpix()` in `spixio.c`
    pub fn read_spix_from_file(_path: impl AsRef<Path>) -> Result<Self> {
        todo!()
    }

    /// Read only the SPIX header without loading raster data.
    ///
    /// # See also
    ///
    /// C Leptonica: `sreadHeaderSpix()` in `spixio.c`
    pub fn read_spix_header(_data: &[u8]) -> Result<SpixHeader> {
        todo!()
    }

    /// Serialize this `Pix` to SPIX binary format via a writer.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixWriteStreamSpix()` in `spixio.c`
    pub fn write_spix(&self, _writer: &mut impl Write) -> Result<()> {
        todo!()
    }

    /// Serialize this `Pix` to SPIX bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixWriteMemSpix()` / `pixSerializeToMemory()` in `spixio.c`
    pub fn write_spix_to_bytes(&self) -> Result<Vec<u8>> {
        todo!()
    }

    /// Write this `Pix` to an SPIX file.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixWriteStreamSpix()` in `spixio.c`
    pub fn write_spix_to_file(&self, _path: impl AsRef<Path>) -> Result<()> {
        todo!()
    }
}

impl PixMut {
    /// Create a new `PixMut` with specified dimensions and depth.
    ///
    /// This is equivalent to `Pix::new` but returns a mutable variant directly.
    pub fn new(width: u32, height: u32, depth: PixelDepth) -> Result<Self> {
        let pix = Pix::new(width, height, depth)?;
        Ok(pix.try_into_mut().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colormap::PixColormap;
    use crate::pix::ImageFormat;
    use std::io::Cursor;

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_roundtrip_1bpp() {
        let mut pix_mut = PixMut::new(64, 48, PixelDepth::Bit1).unwrap();
        pix_mut.data_mut()[0] = 0xAAAA_AAAA;
        pix_mut.data_mut()[1] = 0x5555_5555;
        let pix: Pix = pix_mut.into();

        let bytes = pix.write_spix_to_bytes().unwrap();
        let restored = Pix::read_spix_from_bytes(&bytes).unwrap();

        assert_eq!(pix.width(), restored.width());
        assert_eq!(pix.height(), restored.height());
        assert_eq!(pix.depth(), restored.depth());
        assert_eq!(pix.wpl(), restored.wpl());
        assert_eq!(pix.data(), restored.data());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_roundtrip_8bpp() {
        let mut pix_mut = PixMut::new(100, 50, PixelDepth::Bit8).unwrap();
        for y in 0..50u32 {
            for x in 0..100u32 {
                let val = ((x + y * 100) % 256) as u32;
                let wpl = pix_mut.wpl();
                let word_idx = (y * wpl + x / 4) as usize;
                let byte_pos = 3 - (x % 4) as usize;
                pix_mut.data_mut()[word_idx] |= val << (byte_pos * 8);
            }
        }
        let pix: Pix = pix_mut.into();

        let bytes = pix.write_spix_to_bytes().unwrap();
        let restored = Pix::read_spix_from_bytes(&bytes).unwrap();

        assert_eq!(pix.width(), restored.width());
        assert_eq!(pix.height(), restored.height());
        assert_eq!(pix.depth(), restored.depth());
        assert_eq!(pix.data(), restored.data());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_roundtrip_32bpp() {
        let mut pix_mut = PixMut::new(10, 10, PixelDepth::Bit32).unwrap();
        for (i, word) in pix_mut.data_mut().iter_mut().enumerate() {
            *word = (i as u32).wrapping_mul(0x0101_0101);
        }
        let pix: Pix = pix_mut.into();

        let bytes = pix.write_spix_to_bytes().unwrap();
        let restored = Pix::read_spix_from_bytes(&bytes).unwrap();

        assert_eq!(pix.width(), restored.width());
        assert_eq!(pix.height(), restored.height());
        assert_eq!(pix.depth(), restored.depth());
        assert_eq!(pix.data(), restored.data());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_roundtrip_with_colormap() {
        let mut pix_mut = PixMut::new(16, 16, PixelDepth::Bit8).unwrap();
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(255, 0, 0, 255).unwrap();
        cmap.add_rgba(0, 255, 0, 200).unwrap();
        cmap.add_rgba(0, 0, 255, 128).unwrap();
        pix_mut.set_colormap(Some(cmap)).unwrap();
        pix_mut.data_mut()[0] = 0x0001_0200;
        let pix: Pix = pix_mut.into();

        let bytes = pix.write_spix_to_bytes().unwrap();
        let restored = Pix::read_spix_from_bytes(&bytes).unwrap();

        assert_eq!(pix.width(), restored.width());
        assert_eq!(pix.height(), restored.height());
        assert!(restored.has_colormap());
        let rcmap = restored.colormap().unwrap();
        assert_eq!(rcmap.len(), 3);
        assert_eq!(rcmap.get_rgba(0), Some((255, 0, 0, 255)));
        assert_eq!(rcmap.get_rgba(1), Some((0, 255, 0, 200)));
        assert_eq!(rcmap.get_rgba(2), Some((0, 0, 255, 128)));
        assert_eq!(pix.data(), restored.data());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_header_read() {
        let pix = Pix::new(320, 240, PixelDepth::Bit8).unwrap();
        let bytes = pix.write_spix_to_bytes().unwrap();

        let header = Pix::read_spix_header(&bytes).unwrap();
        assert_eq!(header.width, 320);
        assert_eq!(header.height, 240);
        assert_eq!(header.depth, 8);
        assert_eq!(header.ncolors, 0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_invalid_magic() {
        let data = b"notspix_invalid_data_here_padding";
        assert!(Pix::read_spix_from_bytes(data).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_truncated_data() {
        let data = b"spix";
        assert!(Pix::read_spix_from_bytes(data).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_file_roundtrip() {
        let mut pix_mut = PixMut::new(32, 32, PixelDepth::Bit8).unwrap();
        for (i, word) in pix_mut.data_mut().iter_mut().enumerate() {
            *word = (i as u32).wrapping_mul(0x0403_0201);
        }
        let pix: Pix = pix_mut.into();

        let dir = std::env::temp_dir().join("leptonica_test_spix");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_roundtrip.spix");

        pix.write_spix_to_file(&path).unwrap();
        let restored = Pix::read_spix_from_file(&path).unwrap();

        assert_eq!(pix.width(), restored.width());
        assert_eq!(pix.height(), restored.height());
        assert_eq!(pix.depth(), restored.depth());
        assert_eq!(pix.data(), restored.data());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_reader_roundtrip() {
        let pix = Pix::new(20, 15, PixelDepth::Bit32).unwrap();

        let mut buf = Vec::new();
        pix.write_spix(&mut buf).unwrap();

        let restored = Pix::read_spix(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(pix.width(), restored.width());
        assert_eq!(pix.height(), restored.height());
        assert_eq!(pix.depth(), restored.depth());
        assert_eq!(pix.data(), restored.data());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_informat_is_set() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let bytes = pix.write_spix_to_bytes().unwrap();
        let restored = Pix::read_spix_from_bytes(&bytes).unwrap();
        assert_eq!(restored.informat(), ImageFormat::Spix);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_spix_all_depths() {
        for depth in [
            PixelDepth::Bit1,
            PixelDepth::Bit2,
            PixelDepth::Bit4,
            PixelDepth::Bit8,
            PixelDepth::Bit16,
            PixelDepth::Bit32,
        ] {
            let pix = Pix::new(64, 48, depth).unwrap();
            let bytes = pix.write_spix_to_bytes().unwrap();
            let restored = Pix::read_spix_from_bytes(&bytes).unwrap();
            assert_eq!(pix.width(), restored.width());
            assert_eq!(pix.height(), restored.height());
            assert_eq!(pix.depth(), restored.depth());
            assert_eq!(pix.wpl(), restored.wpl());
        }
    }
}
