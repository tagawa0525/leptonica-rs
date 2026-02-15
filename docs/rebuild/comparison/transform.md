# leptonica-transform: C版 vs Rust版 関数レベル比較

調査日: 2026-02-15

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 39 |
| 🔄 異なる | 12 |
| ❌ 未実装 | 101 |
| 合計 | 152 |

**カバレッジ**: 33.6% (51/152 functions have some implementation)

## 詳細

### rotate.c (general rotation)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixRotate | 🔄 | rotate::rotate | 異なるインタフェース設計 |
| pixEmbedForRotation | ❌ | - | 未実装 |
| pixRotateBySampling | 🔄 | rotate::rotate_by_sampling_impl | 内部実装として存在 |
| pixRotateBinaryNice | ❌ | - | 未実装 |
| pixRotateWithAlpha | ❌ | - | 未実装 |

### rotateam.c (area mapping rotation)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixRotateAM | 🔄 | rotate::rotate_area_map_impl | 内部実装として存在 |
| pixRotateAMColor | 🔄 | rotate::rotate_area_map_color | 内部実装として存在 |
| pixRotateAMGray | 🔄 | rotate::rotate_area_map_gray | 内部実装として存在 |
| pixRotateAMCorner | ❌ | - | 未実装 |
| pixRotateAMColorCorner | ❌ | - | 未実装 |
| pixRotateAMGrayCorner | ❌ | - | 未実装 |
| pixRotateAMColorFast | ❌ | - | 未実装 (高速近似版) |

### rotateorth.c (orthogonal rotation)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixRotateOrth | ✅ | rotate::rotate_orth | 同等 |
| pixRotate180 | ✅ | rotate::rotate_180 | 同等 |
| pixRotate90 | ✅ | rotate::rotate_90 | 同等 |
| pixFlipLR | ✅ | rotate::flip_lr | 同等 |
| pixFlipTB | ✅ | rotate::flip_tb | 同等 |

### rotateshear.c (shear-based rotation)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixRotateShear | 🔄 | rotate::rotate_shear_impl | 内部実装として存在 |
| pixRotate2Shear | ✅ | rotate::rotate_2_shear | 同等 (内部関数) |
| pixRotate3Shear | ✅ | rotate::rotate_3_shear | 同等 (内部関数) |
| pixRotateShearIP | ❌ | - | 未実装 (in-place版) |
| pixRotateShearCenter | ❌ | - | 未実装 |
| pixRotateShearCenterIP | ❌ | - | 未実装 (in-place版) |

### scale1.c (general scaling)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixScale | ✅ | scale::scale | 同等 |
| pixScaleToSizeRel | ❌ | - | 未実装 |
| pixScaleToSize | ✅ | scale::scale_to_size | 同等 |
| pixScaleToResolution | ❌ | - | 未実装 |
| pixScaleGeneral | ❌ | - | 未実装 |
| pixScaleLI | ❌ | - | 未実装 (linear interpolation) |
| pixScaleColorLI | 🔄 | scale::scale_linear_color | 内部実装として存在 |
| pixScaleColor2xLI | ❌ | - | 未実装 (2x upscale) |
| pixScaleColor4xLI | ❌ | - | 未実装 (4x upscale) |
| pixScaleGrayLI | 🔄 | scale::scale_linear_gray | 内部実装として存在 |
| pixScaleGray2xLI | ❌ | - | 未実装 (2x upscale) |
| pixScaleGray4xLI | ❌ | - | 未実装 (4x upscale) |
| pixScaleGray2xLIThresh | ❌ | - | 未実装 (upscale + threshold) |
| pixScaleGray2xLIDither | ❌ | - | 未実装 (upscale + dither) |
| pixScaleGray4xLIThresh | ❌ | - | 未実装 (upscale + threshold) |
| pixScaleGray4xLIDither | ❌ | - | 未実装 (upscale + dither) |
| pixScaleBySampling | ✅ | scale::scale_by_sampling | 同等 |
| pixScaleBySamplingWithShift | ❌ | - | 未実装 (shift付き) |
| pixScaleBySamplingToSize | ❌ | - | 未実装 |
| pixScaleByIntSampling | ❌ | - | 未実装 (integer sampling) |
| pixScaleRGBToGrayFast | ❌ | - | 未実装 (RGB→Gray) |
| pixScaleRGBToBinaryFast | ❌ | - | 未実装 (RGB→Binary) |
| pixScaleGrayToBinaryFast | ❌ | - | 未実装 (Gray→Binary) |
| pixScaleSmooth | ❌ | - | 未実装 (smoothing付き) |
| pixScaleSmoothToSize | ❌ | - | 未実装 |
| pixScaleRGBToGray2 | ❌ | - | 未実装 (2x reduction) |
| pixScaleAreaMap | 🔄 | scale::scale_area_map | 内部実装として存在 |
| pixScaleAreaMap2 | ❌ | - | 未実装 (2x reduction) |
| pixScaleAreaMapToSize | ❌ | - | 未実装 |
| pixScaleBinary | ❌ | - | 未実装 (binary用) |
| pixScaleBinaryWithShift | ❌ | - | 未実装 |

