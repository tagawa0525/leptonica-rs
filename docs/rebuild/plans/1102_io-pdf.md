# PDF出力機能 実装計画

## 概要

画像をPDF形式で出力する機能を実装する。C版leptonicaの`pdfio1.c`, `pdfio2.c`を参考に、`pdf-writer`クレートを使用してRust実装を行う。

## 参照情報

### C版の主要関数

- `pixWriteMemPdf()` - 画像をPDFバイト列として出力（最もシンプルなインターフェース）
- `pixWriteStreamPdf()` - 画像をストリームにPDF出力
- `pixConvertToPdf()` - 画像をPDFファイルに変換
- `pixConvertToPdfData()` - 画像をPDFデータに変換（内部関数）

### C版のエンコーディング戦略

- 1bpp画像: G4エンコード
- 8bpp（カラーマップなし）、32bpp: JPEGエンコード
- その他: FLATEエンコード

### 既存パターン（png.rsより）

- `read_png()` / `write_png()` の形式で読み書き関数を定義
- `IoError` / `IoResult` でエラー処理
- feature gateで制御

## 設計

### 1. Feature Gate

```toml
# Cargo.toml (leptonica-io)
[features]
pdf-format = ["pdf-writer"]

# workspace Cargo.toml
pdf-writer = "0.14"
```

### 2. モジュール構成

```text
crates/leptonica-io/
  src/
    pdf.rs          # PDF出力実装
    lib.rs          # モジュール追加
```

### 3. 公開API

```rust
// pdf.rs

/// PDF圧縮方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfCompression {
    /// 自動選択（1bpp: CCITT G4, 8/32bpp: DCT, その他: Flate）
    #[default]
    Auto,
    /// Flate（Deflate）圧縮
    Flate,
    /// DCT（JPEG）圧縮
    Dct,
    /// CCITT Group 4（1bppのみ）
    CcittG4,
}

/// PDF出力オプション
#[derive(Debug, Clone)]
pub struct PdfOptions {
    /// 圧縮方式
    pub compression: PdfCompression,
    /// JPEG品質（1-100、0でデフォルト75）
    pub quality: u8,
    /// 解像度（ppi、0で画像のres値を使用、それもなければ300）
    pub resolution: u32,
    /// ドキュメントタイトル
    pub title: Option<String>,
}

/// 画像をPDFバイト列として出力
pub fn write_pdf_mem(pix: &Pix, options: &PdfOptions) -> IoResult<Vec<u8>>;

/// 画像をPDFとしてWriterに出力
pub fn write_pdf<W: Write>(pix: &Pix, writer: W, options: &PdfOptions) -> IoResult<()>;

/// 複数画像を1つのPDFに出力（1ページ1画像）
pub fn write_pdf_multi<W: Write>(
    images: &[&Pix],
    writer: W,
    options: &PdfOptions,
) -> IoResult<()>;
```

### 4. 内部実装

pdf-writerクレートを使用してPDFを生成：

1. **画像データの準備**
   - 圧縮方式に応じてピクセルデータを変換
   - Flate: 生データをdeflate圧縮
   - DCT: JPEG形式にエンコード（別途jpeg-encoderが必要かも→flate圧縮で代替可能）
   - CCITT G4: 1bpp画像をG4圧縮（複雑なため、当面Flateで代替）

2. **PDF構造の生成**

   ```text
   PDF構造:
   - Catalog
   - Pages
     - Page（各画像につき1ページ）
       - MediaBox（ページサイズ）
       - Resources
         - XObject（画像データ）
       - Contents（画像描画コマンド）
   ```

3. **座標系の変換**
   - PDFの座標系: 左下原点、ポイント単位（1pt = 1/72 inch）
   - 画像サイズ: pixels * 72 / resolution = points

### 5. 簡略化設計（初期実装）

複雑さを避けるため、初期実装では：

- **圧縮**: Flate（Deflate）のみをサポート
  - pdf-writerが直接サポートしている
  - すべてのピクセル深度に対応可能
  - G4やDCTは将来的に追加可能

- **カラー処理**:
  - 1bpp: DeviceGray（0=黒、1=白 or カラーマップ展開）
  - 8bpp: DeviceGray（カラーマップなし）または DeviceRGB（カラーマップあり→展開）
  - 32bpp: DeviceRGB

## 実装タスク

1. [x] workspace Cargo.tomlに`pdf-writer`依存を追加
2. [x] leptonica-io/Cargo.tomlにfeatureとdependencyを追加
3. [x] `pdf.rs`モジュールを作成
4. [x] `lib.rs`にモジュールを追加
5. [x] `write_pdf_mem()` / `write_pdf()` を実装
6. [x] `write_pdf_multi()` を実装
7. [x] `write_image_format()`にPDF対応を追加
8. [x] テストを作成
9. [x] cargo fmt && cargo clippyで品質チェック
10. [x] テスト実行確認

## 実装完了

すべてのタスクが完了しました。

## 質問

（なし）

## 備考

- 読み込み（PDF→画像）は複雑なため対象外
- 初期実装ではFlate圧縮のみ。JPEG圧縮やG4圧縮は将来的な拡張として検討
- `pdf-writer` 0.14は低レベルAPIのため、PDF仕様の理解が必要
