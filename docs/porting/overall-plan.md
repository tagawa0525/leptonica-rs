# Leptonica Rust移植 全体計画

**Status: IMPLEMENTED**（全フェーズ実装完了 2026-02-22、その後 plan 901
で C 互換性検証を追加 2026-05）

## 概要

Leptonica画像処理ライブラリ（C言語、約240,000行）をRustに移植する。
機能群ごとにフェーズ分けし、段階的に実装可能な成果物を作成する。

### 決定事項

- **プロジェクト構成**: Workspace（複数crate） — 2026-02 時点。
  **その後 [plan 006_single-crate-consolidation](../plans/006_single-crate-consolidation.md)
  で単一 crate `leptonica` に統合済み**。本書 1 章の workspace 図は
  当時のもの。実際の現状構成は `src/{core,io,morph,transform,filter,
  color,region,recog}/` の単一 crate
- **image crate相互運用**: 必要（DynamicImage ↔ Pix変換）
- **認識・分類機能（フェーズ7）**: 実装範囲に含める

### 拡張: C 互換性検証 (plan 901、2026-05)

移植本体の完了後、C 版 leptonica との **pixel-level 互換性** を継続的に
測定する仕組みを [plan 901](../plans/901_c-hash-compat.md) で導入した
(PR #377〜#391 で実装)。

- C 側 hash manifest (`tests/golden_manifest_c.tsv`、1879 entries) と
  Rust 出力ハッシュをランタイムで照合
- 現状 (2026-05-20): **Ok 22 / Mismatch 31 / MissingC 0 / Unmapped 520**
- 詳細: [c-compat-status.md](c-compat-status.md)

---

## 1. プロジェクト構成

> **2026-05-20 注**: 本章の Workspace 構成図は 2026-02 の plan 006 単一
> crate 統合前のもの。**現状の構成は単一 crate `leptonica` の
> `src/{core,io,morph,transform,filter,color,region,recog}/`** で、本図
> とは異なる。本章は当時の決定経緯を残すための歴史的記録として保持し、
> 現状構成は `CLAUDE.md` または `docs/plans/006_single-crate-
> consolidation.md` を参照する。

### Workspace構成（2026-02 時点。plan 006 で単一 crate に統合済み）

```text
leptonica-rs/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── leptonica-core/           # 基本データ構造（Pix/PixMut, Box/Boxa, Numa, Pta 等）
│   ├── leptonica-io/             # 画像I/O（PNG/JPEG/TIFF/GIF/WebP/BMP/PNM/SPIX）
│   ├── leptonica-morph/          # 形態学演算（binary/gray morph, DWA, thin 等）
│   ├── leptonica-transform/      # 幾何変換（rotate, scale, affine, shear 等）
│   ├── leptonica-filter/         # フィルタリング（convolve, edge, adapt 等）
│   ├── leptonica-color/          # 色処理（colorspace, quantize, threshold 等）
│   ├── leptonica-region/         # 領域解析（conncomp, seedfill, watershed 等）
│   ├── leptonica-doc/            # ドキュメント処理（PDF生成補助）
│   ├── leptonica-recog/          # 文字認識・バーコード・デワープ・スキュー
│   └── leptonica-test/           # 回帰テストインフラ（RegParams, compare_* 等）
├── leptonica/                    # ファサードcrate（全crateのre-export）
└── reference/leptonica/          # C版 leptonica（手動clone）
```

### 依存関係（実際の構成）

```text
leptonica-recog → leptonica-morph, leptonica-transform,
                  leptonica-region, leptonica-color, leptonica-core
leptonica-morph, leptonica-transform,
leptonica-filter, leptonica-color  → leptonica-io, leptonica-core
leptonica-region                   → leptonica-core
leptonica-io                       → leptonica-core
leptonica-doc                      → leptonica-core
leptonica-test                     → leptonica-core, leptonica-io
leptonica (facade)                 → 全 crate
```

---

## 2. 実装状況（完了 2026-02-22）

| crate                      | 主な内容                                              | 状態           | 計画書                          |
| -------------------------- | ----------------------------------------------------- | -------------- | ------------------------------- |
| leptonica (src/core/)      | Pix/PixMut, Box/Boxa, Numa, Pta, Pixa, Colormap       | ✅ IMPLEMENTED | `100_core-full-porting.md`      |
| leptonica (src/io/)        | PNG/JPEG/TIFF/GIF/WebP/BMP/PNM/SPIX 読み書き          | ✅ IMPLEMENTED | `201_io-full-porting.md`        |
| leptonica (src/morph/)     | 膨張/収縮/開閉, DWA, thin, morph sequence             | ✅ IMPLEMENTED | `400_morph-full-porting.md`     |
| leptonica (src/transform/) | 回転, スケール, アフィン, bilinear, projective, shear | ✅ IMPLEMENTED | `300_transform-full-porting.md` |
| leptonica (src/filter/)    | 畳み込み, エッジ, adapt, bilateral, rank              | ✅ IMPLEMENTED | `500_filter-full-porting.md`    |
| leptonica (src/color/)     | 色空間変換, 量子化, threshold, coloring               | ✅ IMPLEMENTED | -                               |
| leptonica (src/region/)    | conncomp, seedfill, watershed, quadtree               | ✅ IMPLEMENTED | `700_region-full-porting.md`    |
| leptonica-doc              | PDF補助（最小限）                                     | ✅ 基本実装    | -                               |
| leptonica (src/recog/)     | OCR, barcode, dewarp, skew, baseline (Phase 1-13)     | ✅ IMPLEMENTED | `800_recog-full-porting.md`     |
| leptonica-test             | RegParams, compare_pix, compare_values                | ✅ IMPLEMENTED | -                               |
| leptonica                  | ファサード（全 crate re-export）                      | ✅ IMPLEMENTED | -                               |

---

## 3. フェーズ1: leptonica (src/core/) 詳細

### 3.1 PIX構造体

```rust
pub struct Pix {
    inner: Arc<PixData>,
}

struct PixData {
    width: u32,
    height: u32,
    depth: PixelDepth,      // 1, 2, 4, 8, 16, 32 bpp
    spp: u32,               // samples per pixel
    wpl: u32,               // words per line
    data: Vec<u32>,
    colormap: Option<PixColormap>,
    // ...
}
```

### 3.2 実装する構造体

| Leptonica          | Rust                            | 用途         |
| ------------------ | ------------------------------- | ------------ |
| PIX                | `Pix` / `PixMut`                | 画像データ   |
| BOX / BOXA / BOXAA | `Box` (Copy) / `Boxa` / `Boxaa` | 矩形領域     |
| PTA / PTAA         | `Pta` / `Ptaa`                  | 点配列       |
| NUMA / NUMAA       | `Numa` / `Numaa`                | 数値配列     |
| L_DNA / L_DNAA     | `Dna` / `Dnaa`                  | 倍精度配列   |
| SARRAY             | `Sarray`                        | 文字列配列   |
| PIXCMAP            | `PixColormap`                   | カラーマップ |

### 3.3 参照ファイル

- `reference/leptonica/src/pix_internal.h` - PIX構造体定義
- `reference/leptonica/src/pix1.c` - PIX生成・破棄
- `reference/leptonica/src/array_internal.h` - 配列構造体定義
- `reference/leptonica/src/boxbasic.c` - BOX操作

---

## 4. フェーズ2: leptonica (src/io/) 詳細

### 4.1 対応フォーマット

| フォーマット | 依存crate      | 優先度 |
| ------------ | -------------- | ------ |
| PNG          | `png`          | 高     |
| JPEG         | `jpeg-decoder` | 高     |
| BMP          | 組み込み実装   | 高     |
| PNM          | 組み込み実装   | 中     |
| TIFF         | `tiff`         | 中     |
| GIF          | `gif`          | 中     |
| WebP         | `webp` or FFI  | 低     |

### 4.2 API設計

```rust
// 読み込み
let pix = Pix::read("image.png")?;
let pix = Pix::read_mem(&bytes)?;

// 書き込み
pix.write("output.png")?;
pix.write_mem(ImageFormat::Png)?;
```

---

## 5. Rust設計方針

### 5.1 メモリ管理

| Leptonicaパターン   | Rust実装       |
| ------------------- | -------------- |
| refcount + clone    | `Arc<T>`       |
| copy                | `Clone` trait  |
| insert (所有権移動) | move semantics |

### 5.2 エラーハンドリング

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid dimensions: {width}x{height}")]
    InvalidDimension { width: u32, height: u32 },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    // ...
}

