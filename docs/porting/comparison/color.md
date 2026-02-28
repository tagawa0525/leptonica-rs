# leptonica (src/color/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-26（color全関数実装完了を反映）

## サマリー

| 項目      | 数  |
| --------- | --- |
| ✅ 同等   | 104 |
| 🔄 異なる | 16  |
| ❌ 未実装 | 0   |
| 🚫 不要   | 13  |
| 合計      | 133 |

## 詳細

### colorspace.c

| C関数                     | 状態      | Rust対応                                      | 備考                           |
| ------------------------- | --------- | --------------------------------------------- | ------------------------------ |
| pixConvertRGBToHSV        | ✅ 同等   | pix_convert_rgb_to_hsv                        |                                |
| pixConvertHSVToRGB        | ✅ 同等   | pix_convert_hsv_to_rgb                        |                                |
| convertRGBToHSV           | ✅ 同等   | rgb_to_hsv                                    |                                |
| convertHSVToRGB           | ✅ 同等   | hsv_to_rgb                                    |                                |
| pixcmapConvertRGBToHSV    | ✅ 同等   | pix_colormap_convert_rgb_to_hsv               | カラーマップ操作               |
| pixcmapConvertHSVToRGB    | ✅ 同等   | pix_colormap_convert_hsv_to_rgb               | カラーマップ操作               |
| pixConvertRGBToHue        | 🔄 異なる | pix_extract_channel(ColorChannel::Hue)        | チャネル抽出として実装         |
| pixConvertRGBToSaturation | 🔄 異なる | pix_extract_channel(ColorChannel::Saturation) | チャネル抽出として実装         |
| pixConvertRGBToValue      | 🔄 異なる | pix_extract_channel(ColorChannel::Value)      | チャネル抽出として実装         |
| pixMakeRangeMaskHS        | ✅ 同等   | make_range_mask_hs                            | -                              |
| pixMakeRangeMaskHV        | ✅ 同等   | make_range_mask_hv                            | -                              |
| pixMakeRangeMaskSV        | ✅ 同等   | make_range_mask_sv                            | -                              |
| pixMakeHistoHS            | ✅ 同等   | make_histo_hs                                 | -                              |
| pixMakeHistoHV            | ✅ 同等   | make_histo_hv                                 | -                              |
| pixMakeHistoSV            | ✅ 同等   | make_histo_sv                                 | -                              |
| pixFindHistoPeaksHSV      | ✅ 同等   | find_histo_peaks_hsv                          | HSVヒストグラムピーク検出      |
| displayHSVColorRange      | 🚫 不要   | -                                             | 表示/可視化関数                |
| pixConvertRGBToYUV        | ✅ 同等   | pix_convert_rgb_to_yuv                        | -                              |
| pixConvertYUVToRGB        | ✅ 同等   | pix_convert_yuv_to_rgb                        | -                              |
| convertRGBToYUV           | ✅ 同等   | rgb_to_yuv                                    |                                |
| convertYUVToRGB           | ✅ 同等   | yuv_to_rgb                                    |                                |
| pixcmapConvertRGBToYUV    | ✅ 同等   | pix_colormap_convert_rgb_to_yuv               | カラーマップ操作               |
| pixcmapConvertYUVToRGB    | ✅ 同等   | pix_colormap_convert_yuv_to_rgb               | カラーマップ操作               |
| pixConvertRGBToXYZ        | ✅ 同等   | pix_convert_rgb_to_xyz                        | (FPix, FPix, FPix)タプルで返却 |
| fpixaConvertXYZToRGB      | ✅ 同等   | fpixa_convert_xyz_to_rgb                      | (FPix, FPix, FPix)タプルを入力 |
| convertRGBToXYZ           | ✅ 同等   | rgb_to_xyz                                    |                                |
| convertXYZToRGB           | ✅ 同等   | xyz_to_rgb                                    |                                |
| fpixaConvertXYZToLAB      | ✅ 同等   | fpixa_convert_xyz_to_lab                      | (FPix, FPix, FPix)タプルで操作 |
| fpixaConvertLABToXYZ      | ✅ 同等   | fpixa_convert_lab_to_xyz                      | (FPix, FPix, FPix)タプルで操作 |
| convertXYZToLAB           | ✅ 同等   | xyz_to_lab                                    |                                |
| convertLABToXYZ           | ✅ 同等   | lab_to_xyz                                    |                                |
| pixConvertRGBToLAB        | ✅ 同等   | pix_convert_rgb_to_lab                        | (FPix, FPix, FPix)タプルで返却 |
| fpixaConvertLABToRGB      | ✅ 同等   | fpixa_convert_lab_to_rgb                      | (FPix, FPix, FPix)タプルを入力 |
| convertRGBToLAB           | ✅ 同等   | rgb_to_lab                                    |                                |
| convertLABToRGB           | ✅ 同等   | lab_to_rgb                                    |                                |
| pixMakeGamutRGB           | 🚫 不要   | -                                             | 表示/可視化関数                |

