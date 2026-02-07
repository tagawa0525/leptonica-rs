# ワーパー (Warper) 実装計画

## 概要

ワーパーは画像をランダムまたは指定パターンで歪ませる機能を提供する。
CAPTCHAの生成、ステレオスコピック効果、画像の芸術的な歪みなどに使用される。

### 主な機能

1. **ランダム調和関数歪み (Random Harmonic Warp)**: 正弦波の組み合わせによるランダムな歪み
2. **ステレオスコピック歪み**: 赤-シアンアナグリフ用の立体視効果
3. **水平ストレッチ**: 線形または二次の水平方向の伸縮
4. **二次垂直シアー**: 二次曲線に沿った垂直方向のシアー

## 参照

- C版: `reference/leptonica/src/warper.c`
- パターン: `crates/leptonica-transform/src/affine.rs`, `projective.rs`, `shear.rs`

## 実装内容

### 1. 定数と列挙型

```rust
/// ワープの方向
pub enum WarpDirection {
    /// 左方向へワープ (L_WARP_TO_LEFT)
    ToLeft,
    /// 右方向へワープ (L_WARP_TO_RIGHT)
    ToRight,
}

/// ワープのタイプ
pub enum WarpType {
    /// 線形ワープ (L_LINEAR_WARP)
    Linear,
    /// 二次ワープ (L_QUADRATIC_WARP)
    Quadratic,
}

/// 操作タイプ
pub enum WarpOperation {
    /// サンプリング (L_SAMPLED)
    Sampled,
    /// 補間 (L_INTERPOLATED)
    Interpolated,
}

/// 背景色
pub enum WarpFill {
    White,  // L_BRING_IN_WHITE
    Black,  // L_BRING_IN_BLACK
}
```

### 2. 主要関数

#### 2.1 ランダム調和関数歪み

- `random_harmonic_warp(pix, xmag, ymag, xfreq, yfreq, nx, ny, seed,
  gray_val) -> TransformResult<Pix>`
  - 複数の正弦波項による歪み生成
  - CAPTCHAなどに使用

#### 2.2 ステレオスコピック歪み

- `warp_stereoscopic(pix: &Pix, params: StereoscopicParams) -> TransformResult<Pix>`
  - 立体視効果の生成
  - 赤チャンネルを水平方向に歪ませ、シアンチャンネルと合成

```rust
pub struct StereoscopicParams {
    /// 水平曲がり量（エッジでの赤-シアン分離量）
    pub zbend: i32,
    /// 上部の垂直方向シフト
    pub zshift_top: i32,
    /// 下部の垂直方向シフト
    pub zshift_bottom: i32,
    /// 上部の面内垂直変位
    pub ybend_top: i32,
    /// 下部の面内垂直変位
    pub ybend_bottom: i32,
    /// 赤フィルタが左目用か
    pub red_left: bool,
}
```

#### 2.3 水平ストレッチ

- `stretch_horizontal(pix, direction, warp_type, hmax, operation,
  fill) -> TransformResult<Pix>`
  - 線形または二次の水平ストレッチ

- `stretch_horizontal_sampled(pix, direction, warp_type, hmax,
  fill) -> TransformResult<Pix>`
  - サンプリング版

- `stretch_horizontal_li(pix, direction, warp_type, hmax,
  fill) -> TransformResult<Pix>`
  - 線形補間版

#### 2.4 二次垂直シアー

- `quadratic_v_shear(pix, direction, vmax_top, vmax_bottom,
  operation, fill) -> TransformResult<Pix>`
  - 二次曲線に沿った垂直シアー

- `quadratic_v_shear_sampled(pix, direction, vmax_top,
  vmax_bottom, fill) -> TransformResult<Pix>`
  - サンプリング版

- `quadratic_v_shear_li(pix, direction, vmax_top, vmax_bottom,
  fill) -> TransformResult<Pix>`
  - 線形補間版

#### 2.5 ステレオペアから立体画像

- `stereo_from_pair(pix1: &Pix, pix2: &Pix, rwt: f32, gwt: f32, bwt: f32) -> TransformResult<Pix>`
  - 2枚のステレオ画像から赤-シアンアナグリフを生成

### 3. 対応ピクセル深度

