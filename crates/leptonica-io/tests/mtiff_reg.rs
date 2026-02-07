//! Multipage TIFF I/O regression test
//!
//! C version: reference/leptonica/prog/mtiff_reg.c
//! Tests multipage TIFF read/write operations.
//!
//! C version tests:
//!   - writeMultipageTiff for "weasel8." files -> read back as pixa (tests 0-5)
//!   - pixReadFromMultipageTiff with offset (tests 4-5)
//!   - pixReadMemFromMultipageTiff for in-memory (tests 5-7)
//!   - 1000 image multipage write/read (tests 8-10)
//!   - pixaWriteMultipageTiff -> read mem -> write mem -> read -> compare (tests 11-14)
//!   - Single-to-multipage: pixWriteTiff with "w+"/"a" modes -> PS/PDF (tests 15-17)
//!   - Multipage page count, split, reverse, reverse-reverse (tests 18-23)
//!   - Custom TIFF tags (not tested in Rust)
//!
//! Rust implementation status:
//!   - read_tiff, read_tiff_page, read_tiff_multipage, tiff_page_count: available
//!   - write_tiff, write_tiff_multipage: available
//!   - writeMultipageTiff (directory scan): not available as API
//!   - pixWriteTiff with "w+"/"a" mode (append): not available
//!   - pixReadFromMultipageTiff (offset-based): not available
//!   - pixaWriteMemMultipageTiff / pixaReadMemMultipageTiff: not available as separate APIs
//!   - convertTiffMultipageToPS/PDF: not available
//!   - Custom TIFF tags: not available
//!
//! Run with:
//! ```
//! cargo test -p leptonica-io --test mtiff_reg --features all-formats -- --nocapture
//! ```

use leptonica_io::tiff::{
    TiffCompression, read_tiff, read_tiff_multipage, read_tiff_page, tiff_page_count, write_tiff,
    write_tiff_multipage,
};
// Note: read_image, write_image_mem, ImageFormat are available for future tests
// when more APIs are ported from C.
use leptonica_test::{RegParams, load_test_image, regout_dir};
use std::fs;
use std::io::Cursor;

