# シアー変換 (Shear Transform) 実装計画

## 概要

Leptonica の shear.c に相当するシアー変換機能を leptonica-transform クレートに実装する。

## 参照ソース

- `reference/leptonica/src/shear.c`

## 実装する機能

### 基本的なシアー変換

- `h_shear()` - `pixHShear` 相当：任意のライン基準の水平シアー
- `v_shear()` - `pixVShear` 相当：任意のライン基準の垂直シアー

### 特定の基準点でのシアー

- `h_shear_corner()` - `pixHShearCorner` 相当：左上コーナー基準
- `v_shear_corner()` - `pixVShearCorner` 相当：左上コーナー基準
- `h_shear_center()` - `pixHShearCenter` 相当：中心基準
- `v_shear_center()` - `pixVShearCenter` 相当：中心基準

### インプレースシアー

- `h_shear_ip()` - `pixHShearIP` 相当：水平シアー（インプレース）
- `v_shear_ip()` - `pixVShearIP` 相当：垂直シアー（インプレース）

### 線形補間シアー（高品質）

- `h_shear_li()` - `pixHShearLI` 相当：水平シアー（線形補間）
- `v_shear_li()` - `pixVShearLI` 相当：垂直シアー（線形補間）

## データ型設計

### ShearFill 列挙型

```rust
/// シアー変換時の背景塗りつぶし色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShearFill {
    /// 白で塗りつぶし
    #[default]
    White,
    /// 黒で塗りつぶし
    Black,
}
```

### 定数

```rust
/// シアー角度の最小差（±π/2 から離れている必要がある）
const MIN_DIFF_FROM_HALF_PI: f32 = 0.04;
```

## アルゴリズム概要

### 水平シアー (H-Shear)

```text
y = yloc のラインを不変とし、
各行を tan(angle) * (y - yloc) だけ水平方向にシフト

正の角度の場合：
- yloc より上の行は右にシフト
- yloc より下の行は左にシフト
```

### 垂直シアー (V-Shear)

```text
x = xloc のラインを不変とし、
各列を tan(angle) * (x - xloc) だけ垂直方向にシフト

正の角度の場合：
- xloc より右の列は下にシフト
- xloc より左の列は上にシフト
```

### ラスター操作による最適化（C 版）

C 版では `pixRasterop` や `pixRasteropHip`/`pixRasteropVip` を使用して効率的に行/列のシフトを行っている。
Rust 版では、最初はピクセルごとの処理で実装し、性能が問題になれば最適化する。

### 線形補間シアー (LI)

8bpp と 32bpp 画像に対して、サブピクセル精度で補間を行う。
64 分割の精度で補間計算を行う。

## 実装手順

### ステップ 1: 基本インフラ

1. `shear.rs` ファイルを作成
2. `ShearFill` 列挙型を定義
3. 角度正規化関数 `normalize_angle_for_shear` を実装
4. エラーハンドリング（既存の `TransformError` を使用）

### ステップ 2: 基本的なシアー変換

1. `h_shear` - ピクセル単位の実装
2. `v_shear` - ピクセル単位の実装

### ステップ 3: 便利関数

1. `h_shear_corner` - h_shear のラッパー（yloc=0）
2. `v_shear_corner` - v_shear のラッパー（xloc=0）
3. `h_shear_center` - h_shear のラッパー（yloc=height/2）
4. `v_shear_center` - v_shear のラッパー（xloc=width/2）

### ステップ 4: インプレース変換

1. `h_shear_ip` - PixMut を受け取るバージョン
2. `v_shear_ip` - PixMut を受け取るバージョン

### ステップ 5: 線形補間シアー

1. `h_shear_li` - 8bpp/32bpp 用高品質シアー
2. `v_shear_li` - 8bpp/32bpp 用高品質シアー

### ステップ 6: lib.rs 更新とエクスポート

1. `mod shear;` を追加
2. 公開 API をエクスポート

## API 設計

### 基本的な使い方

```rust
use leptonica_transform::{h_shear, v_shear, ShearFill};
use leptonica_core::Pix;

// 水平シアー（中央のラインを基準）
let result = h_shear(&pix, pix.height() / 2, 0.1, ShearFill::White)?;

// 垂直シアー（左端を基準）
let result = v_shear(&pix, 0, -0.1, ShearFill::Black)?;
```

### 便利関数

```rust
use leptonica_transform::{h_shear_center, v_shear_corner, ShearFill};

// 中心基準の水平シアー
let result = h_shear_center(&pix, 0.05, ShearFill::White)?;

// 左上コーナー基準の垂直シアー
let result = v_shear_corner(&pix, 0.1, ShearFill::White)?;
```

### 高品質シアー

```rust
use leptonica_transform::{h_shear_li, v_shear_li, ShearFill};

// 線形補間を使用した高品質水平シアー（8bpp/32bpp のみ）
let result = h_shear_li(&pix, yloc, 0.1, ShearFill::White)?;
```

## テスト計画

### 単体テスト

1. 角度正規化関数のテスト
2. 角度 0 の場合のテスト（元画像のコピー）
3. 小さな角度での水平/垂直シアー
4. 様々な基準点でのシアー
5. 1bpp, 8bpp, 32bpp での動作確認
6. 境界値テスト（画像端での動作）
7. 線形補間の品質テスト

### 回帰テスト（検討中）

- 既知の入力画像に対する期待出力との比較

## ファイル構成

```text
crates/leptonica-transform/src/
├── lib.rs        (mod shear 追加)
├── shear.rs      (新規作成)
├── rotate.rs     (既存)
├── affine.rs     (既存)
├── scale.rs      (既存)
└── error.rs      (既存)
```

## 質問

特になし。

## 完了条件

- [x] 計画承認
- [x] shear.rs 実装完了
- [x] lib.rs 更新
- [x] 全テストパス
- [x] cargo fmt 通過
- [x] cargo clippy 通過
- [ ] コミット作成
