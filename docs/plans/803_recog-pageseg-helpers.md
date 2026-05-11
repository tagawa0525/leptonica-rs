# Recog: pageseg.c の補助 4 関数 (plan 032 カテゴリ C の一部)

Status: PLANNED
作成日: 2026-05-11
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ C

## 対象 C 関数 (4)

軽量・独立性の高い pageseg.c 補助関数。重い高レベル関数
(`pixCleanImage` / `pixCropImage` / `pixCountTextColumns` /
`pixDecideIfText` / `pixEstimateBackground` /
`pixExtractRawTextlines`) は別の plan 803b で扱う。

### Threshold 範囲検出

- `pixFindThreshFgExtent(pix, thresh, &top, &bot)` — 1bpp の各行の

  FG ピクセル数を `thresh` と比較し、`top` (上端) と `bot` (下端) を
  返す

### マスク生成

- `pixGenHalftoneMask(pixs, &ppixtext, &htfound)` — ハーフトーン

  領域マスクとテキストピクセルを返す

- `pixGenTextlineMask(pixs, &ppixvws, &tlfound)` — テキスト行マスクと

  垂直空白マスクを返す

- `pixGenTextblockMask(pixs, pixvws)` — `pixGenTextlineMask` の

  `pixvws` を入力に取りテキストブロックマスクを生成

## API 設計

```rust
// in src/recog/pageseg.rs (extend existing module)

/// C: `pixFindThreshFgExtent`
pub fn pix_find_thresh_fg_extent(pixs: &Pix, thresh: u32) -> Result<(u32, u32)>;

/// C: `pixGenHalftoneMask`
/// Returns (halftone_mask, text_pix, halftone_found)
pub fn pix_gen_halftone_mask(pixs: &Pix) -> Result<(Pix, Pix, bool)>;

/// C: `pixGenTextlineMask`
/// Returns (textline_mask, vertical_whitespace_mask, textline_found)
pub fn pix_gen_textline_mask(pixs: &Pix) -> Result<(Pix, Pix, bool)>;

/// C: `pixGenTextblockMask` (pixvws comes from pix_gen_textline_mask)
pub fn pix_gen_textblock_mask(pixs: &Pix, pixvws: &Pix) -> Result<Option<Pix>>;
```

## 依存

- 既存: `Pix::count_by_row`, `Pix::invert`, `Pix::subtract`, `Pix::is_zero`
- 既存: `morph_sequence`, `morph_comp_sequence`, `morph_sequence_by_component`, `close_safe_brick`, `open_brick`
- 既存: `region::pix_select_by_size`

## テスト方針

- `pix_find_thresh_fg_extent`:
  - 全 0 (top=bot=0)
  - 単一行 FG (top=bot=row_idx)
  - 連続 FG 領域 (top, bot 一致)
  - 非 1bpp で Err
- `pix_gen_halftone_mask`: ハーフトーンパターン入力で halftone_found=true
- `pix_gen_textline_mask`: テキスト風パターン入力で textline_found=true
- `pix_gen_textblock_mask`: 空 FG -> None、有効 FG で Pix

## 完了条件

- [ ] cargo test/clippy/fmt 通過
- [ ] recog.md 4 件 ❌ -> ✅
- [ ] plan 032 で 803 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ
