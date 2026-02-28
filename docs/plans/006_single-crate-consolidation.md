# Plan: 単一クレート統合

Status: IMPLEMENTED

## Context

crates.io 公開時に `leptonica-core`, `leptonica-io` 等の名前を占有したくない。
画像処理ライブラリとしての性質上（C 版も単一ライブラリ）、`leptonica` 一本で公開するのが適切。
commit `46185bd` を base として `refactor/single-crate` ブランチで作業する。

## 方針

- **ワークスペースを廃止**し、ルート `Cargo.toml` を単一パッケージ定義にする
- **`crates/` と `leptonica/` を削除**し、全ソースを `src/` 以下に集約
- **公開 API は現状維持**（`leptonica::Pix`, `leptonica::morph::dilate` 等）
- `leptonica-doc`（5行）は削除
- `leptonica-test` は `tests/common/` に移動（integration test helper）
- **ファイル移動は `git mv` で行い、リネーム履歴を保持する**
- `CARGO_MANIFEST_DIR` がリポジトリルートになるため、パス計算が簡素化される

## 命名衝突の解消

`leptonica-core/src/lib.rs:62` に `pub mod color { ... }` （RGBA ピクセル操作）が存在し、
`color` モジュール（旧 leptonica-color: 色処理全般）と衝突する。

**解決策**: インラインの `pub mod color { ... }` を `core/pixel.rs` に抽出・リネーム。

- 内部コードは `crate::core::pixel::compose_rgba` 等に変換
- 外部 API は `leptonica::core::pixel` として公開

## 統合後のディレクトリ構成

```rust
leptonica-rs/
├── Cargo.toml              # パッケージ定義（workspace なし）
├── Cargo.lock
├── src/
│   ├── lib.rs
│   ├── core/               # ← src/core/src/
│   │   ├── mod.rs          # (旧 lib.rs)
│   │   ├── pixel.rs        # ← lib.rs のインライン pub mod color を抽出
│   │   ├── box_/
│   │   ├── colormap/
│   │   ├── error.rs
│   │   ├── fpix/
│   │   ├── numa/
│   │   ├── pix/
│   │   ├── pixa/
│   │   ├── pta/
│   │   └── sarray/
│   ├── io/                 # ← src/io/src/
│   ├── transform/          # ← src/transform/src/
│   ├── color/              # ← src/color/src/
│   ├── region/             # ← src/region/src/
│   ├── morph/              # ← src/morph/src/
│   ├── filter/             # ← src/filter/src/
│   └── recog/              # ← src/recog/src/
├── tests/
│   ├── common/
│   │   ├── mod.rs          # ← leptonica-test/src/lib.rs
│   │   ├── error.rs
│   │   └── params.rs
│   ├── boxa1_reg.rs        # ← src/core/tests/
│   ├── gifio_reg.rs        # ← src/io/tests/
│   ├── ...                 # （計162ファイル、名前衝突なし）
│   ├── data/images/        # 変更なし
│   ├── golden/             # 変更なし
│   └── regout/             # 変更なし
└── reference/              # 変更なし
```

## `Cargo.toml`（ルート、パッケージ定義）

```toml
[package]
name = "leptonica"
version = "0.1.0"
edition = "2024"
license = "BSD-2-Clause"
repository = "https://github.com/tagawa/leptonica-rs"
description = "Rust port of Leptonica image processing library"

[dependencies]
thiserror = "2.0.18"
rand = "0.10.0"
png         = { version = "0.18.1", optional = true }
jpeg-decoder = { version = "0.3.2", optional = true }
jpeg-encoder = { version = "0.7.0", optional = true }
gif         = { version = "0.14.1", optional = true }
tiff        = { version = "0.11.3", optional = true }
half        = { version = "2.7.1", optional = true }
image-webp  = { version = "0.2.4", optional = true }
hayro-jpeg2000 = { version = "0.3.2", optional = true }
pdf-writer  = { version = "0.14.0", optional = true }
miniz_oxide = { version = "0.9.0", optional = true }

[features]
default = ["bmp", "pnm", "png-format", "jpeg"]
bmp = []
pnm = []
png-format  = ["png"]
jpeg        = ["jpeg-decoder", "jpeg-encoder"]
gif-format  = ["gif"]
tiff-format = ["tiff", "half"]
webp-format = ["image-webp"]
jp2k-format = ["hayro-jpeg2000"]
pdf-format  = ["pdf-writer", "miniz_oxide"]
ps-format   = ["miniz_oxide"]
all-formats = ["bmp", "pnm", "png-format", "jpeg", "gif-format",
               "tiff-format", "webp-format", "jp2k-format",
               "pdf-format", "ps-format"]

[profile.dev]
opt-level = 1
```