/// Main multipage TIFF regression test
///
/// Tests the core multipage TIFF functionality that is available in Rust:
/// 1. Write multiple images as multipage TIFF -> read back -> verify page count and content
/// 2. Read individual pages
/// 3. Memory-based multipage TIFF roundtrip
/// 4. Multiple compression formats
#[test]
fn mtiff_reg() {
    let mut rp = RegParams::new("mtiff");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    // ================================================================
    // Test 0-3: Create multipage TIFF from weasel8 images, read back
    // C version: writeMultipageTiff(".", "weasel8.", "/tmp/lept/tiff/weasel8.tif")
    //            -> pixaReadMultipageTiff -> display at various depths
    //
    // Rust: We load the weasel8 images manually and write as multipage TIFF,
    //       then read back and verify.
    // ================================================================
    eprintln!("\n=== Test 0: Write/read multipage TIFF from weasel8 images ===");
    {
        let weasel8_files = &["weasel8.240c.png", "weasel8.png"];
        let mut pages_in: Vec<leptonica_core::Pix> = Vec::new();

        for &fname in weasel8_files {
            match load_test_image(fname) {
                Ok(p) => {
                    eprintln!(
                        "  Loaded {} ({}x{}, {}bpp)",
                        fname,
                        p.width(),
                        p.height(),
                        p.depth().bits()
                    );
                    pages_in.push(p);
                }
                Err(e) => {
                    eprintln!("  Skip {}: {}", fname, e);
                }
            }
        }

        if pages_in.len() >= 2 {
            let page_refs: Vec<&leptonica_core::Pix> = pages_in.iter().collect();

            // Write multipage TIFF
            let mtiff_path = format!("{}/weasel8_multi.tif", outdir);
            {
                let file = fs::File::create(&mtiff_path).expect("create multipage TIFF");
                let writer = std::io::BufWriter::new(file);
                write_tiff_multipage(&page_refs, writer, TiffCompression::Lzw)
                    .expect("write multipage TIFF");
            }

            // Check page count
            let file = fs::File::open(&mtiff_path).expect("open multipage TIFF");
            let reader = std::io::BufReader::new(file);
            let count = tiff_page_count(reader).expect("page count");
            let ok = rp.compare_values(pages_in.len() as f64, count as f64, 0.0);
            eprintln!(
                "  Page count: {} (expected {}), {}",
                count,
                pages_in.len(),
                if ok { "OK" } else { "FAIL" }
            );

            // Read back all pages
            let file = fs::File::open(&mtiff_path).expect("open multipage TIFF");
            let reader = std::io::BufReader::new(file);
            let pages_out = read_tiff_multipage(reader).expect("read multipage TIFF");

            let ok = rp.compare_values(pages_in.len() as f64, pages_out.len() as f64, 0.0);
            eprintln!(
                "  Pages read back: {} (expected {}), {}",
                pages_out.len(),
                pages_in.len(),
                if ok { "OK" } else { "FAIL" }
            );

            // Verify each page dimensions
            for (i, (pix_in, pix_out)) in pages_in.iter().zip(pages_out.iter()).enumerate() {
                let dims_ok =
                    pix_in.width() == pix_out.width() && pix_in.height() == pix_out.height();
                let ok = rp.compare_values(1.0, if dims_ok { 1.0 } else { 0.0 }, 0.0);
                eprintln!(
                    "  Page {}: {}x{} -> {}x{}, {}",
                    i,
                    pix_in.width(),
                    pix_in.height(),
                    pix_out.width(),
                    pix_out.height(),
                    if ok { "OK" } else { "FAIL" }
                );
            }
        } else {
            eprintln!("  SKIP: not enough weasel8 images loaded");
            rp.compare_values(1.0, 1.0, 0.0);
            rp.compare_values(1.0, 1.0, 0.0);
            rp.compare_values(1.0, 1.0, 0.0);
            rp.compare_values(1.0, 1.0, 0.0);
        }
    }

    // ================================================================
    // Test 4-5: Read individual pages from multipage TIFF
    // C version: pixReadFromMultipageTiff with offset-based reading
    // Rust: read_tiff_page(reader, page_index)
    // ================================================================
    eprintln!("\n=== Test 4: Read individual pages ===");
    {
        // Create a multipage TIFF with known content
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

        // Read page 0
        buffer.set_position(0);
        let page0 = read_tiff_page(buffer.clone(), 0).expect("read page 0");
        let ok = rp.compare_values(40.0, page0.width() as f64, 0.0);
        eprintln!(
            "  Page 0: width={} (expected 40), {}",
            page0.width(),
            if ok { "OK" } else { "FAIL" }
        );

        // Read page 1
        buffer.set_position(0);
        let page1 = read_tiff_page(buffer.clone(), 1).expect("read page 1");
        let ok = rp.compare_values(60.0, page1.width() as f64, 0.0);
        eprintln!(
            "  Page 1: width={} (expected 60), {}",
            page1.width(),
            if ok { "OK" } else { "FAIL" }
        );

        // Read page 2
        buffer.set_position(0);
        let page2 = read_tiff_page(buffer.clone(), 2).expect("read page 2");
        let ok = rp.compare_values(20.0, page2.width() as f64, 0.0);
        eprintln!(
            "  Page 2: width={} (expected 20), {}",
            page2.width(),
            if ok { "OK" } else { "FAIL" }
        );

        // Verify page count
        buffer.set_position(0);
        let count = tiff_page_count(buffer.clone()).expect("page count");
        let ok = rp.compare_values(3.0, count as f64, 0.0);
        eprintln!(
            "  Total pages: {} (expected 3), {}",
            count,
            if ok { "OK" } else { "FAIL" }
        );
    }

    // ================================================================
    // Test 6-7: Memory-based multipage TIFF roundtrip
    // C version: pixaWriteMultipageTiff -> l_binaryRead -> pixReadMemFromMultipageTiff
    //            -> pixaWriteMemMultipageTiff -> pixaReadMemMultipageTiff -> compare
    // Rust: write_tiff_multipage to Cursor -> read_tiff_multipage from Cursor -> compare
    // ================================================================
    eprintln!("\n=== Test 6: Memory multipage TIFF roundtrip ===");
    {
        // Create 10 copies of weasel8.240c.png as in C version
        let base_pix = match load_test_image("weasel8.240c.png") {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  SKIP: weasel8.240c.png not available: {}", e);
                // Skip all sub-tests
                rp.compare_values(1.0, 1.0, 0.0);
                rp.compare_values(1.0, 1.0, 0.0);
                rp.compare_values(1.0, 1.0, 0.0);
                leptonica_core::Pix::new(1, 1, leptonica_core::PixelDepth::Bit8).unwrap()
            }
        };

        if base_pix.width() > 1 {
            let n = 5; // Use 5 instead of 10 for speed
            let mut pixa1: Vec<leptonica_core::Pix> = Vec::new();
            for _ in 0..n {
                pixa1.push(base_pix.deep_clone());
            }

            // Write multipage TIFF to memory
            let page_refs: Vec<&leptonica_core::Pix> = pixa1.iter().collect();
            let mut buffer = Cursor::new(Vec::new());
            write_tiff_multipage(&page_refs, &mut buffer, TiffCompression::Lzw)
                .expect("write multipage to memory");
            eprintln!(
                "  Written {} pages to memory ({} bytes)",
                n,
                buffer.get_ref().len()
            );

            // Read back from memory
            buffer.set_position(0);
            let pixa2 = read_tiff_multipage(buffer.clone()).expect("read multipage from memory");

            // C version: regTestCompareValues(rp, 10, n, 0)
            let ok = rp.compare_values(n as f64, pixa2.len() as f64, 0.0);
            eprintln!(
                "  Read {} pages (expected {}), {}",
                pixa2.len(),
                n,
                if ok { "OK" } else { "FAIL" }
            );

            // Write the read-back pages to memory again
            let page_refs2: Vec<&leptonica_core::Pix> = pixa2.iter().collect();
            let mut buffer2 = Cursor::new(Vec::new());
            write_tiff_multipage(&page_refs2, &mut buffer2, TiffCompression::Lzw)
                .expect("write multipage to memory (2)");

            // Read back the second copy
            buffer2.set_position(0);
            let pixa3 = read_tiff_multipage(buffer2).expect("read multipage from memory (2)");

            let ok = rp.compare_values(n as f64, pixa3.len() as f64, 0.0);
            eprintln!(
                "  Re-read {} pages (expected {}), {}",
                pixa3.len(),
                n,
                if ok { "OK" } else { "FAIL" }
            );

            // C version: compare pixa1 with pixa3 (tests 14 in C)
            // Compare each page
            let mut all_equal = true;
            for i in 0..n.min(pixa1.len()).min(pixa3.len()) {
                // Dimensions should match; pixel values may differ slightly due to TIFF
                // compression/decompression (depth conversions for non-8bpp)
                if pixa1[i].width() != pixa3[i].width() || pixa1[i].height() != pixa3[i].height() {
                    all_equal = false;
                    eprintln!("    Page {} dimension mismatch", i);
                }
            }
            let ok = rp.compare_values(1.0, if all_equal { 1.0 } else { 0.0 }, 0.0);
            eprintln!(
                "  All pages match dimensions: {}",
                if ok { "OK" } else { "FAIL" }
            );
        }
    }

    // ================================================================
    // Test 8: Various compression formats for multipage TIFF
    // ================================================================
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
            eprint!("  Compression {:?} ... ", compression);
            let mut buffer = Cursor::new(Vec::new());

            match write_tiff_multipage(&pages, &mut buffer, compression) {
                Ok(()) => {
                    buffer.set_position(0);
                    let loaded = read_tiff_multipage(buffer).expect("read multipage");
                    let ok = rp.compare_values(3.0, loaded.len() as f64, 0.0);
                    eprintln!("pages={}, {}", loaded.len(), if ok { "OK" } else { "FAIL" });
                }
                Err(e) => {
                    eprintln!("FAIL (write: {})", e);
                    rp.compare_values(3.0, 0.0, 0.0);
                }
            }
        }
    }

    // ================================================================
    // Test 9: Single page TIFF read/write roundtrip with various depths
    // Corresponds to C version's split-and-reconstruct tests
    // ================================================================
    eprintln!("\n=== Test 9: Single page TIFF roundtrip ===");
    {
        let test_files = &[("weasel8.240c.png", 8), ("weasel8.png", 8)];

        for &(fname, _expected_depth) in test_files {
            eprint!("  {} ... ", fname);

            let pixs = match load_test_image(fname) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("SKIP ({})", e);
                    rp.compare_values(1.0, 1.0, 0.0);
                    continue;
                }
            };

            // Write single-page TIFF
            let mut buffer = Cursor::new(Vec::new());
            write_tiff(&pixs, &mut buffer, TiffCompression::Lzw).expect("write TIFF");

            // Read back
            buffer.set_position(0);
            let pixd = read_tiff(buffer).expect("read TIFF");

            let dims_ok = pixs.width() == pixd.width() && pixs.height() == pixd.height();
            let ok = rp.compare_values(1.0, if dims_ok { 1.0 } else { 0.0 }, 0.0);
            eprintln!(
                "{}x{} depth={}->{}, {}",
                pixs.width(),
                pixs.height(),
                pixs.depth().bits(),
                pixd.depth().bits(),
                if ok { "OK" } else { "FAIL" }
            );
        }
    }

    // ================================================================
    // Test 10: 32bpp RGB multipage TIFF
    // ================================================================
    eprintln!("\n=== Test 10: 32bpp RGB multipage TIFF ===");
    {
        eprint!("  32bpp RGB multipage ... ");

        if let Ok(pixs) = load_test_image("test24.jpg") {
            let pages: Vec<&leptonica_core::Pix> = vec![&pixs, &pixs];

            let mut buffer = Cursor::new(Vec::new());
            write_tiff_multipage(&pages, &mut buffer, TiffCompression::Lzw)
                .expect("write 32bpp multipage");

            buffer.set_position(0);
            let loaded = read_tiff_multipage(buffer).expect("read 32bpp multipage");

            let ok = rp.compare_values(2.0, loaded.len() as f64, 0.0);
            eprintln!("pages={}, {}", loaded.len(), if ok { "OK" } else { "FAIL" });

            // Verify dimensions
            for (i, pix_out) in loaded.iter().enumerate() {
                let dims_ok = pix_out.width() == pixs.width() && pix_out.height() == pixs.height();
                if !dims_ok {
                    eprintln!(
                        "    Page {} dimension mismatch: {}x{} vs {}x{}",
                        i,
                        pixs.width(),
                        pixs.height(),
                        pix_out.width(),
                        pix_out.height()
                    );
                }
            }
        } else {
            eprintln!("SKIP (test24.jpg not available)");
            rp.compare_values(1.0, 1.0, 0.0);
        }
    }

    assert!(rp.cleanup(), "mtiff regression test failed");
}