### colorquant1.c

| C関数                         | 状態    | Rust対応                       | 備考                   |
| ----------------------------- | ------- | ------------------------------ | ---------------------- |
| pixOctreeQuantByPopulation    | ✅ 同等 | octree_quant_by_population     | -                      |
| pixOctreeQuantNumColors       | ✅ 同等 | octree_quant_num_colors        | -                      |
| pixOctcubeQuantMixedWithGray  | ✅ 同等 | octcube_quant_mixed_with_gray  | Mixed量子化            |
| pixFixedOctcubeQuant256       | ✅ 同等 | fixed_octcube_quant_256        | -                      |
| pixFewColorsOctcubeQuant1     | ✅ 同等 | few_colors_octcube_quant1      | Few colors量子化       |
| pixFewColorsOctcubeQuant2     | ✅ 同等 | few_colors_octcube_quant2      | Few colors量子化       |
| pixFewColorsOctcubeQuantMixed | ✅ 同等 | few_colors_octcube_quant_mixed | Few colors mixed量子化 |
| pixFixedOctcubeQuantGenRGB    | ✅ 同等 | fixed_octcube_quant_gen_rgb    | 固定Octcube量子化      |
| pixQuantFromCmap              | ✅ 同等 | quant_from_cmap                | -                      |
| pixOctcubeQuantFromCmap       | ✅ 同等 | octcube_quant_from_cmap        | Octcube量子化          |
| pixOctcubeQuantFromCmapLUT    | ✅ 同等 | octcube_quant_from_cmap_lut    | LUT使用量子化          |
| makeRGBToIndexTables          | 🚫 不要 | -                              | C版LUT専用ヘルパー     |
| getOctcubeIndexFromRGB        | 🚫 不要 | -                              | C版LUT専用ヘルパー     |
| getRGBFromOctcubeIndex        | 🚫 不要 | -                              | C版LUT専用ヘルパー     |
| pixOctcubeTree                | ✅ 同等 | octcube_tree                   | Octcubeツリー構築      |
| pixRemoveUnusedColors         | ✅ 同等 | remove_unused_colors           | -                      |
| pixNumberOccupiedOctcubes     | ✅ 同等 | number_occupied_octcubes       | 占有Octcube数計算      |

### colorquant2.c

| C関数                           | 状態      | Rust対応                          | 備考                           |
| ------------------------------- | --------- | --------------------------------- | ------------------------------ |
| pixMedianCutQuant               | 🔄 異なる | median_cut_quant_simple           | アルゴリズムの詳細が異なる     |
| pixMedianCutQuantGeneral        | 🔄 異なる | median_cut_quant                  | パラメータ構造が異なる         |
| pixMedianCutQuantMixed          | ✅ 同等   | median_cut_quant_mixed            | -                              |
| pixFewColorsMedianCutQuantMixed | ✅ 同等   | few_colors_median_cut_quant_mixed | Few colors mixed量子化         |
| pixMedianCutHisto               | 🚫 不要   | -                                 | 内部実装の詳細（ヘルパー関数） |

### colorseg.c

| C関数                       | 状態      | Rust対応                    | 備考                         |
| --------------------------- | --------- | --------------------------- | ---------------------------- |
| pixColorSegment             | 🔄 異なる | color_segment               | Phase 3が未実装              |
| pixColorSegmentCluster      | ✅ 同等   | color_segment_cluster       |                              |
| pixAssignToNearestColor     | 🔄 異なる | assign_to_nearest_color     | 実装の詳細が異なる           |
| pixColorSegmentClean        | ✅ 同等   | color_segment_clean         | モーフォロジークリーンアップ |
| pixColorSegmentRemoveColors | 🔄 異なる | color_segment_remove_colors | 内部関数として実装           |

### colorcontent.c

