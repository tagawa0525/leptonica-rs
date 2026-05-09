//! String regression test
//!
//! Covers `Sarray`/`Sarraya` operations and serialization.
//!
//! # See also
//!
//! C Leptonica: `prog/string_reg.c`

use crate::common::RegParams;
use leptonica::{Sarray, Sarraya};

// ============================================================================
// C-equivalent regression test skeletons — missing string operations
// ============================================================================

/// Sarray substring search (C: sarrayFindStringByHash).
#[test]
#[ignore = "sarrayFindStringByHash not implemented"]
fn string_reg_find_by_hash() {}

/// Sarray replace substring (C: sarrayReplaceString).
#[test]
#[ignore = "string search/replace via hash not implemented"]
fn string_reg_replace_string() {}

/// Binary sequence operations (C: arrayFindSequence).
#[test]
fn string_reg_binary_sequence() {
    use leptonica::core::sarray::array_find_sequence;

    let data: &[u8] = b"hello world";
    assert_eq!(array_find_sequence(data, b"hello"), Some(0));
    assert_eq!(array_find_sequence(data, b"world"), Some(6));
    assert_eq!(array_find_sequence(data, b"o w"), Some(4));
    assert_eq!(array_find_sequence(data, b"xyz"), None);
    // Sequence longer than data → None.
    assert_eq!(array_find_sequence(b"hi", b"hello"), None);
    // Empty sequence is treated as found at offset 0.
    assert_eq!(array_find_sequence(data, b""), Some(0));
    // Empty data with non-empty sequence → None.
    assert_eq!(array_find_sequence(b"", b"x"), None);
    // Embedded null bytes are honored as part of the byte stream.
    let nul: &[u8] = b"a\0b\0c";
    assert_eq!(array_find_sequence(nul, b"\0b"), Some(1));
    assert_eq!(array_find_sequence(nul, b"\0c"), Some(3));
}

/// Sarray file/stream I/O round-trip (C: sarrayWrite/sarrayRead).
///
/// `Sarray` already exposes `read_from_reader` / `write_to_writer` /
/// `write_to_bytes` etc.; this regression test verifies the round-trip
/// covers empty arrays, single strings, multi-string content and strings
/// that contain spaces.
#[test]
fn string_reg_file_io() {
    fn roundtrip(input: Vec<&str>) {
        let sa = Sarray::from_str_slice(&input);
        // write_to_writer + read_from_reader
        let mut buf: Vec<u8> = Vec::new();
        sa.write_to_writer(&mut buf).expect("write_to_writer");
        let restored = Sarray::read_from_reader(&mut buf.as_slice()).expect("read_from_reader");
        assert_eq!(restored.len(), sa.len());
        for i in 0..sa.len() {
            assert_eq!(restored.get(i), sa.get(i), "string {i} after stream rt");
        }

        // write_to_bytes + read_from_bytes
        let bytes = sa.write_to_bytes().expect("write_to_bytes");
        let restored2 = Sarray::read_from_bytes(&bytes).expect("read_from_bytes");
        assert_eq!(restored2.len(), sa.len());
        for i in 0..sa.len() {
            assert_eq!(restored2.get(i), sa.get(i), "string {i} after mem rt");
        }
    }

    roundtrip(vec![]);
    roundtrip(vec!["alone"]);
    roundtrip(vec!["alpha", "beta", "gamma"]);
    roundtrip(vec!["hello world", "containing  multiple  spaces"]);
}

#[test]
fn string_reg() {
    let mut rp = RegParams::new("string");

    let mut sa = Sarray::from_words("alpha beta gamma");
    sa.insert(1, "inserted").expect("insert");
    rp.compare_values(4.0, sa.len() as f64, 0.0);
    rp.compare_values(
        1.0,
        if sa.get(1) == Some("inserted") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let bytes = sa.write_to_bytes().expect("write_to_bytes");
    let sa2 = Sarray::read_from_bytes(&bytes).expect("read_from_bytes");
    rp.compare_values(sa.len() as f64, sa2.len() as f64, 0.0);
    rp.compare_values(
        1.0,
        if sa2.join(" ") == "alpha inserted beta gamma" {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let mut saa = Sarraya::new();
    saa.push(sa2.clone());
    saa.push(Sarray::from_str_slice(&["delta", "epsilon"]));
    saa.add_string(1, "zeta").expect("add_string");

    let flat = saa.flatten();
    rp.compare_values(7.0, flat.len() as f64, 0.0);
    rp.compare_values(
        1.0,
        if flat.get(6) == Some("zeta") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Additional string operations
    // Remove operation
    sa.remove(1).expect("remove");
    rp.compare_values(3.0, sa.len() as f64, 0.0);
    rp.compare_values(1.0, if sa.get(0) == Some("alpha") { 1.0 } else { 0.0 }, 0.0);

    // Sort
    let sorted = sa.sorted();
    rp.compare_values(3.0, sorted.len() as f64, 0.0);
    rp.compare_values(
        1.0,
        if sorted.get(0) == Some("alpha") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Words from lines
    let multi = Sarray::from_words("one two three\nfour five");
    rp.compare_values(5.0, multi.len() as f64, 0.0);

    // Sarraya operations
    rp.compare_values(2.0, saa.len() as f64, 0.0);
    let flat2 = saa.flatten();
    rp.compare_values(7.0, flat2.len() as f64, 0.0);

    assert!(rp.cleanup(), "string regression test failed");
}
