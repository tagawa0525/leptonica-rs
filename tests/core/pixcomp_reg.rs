//! Compressed Pix (PixComp) regression test
//!
//! Tests compressed pix and compressed pix arrays in memory.
//!
//! # See also
//!
//! C Leptonica: `prog/pixcomp_reg.c`

use crate::common::RegParams;
use leptonica::core::pixcomp::{PixComp, PixaComp};
use leptonica::{ImageFormat, Pixa};

/// Test PixComp round-trip with WPAC (C checks 0-3).
///
/// Creates PixComp from images in various formats, converts back,
/// and writes results to golden manifest.
#[test]
fn pixcomp_reg_roundtrip_wpac() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("pixcomp_rt");

    // C check 0: JPEG round-trip (dimension check only, no WPAC — JPEG is lossy)
    let pix1 = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let pc1 = PixComp::create_from_pix(&pix1, Some(ImageFormat::Jpeg)).unwrap();
    let recovered1 = pc1.to_pix().unwrap();
    rp.compare_values(pix1.width() as f64, recovered1.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, recovered1.height() as f64, 0.0);

    // C check 2: TIFF_G4 round-trip (1bpp)
    let pix_bin = crate::common::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let pc2 = PixComp::create_from_pix(&pix_bin, Some(ImageFormat::TiffG4)).unwrap();
    let recovered2 = pc2.to_pix().unwrap();
    rp.compare_values(pix_bin.width() as f64, recovered2.width() as f64, 0.0);
    rp.write_pix_and_check(&recovered2, ImageFormat::Tiff)
        .expect("check: pixcomp tiff_g4 roundtrip");

    // C check 3: PNG round-trip (8bpp)
    let pix8 = crate::common::load_test_image("weasel8.png").expect("load weasel8.png");
    let pc3 = PixComp::create_from_pix(&pix8, Some(ImageFormat::Png)).unwrap();
    let recovered3 = pc3.to_pix().unwrap();
    rp.compare_values(pix8.width() as f64, recovered3.width() as f64, 0.0);
    rp.write_pix_and_check(&recovered3, ImageFormat::Png)
        .expect("check: pixcomp png roundtrip");

    // PixaComp serialization round-trip
    let mut pac = PixaComp::create(3);
    pac.add_pix(&pix1, None).unwrap();
    pac.add_pix(&pix8, None).unwrap();
    let data = pac.write_mem().unwrap();
    let pac2 = PixaComp::read_mem(&data).unwrap();
    rp.compare_values(2.0, pac2.get_count() as f64, 0.0);

    let rpix = pac2.get_pix(1).unwrap();
    rp.write_pix_and_check(&rpix, ImageFormat::Png)
        .expect("check: pixacomp serialization roundtrip");

    assert!(rp.cleanup(), "pixcomp roundtrip wpac test failed");
}

/// Test PixComp creation and round-trip.
#[test]
fn pixcomp_reg_create_roundtrip() {
    let pix = crate::common::load_test_image("marge.jpg").unwrap();
    let pixcomp = PixComp::create_from_pix(&pix, Some(ImageFormat::Png)).unwrap();
    let (w, h, _d) = pixcomp.get_dimensions();
    assert_eq!(w, pix.width());
    assert_eq!(h, pix.height());

    let recovered = pixcomp.to_pix().unwrap();
    assert_eq!(recovered.width(), pix.width());
    assert_eq!(recovered.height(), pix.height());
}

/// Test PixaComp creation from Pixa.
#[test]
fn pixcomp_reg_pixacomp_from_pixa() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut pixa = Pixa::new();
    let pix1 = crate::common::load_test_image("marge.jpg").unwrap();
    let pix2 = crate::common::load_test_image("weasel4.16c.png").unwrap();
    pixa.push(pix1);
    pixa.push(pix2);

    let pixacomp = PixaComp::create_from_pixa(&pixa, None).unwrap();
    assert_eq!(pixacomp.get_count(), 2);

    // Verify dimensions without decompression
    let (w, h, _d) = pixacomp.get_pix_dimensions(0).unwrap();
    assert_eq!(w, pixa.get(0).unwrap().width());
    assert_eq!(h, pixa.get(0).unwrap().height());
}