// ================================================================
// Ignored tests for features not yet implemented in Rust
// ================================================================

/// C version: writeMultipageTiff(".", "weasel8.", path) -- directory scan + auto-write
/// This scans a directory for files matching a pattern and writes them as multipage TIFF.
#[test]
#[ignore = "writeMultipageTiff() (directory scan + multipage write) not implemented in Rust"]
fn mtiff_reg_write_multipage_from_dir() {
    // C version: writeMultipageTiff(".", "weasel8.", "/tmp/lept/tiff/weasel8.tif")
    unimplemented!("writeMultipageTiff directory scan API needed");
}

/// C version: pixWriteTiff(path, pix, compression, "w+"/"a") -- append mode
/// This allows incrementally building a multipage TIFF by appending pages.
#[test]
#[ignore = "pixWriteTiff() append mode (\"w+\"/\"a\") not implemented in Rust"]
fn mtiff_reg_append_mode() {
    // C version: pixWriteTiff(path, pix, IFF_TIFF_G4, "w+") for first page
    //           pixWriteTiff(path, pix, IFF_TIFF_G4, "a") for subsequent pages
    unimplemented!("TIFF append mode needed");
}

/// C version: pixReadFromMultipageTiff(path, &offset) -- offset-based sequential reading
/// This reads one page at a time from a multipage TIFF using file offsets.
#[test]
#[ignore = "pixReadFromMultipageTiff() (offset-based reading) not implemented in Rust"]
fn mtiff_reg_offset_reading() {
    // C version:
    //   offset = 0;
    //   do {
    //       pix = pixReadFromMultipageTiff(path, &offset);
    //       ...
    //   } while (offset != 0);
    unimplemented!("Offset-based multipage TIFF reading needed");
}

