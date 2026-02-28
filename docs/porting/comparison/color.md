# leptonica (src/color/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-26（color全関数実装完了を反映）

## サマリー

| 項目      | 数  |
| --------- | --- |
| ✅ 同等   | 102 |
| 🔄 異なる | 20  |
| ❌ 未実装 | 0   |
| 🚫 不要   | 17  |
| 合計      | 139 |

## 詳細

### colorspace.c

#### color/colorspace.rs (colorspace.c)

| C関数                     | 状態 | Rust対応                                      | 備考                           |
| ------------------------- | ---- | --------------------------------------------- | ------------------------------ |
| pixConvertRGBToHSV        | ✅   | pix_convert_rgb_to_hsv                        |                                |
| pixConvertHSVToRGB        | ✅   | pix_convert_hsv_to_rgb                        |                                |
| convertRGBToHSV           | ✅   | rgb_to_hsv                                    |                                |
| convertHSVToRGB           | ✅   | hsv_to_rgb                                    |                                |
| pixcmapConvertRGBToHSV    | ✅   | pix_colormap_convert_rgb_to_hsv               | カラーマップ操作               |
| pixcmapConvertHSVToRGB    | ✅   | pix_colormap_convert_hsv_to_rgb               | カラーマップ操作               |
| pixConvertRGBToHue        | 🔄   | pix_extract_channel(ColorChannel::Hue)        | チャネル抽出として実装         |
| pixConvertRGBToSaturation | 🔄   | pix_extract_channel(ColorChannel::Saturation) | チャネル抽出として実装         |
| pixConvertRGBToValue      | 🔄   | pix_extract_channel(ColorChannel::Value)      | チャネル抽出として実装         |
| pixMakeRangeMaskHS        | ✅   | make_range_mask_hs                            | -                              |
| pixMakeRangeMaskHV        | ✅   | make_range_mask_hv                            | -                              |
| pixMakeRangeMaskSV        | ✅   | make_range_mask_sv                            | -                              |
| pixMakeHistoHS            | ✅   | make_histo_hs                                 | -                              |
| pixMakeHistoHV            | ✅   | make_histo_hv                                 | -                              |
| pixMakeHistoSV            | ✅   | make_histo_sv                                 | -                              |
| pixFindHistoPeaksHSV      | ✅   | find_histo_peaks_hsv                          | HSVヒストグラムピーク検出      |
| displayHSVColorRange      | 🚫   | -                                             | 表示/可視化関数                |
| pixConvertRGBToYUV        | ✅   | pix_convert_rgb_to_yuv                        | -                              |
| pixConvertYUVToRGB        | ✅   | pix_convert_yuv_to_rgb                        | -                              |
| convertRGBToYUV           | ✅   | rgb_to_yuv                                    |                                |
| convertYUVToRGB           | ✅   | yuv_to_rgb                                    |                                |
| pixcmapConvertRGBToYUV    | ✅   | pix_colormap_convert_rgb_to_yuv               | カラーマップ操作               |
| pixcmapConvertYUVToRGB    | ✅   | pix_colormap_convert_yuv_to_rgb               | カラーマップ操作               |
| pixConvertRGBToXYZ        | ✅   | pix_convert_rgb_to_xyz                        | (FPix, FPix, FPix)タプルで返却 |
| fpixaConvertXYZToRGB      | ✅   | fpixa_convert_xyz_to_rgb                      | (FPix, FPix, FPix)タプルを入力 |
| convertRGBToXYZ           | ✅   | rgb_to_xyz                                    |                                |
| convertXYZToRGB           | ✅   | xyz_to_rgb                                    |                                |
| fpixaConvertXYZToLAB      | ✅   | fpixa_convert_xyz_to_lab                      | (FPix, FPix, FPix)タプルで操作 |
| fpixaConvertLABToXYZ      | ✅   | fpixa_convert_lab_to_xyz                      | (FPix, FPix, FPix)タプルで操作 |
| convertXYZToLAB           | ✅   | xyz_to_lab                                    |                                |
| convertLABToXYZ           | ✅   | lab_to_xyz                                    |                                |
| pixConvertRGBToLAB        | ✅   | pix_convert_rgb_to_lab                        | (FPix, FPix, FPix)タプルで返却 |
| fpixaConvertLABToRGB      | ✅   | fpixa_convert_lab_to_rgb                      | (FPix, FPix, FPix)タプルを入力 |
| convertRGBToLAB           | ✅   | rgb_to_lab                                    |                                |
| convertLABToRGB           | ✅   | lab_to_rgb                                    |                                |
| pixMakeGamutRGB           | 🚫   | -                                             | 表示/可視化関数                |

