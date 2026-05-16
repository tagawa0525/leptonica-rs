# Bmf: pixaSaveFont を移植 (plan 032 カテゴリ M 残)

Status: PLANNED
作成日: 2026-05-16
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ M

## 対象 C 関数

`bmfdata.h` 経由でコンパイル済みの 9 サイズのフォントを `chars-N.pa`
ファイルとして保存するヘルパー。`docs/porting/comparison/misc.md` で
❌ 未実装として残っていた 1 件。

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
    fontsize: u32,    // {4, 6, 8, 10, 12, 14, 16, 18, 20}
) -> Result<()>;
```

## 依存 (すべて Rust 実装済み)

- `Bmf::new(fontsize)` — 9 サイズの bitmap font 生成
- `Bmf::get_font_pixa()` — 内部 Pixa 取得
- `Pixa::write_to_file(path)` — pixa 書き出し

## 設計差分 (C → Rust)

1. C は `indir` がある場合に画像ファイルからフォント抽出を行うが、
   Rust 版はそのケースをサポートしない (compiled-in font のみ)。
   `indir` 引数は省略。
2. Rust の `Bmf::clamp_size` は 18-point を 16 にクランプするが、C 版
   は 18 をそのまま使う。本 Rust 版は要求サイズで Bmf を作って `Pixa`
   を取得するため、18-point の保存結果は実質 16-point になる。
3. 出力ファイル名は C と同じ `chars-{N}.pa`。

## テスト方針

- valid な size で書き出し成功 (round-trip read で要素数が一致)
- 不正な size (奇数 / 範囲外) で `Err`
- 出力ディレクトリが存在しない場合は `Err` (Pixa::write_to_file が
  失敗する想定)
