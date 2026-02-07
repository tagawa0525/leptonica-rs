# Barcode Implementation Plan

## Overview

1次元バーコードの検出とデコードを行うモジュールを実装する。C版の `bardecode.c` と `readbarcode.c` を参考に、Rust らしい安全で表現力のあるAPIを提供する。

## C版の主要機能

### readbarcode.c (バーコード検出・前処理)

- `pixProcessBarcodes()` - トップレベルAPI（画像からバーコードを検出・デコード）
- `pixExtractBarcodes()` - バーコード領域の抽出
- `pixReadBarcodes()` - 抽出されたバーコードを読み取り
- `pixReadBarcodeWidths()` - バー幅を取得
- `pixLocateBarcodes()` - バーコード位置の検出
- `pixDeskewBarcode()` - バーコード領域のデスキュー
- `pixExtractBarcodeWidths1/2()` - バー幅抽出（2つの方法）
- `pixExtractBarcodeCrossings()` - 遷移点の抽出

### bardecode.c (デコード)

- `barcodeDispatchDecoder()` - フォーマット判定とデコードの振り分け
- `barcodeFindFormat()` - フォーマット自動判定
- `barcodeFormatIsSupported()` - サポート確認
- `barcodeVerifyFormat()` - フォーマット検証

### 対応フォーマット

1. Code 2 of 5 (`barcodeDecode2of5`)
2. Interleaved 2 of 5 (`barcodeDecodeI2of5`)
3. Code 93 (`barcodeDecode93`)
4. Code 39 (`barcodeDecode39`)
5. Codabar (`barcodeDecodeCodabar`)
6. UPC-A (`barcodeDecodeUpca`)
7. EAN-13 (`barcodeDecodeEan13`)

## Rust実装設計

### モジュール構成

```text
crates/leptonica-recog/src/barcode/
├── mod.rs          # モジュールルート、公開API
├── types.rs        # 型定義（BarcodeFormat, DecodeMethod, BarcodeResult等）
├── detect.rs       # バーコード検出・位置特定
├── decode.rs       # デコードディスパッチャー・フォーマット判定
├── formats/        # 各フォーマットのデコーダー
│   ├── mod.rs
│   ├── code2of5.rs
│   ├── codei2of5.rs
│   ├── code93.rs
│   ├── code39.rs
│   ├── codabar.rs
│   ├── upca.rs
│   └── ean13.rs
└── signal.rs       # 信号処理（バー幅定量化）
```

### 主要な型定義 (types.rs)

```rust
/// バーコードフォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarcodeFormat {
    Unknown,
    Any,        // 自動判定
    Code128,
    Ean8,
    Ean13,
    Code2of5,
    CodeI2of5,
    Code39,
    Code93,
    Codabar,
    UpcA,
}

/// バー幅抽出方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DecodeMethod {
    #[default]
    UseWidths,   // ヒストグラムベース
    UseWindows,  // ウィンドウベース
}

/// バーコード検出結果
#[derive(Debug)]
pub struct BarcodeResult {
    /// デコードされたデータ
    pub data: String,
    /// 検出されたフォーマット
    pub format: BarcodeFormat,
    /// バー幅文字列（デバッグ用）
    pub bar_widths: Option<String>,
    /// バウンディングボックス
    pub bbox: Option<PixBox>,
    /// 信頼度スコア
    pub confidence: f32,
}

/// バーコード処理オプション
#[derive(Debug, Clone)]
pub struct BarcodeOptions {
    /// 対象フォーマット（Anyで自動判定）
    pub format: BarcodeFormat,
    /// デコード方法
    pub method: DecodeMethod,
    /// デバッグ出力
    pub debug: bool,
}
```

### 公開API (mod.rs)

```rust
/// 画像からバーコードを検出・デコード
pub fn process_barcodes(
    pix: &Pix,
    options: &BarcodeOptions,
) -> RecogResult<Vec<BarcodeResult>>;

/// バーコード領域を検出
pub fn locate_barcodes(
    pix: &Pix,
    threshold: i32,
) -> RecogResult<(Boxa, Option<Pix>, Option<Pix>)>;

/// バーコード領域を抽出してデスキュー
pub fn extract_barcodes(
    pix: &Pix,
    debug: bool,
) -> RecogResult<Pixa>;

/// バー幅文字列をデコード
pub fn decode_barcode(
    bar_str: &str,
    format: BarcodeFormat,
) -> RecogResult<BarcodeResult>;

/// フォーマットがサポートされているか確認
pub fn is_format_supported(format: BarcodeFormat) -> bool;
```

### 信号処理 (signal.rs)

```rust
/// バー幅の定量化（ヒストグラムベース）
pub fn quantize_crossings_by_width(
    crossings: &[f32],
    bin_fract: f32,
) -> RecogResult<Vec<u8>>;

/// バー幅の定量化（ウィンドウベース）
pub fn quantize_crossings_by_window(
    crossings: &[f32],
    ratio: f32,
) -> RecogResult<(Vec<u8>, f32)>;

/// 遷移点の抽出
pub fn extract_crossings(
    pix: &Pix,
    threshold: f32,
) -> RecogResult<Vec<f32>>;
```

## 実装手順

### Phase 1: 基本構造

1. `barcode/types.rs` - 型定義
2. `barcode/mod.rs` - モジュール構造
3. エラー型追加 (`error.rs` に `BarcodeError` 追加)

### Phase 2: 検出機能

1. `barcode/detect.rs` - バーコード検出
   - `locate_barcodes()` - Sobel エッジ検出 + 形態学処理
   - `extract_barcodes()` - 抽出とデスキュー
   - `deskew_barcode()` - 個別バーコードのデスキュー

### Phase 3: 信号処理

1. `barcode/signal.rs` - バー幅抽出
   - `extract_crossings()` - 遷移点検出
   - `quantize_crossings_by_width()` - ヒストグラムベース
   - `quantize_crossings_by_window()` - ウィンドウベース

### Phase 4: デコーダー実装

1. `barcode/formats/` - 各フォーマットのデコーダー
   - シンボルテーブル定義
   - `barcode/decode.rs` - ディスパッチャー

### Phase 5: 統合・テスト

1. `process_barcodes()` 統合API
2. ユニットテスト
3. `lib.rs` への統合

## テスト計画

### ユニットテスト

- 各フォーマットのデコーダー（既知のバー幅文字列）
- フォーマット判定
- 信号処理（合成データ）

### 統合テスト

- 実際のバーコード画像でのE2Eテスト（テスト画像が利用可能な場合）

## 依存関係

- `leptonica-core`: Pix, Box, Boxa 等
- `leptonica-transform`: 回転、スケーリング
- `leptonica-morph`: エッジ検出、形態学処理
- `leptonica-region`: 連結成分分析

## 質問

現時点で質問はありません。

## 進捗

- [x] Phase 1: 基本構造
- [x] Phase 2: 検出機能
- [x] Phase 3: 信号処理
- [x] Phase 4: デコーダー実装
- [x] Phase 5: 統合・テスト

## 実装ノート

- 検出機能は簡易版として実装。完全な実装には leptonica-core のピクセル深度変換とエッジ検出が必要。
- Code 128 と EAN-8 は未実装（C版でもCode128は部分的）。
- 各フォーマットのデコーダーはC版の実装を忠実に移植。
- チェックディジット検証は警告のみ（デコードは継続）。
