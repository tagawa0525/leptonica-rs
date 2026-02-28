# leptonica (src/filter/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## サマリー

| 項目      | 数  |
| --------- | --- |
| ✅ 同等   | 107 |
| 🔄 異なる | 0   |
| ❌ 未実装 | 0   |
| 🚫 不要   | 11  |
| 合計      | 118 |

## 詳細

### convolve.c

#### block_conv.rs

| C関数                        | 状態 | Rust対応                      | 備考                                   |
| ---------------------------- | ---- | ----------------------------- | -------------------------------------- |
| pixBlockconv                 | ✅   | blockconv()                   | ブロック畳み込み(自動でgray/color判定) |
| pixBlockconvGray             | ✅   | blockconv_gray()              | グレースケールブロック畳み込み         |
| pixBlockconvAccum            | ✅   | blockconv_accum()             | 畳み込み用アキュムレータ               |
| pixBlockconvGrayUnnormalized | ✅   | blockconv_gray_unnormalized() | 正規化なしブロック畳み込み             |
| pixBlockconvTiled            | ✅   | blockconv_tiled()             | タイル化ブロック畳み込み               |
| pixBlockconvGrayTile         | ✅   | blockconv_gray_tile()         | グレースケールタイル化ブロック畳み込み |

#### windowed.rs

| C関数                 | 状態 | Rust対応               | 備考                                               |
| --------------------- | ---- | ---------------------- | -------------------------------------------------- |
| pixWindowedStats      | ✅   | windowed_stats()       | ウィンドウ統計量(mean, mean-square, variance, RMS) |
| pixWindowedMean       | ✅   | windowed_mean()        | ウィンドウ平均                                     |
| pixWindowedMeanSquare | ✅   | windowed_mean_square() | ウィンドウ平均二乗                                 |
| pixWindowedVariance   | ✅   | windowed_variance()    | ウィンドウ分散                                     |
| pixMeanSquareAccum    | ✅   | mean_square_accum()    | 平均二乗アキュムレータ (returns DPIX*)             |

#### convolve.rs

| C関数               | 状態 | Rust対応             | 備考                                  |
| ------------------- | ---- | -------------------- | ------------------------------------- |
| pixBlockrank        | ✅   | blockrank()          | バイナリブロックランクフィルタ        |
| pixBlocksum         | ✅   | blocksum()           | バイナリブロック和                    |
| pixCensusTransform  | ✅   | census_transform()   | センサス変換                          |
| pixConvolveSep      | ✅   | convolve_sep()       | 分離可能畳み込み                      |
| pixConvolveRGBSep   | ✅   | convolve_rgb_sep()   | RGB分離可能畳み込み                   |
| fpixConvolve        | ✅   | fpix_convolve()      | 浮動小数点畳み込み (FPix対応)         |
| fpixConvolveSep     | ✅   | fpix_convolve_sep()  | 浮動小数点分離可能畳み込み (FPix対応) |
| pixConvolveWithBias | ✅   | convolve_with_bias() | バイアス付き畳み込み                  |
| pixAddGaussianNoise | ✅   | add_gaussian_noise() | ガウシアンノイズ追加                  |

#### その他

| C関数          | 状態 | Rust対応         | 備考         |
| -------------- | ---- | ---------------- | ------------ |
| pixConvolve    | ✅   | convolve()       | 汎用畳み込み |
| pixConvolveRGB | ✅   | convolve_color() | RGB畳み込み  |

#### 対応なし

| C関数                 | 状態 | Rust対応 | 備考                                                   |
| --------------------- | ---- | -------- | ------------------------------------------------------ |
| l_setConvolveSampling | 🚫   | -        | グローバル変数セッター（C固有パターン）                |
| gaussDistribSampling  | 🚫   | -        | Box-Muller法ヘルパー; add_gaussian_noise()内で実装済み |

### kernel.c

#### kernel.rs

| C関数               | 状態 | Rust対応                                 | 備考                       |
| ------------------- | ---- | ---------------------------------------- | -------------------------- |
| kernelCreate        | ✅   | Kernel::new()                            | カーネル生成（初期値ゼロ） |
| kernelGetElement    | ✅   | Kernel::get()                            | 要素読み込み               |
| kernelSetElement    | ✅   | Kernel::set()                            | 要素書き込み               |
| kernelGetParameters | ✅   | Kernel::width/height/center_x/center_y() | パラメータゲッター         |
| kernelSetOrigin     | ✅   | Kernel::set_center()                     | カーネル原点設定           |
| kernelGetSum        | ✅   | Kernel::sum()                            | カーネル合計値             |
| kernelNormalize     | ✅   | Kernel::normalize()                      | カーネル正規化             |

