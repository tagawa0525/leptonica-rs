# Phase 4: 統計/Numa拡張 実装計画

Status: IMPLEMENTED

## Context

Phase 1-3 がほぼ完了し、残りギャップのうち最も基盤的な Phase 4 に着手する。
Phase 4 は以下の2つのサブフェーズから成る：

- **4.2 Numa高度操作**: sort, reverse, make_constant, rank_value, median, mode
- **4.1 ヒストグラム/統計拡張**: masked histogram, rect histogram, extreme value, rank value, max-in-rect

4.2 を先に実装する（4.1 の `pixel_rank_value` が Numa のソート機能を活用できるため）。

## Phase 4.2: Numa高度操作

### ブランチ: `feat/core-numa-ops`

### 対象ファイル

| ファイル | 変更種別 |
|---------|---------|
| `crates/leptonica-core/src/numa/operations.rs` | 既存拡張 |
| `crates/leptonica-core/src/numa/mod.rs` | re-export追加 |
| `crates/leptonica-core/src/lib.rs` | re-export追加 |

### C参照コード

- `reference/leptonica/src/numafunc1.c`
  - `numaMakeConstant` (line 820)
  - `numaReverse` (line 1304)
  - `numaSort` (line 2567)
  - `numaGetRankValue` (line 3245)
  - `numaGetMedian` (line 3296)
  - `numaGetMode` (line 3443)

### 実装する関数

#### 1. `Numa::make_constant(val, count) -> Numa`

- C: `numaMakeConstant(val, size)`
- `make_sequence(val, 0.0, count)` のラッパー。1行で済む

#### 2. `Numa::reversed(&self) -> Numa` / `Numa::reverse(&mut self)`

- C: `numaReverse(nad, nas)`
- `reversed()`: 新規Numa作成、末尾から先頭へコピー
- `reverse()`: in-placeでスワップ
- メタデータも反転: `startx = startx + (n-1) * delx`, `delx = -delx`

#### 3. `SortOrder` enum + `Numa::sorted(&self, order) -> Numa` / `Numa::sort(&mut self, order)`

- C: `numaSort(naout, nain, sortorder)` — Shell sort
- Rust: `slice::sort_by(f32::total_cmp)` を使用（標準ライブラリ活用、NaN安全）
- `SortOrder::Increasing` / `SortOrder::Decreasing`

#### 4. `Numa::rank_value(&self, fract) -> Result<f32>`

- C: `numaGetRankValue(na, fract, nasort, usebins, &pval)`
- `fract`: 0.0（最小）〜 1.0（最大）
- アルゴリズム: ソート済みコピーを作成、`index = (fract * (n-1) + 0.5) as usize` で取得
- 空配列はエラー、fract範囲外もエラー

#### 5. `Numa::median(&self) -> Result<f32>`

- C: `numaGetMedian(na, &pval)`
- `self.rank_value(0.5)` のラッパー

#### 6. `Numa::mode(&self) -> Result<(f32, usize)>`

- C: `numaGetMode(na, &pval, &pcount)`
- アルゴリズム: 降順ソート → 同値の連続をカウント → 最長ランの値と回数を返す
- 返り値: `(最頻値, 出現回数)`

### 実装順序（依存関係順）

```
make_constant  ← standalone
reverse        ← standalone
sort           ← standalone
rank_value     ← sort に依存
median         ← rank_value に依存
mode           ← sort に依存
```

### TDDコミット計画

1. `test(core): add tests for Numa make_constant and reverse` — RED（`#[ignore]`付き）
2. `feat(core): implement Numa make_constant and reverse` — GREEN
3. `test(core): add tests for Numa sort` — RED
4. `feat(core): implement Numa sort` — GREEN
5. `test(core): add tests for Numa rank_value, median, and mode` — RED
6. `feat(core): implement Numa rank_value, median, and mode` — GREEN
7. （必要に応じて）`refactor(core): ...` — REFACTOR

---

## Phase 4.1: ヒストグラム/統計拡張

### ブランチ: `feat/core-stats-histogram`（4.2マージ後にmainから作成）

### 対象ファイル

| ファイル | 変更種別 |
|---------|---------|
| `crates/leptonica-core/src/pix/histogram.rs` | 既存拡張（masked/rect histogram追加） |
| `crates/leptonica-core/src/pix/statistics.rs` | 既存拡張（extreme/rank/max関数追加） |
| `crates/leptonica-core/src/pix/mod.rs` | re-export追加、`clip_box_to_rect` を `pub(crate)` 化 |
| `crates/leptonica-core/src/lib.rs` | re-export追加 |

### C参照コード

- `reference/leptonica/src/pix4.c`
  - `pixGetGrayHistogramInRect` (line 287)
  - `pixGetGrayHistogramMasked` (line 209)
  - `pixGetColorHistogramMasked` (line 508)
  - `pixGetRankValue` (line 980)
  - `pixGetExtremeValue` (line 2165)
  - `pixGetMaxValueInRect` (line 2300)
  - `pixGetRangeValues` (line 2089)

### 準備: `clip_box_to_rect` の共有化

`statistics.rs:34` の `fn clip_box_to_rect` を `pub(crate)` にして `histogram.rs` からも利用可能にする。
現在 `statistics.rs` 内のプライベート関数のため、`pub(crate)` に変更するだけで `histogram.rs` から `use super::statistics::clip_box_to_rect;` でインポートできる。

