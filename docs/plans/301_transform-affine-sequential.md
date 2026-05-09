# transform/affine: pixAffineSequential の移植

Status: IN_PROGRESS
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 H)

## Context

C 版 `affine.c::pixAffineSequential()` (1436, 152 行) は、3 点対応 (`ptas` → `ptad`) を
水平シア → 垂直シア → スケーリング → ラスター移動 → 逆シア の連続変換で近似的に
アフィン適用する関数。Notes に「文字には不向きだが 1bpp で 3x 高速」とあるとおり
教育的・速度重視のバリアント。

`pixAffineSampled` / `pixAffinePtaColor` 等の本流アフィンは Rust 既存 API でカバー
済みだが、Sequential 版だけ未移植。

## アルゴリズム概要

1. 必要に応じて入力に border を付加し、3 点座標も平行移動
2. ソース 3 点を x/y 軸上に乗せる水平・垂直シア角を `atan2` で計算
3. 同様にデスト 3 点を軸に乗せる角度（th3p, ph2p）を計算
4. ソース画像に H シア → V シアを in-place で適用 (`pixHShearIP`, `pixVShearIP`)
5. ソース→デスト軸スケール (`scalex = (x2sp - x1p) / (x2s - x1)` 等) で `pixScale`
6. スケール後の原点を dest 原点に合わせて `pixRasteropIP` で平行移動
7. `-ph2p`, `-th3p` で逆シア（順番も逆）
8. border 付加していたら `pixRemoveBorderGeneral` で除去

## 既存 Rust 依存（確認済み）

| 必要関数                 | Rust 実装                                       |
| ------------------------ | ----------------------------------------------- |
| `pixHShearIP`            | `src/transform/shear.rs::h_shear_ip`            |
| `pixVShearIP`            | `src/transform/shear.rs::v_shear_ip`            |
| `pixScale`               | `src/transform/scale.rs::scale`                 |
| `pixRasteropIP`          | `src/core/pix/rop.rs::Pix::rasterop_ip`         |
| `pixAddBorderGeneral`    | `src/core/pix/border.rs::add_border_general`    |
| `pixRemoveBorderGeneral` | `src/core/pix/border.rs::remove_border_general` |

## 配置先・API 設計

- ファイル: `src/transform/affine.rs` に追記
- API:

```rust
/// pixAffineSequential 相当
pub fn affine_sequential(
    pix: &Pix,
    ptad: &Pta,
    ptas: &Pta,
    bw: i32,
    bh: i32,
) -> TransformResult<Pix>;
```

エラーは `TransformError::InvalidArgument` で：

- ptas / ptad の点数が 3 でない
- y1 == y3 または y1p == y3p（degenerate）
- x2s == x1 / x2sp == x1p（degenerate）

## TDD ステップ

1. **RED** (`test(transform): pixAffineSequential の RED テスト`)
   - `tests/transform/affine_reg.rs` の `#[ignore = "pixAffineSequential not implemented..."]` を unignore
   - C 版テストの「順方向適用 → 逆方向適用 で原画像に戻る」（恒等写像近似）をアサート

2. **GREEN** (`feat(transform): port pixAffineSequential`)
   - 上記アルゴリズムを Rust に移植

3. **REFACTOR**: 不要

## テスト戦略

- 既存 `tests/transform/affine_reg.rs:326` の C 版「sequential affine 反転テスト」を

  そのまま再現（3 点を変換 → 逆 3 点で戻す → 元と RMS 比較）

- 結果は近似なので `compare_pix` の許容差を広めに

## ブランチ・PR

- ブランチ: `feat/transform-affine-sequential`
- PR: `feat(transform): port pixAffineSequential`
- 1PR、RED → GREEN

## ステータス

- [ ] RED コミット
- [ ] GREEN コミット
- [ ] manifest 更新
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新