#### 対応なし

| C関数              | 状態 | Rust対応 | 備考                       |
| ------------------ | ---- | -------- | -------------------------- |
| kernelDestroy      | ✅   | -        | Rust自動Drop（メモリ管理） |
| create2dFloatArray | 🚫   | -        | C配列確保用内部ヘルパー    |

#### その他

| C関数                  | 状態 | Rust対応               | 備考                          |
| ---------------------- | ---- | ---------------------- | ----------------------------- |
| kernelCopy             | ✅   | Kernel::clone()        | カーネルクローン（Copy/Clone) |
| kernelGetMinMax        | ✅   | Kernel::get_min_max    | カーネルMin/Max値             |
| kernelInvert           | ✅   | Kernel::invert         | カーネル反転                  |
| kernelRead             | ✅   | Kernel::read           | ファイルからカーネル読み込み  |
| kernelReadStream       | ✅   | Kernel::read           | ストリーム読み込み            |
| kernelWrite            | ✅   | Kernel::write          | ファイルにカーネル書き込み    |
| kernelWriteStream      | ✅   | Kernel::write          | ストリーム書き込み            |
| kernelCreateFromString | ✅   | Kernel::from_string    | 文字列パースからカーネル生成  |
| kernelCreateFromFile   | ✅   | Kernel::from_file      | ファイルからカーネル生成      |
| kernelCreateFromPix    | ✅   | Kernel::from_pix       | Pixからカーネル生成           |
| kernelDisplayInPix     | ✅   | Kernel::display_in_pix | Pix内にカーネル可視化         |

### edge.c

| C関数                    | 状態 | Rust対応                | 備考                                     |
| ------------------------ | ---- | ----------------------- | ---------------------------------------- |
| pixSobelEdgeFilter       | ✅   | sobel_edge()            | Sobelエッジ検出                          |
| pixTwoSidedEdgeFilter    | ✅   | two_sided_edge_filter   | 両側エッジ勾配フィルタ                   |
| pixMeasureEdgeSmoothness | ✅   | measure_edge_smoothness | エッジ滑らかさ測定 (returns l_ok)        |
| pixGetEdgeProfile        | ✅   | get_edge_profile        | エッジプロファイル取得 (returns NUMA*)   |
| pixGetLastOffPixelInRun  | 🚫   | -                       | エッジプロファイル用低レベル内部ヘルパー |
| pixGetLastOnPixelInRun   | 🚫   | -                       | エッジプロファイル用低レベル内部ヘルパー |

### enhance.c

#### その他

| C関数                 | 状態 | Rust対応               | 備考                            |
| --------------------- | ---- | ---------------------- | ------------------------------- |
| pixGammaTRC           | ✅   | gamma_trc_pix()        | ガンマTRCマッピング             |
| pixGammaTRCMasked     | ✅   | gamma_trc_masked()     | マスク付きガンマTRC             |
| pixGammaTRCWithAlpha  | ✅   | gamma_trc_with_alpha() | アルファチャンネル付きガンマTRC |
| numaGammaTRC          | ✅   | gamma_trc()            | TrcLut([u8;256])を返す          |
| pixContrastTRC        | ✅   | contrast_trc_pix()     | コントラストTRC                 |
| pixContrastTRCMasked  | ✅   | contrast_trc_masked()  | マスク付きコントラストTRC       |
| numaContrastTRC       | ✅   | contrast_trc()         | TrcLut([u8;256])を返す          |
| pixEqualizeTRC        | ✅   | equalize_trc_pix()     | ヒストグラム均等化TRC           |
| numaEqualizeTRC       | ✅   | equalize_trc()         | TrcLut([u8;256])を返す          |
| pixTRCMap             | ✅   | trc_map()              | 汎用TRCマッパー                 |
| pixTRCMapGeneral      | ✅   | trc_map_general()      | R,G,B個別LUT適用                |
| pixModifyHue          | ✅   | modify_hue()           | 色相変更                        |
| pixModifySaturation   | ✅   | modify_saturation()    | 彩度変更                        |
| pixMeasureSaturation  | ✅   | measure_saturation()   | 彩度測定                        |
| pixModifyBrightness   | ✅   | modify_brightness()    | 明度変更                        |
| pixColorShiftRGB      | ✅   | color_shift_rgb()      | 色シフト                        |
| pixDarkenGray         | ✅   | darken_gray()          | グレーピクセル暗色化            |
| pixMultConstantColor  | ✅   | mult_constant_color()  | 定数乗算カラー変換              |
| pixMultMatrixColor    | ✅   | mult_matrix_color()    | 行列乗算カラー変換              |
| pixHalfEdgeByBandpass | ✅   | half_edge_by_bandpass  | バンドパスによるハーフエッジ    |

