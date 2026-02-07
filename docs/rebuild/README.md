# leptonica-rs

[Leptonica](http://www.leptonica.org/) 画像処理ライブラリのRust移植。

C版 Leptonica（約240,000行）の機能をRustで再実装し、安全性・保守性を向上させる。

## Crate構成

```text
leptonica-rs/
├── crates/
│   ├── leptonica-core/        # Pix, Box, Numa, FPix等の基本データ構造
│   ├── leptonica-io/          # 画像I/O (PNG, JPEG, TIFF, GIF, WebP等)
│   ├── leptonica-transform/   # 幾何変換 (rotate, scale, affine等)
│   ├── leptonica-filter/      # フィルタリング (bilateral, rank, adaptmap, convolve, edge)
│   ├── leptonica-color/       # 色処理 (segmentation, quantize, threshold, colorspace)
│   ├── leptonica-morph/       # 形態学演算 (binary, grayscale, DWA, thinning)
│   ├── leptonica-region/      # 領域解析 (conncomp, ccbord, quadtree, watershed, maze)
│   ├── leptonica-recog/       # 認識 (barcode, dewarp, baseline, pageseg, jbclass)
│   └── leptonica-test/        # テストインフラ
├── leptonica/                 # ファサードcrate (re-export)
└── reference/leptonica/       # C版ソース (git submodule, read-only参照)
```

### 依存関係

```text
leptonica-recog → leptonica-region → leptonica-filter → leptonica-color
    → leptonica-transform → leptonica-morph → leptonica-io → leptonica-core
```

## ビルド・テスト

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace
```

## C版リファレンス

`reference/leptonica/` にC版ソースをgit submoduleとして配置:

```bash
git submodule update --init
```

回帰テストはC版の `prog/*_reg.c`（160ファイル）に対応する形で作成する。

## ドキュメント

- `CLAUDE.md` — 開発規約・プロセスルール
- `docs/rebuild/prompt.md` — 移植作業の詳細プロンプト
- `docs/rebuild/` — 前回実装からの引き継ぎ資料
- `docs/plans/` — 各機能の実装計画書

## ライセンス

Apache-2.0
