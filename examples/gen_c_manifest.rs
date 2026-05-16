//! Generate `tests/golden_manifest_c.tsv` from C-version leptonica golden outputs.
//!
//! Walks a directory of C-version regression outputs (typically `/tmp/lept/regout`),
//! computes the same FNV-1a pixel-content hash used by `tests/common/params.rs`,
//! and writes a TSV manifest sorted by filename.
//!
//! See `docs/plans/901_c-hash-compat.md`.
//!
//! # Usage
//!
//! ```text
//! cargo run --release --example gen_c_manifest --features all-formats -- \
//!     --c-dir /tmp/lept/regout \
//!     --out  tests/golden_manifest_c.tsv \
//!     [--rust-manifest tests/golden_manifest.tsv]
//! ```
//!
//! If `--rust-manifest` is given, only the filenames present in the Rust manifest
//! are emitted; otherwise every supported file under `--c-dir` is hashed.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod logic {
    use leptonica::Pix;
    use leptonica::io::read_image;
    use std::path::Path;

    pub const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    pub const FNV_PRIME: u64 = 0x100000001b3;

    /// FNV-1a hash of Pix dimensions and pixel content. Must match
    /// `tests/common/params.rs::pixel_content_hash` so manifests stay comparable.
    pub fn pixel_content_hash(pix: &Pix) -> u64 {
        let mut h = FNV_OFFSET_BASIS;
        for b in pix.width().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        for b in pix.height().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        for b in (pix.depth() as u32).to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        for y in 0..pix.height() {
            for x in 0..pix.width() {
                let px = pix.get_pixel(x, y).unwrap_or(0);
                for b in px.to_le_bytes() {
                    h ^= b as u64;
                    h = h.wrapping_mul(FNV_PRIME);
                }
            }
        }
        h
    }

    /// FNV-1a hash of raw byte data. Must match
    /// `tests/common/params.rs::data_content_hash`.
    pub fn data_content_hash(data: &[u8]) -> u64 {
        let mut h = FNV_OFFSET_BASIS;
        for &b in data {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        h
    }

    /// `true` for extensions handled by `read_image`.
    pub fn is_image_extension(ext: &str) -> bool {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "tif" | "tiff" | "bmp" | "gif" | "webp"
        )
    }

    /// Hash a file: images via pixel hash, others as raw bytes.
    pub fn hash_file(path: &Path) -> Result<u64, String> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        if is_image_extension(&ext) {
            let path_str = path.to_str().ok_or("non-utf8 path")?;
            let pix = read_image(path_str)
                .map_err(|e| format!("read_image failed for {}: {}", path.display(), e))?;
            Ok(pixel_content_hash(&pix))
        } else {
            let bytes = std::fs::read(path)
                .map_err(|e| format!("std::fs::read failed for {}: {}", path.display(), e))?;
            Ok(data_content_hash(&bytes))
        }
    }

    /// Format a slice of `(name, hash)` entries as TSV manifest body (caller sorts).
    pub fn format_manifest(entries: &[(String, u64)]) -> String {
        let mut s = String::from(
            "# C-version golden manifest - pixel-content hashes from C leptonica output\n",
        );
        s.push_str("# Format: name<TAB>hash (FNV-1a hex)\n");
        for (name, h) in entries {
            s.push_str(&format!("{}\t{:016x}\n", name, h));
        }
        s
    }
}

fn load_rust_manifest_keys(path: &Path) -> std::io::Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let mut keys = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('\t') {
            keys.push(name.to_string());
        }
    }
    Ok(keys)
}

struct Args {
    c_dir: PathBuf,
    out: PathBuf,
    rust_manifest: Option<PathBuf>,
}

