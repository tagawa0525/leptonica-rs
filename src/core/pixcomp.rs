//! Compressed Pix container
//!
//! `PixComp` stores an image in compressed format (PNG, JPEG, etc.) to save
//! memory. `PixaComp` is an array of `PixComp` entries with optional
//! bounding boxes.
//!
//! This is useful for holding large collections of images in memory while
//! using significantly less space than uncompressed `Pix` arrays.
//!
//! # Reference
//!
//! Based on Leptonica's `pixcomp.c`.

use crate::core::{Box, Boxa, Error, ImageFormat, Pix, Pixa, PixelDepth, Result};
use std::io::Read;
use std::path::Path;

// ---------------------------------------------------------------------------
// PixComp
// ---------------------------------------------------------------------------

/// A compressed image container.
///
/// Stores image data in a compressed format (PNG, JPEG, etc.) along with
/// metadata about dimensions and depth.
#[derive(Clone, Debug)]
pub struct PixComp {
    /// Image width
    w: u32,
    /// Image height
    h: u32,
    /// Pixel depth
    d: PixelDepth,
    /// Horizontal resolution
    xres: i32,
    /// Vertical resolution
    yres: i32,
    /// Compression format
    comptype: ImageFormat,
    /// Whether the original had a colormap
    cmapflag: bool,
    /// Compressed image data
    data: Vec<u8>,
}

impl PixComp {
    /// Create a compressed Pix from an uncompressed Pix.
    ///
    /// The image is compressed using the specified format. If `comptype` is
    /// `None`, an appropriate format is automatically determined.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixcompCreateFromPix()`
    pub fn create_from_pix(pix: &Pix, comptype: Option<ImageFormat>) -> Result<Self> {
        let format = comptype.unwrap_or_else(|| determine_format(pix));
        let data = crate::io::write_image_mem(pix, format)
            .map_err(|e| Error::EncodeError(format!("failed to compress pix: {e}")))?;

        Ok(Self {
            w: pix.width(),
            h: pix.height(),
            d: pix.depth(),
            xres: pix.xres(),
            yres: pix.yres(),
            comptype: format,
            cmapflag: pix.has_colormap(),
            data,
        })
    }

    /// Create a PixComp from raw compressed data.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixcompCreateFromString()`
    pub fn create_from_string(
        data: Vec<u8>,
        w: u32,
        h: u32,
        d: PixelDepth,
        comptype: ImageFormat,
    ) -> Self {
        Self {
            w,
            h,
            d,
            xres: 0,
            yres: 0,
            comptype,
            cmapflag: false,
            data,
        }
    }

    /// Create a PixComp from a file.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixcompCreateFromFile()`
    pub fn create_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path.as_ref())?;
        let format = crate::io::detect_format_from_bytes(&data)
            .map_err(|e| Error::DecodeError(format!("failed to detect format: {e}")))?;
        let pix = crate::io::read_image_mem(&data)
            .map_err(|e| Error::DecodeError(format!("failed to read image: {e}")))?;

        Ok(Self {
            w: pix.width(),
            h: pix.height(),
            d: pix.depth(),
            xres: pix.xres(),
            yres: pix.yres(),
            comptype: format,
            cmapflag: pix.has_colormap(),
            data,
        })
    }

    /// Get the image dimensions.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixcompGetDimensions()`
    pub fn get_dimensions(&self) -> (u32, u32, PixelDepth) {
        (self.w, self.h, self.d)
    }

    /// Get the compression parameters.
    ///
    /// Returns (xres, yres, comptype, cmapflag).
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixcompGetParameters()`
    pub fn get_parameters(&self) -> (i32, i32, ImageFormat, bool) {
        (self.xres, self.yres, self.comptype, self.cmapflag)
    }

    /// Decompress to a Pix.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixCreateFromPixcomp()`
    pub fn to_pix(&self) -> Result<Pix> {
        crate::io::read_image_mem(&self.data)
            .map_err(|e| Error::DecodeError(format!("failed to decompress pixcomp: {e}")))
    }

    /// Get a reference to the compressed data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the compression format.
    pub fn comptype(&self) -> ImageFormat {
        self.comptype
    }

    /// Write the compressed data to a file.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixcompWriteFile()`
    pub fn write_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        std::fs::write(path, &self.data)?;
        Ok(())
    }
}

