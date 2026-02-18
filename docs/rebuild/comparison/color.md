# leptonica-color: C版 vs Rust版 関数レベル比較

調査日: 2026-02-18（Phase 7.1-7.4 実装を反映）

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 51 |
| 🔄 異なる | 16 |
| ❌ 未実装 | 59 |
| 合計 | 126 |

## 詳細

### colorspace.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixConvertRGBToHSV | ✅ 同等 | pix_convert_rgb_to_hsv | |
| pixConvertHSVToRGB | ✅ 同等 | pix_convert_hsv_to_rgb | |
| convertRGBToHSV | ✅ 同等 | rgb_to_hsv | |
| convertHSVToRGB | ✅ 同等 | hsv_to_rgb | |
| pixcmapConvertRGBToHSV | ❌ 未実装 | - | カラーマップ操作未実装 |
| pixcmapConvertHSVToRGB | ❌ 未実装 | - | カラーマップ操作未実装 |
| pixConvertRGBToHue | 🔄 異なる | pix_extract_channel(ColorChannel::Hue) | チャネル抽出として実装 |
| pixConvertRGBToSaturation | 🔄 異なる | pix_extract_channel(ColorChannel::Saturation) | チャネル抽出として実装 |
| pixConvertRGBToValue | 🔄 異なる | pix_extract_channel(ColorChannel::Value) | チャネル抽出として実装 |
| pixMakeRangeMaskHS | ✅ 同等 | make_range_mask_hs | - |
| pixMakeRangeMaskHV | ✅ 同等 | make_range_mask_hv | - |
| pixMakeRangeMaskSV | ✅ 同等 | make_range_mask_sv | - |
| pixMakeHistoHS | ✅ 同等 | make_histo_hs | - |
| pixMakeHistoHV | ✅ 同等 | make_histo_hv | - |
| pixMakeHistoSV | ✅ 同等 | make_histo_sv | - |
| pixFindHistoPeaksHSV | ❌ 未実装 | - | HSVヒストグラムピーク未実装 |
| displayHSVColorRange | ❌ 未実装 | - | HSV範囲表示未実装 |
| pixConvertRGBToYUV | ✅ 同等 | pix_convert_rgb_to_yuv | - |
| pixConvertYUVToRGB | ✅ 同等 | pix_convert_yuv_to_rgb | - |
| convertRGBToYUV | ✅ 同等 | rgb_to_yuv | |
| convertYUVToRGB | ✅ 同等 | yuv_to_rgb | |
| pixcmapConvertRGBToYUV | ❌ 未実装 | - | カラーマップ操作未実装 |
| pixcmapConvertYUVToRGB | ❌ 未実装 | - | カラーマップ操作未実装 |
| pixConvertRGBToXYZ | ❌ 未実装 | - | 画像レベル変換未実装(FPIXA使用) |
| fpixaConvertXYZToRGB | ❌ 未実装 | - | FPIXA未実装 |
| convertRGBToXYZ | ✅ 同等 | rgb_to_xyz | |
| convertXYZToRGB | ✅ 同等 | xyz_to_rgb | |
| fpixaConvertXYZToLAB | ❌ 未実装 | - | FPIXA未実装 |
| fpixaConvertLABToXYZ | ❌ 未実装 | - | FPIXA未実装 |
| convertXYZToLAB | ✅ 同等 | xyz_to_lab | |
| convertLABToXYZ | ✅ 同等 | lab_to_xyz | |
| pixConvertRGBToLAB | ❌ 未実装 | - | FPIXA未実装 |
| fpixaConvertLABToRGB | ❌ 未実装 | - | FPIXA未実装 |
| convertRGBToLAB | ✅ 同等 | rgb_to_lab | |
| convertLABToRGB | ✅ 同等 | lab_to_rgb | |
| pixMakeGamutRGB | ❌ 未実装 | - | RGB色域表示未実装 |