**変更点**: `gif-format` から `leptonica-color` 依存を除去（同一クレート内なので不要）。

## `src/lib.rs` の構成

```rust
// internal modules
pub mod core;
pub mod io;
pub mod transform;
pub mod color;
pub mod region;
pub mod morph;
pub mod filter;
pub mod recog;

// core 型をルートに再エクスポート（現行 API 維持）
pub use core::{
    Box, Boxa, Boxaa, SizeRelation,
    PixColormap, RgbaQuad, ColormapArrays, ComponentsPerColor,
    NonOpaqueInfo, RangeComponent, RangeValues,
    Error, Result,
    DPix, FPix, NegativeHandling,
    CountRelativeToZero, HistogramResult, HistogramStats, InterpolationType,
    Numa, Numaa, SortOrder, ThresholdComparison, WindowedStats,
    SpixHeader,
    DiffDirection, ExtremeResult, ExtremeType, MaxValueResult, PixelMaxType,
    PixelStatType, RowColumnStats, StatsRequest,
    BlendMode, Color, ColorHistogram, CompareResult, CompareType, ContourOutput,
    GrayBlendType, ImageFormat, InColor, InitColor, MaskBlendType,
    Pix, PixMut, PixelDepth, PixelDiffResult, PixelOp, RopOp, ScanDirection,
    blend_with_gray_mask, correlation_binary,
    Pixa, PixaSortType, Pixaa,
    Pta, Ptaa,
    Sarray, Sarraya,
};
```

## use 文の変換ルール

### ライブラリソース (src/)

**クレート間参照（全モジュール共通）:**

| 変換前                            | 変換後                       |
| --------------------------------- | ---------------------------- |
| `use leptonica_core::X`           | `use crate::core::X`         |
| `use crate::core::pixel::X`       | `use crate::core::pixel::X`  |
| `crate::core::pixel::X`（パス内） | `crate::core::pixel::X`      |
| `use leptonica_io::X`             | `use crate::io::X`           |
| `use leptonica_transform::X`      | `use crate::transform::X`    |
| `use leptonica_morph::X`          | `use crate::morph::X`        |
| `use leptonica_color::X`          | `use crate::color::X`        |
| `use leptonica_region::X`         | `use crate::region::X`       |
| `use leptonica_filter::X`         | `use crate::filter::X`       |
| `use leptonica_recog::X`          | `use crate::recog::X`        |
| `#[from] leptonica_core::Error`   | `#[from] crate::core::Error` |

**自クレート内参照 (`use crate::X`):**

各モジュール内のファイルは旧クレートの `use crate::X` を使っている。
移動後は `crate` が `leptonica` クレートルートを指すため変換が必要:

| モジュール    | 変換前         | 変換後                    |
| ------------- | -------------- | ------------------------- |
| core/ 内      | `use crate::X` | `use crate::core::X`      |
| io/ 内        | `use crate::X` | `use crate::io::X`        |
| transform/ 内 | `use crate::X` | `use crate::transform::X` |
| color/ 内     | `use crate::X` | `use crate::color::X`     |
| region/ 内    | `use crate::X` | `use crate::region::X`    |
| morph/ 内     | `use crate::X` | `use crate::morph::X`     |
| filter/ 内    | `use crate::X` | `use crate::filter::X`    |
| recog/ 内     | `use crate::X` | `use crate::recog::X`     |

