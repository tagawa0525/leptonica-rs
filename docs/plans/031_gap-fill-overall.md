# Gap-Fill 全体計画（C版未移植機能の補完）

Status: IMPLEMENTED
作成日: 2026-05-09
完了日: 2026-05-10

## Context

`docs/porting/feature-comparison.md` 上は実カバレッジ 100%（✅ 同等 + 🔄 異なる）だが、
ソースを `pub fn` で grep して照合した結果、以下の C 版関数は対応する Rust 実装が
存在しない。本計画はこれら「真の溝」を埋めるためのロードマップである。

検証手順:

1. 各候補を `rg "pub fn <snake_case>" src/` および同義語で網羅検索
2. 対応モジュールが存在しなければ「未実装」と確定
3. テストの `#[ignore]` メッセージは古いものが多いため鵜呑みにしない

## 結果サマリ

全 12 項目（A〜L）の方針確定。内訳は実装 8 件、部分実装 1 件、HOLD 2 件、
修正 1 件。Ignored test を 19 件解除（core 31→22 / transform 13→12 /
region 1→0 / recog 13→2 / io 6→4 + 37→36）。

| # | カテゴリ              | C 版関数                                                                    | サブ計画 | PR   | 結果                                                                       |
| - | --------------------- | --------------------------------------------------------------------------- | -------- | ---- | -------------------------------------------------------------------------- |
| A | core/compare          | `pixBestCorrelation`, `pixCompareWithTranslation`                           | 102      | #317 | ✅ IMPLEMENTED                                                             |
| B | core/fpix             | `fpixRotateOrth` 等 8 関数                                                  | 103      | #316 | ✅ IMPLEMENTED                                                             |
| C | core/boxa             | `boxaaTranspose`                                                            | 104      | #315 | ✅ 取りこぼし発覚、テスト追加のみで完了                                    |
| D | core/sarray           | sarray I/O 6 関数 + `arrayFindSequence`                                     | 105      | #320 | ✅ IMPLEMENTED（I/O は実装済み取りこぼし、`array_find_sequence` のみ新規） |
| E | io/jpeg               | `pixSetChromaSampling`                                                      | 202      | #319 | ✅ IMPLEMENTED                                                             |
| F | io/jp2k               | scaled / cropped read + write                                               | 203      | #325 | 🟡 PARTIAL（read のみ実装、write は HOLD: 純 Rust encoder 不在）           |
| G | io/pdf                | `concatenatePdf` 系 4 関数                                                  | 204      | #324 | 🟡 HOLD（Leptonica 専用 PDF 構造前提、汎用 PDF 連結は外部依存追加が必要）  |
| H | transform/affine      | `pixAffineSequential`                                                       | 301      | #318 | ✅ IMPLEMENTED                                                             |
| I | morph 自動生成        | `pixFMorphopGen_1`, `pixFHMTGen_1`, `pixHMTDwa_1`                           | 401      | #326 | 🟡 HOLD（既存 brick / DWA で代替可、性能要求が出たら再検討）               |
| J | region/ccbord         | `get_all_borders` の OOM                                                    | 701      | #323 | ✅ FIXED（実態はトレーサ無限ループ、計算量問題ではなかった）               |
| K | recog/pageseg 矩形    | `pixFindLargestRectangle`, `pixFindRectangleInCC`, `pixFindLargeRectangles` | 801      | #321 | ✅ IMPLEMENTED                                                             |
| L | recog/pageseg 表/反転 | `pixDecideIfTable`, `pixAutoPhotoinvert`                                    | 802      | #322 | ✅ IMPLEMENTED                                                             |

## 不要分類（実装しない）

- `L_ASET`（順序集合）, `L_HASHMAP` — `BTreeSet`/`HashMap` で代替可
- `sarrayFindStringByHash` — 利用箇所がなく `find_string` で代替可（必要時のみ実装）

## 学んだこと

1. **`#[ignore]` のメッセージは情報源として弱い**: Boxaa::transpose / sarray I/O

   など、本当は実装済みなのに古い ignore メッセージで未実装と分類されていた
   ケースが複数あった。検索は `pub fn boxaa_transpose` のような prefix 形式
   だけでなく `fn transpose` のようなメソッド形式でも grep する必要がある

2. **OOM や「メモリ複雑度」も鵜呑みにしない**: J (ccbord) は計算量問題と

   分類されていたが、release ビルドのバックトレースを取って初めて
   「無限ループによる 4 GiB allocation」が真因と判明した

3. **HOLD 判断もドキュメント成果物**: G (PDF 連結) と I (morph 自動生成) は

   実装よりも「なぜ実装しないか」を計画書として残すことに価値がある

4. **Copilot レビューはエッジケースに強い**: overflow / off-by-one / 早期

   バリデーション / API consistency など、実装中に見落としがちな観点を
   多くキャッチしてくれた

## グルーピングと進行（参考: 当初計画）

実装規模・依存関係から 3 グループに分類して順次進めた。

### Group 1: 小規模・独立 — A/B/C/E/H 全実装

### Group 2: 中規模・横断 — D/K/L 全実装

### Group 3: 大規模・外部依存 — J 修正、F 部分実装、G/I は HOLD

## 残課題（別 issue 候補）

- **JP2K 書き込み**: `jpeg2k` (libopenjp2 binding) の追加で実装可能
- **PDF 連結**: `lopdf` 等の PDF パーサ追加で実装可能
- **morph 自動生成**: 性能ボトルネックが実測されたら、ターゲット SEL のみ

  手書き (~50-200 行) を優先

- **ccbord の Moore-tracing 終了条件**: 本 PR の cap は応急処置。

  ill-formed hole が発生する根本原因の調査

- **`#[ignore]` メッセージの一斉クリーンアップ**: 各サブ計画で必要分は

  解除済みだが、まだ古い記述が残るテストがありうる