### colorquant1.c

#### color/quantize.rs (colorquant1.c)

| C関数                         | 状態 | Rust対応                       | 備考                                            |
| ----------------------------- | ---- | ------------------------------ | ----------------------------------------------- |
| pixOctreeColorQuant           | 🔄   | octree_quant                   | Rust版は `OctreeOptions` ベース                 |
| pixOctreeColorQuantGeneral    | 🔄   | octree_quant                   | Rust版はオプション統合API                       |
| pixOctreeQuantByPopulation    | ✅   | octree_quant_by_population     | -                                               |
| pixOctreeQuantNumColors       | ✅   | octree_quant_num_colors        | -                                               |
| pixOctcubeQuantMixedWithGray  | ✅   | octcube_quant_mixed_with_gray  | Mixed量子化                                     |
| pixFixedOctcubeQuant256       | ✅   | fixed_octcube_quant_256        | -                                               |
| pixFewColorsOctcubeQuant1     | ✅   | few_colors_octcube_quant1      | Few colors量子化                                |
| pixFewColorsOctcubeQuant2     | ✅   | few_colors_octcube_quant2      | Few colors量子化                                |
| pixFewColorsOctcubeQuantMixed | ✅   | few_colors_octcube_quant_mixed | Few colors mixed量子化                          |
| pixFixedOctcubeQuantGenRGB    | ✅   | fixed_octcube_quant_gen_rgb    | 固定Octcube量子化                               |
| pixQuantFromCmap              | ✅   | quant_from_cmap                | -                                               |
| pixOctcubeQuantFromCmap       | ✅   | octcube_quant_from_cmap        | Octcube量子化                                   |
| pixOctcubeQuantFromCmapLUT    | 🔄   | octcube_quant_from_cmap_lut    | C版はstatic。Rust版は公開APIとして提供          |
| makeRGBToIndexTables          | 🚫   | -                              | C版LUT専用ヘルパー                              |
| getOctcubeIndexFromRGB        | 🚫   | -                              | C版LUT専用ヘルパー                              |
| getRGBFromOctcube             | 🚫   | -                              | C版LUT専用ヘルパー                              |
| pixOctcubeHistogram           | 🔄   | octcube_tree                   | C版はNUMAヒストグラム。Rust版はツリー構造を返す |
| pixcmapToOctcubeLUT           | 🚫   | -                              | C版LUT専用ヘルパー                              |
| pixRemoveUnusedColors         | ✅   | remove_unused_colors           | -                                               |
| pixNumberOccupiedOctcubes     | ✅   | number_occupied_octcubes       | 占有Octcube数計算                               |

### colorquant2.c

#### color/quantize.rs (colorquant2.c)

| C関数                           | 状態 | Rust対応                          | 備考                           |
| ------------------------------- | ---- | --------------------------------- | ------------------------------ |
| pixMedianCutQuant               | 🔄   | median_cut_quant_simple           | アルゴリズムの詳細が異なる     |
| pixMedianCutQuantGeneral        | 🔄   | median_cut_quant                  | パラメータ構造が異なる         |
| pixMedianCutQuantMixed          | ✅   | median_cut_quant_mixed            | -                              |
| pixFewColorsMedianCutQuantMixed | ✅   | few_colors_median_cut_quant_mixed | Few colors mixed量子化         |
| pixMedianCutHisto               | 🚫   | -                                 | 内部実装の詳細（ヘルパー関数） |

### colorseg.c

#### color/segment.rs (colorseg.c)

| C関数                       | 状態 | Rust対応                    | 備考                         |
| --------------------------- | ---- | --------------------------- | ---------------------------- |
| pixColorSegment             | 🔄   | color_segment               | Phase 3が未実装              |
| pixColorSegmentCluster      | ✅   | color_segment_cluster       |                              |
| pixAssignToNearestColor     | 🔄   | assign_to_nearest_color     | 実装の詳細が異なる           |
| pixColorSegmentClean        | ✅   | color_segment_clean         | モーフォロジークリーンアップ |
| pixColorSegmentRemoveColors | 🔄   | color_segment_remove_colors | 内部関数として実装           |