**注意**: 変換後のパスが `crate::core::core::` のように二重にならないよう、
すでに `crate::(core|io|...)::` になっている行は除外する。

**io/mod.rs 特有**: `pub use leptonica_core::{ImageFormat, Pix, PixMut, PixelDepth};`
→ `pub use crate::core::{ImageFormat, Pix, PixMut, PixelDepth};`

**io/gif.rs 特有**: `use leptonica_color::{OctreeOptions, octree_quant};`
→ `use crate::color::{OctreeOptions, octree_quant};`
（旧: optional 依存 → 新: 同一クレート内モジュール、常時利用可能）

### テストファイル (tests/*.rs)

| 変換前                            | 変換後                                         |
| --------------------------------- | ---------------------------------------------- |
| `use leptonica_core::X`           | `use leptonica::X`（ルート再エクスポート経由） |
| `use crate::core::pixel::X`       | `use leptonica::core::pixel::X`                |
| `use leptonica_core::color`       | `use leptonica::core::pixel`                   |
| `crate::core::pixel::X`（パス内） | `leptonica::core::pixel::X`                    |
| `use leptonica_morph::X`          | `use leptonica::morph::X`                      |
| `use leptonica_io::X`             | `use leptonica::io::X`                         |
| `use leptonica_transform::X`      | `use leptonica::transform::X`                  |
| `use leptonica_color::X`          | `use leptonica::color::X`                      |
| `use leptonica_region::X`         | `use leptonica::region::X`                     |
| `use leptonica_filter::X`         | `use leptonica::filter::X`                     |
| `use leptonica_recog::X`          | `use leptonica::recog::X`                      |
| `use leptonica_test::RegParams`   | `mod common; use common::RegParams;`           |
| `use leptonica_test::X`           | `use common::X;`                               |

**注意**: `mod common;` は各テストファイルに1回だけ必要。

### tests/common/ (旧 leptonica-test)

| 変換前                    | 変換後                                 |
| ------------------------- | -------------------------------------- |
| `use leptonica_core::Pix` | `use leptonica::Pix`                   |
| `use leptonica_io::X`     | `use leptonica::io::X`                 |
| `use crate::X`            | `use super::X`（common/ 内の相対参照） |

`workspace_root()` のパス計算を修正:

- 変換前: `format!("{}/../..", env!("CARGO_MANIFEST_DIR"))` (2階層上)
- 変換後: `env!("CARGO_MANIFEST_DIR").to_string()` (ルート直下)

## git 戦略

ブランチ: `refactor/single-crate`（`46185bd` から分岐済み）

### コミット構成

`git mv` でリネーム履歴を保持する。中間コミットはビルドが通らなくてもよい。
最終状態で `cargo check/clippy/test` が全て通ること。

| # | メッセージ                                  | 内容                                                                                                                                    | ビルド |
| - | ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| 1 | `docs: add single-crate consolidation plan` | 本プランファイル                                                                                                                        | ✓      |
| 2 | `refactor: git mv で全ソースを src/ に集約` | 全モジュールの `git mv` + `lib.rs` → `mod.rs` リネーム + テストファイル移動 + テストヘルパー移動（内容変更なし）                        | ✗      |
| 3 | `refactor: 単一クレートに統合`              | use 文一括変換 + `pub mod color` → `pixel.rs` 抽出 + Cargo.toml 置換 + lib.rs 書換 + tests/common 変換 + `git rm -r crates/ leptonica/` | ✓      |
| 4 | `docs: CLAUDE.md を単一クレート構成に更新`  | ドキュメント更新                                                                                                                        | ✓      |

**1つの PR** にまとめる。

### コミット2: git mv の詳細

全ファイルを純粋に移動。内容の変更は一切しない。

