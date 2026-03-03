# Phase 3 PR 4/8: transform モジュール回帰テスト強化

**Status**: IMPLEMENTED

## Context

Phase 3 PR 1/8（filter, #258）、PR 2/8（morph, #263）、PR 3/8（io）がマージ済み。
PR 4/8 として transform モジュールの回帰テスト15件を強化する。

全15テストは既に `RegParams` を使用しているが、`compare_values()` のみで
`write_pix_and_check()` を使用していない。golden manifest にエントリがなく、
ピクセル単位の回帰検出ができていない。

主な修正: 各テストの主要な変換結果に `write_pix_and_check` を追加し、
golden manifest で出力変化を検出可能にする。

## 対象外（7テスト）

以下は RegParams 未使用・全 #[ignore]・画像出力なし等の理由で対象外:

| テスト                 | 理由                                         |
| ---------------------- | -------------------------------------------- |
| smallpix_reg           | RegParams 不使用（plain assert のみ）        |
| circle_reg             | RegParams 不使用（synthetic smoke test）     |
| checkerboard_reg       | RegParams 未活用（`_rp`）                    |
| subpixel_reg           | 全テスト #[ignore]                           |
| transform_coverage_reg | RegParams 不使用（unit test 形式）           |
| projection_reg         | Numa 統計テスト、画像出力は gplot のみ       |
| xformbox_reg           | Boxa 変換テスト、画像出力は hash render のみ |

## 修正対象（15テスト、8グループ）

### グループ 1: 回転（3テスト、新規golden: ~10）

#### rotate1_reg

**ファイル**: `tests/transform/rotate1_reg.rs`
**現状**: compare_values で寸法・identity チェックのみ

**方針**: 中間結果に write_pix_and_check を追加

```rust
// r90 (90度CW)、r180、flr (左右反転)、ftb (上下反転) に追加
rp.write_pix_and_check(&r90, ImageFormat::Tiff)?;
rp.write_pix_and_check(&r180, ImageFormat::Tiff)?;
rp.write_pix_and_check(&flr, ImageFormat::Tiff)?;
rp.write_pix_and_check(&ftb, ImageFormat::Tiff)?;
```

**新規golden**: 4

#### rotate2_reg

**ファイル**: `tests/transform/rotate2_reg.rs`
**現状**: compare_values で有効出力チェックのみ

**方針**: 主要な回転結果に追加

```rust
// 5度回転、pi/6回転、forward+back結果
rp.write_pix_and_check(&rotated, ImageFormat::Png)?;  // 5 deg
rp.write_pix_and_check(&rot_rad, ImageFormat::Png)?;  // pi/6
rp.write_pix_and_check(&back, ImageFormat::Png)?;      // +10 then -10
```

**新規golden**: 3

#### rotateorth_reg

**ファイル**: `tests/transform/rotateorth_reg.rs`
**現状**: compare_values で identity・一致チェックのみ

**方針**: 各深度の rotate_orth(1) 結果（90度CW）に追加

```rust
// test_orth_rotation 内で r1 に追加（1bpp, 8bpp, 32bpp）
rp.write_pix_and_check(&r1, ImageFormat::Png)?;
```

**新規golden**: 3

### グループ 2: スケール（2テスト、新規golden: ~8）

#### scale_reg

**ファイル**: `tests/transform/scale_reg.rs`
**現状**: compare_values で寸法チェックのみ

**方針**: 主要スケール結果に追加

```rust
rp.write_pix_and_check(&up2, ImageFormat::Png)?;      // 2x
rp.write_pix_and_check(&down2, ImageFormat::Png)?;     // 0.5x
rp.write_pix_and_check(&sized, ImageFormat::Png)?;     // to_size
rp.write_pix_and_check(&aniso, ImageFormat::Png)?;     // aniso
```

**新規golden**: 4

#### expand_reg

**ファイル**: `tests/transform/expand_reg.rs`
**現状**: compare_values + compare_pix で寸法・identity チェック

**方針**: 各深度の 2x expand 結果に追加（expand_reg_1bpp, 2bpp, 4bpp, 8bpp の計4箇所）

