# 未実装関数の実装（カバレッジ向上）

**Status: IMPLEMENTED**

## 概要

009_coverage-classification.mdで分類された❌未実装489関数を実装し、実カバレッジを向上させる。
モジュールごとに順次実装する（並列ブランチは使用しない）。

## 実装優先度（カバレッジ向上効率順）

| # | モジュール | ❌数 | 現在の実カバレッジ | 実装後見込み |
|---|-----------|------|-------------------|------------|
| 1 | filter | 6 | 93.2% | 100% |
| 2 | morph | 10 | 90.7% | 100% |
| 3 | transform | 16 | 87.0% | 100% |
| 4 | region | 25 | 65.8% | 100% |
| 5 | recog | 28 | 81.1% | 100% |
| 6 | color | 45 | 60.2% | 100% |
| 7 | io | 51 | 67.5% | 100% |
| 8 | misc | 78 | 12.4% | 100% |
| 9 | core | 230 | 70.3% | 100% |

## ワークフロー（各モジュール）

CLAUDE.mdのTDDルールに従う:

1. `feat/<module>-coverage` ブランチを作成
2. RED: テスト作成（`#[ignore = "not yet implemented"]`付き）
3. GREEN: 実装（`#[ignore]`除去）
4. `cargo test --all-features` + `cargo clippy` + `cargo fmt` 確認
5. push → PR作成
6. CI + Copilotレビュー待ち
7. レビューコメント対応
8. マージ → mainに戻る

## C版ソース参照

`reference/leptonica/src/` にC版ソースあり。

## 対象関数詳細

### filter (6関数)
- edge.c: pixTwoSidedEdgeFilter, pixMeasureEdgeSmoothness, pixGetEdgeProfile
- enhance.c: pixHalfEdgeByBandpass
- adaptmap.c: pixGetForegroundGrayMap
- rank.c: pixRankFilterWithScaling

### morph (10関数)
- morphapp.c: pixMorphSequenceByComponent, pixMorphSequenceByRegion
- selgen.c: pixGenerateSelBoundary, pixGenerateSelWithRuns, pixGenerateSelRandom, pixGetRunCentersOnLine, pixGetRunsOnLine, pixSubsampleBoundaryPixels
- sel1.c: selaCreateFromColorPixa
- ccthin.c: pixaThinConnected

### transform (16関数)
- rotate.c: pixEmbedForRotation, pixRotateBinaryNice
- scale1.c: pixScaleToSizeRel, pixScaleBySamplingToSize, pixScaleSmoothToSize, pixScaleAreaMap2, pixScaleAreaMapToSize, pixScaleBinaryWithShift, pixScaleGrayMinMax2, pixScaleGrayRank2, pixScaleWithAlpha
- affinecompose.c: ptaTranslate, ptaScale, boxaTranslate, boxaScale, boxaRotate

### region (25関数)
- conncomp.c: pixSeedfillBB/4BB/8BB, pixSeedfill/4/8
- ccbord.c: 16 CCBORDA functions
- pixlabel.c: pixConnCompIncrInit, pixConnCompIncrAdd, pixLocToColorTransform

### recog (28関数)
- recogbasic.c (2), recogident.c (5), recogtrain.c (7), pageseg.c (5), skew.c (2), dewarp1.c (2), jbclass.c (4), readbarcode.c (1)

### color (45関数)
- colorspace.c (7), colorquant1.c (10), colorquant2.c (1), colorseg.c (1), colorcontent.c (5), colorfill.c (1), coloring.c (2), binarize.c (6), grayquant.c (9), colormap.c (3)

### io (51関数)
- pdfio1.c (15), pdfio2.c (4), psio1.c (8), psio2.c (9), tiffio.c (3), webpanimio.c (3), jp2kio.c (3), pngio.c (2), jpegio.c (1), readfile.c (1), writefile.c (2)

### misc (78関数)
- pixcomp.c (35), paintcmap.c (7), pixtiling.c (5), pixacc.c (8), correlscore.c (4), binexpand.c/binreduce.c (6), encoding.c (3), pdfapp.c (3), warper.c (1), pixlabel.c (6)

### core (230関数)
- pix2.c (26), pix3.c (16), pix4.c (20), pix5.c (21), boxfunc2.c (14), boxfunc3.c (7), boxfunc4.c (11), boxfunc5.c (10), ptafunc1.c (20), ptafunc2.c (8), pixabasic.c (4), pixafunc1.c (16), pixafunc2.c (7), numabasic.c (10), numafunc1.c (11), numafunc2.c (6), sarray1.c (6), sarray2.c (5), fpix1.c (6), fpix2.c (4), colormap.c (18), pixconv.c (12), pixarith.c (3), compare.c (8), blend.c (5), graphics.c (2)