### colorquant1.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixOctreeQuantByPopulation | ✅ 同等 | octree_quant_by_population | - |
| pixOctreeQuantNumColors | ✅ 同等 | octree_quant_num_colors | - |
| pixOctcubeQuantMixedWithGray | ❌ 未実装 | - | Mixed量子化未実装 |
| pixFixedOctcubeQuant256 | ✅ 同等 | fixed_octcube_quant_256 | - |
| pixFewColorsOctcubeQuant1 | ❌ 未実装 | - | Few colors量子化未実装 |
| pixFewColorsOctcubeQuant2 | ❌ 未実装 | - | Few colors量子化未実装 |
| pixFewColorsOctcubeQuantMixed | ❌ 未実装 | - | Few colors mixed未実装 |
| pixFixedOctcubeQuantGenRGB | ❌ 未実装 | - | 固定Octcube未実装 |
| pixQuantFromCmap | ✅ 同等 | quant_from_cmap | - |
| pixOctcubeQuantFromCmap | ❌ 未実装 | - | Octcube量子化未実装 |
| pixOctcubeQuantFromCmapLUT | ❌ 未実装 | - | LUT使用量子化未実装 |
| makeRGBToIndexTables | ❌ 未実装 | - | インデックステーブル未実装 |
| getOctcubeIndexFromRGB | ❌ 未実装 | - | RGB→Index未実装 |
| getRGBFromOctcubeIndex | ❌ 未実装 | - | Index→RGB未実装 |
| pixOctcubeTree | ❌ 未実装 | - | Octcubeツリー未実装 |
| pixRemoveUnusedColors | ✅ 同等 | remove_unused_colors | - |
| pixNumberOccupiedOctcubes | ❌ 未実装 | - | 占有Octcube数未実装 |

### colorquant2.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixMedianCutQuant | 🔄 異なる | median_cut_quant_simple | アルゴリズムの詳細が異なる |
| pixMedianCutQuantGeneral | 🔄 異なる | median_cut_quant | パラメータ構造が異なる |
| pixMedianCutQuantMixed | ✅ 同等 | median_cut_quant_mixed | - |
| pixFewColorsMedianCutQuantMixed | ❌ 未実装 | - | Few colors mixed未実装 |
| pixMedianCutHisto | ❌ 未実装 | - | ヒストグラム生成は内部実装 |

### colorseg.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixColorSegment | 🔄 異なる | color_segment | Phase 3が未実装 |
| pixColorSegmentCluster | ✅ 同等 | color_segment_cluster | |
| pixAssignToNearestColor | 🔄 異なる | assign_to_nearest_color | 実装の詳細が異なる |
| pixColorSegmentClean | ❌ 未実装 | - | モーフォロジークリーンアップ未実装 |
| pixColorSegmentRemoveColors | 🔄 異なる | color_segment_remove_colors | 内部関数として実装 |

### colorcontent.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixColorContent | ✅ 同等 | color_content | - |
| pixColorMagnitude | ✅ 同等 | color_magnitude | 3種の計算方式対応 |
| pixColorFraction | ✅ 同等 | color_fraction | - |
| pixColorShiftWhitePoint | ❌ 未実装 | - | White point shift未実装 |
| pixMaskOverColorPixels | ✅ 同等 | mask_over_color_pixels | - |
| pixMaskOverGrayPixels | ✅ 同等 | mask_over_gray_pixels | - |
| pixMaskOverColorRange | ✅ 同等 | mask_over_color_range | - |
| pixFindColorRegions | ❌ 未実装 | - | Color region検出未実装 |
| pixNumSignificantGrayColors | ✅ 同等 | num_significant_gray_colors | - |
| pixColorsForQuantization | ✅ 同等 | colors_for_quantization | - |
| pixNumColors | 🔄 異なる | count_colors | |
| pixConvertRGBToCmapLossless | ❌ 未実装 | - | Lossless変換未実装 |
| pixGetMostPopulatedColors | ✅ 同等 | most_populated_colors | - |
| pixSimpleColorQuantize | ❌ 未実装 | - | Simple量子化未実装 |
| pixGetRGBHistogram | ✅ 同等 | rgb_histogram | - |
| makeRGBIndexTables | ❌ 未実装 | - | RGBインデックス未実装 |
| getRGBFromIndex | ❌ 未実装 | - | Index→RGB未実装 |
| pixHasHighlightRed | ❌ 未実装 | - | Highlight red検出未実装 |

