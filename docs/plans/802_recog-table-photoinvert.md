# recog/pageseg: 表判定・自動反転の移植

Status: IN_PROGRESS
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 L)

## Context

C 版 `pageseg.c` の以下 2 関数が Rust に未移植。`tests/recog/pageseg_reg.rs`
には 2 件の `#[ignore]` が残っている。

| C 関数               | 行             | 役割                                                                                     |
| -------------------- | -------------- | ---------------------------------------------------------------------------------------- |
| `pixDecideIfTable`   | pageseg.c:2172 | テーブル判定。横/縦の前景線数 + 縦の背景線数からスコア 0–4 を返す（>= 2 でテーブル判定） |
| `pixAutoPhotoinvert` | pageseg.c:2923 | 反転テキスト領域を halftone mask + 形態学で検出し、領域内 fg ≥ 60% なら photoinvert する |

Rust 既存依存（確認済み）:

- `Pix::convert_to_1`、`Pix::invert`、`Pix::or`、`Pix::subtract`、

  `Pix::clip_rectangle`、`Pix::foreground_fraction`、
  `Pix::combine_masked`、`Pix::is_zero`

- `morph_sequence` (`src/morph/sequence.rs`)
- `dilate_brick` (`src/morph/binary.rs`)
- `seedfill_binary`、`fill_holes_to_bounding_rect` (`src/region/seedfill.rs`)
- `count_conn_comp`、`find_connected_components` (`src/region/conncomp.rs`)
- `pix_select_by_size` (`src/region/select.rs`)
- `deskew_both` (`src/recog/skew.rs`)
- `rotate_90` (`src/transform/rotate.rs`)
- `generate_halftone_mask` (`src/recog/pageseg.rs`、現在 private — `pub(crate)` に昇格)

未実装依存:

- `pix_prepare_1bpp` 相当: 任意 box clip → 任意 background normalize → 1bpp 化 を 1 回でやる pageseg.c のヘルパ。本タスク内で recog/pageseg の内部 helper として実装。

## アルゴリズム概要

### `decide_if_table(pix, box, orient) -> Result<i32>`

1. 175 ppi で 1bpp 化 → halftone mask 検出。halftone があれば `score = 0` を返す
2. 75 ppi で 1bpp 化 → 2x2 dilate
3. `deskew_both` で水平/垂直 deskew
4. `orient == Landscape` なら 90° 回転
5. 4 つの形態学シーケンスで横/縦黒線、縦白線を抽出
6. 連結成分数 `nhb`、`nvb`、`nvw` を数える
7. スコア:
   - `nhb > 1` で +1
   - `nvb > 2` で +1
   - `nvw > 3` で +1
   - `nvw > 6` で +1
8. score >= 2 ならテーブル

### `auto_photoinvert(pix, thresh) -> Result<(Pix, Option<Pix>)>`

1. `convert_to_1(thresh)` で 1bpp 化
2. `generate_halftone_mask` で halftone 候補を抽出
3. 形態学 `o15.15 + c25.25` でノイズ除去
4. `fill_holes_to_bounding_rect` で穴埋め
5. mask が空なら 1bpp 結果を返す
6. 各連結成分について、領域内 fg ≥ 60% なら反転対象として保持、そうでなければ mask から消去
7. 残った mask 領域に photoinvert を適用 (`pix_inverted = pix.invert()`、

   `pix_combined = combine_masked(pix1, pix_inverted, mask)`)

8. 1bpp 結果と `Option<mask>` を返す

## 配置先・API 設計

- ファイル: `src/recog/pageseg.rs` に追記
- 公開 API:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageOrientation {
    Portrait,  // C: L_PORTRAIT_MODE
    Landscape, // C: L_LANDSCAPE_MODE
}

/// pixDecideIfTable 相当
///
/// Returns -1 if not determined (per C semantics), or 0..=4 score.
pub fn decide_if_table(
    pix: &Pix,
    box_: Option<&Box>,
    orient: PageOrientation,
) -> RecogResult<i32>;

/// pixAutoPhotoinvert 相当
///
/// Returns the post-processed 1bpp image and an optional mask of inverted
/// regions.
pub fn auto_photoinvert(
    pix: &Pix,
    thresh: u32,
) -> RecogResult<(Pix, Option<Pix>)>;
```

`PageOrientation` は新規 enum として `recog/pageseg.rs` 内に定義。

## TDD ステップ

1. **RED**: 公開 API stub と `tests/recog/pageseg_reg.rs::test_23_30_decide_if_table` /

   `test_31_36_auto_photoinvert` の `#[ignore]` を RED に書き換え。テストは
   実画像 (table.* / invertedtext.tif など) が無い場合は内部合成で実行する
   smoke 観点に絞る

2. **GREEN**: 上記アルゴリズムを実装、`#[ignore]` 解除

## ブランチ・PR

- ブランチ: `feat/recog-table-photoinvert`
- PR: `feat(recog): port pixDecideIfTable / pixAutoPhotoinvert`
- 1PR、RED → GREEN

## ステータス

- [x] 計画書作成
- [ ] RED コミット
- [ ] GREEN コミット
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新