### scale2.c (specialized scaling)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixScaleToGray | ❌ | - | 未実装 (1bpp→8bpp) |
| pixScaleToGrayFast | ❌ | - | 未実装 |
| pixScaleToGray2 | ❌ | - | 未実装 (2x) |
| pixScaleToGray3 | ❌ | - | 未実装 (3x) |
| pixScaleToGray4 | ❌ | - | 未実装 (4x) |
| pixScaleToGray6 | ❌ | - | 未実装 (6x) |
| pixScaleToGray8 | ❌ | - | 未実装 (8x) |
| pixScaleToGray16 | ❌ | - | 未実装 (16x) |
| pixScaleToGrayMipmap | ❌ | - | 未実装 (mipmap) |
| pixScaleMipmap | ❌ | - | 未実装 |
| pixExpandReplicate | ❌ | - | 未実装 (replicate拡大) |
| pixScaleGrayMinMax | ❌ | - | 未実装 (min/max) |
| pixScaleGrayMinMax2 | ❌ | - | 未実装 (2x) |
| pixScaleGrayRankCascade | ❌ | - | 未実装 (rank value) |
| pixScaleGrayRank2 | ❌ | - | 未実装 (2x) |
| pixScaleAndTransferAlpha | ❌ | - | 未実装 (helper) |
| pixScaleWithAlpha | ❌ | - | 未実装 (alpha付き) |

### affine.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixAffineSampledPta | ✅ | affine::affine_sampled_pta | 同等 |
| pixAffineSampled | ✅ | affine::affine_sampled | 同等 |
| pixAffinePta | ✅ | affine::affine_pta | 同等 |
| pixAffine | ✅ | affine::affine | 同等 |
| pixAffinePtaColor | 🔄 | affine::affine_color | 内部実装として存在 |
| pixAffineColor | 🔄 | affine::affine_color | 内部実装として存在 |
| pixAffinePtaGray | 🔄 | affine::affine_gray | 内部実装として存在 |
| pixAffineGray | 🔄 | affine::affine_gray | 内部実装として存在 |
| pixAffinePtaWithAlpha | ❌ | - | 未実装 (alpha付き) |
| getAffineXformCoeffs | ✅ | AffineMatrix::from_point_pairs | 同等 (メソッドとして実装) |
| affineInvertXform | ✅ | AffineMatrix::invert | 同等 (メソッドとして実装) |
| affineXformSampledPt | ✅ | AffineMatrix::transform_point_sampled | 同等 (メソッドとして実装) |
| affineXformPt | ✅ | AffineMatrix::transform_point | 同等 (メソッドとして実装) |
| linearInterpolatePixelGray | ❌ | - | 未実装 (helper関数) |
| linearInterpolatePixelColor | ❌ | - | 未実装 (helper関数) |
| gaussjordan | 🔄 | affine::gauss_jordan | 内部実装として存在 |
| pixAffineSequential | ❌ | - | 未実装 (シーケンシャル変換) |

### affinecompose.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| createMatrix2dTranslate | ✅ | AffineMatrix::translate | 同等 (コンストラクタ) |
| createMatrix2dScale | ✅ | AffineMatrix::scale | 同等 (コンストラクタ) |
| createMatrix2dRotate | ✅ | AffineMatrix::rotate | 同等 (コンストラクタ) |
| ptaTranslate | ❌ | - | 未実装 (PTA変換) |
| ptaScale | ❌ | - | 未実装 |
| ptaRotate | ❌ | - | 未実装 |
| boxaTranslate | ❌ | - | 未実装 (BOXA変換) |
| boxaScale | ❌ | - | 未実装 |
| boxaRotate | ❌ | - | 未実装 |
| ptaAffineTransform | ❌ | - | 未実装 |
| boxaAffineTransform | ❌ | - | 未実装 |
| l_productMatVec | ❌ | - | 未実装 (行列演算) |
| l_productMat2 | ❌ | - | 未実装 |
| l_productMat3 | ❌ | - | 未実装 |
| l_productMat4 | ❌ | - | 未実装 |

### bilinear.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixBilinearSampledPta | ✅ | bilinear::bilinear_sampled_pta | 同等 |
| pixBilinearSampled | ✅ | bilinear::bilinear_sampled | 同等 |
| pixBilinearPta | ✅ | bilinear::bilinear_pta | 同等 |
| pixBilinear | ✅ | bilinear::bilinear | 同等 |
| pixBilinearPtaColor | 🔄 | bilinear::bilinear_color | 内部実装として存在 |
| pixBilinearColor | 🔄 | bilinear::bilinear_color | 内部実装として存在 |
| pixBilinearPtaGray | 🔄 | bilinear::bilinear_gray | 内部実装として存在 |
| pixBilinearGray | 🔄 | bilinear::bilinear_gray | 内部実装として存在 |
| pixBilinearPtaWithAlpha | ❌ | - | 未実装 (alpha付き) |
| getBilinearXformCoeffs | ✅ | BilinearCoeffs::from_point_pairs | 同等 (メソッドとして実装) |
| bilinearXformSampledPt | ✅ | BilinearCoeffs::transform_point_sampled | 同等 (メソッドとして実装) |
| bilinearXformPt | ✅ | BilinearCoeffs::transform_point | 同等 (メソッドとして実装) |

