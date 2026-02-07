# P2テスト実装計画

## Context

C版Leptonicaの回帰テスト移植プロジェクト。P1テスト19個は全完了済み。次のフェーズとしてP2テスト35個の実装に進む。

旧計画(golden-doodling-sedgewick.md)からの改善点:

- サブエージェントへの過度に規定的なテンプレートを廃止。実装エージェントが既存テストパターンを参照し、自律的に判断する方針に変更
- テストの実現可能性をTier分類し、既存APIで実装可能なものを優先

## P2テスト実現可能性分類

### Tier A: テスト移植のみ（19個） -- 即実装可能

既存Rust APIで主要機能をカバー済み。テストコード作成のみで完了。

| クレート | テスト | 対象API |
| --------- | -------- | --------- |
| **Morph** | binmorph4 | DWA brick vs standard morph比較 |
| **Morph** | binmorph5 | DWA composite morph比較 |
| **Morph** | graymorph1 | gray_morph_sequence, tophat, bottomhat |
| **Morph** | colormorph | dilate/erode/open/close_color |
| **Morph** | morphseq | morph_sequence, gray_morph_sequence |
| **Transform** | affine | affine_pta, affine_sampled_pta |
| **Transform** | bilinear | bilinear_pta, bilinear_sampled_pta |
| **Transform** | projective | projective_pta, projective_sampled_pta |
| **Filter** | bilateral1 | bilateral_exact, bilateral_gray_exact |
| **Filter** | bilateral2 | bilateral パラメータバリエーション |
| **Filter** | adaptmap | background_norm, contrast_norm |
| **Color** | colorquant | median_cut_quant, octree_quant |
| **Color** | colorseg | color_segment, color_segment_simple |
| **Color** | colorcontent | color_content, count_colors |
| **Color** | colorfill | color_fill, color_fill_from_seed |
| **Region** | watershed | watershed_segmentation |
| **Region** | quadtree | quadtree_mean/variance/regions, IntegralImage |
| **Recog** | baseline | find_baselines, get_local_skew_angles |
| **I/O** | pnmio | PNM read/write roundtrip |

### Tier B: 軽微なAPI追加 + テスト（8個）

| クレート | テスト | 不足API |
| --------- | -------- | --------- |
| **I/O** | gifio | pix_equal比較ユーティリティ |
| **I/O** | webpio | lossy許容差比較 |
| **I/O** | mtiff | multi-page TIFF read/write |
| **I/O** | selio | Selシリアライズ I/O |
| **Filter** | rank | rank_filter部分は可。pixScaleGrayMinMax等は未実装 |
| **Filter** | adaptnorm | contrast_norm部分は可。pixGammaTRC等は未実装 |
| **Color** | cmapquant | colormap量子化API調査必要 |
| **Recog** | pageseg | segment_regions部分は可。table検出は未実装 |

### Tier C: 大規模API実装必要（12個） -- 後回し

Core P2全8個(boxa3, boxa4, ptra1, numa1, numa2, fpix1,
pixa1, pixa2)、dwamorph2、iomisc、threshnorm、seedspread。
GPLOT/Numa高度機能/Ptra/SARRAY等の未実装モジュールに依存。

## 実装方針

### 核心原則: C版テストの忠実な移植

**C版テスト(`reference/leptonica/prog/*_reg.c`)を移植の正とする。** 各テストの実装にあたっては:

1. **C版テストを最初に精読する**: テストの構造、テストケースの順序、検証する値・条件をC版から正確に把握する
2. **C版のテストロジックを忠実に再現する**: テスト項目の選択や検証値はC版に合わせる。Rust風にリファクタリングはするが、テスト内容の取捨選択はC版を尊重する
3. **C版にあってRust側に対応APIがない部分は明示的にスキップ**:
   `// C版: pixFoo() -- Rust未実装のためスキップ` のように
   C版の関数名を記録し、将来の実装時にすぐ追加できるようにする
4. **C版のテスト画像・パラメータを使用する**:
   C版で使用されているテスト画像やパラメータ値をそのまま使う。
   テスト画像が `tests/data/images/` にない場合は
   `reference/leptonica/prog/` からコピーする

### テスト失敗時の対処原則

