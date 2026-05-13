//! Regression tests for plan 126 (Pixa::select_to_pdf).

#![cfg(feature = "pdf-format")]

use leptonica::core::pixa::Pixa;
use leptonica::io::pdf::PdfOptions;
use leptonica::{PixMut, PixelDepth};

fn pixa_2_pages() -> Pixa {
    let mut pa = Pixa::new();
    pa.push(PixMut::new(8, 8, PixelDepth::Bit8).unwrap().into());
    pa.push(PixMut::new(16, 12, PixelDepth::Bit8).unwrap().into());
    pa
}

#[test]

fn pixa_select_to_pdf_writes_bytes() {
    let pa = pixa_2_pages();
    let mut buf = Vec::new();
    pa.select_to_pdf(0, None, &PdfOptions::default(), &mut buf)
        .unwrap();
    // Only assert on the PDF header; sizes vary with compression / metadata
    // tweaks and a numeric lower-bound would just become brittle.
    assert!(buf.starts_with(b"%PDF-"));
}

#[test]

fn pixa_select_to_pdf_partial_range() {
    let pa = pixa_2_pages();
    let mut buf = Vec::new();
    pa.select_to_pdf(1, Some(1), &PdfOptions::default(), &mut buf)
        .unwrap();
    assert!(buf.starts_with(b"%PDF-"));
}

#[test]

fn pixa_select_to_pdf_empty_range_errors() {
    let pa = pixa_2_pages();
    let mut buf = Vec::new();
    // first > end of pixa → empty range → PDF generation should reject
    assert!(
        pa.select_to_pdf(5, None, &PdfOptions::default(), &mut buf)
            .is_err()
    );
}
