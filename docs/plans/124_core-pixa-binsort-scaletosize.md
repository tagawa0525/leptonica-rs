# Core: Pixa bin_sort / Pixaa scale_to_size_var (plan 032 残)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-3 (108b)

## 対象 C 関数 (2)

`pixafunc1.c` の Pixa/Pixaa 残課題から、既存実装を利用する形で
追加できる 2 関数を移植する。

- `pixaBinSort(pixas, sorttype, sortorder) -> (Pixa, Numa)` — O(n) bin sort で Pixa を並び替え、index Numa を返す (`pixaSort` が O(n log n) なのに対し、box dimension のみ 対象とする高速版)
- `pixaaScaleToSizeVar(paas, nawd, nahd) -> Pixaa` — 各 inner Pixa を per-image target サイズに揃える

## API 設計

```rust
impl Pixa {
    /// C: `pixaBinSort` (5 種類のキーのみ。それ以外は Err)
    pub fn bin_sort(
        &self,
        sort_type: PixaSortType,
        order: SortOrder,
    ) -> Result<(Pixa, Vec<usize>)>;
}

impl Pixaa {
    /// C: `pixaaScaleToSizeVar` (nawd/nahd 少なくとも一方は Some)
    pub fn scale_to_size_var(
        &self,
        nawd: Option<&Numa>,
        nahd: Option<&Numa>,
    ) -> Result<Pixaa>;
}
```

## 依存

- 既存 `Pixa::sort_by_index`、`Numa::bin_sort_index`、`Numa::push`
- 既存 `Pixa::scale_to_size`、`Numa::get_i32`
- 既存 `Pixaa::push`、`Pixaa::get`、`Pixaa::with_capacity`

## テスト方針

- bin_sort:
  - 5 種類のキー (ByX, ByY, ByWidth, ByHeight, ByPerimeter) で結果が `Pixa::sort` と一致
  - 不適合 (例: ByArea) で Err
  - 空 Pixa で空 + 空 indices
  - 降順 (`SortOrder::Decreasing`) で反転
- scale_to_size_var:
  - nawd のみ指定で各 Pixa の Pix が wd 揃え
  - nahd のみ指定で hd 揃え
  - 両方指定で wd × hd
  - nawd/nahd 共に None で Err
  - サイズミスマッチ (nawd.len() != n) で Err

## 完了条件

- [x] cargo test/clippy/fmt 通過 (9 件パス)
- [x] core.md 2 件 ❌ → ✅
- [x] plan 032 で 124 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- bin_sort: 内部で Numa を作って box dimension を集め、 `Numa::bin_sort_index` → `Pixa::sort_by_index` で完成
- scale_to_size_var: 各 inner で `Pixa::scale_to_size(wd_i, hd_i)` を呼び、Pixaa に push
