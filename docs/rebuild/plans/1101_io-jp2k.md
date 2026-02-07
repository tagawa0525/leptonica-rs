# JP2K (JPEG 2000) I/O 実装計画

## 概要

JPEG 2000形式の画像読み書き機能を `leptonica-io` クレートに追加する。

## 背景調査

### C版Leptonica (jp2kio.c) の機能

C版では以下の機能を提供:

- `pixReadJp2k()` - ファイルからJPEG2000画像を読み込み
- `pixWriteJp2k()` - JPEG2000画像をファイルに書き込み
- `pixReadMemJp2k()` - メモリからの読み込み
- `pixWriteMemJp2k()` - メモリへの書き込み
- reduction (縮小倍率: 1, 2, 4, 8, 16) パラメータ
- box (部分領域抽出) パラメータ
- quality (SNR: 0=デフォルト34, 100=ロスレス)
- nlevels (解像度レベル: 6または7)

### Rustクレートの選択肢

| クレート | 種類 | 長所 | 短所 |
| --- | --- | --- | --- |
| hayro-jpeg2000 | 純Rust | メモリ安全、SIMD最適化 | デコードのみ |
| jpeg2k | ラッパー | 高機能、双方向 | C依存 |
| openjp2 | 純Rust | 外部依存なし | 低レベル |
| jp2k | ラッパー | MIT/Apache | C依存 |

**選択: `hayro-jpeg2000` (デコード) + `openjp2` (エンコード)**

理由:

1. hayro-jpeg2000は純粋Rustで安全、20,000+画像でテスト済み
2. openjp2もC依存なし (C2Rustポート)
3. 両方ともCライブラリ不要でビルドが容易
4. MIT/Apache または BSD-2-Clause ライセンス

## 実装設計

### ファイル構成

```text
crates/leptonica-io/
  src/
    jp2k.rs       # 新規: JP2K I/O実装
    lib.rs        # 変更: jp2kモジュール追加
    format.rs     # 変更: JP2K検出追加
  Cargo.toml      # 変更: jp2k feature追加
```

### API設計

```rust
// crates/leptonica-io/src/jp2k.rs

/// JPEG 2000 エンコードオプション
pub struct Jp2kOptions {
    /// 品質 (SNR): 0=デフォルト(34), 100=ロスレス
    pub quality: u8,
    /// 解像度レベル数: デフォルト6
    pub num_resolutions: u8,
}

/// JPEG 2000 画像を読み込む
pub fn read_jp2k<R: Read + Seek>(reader: R) -> IoResult<Pix>

/// JPEG 2000 画像を書き込む
pub fn write_jp2k<W: Write>(pix: &Pix, writer: W) -> IoResult<()>

/// オプション指定でJPEG 2000 画像を書き込む
pub fn write_jp2k_with_options<W: Write>(
    pix: &Pix,
    writer: W,
    options: &Jp2kOptions,
) -> IoResult<()>
```

### Feature Gate

```toml
# Cargo.toml
[features]
jp2k-format = ["hayro-jpeg2000", "openjp2"]

[dependencies]
hayro-jpeg2000 = { version = "0.3", optional = true }
openjp2 = { version = "0.6", optional = true }
```

### 実装詳細

#### 読み込み (read_jp2k) - hayro-jpeg2000使用

```rust
use hayro_jpeg2000::{Image, DecodeSettings};

// 1. バイトデータから Image を作成
let image = Image::new(&data, &DecodeSettings::default())?;

// 2. デコードしてBitmapを取得
let bitmap = image.decode()?;

// 3. ColorSpaceに応じてPixを作成
//    - Grayscale -> 8bpp
//    - RGB -> 32bpp (spp=3)
//    - CMYK -> RGB変換
```

#### 書き込み (write_jp2k) - openjp2使用

1. Pixの深度に応じて変換
   - 1/2/4/8bpp grayscale -> 8bpp
   - 16bpp -> 8bpp (上位8bit)
   - 32bpp -> RGB/RGBA
   - colormap -> RGB
2. openjp2の低レベルAPIを使用してエンコード
3. 出力ストリームに書き込み

注: openjp2のエンコードAPIが複雑な場合、初期実装ではデコードのみをサポートし、
エンコードは将来の拡張として検討する可能性あり。

### format.rs への追加

```rust
// JP2K magic numbers
// JP2 signature box: 00 00 00 0C 6A 50 20 20 (最初の12バイト中)
// または 0x6A502020 ('jP  ')
// J2K codestream: FF 4F FF 51

/// JPEG 2000 Part 1 (JP2) signature
pub const JP2_SIGNATURE: &[u8] = &[0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20];

/// JPEG 2000 codestream (J2K) signature
pub const J2K_SIGNATURE: &[u8] = &[0xFF, 0x4F, 0xFF, 0x51];
```

### lib.rs への追加

```rust
#[cfg(feature = "jp2k-format")]
pub mod jp2k;

// read_image_format内
#[cfg(feature = "jp2k-format")]
ImageFormat::Jp2 => jp2k::read_jp2k(reader),

// write_image_format内
#[cfg(feature = "jp2k-format")]
ImageFormat::Jp2 => jp2k::write_jp2k(pix, writer),
```

## テスト計画

1. **ラウンドトリップテスト**
   - RGB画像の読み書き
   - グレースケール画像の読み書き
   - RGBA画像の読み書き

2. **変換テスト**
   - 各深度からの変換確認

3. **オプションテスト**
   - quality設定の確認
   - ロスレスモードの確認

## 実装手順

1. [x] Cargo.tomlに依存関係追加
2. [x] format.rsにJP2Kマジックナンバー検出追加
3. [x] jp2k.rs新規作成
   - [x] read_jp2k実装
   - [ ] write_jp2k実装 (将来の拡張として保留)
4. [x] lib.rs更新
5. [x] テスト作成
6. [x] cargo fmt && cargo clippy
7. [x] 全テストパス確認

## 質問

1. **reduction (縮小読み込み) 機能は必要か?**
   - C版では reduction パラメータで1/2, 1/4...の縮小読み込みが可能
   - 初期実装ではスキップし、将来の拡張として検討する予定

2. **box (部分領域抽出) 機能は必要か?**
   - C版ではBoxで指定した領域のみデコード可能
   - 初期実装ではスキップし、将来の拡張として検討する予定

## 参考資料

- C版ソース: reference/leptonica/src/jp2kio.c
- jpeg2k crate: <https://crates.io/crates/jpeg2k>
- JPEG 2000 規格: ISO/IEC 15444
