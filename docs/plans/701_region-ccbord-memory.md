# region/ccbord: feyn-fract.tif で OOM していた境界追跡の修正

Status: IMPLEMENTED
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 J)

## Context

`tests/region/ccbord_reg.rs::ccbord_reg_feyn_fract` は
"Rust ccbord has O(n_components × image_size) memory" との理由で
`#[ignore]` されていた。31 起草時はメモリ計算量の問題と分類していたが、
実際にテストを動かして backtrace を取ると別の原因が判明:

```text
memory allocation of 4294967296 bytes failed  (= 4 GB)
stack backtrace:
  ...
  trace_hole_border
  get_component_borders
  get_all_borders
  ccbord_reg_feyn_fract
```

`extract_component_image` は bbox-size の Pix しか確保しておらず、
メモリ計算量は実際には O(Σ bbox_size_i)。OOM の真因は:

- `trace_hole_border` の終了条件 `(px, py) == (start) && next == second`

  が緩く、ill-formed な hole で標準 Moore-tracing 終了条件が発火しない

- 結果としてループが無限に回り、`points: Vec<BorderPoint>` が

  4 GB 以上を要求して OOM

`trace_outer_border`（`get_outer_border` 内）にも同じ構造のループがあり、
同様の暴走リスクを抱えていた。

## 修正内容

両ループに、border の周長上限である `4 * (W + H) + 16` を `points.len()`
の上限として追加。これを超えたら `Err(RegionError::InvalidParameters(..))`
で打ち切る。

`get_component_borders` 内で `trace_hole_border` のエラーは既に
`Err(_) => continue` で吸収されているため、hole 単位での失敗は
コンポーネント全体を失敗にしない。`trace_outer_border` の失敗は
`get_all_borders` の `match` で 1 コンポーネントだけ skip される。

## 実装

- `src/region/ccbord.rs::trace_hole_border`: 行 1230 付近にループ

  上限チェックを追加

- `src/region/ccbord.rs::get_outer_border` (= 行 923 付近の outer

  tracer): 同じ上限チェックを追加

## テスト

- `tests/region/ccbord_reg.rs::ccbord_reg_feyn_fract` の `#[ignore]`

  を解除（元の理由がもはや当てはまらない）

- `cargo test --release` で feyn-fract.tif (1080×485, 1bpp の

  text-heavy 画像) が OOM せず通過することを確認

## ステータス

- [x] OOM 原因の特定 (バックトレース)
- [x] 上限チェックの追加
- [x] `#[ignore]` 解除
- [x] cargo test / clippy / fmt 通過
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新

## 残課題

- ill-formed hole が発生する根本原因（Moore tracing の終了条件設計か

  pixel-classification か）はこの PR では追跡していない。border 追跡が
  早期に打ち切られた場合、その component の hole 情報は欠落する

- 別 issue として「tracer の正しい終了条件」を将来再検討する余地あり
