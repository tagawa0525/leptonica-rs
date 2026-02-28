# 012: 無視テストの復活

## Status: IMPLEMENTED

## 概要

`#[ignore]`付きテストのうち、依存関数・テスト画像がすべて揃っているものを復活させる。
全118件の無視テストを精査し、復活可能な約30件を実装する。

## 復活対象テスト

### core (5件)
| テストファイル | テスト関数 | 理由 | 備考 |
|---|---|---|---|
| equal_reg.rs:40 | equal_reg_8bpp_colormap | equals_with_cmap, remove_colormap 実装済み | コードあり、#[ignore]除去のみ |
| equal_reg.rs:122 | equal_reg_2bpp_colormap | octree_quant_num_colors, convert_rgb_to_colormap 実装済み | スタブ→実装 |
| equal_reg.rs:136 | equal_reg_4bpp_colormap | 同上 | スタブ→実装 |
| insert_reg.rs:80 | insert_reg_pixa | conncomp, pixa insert/remove 実装済み | スタブ→実装 |
| overlap_reg.rs:177 | splitcomp_reg_split_into_boxa | split_into_boxa 実装済み | スタブ→実装 |

### filter (14件)
| テストファイル | テスト関数 | 理由 |
|---|---|---|
| adaptnorm_reg.rs:414 | adaptnorm_reg_gamma_trc | gamma_trc 実装済み |
| adaptnorm_reg.rs:419 | adaptnorm_reg_quantization | dither_to_2bpp, threshold_to_4bpp 実装済み |
| adaptnorm_reg.rs:424 | adaptnorm_reg_binarization | threshold_to_binary 実装済み |
| adaptnorm_reg.rs:429 | adaptnorm_reg_local_extrema_pipeline | local_extrema, seedfill_gray_basin 実装済み |
| adaptnorm_reg.rs:434 | adaptnorm_reg_gamma_trc_masked | gamma_trc_masked 実装済み |
| adaptmap_reg.rs:312 | adaptmap_reg_gamma_trc_masked | gamma_trc_masked 実装済み |
| bilateral1_reg.rs:125 | bilateral1_reg_separable | bilateral 実装済み |
| bilateral2_reg.rs:154 | bilateral2_reg_full_sweep_test24 | bilateral 実装済み |
| compfilter_reg.rs:20 | compfilter_reg_select_by_size | select_by_size, conncomp 実装済み |
| compfilter_reg.rs:33 | compfilter_reg_select_by_shape | shape metric functions 実装済み |
| locminmax_reg.rs:45 | locminmax_reg_extrema | local_extrema 実装済み |
| lowsat_reg.rs:84 | lowsat_reg_mask_gray | mask_over_gray_pixels 実装済み |
| rank_reg.rs:73 | rank_reg_gray_morph_comparison | morph dilate/erode 実装済み |
| rank_reg.rs:133 | rank_reg_color_morph_comparison | morph 実装済み |

### color (8件)
| テストファイル | テスト関数 | 理由 |
|---|---|---|
| cmapquant_reg.rs:320 | cmapquant_threshold_to_4bpp | threshold_to_4bpp 実装済み |
| cmapquant_reg.rs:326 | cmapquant_octcube_from_cmap | octcube_quant_from_cmap 実装済み |
| cmapquant_reg.rs:332 | cmapquant_few_colors_mixed | few_colors_median_cut_quant_mixed 実装済み |
| cmapquant_reg.rs:338 | cmapquant_octcube_mixed_gray | octcube_quant_mixed_with_gray 実装済み |
| cmapquant_reg.rs:344 | cmapquant_remove_unused_colors | remove_unused_colors 実装済み |
| dither_reg.rs:81 | dither_reg_2bpp_and_scaled | dither_to_2bpp, scale_gray_2x_li_dither 実装済み |
| paintmask_reg.rs:91 | paintmask_reg_clip_masked | clip_masked 実装済み |
| colorize_reg.rs:92 | colorize_reg_highlight_detect | highlight_red, color_gray_regions 実装済み |

### io (3件)
| テストファイル | テスト関数 | 理由 |
|---|---|---|
| iomisc_reg.rs:126 | iomisc_reg_alpha_blend_operations | alpha_blend_uniform, set_alpha_over_white, get_rgb_component 実装済み |
| iomisc_reg.rs:177 | iomisc_reg_remove_regen_rgb_colormap | remove_colormap, convert_rgb_to_colormap 実装済み |
| iomisc_reg.rs:183 | iomisc_reg_remove_regen_gray_colormap | remove_colormap, convert_gray_to_colormap 実装済み |

### transform (3件)
| テストファイル | テスト関数 | 理由 |
|---|---|---|
| xformbox_reg.rs:133 | xformbox_reg_ordered | transform_ordered 実装済み |
| xformbox_reg.rs:148 | xformbox_reg_hash_rendering | render_hash_box, conncomp 実装済み |
| projection_reg.rs:138 | projection_reg_visualization | gplot_simple_pix_1, column/row_stats 実装済み |

## 復活対象外の理由

- テスト画像不足: checkerboard1/2.tif, stampede2.jpg, blend-green*.jpg, greencover.jpg, redcover.jpg 等
- 関数未実装: best_correlation, compare_with_translation, remove_with_indicator, affine_sequential, shift_by_component, auto_photoinvert, decide_if_table, find_large_rectangles
- OOMリスク: ccbord_reg_feyn_fract
- 低速テスト: dwamorph2_reg_full (~35s)
- 設計差異: selio_reg origin検証 (Rust版は別引数)
- JP2K: Rust版でサポートなし
- 外部ツール依存: PS/PDF segmented出力

### 画像追加で新たに復活可能 (7件)
| テストファイル | テスト関数 | 追加画像 |
|---|---|---|
| blackwhite_reg.rs:97 | blackwhite_reg_full_image_set | speckle2.png, speckle4.png, test-cmap-alpha.png |
| hardlight_reg.rs:83 | hardlight_reg_original_images | hardlight1_1.jpg, hardlight1_2.jpg, hardlight2_1.jpg, hardlight2_2.jpg |
| compare_reg.rs:118 | compare_reg_perceptual_diff | greencover.jpg, redcover.jpg |
| threshnorm_reg.rs:51 | threshnorm_reg_spread_norm | stampede2.jpg |
| checkerboard_reg.rs:20 | checkerboard_reg_find_corners | checkerboard1.tif, checkerboard2.tif |
| alphaops_reg.rs:125 | alphaops_reg_set_alpha_over_white | blend-green1.jpg 等 |
| grayquant_reg.rs:124 | grayquant_reg_advanced_threshold | stampede2.jpg (threshold_on_8bpp等も実装済み) |

## 合計: 40件の復活対象
