# leptonica-rs 実装計画

## 概要

leptonica-rs の継続的な機能拡張計画。

- **Phase 1** ✅完了: OCR前処理機能（スキュー検出・補正、ベースライン検出、ページセグメンテーション）
- **Phase 2** ✅完了: 文字認識（Character Recognition）とJBIG2分類
- **Phase 3** 🔄実装中: TIFF I/O（leptonica-io拡張）

---

## Phase 2: 文字認識とJBIG2分類

Phase 2では以下を実装:

1. **文字認識（recog）**: テンプレートベースの文字認識システム
2. **JBIG2分類（jbclass）**: 連結成分のクラスタリングとテンプレート圧縮

## Phase 2 モジュール構成

```text
crates/leptonica-recog/src/
├── lib.rs               # モジュール宣言、pub use（拡張）
├── error.rs             # RecogError（拡張）
├── skew.rs              # ✅ Phase 1完了
├── baseline.rs          # ✅ Phase 1完了
├── pageseg.rs           # ✅ Phase 1完了
├── recog/               # 文字認識（新規）
│   ├── mod.rs           # モジュール宣言
│   ├── types.rs         # Recog, Rch, Rcha, Rdid構造体
│   ├── train.rs         # テンプレート学習
│   ├── ident.rs         # 文字識別
│   └── did.rs           # Document Image Decoding (Viterbi)
└── jbclass/             # JBIG2分類（新規）
    ├── mod.rs           # モジュール宣言
    ├── types.rs         # JbClasser, JbData構造体
    └── classify.rs      # 分類処理
```

### Phase 1 モジュール構成（完了済み）

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

## 実装スコープ（Phase 1）✅完了

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
pub fn find_skew_and_deskew(pix: &Pix, options: &SkewDetectOptions)
    -> RecogResult<(Pix, SkewResult)>;

/// スキュー補正のみ
pub fn deskew(pix: &Pix, angle: f32) -> RecogResult<Pix>;

/// 範囲走査でスキュー角度を検出（内部関数）
fn find_skew_sweep(pix: &Pix, sweep_range: f32, sweep_delta: f32, reduction: u32)
    -> RecogResult<(f32, f64)>;

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
pub fn get_local_skew_angles(pix: &Pix, num_slices: u32, sweep_range: f32)
    -> RecogResult<Vec<f32>>;

/// ローカルスキュー補正
pub fn deskew_local(pix: &Pix, options: &BaselineOptions, skew_options: &SkewDetectOptions)
    -> RecogResult<Pix>;
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
| --- | --- |
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

leptonica-transform は現在90度単位の回転のみ対応しています。スキュー補正には任意角度回転が必要なため、以下の関数を追加します:

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
- `crates/leptonica-transform/src/rotate.rs` - 回転処理

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

## 見積もりテスト数（Phase 1）

- skew: 6-8テスト（角度検出、補正、境界条件）
- baseline: 4-6テスト（検出精度、ローカルスキュー）
- pageseg: 5-7テスト（領域分割、テキスト行抽出）
- **合計: 15-21テスト**

---

## Phase 2: 実装詳細

### 1. recog/types.rs - 文字認識の型定義

### 文字セット種別

```rust
/// 文字セット種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharsetType {
    #[default]
    Unknown = 0,
    ArabicNumerals = 1,      // 0-9
    LcRomanNumerals = 2,     // i,v,x,l,c,d,m
    UcRomanNumerals = 3,     // I,V,X,L,C,D,M
    LcAlpha = 4,             // a-z
    UcAlpha = 5,             // A-Z
}

/// テンプレート使用モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TemplateUse {
    #[default]
    All = 0,        // 全テンプレートを使用
    Average = 1,    // 平均テンプレートを使用
}
```

### Recog構造体（文字認識器）

