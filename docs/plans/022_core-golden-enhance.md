# 022: core モジュール回帰テスト golden manifest 強化

Status: IMPLEMENTED

Phase 3 PR 8/8: `tests/core/` 配下の回帰テストに `write_pix_and_check()` を追加し、
golden manifest によるピクセルレベル回帰検出を有効化する。

## Context

Phase 3（回帰テスト強化）の最終PR。core モジュールには7つの RegParams 使用ファイルがあり、
うち boxa3_reg は既に golden 済み（3エントリ）。残り6ファイルのうち4ファイルが
決定的な Pix 出力を持ち golden 化の対象となる。

親計画: `docs/plans/014_regression-test-enhance.md`

## 対象ファイル (4ファイル, ~12 golden)

- `conversion_reg.rs` — ~9 golden (depth conversion results from PNG/TIFF sources)
- `hash_reg.rs` — 1 golden (hash-line rendered)
- `logicops_reg.rs` — 1 golden (inverted image)
- `fpix1_reg.rs` — 1 golden (FPix→Pix conversion)

## 除外ファイル

- `boxa3_reg.rs` — 既に golden 済み（3エントリ）
- `equal_reg.rs` — compare_pix で既にピクセル比較済み
- `insert_reg.rs` — Pix 出力なし（Numa/Boxa/Pixa 要素操作のみ）
- JPEG ソースのテスト — デコーダ差異リスク
- RegParams 未使用ファイル（13ファイル） — Phase 3 スコープ外
