# Dewarp Implementation Plan

## Overview

デワーピング（Dewarping）は、スキャン時に発生するページの曲がり等による歪みを補正する機能です。
本実装では、C版Leptonica dewarp1-4.c を参考に、単一ページのデワーピングに焦点を当てた
基本的なフレームワークを提供します。

## C版の機能分析

### dewarp1.c - 基本構造とシリアライゼーション

- `L_DEWARP` 構造体: 単一ページの歪み補正データ
- `L_DEWARPA` 構造体: 複数ページ管理（今回は対象外）
- `dewarpCreate()`: Dewarp構造体の作成
- `dewarpDestroy()`: 破棄

### dewarp2.c - 視差モデル構築

- `dewarpBuildPageModel()`: ページモデル構築のメイン関数
- `dewarpFindVertDisparity()`: 垂直視差配列の構築
- `dewarpFindHorizDisparity()`: 水平視差配列の構築
- `dewarpGetTextlineCenters()`: テキストライン中心点の取得

### dewarp3.c - 視差適用

- `dewarpaApplyDisparity()`: 視差マップを画像に適用
- `pixApplyVertDisparity()`: 垂直視差の適用
- `pixApplyHorizDisparity()`: 水平視差の適用
- `dewarpPopulateFullRes()`: フル解像度視差配列の生成

### dewarp4.c - 単一ページ処理

- `dewarpSinglePage()`: 単一ページのデワーピング（最上位API）
- `dewarpSinglePageInit()`: 初期化
- `dewarpSinglePageRun()`: 実行

## Rust実装計画

### Phase 1: 型定義とデータ構造

```rust
/// デワーピングのオプション設定
pub struct DewarpOptions {
    /// サンプリング間隔（デフォルト: 30）
    pub sampling: u32,
    /// 解像度削減係数（1または2）
    pub reduction_factor: u32,
    /// 最小ライン数（デフォルト: 15）
    pub min_lines: u32,
    /// 両方向（垂直・水平）の補正を使用
    pub use_both: bool,
}

/// 単一ページのデワーピングデータ
pub struct Dewarp {
    /// ページ番号
    page_number: u32,
    /// 元画像の幅
    width: u32,
    /// 元画像の高さ
    height: u32,
    /// サンプリングされた垂直視差配列
    sampled_v_disparity: Option<FPix>,
    /// サンプリングされた水平視差配列
    sampled_h_disparity: Option<FPix>,
    /// フル解像度垂直視差配列
    full_v_disparity: Option<FPix>,
    /// フル解像度水平視差配列
    full_h_disparity: Option<FPix>,
    /// 設定
    options: DewarpOptions,
    /// 垂直モデル構築成功フラグ
    v_success: bool,
    /// 水平モデル構築成功フラグ
    h_success: bool,
}

/// ライン（テキスト行）モデル
pub struct LineModel {
    /// 各ラインの中点y座標
    mid_ys: Vec<f32>,
    /// 各ラインの曲率
    curvatures: Vec<f32>,
    /// 最小曲率（マイクロ単位）
    min_curvature: i32,
    /// 最大曲率（マイクロ単位）
    max_curvature: i32,
}
```

### Phase 2: テキストライン検出

```rust
/// テキストラインの中心点を取得
fn find_textline_centers(pix: &Pix) -> RecogResult<Vec<Vec<(f32, f32)>>> {
    // 1. モルフォロジー処理でテキストライン領域を固化
    // 2. 連結成分を抽出
    // 3. 各成分の垂直方向の重心を計算
}

/// 短いラインを除去
fn remove_short_lines(
    lines: Vec<Vec<(f32, f32)>>,
    min_fraction: f32,
) -> Vec<Vec<(f32, f32)>> {
    // 最長ラインの fraction 以下のラインを除去
}
```

### Phase 3: 垂直視差モデル構築

```rust
/// 垂直視差を計算
fn build_vertical_disparity(
    dewarp: &mut Dewarp,
    textline_centers: &[Vec<(f32, f32)>],
) -> RecogResult<()> {
    // 1. 各ラインに対して二次フィッティング
    // 2. 均一サンプリング
    // 3. 垂直方向の二次フィッティング
    // 4. サンプル視差配列の生成
}
```

