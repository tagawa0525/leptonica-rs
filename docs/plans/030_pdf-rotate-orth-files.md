# rotateorthFilesToPdf の Rust 移植

Status: IMPLEMENTED

## Context

C 版 leptonica のコミット `bb244a3176f3d9a29acb33458548b56776f87a25`
（"New program for selective orthogonal rotation of images in a pdf."）で、
`src/pdfapp.c` に `rotateorthFilesToPdf()` が追加された。
3 種類のモードを持つ回転指定文字列に従って、画像群を選択的に 90° 単位で
回転 → スケール → PDF 化する高水準ヘルパーである。

Rust 版には既に同系統の関数 (`compress_files_to_pdf`,
`crop_files_to_pdf`, `clean_to_1bpp_files_to_pdf`) が `src/io/pdf.rs` に実装
されているが、`rotate_orth_files_to_pdf` 相当が無いため、画像の選択的直交
回転を伴う PDF 化を行いたい場合の標準 API が無い。

## Goal

| C 版                                           | Rust 版                     | 場所                            |
| ---------------------------------------------- | --------------------------- | ------------------------------- |
| `rotateorthFilesToPdf` (`src/pdfapp.c`)        | `rotate_orth_files_to_pdf`  | `src/io/pdf.rs`                 |
| `parseRotationString` (`src/pdfapp.c`, static) | `parse_rotation_string`     | `src/io/pdf.rs`（モジュール内） |
| `prog/rotateorthpdf.c`                         | `examples/rotateorthpdf.rs` | `examples/`                     |

### 回転指定文字列の仕様

- **Mode 1** (`'0'..='3'` で開始): 各文字 1 桁が画像 i の 90° cw 回転回数。

  文字列長が画像数より短ければ、残りは 0 (無回転)。

- **Mode 2** (`'4'` で開始): 続く 1 桁を全画像に共通で適用。
- **Mode 3** (`'5'` で開始): `(index, rotval)` ペアの並び。区切り文字は任意

  (`,`, `;`, `#`, 空白等は無視)。範囲外 / 無効ペアは警告してスキップ。
  解釈は `sscanf("(%d,%d)", ...)` 相当。

戻り値は長さ `n` の `Vec<u8>` (各値は `0..=3`)。

## 非Goal

- C 版 `l_pdfRenderFile()`（外部 `pdftoppm` 呼び出しによる PDF→画像変換）の

  移植は対象外。これは元々 Rust 版でも未対応の領域で、独立した課題。

- C 版が `n > 25` で `PIXAC` (圧縮 pixa) に切り替えるメモリ最適化は、

  Rust 版では `Pixa` 1 種類で扱う（`Pixa` は `Vec<Pix>` 相当で `Arc` 共有
  されているため、C 版ほどメモリ圧迫しない）。

## TDD コミット構成

1. **RED**: `parse_rotation_string` のユニットテスト（3 モード × 正常 / 異常）と、

   `rotate_orth_files_to_pdf` の統合テスト（PNG ファイル数枚を入力に PDF 出力で
   先頭が `%PDF-` であること、ページ数が一致することを検証）。
   `parse_rotation_string` / `rotate_orth_files_to_pdf` のシグネチャだけ追加し、
   本体は `unimplemented!()` で `#[ignore]` 付き。

2. **GREEN**: 本体実装。`#[ignore]` を外す。
3. **REFACTOR** (必要に応じて)

その後 `examples/rotateorthpdf.rs` を別コミットで追加し、
`docs/plans/030_pdf-rotate-orth-files.md` の Status を IMPLEMENTED に更新。
