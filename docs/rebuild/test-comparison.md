# C版 vs Rust版 回帰テスト比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## 概要

C版の `prog/*_reg.c` とRust版の `crates/*/tests/*_reg.rs` の対応関係。

| 項目             | C版 (reference/leptonica) | Rust版 (leptonica-rs) |
| ---------------- | ------------------------- | --------------------- |
| テスト総数       | **305個** (.c)            | **80ファイル**        |
| 回帰テスト       | **160個** (*_reg.c)       | **80個** (*_reg.rs)  |
| 個別テスト関数   | 多数                      | **3,064個**（3,004 passed + 60 ignored）|
| テストランナー   | alltests_reg.c            | `cargo test`          |

※ C版160個のうち `alltests_reg.c` はテストランナーのため集計から除外。

## 全テスト対応表

凡例:
- ✅ C版と同名のRust回帰テストが存在
- ❌ 未移植

### leptonica-core（Pix, Box, Numa, FPix, Pta, Pixa等）

| C版テスト   | Rust対応       | 状態 |
| ----------- | -------------- | ---- |
| boxa1       | boxa1_reg.rs   | ✅   |
| boxa2       | boxa2_reg.rs   | ✅   |
| boxa3       | -              | ❌   |
| boxa4       | -              | ❌   |
| bytea       | -              | ❌   |
| compare     | -              | ❌   |
| conversion  | -              | ❌   |
| dna         | -              | ❌   |
| encoding    | -              | ❌   |
| equal       | -              | ❌   |
| expand      | -              | ❌   |
| extrema     | -              | ❌   |
| fpix1       | fpix1_reg.rs   | ✅   |
| fpix2       | fpix2_reg.rs   | ✅   |
| hash        | -              | ❌   |
| heap        | -              | ❌   |
| insert      | -              | ❌   |
| logicops    | -              | ❌   |
| lowaccess   | -              | ❌   |
| multitype   | -              | ❌   |
| numa1       | numa1_reg.rs   | ✅   |
| numa2       | numa2_reg.rs   | ✅   |
| numa3       | -              | ❌   |
| pixa1       | pixa1_reg.rs   | ✅   |
| pixa2       | pixa2_reg.rs   | ✅   |
| pixalloc    | -              | ❌   |
| pixcomp     | -              | ❌   |
| pixmem      | -              | ❌   |
| pixserial   | -              | ❌   |
| pta         | pta_reg.rs     | ✅   |
| ptra1       | -              | ❌   |
| ptra2       | -              | ❌   |
| rasterop    | -              | ❌   |
| rasteropip  | -              | ❌   |
| rectangle   | -              | ❌   |
| smallpix    | -              | ❌   |
| string      | -              | ❌   |

Rust独自: boxfunc, numa_sort_interp, pix_arith_rop, pix_clip_advanced, pix_clip_advanced_ext, pix_histogram_advanced, pix_stats_advanced, pixafunc

✅ 10 / ❌ 27（C版37個中）

### leptonica-io（画像I/O）

| C版テスト   | Rust対応         | 状態 |
| ----------- | ---------------- | ---- |
| files       | -                | ❌   |
| gifio       | gifio_reg.rs     | ✅   |
| ioformats   | ioformats_reg.rs | ✅   |
| iomisc      | iomisc_reg.rs    | ✅   |
| jp2kio      | -                | ❌   |
| jpegio      | jpegio_reg.rs    | ✅   |
| mtiff       | mtiff_reg.rs     | ✅   |
| pdfio1      | -                | ❌   |
| pdfio2      | -                | ❌   |
| pdfseg      | -                | ❌   |
| pixtile     | -                | ❌   |
| pngio       | pngio_reg.rs     | ✅   |
| pnmio       | pnmio_reg.rs     | ✅   |
| psio        | -                | ❌   |
| psioseg     | -                | ❌   |
| webpanimio  | -                | ❌   |
| webpio      | webpio_reg.rs    | ✅   |
| writetext   | -                | ❌   |

Rust独自: spixio

✅ 8 / ❌ 10（C版18個中）

### leptonica-morph（形態学演算）

