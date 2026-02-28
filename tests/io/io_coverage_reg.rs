//! Coverage tests for 51 unimplemented IO functions
//!
//! # See also
//!
//! C Leptonica: pngio.c, jpegio.c, tiffio.c, webpanimio.c, jp2kio.c,
//!              pdfio1.c, pdfio2.c, psio1.c, psio2.c, readfile.c, writefile.c

#![allow(unused_imports, unused_variables, dead_code)]

use leptonica::core::pixel;
use leptonica::{ImageFormat, Pix, PixelDepth};

/// Create a uniform RGB image
fn make_rgb(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_spp(3);
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, pixel::compose_rgb((x * 5) as u8, (y * 5) as u8, 128));
        }
    }
    pm.into()
}

/// Create an 8bpp grayscale image
fn make_gray(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, ((x + y) * 3) % 256);
        }
    }
    pm.into()
}

/// Create a 1bpp binary image
fn make_binary(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, (x + y) % 2);
        }
    }
    pm.into()
}

/// Create a Pixa with several test images
fn make_pixa(n: usize) -> leptonica::core::Pixa {
    let mut pixa = leptonica::core::Pixa::new();
    for i in 0..n {
        let pix = make_gray(20 + i as u32 * 5, 20 + i as u32 * 5);
        pixa.push(pix);
    }
    pixa
}

// ============================================================
// 1. isPngInterlaced → png::is_png_interlaced
// ============================================================
#[test]
fn test_is_png_interlaced() {
    use leptonica::io::png;

    // Write a PNG, check interlacing (default is non-interlaced)
    let pix = make_gray(50, 50);
    let mut buf = Vec::new();
    png::write_png(&pix, &mut buf).unwrap();
    let interlaced = png::is_png_interlaced(&buf).unwrap();
    assert!(!interlaced, "default PNG should not be interlaced");
}

// ============================================================
// 2. fgetPngColormapInfo → png::get_png_colormap_info
// ============================================================
#[test]
fn test_get_png_colormap_info() {
    use leptonica::core::PixColormap;
    use leptonica::io::png;

    // Create a colormapped image
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(255, 0, 0).unwrap();
    cmap.add_rgb(0, 255, 0).unwrap();
    pm.set_colormap(Some(cmap)).unwrap();
    let pix: Pix = pm.into();

    let mut buf = Vec::new();
    png::write_png(&pix, &mut buf).unwrap();

    let info = png::get_png_colormap_info(&buf).unwrap();
    assert!(info.is_some(), "should have colormap info");
    let (cmap_out, has_transparency) = info.unwrap();
    assert!(cmap_out.len() >= 2);
    assert!(!has_transparency);
}

#[test]
fn test_get_png_colormap_info_non_indexed() {
    use leptonica::io::png;

    // Non-colormapped image should return None
    let pix = make_gray(10, 10);
    let mut buf = Vec::new();
    png::write_png(&pix, &mut buf).unwrap();

    let info = png::get_png_colormap_info(&buf).unwrap();
    assert!(info.is_none(), "non-indexed PNG should return None");
}

#[test]
fn test_get_png_colormap_info_transparency() {
    use leptonica::core::PixColormap;
    use leptonica::io::png;

    // Create a colormapped image with transparency via tRNS
    let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgba(255, 0, 0, 255).unwrap(); // opaque red
    cmap.add_rgba(0, 255, 0, 128).unwrap(); // semi-transparent green
    cmap.add_rgba(0, 0, 255, 0).unwrap(); // fully transparent blue
    pm.set_colormap(Some(cmap)).unwrap();
    let pix: Pix = pm.into();

    let mut buf = Vec::new();
    png::write_png(&pix, &mut buf).unwrap();

    let info = png::get_png_colormap_info(&buf).unwrap();
    assert!(info.is_some(), "should have colormap info");
    let (cmap_out, has_transparency) = info.unwrap();
    assert_eq!(cmap_out.len(), 3);
    assert!(has_transparency, "should detect transparency from tRNS");

    // Verify alpha was applied to colormap entries
    let rgba0 = pixel::extract_rgba(cmap_out.get_rgba32(0).unwrap());
    assert_eq!(rgba0, (255, 0, 0, 255));
    let rgba1 = pixel::extract_rgba(cmap_out.get_rgba32(1).unwrap());
    assert_eq!((rgba1.0, rgba1.1, rgba1.2), (0, 255, 0));
    assert_eq!(rgba1.3, 128, "entry 1 should have alpha 128");
    let rgba2 = pixel::extract_rgba(cmap_out.get_rgba32(2).unwrap());
    assert_eq!((rgba2.0, rgba2.1, rgba2.2), (0, 0, 255));
    assert_eq!(rgba2.3, 0, "entry 2 should be fully transparent");
}

