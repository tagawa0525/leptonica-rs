# 019: color モジュール回帰テスト golden manifest 強化

Status: IMPLEMENTED

Phase 3 PR 5/8: `tests/color/` 配下の回帰テストに `write_pix_and_check()` を追加し、
golden manifest によるピクセルレベル回帰検出を有効化する。

## Context

Phase 3（回帰テスト強化）の第5弾。color モジュールには28テストファイルがあり、
そのうち21ファイルが `RegParams` を使用するが、`write_pix_and_check` は0件。
現在の manifest（260エントリ）に color エントリはゼロ。

親計画: `docs/plans/014_regression-test-enhance.md`

## 対象ファイルと変更内容

### Commit 1: blend1-3 (10 golden)

| ファイル        | テスト関数         | 変数      | Format | 備考  |
| --------------- | ------------------ | --------- | ------ | ----- |
| `blend1_reg.rs` | `gray_straight`    | `result`  | Png    | 32bpp |
|                 | `gray_inverse`     | `result`  | Png    | 32bpp |
|                 | `adapt`            | `result`  | Png    | 8bpp  |
|                 | `color`            | `result`  | Png    | 32bpp |
| `blend2_reg.rs` | `rgb`              | `blended` | Png    | 32bpp |
|                 | `gray`             | `blended` | Png    | 8bpp  |
|                 | `negative_offset`  | `blended` | Png    | 32bpp |
| `blend3_reg.rs` | `gray_inverse`     | `blended` | Png    | 8bpp  |
|                 | `color_by_channel` | `blended` | Png    | 32bpp |
|                 | `gray_base`        | `blended` | Png    | 8bpp  |

### Commit 2: blend4-5 (8 golden)

| ファイル        | テスト関数        | 変数             | Format | 備考              |
| --------------- | ----------------- | ---------------- | ------ | ----------------- |
| `blend4_reg.rs` | `add_alpha`       | `with_alpha`     | Png    | 32bpp             |
|                 | `alpha_composite` | `composited`     | Png    | 32bpp             |
|                 | `gray_mask_blend` | `blended`        | Png    | 32bpp             |
|                 | `mask_offset`     | `blended`        | Png    | 32bpp             |
| `blend5_reg.rs` | `snap_color_rgb`  | `snapped`        | Png    | 32bpp             |
|                 | `snap_color_gray` | `snapped`        | Png    | 8bpp              |
|                 | `edge_fade_rgb`   | PixMut `.into()` | Png    | fade_left のみ    |
|                 | `edge_fade_rgb`   | PixMut `.into()` | Png    | fade_top のみ     |

### Commit 3: alphaops + hardlight (8 golden)

| ファイル           | テスト関数             | 変数         | Format | 備考                             |
| ------------------ | ---------------------- | ------------ | ------ | -------------------------------- |
| `alphaops_reg.rs`  | `blend_uniform`        | `blended`    | Png    | 32bpp                            |
|                    | `remove_add_alpha`     | `readded`    | Png    | 32bpp                            |
|                    | `multiply_by_color`    | `multiplied` | Png    | 32bpp                            |
|                    | `blend_with_mask`      | `blended`    | Png    | 32bpp                            |
|                    | `set_alpha_over_white` | `over_white` | Png    | 32bpp                            |
| `hardlight_reg.rs` | `color`                | `result`     | Png    | 32bpp                            |
|                    | `gray`                 | `result`     | Png    | 8bpp                             |
|                    | `original_images`      | `result`     | Png    | 最初のペアのみ (guard: `i == 0`) |

### Commit 4: colorize + coloring + paint (9 golden)

| ファイル          | テスト関数          | 変数            | Format | 備考              |
| ----------------- | ------------------- | --------------- | ------ | ----------------- |
| `colorize_reg.rs` | `color_gray`        | `colored`       | Png    | 32bpp             |
|                   | `color_gray_masked` | `colored`       | Png    | 32bpp             |
|                   | `highlight_detect`  | `regions`       | Png    | 32bpp             |
| `coloring_reg.rs` | `background_shift`  | `result`        | Png    | Pix (変換不要)    |
|                   | `foreground_shift`  | `result`        | Png    | Pix (変換不要)    |
| `paint_reg.rs`    | `color_gray`        | `result`        | Png    | Pix (変換不要)    |
|                   | `through_mask`      | `pixmut.into()` | Png    | PixMut→Pix 変換要 |
|                   | `render_color`      | `pixmut.into()` | Png    | PixMut→Pix 変換要 |
|                   | `render_blend`      | `pixmut.into()` | Png    | PixMut→Pix 変換要 |

