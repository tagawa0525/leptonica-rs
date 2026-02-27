//! Tests for convertfiles module
//!
//! Tests batch file conversion to 1 bpp.

/// Test convert_files_to_1bpp with a temp directory
#[test]
fn convertfiles_basic() {
    use leptonica::io::convertfiles::convert_files_to_1bpp;
    use leptonica::{ImageFormat, Pix, PixelDepth};
    use std::path::Path;

    // Create temp directories
    let tmp_dir = std::env::temp_dir().join("leptonica_test_convertfiles");
    let in_dir = tmp_dir.join("input");
    let out_dir = tmp_dir.join("output");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&in_dir).unwrap();
    std::fs::create_dir_all(&out_dir).unwrap();

    // Create a test 8-bit image and write it
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 5..15 {
        for x in 5..15 {
            pix_mut.set_pixel(x, y, 50).unwrap(); // dark pixels
        }
    }
    let pix: Pix = pix_mut.into();
    leptonica::io::write_image(&pix, in_dir.join("test1.png"), ImageFormat::Png).unwrap();

    // Convert
    let result = convert_files_to_1bpp(
        &in_dir,
        None,
        1,   // no upscaling
        128, // threshold
        0,   // first page
        0,   // all pages
        &out_dir,
        ImageFormat::Png,
    );
    assert!(result.is_ok(), "conversion should succeed: {:?}", result);

    // Check output exists
    assert!(Path::new(&out_dir.join("test1.png")).exists());

    // Read and verify it's 1bpp
    let out_pix = leptonica::io::read_image(out_dir.join("test1.png")).unwrap();
    assert_eq!(out_pix.depth(), PixelDepth::Bit1);

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp_dir);
}

/// Test convert_files_to_1bpp with invalid upscaling
#[test]
fn convertfiles_invalid_upscaling() {
    use leptonica::ImageFormat;
    use leptonica::io::convertfiles::convert_files_to_1bpp;

    let result =
        convert_files_to_1bpp("/nonexistent", None, 3, 128, 0, 0, "/tmp", ImageFormat::Png);
    assert!(result.is_err(), "upscaling=3 should fail");
}
