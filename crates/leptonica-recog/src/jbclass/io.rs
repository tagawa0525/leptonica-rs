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

use leptonica_core::Pix;

use crate::error::{RecogError, RecogResult};

use super::types::JbData;

const MAGIC: &[u8; 8] = b"JBDATA\x01\x00";

fn write_u32<W: Write>(w: &mut W, v: u32) -> RecogResult<()> {
    w.write_all(&v.to_le_bytes())
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))
}

fn write_i32<W: Write>(w: &mut W, v: i32) -> RecogResult<()> {
    w.write_all(&v.to_le_bytes())
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))
}

fn read_u32<R: Read>(r: &mut R) -> RecogResult<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_i32<R: Read>(r: &mut R) -> RecogResult<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    Ok(i32::from_le_bytes(buf))
}

impl JbData {
    /// Serialises this [`JbData`] to `writer` in binary format.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails or the Pix cannot be encoded.
    pub fn write<W: Write>(&self, mut writer: W) -> RecogResult<()> {
        writer
            .write_all(MAGIC)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        let npages = u32::try_from(self.npages)
            .map_err(|_| RecogError::InvalidParameter("npages too large for u32".to_string()))?;
        write_u32(&mut writer, npages)?;
        write_i32(&mut writer, self.w)?;
        write_i32(&mut writer, self.h)?;
        let nclass = u32::try_from(self.nclass)
            .map_err(|_| RecogError::InvalidParameter("nclass too large for u32".to_string()))?;
        write_u32(&mut writer, nclass)?;
        write_i32(&mut writer, self.lattice_w)?;
        write_i32(&mut writer, self.lattice_h)?;
        let n_comps = u32::try_from(self.naclass.len()).map_err(|_| {
            RecogError::InvalidParameter("naclass length too large for u32".to_string())
        })?;
        write_u32(&mut writer, n_comps)?;
        for &v in &self.naclass {
            let v_u32 = u32::try_from(v).map_err(|_| {
                RecogError::InvalidParameter("naclass value too large for u32".to_string())
            })?;
            write_u32(&mut writer, v_u32)?;
        }
        for &v in &self.napage {
            let v_u32 = u32::try_from(v).map_err(|_| {
                RecogError::InvalidParameter("napage value too large for u32".to_string())
            })?;
            write_u32(&mut writer, v_u32)?;
        }
        for &(x, _) in &self.ptaul {
            write_i32(&mut writer, x)?;
        }
        for &(_, y) in &self.ptaul {
            write_i32(&mut writer, y)?;
        }
        self.pix
            .write_spix(&mut writer)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        Ok(())
    }

    /// Deserialises a [`JbData`] from `reader`.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or reading fails.
    pub fn read<R: Read>(mut reader: R) -> RecogResult<JbData> {
        let mut magic = [0u8; 8];
        reader
            .read_exact(&mut magic)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        if &magic != MAGIC {
            return Err(RecogError::InvalidParameter(
                "invalid JbData magic bytes".to_string(),
            ));
        }
        let npages = read_u32(&mut reader)? as usize;
        let w = read_i32(&mut reader)?;
        let h = read_i32(&mut reader)?;
        let nclass = read_u32(&mut reader)? as usize;
        let lattice_w = read_i32(&mut reader)?;
        let lattice_h = read_i32(&mut reader)?;
        let n_comps = read_u32(&mut reader)? as usize;
        let mut naclass = Vec::with_capacity(n_comps);
        for _ in 0..n_comps {
            naclass.push(read_u32(&mut reader)? as usize);
        }
        let mut napage = Vec::with_capacity(n_comps);
        for _ in 0..n_comps {
            napage.push(read_u32(&mut reader)? as usize);
        }
        let mut xs = Vec::with_capacity(n_comps);
        for _ in 0..n_comps {
            xs.push(read_i32(&mut reader)?);
        }
        let mut ys = Vec::with_capacity(n_comps);
        for _ in 0..n_comps {
            ys.push(read_i32(&mut reader)?);
        }
        let ptaul = xs.into_iter().zip(ys).collect();
        let pix =
            Pix::read_spix(&mut reader).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        Ok(JbData {
            pix,
            npages,
            w,
            h,
            nclass,
            lattice_w,
            lattice_h,
            naclass,
            napage,
            ptaul,
        })
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

    use super::super::types::JbData;

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
    fn test_jbdata_invalid_magic() {
        let bad = b"BADJBDAT";
        let result = JbData::read(bad.as_slice());
        assert!(result.is_err());
    }

    #[test]
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
