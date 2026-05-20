# C 互換性ベースライン (Phase 3)

Phase 1 / Phase 1.5 / Phase 2 / Phase 2.5 (一連の PR #377〜#389) で確立した
C 版 leptonica との互換性検証の **現在地** を整理する。`cargo test
--all-features` 1 回で `tests/c_compat_report.*.txt` (8 個の test binary
別ファイル) に集計が出力される仕組みは Phase 2 (PR #378) で導入済み。

> **As of 2026-05-20** (PR #389 merge 後)。

## 全体集計

| 状態        |    件数 | 説明                                              |
| ----------- | ------: | ------------------------------------------------- |
| ✅ Ok       |  **22** | C 版と pixel-level 完全一致                       |
| ⚠️ Mismatch |  **31** | C と Rust で hash 不一致 (内訳は後述)             |
| ⛔ MissingC |   **0** | (PR #381 / Phase 1.5 で解消)                      |
| 📭 Unmapped | **520** | `scripts/golden_map.tsv` 未登録 (Phase 3+ で整理) |

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
| `morph`     | **20** |   **26** |        0 |        9 |
| `recog`     |      0 |        0 |        0 |       45 |
| `region`    |      0 |        0 |        0 |       78 |
| `transform` |      0 |        0 |        0 |       82 |

**morph** が現状最も Ok/Mismatch が集中している binary。これは:

- 1bpp morph (binmorph, fhmtauto, ccthin) が C との pixel 一致を意図して

  golden_map.tsv に登録されているため

- Phase 2.5 で重点的に修正を進めた領域

## Ok 22 件の内訳

C 版と完全一致している領域:

| カテゴリ                              | 件数 | 由来                                                                          |
| ------------------------------------- | ---: | ----------------------------------------------------------------------------- |
| `cthin1_thin` (4) + `cthin2_set` (11) |   15 | `pixThinConnected` / `pixThinConnectedBySet` (1bpp 細線化)                    |
| `compfilter_write_synthetic`          |    2 | `pixFillClosedBorders` 等 (合成画像)                                          |
| `binmorph1`                           |    3 | `pixDilate/Erode/Open*CompBrick(21,15)` (PR #389 で Mismatch 4→1、3 件 Ok 化) |
| `binmorph3`                           |    1 | `pixDilate*CompBrick(21,1)`                                                   |
| `fhmtauto_id`                         |    1 | Identity 1x1 HMT (PR #389 で解消)                                             |

PR #389 の TIFF invert bug 修正で **5 件追加で Ok 化**。

## Mismatch 31 件の内訳

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

### 修正対象 (Rust 実装の C 互換性改善): 10 件

| カテゴリ                          | 件数 | finding                                                   | 修正方針                                                                                                                                                                              |
| --------------------------------- | ---: | --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fhmtauto_hmt` (sel_4_*, sel_8_*) |    7 | [004](c-compat-findings/004-hmt-impl-diff.md)             | 2026-05-20 実験: Clear near edges 追加では効果なし (Rust 出力 hash 不変)。root cause は SEL 生成差または bit-shift 内部実装にあり、追加調査が必要 (004 finding 末尾「Next step」参照) |
| `binmorph3` (sep/dir dilate)      |    2 | [003](c-compat-findings/003-morph-brick-comp-vs-plain.md) | `dilate_1d_composite` の factor 選択 / SEL origin / 境界処理を C `pixDilateCompBrick` 内部と pixel-level で突き合わせ                                                                 |
| `binmorph1` (close 21x15)         |    1 | [003](c-compat-findings/003-morph-brick-comp-vs-plain.md) | 同上 (close は `pixCloseCompBrick` の padding 差)                                                                                                                                     |

## 解消した発見の系譜 (Phase 2.5 の調査履歴)

| Finding                                                                                       | 解消状況                                                                                                            |
| --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| [001 JPEG codec diff](c-compat-findings/001-jpeg-codec-diffs.md)                              | 仮説段階 (修正対象外、21 件カバー)                                                                                  |
| [002 TIFF 1bpp write limit](c-compat-findings/002-tiff-1bpp-write-limit.md)                   | ✅ 実装完了 (PR #383)、bps=1 で書けるように                                                                         |
| [003 brick comp vs plain](c-compat-findings/003-morph-brick-comp-vs-plain.md)                 | 部分対応 (PR #385 で verify を Comp 化、`composite 細部差` 残課題)                                                  |
| [004 HMT impl diff](c-compat-findings/004-hmt-impl-diff.md)                                   | 部分対応 (Identity は 005 真因で解消、`fhmtauto_hmt` 7 件は Clear near edges 追加でも不変と判明 → SEL 生成差調査へ) |
| [005 TIFF 1bpp photometric invert](c-compat-findings/005-tiff-1bpp-photometric-invert-bug.md) | ✅ 実装完了 (PR #389)、5 件 Ok 化                                                                                   |

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

### Phase 2.5 残作業 (Rust 修正対象 10 件)

1. **004 finding 続き**: 2026-05-20 実験で Clear near edges 追加は不変と判明。次は `make_thin_sels(Set4cc1/Set8cc1)` の SEL 内容を C `selaAddHitMiss` と byte-level 比較し、SEL 生成差の有無を切り分け → 切り分け後の方針で fhmtauto_hmt 7 件解消を目指す
2. **003 finding 続き**: `src/morph/binary.rs::dilate_1d_composite` の pixel-level audit → binmorph 3 件解消見込み

両方完了で Mismatch 31 → 21 (JPEG codec 差のみが残る) になる見込み。ただし 004 は当初想定より深い調査が必要なため、複数 PR に分割される可能性がある。

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
