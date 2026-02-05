# アフィン変換実装計画

## 概要

Leptonicaの`affine.c`および`affinecompose.c`を参考に、アフィン変換機能を実装する。

## 参照ソース

- `reference/leptonica/src/affine.c` - 画像アフィン変換の実装
- `reference/leptonica/src/affinecompose.c` - 変換行列の合成操作

## 実装先

- `crates/leptonica-transform/src/affine.rs` (新規作成)
- `crates/leptonica-transform/src/lib.rs` (エクスポート追加)

## 実装内容

### 1. 型定義

#### 1.1 AffineMatrix (変換行列)

```rust
/// 2Dアフィン変換行列
///
/// 行列形式:
/// | a  b  tx |
/// | c  d  ty |
/// | 0  0  1  |
///
/// 変換式:
///   x' = a*x + b*y + tx
///   y' = c*x + d*y + ty
///
/// 内部表現: [a, b, tx, c, d, ty] (6要素の配列)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineMatrix {
    coeffs: [f32; 6],  // [a, b, tx, c, d, ty]
}
```

#### 1.2 Point (2D点)

```rust
/// 2D浮動小数点座標
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
```

#### 1.3 AffineFill (背景色指定)

```rust
/// アフィン変換時の背景色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AffineFill {
    /// 白色
    #[default]
    White,
    /// 黒色
    Black,
    /// 任意の色値
    Color(u32),
}
```

### 2. AffineMatrix の操作

#### 2.1 基本構築関数

```rust
impl AffineMatrix {
    /// 単位行列を作成
    pub fn identity() -> Self;

    /// 平行移動行列を作成
    pub fn translation(tx: f32, ty: f32) -> Self;

    /// スケーリング行列を作成
    pub fn scale(sx: f32, sy: f32) -> Self;

    /// 回転行列を作成（回転中心指定）
    pub fn rotation(center_x: f32, center_y: f32, angle: f32) -> Self;

    /// 3点対応から変換行列を計算
    /// Leptonicaの getAffineXformCoeffs 相当
    pub fn from_three_points(
        src_pts: [Point; 3],
        dst_pts: [Point; 3],
    ) -> TransformResult<Self>;
}
```

#### 2.2 行列演算

```rust
impl AffineMatrix {
    /// 行列の逆変換を計算
    pub fn inverse(&self) -> TransformResult<Self>;

    /// 行列を合成（self * other の順に適用）
    pub fn compose(&self, other: &Self) -> Self;

    /// 点を変換
    pub fn transform_point(&self, pt: Point) -> Point;

    /// 点を変換（整数座標、サンプリング用）
    pub fn transform_point_sampled(&self, x: i32, y: i32) -> (i32, i32);

    /// 係数を取得
    pub fn coeffs(&self) -> &[f32; 6];
}
```

### 3. 画像アフィン変換

#### 3.1 サンプリングアフィン

```rust
/// サンプリング（最近傍）によるアフィン変換
/// Leptonicaの pixAffineSampled 相当
///
/// 全深度対応。最も高速。
pub fn affine_sampled(
    pix: &Pix,
    matrix: &AffineMatrix,
    fill: AffineFill,
) -> TransformResult<Pix>;

/// 3点対応によるサンプリングアフィン変換
/// Leptonicaの pixAffineSampledPta 相当
pub fn affine_sampled_pta(
    pix: &Pix,
    src_pts: [Point; 3],
    dst_pts: [Point; 3],
    fill: AffineFill,
) -> TransformResult<Pix>;
```

#### 3.2 補間アフィン

