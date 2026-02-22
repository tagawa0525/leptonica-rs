# leptonica-transform: C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（300_transform全移植計画完了を反映）

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 82 |
| 🔄 異なる | 9 |
| ❌ 未実装 | 61 |
| 合計 | 152 |

**カバレッジ**: 59.9% (91/152 functions have some implementation)

## 詳細

### rotate.c (general rotation)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixRotate | 🔄 | rotate::rotate | 異なるインタフェース設計 |
| pixEmbedForRotation | ❌ | - | 未実装 |
| pixRotateBySampling | 🔄 | rotate::rotate_by_sampling_impl | 内部実装として存在 |
| pixRotateBinaryNice | ❌ | - | 未実装 |
| pixRotateWithAlpha | ✅ | rotate::rotate_with_alpha | 同等 |

### rotateam.c (area mapping rotation)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixRotateAM | 🔄 | rotate::rotate_area_map_impl | 内部実装として存在 |
| pixRotateAMColor | 🔄 | rotate::rotate_area_map_color | 内部実装として存在 |
| pixRotateAMGray | 🔄 | rotate::rotate_area_map_gray | 内部実装として存在 |
| pixRotateAMCorner | ✅ | rotate::rotate_am_corner | 同等 |
| pixRotateAMColorCorner | ✅ | rotate::rotate_am_color_corner | 同等 |
| pixRotateAMGrayCorner | ✅ | rotate::rotate_am_gray_corner | 同等 |
| pixRotateAMColorFast | ❌ | - | 未実装 (高速近似版、スコープ除外) |

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
| pixRotateShear | ✅ | rotate::rotate_shear | 同等 |
| pixRotate2Shear | ✅ | rotate::rotate_2_shear | 同等 (内部関数) |
| pixRotate3Shear | ✅ | rotate::rotate_3_shear | 同等 (内部関数) |
| pixRotateShearIP | ✅ | rotate::rotate_shear_ip | 同等 (in-place版) |
| pixRotateShearCenter | ✅ | rotate::rotate_shear_center | 同等 |
| pixRotateShearCenterIP | ✅ | rotate::rotate_shear_center_ip | 同等 (in-place版) |

### scale1.c (general scaling)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixScale | ✅ | scale::scale | 同等 |
| pixScaleToSizeRel | ❌ | - | 未実装 |
| pixScaleToSize | ✅ | scale::scale_to_size | 同等 |
| pixScaleToResolution | ✅ | scale::scale_to_resolution | 同等 |
| pixScaleGeneral | ✅ | scale::scale_general | 同等 |
| pixScaleLI | ✅ | scale::scale_li | 同等 |
| pixScaleColorLI | ✅ | scale::scale_color_li | 同等 |
| pixScaleColor2xLI | ✅ | scale::scale_color_2x_li | 同等 |
| pixScaleColor4xLI | ✅ | scale::scale_color_4x_li | 同等 |
| pixScaleGrayLI | ✅ | scale::scale_gray_li | 同等 |
| pixScaleGray2xLI | ✅ | scale::scale_gray_2x_li | 同等 |
| pixScaleGray4xLI | ✅ | scale::scale_gray_4x_li | 同等 |
| pixScaleGray2xLIThresh | ✅ | scale::scale_gray_2x_li_thresh | 同等 |
| pixScaleGray2xLIDither | ✅ | scale::scale_gray_2x_li_dither | 同等 |
| pixScaleGray4xLIThresh | ✅ | scale::scale_gray_4x_li_thresh | 同等 |
| pixScaleGray4xLIDither | ✅ | scale::scale_gray_4x_li_dither | 同等 |
| pixScaleBySampling | ✅ | scale::scale_by_sampling | 同等 |
| pixScaleBySamplingWithShift | ✅ | scale::scale_by_sampling_with_shift | 同等 |
| pixScaleBySamplingToSize | ❌ | - | 未実装 |
| pixScaleByIntSampling | ✅ | scale::scale_by_int_sampling | 同等 |
| pixScaleRGBToGrayFast | ❌ | - | 未実装 (スコープ除外) |
| pixScaleRGBToBinaryFast | ❌ | - | 未実装 (スコープ除外) |
| pixScaleGrayToBinaryFast | ❌ | - | 未実装 (スコープ除外) |
| pixScaleSmooth | ✅ | scale::scale_smooth | 同等 |
| pixScaleSmoothToSize | ❌ | - | 未実装 |
| pixScaleRGBToGray2 | ❌ | - | 未実装 (スコープ除外) |
| pixScaleAreaMap | 🔄 | scale::scale_area_map | 内部実装として存在 |
| pixScaleAreaMap2 | ❌ | - | 未実装 |
| pixScaleAreaMapToSize | ❌ | - | 未実装 |
| pixScaleBinary | ✅ | scale::scale_binary | 同等 |
| pixScaleBinaryWithShift | ❌ | - | 未実装 |

