# C版回帰テスト全移植計画

Status: IN_PROGRESS

## Context

C版leptonicaには160個の回帰テスト（`reference/leptonica/prog/*_reg.c`）があり、現在59個（37.1%）が移植済み。残り100個（+ alltests_reg.c ランナー）を移植し、テストカバレッジを向上させる。

目標: 移植可能なテストをすべて移植し、除外するものは理由を明記する。

## 除外テスト一覧（17個）

以下のテストはRustの言語特性・設計上、移植が不要または不可能:

| C版テスト | 理由 |
|-----------|------|
| alltests | テストランナー。`cargo test`が代替 |
| bytea | L_Bytea型。Rustでは`Vec<u8>`が標準。専用型不要 |
| dna | L_Dna/L_Dnaa型。Rustでは`Vec<f64>`/`Numa`が代替 |
| hash | L_ASet/L_AMap/L_HMap。Rustでは`std::collections::{BTreeSet,HashMap}`が代替 |
| heap | L_Heap型。Rustでは`std::collections::BinaryHeap`が代替 |
| ptra1 | L_Ptra型（ポインタ配列）。Rustでは`Vec<T>`が完全に代替 |
| ptra2 | 同上 |
| pixalloc | カスタムメモリアロケータ（pms）。Rustのメモリモデルで不要 |
| pixmem | pixTransferAllData等の低レベルメモリ操作。Rustの所有権モデルで不要 |
| string | Sarray文字列操作。Rustでは`Vec<String>`と標準文字列APIが代替 |
| files | ファイルシステムユーティリティ（lept_mkdir等）。Rustでは`std::fs`が代替 |
| genfonts | ビットマップフォント生成（BMF）。Rust版未実装で優先度低 |
| writetext | BMF依存のテキスト描画。genfontsと同様 |
| webpanimio | アニメーションWebP。leptonica-io未対応フォーマット |
| falsecolor | pixFalseColor関数未実装 |
| findcorners | pixFindCornerPixels未実装 |
| nearline | pixGetNearLine未実装 |

## 移植対象テスト（83個）

### Phase 1: leptonica-core 基本データ構造（15個・3PR）

Rust APIが完全に存在し、そのままテストを書ける。

**PR 1-1: Box/Boxa 拡張テスト（5個）**
- `boxa3` → boxa3_reg.rs: Boxa::median_dimensions, size_consistency, reconcile_by_median
- `boxa4` → boxa4_reg.rs: Boxa::smooth_sequence_median, reconcile_all_by_median, split_even_odd
- `rectangle` → rectangle_reg.rs: Box::new, adjust_sides, get_geometry, equals
- `insert` → insert_reg.rs: Numa::insert/remove, Boxa::remove_and_save
- `smallpix` → smallpix_reg.rs: Pix::scale_to_size (smart_subsample未実装なら部分移植)

**PR 1-2: ピクセル演算・比較テスト（5個）**
- `logicops` → logicops_reg.rs: Pix::and, or, xor, subtract, invert
- `rasterop` → rasterop_reg.rs: Pix::rasterop_vip, rasterop_hip, translate
- `rasteropip` → rasteropip_reg.rs: PixMut版のrasterop操作
- `compare` → compare_reg.rs: Pix::count_pixels, equals, correlation_binary（部分移植: best_correlation/perceptual_diff未実装なら省略）
- `equal` → equal_reg.rs: Pix::remove_colormap, convert, write/read round-trip比較

**PR 1-3: 変換・統計テスト（5個）**
- `conversion` → conversion_reg.rs: Pix::convert_to_8, threshold, remove_colormap
- `expand` → expand_reg.rs: expand_replicate, convert_1_to_2/4/8（expand_binary_power2未実装なら部分移植）
- `extrema` → extrema_reg.rs: Numa::find_extrema
- `lowaccess` → lowaccess_reg.rs: Pix::get_pixel, set_pixel, width, height, depth, wpl
- `multitype` → multitype_reg.rs: Pix::render_line, convert_to_8, scale_to_size

