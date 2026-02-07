# Leptonica Rust移植 全体計画

## 概要

Leptonica画像処理ライブラリ（C言語、約240,000行）をRustに移植する。
機能群ごとにフェーズ分けし、段階的に実装可能な成果物を作成する。

### 決定事項

- **プロジェクト構成**: Workspace（複数crate）
- **image crate相互運用**: 必要（DynamicImage ↔ Pix変換）
- **認識・分類機能（フェーズ6）**: 実装範囲に含める

---

## 1. プロジェクト構成

### Workspace構成

```text
leptonica-rs/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── leptonica-core/           # 基本データ構造とコア機能
│   ├── leptonica-io/             # 画像I/O
│   ├── leptonica-morph/          # モルフォロジー処理
│   ├── leptonica-transform/      # 幾何変換
│   ├── leptonica-filter/         # フィルタリング
│   ├── leptonica-color/          # カラー処理
│   ├── leptonica-region/         # 領域処理
│   ├── leptonica-recog/          # 認識・分類（スキュー、デワープ、バーコード、ページセグ、JBIG2）
│   └── leptonica-test/           # テストインフラ
├── leptonica/                    # 統合crate（re-export）
└── examples/
```

### 依存関係

```text
leptonica-recog → leptonica-region → leptonica-filter → leptonica-color
    → leptonica-transform → leptonica-morph → leptonica-io → leptonica-core
```

---

## 2. 実装フェーズ

| フェーズ | crate | 内容 | 推定ファイル数 |
| --- | --- | --- | --- |
| 1 | leptonica-core | PIX、BOX、PTA、NUMA等の基本データ構造 | 25-30 |
| 2 | leptonica-io | PNG/JPEG/TIFF/GIF/WebP/BMP/PNM読み書き | 15-20 |
| 3 | leptonica-morph | 膨張、収縮、開閉演算、構造要素 | 15-18 |
| 3 | leptonica-transform | 回転、スケーリング、アフィン変換 | 15-17 |
| 4 | leptonica-filter | 畳み込み、エッジ検出、画像強調 | 10-12 |
| 4 | leptonica-color | 色空間変換、量子化、セグメンテーション | 10-13 |
| 5 | leptonica-region | 連結成分、シードフィル、分水嶺 | 12-15 |
| 6 | leptonica-recog | スキュー補正、デワープ、ページセグ、文字認識、JBIG2分類 | 30-40 |

---

## 3. フェーズ1: leptonica-core 詳細

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

| Leptonica | Rust | 用途 |
| --- | --- | --- |
| PIX | `Pix` / `PixMut` | 画像データ |
| BOX / BOXA / BOXAA | `Box` (Copy) / `Boxa` / `Boxaa` | 矩形領域 |
| PTA / PTAA | `Pta` / `Ptaa` | 点配列 |
| NUMA / NUMAA | `Numa` / `Numaa` | 数値配列 |
| L_DNA / L_DNAA | `Dna` / `Dnaa` | 倍精度配列 |
| SARRAY | `Sarray` | 文字列配列 |
| PIXCMAP | `PixColormap` | カラーマップ |

### 3.3 参照ファイル

- `reference/leptonica/src/pix_internal.h` - PIX構造体定義
- `reference/leptonica/src/pix1.c` - PIX生成・破棄
- `reference/leptonica/src/array_internal.h` - 配列構造体定義
- `reference/leptonica/src/boxbasic.c` - BOX操作

---

## 4. フェーズ2: leptonica-io 詳細

### 4.1 対応フォーマット

| フォーマット | 依存crate | 優先度 |
| --- | --- | --- |
| PNG | `png` | 高 |
| JPEG | `jpeg-decoder` | 高 |
| BMP | 組み込み実装 | 高 |
| PNM | 組み込み実装 | 中 |
| TIFF | `tiff` | 中 |
| GIF | `gif` | 中 |
| WebP | `webp` or FFI | 低 |

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

| Leptonicaパターン | Rust実装 |
| --- | --- |
| refcount + clone | `Arc<T>` |
| copy | `Clone` trait |
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
