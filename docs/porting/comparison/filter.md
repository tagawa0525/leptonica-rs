# leptonica (src/filter/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 100 |
| 🔄 異なる | 0 |
| ❌ 未実装 | 6 |
| 🚫 不要 | 11 |
| 合計 | 118 |

## 詳細

### convolve.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixBlockconv | ✅ 同等 | block_conv.rs blockconv() | ブロック畳み込み(自動でgray/color判定) |
| pixBlockconvGray | ✅ 同等 | block_conv.rs blockconv_gray() | グレースケールブロック畳み込み |
| pixBlockconvAccum | ✅ 同等 | block_conv.rs blockconv_accum() | 畳み込み用アキュムレータ |
| pixBlockconvGrayUnnormalized | ✅ 同等 | block_conv.rs blockconv_gray_unnormalized() | 正規化なしブロック畳み込み |
| pixBlockconvTiled | ✅ 同等 | block_conv.rs blockconv_tiled() | タイル化ブロック畳み込み |
| pixBlockconvGrayTile | ✅ 同等 | block_conv.rs blockconv_gray_tile() | グレースケールタイル化ブロック畳み込み |
| pixWindowedStats | ✅ 同等 | windowed.rs windowed_stats() | ウィンドウ統計量(mean, mean-square, variance, RMS) |
| pixWindowedMean | ✅ 同等 | windowed.rs windowed_mean() | ウィンドウ平均 |
| pixWindowedMeanSquare | ✅ 同等 | windowed.rs windowed_mean_square() | ウィンドウ平均二乗 |
| pixWindowedVariance | ✅ 同等 | windowed.rs windowed_variance() | ウィンドウ分散 |
| pixMeanSquareAccum | ✅ 同等 | windowed.rs mean_square_accum() | 平均二乗アキュムレータ (returns DPIX*) |
| pixBlockrank | ✅ 同等 | convolve.rs blockrank() | バイナリブロックランクフィルタ |
| pixBlocksum | ✅ 同等 | convolve.rs blocksum() | バイナリブロック和 |
| pixCensusTransform | ✅ 同等 | convolve.rs census_transform() | センサス変換 |
| pixConvolve | ✅ 同等 | convolve() | 汎用畳み込み |
| pixConvolveSep | ✅ 同等 | convolve.rs convolve_sep() | 分離可能畳み込み |
| pixConvolveRGB | ✅ 同等 | convolve_color() | RGB畳み込み |
| pixConvolveRGBSep | ✅ 同等 | convolve.rs convolve_rgb_sep() | RGB分離可能畳み込み |
| fpixConvolve | ✅ 同等 | convolve.rs fpix_convolve() | 浮動小数点畳み込み (FPix対応) |
| fpixConvolveSep | ✅ 同等 | convolve.rs fpix_convolve_sep() | 浮動小数点分離可能畳み込み (FPix対応) |
| pixConvolveWithBias | ✅ 同等 | convolve.rs convolve_with_bias() | バイアス付き畳み込み |
| l_setConvolveSampling | 🚫 不要 | - | グローバル変数セッター（C固有パターン） |
| pixAddGaussianNoise | ✅ 同等 | convolve.rs add_gaussian_noise() | ガウシアンノイズ追加 |
| gaussDistribSampling | 🚫 不要 | - | Box-Muller法ヘルパー; add_gaussian_noise()内で実装済み |

### kernel.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| kernelCreate | ✅ 同等 | kernel.rs Kernel::new() | カーネル生成（初期値ゼロ） |
| kernelDestroy | ✅ 同等 | - | Rust自動Drop（メモリ管理） |
| kernelCopy | ✅ 同等 | Kernel::clone() | カーネルクローン（Copy/Clone) |
| kernelGetElement | ✅ 同等 | kernel.rs Kernel::get() | 要素読み込み |
| kernelSetElement | ✅ 同等 | kernel.rs Kernel::set() | 要素書き込み |
| kernelGetParameters | ✅ 同等 | kernel.rs Kernel::width/height/cx/cy() | パラメータゲッター |
| kernelSetOrigin | ✅ 同等 | kernel.rs Kernel::set_origin() | カーネル原点設定 |
| kernelGetSum | ✅ 同等 | kernel.rs Kernel::sum() | カーネル合計値 |
| kernelGetMinMax | ✅ 同等 | kernel.rs Kernel::min_max() | カーネルMin/Max値 |
| kernelNormalize | ✅ 同等 | kernel.rs Kernel::normalize() | カーネル正規化 |
| kernelInvert | ✅ 同等 | kernel.rs Kernel::invert() | カーネル反転 |
| kernelRead | ✅ 同等 | kernel.rs Kernel::read() | ファイルからカーネル読み込み |
| kernelReadStream | ✅ 同等 | kernel.rs (read()に統合) | ストリーム読み込み（Rust版では統合） |
| kernelWrite | ✅ 同等 | kernel.rs Kernel::write() | ファイルにカーネル書き込み |
| kernelWriteStream | ✅ 同等 | kernel.rs (write()に統合) | ストリーム書き込み（Rust版では統合） |
| kernelCreateFromString | ✅ 同等 | kernel.rs Kernel::from_string() | 文字列パースからカーネル生成 |
| kernelCreateFromFile | ✅ 同等 | kernel.rs Kernel::from_file() | ファイルからカーネル生成 |
| kernelCreateFromPix | ✅ 同等 | kernel.rs Kernel::from_pix() | Pixからカーネル生成 |
| kernelDisplayInPix | ✅ 同等 | kernel.rs Kernel::display_in_pix() | Pix内にカーネル可視化 |