### Phase 2: leptonica-core 残り（5個・1PR）

**PR 2-1: シリアライズ・その他（5個）**
- `encoding` → encoding_reg.rs: Base85エンコード（部分移植: Rust標準エンコーディング機能でテスト）
- `numa3` → numa3_reg.rs: Numa高度操作（histogram, interpolation, sort等）
- `pixcomp` → pixcomp_reg.rs: PixComp圧縮Pix操作（部分移植: 利用可能API範囲で）
- `pixserial` → pixserial_reg.rs: Spix serialize/deserialize round-trip
- `pixa2` は移植済み。追加候補なし

### Phase 3: leptonica-io（7個・2PR）

**PR 3-1: フォーマットI/O テスト（4個）**
- `jp2kio` → jp2kio_reg.rs: JP2K read/write round-trip
- `pdfio1` → pdfio1_reg.rs: PDF生成・変換
- `pdfio2` → pdfio2_reg.rs: PDF高度操作
- `psio` → psio_reg.rs: PostScript出力

**PR 3-2: I/O 応用テスト（3個）**
- `pdfseg` → pdfseg_reg.rs: PDF segmented出力
- `psioseg` → psioseg_reg.rs: PS segmented出力
- `pixtile` → pixtile_reg.rs: タイリングI/O（部分移植: tiling APIがあれば）

### Phase 4: leptonica-transform（11個・2PR）

**PR 4-1: シア・変換テスト（6個）**
- `shear1` → shear1_reg.rs: h_shear, v_shear, h_shear_corner, v_shear_corner
- `shear2` → shear2_reg.rs: quadratic_v_shear
- `translate` → translate_reg.rs: translate（affine.rs:942）
- `crop` → crop_reg.rs: average_intensity_profile, clip_rectangle（部分移植）
- `projection` → projection_reg.rs: Pix::column_stats, row_stats
- `xformbox` → xformbox_reg.rs: Boxa affine/projective transform

**PR 4-2: 特殊変換テスト（5個）**
- `warper` → warper_reg.rs: random_harmonic_warp, warp_stereoscopic
- `alphaxform` → alphaxform_reg.rs: alpha付き変換（部分移植: blend APIを活用）
- `checkerboard` → checkerboard_reg.rs: （部分移植: 利用可能API範囲で）
- `circle` → circle_reg.rs: 円関連操作（部分移植: seedfill, morph等を組み合わせ）
- `subpixel` → subpixel_reg.rs: サブピクセルレンダリング（部分移植: 未実装API多い可能性）

### Phase 5: leptonica-filter（6個・1PR）

**PR 5-1: フィルタ拡張テスト（6個）**
- `enhance` → enhance_reg.rs: gamma_trc_pix, modify_hue, modify_saturation, measure_saturation, unsharp_masking
- `kernel` → kernel_reg.rs: Kernel::new, from_slice, box_kernel, gaussian, sobel, convolve
- `compfilter` → compfilter_reg.rs: Pixa::select_by_size, Pix::count_connected_components
- `locminmax` → locminmax_reg.rs: local_extrema, paint_through_mask
- `rankbin` → rankbin_reg.rs: Numa rank bin操作（部分移植）
- `rankhisto` → rankhisto_reg.rs: rank_filter, median_filter

### Phase 6: leptonica-color（13個・3PR）

**PR 6-1: ブレンド・アルファテスト（5個）**
- `alphaops` → alphaops_reg.rs: alpha_blend_uniform, remove_alpha, colorize_gray
- `hardlight` → hardlight_reg.rs: blend_hard_light
- `blend1` → blend1_reg.rs: blend_gray, blend_color
- `blend2` → blend2_reg.rs: blend_gray_adapt, blend_gray_inverse
- `blend3` → blend3_reg.rs: blend_with_gray_mask

