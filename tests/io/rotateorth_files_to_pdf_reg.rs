//! Regression tests for `rotateorth_files_to_pdf` (plan 811).
//!
//! C Leptonica: `prog/rotateorthpdf.c` (CLI driving `rotateorthFilesToPdf`).

use leptonica::io::pdf::{PdfCompression, rotateorth_files_to_pdf};

fn unique_tmp(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "leptonica_rotateorth_files_to_pdf_{tag}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_test_image_to(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/images")
        .join(name);
    let dst = dir.join(name);
    std::fs::copy(&src, &dst)
        .unwrap_or_else(|e| panic!("copy {} -> {}: {e}", src.display(), dst.display()));
    dst
}

#[test]
#[ignore = "plan 811: not yet implemented"]
fn rotateorth_files_to_pdf_basic() {
    let dir = unique_tmp("basic");
    let p1 = copy_test_image_to(&dir, "karen8.jpg");
    let p2 = copy_test_image_to(&dir, "marge.jpg");
    let out = dir.join("out.pdf");

    rotateorth_files_to_pdf(
        &[p1, p2],
        "00", // no rotation
        1.0,
        75,
        None,
        PdfCompression::Auto,
        &out,
    )
    .unwrap();

    let data = std::fs::read(&out).unwrap();
    assert!(data.starts_with(b"%PDF"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "plan 811: not yet implemented"]
fn rotateorth_files_to_pdf_with_rotation_and_scale() {
    let dir = unique_tmp("rot");
    let p1 = copy_test_image_to(&dir, "karen8.jpg");
    let p2 = copy_test_image_to(&dir, "marge.jpg");
    let out = dir.join("out.pdf");

    rotateorth_files_to_pdf(
        &[p1, p2],
        "13", // image 0 rotated 90 cw, image 1 rotated 270 cw
        0.5,
        75,
        Some("rotated"),
        PdfCompression::Auto,
        &out,
    )
    .unwrap();

    let data = std::fs::read(&out).unwrap();
    assert!(data.starts_with(b"%PDF"));
    assert!(
        data.windows(b"rotated".len()).any(|w| w == b"rotated"),
        "PDF should embed the supplied title"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "plan 811: not yet implemented"]
fn rotateorth_files_to_pdf_rejects_empty_paths() {
    let dir = unique_tmp("empty");
    let out = dir.join("out.pdf");
    let paths: &[std::path::PathBuf] = &[];
    assert!(
        rotateorth_files_to_pdf(paths, "0", 1.0, 75, None, PdfCompression::Auto, &out).is_err()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "plan 811: not yet implemented"]
fn rotateorth_files_to_pdf_rejects_invalid_rotstring() {
    let dir = unique_tmp("badrot");
    let p1 = copy_test_image_to(&dir, "karen8.jpg");
    let out = dir.join("out.pdf");
    assert!(
        rotateorth_files_to_pdf(
            &[p1],
            "", // empty rotstring
            1.0,
            75,
            None,
            PdfCompression::Auto,
            &out,
        )
        .is_err()
    );
    let _ = std::fs::remove_dir_all(&dir);
}