**テストを通すためだけにテストを変更することは禁止。** テストが失敗した場合の対処順序:

1. **ライブラリ実装を修正する**: C版の実装(`reference/leptonica/src/`)を参考に、Rust側のライブラリコードを修正する。テストほどの厳密さは求めないが、C版の挙動に近づける
2. **テスト側の不備を確認する**: 実装修正後も失敗する場合、C版テストと照合してRust版テストの実装に不備がないか確認する
3. **`#[ignore]`にする**: テスト側に不備がなく、実装の修正も困難な場合は `#[ignore]` 属性を付与し、理由をコメントで記録する

### その他の原則

- **git-worktreeによる並列作業の分離**: クレート単位の並列実装では `git worktree` を積極的に使用し、各エージェントが独立したworktreeで作業することで干渉を排除する
- **Tier Bは部分実装を許容**: 既存APIでテスト可能な部分のみ実装し、未実装API部分はC版関数名付きコメントでスキップ

### git-worktree運用

```bash
# クレート単位でworktreeを作成
git worktree add ../leptonica-rs-p2-morph     p2/morph
git worktree add ../leptonica-rs-p2-transform p2/transform
git worktree add ../leptonica-rs-p2-filter    p2/filter
git worktree add ../leptonica-rs-p2-color     p2/color
git worktree add ../leptonica-rs-p2-region    p2/region
git worktree add ../leptonica-rs-p2-recog     p2/recog
git worktree add ../leptonica-rs-p2-io        p2/io
```

- 各エージェントは割り当てられたworktreeで作業し、テストファイル作成・ライブラリ修正・テスト実行を行う
- 完了後、メインブランチにマージする
- ライブラリ実装の修正が必要な場合もworktree内で行い、マージ時にコンフリクトを解決する

### 実装Wave

**Wave 1（Tier A: 19テスト）**: クレート単位で並列にTask agentを起動

- Morph (5テスト): binmorph4, binmorph5, graymorph1, colormorph, morphseq
- Transform (3テスト): affine, bilinear, projective
- Filter (3テスト): bilateral1, bilateral2, adaptmap
- Color (4テスト): colorquant, colorseg, colorcontent, colorfill
- Region (2テスト): watershed, quadtree
- Recog (1テスト): baseline
- I/O (1テスト): pnmio

**Wave 2（Tier B: 8テスト）**: Wave 1完了後、同様にクレート単位で実装

**Wave 3（Tier C: 12テスト）**: API実装が進んだ段階で計画

### エージェントへの委託方法

クレート単位でgeneral-purpose agentを起動。プロンプトには以下を含める:

- 対象テスト名のリスト
- **C版テストファイルを必ず読むこと**: `reference/leptonica/prog/{testname}_reg.c`
- 既存Rustテストのパターン参照: `crates/leptonica-morph/tests/binmorph1_reg.rs` 等
- テストフレームワーク: `crates/leptonica-test/src/params.rs`
- 完了基準: `cargo test` でPASS、かつC版テストの主要テストケースが網羅されていること

エージェントにはC版テストの忠実な移植を指示し、Rust側のAPI選択やコード構造はエージェントに委ねる。

## 重要ファイル

| ファイル | 用途 |
| --------- | ------ |
| `crates/leptonica-test/src/params.rs` | RegParamsテストフレームワーク |
| `crates/leptonica-test/src/lib.rs` | load_test_image等のヘルパー |
| `crates/leptonica-morph/tests/binmorph1_reg.rs` | 実装パターン参照 |
| `crates/leptonica-region/tests/conncomp_reg.rs` | 実装パターン参照 |
| `reference/leptonica/prog/*_reg.c` | C版テスト（移植元） |
| `tests/data/images/` | テスト画像 |

## 検証方法

```bash
# 個別テスト実行
cargo test -p leptonica-morph --test binmorph4_reg

# クレート内全回帰テスト
cargo test -p leptonica-morph --test '*_reg'

# ワークスペース全体の回帰テスト
cargo test --workspace --test '*_reg'

# 回帰テストファイル数の確認
find crates/*/tests -name '*_reg.rs' | wc -l
```

## golden-doodling-sedgewick.md 更新内容

P1完了状態への更新と、P2テストのTier分類情報を反映する。
