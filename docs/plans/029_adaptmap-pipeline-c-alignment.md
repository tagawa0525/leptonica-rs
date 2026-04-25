# adaptmap pipeline 全体を C版と bit 同等に揃える

Status: PLANNED

## Context

Plan 028 で `fill_map_holes` 自体は C版 `pixFillMapHoles` と bit-identical に整列した。しかしその上流で動く pipeline 関数群はまだ Rust 独自実装で、C と bit-equivalent ではない。

PR #297 終了時点での実測値: `pixBackgroundNorm(dreyfus8.png)` 直接出力で **120,500 / 131,600 pixel (91.57%) が C と異なる**（max delta=45、avg=12.21）。
これにより、`tests/golden_manifest.tsv` で plan 028 PR2 が更新した下流テストの hash 群（`adaptmap_bg_*`, `adaptmap_contrast.*`, `adaptmap_gray_pipeline.*`, `binarize.*`, `binarize_double_norm.*`, `binarize_contrast_sauvola.*`）は **「Rust 現在値の pin」であり「C 参照値」ではない** 状態にある。

本計画の目的は、pipeline の各内部関数を C 版と 1:1 移植化し、上記下流テストを **C 参照値で再固定** すること。

## Goal

| 関数 | C 版 | 完了条件 |
| --- | --- | --- |
| `get_background_gray_map_inner` | `pixGetBackgroundGrayMap` (adaptmap.c:876) | 同入力で **bit-identical Pix** |
| `get_background_rgb_map_inner` | `pixGetBackgroundRGBMap` (adaptmap.c:1071) | 同上 |
| `get_inv_background_map_inner` | `pixGetInvBackgroundMap` (adaptmap.c:1857) | 同上 |
| `apply_inv_background_gray_map_inner` | `pixApplyInvBackgroundGrayMap` (adaptmap.c:1918) | 同上 |
| `apply_inv_background_rgb_map_inner` | `pixApplyInvBackgroundRGBMap` (adaptmap.c:1982) | 同上 |
| `min_max_tiles` | `pixMinMaxTiles` (adaptmap.c:2655) | 同上 |
| `set_low_contrast` | `pixSetLowContrast` (adaptmap.c:2744) | 同上 |
| `linear_trc_tiled` | `pixLinearTRCTiled` (adaptmap.c:2825) | 同上 |
| (集約) `pixBackgroundNorm`, `pixContrastNorm` | （上記全部） | `verify_pipeline.c` で **両方 IDENTICAL** |

成功時に `tests/golden_manifest.tsv` の以下 hash 群が C 参照値（`scripts/verify_pipeline.c` 等で C 出力から採取した値）に置き換わる:

- `adaptmap_bg_gray.04.png`, `adaptmap_bg_color.06.png`, `adaptmap_bg_highlevel.04.jpg`, `adaptmap_bg_highlevel.07.jpg`
- `adaptmap_contrast.07.png`, `adaptmap_contrast.10.png`
- `adaptmap_gray_pipeline.{02,05,09,13}*`, `adaptmap_color_pipeline.{04..09,13,17}*`
- `binarize.04.tif`, `binarize.20.tif`, `binarize_double_norm.{01,04}.tif`, `binarize_contrast_sauvola.{01,02}.tif`

### 非Goal

- 上流関数（`pixCleanBackgroundToWhite`, `pixBackgroundNormFlex`, `pixGlobalNormRGB`, `pixThresholdSpreadNorm` 等）は Goal 外。必要に応じ別計画で扱う
- morph variants (`pixGetBackgroundGrayMapMorph`, `pixGetBackgroundRGBMapMorph`) は使用箇所が限定的なので別 PR・別計画で扱う
- 現在の Rust API シグネチャ（`fn ... -> FilterResult<Pix>` 等）の変更はしない。あくまで内部実装の C 整列

## C版アルゴリズムの構造

### `pixGetBackgroundGrayMap` (lines 876-1069)

8bpp grayscale → 8bpp tile-resolution background map。

主要手順:

1. `pixDownsampleLineAvg` 等で `pixb` (binary mask of fg pixels) を生成
2. 入力を tile size `(sx, sy)` で割り、各 tile につき:
   - tile 内 pixel のうち non-fg な値の合計と count を集計
   - count >= mincount なら `pixd[i,j] = sum/count`、それ未満なら 0
3. `pixim` (image mask) があれば該当 tile を 0 に
4. `pixFillMapHoles(pixd, nx, ny, L_FILL_BLACK)` で埋める
5. `pixSmoothConnectedRegions(pixd, pixim, 2)` で smooth (画像マスク領域のみ)

Rust側の `get_background_gray_map_inner` は同概要だが、tile 内集計のループ構造、`pixb` 構築方法、smooth 処理が C と細部で違う可能性が高い。

### `pixGetBackgroundRGBMap` (lines 1071-1252)

3 channel 版。`pixGetBackgroundGrayMap` を内部で 3 回呼ぶ構造ではなく、独自に 3 plane を 1 pass で集計する。Rust 版がこの最適化に従っているか要確認。

### `pixGetInvBackgroundMap` (lines 1857-1916)

```text
input: bg_map (8bpp), bg_val (target), smooth_x, smooth_y
output: inv_map (16bpp)

1. pixSmoothMap = pixBlockconvGray(bg_map, smooth_x, smooth_y)  // smoothing
2. for each pixel: inv_map[i,j] = (256 * bg_val) / pixSmoothMap[i,j]
   特殊ケース: pixSmoothMap[i,j] == 0 → undefined (実装ではカットオフ等)
```

Rust側の特殊ケース fallback (`bg_val / 2`) が C と一致するか確認必要。

### `pixApplyInvBackgroundGrayMap` (lines 1918-1980)

