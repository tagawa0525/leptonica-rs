# Quadtree (四分木) 実装計画

## 概要

画像を再帰的に4分割して領域を階層化するデータ構造（四分木）を実装する。
C版 `quadtree.c` を参考に、各ブロックの統計量（平均・分散）を階層的に計算する機能を提供する。

## 参照

- C版ソース: `reference/leptonica/src/quadtree.c`
- 実装先: `crates/leptonica-region/src/quadtree.rs`
- 既存パターン参照: `crates/leptonica-region/src/conncomp.rs`

## C版の主要関数

1. **pixQuadtreeMean()** - 各ブロックの平均値を計算
2. **pixQuadtreeVariance()** - 各ブロックの分散を計算
3. **pixMeanInRectangle()** - 積分画像を使った矩形内の平均値計算
4. **pixVarianceInRectangle()** - 積分画像を使った矩形内の分散計算
5. **boxaaQuadtreeRegions()** - 各レベルの領域ボックスを生成
6. **quadtreeGetParent()** - 親ノードの取得
7. **quadtreeGetChildren()** - 子ノードの取得
8. **quadtreeMaxLevels()** - 最大レベル数を計算

## 設計方針

### 依存関係の整理

C版は以下を使用している：

- `PIX` (8bpp グレースケール) -> `Pix` (leptonica-core)
- `BOX`, `BOXA`, `BOXAA` -> `Box`, `Boxa`, `Boxaa` (leptonica-core)
- `FPIX`, `FPIXA` -> `FPix` は存在するが `FPixa` は未実装
- `DPIX` (mean square accumulator) -> 未実装

### Rust版の設計

FPixa が存在しないため、以下の2つのアプローチを検討：

#### アプローチ1: FPixa を実装せず、Vec\<FPix\> を直接使用

- シンプルで実装コストが低い
- QuadtreeResult 構造体で結果をラップ

#### アプローチ2: FPixa を新規実装

- C版との互換性が高い
- leptonica-core への変更が必要

**採用アプローチ：** アプローチ1（leptonica-region クレート内で完結させる）

### データ構造

```rust
/// 四分木の統計量結果
pub struct QuadtreeResult {
    /// 各レベルのFPix (level 0 = 1x1, level 1 = 2x2, level 2 = 4x4, ...)
    levels: Vec<FPix>,
}

/// 積分画像（Summed Area Table）
/// 平均値計算用
pub struct IntegralImage {
    data: Vec<u64>,  // オーバーフロー防止のためu64
    width: u32,
    height: u32,
}

/// 二乗積分画像
/// 分散計算用
pub struct SquaredIntegralImage {
    data: Vec<f64>,  // 精度のためf64
    width: u32,
    height: u32,
}
```

### API設計

```rust
// 公開API
pub fn quadtree_mean(pix: &Pix, nlevels: u32) -> RegionResult<QuadtreeResult>;
pub fn quadtree_mean_with_integral(
    pix: &Pix,
    nlevels: u32,
    integral: Option<&IntegralImage>,
) -> RegionResult<QuadtreeResult>;

pub fn quadtree_variance(
    pix: &Pix,
    nlevels: u32,
) -> RegionResult<(QuadtreeResult, QuadtreeResult)>;  // (variance, root_variance)

pub fn quadtree_max_levels(width: u32, height: u32) -> u32;

// 積分画像
pub fn compute_integral_image(pix: &Pix) -> RegionResult<IntegralImage>;
pub fn compute_squared_integral_image(pix: &Pix) -> RegionResult<SquaredIntegralImage>;

pub fn mean_in_rectangle(
    pix: &Pix,
    rect: &Box,
    integral: &IntegralImage,
) -> RegionResult<f32>;

pub fn variance_in_rectangle(
    pix: &Pix,
    rect: &Box,
    integral: &IntegralImage,
    sq_integral: &SquaredIntegralImage,
) -> RegionResult<(f32, f32)>;  // (variance, root_variance)

// 四分木アクセス
impl QuadtreeResult {
    pub fn get_parent(&self, level: u32, x: u32, y: u32) -> Option<f32>;
    pub fn get_children(&self, level: u32, x: u32, y: u32) -> Option<[f32; 4]>;
    pub fn get_value(&self, level: u32, x: u32, y: u32) -> Option<f32>;
    pub fn num_levels(&self) -> usize;
}

// 領域生成
pub fn quadtree_regions(width: u32, height: u32, nlevels: u32) -> RegionResult<Boxaa>;
```

## 実装手順

### Phase 1: 基本構造とユーティリティ

1. [ ] `quadtree.rs` ファイル作成
2. [ ] `QuadtreeResult` 構造体実装
3. [ ] `quadtree_max_levels()` 実装
4. [ ] `quadtree_regions()` 実装（Boxaa生成）

### Phase 2: 積分画像

1. [ ] `IntegralImage` 構造体実装
2. [ ] `SquaredIntegralImage` 構造体実装
3. [ ] `compute_integral_image()` 実装
4. [ ] `compute_squared_integral_image()` 実装

### Phase 3: 矩形内統計量

1. [ ] `mean_in_rectangle()` 実装
2. [ ] `variance_in_rectangle()` 実装

### Phase 4: 四分木統計量

1. [ ] `quadtree_mean()` 実装
2. [ ] `quadtree_variance()` 実装

### Phase 5: アクセス機能

1. [ ] `QuadtreeResult::get_parent()` 実装
2. [ ] `QuadtreeResult::get_children()` 実装
3. [ ] `QuadtreeResult::get_value()` 実装

### Phase 6: テストとドキュメント

1. [ ] ユニットテスト作成
2. [ ] lib.rsへのエクスポート追加
3. [ ] cargo fmt && cargo clippy

## 積分画像アルゴリズム

積分画像（Summed Area Table）は、任意の矩形領域の合計をO(1)で計算できるデータ構造。

```math
I(x, y) = Σ(i=0 to x, j=0 to y) p(i, j)
```

矩形(x1,y1)-(x2,y2)の合計:

```math
sum = I(x2,y2) - I(x1-1,y2) - I(x2,y1-1) + I(x1-1,y1-1)
```

分散の計算:

```math
variance = E[X^2] - E[X]^2
         = mean_of_squares - mean^2
```

## テスト計画

1. **境界条件テスト**
   - 1x1画像
   - 非正方形画像
   - 最大レベル制限

2. **精度テスト**
   - 既知の値を持つ画像で計算結果を検証
   - 均一画像（分散=0）

3. **親子関係テスト**
   - get_parent/get_childrenの整合性

4. **性能テスト**
   - 大きな画像での処理時間

## 質問

（なし）

## 進捗

- [x] Phase 1: 基本構造
- [x] Phase 2: 積分画像
- [x] Phase 3: 矩形内統計量
- [x] Phase 4: 四分木統計量
- [x] Phase 5: アクセス機能
- [x] Phase 6: テストとドキュメント
