# leptonica-recog 実装計画

## 概要

Phase 8: leptonica-recog クレートの実装（Phase 1）。OCR前処理機能としてスキュー検出・補正とページセグメンテーションを提供。

## モジュール構成

```text
crates/leptonica-recog/
├── Cargo.toml
└── src/
    ├── lib.rs           # モジュール宣言、pub use
    ├── error.rs         # RecogError, RecogResult
    ├── skew.rs          # スキュー検出・補正
    ├── baseline.rs      # ベースライン検出
    └── pageseg.rs       # ページセグメンテーション
```

## 実装スコープ（Phase 1）

### 1. error.rs - エラー型定義

```rust
#[derive(Debug, Error)]
pub enum RecogError {
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),
    #[error("unsupported depth: expected {expected}, got {actual}")]
    UnsupportedDepth { expected: &'static str, actual: u32 },
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("skew detection failed: {0}")]
    SkewDetectionFailed(String),
    #[error("segmentation error: {0}")]
    SegmentationError(String),
}
pub type RecogResult<T> = Result<T, RecogError>;
```

### 2. skew.rs - スキュー検出・補正

**主要な型:**

```rust
pub struct SkewDetectOptions {
    pub sweep_range: f32,       // 走査範囲（度）、デフォルト: 7.0
    pub sweep_delta: f32,       // 走査ステップ、デフォルト: 1.0
    pub min_bs_delta: f32,      // 二分探索精度、デフォルト: 0.01
    pub sweep_reduction: u32,   // 走査用縮小率、デフォルト: 4
    pub bs_reduction: u32,      // 二分探索用縮小率、デフォルト: 2
}

pub struct SkewResult {
    pub angle: f32,             // 検出角度（度）
    pub confidence: f32,        // 信頼度 (0.0-1.0)
}
```

**主要な関数:**

```rust
/// スキュー角度を検出
pub fn find_skew(pix: &Pix, options: &SkewDetectOptions) -> RecogResult<SkewResult>;

/// スキュー検出と補正を同時実行
pub fn find_skew_and_deskew(pix: &Pix, options: &SkewDetectOptions) -> RecogResult<(Pix, SkewResult)>;

/// スキュー補正のみ
pub fn deskew(pix: &Pix, angle: f32) -> RecogResult<Pix>;

/// 範囲走査でスキュー角度を検出（内部関数）
fn find_skew_sweep(pix: &Pix, sweep_range: f32, sweep_delta: f32, reduction: u32) -> RecogResult<(f32, f64)>;

/// 差分二乗和スコア計算（内部関数）
fn compute_differential_score(pix: &Pix, angle: f32, reduction: u32) -> RecogResult<f64>;
```

### 3. baseline.rs - ベースライン検出

**主要な型:**

```rust
pub struct BaselineOptions {
    pub min_block_width: u32,   // 最小テキストブロック幅、デフォルト: 80
    pub peak_threshold: u32,    // ピーク判定閾値、デフォルト: 80
    pub num_slices: u32,        // ローカルスキュー分割数、デフォルト: 10
}

pub struct BaselineResult {
    pub baselines: Vec<i32>,    // y座標配列
    pub endpoints: Option<Vec<(i32, i32, i32, i32)>>,  // (x1, y1, x2, y2)
}
```

**主要な関数:**

```rust
/// ベースラインを検出
pub fn find_baselines(pix: &Pix, options: &BaselineOptions) -> RecogResult<BaselineResult>;

/// ローカルスキュー角度を計算
pub fn get_local_skew_angles(pix: &Pix, num_slices: u32, sweep_range: f32) -> RecogResult<Vec<f32>>;

/// ローカルスキュー補正
pub fn deskew_local(pix: &Pix, options: &BaselineOptions, skew_options: &SkewDetectOptions) -> RecogResult<Pix>;
```

### 4. pageseg.rs - ページセグメンテーション

**主要な型:**

```rust
pub struct PageSegOptions {
    pub min_width: u32,         // 最小幅、デフォルト: 100
    pub min_height: u32,        // 最小高さ、デフォルト: 100
}

pub struct SegmentationResult {
    pub halftone_mask: Option<Pix>,
    pub textline_mask: Pix,
    pub textblock_mask: Pix,
}
```

**主要な関数:**

