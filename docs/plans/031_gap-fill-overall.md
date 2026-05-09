# Gap-Fill 全体計画（C版未移植機能の補完）

Status: PLANNED
作成日: 2026-05-09

## Context

`docs/porting/feature-comparison.md` 上は実カバレッジ 100%（✅ 同等 + 🔄 異なる）だが、
ソースを `pub fn` で grep して照合した結果、以下の C 版関数は対応する Rust 実装が
存在しない。本計画はこれら「真の溝」を埋めるためのロードマップである。

検証手順:

1. 各候補を `rg "pub fn <snake_case>" src/` および同義語で網羅検索
2. 対応モジュールが存在しなければ「未実装」と確定
3. テストの `#[ignore]` メッセージは古いものが多いため鵜呑みにしない

## 未実装一覧（モジュール別）

| # | カテゴリ              | C版関数                                                                                                                             | C版ファイル                                    | 行数                | サブ計画 |
| - | --------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | ------------------- | -------- |
| A | core/compare          | `pixBestCorrelation`, `pixCompareWithTranslation`                                                                                   | compare.c, correlscore.c                       | 79+109              | 102      |
| B | core/fpix             | `fpixRotateOrth`, `fpixRotate90`                                                                                                    | fpix2.c                                        | ~150                | 103      |
| C | core/boxa             | `boxaaTranspose` ※実装済みで取りこぼし、テスト追加のみで完了                                                                        | boxfunc2.c                                     | ~50                 | 104 ✅   |
| D | core/sarray           | `sarrayRead`, `sarrayWrite`, `sarrayReadStream/Mem`, `sarrayWriteStream/Mem`, `sarrayFindStringByHash`（任意）, `arrayFindSequence` | sarray1.c, utils2.c                            | ~250                | 105      |
| E | io/jpeg               | `pixSetChromaSampling`                                                                                                              | jpegio.c                                       | ~30                 | 202      |
| F | io/jp2k               | `pixWriteJp2k` 系（write）, cropped/scaled read                                                                                     | jp2kio.c                                       | ~700                | 203      |
| G | io/pdf                | `concatenatePdf`, `concatenatePdfToData`, `ptraConcatenatePdf`, `ptraConcatenatePdfToData`                                          | pdfio1.c, pdfio2.c                             | ~300                | 204      |
| H | transform/affine      | `pixAffineSequential`                                                                                                               | affine.c                                       | 152                 | 301      |
| I | morph 自動生成        | `pixFMorphopGen_1`, `pixFHMTGen_1`, `pixHMTDwa_1`                                                                                   | fmorphgen.1.c, fhmtgen.1.c, dwacomb.2.c (+low) | ~12,000（機械生成） | 401      |
| J | region/ccbord         | `pixGetAllCCBorders` のメモリ最適化                                                                                                 | ccbord.c（既存実装あり、効率課題）             | -                   | 701      |
| K | recog/pageseg 矩形    | `pixFindLargestRectangle`, `pixFindRectangleInCC`, `pixFindLargeRectangles`                                                         | pageseg.c                                      | 113+163+48          | 801      |
| L | recog/pageseg 表/反転 | `pixDecideIfTable`, `pixAutoPhotoinvert`                                                                                            | pageseg.c                                      | 127+79              | 802      |

合計の純実装規模は自動生成除外で約 2,500 行。
自動生成 morph (I) は機械生成コードで規模が桁違いに大きいため別フェーズ扱いとする。

## 不要分類（実装しない）

- `L_ASET`（順序集合）, `L_HASHMAP` — `BTreeSet`/`HashMap` で代替可
- `sarrayFindStringByHash` — 利用箇所がなく `find_string` で代替可（必要時のみ実装）

## グルーピングと優先度

実装規模・依存関係から3グループに分類する。

### Group 1: 小規模・独立（推定: 1-2日 / 項目）

すぐ完了でき、外部依存も少ない。先に着手して勢いをつける。

- **A** core/compare 並進付き比較
- **B** core/fpix 直交回転
- **C** core/boxa 転置
- **E** io/jpeg クロマサンプリング設定
- **H** transform/affine 連結アフィン

### Group 2: 中規模・横断（推定: 3-5日 / 項目）

複数関数が絡むがアルゴリズム自体は素直。

- **D** core/sarray ファイル/ストリーム I/O
- **K** recog/pageseg 矩形検出 3関数
- **L** recog/pageseg 表判定・自動反転

### Group 3: 大規模・外部依存（推定: 1週以上 / 項目）

外部 crate 依存や効率最適化を伴う。

- **F** io/jp2k 書き込み + 高度読み込み（依存: `jpeg2k` crate の機能調査）
- **G** io/pdf 連結（PDF オブジェクト構造の解析が必要）
- **J** region/ccbord メモリ最適化（既存実装のリファクタリング、OOM 再現テストが必要）
- **I** morph 自動生成（コード生成スクリプトの移植 or 手書き化）

## 副次的整理タスク

- `#[ignore]` メッセージのうち実装済みなのに古いまま残っているもの（推定 60+ 件）を更新する

  → 各サブ計画の REFACTOR フェーズで該当テストの ignore を解除しつつ実施

## ステータス管理

各サブ計画は独立した `docs/plans/NNN_*.md` を持ち、Status: PLANNED → IN_PROGRESS → IMPLEMENTED で進行する。
着手時にこの文書の表に対応する PR リンクを追記する。

## 進行順序（提案）

1. Group 1 を一気に片付けて feature-comparison のメンテ精度を上げる
2. Group 2 で recog 系の `#[ignore]` を解消
3. Group 3 を1項目ずつ着手（外部依存調査を先行）