| C版テスト   | Rust対応          | 状態 |
| ----------- | ----------------- | ---- |
| binmorph1   | binmorph1_reg.rs  | ✅   |
| binmorph2   | binmorph2_reg.rs  | ✅   |
| binmorph3   | binmorph3_reg.rs  | ✅   |
| binmorph4   | binmorph4_reg.rs  | ✅   |
| binmorph5   | binmorph5_reg.rs  | ✅   |
| binmorph6   | -                 | ❌   |
| ccthin1     | -                 | ❌   |
| ccthin2     | -                 | ❌   |
| colormorph  | colormorph_reg.rs | ✅   |
| dwamorph1   | dwamorph1_reg.rs  | ✅   |
| dwamorph2   | dwamorph2_reg.rs  | ✅   |
| fhmtauto    | -                 | ❌   |
| fmorphauto  | -                 | ❌   |
| graymorph1  | graymorph1_reg.rs | ✅   |
| graymorph2  | -                 | ❌   |
| morphseq    | morphseq_reg.rs   | ✅   |
| selio       | selio_reg.rs      | ✅   |

Rust独自: sel_morphapp

✅ 11 / ❌ 6（C版17個中）

### leptonica-transform（幾何変換）

| C版テスト    | Rust対応           | 状態 |
| ------------ | ------------------ | ---- |
| affine       | affine_reg.rs      | ✅   |
| alphaxform   | -                  | ❌   |
| bilinear     | bilinear_reg.rs    | ✅   |
| checkerboard | -                  | ❌   |
| circle       | -                  | ❌   |
| crop         | -                  | ❌   |
| projection   | -                  | ❌   |
| projective   | projective_reg.rs  | ✅   |
| rotate1      | rotate1_reg.rs     | ✅   |
| rotate2      | rotate2_reg.rs     | ✅   |
| rotateorth   | rotateorth_reg.rs  | ✅   |
| scale        | scale_reg.rs       | ✅   |
| shear1       | -                  | ❌   |
| shear2       | -                  | ❌   |
| subpixel     | -                  | ❌   |
| translate    | -                  | ❌   |
| warper       | -                  | ❌   |
| xformbox     | -                  | ❌   |

✅ 7 / ❌ 11（C版18個中）

### leptonica-filter（フィルタリング）

| C版テスト   | Rust対応          | 状態 |
| ----------- | ----------------- | ---- |
| adaptmap    | adaptmap_reg.rs   | ✅   |
| adaptnorm   | adaptnorm_reg.rs  | ✅   |
| bilateral1  | bilateral1_reg.rs | ✅   |
| bilateral2  | bilateral2_reg.rs | ✅   |
| compfilter  | -                 | ❌   |
| convolve    | convolve_reg.rs   | ✅   |
| edge        | edge_reg.rs       | ✅   |
| enhance     | -                 | ❌   |
| kernel      | -                 | ❌   |
| locminmax   | -                 | ❌   |
| rank        | rank_reg.rs       | ✅   |
| rankbin     | -                 | ❌   |
| rankhisto   | -                 | ❌   |

Rust独自: adaptmap_advanced, adaptmap_bg, adaptmap_morph, bilateral_fast, extend_replication

✅ 7 / ❌ 6（C版13個中）

### leptonica-color（色処理・二値化）

| C版テスト    | Rust対応             | 状態 |
| ------------ | -------------------- | ---- |
| alphaops     | -                    | ❌   |
| binarize     | binarize_reg.rs      | ✅   |
| blackwhite   | -                    | ❌   |
| cmapquant    | cmapquant_reg.rs     | ✅   |
| colorcontent | colorcontent_reg.rs  | ✅   |
| colorfill    | colorfill_reg.rs     | ✅   |
| coloring     | -                    | ❌   |
| colorize     | -                    | ❌   |
| colormask    | -                    | ❌   |
| colorquant   | colorquant_reg.rs    | ✅   |
| colorseg     | colorseg_reg.rs      | ✅   |
| colorspace   | colorspace_reg.rs    | ✅   |
| dither       | -                    | ❌   |
| falsecolor   | -                    | ❌   |
| grayquant    | -                    | ❌   |
| hardlight    | -                    | ❌   |
| lowsat       | -                    | ❌   |
| paint        | -                    | ❌   |
| paintmask    | -                    | ❌   |
| threshnorm   | -                    | ❌   |

Rust独自: binarize_advanced, color_magnitude, colorcontent_advanced, colorspace_hsv, quantize_ext

✅ 7 / ❌ 13（C版20個中）

### leptonica-region（領域解析）

| C版テスト   | Rust対応           | 状態 |
| ----------- | ------------------ | ---- |
| ccbord      | ccbord_reg.rs      | ✅   |
| conncomp    | conncomp_reg.rs    | ✅   |
| distance    | -                  | ❌   |
| grayfill    | -                  | ❌   |
| label       | label_reg.rs       | ✅   |
| maze        | -                  | ❌   |
| overlap     | -                  | ❌   |
| quadtree    | quadtree_reg.rs    | ✅   |
| seedspread  | seedspread_reg.rs  | ✅   |
| smoothedge  | -                  | ❌   |
| speckle     | -                  | ❌   |
| splitcomp   | -                  | ❌   |
| texturefill | -                  | ❌   |
| watershed   | watershed_reg.rs   | ✅   |

