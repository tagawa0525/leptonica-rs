//! Serialization for PixColormap
//!
//! Text-based serialization format compatible with C Leptonica.
//! Note: The C Leptonica colormap format has no version number.
//!
//! # Format
//!
//! ```text
//! \nPixcmap: depth = D bpp; N colors\n
//! Color    R-val    G-val    B-val   Alpha\n
//! ----------------------------------------\n
//!   0       R        G        B        A\n
//!   1       R        G        B        A\n
//!   ...
//! \n
//! ```
//!
//! # See also
//!
//! C Leptonica: `colormap.c` (`pixcmapReadStream`, `pixcmapWriteStream`)

use crate::PixColormap;
use crate::error::{Error, Result};
use std::io::{Read, Write};
use std::path::Path;

/// Maximum input size in bytes.
const MAX_INPUT_SIZE: usize = 100_000_000;

impl PixColormap {
    /// Read a PixColormap from a reader.
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        todo!()
    }

    /// Read a PixColormap from a file.
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        todo!()
    }

    /// Read a PixColormap from a byte slice.
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        todo!()
    }

    /// Write a PixColormap to a writer.
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        todo!()
    }

    /// Write a PixColormap to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        todo!()
    }

    /// Write a PixColormap to a byte vector.
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "not yet implemented"]
    fn test_colormap_roundtrip() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(255, 0, 0, 255).unwrap();
        cmap.add_rgba(0, 255, 0, 255).unwrap();
        cmap.add_rgba(0, 0, 255, 128).unwrap();

        let bytes = cmap.write_to_bytes().unwrap();
        let restored = PixColormap::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.depth(), 8);
        assert_eq!(restored.len(), 3);
        let (r, g, b, a) = restored.get_rgba(0).unwrap();
        assert_eq!((r, g, b, a), (255, 0, 0, 255));
        let (r, g, b, a) = restored.get_rgba(2).unwrap();
        assert_eq!((r, g, b, a), (0, 0, 255, 128));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_colormap_write_format() {
        let mut cmap = PixColormap::new(4).unwrap();
        cmap.add_rgba(10, 20, 30, 255).unwrap();

        let bytes = cmap.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("Pixcmap: depth = 4 bpp"));
        assert!(text.contains("1 colors"));
        assert!(text.contains("R-val"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_colormap_file_roundtrip() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(0, 0, 0, 255).unwrap();
        cmap.add_rgba(255, 255, 255, 255).unwrap();

        let dir = std::env::temp_dir().join("leptonica_test_colormap");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_cmap.txt");

        cmap.write_to_file(&path).unwrap();
        let restored = PixColormap::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 2);
        let (r, g, b, _) = restored.get_rgba(0).unwrap();
        assert_eq!((r, g, b), (0, 0, 0));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_colormap_invalid_data() {
        let result = PixColormap::read_from_bytes(b"not valid");
        assert!(result.is_err());
    }
}