| C関数                       | 状態      | Rust対応                     | 備考                                                               |
| --------------------------- | --------- | ---------------------------- | ------------------------------------------------------------------ |
| pixColorContent             | ✅ 同等   | color_content                | -                                                                  |
| pixColorMagnitude           | ✅ 同等   | color_magnitude              | 3種の計算方式対応                                                  |
| pixColorFraction            | ✅ 同等   | color_fraction               | -                                                                  |
| pixColorShiftWhitePoint     | ✅ 同等   | color_shift_white_point      | White point shift                                                  |
| pixMaskOverColorPixels      | ✅ 同等   | mask_over_color_pixels       | -                                                                  |
| pixMaskOverGrayPixels       | ✅ 同等   | mask_over_gray_pixels        | -                                                                  |
| pixMaskOverColorRange       | ✅ 同等   | mask_over_color_range        | -                                                                  |
| pixFindColorRegions         | ✅ 同等   | find_color_regions           | Color region検出                                                   |
| pixNumSignificantGrayColors | ✅ 同等   | num_significant_gray_colors  | -                                                                  |
| pixColorsForQuantization    | ✅ 同等   | colors_for_quantization      | -                                                                  |
| pixNumColors                | 🔄 異なる | count_colors                 |                                                                    |
| pixConvertRGBToCmapLossless | ✅ 同等   | convert_rgb_to_cmap_lossless | Lossless変換                                                       |
| pixGetMostPopulatedColors   | ✅ 同等   | most_populated_colors        | -                                                                  |
| pixSimpleColorQuantize      | ✅ 同等   | simple_color_quantize        | Simple量子化                                                       |
| pixGetRGBHistogram          | ✅ 同等   | rgb_histogram                | -                                                                  |
| makeRGBIndexTables          | 🚫 不要   | -                            | プライベート関数 `make_rgb_index_tables` として存在（analysis.rs） |
| getRGBFromIndex             | 🚫 不要   | -                            | プライベート関数 `get_rgb_from_index` として存在（analysis.rs）    |
| pixHasHighlightRed          | ✅ 同等   | has_highlight_red            | Highlight red検出                                                  |

### colorfill.c

| C関数                     | 状態      | Rust対応                  | 備考                                |
| ------------------------- | --------- | ------------------------- | ----------------------------------- |
| l_colorfillCreate         | 🚫 不要   | -                         | C版構造体管理（Rustでは異なる設計） |
| l_colorfillDestroy        | 🚫 不要   | -                         | C版構造体管理（Rustでは異なる設計） |
| pixColorContentByLocation | ✅ 同等   | color_content_by_location | Location-based色内容分析            |
| pixColorFill              | 🔄 異なる | color_fill                | インターフェース異なる              |
| makeColorfillTestData     | 🚫 不要   | -                         | テスト用データ生成関数              |

### coloring.c

| C関数                       | 状態      | Rust対応                                 | 備考                |
| --------------------------- | --------- | ---------------------------------------- | ------------------- |
| pixColorGrayRegions         | ✅ 同等   | color_gray_regions                       | Region coloring     |
| pixColorGray                | 🔄 異なる | pix_color_gray                           |                     |
| pixColorGrayMasked          | ✅ 同等   | pix_color_gray_masked                    |                     |
| pixSnapColor                | 🔄 異なる | pix_snap_color                           |                     |
| pixSnapColorCmap            | ✅ 同等   | snap_color_cmap                          | カラーマップ版      |
| pixLinearMapToTargetColor   | ✅ 同等   | pix_linear_map_to_target_color           |                     |
| pixelLinearMapToTargetColor | ✅ 同等   | pixel_linear_map_to_target_color         |                     |
| pixShiftByComponent         | ✅ 同等   | pix_shift_by_component                   |                     |
| pixelShiftByComponent       | ✅ 同等   | pixel_shift_by_component                 |                     |
| pixelFractionalShift        | ✅ 同等   | pixel_fractional_shift                   |                     |
| pixShiftWithInvariantHue    | ✅ 同等   | coloring.rs pix_map_with_invariant_hue() | Hue-invariant shift |

### binarize.c

