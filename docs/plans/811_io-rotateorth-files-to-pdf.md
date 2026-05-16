# io/pdf: rotateorthFilesToPdf を移植 (plan 032 カテゴリ M 残)

Status: PLANNED
作成日: 2026-05-16
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ M

## 対象 C 関数

複数の画像ファイルを直角回転 + スケーリングして、まとめて PDF に
書き出すヘルパー (`prog/rotateorthpdf.c` の本体)。
`docs/porting/comparison/misc.md` で ❌ 未実装として残っていた 1 件。

| C 関数                 | 役割                                           |
| ---------------------- | ---------------------------------------------- |
| `rotateorthFilesToPdf` | 入力画像群を回転 + スケール後に PDF へまとめる |

## API 設計

```rust
// in src/io/pdf.rs (extend existing module)

/// C: `rotateorthFilesToPdf`
#[allow(clippy::too_many_arguments)]
pub fn rotateorth_files_to_pdf(
    paths: &[impl AsRef<Path>],
    rotstring: &str,
    scalefactor: f32,        // (0.0, 2.0]; <= 0 → 1.0; > 2.0 → 2.0
    quality: u8,             // jpeg quality: 25..=95, それ以外 → 75
    title: Option<&str>,
    compression: PdfCompression,
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
2. C は `pixacomp` 経由で大量画像のメモリ節約を試みるが、Rust 版は
   常に `Vec<Pix>` を保持する。25 ページ程度までの想定。
3. C の `quality <= 0` 既定は 75。Rust は `u8` で表現するため
   範囲外 (`< 25` または `> 95`) を 75 に丸める。
4. PDF 圧縮方式は `PdfCompression` で渡せる (C は固定で
   `L_DEFAULT_ENCODE`)。

## テスト方針

- 2 画像を回転なしで PDF 化 → 出力ファイルが `%PDF` で始まる
- 回転 + スケール + title を指定 → 出力に title 文字列が含まれる
- 空 paths で `Err`
- 不正な rotstring で `Err`