### colorfill.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| l_colorfillCreate | ❌ 未実装 | - | L_COLORFILL構造体未実装 |
| l_colorfillDestroy | ❌ 未実装 | - | L_COLORFILL構造体未実装 |
| pixColorContentByLocation | ❌ 未実装 | - | Location-based未実装 |
| pixColorFill | 🔄 異なる | color_fill | インターフェース異なる |
| makeColorfillTestData | ❌ 未実装 | - | テストデータ生成未実装 |

### coloring.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixColorGrayRegions | ❌ 未実装 | - | Region coloring未実装 |
| pixColorGray | 🔄 異なる | pix_color_gray | |
| pixColorGrayMasked | ✅ 同等 | pix_color_gray_masked | |
| pixSnapColor | 🔄 異なる | pix_snap_color | |
| pixSnapColorCmap | ❌ 未実装 | - | カラーマップ版未実装 |
| pixLinearMapToTargetColor | ✅ 同等 | pix_linear_map_to_target_color | |
| pixelLinearMapToTargetColor | ✅ 同等 | pixel_linear_map_to_target_color | |
| pixShiftByComponent | ✅ 同等 | pix_shift_by_component | |
| pixelShiftByComponent | ✅ 同等 | pixel_shift_by_component | |
| pixelFractionalShift | ✅ 同等 | pixel_fractional_shift | |
| pixShiftWithInvariantHue | ❌ 未実装 | - | Hue-invariant shift未実装 |

### binarize.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixOtsuAdaptiveThreshold | ✅ 同等 | otsu_adaptive_threshold | - |
| pixOtsuThreshOnBackgroundNorm | ❌ 未実装 | - | BG normalization未実装 |
| pixMaskedThreshOnBackgroundNorm | ❌ 未実装 | - | Masked BG norm未実装 |
| pixSauvolaBinarizeTiled | ✅ 同等 | sauvola_binarize_tiled | - |
| pixSauvolaBinarize | 🔄 異なる | sauvola_threshold | 実装が異なる |
| pixSauvolaOnContrastNorm | ❌ 未実装 | - | Contrast norm未実装 |
| pixThreshOnDoubleNorm | ❌ 未実装 | - | Double norm未実装 |
| pixThresholdByConnComp | ❌ 未実装 | - | ConnComp threshold未実装 |
| pixThresholdByHisto | ❌ 未実装 | - | Histo threshold未実装 |

### grayquant.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixDitherToBinary | 🔄 異なる | dither_to_binary | |
| pixDitherToBinarySpec | 🔄 異なる | dither_to_binary_with_threshold | |
| pixThresholdToBinary | ✅ 同等 | threshold_to_binary | |
| pixVarThresholdToBinary | ✅ 同等 | var_threshold_to_binary | - |
| pixAdaptThresholdToBinary | 🔄 異なる | adaptive_threshold | |
| pixAdaptThresholdToBinaryGen | ❌ 未実装 | - | Generic adaptive未実装 |
| pixGenerateMaskByValue | ✅ 同等 | generate_mask_by_value | - |
| pixGenerateMaskByBand | ✅ 同等 | generate_mask_by_band | - |
| pixDitherTo2bpp | ❌ 未実装 | - | 2bpp dither未実装 |
| pixDitherTo2bppSpec | ❌ 未実装 | - | 2bpp dither spec未実装 |
| pixThresholdTo2bpp | ✅ 同等 | threshold_to_2bpp | - |
| pixThresholdTo4bpp | ✅ 同等 | threshold_to_4bpp | - |
| pixThresholdOn8bpp | ❌ 未実装 | - | 8bpp threshold未実装 |
| pixThresholdGrayArb | ❌ 未実装 | - | Arbitrary threshold未実装 |
| makeGrayQuantIndexTable | ❌ 未実装 | - | Quant index table未実装 |
| makeGrayQuantTableArb | ❌ 未実装 | - | Arbitrary quant table未実装 |
| pixGenerateMaskByBand32 | ❌ 未実装 | - | 32bpp band mask未実装 |
| pixGenerateMaskByDiscr32 | ❌ 未実装 | - | 32bpp discrimination mask未実装 |
| pixGrayQuantFromHisto | ❌ 未実装 | - | Histo-based quant未実装 |
| pixGrayQuantFromCmap | ❌ 未実装 | - | Cmap-based quant未実装 |