```rust
/// 文字認識器
pub struct Recog {
    // スケーリングパラメータ
    pub scale_w: i32,           // 水平スケール（0=スケールしない）
    pub scale_h: i32,           // 垂直スケール（0=スケールしない）
    pub line_w: i32,            // 線幅変換（0=スキップ）

    // テンプレート設定
    pub templ_use: TemplateUse,
    pub max_array_size: usize,
    pub set_size: usize,        // 文字クラス数

    // 識別パラメータ
    pub threshold: i32,         // 二値化閾値
    pub max_y_shift: i32,       // 垂直シフト許容値（0または1）

    // 文字セット
    pub charset_type: CharsetType,
    pub charset_size: usize,

    // 学習統計
    pub min_nopad: i32,
    pub num_samples: usize,

    // テンプレートサイズ情報（非スケール）
    pub minwidth_u: i32,
    pub maxwidth_u: i32,
    pub minheight_u: i32,
    pub maxheight_u: i32,

    // テンプレートサイズ情報（スケール後）
    pub minwidth: i32,
    pub maxwidth: i32,

    // 状態フラグ
    pub ave_done: bool,
    pub train_done: bool,

    // 分割パラメータ
    pub max_wh_ratio: f32,      // 分割用最大幅/高さ比
    pub max_ht_ratio: f32,      // テンプレート高さ比最大値
    pub min_split_w: i32,
    pub max_split_h: i32,

    // テキストマッピング（任意文字セット用）
    pub sa_text: Vec<String>,   // 文字列配列
    pub dna_tochar: Vec<f64>,   // インデックス→文字LUT

    // ルックアップテーブル
    centtab: Vec<i32>,          // 重心計算用
    sumtab: Vec<i32>,           // ピクセル合計用

    // テンプレート（非スケール）
    pub pixaa_u: Vec<Vec<Pix>>,     // クラスごとの全テンプレート
    pub ptaa_u: Vec<Vec<(f32, f32)>>, // 全テンプレートの重心
    pub naasum_u: Vec<Vec<i32>>,     // 全テンプレートの面積

    // テンプレート（スケール後）
    pub pixaa: Vec<Vec<Pix>>,
    pub ptaa: Vec<Vec<(f32, f32)>>,
    pub naasum: Vec<Vec<i32>>,

    // 平均テンプレート
    pub pixa_u: Vec<Pix>,       // 非スケール平均
    pub pta_u: Vec<(f32, f32)>, // 非スケール平均の重心
    pub nasum_u: Vec<i32>,      // 非スケール平均の面積

    pub pixa: Vec<Pix>,         // スケール後平均
    pub pta: Vec<(f32, f32)>,   // スケール後平均の重心
    pub nasum: Vec<i32>,        // スケール後平均の面積

    // デバッグ/作業用
    pub pixa_tr: Vec<Pix>,      // 全学習画像
}
```

### Rch/Rcha構造体（認識結果）

```rust
/// 単一文字の認識結果
#[derive(Debug, Clone)]
pub struct Rch {
    pub index: i32,         // 最良テンプレートのインデックス
    pub score: f32,         // 相関スコア
    pub text: String,       // 認識文字列
    pub sample: i32,        // サンプルインデックス
    pub xloc: i32,          // x位置
    pub yloc: i32,          // y位置
    pub width: i32,         // 幅
}

/// 複数文字の認識結果配列
#[derive(Debug, Clone, Default)]
pub struct Rcha {
    pub indices: Vec<i32>,
    pub scores: Vec<f32>,
    pub texts: Vec<String>,
    pub samples: Vec<i32>,
    pub xlocs: Vec<i32>,
    pub ylocs: Vec<i32>,
    pub widths: Vec<i32>,
}
```

### Rdid構造体（Document Image Decoding用）

```rust
/// Document Image Decoding用データ
pub struct Rdid {
    pub pixs: Pix,              // デコード対象画像
    pub counta: Vec<Vec<i32>>,  // 各テンプレートのカウント配列
    pub delya: Vec<Vec<i32>>,   // 最良y-shift配列
    pub narray: usize,          // 平均テンプレート数
    pub size: usize,            // カウント配列サイズ（pixsの幅）
    pub setwidth: Vec<i32>,     // 各テンプレートのセット幅
    pub nasum: Vec<i32>,        // 列ごとのピクセル数
    pub namoment: Vec<i32>,     // 列ごとの一次モーメント
    pub fullarrays: bool,       // 完全配列作成済みフラグ

    // Viterbiパラメータ
    pub beta: Vec<f32>,         // テンプレートFG項の係数
    pub gamma: Vec<f32>,        // bit-and項の係数
    pub trellisscore: Vec<f32>, // トレリススコア
    pub trellistempl: Vec<i32>, // トレリステンプレート（バックトラック用）

    // 最良パス結果
    pub natempl: Vec<i32>,      // 最良パステンプレートインデックス
    pub naxloc: Vec<i32>,       // 最良パスx位置
    pub nadely: Vec<i32>,       // 最良パスy位置
    pub nawidth: Vec<i32>,      // 最良パステンプレート幅
    pub boxa: Vec<leptonica_core::Rect>, // 分割結果
    pub nascore: Vec<f32>,      // 相関スコア
}
```