| C関数                           | 状態      | Rust対応                         | 備考               |
| ------------------------------- | --------- | -------------------------------- | ------------------ |
| pixOtsuAdaptiveThreshold        | ✅ 同等   | otsu_adaptive_threshold          | -                  |
| pixOtsuThreshOnBackgroundNorm   | ✅ 同等   | otsu_thresh_on_background_norm   | BG normalization   |
| pixMaskedThreshOnBackgroundNorm | ✅ 同等   | masked_thresh_on_background_norm | Masked BG norm     |
| pixSauvolaBinarizeTiled         | ✅ 同等   | sauvola_binarize_tiled           | -                  |
| pixSauvolaBinarize              | 🔄 異なる | sauvola_threshold                | 実装が異なる       |
| pixSauvolaOnContrastNorm        | ✅ 同等   | sauvola_on_contrast_norm         | Contrast norm      |
| pixThreshOnDoubleNorm           | ✅ 同等   | thresh_on_double_norm            | Double norm        |
| pixThresholdByConnComp          | ✅ 同等   | threshold_by_conn_comp           | ConnComp threshold |
| pixThresholdByHisto             | ✅ 同等   | threshold_by_histo               | Histo threshold    |

### paintcmap.c

| C関数                   | 状態    | Rust対応                    | 備考         |
| ----------------------- | ------- | --------------------------- | ------------ |
| pixSetSelectCmap        | ✅ 同等 | pix_set_select_cmap         | paintcmap.rs |
| pixColorGrayRegionsCmap | ✅ 同等 | pix_color_gray_regions_cmap | paintcmap.rs |
| pixColorGrayCmap        | ✅ 同等 | pix_color_gray_cmap         | paintcmap.rs |
| pixColorGrayMaskedCmap  | ✅ 同等 | pix_color_gray_masked_cmap  | paintcmap.rs |
| addColorizedGrayToCmap  | ✅ 同等 | add_colorized_gray_to_cmap  | paintcmap.rs |
| pixSetSelectMaskedCmap  | ✅ 同等 | pix_set_select_masked_cmap  | paintcmap.rs |
| pixSetMaskedCmap        | ✅ 同等 | pix_set_masked_cmap         | paintcmap.rs |

### grayquant.c

| C関数                        | 状態      | Rust対応                        | 備考                          |
| ---------------------------- | --------- | ------------------------------- | ----------------------------- |
| pixDitherToBinary            | 🔄 異なる | dither_to_binary                |                               |
| pixDitherToBinarySpec        | 🔄 異なる | dither_to_binary_with_threshold |                               |
| pixThresholdToBinary         | ✅ 同等   | threshold_to_binary             |                               |
| pixVarThresholdToBinary      | ✅ 同等   | var_threshold_to_binary         | -                             |
| pixAdaptThresholdToBinary    | 🔄 異なる | adaptive_threshold              |                               |
| pixAdaptThresholdToBinaryGen | ✅ 同等   | adapt_threshold_to_binary_gen   | Generic adaptive              |
| pixGenerateMaskByValue       | ✅ 同等   | generate_mask_by_value          | -                             |
| pixGenerateMaskByBand        | ✅ 同等   | generate_mask_by_band           | -                             |
| pixDitherTo2bpp              | ✅ 同等   | dither_to_2bpp                  | 2bpp dither                   |
| pixDitherTo2bppSpec          | ✅ 同等   | dither_to_2bpp_spec             | 2bpp dither spec              |
| pixThresholdTo2bpp           | ✅ 同等   | threshold_to_2bpp               | -                             |
| pixThresholdTo4bpp           | ✅ 同等   | threshold_to_4bpp               | -                             |
| pixThresholdOn8bpp           | ✅ 同等   | threshold_on_8bpp               | 8bpp threshold                |
| pixThresholdGrayArb          | ✅ 同等   | threshold_gray_arb              | Arbitrary threshold           |
| makeGrayQuantIndexTable      | 🚫 不要   | -                               | 内部実装の詳細（LUTヘルパー） |
| makeGrayQuantTableArb        | 🚫 不要   | -                               | 内部実装の詳細（LUTヘルパー） |
| pixGenerateMaskByBand32      | ✅ 同等   | generate_mask_by_band_32        | 32bpp band mask               |
| pixGenerateMaskByDiscr32     | ✅ 同等   | generate_mask_by_discr_32       | 32bpp discrimination mask     |
| pixGrayQuantFromHisto        | ✅ 同等   | gray_quant_from_histo           | Histo-based quant             |
| pixGrayQuantFromCmap         | ✅ 同等   | gray_quant_from_cmap            | Cmap-based quant              |

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
2. **C版LUT専用ヘルパー**: makeRGBToIndexTables, getOctcubeIndexFromRGB, getRGBFromOctcubeIndex（Rustでは異なるアプローチ）
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