pub type Result<T> = std::result::Result<T, Error>;
```

### 5.3 Feature Flags

```toml
[features]
default = ["png", "jpeg"]
png = ["dep:png"]
jpeg = ["dep:jpeg-decoder"]
tiff = ["dep:tiff"]
all-formats = ["png", "jpeg", "tiff", "gif", "webp"]
```

### 5.4 image crateとの相互運用

```rust
impl From<&Pix> for image::DynamicImage { ... }
impl TryFrom<&image::DynamicImage> for Pix { ... }
```

---

## 6. 検証方法

1. **単体テスト**: 各関数のテスト（カバレッジ80%目標）
2. **リグレッションテスト**: Leptonica元実装との出力比較
3. **ベンチマーク**: `criterion`による性能測定

---

## 7. 実装完了状況（2026-02-22）

全フェーズの実装が完了した。各 crate の詳細は以下の計画書を参照:

| crate                      | 計画書                                     | 状態                     |
| -------------------------- | ------------------------------------------ | ------------------------ |
| leptonica (src/recog/)     | `docs/plans/800_recog-full-porting.md`     | IMPLEMENTED (Phase 1-13) |
| leptonica (src/transform/) | `docs/plans/300_transform-full-porting.md` | IMPLEMENTED              |
| leptonica (src/morph/)     | `docs/plans/400_morph-full-porting.md`     | IMPLEMENTED              |
| leptonica (src/filter/)    | `docs/plans/500_filter-full-porting.md`    | IMPLEMENTED              |
| leptonica (src/region/)    | `docs/plans/700_region-full-porting.md`    | IMPLEMENTED              |
| leptonica (src/io/)        | `docs/plans/201_io-full-porting.md`        | IMPLEMENTED              |
| leptonica (src/core/)      | `docs/plans/100_core-full-porting.md`      | IMPLEMENTED              |