## 2. recog/train.rs - テンプレート学習

```rust
/// 空のRecogを作成
pub fn create(
    scale_w: i32,
    scale_h: i32,
    line_w: i32,
    threshold: i32,
    max_y_shift: i32,
) -> RecogResult<Recog>;

/// Pixaからラベル付きRecogを作成
pub fn create_from_pixa(
    pixa: &[Pix],
    scale_w: i32,
    scale_h: i32,
    line_w: i32,
    threshold: i32,
    max_y_shift: i32,
) -> RecogResult<Recog>;

impl Recog {
    /// ラベル付きサンプルを追加
    pub fn train_labeled(&mut self, pix: &Pix, label: &str) -> RecogResult<()>;

    /// サンプルを追加（内部用）
    pub fn add_sample(&mut self, pix: &Pix, class: usize) -> RecogResult<()>;

    /// 平均テンプレートを計算
    pub fn average_samples(&mut self) -> RecogResult<()>;

    /// 学習を完了
    pub fn finish_training(&mut self) -> RecogResult<()>;

    /// 外れ値を除去（方法1: 閾値ベース）
    pub fn remove_outliers1(&mut self, min_score: f32) -> RecogResult<usize>;

    /// 外れ値を除去（方法2: 他クラスとの比較）
    pub fn remove_outliers2(&mut self) -> RecogResult<usize>;
}
```

## 3. recog/ident.rs - 文字識別

```rust
impl Recog {
    /// 単一文字を識別
    pub fn identify_pix(&self, pix: &Pix) -> RecogResult<Rch>;

    /// 複数文字を識別（接触文字の分割含む）
    pub fn identify_multiple(&self, pix: &Pix) -> RecogResult<Rcha>;

    /// Pixa内の全画像を識別
    pub fn identify_pixa(&self, pixa: &[Pix]) -> RecogResult<Vec<Rcha>>;

    /// 相関スコアを計算（最良行）
    pub fn correlation_best_row(&self, pix: &Pix) -> RecogResult<(i32, f32)>;

    /// 相関スコアを計算（最良文字）
    pub fn correlation_best_char(&self, pix: &Pix) -> RecogResult<Rch>;

    /// 接触文字を分割
    pub fn split_into_characters(&self, pix: &Pix) -> RecogResult<Vec<Pix>>;
}

/// 2画像間の相関スコアを計算
pub fn compute_correlation_score(
    pix1: &Pix,
    pix2: &Pix,
    tab: &[i32],
) -> RecogResult<f32>;

/// 重心揃えで相関を計算
pub fn compute_correlation_with_centering(
    pix1: &Pix,
    pix2: &Pix,
    cx1: f32, cy1: f32,
    cx2: f32, cy2: f32,
    max_y_shift: i32,
) -> RecogResult<(f32, i32)>;
```

## 4. recog/did.rs - Document Image Decoding

```rust
impl Recog {
    /// DIDでテキスト行をデコード
    pub fn decode(&self, pix: &Pix) -> RecogResult<Rcha>;

    /// DID状態を初期化
    pub fn create_did(&mut self, pix: &Pix) -> RecogResult<()>;

    /// DID状態を破棄
    pub fn destroy_did(&mut self);

    /// Viterbiアルゴリズムを実行
    pub fn run_viterbi(&mut self) -> RecogResult<()>;

    /// DID結果を再スコアリング
    pub fn rescore_did_result(&mut self) -> RecogResult<()>;
}

/// Viterbiパスから結果を抽出
fn extract_viterbi_result(rdid: &Rdid) -> RecogResult<Rcha>;
```

## 5. jbclass/types.rs - JBIG2分類の型定義