Rust独自: conncomp_ext, seedfill_ext

✅ 6 / ❌ 8（C版14個中）

### leptonica-recog（認識・ページ解析）

| C版テスト    | Rust対応         | 状態 |
| ------------ | ---------------- | ---- |
| baseline     | baseline_reg.rs  | ✅   |
| dewarp       | -                | ❌   |
| findcorners  | -                | ❌   |
| findpattern1 | -                | ❌   |
| findpattern2 | -                | ❌   |
| flipdetect   | -                | ❌   |
| genfonts     | -                | ❌   |
| italic       | -                | ❌   |
| jbclass      | -                | ❌   |
| lineremoval  | -                | ❌   |
| nearline     | -                | ❌   |
| newspaper    | -                | ❌   |
| pageseg      | pageseg_reg.rs   | ✅   |
| partition    | -                | ❌   |
| pixadisp     | -                | ❌   |
| skew         | skew_reg.rs      | ✅   |
| wordboxes    | -                | ❌   |

✅ 3 / ❌ 14（C版17個中）

### 分類未定（ブレンド等、Rust側に対応crateなし）

| C版テスト | 備考              | 状態 |
| --------- | ----------------- | ---- |
| blend1    | ブレンド（未実装）| ❌   |
| blend2    | ブレンド（未実装）| ❌   |
| blend3    | ブレンド（未実装）| ❌   |
| blend4    | ブレンド（未実装）| ❌   |
| blend5    | ブレンド（未実装）| ❌   |

❌ 5

## サマリ

### クレート別カバレッジ

| クレート            | C版 | ✅  | ❌  | Rust独自 | カバレッジ |
| ------------------- | --- | --- | --- | -------- | ---------- |
| leptonica-core      | 37  | 10  | 27  | 8        | 27.0%      |
| leptonica-io        | 18  | 8   | 10  | 1        | 44.4%      |
| leptonica-morph     | 17  | 11  | 6   | 1        | 64.7%      |
| leptonica-transform | 18  | 7   | 11  | 0        | 38.9%      |
| leptonica-filter    | 13  | 7   | 6   | 5        | 53.8%      |
| leptonica-color     | 20  | 7   | 13  | 5        | 35.0%      |
| leptonica-region    | 14  | 6   | 8   | 2        | 42.9%      |
| leptonica-recog     | 17  | 3   | 14  | 0        | 17.6%      |
| (分類未定)          | 5   | 0   | 5   | 0        | 0.0%       |
| **合計**            |**159**|**59**|**100**|**22** | **37.1%**  |

## Rust版テストの現状

### 構造（Rust版）

- 各クレートの`src/*.rs`内に`#[cfg(test)]`モジュール（単体テスト）
- `crates/*/tests/`に統合テスト（80ファイル、C版`*_reg.c`に対応）
- テストデータ: `tests/data/images/`（実画像使用）
- テスト出力: `tests/regout/`（`.gitignore`対象、REGTEST_MODE=generateで生成）

### クレート別テスト数

