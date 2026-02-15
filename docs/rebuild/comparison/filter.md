# leptonica-filter: C版 vs Rust版 関数レベル比較

調査日: 2026-02-15

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 11 |
| 🔄 異なる | 0 |
| ❌ 未実装 | 83 |
| 合計 | 94 |

## 詳細

### convolve.c

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixBlockconv | ❌ 未実装 | - | ブロック畳み込み(自動でgray/color判定) |
| pixBlockconvGray | ❌ 未実装 | - | グレースケールブロック畳み込み |
| pixBlockconvAccum | ❌ 未実装 | - | 畳み込み用アキュムレータ |
| pixBlockconvGrayUnnormalized | ❌ 未実装 | - | 正規化なしブロック畳み込み |
| pixBlockconvTiled | ❌ 未実装 | - | タイル化ブロック畳み込み |
| pixBlockconvGrayTile | ❌ 未実装 | - | グレースケールタイル化ブロック畳み込み |
| pixWindowedStats | ❌ 未実装 | - | ウィンドウ統計量(mean, mean-square, variance, RMS) |
| pixWindowedMean | ❌ 未実装 | - | ウィンドウ平均 |
| pixWindowedMeanSquare | ❌ 未実装 | - | ウィンドウ平均二乗 |
| pixWindowedVariance | ❌ 未実装 | - | ウィンドウ分散 |
| pixMeanSquareAccum | ❌ 未実装 | - | 平均二乗アキュムレータ (returns DPIX*) |
| pixBlockrank | ❌ 未実装 | - | バイナリブロックランクフィルタ |
| pixBlocksum | ❌ 未実装 | - | バイナリブロック和 |
| pixCensusTransform | ❌ 未実装 | - | センサス変換 |
| pixConvolve | ✅ 同等 | convolve() | 汎用畳み込み |
| pixConvolveSep | ❌ 未実装 | - | 分離可能畳み込み |
| pixConvolveRGB | ✅ 同等 | convolve_color() | RGB畳み込み |
| pixConvolveRGBSep | ❌ 未実装 | - | RGB分離可能畳み込み |
| fpixConvolve | ❌ 未実装 | - | 浮動小数点畳み込み (operates on FPIX) |
| fpixConvolveSep | ❌ 未実装 | - | 浮動小数点分離可能畳み込み (operates on FPIX) |
| pixConvolveWithBias | ❌ 未実装 | - | バイアス付き畳み込み |
| l_setConvolveSampling | ❌ 未実装 | - | 畳み込みサブサンプリングパラメータ設定 (void) |
| pixAddGaussianNoise | ❌ 未実装 | - | ガウシアンノイズ追加 |
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
| pixUnsharpMaskingFast | ❌ 未実装 | - | 高速アンシャープマスキング(カラー対応) |
| pixUnsharpMaskingGrayFast | ❌ 未実装 | - | 高速グレースケールアンシャープマスキング |
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
| pixBilateral | ❌ 未実装 | - | 高速分離可能バイラテラルフィルタ(カラー/グレー自動判定) |
| pixBilateralGray | ❌ 未実装 | - | 高速分離可能バイラテラルフィルタ(グレースケール) |
| pixBilateralExact | ✅ 同等 | bilateral_exact() | 厳密バイラテラルフィルタ(カラー/グレー自動判定) |
| pixBilateralGrayExact | ✅ 同等 | bilateral_gray_exact() | 厳密バイラテラルフィルタ(グレースケール) |
| pixBlockBilateralExact | ❌ 未実装 | - | ブロックベース厳密バイラテラルフィルタ |
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
2. **エッジ検出**: sobel_edge(), laplacian_edge(), sharpen(), emboss()
3. **アンシャープマスク**: unsharp_mask() (基本実装のみ)
4. **バイラテラルフィルタ**: bilateral_exact(), bilateral_gray_exact(), make_range_kernel()
5. **ランクフィルタ**: rank_filter(), rank_filter_gray(), rank_filter_color(), median_filter(), min_filter(), max_filter()
6. **適応マッピング**: background_norm(), background_norm_simple(), contrast_norm(), contrast_norm_simple()

### 主要な未実装機能

#### 高優先度
1. **ブロック畳み込み最適化**: pixBlockconv系の高速実装群
2. **分離可能畳み込み**: pixConvolveSep, pixConvolveRGBSep
3. **ウィンドウ統計**: pixWindowedMean, pixWindowedVariance等
4. **高速バイラテラル**: pixBilateral, pixBilateralGray (分離可能近似版)
5. **enhance.c全般**: TRCマッピング、色調整、アンシャープマスキングバリエーション

#### 中優先度
6. **センサス変換**: pixCensusTransform
7. **エッジ測定**: pixMeasureEdgeSmoothness, pixGetEdgeProfile
8. **adaptmap.c詳細機能**: モルフォロジーベース正規化、マップ操作群
9. **カラー変換**: pixModifyHue, pixModifySaturation, pixColorShiftRGB等

#### 低優先度
10. **ノイズ追加**: pixAddGaussianNoise
11. **バイナリ操作**: pixBlockrank, pixBlocksum
12. **補助関数**: l_setConvolveSampling等

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
1. 分離可能畳み込み(性能向上に直結)
2. ブロック畳み込み系(pixBlockconv, pixWindowedMean等)
3. enhance.c の主要TRC関数(ガンマ、コントラスト、均等化)
4. 高速バイラテラルフィルタ(pixBilateral, pixBilateralGray)
5. adaptmap.c の詳細機能(モルフォロジーベース等)
