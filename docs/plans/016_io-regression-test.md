# Phase 3 PR 3/8: io モジュール回帰テスト強化

Status: PLANNED

## Context

Phase 3 PR 2/8（morph, #267）がマージ済み。PR 3/8 として io モジュールの
B分類テスト5件を強化する。filter/morph で確立したパターン（RegParams +
write_pix_and_check）を踏襲し、C版チェックポイントとの対応を追加する。

selio は `tests/morph/` に配置されており、PR 2/8 の morph 計画で対処済み
（SEL→Pix変換がないため golden 追加は限定的）。

## 修正対象（5テスト）

### 1. gifio_reg（難易度: 低、優先度: 高）

- **C**: 8チェック（2 write_pix_and_check + 6 compare_values）
- **Rust現状**: 16 compare_values（ファイル・メモリ各8件のラウンドトリップ）、0 write_pix_and_check
- **方針**: ファイルラウンドトリップ後の Pix を write_pix_and_check で golden 化。

  C版は GIF→Pix 結果を write_pix するので、Rust も同様に追加。
  メモリラウンドトリップ結果も golden 化して計8-16ファイル。
  palette 保存テストの #[ignore] は維持（Rust GIF I/O の制約）。

### 2. ioformats_reg（難易度: 低、優先度: 中）

- **C**: 10チェック（3 write_pix_and_check + 7 compare_values — header/format detection）
- **Rust現状**: 22 compare_values（寸法・フォーマット検証）、0 write_pix_and_check
- **方針**: 各フォーマット読み込み結果（1bpp/8bpp/32bpp/colormap）を write_pix_and_check。

  PNG ラウンドトリップ結果も golden 化。header reading（pixReadHeader）は未実装 → 既存 #[ignore] 維持。
  約6-8 golden ファイル追加。

### 3. iomisc_reg（難易度: 中、優先度: 最高）

- **C**: 32チェック（7 write_pix_and_check + 6 check_file + 19 compare_values）
- **Rust現状**: 52+ compare_values、0 write_pix_and_check
- **方針**: 以下のカテゴリで write_pix_and_check を追加:
  - **16bit PNG**: 読み込み結果（strip 16→8 後の画像）
  - **Alpha PNG**: 元画像・ラウンドトリップ結果・アルファチャネル抽出
  - **Alpha ブレンド**: blend_uniform 結果・set_alpha_over_white 結果
  - **Colormap**: 元画像・除去後 RGB・再生成後
  - **TIFF 圧縮**: 各圧縮形式（uncompressed, packbits, rle, g3, g4, lzw）のラウンドトリップ
  - **PNM alpha**: ラウンドトリップ結果 + compare_pix

  JPEG chroma sampling（pixSetChromaSampling）は未実装 → 既存 #[ignore] 維持。
  colormap serialization（stream I/O）は未実装 → 既存 #[ignore] 維持。
  約18-22 golden ファイル追加。

### 4. pdfio1_reg（難易度: 低、優先度: 低）

- **C**: 27チェック（全て check_file/compare_values — PDF構造検証）
- **Rust現状**: 12 compare_values（PDF構造 + ヘッダ検証）、0 write_pix_and_check
- **方針**: PDF出力は Pix ではないため write_pix_and_check は不適切。

  代わりに PDF 変換前の入力画像を write_pix_and_check で golden 化し、
  PDF 出力のバイトサイズを compare_values で検証するチェックを追加。
  segmented PDF（convertToPdfSegmented）等は未実装 → 既存 #[ignore] 維持。
  約2-4 golden ファイル追加（入力画像のみ）。

### 5. webpio_reg（難易度: 低、優先度: 低-中）

- **C**: 4チェック（1 check_file + 1 compare_similar_pix + 2 compare_values）
- **Rust現状**: 7 compare_values + .equals() 検証、0 write_pix_and_check
- **方針**: lossless ラウンドトリップ後の Pix を write_pix_and_check で golden 化。

  lossy quality テスト（pixWriteWebP quality, pixGetPSNR）は未実装 → 既存 #[ignore] 維持。
  約4-5 golden ファイル追加。

## Golden ファイル見積もり

| テスト    | 推定 golden 数 | フォーマット            |
| --------- | -------------- | ----------------------- |
| gifio     | 8-16           | GIF (lossless)          |
| ioformats | 6-8            | PNG/BMP/TIFF (lossless) |
| iomisc    | 18-22          | PNG/TIFF (lossless)     |
| pdfio1    | 2-4            | PNG/TIFF (入力画像)     |
| webpio    | 4-5            | WebP (lossless)         |
| **合計**  | **38-55**      |                         |

## 重要ファイル

- `tests/io/*.rs` — 修正対象テスト
- `reference/leptonica/prog/{gifio,ioformats,iomisc,pdfio1,webpio}_reg.c`
- `src/io/` — Rust実装（gif.rs, png.rs, tiff.rs, bmp.rs, webp.rs, pdf.rs）
- `tests/common/params.rs` — RegParams インフラ

## 検証

1. `cargo test --test io` — 全テスト通過
2. `cargo clippy --all-features --all-targets -- -D warnings`
3. `cargo fmt --all -- --check`
4. C版 golden 比較: lossless フォーマット（GIF/PNG/TIFF/WebP）は bit 完全一致を期待
