//! Regression tests for 1bpp TIFF write/read round-trip.
//!
//! plan 901 Phase 2.5 第二弾の対象 (docs/porting/c-compat-findings/
//! 002-tiff-1bpp-write-limit.md)。`src/io/tiff.rs::write_pix_to_encoder` の
//! `PixelDepth::Bit1` ブランチが「convert to 8-bit for simplicity」と
//! 8bpp に拡張して書き出していたため、binmorph1/3/fhmtauto の C 互換性
//! チェックで 15 件の Mismatch が発生していた。本ファイルは 1bpp 出力が
//! 1bpp として保たれることをロックダウンする。

use leptonica::core::{Pix, PixelDepth};
use leptonica::io::{ImageFormat, read_image, write_image};

fn make_1bpp_checkerboard(width: u32, height: u32) -> Pix {
    let pix = Pix::new(width, height, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 0..height {
        for x in 0..width {
            let v = if (x + y) % 2 == 0 { 1 } else { 0 };
            pix_mut.set_pixel_unchecked(x, y, v);
        }
    }
    pix_mut.into()
}

fn tmp_path(stem: &str) -> String {
    format!("/tmp/leptonica_rs_test_tiff_1bpp_{}.tif", stem)
}

#[test]
fn round_trip_preserves_bit1_depth_byte_aligned_width() {
    // 16 は 8 の倍数なので末尾 padding なし
    let pix = make_1bpp_checkerboard(16, 8);
    let path = tmp_path("byte_aligned");
    write_image(&pix, &path, ImageFormat::Tiff).expect("write tiff");
    let pix2 = read_image(&path).expect("read tiff");
    let _ = std::fs::remove_file(&path);

    assert_eq!(
        pix2.depth(),
        PixelDepth::Bit1,
        "1bpp Pix を書き出して読み戻したら depth が Bit1 のままである必要がある"
    );
    assert_eq!(pix2.width(), pix.width());
    assert_eq!(pix2.height(), pix.height());

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            assert_eq!(
                pix.get_pixel(x, y).unwrap_or(0),
                pix2.get_pixel(x, y).unwrap_or(0),
                "pixel mismatch at ({x}, {y})"
            );
        }
    }
}

#[test]
fn round_trip_preserves_bit1_depth_non_byte_aligned_width() {
    // 17 は 8 の倍数ではないので末尾 padding bits がある (各行末で 7 bits 余る)
    // これらの padding bits が pixel data として読み戻されないことを確認
    let pix = make_1bpp_checkerboard(17, 5);
    let path = tmp_path("non_byte_aligned");
    write_image(&pix, &path, ImageFormat::Tiff).expect("write tiff");
    let pix2 = read_image(&path).expect("read tiff");
    let _ = std::fs::remove_file(&path);

    assert_eq!(pix2.depth(), PixelDepth::Bit1);
    assert_eq!(pix2.width(), 17);
    assert_eq!(pix2.height(), 5);

    for y in 0..5 {
        for x in 0..17 {
            assert_eq!(
                pix.get_pixel(x, y).unwrap_or(0),
                pix2.get_pixel(x, y).unwrap_or(0),
                "pixel mismatch at ({x}, {y})"
            );
        }
    }
}

#[test]
fn round_trip_preserves_all_white_and_all_black_1bpp() {
    for (label, fill) in [("white", 0u32), ("black", 1u32)] {
        let pix = Pix::new(24, 6, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..6 {
            for x in 0..24 {
                pm.set_pixel_unchecked(x, y, fill);
            }
        }
        let pix: Pix = pm.into();

        let path = tmp_path(label);
        write_image(&pix, &path, ImageFormat::Tiff).expect("write tiff");
        let pix2 = read_image(&path).expect("read tiff");
        let _ = std::fs::remove_file(&path);

        assert_eq!(pix2.depth(), PixelDepth::Bit1, "{label}: depth Bit1");
        for y in 0..6 {
            for x in 0..24 {
                assert_eq!(
                    pix2.get_pixel(x, y).unwrap_or(0),
                    fill,
                    "{label}: pixel ({x},{y}) should be {fill}"
                );
            }
        }
    }
}