```rust
/// 分類方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JbMethod {
    RankHaus = 0,    // ランクハウスドルフ距離
    Correlation = 1, // 相関ベース
}

/// 成分種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JbComponent {
    ConnComps = 0,   // 連結成分
    Characters = 1,  // 文字
    Words = 2,       // 単語
}

/// JBIG2分類器
pub struct JbClasser {
    pub files: Vec<String>,         // 入力ファイル名
    pub method: JbMethod,
    pub components: JbComponent,
    pub max_width: i32,
    pub max_height: i32,
    pub npages: usize,
    pub base_index: usize,
    pub nacomps: Vec<usize>,        // ページごとの成分数

    // ハウスドルフパラメータ
    pub size_haus: i32,             // 構造要素サイズ
    pub rank_haus: f32,             // ランク値

    // 相関パラメータ
    pub thresh: f32,                // 相関閾値
    pub weight_factor: f32,         // 重み係数

    // テンプレート情報
    pub naarea: Vec<i32>,           // 面積配列
    pub w: i32,                     // 最大ページ幅
    pub h: i32,                     // 最大ページ高さ
    pub nclass: usize,              // クラス数

    // テンプレート画像
    pub keep_pixaa: bool,
    pub pixaa: Vec<Vec<Pix>>,       // クラスごとのインスタンス
    pub pixat: Vec<Pix>,            // 境界付きテンプレート（非膨張）
    pub pixatd: Vec<Pix>,           // 境界付きテンプレート（膨張済み）

    // ハッシュテーブル
    pub dahash: std::collections::HashMap<(i32, i32), Vec<usize>>,

    // 統計情報
    pub nafgt: Vec<i32>,            // 非膨張テンプレートのFG面積
    pub ptac: Vec<(f32, f32)>,      // 全成分の重心
    pub ptact: Vec<(f32, f32)>,     // 全テンプレートの重心

    // 分類結果
    pub naclass: Vec<usize>,        // 成分→クラスマッピング
    pub napage: Vec<usize>,         // 成分→ページマッピング
    pub ptaul: Vec<(i32, i32)>,     // 左上座標
    pub ptall: Vec<(i32, i32)>,     // 左下座標
}

/// JBIG2圧縮データ
pub struct JbData {
    pub pix: Pix,                   // テンプレート合成画像
    pub npages: usize,
    pub w: i32,
    pub h: i32,
    pub nclass: usize,
    pub lattice_w: i32,             // ラティス幅
    pub lattice_h: i32,             // ラティス高さ
    pub naclass: Vec<usize>,
    pub napage: Vec<usize>,
    pub ptaul: Vec<(i32, i32)>,
}
```

## 6. jbclass/classify.rs - 分類処理

```rust
/// ランクハウスドルフ分類器を初期化
pub fn rank_haus_init(
    components: JbComponent,
    max_width: i32,
    max_height: i32,
    size_haus: i32,
    rank_haus: f32,
) -> RecogResult<JbClasser>;

/// 相関ベース分類器を初期化
pub fn correlation_init(
    components: JbComponent,
    max_width: i32,
    max_height: i32,
    thresh: f32,
    weight_factor: f32,
) -> RecogResult<JbClasser>;

impl JbClasser {
    /// ページを追加して分類
    pub fn add_page(&mut self, pix: &Pix) -> RecogResult<()>;

    /// 複数ページを追加
    pub fn add_pages(&mut self, pixs: &[Pix]) -> RecogResult<()>;

    /// 成分を抽出
    pub fn get_components(&self, pix: &Pix) -> RecogResult<(Vec<Pix>, Vec<leptonica_core::Rect>)>;

    /// ランクハウスドルフ分類
    pub fn classify_rank_haus(&mut self, pix: &Pix) -> RecogResult<usize>;

    /// 相関ベース分類
    pub fn classify_correlation(&mut self, pix: &Pix) -> RecogResult<usize>;

    /// JbDataを生成
    pub fn get_data(&self) -> RecogResult<JbData>;

    /// テンプレートから合成画像を生成
    pub fn templates_from_composites(&self) -> RecogResult<Vec<Pix>>;
}

impl JbData {
    /// ページを再構成
    pub fn render_page(&self, page: usize) -> RecogResult<Pix>;

    /// 全ページを再構成
    pub fn render_all(&self) -> RecogResult<Vec<Pix>>;
}

/// ハウスドルフ距離を計算
pub fn hausdorff_distance(
    pix1: &Pix,
    pix2: &Pix,
    size: i32,
    rank: f32,
) -> RecogResult<bool>;
```

## 修正対象ファイル（Phase 2）

