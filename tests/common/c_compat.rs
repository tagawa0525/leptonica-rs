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
use std::io::Write;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CCompatMode {
    Off,
    Report,
    Strict,
}

impl CCompatMode {
    pub fn from_env() -> Self {
        match std::env::var("REGTEST_C_COMPAT")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "0" | "off" | "no" | "false" => Self::Off,
            "strict" | "fail" => Self::Strict,
            _ => Self::Report,
        }
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
/// The prefix may itself contain dots (`"rotate1_amcorner.02.tif"` → prefix
/// `"rotate1_amcorner"`); the index/ext are the last two dot-separated segments.
/// Returns `None` for non-conforming inputs (no two dots, non-numeric index).
pub fn parse_manifest_key(key: &str) -> Option<(String, usize, String)> {
    let last_dot = key.rfind('.')?;
    let ext = &key[last_dot + 1..];
    let before_ext = &key[..last_dot];
    let prev_dot = before_ext.rfind('.')?;
    let index_str = &before_ext[prev_dot + 1..];
    let prefix = &before_ext[..prev_dot];
    if prefix.is_empty() || ext.is_empty() {
        return None;
    }
    let index: usize = index_str.parse().ok()?;
    Some((prefix.to_string(), index, ext.to_string()))
}

/// Parse the TSV content of `scripts/golden_map.tsv` into a
/// `(rust_prefix, rust_index) → CKey { c_prefix, c_index }` map.
///
/// Lines starting with `#` or empty lines are skipped. Lines with fewer than
/// 5 fields, or with non-numeric indices, are silently ignored.
pub fn parse_golden_map(content: &str) -> GoldenMap {
    let mut map = GoldenMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 5 {
            continue;
        }
        // fields: module, c_prefix, c_index, rust_prefix, rust_index, [description]
        let c_prefix = fields[1].trim();
        let Ok(c_index) = fields[2].trim().parse::<usize>() else {
            continue;
        };
        let rust_prefix = fields[3].trim();
        let Ok(rust_index) = fields[4].trim().parse::<usize>() else {
            continue;
        };
        map.insert(
            (rust_prefix.to_string(), rust_index),
            CKey {
                prefix: c_prefix.to_string(),
                index: c_index,
            },
        );
    }
    map
}

/// Parse the TSV content of `tests/golden_manifest_c.tsv` into a
/// `filename → hash` map. Header (`#`) and malformed lines are skipped.
pub fn parse_c_manifest(content: &str) -> ManifestMap {
    let mut map = ManifestMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, hash_str)) = line.split_once('\t') else {
            continue;
        };
        let Ok(hash) = u64::from_str_radix(hash_str.trim(), 16) else {
            continue;
        };
        map.insert(name.to_string(), hash);
    }
    map
}

/// Extensions tried when resolving a C key to a manifest entry. The C version
/// writes the same operation to different formats depending on bit-depth, so
/// `lookup_c_hash_in` searches a fixed list of candidate extensions and returns
/// the first hit. PDF / PS / data-stream files are included because some C
/// regression outputs are raw byte streams (`.ba`, `.na`, `.pdf`).
const CANDIDATE_C_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "tif", "tiff", "bmp", "gif", "webp", "jp2", "j2k", "spix", "pnm", "pbm",
    "pgm", "ppm", "pam", "pdf", "ps", "ba", "na", "pta",
];

/// Look up the C-side hash that should correspond to the given Rust key.
/// Returns the matched C filename and its hash, or `None` if either the
/// golden_map mapping or the C manifest entry is absent.
pub fn lookup_c_hash_in(
    rust_key: &str,
    golden_map: &GoldenMap,
    c_manifest: &ManifestMap,
) -> Option<(String, u64)> {
    let (rust_prefix, rust_index, _rust_ext) = parse_manifest_key(rust_key)?;
    let c_key = golden_map.get(&(rust_prefix, rust_index))?;
    for ext in CANDIDATE_C_EXTENSIONS {
        let candidate = format!("{}.{:02}.{}", c_key.prefix, c_key.index, ext);
        if let Some(&hash) = c_manifest.get(&candidate) {
            return Some((candidate, hash));
        }
    }
    None
}