/// Test PixaComp add/replace/get_pix operations.
#[test]
fn pixcomp_reg_pixacomp_operations() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut pixacomp = PixaComp::create(4);
    let pix = crate::common::load_test_image("marge.jpg").unwrap();

    pixacomp.add_pix(&pix, None).unwrap();
    assert_eq!(pixacomp.get_count(), 1);

    let recovered = pixacomp.get_pix(0).unwrap();
    assert_eq!(recovered.width(), pix.width());
    assert_eq!(recovered.height(), pix.height());

    // Replace
    let pix2 = crate::common::load_test_image("weasel4.16c.png").unwrap();
    pixacomp.replace_pix(0, &pix2, None).unwrap();
    let replaced = pixacomp.get_pix(0).unwrap();
    assert_eq!(replaced.width(), pix2.width());
}

/// Test PixaComp serialization round-trip.
#[test]
fn pixcomp_reg_serialization() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut pixacomp = PixaComp::create(2);
    let pix = crate::common::load_test_image("marge.jpg").unwrap();
    pixacomp.add_pix(&pix, None).unwrap();

    let data = pixacomp.write_mem().unwrap();
    let recovered = PixaComp::read_mem(&data).unwrap();
    assert_eq!(recovered.get_count(), 1);

    let rpix = recovered.get_pix(0).unwrap();
    assert_eq!(rpix.width(), pix.width());
    assert_eq!(rpix.height(), pix.height());
}

/// Test PixaComp join and interleave.
#[test]
fn pixcomp_reg_join_interleave() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut pac1 = PixaComp::create(2);
    let mut pac2 = PixaComp::create(2);
    let pix = crate::common::load_test_image("marge.jpg").unwrap();
    pac1.add_pix(&pix, None).unwrap();
    pac2.add_pix(&pix, None).unwrap();

    pac1.join(&pac2, 0, None).unwrap();
    assert_eq!(pac1.get_count(), 2);

    let interleaved = pac1.interleave(&pac2);
    assert_eq!(interleaved.get_count(), 3); // 2 from pac1 + 1 from pac2

    // Test to_pixa
    let pixa = pac1.to_pixa().unwrap();
    assert_eq!(pixa.len(), 2);
}

/// Test PixComp string creation and parameters.
#[test]
fn pixcomp_reg_from_string() {
    if crate::common::is_display_mode() {
        return;
    }

    let data = vec![0u8; 100];
    let pixcomp =
        PixComp::create_from_string(data, 10, 10, leptonica::PixelDepth::Bit8, ImageFormat::Png);
    let (w, h, d) = pixcomp.get_dimensions();
    assert_eq!(w, 10);
    assert_eq!(h, 10);
    assert_eq!(d, leptonica::PixelDepth::Bit8);

    let (_xres, _yres, comptype, _cmap) = pixcomp.get_parameters();
    assert_eq!(comptype, ImageFormat::Png);
}

/// Test PixaComp with init.
#[test]
fn pixcomp_reg_create_with_init() {
    if crate::common::is_display_mode() {
        return;
    }

    let pix = leptonica::Pix::new(20, 20, leptonica::PixelDepth::Bit8).unwrap();
    let pac = PixaComp::create_with_init(3, 0, Some(&pix), None).unwrap();
    assert_eq!(pac.get_count(), 3);
}

/// Test PixaComp write files and PixComp write file.
#[test]
fn pixcomp_reg_write_files() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut pac = PixaComp::create(2);
    let pix = leptonica::Pix::new(20, 20, leptonica::PixelDepth::Bit8).unwrap();
    pac.add_pix(&pix, Some(ImageFormat::Png)).unwrap();

    let outdir = std::path::PathBuf::from(crate::common::regout_dir()).join("pixcomp_files");
    let _ = std::fs::create_dir_all(&outdir);
    let rootname = outdir.join("test_");
    pac.write_files(rootname.to_str().unwrap()).unwrap();

    // Verify file was created
    let expected = outdir.join("test_000.png");
    assert!(expected.exists());
    let _ = std::fs::remove_dir_all(&outdir);
}
