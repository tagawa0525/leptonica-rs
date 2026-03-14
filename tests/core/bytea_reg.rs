//! Byte-array regression test
//!
//! Covers Base64 round-trip and compressed Pix text storage.
//!
//! # See also
//!
//! C Leptonica: `prog/bytea_reg.c`

use crate::common::RegParams;
use leptonica::{Pix, PixMut, PixelDepth, decode_base64, encode_base64};

#[test]
fn bytea_reg() {
    let mut rp = RegParams::new("bytea");

    let payload = b"leptonica-bytea-roundtrip-1234567890";
    let encoded = encode_base64(payload);
    let decoded = decode_base64(&encoded).expect("decode_base64");
    rp.compare_strings(payload, &decoded);

    let pix = Pix::new(12, 8, PixelDepth::Bit8).expect("create pix");
    let mut pm = pix.to_mut();
    PixMut::set_text_comp_new(&mut pm, payload).expect("set_text_comp_new");
    let pix: Pix = pm.into();

    let restored = pix.get_text_comp_new().expect("get_text_comp_new");
    rp.compare_strings(payload, &restored);

    assert!(rp.cleanup(), "bytea regression test failed");
}