## 分析

### 実装済み機能の特徴

Rust版で実装済みの機能は主に以下のカテゴリに集中している:

1. **色空間変換** (RGB ↔ HSV, LAB, XYZ, YUV)
   - ピクセルレベル変換は完全実装
   - 画像レベル変換: HSV, Grayscale, YUV実装済み（XYZ/LABはFPIXA依存で未実装）
   - HSV範囲マスク・2Dヒストグラム実装済み

2. **色量子化** (Median Cut, Octree)
   - Median Cut: basic, simple, mixed実装済み
   - Octree: basic, 256色, population-based, N色, fixed octcube実装済み
   - カラーマップからの量子化、未使用色削除も実装済み

3. **色セグメンテーション** (Clustering, Nearest color assignment)
   - Phase 1,2,4は実装済み
   - Phase 3(モーフォロジークリーンアップ)が未実装

4. **二値化・閾値処理**
   - 固定閾値, Otsu, Adaptive, Dithering実装済み
   - タイル別Otsu/Sauvola, 可変閾値, マスク生成実装済み
   - 背景正規化ベースの二値化は未実装

5. **色内容分析**
   - color_content, color_magnitude, color_fraction実装済み
   - 色/グレーマスク生成, RGBヒストグラム, 有意色数計算実装済み

6. **グレースケール→カラー変換** (Coloring)
   - 基本的なColorize機能は実装
   - Region-basedやカラーマップ版は未実装

### 未実装機能の特徴

以下の分野が主な未実装領域:

1. **カラーマップ(PIXCMAP)関連操作**
   - C版のカラーマップ直接操作関数（pixcmapConvert系）は未対応

2. **FPIXA(FPix Array)依存機能**
   - XYZ/LAB変換の画像レベル操作
   - Rust版にFPIXA相当の実装なし

3. **Few Colors系量子化**
   - pixFewColorsOctcubeQuant1/2/Mixed等
   - LUT-based operations

4. **高度な二値化**
   - Background normalization, Contrast normalization
   - Connected component based thresholding

5. **Color fill高度機能**
   - L_COLORFILL構造体とlocation-based処理
   - Rust版は基本的なfill機能のみ

### 実装方針の違い

- **C版**: 多機能で詳細なパラメータ制御が可能
- **Rust版**: コア機能に絞り、シンプルなAPIを提供

例:
- Median Cut: C版は6パラメータ、Rust版は2-3パラメータ
- Color Segment: C版は4フェーズ完全実装、Rust版はPhase 3省略
- Quantization: C版は10種類以上の関数、Rust版は2種類(median_cut, octree)

## 推奨事項

### 優先度高(コア機能の完成)

1. **カラーマップ操作の拡充**
   - `pixcmapConvertRGBToHSV/YUV` 等

2. **Color Segmentationの完成**
   - Phase 3モーフォロジークリーンアップ

3. **FPIXA依存の画像レベル変換**
   - XYZ/LAB画像レベル変換（FPIXA実装後に対応可能）

### 優先度中(機能拡張)

4. **Few Colors系量子化**
   - `pixFewColorsOctcubeQuant1/2/Mixed`

5. **高度な二値化**
   - Background normalization (`pixOtsuThreshOnBackgroundNorm`)
   - `pixThresholdByConnComp`

6. **HSVヒストグラムピーク検出**
   - `pixFindHistoPeaksHSV`

### 優先度低(特殊用途)

7. **Color fill高度機能**
   - L_COLORFILL構造体ベース処理

8. **表示・可視化**
   - `displayHSVColorRange`
   - `pixMakeGamutRGB`

9. **その他特殊機能**
   - `pixHasHighlightRed`
   - `pixColorShiftWhitePoint`