### edge.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixSobelEdgeFilter | ✅ 同等 | sobel_edge() | Sobelエッジ検出 |
| pixTwoSidedEdgeFilter | ❌ 未実装 | - | 両側エッジ勾配フィルタ |
| pixMeasureEdgeSmoothness | ❌ 未実装 | - | エッジ滑らかさ測定 (returns l_ok) |
| pixGetEdgeProfile | ❌ 未実装 | - | エッジプロファイル取得 (returns NUMA*) |
| pixGetLastOffPixelInRun | 🚫 不要 | - | エッジプロファイル用低レベル内部ヘルパー |
| pixGetLastOnPixelInRun | 🚫 不要 | - | エッジプロファイル用低レベル内部ヘルパー |

### enhance.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixGammaTRC | ✅ 同等 | gamma_trc_pix() | ガンマTRCマッピング |
| pixGammaTRCMasked | ✅ 同等 | gamma_trc_masked() | マスク付きガンマTRC |
| pixGammaTRCWithAlpha | ✅ 同等 | gamma_trc_with_alpha() | アルファチャンネル付きガンマTRC |
| numaGammaTRC | ✅ 同等 | gamma_trc() | TrcLut([u8;256])を返す |
| pixContrastTRC | ✅ 同等 | contrast_trc_pix() | コントラストTRC |
| pixContrastTRCMasked | ✅ 同等 | contrast_trc_masked() | マスク付きコントラストTRC |
| numaContrastTRC | ✅ 同等 | contrast_trc() | TrcLut([u8;256])を返す |
| pixEqualizeTRC | ✅ 同等 | equalize_trc_pix() | ヒストグラム均等化TRC |
| numaEqualizeTRC | ✅ 同等 | equalize_trc() | TrcLut([u8;256])を返す |
| pixTRCMap | ✅ 同等 | trc_map() | 汎用TRCマッパー |
| pixTRCMapGeneral | ✅ 同等 | trc_map_general() | R,G,B個別LUT適用 |
| pixUnsharpMasking | ✅ 同等 | enhance.rs unsharp_masking() | アンシャープマスキング(カラー対応) |
| pixUnsharpMaskingGray | ✅ 同等 | unsharp_mask() | グレースケールアンシャープマスキング |
| pixUnsharpMaskingFast | ✅ 同等 | edge.rs unsharp_masking_fast() | 高速アンシャープマスキング(カラー対応) |
| pixUnsharpMaskingGrayFast | ✅ 同等 | edge.rs unsharp_masking_gray_fast() | 高速グレースケールアンシャープマスキング |
| pixUnsharpMaskingGray1D | 🚫 不要 | - | unsharp_masking_gray_fast()の内部ヘルパー（実装済み） |
| pixUnsharpMaskingGray2D | 🚫 不要 | - | unsharp_masking_gray_fast()の内部ヘルパー（実装済み） |
| pixModifyHue | ✅ 同等 | modify_hue() | 色相変更 |
| pixModifySaturation | ✅ 同等 | modify_saturation() | 彩度変更 |
| pixMeasureSaturation | ✅ 同等 | measure_saturation() | 彩度測定 |
| pixModifyBrightness | ✅ 同等 | modify_brightness() | 明度変更 |
| pixMosaicColorShiftRGB | 🚫 不要 | - | デバッグ/表示用モザイク可視化関数 |
| pixColorShiftRGB | ✅ 同等 | color_shift_rgb() | 色シフト |
| pixDarkenGray | ✅ 同等 | darken_gray() | グレーピクセル暗色化 |
| pixMultConstantColor | ✅ 同等 | mult_constant_color() | 定数乗算カラー変換 |
| pixMultMatrixColor | ✅ 同等 | mult_matrix_color() | 行列乗算カラー変換 |
| pixHalfEdgeByBandpass | ❌ 未実装 | - | バンドパスによるハーフエッジ |

