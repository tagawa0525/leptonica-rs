# leptonica-rs

A Rust reimplementation of the [Leptonica](http://www.leptonica.org/) image processing library.

[日本語](README.ja.md)

## About Leptonica

[Leptonica](http://www.leptonica.org/) is an open-source C library for image processing and analysis, created and maintained by Dan Bloomberg. With approximately 240,000 lines of code and over 2,700 functions, it covers a broad range of operations from document image processing to natural image analysis. Leptonica has served as a foundational library for projects such as [Tesseract OCR](https://github.com/tesseract-ocr/tesseract/) and [OpenCV](https://github.com/opencv/opencv) for over 20 years.

This project reimplements Leptonica's design and algorithms in Rust. The original C source code and documentation serve as the primary reference, included as a git submodule under `reference/leptonica/`.

## Porting Status

Progress against the original 182 source files and 1,880 public functions.

| Metric               | Value                      |
| -------------------- | -------------------------- |
| Lines of code        | ~120,000 / ~240,000       |
| Function coverage    | 1,128 / 1,880 (60.0%)    |
| Regression test coverage | 59 / 159 (37.1%)      |

Details: [Feature comparison](docs/porting/feature-comparison.md) / [Test comparison](docs/porting/test-comparison.md)

## Crate Structure

```text
leptonica-rs/
├── crates/
│   ├── leptonica-core/        # Pix, Box, Numa, FPix and other base data structures
│   ├── leptonica-io/          # Image I/O (PNG, JPEG, TIFF, GIF, WebP, etc.)
│   ├── leptonica-morph/       # Morphological operations (binary, grayscale, DWA, thinning)
│   ├── leptonica-transform/   # Geometric transforms (rotate, scale, affine, etc.)
│   ├── leptonica-filter/      # Filtering (bilateral, rank, adaptmap, convolve, edge)
│   ├── leptonica-color/       # Color processing (segmentation, quantize, threshold, colorspace)
│   ├── leptonica-region/      # Region analysis (conncomp, ccbord, quadtree, watershed, maze)
│   ├── leptonica-recog/       # Recognition (barcode, dewarp, baseline, pageseg, jbclass)
│   └── leptonica-test/        # Test infrastructure
├── leptonica/                 # Facade crate (re-exports)
└── reference/leptonica/       # Original C source (git submodule, read-only)
```

### Dependency Graph

```text
leptonica-recog → leptonica-morph, leptonica-transform, leptonica-region, leptonica-color, leptonica-core
leptonica-morph, leptonica-transform, leptonica-filter, leptonica-color → leptonica-io, leptonica-core
leptonica-region → leptonica-core
leptonica-io → leptonica-core
```

## Build & Test

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace
```

### Fetching the C Reference

```bash
git submodule update --init
```

> **Note**: `.gitmodules` uses SSH URLs (`git@github.com:...`).
> If SSH keys are not configured, change the URL to HTTPS format.

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
