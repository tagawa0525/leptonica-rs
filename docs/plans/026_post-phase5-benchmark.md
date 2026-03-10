# C版 vs Rust版 回帰テストベンチマーク — Phase 5 後

Status: IN_PROGRESS

## Context

Phase 5 完了（PR #284-#287、manifest 478→541）後にベンチマークを再実行。
Phase 5 では Ratio < 0.1 だった 10 テストを C 版同等に拡充済み。

Ratio ≤ 0.8（Rust が C より 20%以上高速）のテストは、Rust テストの内容不足の
可能性があるため調査する。

## 実行結果 (2026-03-08, Phase 5 後)

### ベンチマーク概要

- C版: 10回実行、約34.2秒/回
- Rust版: 10回実行、約26.4秒/回
- **全体: Rust版はC版より約23%高速**

### Phase 5 の効果

Phase 5 で拡充した 10 テストの Ratio 変化:

| テスト           | Phase 4 後 | Phase 5 後 | 変化        |
| ---------------- | ---------: | ---------: | ----------- |
| speckle_reg      |     0.031x |     1.476x | ✅ 大幅改善 |
| psioseg_reg      |     0.057x |     1.366x | ✅ 大幅改善 |
| findpattern1_reg |     0.071x |     1.234x | ✅ 大幅改善 |
| colorfill_reg    |     0.020x |     1.095x | ✅ 大幅改善 |
| binarize_reg     |     0.041x |     0.772x | ✅ 改善     |
| writetext_reg    |     0.034x |     0.303x | ⬆ 改善      |
| rotate1_reg      |     0.027x |     0.201x | ⬆ 改善      |
| distance_reg     |     0.059x |     0.109x | ⬆ 改善      |
| colorspace_reg   |     0.001x |     0.004x | → 軽微      |
| findpattern2_reg |     0.024x |     0.135x | ⬆ 改善      |

### Ratio > 1.0 のテスト（Rust が C より遅い）(28件)

Phase 5 で拡充した speckle_reg (1.48x)、psioseg_reg (1.37x)、findpattern1_reg
(1.23x)、colorfill_reg (1.10x) が新たに > 1.0 に到達。

| テスト           | C mean | Rust mean |  Ratio | 原因分類               |
| ---------------- | -----: | --------: | -----: | ---------------------- |
| convolve_reg     | 0.047s |    0.320s | 6.768x | テスト内容差異         |
| blend1_reg       | 0.085s |    0.405s | 4.796x | テスト内容差異         |
| adaptmap_reg     | 0.122s |    0.457s | 3.756x | テスト内容差異         |
| enhance_reg      | 0.413s |    1.159s | 2.809x | テスト内容差異         |
| colorize_reg     | 0.218s |    0.604s | 2.775x | テスト内容差異         |
| adaptnorm_reg    | 0.098s |    0.259s | 2.635x | テスト内容差異         |
| blend4_reg       | 0.088s |    0.206s | 2.347x | テスト内容差異         |
| iomisc_reg       | 0.024s |    0.046s | 1.910x | インフラオーバーヘッド |
| colormorph_reg   | 0.149s |    0.278s | 1.867x | テスト内容差異         |
| wordboxes_reg    | 0.470s |    0.800s | 1.702x | テスト内容差異         |
| rankhisto_reg    | 0.153s |    0.259s | 1.698x | テスト内容差異         |
| blackwhite_reg   | 0.049s |    0.077s | 1.577x | インフラオーバーヘッド |
| dewarp_reg       | 0.186s |    0.278s | 1.497x | テスト内容差異         |
| speckle_reg      | 0.032s |    0.047s | 1.476x | ✅ Phase 5 で改善      |
| graymorph2_reg   | 0.035s |    0.051s | 1.458x | 実装差異               |
| fhmtauto_reg     | 0.015s |    0.021s | 1.414x | 実装差異               |
| psioseg_reg      | 0.039s |    0.054s | 1.366x | ✅ Phase 5 で改善      |
| flipdetect_reg   | 0.085s |    0.110s | 1.300x | 実装差異               |
| rankbin_reg      | 0.103s |    0.134s | 1.299x | テスト内容差異         |
| blend2_reg       | 0.153s |    0.190s | 1.240x | テスト内容差異         |
| findpattern1_reg | 0.261s |    0.322s | 1.234x | ✅ Phase 5 で改善      |
| projective_reg   | 0.159s |    0.193s | 1.215x | 実装差異               |
| gifio_reg        | 0.266s |    0.323s | 1.214x | テスト内容差異         |
| italic_reg       | 0.270s |    0.317s | 1.171x | テスト内容差異         |
| equal_reg        | 0.044s |    0.052s | 1.167x | インフラオーバーヘッド |
| colorfill_reg    | 0.093s |    0.102s | 1.095x | ✅ Phase 5 で改善      |
| bilinear_reg     | 0.152s |    0.161s | 1.055x | ほぼ等速               |
| logicops_reg     | 0.010s |    0.010s | 1.054x | ほぼ等速               |