```text
input: pixs (8bpp), inv_map (16bpp), tile_w, tile_h
output: 8bpp normalized

for each tile-region in pixs:
    factor = inv_map[ix, iy]  // 16-bit value
    for each pixel in tile:
        out[x, y] = min(255, (in[x, y] * factor) / 256)
```

### `pixMinMaxTiles` (lines 2655-2742)

contrast_norm の起点。各 tile の min/max を求め、`pixSetLowContrast` で diff < `mindiff` の tile を 0 に、`pixFillMapHoles` で埋め、smooth で滑らかに。

### `pixSetLowContrast` (lines 2744-2823)

`pix_min` と `pix_max` の同じ tile で `max - min < mindiff` なら両方を 0 に。

### `pixLinearTRCTiled` (lines 2825-end)

各 tile の min/max からスケール係数を求め、画像にタイル単位で line scale を適用。

## TDD サイクル（PR分割）

各 PR は **個別 verify ヘルパー + Rust テスト assertion + 実装書き換え + manifest 更新** を含む。プロセスは plan 028 と同じ:

1. `scripts/verify_<func>.c` を書いて C 出力ハッシュを採取
2. Rust テスト (`tests/filter/adaptmap_c_parity.rs` に追記) で hash 定数 + assertion を追加 → ローカルで RED 確認
3. Rust 実装を C 1:1 移植化 → assertion PASS
4. `REGTEST_MODE=generate` で関連 manifest hash を再生成
5. **再度 C と比較**して下流テストの hash も C 参照値であることを確認

### PR 1: `get_background_gray_map`

ブランチ: `feat/filter-bg-gray-map-c-aligned`

- `verify_bg_gray_map.c`: dreyfus8 / lucasta.150 で C 出力ハッシュ採取
- `c_parity_bg_gray_map_*` テスト追加
- `get_background_gray_map_inner` を C 1:1 移植
- `adaptmap_bg_gray.*` manifest 更新（C 値で）

### PR 2: `get_inv_background_map`

ブランチ: `feat/filter-inv-bg-map-c-aligned`

- `verify_inv_bg_map.c`
- `c_parity_inv_bg_map` テスト追加
- `get_inv_background_map_inner` C 整列
- 関連 manifest 更新

### PR 3: `apply_inv_background_gray_map`

ブランチ: `feat/filter-apply-inv-bg-gray-c-aligned`

- 上記2つが C 整列済みなので、これも揃えると `pixBackgroundNorm` 全体が C bit-identical になる予定
- `verify_bg_norm.c` で `pixBackgroundNorm(dreyfus8)` IDENTICAL を確認
- `adaptmap_bg_gray.04.png`, `adaptmap_bg_highlevel.*`, `adaptmap_gray_pipeline.*` manifest 更新（C 値で）

### PR 4: RGB variants

ブランチ: `feat/filter-bg-rgb-c-aligned`

- `pixGetBackgroundRGBMap`, `pixApplyInvBackgroundRGBMap` を整列
- color pipeline 系 manifest 更新

### PR 5: `min_max_tiles` + `set_low_contrast` + `linear_trc_tiled`

ブランチ: `feat/filter-contrast-norm-c-aligned`

- `pixContrastNorm` 系を C 整列
- `adaptmap_contrast.*`, `adaptmap_gray_pipeline.*`（contrast 経路）, `binarize.04`, `binarize.20`, `binarize_double_norm.*`, `binarize_contrast_sauvola.*` manifest 更新

### PR 6 (任意): morph variants

`pixGetBackgroundGrayMapMorph`, `pixGetBackgroundRGBMapMorph` も整列したくなった時に。

## リスク

1. **C 内部 helper の depth**: `pixGetBackgroundGrayMap` は `pixDownsampleLineAvg`, `pixSmoothConnectedRegions` 等の C 関数を呼ぶ。これらが Rust 側で未実装なら、本計画の中で同時に C 整列移植する必要がある（scope が広がる）
2. **浮動小数点の丸め**: `pixGetInvBackgroundMap` は整数除算 + `min(255)` 等の整数演算なので bit-identical 化は容易。ただし smoothing の中間状態が float なら丸めの差が出る可能性
3. **`pixim` 引数**: 多くの関数が `PIX *pixim` (image mask) を取るが、Rust API では `Option<&Pix>` で渡している。null/None semantic が C と一致しているか個別確認
4. **`pixSmoothConnectedRegions` の有無**: Rust 側に同等関数が無い場合、`get_background_gray_map_inner` の最後の smooth ステップが C と異なる → bit-equiv 不可。要先行調査

## 完了条件

- [ ] `scripts/verify_pipeline.c`（または個別 verify_*.c 群）で `pixBackgroundNorm(dreyfus8)`, `pixContrastNorm` その他主要 pipeline が **両ケース IDENTICAL**
- [ ] `tests/filter/adaptmap_c_parity.rs` に各 pipeline 関数の C 参照ハッシュ assertion を追加（5本以上）
- [ ] `tests/golden_manifest.tsv` の対象エントリ全てが C 参照値で更新済み
- [ ] `cargo test --all-features` 全 PASS
- [ ] `cargo clippy --all-features --all-targets -- -D warnings` clean
- [ ] PR 5 (or 最終 PR) 後に本計画書 Status を `IMPLEMENTED` に

## 参考

- C 実装: `reference/leptonica/src/adaptmap.c` lines 876-2900 周辺
- 関連 plan: 028 (`fill_map_holes` 単体整列、本計画の前提)
- 既存 verify ヘルパー: `scripts/verify_fillmapholes.c`, `scripts/verify_findbaselines.c`
- 関連 PR: #293 (parity infra), #297 (fill_map_holes alignment)