/// Determine an appropriate compression format for a Pix.
///
/// # Reference
///
/// C Leptonica: `pixcompDetermineFormat()`
pub fn determine_format(pix: &Pix) -> ImageFormat {
    match pix.depth() {
        PixelDepth::Bit1 => ImageFormat::Png,
        PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8 => ImageFormat::Png,
        PixelDepth::Bit16 => ImageFormat::Png,
        PixelDepth::Bit32 => ImageFormat::Png,
    }
}

// ---------------------------------------------------------------------------
// PixaComp
// ---------------------------------------------------------------------------

/// An array of compressed images with optional bounding boxes.
///
/// # Reference
///
/// C Leptonica: `PIXAC` / `PixaComp`
#[derive(Clone, Debug)]
pub struct PixaComp {
    /// Compressed images
    pixcomps: Vec<PixComp>,
    /// Bounding boxes
    boxa: Boxa,
    /// Index offset
    offset: i32,
}

impl PixaComp {
    /// Create an empty PixaComp with the given initial capacity.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompCreate()`
    pub fn create(capacity: usize) -> Self {
        Self {
            pixcomps: Vec::with_capacity(capacity),
            boxa: Boxa::new(),
            offset: 0,
        }
    }

    /// Create a PixaComp with `n` entries initialized from a template Pix.
    ///
    /// If `pix` is Some, each entry is initialized from that Pix.
    /// If `b` is Some, each box entry is initialized from that Box.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompCreateWithInit()`
    pub fn create_with_init(
        n: usize,
        offset: i32,
        pix: Option<&Pix>,
        b: Option<&Box>,
    ) -> Result<Self> {
        let mut pixcomps = Vec::with_capacity(n);
        let mut boxa = Boxa::with_capacity(n);

        for _ in 0..n {
            if let Some(p) = pix {
                pixcomps.push(PixComp::create_from_pix(p, None)?);
            }
            if let Some(bx) = b {
                boxa.push(*bx);
            }
        }

        Ok(Self {
            pixcomps,
            boxa,
            offset,
        })
    }

    /// Create a PixaComp from a Pixa.
    ///
    /// Each image in the Pixa is compressed.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompCreateFromPixa()`
    pub fn create_from_pixa(pixa: &Pixa, comptype: Option<ImageFormat>) -> Result<Self> {
        let mut pixcomps = Vec::with_capacity(pixa.len());
        for i in 0..pixa.len() {
            let pix = pixa.get(i).ok_or(Error::IndexOutOfBounds {
                index: i,
                len: pixa.len(),
            })?;
            pixcomps.push(PixComp::create_from_pix(pix, comptype)?);
        }
        Ok(Self {
            pixcomps,
            boxa: pixa.boxa().clone(),
            offset: 0,
        })
    }

