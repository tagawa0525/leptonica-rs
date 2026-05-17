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

    /// `true` for extensions handled by `leptonica::io::read_image` under
    /// the `all-formats` feature. Mirrors the magic-byte detection in
    /// `src/io/format.rs::detect_format_from_bytes` so that any C output we
    /// can decode is hashed at the pixel level rather than as raw bytes.
    pub fn is_image_extension(ext: &str) -> bool {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            // Common formats
            "png" | "jpg" | "jpeg" | "tif" | "tiff" | "bmp" | "gif" | "webp"
            // JPEG 2000 family
            | "jp2" | "j2k" | "jpf" | "jpx" | "jpm"
            // Leptonica native
            | "spix"
            // PNM family
            | "pnm" | "pbm" | "pgm" | "ppm" | "pam"
        )
    }

    /// `true` for extensions whose C-side output embeds non-determinism
    /// (e.g. `CreationDate`, sequential object identifiers) and therefore
    /// changes hash on every regeneration even when the actual rendered
    /// content is the same. These are skipped entirely so they never end up
    /// in `tests/golden_manifest_c.tsv` and never trigger spurious diffs
    /// on `bash scripts/gen_c_manifest.sh` re-runs.
    ///
    /// The Rust side does not hit this issue because `RegParams::
    /// write_pix_and_check` hashes the in-memory `Pix`, not the serialized
    /// PDF/PS bytes — so Rust manifest entries for `*.pdf` / `*.ps` are
    /// stable.
    pub fn is_unstable_extension(ext: &str) -> bool {
        matches!(ext.to_ascii_lowercase().as_str(), "pdf" | "ps")
    }

    /// Reason a file could not be hashed. `unsupported` files are skipped
    /// without aborting the run because leptonica-rs simply does not implement
    /// the corresponding decoder yet (e.g. TIFF Fax3 / Huffman / RGBPalette);
    /// the matching Rust test will fail to read the file too, so the absence
    /// from the manifest is harmless. Other failures indicate a broken C
    /// output or environment problem and must abort `main`.
    #[derive(Debug)]
    pub struct HashFailure {
        pub unsupported: bool,
        pub message: String,
    }

    /// Hash a file: images via pixel hash, others as raw bytes.
    pub fn hash_file(path: &Path) -> Result<u64, HashFailure> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        if is_image_extension(&ext) {
            let path_str = path.to_str().ok_or_else(|| HashFailure {
                unsupported: false,
                message: format!("non-utf8 path: {}", path.display()),
            })?;
            let pix = read_image(path_str).map_err(|e| {
                let message = format!("read_image failed for {}: {}", path.display(), e);
                // The TIFF / image-decoder crates surface unsupported features
                // with the literal "unsupported" in the message. We rely on
                // that string here; tests/common/params.rs does not need this
                // distinction because no Rust test attempts these files.
                let unsupported = message.contains("unsupported");
                HashFailure {
                    unsupported,
                    message,
                }
            })?;
            Ok(pixel_content_hash(&pix))
        } else {
            let bytes = std::fs::read(path).map_err(|e| HashFailure {
                unsupported: false,
                message: format!("std::fs::read failed for {}: {}", path.display(), e),
            })?;
            Ok(data_content_hash(&bytes))
        }
    }

    /// Format a slice of `(name, hash)` entries as TSV manifest body (caller sorts).
    pub fn format_manifest(entries: &[(String, u64)]) -> String {
        let mut s = String::from(
            "# C-version golden manifest - FNV-1a hashes \
             (pixel content for images, raw bytes for other files)\n",
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
    let usage = "Usage: gen_c_manifest --c-dir DIR --out FILE [--rust-manifest FILE]";
    let mut i = 1;
    while i < argv.len() {
        let flag = argv[i].clone();
        let needs_value = matches!(flag.as_str(), "--c-dir" | "--out" | "--rust-manifest");
        if needs_value && i + 1 >= argv.len() {
            eprintln!("Missing value for {flag}\n{usage}");
            std::process::exit(1);
        }
        match flag.as_str() {
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
                eprintln!("{usage}");
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {other}\n{usage}");
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
    let mut unsupported = 0usize;
    let mut skipped = 0usize;

    for dirent in std::fs::read_dir(&args.c_dir).expect("read_dir") {
        let dirent = match dirent {
            Ok(d) => d,
            Err(e) => {
                eprintln!("error: read_dir entry: {}", e);
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
        // Skip formats that embed non-deterministic metadata (PDF
        // CreationDate, PS object ids, etc.) so the manifest stays stable
        // across `bash scripts/gen_c_manifest.sh` re-runs. See the
        // is_unstable_extension doc for why Rust side is unaffected.
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if logic::is_unstable_extension(ext) {
            skipped += 1;
            continue;
        }
        match logic::hash_file(&path) {
            Ok(h) => {
                entries.insert(name, h);
            }
            Err(f) if f.unsupported => {
                eprintln!("skip (unsupported by leptonica-rs): {}", f.message);
                unsupported += 1;
            }
            Err(f) => {
                eprintln!("error: {}", f.message);
                errors += 1;
            }
        }
    }

    if errors > 0 {
        eprintln!(
            "\nAborting: {errors} hashing errors prevent a clean baseline. \
             Manifest not written to {}.",
            args.out.display()
        );
        std::process::exit(1);
    }

    let sorted: Vec<(String, u64)> = entries.into_iter().collect();
    let body = logic::format_manifest(&sorted);

    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent).expect("create_dir_all");
    }
    std::fs::write(&args.out, &body).expect("write manifest");

    eprintln!(
        "Wrote {} entries to {} (unsupported: {}, skipped: {})",
        sorted.len(),
        args.out.display(),
        unsupported,
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
    fn unstable_extensions_recognised() {
        for ext in ["pdf", "PDF", "ps", "PS"] {
            assert!(is_unstable_extension(ext), "{ext} should be unstable");
        }
        for ext in ["png", "tif", "jpg", "txt", "ba", ""] {
            assert!(!is_unstable_extension(ext), "{ext} should NOT be unstable");
        }
    }

    #[test]
    fn image_extensions_recognised() {
        // Common formats
        for ext in [
            "png", "PNG", "jpg", "Jpeg", "tif", "tiff", "bmp", "gif", "webp",
        ] {
            assert!(is_image_extension(ext), "{ext} should be image");
        }
        // JPEG 2000 family
        for ext in ["jp2", "j2k", "JPF", "jpx", "jpm"] {
            assert!(is_image_extension(ext), "{ext} should be image");
        }
        // Leptonica native + PNM family
        for ext in ["spix", "pnm", "pbm", "pgm", "ppm", "pam", "PAM"] {
            assert!(is_image_extension(ext), "{ext} should be image");
        }
        // Definitely not image extensions
        for ext in ["ba", "na", "pdf", "ps", "txt", "pta", ""] {
            assert!(!is_image_extension(ext), "{ext} should NOT be image");
        }
    }

    #[test]
    fn format_manifest_header_covers_pixel_and_raw_hashes() {
        let mut sorted = vec![
            ("zzz.png".to_string(), 0xdead_beef_dead_beef_u64),
            ("aaa.jpg".to_string(), 0x0123_4567_89ab_cdef_u64),
        ];
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        let s = format_manifest(&sorted);
        let header = s
            .lines()
            .filter(|l| l.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            header.contains("pixel content for images"),
            "header should explain pixel hashing for images: {header}"
        );
        assert!(
            header.contains("raw bytes for other files"),
            "header should explain raw-byte hashing for non-images: {header}"
        );
        let body_lines: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(
            body_lines,
            ["aaa.jpg\t0123456789abcdef", "zzz.png\tdeadbeefdeadbeef"]
        );
    }

    #[test]
    fn hash_file_classifies_unsupported_and_real_errors() {
        use super::logic::hash_file;
        // Missing file → real error, not "unsupported".
        let f = hash_file(std::path::Path::new(
            "/tmp/__definitely_does_not_exist__.png",
        ))
        .unwrap_err();
        assert!(!f.unsupported, "missing file should be a real error");

        // Non-image extension reads raw bytes, missing file is still an error.
        let f = hash_file(std::path::Path::new(
            "/tmp/__definitely_does_not_exist__.ba",
        ))
        .unwrap_err();
        assert!(
            !f.unsupported,
            "missing raw-byte file should be a real error"
        );
    }
}