| ファイル | 操作 |
| --- | --- |
| `crates/leptonica-recog/src/lib.rs` | 編集（モジュール追加） |
| `crates/leptonica-recog/src/error.rs` | 編集（エラー種別追加） |
| `crates/leptonica-recog/src/recog/mod.rs` | 新規作成 |
| `crates/leptonica-recog/src/recog/types.rs` | 新規作成 |
| `crates/leptonica-recog/src/recog/train.rs` | 新規作成 |
| `crates/leptonica-recog/src/recog/ident.rs` | 新規作成 |
| `crates/leptonica-recog/src/recog/did.rs` | 新規作成 |
| `crates/leptonica-recog/src/jbclass/mod.rs` | 新規作成 |
| `crates/leptonica-recog/src/jbclass/types.rs` | 新規作成 |
| `crates/leptonica-recog/src/jbclass/classify.rs` | 新規作成 |

## 依存関係（追加）

```toml
[dependencies]
# 既存
leptonica-core.workspace = true
leptonica-transform.workspace = true
leptonica-region.workspace = true
leptonica-morph.workspace = true
thiserror.workspace = true
```

## 参照ファイル（Phase 2）

- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/recog.h` - 構造体定義
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/recogbasic.c` - 基本操作
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/recogtrain.c` - 学習
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/recogident.c` - 識別
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/recogdid.c` - DID
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/jbclass.h` - JBIG2構造体
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/jbclass.c` - JBIG2分類

## 検証方法（Phase 2）

1. **ユニットテスト**
   - テンプレート登録と学習
   - 単一文字識別（数字0-9）
   - 接触文字の分割
   - Viterbiデコード
   - JBIG2分類と再構成

2. **ビルド確認**

   ```bash
   cargo build -p leptonica-recog
   cargo test -p leptonica-recog
   ```

## 見積もりテスト数（Phase 2）

- recog/types: 3-4テスト（構造体作成、バリデーション）
- recog/train: 5-7テスト（学習、平均化、外れ値除去）
- recog/ident: 6-8テスト（識別、相関、分割）
- recog/did: 4-6テスト（Viterbi、デコード）
- jbclass: 5-7テスト（分類、再構成）
- **合計: 23-32テスト**

---

## Phase 1 詳細（完了済み）

（省略 - Phase 1はスキュー検出、ベースライン検出、ページセグメンテーションを実装済み）

---

## Phase 3: TIFF I/O

### TIFF機能概要

leptonica-io クレートにTIFFフォーマットのサポートを追加する。

**目標:**

- 単一ページおよびマルチページTIFFの読み書き
- 複数の圧縮形式サポート（G4, LZW, ZIP等）
- 既存のI/Oパターンに準拠したAPI設計

### モジュール構成

```text
crates/leptonica-io/src/
├── lib.rs          # 編集（tiffモジュール追加）
├── format.rs       # 既存（TIFFマジックナンバー検出済み）
├── error.rs        # 既存
├── bmp.rs          # 既存
├── png.rs          # 既存（参考パターン）
├── jpeg.rs         # 既存
├── pnm.rs          # 既存
└── tiff.rs         # 新規作成
```

### 実装スコープ

#### 1. TiffCompression列挙型

```rust
/// TIFF圧縮形式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TiffCompression {
    #[default]
    None,       // 非圧縮
    G3,         // CCITT Group 3 (ファックス)
    G4,         // CCITT Group 4 (最も効率的なバイナリ圧縮)
    Rle,        // Run-Length Encoding
    PackBits,   // PackBits
    Lzw,        // LZW
    Zip,        // Adobe DEFLATE (ZIP)
}
```

#### 2. 主要関数

```rust
/// 単一ページTIFFを読み込み
pub fn read_tiff<R: Read + Seek>(reader: R) -> IoResult<Pix>;

/// 指定ページのTIFFを読み込み
pub fn read_tiff_page<R: Read + Seek>(reader: R, page: usize) -> IoResult<Pix>;

/// 単一ページTIFFを書き込み
pub fn write_tiff<W: Write + Seek>(
    pix: &Pix,
    writer: W,
    compression: TiffCompression,
) -> IoResult<()>;

/// マルチページTIFFの全ページを読み込み
pub fn read_tiff_multipage<R: Read + Seek>(reader: R) -> IoResult<Vec<Pix>>;

/// マルチページTIFFを書き込み
pub fn write_tiff_multipage<W: Write + Seek>(
    pages: &[&Pix],
    writer: W,
    compression: TiffCompression,
) -> IoResult<()>;

/// TIFFファイルのページ数を取得
pub fn tiff_page_count<R: Read + Seek>(reader: R) -> IoResult<usize>;