### Ratio ≤ 0.8 のテスト分析（C ≥ 50ms、Rust実装あり）

62 テストが該当。ここでの WPAC（write_pix_and_check）は、「テスト内でピクセルを書き込む処理（write_pix 系）を実行し、その結果を直後の check 系アサーションで検証する」1 組の操作を 1 カウントとした値。各テストの C 版・Rust 版の実行ログに出力される `write_pix_and_check=XXX` の行を集計したもので、処理したピクセル数と検証回数の近似指標としてテストの仕事量・カバレッジの proxy とみなす。以下では、この WPAC 数の C/Rust 比較による分類を行う:

#### カテゴリ A: WPAC ギャップ大（C WPAC >> Rust WPAC）— テスト拡充が必要

| テスト         | C WPAC | Rust WPAC |  Ratio | 備考                     |
| -------------- | -----: | --------: | -----: | ------------------------ |
| grayquant_reg  |     47 |        13 | 0.219x | 34 WPAC 不足             |
| scale_reg      |     33 |         9 | 0.106x | 24 WPAC 不足             |
| graymorph1_reg |     30 |         9 | 0.501x | 21 WPAC 不足             |
| paint_reg      |     29 |         7 | 0.427x | 22 WPAC 不足             |
| affine_reg     |     29 |         6 | 0.685x | 23 WPAC 不足             |
| paintmask_reg  |     22 |         7 | 0.593x | 15 WPAC 不足             |
| multitype_reg  |     17 |         5 | 0.091x | 12 WPAC 不足             |
| alphaops_reg   |     15 |         5 | 0.246x | 10 WPAC 不足             |
| pixadisp_reg   |     12 |         3 | 0.073x | 9 WPAC 不足              |
| baseline_reg   |     12 |         0 | 0.540x | 全 WPAC 不足             |
| colorspace_reg |     10 |         6 | 0.004x | 4 WPAC 不足 + 処理差異大 |
| bilateral2_reg |      8 |         0 | 0.726x | 全 WPAC 不足             |
| cmapquant_reg  |      8 |         2 | 0.280x | 6 WPAC 不足              |
| crop_reg       |      7 |         2 | 0.276x | 5 WPAC 不足              |

#### カテゴリ B: WPAC 同等だが処理量差異 — C がより重い計算を実行

| テスト        | C WPAC | Rust WPAC |  Ratio | 備考                         |
| ------------- | -----: | --------: | -----: | ---------------------------- |
| distance_reg  |     10 |         7 | 0.109x | C は全距離関数×全画像で計算  |
| warper_reg    |      2 |         3 | 0.113x | C の歪み計算が重い           |
| pdfio2_reg    |      0 |         0 | 0.137x | C の PDF マルチページが重い  |
| colorseg_reg  |      3 |         1 | 0.117x | C のセグメンテーション処理   |
| rotate2_reg   |      2 |         3 | 0.122x | C の全角度回転ループ         |
| xformbox_reg  |      6 |         0 | 0.200x | C は Box 変換の全パス実行    |
| partition_reg |      0 |         3 | 0.251x | C のパーティション分割が重い |

#### カテゴリ C: Rust 実装が本質的に高速 — 問題なし

| テスト         | C WPAC | Rust WPAC |  Ratio | 備考                      |
| -------------- | -----: | --------: | -----: | ------------------------- |
| pageseg_reg    |     12 |         4 | 0.779x | Rust 最適化済み           |
| pdfio1_reg     |      0 |         0 | 0.696x | I/O 効率差                |
| coloring_reg   |      1 |         2 | 0.730x | Rust が同等以上の処理     |
| colorquant_reg |      1 |         1 | 0.301x | Rust の量子化が高速       |
| kernel_reg     |      0 |         0 | 0.422x | Rust のカーネル処理が高速 |

### 結論

1. **アルゴリズム実装の不適切さは検出されなかった。**

   Ratio ≤ 0.8 の主因は「Rust テストの WPAC 数不足」（カテゴリ A）と
   「C 版がより多くの計算パスを実行」（カテゴリ B）。

2. **Phase 5 の効果は顕著。** 10 テスト中 4 テストが Ratio > 1.0 に到達。

3. **今後の拡充対象候補（WPAC ギャップ順）:**
   - grayquant_reg (34 WPAC 不足)
   - scale_reg (24 WPAC 不足)
   - affine_reg (23 WPAC 不足)
   - paint_reg (22 WPAC 不足)
   - graymorph1_reg (21 WPAC 不足)

## 重要ファイル

- `scripts/benchmark-regression-145.py` — ベンチマーク実行
- `docs/porting/c-rust-regression-benchmark-145.md` — 生成レポート
- `target/benchmark-regression-145/` — 生データ
