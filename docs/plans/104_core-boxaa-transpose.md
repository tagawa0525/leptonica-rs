# core/boxa: Boxaa::transpose の検証

Status: IMPLEMENTED
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 C)

## Context

C 版 `boxfunc2.c::boxaaTranspose()` (1781) は、Boxaa を行優先 2D 配列とみなして
列優先に転置する。031 起草時には未実装と分類していたが、実装着手時に
`src/core/box_/mod.rs:1216` に既に `Boxaa::transpose` が存在していたことが判明した。

つまり 031 の項目 C は「実装漏れ」ではなく「テストでカバーされていない実装済み機能」
だった。本計画は ignored regression test を unignore して回帰防止を担保することを
目的とする。

## 既存実装の所在

```rust
// src/core/box_/mod.rs:1216
impl Boxaa {
    pub fn transpose(&self) -> Result<Boxaa> { /* ... */ }
}
```

挙動:

- 空 Boxaa は `Error::InvalidParameter("boxaa is empty")`
- 各 Boxa の長さが揃っていなければ `Error::InvalidParameter("boxa[i] has ... boxes, expected ...")`
- それ以外は `baad[i][j] = baas[j][i]` の新 Boxaa を返す

## 変更内容

- `tests/core/boxa4_reg.rs::boxa4_reg_boxaa_transpose` の `#[ignore]` を解除
- 形状反転 / ラウンドトリップ / 空エラー / ragged エラー の 4 ケースをアサート
- `Boxaa::transpose` 自体への変更は **無し**

## 検証結果

```text
$ cargo test --test core boxa4_reg_boxaa_transpose
test boxa4_reg::boxa4_reg_boxaa_transpose ... ok
```

## 教訓

031 起草時の grep `pub fn boxaa_transpose` は構造体メソッド `Boxaa::transpose` を
取りこぼしていた。今後同様のロードマップ作成時は `fn <verb>` 形式でも検索する
（例: `fn transpose`, `fn rotate`）。

実際、改めて他 Group 1 項目 (B, A, H, E) を `fn <verb>` 形式で再検索した結果、
それらは本当に未実装であることを確認済み。

## ステータス

- [x] 取りこぼし発覚・既存実装の確認
- [x] regression test の unignore
- [x] cargo test 通過
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を更新