```rust
/// ページを3領域に分割（ハーフトーン、テキスト行、テキストブロック）
pub fn segment_regions(pix: &Pix, options: &PageSegOptions) -> RecogResult<SegmentationResult>;

/// テキスト行マスクを生成
pub fn generate_textline_mask(pix: &Pix) -> RecogResult<(Pix, Pix)>;

/// テキストブロックマスクを生成
pub fn generate_textblock_mask(textline_mask: &Pix, vws: &Pix) -> RecogResult<Pix>;

/// テキスト行を個別に抽出
pub fn extract_textlines(pix: &Pix) -> RecogResult<Vec<Pix>>;

/// テキスト/画像判定
pub fn is_text_region(pix: &Pix) -> RecogResult<bool>;
```

## アルゴリズム概要

### スキュー検出

1. **粗い走査（Sweep）**: ±sweep_range度の範囲を sweep_delta刻みで走査
2. **スコア計算**: 各角度で行ごとの差分二乗和を計算（テキスト行が水平なほど高スコア）
3. **二分探索（Binary Search）**: 最良角度付近を min_bs_delta 精度で絞り込み
4. **縮小処理**: sweep_reduction, bs_reduction で計算量削減

### ベースライン検出

1. **水平投影**: 行ごとのピクセル数をカウント
2. **ピーク検出**: 投影ヒストグラムからテキスト行位置を特定
3. **エンドポイント**: 各行の左端・右端を検出
4. **ローカルスキュー**: 画像をスライスに分割し、各スライスのスキューを個別補正

### ページセグメンテーション

1. **ハーフトーン検出**: 規則的なスクリーンパターンを検出
2. **垂直空白検出**: テキスト行間の空白領域を特定
3. **テキスト行マスク**: モルフォロジー操作で行領域をマスク化
4. **テキストブロック**: テキスト行を統合してブロック化

## 修正対象ファイル

| ファイル | 操作 |
|---------|------|
| `crates/leptonica-transform/src/rotate.rs` | 編集（任意角度回転追加） |
| `crates/leptonica-transform/src/lib.rs` | 編集（新関数エクスポート） |
| `crates/leptonica-recog/Cargo.toml` | 編集（依存関係追加） |
| `crates/leptonica-recog/src/lib.rs` | 書き換え |
| `crates/leptonica-recog/src/error.rs` | 新規作成 |
| `crates/leptonica-recog/src/skew.rs` | 新規作成 |
| `crates/leptonica-recog/src/baseline.rs` | 新規作成 |
| `crates/leptonica-recog/src/pageseg.rs` | 新規作成 |

## 依存関係

```toml
[dependencies]
leptonica-core.workspace = true
leptonica-transform.workspace = true  # 回転処理
leptonica-region.workspace = true     # 連結成分分析
leptonica-morph.workspace = true      # 形態学操作
thiserror.workspace = true
```

## 前提条件: leptonica-transform への追加

leptonica-transform には現在90度単位の回転のみ実装されています。スキュー補正には任意角度回転が必要なため、leptonica-recog の実装前に以下の関数を追加します:

**追加する関数（rotate.rs）:**

```rust
/// 任意角度で画像を回転（度単位）
pub fn rotate_by_angle(pix: &Pix, angle: f32) -> TransformResult<Pix>;

/// 任意角度で画像を回転（ラジアン単位）
pub fn rotate_by_radians(pix: &Pix, radians: f32) -> TransformResult<Pix>;

/// 指定角度で画像を回転（回転中心とパディングオプション付き）
pub fn rotate_by_angle_with_options(
    pix: &Pix,
    angle: f32,
    fill_value: u32
) -> TransformResult<Pix>;
```

**アルゴリズム:**
- バイリニア補間またはニアレストネイバー補間
- 回転行列: [cos(θ), -sin(θ); sin(θ), cos(θ)]
- 逆変換で出力ピクセルから入力座標を計算

## 参照ファイル

- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/skew.c` - スキュー検出
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/baseline.c` - ベースライン
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/pageseg.c` - ページセグメンテーション
- `/home/tagawa/github/leptonica-rs/crates/leptonica-transform/src/rotate.rs` - 回転処理

## 検証方法

1. **ユニットテスト**
   - スキュー角度検出精度（±0.1度以内）
   - 0度、±5度、±10度の画像でテスト
   - ベースライン検出数の正確性
   - テキスト行抽出の完全性

2. **ビルド確認**
   ```bash
   cargo build -p leptonica-recog
   cargo test -p leptonica-recog
   ```

## 見積もりテスト数

- skew: 6-8テスト（角度検出、補正、境界条件）
- baseline: 4-6テスト（検出精度、ローカルスキュー）
- pageseg: 5-7テスト（領域分割、テキスト行抽出）
- **合計: 15-21テスト**