### bilateral.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixBilateral | ✅ 同等 | bilateral.rs bilateral() | 高速分離可能バイラテラルフィルタ(カラー/グレー自動判定) |
| pixBilateralGray | ✅ 同等 | bilateral.rs bilateral_gray() | 高速分離可能バイラテラルフィルタ(グレースケール) |
| pixBilateralExact | ✅ 同等 | bilateral_exact() | 厳密バイラテラルフィルタ(カラー/グレー自動判定) |
| pixBilateralGrayExact | ✅ 同等 | bilateral_gray_exact() | 厳密バイラテラルフィルタ(グレースケール) |
| pixBlockBilateralExact | ✅ 同等 | bilateral.rs block_bilateral_exact() | ブロックベース厳密バイラテラルフィルタ |
| makeRangeKernel | ✅ 同等 | make_range_kernel() | レンジカーネル生成 (returns L_KERNEL*) |

### adaptmap.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixCleanBackgroundToWhite | ✅ 同等 | adaptmap.rs clean_background_to_white() | 背景を白にクリーン化 |
| pixBackgroundNormSimple | ✅ 同等 | background_norm_simple() | シンプル背景正規化 |
| pixBackgroundNorm | ✅ 同等 | background_norm() | 背景正規化 |
| pixBackgroundNormMorph | ✅ 同等 | adaptmap.rs background_norm_morph() | モルフォロジーベース背景正規化 |
| pixBackgroundNormGrayArray | ✅ 同等 | adaptmap.rs background_norm_gray_array() | グレー背景正規化配列 |
| pixBackgroundNormRGBArrays | ✅ 同等 | adaptmap.rs background_norm_rgb_arrays() | RGB背景正規化配列 |
| pixBackgroundNormGrayArrayMorph | ✅ 同等 | adaptmap.rs background_norm_gray_array_morph() | モルフォロジーベースグレー背景正規化配列 |
| pixBackgroundNormRGBArraysMorph | ✅ 同等 | adaptmap.rs background_norm_rgb_arrays_morph() | モルフォロジーベースRGB背景正規化配列 |
| pixGetBackgroundGrayMap | ✅ 同等 | adaptmap.rs get_background_gray_map() | グレー背景マップ取得 |
| pixGetBackgroundRGBMap | ✅ 同等 | adaptmap.rs get_background_rgb_map() | RGB背景マップ取得 |
| pixGetBackgroundGrayMapMorph | ✅ 同等 | adaptmap.rs get_background_gray_map_morph() | モルフォロジーベースグレー背景マップ取得 |
| pixGetBackgroundRGBMapMorph | ✅ 同等 | adaptmap.rs get_background_rgb_map_morph() | モルフォロジーベースRGB背景マップ取得 |
| pixFillMapHoles | ✅ 同等 | adaptmap.rs fill_map_holes() | マップの穴埋め |
| pixExtendByReplication | ✅ 同等 | adaptmap.rs extend_by_replication() | 複製による拡張 |
| pixSmoothConnectedRegions | ✅ 同等 | adaptmap.rs smooth_connected_regions() | 連結領域の平滑化 |
| pixGetForegroundGrayMap | ❌ 未実装 | - | グレー前景マップ取得 (returns l_int32) |
| pixGetInvBackgroundMap | ✅ 同等 | adaptmap.rs get_inv_background_map() | 逆背景マップ取得 |
| pixApplyInvBackgroundGrayMap | ✅ 同等 | adaptmap.rs apply_inv_background_gray_map() | グレー逆背景マップ適用 |
| pixApplyInvBackgroundRGBMap | ✅ 同等 | adaptmap.rs apply_inv_background_rgb_map() | RGB逆背景マップ適用 |
| pixApplyVariableGrayMap | ✅ 同等 | adaptmap.rs apply_variable_gray_map() | 可変グレーマップ適用 |
| pixGlobalNormRGB | ✅ 同等 | adaptmap.rs global_norm_rgb() | グローバルRGB正規化 |
| pixGlobalNormNoSatRGB | ✅ 同等 | adaptmap.rs global_norm_no_sat_rgb() | 彩度保持グローバルRGB正規化 |
| pixThresholdSpreadNorm | ✅ 同等 | adaptmap.rs threshold_spread_norm() | 閾値スプレッド正規化 |
| pixBackgroundNormFlex | ✅ 同等 | adaptmap.rs background_norm_flex() | フレキシブル背景正規化 |
| pixContrastNorm | ✅ 同等 | contrast_norm() | コントラスト正規化 |
| pixMinMaxTiles | 🚫 不要 | - | static内部ヘルパー（contrast_norm内で実装済み） |
| pixSetLowContrast | 🚫 不要 | - | static内部ヘルパー（contrast_norm内で実装済み） |
| pixLinearTRCTiled | 🚫 不要 | - | static内部ヘルパー（contrast_norm内で実装済み） |
| pixBackgroundNormTo1MinMax | ✅ 同等 | adaptmap.rs background_norm_to_1_min_max() | 背景正規化→1 bpp MinMax |
| pixConvertTo8MinMax | ✅ 同等 | adaptmap.rs convert_to_8_min_max() | 8 bpp MinMax変換 |
| pixSelectiveContrastMod | 🚫 不要 | - | static内部ヘルパー |