fn parse_args() -> Args {
    let mut c_dir = PathBuf::from("/tmp/lept/regout");
    let mut out = PathBuf::from(format!(
        "{}/tests/golden_manifest_c.tsv",
        env!("CARGO_MANIFEST_DIR")
    ));
    let mut rust_manifest: Option<PathBuf> = None;

    let argv: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "--c-dir" => {
                i += 1;
                c_dir = PathBuf::from(&argv[i]);
            }
            "--out" => {
                i += 1;
                out = PathBuf::from(&argv[i]);
            }
            "--rust-manifest" => {
                i += 1;
                rust_manifest = Some(PathBuf::from(&argv[i]));
            }
            "--help" | "-h" => {
                eprintln!("Usage: gen_c_manifest --c-dir DIR --out FILE [--rust-manifest FILE]");
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    Args {
        c_dir,
        out,
        rust_manifest,
    }
}

fn main() {
    let args = parse_args();

    if !args.c_dir.is_dir() {
        eprintln!(
            "C output directory not found: {}\n\
             Run scripts/gen_c_manifest.sh first to populate it.",
            args.c_dir.display()
        );
        std::process::exit(1);
    }

    let restrict_keys: Option<Vec<String>> = args
        .rust_manifest
        .as_deref()
        .map(|p| load_rust_manifest_keys(p).expect("read rust manifest"));

    let mut entries: BTreeMap<String, u64> = BTreeMap::new();
    let mut errors = 0usize;
    let mut skipped = 0usize;

    for dirent in std::fs::read_dir(&args.c_dir).expect("read_dir") {
        let dirent = match dirent {
            Ok(d) => d,
            Err(e) => {
                eprintln!("read_dir entry error: {}", e);
                errors += 1;
                continue;
            }
        };
        let path = dirent.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => {
                skipped += 1;
                continue;
            }
        };
        if let Some(keys) = restrict_keys.as_ref()
            && !keys.contains(&name)
        {
            skipped += 1;
            continue;
        }
        match logic::hash_file(&path) {
            Ok(h) => {
                entries.insert(name, h);
            }
            Err(e) => {
                eprintln!("warning: {}", e);
                errors += 1;
            }
        }
    }

    let sorted: Vec<(String, u64)> = entries.into_iter().collect();
    let body = logic::format_manifest(&sorted);

    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent).expect("create_dir_all");
    }
    std::fs::write(&args.out, &body).expect("write manifest");

    eprintln!(
        "Wrote {} entries to {} (errors: {}, skipped: {})",
        sorted.len(),
        args.out.display(),
        errors,
        skipped,
    );
}

#[cfg(test)]
mod tests {
    use super::logic::*;

    #[test]
    fn data_hash_empty_input_returns_offset_basis() {
        assert_eq!(data_content_hash(b""), FNV_OFFSET_BASIS);
    }

    #[test]
    fn data_hash_is_deterministic_and_input_sensitive() {
        assert_eq!(data_content_hash(b"foo"), data_content_hash(b"foo"));
        assert_ne!(data_content_hash(b"foo"), data_content_hash(b"bar"));
        assert_ne!(data_content_hash(b"foo"), data_content_hash(b"foo\0"));
    }

    #[test]
    fn image_extensions_recognised() {
        for ext in [
            "png", "PNG", "jpg", "Jpeg", "tif", "tiff", "bmp", "gif", "webp",
        ] {
            assert!(is_image_extension(ext), "{ext} should be image");
        }
        for ext in ["ba", "na", "pdf", "txt", "pta", ""] {
            assert!(!is_image_extension(ext), "{ext} should NOT be image");
        }
    }

    #[test]
    fn format_manifest_emits_header_and_tsv_rows() {
        let mut sorted = vec![
            ("zzz.png".to_string(), 0xdead_beef_dead_beef_u64),
            ("aaa.jpg".to_string(), 0x0123_4567_89ab_cdef_u64),
        ];
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        let s = format_manifest(&sorted);
        let header_lines: Vec<&str> = s.lines().filter(|l| l.starts_with('#')).collect();
        assert!(
            header_lines
                .iter()
                .any(|l| l.contains("C-version golden manifest")),
            "header should mention C-version: {s}"
        );
        let body_lines: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(
            body_lines,
            ["aaa.jpg\t0123456789abcdef", "zzz.png\tdeadbeefdeadbeef"]
        );
    }
}
