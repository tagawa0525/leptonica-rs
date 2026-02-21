# C版 vs Rust版 テストケース比較

調査日: 2026-02-21（IO全移植計画完了を反映）

## 概要

| 項目             | C版 (reference/leptonica) | Rust版 (leptonica-rs) |
| ---------------- | ------------------------- | --------------------- |
| テスト総数       | **305個** (.c)            | **42+ファイル**       |
| 回帰テスト       | **160個** (*_reg.c)       | **9個** (IO回帰テスト)|
| 個別テスト関数   | 多数                      | **2,592個**           |
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

- 各クレートの`src/*.rs`内に`#[cfg(test)]`モジュール（単体テスト）
- `crates/leptonica-io/tests/`に統合テスト（9ファイル、C版`*_reg.c`に対応）
- テストデータ: `tests/data/images/`（実画像使用）
- テスト出力: `tests/regout/`（`.gitignore`対象、REGTEST_MODE=generateで生成）

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
| leptonica-io        | jpeg.rs            | 5+       |
| leptonica-io        | tiff.rs            | 6+       |
| leptonica-io        | spix.rs            | 5+       |
| leptonica-io        | header.rs          | 10+      |
| leptonica-io        | ps/mod.rs          | 19       |
| leptonica-io        | pdf.rs             | 10+      |
| leptonica-io (統合) | jpegio_reg.rs      | 2        |
| leptonica-io (統合) | spixio_reg.rs      | 2        |
| leptonica-io (統合) | pnmio_reg.rs       | 7        |
| leptonica-io (統合) | mtiff_reg.rs       | 10       |
| leptonica-io (統合) | iomisc_reg.rs      | 13       |
| leptonica-io (統合) | ioformats_reg.rs   | 1        |
| leptonica-io (統合) | pngio_reg.rs       | 1        |
| leptonica-io (統合) | gifio_reg.rs       | 2        |
| leptonica-io (統合) | webpio_reg.rs      | 3        |
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
| **合計**            | **42+ファイル**    | **2,592個**|

### クレート別集計

| クレート            | テスト数 | カバー範囲                            |
| ------------------- | -------- | ------------------------------------- |
| leptonica-core      | 1,372    | Pix、Box、Colormap、Pta、Numa、Pixa等 |
| leptonica-filter    | 250      | 畳み込み、エッジ検出、バイラテラル、ランク |
| leptonica-transform | 183      | 回転、スケーリング、アフィン、射影    |
| leptonica-morph     | 182      | 二値/グレースケール/カラー形態学、DWA |
| leptonica-color     | 164      | 色空間変換、分析、量子化、二値化      |
| leptonica-recog     | 156      | ページ分割、傾き検出、文字認識、JBIG2 |
| leptonica-io        | 150      | 全フォーマット読み書き、ヘッダー、回帰テスト |
| leptonica-region    | 131      | 連結成分、ラベリング、シードフィル    |
| leptonica-test      | 4        | テストインフラ                        |

### テストがないクレート

- leptonica-doc（ドキュメント専用クレート）

## 主な差分

| 観点             | C版                    | Rust版                        |
| ---------------- | ---------------------- | ----------------------------- |
| **回帰テスト**   | ゴールデンファイル比較 | ✅ RegParams + goldenファイル  |
| **視覚テスト**   | 画像出力・目視確認     | REGTEST_MODE=displayで対応    |
| **I/Oテスト**    | 全フォーマット網羅     | ✅ 全フォーマット対応          |
| **統合テスト**   | alltests_reg.c         | 9ファイル（IO回帰テスト）     |
| **テストデータ** | 豊富（画像、PDF等）    | tests/data/images/に実画像    |
| **カバレッジ**   | 160分野                | 9クレート、2,592テスト関数    |

## 推奨アクション

### 優先度高

1. ~~**統合テスト追加**: `tests/`ディレクトリに回帰テスト~~ ✅ IO回帰テスト完了
2. ~~**ゴールデンファイル**: C版の出力を参照データとして利用~~ ✅ RegParams実装済み
3. ~~**テストデータ**: `reference/leptonica/prog/`の画像を活用~~ ✅ tests/data/images/使用中
4. **非IO回帰テスト**: morph/transform/filter/color等の回帰テスト追加

### 優先度中

1. ~~**JPEG/TIFF I/O**: テスト追加が必要~~ ✅ jpegio_reg, mtiff_reg実装済み
2. **モルフォロジー**: グレースケールモルフォロジーの回帰テスト追加
3. **幾何変換**: affine, bilinear, projective変換の回帰テスト追加

### 優先度低

1. **ベンチマーク**: C版との性能比較
2. ~~**視覚テスト**: 画像出力による確認機構~~ ✅ REGTEST_MODE=display実装済み

## 参考

- C版ソース: `reference/leptonica/prog/`
- C版テスト画像: `reference/leptonica/prog/*.{jpg,png,tif,...}`
- Rustテスト実行: `cargo test --workspace`