    /// Create a PixaComp from image files.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompCreateFromFiles()`
    pub fn create_from_files<P: AsRef<Path>>(dir: P, substr: Option<&str>) -> Result<Self> {
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(dir.as_ref())?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                match substr {
                    Some(s) => name.contains(s),
                    None => true,
                }
            })
            .map(|e| e.path())
            .collect();
        paths.sort();

        let mut pixcomps = Vec::with_capacity(paths.len());
        for path in &paths {
            pixcomps.push(PixComp::create_from_file(path)?);
        }
        Ok(Self {
            pixcomps,
            boxa: Boxa::new(),
            offset: 0,
        })
    }

    /// Add a Pix (compressing it) to the array.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompAddPix()`
    pub fn add_pix(&mut self, pix: &Pix, comptype: Option<ImageFormat>) -> Result<()> {
        self.pixcomps.push(PixComp::create_from_pix(pix, comptype)?);
        Ok(())
    }

    /// Add an already-compressed PixComp to the array.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompAddPixcomp()`
    pub fn add_pixcomp(&mut self, pixcomp: PixComp) {
        self.pixcomps.push(pixcomp);
    }

    /// Replace a Pix at the given index.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompReplacePix()`
    pub fn replace_pix(
        &mut self,
        index: usize,
        pix: &Pix,
        comptype: Option<ImageFormat>,
    ) -> Result<()> {
        if index >= self.pixcomps.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.pixcomps.len(),
            });
        }
        self.pixcomps[index] = PixComp::create_from_pix(pix, comptype)?;
        Ok(())
    }

    /// Get the number of entries.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompGetCount()`
    pub fn get_count(&self) -> usize {
        self.pixcomps.len()
    }

    /// Get a reference to a PixComp at the given index.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompGetPixcomp()`
    pub fn get_pixcomp(&self, index: usize) -> Option<&PixComp> {
        let idx = (index as i32 - self.offset) as usize;
        self.pixcomps.get(idx)
    }

    /// Decompress and return a Pix at the given index.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompGetPix()`
    pub fn get_pix(&self, index: usize) -> Result<Pix> {
        let pc = self.get_pixcomp(index).ok_or(Error::IndexOutOfBounds {
            index,
            len: self.pixcomps.len(),
        })?;
        pc.to_pix()
    }

    /// Get the dimensions of an image at the given index without decompressing.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompGetPixDimensions()`
    pub fn get_pix_dimensions(&self, index: usize) -> Option<(u32, u32, PixelDepth)> {
        self.get_pixcomp(index).map(|pc| pc.get_dimensions())
    }

    /// Get a reference to the Boxa.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompGetBoxa()`
    pub fn get_boxa(&self) -> &Boxa {
        &self.boxa
    }

    /// Get a mutable reference to the Boxa.
    pub fn get_boxa_mut(&mut self) -> &mut Boxa {
        &mut self.boxa
    }

    /// Get a reference to a Box at the given index.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompGetBox()`
    pub fn get_box(&self, index: usize) -> Option<&Box> {
        self.boxa.get(index)
    }

    /// Get box geometry at the given index.
    ///
    /// Returns (x, y, w, h).
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompGetBoxGeometry()`
    pub fn get_box_geometry(&self, index: usize) -> Option<(i32, i32, i32, i32)> {
        self.boxa.get(index).map(|b| (b.x, b.y, b.w, b.h))
    }

    /// Convert all entries to a Pixa (decompressing each).
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixaCreateFromPixacomp()`
    pub fn to_pixa(&self) -> Result<Pixa> {
        let mut pixa = Pixa::with_capacity(self.pixcomps.len());
        for pc in &self.pixcomps {
            pixa.push(pc.to_pix()?);
        }
        pixa.set_boxa(self.boxa.clone());
        Ok(pixa)
    }

    /// Join another PixaComp to this one.
    ///
    /// Appends entries from `src[istart..iend]` to this PixaComp.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompJoin()`
    pub fn join(&mut self, src: &PixaComp, istart: usize, iend: Option<usize>) -> Result<()> {
        let end = iend.unwrap_or(src.pixcomps.len()).min(src.pixcomps.len());
        if istart > end {
            return Err(Error::InvalidParameter(format!(
                "istart ({istart}) > iend ({end})"
            )));
        }
        for i in istart..end {
            self.pixcomps.push(src.pixcomps[i].clone());
        }
        for i in istart..end.min(src.boxa.len()) {
            if let Some(b) = src.boxa.get(i) {
                self.boxa.push(*b);
            }
        }
        Ok(())
    }

    /// Interleave two PixaComps.
    ///
    /// Returns a new PixaComp with alternating entries from self and other.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompInterleave()`
    pub fn interleave(&self, other: &PixaComp) -> PixaComp {
        let max_len = self.pixcomps.len().max(other.pixcomps.len());
        let mut result = PixaComp::create(max_len * 2);
        for i in 0..max_len {
            if i < self.pixcomps.len() {
                result.pixcomps.push(self.pixcomps[i].clone());
            }
            if i < other.pixcomps.len() {
                result.pixcomps.push(other.pixcomps[i].clone());
            }
        }
        result
    }

    /// Read a PixaComp from a file.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompRead()`
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path)?;
        Self::read_mem(&data)
    }

    /// Read a PixaComp from a reader.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompReadStream()`
    pub fn read_stream<R: Read>(mut reader: R) -> Result<Self> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Self::read_mem(&data)
    }

    /// Read a PixaComp from memory.
    ///
    /// Format: [n: u32][offset: i32] then for each entry:
    /// [w: u32][h: u32][d: u32][comptype: u32][cmapflag: u32][data_len: u32][data: bytes]
    /// then [nbox: u32] then for each box: [x: i32][y: i32][w: i32][h: i32]
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompReadMem()`
    pub fn read_mem(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(Error::InvalidParameter(
                "PixaComp data too short".to_string(),
            ));
        }
        let mut pos = 0;

        let n = read_u32(data, &mut pos)? as usize;
        let offset = read_i32(data, &mut pos)?;

        let mut pixcomps = Vec::with_capacity(n);
        for _ in 0..n {
            let w = read_u32(data, &mut pos)?;
            let h = read_u32(data, &mut pos)?;
            let d_bits = read_u32(data, &mut pos)?;
            let comptype_val = read_u32(data, &mut pos)?;
            let cmapflag = read_u32(data, &mut pos)? != 0;
            let data_len = read_u32(data, &mut pos)? as usize;

            if pos + data_len > data.len() {
                return Err(Error::InvalidParameter(
                    "PixaComp data truncated".to_string(),
                ));
            }
            let comp_data = data[pos..pos + data_len].to_vec();
            pos += data_len;

            let d = PixelDepth::from_bits(d_bits)?;
            let comptype = image_format_from_u32(comptype_val);

            pixcomps.push(PixComp {
                w,
                h,
                d,
                xres: 0,
                yres: 0,
                comptype,
                cmapflag,
                data: comp_data,
            });
        }

        // Read boxes
        let mut boxa = Boxa::new();
        if pos + 4 <= data.len() {
            let nbox = read_u32(data, &mut pos)? as usize;
            for _ in 0..nbox {
                let x = read_i32(data, &mut pos)?;
                let y = read_i32(data, &mut pos)?;
                let bw = read_i32(data, &mut pos)?;
                let bh = read_i32(data, &mut pos)?;
                if let Ok(b) = Box::new(x, y, bw, bh) {
                    boxa.push(b);
                }
            }
        }

        Ok(Self {
            pixcomps,
            boxa,
            offset,
        })
    }

    /// Write the PixaComp to a file.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompWrite()`
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = self.write_mem()?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Write the PixaComp to a writer.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompWriteStream()`
    pub fn write_stream<W: std::io::Write>(&self, mut writer: W) -> Result<()> {
        let data = self.write_mem()?;
        writer.write_all(&data)?;
        Ok(())
    }

    /// Serialize the PixaComp to memory.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompWriteMem()`
    pub fn write_mem(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        write_u32(&mut buf, self.pixcomps.len() as u32);
        write_i32(&mut buf, self.offset);

        for pc in &self.pixcomps {
            write_u32(&mut buf, pc.w);
            write_u32(&mut buf, pc.h);
            write_u32(&mut buf, pc.d.bits());
            write_u32(&mut buf, image_format_to_u32(pc.comptype));
            write_u32(&mut buf, if pc.cmapflag { 1 } else { 0 });
            write_u32(&mut buf, pc.data.len() as u32);
            buf.extend_from_slice(&pc.data);
        }

        // Write boxes
        write_u32(&mut buf, self.boxa.len() as u32);
        for b in self.boxa.boxes() {
            write_i32(&mut buf, b.x);
            write_i32(&mut buf, b.y);
            write_i32(&mut buf, b.w);
            write_i32(&mut buf, b.h);
        }

        Ok(buf)
    }

    /// Write all compressed images to individual files.
    ///
    /// Files are named `{rootname}NNN.ext` where NNN is a zero-padded index.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompWriteFiles()`
    pub fn write_files(&self, rootname: &str) -> Result<()> {
        for (i, pc) in self.pixcomps.iter().enumerate() {
            let ext = crate::io::get_format_extension(pc.comptype);
            let filename = format!("{rootname}{i:03}.{ext}");
            pc.write_file(&filename)?;
        }
        Ok(())
    }

    /// Convert to PDF data.
    ///
    /// Decompresses all images and generates PDF output.
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompConvertToPdfData()`
    #[cfg(feature = "pdf-format")]
    pub fn convert_to_pdf_data(&self, title: &str) -> Result<Vec<u8>> {
        let pixa = self.to_pixa()?;
        let pix_refs: Vec<&Pix> = pixa.pix_slice().iter().collect();
        let mut buf = Vec::new();
        crate::io::pdf::write_pdf_multi(
            &pix_refs,
            &mut buf,
            &crate::io::pdf::PdfOptions {
                title: Some(title.to_string()),
                ..Default::default()
            },
        )
        .map_err(|e| Error::EncodeError(format!("PDF generation failed: {e}")))?;
        Ok(buf)
    }

    /// Fast convert to PDF data (uses compressed data directly when possible).
    ///
    /// # Reference
    ///
    /// C Leptonica: `pixacompFastConvertToPdfData()`
    #[cfg(feature = "pdf-format")]
    pub fn fast_convert_to_pdf_data(&self, title: &str) -> Result<Vec<u8>> {
        // For now, use the same implementation as convert_to_pdf_data
        self.convert_to_pdf_data(title)
    }

    /// Get the index offset.
    pub fn offset(&self) -> i32 {
        self.offset
    }

    /// Set the index offset.
    pub fn set_offset(&mut self, offset: i32) {
        self.offset = offset;
    }
}

// ---------------------------------------------------------------------------
// Serialization helpers
// ---------------------------------------------------------------------------

fn read_u32(data: &[u8], pos: &mut usize) -> Result<u32> {
    if *pos + 4 > data.len() {
        return Err(Error::InvalidParameter(
            "unexpected end of data".to_string(),
        ));
    }
    let val = u32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
    *pos += 4;
    Ok(val)
}

fn read_i32(data: &[u8], pos: &mut usize) -> Result<i32> {
    read_u32(data, pos).map(|v| v as i32)
}

fn write_u32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_le_bytes());
}

fn write_i32(buf: &mut Vec<u8>, val: i32) {
    buf.extend_from_slice(&val.to_le_bytes());
}

fn image_format_from_u32(val: u32) -> ImageFormat {
    match val {
        1 => ImageFormat::Bmp,
        2 => ImageFormat::Jpeg,
        3 => ImageFormat::Png,
        4 => ImageFormat::Tiff,
        5 => ImageFormat::Gif,
        6 => ImageFormat::Pnm,
        7 => ImageFormat::Ps,
        8 => ImageFormat::Spix,
        9 => ImageFormat::WebP,
        10 => ImageFormat::Lpdf,
        11 => ImageFormat::Jp2,
        _ => ImageFormat::Unknown,
    }
}

fn image_format_to_u32(format: ImageFormat) -> u32 {
    match format {
        ImageFormat::Bmp => 1,
        ImageFormat::Jpeg => 2,
        ImageFormat::Png => 3,
        ImageFormat::Tiff => 4,
        ImageFormat::Gif => 5,
        ImageFormat::Pnm => 6,
        ImageFormat::Ps => 7,
        ImageFormat::Spix => 8,
        ImageFormat::WebP => 9,
        ImageFormat::Lpdf => 10,
        ImageFormat::Jp2 => 11,
        _ => 0,
    }
}
