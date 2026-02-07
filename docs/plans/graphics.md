# Graphics Implementation Plan

## 概要

画像上に図形を描画するグラフィックス機能を実装します。C版の graphics.c を参考に、Rust流のAPI設計で実装します。

## 参照

- C版ソース: `reference/leptonica/src/graphics.c`
- 実装先: `crates/leptonica-core/src/pix/graphics.rs`

## 実装する機能

### 1. 点描画の基本操作（PixelOp enum）

```rust
/// ピクセル操作の種類
pub enum PixelOp {
    Set,    // L_SET_PIXELS - ピクセルを設定
    Clear,  // L_CLEAR_PIXELS - ピクセルをクリア
    Flip,   // L_FLIP_PIXELS - ピクセルを反転
}
```

### 2. RGBA カラー型

```rust
/// 描画色の指定
#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
```

### 3. 点配列の描画

- `render_pta()` - Pta（点配列）を描画（PixelOp指定）
- `render_pta_color()` - Pta を任意の RGB 色で描画

### 4. 直線の描画

- `render_line()` - 直線の描画（PixelOp指定）
- `render_line_color()` - 任意色の直線描画
- `render_line_blend()` - ブレンド付き直線描画

### 5. 矩形の描画

- `render_box()` - 矩形枠線の描画（PixelOp指定）
- `render_box_color()` - 任意色の矩形描画
- `render_box_blend()` - ブレンド付き矩形描画

### 6. 折れ線の描画

- `render_polyline()` - 折れ線の描画（PixelOp指定）
- `render_polyline_color()` - 任意色の折れ線描画
- `render_polyline_blend()` - ブレンド付き折れ線描画

### 7. 円の描画

- `render_circle()` - 円の描画
- `generate_filled_circle_pta()` - 塗りつぶし円の点配列生成

### 8. 等高線の描画

- `render_contours()` - グレースケール画像の等高線描画

## 内部ヘルパー関数（点配列生成）

```rust
/// Bresenham線分アルゴリズムで直線の点配列を生成
fn generate_line_pta(x1: i32, y1: i32, x2: i32, y2: i32) -> Pta

/// 太さ付き直線の点配列を生成
fn generate_wide_line_pta(x1: i32, y1: i32, x2: i32, y2: i32, width: u32) -> Pta

/// 矩形枠の点配列を生成
fn generate_box_pta(b: &Box, width: u32) -> Pta

/// 折れ線の点配列を生成
fn generate_polyline_pta(vertices: &Pta, width: u32, close: bool) -> Pta
```

## API設計

### PixMut への拡張メソッド

```rust
impl PixMut {
    // 直線描画
    pub fn render_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32,
                       width: u32, op: PixelOp) -> Result<()>;
    pub fn render_line_color(&mut self, x1: i32, y1: i32, x2: i32, y2: i32,
                             width: u32, color: Color) -> Result<()>;

    // 矩形描画
    pub fn render_box(&mut self, b: &Box, width: u32, op: PixelOp) -> Result<()>;
    pub fn render_box_color(&mut self, b: &Box, width: u32, color: Color) -> Result<()>;

    // 折れ線描画
    pub fn render_polyline(&mut self, vertices: &Pta, width: u32,
                           close: bool, op: PixelOp) -> Result<()>;
    pub fn render_polyline_color(&mut self, vertices: &Pta, width: u32,
                                 close: bool, color: Color) -> Result<()>;

    // 円描画
    pub fn render_circle(&mut self, cx: i32, cy: i32, radius: u32,
                         width: u32, op: PixelOp) -> Result<()>;
    pub fn render_circle_color(&mut self, cx: i32, cy: i32, radius: u32,
                               width: u32, color: Color) -> Result<()>;
}
```

### Pix からの等高線描画（新規 Pix を生成）

```rust
impl Pix {
    pub fn render_contours(&self, start_val: u32, increment: u32,
                           out_depth: ContourOutput) -> Result<Pix>;
}

pub enum ContourOutput {
    Binary,     // 1bpp の等高線のみ
    Overlay,    // 元画像に等高線を重ねる
}
```

## ファイル構成

```text
crates/leptonica-core/src/pix/
├── mod.rs           # graphics モジュールを追加
└── graphics.rs      # 新規作成（このファイル）
```

## 実装順序

1. `PixelOp` enum と `Color` 構造体の定義
2. 点配列生成ヘルパー関数（`generate_line_pta` 等）
3. `render_pta()` と `render_pta_color()` 基本メソッド
4. `render_line()` 系メソッド
5. `render_box()` 系メソッド
6. `render_polyline()` 系メソッド
7. `render_circle()` 系メソッド
8. `render_contours()` メソッド
9. テストの作成

## テスト計画

- 直線描画: 水平、垂直、斜め線の各種テスト
- 矩形描画: 通常矩形、境界ケース
- 折れ線描画: 開いた折れ線、閉じた折れ線
- 円描画: 様々なサイズの円
- 等高線: 8bit/16bit グレースケール画像での等高線

## 質問

特になし