| 関数 | 1bpp | 8bpp | 32bpp | 備考 |
| --- | --- | --- | --- | --- |
| random_harmonic_warp | - | Yes | - | 8bpp専用 |
| warp_stereoscopic | Yes | Yes | Yes | 32bpp出力 |
| stretch_horizontal_sampled | Yes | Yes | Yes | 全深度対応 |
| stretch_horizontal_li | - | Yes | Yes | 8bpp/32bpp |
| quadratic_v_shear_sampled | Yes | Yes | Yes | 全深度対応 |
| quadratic_v_shear_li | - | Yes | Yes | 8bpp/32bpp |
| stereo_from_pair | - | - | Yes | 32bpp専用 |

## 実装手順

### Phase 1: 基本構造と列挙型

1. `warper.rs` ファイル作成
2. 列挙型の定義（`WarpDirection`, `WarpType`, `WarpOperation`, `WarpFill`）
3. `StereoscopicParams` 構造体定義

### Phase 2: 水平ストレッチ

1. `stretch_horizontal_sampled()` 実装
2. `stretch_horizontal_li()` 実装
3. `stretch_horizontal()` ディスパッチャー実装

### Phase 3: 二次垂直シアー

1. `quadratic_v_shear_sampled()` 実装
2. `quadratic_v_shear_li()` 実装
3. `quadratic_v_shear()` ディスパッチャー実装

### Phase 4: ランダム調和関数歪み

1. 乱数配列生成ヘルパー実装
2. ワープ変換適用関数実装
3. `random_harmonic_warp()` 実装
4. オプション: `simple_captcha()` 実装

### Phase 5: ステレオスコピック

1. `warp_stereoscopic()` 実装
2. `stereo_from_pair()` 実装

### Phase 6: 統合・テスト

1. `lib.rs` へのエクスポート追加
2. 単体テスト作成
3. 品質チェック（fmt, clippy）

## テスト計画

### 単体テスト

1. **列挙型テスト**
   - WarpFillのピクセル値変換

2. **水平ストレッチテスト**
   - ゼロ変位で同一画像
   - 線形と二次の変位パターン確認
   - 方向（左/右）の動作確認

3. **二次垂直シアーテスト**
   - ゼロシアーで同一画像
   - 上下異なる変位量の確認
   - 方向の動作確認

4. **ランダム調和関数テスト**
   - 同じシードで再現可能
   - 異なるシードで異なる結果
   - 基本的なCAPTCHA生成

5. **ステレオスコピックテスト**
   - 出力が32bpp
   - 赤チャンネルの歪み確認
   - ステレオペア合成

6. **深度別テスト**
   - 各対応深度で動作確認
   - 非対応深度でエラー確認

## 技術的詳細

### ランダム調和関数の数式

```text
x' = x + Σ[i=0..nx-1] xmag * randa[3i] * sin(anglex_i) * sin(angley_i)
y' = y + Σ[i=nx..nx+ny-1] ymag * randa[3i] * sin(angley_i) * sin(anglex_i)

where:
  anglex = xfreq * randa[3i+1] * x + 2π * randa[3i+2]
  angley = yfreq * randa[3i+3] * y + 2π * randa[3i+4]
```

### 水平ストレッチの計算

**線形ワープ (direction=ToLeft):**

```text
j = jd - (hmax * (wm - jd)) / wm
```

**二次ワープ (direction=ToLeft):**

```text
j = jd - (hmax * (wm - jd)^2) / wm^2
```

### 二次垂直シアーの計算

```text
delrowt = (vmaxt * (wm - j)^2) / wm^2  (direction=ToLeft)
delrowb = (vmaxb * (wm - j)^2) / wm^2
dely = (delrowt * (hm - id) + delrowb * id) / h
i = id - dely
```

## 質問

なし（現時点）

## 見積もり

- Phase 1: 基本構造 - 30分
- Phase 2: 水平ストレッチ - 1時間
- Phase 3: 二次垂直シアー - 1時間
- Phase 4: ランダム調和関数 - 1.5時間
- Phase 5: ステレオスコピック - 1時間
- Phase 6: 統合・テスト - 30分
- 合計: 5-6時間

## 承認

承認後、実装を開始します。
