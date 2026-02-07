# 算術演算モジュール実装計画

## 概要

画像の算術演算（加算、減算、乗算、除算、絶対値、定数演算）を実装する。

## 対応するLeptonica関数

- `pixAddConstantGray` - 定数加算（インプレース）
- `pixMultConstantGray` - 定数乗算（インプレース）
- `pixAddGray` - 2画像加算
- `pixSubtractGray` - 2画像減算
- `pixMultiplyGray` - グレースケール乗算
- `pixAbsDifference` - 絶対値差分
- `pixMinOrMax` - 最小/最大演算

## 設計方針

### ファイル構成

- 新規ファイル: `crates/leptonica-core/src/pix/arith.rs`

### API設計（既存パターンに従う）

```rust
/// 算術演算の型
pub enum ArithOp {
    Add,        // 加算
    Subtract,   // 減算
    Multiply,   // 乗算
    Divide,     // 除算
    AbsDiff,    // 絶対値差分
    Min,        // 最小値
    Max,        // 最大値
}

/// Pix への算術演算メソッド（新規画像を返す）
impl Pix {
    // 定数演算
    pub fn add_constant(&self, val: i32) -> Pix;
    pub fn multiply_constant(&self, val: f32) -> Result<Pix>;

    // 2画像演算
    pub fn add(&self, other: &Pix) -> Result<Pix>;
    pub fn subtract(&self, other: &Pix) -> Result<Pix>;
    pub fn multiply_gray(&self, gray: &Pix, norm: Option<f32>) -> Result<Pix>;
    pub fn abs_difference(&self, other: &Pix) -> Result<Pix>;
    pub fn min(&self, other: &Pix) -> Result<Pix>;
    pub fn max(&self, other: &Pix) -> Result<Pix>;
}

/// PixMut へのインプレース演算メソッド
impl PixMut {
    pub fn add_constant_inplace(&mut self, val: i32);
    pub fn multiply_constant_inplace(&mut self, val: f32) -> Result<()>;
    pub fn add_inplace(&mut self, other: &Pix) -> Result<()>;
    pub fn subtract_inplace(&mut self, other: &Pix) -> Result<()>;
}
```

### 対応ピクセル深度

- 8bpp（グレースケール）: 主要サポート
- 32bpp（RGB）: チャンネル別処理
- 16bpp: 必要に応じてサポート

### クリッピング処理

- 8bpp: [0, 255] にクリップ
- 16bpp: [0, 65535] にクリップ
- 32bpp: チャンネルごとに [0, 255] にクリップ

## 実装手順

### Phase 1: 基本構造

1. `arith.rs` ファイル作成
2. `ArithOp` enum 定義
3. モジュール宣言を `mod.rs` に追加

### Phase 2: 定数演算

1. `add_constant` / `add_constant_inplace` 実装
2. `multiply_constant` / `multiply_constant_inplace` 実装
3. 単体テスト作成

### Phase 3: 2画像演算

1. `add` / `add_inplace` 実装
2. `subtract` / `subtract_inplace` 実装
3. `abs_difference` 実装
4. 単体テスト作成

### Phase 4: 高度な演算

1. `multiply_gray` 実装（照明補正用）
2. `min` / `max` 実装
3. 単体テスト作成

### Phase 5: 品質保証

1. `cargo fmt` 実行
2. `cargo clippy` 実行
3. 全テスト実行

## テスト計画

- 各演算の正常系テスト
- 境界値テスト（クリッピング）
- 深度不一致エラーテスト
- サイズ不一致エラーテスト
- 32bpp RGB チャンネル別テスト

## 質問

特になし。

## 承認後の作業

1. `arith.rs` 実装
2. `mod.rs` 更新
3. `lib.rs` 更新（必要に応じて）
4. テスト実行
5. コミット作成