```rust
rp.write_pix_and_check(&pix2x, ImageFormat::Png)?;
```

**新規golden**: 4

### グループ 3: アフィン変換（1テスト、新規golden: ~6）

#### affine_reg

**ファイル**: `tests/transform/affine_reg.rs`（6テスト関数、うち2つ #[ignore]）
**現状**: compare_values で寸法・diff fraction チェックのみ

**方針**: 各テスト関数の forward transform 結果に追加

- `affine_reg_sampling_invertability`: ループ i=0 の pix1（forward）→ 1
- `affine_reg_grayscale_interpolation_invertability`: ループ i=0 の pix1 → 1
- `affine_reg_large_distortion`: pix_sampled, pix_interp → 2
- `affine_reg_pta_basic`: out（interpolated）→ 1
- `affine_reg_color_interpolation`: out（32bpp interpolated）→ 1

**新規golden**: 6

### グループ 4: バイリニア・射影変換（2テスト、新規golden: ~10）

#### bilinear_reg

**ファイル**: `tests/transform/bilinear_reg.rs`（6テスト関数）
**現状**: compare_values で寸法・diff チェックのみ

**方針**: 各テストの forward 結果に追加

- sampling_invertability: i=1 の pix1 → 1
- grayscale_interpolation_invertability: i=1 の pix1 → 1
- compare_sampling_interpolated: pix_sampled → 1
- large_distortion: pix1（sampled）→ 1
- pta_basic: out（interpolated）→ 1

**新規golden**: 5

#### projective_reg

**ファイル**: `tests/transform/projective_reg.rs`（6テスト関数）
**現状**: bilinear_reg と同パターン

**方針**: bilinear_reg と同一パターン

- sampling_invertability: i=0 の pix1 → 1
- grayscale_interpolation_invertability: i=0 の pix1 → 1
- compare_sampling_interpolated: pix_sampled → 1
- pta_basic: out（interpolated）→ 1
- color_interpolation: out（32bpp）→ 1

**新規golden**: 5

### グループ 5: シアー変換（2テスト、新規golden: ~7）

#### shear1_reg

**ファイル**: `tests/transform/shear1_reg.rs`（4テスト関数 + 1 #[ignore]）
**現状**: compare_values + compare_pix で寸法・一致チェック

**方針**:

- grayscale_8bpp: sheared（h_shear_corner white）→ 1
- binary: hw（h_shear_corner white）→ 1
- in_place: expected（h_shear）→ 1
- interpolated: hli（8bpp）→ 1

**新規golden**: 4

#### shear2_reg

**ファイル**: `tests/transform/shear2_reg.rs`（3テスト関数）
**現状**: compare_values で寸法チェックのみ

**方針**:

- color_sampled: left → 1
- gray_interpolated: left → 1
- general: sampled → 1

**新規golden**: 3

### グループ 6: 移動・切り抜き（2テスト、新規golden: ~5）

#### translate_reg

**ファイル**: `tests/transform/translate_reg.rs`（3テスト + 1 #[ignore]）
**現状**: compare_values でピクセル対応チェック

**方針**: 各テストの shifted 結果に追加

- positive_shift: shifted → 1
- negative_shift: shifted → 1
- rgb: shifted → 1

**新規golden**: 3

#### crop_reg

**ファイル**: `tests/transform/crop_reg.rs`（3テスト + 1 #[ignore]）
**現状**: compare_values で寸法チェック

**方針**: 各テストの clipped 結果に追加

- clip_with_border_contained: clipped → 1
- basic_clip: clipped → 1

**新規golden**: 2

### グループ 7: マルチ深度・アルファ（2テスト、新規golden: ~8）

#### multitype_reg

**ファイル**: `tests/transform/multitype_reg.rs`（6テスト関数）
**現状**: compare_values + compare_pix で寸法チェック

**方針**: 各テストで代表1画像（8bpp or 32bpp）の結果に追加

- rotate: marge.jpg の rotated → 1
- affine: marge.jpg の result → 1
- projective: marge.jpg の result → 1
- bilinear: marge.jpg の result → 1
- scale: marge.jpg の scaled → 1

**新規golden**: 5

