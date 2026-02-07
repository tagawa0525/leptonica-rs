# C版 Leptonica テスト一覧（移植目標）

調査日: 2026-02-05

## 概要

| 項目 | C版 (reference/leptonica) |
| ---- | ------------------------- |
| テスト総数 | **305個** (.c) |
| 回帰テスト | **160個** (*_reg.c) |
| その他（ユーティリティ/デモ/ベンチマーク） | **145個** |
| テストランナー | alltests_reg.c |

## C版テストの特徴

### 構造

- `prog/` ディレクトリに全テストが集約
- `*_reg.c`: 回帰テスト（160個）- ゴールデンファイルと比較
- その他: ユーティリティ/デモ/ベンチマーク（145個）

### カバー範囲（160分野）

| カテゴリ | テスト数 | 内容 |
| -------------- | -------- | ------------------------------- |
| 画像I/O | 15+ | png, jpeg, gif, webp, tiff等 |
| モルフォロジー | 12 | binmorph1-6, graymorph1-2等 |
| 幾何変換 | 12 | affine, bilinear, projective等 |
| 色処理 | 12 | colorspace, colorquant等 |
| ブレンド | 5 | blend1-5 |
| 二値化 | 5 | binarize, dither, grayquant等 |
| 領域/Box | 8 | boxa1-4, pixa1-2, conncomp等 |
| フィルタ | 5 | convolve, edge, enhance等 |
| その他 | 多数 | dewarp, baseline, watershed等 |

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

## 前回の実装から得た教訓

| 観点 | 問題 | 対策 |
| ---- | ---- | ---- |
| テストデータ | in-memory生成のみで実画像テストなし | `reference/leptonica/prog/` の画像を `tests/data/images/` に配置 |
| 回帰テスト | C版のゴールデンファイル比較に相当する仕組みがなかった | leptonica-test crateの3モード（Generate/Compare/Display）を活用 |
| 統合テスト | `#[cfg(test)]` モジュール内の単体テストのみ | `tests/` ディレクトリに統合テストを配置 |
| I/Oテスト | BMP/PNG/PNMのみカバー | 全フォーマットを網羅する |

## 参考

- C版テストソース: `reference/leptonica/prog/`
- C版テスト画像: `reference/leptonica/prog/*.{jpg,png,tif,...}`