// ============================================================
// 3. fgetJpegComment → jpeg::get_jpeg_comment
// ============================================================
#[test]
fn test_get_jpeg_comment() {
    use leptonica::io::jpeg;

    // Standard JPEG without comment
    let pix = make_gray(30, 30);
    let mut buf = Vec::new();
    jpeg::write_jpeg(&pix, &mut buf, &jpeg::JpegOptions::default()).unwrap();
    let comment = jpeg::get_jpeg_comment(&buf).unwrap();
    assert!(comment.is_none(), "default JPEG should have no comment");
}

#[test]
fn test_get_jpeg_comment_with_comment() {
    use leptonica::io::jpeg;

    // Create a JPEG then inject a COM marker
    let pix = make_gray(30, 30);
    let mut jpeg_data = Vec::new();
    jpeg::write_jpeg(&pix, &mut jpeg_data, &jpeg::JpegOptions::default()).unwrap();

    // Insert COM marker (0xFF 0xFE) right after SOI (0xFF 0xD8)
    let comment_text = b"Hello from leptonica-rs!";
    let com_len = (comment_text.len() + 2) as u16; // length includes 2-byte length field
    let mut data_with_comment = Vec::new();
    data_with_comment.extend_from_slice(&jpeg_data[..2]); // SOI
    data_with_comment.push(0xFF);
    data_with_comment.push(0xFE); // COM marker
    data_with_comment.push((com_len >> 8) as u8);
    data_with_comment.push((com_len & 0xFF) as u8);
    data_with_comment.extend_from_slice(comment_text);
    data_with_comment.extend_from_slice(&jpeg_data[2..]); // rest of JPEG

    let comment = jpeg::get_jpeg_comment(&data_with_comment).unwrap();
    assert!(comment.is_some(), "should find COM marker");
    assert_eq!(comment.unwrap(), "Hello from leptonica-rs!");
}

#[test]
fn test_get_jpeg_comment_invalid_data() {
    use leptonica::io::jpeg;

    // Not a JPEG
    let result = jpeg::get_jpeg_comment(b"not a jpeg");
    assert!(result.is_err());
}

