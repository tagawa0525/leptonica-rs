# 回帰テスト網羅性監査レポート

生成日: 2026-03-01

## サマリー

| 分類                  | テスト数 | 説明                             |
| --------------------- | -------- | -------------------------------- |
| A: 等価               | 39       | C版と同等のカバレッジ            |
| B: 部分的             | 84       | 修正可能な不足あり               |
| C: 不足（未実装依存） | 18       | 必要な関数がRustに未実装         |
| D: 設計差異           | 8        | C版と1:1対応しない正当な理由あり |
| **合計**              | **149**  |                                  |

## 全テスト一覧

> **注**: "C checks" / "Rust checks" 列の数値はLLMによる意味的解析結果であり、
> `scripts/audit-regression-tests.py` が生成するCSVの機械的カウント（正規表現による
> 呼び出し数）とは異なる場合があります。LLM解析は関数の意味（例: ループ内の複数
> 呼び出し、暗黙の検証）も考慮するため、CSVとの差異は設計上の意図によるものです。
> 機械的カウントは `docs/porting/regression-test-audit.csv` を参照してください。

| test         | 分類 | C checks | Rust checks | Rust ignored | bit一致 | 欠落チェック                         | 未実装関数                                          | 備考                                         |
| ------------ | ---- | -------- | ----------- | ------------ | ------- | ------------------------------------ | --------------------------------------------------- | -------------------------------------------- |
| adaptmap     | B    | 16       | 18          | 1            | 未検証  | golden image比較なし                 | pixGetBackgroundGrayMap, pixFillMapHoles            | compare_valuesのみで画像出力検証なし         |
| adaptnorm    | B    | 18       | 28          | 0            | 未検証  | Sauvola系テスト                      | pixSauvolaBinarize, pixSauvolaBinarizeTiled         | Sauvola二値化未実装                          |
| affine       | A    | 15       | 8           | 0            | 未検証  | none                                 | none                                                | sampling/interpolation invertability検証済み |
| alphaops     | A    | 15       | 8           | 0            | 未検証  | none                                 | none                                                | alpha操作全般カバー                          |
| alphaxform   | D    | 5        | 0           | 0            | 未検証  | 全チェック                           | pixScaleWithAlpha, pixRotateWithAlpha等             | テストファイル未作成、alpha付き変換未実装    |
| baseline     | B    | 17       | 8           | 0            | 未検証  | find_baselines多画像テスト           | find_baselines(部分)                                | deskew検証済み、baseline検出は最小限         |
| bilateral2   | A    | 8        | 16          | 0            | 未検証  | none                                 | none                                                | 全パラメータ組み合わせテスト済み             |
| bilinear     | A    | 33       | 18          | 0            | 未検証  | none                                 | none                                                | invertability/XOR差分で検証                  |
| binarize     | D    | 33       | 0           | 0            | 未検証  | 全チェック                           | pixSauvolaBinarize等                                | テストファイル未作成                         |
| binmorph1    | B    | 100      | 6           | 0            | 未検証  | 複数アルゴリズム実装比較             | none                                                | 性質検証のみ、実装間比較なし                 |
| binmorph3    | B    | 100      | 9           | 0            | 未検証  | DWA/separable比較                    | none                                                | identity/separability検証のみ                |
| binmorph6    | A    | 7        | 6           | 0            | 未検証  | none                                 | none                                                | dilate/open/close_safe/subtract検証済み      |
| blackwhite   | A    | 2        | 4           | 0            | 未検証  | none                                 | none                                                | 11画像で白黒ボーダー+alpha検証               |
| blend1       | B    | 16       | 6           | 0            | 未検証  | graywash/colorwash生成               | MakeGrayWash, MakeColorWash                         | 画像出力検証なし                             |
| blend2       | A    | 14       | 3           | 0            | 未検証  | none                                 | none                                                | RGB/grayscale/offsetブレンド検証済み         |
| blend3       | B    | 6        | 2           | 0            | 未検証  | pixBlend, pixBlendGray               | pixBlend, pixBlendColor                             | 基本blend関数不足                            |
| blend4       | B    | 5        | 3           | 0            | 未検証  | pixMirroredTiling                    | pixMirroredTiling                                   | パターン生成関数不足                         |
| blend5       | B    | 8        | 4           | 0            | 未検証  | linear_edge_fade全方向               | none                                                | snap_colorのみ集中テスト                     |
| boxa1        | C    | 9        | 10          | 0            | 未検証  | serialization, point array           | boxaWriteMem, boxaReadMem, boxaRead, boxaWrite      | シリアライズ/Pta操作未実装                   |
| boxa2        | A    | 6        | 22          | 0            | 未検証  | none                                 | none                                                | Box算術/Boxa操作を広範にカバー               |
| boxa3        | B    | 45       | 1           | 0            | 未検証  | 44チェック(可視化、中央値、一貫性)   | boxaMedianDimensions, boxaSizeConsistency等         | 読み込み/スケーリングのみ                    |
| boxa4        | B    | 14       | 1           | 0            | 未検証  | smoothing, reconciliation, transpose | boxaSmoothSequenceMedian等                          | split/mergeのみ                              |
| bytea        | C    | 8        | 2           | 0            | 未検証  | file I/O, join, search               | l_byteaInitFromFile, l_byteaJoin等                  | Base64/Pix圧縮のみテスト                     |
| ccbord       | A    | 16       | 8           | 0            | 未検証  | none                                 | none                                                | border取得/チェーンコードroundtrip検証済み   |
| ccthin1      | B    | 11       | 4           | 0            | 未検証  | 画像出力検証                         | selaDisplayInPix, pixThinConnected                  | SEL生成/プロパティのみ                       |
| ccthin2      | B    | 12       | 7           | 0            | 未検証  | 画像出力検証                         | pixThinConnectedBySet, pixaDisplayTiledAndScaled    | anti-extensive性質検証のみ                   |
| checkerboard | B    | 6        | 3           | 0            | 未検証  | テストデータ(checkerboard1/2.tif)    | none                                                | 合成パターンのみ                             |
| circle       | B    | 2        | 1           | 1            | 未検証  | circles.paテストデータ               | none                                                | smoke test + erode_brick                     |
| cmapquant    | B    | 9        | 6           | 0            | 未検証  | pixColorGray                         | pixColorGray(colormap版)                            | 合成画像フォールバック                       |
| colorcontent | B    | 19       | 12          | 0            | 未検証  | 画像出力/PDF生成                     | pixDisplayColorArray, pixGetMostPopulatedColors等   | 値レベル統計のみ                             |
| colorfill    | B    | 7        | 16          | 0            | 未検証  | visual output checks                 | l_colorfillCreate等                                 | seed/connectivity/統計検証済み               |
| coloring     | C    | 10       | 6           | 1            | 未検証  | colormap操作                         | pixcmapResetColor, pixAddSingleTextblock            | 32bpp RGB shift_by_componentのみ             |
| colorize     | B    | 21       | 8           | 0            | 未検証  | 複雑な画像処理パイプライン           | pixBackgroundNormSimple, pixGammaTRC等              | 基本color_gray/highlight検出のみ             |
| colormask    | D    | 11       | 0           | 1            | 未検証  | 全チェック                           | pixMakeHistoHS, pixFindHistoPeaksHSV等              | HSVヒストグラム/マスク生成未実装             |
| colormorph   | B    | 8        | 12          | 0            | 未検証  | sequence操作比較                     | pixColorMorphSequence                               | 数学的性質検証のみ                           |
| colorquant   | B    | 50       | 18          | 0            | 未検証  | 高度な量子化モード                   | pixMedianCutQuantGeneral, pixFixedOctcubeQuant256等 | 基本median_cut/octreeのみ                    |
| colorseg     | B    | 5        | 11          | 0            | 未検証  | tiled display/PDF                    | pixaDisplayTiledInColumns, pixMakeMaskFromVal       | セグメンテーション論理検証済み               |
| colorspace   | B    | 12       | 10          | 0            | 未検証  | colormap HSV変換                     | pixColorContent, pixcmapConvertRGBToHSV等           | ピクセル/画像レベル変換検証済み              |
| compare      | C    | 13       | 6           | 2            | 未検証  | correlation/translation              | pixBestCorrelation, pixCompareWithTranslation       | count_pixels/equals/perceptual_diffのみ      |
| compfilter   | B    | 26       | 4           | 2            | 未検証  | ratio-based filtering                | pixSelectByPerimToAreaRatio等                       | サイズ選択のみ                               |
| conncomp     | B    | 12       | 6           | 0            | 未検証  | boxa I/O, 色付き表示                 | pixaDisplay, pixaDisplayRandomCmap等                | カウントのみ、画像検証なし                   |
| conversion   | B    | 32       | 15          | 0            | 未検証  | 高度な量子化/colormap                | pixQuantizeIfFewColors                              | 基本深度変換マトリクスのみ                   |
| convolve     | B    | 11       | 5           | 0            | 未検証  | blockconv操作                        | pixBlockconvGray, pixBlockrank等                    | box_blur/gaussian/kernel畳み込みのみ         |
| crop         | C    | 9        | 3           | 1            | 未検証  | profile-based crop                   | pixReversalProfile, numaThresholdEdges等            | clip_rectangle_with_borderのみ               |
| dewarp       | B    | 21       | 6           | 0            | 未検証  | serialization/contour/multi-page     | dewarpWrite/Read, fpixWrite/Read等                  | single-pageパイプラインのみ                  |
| distance     | B    | 16       | 8           | 0            | 未検証  | contour/visualization                | pixMaxDynamicRange, pixRenderContours               | 8組み合わせ全て検証、可視化なし              |
| dither       | B    | 6        | 5           | 0            | 未検証  | 2bpp/scaled dithering                | pixGammaTRC                                         | gamma前処理/golden比較なし                   |
| dna          | B    | 9        | 7           | 0            | 未検証  | file I/O/histogram                   | l_dnaWrite, l_dnaRead                               | Numa/Numaaへの変換/統計検証済み              |
| dwamorph1    | A    | 28       | 28          | 0            | 未検証  | none                                 | none                                                | 7カーネル×4操作全比較済み                    |
| edge         | B    | 4        | 7           | 0            | 未検証  | threshold/binary output              | pixThresholdToBinary, pixMinOrMax                   | Sobel/Laplacian/sharpen/emboss検証済み       |
| encoding     | C    | 5        | 0           | 0            | 未検証  | 全チェック                           | encodeAscii85, decodeAscii85等                      | テストファイル未作成                         |
| enhance      | B    | 20       | 5           | 0            | 未検証  | sweep可視化                          | pixaDisplayTiledAndScaled, gplot等                  | identity test + 基本操作検証                 |
| equal        | B    | 17       | 5           | 0            | 未検証  | octree量子化                         | pixOctreeQuantNumColors, pixConvertRGBToColormap    | binary I/O/colormap除去/深度変換             |
| expand       | A    | 31       | 10          | 0            | 未検証  | none                                 | none                                                | 深度別+power-of-2+subsample検証              |
| extrema      | B    | 2        | 6           | 0            | 未検証  | gplot出力                            | none                                                | hysteresis delta検出実装済み                 |
| falsecolor   | B    | 8        | 7           | 0            | 未検証  | gamma別gradient出力                  | pixConvertGrayToFalseColor                          | color mapping/shift検証済み                  |
| fhmtauto     | B    | 20       | 3           | 1            | 未検証  | auto-gen DWA比較                     | pixFHMTGen_1, pixHMTDwa_1                           | 基本HMT + sel set検証のみ                    |
| files        | C    | 45       | 4           | 0            | 未検証  | path utilities全般                   | pathJoin, lept_rmdir等                              | pixa_read/write_filesのみ                    |
| findcorners  | C    | 1        | 3           | 0            | 未検証  | morph操作/barcode                    | pixUnionOfMorphOps, SELA等                          | 合成checkerboard検証のみ                     |
| findpattern1 | B    | 6        | 3           | 1            | 未検証  | template生成/表示                    | pixGenerateSelBoundary等                            | 基本HMTテストのみ                            |
| findpattern2 | C    | 7        | 3           | 0            | 未検証  | SEL生成メソッド3種                   | pixGenerateSelBoundary/WithRuns/Random              | 手動SELによるHMTのみ                         |
| flipdetect   | C    | 14       | 0           | 0            | 未検証  | 全チェック                           | pixOrientDetect, pixMirrorDetect等                  | テストファイル未作成                         |
| fpix1        | A    | 8        | 29          | 0            | 未検証  | none                                 | none                                                | FPix API包括テスト                           |
| fpix2        | D    | 8        | 0           | 0            | 未検証  | 全チェック                           | fpixRotateOrth, fpixAddMirroredBorder等             | 無関係なFPix/DPixテストのみ                  |
| genfonts     | C    | 27       | 2           | 0            | 未検証  | font生成/エンコード全般              | pixaSaveFont, pixaGetFont, encodeBase64             | Recog訓練のみ                                |
| gifio        | B    | 8        | 2           | 1            | 未検証  | palette保存テスト                    | none                                                | lossless roundtrip検証済み                   |
| graymorph1   | A    | 42       | 13          | 0            | 未検証  | none                                 | none                                                | gray morph全操作+duality検証                 |
| graymorph2   | B    | 12       | 6           | 0            | 未検証  | opening/closing比較                  | none                                                | monotonicity検証のみ                         |
| grayquant    | A    | 50       | 4           | 0            | 未検証  | none                                 | none                                                | threshold/量子化全般カバー                   |
| grayfill     | B    | 35       | 13          | 0            | 未検証  | 中間画像出力                         | pixCombineMasked等                                  | seedfill正確性検証済み、画像I/Oなし          |
| hardlight    | A    | 18       | 6           | 0            | 未検証  | none                                 | none                                                | 8bpp/32bpp/colormap全組み合わせ              |
| hash         | B    | 8        | 2           | 0            | 未検証  | set操作(union/intersection等)        | none                                                | color counting/hashbox描画のみ               |
| heap         | B    | 6        | 3           | 0            | 未検証  | NUMA serialization                   | none                                                | sorted insert/median/sort検証のみ            |
| insert       | B    | 11       | 3           | 0            | 未検証  | file-based golden比較                | none                                                | 合成データでのinsert/remove検証              |
| ioformats    | B    | 10       | 6           | 2            | 未検証  | header reading                       | pixReadHeader                                       | 4形式の基本検出のみ                          |
| iomisc       | B    | 32       | 8           | 2            | 未検証  | JPEG writer/colormap serial          | l_pngSetReadStrip16To8等                            | alpha/colormap/TIFF圧縮検証済み              |
| italic       | B    | 8        | 5           | 0            | 未検証  | pixItalicWords                       | pixItalicWords                                      | word mask/morph seq検証済み                  |
| jbclass      | C    | 4        | 0           | 0            | 未検証  | template render/reconstruct          | jbDataSave, jbDataRender                            | テスト不完全(50行で切断)                     |
| jp2kio       | C    | 17       | 1           | 4            | 未検証  | 全JP2K I/O                           | pixWriteJp2k, pixReadJp2k                           | format検出のみ、libopenjp2未対応             |
| jpegio       | A    | 4        | 5           | 0            | 未検証  | none                                 | none                                                | read/write/format detection検証済み          |
| kernel       | A    | 10       | 7           | 0            | 未検証  | none                                 | none                                                | kernel作成/I/O/convolution検証済み           |
| label        | A    | 6        | 11          | 0            | 未検証  | none                                 | none                                                | 4/8-connected labeling包括テスト             |
| lineremoval  | B    | 9        | 4           | 0            | 未検証  | interpolated rotation                | pixRotateAMGray, pixThresholdToValue                | skew検出/gray morph/combine_masked検証       |
| locminmax    | C    | 6        | 2           | 0            | 未検証  | extrema overlay                      | pixLocalExtrema(overlay)                            | blockconv/extrema検出のみ                    |
| logicops     | B    | 28       | 11          | 0            | 未検証  | write checks                         | pixOpenBrick, pixDilateBrick                        | 論理演算包括テスト、画像書き出しなし         |
| lowaccess    | A    | 28       | 8           | 0            | 未検証  | none                                 | none                                                | 全深度ピクセルアクセス検証済み               |
| lowsat       | C    | 7        | 0           | 0            | 未検証  | 全チェック                           | pixDarkenGray, pixModifyColorSaturation             | テストファイル未作成                         |
| maze         | A    | 6        | 3           | 0            | 未検証  | none                                 | none                                                | maze生成/探索/gray迷路検証済み               |
| morphseq     | —    | —        | —           | —            | —       | —                                    | —                                                   | alltests_regに含まれない                     |
| mtiff        | A    | 10       | 10          | 7            | 未検証  | none                                 | none                                                | multipage TIFF各圧縮検証済み                 |
| multitype    | D    | 17       | 0           | 1            | 未検証  | 全チェック                           | 全変換関数                                          | テストファイル未ポート                       |
| nearline     | C    | 6        | 3           | 0            | 未検証  | pixMinMaxNearLine                    | pixMinMaxNearLine                                   | 異なるrecog APIでテスト                      |
| newspaper    | B    | 13       | 5           | 2            | 未検証  | binary rank reduction                | pixReduceRankBinary2, pixExpandBinaryPower2         | morph pipeline検証済み                       |
| numa1        | A    | 11       | 28          | 0            | 未検証  | none                                 | none                                                | histogram/statistics包括テスト               |
| numa2        | B    | 38       | 8           | 0            | 未検証  | Pix統計(extraction/averaging)        | pixExtractOnLine, pixAverageByColumn等              | Numa API合成テストのみ                       |
| numa3        | B    | 13       | 11          | 1            | 未検証  | histogram rank抽出                   | pixGetGrayHistogramMasked等                         | smoothing/morph/threshold検証済み            |
| overlap      | A    | 13       | 6           | 2            | 未検証  | none                                 | none                                                | combine/contain/distance検証済み             |
| pageseg      | B    | 37       | 12          | 6            | 未検証  | debug画像/specialized関数            | pixFindPageForeground等                             | segment/textline/textblock検証済み           |
| paint        | B    | 22       | 4           | 1            | 未検証  | colormap painting                    | pixColorGrayCmap等                                  | 32bpp painting検証済み                       |
| paintmask    | A    | 22       | 3           | 0            | 未検証  | none                                 | none                                                | paint_through_mask/clip_masked検証済み       |
| partition    | B    | 4        | 4           | 0            | 未検証  | PDF出力                              | boxaGetWhiteblocks等                                | conncomp/dilation/box選択検証済み            |
| pdfio1       | B    | 27       | 12          | 3            | 未検証  | segmented PDF                        | convertToPdfSegmented等                             | 基本圧縮/multipage/title検証済み             |
| pdfio2       | C    | 20       | 3           | 4            | 未検証  | segmented PDF/batch                  | convertToPdfSegmented, concatenatePdf等             | 基本メモリ出力のみ                           |
| pdfseg       | D    | 8        | 2           | 2            | 未検証  | segmented PDF生成                    | convertSegmentedFilesToPdf等                        | 基本PDF出力のみ                              |
| pixa1        | A    | 2        | 24          | 0            | 未検証  | none                                 | none                                                | Pixa操作包括テスト                           |
| pixa2        | A    | 16       | 16          | 0            | 未検証  | none                                 | none                                                | 算術/ROP/ブレンド/統計包括テスト             |
| pixadisp     | A    | 6        | 6           | 0            | 未検証  | none                                 | none                                                | display_tiled/scaled検証済み                 |
| pixcomp      | B    | 15       | 8           | 0            | 未検証  | PDF生成/可視化                       | pixcompCreateFromPix(C式), pixacompConvertToPdfData | 基本PixComp操作検証済み                      |
| pixmem       | A    | 13       | 4           | 0            | 未検証  | none                                 | none                                                | SPIX serialization検証済み                   |
| pixserial    | A    | 40       | 3           | 0            | 未検証  | none                                 | none                                                | 10画像タイプでmemory/file roundtrip          |
| pngio        | A    | 12       | 6           | 0            | 未検証  | none                                 | none                                                | 1/8/32bpp roundtrip検証済み                  |
| pnmio        | A    | 12       | 7           | 3            | 未検証  | none                                 | none                                                | PBM/PGM/PPM/PAM roundtrip検証済み            |
| projection   | A    | 38       | 4           | 0            | 未検証  | none                                 | none                                                | column/row stats + 対称性検証                |
| projective   | A    | 24       | 5           | 0            | 未検証  | none                                 | none                                                | sampling/interpolation invertability検証     |
| psio         | A    | 11       | 6           | 3            | 未検証  | none                                 | none                                                | PS levels 1/2/3 + EPS検証済み                |
| psioseg      | C    | 5        | 4           | 3            | 未検証  | segmented PS生成                     | convertSegmentedPagesToPS等                         | 基本PSヘッダー検証のみ                       |
| pta          | B    | 20       | 12          | 0            | 未検証  | 画像描画/比較                        | pixRenderRandomCmapPtaa等                           | 数値プロパティ検証のみ                       |
| ptra1        | B    | 6        | 1           | 0            | 未検証  | clone/copy/removal                   | none                                                | 基本Ptaa操作のみ                             |
| ptra2        | B    | 10       | 5           | 0            | 未検証  | boxa/pixa sorting                    | none                                                | Ptaa init/flatten/pop/clear のみ             |
| quadtree     | A    | 9        | 18          | 0            | 未検証  | none                                 | none                                                | C版を超えるカバレッジ                        |
| rank         | A    | 11       | 10          | 0            | 未検証  | none                                 | none                                                | gray/color rank filter全般検証済み           |
| rankbin      | B    | 11       | 1           | 2            | 未検証  | discretization                       | numaGetRankBinValues等                              | rank_filter_grayのみ                         |
| rankhisto    | B    | 6        | 2           | 1            | 未検証  | rank color array                     | pixGetRankColorArray等                              | rank_filter_color/gamma検証のみ              |
| rasterop     | B    | 63       | 6           | 1            | 未検証  | 63要素dilation比較                   | pixDilate(region-based)                             | 基本rasteropプリミティブのみ                 |
| rasteropip   | B    | 2        | 5           | 1            | 未検証  | column/row copy                      | pixRasterop(region-based)                           | mirrored border検証済み                      |
| rectangle    | D    | 9        | 0           | 2            | 未検証  | 全チェック                           | pixFindLargestRectangle等                           | プレースホルダーテストのみ                   |
| rotate1      | A    | 8        | 8           | 0            | 未検証  | none                                 | none                                                | orth rotation/flip identity検証済み          |
| rotate2      | B    | 16       | 7           | 0            | 未検証  | method比較/多深度                    | pixRotate(各method)                                 | 角度回転検証、method比較なし                 |
| rotateorth   | A    | 4        | 12          | 0            | 未検証  | none                                 | none                                                | 全orthogonal identity検証済み                |
| scale        | B    | 50       | 9           | 0            | 未検証  | 全50画像出力チェック                 | pixScaleToGray3/4/6/8/16, pixScaleSmooth            | 基本寸法チェックのみ                         |
| seedspread   | B    | 6        | 4           | 0            | 未検証  | golden file比較                      | none                                                | seedspread論理検証済み、画像検証なし         |
| selio        | B    | 8        | 30          | 5            | 未検証  | SELA file I/O                        | selaWrite, selaRead, selaDisplayInPix               | SEL操作包括的、I/O未実装                     |
| shear1       | B    | 13       | 9           | 1            | 未検証  | multi-depth/colormap                 | pixOctreeColorQuant                                 | 8bpp/32bppのみ                               |
| shear2       | B    | 4        | 3           | 0            | 未検証  | quadratic shear出力                  | pixCreate(test patterns)                            | 合成パターンなし                             |
| skew         | B    | 3        | 4           | 0            | 未検証  | rotated image tests                  | pixReduceRankBinaryCascade                          | deskew検証済み、rotation test skip           |
| smallpix     | B    | 11       | 5           | 1            | 未検証  | golden file比較                      | none                                                | scaling/rotation寸法検証のみ                 |
| speckle      | B    | 15       | 2           | 0            | 未検証  | normalization/gamma/morph            | pixBackgroundNormFlex等                             | conncomp analysisのみ                        |
| splitcomp    | B    | 2        | 3           | 0            | 未検証  | visual split品質                     | pixSplitComponentIntoBoxa                           | 基本2-componentケースのみ                    |
| string       | B    | 19       | 3           | 0            | 未検証  | substring操作/file I/O               | none                                                | Sarray基本操作のみ                           |
| subpixel     | D    | 9        | 0           | 2            | 未検証  | 全チェック                           | pixConvertGrayToSubpixelRGB等                       | subpixel変換未実装                           |
| texturefill  | B    | 3        | 2           | 0            | 未検証  | 実画像テスト                         | none                                                | 合成リングパターンのみ                       |
| threshnorm   | B    | 1        | 2           | 0            | 未検証  | tiled display出力                    | none                                                | threshold_spread_norm個別検証のみ            |
| translate    | B    | 7        | 3           | 1            | 未検証  | multi-depth/golden                   | pixRemoveColormap                                   | 8bpp/32bpp正負シフトのみ                     |
| warper       | B    | 8        | 3           | 0            | 未検証  | golden file/CAPTCHA                  | pixRenderLineArb等                                  | harmonic warp再現性検証済み                  |
| watershed    | B    | 24       | 10          | 0            | 未検証  | 画像出力/比較                        | pixLocalExtrema, pixSetOrClearBorder等              | アルゴリズム正確性検証、画像検証なし         |
| webpanimio   | A    | 3        | 5           | 0            | 未検証  | none                                 | none                                                | WebPアニメーションI/O検証済み                |
| webpio       | B    | 4        | 3           | 2            | 未検証  | lossy quality/PSNR                   | pixWriteWebP(quality), pixGetPSNR                   | lossless roundtripのみ                       |
| wordboxes    | B    | 8        | 5           | 0            | 未検証  | word/char box抽出                    | pixGetWordsInTextlines等                            | word_mask_by_dilationのみ                    |
| writetext    | B    | 32       | 5           | 0            | 未検証  | multi-format/location                | none                                                | single 8bpp→text→PNGのみ                     |
| xformbox     | B    | 6        | 8           | 0            | 未検証  | hash box rendering                   | none                                                | transform論理検証、golden比較なし            |