### rank.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixRankFilter | ✅ 同等 | rank_filter() | ランクフィルタ(グレー/カラー自動判定) |
| pixRankFilterRGB | ✅ 同等 | rank_filter_color() | RGBランクフィルタ |
| pixRankFilterGray | ✅ 同等 | rank_filter_gray() | グレースケールランクフィルタ |
| pixMedianFilter | ✅ 同等 | median_filter() | メディアンフィルタ |
| pixRankFilterWithScaling | ❌ 未実装 | - | スケーリング加速付きランクフィルタ |

## 実装状況分析

### 実装済み機能

1. **基本畳み込み**: convolve(), convolve_color(), box_blur(), gaussian_blur()
2. **ブロック畳み込み**: blockconv(), blockconv_gray(), blockconv_accum(), blockconv_gray_unnormalized()
3. **分離可能畳み込み**: convolve_sep(), convolve_rgb_sep()
4. **ウィンドウ統計**: windowed_stats(), windowed_mean(), windowed_mean_square(), windowed_variance(), mean_square_accum()
5. **エッジ検出**: sobel_edge(), laplacian_edge(), sharpen(), emboss()
6. **アンシャープマスク**: unsharp_mask(), unsharp_masking_fast(), unsharp_masking_gray_fast()
7. **バイラテラルフィルタ**: bilateral(), bilateral_gray(), bilateral_exact(), bilateral_gray_exact(), block_bilateral_exact(), make_range_kernel()
8. **ランクフィルタ**: rank_filter(), rank_filter_gray(), rank_filter_color(), median_filter(), min_filter(), max_filter()
9. **適応マッピング**: background_norm(), background_norm_simple(), contrast_norm(), contrast_norm_simple()
10. **その他**: blockrank(), blocksum(), census_transform(), add_gaussian_noise()

### 主要な未実装機能

#### 中優先度
1. **エッジ検出**: pixTwoSidedEdgeFilter, pixHalfEdgeByBandpass
2. **エッジ測定**: pixMeasureEdgeSmoothness, pixGetEdgeProfile
3. **適応マップ**: pixGetForegroundGrayMap

#### 低優先度
4. **ランクフィルタ**: pixRankFilterWithScaling（スケーリング加速付き）

### 不要と判断した機能（🚫 不要: 11件）

1. **C固有パターン**: l_setConvolveSampling（グローバル変数セッター）
2. **内部ヘルパー（実装済み高レベルAPIでカバー）**:
   - gaussDistribSampling（add_gaussian_noise内で実装済み）
   - pixUnsharpMaskingGray1D/2D（unsharp_masking_gray_fast内で実装済み）
   - pixMinMaxTiles, pixSetLowContrast, pixLinearTRCTiled, pixSelectiveContrastMod（static、contrast_norm等で実装済み）
3. **低レベルヘルパー**: pixGetLastOffPixelInRun, pixGetLastOnPixelInRun（エッジプロファイル内部関数）
4. **デバッグ/表示専用**: pixMosaicColorShiftRGB（モザイク可視化）

## 設計ノート

### Rust版の特徴
- エラー処理は`FilterResult<T>`で統一
- カーネルは独自の`Kernel`型を使用(L_KERNELとは非互換)
- 一部関数はRust慣用的な名前に変更(例: pixSobelEdgeFilter → sobel_edge)
- 高速化のための低レベル実装は未実装(ブロック畳み込み、分離可能畳み込み等)

### C版の戦略
- ブロック畳み込みによる高速化を多用
- アキュムレータベースの最適化
- タイル化による大画像処理対応
- 分離可能畳み込みによる計算量削減

### 今後の実装推奨順序
1. エッジ検出バリエーション（pixTwoSidedEdgeFilter, pixHalfEdgeByBandpass）
2. エッジ測定関数（pixMeasureEdgeSmoothness, pixGetEdgeProfile）
3. 適応マップ（pixGetForegroundGrayMap）
4. スケーリング付きランクフィルタ（pixRankFilterWithScaling）
