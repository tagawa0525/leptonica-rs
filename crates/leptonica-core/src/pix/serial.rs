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

use crate::PixColormap;
use crate::error::{Error, Result};
use crate::pix::{ImageFormat, Pix, PixMut, PixelDepth};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Magic bytes identifying SPIX format
const SPIX_MAGIC: &[u8; 4] = b"spix";

/// Maximum allowed image width
const MAX_ALLOWED_WIDTH: u32 = 1_000_000;
/// Maximum allowed image height
const MAX_ALLOWED_HEIGHT: u32 = 1_000_000;
/// Maximum allowed image area (width * height)
const MAX_ALLOWED_AREA: u64 = 400_000_000;

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
    /// Reads the entire content from the reader and deserializes it.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixReadStreamSpix()` in `spixio.c`
    pub fn read_spix(reader: &mut impl Read) -> Result<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Self::read_spix_from_bytes(&buf)
    }

    /// Deserialize a `Pix` from SPIX bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixReadMemSpix()` / `pixDeserializeFromMemory()` in `spixio.c`
    pub fn read_spix_from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 28 {
            return Err(Error::DecodeError("SPIX data too short".into()));
        }
        if data.len() > i32::MAX as usize {
            return Err(Error::DecodeError("SPIX data too large".into()));
        }

        // Check magic
        if &data[0..4] != SPIX_MAGIC {
            return Err(Error::DecodeError("invalid SPIX magic bytes".into()));
        }

        // Read header
        let header = read_header_from_words(data);
        let w = header.width;
        let h = header.height;
        let d = header.depth;
        let ncolors = header.ncolors;

        // Validate dimensions
        if !(1..=MAX_ALLOWED_WIDTH).contains(&w) || !(1..=MAX_ALLOWED_HEIGHT).contains(&h) {
            return Err(Error::InvalidDimension {
                width: w,
                height: h,
            });
        }
        if (w as u64) * (h as u64) > MAX_ALLOWED_AREA {
            return Err(Error::InvalidDimension {
                width: w,
                height: h,
            });
        }

        // Validate ncolors
        if ncolors > 256 {
            return Err(Error::DecodeError(format!("invalid ncolors: {ncolors}")));
        }

        let depth = PixelDepth::from_bits(d)?;
        let expected_wpl = Pix::compute_wpl(w, depth);
        let pixdata_size = 4 * expected_wpl as usize * h as usize;

        // Verify we have enough data for header + colormap + rdata_size field
        let rdata_size_word = 6 + ncolors as usize;
        if data.len() < (rdata_size_word + 1) * 4 {
            return Err(Error::DecodeError("SPIX data truncated".into()));
        }

        let memdata_size = data.len() - 24 - 4 * ncolors as usize - 4;
        let imdata_size = read_u32_at(data, rdata_size_word) as usize;

        if pixdata_size != memdata_size || pixdata_size != imdata_size {
            return Err(Error::DecodeError(format!(
                "SPIX data size mismatch: computed={pixdata_size}, mem={memdata_size}, recorded={imdata_size}"
            )));
        }

        // Create the Pix
        let mut pix_mut = PixMut::new(w, h, depth)?;
        pix_mut.set_informat(ImageFormat::Spix);

        // Read colormap if present
        if ncolors > 0 {
            let cmap = deserialize_colormap(data, ncolors, d)?;
            pix_mut.set_colormap(Some(cmap))?;
        }

        // Copy raster data
        let raster_start = (rdata_size_word + 1) * 4;
        let raster_bytes = &data[raster_start..raster_start + imdata_size];
        copy_bytes_to_u32_slice(raster_bytes, pix_mut.data_mut());

        Ok(pix_mut.into())
    }

    /// Deserialize a `Pix` from an SPIX file.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixReadStreamSpix()` in `spixio.c`
    pub fn read_spix_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        Self::read_spix(&mut BufReader::new(file))
    }

    /// Read only the SPIX header without loading raster data.
    ///
    /// # See also
    ///
    /// C Leptonica: `sreadHeaderSpix()` in `spixio.c`
    pub fn read_spix_header(data: &[u8]) -> Result<SpixHeader> {
        if data.len() < 28 {
            return Err(Error::DecodeError("SPIX data too short for header".into()));
        }
        if &data[0..4] != SPIX_MAGIC {
            return Err(Error::DecodeError("invalid SPIX magic bytes".into()));
        }
        Ok(read_header_from_words(data))
    }

    /// Serialize this `Pix` to SPIX binary format via a writer.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixWriteStreamSpix()` in `spixio.c`
    pub fn write_spix(&self, writer: &mut impl Write) -> Result<()> {
        let bytes = self.write_spix_to_bytes()?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    /// Serialize this `Pix` to SPIX bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixWriteMemSpix()` / `pixSerializeToMemory()` in `spixio.c`
    pub fn write_spix_to_bytes(&self) -> Result<Vec<u8>> {
        let w = self.width();
        let h = self.height();
        let d = self.depth().bits();
        let wpl = self.wpl();
        let rdatasize = 4 * wpl as usize * h as usize;

        // Serialize colormap if present
        let (ncolors, cdata) = if let Some(cmap) = self.colormap() {
            serialize_colormap(cmap)
        } else {
            (0u32, Vec::new())
        };

        let nbytes = 24 + 4 * ncolors as usize + 4 + rdatasize;
        let mut data = vec![0u8; nbytes];

        // Write magic
        data[0..4].copy_from_slice(SPIX_MAGIC);

        // Write header fields
        write_u32_at(&mut data, 1, w);
        write_u32_at(&mut data, 2, h);
        write_u32_at(&mut data, 3, d);
        write_u32_at(&mut data, 4, wpl);
        write_u32_at(&mut data, 5, ncolors);

        // Write colormap data
        if ncolors > 0 {
            let cmap_offset = 24;
            data[cmap_offset..cmap_offset + cdata.len()].copy_from_slice(&cdata);
        }

        // Write raster data size
        let rdata_size_word = 6 + ncolors as usize;
        write_u32_at(&mut data, rdata_size_word, rdatasize as u32);

        // Write raster data
        let raster_start = (rdata_size_word + 1) * 4;
        copy_u32_slice_to_bytes(
            self.data(),
            &mut data[raster_start..raster_start + rdatasize],
        );

        Ok(data)
    }

    /// Write this `Pix` to an SPIX file.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixWriteStreamSpix()` in `spixio.c`
    pub fn write_spix_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        self.write_spix(&mut BufWriter::new(file))
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

// --- Helper functions ---

/// Read a native-endian u32 at word index `i` from a byte slice.
#[inline]
fn read_u32_at(data: &[u8], word_index: usize) -> u32 {
    let offset = word_index * 4;
    u32::from_ne_bytes(data[offset..offset + 4].try_into().unwrap())
}

/// Write a native-endian u32 at word index `i` into a byte slice.
#[inline]
fn write_u32_at(data: &mut [u8], word_index: usize, value: u32) {
    let offset = word_index * 4;
    data[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
}

/// Read SPIX header fields from byte data (assumes magic already validated).
fn read_header_from_words(data: &[u8]) -> SpixHeader {
    SpixHeader {
        width: read_u32_at(data, 1),
        height: read_u32_at(data, 2),
        depth: read_u32_at(data, 3),
        wpl: read_u32_at(data, 4),
        ncolors: read_u32_at(data, 5),
    }
}

/// Deserialize a colormap from SPIX data.
///
/// In SPIX format, colormap data starts at byte offset 24 (word offset 6),
/// with 4 bytes per color entry (R, G, B, A).
///
/// The `image_depth` parameter is used instead of inferring from ncolors,
/// because the colormap depth must match the image depth for `set_colormap`.
///
/// # See also
///
/// C Leptonica: `pixcmapDeserializeFromMemory()` in `colormap.c`
fn deserialize_colormap(data: &[u8], ncolors: u32, image_depth: u32) -> Result<PixColormap> {
    let n = ncolors as usize;

    let mut cmap = PixColormap::new(image_depth)?;
    let cmap_start = 24;
    for i in 0..n {
        let base = cmap_start + i * 4;
        cmap.add_rgba(data[base], data[base + 1], data[base + 2], data[base + 3])?;
    }

    Ok(cmap)
}

/// Serialize a colormap to SPIX format (4 bytes per color: R, G, B, A).
///
/// # See also
///
/// C Leptonica: `pixcmapSerializeToMemory()` in `colormap.c` with cpc=4
fn serialize_colormap(cmap: &PixColormap) -> (u32, Vec<u8>) {
    let ncolors = cmap.len() as u32;
    let mut data = Vec::with_capacity(ncolors as usize * 4);

    for color in cmap.colors() {
        data.push(color.red);
        data.push(color.green);
        data.push(color.blue);
        data.push(color.alpha);
    }

    (ncolors, data)
}

/// Copy bytes into a u32 slice using native endian byte order.
fn copy_bytes_to_u32_slice(bytes: &[u8], dest: &mut [u32]) {
    for (i, chunk) in bytes.chunks_exact(4).enumerate() {
        dest[i] = u32::from_ne_bytes(chunk.try_into().unwrap());
    }
}

/// Copy a u32 slice into bytes using native endian byte order.
fn copy_u32_slice_to_bytes(src: &[u32], dest: &mut [u8]) {
    for (i, &word) in src.iter().enumerate() {
        let offset = i * 4;
        dest[offset..offset + 4].copy_from_slice(&word.to_ne_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colormap::PixColormap;
    use crate::pix::ImageFormat;
    use std::io::Cursor;

    #[test]
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
    fn test_spix_invalid_magic() {
        let data = b"notspix_invalid_data_here_padding";
        assert!(Pix::read_spix_from_bytes(data).is_err());
    }

    #[test]
    fn test_spix_truncated_data() {
        let data = b"spix";
        assert!(Pix::read_spix_from_bytes(data).is_err());
    }

    #[test]
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
    fn test_spix_informat_is_set() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let bytes = pix.write_spix_to_bytes().unwrap();
        let restored = Pix::read_spix_from_bytes(&bytes).unwrap();
        assert_eq!(restored.informat(), ImageFormat::Spix);
    }

    #[test]
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
