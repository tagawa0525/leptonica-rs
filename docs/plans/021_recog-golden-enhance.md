# 021: recog モジュール回帰テスト golden manifest 強化

Status: IMPLEMENTED

Phase 3 PR 7/8: `tests/recog/` 配下の回帰テストに `write_pix_and_check()` を追加し、
golden manifest によるピクセルレベル回帰検出を有効化する。

## Context

Phase 3（回帰テスト強化）の第7弾。recog モジュールには22テストファイルがあり、
そのうち19ファイルが `RegParams` を使用するが、`write_pix_and_check` は0件。
現在の manifest（340エントリ）に recog エントリはゼロ。

親計画: `docs/plans/014_regression-test-enhance.md`

## 対象ファイル (15ファイル, 31 golden)

- `newspaper_reg.rs` — 4 golden
- `lineremoval_reg.rs` — 4 golden
- `flipdetect_reg.rs` — 1 golden
- `skew_reg.rs` — 1 golden
- `findcorners_reg.rs` — 1 golden
- `jbclass_reg.rs` — 3 golden
- `wordboxes_reg.rs` — 2 golden
- `findpattern1_reg.rs` — 2 golden
- `findpattern2_reg.rs` — 1 golden
- `partition_reg.rs` — 3 golden
- `pageseg_reg.rs` — 4 golden
- `pixadisp_reg.rs` — 3 golden
- `italic_reg.rs` — 2 golden

## 除外ファイル

- `correlscore_reg.rs`, `recog_coverage_reg.rs`, `strokes_reg.rs`, `finditalic_reg.rs` — RegParams 未使用
- `classapp_reg.rs`, `genfonts_reg.rs`, `nearline_reg.rs` — Pix 出力なし
- `dewarp_reg.rs` — NoContent fallback リスク
- `baseline_reg.rs` — 浮動小数点回転精度リスク
- JPEG ソースのテスト — デコーダ差異リスク