### scale2.c (specialized scaling)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixScaleToGray | ✅ | scale::scale_to_gray | 同等 |
| pixScaleToGrayFast | ✅ | scale::scale_to_gray_fast | 同等 |
| pixScaleToGray2 | ✅ | scale::scale_to_gray_2 | 同等 |
| pixScaleToGray3 | ✅ | scale::scale_to_gray_3 | 同等 |
| pixScaleToGray4 | ✅ | scale::scale_to_gray_4 | 同等 |
| pixScaleToGray6 | ✅ | scale::scale_to_gray_6 | 同等 |
| pixScaleToGray8 | ✅ | scale::scale_to_gray_8 | 同等 |
| pixScaleToGray16 | ✅ | scale::scale_to_gray_16 | 同等 |
| pixScaleToGrayMipmap | ✅ | scale::scale_to_gray_mipmap | 同等 |
| pixScaleMipmap | ❌ | - | 未実装 (内部ヘルパー) |
| pixExpandReplicate | ✅ | scale::expand_replicate | 同等 |
| pixScaleGrayMinMax | ✅ | scale::scale_gray_min_max | 同等 |
| pixScaleGrayMinMax2 | ❌ | - | 未実装 (2x特殊版) |
| pixScaleGrayRankCascade | ✅ | scale::scale_gray_rank_cascade | 同等 |
| pixScaleGrayRank2 | ❌ | - | 未実装 (2x特殊版) |
| pixScaleAndTransferAlpha | ❌ | - | 未実装 (内部ヘルパー) |
| pixScaleWithAlpha | ❌ | - | 未実装 |

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
| pixAffinePtaWithAlpha | ✅ | affine::affine_pta_with_alpha | 同等 |
| getAffineXformCoeffs | ✅ | AffineMatrix::from_point_pairs | 同等 (メソッドとして実装) |
| affineInvertXform | ✅ | AffineMatrix::invert | 同等 (メソッドとして実装) |
| affineXformSampledPt | ✅ | AffineMatrix::transform_point_sampled | 同等 (メソッドとして実装) |
| affineXformPt | ✅ | AffineMatrix::transform_point | 同等 (メソッドとして実装) |
| linearInterpolatePixelGray | ❌ | - | 未実装 (内部ヘルパー) |
| linearInterpolatePixelColor | ❌ | - | 未実装 (内部ヘルパー) |
| gaussjordan | 🔄 | affine::gauss_jordan | 内部実装として存在 |
| pixAffineSequential | ❌ | - | 未実装 (スコープ除外: AffineMatrix::compose で対応) |

### affinecompose.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| createMatrix2dTranslate | ✅ | AffineMatrix::translate | 同等 (コンストラクタ) |
| createMatrix2dScale | ✅ | AffineMatrix::scale | 同等 (コンストラクタ) |
| createMatrix2dRotate | ✅ | AffineMatrix::rotate | 同等 (コンストラクタ) |
| ptaTranslate | ❌ | - | 未実装 (Pta::translate は in-place) |
| ptaScale | ❌ | - | 未実装 (Pta::scale は in-place) |
| ptaRotate | ✅ | Pta::rotate_around | 同等 (rotated_about に委譲) |
| boxaTranslate | ❌ | - | 未実装 |
| boxaScale | ❌ | - | 未実装 |
| boxaRotate | ❌ | - | 未実装 |
| ptaAffineTransform | ✅ | Pta::affine_transform | 同等 |
| boxaAffineTransform | ✅ | Boxa::affine_transform | 同等 |
| l_productMatVec | ❌ | - | 未実装 (スコープ除外) |
| l_productMat2 | ❌ | - | 未実装 (スコープ除外) |
| l_productMat3 | ❌ | - | 未実装 (スコープ除外) |
| l_productMat4 | ❌ | - | 未実装 (スコープ除外) |

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
| pixBilinearPtaWithAlpha | ✅ | bilinear::bilinear_pta_with_alpha | 同等 |
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
| pixProjectivePtaWithAlpha | ✅ | projective::projective_pta_with_alpha | 同等 |
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
| pixHShearIP | ✅ | shear::h_shear_ip | 同等 |
| pixVShearIP | ✅ | shear::v_shear_ip | 同等 |
| pixHShearLI | ✅ | shear::h_shear_li | 同等 |
| pixVShearLI | ✅ | shear::v_shear_li | 同等 |

### flipdetect.c (leptonica-recog に実装)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixOrientDetect | ✅ | recog::flipdetect::orient_detect | leptonica-recog crateに実装 |
| pixOrientCorrect | ✅ | recog::flipdetect::orient_correct | leptonica-recog crateに実装 |
| pixMirrorDetect | ✅ | recog::flipdetect::mirror_detect | leptonica-recog crateに実装 |

*注: flipdetect.c の3関数はleptonica-recog crateで実装済み。上記152関数カウントには含まれない。*

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

1. **変換・スケーリングは高カバレッジ**:
   - Affine, Bilinear, Projective変換: WithAlpha含め実装済み
   - Shear変換: 完全実装
   - Scale: LI, ToGray系, 2x/4x, MinMax/Rank など大部分実装済み

2. **残存する未実装**:
   - PTA/BOXAのtranslate/scaleユーティリティ（in-place版はあるが新Pta/Boxa返却版なし）
   - pixScaleToSizeRel, pixScaleBySamplingToSize 等の特殊バリアント
   - pixScaleMipmap (内部ヘルパー)、ScaleGrayMinMax2/GrayRank2 (2x特殊版)
   - l_productMat系（スコープ除外）、RGBToGrayFast系（スコープ除外）

3. **設計の違い**:
   - C版: 関数ベースのAPI（Gray/Color別）
   - Rust版: 統一APIで深度自動判定
   - 係数計算はメソッド化（AffineMatrix::from_point_pairs等）
   - flipdetect機能はleptonica-recog crateに配置