/// TIFFメタデータ（解像度）を取得
pub fn tiff_resolution<R: Read + Seek>(reader: R) -> IoResult<Option<(f32, f32)>>;
```

#### 3. lib.rs への統合

```rust
#[cfg(feature = "tiff-format")]
pub mod tiff;
#[cfg(feature = "tiff-format")]
pub use tiff::*;

// read_image_format() 内
#[cfg(feature = "tiff-format")]
ImageFormat::Tiff | ImageFormat::TiffG3 | ImageFormat::TiffG4 |
ImageFormat::TiffRle | ImageFormat::TiffPackbits |
ImageFormat::TiffLzw | ImageFormat::TiffZip => {
    tiff::read_tiff(reader)
}
```

### ビット深度サポート

| ビット深度      | 読み込み | 書き込み | 備考            |
| --------------- | -------- | -------- | --------------- |
| 1-bit           | ✅       | ✅       | G4圧縮推奨      |
| 8-bit グレー    | ✅       | ✅       | LZW/ZIP推奨     |
| 24-bit RGB      | ✅       | ✅       | LZW/ZIP推奨     |
| 32-bit RGBA     | ✅       | ✅       | LZW/ZIP推奨     |

### 圧縮形式マッピング

| TiffCompression | ImageFormat    | 用途               |
| --------------- | -------------- | ------------------ |
| None            | Tiff           | 非圧縮             |
| G3              | TiffG3         | ファックス互換     |
| G4              | TiffG4         | 1-bitバイナリ最適  |
| Rle             | TiffRle        | 単純なRLE          |
| PackBits        | TiffPackbits   | Mac互換            |
| Lzw             | TiffLzw        | 汎用圧縮           |
| Zip             | TiffZip        | 最新の汎用圧縮     |

### ファイル構成

| ファイル                           | 操作                   |
| ---------------------------------- | ---------------------- |
| `crates/leptonica-io/src/tiff.rs`  | 新規作成               |
| `crates/leptonica-io/src/lib.rs`   | 編集（モジュール追加） |
| `crates/leptonica-io/Cargo.toml`   | 確認（tiff依存既存）   |

### Tiff依存関係

```toml
# Cargo.toml（既存）
[dependencies]
tiff = { version = "0.11.2", optional = true }

[features]
tiff-format = ["tiff"]
all-formats = ["png-format", "jpeg-format", "pnm-format", "tiff-format"]
```

### 実装フェーズ

#### フェーズ1: 基本サポート（必須）

- [x] `read_tiff()` - 単一ページ読み込み
- [x] `write_tiff()` - 単一ページ書き込み（非圧縮、G4）
- [x] 1/8/24/32-bitサポート
- [x] lib.rs統合
- [x] 基本テスト

#### フェーズ2: 圧縮サポート

- [x] 追加圧縮形式（LZW, ZIP, PackBits, G3, RLE）
- [x] 読み込み時の圧縮自動検出
- [x] 各圧縮形式のテスト

#### フェーズ3: マルチページ

- [x] `read_tiff_multipage()`
- [x] `write_tiff_multipage()`
- [x] `tiff_page_count()`
- [x] `read_tiff_page()`

#### フェーズ4: メタデータ（オプション）

- [x] `tiff_resolution()` - DPI取得
- [x] 解像度設定での書き込み

### 参照実装

- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/tiffio.c` - C実装
- `/home/tagawa/github/leptonica-rs/crates/leptonica-io/src/png.rs` - パターン参考
- `/home/tagawa/github/leptonica-rs/crates/leptonica-core/src/pix/mod.rs` - ImageFormat

### Tiff検証方法

1. **ユニットテスト**
   - ラウンドトリップテスト（書き込み→読み込み）
   - 各ビット深度でのテスト
   - 各圧縮形式でのテスト
   - マルチページテスト

2. **ビルド確認**

   ```bash
   cargo build -p leptonica-io --features tiff-format
   cargo test -p leptonica-io --features tiff-format
   ```

3. **統合テスト**

   ```bash
   cargo build --workspace
   cargo test --workspace --features all-formats
   ```

## 見積もりテスト数

- 読み込みテスト: 4-6（ビット深度別）
- 書き込みテスト: 4-6（ビット深度別）
- 圧縮テスト: 6-8（形式別）
- マルチページテスト: 3-4
- **合計: 17-24テスト**

## 見積もり行数

- tiff.rs: 400-600行
- lib.rs変更: 20-30行
- **合計: 420-630行**
