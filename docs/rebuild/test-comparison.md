# C版 vs Rust版 テストケース比較

調査日: 2026-02-05（更新）

## 概要

| 項目             | C版 (reference/leptonica) | Rust版 (leptonica-rs) |
| ---------------- | ------------------------- | --------------------- |
| テスト総数       | **305個** (.c)            | **33ファイル**        |
| 回帰テスト       | **160個** (*_reg.c)       | なし                  |
| 個別テスト関数   | 多数                      | **217個**             |
| テストランナー   | alltests_reg.c            | `cargo test`          |

## C版テストの特徴

### 構造

- `prog/` ディレクトリに全テストが集約
- `*_reg.c`: 回帰テスト（160個）- ゴールデンファイルと比較
- その他: ユーティリティ/デモ/ベンチマーク（145個）

### カバー範囲（160分野）

| カテゴリ       | テスト数 | 内容                            |
| -------------- | -------- | ------------------------------- |
| 画像I/O        | 15+      | png, jpeg, gif, webp, tiff等    |
| モルフォロジー | 12       | binmorph1-6, graymorph1-2等     |
| 幾何変換       | 12       | affine, bilinear, projective等  |
| 色処理         | 12       | colorspace, colorquant等        |
| ブレンド       | 5        | blend1-5                        |
| 二値化         | 5        | binarize, dither, grayquant等   |
| 領域/Box       | 8        | boxa1-4, pixa1-2, conncomp等    |
| フィルタ       | 5        | convolve, edge, enhance等       |
| その他         | 多数     | dewarp, baseline, watershed等   |

### 回帰テスト一覧（160個）

```text
adaptmap, adaptnorm, affine, alltests, alphaops, alphaxform,
baseline, bilateral1, bilateral2, bilinear, binarize,
binmorph1-6, blackwhite, blend1-5, boxa1-4, bytea,
ccbord, ccthin1, ccthin2, checkerboard, circle, cmapquant,
colorcontent, colorfill, coloring, colorize, colormask,
colormorph, colorquant, colorseg, colorspace, compare,
compfilter, conncomp, conversion, convolve, crop, dewarp,
distance, dither, dna, dwamorph1, dwamorph2, edge, encoding,
enhance, equal, expand, extrema, falsecolor, fhmtauto, files,
findcorners, findpattern1, findpattern2, flipdetect, fmorphauto,
fpix1, fpix2, genfonts, gifio, grayfill, graymorph1, graymorph2,
grayquant, hardlight, hash, heap, insert, ioformats, iomisc,
italic, jbclass, jp2kio, jpegio, kernel, label, lineremoval,
locminmax, logicops, lowaccess, lowsat, maze, morphseq, mtiff,
multitype, nearline, newspaper, numa1, numa2, numa3, overlap,
pageseg, paint, paintmask, partition, pdfio1, pdfio2, pdfseg,
pixa1, pixa2, pixadisp, pixalloc, pixcomp, pixmem, pixserial,
pixtile, pngio, pnmio, projection, projective, psio, psioseg,
pta, ptra1, ptra2, quadtree, rank, rankbin, rankhisto, rasterop,
rasteropip, rectangle, rotate1, rotate2, rotateorth, scale,
seedspread, selio, shear1, shear2, skew, smallpix, smoothedge,
speckle, splitcomp, string, subpixel, texturefill, threshnorm,
translate, warper, watershed, webpanimio, webpio, wordboxes,
writetext, xformbox
```

## Rust版テストの現状

### 構造（Rust版）

- 各クレートの`src/*.rs`内に`#[cfg(test)]`モジュール
- 単体テストのみ、統合テストなし
- テストデータなし（in-memory生成のみ）

### テスト分布

