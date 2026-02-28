# C版 vs Rust版 回帰テスト比較

調査日: 2026-02-23（全クレートの実ファイル配置に基づく正確な状態を反映）

## 概要

C版の `prog/*_reg.c` とRust版の `crates/*/tests/*_reg.rs` の対応関係。

| 項目           | C版 (reference/leptonica) | Rust版 (leptonica-rs) |
| -------------- | ------------------------- | --------------------- |
| テスト総数     | **305個** (.c)            | **162ファイル**       |
| 回帰テスト     | **160個** (*_reg.c)       | **140個** (*_reg.rs)  |
| 個別テスト関数 | 多数                      | **3,270個**           |
| テストランナー | alltests_reg.c            | `cargo test`          |

※ C版160個のうち `alltests_reg.c` はテストランナーのため集計から除外（159個が対象）。
※ C版テストの分類は、Rust側のテストファイルの実際の配置先クレートに基づく。

## 全テスト対応表

凡例:

- ✅ C版と同名のRust回帰テストが存在
- ❌ 未移植

### leptonica (src/core/)（Pix, Box, Numa, FPix, Pta, Pixa等）

※ encoding→io、expand/multitype/smallpix→transform、rectangle→regionへ再分類。overlap←regionから移動。

| C版テスト  | Rust対応          | 状態 |
| ---------- | ----------------- | ---- |
| boxa1      | boxa1_reg.rs      | ✅   |
| boxa2      | boxa2_reg.rs      | ✅   |
| boxa3      | boxa3_reg.rs      | ✅   |
| boxa4      | boxa4_reg.rs      | ✅   |
| bytea      | -                 | ❌   |
| compare    | compare_reg.rs    | ✅   |
| conversion | conversion_reg.rs | ✅   |
| dna        | -                 | ❌   |
| equal      | equal_reg.rs      | ✅   |
| extrema    | extrema_reg.rs    | ✅   |
| fpix1      | fpix1_reg.rs      | ✅   |
| fpix2      | fpix2_reg.rs      | ✅   |
| hash       | -                 | ❌   |
| heap       | -                 | ❌   |
| insert     | insert_reg.rs     | ✅   |
| logicops   | logicops_reg.rs   | ✅   |
| lowaccess  | lowaccess_reg.rs  | ✅   |
| numa1      | numa1_reg.rs      | ✅   |
| numa2      | numa2_reg.rs      | ✅   |
| numa3      | numa3_reg.rs      | ✅   |
| overlap    | overlap_reg.rs    | ✅   |
| pixa1      | pixa1_reg.rs      | ✅   |
| pixa2      | pixa2_reg.rs      | ✅   |
| pixalloc   | -                 | ❌   |
| pixcomp    | pixcomp_reg.rs    | ✅   |
| pixmem     | -                 | ❌   |
| pixserial  | pixserial_reg.rs  | ✅   |
| pta        | pta_reg.rs        | ✅   |
| ptra1      | -                 | ❌   |
| ptra2      | -                 | ❌   |
| rasterop   | rasterop_reg.rs   | ✅   |
| rasteropip | rasteropip_reg.rs | ✅   |
| string     | -                 | ❌   |

Rust独自: boxfunc, numa_sort_interp, pix_arith_rop, pix_clip_advanced, pix_clip_advanced_ext, pix_histogram_advanced, pix_stats_advanced, pixafunc

✅ 24 / ❌ 9（C版33個中）

### leptonica (src/io/)（画像I/O）

※ encoding←coreから移動。

