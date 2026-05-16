# Phase 1 完全性レポート: C 版回帰テストと Rust 側カバレッジ

Phase 2.5 (個別不一致の修正) に進む前に、`tests/golden_manifest_c.tsv` (Phase 1, PR #377) と Rust 側回帰テスト群が「C 版の回帰テストを過不足なく覆っているか」を整理する。

## 結論

| 項目                                                   |          数 | 評価                                       |
| ------------------------------------------------------ | ----------: | ------------------------------------------ |
| C 版 `prog/*_reg.c` 総数                               |         160 | —                                          |
| ラッパーで除外妥当 (`alltests_reg`)                    |           1 | C 版で他の `_reg` を `system()` で呼ぶだけ |
| `tests/golden_manifest_c.tsv` に hash が入っている     |     **155** | full run で取得                            |
| C 側で **出力ファイルを生成しない assertion 型テスト** |           4 | manifest ハッシュ化対象外 (後述)           |
| **検査対象 159 件のうち hash 取得相当のもの**: 159/159 |     ✅ 100% |                                            |
| Rust 側 `tests/**/*_reg.rs` 総数                       |         159 | C 版と 1:1 対応                            |
| Rust 側カバレッジ (`alltests_reg` を除く)              | **159/159** | ✅ 100%                                    |
| Rust 独自追加テスト                                    |          84 | C 版にない拡張                             |

### Phase 2.5 に進むための前提条件は満たされている

- C 版が出力ファイルを持つテストはすべて `golden_manifest_c.tsv` に取り込み済み
- Rust 側は C 版の全テストに 1:1 対応している
- 残った差分 (Phase 2 で観測された 9 件の Mismatch) は個別調査フェーズへ

## SKIP_REGS の 4 件 ─ なぜハッシュ化不要か

`scripts/gen_c_manifest.sh` の `SKIP_REGS` に登録されている 4 件は、いずれも C 側 `pixEqual()` で **内部一致確認するだけ** のテストで、出力ファイルを書かない。Rust 側も同設計 (`RegParams::compare_values` で内部一致確認、`write_pix_and_check` 不使用) なので、`golden_manifest_c.tsv` / `golden_manifest.tsv` の両方で対象外となるのが正しい挙動。

| _reg             | C 側の出力                                                                     | Rust 側 (`tests/morph/*.rs`)                              | 機能                                              |
| ---------------- | ------------------------------------------------------------------------------ | --------------------------------------------------------- | ------------------------------------------------- |
| `binmorph2_reg`  | なし (`pixEqual` 内部比較)                                                     | `binmorph2_reg.rs` (`compare_values` 内部比較)            | 2-way composite Sel vs unitary Sel の結果一致確認 |
| `dwamorph2_reg`  | `/tmp/lept/morph/timings.png` のみ (タイミング表)                              | `dwamorph2_reg.rs` (5 サブテスト、全て `compare_values`)  | DWA 形態演算タイミング測定                        |
| `fmorphauto_reg` | なし (`pixEqual` 内部比較)                                                     | `fmorphauto_reg.rs` (2 サブテスト、全て `compare_values`) | rasterop vs DWA 自動生成コードの結果一致確認      |
| `morphseq_reg`   | `/tmp/lept/morphseq[1-7].png` (7 個。形態演算 sequence interpreter のデモ出力) | `morphseq_reg.rs` (全て `compare_values`)                 | sequence interpreter の動作確認                   |

### `dwamorph2_reg` / `morphseq_reg` の例外的出力について

- `dwamorph2_reg` の `timings.png` は **タイミング表の可視化** で、テスト一致性を測る対象ではない。Rust 側にも同等の可視化出力はなく、比較する意味がない
- `morphseq_reg` の `/tmp/lept/morphseq[1-7].png` は sequence interpreter の **デモンストレーション出力**。Rust 側は同じ計算結果を `compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0)` で確認しており、画像ファイルとしては書き出さない。manifest に取り込もうとすると Rust 側に対応エントリが無いため `Unmapped` になるだけ

これらは Phase 2 のレポート機構 (`tests/c_compat_report.*.txt`) でも **`MissingC` ではなく `Unmapped` (= そもそも Rust 側が manifest に何も出していない)** として扱われており、整合が取れている。

## Rust 独自テストの位置付け

Rust 側 159 個のうち 84 個 (≒ 53%) は C 版にない独自テストで、主に以下を補強:

- `pix_histogram_advanced` / `numa_advanced` 系の数値プリミティブ
- `pageseg_heavy` 等のページ分割の重ステスト
- `pixa_select_to_pdf` / `io_coverage` 等の I/O 経路網羅
- `recog_coverage` / `recog_helpers` 等の認識系補助

これらは **C 版に対応 reg が存在しない** ため Phase 2 のレポートでは全て `Unmapped`。`scripts/golden_map.tsv` に追加するべき項目もない (C 側に対応物がない)。

## 既存ドキュメントの陳腐化 (別 PR で対応推奨)

調査の過程で次のドキュメントが現状と合致していないことが判明した:

- `docs/porting/test-comparison.md` (2026-05-10): 独自テスト数を「46」と記載 vs 実測 84
- `docs/porting/regression-test-audit.md` (2026-03-01): 対象テスト数を「149」と記載 vs 実測 160
- `docs/porting/regression-test-audit.csv` も同期が取れていない可能性

本 PR では現状の数値のみ本書 (`c-compat-coverage.md`) に記録し、既存ドキュメントの更新は別 PR で取り扱う。

## Phase 2.5 移行に向けて

本書の判定をもって、Phase 2.5 (個別 `Mismatch` の調査・修正) に進むための前提条件 (C 側ハッシュの網羅性 + Rust 側回帰テストのカバレッジ) は満たされたとみなす。

- Phase 2.5 第一弾 ([001-jpeg-codec-diffs.md](c-compat-findings/001-jpeg-codec-diffs.md)): 完了 (PR #379)
- Phase 2.5 第二弾以降: 新たに `Mismatch` / `MissingC` を要因別に切り分け、修正対象を 1 件ずつ PR 化する
