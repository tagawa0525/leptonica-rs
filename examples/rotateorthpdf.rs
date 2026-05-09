//! Rotate selected images by 90° increments and combine them into a PDF.
//!
//! Rust port of C Leptonica's `prog/rotateorthpdf.c`.
//!
//! # Usage
//!
//! ```text
//! rotateorthpdf <indir> <substr> <rotstring> <scalefactor> <quality> <title> <fileout>
//! ```
//!
//! * `indir` — directory containing input images
//! * `substr` — substring filter (`""` for all files)
//! * `rotstring` — rotation specifier; see `parse_rotation_string` for the
//!   three accepted modes (per-image digits, uniform, or `(idx,rot)` pairs)
//! * `scalefactor` — `0.0` selects the default of `1.0`, otherwise clamped to
//!   `(0.0, 2.0]`
//! * `quality` — JPEG quality, `0` for default 75, otherwise clamped to `[25, 95]`
//! * `title` — pdf title; use `none` to omit
//! * `fileout` — output pdf path
//!
//! # Examples
//!
//! ```sh
//! # Rotate page 2 by 180° and page 4 by 90° cw
//! cargo run --example rotateorthpdf -- ./scans "" 00201 1.0 0 "scanned" out.pdf
//!
//! # Rotate every page by 90° cw
//! cargo run --example rotateorthpdf -- ./scans "" 41 1.0 0 none out.pdf
//!
//! # Rotate only the listed pages
//! cargo run --example rotateorthpdf -- ./scans "" "5(3,2)(8,1)" 1.0 0 none out.pdf
//! ```

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use leptonica::io::pdf::rotate_orth_files_to_pdf;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 8 {
        eprintln!(
            "usage: {} <indir> <substr> <rotstring> <scalefactor> <quality> <title> <fileout>",
            args[0]
        );
        return ExitCode::from(1);
    }

    let indir = &args[1];
    let substr = &args[2];
    let rotstring = &args[3];
    let scalefactor: f32 = args[4].parse().unwrap_or(1.0);
    let quality: i32 = args[5].parse().unwrap_or(0);
    let title = &args[6];
    let fileout = &args[7];

    let mut paths: Vec<PathBuf> = match fs::read_dir(indir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| {
                substr.is_empty()
                    || p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.contains(substr.as_str()))
                        .unwrap_or(false)
            })
            .collect(),
        Err(e) => {
            eprintln!("failed to read directory {indir}: {e}");
            return ExitCode::from(1);
        }
    };
    paths.sort();

    if paths.is_empty() {
        eprintln!("no input images matched (dir={indir}, substr={substr:?})");
        return ExitCode::from(1);
    }

    eprintln!("rotating {} images ...", paths.len());
    let pdf = match rotate_orth_files_to_pdf(&paths, rotstring, scalefactor, quality, title) {
        Ok(buf) => buf,
        Err(e) => {
            eprintln!("rotate_orth_files_to_pdf failed: {e}");
            return ExitCode::from(1);
        }
    };

    if let Err(e) = fs::write(fileout, &pdf) {
        eprintln!("failed to write {fileout}: {e}");
        return ExitCode::from(1);
    }
    eprintln!("wrote {fileout} ({} bytes)", pdf.len());
    ExitCode::SUCCESS
}
