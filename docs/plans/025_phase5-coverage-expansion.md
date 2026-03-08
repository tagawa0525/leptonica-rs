# Phase 5: テストカバレッジ拡充 — Ratio < 0.1 テストの C版同等化

Status: IN_PROGRESS

## Context

Phase 4 完了後のベンチマーク（C 10回 / Rust 10回）で、Ratio < 0.1（Rust が C の 10倍以上高速）
のテストが 10件検出された。これは Rust テストが C テストに比べて大幅に作業量が少ないことを示す。

API 監査の結果、**大半の必要な API は Rust に既に実装済み**だがテストで使われていないことが判明。
新規移植が必要な API は `max_dynamic_range` の 1関数のみ（残り 2関数は `#[ignore]` 対応）。

## 対象テスト

| テスト           |  Ratio | C WPAC | 現Rust WPAC | 原因                       |
| ---------------- | -----: | -----: | ----------: | -------------------------- |
| colorspace_reg   | 0.001x |     10 |           1 | API存在だがテスト未使用    |
| colorfill_reg    | 0.020x |     12 |           1 | API存在だがテスト未使用    |
| findpattern2_reg | 0.024x |      6 |           1 | SEL生成API未使用           |
| rotate1_reg      | 0.027x |      5 |           4 | 任意角度回転未テスト       |
| speckle_reg      | 0.031x |      9 |           2 | パイプライン未構築         |
| writetext_reg    | 0.034x |      7 |           5 | テスト内容不足             |
| binarize_reg     | 0.041x |      9 |           6 | 高度二値化メソッド未テスト |
| psioseg_reg      | 0.057x |      6 |           4 | セグメント化PS未実装       |
| distance_reg     | 0.059x |     10 |           9 | 可視化関数未使用           |
| findpattern1_reg | 0.071x |      4 |           9 | SEL生成API未使用           |

## PR構成 (3 PR)

### PR 1: テスト拡充 — API既存テスト (7テスト)

ブランチ: `test/phase5-coverage-expansion`

全て既存APIの活用。新規プロダクションコードなし。

| テスト           | モジュール | アクション                                                                                       |
| ---------------- | ---------- | ------------------------------------------------------------------------------------------------ |
| findpattern1_reg | recog      | `generate_sel_boundary`でSEL自動生成、`display_matched_pattern`/`remove_matched_pattern`追加     |
| findpattern2_reg | recog      | boundary/runs/random 3種SEL生成、HMTパイプライン、パターン除去                                   |
| rotate1_reg      | transform  | 任意角度回転（Shear/Sampling/AreaMap/AMCorner）× 複数画像型                                      |
| speckle_reg      | region     | `background_norm_flex`→`gamma_trc_masked`→`threshold_to_binary`→HMT→dilate→subtract パイプライン |
| binarize_reg     | color      | tiled Sauvola比較、`sauvola_on_contrast_norm`、`thresh_on_double_norm`                           |
| colorspace_reg   | color      | HSV画像生成、colormap変換roundtrip、color_magnitude閾値スイープ                                  |
| colorfill_reg    | color      | `expand_replicate`、`color_content_by_location`タイル戦略、実画像処理                            |

### PR 2: API移植 + テスト拡充 — distance_reg, writetext_reg

ブランチ: `feat/filter-max-dynamic-range`

新規API: `max_dynamic_range` (~80行, `src/filter/enhance.rs`)

| テスト        | モジュール | アクション                                                                  |
| ------------- | ---------- | --------------------------------------------------------------------------- |
| distance_reg  | region     | 8コンボ × (distance→max_dynamic_range→WPAC + distance→render_contours→WPAC) |
| writetext_reg | io         | multi-depth text rendering、Bmf拡張テスト                                   |

### PR 3: テスト拡充 + スケルトン — psioseg_reg

ブランチ: `test/phase5-psioseg-expansion`

| テスト      | モジュール | アクション                                                                               |
| ----------- | ---------- | ---------------------------------------------------------------------------------------- |
| psioseg_reg | io         | 合成画像パイプライン（scale→compose→quantize→PS出力）、セグメンテーション部は`#[ignore]` |
