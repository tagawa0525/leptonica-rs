# Phase 1 完全性レポート: C 版回帰テストと Rust 側カバレッジ

Phase 2.5 (個別不一致の修正) に進む前に、`tests/golden_manifest_c.tsv` (Phase 1, PR #377) と Rust 側回帰テスト群が「C 版の回帰テストを過不足なく覆っているか」を整理する。

> **Status**: ⚠️ Phase 1 は **未完成**。C 側 assertion-only テストで `scripts/verify_*.c` 由来のハッシュが 27 件分まだ取り込まれていない (詳細は後述)。Phase 2.5 (個別修正) に進む前に、本書末尾の「次のアクション」で示す手順で C 側ハッシュを 100% にする必要がある。

## 現状の数値

| 項目                                                                               |                   数 | 評価                                        |
| ---------------------------------------------------------------------------------- | -------------------: | ------------------------------------------- |
| C 版 `prog/*_reg.c` 総数                                                           |                  160 | —                                           |
| ラッパーで除外妥当 (`alltests_reg`)                                                |                    1 | C 版で他の `_reg` を `system()` で呼ぶだけ  |
| `prog/*_reg` で出力ファイルを書き、`golden_manifest_c.tsv` に hash 登録済み        |                  155 | full run (Phase 1, PR #377) で取得          |
| `prog/*_reg` が出力ファイルを書かない assertion-only テスト (`SKIP_REGS` 4 件 + α) |                 多数 | 後述                                        |
| そのうち `scripts/verify_*.c` で C 出力が取得可能なケース                          | 27 (= MissingC 件数) | 取得仕組み未実装                            |
| Rust 側 `tests/**/*_reg.rs` 総数                                                   |                  159 | C 版と 1:1 対応 (`alltests_reg` のみ未移植) |
| Rust 独自追加テスト                                                                |                   84 | C 版にない拡張                              |

### 「100% カバー」と言える条件

| 観点                                          | 現状                                                                                            | 目標                                         |
| --------------------------------------------- | ----------------------------------------------------------------------------------------------- | -------------------------------------------- |
| C 側 hash 取得率 (manifest_c.tsv)             | 1871 entries / 完全数 1898 entries (現状 155 + 取得すべき 27 = 182 件の reg に相当) ≒ **98.6%** | 100% (verify_*.c 由来 27 件の取り込みが必要) |
| Rust 側回帰テストカバレッジ (`alltests` 除く) | 159/159 = 100%                                                                                  | 既達 ✅                                      |
| Phase 2 レポートでの `MissingC` 件数          | 27                                                                                              | 0 (verify_*.c 取り込み後)                    |

## C 側 assertion-only テストの全体像

C 版 `prog/*_reg.c` には `regTestWritePixAndCheck` を呼ばず `pixEqual()` や `compare_pix()` で内部一致確認するだけのテストが存在する。これらは出力ファイルを書かないため `golden_manifest_c.tsv` には載らない。

### (A) 完全に出力ファイルを書かないもの — manifest 化対象外

`scripts/gen_c_manifest.sh::SKIP_REGS` に登録されている 4 件:

| _reg             | C 側の挙動                                                      | Rust 側 (`tests/morph/*.rs`)                         |
| ---------------- | --------------------------------------------------------------- | ---------------------------------------------------- |
| `binmorph2_reg`  | なし (`pixEqual` 内部比較)                                      | `binmorph2_reg.rs` (`compare_values` 内部比較)       |
| `dwamorph2_reg`  | `/tmp/lept/morph/timings.png` のみ (タイミング表)               | `dwamorph2_reg.rs` (5 サブテスト、`compare_values`)  |
| `fmorphauto_reg` | なし (`pixEqual` 内部比較)                                      | `fmorphauto_reg.rs` (2 サブテスト、`compare_values`) |
| `morphseq_reg`   | `/tmp/lept/morphseq[1-7].png` (sequence interpreter のデモ出力) | `morphseq_reg.rs` (`compare_values`)                 |

**注意 (Copilot レビューを反映)**: 「Rust 側に同等のテストがある」と書いてはいるが、各 Rust テストは C 版実装の一部しかカバーしていない。たとえば `dwamorph2_reg.rs` は smaller size range だけ、`fmorphauto_reg.rs` は auto-generated function 比較を省略、`morphseq_reg.rs` は R/X/DWA 等の一部 sequence syntax をサポート外として除外している。**機能対応の質は同等ではない**。これは本書のスコープを超えるが、Phase 3 以降のテスト拡張で個別に詰める必要がある。

### (B) 出力ファイルを書かないが、`scripts/verify_*.c` で C 側中間結果を取得できるもの

このカテゴリは PR #377 時点で見落としていた。`scripts/golden_map.tsv` のコメントを再確認すると、以下の reg は C 側で `regTestWritePixAndCheck` を呼ばないが、Rust 側は `write_pix_and_check` で manifest に出力しており、ペアになる C ハッシュを `scripts/verify_*.c` (検証用に別途用意された C プログラム) から取得することが想定されている:

| C verify プログラム                               | Rust 側出力 (manifest_c.tsv の MissingC 件数) | C 出力先                                         |
| ------------------------------------------------- | --------------------------------------------: | ------------------------------------------------ |
| `scripts/verify_binmorph.c` (binmorph1_reg 系)    |                                             4 | `/tmp/c_binmorph1_{dilate,erode,open,close}.tif` |
| `scripts/verify_binmorph.c` (binmorph3_reg 系)    |                                             3 | `/tmp/c_binmorph3_*.tif`                         |
| `scripts/verify_fhmtauto.c` (fhmtauto_reg 系)     |                                             8 | `/tmp/c_fhmtauto_{NN,id}.tif`                    |
| `scripts/verify_graymorph2.c` (graymorph2_reg 系) |                                            12 | `/tmp/c_graymorph2_*.jpg`                        |
| **合計**                                          |                                        **27** |                                                  |

これらは現状 `scripts/gen_c_manifest.sh` の実行対象に **入っていない** ため、Phase 2 のレポートで `MissingC` (golden_map にエントリはあるが C manifest にエントリがない) として現れている。`gen_c_manifest.sh` が `prog/build/bin/*_reg` しか実行しないことに起因する。

`scripts/verify_*.c` は 10 ファイルあり、上記 4 件以外にもバックグラウンド処理系 (`verify_apply_inv_bg`, `verify_bg_gray_map`, `verify_bg_rgb_map`, `verify_inv_bg_map`, `verify_contrast_norm`, `verify_fillmapholes`, `verify_findbaselines`) があるが、これらは `golden_map.tsv` に対応するマッピングがまだ登録されていない (現状 `Unmapped` 扱い)。

## 結論 (修正版)

1. **prog/\*_reg の hash 取得**: 155 件取得済み (assertion-only な SKIP_REGS 4 件は出力ファイル無しのため対象外で正しい)
2. **scripts/verify_\*.c の hash 取得**: **未実装**。27 件の `MissingC` がここに該当する
3. **Rust 側カバレッジ**: 159/159 ファイルで対応はある (alltests_reg のみ未移植)。ただし「ファイルがある = 機能カバー 100%」ではなく、上記 (A) で挙げたように一部の C 機能はスキップされている

**Phase 2.5 の細かな個別修正に進む前に、上記 (2) の verify_\*.c 取り込みを完了させる必要がある。** これにより MissingC 27 件が消えて、純粋に `Mismatch` (= Rust と C で結果が異なる) のみが残り、修正対象を絞り込める。

## 次のアクション

### 必須 (Phase 1 を真の意味で完了させる)

1. `scripts/build_c_verify.sh` (新規) で `scripts/verify_*.c` をコンパイル
2. `scripts/gen_c_manifest.sh` を拡張して verify バイナリを実行
3. `examples/gen_c_manifest` を拡張して `/tmp/c_*.tif` や `/tmp/c_*.jpg` も拾い、`golden_map.tsv` の合成 prefix (`binmorph1_verify.NN.tif` 等) と対応付ける
4. `tests/golden_manifest_c.tsv` を再生成して 27 件のエントリを追加 (1871 → 1898)
5. `cargo test --all-features` で `MissingC` が 0 件になることを確認

これは独立した PR (Phase 1.5 完全化 PR) として実施するのが筋。本 PR (`docs/c-compat-coverage`) はあくまで **現状把握** に留める。

### 任意 (Phase 2.5 以降に逐次)

- (B) で残った verify_*.c (bg系、contrast_norm 等) について `golden_map.tsv` にマッピング追加。これは Phase 2.5 で Rust 側修正と並行して進める
- (A) の Rust 側カバー漏れ (dwamorph2 の size range、fmorphauto の auto-gen、morphseq の不対応 syntax 等) を個別 Rust テストとして拡充

## 既存ドキュメントの陳腐化 (別 PR で対応推奨)

調査の過程で次のドキュメントが現状と合致していないことが判明した:

- `docs/porting/test-comparison.md` (2026-05-10): 独自テスト数を「46」と記載 vs 実測 84
- `docs/porting/regression-test-audit.md` (2026-03-01): 対象テスト数を「149」と記載 vs 実測 160
- `docs/porting/regression-test-audit.csv` も同期が取れていない可能性

本 PR では現状の数値のみ本書に記録し、既存ドキュメントの更新は別 PR で取り扱う。

## 関連

- Phase 2.5 第一弾 ([001-jpeg-codec-diffs.md](c-compat-findings/001-jpeg-codec-diffs.md)): 完了 (PR #379)
- Phase 1 完全化 PR: 未着手 (本書「次のアクション (必須)」の実装)
