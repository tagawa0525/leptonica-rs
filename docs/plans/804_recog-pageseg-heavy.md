# Recog: pageseg.c の重い高レベル 5 関数 (plan 032 カテゴリ C 残り)

Status: IMPLEMENTED
作成日: 2026-05-16
完了日: 2026-05-16
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ C
関連計画: docs/plans/803_recog-pageseg-helpers.md (補助 4 関数)

## 対象 C 関数 (5)

plan 803 (補助関数) で実装を見送り、別計画で扱うとされていた高レベル
ページセグメンテーション関数群。`docs/porting/comparison/recog.md` の
gap-fill audit 2026-05-10 でも ❌ 未実装として残っていた 5 件。

| C 関数                   | 役割                                                  |
| ------------------------ | ----------------------------------------------------- |
| `pixCleanImage`          | deskew + 任意回転 + 背景白化 + 2値化 + 微小ノイズ除去 |
| `pixCountTextColumns`    | テキスト列数を列方向 FG 投影のピーク数から推定        |
| `pixCropImage`           | ページ前景の検出と再配置 (主に印刷用前処理)           |
| `pixDecideIfText`        | テキスト vs 写真の判定 (連結成分解析ベース)           |
| `pixExtractRawTextlines` | テキスト行を分離して PIXA に格納                      |

## API 設計

```rust
// in src/recog/pageseg.rs (extend existing module)

/// C: `pixCleanImage`
pub fn pix_clean_image(
    pixs: &Pix,
    contrast: u32,   // 1..=10
    rotation: u32,   // 0..=3 (cw 90° quad)
    scale: u32,      // 1 or 2
    opensize: u32,   // 0/1=skip, 2 or 3
) -> RecogResult<Pix>;

/// C: `pixCountTextColumns`
/// `pixadb` (デバッグ用) は省略。Rust 版は `n_cols` (>= 0) を返す。
pub fn pix_count_text_columns(
    pixs: &Pix,
    deltafract: f32,    // 0.15..=0.75
    peakfract: f32,     // 0.25..=0.9
    clipfract: f32,     // 0.0..0.5
) -> RecogResult<u32>;

/// C: `pixDecideIfText`. `box_` を省略すると全体を解析する。
/// Result: `Some(true)` = text / `Some(false)` = photo / `None` = 判定不能。
pub fn pix_decide_if_text(
    pixs: &Pix,
    box_: Option<&crate::core::Box>,
) -> RecogResult<Option<bool>>;

/// C: `pixExtractRawTextlines`
pub fn pix_extract_raw_textlines(
    pixs: &Pix,
    maxw: i32,   // 0 = 0.5 * resolution
    maxh: i32,   // 0 = 0.5 * resolution
    adjw: i32,
    adjh: i32,
) -> RecogResult<Pixa>;

/// C: `pixCropImage`. Returns `(cropped, crop_box)`.
pub fn pix_crop_image(
    pixs: &Pix,
    lr_clear: i32,
    tb_clear: i32,
    edgeclean: i32,    // -2, -1, 0..=15
    lr_border: i32,
    tb_border: i32,
    maxwiden: f32,
    printwiden: u32,    // 0=skip, 1=US Letter, 2=A4
) -> RecogResult<(Pix, crate::core::Box)>;
```

## 依存 (すべて Rust 実装済み)

- `filter::background_norm_to_1_min_max`
- `filter::clean_background_to_white`
- `color::threshold_to_binary`
- `morph::morph_sequence` / `morph_comp_sequence` / `close_safe_brick` /
  `hit_miss_transform`
- `transform::scale` / `rotate_orth` / `expand_binary_replicate` /
  `reduce_rank_binary_2` / `reduce_rank_binary_cascade`
- `region::pix_select_by_size` / `conncomp_pixa` / `seedfill_binary_restricted`
- `recog::skew::deskew` / `find_skew_and_deskew`
- `recog::pageseg::pix_find_thresh_fg_extent`
- `Pix::convert_to_8` / `convert_to_1_*` / `clip_rectangle` /
  `clip_rectangles` / `clip_to_foreground` / `count_by_column` /
  `count_by_row` / `set_or_clear_border` / `xor` / `or` / `invert` /
  `subtract` / `is_zero`
- `Boxa::sort_2d` / `select_by_size` / `sort_by_*` / `adjust_sides`
- `Boxaa::get_extent`
- `Numa::find_extrema_with_values` / `transform` / `min_value` /
  `max_value` / `get`

## 設計差分 (C → Rust)

1. デバッグ用 `pixa` 引数は省略 (Debug trait と外部ツールで代替する方針)。
2. `pixDecideIfText` は C の out-param `*pistext ∈ {-1, 0, 1}` を
   `Option<bool>` で表現。
3. `pixCropImage` は C の `BOX **pcropbox` (任意) を必須の戻り値
   `(Pix, Box)` に変更。crop box は常に意味があるため。
4. `pixBackgroundNormTo1MinMax` の C 版 `scale` 引数は Rust 公開関数で
   未公開。`scale == 2` の場合は先に入力 8bpp を 2x bilinear 拡大して
   から `background_norm_to_1_min_max(contrast)` を呼ぶ。

## 内部ヘルパー (pixCropImage で必要)

C には以下の static 関数があるが、Rust 版でも `pageseg.rs` 内の
プライベートヘルパーとして移植する:

- `pixMaxCompAfterVClosing` — 大きい縦方向クローズ後の最大成分検出
  (edgeclean == -1)
- `pixFindPageInsideBlackBorder` — 黒枠を持つページの内側検出
  (edgeclean == -2)
- `pixRescaleForCropping` — クロップ後のページを元サイズに再配置

## テスト方針

- 単純な合成画像 (`Pix::new` で生成) で動作を確認
- C版の出力との完全一致は目的としない (浮動小数演算の差異あり)
- 失敗パス: 不正パラメータで `Err`、空画像で適切な戻り値
- 既存の `tests/recog/pageseg_*` テストパターンに合わせる

## ステータス

- [x] plan コミット
- [x] `pix_clean_image` 実装 + テスト
- [x] `pix_count_text_columns` 実装 + テスト
- [x] `pix_decide_if_text` 実装 + テスト
- [x] `pix_extract_raw_textlines` 実装 + テスト
- [x] `pix_crop_image` 実装 + テスト
- [x] `docs/porting/comparison/recog.md` 更新 (5 件 ❌ → ✅)

## 副次修正

- `Boxaa::align_box`: 空ボックス配列のとき `max_ovlp = i32::MIN`
  との加算でオーバーフローしていたため `saturating_add` に変更。
- `RecogError` に `Filter(FilterError)` バリアントを追加し、
  `background_norm_to_1_min_max` の `?` 変換を有効化。
