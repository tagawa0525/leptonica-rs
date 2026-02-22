# leptonica-filter: C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 56 |
| 🔄 異なる | 0 |
| ❌ 未実装 | 43 |
| 合計 | 99 |

## 詳細

### convolve.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixBlockconv | ✅ 同等 | block_conv.rs blockconv() | ブロック畳み込み(自動でgray/color判定) |
| pixBlockconvGray | ✅ 同等 | block_conv.rs blockconv_gray() | グレースケールブロック畳み込み |
| pixBlockconvAccum | ✅ 同等 | block_conv.rs blockconv_accum() | 畳み込み用アキュムレータ |
| pixBlockconvGrayUnnormalized | ✅ 同等 | block_conv.rs blockconv_gray_unnormalized() | 正規化なしブロック畳み込み |
| pixBlockconvTiled | ❌ 未実装 | - | タイル化ブロック畳み込み |
| pixBlockconvGrayTile | ❌ 未実装 | - | グレースケールタイル化ブロック畳み込み |
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
| l_setConvolveSampling | ❌ 未実装 | - | 畳み込みサブサンプリングパラメータ設定 (void) |
| pixAddGaussianNoise | ✅ 同等 | convolve.rs add_gaussian_noise() | ガウシアンノイズ追加 |
| gaussDistribSampling | ❌ 未実装 | - | ガウス分布サンプリング (returns l_float32) |

