# 020: region モジュール回帰テスト golden manifest 強化

Status: IN_PROGRESS

Phase 3 PR 6/8: `tests/region/` 配下の回帰テストに `write_pix_and_check()` を追加し、
golden manifest によるピクセルレベル回帰検出を有効化する。

## Context

Phase 3（回帰テスト強化）の第6弾。region モジュールには19テストファイルがあり、
そのうち12ファイルが `RegParams` を使用するが、`write_pix_and_check` は0件。
現在の manifest（320エントリ）に region エントリはゼロ。

12ファイルのうち8ファイルが Pix 出力を持ち golden 化の対象となる。
残り4ファイル（conncomp, quadtree, maze, splitcomp）は数値比較のみ、
非決定的出力、または構造テストのため除外。

親計画: `docs/plans/014_regression-test-enhance.md`

## 対象ファイルと変更内容

### Commit 1: ccbord + distance (4 golden)

| ファイル          | テスト関数                       | 変数       | Format | 備考                                                  |
| ----------------- | -------------------------------- | ---------- | ------ | ----------------------------------------------------- |
| `ccbord_reg.rs`   | `ccbord_reg_dreyfus1_smoke`      | `rendered` | Tiff   | 1bpp, render_borders 出力                             |
| `distance_reg.rs` | `distance_reg_all_combos`        | `result`   | Png    | 8bpp, guard: `combo_idx == 0`（最初の組み合わせのみ） |
| `distance_reg.rs` | `distance_reg_seedfill_labeling` | `dist`     | Png    | 8bpp, distance_function 出力                          |
| `distance_reg.rs` | `distance_reg_seedfill_labeling` | `labeled`  | Png    | 8bpp, seedfill_gray 出力                              |

distance_reg_all_combos はネストループ（2×2×2=8組み合わせ）。フラットカウンタを導入し
最初のイテレーション（FourWay/Bit8/Background）のみキャプチャ。

### Commit 2: grayfill (4 golden)

| ファイル          | テスト関数                       | 変数      | Format | 備考                      |
| ----------------- | -------------------------------- | --------- | ------ | ------------------------- |
| `grayfill_reg.rs` | `grayfill_reg_inv`               | `result4` | Png    | 8bpp, seedfill_gray_inv   |
| `grayfill_reg.rs` | `grayfill_reg_standard`          | `result4` | Png    | 8bpp, seedfill_gray       |
| `grayfill_reg.rs` | `grayfill_reg_basin`             | `result4` | Png    | 8bpp, seedfill_gray_basin |
| `grayfill_reg.rs` | `grayfill_reg_hybrid_comparison` | `h4`      | Png    | 8bpp, hybrid 4-way のみ   |

全4テストで make_mask_200() による合成画像を使用。決定的出力。
各テストから 4-way 結果のみキャプチャ（8-way は構造的に同一操作）。

### Commit 3: label + speckle (4 golden)

| ファイル         | テスト関数                   | 変数       | Format | 備考                              |
| ---------------- | ---------------------------- | ---------- | ------ | --------------------------------- |
| `label_reg.rs`   | `label_reg`                  | `labeled4` | Png    | 32bpp, label_connected_components |
| `label_reg.rs`   | `label_reg`                  | `labeled8` | Png    | 32bpp, label_connected_components |
| `speckle_reg.rs` | `speckle_reg_clear_border`   | `cleared`  | Tiff   | 1bpp, clear_border 4-way          |
| `speckle_reg.rs` | `speckle_reg_select_by_size` | `filtered` | Tiff   | 1bpp, pix_select_by_size          |

label_reg には display モード分岐あり。非 display 分岐（L36以降）に追加。
speckle の count_components / full_pipeline は対象外。

### Commit 4: smoothedge + texturefill + watershed (8 golden)

| ファイル             | テスト関数                     | 変数          | Format | 備考                                |
| -------------------- | ------------------------------ | ------------- | ------ | ----------------------------------- |
| `smoothedge_reg.rs`  | `smoothedge_reg`               | `grad`        | Png    | 8bpp, compute_gradient              |
| `smoothedge_reg.rs`  | `smoothedge_reg`               | `seg`         | Png    | 32bpp, watershed_segmentation       |
| `texturefill_reg.rs` | `texturefill_reg`              | `holes`       | Tiff   | 1bpp, holes_by_filling              |
| `texturefill_reg.rs` | `texturefill_reg`              | `filled`      | Tiff   | 1bpp, fill_closed_borders           |
| `texturefill_reg.rs` | `texturefill_reg`              | `rect_filled` | Tiff   | 1bpp, fill_holes_to_bounding_rect   |
| `watershed_reg.rs`   | `do_watershed` (variant 0 & 1) | `segmented`   | Png    | 32bpp, Ok arm 内。2回呼出=2エントリ |
| `watershed_reg.rs`   | `watershed_gradient`           | `gradient`    | Png    | 8bpp, compute_gradient              |

### Commit 5: manifest 生成

```bash
REGTEST_MODE=generate cargo test --test region
```

## 除外ファイル

| ファイル                      | 理由                                            |
| ----------------------------- | ----------------------------------------------- |
| `conncomp_reg.rs`             | 値比較のみ（コンポーネント数）、Pix 出力なし    |
| `quadtree_reg.rs`             | 値比較のみ（領域数、平均、分散）、Pix 出力なし  |
| `maze_reg.rs`                 | ランダム迷路生成（`rand::rng()`）、非決定的出力 |
| `splitcomp_reg.rs`            | 構造テスト（Boxa/Pixa 返却）、個別 Pix 検証なし |
| `checkerboard_reg.rs`         | RegParams 未使用                                |
| `conncomp_ext_reg.rs`         | RegParams 未使用                                |
| `partition_whitespace_reg.rs` | RegParams 未使用                                |
| `region_coverage_reg.rs`      | RegParams 未使用                                |
| `seedfill_ext_reg.rs`         | RegParams 未使用                                |
| `seedspread_reg.rs`           | RegParams 未使用                                |
| `rectangle_reg.rs`            | 全テスト #[ignore]（未実装）                    |

## ブランチ・コミット規約

- ブランチ: `test/region-golden-enhance`
- コミット prefix: `test(region): enhance <files>_reg with write_pix_and_check`
- 最終コミット: `test(region): generate golden manifest for region regression tests`
