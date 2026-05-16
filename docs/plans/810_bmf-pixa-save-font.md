# Bmf: pixaSaveFont を移植 (plan 032 カテゴリ M 残)

Status: IMPLEMENTED
作成日: 2026-05-16
完了日: 2026-05-16
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ M

## 対象 C 関数

C 版は `bmfdata.h` でコンパイル済みの 9 サイズ (4,6,8,10,12,14,16,18,20)
のフォントを `chars-N.pa` ファイルとして保存する。Rust 版 `Bmf::new`
は 18 pt のグリフデータを持たないため、サポートサイズは
{4,6,8,10,12,14,16,20} の 8 種類に限定される。
`docs/porting/comparison/misc.md` で ❌ 未実装として残っていた 1 件。

| C 関数         | 役割                                                   |
| -------------- | ------------------------------------------------------ |
| `pixaSaveFont` | 指定サイズの bitmap font の `Pixa` を `.pa` で書き出す |

## API 設計

```rust
// in src/core/bmf.rs (extend existing module)

/// C: `pixaSaveFont` (indir 引数は省略;
///                    Rust 版は常に compiled-in font_data から生成する)
pub fn pixa_save_font(
    outdir: impl AsRef<Path>,
    fontsize: u32,    // {4, 6, 8, 10, 12, 14, 16, 20} — 18 は Rust に
                      // glyph がないため拒否 (C と異なる)
) -> Result<()>;
```

## 依存 (すべて Rust 実装済み)

- `Bmf::new(fontsize)` — 9 サイズの bitmap font 生成
- `Bmf::get_font_pixa()` — 内部 Pixa 取得
- `Pixa::write_to_file(path)` — pixa 書き出し

## 設計差分 (C → Rust)

1. C は `indir` がある場合に画像ファイルからフォント抽出を行うが、Rust 版はそのケースをサポートしない (compiled-in font のみ)。 `indir` 引数は省略。
2. `Bmf::new` は 18 pt の glyph をコンパイル時に持たず暗黙クランプするため、`pixa_save_font` は 18 を **明示的に拒否** する (`chars-18.pa` に 16 pt の中身が入る不整合を防ぐ)。これは C との主な差分。
3. 出力ファイル名は C と同じ `chars-{N}.pa`。

## テスト方針

- valid な size (10 pt) で書き出し成功 (round-trip read で要素数 95)
- 奇数 / 範囲外で `Err`
- サポート外の偶数 (18 pt) で `Err`
- 出力ディレクトリが存在しない場合に `Err`
