# core/fpix: 直交回転・反転・境界拡張の移植

Status: IMPLEMENTED
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 B)

## Context

C 版 `fpix2.c` のうち、FPix（浮動小数点画像）の直交回転・反転・境界拡張系が
Rust に未移植。`tests/core/fpix2_reg.rs` には 5 件の `#[ignore]` テストが残っている。

| C 関数                   | 行   | 役割                                      |
| ------------------------ | ---- | ----------------------------------------- |
| `fpixRotateOrth`         | 1706 | quads∈{0,1,2,3} で 90° 回転をディスパッチ |
| `fpixRotate90`           | 1778 | direction=±1 の 90° 回転                  |
| `fpixRotate180`          | 1747 | LR + TB flip                              |
| `fpixFlipLR`             | 1844 | 左右反転                                  |
| `fpixFlipTB`             | 1894 | 上下反転                                  |
| `fpixAddBorder`          | -    | 単色（0 埋め）境界拡張                    |
| `fpixAddMirroredBorder`  | 1433 | ミラー境界拡張                            |
| `fpixAddContinuedBorder` | 1478 | エッジ値の延長境界拡張                    |

Pix 側の `rotate_orth` / `add_mirrored_border` / `add_continued_border` は既に実装済み。
FPix 側は座標移動と単純コピーのみで色変換不要なので素直に移植可能。

## 配置先・API 設計

- ファイル: `src/core/fpix/transform.rs` を新設（`mod.rs` から `pub mod transform`）
- API:

```rust
impl FPix {
    /// fpixRotateOrth 相当（quads ∈ 0..=3, 時計回り回数）
    pub fn rotate_orth(&self, quads: u8) -> Result<FPix>;
    /// fpixRotate90 相当
    pub fn rotate_90(&self, direction: RotateDirection) -> Result<FPix>;
    /// fpixRotate180 相当
    pub fn rotate_180(&self) -> Result<FPix>;
    /// fpixFlipLR 相当
    pub fn flip_lr(&self) -> Result<FPix>;
    /// fpixFlipTB 相当
    pub fn flip_tb(&self) -> Result<FPix>;

    /// fpixAddBorder 相当（fill 値で四方を拡張）
    pub fn add_border(&self, left: u32, right: u32, top: u32, bot: u32, fill: f32) -> Result<FPix>;
    /// fpixAddMirroredBorder 相当
    pub fn add_mirrored_border(&self, left: u32, right: u32, top: u32, bot: u32) -> Result<FPix>;
    /// fpixAddContinuedBorder 相当
    pub fn add_continued_border(&self, left: u32, right: u32, top: u32, bot: u32) -> Result<FPix>;
}
```

`RotateDirection` は新規に `core::fpix::RotateDirection { Cw, Ccw }` として定義。
in-place 版は提供せず、Rust では `&self → 新 FPix` のシンプル形に統一。

## TDD ステップ

1. **RED** (`test(core): FPix::rotate_orth/border の RED テスト`)
   - `tests/core/fpix2_reg.rs` の 5 件の `#[ignore]` を解除し、テスト本体を埋める
   - 実装は `unimplemented!()` の stub で公開し、テストは `#[ignore = "RED: ..."]` を付けたまま残す
2. **GREEN** (`feat(core): FPix 直交回転・反転・境界拡張を実装`)
   - 上記 API を本実装に置き換え、`#[ignore]` を解除
3. **REFACTOR**: 不要

## テスト戦略

- 既存 `fpix2_reg.rs` の 5 ケース:
  - 90/180/270 直交回転は Pix 側 `rotate_orth` の結果と f32 値で完全一致を確認
  - mirrored/continued border は Pix 側 `add_mirrored_border` / `add_continued_border` と

    形状・値の一致を確認（int → float キャストで誤差なく比較可）

- 必要に応じて `tests/golden_manifest.tsv` を `REGTEST_MODE=generate` で更新

## ブランチ・PR

- ブランチ: `feat/core-fpix-rotate-orth`
- PR: `feat(core): port FPix orthogonal rotation, flips, and border ops`
- 1PR、RED → GREEN

## ステータス

- [ ] RED コミット（stub + ignored test 内容追加）
- [ ] GREEN コミット（実装 + ignore 解除）
- [ ] cargo test / clippy / fmt 通過
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新
