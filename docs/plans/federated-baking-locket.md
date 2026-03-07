# Phase 4: C版同等テスト内容強化

Status: IMPLEMENTED

## Context

C版とのベンチマーク比較が成り立つには、Rust版テストがC版と同等の検証を行っている必要がある。
Phase 3 (PR 1-8) は既存テストに golden manifest インフラ (`write_pix_and_check`) を追加したが、
**C版にあるテスト内容自体の追加は行われていない**。

監査結果 (`scripts/audit-regression-tests.py`):

- High divergence (>= 1.0): **25テスト**
- Medium divergence (0.3-1.0): **76テスト**
- うち実際にアクション必要: **~42テスト**（残りはRust側の方がチェック数多い or parity済み）

親計画: `docs/plans/014_regression-test-enhance.md`

## 方針

1. C版 `*_reg.c` の全チェックポイントをRust版と比較
2. Rust APIが存在する → 同等チェックを追加
3. Rust APIが存在しない → `#[ignore = "function_name not implemented"]` スケルトン追加
4. JPEGソーステストは golden 除外（デコーダ差異リスク）

## PR構成 (7 PR)

### PR 1: Easy Wins — API存在・RegParams不足 (4テスト)

ブランチ: `test/phase4-easy-wins`

| テスト       | モジュール | C      | Rust | アクション                                                       |
| ------------ | ---------- | ------ | ---- | ---------------------------------------------------------------- |
| seedspread   | region     | 7 WPAC | 0    | RegParamsテストに書き換え。4-cc/8-cc/lattice/sparse の 6-7 WPAC  |
| fpix2        | core       | 5 CP   | 0    | FPix border ops の 2 CP 追加。rotate_orth は未実装 → 3 #[ignore] |
| checkerboard | region     | 3 WPAC | 0    | checkerboard1/2.tif でコーナー検出。6 WPAC                       |
| circle       | transform  | 2 WPAC | 0    | circles.pa テストデータ不在。#[ignore] 維持                      |

見積: +15 checks, +4 #[ignore], ~13 golden

### PR 2: Color モジュール (8テスト)

ブランチ: `test/phase4-color-content`

| テスト       | div   | C  | Rust | アクション                                        |
| ------------ | ----- | -- | ---- | ------------------------------------------------- |
| grayquant    | 0.977 | 50 | 24   | dither結果WPAC, threshold_gray_arb 追加。+8       |
| paint        | 0.964 | 30 | 14   | RGB variants の WPAC 追加。cmap系は #[ignore]。+3 |
| paintmask    | 0.932 | 22 | 11   | 追加 depth variant の clip_masked。+5             |
| blend2       | 0.921 | 20 | 10   | C のオフセットバリエーション追加。+4              |
| colorize     | 0.713 | 16 | 11   | 追加 colorize バリエーション。+3                  |
| blend1       | 0.5   | 17 | 15   | 残りブレンドモード。+2                            |
| colorcontent | 0.5   | 19 | 25   | R>C だが 0 WPAC → 4 WPAC 追加                     |
| falsecolor   | 0.575 | 4  | 8    | R>C だが 1 WPAC → 3 WPAC 追加                     |

見積: +32 checks, ~15 golden

### PR 3: Core モジュール (8テスト)

ブランチ: `test/phase4-core-content`

| テスト  | div   | C     | Rust | アクション                                                           |
| ------- | ----- | ----- | ---- | -------------------------------------------------------------------- |
| hash    | 1.057 | 28 CV | 4    | string/pta/dna ハッシュの CV 追加。+10 CV, +5 #[ignore]              |
| ptra1   | 1.478 | 18    | 4    | Ptaa 操作の CV 追加。+6                                              |
| ptra2   | 1.384 | 19    | 6    | シリアライズ + 操作の CV 追加。+6                                    |
| string  | 1.506 | 31    | 6    | SArray 操作の CV 追加。search/replace は #[ignore]。+8, +4 #[ignore] |
| boxa3   | 1.256 | 9     | 4    | range/rank クエリの CV 追加。+4                                      |
| boxa4   | 1.518 | 11    | 2    | 全て未実装API。+6 #[ignore]                                          |
| pixcomp | 1.25  | 12    | 3    | 圧縮モードの WPAC 追加。+3                                           |
| insert  | 1.125 | 8     | 3    | シリアライズ round-trip 追加。+3                                     |

見積: +40 checks, +15 #[ignore], ~3 golden

### PR 4: Region モジュール (6テスト)

ブランチ: `test/phase4-region-content`

| テスト      | div   | C  | Rust | アクション                              |
| ----------- | ----- | -- | ---- | --------------------------------------- |
| conncomp    | 1.071 | 14 | 6    | pixaDisplay 再構成 + CP + centroids。+6 |
| texturefill | 1.075 | 12 | 6    | テクスチャフィルバリエーション。+4 WPAC |
| label       | 0.917 | 29 | 14   | ラベル統計 CV 追加。+6                  |
| grayfill    | 0.779 | 23 | 19   | フィルモードバリエーション。+3 WPAC     |
| watershed   | 0.6   | 11 | 27   | R>C だが 2 WPAC → +3 WPAC               |
| findcorners | 0.575 | 4  | 6    | R>C だが 1 WPAC → +2 WPAC               |

見積: +22 checks, ~14 golden

### PR 5: Transform + IO モジュール (6テスト)

ブランチ: `test/phase4-transform-io-content`

