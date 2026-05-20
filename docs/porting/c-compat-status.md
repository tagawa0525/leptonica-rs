# C 互換性ベースライン (Phase 3)

Phase 1 / Phase 1.5 / Phase 2 / Phase 2.5 (一連の PR #377〜#400) で確立した
C 版 leptonica との互換性検証の **現在地** を整理する。`cargo test
--all-features` 1 回で `tests/c_compat_report.*.txt` (8 個の test binary
別ファイル) に集計が出力される仕組みは Phase 2 (PR #378) で導入済み。

> **As of 2026-05-20** (PR #400 merge 後)。**Phase 2.5 修正対象
> 10 件すべて解消 (100%)** — fhmtauto_hmt 7 件 (PR #397)、binmorph1.12
> close 1 件 (PR #398)、binmorph3.14/15 dilate 2 件 (PR #400)。
> 全体 Mismatch は 31 → 21 (10 件減、残 21 件は既知の JPEG codec 差で
> 修正対象外、finding 001 参照)。

## 全体集計

| 状態        |    件数 | 説明                                                       |
| ----------- | ------: | ---------------------------------------------------------- |
| ✅ Ok       |  **32** | C 版と pixel-level 完全一致 (10 件追加、PR #397/#398/#400) |
| ⚠️ Mismatch |  **21** | C と Rust で hash 不一致 (全件 JPEG codec 差、修正対象外)  |
| ⛔ MissingC |   **0** | (PR #381 / Phase 1.5 で解消)                               |
| 📭 Unmapped | **520** | `scripts/golden_map.tsv` 未登録 (Phase 3+ で整理)          |

合計 573 entries が C 比較対象。Rust manifest (`tests/golden_manifest.tsv`)
全体は **580 entries** (582 行 - ヘッダ 2 行)。加えて Rust 独自テスト 84
件 (C 版に対応なし)。

## test binary 別の内訳

| Binary      |     Ok | Mismatch | MissingC | Unmapped |
| ----------- | -----: | -------: | -------: | -------: |
| `color`     |      0 |        0 |        0 |      114 |
| `core`      |      0 |        0 |        0 |       35 |
| `filter`    |      2 |        5 |        0 |       97 |
| `io`        |      0 |        0 |        0 |       60 |
| `morph`     | **30** |   **16** |        0 |        9 |
| `recog`     |      0 |        0 |        0 |       45 |
| `region`    |      0 |        0 |        0 |       78 |
| `transform` |      0 |        0 |        0 |       82 |

**morph** が現状最も Ok/Mismatch が集中している binary。これは:

- 1bpp morph (binmorph, fhmtauto, ccthin) が C との pixel 一致を意図して

  golden_map.tsv に登録されているため

- Phase 2.5 で重点的に修正を進めた領域

## Ok 32 件の内訳

C 版と完全一致している領域:

| カテゴリ                              | 件数 | 由来                                                                                         |
| ------------------------------------- | ---: | -------------------------------------------------------------------------------------------- |
| `cthin1_thin` (4) + `cthin2_set` (11) |   15 | `pixThinConnected` / `pixThinConnectedBySet` (1bpp 細線化)                                   |
| `compfilter_write_synthetic`          |    2 | `pixFillClosedBorders` 等 (合成画像)                                                         |
| `binmorph1`                           |    4 | `pixDilate/Erode/Open/CloseCompBrick(21,15)` (PR #389 で 3 件 + PR #398 で `close` 1 件追加) |
| `binmorph3`                           |    3 | `pixDilateCompBrick(21,1)` 1 件 + `(11,7)` sep/dir 2 件 (PR #400 で追加)                     |
| `fhmtauto_id`                         |    1 | Identity 1x1 HMT (PR #389 で解消)                                                            |
| `fhmtauto_hmt` (sel_4_*, sel_8_*)     |    7 | PR #397 で thinning SEL 中心 `'C'` を DontCare に修正し解消                                  |

通算で **10 件追加 Ok 化** (PR #397 で fhmtauto 7、PR #398 で binmorph1.12 1、PR #400 で binmorph3.14/15 2)。

## Mismatch 21 件の内訳

### 修正対象外: 既知の JPEG codec 差 (21 件)

| カテゴリ                                      | 件数 | 根拠                                                        |
| --------------------------------------------- | ---: | ----------------------------------------------------------- |
| `gmorph2_dilate_erode` + `gmorph2_open_close` |   12 | graymorph2 (8bpp JPEG quality=75)                           |
| `edge`                                        |    4 | edge_reg (8bpp JPEG)                                        |
| `colormorph`                                  |    4 | dilate_color/erode_color/open_color/close_color (8bpp JPEG) |
| `convolve_blockconv_gray`                     |    1 | blockconv_gray (8bpp JPEG)                                  |

すべて `docs/porting/c-compat-findings/001-jpeg-codec-diffs.md` で
**JPEG codec 差** (libjpeg-turbo vs jpeg-decoder/jpeg-encoder) と仮説判定
済み。Rust 実装のアルゴリズムは正しいと推定 (確定検証は別途)。

### 修正対象 (Rust 実装の C 互換性改善): **0 件 (✅ 完全解消)**

Phase 2.5 開始時の修正対象 10 件はすべて Ok 化済み。詳細は finding
003 / 004 を参照。

## 解消した発見の系譜 (Phase 2.5 の調査履歴)

| Finding                                                                                       | 解消状況                                                                                                                                                                                  |
| --------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [001 JPEG codec diff](c-compat-findings/001-jpeg-codec-diffs.md)                              | 仮説段階 (修正対象外、21 件カバー)                                                                                                                                                        |
| [002 TIFF 1bpp write limit](c-compat-findings/002-tiff-1bpp-write-limit.md)                   | ✅ 実装完了 (PR #383)、bps=1 で書けるように                                                                                                                                               |
| [003 brick comp vs plain](c-compat-findings/003-morph-brick-comp-vs-plain.md)                 | ✅ 解消 (PR #385 で verify を Comp 化、PR #398 で `pix*CompBrick` 準拠書き直し + `binmorph1.12 close` Ok 化、PR #400 で `dilate_rasterop` 方向修正 + `binmorph3.14/15 dilate 11x7` Ok 化) |
| [004 HMT impl diff](c-compat-findings/004-hmt-impl-diff.md)                                   | ✅ 解消 (PR #397 で thinning SEL 中心 `'C'` を C 準拠で DontCare に修正、`fhmtauto_hmt` 7 件すべて Ok 化)                                                                                 |
| [005 TIFF 1bpp photometric invert](c-compat-findings/005-tiff-1bpp-photometric-invert-bug.md) | ✅ 実装完了 (PR #389)、5 件 Ok 化                                                                                                                                                         |

## Unmapped 520 件について

`scripts/golden_map.tsv` に C↔Rust の対応マッピングが登録されていない
テスト。これは「Phase 1 / 1.5 で C 出力は取得済みだが、対応する Rust 出力
との pairing 情報がない」状態。本書のスコープ外で、Phase 3+ (golden_map
拡充フェーズ) で個別に拡張する。

優先度の高い拡張候補 (現状 Unmapped が多い領域):

- `color` (114), `transform` (82), `region` (78), `filter` (97), `io` (60)

これらの大半は Rust 独自テスト (C に対応なし) も含むため、実質マッピング
可能な件数はもっと少ない可能性。

## 次のアクション

### Phase 2.5: ✅ 完了 (2026-05-20、PR #400 まで)

修正対象 10 件すべて解消。残 21 件は finding 001 で JPEG codec 差と
判定済みで、Rust 実装のアルゴリズムは正しいと推定 (Rust JPEG エンコーダ
の出力 byte 差が hash に反映されているのみ)。

### Phase 3 残作業 (本書整理 + golden_map 拡充)

- 本書の継続更新 (Mismatch 数が変わるたびに表を refresh)
- `scripts/golden_map.tsv` に未登録のテスト群 (Unmapped 520 件) のうち、

  実際に C と pair 可能なものを段階的に登録

### Phase 4: CI 統合 (PR #391 で実装)

- ✅ `tests/c_compat_report.*.txt` を GitHub Actions の artifact として保存 (retention: 14 日)
- ✅ workflow の Job Summary に Ok / Mismatch / MissingC / Unmapped の集計テーブル (全体 + binary 別) を出力
- PR 本文への自動コメント投稿は将来の拡張対象

## 関連

- [plan 901: C版ハッシュとの互換性検証](../plans/901_c-hash-compat.md)
- [Phase 1 完全性レポート](c-compat-coverage.md) (PR #380)
- Phase 2.5 findings (001-005)
- `examples/compare_golden.rs` (pixel-level 詳細比較ツール)
- `examples/gen_c_manifest.rs` (C 出力からの hash 生成ツール)
- `tests/common/c_compat.rs` (Phase 2 レポート機構)
