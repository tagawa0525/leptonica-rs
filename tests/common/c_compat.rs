//! C-version hash compatibility check (plan 901 Phase 2).
//!
//! Runs alongside the existing Rust-vs-Rust hash compare in `RegParams::check_hash`
//! to report whether a Rust regression output matches the C version's golden
//! output (`tests/golden_manifest_c.tsv`, produced by `examples/gen_c_manifest`).
//!
//! Every comparison (including `Ok` matches) is recorded to
//! `tests/c_compat_report.<binary>.txt` (one file per cargo-test integration
//! binary, e.g. `core` / `io` / `filter`) so the report doubles as a baseline
//! of which Rust outputs match the C reference. Lines do **not** fail the
//! test by default. Set `REGTEST_C_COMPAT=strict` to escalate mismatches to
//! test failures, or `REGTEST_C_COMPAT=off` to disable the check entirely.
//!
//! Keys with no `golden_map.tsv` entry are classified `Unmapped`, unless an
//! exclusion rule in `scripts/c_compat_exclude.tsv` marks them un-mappable
//! by design (JPEG codec differences, non-deterministic formats) — those are
//! reported as `Excluded` so `Unmapped` counts only actionable work
//! (plan 902).

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
        Self::from_value(&std::env::var("REGTEST_C_COMPAT").unwrap_or_default())
    }

    /// Pure decoder used by `from_env`. Split out so unit tests can exercise
    /// the parsing rules without mutating process-wide environment state
    /// (`std::env::set_var` is `unsafe` because other threads may be
    /// reading any env var concurrently).
    pub fn from_value(s: &str) -> Self {
        match s.to_lowercase().as_str() {
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
    /// Not mapped, and matched by an exclusion rule in `scripts/c_compat_exclude.tsv`
    /// (e.g. JPEG codec differences, non-deterministic formats). Never a failure.
    Excluded,
}

/// What an exclusion rule matches against (plan 902).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExcludeKind {
    /// Matches the file extension of the Rust manifest key (`edge.04.jpg` → `jpg`).
    Ext,
    /// Matches the parsed prefix of the Rust manifest key (`edge.04.jpg` → `edge`).
    Prefix,
}

/// One parsed line of `scripts/c_compat_exclude.tsv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcludeRule {
    pub kind: ExcludeKind,
    pub value: String,
    pub reason: String,
}

/// Parse the TSV content of `scripts/c_compat_exclude.tsv` into exclusion
/// rules. Fields: `kind` (`ext` | `prefix`), `value`, `reason`. Lines starting
/// with `#`, empty lines, and malformed lines (unknown kind, fewer than 3
/// fields) are skipped.
pub fn parse_exclude_rules(content: &str) -> Vec<ExcludeRule> {
    let mut rules = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }
        let kind = match fields[0].trim() {
            "ext" => ExcludeKind::Ext,
            "prefix" => ExcludeKind::Prefix,
            _ => continue,
        };
        rules.push(ExcludeRule {
            kind,
            value: fields[1].trim().to_string(),
            reason: fields[2].trim().to_string(),
        });
    }
    rules
}

/// Return the first exclusion rule matching the given Rust manifest key, or
/// `None`. Only consulted when the key has no `golden_map.tsv` entry — an
/// explicit mapping always wins over exclusion.
pub fn find_exclusion<'a>(rust_key: &str, rules: &'a [ExcludeRule]) -> Option<&'a ExcludeRule> {
    let (prefix, _index, ext) = parse_manifest_key(rust_key)?;
    rules.iter().find(|r| match r.kind {
        ExcludeKind::Ext => r.value == ext,
        ExcludeKind::Prefix => r.value == prefix,
    })
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
/// the lookup searches this fixed list and returns the first hit. PDF / PS /
/// data-stream files are included because some C regression outputs are raw
/// byte streams (`.ba`, `.na`, `.pdf`, `.pa` for Pta).
const CANDIDATE_C_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "tif", "tiff", "bmp", "gif", "webp", "jp2", "j2k", "spix", "pnm", "pbm",
    "pgm", "ppm", "pam", "pdf", "ps", "ba", "na", "pa",
];

/// Detailed lookup result used by both `lookup_c_hash_in` (Option facade) and
/// `check_c_hash_against` (status classifier). Returns the matched C name +
/// hash on success, or a `(status, detail)` pair describing why the lookup
/// failed (`Unmapped` for parse / golden_map miss, `MissingC` for no C entry).
fn lookup_c_hash_detailed(
    rust_key: &str,
    golden_map: &GoldenMap,
    c_manifest: &ManifestMap,
) -> Result<(String, u64), (CCompatStatus, String)> {
    let Some((rust_prefix, rust_index, _rust_ext)) = parse_manifest_key(rust_key) else {
        return Err((
            CCompatStatus::Unmapped,
            format!("could not parse key '{rust_key}'"),
        ));
    };
    let Some(c_key) = golden_map.get(&(rust_prefix, rust_index)) else {
        return Err((
            CCompatStatus::Unmapped,
            "no golden_map.tsv entry".to_string(),
        ));
    };
    for ext in CANDIDATE_C_EXTENSIONS {
        let candidate = format!("{}.{:02}.{}", c_key.prefix, c_key.index, ext);
        if let Some(&hash) = c_manifest.get(&candidate) {
            return Ok((candidate, hash));
        }
    }
    Err((
        CCompatStatus::MissingC,
        format!(
            "golden_map → {}/{:02} but no C manifest entry",
            c_key.prefix, c_key.index
        ),
    ))
}

