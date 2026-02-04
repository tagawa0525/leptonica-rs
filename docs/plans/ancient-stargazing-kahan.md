# leptonica-region 実装計画

## 概要

Phase 7: leptonica-regionクレートの実装。連結成分分析、シードフィル、分水嶺アルゴリズムなどの領域処理機能を提供。

## モジュール構成

```text
crates/leptonica-region/
├── Cargo.toml
└── src/
    ├── lib.rs           # モジュール宣言、pub use
    ├── error.rs         # RegionError, RegionResult
    ├── conncomp.rs      # 連結成分分析 (Connected Components)
    ├── seedfill.rs      # シードフィル操作
    ├── watershed.rs     # 分水嶺セグメンテーション
    └── label.rs         # ピクセルラベリング
```

## 実装順序

### Phase 1: 基礎（高優先度）

1. **error.rs** - エラー型定義
   - `RegionError` enum（UnsupportedDepth, InvalidSeed, SegmentationError等）
   - `RegionResult<T>` 型エイリアス

2. **conncomp.rs（基本）**
   - `ConnectedComponents` 構造体
   - `find_connected_components` - 4/8連結の成分検出
   - `label_connected_components` - 連結成分にラベル付け
   - `ConnectivityType` enum (FourWay, EightWay)

3. **label.rs**
   - `pix_label_connected_components` - 画像全体のラベリング
   - `pix_count_components` - 成分数カウント
   - `pix_get_component_bounds` - 各成分のバウンディングボックス

### Phase 2: 実用機能（中優先度）

1. **seedfill.rs**
   - `SeedFillOptions` 構造体
   - `seedfill_binary` - バイナリ画像のシードフィル
   - `seedfill_gray` - グレースケールのシードフィル
   - `floodfill` - フラッドフィル（塗りつぶし）

2. **conncomp.rs（拡張）**
   - `extract_component` - 単一成分の抽出
   - `filter_components_by_size` - サイズによる成分フィルタリング
   - `pix_component_area_transform` - 面積変換

### Phase 3: 高度な機能（低優先度）

1. **watershed.rs**
   - `WatershedOptions` 構造体
   - `watershed_segmentation` - 分水嶺セグメンテーション
   - `find_basins` - 流域の検出
   - `compute_gradient` - 勾配計算（エッジ情報）

## 主要な型・関数シグネチャ

### error.rs

```rust
#[derive(Debug, Error)]
pub enum RegionError {
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),
    #[error("unsupported depth: expected {expected}, got {actual}")]
    UnsupportedDepth { expected: &'static str, actual: u32 },
    #[error("invalid seed position: ({x}, {y})")]
    InvalidSeed { x: u32, y: u32 },
    #[error("segmentation error: {0}")]
    SegmentationError(String),
}
pub type RegionResult<T> = Result<T, RegionError>;
```

### conncomp.rs

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityType {
    FourWay,  // 上下左右のみ
    EightWay, // 斜めも含む
}

pub struct ConnectedComponent {
    pub label: u32,
    pub pixel_count: u32,
    pub bounds: Box,
}

pub fn find_connected_components(
    pix: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Vec<ConnectedComponent>>;

pub fn label_connected_components(
    pix: &Pix,
    connectivity: ConnectivityType,
) -> RegionResult<Pix>;
```

### seedfill.rs

```rust
pub struct SeedFillOptions {
    pub connectivity: ConnectivityType,
    pub fill_value: u32,
}

pub fn seedfill_binary(
    pix: &Pix,
    seed_x: u32,
    seed_y: u32,
    options: &SeedFillOptions,
) -> RegionResult<Pix>;

pub fn floodfill(
    pix: &mut PixMut,
    seed_x: u32,
    seed_y: u32,
    new_value: u32,
    connectivity: ConnectivityType,
) -> RegionResult<u32>;
```

### watershed.rs

```rust
pub struct WatershedOptions {
    pub min_depth: u32,
    pub connectivity: ConnectivityType,
}

pub fn watershed_segmentation(
    pix: &Pix,
    options: &WatershedOptions,
) -> RegionResult<Pix>;
```

## 修正対象ファイル

| ファイル                                    | 操作                   |
| ------------------------------------------- | ---------------------- |
| `crates/leptonica-region/Cargo.toml`        | 編集（thiserror追加）  |
| `crates/leptonica-region/src/lib.rs`        | 書き換え               |
| `crates/leptonica-region/src/error.rs`      | 新規作成               |
| `crates/leptonica-region/src/conncomp.rs`   | 新規作成               |
| `crates/leptonica-region/src/label.rs`      | 新規作成               |
| `crates/leptonica-region/src/seedfill.rs`   | 新規作成               |
| `crates/leptonica-region/src/watershed.rs`  | 新規作成               |

## 参照ファイル

- `/home/tagawa/github/leptonica-rs/crates/leptonica-color/src/error.rs` - エラー型パターン
- `/home/tagawa/github/leptonica-rs/crates/leptonica-core/src/box_/mod.rs` - Box構造体
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/pixlabel.c` - ラベリング
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/seedfill.c` - シードフィル
- `/home/tagawa/github/leptonica-rs/reference/leptonica/src/watershed.c` - 分水嶺

## アルゴリズム概要

### 連結成分分析

Union-Find（素集合データ構造）を使用した効率的なラベリング:

1. 第1パス: 各ピクセルに仮ラベルを付け、隣接ピクセルとの関係を記録
2. 第2パス: Union-Findで等価ラベルを解決
3. 第3パス: 最終ラベルを割り当て

### シードフィル

キューベースのフラッドフィルアルゴリズム:

1. シード位置をキューに追加
2. キューから取り出し、条件を満たせば塗りつぶし
3. 隣接ピクセルをキューに追加
4. キューが空になるまで繰り返し

### 分水嶺

マーカーベースの分水嶺アルゴリズム:

1. 勾配画像を計算
2. 各ローカルミニマムからフラッディング開始
3. 異なる流域が出会う場所に分水嶺線を形成

## 検証方法

1. **ユニットテスト**
   - 単純な図形（正方形、円）の連結成分検出
   - 4連結と8連結の結果の違い
   - シードフィルの境界処理
   - 分水嶺の分離精度

2. **ビルド確認**

   ```bash
   cargo build -p leptonica-region
   cargo test -p leptonica-region
   ```

3. **統合テスト**
   - leptonica-morphとの連携（膨張→連結成分分析）
   - leptonica-colorとの連携（二値化→ラベリング）

## 見積もりテスト数

- conncomp: 6-8テスト（4/8連結、境界条件）
- label: 4-5テスト（ラベル数、バウンディングボックス）
- seedfill: 4-6テスト（フラッドフィル、境界処理）
- watershed: 3-4テスト（基本セグメンテーション）
- **合計: 17-23テスト**