```bash
# 8モジュールのソースを src/ に移動
for mod in core io transform color region morph filter recog; do
  mkdir -p src/$mod
  git mv crates/leptonica-$mod/src/* src/$mod/
  git mv src/$mod/lib.rs src/$mod/mod.rs
done

# io/ps/ サブディレクトリは git mv で自動的に移動される

# ファサードの lib.rs を src/ に移動
git mv leptonica/src/lib.rs src/lib.rs

# leptonica-test → tests/common/
mkdir -p tests/common
git mv tests/common/src/* tests/common/
git mv tests/common/lib.rs tests/common/mod.rs

# 各クレートのテストを tests/ に移動（計162ファイル、名前衝突なし）
for crate_dir in crates/leptonica-*/tests; do
  [ -d "$crate_dir" ] || continue
  for test_file in "$crate_dir"/*.rs; do
    [ -f "$test_file" ] || continue
    git mv "$test_file" tests/
  done
done
```

### コミット3: コード変更の詳細

**3a. `core/mod.rs` からインライン `pub mod color { ... }` を `pixel.rs` に抽出**

- `src/core/mod.rs` の行62〜360 にある `pub mod color { ... }` ブロックの中身を

  `src/core/pixel.rs` に抽出（インデントを1段除去）

- `mod.rs` では `pub mod color { ... }` を `pub mod pixel;` に置換

**3b. 各モジュール内の `use crate::` → `use crate::<module>::` 変換**

- すでに `crate::(core|io|transform|color|region|morph|filter|recog)::` の行は除外

**3c. クレート間参照の変換**

```text
leptonica_core:: → crate::core::
leptonica_io:: → crate::io::
leptonica_transform:: → crate::transform::
leptonica_color:: → crate::color::
leptonica_region:: → crate::region::
leptonica_morph:: → crate::morph::
leptonica_filter:: → crate::filter::
leptonica_recog:: → crate::recog::
```

**3d. `core::color` → `core::pixel` リネーム**

- `crate::core::color::` → `crate::core::pixel::` （ただし `crate::core::colormap::` は変換しない）
- `crate::color` はそのまま（color モジュール = 旧 leptonica-color）

**3e. テストファイルの use 文変換**

- `leptonica_core::X` → `leptonica::X`（ルート再エクスポート経由）
- `leptonica_XXX::Y` → `leptonica::XXX::Y`
- `leptonica_test::Z` → `common::Z`（+ `mod common;` 追加）
- `leptonica_core::color` → `leptonica::core::pixel`

**3f. tests/common/ の変換**

- `leptonica_core::Pix` → `leptonica::Pix`
- `leptonica_io::X` → `leptonica::io::X`
- `crate::X` → `super::X`
- `workspace_root()` → `env!("CARGO_MANIFEST_DIR").to_string()`

**3g. Cargo.toml の置換**

- ルート `Cargo.toml` を上記テンプレートで置換
- `leptonica/Cargo.toml` は `git rm`

**3h. `src/lib.rs` の書換**

- 上記テンプレートの通り

**3i. 旧ディレクトリの削除**

- `git rm -r crates/`
- `git rm -r leptonica/` （src/lib.rs は既に移動済み、Cargo.toml を削除）

**3j. io/mod.rs の re-export**

- `pub use leptonica_core::{ImageFormat, Pix, PixMut, PixelDepth};`

  → `pub use crate::core::{ImageFormat, Pix, PixMut, PixelDepth};`

## 重要ファイル

- `Cargo.toml` — ルートパッケージ定義（workspace 廃止）
- `src/lib.rs` — 公開 API の入口
- `src/core/src/lib.rs:62-360` — `pub mod color` 命名衝突の発生源
- `src/io/src/gif.rs:8` — `leptonica_color` optional 依存
- `tests/common/src/lib.rs:48-52` — `workspace_root()` パス計算
- `src/io/Cargo.toml:32` — `gif-format = ["gif", "leptonica-color"]`

## 検証方法

コミット3以降で実行:

```bash
cargo check --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --all -- --check
cargo test --all-features
```
