//! Generate `tests/golden_manifest_c.tsv` from C-version leptonica golden outputs.
//!
//! Walks a directory of C-version regression outputs (typically `/tmp/lept/regout`),
//! computes the same FNV-1a pixel-content hash used by `tests/common/params.rs`,
//! and writes a TSV manifest sorted by filename.
//!
//! See `docs/plans/901_c-hash-compat.md`.

use std::path::Path;

#[allow(dead_code)]
mod logic {
    use leptonica::Pix;
    use std::path::Path;

    pub const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    pub const FNV_PRIME: u64 = 0x100000001b3;

    /// FNV-1a hash of Pix dimensions and pixel content. Must match
    /// `tests/common/params.rs::pixel_content_hash` so manifests stay comparable.
    pub fn pixel_content_hash(_pix: &Pix) -> u64 {
        unimplemented!("pixel_content_hash")
    }

    /// FNV-1a hash of raw byte data. Must match
    /// `tests/common/params.rs::data_content_hash`.
    pub fn data_content_hash(_data: &[u8]) -> u64 {
        unimplemented!("data_content_hash")
    }

    /// `true` for extensions handled by `read_image`.
    pub fn is_image_extension(_ext: &str) -> bool {
        unimplemented!("is_image_extension")
    }

    /// Hash a file: images via pixel hash, others as raw bytes.
    pub fn hash_file(_path: &Path) -> Result<u64, String> {
        unimplemented!("hash_file")
    }

    /// Format a slice of `(name, hash)` entries as TSV manifest body (caller sorts).
    pub fn format_manifest(_entries: &[(String, u64)]) -> String {
        unimplemented!("format_manifest")
    }
}

#[allow(dead_code)]
fn load_rust_manifest_keys(_path: &Path) -> std::io::Result<Vec<String>> {
    unimplemented!("load_rust_manifest_keys")
}

fn main() {
    unimplemented!("gen_c_manifest main (Phase 1 GREEN コミットで実装)");
}

#[cfg(test)]
mod tests {
    use super::logic::*;

    #[test]
    #[ignore = "not yet implemented"]
    fn data_hash_empty_input_returns_offset_basis() {
        assert_eq!(data_content_hash(b""), FNV_OFFSET_BASIS);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn data_hash_is_deterministic_and_input_sensitive() {
        assert_eq!(data_content_hash(b"foo"), data_content_hash(b"foo"));
        assert_ne!(data_content_hash(b"foo"), data_content_hash(b"bar"));
        assert_ne!(data_content_hash(b"foo"), data_content_hash(b"foo\0"));
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