### colorcontent.c

#### color/analysis.rs (colorcontent.c)

| C関数                       | 状態 | Rust対応                     | 備考                                                               |
| --------------------------- | ---- | ---------------------------- | ------------------------------------------------------------------ |
| pixColorContent             | ✅   | color_content                | -                                                                  |
| pixColorMagnitude           | ✅   | color_magnitude              | 3種の計算方式対応                                                  |
| pixColorFraction            | ✅   | color_fraction               | -                                                                  |
| pixColorShiftWhitePoint     | ✅   | color_shift_white_point      | White point shift                                                  |
| pixMaskOverColorPixels      | ✅   | mask_over_color_pixels       | -                                                                  |
| pixMaskOverGrayPixels       | ✅   | mask_over_gray_pixels        | -                                                                  |
| pixMaskOverColorRange       | ✅   | mask_over_color_range        | -                                                                  |
| pixFindColorRegions         | ✅   | find_color_regions           | Color region検出                                                   |
| pixNumSignificantGrayColors | ✅   | num_significant_gray_colors  | -                                                                  |
| pixColorsForQuantization    | ✅   | colors_for_quantization      | -                                                                  |
| pixNumColors                | 🔄   | count_colors                 |                                                                    |
| pixConvertRGBToCmapLossless | ✅   | convert_rgb_to_cmap_lossless | Lossless変換                                                       |
| pixGetMostPopulatedColors   | ✅   | most_populated_colors        | -                                                                  |
| pixSimpleColorQuantize      | ✅   | simple_color_quantize        | Simple量子化                                                       |
| pixGetRGBHistogram          | ✅   | rgb_histogram                | -                                                                  |
| makeRGBIndexTables          | 🚫   | -                            | プライベート関数 `make_rgb_index_tables` として存在（analysis.rs） |
| getRGBFromIndex             | 🚫   | -                            | プライベート関数 `get_rgb_from_index` として存在（analysis.rs）    |
| pixHasHighlightRed          | ✅   | has_highlight_red            | Highlight red検出                                                  |

### colorfill.c

#### color/colorfill.rs (colorfill.c)

| C関数                     | 状態 | Rust対応                  | 備考                                |
| ------------------------- | ---- | ------------------------- | ----------------------------------- |
| l_colorfillCreate         | 🚫   | -                         | C版構造体管理（Rustでは異なる設計） |
| l_colorfillDestroy        | 🚫   | -                         | C版構造体管理（Rustでは異なる設計） |
| pixColorContentByLocation | ✅   | color_content_by_location | Location-based色内容分析            |
| pixColorFill              | 🔄   | color_fill                | インターフェース異なる              |
| makeColorfillTestData     | 🚫   | -                         | テスト用データ生成関数              |

### coloring.c

#### color/coloring.rs (coloring.c)

| C関数                       | 状態 | Rust対応                         | 備考                |
| --------------------------- | ---- | -------------------------------- | ------------------- |
| pixColorGrayRegions         | ✅   | color_gray_regions               | Region coloring     |
| pixColorGray                | 🔄   | pix_color_gray                   |                     |
| pixColorGrayMasked          | ✅   | pix_color_gray_masked            |                     |
| pixSnapColor                | 🔄   | pix_snap_color                   |                     |
| pixSnapColorCmap            | ✅   | snap_color_cmap                  | カラーマップ版      |
| pixLinearMapToTargetColor   | ✅   | pix_linear_map_to_target_color   |                     |
| pixelLinearMapToTargetColor | ✅   | pixel_linear_map_to_target_color |                     |
| pixShiftByComponent         | ✅   | pix_shift_by_component           |                     |
| pixelShiftByComponent       | ✅   | pixel_shift_by_component         |                     |
| pixelFractionalShift        | ✅   | pixel_fractional_shift           |                     |
| pixMapWithInvariantHue      | ✅   | pix_map_with_invariant_hue()     | Hue-invariant shift |

### binarize.c

#### color/threshold.rs (binarize.c)