## 分類別テスト一覧

### A: 等価 (39テスト)

affine, alphaops, bilateral2, bilinear, binmorph6, blackwhite, blend2, boxa2, ccbord, dwamorph1, expand, fpix1, graymorph1, grayquant, hardlight, jpegio, kernel, label, lowaccess, maze, mtiff, numa1, overlap, paintmask, pixa1, pixa2, pixadisp, pixmem, pixserial, pngio, pnmio, projection, projective, psio, quadtree, rank, rotate1, rotateorth, webpanimio

### B: 部分的（修正可能） (84テスト)

adaptmap, adaptnorm, baseline, binmorph1, binmorph3, blend1, blend3, blend4, blend5, boxa3, boxa4, ccthin1, ccthin2, checkerboard, circle, cmapquant, colorcontent, colorfill, colorize, colormorph, colorquant, colorseg, colorspace, compfilter, conncomp, conversion, convolve, dewarp, distance, dither, dna, edge, enhance, equal, extrema, falsecolor, fhmtauto, findpattern1, gifio, graymorph2, grayfill, hash, heap, insert, ioformats, iomisc, italic, lineremoval, logicops, newspaper, numa2, numa3, pageseg, paint, partition, pdfio1, pixcomp, pta, ptra1, ptra2, rankbin, rankhisto, rasterop, rasteropip, rotate2, scale, seedspread, selio, shear1, shear2, skew, smallpix, speckle, splitcomp, string, texturefill, threshnorm, translate, warper, watershed, webpio, wordboxes, writetext, xformbox