```rust
/// 補間によるアフィン変換
/// Leptonicaの pixAffine/pixAffineGray/pixAffineColor 相当
///
/// 8bpp/32bppでバイリニア補間を使用。1bppはサンプリングにフォールバック。
pub fn affine(
    pix: &Pix,
    matrix: &AffineMatrix,
    fill: AffineFill,
) -> TransformResult<Pix>;

/// 3点対応による補間アフィン変換
/// Leptonicaの pixAffinePta 相当
pub fn affine_pta(
    pix: &Pix,
    src_pts: [Point; 3],
    dst_pts: [Point; 3],
    fill: AffineFill,
) -> TransformResult<Pix>;
```

### 4. ユーティリティ関数

```rust
/// 平行移動
pub fn translate(pix: &Pix, tx: f32, ty: f32) -> TransformResult<Pix>;

/// スケーリング（アフィン変換として）
pub fn affine_scale(pix: &Pix, sx: f32, sy: f32) -> TransformResult<Pix>;

/// 回転（アフィン変換として）
pub fn affine_rotate(
    pix: &Pix,
    center_x: f32,
    center_y: f32,
    angle: f32,
) -> TransformResult<Pix>;
```

## 実装詳細

### 3点対応からの係数計算 (getAffineXformCoeffs)

Leptonicaのアルゴリズム:

1. 6つの方程式を立てる:

   ```text
   x1' = c[0]*x1 + c[1]*y1 + c[2]
   y1' = c[3]*x1 + c[4]*y1 + c[5]
   x2' = c[0]*x2 + c[1]*y2 + c[2]
   y2' = c[3]*x2 + c[4]*y2 + c[5]
   x3' = c[0]*x3 + c[1]*y3 + c[2]
   y3' = c[3]*x3 + c[4]*y3 + c[5]
   ```

2. 6x6行列の連立方程式をガウス・ジョルダン法で解く

### 逆変換 (affineInvertXform)

重要: 画像変換には逆変換行列を使用する必要がある。

- 点の変換: 順変換 (src -> dst)
- 画像の変換: 逆変換 (dst -> src) で各出力ピクセルの入力座標を求める

### 補間アルゴリズム

8bpp/32bppでは16x16サブピクセルグリッドの面積重み付け補間を使用:

```rust
// サブピクセル位置計算
let xpm = (16.0 * x).floor() as i32;
let ypm = (16.0 * y).floor() as i32;
let xp = xpm >> 4;
let yp = ypm >> 4;
let xf = xpm & 0x0f;
let yf = ypm & 0x0f;

// 面積重み付け補間
let val = ((16 - xf) * (16 - yf) * p00
         + xf * (16 - yf) * p10
         + (16 - xf) * yf * p01
         + xf * yf * p11
         + 128) / 256;
```

## ファイル構成

```text
crates/leptonica-transform/src/
├── lib.rs         # affine モジュールをエクスポート
├── error.rs       # 既存
├── rotate.rs      # 既存
├── scale.rs       # 既存
└── affine.rs      # 新規作成
```

## テスト計画

### 単体テスト

1. **行列演算テスト**
   - 単位行列の検証
   - 逆行列の検証 (M * M^-1 = I)
   - 行列合成の検証
   - 3点対応からの係数計算の検証

2. **点変換テスト**
   - 平行移動
   - スケーリング
   - 回転
   - 合成変換

3. **画像変換テスト**
   - サンプリングアフィン: 各深度 (1bpp, 8bpp, 32bpp)
   - 補間アフィン: 8bpp, 32bpp
   - 背景色の検証
   - 境界条件

4. **エッジケース**
   - 特異行列（逆行列なし）のエラーハンドリング
   - 共線な3点のエラーハンドリング

### リグレッションテスト

テストデータ: `test-data/`配下に画像を用意し、期待される出力と比較

## 質問

特になし。既存の`rotate.rs`のパターンに従い、上記計画で実装を進めます。

## 実装優先順位

1. `AffineMatrix` 型と基本操作
2. ガウス・ジョルダン法の実装（3点対応用）
3. サンプリングアフィン変換
4. 補間アフィン変換
5. ユーティリティ関数
6. lib.rsへのエクスポート
7. テスト