| C版テスト  | Rust対応         | 状態 |
| ---------- | ---------------- | ---- |
| encoding   | encoding_reg.rs  | ✅   |
| files      | -                | ❌   |
| gifio      | gifio_reg.rs     | ✅   |
| ioformats  | ioformats_reg.rs | ✅   |
| iomisc     | iomisc_reg.rs    | ✅   |
| jp2kio     | jp2kio_reg.rs    | ✅   |
| jpegio     | jpegio_reg.rs    | ✅   |
| mtiff      | mtiff_reg.rs     | ✅   |
| pdfio1     | pdfio1_reg.rs    | ✅   |
| pdfio2     | pdfio2_reg.rs    | ✅   |
| pdfseg     | pdfseg_reg.rs    | ✅   |
| pixtile    | pixtile_reg.rs   | ✅   |
| pngio      | pngio_reg.rs     | ✅   |
| pnmio      | pnmio_reg.rs     | ✅   |
| psio       | psio_reg.rs      | ✅   |
| psioseg    | psioseg_reg.rs   | ✅   |
| webpanimio | -                | ❌   |
| webpio     | webpio_reg.rs    | ✅   |
| writetext  | -                | ❌   |

Rust独自: spixio

✅ 16 / ❌ 3（C版19個中）

### leptonica (src/morph/)（形態学演算）

| C版テスト  | Rust対応          | 状態 |
| ---------- | ----------------- | ---- |
| binmorph1  | binmorph1_reg.rs  | ✅   |
| binmorph2  | binmorph2_reg.rs  | ✅   |
| binmorph3  | binmorph3_reg.rs  | ✅   |
| binmorph4  | binmorph4_reg.rs  | ✅   |
| binmorph5  | binmorph5_reg.rs  | ✅   |
| binmorph6  | binmorph6_reg.rs  | ✅   |
| ccthin1    | ccthin1_reg.rs    | ✅   |
| ccthin2    | ccthin2_reg.rs    | ✅   |
| colormorph | colormorph_reg.rs | ✅   |
| dwamorph1  | dwamorph1_reg.rs  | ✅   |
| dwamorph2  | dwamorph2_reg.rs  | ✅   |
| fhmtauto   | fhmtauto_reg.rs   | ✅   |
| fmorphauto | fmorphauto_reg.rs | ✅   |
| graymorph1 | graymorph1_reg.rs | ✅   |
| graymorph2 | graymorph2_reg.rs | ✅   |
| morphseq   | morphseq_reg.rs   | ✅   |
| selio      | selio_reg.rs      | ✅   |

Rust独自: sel_morphapp

✅ 17 / ❌ 0（C版17個中）

### leptonica (src/transform/)（幾何変換）

※ expand/multitype/smallpix←coreから移動。

| C版テスト    | Rust対応            | 状態 |
| ------------ | ------------------- | ---- |
| affine       | affine_reg.rs       | ✅   |
| alphaxform   | alphaxform_reg.rs   | ✅   |
| bilinear     | bilinear_reg.rs     | ✅   |
| checkerboard | checkerboard_reg.rs | ✅   |
| circle       | circle_reg.rs       | ✅   |
| crop         | crop_reg.rs         | ✅   |
| expand       | expand_reg.rs       | ✅   |
| multitype    | multitype_reg.rs    | ✅   |
| projection   | projection_reg.rs   | ✅   |
| projective   | projective_reg.rs   | ✅   |
| rotate1      | rotate1_reg.rs      | ✅   |
| rotate2      | rotate2_reg.rs      | ✅   |
| rotateorth   | rotateorth_reg.rs   | ✅   |
| scale        | scale_reg.rs        | ✅   |
| shear1       | shear1_reg.rs       | ✅   |
| shear2       | shear2_reg.rs       | ✅   |
| smallpix     | smallpix_reg.rs     | ✅   |
| subpixel     | subpixel_reg.rs     | ✅   |
| translate    | translate_reg.rs    | ✅   |
| warper       | warper_reg.rs       | ✅   |
| xformbox     | xformbox_reg.rs     | ✅   |

✅ 21 / ❌ 0（C版21個中）

### leptonica (src/filter/)（フィルタリング）

※ lowsat←colorから移動。

