# C 互換性ベースライン (Phase 3)

Phase 1 / Phase 1.5 / Phase 2 / Phase 2.5 / Phase 3 (一連の PR #377〜) で
確立した C 版 leptonica との互換性検証の **現在地** を整理する。
`cargo test --all-features` 1 回で `tests/c_compat_report.*.txt`
(8 個の test binary 別ファイル) に集計が出力される仕組みは Phase 2
(PR #378) で導入済み。

> **As of 2026-05-20** (Phase 3 第三弾 merge 後)。Phase 2.5 修正対象
> 10 件すべて解消 (PR #397/#398/#400)。Phase 3 第一弾で hash-match
> ペア 8 件追加、第二弾で region/seedspread の semantic ペア 6 件 (新
> finding 006 で可視化)、第三弾で gifio Part 1 の残 6 件 (4 Ok + 2
> Mismatch、新 finding 007 で可視化) を追加。全体 Ok の推移は
> **22 (Phase 2.5 開始時) → 32 (Phase 2.5 完了時) → 40 (第一弾完了時)
> → 40 (第二弾、Mismatch +6) → 44 (第三弾、+4 Ok / +2 Mismatch)**。
> Mismatch は 21 → 29 (新規 8 件は finding 006/007 で root cause 調査
> 中、既存 21 件は JPEG codec 差で finding 001)。
>
> 2026-07-16 実測 (v0.5.0 リリース時、Phase 3 第四弾 PR #405 後):
> 集計は変化なし (Ok 44 / Mismatch 29 / MissingC 0 / Unmapped 500)。
>
> **plan 902 (Excluded 導入) 後**: 設計上マップ不能な分
> (jpg/jpeg 45 件 = finding 001、pdf/ps 8 件 = PR #386) を
> `scripts/c_compat_exclude.tsv` の除外ルールで `Excluded` に分離。
> Unmapped は **447** となり「マップ可能な未着手」だけを数える。

## 全体集計

| 状態        |    件数 | 説明                                                                                                                          |
| ----------- | ------: | ----------------------------------------------------------------------------------------------------------------------------- |
| ✅ Ok       |  **44** | C 版と pixel-level 完全一致 (Phase 2.5 で +10、Phase 3 で +12)                                                                |
| ⚠️ Mismatch |  **29** | 内訳: 既知の JPEG codec 差 21 件 (finding 001) + seedspread 6 件 (finding 006) + gifio 2 件 (finding 007、第三弾で新規可視化) |
| ⛔ MissingC |   **0** | (PR #381 / Phase 1.5 で解消)                                                                                                  |
| 📭 Unmapped | **447** | `scripts/golden_map.tsv` 未登録かつマップ可能 (Phase 3 進行中、520 → 500 → 447)                                               |
| 🚫 Excluded |  **53** | 設計上マップ不能 (`scripts/c_compat_exclude.tsv`)。jpg/jpeg 45 件 (finding 001) + pdf/ps 8 件 (PR #386)                       |

合計 573 entries が C 比較対象。Rust manifest (`tests/golden_manifest.tsv`)
全体は **580 entries** (582 行 - ヘッダ 2 行)。加えて Rust 独自テスト 84
件 (C 版に対応なし)。

## test binary 別の内訳

| Binary      |     Ok | Mismatch | MissingC | Unmapped | Excluded |
| ----------- | -----: | -------: | -------: | -------: | -------: |
| `color`     |      0 |        0 |        0 |      114 |        0 |
| `core`      |      1 |        0 |        0 |       34 |        0 |
| `filter`    |      2 |        5 |        0 |       57 |       40 |
| `io`        |      7 |        2 |        0 |       41 |       10 |
| `morph`     | **30** |   **16** |        0 |        9 |        0 |
| `recog`     |      0 |        0 |        0 |       45 |        0 |
| `region`    |      0 |        6 |        0 |       69 |        3 |
| `transform` |      4 |        0 |        0 |       78 |        0 |

**morph** が現状最も Ok/Mismatch が集中している binary。これは:

- 1bpp morph (binmorph, fhmtauto, ccthin) が C との pixel 一致を意図して

  golden_map.tsv に登録されているため

- Phase 2.5 で重点的に修正を進めた領域

## Ok 44 件の内訳

C 版と完全一致している領域:

| カテゴリ                              | 件数 | 由来                                                                                         |
| ------------------------------------- | ---: | -------------------------------------------------------------------------------------------- |
| `cthin1_thin` (4) + `cthin2_set` (11) |   15 | `pixThinConnected` / `pixThinConnectedBySet` (1bpp 細線化)                                   |
| `compfilter_write_synthetic`          |    2 | `pixFillClosedBorders` 等 (合成画像)                                                         |
| `binmorph1`                           |    4 | `pixDilate/Erode/Open/CloseCompBrick(21,15)` (PR #389 で 3 件 + PR #398 で `close` 1 件追加) |
| `binmorph3`                           |    3 | `pixDilateCompBrick(21,1)` 1 件 + `(11,7)` sep/dir 2 件 (PR #400 で追加)                     |
| `fhmtauto_id`                         |    1 | Identity 1x1 HMT (PR #389 で解消)                                                            |
| `fhmtauto_hmt` (sel_4_*, sel_8_*)     |    7 | PR #397 で thinning SEL 中心 `'C'` を DontCare に修正し解消                                  |
| `expand_{1,2,4,8}bpp`                 |    4 | Phase 3 第一弾: `expand_replicate` 2× (1/2/4/8bpp、transform binary 初登場)                  |
| `gifio`                               |    6 | Phase 3 第一弾 2 件 + 第三弾 4 件 (Part 1 lossless r/w 6 ファイル分、io binary)              |
| `iomisc_regen_rgb_cmap`               |    1 | Phase 3 第一弾: 8bpp colormap → 32bpp RGB 復元 (io binary)                                   |
| `logicops_invert`                     |    1 | Phase 3 第一弾: `pixInvert` 二値反転 (core binary 初登場)                                    |

通算で **22 件追加 Ok 化** (Phase 2.5 で 10 件、Phase 3 第一弾で 8 件、
第三弾で 4 件)。Phase 3 で **core / io / transform / region** の 4 新
binary が C 比較対象に組み込まれた。

## Mismatch 29 件の内訳

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

### Phase 3 で可視化 (8 件、要追加調査)

| カテゴリ     | 件数 | 根拠                                                                                                                    |
| ------------ | ---: | ----------------------------------------------------------------------------------------------------------------------- |
| `seedspread` |    6 | [006](c-compat-findings/006-seedspread-output-diff.md): 仮説段階 (第二弾で可視化、要切り分け)                           |
| `gifio`      |    2 | [007](c-compat-findings/007-gifio-quantization-diff.md): FILE_8BPP_3 / FILE_32BPP の GIF round-trip 差 (第三弾で可視化) |

Phase 3 で `golden_map.tsv` の Unmapped から semantic マッピングを
追加し、それまで隠れていた出力差を Mismatch として明示。各々 finding
ドキュメントで仮説を列挙、別 PR で root cause 切り分け予定。

### Phase 2.5 修正対象: **0 件 (✅ 完全解消)**

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
| [006 seedspread output diff](c-compat-findings/006-seedspread-output-diff.md)                 | 仮説段階 (Phase 3 第二弾で可視化、6 件カバー)、root cause 切り分け予定                                                                                                                    |
| [007 gifio quantization diff](c-compat-findings/007-gifio-quantization-diff.md)               | 仮説段階 (Phase 3 第三弾で可視化、2 件カバー — FILE_8BPP_3 / FILE_32BPP)、root cause 切り分け予定                                                                                         |

## Unmapped 500 件について

`scripts/golden_map.tsv` に C↔Rust の対応マッピングが登録されていない
テスト。これは「Phase 1 / 1.5 で C 出力は取得済みだが、対応する Rust 出力
との pairing 情報がない」状態。Phase 3 で段階的に拡張中
(2026-05-20 第一弾で 520 → 512、第二弾で 512 → 506、第三弾で 506 → 500)。

優先度の高い拡張候補 (現状 Unmapped が多い領域):

- `color` (114), `filter` (97), `transform` (78), `region` (72), `io` (51)

これらの大半は Rust 独自テスト (C に対応なし) も含むため、実質マッピング
可能な件数はもっと少ない可能性。

## 次のアクション

### Phase 2.5: ✅ 完了 (2026-05-20、PR #400 まで)

修正対象 10 件すべて解消。残 21 件は finding 001 で JPEG codec 差と
判定済みで、Rust 実装のアルゴリズムは正しいと推定 (Rust JPEG エンコーダ
の出力 byte 差が hash に反映されているのみ)。

### Phase 3 残作業 (本書整理 + golden_map 拡充)

Phase 3 第一弾 (2026-05-20): `scripts/phase3_find_hash_match_pairs.py`
で one-to-one hash-match pair 8 件を抽出し `scripts/golden_map.tsv` に
追加。core / io / transform の 3 binary が C 比較対象に初登場。

Phase 3 第二弾 (2026-05-20): `region/seedspread` の semantic-mapped
pair 6 件を `golden_map.tsv` に追加し、Rust と C で出力 pixel-level
が異なる差を **Mismatch として可視化** (finding 006 で root cause
切り分け予定)。region binary が C 比較対象に初登場。

Phase 3 第三弾 (2026-05-20): `io/gifio` Part 1 mapping を完成
(第一弾の 2 件 + 第三弾の 6 件 = 計 8 件、うち 6 件 Ok / 2 件
Mismatch)。Mismatch 2 件 (`FILE_8BPP_3` / `FILE_32BPP`) は finding 007
で GIF round-trip 差として可視化。

Phase 3 第四弾以降:

- 残 Unmapped 500 件のうち、Rust と C の `*_reg.{rs,c}` の write 順序

  が semantic 一致する prefix を 1 module ずつ追加 (現状の優先順位:
  io 残り → transform → filter → region 残り → color)

- finding 006 / 007 の root cause 切り分け (それぞれ別 PR で対応)
- 各追加で Mismatch が増えたら新 finding として記録

### Phase 4: CI 統合 (PR #391 で実装)

- ✅ `tests/c_compat_report.*.txt` を GitHub Actions の artifact として保存 (retention: 14 日)
- ✅ workflow の Job Summary に Ok / Mismatch / MissingC / Unmapped の集計テーブル (全体 + binary 別) を出力
- PR 本文への自動コメント投稿は将来の拡張対象

## 関連

- [plan 901: C版ハッシュとの互換性検証](../plans/901_c-hash-compat.md)
- [Phase 1 完全性レポート](c-compat-coverage.md) (PR #380)
- Phase 2.5 findings (001-005)、Phase 3 findings 006-007
- `examples/compare_golden.rs` (pixel-level 詳細比較ツール)
- `examples/gen_c_manifest.rs` (C 出力からの hash 生成ツール)
- `tests/common/c_compat.rs` (Phase 2 レポート機構)