### 実装する関数

#### 1. `Pix::gray_histogram_in_rect(region, factor) -> Result<Numa>`（histogram.rs）

- C: `pixGetGrayHistogramInRect(pixs, box, factor)`
- 8bpp or colormapped。矩形内の256ビンヒストグラム
- `region: Option<&Box>` — Noneなら既存の `gray_histogram` に委譲
- `clip_box_to_rect` でクリッピング

#### 2. `Pix::gray_histogram_masked(mask, x, y, factor) -> Result<Numa>`（histogram.rs）

- C: `pixGetGrayHistogramMasked(pixs, pixm, x, y, factor)`
- 1bppマスク画像を `(x, y)` の位置に配置、マスクONのピクセルのみ集計
- `mask: Option<&Pix>` — Noneなら `gray_histogram` に委譲
- マスクの各ピクセル `(mx, my)` に対しソース座標 `(x+mx, y+my)` を計算、境界外はスキップ

#### 3. `Pix::color_histogram_masked(mask, x, y, factor) -> Result<ColorHistogram>`（histogram.rs）

- C: `pixGetColorHistogramMasked(pixs, pixm, x, y, factor, ...)`
- 32bpp画像、R/G/B 各256ビン、マスク適用
- `mask: Option<&Pix>` — Noneなら `color_histogram` に委譲

#### 4. `ExtremeType` enum + `Pix::extreme_value(factor, extreme_type) -> Result<ExtremeResult>`（statistics.rs）

- C: `pixGetExtremeValue(pixs, factor, type, ...)`
- `ExtremeType::Min` / `ExtremeType::Max`
- 8bpp → `ExtremeResult::Gray(u32)`, 32bpp → `ExtremeResult::Rgb { r, g, b }`
- colormapped対応: カラーマップ経由でグレー値取得

#### 5. `MaxValueResult` struct + `Pix::max_value_in_rect(region) -> Result<MaxValueResult>`（statistics.rs）

- C: `pixGetMaxValueInRect(pixs, box, &pmaxval, &pxmax, &pymax)`
- 8/16/32bpp対応。最大値とその座標 `(x, y)` を返す
- 全ゼロの場合は矩形中央を返す（C版準拠）

#### 6. `Pix::range_values(factor, color) -> Result<(u32, u32)>`（statistics.rs）

- C: `pixGetRangeValues(pixs, factor, color, &pminval, &pmaxval)`
- `extreme_value` を2回呼んでmin/maxをまとめて返すラッパー
- 8bppではcolor引数を無視、32bppでは指定チャンネルのみ

#### 7. `Pix::pixel_rank_value(factor, rank) -> Result<u32>`（statistics.rs）

- C: `pixGetRankValue(pixs, factor, rank, &pvalue)`
- 8bpp: `gray_histogram` → `histogram_val_from_rank`
- 32bpp: 1パスでR/G/B各256ビンヒストグラムを構築 → 各チャンネルrank値 → `compose_rgb`

### 新規型定義

```rust
// statistics.rs
pub enum ExtremeType { Min, Max }
pub enum ExtremeResult { Gray(u32), Rgb { r: u32, g: u32, b: u32 } }
pub struct MaxValueResult { pub max_val: u32, pub x: u32, pub y: u32 }
```

`ColorSelect` は `range_values` 用だが、既存の `RgbComponent`（rgb.rs）と重複するため、`RgbComponent` を再利用する。Alpha以外のR/G/Bを受け付ける。

### 実装順序

```
[準備] clip_box_to_rect pub(crate)化
gray_histogram_in_rect     ← clip_box_to_rect に依存
gray_histogram_masked      ← standalone
color_histogram_masked     ← standalone
extreme_value              ← standalone
max_value_in_rect          ← clip_box_to_rect に依存
range_values               ← extreme_value に依存
pixel_rank_value           ← gray_histogram / color_histogram + histogram_val_from_rank に依存
```

### TDDコミット計画

1. `refactor(core): make clip_box_to_rect pub(crate) in statistics.rs` — REFACTOR
2. `test(core): add tests for gray_histogram_in_rect and gray_histogram_masked` — RED
3. `feat(core): implement gray_histogram_in_rect and gray_histogram_masked` — GREEN
4. `test(core): add tests for color_histogram_masked` — RED
5. `feat(core): implement color_histogram_masked` — GREEN
6. `test(core): add tests for extreme_value and max_value_in_rect` — RED
7. `feat(core): implement extreme_value and max_value_in_rect` — GREEN
8. `test(core): add tests for range_values and pixel_rank_value` — RED
9. `feat(core): implement range_values and pixel_rank_value` — GREEN
10. （必要に応じて）`refactor(core): ...` — REFACTOR

---

## ドキュメント更新

各PRマージ後に更新:
- `docs/rebuild/comparison/core.md` — 実装した関数の ❌→✅
- `docs/rebuild/feature-comparison.md` — カバレッジ数値更新

## 検証

各コミット前:
```bash
cargo fmt --check -p leptonica-core
cargo clippy -p leptonica-core -- -D warnings
cargo test -p leptonica-core
```

各PR作成前:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## 対象関数サマリー

| Phase | 関数数 | ブランチ |
|-------|--------|---------|
| 4.2 Numa ops | 6 | `feat/core-numa-ops` |
| 4.1 Histogram/Stats | 7 | `feat/core-stats-histogram` |
| **合計** | **13** | 2 PR |