### Phase 4: 水平視差モデル構築

```rust
/// 水平視差を計算
fn build_horizontal_disparity(
    dewarp: &mut Dewarp,
    textline_centers: &[Vec<(f32, f32)>],
) -> RecogResult<()> {
    // 1. ラインの左右端点を取得
    // 2. 線形/二次フィッティング
    // 3. 水平視差配列の生成
}
```

### Phase 5: 視差適用

```rust
/// フル解像度視差配列を生成
fn populate_full_resolution(
    dewarp: &mut Dewarp,
    target_width: u32,
    target_height: u32,
) -> RecogResult<()> {
    // サンプル配列からフル解像度配列を補間で生成
}

/// 垂直視差を適用
fn apply_vertical_disparity(
    pix: &Pix,
    v_disparity: &FPix,
    gray_in: u8,
) -> RecogResult<Pix> {
    // 各ピクセルに対して視差マップに基づいて再マッピング
}

/// 水平視差を適用
fn apply_horizontal_disparity(
    pix: &Pix,
    h_disparity: &FPix,
    gray_in: u8,
) -> RecogResult<Pix> {
    // 水平方向の再マッピング
}
```

### Phase 6: 高レベルAPI

```rust
/// 単一ページをデワーピング
pub fn dewarp_single_page(
    pix: &Pix,
    options: &DewarpOptions,
) -> RecogResult<DewarpResult> {
    // 1. 二値化
    // 2. テキストライン検出
    // 3. モデル構築
    // 4. 視差適用
}

/// デワーピング結果
pub struct DewarpResult {
    /// 補正後の画像
    pub pix: Pix,
    /// 使用したDewarpデータ
    pub dewarp: Dewarp,
    /// 垂直補正が成功したか
    pub v_success: bool,
    /// 水平補正が成功したか
    pub h_success: bool,
}
```

## 実装優先度

1. **必須**: 型定義、`DewarpOptions`、`Dewarp`構造体
2. **必須**: テキストライン中心点検出（簡略版）
3. **必須**: 垂直視差モデル構築（基本版）
4. **必須**: 視差適用（`apply_vertical_disparity`）
5. **オプション**: 水平視差モデル
6. **オプション**: フル解像度補間

## ファイル構成

```text
crates/leptonica-recog/src/
├── dewarp/
│   ├── mod.rs          # モジュール定義とre-export
│   ├── types.rs        # Dewarp, DewarpOptions, DewarpResult
│   ├── textline.rs     # テキストライン検出
│   ├── model.rs        # 視差モデル構築
│   └── apply.rs        # 視差適用
└── lib.rs              # dewarpモジュールの追加
```

## 依存関係

- `leptonica-core`: `Pix`, `FPix`, `Pta`
- `leptonica-morph`: モルフォロジー処理（テキストライン検出用）
- `leptonica-region`: 連結成分（テキストライン検出用）

## テスト計画

1. **単体テスト**:
   - `DewarpOptions` のデフォルト値とバリデーション
   - `Dewarp` 構造体の作成と操作
   - 視差配列のスケーリングと補間

2. **統合テスト**:
   - 水平なテキスト画像（補正不要のケース）
   - 軽度の曲がりがある画像
   - 空の画像やテキストが少ない画像（エラー処理）

## 質問

1. **Ptaaの利用**: C版ではPTAA（点配列の配列）を多用していますが、Rustでは
   `Vec<Vec<(f32, f32)>>` で代替可能です。既存のPtaa実装を使用すべきですか？

2. **モルフォロジー依存**: テキストライン検出にはモルフォロジー処理が必要ですが、
   leptonica-morphの既存機能で十分ですか？特に `pixMorphSequence` のような
   シーケンス処理が必要な場合はどうしますか？

3. **精度 vs 簡潔さ**: C版は非常に複雑な処理を行っていますが、基本的なケースに
   対応する簡略版で十分ですか？

## 進捗状況

- [x] Phase 1: 型定義
- [x] Phase 2: テキストライン検出
- [x] Phase 3: 垂直視差モデル
- [x] Phase 4: 水平視差モデル
- [x] Phase 5: 視差適用
- [x] Phase 6: 高レベルAPI
- [x] テスト作成
- [x] ドキュメント整備