**PR 6-2: 色操作テスト（5個）**
- `coloring` → coloring_reg.rs: shift_by_component（colormap経由）
- `colorize` → colorize_reg.rs: colorize_gray
- `colormask` → colormask_reg.rs: mask_by_color（部分移植）
- `blackwhite` → blackwhite_reg.rs: add_border_general, alpha_blend_uniform
- `lowsat` → lowsat_reg.rs: modify_saturation, measure_saturation

**PR 6-3: 量子化・二値化・ペイントテスト（5個）**
- `dither` → dither_reg.rs: dither_to_binary, ordered_dither
- `grayquant` → grayquant_reg.rs: median_cut_quant, octree_quant
- `threshnorm` → threshnorm_reg.rs: background_norm, threshold操作
- `paint` → paint_reg.rs: paint_through_mask, render_line, render_box（部分移植）
- `paintmask` → paintmask_reg.rs: paint_through_mask（部分移植）

**除外（color内）:**
- `falsecolor`: false_color関数未実装

### Phase 7: leptonica-region（8個・2PR）

**PR 7-1: シードフィル・距離テスト（4個）**
- `distance` → distance_reg.rs: distance_function
- `grayfill` → grayfill_reg.rs: seedfill_gray, seedfill_gray_inv
- `maze` → maze_reg.rs: generate_binary_maze, search_binary_maze, search_gray_maze
- `speckle` → speckle_reg.rs: clear_border, count_components, select_by_size

**PR 7-2: 領域解析テスト（4個）**
- `overlap` → overlap_reg.rs: Box::overlaps, overlap_area, overlap_fraction
- `splitcomp` → splitcomp_reg.rs: （部分移植: 利用可能API範囲で）
- `smoothedge` → smoothedge_reg.rs: morph_sequence による平滑化（部分移植）
- `texturefill` → texturefill_reg.rs: （部分移植: texture_fill未実装の場合edge+morph組み合わせ）

### Phase 8: leptonica-morph（6個・1PR）

**PR 8-1: 形態学拡張テスト（6個）**
- `binmorph6` → binmorph6_reg.rs: カスタムSELによるdilate, open, close_safe, subtract
- `ccthin1` → ccthin1_reg.rs: make_thin_sels, thin_connected_by_set のSEL表示・検証
- `ccthin2` → ccthin2_reg.rs: thin_connected, thin_connected_by_set 実行結果検証
- `graymorph2` → graymorph2_reg.rs: gray_morph_sequence 高度テスト
- `fhmtauto` → fhmtauto_reg.rs: hit_miss_transform（部分移植: auto-gen部分は除外）
- `fmorphauto` → fmorphauto_reg.rs: morph_sequence（部分移植: auto-gen部分は除外）

### Phase 9: leptonica-recog（11個・2PR）

**PR 9-1: 検出・補正テスト（5個）**
- `flipdetect` → flipdetect_reg.rs: orient_detect, orient_correct
- `dewarp` → dewarp_reg.rs: dewarp_single_page, find_textline_centers
- `jbclass` → jbclass_reg.rs: rank_haus_init, JbClasser分類
- `wordboxes` → wordboxes_reg.rs: （部分移植: pageseg API活用）
- `pixadisp` → pixadisp_reg.rs: Pixa::display_tiled, display_tiled_and_scaled

**PR 9-2: 応用テスト（6個）**
- `lineremoval` → lineremoval_reg.rs: morph_sequence + seedfill組み合わせ（部分移植）
- `italic` → italic_reg.rs: （部分移植: skew系API活用）
- `findpattern1` → findpattern1_reg.rs: （部分移植: correlation系API活用）
- `findpattern2` → findpattern2_reg.rs: 同上
- `partition` → partition_reg.rs: （部分移植: Boxa操作でwhiteblock検出）
- `newspaper` → newspaper_reg.rs: （部分移植: pageseg + morph組み合わせ）

**除外（recog内）:**
- `findcorners`: pixFindCornerPixels未実装
- `nearline`: pixGetNearLine未実装

### Phase 10: blend残り + ドキュメント更新（2個・1PR）