### Commit 5: paintmask + blackwhite (6 golden)

| ファイル            | テスト関数     | 変数                | Format | 備考                   |
| ------------------- | -------------- | ------------------- | ------ | ---------------------- |
| `paintmask_reg.rs`  | `paint_32bpp`  | `painted` (Pix化後) | Png    | 32bpp                  |
|                     | `quant_clip`   | `clipped`           | Png    | 8bpp                   |
|                     | `clip_masked`  | `clipped`           | Png    | 8bpp                   |
| `blackwhite_reg.rs` | `white_border` | `bordered`          | Png    | ループ初回のみ (guard) |
|                     | `black_border` | `bordered`          | Png    | ループ初回のみ (guard) |
|                     | `alpha_blend`  | `blended`           | Png    | 32bpp                  |

### Commit 6: binarize + dither + threshnorm (8 golden)

| ファイル            | テスト関数        | 変数         | Format | 備考                         |
| ------------------- | ----------------- | ------------ | ------ | ---------------------------- |
| `binarize_reg.rs`   | `binarize_reg`    | `bin128`     | Tiff   | 1bpp                         |
|                     |                   | `otsu_bin`   | Tiff   | 1bpp                         |
|                     |                   | `sauvola`    | Tiff   | 1bpp                         |
| `dither_reg.rs`     | `to_binary`       | `dithered`   | Tiff   | 1bpp                         |
|                     | `ordered`         | `dithered`   | Tiff   | 1bpp                         |
|                     | `bpp_and_scaled`  | `dithered_2` | Png    | 2bpp                         |
| `threshnorm_reg.rs` | `threshold_sweep` | `result`     | Tiff   | 1bpp, guard: `thresh == 128` |
|                     | `spread_norm`     | `normed`     | Png    | 8bpp                         |

### Commit 7: cmapquant + colorquant + grayquant (7 golden)

| ファイル            | テスト関数              | 変数        | Format | 備考                                                        |
| ------------------- | ----------------------- | ----------- | ------ | ----------------------------------------------------------- |
| `cmapquant_reg.rs`  | `main_median_cut_quant` | `pix_mc`    | Png    | Ok arm 内                                                   |
|                     | `algo_comparison`       | `mc_result` | Png    | Ok arm 内                                                   |
| `colorquant_reg.rs` | `colorquant_reg`        | `result`    | Png    | helper 内, guard: `name == "marge.jpg" && max_colors == 16` |
| `grayquant_reg.rs`  | `threshold_binary`      | `result`    | Tiff   | 1bpp                                                        |
|                     | `threshold_multi`       | `result`    | Png    | 8bpp                                                        |
|                     | `color_quant`           | `result`    | Png    | 32bpp                                                       |
|                     | `advanced_threshold`    | `result`    | Tiff   | 1bpp                                                        |

### Commit 8: colorfill + colorseg + colorspace + falsecolor (4 golden)

| ファイル            | テスト関数       | 変数                  | Format | 備考                   |
| ------------------- | ---------------- | --------------------- | ------ | ---------------------- |
| `colorfill_reg.rs`  | `colorfill_reg`  | `r` (Ok(Some(r)) arm) | Tiff   | 1bpp, 最初の成功 match |
| `colorseg_reg.rs`   | `colorseg_reg`   | `result` (Ok arm)     | Png    | 初回設定のみ (guard)   |
| `colorspace_reg.rs` | `colorspace_reg` | `gray_img`            | Png    | 8bpp                   |
| `falsecolor_reg.rs` | `falsecolor_reg` | `mapped`              | Png    | 32bpp                  |

### Commit 9: manifest 生成

```bash
REGTEST_MODE=generate cargo test --test color
```

生成後 `git diff tests/golden_manifest.tsv` で ~61 行の追加を確認。

## 除外ファイル

| ファイル                       | 理由                     |
| ------------------------------ | ------------------------ |
| `colorcontent_reg.rs`          | 値比較のみ、Pix 出力なし |
| `binarize_advanced_reg.rs`     | RegParams 未使用         |
| `color_coverage_reg.rs`        | RegParams 未使用         |
| `color_magnitude_reg.rs`       | RegParams 未使用         |
| `colorcontent_advanced_reg.rs` | RegParams 未使用         |
| `colormask_reg.rs`             | RegParams 未使用         |
| `quantize_ext_reg.rs`          | RegParams 未使用         |
| `colorspace_hsv_reg.rs`        | RegParams 未使用         |
| `paintcmap_reg.rs`             | RegParams 未使用         |

