# io/pdf: rotateorthFilesToPdf を移植 (plan 032 カテゴリ M 残)

Status: IMPLEMENTED
作成日: 2026-05-16
完了日: 2026-05-16
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ M

## 対象 C 関数

複数の画像ファイルを直角回転 + スケーリングして、まとめて PDF に
書き出すヘルパー (`prog/rotateorthpdf.c` の本体)。
`docs/porting/comparison/misc.md` で ❌ 未実装として残っていた 1 件。

| C 関数                 | 役割                                           |
| ---------------------- | ---------------------------------------------- |
| `rotateorthFilesToPdf` | 入力画像群を回転 + スケール後に PDF へまとめる |

## 補足: 既存実装

調査の結果、`rotate_orth_files_to_pdf` (Vec<u8> を返す版) が plan 030 で
既に実装されていた。`misc.md` の audit 表で ❌ になっていたのは命名揺れ
(`rotateorth_files_to_pdf` を探していた) のため。

そのため本計画は次のように修正:

1. 既存 `rotate_orth_files_to_pdf` はそのまま (バイト列を返す idiomatic 形)
2. C の `fileout` 引数に対応する薄いファイル出力ラッパーを追加

## API 設計

```rust
// in src/io/pdf.rs (extend existing module)

/// C 形と同じく fileout を取るラッパー
pub fn rotate_orth_files_to_pdf_file(
    paths: &[impl AsRef<Path>],
    rotstring: &str,
    scalefactor: f32,
    quality: i32,
    title: &str,
    output: impl AsRef<Path>,
) -> IoResult<()>;
```

## 依存 (すべて Rust 実装済み)

- `io::pdf::parse_rotation_string` (既存 internal helper)
- `io::pdf::generate_pdf` (既存 internal helper)
- `io::read_image`
- `transform::rotate_orth`
- `transform::scale` / `ScaleMethod::Auto`
- `Pix::infer_resolution`

## 設計差分 (C → Rust)

1. C は `SARRAY *` を受け取るが、Rust 版は `&[impl AsRef<Path>]`。
2. C は `pixacomp` 経由で大量画像のメモリ節約を試みるが、Rust 版は常に `Vec<Pix>` を保持する (既存 `rotate_orth_files_to_pdf` の挙動)。
3. PDF 圧縮方式は first page の `select_default_encoding` で自動選択 (RGB は Jpeg、低ビット深度は Flate)。
4. 戻り値は C の 0/1 ではなく `IoResult<()>`。

## テスト方針

- ファイル出力ラッパー: 基本ケース / title 埋め込み / 空 paths Err / 不正 rotstring Err
- バイト列ラッパーの 振る舞いカバレッジ:
  - 回転 1 と 0 で出力バイト列が異なる (rotation が無視されないこと)
  - scalefactor 1.0 と 0.5 で出力バイト列が異なる (scaling が無視されないこと)
