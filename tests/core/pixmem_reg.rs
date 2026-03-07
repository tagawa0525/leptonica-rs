//! Pix memory regression test
//!
//! Covers SPIX in-memory and file round-trips.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pixmem_reg.c`

use crate::common::RegParams;
use leptonica::{Pix, PixelDepth};

#[test]
fn pixmem_reg() {
    let mut rp = RegParams::new("pixmem");

    let pix = Pix::new(32, 20, PixelDepth::Bit8).expect("create pix");
    let mut pm = pix.to_mut();
    for y in 0..20u32 {
        for x in 0..32u32 {
            pm.set_pixel_unchecked(x, y, (x + 3 * y) % 256);
        }
    }
    let pix: Pix = pm.into();

    let data = pix.write_spix_to_bytes().expect("write_spix_to_bytes");
    let header = Pix::read_spix_header(&data).expect("read_spix_header");
    rp.compare_values(32.0, header.width as f64, 0.0);
    rp.compare_values(20.0, header.height as f64, 0.0);
    rp.compare_values(8.0, header.depth as f64, 0.0);

    let pix2 = Pix::read_spix_from_bytes(&data).expect("read_spix_from_bytes");
    rp.compare_pix(&pix, &pix2);

    let dir = std::env::temp_dir().join("leptonica_pixmem_reg");
    std::fs::create_dir_all(&dir).expect("create dir");
    let path = dir.join("pixmem.spix");
    pix.write_spix_to_file(&path).expect("write_spix_to_file");
    let pix3 = Pix::read_spix_from_file(&path).expect("read_spix_from_file");
    rp.compare_pix(&pix, &pix3);
    std::fs::remove_dir_all(&dir).ok();

    // C checks 0-1 equivalent: deep_clone preserves pixel data (Rust's pixCopy)
    let pix_dc = pix.deep_clone();
    rp.compare_pix(&pix, &pix_dc);

    // C checks 2-3 equivalent: to_mut roundtrip preserves pixel data
    let pix_mut = pix.to_mut();
    let pix_rt: Pix = pix_mut.into();
    rp.compare_pix(&pix, &pix_rt);

    // Additional depth: SPIX roundtrip for 1bpp
    let pix1 = Pix::new(16, 12, PixelDepth::Bit1).expect("create 1bpp pix");
    let data1 = pix1.write_spix_to_bytes().expect("write 1bpp spix");
    let pix1_rt = Pix::read_spix_from_bytes(&data1).expect("read 1bpp spix");
    rp.compare_pix(&pix1, &pix1_rt);

    // Additional depth: SPIX roundtrip for 32bpp
    let pix32 = Pix::new(10, 10, PixelDepth::Bit32).expect("create 32bpp pix");
    let data32 = pix32.write_spix_to_bytes().expect("write 32bpp spix");
    let pix32_rt = Pix::read_spix_from_bytes(&data32).expect("read 32bpp spix");
    rp.compare_pix(&pix32, &pix32_rt);

    assert!(rp.cleanup(), "pixmem regression test failed");
}
