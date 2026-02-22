//! JbData serialisation
//!
//! Binary format for round-tripping [`JbData`] to/from a byte stream.
//!
//! ## Format
//!
//! ```text
//! 8 bytes  magic: b"JBDATA\x01\x00"
//! 4 bytes  n_pages (u32 le)
//! 4 bytes  w       (i32 le)
//! 4 bytes  h       (i32 le)
//! 4 bytes  n_class (u32 le)
//! 4 bytes  lattice_w (i32 le)
//! 4 bytes  lattice_h (i32 le)
//! 4 bytes  n_comps (u32 le) — length of naclass / napage / ptaul arrays
//! n_comps × 4 bytes  naclass (u32 le each)
//! n_comps × 4 bytes  napage  (u32 le each)
//! n_comps × 4 bytes  ptaul_x (i32 le each)
//! n_comps × 4 bytes  ptaul_y (i32 le each)
//! variable  spix-encoded composite template Pix
//! ```

use std::io::{Read, Write};
use std::path::Path;

use crate::error::{RecogError, RecogResult};

use super::types::JbData;

const MAGIC: &[u8; 8] = b"JBDATA\x01\x00";

impl JbData {
    /// Serialises this [`JbData`] to `writer` in binary format.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails or the Pix cannot be encoded.
    pub fn write<W: Write>(&self, _writer: W) -> RecogResult<()> {
        todo!("Phase 10: implement JbData::write")
    }

    /// Deserialises a [`JbData`] from `reader`.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or reading fails.
    pub fn read<R: Read>(_reader: R) -> RecogResult<JbData> {
        todo!("Phase 10: implement JbData::read")
    }

    /// Writes this [`JbData`] to a file at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or writing fails.
    pub fn write_to_file(&self, path: &Path) -> RecogResult<()> {
        let file =
            std::fs::File::create(path).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        self.write(std::io::BufWriter::new(file))
    }

    /// Reads a [`JbData`] from a file at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or parsing fails.
    pub fn read_from_file(path: &Path) -> RecogResult<JbData> {
        let file =
            std::fs::File::open(path).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        JbData::read(std::io::BufReader::new(file))
    }
}

#[cfg(test)]
mod tests {
    use leptonica_core::{Pix, PixelDepth};

    use super::super::classify::rank_haus_init;
    use super::super::types::JbComponent;
    use super::super::types::{JbClasser, JbData, JbMethod};

    fn make_jbdata() -> JbData {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        JbData {
            pix,
            npages: 2,
            w: 800,
            h: 600,
            nclass: 3,
            lattice_w: 20,
            lattice_h: 20,
            naclass: vec![0, 1, 2, 0],
            napage: vec![0, 0, 1, 1],
            ptaul: vec![(10, 20), (30, 40), (50, 60), (70, 80)],
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_jbdata_write_read_roundtrip() {
        let data = make_jbdata();
        let mut buf = Vec::new();
        data.write(&mut buf).unwrap();
        let data2 = JbData::read(buf.as_slice()).unwrap();
        assert_eq!(data2.npages, data.npages);
        assert_eq!(data2.w, data.w);
        assert_eq!(data2.h, data.h);
        assert_eq!(data2.nclass, data.nclass);
        assert_eq!(data2.lattice_w, data.lattice_w);
        assert_eq!(data2.lattice_h, data.lattice_h);
        assert_eq!(data2.naclass, data.naclass);
        assert_eq!(data2.napage, data.napage);
        assert_eq!(data2.ptaul, data.ptaul);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_jbdata_invalid_magic() {
        let bad = b"BADJBDAT";
        let result = JbData::read(bad.as_slice());
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_jbdata_file_roundtrip() {
        let data = make_jbdata();
        let path = std::env::temp_dir().join("jbdata_test.bin");
        data.write_to_file(&path).unwrap();
        let data2 = JbData::read_from_file(&path).unwrap();
        assert_eq!(data2.npages, data.npages);
        assert_eq!(data2.nclass, data.nclass);
        let _ = std::fs::remove_file(&path);
    }
}