#### enhance.rs

| C関数                 | 状態 | Rust対応               | 備考                                 |
| --------------------- | ---- | ---------------------- | ------------------------------------ |
| pixUnsharpMasking     | ✅   | unsharp_masking()      | アンシャープマスキング(カラー対応)   |
| pixUnsharpMaskingGray | ✅   | unsharp_masking_gray() | グレースケールアンシャープマスキング |

#### edge.rs

| C関数                     | 状態 | Rust対応                    | 備考                                     |
| ------------------------- | ---- | --------------------------- | ---------------------------------------- |
| pixUnsharpMaskingFast     | ✅   | unsharp_masking_fast()      | 高速アンシャープマスキング(カラー対応)   |
| pixUnsharpMaskingGrayFast | ✅   | unsharp_masking_gray_fast() | 高速グレースケールアンシャープマスキング |

#### 対応なし

| C関数                   | 状態 | Rust対応 | 備考                                                  |
| ----------------------- | ---- | -------- | ----------------------------------------------------- |
| pixUnsharpMaskingGray1D | 🚫   | -        | unsharp_masking_gray_fast()の内部ヘルパー（実装済み） |
| pixUnsharpMaskingGray2D | 🚫   | -        | unsharp_masking_gray_fast()の内部ヘルパー（実装済み） |
| pixMosaicColorShiftRGB  | 🚫   | -        | デバッグ/表示用モザイク可視化関数                     |

### bilateral.c

#### bilateral.rs

| C関数                  | 状態 | Rust対応                | 備考                                                    |
| ---------------------- | ---- | ----------------------- | ------------------------------------------------------- |
| pixBilateral           | ✅   | bilateral()             | 高速分離可能バイラテラルフィルタ(カラー/グレー自動判定) |
| pixBilateralGray       | ✅   | bilateral_gray()        | 高速分離可能バイラテラルフィルタ(グレースケール)        |
| pixBlockBilateralExact | ✅   | block_bilateral_exact() | ブロックベース厳密バイラテラルフィルタ                  |

#### その他

| C関数                 | 状態 | Rust対応               | 備考                                            |
| --------------------- | ---- | ---------------------- | ----------------------------------------------- |
| pixBilateralExact     | ✅   | bilateral_exact()      | 厳密バイラテラルフィルタ(カラー/グレー自動判定) |
| pixBilateralGrayExact | ✅   | bilateral_gray_exact() | 厳密バイラテラルフィルタ(グレースケール)        |
| makeRangeKernel       | ✅   | make_range_kernel()    | レンジカーネル生成 (returns L_KERNEL*)          |

### adaptmap.c

#### adaptmap.rs