### C: 不足（未実装依存） (18テスト)

boxa1, bytea, coloring, compare, crop, encoding, files, findcorners, findpattern2, flipdetect, genfonts, jbclass, jp2kio, locminmax, lowsat, nearline, pdfio2, psioseg

### D: 設計差異 (8テスト)

alphaxform, binarize, colormask, fpix2, multitype, pdfseg, rectangle, subpixel

## 共通パターン

### B分類の主な不足理由

1. **golden file比較の欠如**: 多くのRustテストが`compare_values`（寸法/統計値チェック）のみで、C版の`regTestWritePixAndCheck`に相当するピクセル単位検証を行っていない
2. **合成データ使用**: C版が実画像を使うテストで、Rustは小さな合成画像を使用
3. **チェック数の不足**: C版の全チェックポイントの一部のみカバー（特にvisualization/display系は省略）
4. **パラメータ網羅性**: C版が複数パラメータ組み合わせをテストする箇所で、Rustは代表的な1-2パターンのみ

### C分類の主な原因

1. **未実装API**: Sauvola二値化、セグメントPDF、flip/orient検出、JP2K I/O等
2. **テストファイル未作成**: flipdetect, lowsat, encoding等
3. **異なるAPI設計**: nearline, findcorners等でC版と異なるアプローチを採用

### D分類の正当な理由

- **alphaxform/binarize/colormask/multitype**: 依存する高レベルAPI群が未実装
- **fpix2**: Rustテストが異なる機能をテスト（FPix回転/ボーダーではなく作成/変換）
- **rectangle/subpixel**: 専用アルゴリズム未実装
- **pdfseg**: セグメンテッドPDF生成インフラ未整備
