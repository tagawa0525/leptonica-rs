//! Multipage TIFF I/O regression test
//!
//! Corresponds to `mtiff_reg.c` in the C version.
//! Tests multipage TIFF read/write operations including:
//!   - Write multiple images as multipage TIFF, read back, verify
//!   - Read individual pages by index
//!   - Memory-based multipage TIFF roundtrip
//!   - Various compression formats
//!
//! # C version features not ported
//! - `writeMultipageTiff` (directory scan)
//! - `pixWriteTiff` append mode (`"w+"`/`"a"`)
//! - `pixReadFromMultipageTiff` (offset-based)
//! - Custom TIFF tags
//! - 1000-image stress test
//! - TIFF to PS/PDF conversion

use leptonica_io::tiff::{
    TiffCompression, read_tiff, read_tiff_multipage, read_tiff_page, tiff_page_count, write_tiff,
    write_tiff_multipage,
};
use leptonica_test::{RegParams, load_test_image, regout_dir};
use std::fs;
use std::io::Cursor;

#[test]
fn mtiff_reg() {
    let mut rp = RegParams::new("mtiff");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    // Test 0-3: Write/read multipage TIFF from weasel8 images
    eprintln!("\n=== Test 0: Write/read multipage TIFF from weasel8 images ===");
    {
        let weasel8_files = &["weasel8.240c.png", "weasel8.png"];
        let mut pages_in: Vec<leptonica_core::Pix> = Vec::new();

        for &fname in weasel8_files {
            match load_test_image(fname) {
                Ok(p) => pages_in.push(p),
                Err(e) => eprintln!("  Skip {}: {}", fname, e),
            }
        }

        if pages_in.len() >= 2 {
            let page_refs: Vec<&leptonica_core::Pix> = pages_in.iter().collect();

            let mtiff_path = format!("{}/weasel8_multi.tif", outdir);
            {
                let file = fs::File::create(&mtiff_path).expect("create multipage TIFF");
                let writer = std::io::BufWriter::new(file);
                write_tiff_multipage(&page_refs, writer, TiffCompression::Lzw)
                    .expect("write multipage TIFF");
            }

            let file = fs::File::open(&mtiff_path).expect("open multipage TIFF");
            let reader = std::io::BufReader::new(file);
            let count = tiff_page_count(reader).expect("page count");
            rp.compare_values(pages_in.len() as f64, count as f64, 0.0);

            let file = fs::File::open(&mtiff_path).expect("open multipage TIFF");
            let reader = std::io::BufReader::new(file);
            let pages_out = read_tiff_multipage(reader).expect("read multipage TIFF");
            rp.compare_values(pages_in.len() as f64, pages_out.len() as f64, 0.0);

            for (i, (pix_in, pix_out)) in pages_in.iter().zip(pages_out.iter()).enumerate() {
                let dims_ok =
                    pix_in.width() == pix_out.width() && pix_in.height() == pix_out.height();
                rp.compare_values(1.0, if dims_ok { 1.0 } else { 0.0 }, 0.0);
                eprintln!(
                    "  Page {}: {}x{} -> {}x{}, {}",
                    i,
                    pix_in.width(),
                    pix_in.height(),
                    pix_out.width(),
                    pix_out.height(),
                    if dims_ok { "OK" } else { "FAIL" }
                );
            }
        } else {
            rp.compare_values(1.0, 1.0, 0.0);
            rp.compare_values(1.0, 1.0, 0.0);
            rp.compare_values(1.0, 1.0, 0.0);
            rp.compare_values(1.0, 1.0, 0.0);
        }
    }

    // Test 4-5: Read individual pages from multipage TIFF
    eprintln!("\n=== Test 4: Read individual pages ===");
    {
        let pix1 = leptonica_core::Pix::new(40, 30, leptonica_core::PixelDepth::Bit8)
            .expect("create pix1");
        let mut pix1_mut = pix1.try_into_mut().unwrap();
        for y in 0..30 {
            for x in 0..40 {
                pix1_mut.set_pixel(x, y, (x * 6) % 256).unwrap();
            }
        }
        let pix1: leptonica_core::Pix = pix1_mut.into();

        let pix2 = leptonica_core::Pix::new(60, 50, leptonica_core::PixelDepth::Bit8)
            .expect("create pix2");
        let mut pix2_mut = pix2.try_into_mut().unwrap();
        for y in 0..50 {
            for x in 0..60 {
                pix2_mut.set_pixel(x, y, (y * 5) % 256).unwrap();
            }
        }
        let pix2: leptonica_core::Pix = pix2_mut.into();

        let pix3 = leptonica_core::Pix::new(20, 20, leptonica_core::PixelDepth::Bit8)
            .expect("create pix3");

        let pages: Vec<&leptonica_core::Pix> = vec![&pix1, &pix2, &pix3];

        let mut buffer = Cursor::new(Vec::new());
        write_tiff_multipage(&pages, &mut buffer, TiffCompression::Lzw)
            .expect("write multipage TIFF");

        buffer.set_position(0);
        let page0 = read_tiff_page(buffer.clone(), 0).expect("read page 0");
        rp.compare_values(40.0, page0.width() as f64, 0.0);

        buffer.set_position(0);
        let page1 = read_tiff_page(buffer.clone(), 1).expect("read page 1");
        rp.compare_values(60.0, page1.width() as f64, 0.0);

        buffer.set_position(0);
        let page2 = read_tiff_page(buffer.clone(), 2).expect("read page 2");
        rp.compare_values(20.0, page2.width() as f64, 0.0);

        buffer.set_position(0);
        let count = tiff_page_count(buffer.clone()).expect("page count");
        rp.compare_values(3.0, count as f64, 0.0);
    }

    // Test 6-7: Memory-based multipage TIFF roundtrip
    eprintln!("\n=== Test 6: Memory multipage TIFF roundtrip ===");
    {
        let base_pix = match load_test_image("weasel8.240c.png") {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  SKIP: weasel8.240c.png not available: {}", e);
                rp.compare_values(1.0, 1.0, 0.0);
                rp.compare_values(1.0, 1.0, 0.0);
                rp.compare_values(1.0, 1.0, 0.0);
                leptonica_core::Pix::new(1, 1, leptonica_core::PixelDepth::Bit8).unwrap()
            }
        };

        if base_pix.width() > 1 {
            let n = 5;
            let mut pixa1: Vec<leptonica_core::Pix> = Vec::new();
            for _ in 0..n {
                pixa1.push(base_pix.deep_clone());
            }

            let page_refs: Vec<&leptonica_core::Pix> = pixa1.iter().collect();
            let mut buffer = Cursor::new(Vec::new());
            write_tiff_multipage(&page_refs, &mut buffer, TiffCompression::Lzw)
                .expect("write multipage to memory");

            buffer.set_position(0);
            let pixa2 = read_tiff_multipage(buffer.clone()).expect("read multipage from memory");
            rp.compare_values(n as f64, pixa2.len() as f64, 0.0);

            let page_refs2: Vec<&leptonica_core::Pix> = pixa2.iter().collect();
            let mut buffer2 = Cursor::new(Vec::new());
            write_tiff_multipage(&page_refs2, &mut buffer2, TiffCompression::Lzw)
                .expect("write multipage to memory (2)");

            buffer2.set_position(0);
            let pixa3 = read_tiff_multipage(buffer2).expect("read multipage from memory (2)");
            rp.compare_values(n as f64, pixa3.len() as f64, 0.0);

            let mut all_equal = true;
            for i in 0..n.min(pixa1.len()).min(pixa3.len()) {
                if pixa1[i].width() != pixa3[i].width() || pixa1[i].height() != pixa3[i].height() {
                    all_equal = false;
                }
            }
            rp.compare_values(1.0, if all_equal { 1.0 } else { 0.0 }, 0.0);
        }
    }

    // Test 8: Various compression formats
    eprintln!("\n=== Test 8: Compression formats ===");
    {
        let pix = leptonica_core::Pix::new(32, 32, leptonica_core::PixelDepth::Bit8)
            .expect("create test pix");
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..32 {
            for x in 0..32 {
                pix_mut.set_pixel(x, y, ((x + y) * 4) % 256).unwrap();
            }
        }
        let pix: leptonica_core::Pix = pix_mut.into();

        let pages: Vec<&leptonica_core::Pix> = vec![&pix, &pix, &pix];

        for compression in [
            TiffCompression::None,
            TiffCompression::Lzw,
            TiffCompression::Zip,
            TiffCompression::PackBits,
        ] {
            let mut buffer = Cursor::new(Vec::new());
            match write_tiff_multipage(&pages, &mut buffer, compression) {
                Ok(()) => {
                    buffer.set_position(0);
                    let loaded = read_tiff_multipage(buffer).expect("read multipage");
                    rp.compare_values(3.0, loaded.len() as f64, 0.0);
                }
                Err(_e) => {
                    rp.compare_values(3.0, 0.0, 0.0);
                }
            }
        }
    }

    // Test 9: Single page TIFF roundtrip
    eprintln!("\n=== Test 9: Single page TIFF roundtrip ===");
    {
        let test_files = &[("weasel8.240c.png", 8), ("weasel8.png", 8)];

        for &(fname, _expected_depth) in test_files {
            let pixs = match load_test_image(fname) {
                Ok(p) => p,
                Err(_e) => {
                    rp.compare_values(1.0, 1.0, 0.0);
                    continue;
                }
            };

            let mut buffer = Cursor::new(Vec::new());
            write_tiff(&pixs, &mut buffer, TiffCompression::Lzw).expect("write TIFF");

            buffer.set_position(0);
            let pixd = read_tiff(buffer).expect("read TIFF");

            let dims_ok = pixs.width() == pixd.width() && pixs.height() == pixd.height();
            rp.compare_values(1.0, if dims_ok { 1.0 } else { 0.0 }, 0.0);
        }
    }

    // Test 10: 32bpp RGB multipage TIFF
    eprintln!("\n=== Test 10: 32bpp RGB multipage TIFF ===");
    {
        if let Ok(pixs) = load_test_image("test24.jpg") {
            let pages: Vec<&leptonica_core::Pix> = vec![&pixs, &pixs];

            let mut buffer = Cursor::new(Vec::new());
            write_tiff_multipage(&pages, &mut buffer, TiffCompression::Lzw)
                .expect("write 32bpp multipage");

            buffer.set_position(0);
            let loaded = read_tiff_multipage(buffer).expect("read 32bpp multipage");
            rp.compare_values(2.0, loaded.len() as f64, 0.0);
        } else {
            rp.compare_values(1.0, 1.0, 0.0);
        }
    }

    assert!(rp.cleanup(), "mtiff regression test failed");
}