| C関数                           | 状態 | Rust対応                         | 備考               |
| ------------------------------- | ---- | -------------------------------- | ------------------ |
| pixOtsuAdaptiveThreshold        | ✅   | otsu_adaptive_threshold          | -                  |
| pixOtsuThreshOnBackgroundNorm   | ✅   | otsu_thresh_on_background_norm   | BG normalization   |
| pixMaskedThreshOnBackgroundNorm | ✅   | masked_thresh_on_background_norm | Masked BG norm     |
| pixSauvolaBinarizeTiled         | ✅   | sauvola_binarize_tiled           | -                  |
| pixSauvolaBinarize              | 🔄   | sauvola_threshold                | 実装が異なる       |
| pixSauvolaOnContrastNorm        | ✅   | sauvola_on_contrast_norm         | Contrast norm      |
| pixThreshOnDoubleNorm           | ✅   | thresh_on_double_norm            | Double norm        |
| pixThresholdByConnComp          | ✅   | threshold_by_conn_comp           | ConnComp threshold |
| pixThresholdByHisto             | ✅   | threshold_by_histo               | Histo threshold    |

### paintcmap.c

#### color/paintcmap.rs (paintcmap.c)

| C関数                   | 状態 | Rust対応                    | 備考         |
| ----------------------- | ---- | --------------------------- | ------------ |
| pixSetSelectCmap        | ✅   | pix_set_select_cmap         | paintcmap.rs |
| pixColorGrayRegionsCmap | ✅   | pix_color_gray_regions_cmap | paintcmap.rs |
| pixColorGrayCmap        | ✅   | pix_color_gray_cmap         | paintcmap.rs |
| pixColorGrayMaskedCmap  | ✅   | pix_color_gray_masked_cmap  | paintcmap.rs |
| addColorizedGrayToCmap  | ✅   | add_colorized_gray_to_cmap  | paintcmap.rs |
| pixSetSelectMaskedCmap  | ✅   | pix_set_select_masked_cmap  | paintcmap.rs |
| pixSetMaskedCmap        | ✅   | pix_set_masked_cmap         | paintcmap.rs |

### grayquant.c

#### color/threshold.rs (grayquant.c)

| C関数                        | 状態 | Rust対応                        | 備考                          |
| ---------------------------- | ---- | ------------------------------- | ----------------------------- |
| pixDitherToBinary            | 🔄   | dither_to_binary                |                               |
| pixDitherToBinarySpec        | 🔄   | dither_to_binary_with_threshold |                               |
| pixThresholdToBinary         | ✅   | threshold_to_binary             |                               |
| pixVarThresholdToBinary      | ✅   | var_threshold_to_binary         | -                             |
| pixAdaptThresholdToBinary    | 🔄   | adaptive_threshold              |                               |
| pixAdaptThresholdToBinaryGen | ✅   | adapt_threshold_to_binary_gen   | Generic adaptive              |
| pixGenerateMaskByValue       | ✅   | generate_mask_by_value          | -                             |
| pixGenerateMaskByBand        | ✅   | generate_mask_by_band           | -                             |
| pixDitherTo2bpp              | ✅   | dither_to_2bpp                  | 2bpp dither                   |
| pixDitherTo2bppSpec          | ✅   | dither_to_2bpp_spec             | 2bpp dither spec              |
| pixThresholdTo2bpp           | ✅   | threshold_to_2bpp               | -                             |
| pixThresholdTo4bpp           | ✅   | threshold_to_4bpp               | -                             |
| pixThresholdOn8bpp           | ✅   | threshold_on_8bpp               | 8bpp threshold                |
| pixThresholdGrayArb          | ✅   | threshold_gray_arb              | Arbitrary threshold           |
| ditherToBinaryLineLow        | 🚫   | -                               | 低レベル内部関数              |
| thresholdToBinaryLineLow     | 🚫   | -                               | 低レベル内部関数              |
| pixDitherToBinaryLUT         | 🚫   | -                               | LUTベース内部最適化           |
| makeGrayQuantIndexTable      | 🚫   | -                               | 内部実装の詳細（LUTヘルパー） |
| makeGrayQuantTableArb        | 🚫   | -                               | 内部実装の詳細（LUTヘルパー） |
| pixGenerateMaskByBand32      | ✅   | generate_mask_by_band_32        | 32bpp band mask               |
| pixGenerateMaskByDiscr32     | ✅   | generate_mask_by_discr_32       | 32bpp discrimination mask     |
| pixGrayQuantFromHisto        | ✅   | gray_quant_from_histo           | Histo-based quant             |
| pixGrayQuantFromCmap         | ✅   | gray_quant_from_cmap            | Cmap-based quant              |