| クレート            | ファイル            | テスト数 |
| ------------------- | ------------------- | -------- |
| leptonica-color     | analysis.rs         | 8        |
| leptonica-color     | colorspace.rs       | 9        |
| leptonica-color     | quantize.rs         | 7        |
| leptonica-color     | threshold.rs        | 9        |
| leptonica-core      | box_/mod.rs         | 7        |
| leptonica-core      | colormap/mod.rs     | 4        |
| leptonica-core      | pix/access.rs       | 7        |
| leptonica-core      | pix/mod.rs          | 7        |
| leptonica-core      | pta/mod.rs          | 7        |
| leptonica-filter    | convolve.rs         | 5        |
| leptonica-filter    | edge.rs             | 6        |
| leptonica-filter    | kernel.rs           | 4        |
| leptonica-io        | bmp.rs              | 2        |
| leptonica-io        | format.rs           | 7        |
| leptonica-io        | png.rs              | 2        |
| leptonica-io        | pnm.rs              | 2        |
| leptonica-io        | jpeg.rs             | 5+       |
| leptonica-io        | tiff.rs             | 6+       |
| leptonica-io        | spix.rs             | 5+       |
| leptonica-io        | header.rs           | 10+      |
| leptonica-io        | ps/mod.rs           | 19       |
| leptonica-io        | pdf.rs              | 10+      |
| leptonica-io (統合) | jpegio_reg.rs       | 2        |
| leptonica-io (統合) | spixio_reg.rs       | 2        |
| leptonica-io (統合) | pnmio_reg.rs        | 7        |
| leptonica-io (統合) | mtiff_reg.rs        | 10       |
| leptonica-io (統合) | iomisc_reg.rs       | 13       |
| leptonica-io (統合) | ioformats_reg.rs    | 1        |
| leptonica-io (統合) | pngio_reg.rs        | 1        |
| leptonica-io (統合) | gifio_reg.rs        | 2        |
| leptonica-io (統合) | webpio_reg.rs       | 3        |
| leptonica-morph     | binary.rs           | 27       |
| leptonica-morph     | morphapp.rs         | 14       |
| leptonica-morph     | sel.rs              | 40       |
| leptonica-morph     | dwa.rs              | 22       |
| leptonica-morph     | sequence.rs         | 18       |
| leptonica-recog     | baseline.rs         | 7        |
| leptonica-recog     | barcode/detect.rs   | 5        |
| leptonica-recog     | barcode/signal.rs   | 8        |
| leptonica-recog     | dewarp/*.rs         | 40+      |
| leptonica-recog     | jbclass/classify.rs | 7        |
| leptonica-recog     | jbclass/io.rs       | 6        |
| leptonica-recog     | jbclass/types.rs    | 5        |
| leptonica-recog     | pageseg.rs          | 10       |
| leptonica-recog     | recog/did.rs        | 5        |
| leptonica-recog     | recog/ident.rs      | 5        |
| leptonica-recog     | recog/io.rs         | 6+       |
| leptonica-recog     | recog/train.rs      | 7        |
| leptonica-recog     | recog/types.rs      | 5        |
| leptonica-recog     | skew.rs             | 9        |
| leptonica-region    | conncomp.rs         | 10       |
| leptonica-region    | label.rs            | 5        |
| leptonica-region    | seedfill.rs         | 7        |
| leptonica-region    | watershed.rs        | 6        |
| leptonica-transform | rotate.rs           | 227 (合計) |
| leptonica-transform | scale.rs            | (上記に含む) |
| **合計**            | **80ファイル**      | **3,064個**（3,004 passed + 60 ignored）|

### クレート別テスト分類

| クレート            | テスト数 | カバー範囲                                                         |
| ------------------- | -------- | ------------------------------------------------------------------ |
| leptonica-core      | 1,169    | Pix、Box、Colormap、Pta、Numa、Pixa等                              |
| leptonica-filter    | 250      | 畳み込み、エッジ検出、バイラテラル、ランク                         |
| leptonica-transform | 227      | 回転、スケーリング、アフィン、射影                                 |
| leptonica-morph     | 211      | 二値/グレースケール/カラー形態学、DWA、SEL、Sela                   |
| leptonica-color     | 164      | 色空間変換、分析、量子化、二値化                                   |
| leptonica-recog     | 264      | ページ分割、傾き検出、文字認識、JBIG2、デワープ、バーコード        |
| leptonica-io        | 150      | 全フォーマット読み書き、ヘッダー、回帰テスト                       |
| leptonica-region    | 131      | 連結成分、ラベリング、シードフィル                                 |
| leptonica-test      | 4        | テストインフラ                                                     |

## 品質比較

| 観点             | C版                    | Rust版                         |
| ---------------- | ---------------------- | ------------------------------ |
| **回帰テスト**   | ゴールデンファイル比較 | ✅ RegParams + goldenファイル   |
| **視覚テスト**   | 画像出力・目視確認     | REGTEST_MODE=displayで対応     |
| **I/Oテスト**    | 全フォーマット網羅     | ✅ 全フォーマット対応           |
| **統合テスト**   | alltests_reg.c         | 80ファイル（全crate回帰テスト） |
| **テストデータ** | 豊富（画像、PDF等）    | tests/data/images/に実画像     |
| **カバレッジ**   | 160分野                | 9クレート、3,064テスト関数     |

## 参考

- C版ソース: `reference/leptonica/prog/*_reg.c`
- Rust版回帰テスト: `crates/*/tests/*_reg.rs`
- 回帰テストモード: `REGTEST_MODE={generate,compare,display}`
- goldenファイル: `tests/golden/`（コミット対象）
- テスト出力: `tests/regout/`（.gitignore対象）