| C版テスト  | Rust対応          | 状態 |
| ---------- | ----------------- | ---- |
| adaptmap   | adaptmap_reg.rs   | ✅   |
| adaptnorm  | adaptnorm_reg.rs  | ✅   |
| bilateral1 | bilateral1_reg.rs | ✅   |
| bilateral2 | bilateral2_reg.rs | ✅   |
| compfilter | compfilter_reg.rs | ✅   |
| convolve   | convolve_reg.rs   | ✅   |
| edge       | edge_reg.rs       | ✅   |
| enhance    | enhance_reg.rs    | ✅   |
| kernel     | kernel_reg.rs     | ✅   |
| locminmax  | locminmax_reg.rs  | ✅   |
| lowsat     | lowsat_reg.rs     | ✅   |
| rank       | rank_reg.rs       | ✅   |
| rankbin    | rankbin_reg.rs    | ✅   |
| rankhisto  | rankhisto_reg.rs  | ✅   |

Rust独自: adaptmap_advanced, adaptmap_bg, adaptmap_morph, bilateral_fast, extend_replication

✅ 14 / ❌ 0（C版14個中）

### leptonica (src/color/)（色処理・二値化・ブレンド）

※ blend1〜5はRust版でleptonica (src/color/) に実装。lowsat→filterへ再分類。

| C版テスト    | Rust対応            | 状態 |
| ------------ | ------------------- | ---- |
| alphaops     | alphaops_reg.rs     | ✅   |
| binarize     | binarize_reg.rs     | ✅   |
| blackwhite   | blackwhite_reg.rs   | ✅   |
| blend1       | blend1_reg.rs       | ✅   |
| blend2       | blend2_reg.rs       | ✅   |
| blend3       | blend3_reg.rs       | ✅   |
| blend4       | blend4_reg.rs       | ✅   |
| blend5       | blend5_reg.rs       | ✅   |
| cmapquant    | cmapquant_reg.rs    | ✅   |
| colorcontent | colorcontent_reg.rs | ✅   |
| colorfill    | colorfill_reg.rs    | ✅   |
| coloring     | coloring_reg.rs     | ✅   |
| colorize     | colorize_reg.rs     | ✅   |
| colormask    | colormask_reg.rs    | ✅   |
| colorquant   | colorquant_reg.rs   | ✅   |
| colorseg     | colorseg_reg.rs     | ✅   |
| colorspace   | colorspace_reg.rs   | ✅   |
| dither       | dither_reg.rs       | ✅   |
| falsecolor   | -                   | ❌   |
| grayquant    | grayquant_reg.rs    | ✅   |
| hardlight    | hardlight_reg.rs    | ✅   |
| paint        | paint_reg.rs        | ✅   |
| paintmask    | paintmask_reg.rs    | ✅   |
| threshnorm   | threshnorm_reg.rs   | ✅   |

Rust独自: binarize_advanced, color_magnitude, colorcontent_advanced, colorspace_hsv, quantize_ext

✅ 23 / ❌ 1（C版24個中）

### leptonica (src/region/)（領域解析）

※ overlap→coreへ移動。rectangle←coreから移動。

| C版テスト   | Rust対応          | 状態 |
| ----------- | ----------------- | ---- |
| ccbord      | ccbord_reg.rs     | ✅   |
| conncomp    | conncomp_reg.rs   | ✅   |
| distance    | distance_reg.rs   | ✅   |
| grayfill    | grayfill_reg.rs   | ✅   |
| label       | label_reg.rs      | ✅   |
| maze        | maze_reg.rs       | ✅   |
| quadtree    | quadtree_reg.rs   | ✅   |
| rectangle   | rectangle_reg.rs  | ✅   |
| seedspread  | seedspread_reg.rs | ✅   |
| smoothedge  | -                 | ❌   |
| speckle     | speckle_reg.rs    | ✅   |
| splitcomp   | -                 | ❌   |
| texturefill | -                 | ❌   |
| watershed   | watershed_reg.rs  | ✅   |

Rust独自: conncomp_ext, seedfill_ext

✅ 11 / ❌ 3（C版14個中）

### leptonica (src/recog/)（認識・ページ解析）

