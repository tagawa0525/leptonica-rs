# Core: Pixa::select_to_pdf (plan 032 残)

Status: PLANNED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-3 (108b)

## 対象 C 関数 (1) + 再分類 (1)

108b 残課題 2 件のうち 1 件を移植、もう 1 件は 🚫 に再分類する。

- `pixaSelectToPdf(pixas, first, last, res, scalefactor, type, quality,
  color, fontsize, fileout) -> int` — 範囲選択した Pixa を PDF に書き出す
  (BMF テキスト注釈は **対象外**; C 版 `fontsize <= 0` 経路に相当)
- `pixaSplitIntoFiles` — ファイルシステム書き出し + デバッグ用途のみ
  で利用される lept_mkdir 系ユーティリティ。Rust ユーザーは
  `write_pdf_multi` + 標準ライブラリのループで自然に書ける。
  comparison/core.md で 🚫 に再分類

## API 設計

```rust
impl Pixa {
    /// C: `pixaSelectToPdf` のシンプル化版 (fontsize <= 0 経路のみ)。
    /// 範囲選択した Pix を multi-page PDF に直接書き出す。
    pub fn select_to_pdf<W: std::io::Write>(
        &self,
        first: usize,
        last: Option<usize>,
        options: &crate::io::pdf::PdfOptions,
        writer: W,
    ) -> crate::io::IoResult<()>;
}
```

## 依存

- 既存 `Pixa::select_range`
- 既存 `crate::io::pdf::write_pdf_multi`

## テスト方針

- 簡単な 2 枚の Pix を Pixa に詰め、select_to_pdf(0, None) で PDF
  バイト列が生成されることを確認 (内容検証は不要、サイズ > 0)
- 範囲外 first (>= len) で空 Pixa → PDF も "空" を許容するか?
  実装上は write_pdf_multi に空スライスを渡してエラーとなる挙動を
  確認

## 完了条件

- [ ] cargo test/clippy/fmt 通過
- [ ] core.md で pixaSelectToPdf を ✅、pixaSplitIntoFiles を 🚫
- [ ] plan 032 で 126 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `select_to_pdf`: BMF / テキスト注釈 (C `pixaAddTextNumber`) は
  実装しない。理由: BMF は Rust 側にデバッグ目的の最小実装しかなく、
  数字注釈は別 API として独立させた方がきれい
- `pixaSplitIntoFiles`: ファイルシステム書き出し + デバッグ用の
  ハードコードパス (`/tmp/lept/split/...`) は Rust では不自然。
  汎用化したいユーザーは自前で `write_pdf_multi` ループを書ける