| クレート            | ファイル           | テスト数 |
| ------------------- | ------------------ | -------- |
| leptonica-color     | analysis.rs        | 8        |
| leptonica-color     | colorspace.rs      | 9        |
| leptonica-color     | quantize.rs        | 7        |
| leptonica-color     | threshold.rs       | 9        |
| leptonica-core      | box_/mod.rs        | 7        |
| leptonica-core      | colormap/mod.rs    | 4        |
| leptonica-core      | pix/access.rs      | 7        |
| leptonica-core      | pix/mod.rs         | 7        |
| leptonica-core      | pta/mod.rs         | 7        |
| leptonica-filter    | convolve.rs        | 5        |
| leptonica-filter    | edge.rs            | 6        |
| leptonica-filter    | kernel.rs          | 4        |
| leptonica-io        | bmp.rs             | 2        |
| leptonica-io        | format.rs          | 7        |
| leptonica-io        | png.rs             | 2        |
| leptonica-io        | pnm.rs             | 2        |
| leptonica-morph     | binary.rs          | 7        |
| leptonica-morph     | sel.rs             | 6        |
| leptonica-recog     | baseline.rs        | 7        |
| leptonica-recog     | jbclass/classify.rs| 7        |
| leptonica-recog     | jbclass/types.rs   | 5        |
| leptonica-recog     | pageseg.rs         | 10       |
| leptonica-recog     | recog/did.rs       | 5        |
| leptonica-recog     | recog/ident.rs     | 5        |
| leptonica-recog     | recog/train.rs     | 7        |
| leptonica-recog     | recog/types.rs     | 5        |
| leptonica-recog     | skew.rs            | 9        |
| leptonica-region    | conncomp.rs        | 10       |
| leptonica-region    | label.rs           | 5        |
| leptonica-region    | seedfill.rs        | 7        |
| leptonica-region    | watershed.rs       | 6        |
| leptonica-transform | rotate.rs          | 15       |
| leptonica-transform | scale.rs           | 8        |
| **合計**            | **33ファイル**     | **217個**|

### クレート別集計

| クレート            | テスト数 | カバー範囲                            |
| ------------------- | -------- | ------------------------------------- |
| leptonica-color     | 33       | 色空間変換、分析、量子化、二値化      |
| leptonica-core      | 32       | Pix、Box、Colormap、Pta基本操作       |
| leptonica-filter    | 15       | 畳み込み、エッジ検出、カーネル        |
| leptonica-io        | 13       | BMP/PNG/PNM読み書き、フォーマット判定 |
| leptonica-morph     | 13       | 二値モルフォロジー、構造化要素        |
| leptonica-recog     | 60       | ページ分割、傾き検出、文字認識、JBIG2 |
| leptonica-region    | 28       | 連結成分、ラベリング、シードフィル    |
| leptonica-transform | 23       | 回転、スケーリング                    |

### テストがないクレート

- leptonica-doc（ドキュメント専用クレート）

## 主な差分

| 観点             | C版                    | Rust版                        |
| ---------------- | ---------------------- | ----------------------------- |
| **回帰テスト**   | ゴールデンファイル比較 | なし                          |
| **視覚テスト**   | 画像出力・目視確認     | なし                          |
| **I/Oテスト**    | 全フォーマット網羅     | BMP/PNG/PNMのみ               |
| **統合テスト**   | alltests_reg.c         | なし                          |
| **テストデータ** | 豊富（画像、PDF等）    | in-memory生成のみ             |
| **カバレッジ**   | 160分野                | 約15分野（8クレート）         |

## 推奨アクション

### 優先度高

1. **統合テスト追加**: `tests/`ディレクトリに回帰テスト
2. **ゴールデンファイル**: C版の出力を参照データとして利用
3. **テストデータ**: `reference/leptonica/prog/`の画像を活用

### 優先度中

1. **JPEG/TIFF I/O**: テスト追加が必要（現在BMP/PNG/PNMのみ）
2. **モルフォロジー**: グレースケールモルフォロジーのテスト追加
3. **幾何変換**: affine, bilinear, projective変換のテスト追加

### 優先度低

1. **ベンチマーク**: C版との性能比較
2. **視覚テスト**: 画像出力による確認機構

## 参考

- C版ソース: `reference/leptonica/prog/`
- C版テスト画像: `reference/leptonica/prog/*.{jpg,png,tif,...}`
- Rustテスト実行: `cargo test --workspace`
