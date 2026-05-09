# core/boxa: Boxaa::transpose の移植

Status: IN_PROGRESS
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 C)

## Context

C 版 `boxfunc2.c::boxaaTranspose()` (1781) は、Boxaa を行優先 2D 配列とみなして列優先に転置する。
全 Boxa が同じ box 数を持つ前提で、`baad[i][j] = baas[j][i]` を構築する。

要件:

- 入力 Boxaa が空ならエラー
- 各 Boxa の box 数が揃っていなければエラー
- 出力は input.outer_count × inner_count を入れ替えた新 Boxaa

## 配置先・API 設計

- ファイル: `src/core/box_/mod.rs` の `impl Boxaa` に追記
- API:

```rust
impl Boxaa {
    /// boxaaTranspose 相当
    pub fn transpose(&self) -> Result<Boxaa>;
}
```

エラー型は既存 `crate::core::Error` を使用（`InvalidArgument` 系）。

## TDD ステップ

1. **RED** (`test(core): boxaaTranspose の RED テスト`)
   - `tests/core/boxa4_reg.rs` の `#[ignore = "boxaaTranspose not implemented"]` (line 85) を unignore
   - 2x10 → 10x2 の転置と、サイズ不一致エラー、空エラーをアサート

2. **GREEN** (`feat(core): Boxaa::transpose を実装`)
   - 既存 `Boxaa::get_box(outer, inner)` を使った素直な 2 重ループ

3. **REFACTOR**: 不要

## テスト戦略

- 構築可能な Boxaa リテラルで往復テスト（transpose 2 回で同一）
- box 数不一致時のエラーケース
- 空 Boxaa のエラーケース

## ブランチ・PR

- ブランチ: `feat/core-boxaa-transpose`
- PR: `feat(core): port boxaaTranspose`
- 1PR、RED → GREEN

## ステータス

- [ ] RED コミット
- [ ] GREEN コミット
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新