/// Look up the C-side hash that should correspond to the given Rust key.
/// Returns the matched C filename and its hash, or `None` if either the
/// golden_map mapping or the C manifest entry is absent.
pub fn lookup_c_hash_in(
    rust_key: &str,
    golden_map: &GoldenMap,
    c_manifest: &ManifestMap,
) -> Option<(String, u64)> {
    lookup_c_hash_detailed(rust_key, golden_map, c_manifest).ok()
}

/// Classify a Rust key against the C baseline. Returns the status plus a
/// human-readable detail line suitable for the report file.
///
/// Exclusion rules are consulted only for keys with no `golden_map.tsv`
/// entry: an explicit mapping always wins, so existing Ok / Mismatch
/// baselines are unaffected by additions to the exclusion list.
pub fn check_c_hash_against(
    rust_key: &str,
    rust_hash: u64,
    golden_map: &GoldenMap,
    c_manifest: &ManifestMap,
    exclude: &[ExcludeRule],
) -> (CCompatStatus, String) {
    match lookup_c_hash_detailed(rust_key, golden_map, c_manifest) {
        Ok((c_name, c_hash)) => {
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
        Err((CCompatStatus::Unmapped, detail)) => match find_exclusion(rust_key, exclude) {
            Some(rule) => (CCompatStatus::Excluded, rule.reason.clone()),
            None => (CCompatStatus::Unmapped, detail),
        },
        Err(failure) => failure,
    }
}

// --- Stateful integration (loaded once per test process) ---

const C_MANIFEST_PATH_REL: &str = "tests/golden_manifest_c.tsv";
const GOLDEN_MAP_PATH_REL: &str = "scripts/golden_map.tsv";
const EXCLUDE_PATH_REL: &str = "scripts/c_compat_exclude.tsv";

fn workspace_root() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

/// `cargo test --all-features` spawns one process per integration-test
/// binary (`tests/core/main.rs`, `tests/io/main.rs`, …). A single shared
/// report file would be silently truncated by whichever process started
/// last, so we derive a per-binary report path from `std::env::current_exe`
/// (the test binary name minus its build hash suffix).
fn report_path() -> String {
    let stem = std::env::current_exe()
        .ok()
        .and_then(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .map(|name| {
            // cargo test binaries are named like "core-9f2b7cd034b6d92b";
            // strip the trailing hex hash (split on first '-' from the right
            // if the suffix looks like a hex blob).
            let parts: Vec<&str> = name.rsplitn(2, '-').collect();
            if parts.len() == 2
                && parts[0].len() == 16
                && parts[0].chars().all(|c| c.is_ascii_hexdigit())
            {
                parts[1].to_string()
            } else {
                name
            }
        })
        .unwrap_or_else(|| "unknown".to_string());
    format!("{}/tests/c_compat_report.{}.txt", workspace_root(), stem)
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

fn exclude_rules() -> &'static [ExcludeRule] {
    static CACHE: OnceLock<Vec<ExcludeRule>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = format!("{}/{}", workspace_root(), EXCLUDE_PATH_REL);
        std::fs::read_to_string(&path)
            .map(|s| parse_exclude_rules(&s))
            .unwrap_or_default()
    })
}