### edge.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixSobelEdgeFilter | ✅ 同等 | sobel_edge() | Sobelエッジ検出 |
| pixTwoSidedEdgeFilter | ❌ 未実装 | - | 両側エッジ勾配フィルタ |
| pixMeasureEdgeSmoothness | ❌ 未実装 | - | エッジ滑らかさ測定 (returns l_ok) |
| pixGetEdgeProfile | ❌ 未実装 | - | エッジプロファイル取得 (returns NUMA*) |
| pixGetLastOffPixelInRun | ❌ 未実装 | - | ランの最後のOFFピクセル取得 (returns l_ok) |
| pixGetLastOnPixelInRun | ❌ 未実装 | - | ランの最後のONピクセル取得 (returns l_int32) |

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
| pixUnsharpMasking | ❌ 未実装 | - | アンシャープマスキング(カラー対応) |
| pixUnsharpMaskingGray | ✅ 同等 | unsharp_mask() | グレースケールアンシャープマスキング |
| pixUnsharpMaskingFast | ✅ 同等 | edge.rs unsharp_masking_fast() | 高速アンシャープマスキング(カラー対応) |
| pixUnsharpMaskingGrayFast | ✅ 同等 | edge.rs unsharp_masking_gray_fast() | 高速グレースケールアンシャープマスキング |
| pixUnsharpMaskingGray1D | ❌ 未実装 | - | 1Dグレースケールアンシャープマスキング |
| pixUnsharpMaskingGray2D | ❌ 未実装 | - | 2Dグレースケールアンシャープマスキング |
| pixModifyHue | ✅ 同等 | modify_hue() | 色相変更 |
| pixModifySaturation | ✅ 同等 | modify_saturation() | 彩度変更 |
| pixMeasureSaturation | ✅ 同等 | measure_saturation() | 彩度測定 |
| pixModifyBrightness | ✅ 同等 | modify_brightness() | 明度変更 |
| pixMosaicColorShiftRGB | ❌ 未実装 | - | モザイク色シフト |
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
| pixCleanBackgroundToWhite | ❌ 未実装 | - | 背景を白にクリーン化 |
| pixBackgroundNormSimple | ✅ 同等 | background_norm_simple() | シンプル背景正規化 |
| pixBackgroundNorm | ✅ 同等 | background_norm() | 背景正規化 |
| pixBackgroundNormMorph | ❌ 未実装 | - | モルフォロジーベース背景正規化 |
| pixBackgroundNormGrayArray | ❌ 未実装 | - | グレー背景正規化配列 (returns l_int32) |
| pixBackgroundNormRGBArrays | ❌ 未実装 | - | RGB背景正規化配列 (returns l_int32) |
| pixBackgroundNormGrayArrayMorph | ❌ 未実装 | - | モルフォロジーベースグレー背景正規化配列 (returns l_int32) |
| pixBackgroundNormRGBArraysMorph | ❌ 未実装 | - | モルフォロジーベースRGB背景正規化配列 (returns l_int32) |
| pixGetBackgroundGrayMap | ❌ 未実装 | - | グレー背景マップ取得 (returns l_int32) |
| pixGetBackgroundRGBMap | ❌ 未実装 | - | RGB背景マップ取得 (returns l_int32) |
| pixGetBackgroundGrayMapMorph | ❌ 未実装 | - | モルフォロジーベースグレー背景マップ取得 (returns l_int32) |
| pixGetBackgroundRGBMapMorph | ❌ 未実装 | - | モルフォロジーベースRGB背景マップ取得 (returns l_int32) |
| pixFillMapHoles | ❌ 未実装 | - | マップの穴埋め (returns l_int32) |
| pixExtendByReplication | ❌ 未実装 | - | 複製による拡張 |
| pixSmoothConnectedRegions | ❌ 未実装 | - | 連結領域の平滑化 (returns l_int32) |
| pixGetForegroundGrayMap | ❌ 未実装 | - | グレー前景マップ取得 (returns l_int32) |
| pixGetInvBackgroundMap | ❌ 未実装 | - | 逆背景マップ取得 |
| pixApplyInvBackgroundGrayMap | ❌ 未実装 | - | グレー逆背景マップ適用 |
| pixApplyInvBackgroundRGBMap | ❌ 未実装 | - | RGB逆背景マップ適用 |
| pixApplyVariableGrayMap | ❌ 未実装 | - | 可変グレーマップ適用 |
| pixGlobalNormRGB | ❌ 未実装 | - | グローバルRGB正規化 |
| pixGlobalNormNoSatRGB | ❌ 未実装 | - | 彩度保持グローバルRGB正規化 |
| pixThresholdSpreadNorm | ❌ 未実装 | - | 閾値スプレッド正規化 (returns l_int32) |
| pixBackgroundNormFlex | ❌ 未実装 | - | フレキシブル背景正規化 |
| pixContrastNorm | ✅ 同等 | contrast_norm() | コントラスト正規化 |
| pixMinMaxTiles | ❌ 未実装 | - | タイル最小最大値 (static, returns l_int32) |
| pixSetLowContrast | ❌ 未実装 | - | 低コントラスト設定 (static, returns l_int32) |
| pixLinearTRCTiled | ❌ 未実装 | - | タイル線形TRC (static) |
| pixBackgroundNormTo1MinMax | ❌ 未実装 | - | 背景正規化→1 bpp MinMax |
| pixConvertTo8MinMax | ❌ 未実装 | - | 8 bpp MinMax変換 |
| pixSelectiveContrastMod | ❌ 未実装 | - | 選択的コントラスト変更 (static, returns l_int32*) |

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

#### 高優先度
1. **adaptmap.c詳細機能**: モルフォロジーベース正規化、マップ操作群

#### 中優先度
2. **エッジ測定**: pixMeasureEdgeSmoothness, pixGetEdgeProfile
3. **タイル化畳み込み**: pixBlockconvTiled, pixBlockconvGrayTile
4. **アンシャープマスクバリエーション**: pixUnsharpMasking (カラー), pixUnsharpMaskingGray1D/2D

#### 低優先度
5. **補助関数**: l_setConvolveSampling, gaussDistribSampling等

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
1. adaptmap.c の詳細機能（モルフォロジーベース背景正規化、マップユーティリティ）
2. 高速バイラテラルフィルタ（pixBilateral, pixBilateralGray）
3. タイル化畳み込み（pixBlockconvTiled, pixBlockconvGrayTile）
4. 残りのアンシャープマスクバリエーション
5. エッジ測定関数
