# 任意角度回転実装計画

## 概要

Leptonicaのrotate.c, rotateam.c, rotateshear.cを参考に、任意角度での画像回転機能を実装する。

## 現状分析

### 既存の実装

`crates/leptonica-transform/src/rotate.rs` に以下が実装済み:

- 直交回転 (90/180/270度)
- フリップ (LR/TB)
- 任意角度回転 (`rotate_by_angle`) - バイリニア補間使用

### 不足している機能

1. 面積マッピング回転 (Area Mapping) - Leptonicaの`pixRotateAM`相当
2. サンプリング回転 - Leptonicaの`pixRotateBySampling`相当
3. シアーベース回転 - Leptonicaの`pixRotateShear`相当
4. 回転中心の任意指定
5. 背景色の詳細指定 (白/黒/任意色)

## 実装計画

### Phase 1: 回転メソッド列挙型の追加

```rust
/// 回転アルゴリズムの選択
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotateMethod {
    /// サンプリング (最近傍) - 高速、低品質
    Sampling,
    /// 面積マッピング - 高品質、低速
    AreaMap,
    /// シアーベース - 中品質、中速 (主に1bpp向け)
    Shear,
    /// 自動選択 (深度と角度に応じて最適な方式を選択)
    #[default]
    Auto,
}
```

### Phase 2: 背景色の指定

```rust
/// 背景色の指定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotateFill {
    /// 白色で埋める
    #[default]
    White,
    /// 黒色で埋める
    Black,
    /// 指定した色で埋める (32bppのRGBA値)
    Color(u32),
}
```

### Phase 3: 回転オプション構造体

```rust
/// 回転のオプション
#[derive(Debug, Clone)]
pub struct RotateOptions {
    /// 回転メソッド
    pub method: RotateMethod,
    /// 背景色
    pub fill: RotateFill,
    /// 回転中心のX座標 (Noneで画像中心)
    pub center_x: Option<f32>,
    /// 回転中心のY座標 (Noneで画像中心)
    pub center_y: Option<f32>,
    /// 出力画像を拡大して全ピクセルを保持するか
    pub expand: bool,
}
```

### Phase 4: 各回転アルゴリズムの実装

#### 4.1 サンプリング回転 (`rotate_sampling`)

- Leptonicaの`pixRotateBySampling`に相当
- 全深度対応
- 最も高速
- 実装: 逆変換で元画像からサンプリング

#### 4.2 面積マッピング回転 (`rotate_area_map`)

- Leptonicaの`pixRotateAMGray`/`pixRotateAMColor`に相当
- 8bpp, 32bpp対応
- 16x16サブピクセルグリッドで面積重み付け
- アンチエイリアス効果あり

#### 4.3 シアーベース回転 (`rotate_shear`)

- Leptonicaの`pixRotateShear`に相当
- 2-shear (小角度: ~3度まで) または 3-shear (大角度: ~20度まで)
- 1bpp向けに最適化

### Phase 5: 高レベルAPI

```rust
/// 任意角度で画像を回転
pub fn rotate(pix: &Pix, angle: f32, options: &RotateOptions) -> TransformResult<Pix>;

/// 角度とメソッドを指定して回転 (簡易API)
pub fn rotate_with_method(pix: &Pix, angle: f32, method: RotateMethod) -> TransformResult<Pix>;

/// 回転中心を指定して回転
pub fn rotate_about_center(
    pix: &Pix,
    angle: f32,
    center_x: f32,
    center_y: f32,
    fill: RotateFill,
) -> TransformResult<Pix>;
```

### Phase 6: 既存APIとの統合

既存の `rotate_by_angle` を新しいAPIと統合し、後方互換性を維持:

- `rotate_by_angle` は `RotateMethod::Auto` + `RotateFill::White` として動作
- 内部実装を新しい関数に委譲

## 実装詳細

### 面積マッピングのアルゴリズム

Leptonicaの実装を参考に:

1. sin/cosを16倍してサブピクセル精度を確保
2. 各出力ピクセルに対して、逆変換で入力座標を計算
3. 4つの隣接ピクセルの面積重み付け平均を計算
4. 境界外は指定した背景色で埋める

```rust
// サブピクセル位置 (16x16グリッド)
let xf = xpm & 0x0f;  // 0-15
let yf = ypm & 0x0f;  // 0-15

// 面積重み
let w00 = (16 - xf) * (16 - yf);  // 左上
let w10 = xf * (16 - yf);          // 右上
let w01 = (16 - xf) * yf;          // 左下
let w11 = xf * yf;                  // 右下

// 重み付け平均 (256で正規化)
let val = (w00 * p00 + w10 * p10 + w01 * p01 + w11 * p11 + 128) / 256;
```

### シアーベース回転のアルゴリズム

3-shearによる回転 (Paeth's algorithm):

1. 垂直シアー: y' = y + tan(angle/2) * (x - xcen)
2. 水平シアー: x' = x + sin(angle) * (y - ycen)
3. 垂直シアー: y' = y + tan(angle/2) * (x - xcen)

シアー操作自体は別モジュール(`shear.rs`)で実装予定だが、
本実装ではラスタ操作で直接実装する。

## ファイル構成

```text
crates/leptonica-transform/src/
├── lib.rs         # 新しいAPIをエクスポート
├── error.rs       # 変更なし
├── rotate.rs      # 拡張
├── scale.rs       # 変更なし
└── (shear.rs)     # 将来の拡張用 (今回は不要)
```

## テスト計画

1. **単体テスト**
   - 各メソッドで90度回転した結果が直交回転と一致
   - 360度回転で元画像に近い結果
   - 境界値テスト (0度、180度、-180度)
   - 各深度でのテスト (1bpp, 8bpp, 32bpp)

2. **品質テスト**
   - 面積マッピングのアンチエイリアス効果
   - サンプリングのジャギー確認

3. **パフォーマンステスト**
   - 各メソッドの速度比較

## 定数

Leptonicaに準拠:

```rust
const MIN_ANGLE_TO_ROTATE: f32 = 0.001;  // ~0.06度
const MAX_TWO_SHEAR_ANGLE: f32 = 0.06;   // ~3度
const MAX_THREE_SHEAR_ANGLE: f32 = 0.35; // ~20度
const MAX_SHEAR_ANGLE: f32 = 0.50;       // ~29度
```

## 質問

特になし。既存実装のパターンに従い、上記計画で進めます。

## 実装優先順位

1. 型定義 (RotateMethod, RotateFill, RotateOptions)
2. サンプリング回転 (最もシンプル)
3. 面積マッピング回転 (8bpp/32bpp)
4. シアーベース回転 (1bpp向け)
5. 高レベルAPI統合
6. テスト
