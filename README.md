# leptonica-rs

[![CI](https://github.com/tagawa0525/leptonica-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/tagawa0525/leptonica-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/leptonica.svg)](https://crates.io/crates/leptonica)
[![docs.rs](https://docs.rs/leptonica/badge.svg)](https://docs.rs/leptonica)
[![License: BSD-2-Clause](https://img.shields.io/badge/license-BSD--2--Clause-blue.svg)](LICENSE)

A pure Rust reimplementation of the [Leptonica](http://www.leptonica.org/) image processing library — **no C dependencies**, cross-compile friendly, and `unsafe`-free.

[日本語](README.ja.md)

## About Leptonica

[Leptonica](http://www.leptonica.org/) is an open-source C library for image processing and analysis, created and maintained by Dan Bloomberg. With approximately 240,000 lines of code and over 2,700 functions, it covers a broad range of operations from document image processing to natural image analysis. Leptonica has served as a foundational library for projects such as [Tesseract OCR](https://github.com/tesseract-ocr/tesseract/) and [OpenCV](https://github.com/opencv/opencv) for over 20 years.

This project reimplements Leptonica's design and algorithms in Rust. The original [C source code](https://github.com/DanBloomberg/leptonica) and documentation serve as the primary reference.

## Porting Status

Progress against the original 182 source files and 2,286 public functions.

| Metric                   | Value                  |
| ------------------------ | ---------------------- |
| Lines of code            | ~144,000 / ~249,000    |
| Function coverage        | 1,874 / 2,286 (82.0%)  |
| Effective coverage       | 1,874 / 1,874 (100.0%) |
| Regression test coverage | 159 / 159 (100.0%)     |

Details: [Feature comparison](docs/en/porting/feature-comparison.md) / [Test comparison](docs/en/porting/test-comparison.md)

## Module Structure

Single crate `leptonica` with modules matching Leptonica's functional areas:

```text
src/
├── lib.rs          # Public API entry (re-exports core types at root)
├── core/           # Base data structures (Pix, Box, Numa, FPix, Pta, Pixa, Colormap, SArray)
│   └── pixel.rs    # RGBA pixel ops (compose_rgba, extract_rgb, etc.)
├── io/             # Image I/O (PNG, JPEG, TIFF, BMP, GIF, WebP, PDF, PS)
├── transform/      # Geometric transforms (rotate, scale, affine, projective, bilinear)
├── morph/          # Morphological ops (dilate, erode, open, close, DWA, thinning)
├── filter/         # Filtering (convolve, bilateral, rank, edge, adaptmap)
├── color/          # Color processing (quantize, binarize, colorspace, segmentation)
├── region/         # Region analysis (conncomp, ccbord, quadtree, watershed, maze)
└── recog/          # Recognition (barcode, dewarp, baseline, pageseg, jbclass)
```

## Build & Test

```bash
cargo check --all-features
cargo test
cargo test --all-features
cargo clippy --all-features --all-targets
```

### C Reference Source (Optional)

Some porting documents and helper scripts reference the original C source at `reference/leptonica/`.
To use them, clone the C source manually:

```bash
mkdir -p reference
git clone https://github.com/DanBloomberg/leptonica.git reference/leptonica
```

The `reference/` directory is listed in `.gitignore` and will not be tracked by git.

## Documentation

- `CLAUDE.md` -- Development conventions and process rules
- `docs/plans/` -- Implementation plans for each feature
- `docs/porting/` -- Porting reference materials (prompts, feature comparison, test comparison)

## License

This project is distributed under the [BSD 2-Clause License](LICENSE), the same license as the original [Leptonica](http://www.leptonica.org/).

## How This Project Is Built

The porting work is carried out primarily by AI coding agents, including [Claude Code](https://docs.anthropic.com/en/docs/claude-code). A human maintainer defines the overall architecture, process rules, and acceptance criteria, while the agents read the original C source, write Rust code, and run tests under those constraints. Every commit goes through CI and automated review before merging.

This means the codebase may contain patterns that reflect AI-assisted development. Bug reports and feedback are welcome.

## Acknowledgments

This project relies entirely on the source code, documentation, and regression tests of the original C Leptonica. It would not exist without the decades of design, implementation, and maintenance work by Dan Bloomberg and the Leptonica contributors.
