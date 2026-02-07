# 射影変換 (Projective Transform) 実装計画

## 概要

射影変換は4点対応によるホモグラフィ（射影）変換で、アフィン変換や双線形変換とは異なり、直線を直線に保ちながらも平行線を保存しない。
台形補正などの透視歪み補正に有用。

### 変換式

```math
x' = (a*x + b*y + c) / (g*x + h*y + 1)
y' = (d*x + e*y + f) / (g*x + h*y + 1)
```

8つの係数 [a, b, c, d, e, f, g, h] を4組の対応点から計算する。

## 参照

- C版: `reference/leptonica/src/projective.c`
- パターン: `crates/leptonica-transform/src/affine.rs`, `bilinear.rs`

## 実装内容

### 1. ProjectiveCoeffs 構造体

```rust
/// 射影変換の8係数
pub struct ProjectiveCoeffs {
    coeffs: [f32; 8],  // [a, b, c, d, e, f, g, h]
}
```

- 8係数を保持
- `identity()` - 恒等変換
- `from_coeffs()` - 生の係数から生成
- `from_four_points()` - 4点対応から係数を計算
- `transform_point_sampled()` - サンプリング変換（整数座標）
- `transform_point_float()` - 浮動小数点座標変換

### 2. サンプリング射影変換

- `projective_sampled()` - 係数を使った変換
- `projective_sampled_pta()` - 4点対応を使った変換
- 全ビット深度対応（1, 2, 4, 8, 16, 32bpp）
- カラーマップの保持

### 3. 補間射影変換

- `projective()` - 係数を使った変換
- `projective_pta()` - 4点対応を使った変換
- `projective_gray()` - 8bpp用双線形補間
- `projective_color()` - 32bpp用双線形補間
- 1bppの場合はサンプリングにフォールバック

### 4. Gauss-Jordan消去法

- 8x8行列用の連立方程式解法
- 特異行列のエラー処理

### 5. 公開API

- lib.rs へのモジュール宣言
- re-export の設定

### 6. テスト

- 恒等変換
- 平行移動
- 拡大縮小
- 台形補正
- 境界外フィル値
- カラーマップ保持

## 実装完了

全てのタスクが完了：

- ✓ `cargo fmt` パス
- ✓ `cargo clippy` 警告なし
- ✓ `cargo test` 全テストパス (132/132 passed)
- ✓ 実装完了
