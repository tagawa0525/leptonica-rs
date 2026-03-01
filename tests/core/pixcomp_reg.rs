//! Compressed Pix (PixComp) regression test
//!
//! Tests compressed pix and compressed pix arrays in memory.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pixcomp_reg.c`

use crate::common::RegParams;
use leptonica::core::pixcomp::{PixComp, PixaComp};
use leptonica::{ImageFormat, Pixa};

/// Test Pixa array operations as partial substitute for PixAComp (C checks 0-2).
#[test]
fn pixcomp_reg_pixa_array() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("pixcomp_pixa");

    let images = ["marge.jpg", "weasel4.16c.png", "weasel8.149g.png"];

    let mut pixa = Pixa::new();
    for img in &images {
        let pix = crate::common::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        pixa.push(pix);
    }

    // Verify count
    rp.compare_values(images.len() as f64, pixa.len() as f64, 0.0);

    // Verify element access preserves dimensions
    for (i, img) in images.iter().enumerate() {
        let pix_orig = crate::common::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        let pix_ref = pixa.get(i).unwrap_or_else(|| panic!("get {i}"));
        rp.compare_values(pix_orig.width() as f64, pix_ref.width() as f64, 0.0);
        rp.compare_values(pix_orig.height() as f64, pix_ref.height() as f64, 0.0);
    }

    assert!(rp.cleanup(), "pixcomp pixa array test failed");
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
