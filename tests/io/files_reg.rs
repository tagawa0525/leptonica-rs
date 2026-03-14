//! File-collection I/O regression test
//!
//! Covers `pixa_read_files` and `pixa_write_files`.
//!
//! # See also
//!
//! C Leptonica: `prog/files_reg.c`

use crate::common::RegParams;
use leptonica::io::{ImageFormat, pixa_read_files, pixa_write_files, read_image, write_image};
use leptonica::{Pix, PixelDepth};

#[test]
fn files_reg() {
    let mut rp = RegParams::new("files");

    let base = std::env::temp_dir().join("leptonica_files_reg");
    let in_dir = base.join("in");
    let out_dir = base.join("out");
    std::fs::create_dir_all(&in_dir).expect("create input dir");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let pix = Pix::new(24, 18, PixelDepth::Bit8).expect("create pix");
    write_image(&pix, in_dir.join("a.png"), ImageFormat::Png).expect("write a.png");
    write_image(&pix, in_dir.join("b.png"), ImageFormat::Png).expect("write b.png");

    let pixa = pixa_read_files(&in_dir, Some(".png")).expect("pixa_read_files");
    rp.compare_values(2.0, pixa.len() as f64, 0.0);

    let root = out_dir.join("frame").to_string_lossy().to_string();
    pixa_write_files(&root, &pixa, ImageFormat::Png).expect("pixa_write_files");

    let out0 = out_dir.join("frame000.png");
    let out1 = out_dir.join("frame001.png");
    rp.compare_values(
        1.0,
        if out0.exists() && out1.exists() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let pix0 = read_image(&out0).expect("read frame000");
    rp.compare_values(24.0, pix0.width() as f64, 0.0);
    rp.compare_values(18.0, pix0.height() as f64, 0.0);

    // Verify round-trip: re-read as pixa and check
    let pixa_rt = pixa_read_files(&out_dir, Some(".png")).expect("pixa_read_files roundtrip");
    rp.compare_values(2.0, pixa_rt.len() as f64, 0.0);

    // Verify each round-tripped image preserves dimensions
    for i in 0..pixa_rt.len() {
        let p = pixa_rt.get(i).expect("get roundtrip pix");
        rp.compare_values(24.0, p.width() as f64, 0.0);
        rp.compare_values(18.0, p.height() as f64, 0.0);
    }

    std::fs::remove_dir_all(&base).ok();
    assert!(rp.cleanup(), "files regression test failed");
}