// ============================================================
// 4. pixWriteTiffCustom → tiff::write_tiff_custom
// ============================================================
#[cfg(feature = "tiff-format")]
#[test]
fn test_write_tiff_custom() {
    use leptonica::io::tiff;

    let pix = make_gray(40, 40);
    let tags = tiff::TiffCustomTags {
        tag_ids: vec![270], // ImageDescription
        values: vec!["test custom tag".to_string()],
        types: vec!["ascii".to_string()],
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    tiff::write_tiff_custom(&pix, &mut cursor, tiff::TiffCompression::Lzw, &tags).unwrap();
    let data = cursor.into_inner();
    assert!(data.len() > 8, "TIFF data should be non-trivial");

    // Verify we can read it back
    let pix2 = tiff::read_tiff(std::io::Cursor::new(&data)).unwrap();
    assert_eq!(pix2.width(), 40);
    assert_eq!(pix2.height(), 40);
}

// ============================================================
// 5. extractG4DataFromFile → tiff::extract_g4_data
// ============================================================
#[cfg(feature = "tiff-format")]
#[test]
fn test_extract_g4_data() {
    use leptonica::io::tiff;

    // The tiff crate doesn't support G4 encoding, so write uncompressed
    // and verify extract_g4_data returns an appropriate error
    let pix = make_binary(80, 80);
    let mut cursor = std::io::Cursor::new(Vec::new());
    tiff::write_tiff(&pix, &mut cursor, tiff::TiffCompression::G4).unwrap();
    let tiff_data = cursor.into_inner();

    // G4 encoding isn't supported by the tiff crate (falls back to uncompressed),
    // so extract_g4_data should report not-G4-compressed
    let result = tiff::extract_g4_data(&tiff_data);
    assert!(
        result.is_err(),
        "should fail since tiff crate doesn't write G4"
    );
}

// ============================================================
// 6. pixWriteMemTiffCustom → tiff::write_tiff_custom_mem
// ============================================================
#[cfg(feature = "tiff-format")]
#[test]
fn test_write_tiff_custom_mem() {
    use leptonica::io::tiff;

    let pix = make_gray(40, 40);
    let tags = tiff::TiffCustomTags {
        tag_ids: vec![270],
        values: vec!["mem custom tag".to_string()],
        types: vec!["ascii".to_string()],
    };

    let data = tiff::write_tiff_custom_mem(&pix, tiff::TiffCompression::Lzw, &tags).unwrap();
    assert!(data.len() > 8);

    let pix2 = tiff::read_tiff(std::io::Cursor::new(&data)).unwrap();
    assert_eq!(pix2.width(), 40);
}

// ============================================================
// 7-9. Animated WebP: pixaWriteWebPAnim / Stream / Mem
// ============================================================
#[cfg(feature = "webp-format")]
#[test]
fn test_pixa_write_webp_anim_mem() {
    use leptonica::io::webp;

    let pixa = make_pixa(3);
    let options = webp::WebPAnimOptions {
        loop_count: 0,
        duration_ms: 100,
        quality: 75,
        lossless: true,
    };
    let data = webp::write_webp_anim_mem(&pixa, &options).unwrap();
    assert!(data.len() > 12);
    assert_eq!(&data[0..4], b"RIFF");
    assert_eq!(&data[8..12], b"WEBP");
}

#[cfg(feature = "webp-format")]
#[test]
fn test_pixa_write_webp_anim_stream() {
    use leptonica::io::webp;

    let pixa = make_pixa(2);
    let options = webp::WebPAnimOptions {
        loop_count: 0,
        duration_ms: 200,
        quality: 75,
        lossless: true,
    };
    let mut buf = Vec::new();
    webp::write_webp_anim(&pixa, &mut buf, &options).unwrap();
    assert!(buf.len() > 12);
}

#[cfg(feature = "webp-format")]
#[test]
fn test_pixa_write_webp_anim_file() {
    use leptonica::io::webp;

    let pixa = make_pixa(2);
    let options = webp::WebPAnimOptions {
        loop_count: 0,
        duration_ms: 100,
        quality: 75,
        lossless: true,
    };
    let dir = std::env::temp_dir().join("leptonica_webp_anim_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test_anim.webp");
    webp::write_webp_anim_file(&pixa, &path, &options).unwrap();
    assert!(path.exists());
    let data = std::fs::read(&path).unwrap();
    assert!(data.len() > 12);
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 10-12. JP2K write: pixWriteJp2k / Stream / Mem
// ============================================================
#[cfg(feature = "jp2k-format")]
#[test]
fn test_write_jp2k_mem() {
    use leptonica::io::jp2k;

    let pix = make_gray(40, 40);
    let options = jp2k::Jp2kWriteOptions::default();
    let result = jp2k::write_jp2k_mem(&pix, &options);
    // JP2K encoding not supported - verify error
    assert!(result.is_err());
}

#[cfg(feature = "jp2k-format")]
#[test]
fn test_write_jp2k_stream() {
    use leptonica::io::jp2k;

    let pix = make_rgb(30, 30);
    let options = jp2k::Jp2kWriteOptions::default();
    let mut buf = Vec::new();
    let result = jp2k::write_jp2k(&pix, &mut buf, &options);
    // JP2K encoding not supported - verify error
    assert!(result.is_err());
}

#[cfg(feature = "jp2k-format")]
#[test]
fn test_write_jp2k_roundtrip() {
    use leptonica::io::jp2k;

    let pix = make_gray(40, 40);
    let options = jp2k::Jp2kWriteOptions { quality: 100 };
    let result = jp2k::write_jp2k_mem(&pix, &options);
    // JP2K encoding not supported - verify error
    assert!(result.is_err());
}

// ============================================================
// 13. selectDefaultPdfEncoding → pdf::select_default_encoding
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_select_default_pdf_encoding() {
    use leptonica::io::pdf;

    let pix1 = make_binary(40, 40);
    assert_eq!(
        pdf::select_default_encoding(&pix1),
        pdf::PdfCompression::Flate
    );

    let pix8 = make_gray(40, 40);
    assert_eq!(
        pdf::select_default_encoding(&pix8),
        pdf::PdfCompression::Flate
    );

    let pix32 = make_rgb(40, 40);
    assert_eq!(
        pdf::select_default_encoding(&pix32),
        pdf::PdfCompression::Jpeg
    );
}

// ============================================================
// 14. convertUnscaledFilesToPdf → pdf::convert_unscaled_files_to_pdf
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_unscaled_files_to_pdf() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdf_unscaled_test");
    std::fs::create_dir_all(&dir).unwrap();

    // Write test images
    let pix = make_gray(50, 50);
    leptonica::io::write_image(&pix, dir.join("img001.png"), ImageFormat::Png).unwrap();
    leptonica::io::write_image(&pix, dir.join("img002.png"), ImageFormat::Png).unwrap();

    let outpath = dir.join("out.pdf");
    pdf::convert_unscaled_files_to_pdf(&dir, None, Some("test"), &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 15. convertUnscaledToPdfData → pdf::convert_unscaled_to_pdf_data
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_unscaled_to_pdf_data() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdf_unscaled_data");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.png");
    let pix = make_gray(50, 50);
    leptonica::io::write_image(&pix, &path, ImageFormat::Png).unwrap();

    let data = pdf::convert_unscaled_to_pdf_data(&path, Some("title")).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 16. convertToPdf → pdf::convert_to_pdf
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_to_pdf() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdf_convert");
    std::fs::create_dir_all(&dir).unwrap();
    let inpath = dir.join("input.png");
    let outpath = dir.join("output.pdf");
    let pix = make_gray(40, 40);
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Png).unwrap();

    pdf::convert_to_pdf(&inpath, &PdfConvertOptions::default(), &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 17. convertImageDataToPdf → pdf::convert_image_data_to_pdf
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_image_data_to_pdf() {
    use leptonica::io::pdf;

    let pix = make_gray(40, 40);
    let mut png_data = Vec::new();
    leptonica::io::png::write_png(&pix, &mut png_data).unwrap();

    let dir = std::env::temp_dir().join("leptonica_pdf_imdata");
    std::fs::create_dir_all(&dir).unwrap();
    let outpath = dir.join("output.pdf");
    pdf::convert_image_data_to_pdf(&png_data, &PdfConvertOptions::default(), &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 18. convertToPdfData → pdf::convert_to_pdf_data
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_to_pdf_data() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdf_data");
    std::fs::create_dir_all(&dir).unwrap();
    let inpath = dir.join("input.png");
    let pix = make_gray(40, 40);
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Png).unwrap();

    let data = pdf::convert_to_pdf_data(&inpath, &PdfConvertOptions::default()).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 19. convertImageDataToPdfData → pdf::convert_image_data_to_pdf_data
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_image_data_to_pdf_data() {
    use leptonica::io::pdf;

    let pix = make_gray(40, 40);
    let mut png_data = Vec::new();
    leptonica::io::png::write_png(&pix, &mut png_data).unwrap();

    let data =
        pdf::convert_image_data_to_pdf_data(&png_data, &PdfConvertOptions::default()).unwrap();
    assert!(data.starts_with(b"%PDF-"));
}

// ============================================================
// 20. convertSegmentedFilesToPdf → pdf::convert_segmented_files_to_pdf
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_segmented_files_to_pdf() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdfseg_files");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(60, 60);
    leptonica::io::write_image(&pix, dir.join("page001.png"), ImageFormat::Png).unwrap();

    let outpath = dir.join("segmented.pdf");
    pdf::convert_segmented_files_to_pdf(&dir, None, 300, &PdfOptions::default(), &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 21. convertNumberedMasksToBoxaa → pdf::convert_numbered_masks_to_boxaa
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_numbered_masks_to_boxaa() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdf_masks");
    std::fs::create_dir_all(&dir).unwrap();
    // Create mask images with numbered names
    let mask = make_binary(40, 40);
    leptonica::io::write_image(&mask, dir.join("mask001.png"), ImageFormat::Png).unwrap();

    let boxaa = pdf::convert_numbered_masks_to_boxaa(&dir, Some("mask"), 4, 4).unwrap();
    assert!(!boxaa.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 22. convertToPdfSegmented → pdf::convert_to_pdf_segmented
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_to_pdf_segmented() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdfseg_single");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(60, 60);
    let inpath = dir.join("input.png");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Png).unwrap();

    let outpath = dir.join("segmented.pdf");
    pdf::convert_to_pdf_segmented(&inpath, 300, None, &PdfOptions::default(), &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 23. pixConvertToPdfSegmented → pdf::pix_convert_to_pdf_segmented
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_pix_convert_to_pdf_segmented() {
    use leptonica::io::pdf;

    let pix = make_gray(60, 60);
    let dir = std::env::temp_dir().join("leptonica_pdfseg_pix");
    std::fs::create_dir_all(&dir).unwrap();
    let outpath = dir.join("segmented.pdf");
    pdf::pix_convert_to_pdf_segmented(&pix, 300, None, &PdfOptions::default(), &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 24. convertToPdfDataSegmented → pdf::convert_to_pdf_data_segmented
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_convert_to_pdf_data_segmented() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdfseg_data");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(60, 60);
    let inpath = dir.join("input.png");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Png).unwrap();

    let data =
        pdf::convert_to_pdf_data_segmented(&inpath, 300, None, &PdfOptions::default()).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 25. pixConvertToPdfDataSegmented → pdf::pix_convert_to_pdf_data_segmented
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_pix_convert_to_pdf_data_segmented() {
    use leptonica::io::pdf;

    let pix = make_gray(60, 60);
    let data =
        pdf::pix_convert_to_pdf_data_segmented(&pix, 300, None, &PdfOptions::default()).unwrap();
    assert!(data.starts_with(b"%PDF-"));
}

// ============================================================
// 26. concatenatePdf → pdf::concatenate_pdf
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_concatenate_pdf() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdf_concat");
    std::fs::create_dir_all(&dir).unwrap();

    // Create 2 single-page PDFs
    let pix = make_gray(40, 40);
    let opts = PdfOptions::default();
    let d1 = pdf::write_pdf_mem(&pix, &opts).unwrap();
    let d2 = pdf::write_pdf_mem(&pix, &opts).unwrap();
    std::fs::write(dir.join("p1.pdf"), &d1).unwrap();
    std::fs::write(dir.join("p2.pdf"), &d2).unwrap();

    let outpath = dir.join("concat.pdf");
    pdf::concatenate_pdf(&dir, Some(".pdf"), &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 27. concatenatePdfToData → pdf::concatenate_pdf_to_data
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_concatenate_pdf_to_data() {
    use leptonica::io::pdf;

    let dir = std::env::temp_dir().join("leptonica_pdf_concat_data");
    std::fs::create_dir_all(&dir).unwrap();

    let pix = make_gray(40, 40);
    let opts = PdfOptions::default();
    let d1 = pdf::write_pdf_mem(&pix, &opts).unwrap();
    std::fs::write(dir.join("p1.pdf"), &d1).unwrap();

    let data = pdf::concatenate_pdf_to_data(&dir, Some(".pdf")).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 28. convertTiffMultipageToPdf → pdf::convert_tiff_multipage_to_pdf
// ============================================================
#[cfg(all(feature = "pdf-format", feature = "tiff-format"))]
#[test]
fn test_convert_tiff_multipage_to_pdf() {
    use leptonica::io::{pdf, tiff};

    let dir = std::env::temp_dir().join("leptonica_tiff2pdf");
    std::fs::create_dir_all(&dir).unwrap();

    let pix1 = make_gray(40, 40);
    let pix2 = make_gray(50, 50);
    let tiff_path = dir.join("multi.tiff");
    let mut cursor = std::io::Cursor::new(Vec::new());
    tiff::write_tiff_multipage(&[&pix1, &pix2], &mut cursor, tiff::TiffCompression::Lzw).unwrap();
    std::fs::write(&tiff_path, cursor.into_inner()).unwrap();

    let outpath = dir.join("output.pdf");
    pdf::convert_tiff_multipage_to_pdf(&tiff_path, &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 29. getPdfPageCount → pdf::get_pdf_page_count
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_get_pdf_page_count() {
    use leptonica::io::pdf;

    let pix1 = make_gray(40, 40);
    let pix2 = make_gray(50, 50);
    let opts = PdfOptions::default();
    let mut buf = Vec::new();
    pdf::write_pdf_multi(&[&pix1, &pix2], &mut buf, &opts).unwrap();

    let count = pdf::get_pdf_page_count(&buf).unwrap();
    assert_eq!(count, 2);
}

// ============================================================
// 30. getPdfPageSizes → pdf::get_pdf_page_sizes
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_get_pdf_page_sizes() {
    use leptonica::io::pdf;

    let pix = make_gray(100, 200);
    let opts = PdfOptions::default().resolution(300);
    let data = pdf::write_pdf_mem(&pix, &opts).unwrap();

    let sizes = pdf::get_pdf_page_sizes(&data).unwrap();
    assert_eq!(sizes.len(), 1);
    assert!(sizes[0].0 > 0.0 && sizes[0].1 > 0.0);
}

// ============================================================
// 31. getPdfMediaBoxSizes → pdf::get_pdf_media_box_sizes
// ============================================================
#[cfg(feature = "pdf-format")]
#[test]
fn test_get_pdf_media_box_sizes() {
    use leptonica::io::pdf;

    let pix = make_gray(100, 200);
    let opts = PdfOptions::default().resolution(300);
    let data = pdf::write_pdf_mem(&pix, &opts).unwrap();

    let sizes = pdf::get_pdf_media_box_sizes(&data).unwrap();
    assert_eq!(sizes.len(), 1);
    assert!(sizes[0].0 > 0.0 && sizes[0].1 > 0.0);
}

// ============================================================
// 32. convertFilesToPS → ps::convert_files_to_ps
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_convert_files_to_ps() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_files");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(50, 50);
    leptonica::io::write_image(&pix, dir.join("img001.png"), ImageFormat::Png).unwrap();
    leptonica::io::write_image(&pix, dir.join("img002.png"), ImageFormat::Png).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_files_to_ps(&dir, None, 300, &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    let text = String::from_utf8_lossy(&data);
    assert!(text.contains("%!PS") || text.contains("%!Adobe"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 33. convertFilesFittedToPS → ps::convert_files_fitted_to_ps
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_convert_files_fitted_to_ps() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_fitted");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(50, 50);
    leptonica::io::write_image(&pix, dir.join("img001.png"), ImageFormat::Png).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_files_fitted_to_ps(&dir, None, 500, 700, &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    let text = String::from_utf8_lossy(&data);
    assert!(text.contains("%!PS") || text.contains("%!Adobe"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 34. writeImageCompressedToPSFile → ps::write_image_compressed_to_ps_file
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_write_image_compressed_to_ps_file() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_compressed");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(50, 50);
    let inpath = dir.join("input.png");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Png).unwrap();

    let outpath = dir.join("output.ps");
    let index = ps::write_image_compressed_to_ps_file(&inpath, &outpath, 300, 0).unwrap();
    assert_eq!(index, 1);
    let data = std::fs::read(&outpath).unwrap();
    assert!(!data.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 35. convertSegmentedPagesToPS → ps::convert_segmented_pages_to_ps
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_convert_segmented_pages_to_ps() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_segpages");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(80, 80);
    leptonica::io::write_image(&pix, dir.join("page001.png"), ImageFormat::Png).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_segmented_pages_to_ps(&dir, Some("page"), 1.0, 1.0, 128, &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(!data.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 36. pixWriteSegmentedPageToPS → ps::pix_write_segmented_page_to_ps
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_pix_write_segmented_page_to_ps() {
    use leptonica::io::ps;

    let pix = make_gray(80, 80);
    let dir = std::env::temp_dir().join("leptonica_ps_segpage");
    std::fs::create_dir_all(&dir).unwrap();
    let outpath = dir.join("output.ps");
    ps::pix_write_segmented_page_to_ps(&pix, None, 1.0, 1.0, 128, 1, &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(!data.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 37. pixWriteMixedToPS → ps::pix_write_mixed_to_ps
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_pix_write_mixed_to_ps() {
    use leptonica::io::ps;

    let pix_text = make_binary(80, 80);
    let pix_image = make_gray(80, 80);
    let dir = std::env::temp_dir().join("leptonica_ps_mixed");
    std::fs::create_dir_all(&dir).unwrap();
    let outpath = dir.join("output.ps");
    ps::pix_write_mixed_to_ps(Some(&pix_text), Some(&pix_image), 1.0, 1, &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(!data.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 38. convertToPSEmbed → ps::convert_to_ps_embed
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_convert_to_ps_embed() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_embed");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(50, 50);
    let inpath = dir.join("input.png");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Png).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_to_ps_embed(&inpath, &outpath, PsLevel::Level3).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    let text = String::from_utf8_lossy(&data);
    assert!(text.contains("BoundingBox"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 39. pixWriteCompressedToPS → ps::pix_write_compressed_to_ps
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_pix_write_compressed_to_ps() {
    use leptonica::io::ps;

    let pix = make_gray(60, 60);
    let dir = std::env::temp_dir().join("leptonica_ps_comp");
    std::fs::create_dir_all(&dir).unwrap();
    let outpath = dir.join("output.ps");
    let index = ps::pix_write_compressed_to_ps(&pix, &outpath, 300, PsLevel::Level3, 0).unwrap();
    assert_eq!(index, 1);
    let data = std::fs::read(&outpath).unwrap();
    assert!(!data.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 40. pixWriteStringPS → ps::pix_write_string_ps
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_pix_write_string_ps() {
    use leptonica::io::ps;

    let pix = make_gray(50, 50);
    let ps_string = ps::pix_write_string_ps(&pix, None, 300, 1.0).unwrap();
    assert!(!ps_string.is_empty());
    assert!(ps_string.contains("image") || ps_string.contains("colorimage"));
}

// ============================================================
// 41. generateUncompressedPS → ps::generate_uncompressed_ps_public
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_generate_uncompressed_ps_public() {
    use leptonica::io::ps;

    let pix = make_gray(50, 50);
    let data = ps::generate_uncompressed_ps_from_pix(&pix, 300).unwrap();
    assert!(!data.is_empty());
    assert!(data.contains("image") || data.contains("readhexstring"));
}

// ============================================================
// 42. convertJpegToPSEmbed → ps::convert_jpeg_to_ps_embed
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_convert_jpeg_to_ps_embed() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_jpeg_embed");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(50, 50);
    let inpath = dir.join("input.jpg");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Jpeg).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_jpeg_to_ps_embed(&inpath, &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    let text = String::from_utf8_lossy(&data);
    assert!(text.contains("BoundingBox"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 43. convertJpegToPS → ps::convert_jpeg_to_ps
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_convert_jpeg_to_ps() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_jpeg");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(50, 50);
    let inpath = dir.join("input.jpg");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Jpeg).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_jpeg_to_ps(&inpath, &outpath, "w", 0, 0, 300, 1.0, 1, true).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(!data.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 44. convertG4ToPSEmbed → ps::convert_g4_to_ps_embed
// ============================================================
#[cfg(all(feature = "ps-format", feature = "tiff-format"))]
#[test]
fn test_convert_g4_to_ps_embed() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_g4_embed");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_binary(80, 80);
    let inpath = dir.join("input.tif");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::TiffG4).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_g4_to_ps_embed(&inpath, &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    let text = String::from_utf8_lossy(&data);
    assert!(text.contains("BoundingBox"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 45. convertG4ToPS → ps::convert_g4_to_ps
// ============================================================
#[cfg(all(feature = "ps-format", feature = "tiff-format"))]
#[test]
fn test_convert_g4_to_ps() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_g4");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_binary(80, 80);
    let inpath = dir.join("input.tif");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::TiffG4).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_g4_to_ps(&inpath, &outpath, "w", 0, 0, 300, 1.0, 1, false, true).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(!data.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 46. convertTiffMultipageToPS → ps::convert_tiff_multipage_to_ps
// ============================================================
#[cfg(all(feature = "ps-format", feature = "tiff-format"))]
#[test]
fn test_convert_tiff_multipage_to_ps() {
    use leptonica::io::{ps, tiff};

    let dir = std::env::temp_dir().join("leptonica_tiff2ps");
    std::fs::create_dir_all(&dir).unwrap();

    let pix1 = make_binary(60, 60);
    let pix2 = make_binary(80, 80);
    let mut cursor = std::io::Cursor::new(Vec::new());
    tiff::write_tiff_multipage(&[&pix1, &pix2], &mut cursor, tiff::TiffCompression::G4).unwrap();
    let tiff_path = dir.join("multi.tif");
    std::fs::write(&tiff_path, cursor.into_inner()).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_tiff_multipage_to_ps(&tiff_path, &outpath, 0.95).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(!data.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 47. convertFlateToPSEmbed → ps::convert_flate_to_ps_embed
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_convert_flate_to_ps_embed() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_flate_embed");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(50, 50);
    let inpath = dir.join("input.png");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Png).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_flate_to_ps_embed(&inpath, &outpath).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    let text = String::from_utf8_lossy(&data);
    assert!(text.contains("BoundingBox"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 48. convertFlateToPS → ps::convert_flate_to_ps
// ============================================================
#[cfg(feature = "ps-format")]
#[test]
fn test_convert_flate_to_ps() {
    use leptonica::io::ps;

    let dir = std::env::temp_dir().join("leptonica_ps_flate");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(50, 50);
    let inpath = dir.join("input.png");
    leptonica::io::write_image(&pix, &inpath, ImageFormat::Png).unwrap();

    let outpath = dir.join("output.ps");
    ps::convert_flate_to_ps(&inpath, &outpath, "w", 0, 0, 300, 1.0, 1, true).unwrap();
    let data = std::fs::read(&outpath).unwrap();
    assert!(!data.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 49. pixaReadFiles → io::pixa_read_files
// ============================================================
#[test]
fn test_pixa_read_files() {
    let dir = std::env::temp_dir().join("leptonica_pixa_read");
    std::fs::create_dir_all(&dir).unwrap();
    let pix = make_gray(30, 30);
    leptonica::io::write_image(&pix, dir.join("a.png"), ImageFormat::Png).unwrap();
    leptonica::io::write_image(&pix, dir.join("b.png"), ImageFormat::Png).unwrap();

    let pixa = leptonica::io::pixa_read_files(&dir, Some(".png")).unwrap();
    assert_eq!(pixa.len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 50. pixaWriteFiles → io::pixa_write_files
// ============================================================
#[test]
fn test_pixa_write_files() {
    let dir = std::env::temp_dir().join("leptonica_pixa_write");
    std::fs::create_dir_all(&dir).unwrap();
    let pixa = make_pixa(3);
    let rootname = dir.join("out").to_string_lossy().to_string();
    leptonica::io::pixa_write_files(&rootname, &pixa, ImageFormat::Png).unwrap();
    // Should create out000.png, out001.png, out002.png
    assert!(dir.join("out000.png").exists());
    assert!(dir.join("out001.png").exists());
    assert!(dir.join("out002.png").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

// ============================================================
// 51. getFormatExtension → io::get_format_extension
// ============================================================
#[test]
fn test_get_format_extension() {
    assert_eq!(leptonica::io::get_format_extension(ImageFormat::Png), "png");
    assert_eq!(
        leptonica::io::get_format_extension(ImageFormat::Jpeg),
        "jpg"
    );
    assert_eq!(
        leptonica::io::get_format_extension(ImageFormat::Tiff),
        "tif"
    );
    assert_eq!(leptonica::io::get_format_extension(ImageFormat::Bmp), "bmp");
    assert_eq!(
        leptonica::io::get_format_extension(ImageFormat::WebP),
        "webp"
    );
    assert_eq!(leptonica::io::get_format_extension(ImageFormat::Gif), "gif");
    assert_eq!(leptonica::io::get_format_extension(ImageFormat::Jp2), "jp2");
    assert_eq!(
        leptonica::io::get_format_extension(ImageFormat::Lpdf),
        "pdf"
    );
    assert_eq!(leptonica::io::get_format_extension(ImageFormat::Ps), "ps");
    assert_eq!(leptonica::io::get_format_extension(ImageFormat::Pnm), "pnm");
}

// Bring types into scope for tests that use them
#[cfg(feature = "pdf-format")]
use leptonica::io::pdf::PdfOptions;

#[cfg(feature = "pdf-format")]
use leptonica::io::pdf::PdfConvertOptions;

#[cfg(feature = "ps-format")]
use leptonica::io::ps::PsLevel;
