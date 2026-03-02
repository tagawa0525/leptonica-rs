# Phase 3 PR 3/8: io モジュール回帰テスト強化

**Status**: IMPLEMENTED

## Context

Phase 3 PR 1/8（filter, #258）と PR 2/8（morph, #263）がマージ済み。
PR 3/8 として io モジュールのB分類テスト6件を強化する。

全6テストは既に `RegParams` を使用しているが、`compare_values()` のみで
`write_pix_and_check()` / `write_data_and_check()` を使用していない。
golden manifest にエントリがなく、ピクセル単位の回帰検出ができていない。

主な修正: 各テストに `write_pix_and_check` / `write_data_and_check` を追加し、
golden manifest で出力変化を検出可能にする。

## 修正対象（6テスト）

### 1. gifio_reg（難易度: 低、新規golden: ~8）

**ファイル**: `tests/io/gifio_reg.rs`
**C版**: `reference/leptonica/prog/gifio_reg.c`

**現状**: RegParams + compare_values で roundtrip の lossless 判定のみ。write_pix_and_check なし。

**方針**:

- Part 1（file roundtrip）のループ内で、`pix1`（GIF read-back）に `write_pix_and_check` を追加
- 8ファイル × 1 write_pix_and_check = 8エントリ
- Part 2（memory roundtrip）は `equals()` で十分（Part 1 と同じ画像の検証重複を避ける）
- 既存の compare_values はそのまま残す

**変更箇所**:

```rust
// Part 1 ループ内、pix1 取得後に追加:
rp.write_pix_and_check(&pix1, ImageFormat::Png).expect("write_pix_and_check");
```

### 2. webpio_reg（難易度: 低、新規golden: ~4）

**ファイル**: `tests/io/webpio_reg.rs`
**C版**: `reference/leptonica/prog/webpio_reg.c`

**現状**: RegParams + compare_values で寸法チェックのみ。write_pix_and_check なし。

**方針**:

- `do_webp_test1` 内で `pix1`（WebP read-back）に `write_pix_and_check` を追加（4ファイル = 4エントリ）
- Part 2（memory roundtrip）と Part 3（lossless exact）は `equals()` 判定で十分。golden は追加しない
- 既存の #[ignore]（lossy quality, PSNR）はそのまま維持

**変更箇所**:

```rust
// do_webp_test1 内、pix1 取得後:
rp.write_pix_and_check(&pix1, ImageFormat::Png).expect("write_pix_and_check");
```

### 3. ioformats_reg（難易度: 低、新規golden: ~7）

**ファイル**: `tests/io/ioformats_reg.rs`
**C版**: `reference/leptonica/prog/ioformats_reg.c`

**現状**: format detection + 読み取りプロパティ検証 + memory roundtrip。寸法チェックのみ。

**方針**:

- Test 3（Read format tests）で読み込んだ4画像に `write_pix_and_check` を追加（4エントリ）
- Test 4（PNG memory roundtrip）の `pix2` に `write_pix_and_check` を追加（3エントリ）
- format detection テストは画像出力がないため golden 対象外
- C版の header reading テスト（`pixReadHeader`）は Rust に `read_image_header` が存在するので

  #[ignore] ではなく追加可能 → compare_values でヘッダフィールドを検証

**変更箇所**:

```rust
// Test 3: 各画像読み込み後に追加
rp.write_pix_and_check(&pix, ImageFormat::Png).expect("check");

// Test 4: test_png_roundtrip 内で pix2 に追加
rp.write_pix_and_check(&pix2, ImageFormat::Png).expect("check");
```

**追加テスト（任意）**:

- `read_image_header` による width/height/depth 検証（compare_values のみ、golden なし）

### 4. iomisc_reg（難易度: 中、新規golden: ~15）

**ファイル**: `tests/io/iomisc_reg.rs`（9テスト関数）
**C版**: `reference/leptonica/prog/iomisc_reg.c`（32 checks）

**現状**: 9テスト関数で compare_values のみ。write_pix_and_check なし。

**方針**（関数ごと）:

| 関数                                    | 追加内容                                                                                 | 新規golden |
| --------------------------------------- | ---------------------------------------------------------------------------------------- | ---------- |
| `iomisc_reg_16bit_png`                  | pix1（PNG read-back）に write_pix_and_check                                              | 1          |
| `iomisc_reg_png_alpha`                  | pixs, pix_back に write_pix_and_check                                                    | 2          |
| `iomisc_reg_alpha_blend_operations`     | pix_red, pix_blend, pix_alpha に write_pix_and_check                                     | 3          |
| `iomisc_reg_colormap`                   | pixs, pix_back に write_pix_and_check                                                    | 2          |
| `iomisc_reg_remove_regen_rgb_colormap`  | pix_rgb, pix_cmap に write_pix_and_check                                                 | 2          |
| `iomisc_reg_remove_regen_gray_colormap` | pix_cmap, pix_gray に write_pix_and_check                                                | 2          |
| `iomisc_reg_tiff_compression`           | 1bpp roundtrip結果に write_pix_and_check（6形式）。8bpp は equals() で十分 → golden 不要 | 1 ※        |
| `iomisc_reg_pnm_alpha`                  | pix1（PNM read-back）に write_pix_and_check                                              | 1          |
| `iomisc_reg_memory_io`                  | 各 roundtrip pix2 に write_pix_and_check                                                 | 4          |

※ TIFF圧縮: 1bpp は全6形式で結果が同一のはず（lossless）。代表1つのみ golden 化し、
残りは既存の `equals()` で十分。ただし6形式それぞれ golden 化する案もあり（+5）。
→ 代表1つ案を採用（合計 ~18 の場合と ~15 の差。実装時に判断）

**既存 #[ignore] の維持**:

- `iomisc_reg_jpeg_chroma` — JPEG writer / chroma sampling 未実装
- `iomisc_reg_colormap_serialization` — PixColormap stream serialization 未実装

**注意**: `iomisc_reg_format_detection` と `iomisc_reg_input_format` は画像出力がないため
write_pix_and_check 対象外。compare_values のままで問題なし。

### 5. pdfio1_reg（難易度: 低-中、新規golden: ~6）

**ファイル**: `tests/io/pdfio1_reg.rs`（5テスト関数 + 3 #[ignore]）
**C版**: `reference/leptonica/prog/pdfio1_reg.c`（27 checks、全て regTestCheckFile）

**現状**: compare_values で PDF ヘッダ・構造を検証。write_data_and_check なし。

**方針**:

- C版は全て `regTestCheckFile`（ファイル存在確認のみ）で、ピクセル検証なし
- PDF はバイナリ出力なので `write_data_and_check` でハッシュ化
- 各テスト関数で生成した PDF バイト列に `write_data_and_check` を追加

| 関数                          | 新規golden                                                                                                      |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `pdfio1_reg_auto_compression` | 3（1bpp/8bpp/32bpp × 各1 PDF）                                                                                  |
| `pdfio1_reg_flate`            | 1（代表1つ。3つ全部だと+2）                                                                                     |
| `pdfio1_reg_jpeg`             | 1（代表1つ）                                                                                                    |
| `pdfio1_reg_multipage`        | 1（multipage PDF）                                                                                              |
| `pdfio1_reg_title`            | 0（title テストはメタデータ検証。PDF バイト列のハッシュ化は title テストでは不安定になる可能性がある → 見送り） |

**リスク**: PDF 出力の決定性。タイムスタンプ等の可変データがあると golden が不安定になる。
→ 実装時に `REGTEST_MODE=generate` を2回実行してハッシュ安定性を確認する。
不安定な場合は write_data_and_check を見送り、compare_values のみに留める。

**既存 #[ignore] の維持**:

- `pdfio1_reg_segmented` — convertToPdfSegmented 未実装
- `pdfio1_reg_ci_data` — CI data generation 未実装
- `pdfio1_reg_g4_mask` — G4 image mask control 未実装

### 6. writetext_reg（難易度: 中、新規golden: ~3）

**ファイル**: `tests/io/writetext_reg.rs`
**C版**: `reference/leptonica/prog/writetext_reg.c`（32 checks）

