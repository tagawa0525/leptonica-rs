# core/fpix: 直交回転・反転の移植

Status: PLANNED
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 B)

## Context

C 版 `fpix2.c` のうち、FPix（浮動小数点画像）の直交回転・反転系が Rust に未移植。

| C 関数           | 行   | 役割                                      |
| ---------------- | ---- | ----------------------------------------- |
| `fpixRotateOrth` | 1706 | quads∈{0,1,2,3} で 90° 回転をディスパッチ |
| `fpixRotate90`   | 1778 | direction=±1 の 90° 回転                  |
| `fpixRotate180`  | 1747 | LR + TB flip                              |
| `fpixFlipLR`     | 1844 | 左右反転（in-place 可）                   |
| `fpixFlipTB`     | 後続 | 上下反転（in-place 可）                   |

Pix 側の `rotate_orth` は実装済み。FPix 側は座標移動のみで色変換不要なので素直に移植可能。

## 配置先・API 設計

- ファイル: 既存 `src/core/fpix/mod.rs` に追記、または分割して `src/core/fpix/transform.rs` を新設
- API:

```rust
impl FPix {
    /// fpixRotateOrth 相当（quads ∈ 0..=3, 時計回り回数）
    pub fn rotate_orth(&self, quads: u8) -> Result<FPix>;
    pub fn rotate_90(&self, direction: RotateDirection) -> Result<FPix>;
    pub fn rotate_180(&self) -> Result<FPix>;
    pub fn flip_lr(&self) -> Result<FPix>;
    pub fn flip_tb(&self) -> Result<FPix>;
}
```

`RotateDirection` は `transform` モジュールの既存 enum を流用（無ければ `core::fpix` 内に定義）。
in-place 版は提供せず、Rust では `&self → 新 FPix` のシンプル形に統一。

## TDD ステップ

1. **RED** (`test(core): fpixRotateOrth/Rotate90/FlipLR/FlipTB の RED テスト`)
   - `tests/core/fpix2_reg.rs` の `#[ignore = "fpix_rotate_orth not implemented"]` 等 5 件を unignore
   - 既存テストは「未実装で skip」状態 → 実装を呼ぶ形に書き換え

2. **GREEN** (`feat(core): FPix::rotate_orth/rotate_90/rotate_180/flip_lr/flip_tb`)
   - 上記 API を実装
   - row-major アクセスで wpl は 4-byte word 単位だが Rust では `Vec<f32>` ベースなので素直なインデックス計算

3. **REFACTOR**: 不要

## テスト戦略

- 既存 `fpix2_reg.rs` の golden manifest を `REGTEST_MODE=generate` で生成、再 compare で固定
- Pix 側 `rotate_orth` との一貫性検証: グレー Pix → FPix 変換 → fpix.rotate_orth → Pix 戻し が pix.rotate_orth と一致

## ブランチ・PR

- ブランチ: `feat/core-fpix-rotate-orth`
- PR: `feat(core): port fpix orthogonal rotation and flips`
- 1PR、RED → GREEN

## ステータス

- [ ] RED コミット
- [ ] GREEN コミット
- [ ] manifest 更新
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新