| C版テスト    | Rust対応            | 状態 |
| ------------ | ------------------- | ---- |
| baseline     | baseline_reg.rs     | ✅   |
| dewarp       | dewarp_reg.rs       | ✅   |
| findcorners  | -                   | ❌   |
| findpattern1 | findpattern1_reg.rs | ✅   |
| findpattern2 | findpattern2_reg.rs | ✅   |
| flipdetect   | flipdetect_reg.rs   | ✅   |
| genfonts     | -                   | ❌   |
| italic       | italic_reg.rs       | ✅   |
| jbclass      | jbclass_reg.rs      | ✅   |
| lineremoval  | lineremoval_reg.rs  | ✅   |
| nearline     | -                   | ❌   |
| newspaper    | newspaper_reg.rs    | ✅   |
| pageseg      | pageseg_reg.rs      | ✅   |
| partition    | partition_reg.rs    | ✅   |
| pixadisp     | pixadisp_reg.rs     | ✅   |
| skew         | skew_reg.rs         | ✅   |
| wordboxes    | wordboxes_reg.rs    | ✅   |

✅ 14 / ❌ 3（C版17個中）

## サマリ

### クレート別カバレッジ

| クレート                   | C版     | ✅      | ❌     | Rust独自 | カバレッジ |
| -------------------------- | ------- | ------- | ------ | -------- | ---------- |
| leptonica (src/core/)      | 33      | 24      | 9      | 8        | 72.7%      |
| leptonica (src/io/)        | 19      | 16      | 3      | 1        | 84.2%      |
| leptonica (src/morph/)     | 17      | 17      | 0      | 1        | 100.0%     |
| leptonica (src/transform/) | 21      | 21      | 0      | 0        | 100.0%     |
| leptonica (src/filter/)    | 14      | 14      | 0      | 5        | 100.0%     |
| leptonica (src/color/)     | 24      | 23      | 1      | 5        | 95.8%      |
| leptonica (src/region/)    | 14      | 11      | 3      | 2        | 78.6%      |
| leptonica (src/recog/)     | 17      | 14      | 3      | 0        | 82.4%      |
| **合計**                   | **159** | **140** | **19** | **22**   | **88.1%**  |

### 未移植テスト一覧（19個）

| クレート | テスト                                                         | 備考                           |
| -------- | -------------------------------------------------------------- | ------------------------------ |
| core     | bytea, dna, hash, heap, pixalloc, pixmem, ptra1, ptra2, string | データ構造・ユーティリティ     |
| io       | files, webpanimio, writetext                                   | ファイル操作・アニメーション   |
| color    | falsecolor                                                     | 疑似カラー                     |
| region   | smoothedge, splitcomp, texturefill                             | エッジ・分割・テクスチャ       |
| recog    | findcorners, genfonts, nearline                                | コーナー検出・フォント・近傍線 |

## Rust版テストの現状

### 構造（Rust版）

- 各クレートの`src/*.rs`内に`#[cfg(test)]`モジュール（単体テスト）
- `crates/*/tests/`に統合テスト（162ファイル、C版`*_reg.c`に対応）
- テストデータ: `tests/data/images/`（実画像使用）
- テスト出力: `tests/regout/`（`.gitignore`対象、REGTEST_MODE=generateで生成）

## 品質比較

| 観点             | C版                    | Rust版                           |
| ---------------- | ---------------------- | -------------------------------- |
| **回帰テスト**   | ゴールデンファイル比較 | ✅ RegParams + goldenファイル    |
| **視覚テスト**   | 画像出力・目視確認     | REGTEST_MODE=displayで対応       |
| **I/Oテスト**    | 全フォーマット網羅     | ✅ 全フォーマット対応            |
| **統合テスト**   | alltests_reg.c         | 162ファイル（全crate回帰テスト） |
| **テストデータ** | 豊富（画像、PDF等）    | tests/data/images/に実画像       |
| **カバレッジ**   | 159分野                | 8クレート、3,270テスト関数       |

## 参考

- C版ソース: `reference/leptonica/prog/*_reg.c`
- Rust版回帰テスト: `crates/*/tests/*_reg.rs`
- 回帰テストモード: `REGTEST_MODE={generate,compare,display}`
- goldenファイル: `tests/golden/`（コミット対象）
- テスト出力: `tests/regout/`（.gitignore対象）