| C関数                           | 状態 | Rust対応                           | 備考                                     |
| ------------------------------- | ---- | ---------------------------------- | ---------------------------------------- |
| pixCleanBackgroundToWhite       | ✅   | clean_background_to_white()        | 背景を白にクリーン化                     |
| pixBackgroundNormMorph          | ✅   | background_norm_morph()            | モルフォロジーベース背景正規化           |
| pixBackgroundNormGrayArray      | ✅   | background_norm_gray_array()       | グレー背景正規化配列                     |
| pixBackgroundNormRGBArrays      | ✅   | background_norm_rgb_arrays()       | RGB背景正規化配列                        |
| pixBackgroundNormGrayArrayMorph | ✅   | background_norm_gray_array_morph() | モルフォロジーベースグレー背景正規化配列 |
| pixBackgroundNormRGBArraysMorph | ✅   | background_norm_rgb_arrays_morph() | モルフォロジーベースRGB背景正規化配列    |
| pixGetBackgroundGrayMap         | ✅   | get_background_gray_map()          | グレー背景マップ取得                     |
| pixGetBackgroundRGBMap          | ✅   | get_background_rgb_map()           | RGB背景マップ取得                        |
| pixGetBackgroundGrayMapMorph    | ✅   | get_background_gray_map_morph()    | モルフォロジーベースグレー背景マップ取得 |
| pixGetBackgroundRGBMapMorph     | ✅   | get_background_rgb_map_morph()     | モルフォロジーベースRGB背景マップ取得    |
| pixFillMapHoles                 | ✅   | fill_map_holes()                   | マップの穴埋め                           |
| pixExtendByReplication          | ✅   | extend_by_replication()            | 複製による拡張                           |
| pixSmoothConnectedRegions       | ✅   | smooth_connected_regions()         | 連結領域の平滑化                         |
| pixGetInvBackgroundMap          | ✅   | get_inv_background_map()           | 逆背景マップ取得                         |
| pixApplyInvBackgroundGrayMap    | ✅   | apply_inv_background_gray_map()    | グレー逆背景マップ適用                   |
| pixApplyInvBackgroundRGBMap     | ✅   | apply_inv_background_rgb_map()     | RGB逆背景マップ適用                      |
| pixApplyVariableGrayMap         | ✅   | apply_variable_gray_map()          | 可変グレーマップ適用                     |
| pixGlobalNormRGB                | ✅   | global_norm_rgb()                  | グローバルRGB正規化                      |
| pixGlobalNormNoSatRGB           | ✅   | global_norm_no_sat_rgb()           | 彩度保持グローバルRGB正規化              |
| pixThresholdSpreadNorm          | ✅   | threshold_spread_norm()            | 閾値スプレッド正規化                     |
| pixBackgroundNormFlex           | ✅   | background_norm_flex()             | フレキシブル背景正規化                   |
| pixBackgroundNormTo1MinMax      | ✅   | background_norm_to_1_min_max()     | 背景正規化→1 bpp MinMax                  |
| pixConvertTo8MinMax             | ✅   | convert_to_8_min_max()             | 8 bpp MinMax変換                         |

#### その他

| C関数                   | 状態 | Rust対応                          | 備考                 |
| ----------------------- | ---- | --------------------------------- | -------------------- |
| pixBackgroundNormSimple | ✅   | background_norm_simple()          | シンプル背景正規化   |
| pixBackgroundNorm       | ✅   | background_norm()                 | 背景正規化           |
| pixGetForegroundGrayMap | ✅   | adaptmap::get_foreground_gray_map | グレー前景マップ取得 |
| pixContrastNorm         | ✅   | contrast_norm()                   | コントラスト正規化   |

#### 対応なし

| C関数                   | 状態 | Rust対応 | 備考                                           |
| ----------------------- | ---- | -------- | ---------------------------------------------- |
| pixMinMaxTiles          | 🚫   | -        | プライベート関数 `min_max_tiles` として存在    |
| pixSetLowContrast       | 🚫   | -        | プライベート関数 `set_low_contrast` として存在 |
| pixLinearTRCTiled       | 🚫   | -        | プライベート関数 `linear_trc_tiled` として存在 |
| iaaGetLinearTRC         | 🚫   | -        | static内部ヘルパー                             |
| pixSelectiveContrastMod | 🚫   | -        | static内部ヘルパー                             |

### rank.c

| C関数                    | 状態 | Rust対応                 | 備考                                  |
| ------------------------ | ---- | ------------------------ | ------------------------------------- |
| pixRankFilter            | ✅   | rank_filter()            | ランクフィルタ(グレー/カラー自動判定) |
| pixRankFilterRGB         | ✅   | rank_filter_color()      | RGBランクフィルタ                     |
| pixRankFilterGray        | ✅   | rank_filter_gray()       | グレースケールランクフィルタ          |
| pixMedianFilter          | ✅   | median_filter()          | メディアンフィルタ                    |
| pixRankFilterWithScaling | ✅   | rank_filter_with_scaling | スケーリング加速付きランクフィルタ    |

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

### 実装完了した機能（元未実装 → 全て実装済み）

以下の関数が実装された:

1. **エッジ検出**: pixTwoSidedEdgeFilter, pixHalfEdgeByBandpass — 実装済み
2. **エッジ測定**: pixMeasureEdgeSmoothness, pixGetEdgeProfile — 実装済み
3. **ランクフィルタ**: pixRankFilterWithScaling — 実装済み

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

### C版の戦略

- ブロック畳み込みによる高速化を多用
- アキュムレータベースの最適化
- タイル化による大画像処理対応
- 分離可能畳み込みによる計算量削減

### 未実装（❌ 0件）

全関数が実装済み。