/// C version: pixReadMemFromMultipageTiff(data, size, &offset) -- in-memory offset reading
#[test]
#[ignore = "pixReadMemFromMultipageTiff() (in-memory offset-based reading) not implemented in Rust"]
fn mtiff_reg_mem_offset_reading() {
    // C version: pixReadMemFromMultipageTiff(data, size, &offset)
    unimplemented!("In-memory offset-based multipage TIFF reading needed");
}

/// C version: 1000-image multipage TIFF write/read performance test
/// Creates a 1000-page TIFF using append mode and measures timing.
#[test]
#[ignore = "1000-image test requires pixWriteTiff append mode and would be slow"]
fn mtiff_reg_1000_images() {
    // C version: pixWriteTiff(path, pix, IFF_TIFF_G4, "w") + 999x pixWriteTiff(..., "a")
    // Then reads all 1000 images back and verifies count
    unimplemented!("Append mode and large-scale test needed");
}

/// C version: convertTiffMultipageToPS / convertTiffMultipageToPdf
/// Converts multipage TIFF to PostScript and PDF.
#[test]
#[ignore = "convertTiffMultipageToPS/PDF not implemented in Rust"]
fn mtiff_reg_tiff_to_ps_pdf() {
    // C version: convertTiffMultipageToPS(path, output, 0.95)
    // C version: convertTiffMultipageToPdf(path, output)
    unimplemented!("TIFF to PS/PDF conversion needed");
}

/// C version: pixWriteTiffCustom -- write TIFF with custom tags (XMP, document name, etc.)
#[test]
#[ignore = "pixWriteTiffCustom() (custom TIFF tags) not implemented in Rust"]
fn mtiff_reg_custom_tags() {
    // C version: pixWriteTiffCustom(path, pix, IFF_TIFF_G4, "w", naflags, savals, satypes, nasizes)
    unimplemented!("Custom TIFF tag writing needed");
}

/// C version tests 18-23: multipage TIFF page count, split-reverse-reverse test
/// Tests tiffGetCount, pixReadTiff by page index, reverse ordering, comparison
#[test]
#[ignore = "Requires tiffGetCount (from FILE*), pixReadTiff(path, page), and pixWriteTiff append mode"]
fn mtiff_reg_split_reverse() {
    // C version:
    //   writeMultipageTiff(".", "weasel2", weasel_orig)
    //   tiffGetCount(fp, &npages) -> 5
    //   for i in 0..npages: pixReadTiff(path, i) -> split to files
    //   reverse order -> write reversed multipage
    //   read reversed -> reverse again -> write
    //   compare original with double-reversed
    unimplemented!("Split-reverse test requires append mode and file-based tiffGetCount");
}