## 分析

### 実装済み機能の特徴

Rust版で実装済みの機能は主に以下のカテゴリに集中している:

1. **色空間変換** (RGB ↔ HSV, LAB, XYZ, YUV)
   - ピクセルレベル変換は完全実装
   - 画像レベル変換: HSV, Grayscale, YUV, XYZ, LAB全て実装済み
   - カラーマップ操作(HSV, YUV)実装済み
   - HSV範囲マスク・2Dヒストグラム・ピーク検出実装済み

2. **色量子化** (Median Cut, Octree)
   - Median Cut: basic, simple, mixed, few colors mixed実装済み
   - Octree: basic, 256色, population-based, N色, fixed octcube, gen RGB実装済み
   - Few colors系量子化(quant1/2/mixed)実装済み
   - カラーマップからの量子化、LUT量子化、未使用色削除も実装済み
   - Octcubeツリー構築、占有Octcube数計算も実装済み

3. **色セグメンテーション** (Clustering, Nearest color assignment)
   - Phase 1,2,4 が `color_segment()` 内で実装済み。Phase 3（モーフォロジークリーンアップ）は `color_segment_clean()` としてスタンドアロン関数で存在するが、`color_segment()` のメインフローでは統合されていない

4. **二値化・閾値処理**
   - 固定閾値, Otsu, Adaptive, Dithering実装済み
   - タイル別Otsu/Sauvola, 可変閾値, マスク生成実装済み
   - 背景正規化ベース(Otsu/Masked/Sauvola/Double)の二値化も実装済み
   - ConnComp/Histo-basedの閾値処理も実装済み
   - 2bpp dither, 8bpp threshold, arbitrary threshold実装済み
   - 32bpp band/discrimination mask生成実装済み

5. **色内容分析**
   - color_content, color_magnitude, color_fraction実装済み
   - 色/グレーマスク生成, RGBヒストグラム, 有意色数計算実装済み
   - White point shift, color region検出, highlight red検出実装済み
   - Lossless CMAP変換, simple量子化実装済み
   - Location-based色内容分析実装済み

6. **グレースケール→カラー変換** (Coloring)
   - 基本的なColorize機能は実装
   - Region-based coloring, カラーマップ版snap colorも実装済み

### 全関数実装完了

❌ 未実装の関数は0件。C版colorモジュールの全関数（🚫 不要と判定した表示/ヘルパー関数を除く）がRust版で実装済み。

XYZ/LAB画像レベル変換はC版のFPIXA(FPix配列)の代わりに `(FPix, FPix, FPix)` タプルを使用して実装。

### 🚫 不要と判定した機能

以下は Rust移植では不要と判定した:

1. **表示/可視化関数**: displayHSVColorRange, pixMakeGamutRGB
2. **C版LUT専用ヘルパー**: makeRGBToIndexTables, getOctcubeIndexFromRGB, getRGBFromOctcube, pixcmapToOctcubeLUT（Rustでは異なるアプローチ）
3. **プライベート関数として存在**: makeRGBIndexTables, getRGBFromIndex（analysis.rs内のプライベート関数として実装済み）
4. **内部実装の詳細**: pixMedianCutHisto, makeGrayQuantIndexTable, makeGrayQuantTableArb（内部ヘルパー）
5. **C版構造体管理**: l_colorfillCreate, l_colorfillDestroy（Rustでは異なる設計）
6. **テスト用データ生成**: makeColorfillTestData

### 実装方針の違い

- **C版**: 多機能で詳細なパラメータ制御が可能
- **Rust版**: コア機能に絞り、シンプルなAPIを提供

例:

- Median Cut: C版は6パラメータ、Rust版は2-3パラメータ
- Color Segment: C版は4フェーズ完全実装、Rust版はPhase 3省略
- Quantization: C版は10種類以上の関数、Rust版は2種類(median_cut, octree)

## 推奨事項

全関数の実装が完了。残る改善は 🔄 異なる と判定された16関数のAPI互換性向上のみ。

### 🔄 異なる関数のAPI改善（任意）

- Median Cut量子化のパラメータ構造をC版に近づける
- Color Segmentの内部実装詳細の改善
- Dither/Adaptive threshold のインターフェース統一