#### alphaxform_reg

**ファイル**: `tests/transform/alphaxform_reg.rs`（4テスト + 1 #[ignore]）
**現状**: compare_values で寸法チェック

**方針**: 各テストの変換結果に追加

- rotate_with_alpha: rot_full → 1
- affine_pta_with_alpha: result → 1
- projective_pta_with_alpha: result → 1

**新規golden**: 3

### グループ 8: ワーパー（1テスト、新規golden: ~3）

#### warper_reg

**ファイル**: `tests/transform/warper_reg.rs`（3テスト + 1 RegParams不使用）
**現状**: compare_values + compare_pix で寸法・再現性チェック

**方針**:

- random_harmonic: warped1 → 1
- stereoscopic: result → 1
- stretch_horizontal: stretched_q → 1

**新規golden**: 3

## 実装順序

1. グループ 1: 回転（rotate1, rotate2, rotateorth）
2. グループ 2: スケール（scale, expand）
3. グループ 3: アフィン（affine）
4. グループ 4: バイリニア・射影（bilinear, projective）
5. グループ 5: シアー（shear1, shear2）
6. グループ 6: 移動・切り抜き（translate, crop）
7. グループ 7: マルチ深度・アルファ（multitype, alphaxform）
8. グループ 8: ワーパー（warper）
9. golden manifest 再生成

## コミット戦略

機能グループ単位でコミット:

```text
test(transform): enhance rotate1/rotate2/rotateorth_reg with write_pix_and_check
test(transform): enhance scale/expand_reg with write_pix_and_check
test(transform): enhance affine_reg with write_pix_and_check
test(transform): enhance bilinear/projective_reg with write_pix_and_check
test(transform): enhance shear1/shear2_reg with write_pix_and_check
test(transform): enhance translate/crop_reg with write_pix_and_check
test(transform): enhance multitype/alphaxform_reg with write_pix_and_check
test(transform): enhance warper_reg with write_pix_and_check
test(transform): generate golden manifest for transform regression tests
```

計9コミット。

## 重要ファイル

- `tests/transform/{rotate1,rotate2,rotateorth,scale,expand,affine,bilinear,projective,shear1,shear2,translate,crop,multitype,alphaxform,warper}_reg.rs` — 修正対象（15ファイル）
- `tests/common/params.rs` — RegParams（write_pix_and_check）
- `tests/golden_manifest.tsv` — manifest（再生成）
- `reference/leptonica/prog/*_reg.c` — C版参照

## 技術的注意

- 1bpp 画像は `ImageFormat::Tiff`、それ以外は `ImageFormat::Png` を使用
- `write_pix_and_check` の `?` オペレータは各テスト関数の戻り値型に依存。

  現在のテストは `assert!(rp.cleanup())` パターンなので `expect()` で呼ぶ

- ループ内のテストでは代表 1 イテレーションのみ golden 化（全イテレーションは冗長）
- `test_orth_rotation` ヘルパーは `rp` を `&mut` で受けるので追加は容易

## 検証

1. `cargo test --test transform` — 全テスト通過
2. `cargo clippy --all-features --all-targets -- -D warnings`
3. `cargo fmt --all -- --check`
4. `REGTEST_MODE=generate cargo test --test transform` — manifest 再生成
5. `golden_manifest.tsv` の transform エントリ数が write_pix_and_check 呼び出し総数と一致
6. manifest を2回生成してハッシュが安定していることを確認

## 新規 golden manifest エントリ見込み

| グループ             | テスト                       | 見込み数 |
| -------------------- | ---------------------------- | -------: |
| 回転                 | rotate1, rotate2, rotateorth |       10 |
| スケール             | scale, expand                |        8 |
| アフィン             | affine                       |        6 |
| バイリニア・射影     | bilinear, projective         |       10 |
| シアー               | shear1, shear2               |        7 |
| 移動・切り抜き       | translate, crop              |        5 |
| マルチ深度・アルファ | multitype, alphaxform        |        8 |
| ワーパー             | warper                       |        3 |
| **合計**             | **15テスト**                 |  **~57** |

PR後の manifest 合計: 203 + ~57 = ~260 エントリ