| テスト   | div   | C  | Rust | アクション                                   |
| -------- | ----- | -- | ---- | -------------------------------------------- |
| scale    | 0.864 | 33 | 19   | 2bpp/4bpp スケーリング variant 追加。+6 WPAC |
| smallpix | 1.8   | 1  | 0    | 小画素操作テスト。+1 WPAC, +1 #[ignore]      |
| compare  | 0.962 | 13 | 7    | correlation 関数の CV 追加。+4               |
| equal    | 0.588 | 17 | 7    | depth 変換 round-trip の CP 追加。+4         |
| jpegio   | 0.658 | 19 | 16   | quality sweep の CV 追加（WPAC 不可）。+3    |
| files    | 0.533 | 6  | 4    | format detection の CV 追加。+2              |

見積: +20 checks, +1 #[ignore], ~5 golden

### PR 6: Morph + Core 追加 (4テスト)

ブランチ: `test/phase4-morph-content`

| テスト     | div   | C  | Rust | アクション                                                                  |
| ---------- | ----- | -- | ---- | --------------------------------------------------------------------------- |
| ccthin2    | 1.3   | 15 | 2    | thin/skeletonize 結果の WPAC 追加。display 系は #[ignore]。+4, +5 #[ignore] |
| graymorph1 | 1.174 | 43 | 14   | gray morph サイズバリエーション + tophat。+8, +3 #[ignore]                  |
| pixmem     | 0.815 | 13 | 5    | copy/clone の CP 追加。+4                                                   |
| genfonts   | 0.95  | 4  | 3    | font bitmap WPAC 追加。+1                                                   |

見積: +17 checks, +8 #[ignore], ~10 golden

### PR 7: API不在テストのスケルトン (8テスト)

ブランチ: `test/phase4-skeletons`

| テスト    | div   | アクション                                   |
| --------- | ----- | -------------------------------------------- |
| colormask | 2.0   | 7 #[ignore] (HSV histogram/peak 系 API 不在) |
| subpixel  | 2.0   | 9 #[ignore] (subpixel rendering API 不在)    |
| rectangle | 1.9   | 9 #[ignore] (largest rectangle API 不在)     |
| numa2     | 1.188 | 5 CV + 6 #[ignore] (Pix extraction API 不在) |
| nearline  | 1.1   | 3 CV                                         |
| pdfio1    | 0.907 | 既存 3 #[ignore] のまま                      |
| psioseg   | 0.833 | 既存 3 #[ignore] のまま                      |
| mtiff     | 0.5   | 既存 7 #[ignore] のまま                      |

見積: +8 checks, +22 #[ignore]

## 対象外テスト

### Category A: Rust側の方がチェック数多い (25テスト, アクション不要)

boxa1(R:36>C:8), boxa2(R:30>C:7), numa1(R:78>C:20), numa3(R:22>C:12),
pixa1(R:47>C:2), pta(R:39>C:20), quadtree(R:59>C:7), selio(R:61>C:7),
colorfill(R:28>C:12), extrema(R:12>C:1), fpix1(R:47>C:28), kernel(R:52>C:18),
projection(R:26>C:19), xformbox(R:38>C:6), splitcomp(R:4>C:2),
webpanimio(R:6>C:1), locminmax(R:8>C:3), maze(R:4>C:3), encoding(R:4>C:2),
lowsat(R:11>C:6), bilateral2(R:15>C:8), binmorph6(R:7=C:7),
dna(R:7=C:7), bytea(R:2=C:2), files(R:4<C:6 minor)

### Category D: Parity達成済み (33テスト, div=0.0)

adaptmap, adaptnorm, binmorph1, binmorph3, blackwhite, coloring, colormorph,
colorquant, convolve, dwamorph1, edge, enhance, fhmtauto, flipdetect, gifio,
heap, ioformats, lowaccess, partition, pixa2, pixserial, pngio, pnmio, rank,
rankhisto, rasterop, rotate2, rotateorth, threshnorm, translate, warper, webpio,
writetext

## 全体見積

| PR             | テスト数 | 新規checks | #[ignore] | 新規golden |
| -------------- | -------- | ---------- | --------- | ---------- |
| 1 Easy Wins    | 4        | 15         | 4         | ~13        |
| 2 Color        | 8        | 32         | 0         | ~15        |
| 3 Core         | 8        | 40         | 15        | ~3         |
| 4 Region       | 6        | 22         | 0         | ~14        |
| 5 Transform+IO | 6        | 20         | 1         | ~5         |
| 6 Morph        | 4        | 17         | 8         | ~10        |
| 7 Skeletons    | 8        | 8          | 22        | 0          |
| **合計**       | **44**   | **~154**   | **~50**   | **~60**    |

Manifest: 383 → ~443 エントリ

## 実行方法

1. 各テストで haiku サブエージェントが C版 (`reference/leptonica/prog/*_reg.c`) を読み、

   各 `regTestWritePixAndCheck` / `regTestCompareValues` / `regTestComparePix` を特定

2. Rust API の有無を確認し、存在すれば等価チェックを追加、不在なら `#[ignore]` スケルトン
3. モジュールバッチ完了後 `REGTEST_MODE=generate cargo test --test <module>` で manifest 更新
4. `python3 scripts/audit-regression-tests.py` で divergence 改善を確認

## コミット規約

1テスト = 1コミット。同一ファイル内の密接なテストのみバッチ可。

```text
test(<module>): add C-equivalent checks to <test>_reg
```

## 検証

```bash
cargo test --test <module>
REGTEST_MODE=generate cargo test --test <module>
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/audit-regression-tests.py
```

## 重要ファイル

- `scripts/audit-regression-tests.py` — divergence 測定
- `docs/porting/regression-test-audit.csv` — 監査結果（PR毎に再生成）
- `tests/common/params.rs` — RegParams インフラ
- `tests/golden_manifest.tsv` — golden manifest（PR毎に更新）
- `reference/leptonica/prog/*_reg.c` — C版リファレンス