## 実装パターン

### 基本パターン

```rust
rp.write_pix_and_check(&result, ImageFormat::Png)
    .expect("write result <test_name>");
```

### PixMut → Pix 変換後

```rust
let pix_result: Pix = pixmut.into();
rp.write_pix_and_check(&pix_result, ImageFormat::Png)
    .expect("write pix_result <test_name>");
```

### match arm 内

```rust
match some_operation(&pix) {
    Ok(result) => {
        rp.write_pix_and_check(&result, ImageFormat::Png)
            .expect("write result <test_name>");
        // 既存のアサーション
    }
    Err(e) => panic!("..."),
}
```

### ループ内ガード

```rust
for (i, img) in images.iter().enumerate() {
    let result = transform(&pix);
    if i == 0 {
        rp.write_pix_and_check(&result, ImageFormat::Png)
            .expect("write result <test_name>");
    }
}
```

### 1bpp 画像

```rust
rp.write_pix_and_check(&binary_result, ImageFormat::Tiff)
    .expect("write binary_result <test_name>");
```

## ブランチ・コミット規約

- ブランチ: `test/color-golden-enhance`
- コミット prefix: `test(color): enhance <file>_reg with write_pix_and_check`
- 最終コミット: `test(color): generate golden manifest for color regression tests`

## 検証

```bash
# 全テスト通過
cargo test --test color

# manifest 検証
wc -l tests/golden_manifest.tsv  # ~321行 (260 + ~61)
grep -E '^(alphaops|binarize|blend|bw|cmapquant|color|dither|falsecolor|gquant|hardlight|paint|pmask|threshnorm)' tests/golden_manifest.tsv | wc -l  # 0 → ~60

# CI チェック
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
```

## C 版との比較

Rust 側は各テストの主要コードパスから代表的な出力を 1〜5 件キャプチャする方針。
C 版はパラメータ網羅的にキャプチャするため、件数に大きな差がある。

| C テスト         | C 出力数 | Rust 出力数 | 比率  | 備考                       |
| ---------------- | -------- | ----------- | ----- | -------------------------- |
| blend1_reg.c     | 17       | 4           | 1:4   | 4 シナリオ各1件            |
| blend2_reg.c     | 19       | 3           | 1:6   |                            |
| blend3_reg.c     | 6        | 3           | 1:2   |                            |
| blend4_reg.c     | 6        | 4           | 1:1.5 |                            |
| blend5_reg.c     | 15       | 4           | 1:4   | fade + snap                |
| alphaops_reg.c   | 15       | 5           | 1:3   |                            |
| hardlight_reg.c  | 9        | 3           | 1:3   |                            |
| colorize_reg.c   | 15       | 3           | 1:5   |                            |
| coloring_reg.c   | 1        | 2           | 2:1   | Rust の方が多い            |
| paint_reg.c      | 29       | 4           | 1:7   | C は colormap テストが多い |
| paintmask_reg.c  | 22       | 3           | 1:7   |                            |
| blackwhite_reg.c | 2        | 3           | 3:2   |                            |
| binarize_reg.c   | 7        | 3           | 1:2   |                            |
| dither_reg.c     | 5        | 3           | 1:2   |                            |
| threshnorm_reg.c | 1        | 2           | 2:1   |                            |
| cmapquant_reg.c  | 8        | 2           | 1:4   |                            |
| colorquant_reg.c | 1        | 1           | 1:1   | 完全一致                   |
| grayquant_reg.c  | 47       | 4           | 1:12  | C は閾値レベル網羅         |
| colorfill_reg.c  | 12       | 1           | 1:12  |                            |
| colorseg_reg.c   | 3        | 1           | 1:3   |                            |
| colorspace_reg.c | 10       | 1           | 1:10  | C は色空間変換パターン網羅 |
| falsecolor_reg.c | 4        | 1           | 1:4   |                            |

**合計: C=254 出力、Rust=60 出力（全体比率 1:4.2）**

Rust テストは代表パス 1 件でも出力変化を検出できるため、回帰検出としては十分。
C 版の網羅的キャプチャはパラメータ別の差分比較に有用だが、現段階では不要。