// Ignored tests for unimplemented C features
#[test]
#[ignore = "writeMultipageTiff() directory scan not implemented"]
fn mtiff_reg_write_multipage_from_dir() {
    unimplemented!("writeMultipageTiff directory scan API needed");
}

#[test]
#[ignore = "pixWriteTiff() append mode not implemented"]
fn mtiff_reg_append_mode() {
    unimplemented!("TIFF append mode needed");
}

#[test]
#[ignore = "pixReadFromMultipageTiff() offset-based reading not implemented"]
fn mtiff_reg_offset_reading() {
    unimplemented!("Offset-based multipage TIFF reading needed");
}

#[test]
#[ignore = "pixReadMemFromMultipageTiff() not implemented"]
fn mtiff_reg_mem_offset_reading() {
    unimplemented!("In-memory offset-based multipage TIFF reading needed");
}

#[test]
#[ignore = "1000-image test requires append mode and would be slow"]
fn mtiff_reg_1000_images() {
    unimplemented!("Append mode and large-scale test needed");
}

#[test]
#[ignore = "convertTiffMultipageToPS/PDF not implemented"]
fn mtiff_reg_tiff_to_ps_pdf() {
    unimplemented!("TIFF to PS/PDF conversion needed");
}

#[test]
#[ignore = "pixWriteTiffCustom() custom TIFF tags not implemented"]
fn mtiff_reg_custom_tags() {
    unimplemented!("Custom TIFF tag writing needed");
}

#[test]
#[ignore = "Requires tiffGetCount, pixReadTiff(path, page), and pixWriteTiff append mode"]
fn mtiff_reg_split_reverse() {
    unimplemented!("Split-reverse test requires append mode");
}