fn report_file() -> Option<&'static Mutex<std::fs::File>> {
    // OnceLock<Option<Mutex<File>>>: we may fail to open the report file (e.g.
    // when the workspace is read-only); in that case we silently degrade to
    // no-op rather than failing the test run.
    //
    // Each cargo-test binary owns its own report file (`report_path()`), so
    // truncating on first open is safe within a single binary and avoids
    // accumulating stale lines across re-runs of that binary.
    static FILE: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    FILE.get_or_init(|| {
        let path = report_path();
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
        let _ = writeln!(
            f,
            "# Statuses: Ok / Mismatch / Unmapped / MissingC / Excluded"
        );
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

/// Compare a Rust output hash against the C baseline (if any) and append
/// **every** result (including `Ok`) to `tests/c_compat_report.<binary>.txt`.
/// Logging all matches keeps the report usable as a baseline of which Rust
/// outputs are bit-for-bit identical with the C reference.
///
/// In `CCompatMode::Off` the comparison is skipped entirely and `Ok` is
/// returned without writing to the report. In `CCompatMode::Strict` callers
/// are expected to treat `Mismatch` as a test failure.
pub fn check_c_hash(test_name: &str, rust_key: &str, rust_hash: u64) -> CCompatStatus {
    if CCompatMode::from_env() == CCompatMode::Off {
        return CCompatStatus::Ok;
    }
    let (status, detail) = check_c_hash_against(
        rust_key,
        rust_hash,
        golden_map(),
        c_manifest(),
        exclude_rules(),
    );
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

        let (s, _) = check_c_hash_against("edge.04.jpg", 0xfcfea0e83ecec76c, &gmap, &cm, &[]);
        assert_eq!(s, CCompatStatus::Ok);

        let (s, _) = check_c_hash_against("edge.04.jpg", 0xdead_beef, &gmap, &cm, &[]);
        assert_eq!(s, CCompatStatus::Mismatch);

        let (s, _) = check_c_hash_against("foo.01.png", 0, &gmap, &cm, &[]);
        assert_eq!(s, CCompatStatus::Unmapped);

        let (s, _) = check_c_hash_against("edge.04.jpg", 0, &gmap, &ManifestMap::new(), &[]);
        assert_eq!(s, CCompatStatus::MissingC);
    }

    #[test]
    fn parse_exclude_rules_extracts_ext_and_prefix_rules() {
        let content = "\
# kind\tvalue\treason
ext\tjpg\tJPEG codec diff (finding 001)
prefix\twebpio\tno C counterpart";
        let rules = parse_exclude_rules(content);
        assert_eq!(
            rules,
            vec![
                ExcludeRule {
                    kind: ExcludeKind::Ext,
                    value: "jpg".to_string(),
                    reason: "JPEG codec diff (finding 001)".to_string(),
                },
                ExcludeRule {
                    kind: ExcludeKind::Prefix,
                    value: "webpio".to_string(),
                    reason: "no C counterpart".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parse_exclude_rules_ignores_comments_blanks_and_malformed_lines() {
        let content = "\
# header

ext\tjpg
glob\t*.jpg\tunknown kind
ext\tpdf\tnon-deterministic (PR #386)";
        let rules = parse_exclude_rules(content);
        assert_eq!(rules.len(), 1, "only the well-formed line should be parsed");
        assert_eq!(rules[0].value, "pdf");
    }

    #[test]
    fn find_exclusion_matches_by_extension_and_prefix() {
        let rules = vec![
            ExcludeRule {
                kind: ExcludeKind::Ext,
                value: "jpg".to_string(),
                reason: "codec".to_string(),
            },
            ExcludeRule {
                kind: ExcludeKind::Prefix,
                value: "webpio".to_string(),
                reason: "no C counterpart".to_string(),
            },
        ];
        assert_eq!(
            find_exclusion("edge.04.jpg", &rules).map(|r| r.reason.as_str()),
            Some("codec")
        );
        assert_eq!(
            find_exclusion("webpio.02.png", &rules).map(|r| r.reason.as_str()),
            Some("no C counterpart")
        );
        // Extension must match exactly, not the prefix of another key.
        assert!(find_exclusion("edge.04.png", &rules).is_none());
        // Prefix must match the full parsed prefix, not a substring.
        assert!(find_exclusion("webpio_lossless.01.png", &rules).is_none());
    }

    #[test]
    fn check_against_returns_excluded_only_when_unmapped() {
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
        let rules = vec![ExcludeRule {
            kind: ExcludeKind::Ext,
            value: "jpg".to_string(),
            reason: "JPEG codec diff (finding 001)".to_string(),
        }];

        // Mapped keys keep their normal classification — mapping wins.
        let (s, _) = check_c_hash_against("edge.04.jpg", 0xdead_beef, &gmap, &cm, &rules);
        assert_eq!(s, CCompatStatus::Mismatch);

        // Unmapped + rule hit → Excluded, with the rule's reason as detail.
        let (s, d) = check_c_hash_against("other.01.jpg", 0, &gmap, &cm, &rules);
        assert_eq!(s, CCompatStatus::Excluded);
        assert_eq!(d, "JPEG codec diff (finding 001)");

        // Unmapped + no rule hit → Unmapped, unchanged.
        let (s, _) = check_c_hash_against("other.01.png", 0, &gmap, &cm, &rules);
        assert_eq!(s, CCompatStatus::Unmapped);
    }

    #[test]
    fn ccompat_mode_from_value_recognises_off_report_strict() {
        // Use the pure `from_value` decoder rather than mutating env vars:
        // `std::env::set_var` is `unsafe` under concurrent reads (e.g. by
        // other tests calling `RegTestMode::from_env()`), so the env-driven
        // path is exercised indirectly via this codec test.
        assert_eq!(CCompatMode::from_value("off"), CCompatMode::Off);
        assert_eq!(CCompatMode::from_value("0"), CCompatMode::Off);
        assert_eq!(CCompatMode::from_value("strict"), CCompatMode::Strict);
        assert_eq!(CCompatMode::from_value("fail"), CCompatMode::Strict);
        assert_eq!(CCompatMode::from_value("report"), CCompatMode::Report);
        assert_eq!(CCompatMode::from_value(""), CCompatMode::Report);
        assert_eq!(
            CCompatMode::from_value("anything-else"),
            CCompatMode::Report
        );
    }
}
