# 双線形変換 (Bilinear Transform) 実装計画

## 概要

双線形変換は4点対応による非線形変換で、アフィン変換（3点対応）よりも柔軟な歪み補正が可能。
射影変換の近似として使用され、小さなワープでは射影変換と非常に類似し、より安定した動作を示す。

### 変換式

```math
x' = a*x + b*y + c*x*y + d
y' = e*x + f*y + g*x*y + h
```

8つの係数 [a, b, c, d, e, f, g, h] を4組の対応点から計算する。

## 参照

- C版: `reference/leptonica/src/bilinear.c`
- パターン: `crates/leptonica-transform/src/affine.rs`

## 実装内容

### 1. データ構造

```rust
/// 双線形変換の8係数
pub struct BilinearCoeffs {
    coeffs: [f32; 8],  // [a, b, c, d, e, f, g, h]
}
```

### 2. 主要関数

#### 2.1 係数計算

- `BilinearCoeffs::from_four_points(src_pts: [Point; 4], dst_pts: [Point; 4])`
  - 4点対応から係数計算
- `BilinearCoeffs::inverse()`
  - 逆変換係数の計算（src->dstの係数からdst->srcを計算）

#### 2.2 点変換

- `transform_point_sampled(&self, x: i32, y: i32) -> (i32, i32)`
  - サンプリング用（整数座標）
- `transform_point_float(&self, x: f32, y: f32) -> (f32, f32)`
  - 補間用（浮動小数点）

#### 2.3 画像変換（サンプリング）

- `bilinear_sampled(pix: &Pix, coeffs: &BilinearCoeffs, fill: BilinearFill)`
  - 最近傍サンプリング
- `bilinear_sampled_pta(pix: &Pix, src_pts: [Point; 4], dst_pts: [Point; 4],`
  `fill: BilinearFill)` - 4点指定版

#### 2.4 画像変換（補間）

- `bilinear(pix: &Pix, coeffs: &BilinearCoeffs, fill: BilinearFill)`
  - 補間付き変換
- `bilinear_pta(pix: &Pix, src_pts: [Point; 4], dst_pts: [Point; 4],`
  `fill: BilinearFill)` - 4点指定版
- `bilinear_gray(pix: &Pix, coeffs: &BilinearCoeffs, gray_val: u8)`
  - 8bpp専用
- `bilinear_color(pix: &Pix, coeffs: &BilinearCoeffs, color_val: u32)`
  - 32bpp専用

### 3. 対応ピクセル深度

|深度|サンプリング|補間|
|---|---|---|
|1bpp|Yes|サンプリングにフォールバック|
|2bpp|Yes|サンプリングにフォールバック|
|4bpp|Yes|サンプリングにフォールバック|
|8bpp|Yes|Yes（グレースケール補間）|
|32bpp|Yes|Yes（色チャンネル別補間）|

### 4. 塗りつぶし色

```rust
pub enum BilinearFill {
    White,
    Black,
    Color(u32),
}
```

affine.rsの`AffineFill`を再利用する方針。

## 実装手順

### Phase 1: 基本構造

1. `bilinear.rs` ファイル作成
2. `BilinearCoeffs` 構造体定義
3. 8x8 ガウス・ジョルダン消去法の実装

### Phase 2: 点変換

1. `from_four_points()` 実装
2. `transform_point_sampled()` 実装
3. `transform_point_float()` 実装

### Phase 3: サンプリング変換

1. `bilinear_sampled()` 実装
2. `bilinear_sampled_pta()` 実装

### Phase 4: 補間変換

1. `bilinear_gray()` 実装（8bpp）
2. `bilinear_color()` 実装（32bpp）
3. `bilinear()` / `bilinear_pta()` 実装

### Phase 5: 統合・テスト

1. `lib.rs` へのエクスポート追加
2. 単体テスト作成
3. 回帰テスト確認

## テスト計画

### 単体テスト

1. **係数計算テスト**
   - 恒等変換（4点が同じ）
   - 平行移動のみ
   - 既知の変換係数との比較

2. **点変換テスト**
   - 4つの対応点が正しく変換されることを確認
   - サンプリングと浮動小数点の一貫性

3. **画像変換テスト**
   - 恒等変換でピクセル値保存
   - 平行移動で正しい位置移動
   - 境界外ピクセルの塗りつぶし確認

4. **深度別テスト**
   - 1bpp, 8bpp, 32bpp それぞれで動作確認

## 技術的詳細

### ガウス・ジョルダン消去法 (8x8)

affine.rsでは6x6行列を使用しているが、bilinearでは8x8行列が必要。
汎用的な実装を検討：

```rust
fn gauss_jordan_8x8(
    a: &mut [[f32; 8]; 8],
    b: &mut [f32; 8]
) -> TransformResult<()>
```

### 係数行列の構造

```text
| x1  y1  x1*y1  1  0   0    0     0  | | a | | x1' |
| 0   0    0     0  x1  y1  x1*y1  1  | | b | | y1' |
| x2  y2  x2*y2  1  0   0    0     0  | | c | | x2' |
| 0   0    0     0  x2  y2  x2*y2  1  |*| d |=| y2' |
| x3  y3  x3*y3  1  0   0    0     0  | | e | | x3' |
| 0   0    0     0  x3  y3  x3*y3  1  | | f | | y3' |
| x4  y4  x4*y4  1  0   0    0     0  | | g | | x4' |
| 0   0    0     0  x4  y4  x4*y4  1  | | h | | y4' |
```

## 質問

なし（現時点）

## 見積もり

- 実装: 2-3時間
- テスト: 1時間
- 合計: 3-4時間

## 承認

承認後、実装を開始します。