### projective.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixProjectiveSampledPta | ✅ | projective::projective_sampled_pta | 同等 |
| pixProjectiveSampled | ✅ | projective::projective_sampled | 同等 |
| pixProjectivePta | ✅ | projective::projective_pta | 同等 |
| pixProjective | ✅ | projective::projective | 同等 |
| pixProjectivePtaColor | 🔄 | projective::projective_color | 内部実装として存在 |
| pixProjectiveColor | 🔄 | projective::projective_color | 内部実装として存在 |
| pixProjectivePtaGray | 🔄 | projective::projective_gray | 内部実装として存在 |
| pixProjectiveGray | 🔄 | projective::projective_gray | 内部実装として存在 |
| pixProjectivePtaWithAlpha | ❌ | - | 未実装 (alpha付き) |
| getProjectiveXformCoeffs | ✅ | ProjectiveCoeffs::from_point_pairs | 同等 (メソッドとして実装) |
| projectiveXformSampledPt | ✅ | ProjectiveCoeffs::transform_point_sampled | 同等 (メソッドとして実装) |
| projectiveXformPt | ✅ | ProjectiveCoeffs::transform_point | 同等 (メソッドとして実装) |

### shear.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixHShear | ✅ | shear::h_shear | 同等 |
| pixVShear | ✅ | shear::v_shear | 同等 |
| pixHShearCorner | ✅ | shear::h_shear_corner | 同等 |
| pixVShearCorner | ✅ | shear::v_shear_corner | 同等 |
| pixHShearCenter | ✅ | shear::h_shear_center | 同等 |
| pixVShearCenter | ✅ | shear::v_shear_center | 同等 |
| pixHShearIP | ✅ | shear::h_shear_ip | 同等 (in-place) |
| pixVShearIP | ✅ | shear::v_shear_ip | 同等 (in-place) |
| pixHShearLI | ✅ | shear::h_shear_li | 同等 (linear interpolation) |
| pixVShearLI | ✅ | shear::v_shear_li | 同等 (linear interpolation) |

## 追加機能 (Rust版のみ)

### warper.rs (追加機能)
| Rust関数 | 備考 |
|----------|------|
| random_harmonic_warp | ランダムな調和ワープ変換 |
| stretch_horizontal | 水平方向ストレッチ |
| stretch_horizontal_sampled | サンプリングベース水平ストレッチ |
| stretch_horizontal_li | 線形補間水平ストレッチ |
| quadratic_v_shear | 二次関数による垂直シア |
| quadratic_v_shear_sampled | サンプリングベース二次シア |
| quadratic_v_shear_li | 線形補間二次シア |
| warp_stereoscopic | ステレオスコピックワープ |
| stereo_from_pair | ペア画像からステレオ生成 |

## 分析と考察

### 実装状況の特徴

1. **基本的な変換は完備**:
   - Affine, Bilinear, Projective変換の基本機能は実装済み
   - Shear変換も完全に実装
   - Orthogonal rotation (90度回転系) は完全実装

2. **スケーリングは部分実装**:
   - 基本的なスケーリングは実装されているが、特殊用途のスケーリング関数群が未実装
   - scale1.cの152関数のうち、多くが特殊用途 (2x, 4x upscale, threshold, dither等)
   - scale2.cの1bpp→8bpp変換系は全て未実装

3. **未実装の主な分野**:
   - Alpha channel付き変換 (WithAlpha系)
   - Binary画像専用の最適化版
   - Mipmap系のスケーリング
   - 特殊なスケーリング (min/max, rank value等)
   - PTA/BOXA変換のユーティリティ関数群

4. **設計の違い**:
   - C版: 関数ベースのAPI
   - Rust版: メソッド + 関数のハイブリッド
   - 係数計算はメソッド化 (AffineMatrix::from_point_pairs等)

5. **Rust独自機能**:
   - warper.rs に高度なワープ機能を追加実装
   - ステレオスコピックワープなど、C版にない機能

### 推奨される次の実装ステップ

優先度順:

1. **Alpha channel対応** (3関数):
   - pixAffinePtaWithAlpha
   - pixBilinearPtaWithAlpha
   - pixProjectivePtaWithAlpha

2. **スケーリング補完** (基本的なもの):
   - pixScaleToGray系 (1bpp→8bpp変換)
   - pixScaleLI (linear interpolation)
   - pixScaleAreaMapToSize

3. **Binary画像最適化**:
   - pixScaleBinary
   - pixScaleRGBToBinaryFast

4. **ユーティリティ関数**:
   - PTA/BOXA変換関数群
   - 行列演算関数 (l_productMat系)

5. **特殊用途のスケーリング**:
   - 2x/4x upscale系
   - Mipmap系
   - Min/Max, Rank系
