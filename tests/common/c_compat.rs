//! C-version hash compatibility check (plan 901 Phase 2).
//!
//! Runs alongside the existing Rust-vs-Rust hash compare in `RegParams::check_hash`
//! to report whether a Rust regression output matches the C version's golden
//! output (`tests/golden_manifest_c.tsv`, produced by `examples/gen_c_manifest`).
//!
//! Differences are written to `tests/c_compat_report.txt` and do **not** fail
//! the test by default. Set `REGTEST_C_COMPAT=strict` to escalate mismatches
//! to test failures, or `REGTEST_C_COMPAT=off` to disable entirely.

#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CCompatMode {
    Off,
    Report,
    Strict,
}

impl CCompatMode {
    pub fn from_env() -> Self {
        unimplemented!("CCompatMode::from_env")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKey {
    pub prefix: String,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CCompatStatus {
    /// Rust hash matches the corresponding C hash.
    Ok,
    /// Rust hash differs from the C hash (move target for Phase 2.5 fixes).
    Mismatch,
    /// `golden_map.tsv` has no entry for this Rust key (`<rust_prefix>.<rust_index>`).
    Unmapped,
    /// `golden_map.tsv` maps to a C key but no matching entry exists in `golden_manifest_c.tsv`.
    MissingC,
}

pub type GoldenMap = HashMap<(String, usize), CKey>;
pub type ManifestMap = HashMap<String, u64>;

/// Parse a manifest key like `"edge.04.jpg"` into `(prefix, index, ext)`.
/// Returns `None` for non-conforming inputs.
pub fn parse_manifest_key(_key: &str) -> Option<(String, usize, String)> {
    unimplemented!("parse_manifest_key")
}

/// Parse the TSV content of `scripts/golden_map.tsv` into a
/// `(rust_prefix, rust_index) → CKey { c_prefix, c_index }` map.
pub fn parse_golden_map(_content: &str) -> GoldenMap {
    unimplemented!("parse_golden_map")
}

/// Parse the TSV content of `tests/golden_manifest_c.tsv` into a
/// `filename → hash` map.
pub fn parse_c_manifest(_content: &str) -> ManifestMap {
    unimplemented!("parse_c_manifest")
}

/// Look up the C-side hash that should correspond to the given Rust key.
/// Returns the matched C filename and its hash, or `None` if either the
/// golden_map mapping or the C manifest entry is absent.
pub fn lookup_c_hash_in(
    _rust_key: &str,
    _golden_map: &GoldenMap,
    _c_manifest: &ManifestMap,
) -> Option<(String, u64)> {
    unimplemented!("lookup_c_hash_in")
}

/// Classify a Rust key against the C baseline. Returns the status plus a
/// human-readable detail line suitable for the report file.
pub fn check_c_hash_against(
    _rust_key: &str,
    _rust_hash: u64,
    _golden_map: &GoldenMap,
    _c_manifest: &ManifestMap,
) -> (CCompatStatus, String) {
    unimplemented!("check_c_hash_against")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_key_accepts_two_digit_index_and_extension() {
        let (p, i, e) = parse_manifest_key("edge.04.jpg").unwrap();
        assert_eq!(p, "edge");
        assert_eq!(i, 4);
        assert_eq!(e, "jpg");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_key_accepts_prefix_with_dot() {
        // Some Rust manifest keys contain extra dots in the prefix (e.g. "1bpp-bw1.png" — no idx).
        // Confirm normal three-segment keys still parse with multi-segment prefixes.
        let (p, i, e) = parse_manifest_key("rotate1_amcorner.02.tif").unwrap();
        assert_eq!(p, "rotate1_amcorner");
        assert_eq!(i, 2);
        assert_eq!(e, "tif");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_key_rejects_missing_index_or_ext() {
        assert!(parse_manifest_key("edge").is_none());
        assert!(parse_manifest_key("edge.png").is_none());
        assert!(parse_manifest_key("edge.xx.png").is_none()); // non-numeric index
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_golden_map_extracts_rust_to_c_mapping() {
        let content = "\
# comment
edge\tedge\t3\tedge\t4\tdescription
convolve\tconvolve\t0\tconvolve_blockconv_gray\t1\tdesc";
        let m = parse_golden_map(content);
        assert_eq!(
            m.get(&("edge".to_string(), 4)),
            Some(&CKey {
                prefix: "edge".to_string(),
                index: 3
            })
        );
        assert_eq!(
            m.get(&("convolve_blockconv_gray".to_string(), 1)),
            Some(&CKey {
                prefix: "convolve".to_string(),
                index: 0
            })
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_golden_map_ignores_comments_blanks_and_malformed_lines() {
        let content = "\
# header

not\tenough\tfields
edge\tedge\t3\tedge\t4\tok
edge\tedge\tNaN\tedge\t9\tinvalid_index";
        let m = parse_golden_map(content);
        assert_eq!(m.len(), 1, "only the well-formed line should be parsed");
        assert!(m.contains_key(&("edge".to_string(), 4)));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_c_manifest_extracts_filename_to_hash() {
        let content = "\
# header
edge.03.jpg\tfcfea0e83ecec76c
broken
zzz.04.png\tnot_hex_value
ok.01.png\tdeadbeefdeadbeef";
        let m = parse_c_manifest(content);
        assert_eq!(m.get("edge.03.jpg"), Some(&0xfcfea0e83ecec76c));
        assert_eq!(m.get("ok.01.png"), Some(&0xdeadbeefdeadbeef));
        assert!(!m.contains_key("zzz.04.png"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn lookup_resolves_through_golden_map_and_tries_multiple_extensions() {
        let mut gmap: GoldenMap = HashMap::new();
        gmap.insert(
            ("edge".to_string(), 4),
            CKey {
                prefix: "edge".to_string(),
                index: 3,
            },
        );
        let mut cm: ManifestMap = HashMap::new();
        cm.insert("edge.03.jpg".to_string(), 0xfcfea0e83ecec76c);

        let (k, h) = lookup_c_hash_in("edge.04.jpg", &gmap, &cm).unwrap();
        assert_eq!(k, "edge.03.jpg");
        assert_eq!(h, 0xfcfea0e83ecec76c);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn lookup_returns_none_when_unmapped_or_missing_c_entry() {
        let mut gmap: GoldenMap = HashMap::new();
        gmap.insert(
            ("edge".to_string(), 4),
            CKey {
                prefix: "edge".to_string(),
                index: 3,
            },
        );
        let cm: ManifestMap = HashMap::new();

        // Unmapped rust key.
        assert!(lookup_c_hash_in("other.01.png", &gmap, &cm).is_none());
        // Mapped but no C manifest entry.
        assert!(lookup_c_hash_in("edge.04.jpg", &gmap, &cm).is_none());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn check_against_classifies_ok_mismatch_unmapped_missing() {
        let mut gmap: GoldenMap = HashMap::new();
        gmap.insert(
            ("edge".to_string(), 4),
            CKey {
                prefix: "edge".to_string(),
                index: 3,
            },
        );
        let mut cm: ManifestMap = HashMap::new();
        cm.insert("edge.03.jpg".to_string(), 0xfcfea0e83ecec76c);

        let (s, _) = check_c_hash_against("edge.04.jpg", 0xfcfea0e83ecec76c, &gmap, &cm);
        assert_eq!(s, CCompatStatus::Ok);

        let (s, _) = check_c_hash_against("edge.04.jpg", 0xdead_beef, &gmap, &cm);
        assert_eq!(s, CCompatStatus::Mismatch);

        let (s, _) = check_c_hash_against("foo.01.png", 0, &gmap, &cm);
        assert_eq!(s, CCompatStatus::Unmapped);

        let (s, _) = check_c_hash_against("edge.04.jpg", 0, &gmap, &ManifestMap::new());
        assert_eq!(s, CCompatStatus::MissingC);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn ccompat_mode_from_env_recognises_off_report_strict() {
        // SAFETY: env mutation is racy across threads; this single test is enough
        // to lock down the parsing logic without exercising globals in parallel.
        // Other tests must avoid setting REGTEST_C_COMPAT.
        unsafe {
            std::env::set_var("REGTEST_C_COMPAT", "off");
            assert_eq!(CCompatMode::from_env(), CCompatMode::Off);
            std::env::set_var("REGTEST_C_COMPAT", "strict");
            assert_eq!(CCompatMode::from_env(), CCompatMode::Strict);
            std::env::set_var("REGTEST_C_COMPAT", "report");
            assert_eq!(CCompatMode::from_env(), CCompatMode::Report);
            std::env::remove_var("REGTEST_C_COMPAT");
            assert_eq!(CCompatMode::from_env(), CCompatMode::Report);
        }
    }
}