**現状**: 合成 8bpp Pix に `add_textlines` → 寸法チェック + SPIX roundtrip。write_pix_and_check なし。

**方針**:

- 既存テストに write_pix_and_check を追加（`with_text` に1つ、`restored` に1つ）
- 実画像ベースのテストを追加: `lucasta.150.jpg`（テストデータに存在）を読み込み、

  `add_textlines` で4方向（Above/Below/Left/Right）のテキスト追加結果を write_pix_and_check
  → ただし C版の `lucasta.047.jpg` とは異なるため C比較は不可。Rust golden のみ

- `set_textline` のテスト追加（C版 checks 5-6 に対応）

**追加テスト**:

```rust
// 実画像 + テキスト追加
let pix = load_test_image("lucasta.150.jpg").expect("load");
let bmf = Bmf::new(6).expect("bmf");
let with_text = bmf.add_textlines(&pix, "regression test", 0, TextLocation::Below)?;
rp.write_pix_and_check(&with_text, ImageFormat::Png)?;
```

**C版で不足する機能の #[ignore]**:

- `pixAddSingleTextblock` — 未実装（C版の主要チェックポイント）
- `pixaDisplayTiledInColumns` — 未実装（C版のタイル表示）

**新規 #[ignore] スタブ**:

```rust
#[test]
#[ignore = "pixAddSingleTextblock not implemented"]
fn writetext_reg_single_textblock() {}
```

## 実装順序

1. **gifio_reg** — 最もシンプル。ループに1行追加
2. **webpio_reg** — gifio と同パターン
3. **ioformats_reg** — 低難易度、7エントリ
4. **iomisc_reg** — 最多関数、中難易度
5. **pdfio1_reg** — write_data_and_check 使用、決定性確認が必要
6. **writetext_reg** — 実画像テスト追加 + #[ignore] スタブ

## コミット戦略

1テスト = 1コミット:

```text
test(io): enhance gifio_reg with write_pix_and_check golden checks
test(io): enhance webpio_reg with write_pix_and_check golden checks
test(io): enhance ioformats_reg with write_pix_and_check golden checks
test(io): enhance iomisc_reg with write_pix_and_check golden checks
test(io): enhance pdfio1_reg with write_data_and_check for PDF output
test(io): enhance writetext_reg with expanded text rendering checks
test(io): generate golden manifest for io regression tests
```

最後に manifest 再生成コミット。

## 重要ファイル

- `tests/io/{gifio,webpio,ioformats,iomisc,pdfio1,writetext}_reg.rs` — 修正対象
- `tests/common/params.rs` — RegParams（write_pix_and_check, write_data_and_check）
- `tests/common/mod.rs` — load_test_image, regout_dir
- `tests/golden_manifest.tsv` — manifest（テスト修正後に再生成）
- `reference/leptonica/prog/{gifio,webpio,ioformats,iomisc,pdfio1,writetext}_reg.c` — C版参照

## 検証

1. `cargo test --test io` — 全テスト通過
2. `cargo clippy --all-features --all-targets -- -D warnings`
3. `cargo fmt --all -- --check`
4. `REGTEST_MODE=generate cargo test --test io` — manifest 再生成
5. `golden_manifest.tsv` の io エントリ数が `write_pix_and_check` + `write_data_and_check` 呼び出し総数と一致
6. manifest を2回生成してハッシュが安定していることを確認（特に pdfio1）

## 新規 golden manifest エントリ見込み

| テスト        | write_pix_and_check | write_data_and_check |   合計 |
| ------------- | ------------------: | -------------------: | -----: |
| gifio_reg     |                   8 |                    0 |      8 |
| webpio_reg    |                   4 |                    0 |      4 |
| ioformats_reg |                   7 |                    0 |      7 |
| iomisc_reg    |                  18 |                    0 |     18 |
| pdfio1_reg    |                   0 |                    8 |      8 |
| writetext_reg |                   3 |                    0 |      3 |
| **合計**      |              **40** |                **8** | **48** |

PR後の manifest 合計: 153 + 48 = 201 エントリ（ヘッダ1行含む）
