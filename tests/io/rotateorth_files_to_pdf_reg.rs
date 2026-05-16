//! Regression tests for `rotate_orth_files_to_pdf_file` (plan 811).
//!
//! The bytes-returning sibling `rotate_orth_files_to_pdf` is already
//! covered by the lib-internal tests in `src/io/pdf.rs` (plan 030).
//! These tests exercise the file-output form added by plan 811 plus a
//! transformation-effect check that goes beyond a `%PDF` magic-byte sniff.
//!
//! C Leptonica: `prog/rotateorthpdf.c` (CLI driving `rotateorthFilesToPdf`).

use leptonica::io::pdf::{rotate_orth_files_to_pdf, rotate_orth_files_to_pdf_file};

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
fn rotate_orth_files_to_pdf_file_basic() {
    let dir = unique_tmp("basic");
    let p1 = copy_test_image_to(&dir, "karen8.jpg");
    let p2 = copy_test_image_to(&dir, "marge.jpg");
    let out = dir.join("out.pdf");

    rotate_orth_files_to_pdf_file(&[p1, p2], "00", 1.0, 75, "none", &out).unwrap();

    let data = std::fs::read(&out).unwrap();
    assert!(data.starts_with(b"%PDF"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rotate_orth_files_to_pdf_file_embeds_title() {
    let dir = unique_tmp("title");
    let p1 = copy_test_image_to(&dir, "karen8.jpg");
    let out = dir.join("out.pdf");

    rotate_orth_files_to_pdf_file(&[p1], "0", 1.0, 75, "rotated", &out).unwrap();

    let data = std::fs::read(&out).unwrap();
    assert!(data.starts_with(b"%PDF"));
    assert!(
        data.windows(b"rotated".len()).any(|w| w == b"rotated"),
        "PDF should embed the supplied title"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Rotating by 1 (90° cw) must produce a different byte stream from
/// rotating by 0 — protects against the rotation argument being silently
/// dropped.
#[test]
fn rotate_orth_files_to_pdf_actually_rotates() {
    let dir = unique_tmp("rotates");
    let p = copy_test_image_to(&dir, "karen8.jpg");

    let no_rot = rotate_orth_files_to_pdf(&[&p], "0", 1.0, 75, "none").unwrap();
    let rot90 = rotate_orth_files_to_pdf(&[&p], "1", 1.0, 75, "none").unwrap();
    assert_ne!(
        no_rot, rot90,
        "rotation should change the encoded PDF bytes"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// scalefactor < 1.0 must produce a smaller (or different) byte stream
/// than scalefactor = 1.0 for the same input — protects against scaling
/// being silently dropped.
#[test]
fn rotate_orth_files_to_pdf_actually_scales() {
    let dir = unique_tmp("scales");
    let p = copy_test_image_to(&dir, "karen8.jpg");

    let scale_full = rotate_orth_files_to_pdf(&[&p], "0", 1.0, 75, "none").unwrap();
    let scale_half = rotate_orth_files_to_pdf(&[&p], "0", 0.5, 75, "none").unwrap();
    assert_ne!(
        scale_full, scale_half,
        "scalefactor should change the encoded PDF bytes"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rotate_orth_files_to_pdf_file_rejects_empty_paths() {
    let dir = unique_tmp("empty");
    let out = dir.join("out.pdf");
    let paths: &[std::path::PathBuf] = &[];
    assert!(rotate_orth_files_to_pdf_file(paths, "0", 1.0, 75, "none", &out).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rotate_orth_files_to_pdf_file_rejects_invalid_rotstring() {
    let dir = unique_tmp("badrot");
    let p = copy_test_image_to(&dir, "karen8.jpg");
    let out = dir.join("out.pdf");
    assert!(rotate_orth_files_to_pdf_file(&[p], "", 1.0, 75, "none", &out).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
