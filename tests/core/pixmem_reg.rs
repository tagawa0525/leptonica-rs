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

    assert!(rp.cleanup(), "pixmem regression test failed");
}