**PR 10-1: ブレンド残り + docs更新**
- `blend4` → blend4_reg.rs: alpha_blend_uniform高度テスト
- `blend5` → blend5_reg.rs: blend_with_gray_mask高度テスト
- `docs/porting/test-comparison.md` 更新

## 除外テスト 最終まとめ（17個）

| テスト | カテゴリ | 除外理由 |
|--------|----------|----------|
| alltests | runner | `cargo test`が代替 |
| bytea | core | `Vec<u8>`が代替 |
| dna | core | `Vec<f64>`/`Numa`が代替 |
| hash | core | `std::collections`が代替 |
| heap | core | `BinaryHeap`が代替 |
| ptra1 | core | `Vec<T>`が代替 |
| ptra2 | core | `Vec<T>`が代替 |
| pixalloc | core | Rustメモリモデルで不要 |
| pixmem | core | Rust所有権モデルで不要 |
| string | core | `Vec<String>`+標準APIが代替 |
| files | io | `std::fs`が代替 |
| genfonts | recog | BMF未実装・優先度低 |
| writetext | io | BMF依存・未実装 |
| webpanimio | io | アニメーションWebP未対応 |
| falsecolor | color | false_color関数未実装 |
| findcorners | recog | pixFindCornerPixels未実装 |
| nearline | recog | pixGetNearLine未実装 |

**移植: 83個 / 部分移植含む / 除外: 17個**
（完全移植 + 部分移植で全159テスト中142個カバー = 89.3%）

## PR構成まとめ

| PR | crate | テスト数 | 内容 |
|----|-------|----------|------|
| 1-1 | core | 5 | Box/Boxa拡張 |
| 1-2 | core | 5 | ピクセル演算・比較 |
| 1-3 | core | 5 | 変換・統計 |
| 2-1 | core | 5 | シリアライズ・その他 |
| 3-1 | io | 4 | フォーマットI/O |
| 3-2 | io | 3 | I/O応用 |
| 4-1 | transform | 6 | シア・変換 |
| 4-2 | transform | 5 | 特殊変換 |
| 5-1 | filter | 6 | フィルタ拡張 |
| 6-1 | color | 5 | ブレンド・アルファ |
| 6-2 | color | 5 | 色操作 |
| 6-3 | color | 5 | 量子化・二値化・ペイント |
| 7-1 | region | 4 | シードフィル・距離 |
| 7-2 | region | 4 | 領域解析 |
| 8-1 | morph | 6 | 形態学拡張 |
| 9-1 | recog | 5 | 検出・補正 |
| 9-2 | recog | 6 | 応用 |
| 10-1 | color+docs | 2+docs | ブレンド残り+ドキュメント |
| **合計** | | **83** | **18 PR** |

## 各PRの進め方

CLAUDE.mdのTDD/PRワークフローに従う:

1. 計画書コミット（`docs: add plan for <phase>` — 本計画書で代替可能）
2. RED: テスト作成（`#[ignore = "not yet implemented"]`付き）
3. GREEN: ignore除去（実装は既存のため基本的にignore除去のみ）
4. REFACTOR: 必要に応じて
5. PR作成 → Copilotレビュー → 修正 → マージ

## 検証方法

```bash
# 全テスト実行
cargo test --all-features

# 特定crateのテスト
cargo test --package leptonica-core
cargo test --package leptonica-morph

# 特定テスト
cargo test graymorph2_reg --package leptonica-morph

# lint & format
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check

# golden生成（新テスト追加時）
REGTEST_MODE=generate cargo test <test_name> --package <crate>
```

## 重要ファイル

- テストインフラ: `tests/common/src/lib.rs` (RegParams, load_test_image)
- テストデータ: `tests/data/images/`
- goldenファイル: `tests/golden/`
- テスト出力: `tests/regout/`
- C版参照: `reference/leptonica/prog/*_reg.c`
- 進捗管理: `docs/porting/test-comparison.md`