/// Classify a Rust key against the C baseline. Returns the status plus a
/// human-readable detail line suitable for the report file.
pub fn check_c_hash_against(
    rust_key: &str,
    rust_hash: u64,
    golden_map: &GoldenMap,
    c_manifest: &ManifestMap,
) -> (CCompatStatus, String) {
    let Some((rust_prefix, rust_index, _ext)) = parse_manifest_key(rust_key) else {
        return (
            CCompatStatus::Unmapped,
            format!("could not parse key '{rust_key}'"),
        );
    };
    let Some(c_key) = golden_map.get(&(rust_prefix, rust_index)) else {
        return (
            CCompatStatus::Unmapped,
            "no golden_map.tsv entry".to_string(),
        );
    };
    let mut found: Option<(String, u64)> = None;
    for ext in CANDIDATE_C_EXTENSIONS {
        let candidate = format!("{}.{:02}.{}", c_key.prefix, c_key.index, ext);
        if let Some(&h) = c_manifest.get(&candidate) {
            found = Some((candidate, h));
            break;
        }
    }
    let Some((c_name, c_hash)) = found else {
        return (
            CCompatStatus::MissingC,
            format!(
                "golden_map → {}/{:02} but no C manifest entry",
                c_key.prefix, c_key.index
            ),
        );
    };
    if c_hash == rust_hash {
        (
            CCompatStatus::Ok,
            format!("matches {c_name} = {c_hash:016x}"),
        )
    } else {
        (
            CCompatStatus::Mismatch,
            format!("rust={rust_hash:016x}, c[{c_name}]={c_hash:016x}"),
        )
    }
}

// --- Stateful integration (loaded once per test process) ---

const C_MANIFEST_PATH_REL: &str = "tests/golden_manifest_c.tsv";
const GOLDEN_MAP_PATH_REL: &str = "scripts/golden_map.tsv";
const REPORT_PATH_REL: &str = "tests/c_compat_report.txt";

fn workspace_root() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

fn c_manifest() -> &'static ManifestMap {
    static CACHE: OnceLock<ManifestMap> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = format!("{}/{}", workspace_root(), C_MANIFEST_PATH_REL);
        std::fs::read_to_string(&path)
            .map(|s| parse_c_manifest(&s))
            .unwrap_or_default()
    })
}

fn golden_map() -> &'static GoldenMap {
    static CACHE: OnceLock<GoldenMap> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = format!("{}/{}", workspace_root(), GOLDEN_MAP_PATH_REL);
        std::fs::read_to_string(&path)
            .map(|s| parse_golden_map(&s))
            .unwrap_or_default()
    })
}

fn report_file() -> Option<&'static Mutex<std::fs::File>> {
    // OnceLock<Option<Mutex<File>>>: we may fail to open the report file (e.g.
    // when the workspace is read-only); in that case we silently degrade to
    // no-op rather than failing the test run.
    static FILE: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    FILE.get_or_init(|| {
        let path = format!("{}/{}", workspace_root(), REPORT_PATH_REL);
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .ok()?;
        let _ = writeln!(
            f,
            "# C-compat report — generated by tests/common/c_compat.rs (plan 901 Phase 2)"
        );
        let _ = writeln!(f, "# Format: [STATUS] test_name :: rust_key :: detail");
        let _ = writeln!(f, "# Statuses: Ok / Mismatch / Unmapped / MissingC");
        let _ = writeln!(f);
        Some(Mutex::new(f))
    })
    .as_ref()
}

fn append_report(line: &str) {
    if let Some(file) = report_file()
        && let Ok(mut f) = file.lock()
    {
        let _ = writeln!(f, "{line}");
    }
}

/// Compare a Rust output hash against the C baseline (if any) and append the
/// result to `tests/c_compat_report.txt`. Returns the classification status.
///
/// In `CCompatMode::Off` the comparison is skipped entirely and `Ok` is
/// returned without writing to the report. In `CCompatMode::Strict` callers
/// are expected to treat `Mismatch` as a test failure.
pub fn check_c_hash(test_name: &str, rust_key: &str, rust_hash: u64) -> CCompatStatus {
    if CCompatMode::from_env() == CCompatMode::Off {
        return CCompatStatus::Ok;
    }
    let (status, detail) = check_c_hash_against(rust_key, rust_hash, golden_map(), c_manifest());
    append_report(&format!(
        "[{status:?}] {test_name} :: {rust_key} :: {detail}"
    ));
    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn parse_key_accepts_two_digit_index_and_extension() {
        let (p, i, e) = parse_manifest_key("edge.04.jpg").unwrap();
        assert_eq!(p, "edge");
        assert_eq!(i, 4);
        assert_eq!(e, "jpg");
    }

    #[test]

    fn parse_key_accepts_prefix_with_dot() {
        // Some Rust manifest keys contain extra dots in the prefix (e.g. "1bpp-bw1.png" — no idx).
        // Confirm normal three-segment keys still parse with multi-segment prefixes.
        let (p, i, e) = parse_manifest_key("rotate1_amcorner.02.tif").unwrap();
        assert_eq!(p, "rotate1_amcorner");
        assert_eq!(i, 2);
        assert_eq!(e, "tif");
    }

    #[test]

    fn parse_key_rejects_missing_index_or_ext() {
        assert!(parse_manifest_key("edge").is_none());
        assert!(parse_manifest_key("edge.png").is_none());
        assert!(parse_manifest_key("edge.xx.png").is_none()); // non-numeric index
    }

    #[test]

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
