# Git履歴再構築計画

## Context

### なぜこの変更が必要か

leptonica-rsのGit履歴が、リファレンス実装（C版leptonica）が存在する比較的単純な移植プロジェクトにもかかわらず、深刻な構造的問題を抱えている。前任AI（Opus 4.5）による開発で、以下の問題が蓄積した。

### 根本原因分析: なぜリファレンスがあるのにカオスになったか

**1. AIエージェントの「最短経路」バイアス**

- AIは「タスクを完了すること」を最優先し、プロセス規律（ブランチ運用、コミット粒度）を軽視する傾向がある
- mainに直接コミットする方が「速い」ため、feature branchの作成→マージという手順を省略した
- 結果: **98/105のfirst-parentコミットがmainへの直接コミット**

**2. ブランチの形骸化（パラレルワールド問題）**

- feature branchは55個以上作成されたが、適切にマージされたのは**7個のみ**
- 残りのブランチでは、変更がcherry-pickでmainに移され、ブランチ自体は放置された
- 一部のブランチ（feat/sarray, feat/adaptmap等）は「ブランチ上にmainのマージ履歴が蓄積する」異常構造
  - 例: `feat/sarray`内に `Merge branch 'feat/dewarp' into main` 等のマージコミットが存在
  - これらのマージは実際のmainには反映されておらず、ブランチ独自のパラレルワールド

**3. PRワークフローの不在（最大の原因）**

- GitHub PRを一切使わなかった。PRさえ使っていれば:
  - ブランチ→PR→レビュー→マージの強制フローになる
  - GitHub Copilotの自動レビューが品質ゲートとして機能する
  - マージコミットが自動的に作成され、first-parentログが整理される
  - PRの説明文が設計ドキュメントとして残る
- `.github/`ディレクトリなし（CI/CD、branch protection、PRテンプレートなし）

**4. バッチ処理志向**

- 「効率的」に19個のテストを1コミットに詰め込む（5ea6343: 37ファイル、5679行）
- 51ファイルのunsafe削除を1コミットで実行（1c3d39f）
- リファレンスがあるため「やること」は明確だったが、「どう記録するか」が軽視された

**5. TDDプロセスの無視**

- リファレンス実装が存在するため「テストと実装を同時に書ける」状況
- 結果としてRED→GREEN→REFACTORのサイクルが1コミットに圧縮され、QMS上のトレーサビリティが消失

---

## 現状の問題点一覧

### P1（致命的）

| # | 問題 | 影響 | 数量 |
|---|------|------|------|
| 1 | mainへの直接コミット | 機能単位の追跡不能 | 98件 |
| 2 | TDD RED状態の欠如 | QMSトレーサビリティ喪失 | 全feat |
| 3 | コミット粒度の過大 | レビュー・bisect困難 | 5ea6343, 1c3d39f |

### P2（重要）

| # | 問題 | 影響 | 数量 |
|---|------|------|------|
| 4 | feature branchの形骸化 | ブランチ残骸55個超 | 55+ |
| 5 | docs/feat/testの分離不足 | 開発プロセスの不可視化 | 全feat |
| 6 | 意味的分割の欠如 | first-parentログが機能単位で読めない | 全体 |
| 7 | PRワークフロー未使用 | レビュー記録・品質ゲート不在 | 全体 |

### P3（改善推奨）

| # | 問題 | 影響 | 数量 |
|---|------|------|------|
| 8 | 計画書のコミットタイミング | 計画→実装の流れが追えない | 大半 |
| 9 | 日本語コミットメッセージ混在 | 国際標準からの逸脱 | 1件 |
| 10 | Co-Authored-Byの欠落 | 貢献者追跡の不備 | 一部test |
| 11 | マージコミットメッセージの不備 | マージ理由が不明 | 7件 |

---

## 再構築アプローチ

### 原則: 手順省略の禁止

**作業効率のために手順を省略しない。** 省略は技術的負債を生み、再作業コストが元の作業コストを上回る。

具体的な禁止事項:

- 「この部分は既に良い状態だから再構築不要」という判断を安易にしない。再構築対象と定めた範囲は全て再構築する
- TDD RED stepを省略しない。全feature branchに#[ignore]付きテストを含める
- PRワークフローを省略しない。ローカルマージで代替しない
- Copilotレビューの確認を省略しない
- マージメッセージの詳細記載を省略しない

**この原則に違反しそうになったら、まずユーザーに相談する。**

### CLAUDE.md への追記事項（実装時に追記する）

```markdown
## 禁止事項

- 作業効率を理由にプロセス手順（TDD、PRワークフロー、レビュー確認）を省略しない
- 再構築対象と定めた範囲を安易に縮小しない。縮小が必要な場合はユーザーに相談する
- 「既に良い状態だから」「変更が少ないから」で手順を飛ばさない
```

### 決定事項

- **起点**: `ae0ee8b`（Initial commit + submodule + 全体計画の3コミット直後）から再構築。初期3マージ（test/reg-base, feat/io-webp, feat/morph-grayscale）も再構築対象
- **TDD RED方式**: `#[ignore = "not yet implemented"]`方式を基本とする
- **バックアップ**: bareリポジトリ含む完全バックアップ
- **GitHub PR経由**: ローカルマージではなく、全ブランチをGitHub PRで統合する
- **Copilotレビュー**: 各PRでGitHub Copilotの自動レビューを受け、指摘を反映する
- **マージコミット詳細化**: Linusの教えに従い、マージメッセージに変更の要約・影響・理由を記載
- **自律実行**: ユーザー承認を都度求めず、自律的に全ブランチを処理する

### メインエージェントのコンテキスト節約方針

この作業は50+ブランチ×5コミットの大規模作業となる。
メインエージェントは以下の方針でコンテキストを節約する:

1. **各ブランチの作業はsubagentに委譲**: ブランチ作成→コミット→push→PR作成をsubagentが実行
2. **メインエージェントはオーケストレーションに専念**: Phase単位の進捗管理とエラーハンドリング
3. **subagentへの指示はテンプレート化**: 共通パターンを定義し、ブランチ固有の差分のみ指示
4. **Copilotレビュー待機もsubagentが担当**: PR作成→レビュー待ち→修正→マージを一貫して処理

### 並行作業との共存

別エージェントが同リポジトリのmainで作業中のため、以下の制約を守る:

- **`git stash`は実行しない** — 別エージェントの未コミット変更を巻き込むため
- **mainブランチには一切触れない** — rebuild/mainブランチとworktreeで完結させる
- **main差し替えは別エージェント完了後** — 別エージェントの作業がコミット・マージされた後に実施

### 事前クリーンアップ（誤った初回実行の残骸除去）

初回実行でf646259起点の誤ったrebuild/mainを作成し、PR #2も作成済み。これらを除去する:

```bash
# 1. PR #2をクローズ
gh pr close 2

# 2. リモートの誤ブランチを削除
git push origin --delete feat/core-pixa
git push origin --delete rebuild/main

# 3. ローカルのworktreeとブランチを削除
git worktree remove /home/tagawa/github/leptonica-rs-rebuild
git branch -D rebuild/main
git branch -D feat/core-pixa 2>/dev/null || true

# 4. バックアップブランチ・タグも一旦削除（再作成するため）
git branch -D backup/main-before-rebuild
git tag -d backup/main-original
```

### バックアップ

```bash
# 1. ブランチ・タグによるバックアップ（mainを変更しない）
git branch backup/main-before-rebuild main
git tag backup/main-original main

# 2. bareリポジトリへの完全バックアップ（既存がある場合は上書き）
rm -rf /home/tagawa/github/leptonica-rs-backup-bare
git clone --bare /home/tagawa/github/leptonica-rs /home/tagawa/github/leptonica-rs-backup-bare
```

### worktree構成

| worktree | パス | 用途 |
|---|---|---|
| メイン | `/home/tagawa/github/leptonica-rs` | 参照元（read-only） |
| 再構築 | `/home/tagawa/github/leptonica-rs-rebuild` | rebuild/main上で作業 |

```bash
git branch rebuild/main ae0ee8b
git worktree add /home/tagawa/github/leptonica-rs-rebuild rebuild/main
```

### 各Feature Branchのコミット構成（TDDテンプレート）

```
feat/xxx
  ├── docs: add xxx implementation plan          # 計画
  ├── test(scope): add xxx regression test (RED) # テスト先行（失敗状態）
  ├── feat(scope): implement xxx                 # 実装（テスト通過）
  ├── fix/refactor(scope): ...                   # 修正（該当時のみ）
  └── docs: update progress for xxx              # 進捗更新
→ GitHub PR を作成 → Copilotレビュー → 修正反映 → マージ
```

RED状態の作り方（`#[ignore]`方式）:

1. テストファイルを配置し、全テスト関数に `#[ignore = "not yet implemented"]` を付与
2. 実装モジュールは空スタブ（`pub mod xxx;`のみ）またはスケルトン
3. `cargo check --workspace` が通る状態でコミット（コンパイルは通るがテストはskip = RED）
4. 次のコミットで実装を追加し、`#[ignore]` を除去（GREEN）

### GitHub PR ワークフロー

各ブランチのPR作成→マージの流れ:

```bash
# 1. ブランチをpush
cd /home/tagawa/github/leptonica-rs-rebuild
git push -u origin feat/xxx

# 2. PR作成（詳細な説明文付き）
gh pr create --base rebuild/main --title "feat(scope): implement xxx" --body "$(cat <<'EOF'
## Summary
- C版leptonicaのxxx機能をRustに移植
- TDDサイクル: RED(テスト先行) → GREEN(実装) → 進捗更新

## Changes
- `docs/plans/xxx.md`: 実装計画書
- `crates/leptonica-xxx/tests/xxx_reg.rs`: 回帰テスト
- `crates/leptonica-xxx/src/xxx.rs`: 実装本体

## Test plan
- [ ] cargo check --workspace
- [ ] cargo test -p leptonica-xxx
- [ ] cargo clippy --workspace
EOF
)"

# 3. Copilotレビュー待機 → 指摘対応
gh pr checks <PR番号> --watch
gh pr view <PR番号> --comments  # レビューコメント確認
# 指摘があれば修正コミット → push

# 4. マージ（詳細なマージメッセージ付き）
gh pr merge <PR番号> --merge --subject "Merge branch 'feat/xxx'" --body "$(cat <<'EOF'
feat/xxx: C版leptonicaのxxx機能をRust移植

* docs: 実装計画書を追加
* test(scope): 回帰テストを追加（RED状態）
* feat(scope): xxx機能を実装（GREEN状態）
* docs: 進捗を更新

C版leptonica の xxxFunc() / xxxFunc2() に相当する機能。
[変更の影響範囲と理由をここに記載]
EOF
)"
```

### Feature Branch分割

全コミットをae0ee8b以降から再構築する。旧test/reg-base内の35コミットもcrate単位のブランチに分割。

#### Phase 0: 基盤構築 (6ブランチ) — 旧test/reg-base前半

旧test/reg-baseマージの35コミットを、機能単位のfeature branchに分割し直す。

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/core-foundation | 184c7ce, 6702a10, d504613, 2e4495a, 36c9408, 3c4e530, ece2721, c38b88f | Workspace構造、Core型（Pix/Box/Pta/Colormap）、crateスタブ、facade |
| feat/io-base | 238a1ef, dfc3a35, c71636a, 24a5793, 04c5cfe | IO基盤（BMP/JPEG/PNG/PNM/TIFF/GIF） |
| feat/transform-base | 37105e8 | Transform基盤（rotate/scale） |
| feat/morph-binary | 73c6089 | Binary morphology基盤 |
| feat/filter-base | ef974de | Filter基盤（convolve/edge/kernel） |
| feat/color-base | ac1fbbc | Color基盤（analysis/colorspace/quantize/threshold） |
| feat/region-base | c785e42, 9c10916 | Region基盤（conncomp/label/seedfill/watershed） |
| feat/recog-base | 146e3e1, e5ad457, 6dc677d | Recog基盤（baseline/jbclass/recog） |

各ブランチのコミット構成:

- feat/core-foundationのみ特殊: TDDの前にworkspace自体が存在しないため、buildコミットを分けて構成
  1. `build: set up Cargo workspace structure`
  2. `test(core): add core data structure tests (RED)` — P1テスト（cf3fa37のBox/Boxa/Pta部分）を#[ignore]で配置
  3. `feat(core): add core data structures (Pix, Box, Pta, Colormap)` — GREEN
  4. `feat: add specialized crate stubs and facade crate`
- feat/io-base: RED step に pngio_reg.rs（fcd25cb由来）を#[ignore]付きで配置
- feat/morph-binary: RED step に binmorph1_reg.rs（fcd25cb由来）を#[ignore]付きで配置
- feat/region-base: RED step に conncomp_reg.rs（fcd25cb由来）を#[ignore]付きで配置
- feat/transform-base: RED step に P1テスト（628fe82）を#[ignore]付きで配置
- feat/filter-base: RED step に P1テスト（fa207cd）を#[ignore]付きで配置
- feat/color-base: RED step に P1テスト（859ddb2）を#[ignore]付きで配置
- feat/recog-base: RED step に P1テスト（c93dd5e）を#[ignore]付きで配置
- テストデータ（beaca91）はfeat/core-foundationに含める

横断的コミットの吸収先:

- af585e0, 258218a (clippy fixes) → 各ブランチのGREENステップに含める
- defdac4, edba3f7, b7b4f69, e7f1e3f, f14ab07 (docs/analysis) → docs/analysisブランチ
- 297b0c7, 1436c53 (orchestration strategy) → feat/core-foundationのdocsステップ
- 5ccac53 (deps/config) → 最後のPhase 0ブランチのchoreステップ

#### Phase 0.5: 旧個別マージ機能 (2ブランチ)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/io-webp | e50a8bf | WebP I/O |
| feat/morph-grayscale | af36dd0 | グレースケール形態学 |

Phase 0の各ブランチがマージ済みであることが前提。

#### docs/analysis (1ブランチ)

| 役割 | SHA | メッセージ |
|---|---|---|
| docs | defdac4 | docs: add feature comparison between C and Rust implementations |
| docs | edba3f7 | docs: add test comparison between C and Rust implementations |
| docs | b7b4f69 | docs: update feature comparison with latest implementation status |
| docs | e7f1e3f | docs: update test comparison with current Rust implementation status |
| docs | f14ab07 | docs: update feature comparison with current implementation status |

#### test/reg-base (1ブランチ) — テストインフラ＋初期回帰テスト

| 役割 | SHA | メッセージ |
|---|---|---|
| feat | 2a91b55 | feat(test): add regression test infrastructure |
| test | fcd25cb | test(io,morph,region): add regression tests (pngio, binmorph1, conncomp) |
| test | beaca91 | test: add test data and expected output files |

#### Phase 1: Core拡張 (2)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/core-pixa | 54073e7 | Pixa/Pixaa構造体 |
| feat/core-numa | 24e4001 | Numa/Numaa数値配列 |

#### Phase 2: Transform系 (6)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/transform-rotate | 5d43e72 | 任意角度回転 |
| feat/transform-affine | 813428d | アフィン変換 |
| feat/transform-shear | 83e4438 | シアー変換 |
| feat/transform-bilinear | deddf28 | バイリニア変換 |
| feat/transform-projective | cfdcaac | 射影変換 |
| feat/transform-warper | e3d5456 | ワーピング変換 |

#### Phase 3: Filter系 (3)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/filter-bilateral | 28aee9b | バイラテラルフィルタ |
| feat/filter-rank | 10594d6 | ランクフィルタ |
| feat/filter-adaptmap | 1b28f5e | 適応マッピング |

#### Phase 4: Color系 (4)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/color-segmentation | 73bb37a | 色セグメンテーション |
| feat/color-colorfill | 1cbdeac | カラーフィル |
| feat/color-coloring | 0e1dd0f | カラー化 |
| feat/color-histogram | 3db5d38 | ヒストグラム分析 |

#### Phase 5: Pix操作 (5)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/pix-compare | e10c797 | 画像比較 |
| feat/pix-blend | 18da8fe | ブレンド・合成 |
| feat/pix-rop | 8630890 | ラスタ操作 |
| feat/pix-arith | d415955 | 算術操作 |
| feat/pix-graphics | 92b9c64 | グラフィックス描画 |

#### Phase 6: 高度構造 (2)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/core-fpix | 1df2af2 | 浮動小数点画像 |
| feat/core-sarray | aefd00d | 文字列配列 |

#### Phase 7: Morph拡張 (4)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/morph-colormorph | 4927fc8 | カラー形態学 |
| feat/morph-ccthin | ed39b23 | 細線化 |
| feat/morph-morphseq | 6f0baa0 | 形態学シーケンス |
| feat/morph-dwa | cc76ad0 | DWA高速形態学 |

#### Phase 8: Region系 (3)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/region-ccbord | 1d8b38a | 境界トレース |
| feat/region-quadtree | 9ac830f | 四分木 |
| feat/region-maze | 0159303 | 迷路生成・解法 |

#### Phase 9: Recog系 (2)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/recog-dewarp | 884f92c | ページデワープ |
| feat/recog-barcode | e0164a3 | バーコード検出 |

#### Phase 10: IO拡張 (3)

| ブランチ | 元コミット | 内容 |
|---|---|---|
| feat/io-jp2k | 200cb0a | JPEG 2000 |
| feat/io-pdf | 2d1e3b1 | PDF出力 |
| feat/io-ps | 5314dc7 | PostScript出力 |

#### テストの統合方針

テストはPhase 1-10の各feature branchのRED stepに統合する。
元の履歴では後半にまとめて追加されたテストも、対応するfeatureのbranchに含める。

**feature branchに統合するテスト（TDD RED step）:**

- affine_reg → feat/transform-affine
- bilinear_reg → feat/transform-bilinear
- projective_reg → feat/transform-projective
- bilateral1_reg, bilateral2_reg → feat/filter-bilateral
- adaptmap_reg, adaptnorm_reg → feat/filter-adaptmap
- rank_reg → feat/filter-rank
- colorseg_reg, colorcontent_reg → feat/color-segmentation
- colorfill_reg → feat/color-colorfill
- colorquant_reg, cmapquant_reg → feat/color-histogram
- morphseq_reg → feat/morph-morphseq
- colormorph_reg → feat/morph-colormorph
- binmorph4_reg, binmorph5_reg → feat/morph-dwa
- quadtree_reg, watershed_reg → feat/region-quadtree
- baseline_reg → feat/recog-barcode
- fpix1_reg → feat/core-fpix
- pixa1_reg, pixa2_reg → feat/core-pixa
- numa1_reg, numa2_reg → feat/core-numa
- selio_reg → feat/morph-morphseq

**維持済み既存機能のテスト（独立ブランチで追加）:**

#### Phase 11: 既存機能テスト (3)

| ブランチ | 内容 |
|---|---|
| docs/test-plans | テスト計画書（51d3ce2, 30b5d8d, b714da9） |
| test/p1-regression | P1テスト — test/reg-base等の既存機能向け（rotate, scale, io, morph基本, color基本, region基本, recog基本） |
| test/io-regression | IO追加テスト — gifio, webpio, mtiff, iomisc, pnmio, pageseg, graymorph1（既存マージ済み機能向け） |

#### Phase 12: リファクタリング (2)

| ブランチ | 内容 |
|---|---|
| docs/unsafe-reduction-plan | unsafe削減計画書 |
| refactor/remove-unsafe | unsafe削除（crate別8コミットに分割） |

### コミット粒度・ワークフロー

`~/.claude/CLAUDE.md` および `CLAUDE.md` で定義済みのルールに従う。

---

## コミット→ブランチ マッピング

ae0ee8b以降の全コミットの割り当て先（旧test/reg-base, io-webp, morph-grayscaleマージ含む）。
5ea6343（19テスト）はファイル単位で分割し、対応するfeature branchに統合。
fix/style/choreコミットは対応するfeature branchに吸収。

各ブランチの`plan`行は対応する`docs/plans/*.md`から取得する（全34ブランチ分が既存）。

テストが存在しない16ブランチのRED stepは、C版の対応テスト（`prog/*_reg.c`）を参考に
回帰テストを新規作成する。対象:
feat/transform-rotate, -shear, -warper, feat/color-coloring,
feat/pix-compare, -blend, -rop, -arith, feat/core-sarray, feat/core-numa-ops,
feat/morph-ccthin, feat/region-ccbord, -maze, feat/recog-dewarp,
feat/io-jp2k, -pdf, -ps

### feat/core-foundation（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| build | 184c7ce | build: set up Cargo workspace structure |
| RED | cf3fa37 | test(core): add P1 regression tests for Box/Boxa and Pta (RED) |
| feat | 6702a10 | feat(core): add error handling and module structure |
| feat | d504613 | feat(core): add Pix image structure implementation |
| feat | 2e4495a | feat(core): add Box geometric structure implementation |
| feat | 36c9408 | feat(core): add Pta point array implementation |
| feat | 3c4e530 | feat(core): add PixColormap palette implementation |
| feat | ece2721 | feat: add specialized crate stubs |
| feat | c38b88f | feat: add main leptonica facade crate |
| test | beaca91 | test: add test data and expected output files |
| docs | 297b0c7, 1436c53 | docs: add feature implementation orchestration strategy |

### feat/io-base（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| RED | fcd25cb部分 | test(io): add pngio regression test (RED) — pngio_reg.rsを#[ignore]付きで配置 |
| RED | 044ca7a | test(io): add P1 regression tests for image I/O (RED) |
| feat | 238a1ef | feat(io): implement image format I/O modules (BMP/JPEG/PNG/PNM) |
| feat | dfc3a35 | feat(io): implement TIFF image format support |
| docs | c71636a | docs: add GIF I/O implementation plan |
| feat | 24a5793 | feat(io): implement GIF image format support |
| fix | 04c5cfe | refactor(io): remove unnecessary type casts in GIF test code |

### feat/transform-base（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| RED | 628fe82 | test(transform): add P1 regression tests for geometric transforms (RED) |
| feat | 37105e8 | feat(transform): implement geometric transformations (rotate/scale) |

### feat/morph-binary（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| RED | fcd25cb部分 | test(morph): add binmorph1 regression test (RED) — binmorph1_reg.rsを#[ignore]付きで配置 |
| RED | 6a6f5a6 | test(morph): add P1 regression tests for morphology (RED) |
| feat | 73c6089 | feat(morph): implement binary morphological operations |

### feat/filter-base（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| RED | fa207cd | test(filter): add P1 regression tests for image filtering (RED) |
| feat | ef974de | feat(filter): implement image filtering modules |

### feat/color-base（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| RED | 859ddb2 | test(color): add P1 regression tests for color processing (RED) |
| feat | ac1fbbc | feat(color): implement comprehensive color processing module |

### feat/region-base（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| docs | 9c10916 | docs: add implementation plan for leptonica-region crate |
| RED | fcd25cb部分 | test(region): add conncomp regression test (RED) — conncomp_reg.rsを#[ignore]付きで配置 |
| RED | c691268 | test(region): add P1 regression tests for region analysis (RED) |
| feat | c785e42 | feat(region): implement region analysis and segmentation module |

### feat/recog-base（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| docs | e5ad457 | docs: update plan for leptonica-recog crate implementation (Phase 1) |
| RED | c93dd5e | test(recog): add P1 regression test for skew detection (RED) |
| feat | 146e3e1 | feat(recog): add error types and module structure for recognition |
| feat | 6dc677d | feat(recog): implement Phase 2 - character recognition and JBIG2 classification |

### docs/analysis（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| docs | defdac4 | docs: add feature comparison between C and Rust implementations |
| docs | edba3f7 | docs: add test comparison between C and Rust implementations |
| docs | b7b4f69 | docs: update feature comparison with latest implementation status |
| docs | e7f1e3f | docs: update test comparison with current Rust implementation status |
| docs | f14ab07 | docs: update feature comparison with current implementation status |

### test/reg-base（Phase 0）

| 役割 | SHA | メッセージ |
|---|---|---|
| feat | 2a91b55 | feat(test): add regression test infrastructure |
| test | fcd25cb | test(io,morph,region): add regression tests |
| chore | 5ccac53 | chore: update dependencies and configuration |

### feat/io-webp（Phase 0.5）

| 役割 | SHA | メッセージ |
|---|---|---|
| feat | e50a8bf | feat(io): add WebP image format support |

### feat/morph-grayscale（Phase 0.5）

| 役割 | SHA | メッセージ |
|---|---|---|
| feat | af36dd0 | feat(morph): implement grayscale morphological operations |

### 横断的fix（Phase 0各ブランチに吸収）

| SHA | メッセージ | 吸収先 |
|---|---|---|
| af585e0 | refactor: resolve all clippy warnings across crates | 各Phase 0ブランチのGREENに含める |
| 258218a | refactor: apply remaining clippy suggestions for unused variables | 同上 |

### feat/core-pixa

| 役割 | SHA/ファイル | メッセージ |
|---|---|---|
| plan | docs/plans/pixa-implementation.md | （既存ファイル） |
| GREEN | 54073e7 | feat(core): implement Pixa and Pixaa structures |
| docs | 253c236 | docs: update progress for WebP, grayscale morph, and Pixa |
| RED | 82290af | test(region): add pixa1 regression test |
| RED | 11f4df0 | test(transform): add pixa2 regression test |

### feat/core-numa

| 役割 | SHA | メッセージ |
|---|---|---|
| plan | 685a55a | docs(plans): add Numa implementation plan |
| GREEN | 24e4001 | feat(core): add Numa and Numaa numeric array structures |
| docs | 50b99cd | docs: update progress for Numa implementation |
| RED | 9b07af4 | test(core): add numa1 regression test |
| RED | 6913a68 | test(core): add numa2 regression test |

### feat/core-numa-ops

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 4ebd324 | feat(core): add Numa operations API |
| GREEN | e888474 | feat(core): add Pix statistics, conversion, and extraction APIs |
| refactor | c0186dd | refactor: promote compare_pix/count_foreground to library APIs |

### feat/transform-rotate

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 5d43e72 | feat(transform): implement arbitrary angle rotation |
| docs | b24ccb5 | docs: update progress for arbitrary angle rotation |

### feat/transform-affine

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 813428d | feat(transform): add comprehensive affine transformation support |
| docs | 67b9d96 | docs: update progress for affine transformation |
| fix | 3911313 | fix(transform): remove unnecessary cast in affine test |
| RED | 5ea6343部分 | affine_reg.rs + テスト画像 |

### feat/transform-shear

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 83e4438 | feat(transform): add comprehensive shear transformation support |
| docs | cda621a | docs: update progress for shear transformation |

### feat/transform-bilinear

| 役割 | SHA | メッセージ |
|---|---|---|
| plan | 71ce008 | docs: add bilinear transform implementation plan |
| GREEN | deddf28 | feat(transform): implement bilinear transformation |
| docs | 2f90e83 | docs: update progress for bilinear transform |
| RED | 5ea6343部分 | bilinear_reg.rs + テスト画像 |

### feat/transform-projective

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | cfdcaac | feat(transform): add projective transformation support |
| docs | 2191fee | docs: update progress for projective transform |
| RED | 5ea6343部分 | projective_reg.rs + テスト画像 |

### feat/transform-warper

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | e3d5456 | feat(transform): add warping transformations module |
| docs | d475fb3 | docs: update progress for warper |

### feat/filter-bilateral

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 28aee9b | feat(filter): implement bilateral filtering |
| docs | db91887 | docs: update progress for bilateral filter |
| RED | 5ea6343部分 | bilateral1_reg.rs, bilateral2_reg.rs |

### feat/filter-rank

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 10594d6 | feat(filter): implement rank filter |
| docs | a73aec4 | docs: update progress for rank filter |
| fix | 47f3209 | fix(filter): remove unnecessary cast in rank filter test |
| RED | 0a56d7f部分 | rank_reg.rs |

### feat/filter-adaptmap

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 1b28f5e | feat(adaptmap): implement adaptive mapping |
| docs | f87a9b0 | docs: update progress for adaptive mapping |
| fix | a3830ed | fix: resolve clippy warnings in adaptmap.rs |
| RED | 5ea6343部分 | adaptmap_reg.rs |
| RED | 0a56d7f部分 | adaptnorm_reg.rs |

### feat/color-segmentation

| 役割 | SHA | メッセージ |
|---|---|---|
| plan | e353a7e | docs(plans): update color-segmentation plan status |
| GREEN | 73bb37a | feat(color): implement color segmentation algorithm |
| docs | 42b55f4 | docs: update progress for color segmentation |
| RED | 5ea6343部分 | colorseg_reg.rs, colorcontent_reg.rs |

### feat/color-colorfill

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 1cbdeac | feat: add color fill implementation |
| docs | e9c5d3a | docs: update progress for color fill |
| fix | decae4c | fix: remove unnecessary casts in colorfill.rs |
| RED | 5ea6343部分 | colorfill_reg.rs |

### feat/color-coloring

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 0e1dd0f | feat(coloring): implement colorization functions |
| docs | 85a7c1d | docs: update progress for coloring |

### feat/color-histogram

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 3db5d38 | feat(histogram): implement pixel histogram analysis |
| docs | c4fd2ce | docs: update progress for histogram |
| RED | 5ea6343部分 | colorquant_reg.rs |
| RED | 76841fa | test(color): add cmapquant regression test |

### feat/pix-compare

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | e10c797 | feat(compare): implement image comparison functions |
| docs | f181f84 | docs: update progress for image comparison |

### feat/pix-blend

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 18da8fe | feat: implement image blending and compositing |
| docs | 958a013 | docs: update progress for image blending |

### feat/pix-rop

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 8630890 | feat: implement image raster operations |
| docs | 4e872e3 | docs: update progress for logical operations |

### feat/pix-arith

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | d415955 | feat(pix): implement arithmetic operations module |
| docs | 7786b32 | docs: update progress for arithmetic operations |

### feat/pix-graphics

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 92b9c64 | feat(graphics): add graphics rendering module |
| chore | a7c3eae | chore(graphics): export graphics types from pix module |
| docs | 054f140 | docs: update progress for graphics |

### feat/core-fpix

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 1df2af2 | feat(fpix): implement floating-point image container |
| docs | 1d9cf96 | docs: update progress for FPix |
| RED | 0c95517 | test(core): add fpix1 regression test |

### feat/core-sarray

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | aefd00d | feat(sarray): implement Sarray string array data structure |
| docs | 3473512 | docs: update progress for Sarray |

### feat/morph-colormorph

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 4927fc8 | feat(morph): implement color morphological operations |
| RED | 5ea6343部分 | colormorph_reg.rs |

### feat/morph-ccthin

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | ed39b23 | feat(morph): implement connectivity-preserving thinning |
| docs | 6f21cf6 | docs: update progress for thinning implementation |

### feat/morph-morphseq

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 6f0baa0 | feat(morph): add morphological sequence operations |
| docs | 5ce4a4c | docs: update progress for morphological sequence |
| RED | 5ea6343部分 | morphseq_reg.rs |
| RED | 5cdc9a3 | test(morph): add selio regression test |

### feat/morph-dwa

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | cc76ad0 | feat(dwa): implement high-speed morphological operations |
| docs | 185a89c | docs: update progress for DWA |
| fix | 39f62ff | fix(morph): correct DWA structuring element range for even sizes |
| RED | 5ea6343部分 | binmorph4_reg.rs, binmorph5_reg.rs |

### feat/region-ccbord

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 1d8b38a | feat(region): implement border tracing |
| docs | 627dc70 | docs: update progress for border tracing |
| docs | fbb76a9 | docs(test): improve ccbord regression test |

### feat/region-quadtree

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 9ac830f | feat(quadtree): implement hierarchical quadtree |
| docs | b755c91 | docs: update progress for quadtree |
| RED | 5ea6343部分 | quadtree_reg.rs, watershed_reg.rs |

### feat/region-maze

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 0159303 | feat(region): add maze generation and solving |
| docs | bee1016 | docs: update progress for maze |

### feat/recog-dewarp

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 884f92c | feat(dewarp): implement page dewarping module |
| docs | c7214b6 | docs: update progress for dewarping |

### feat/recog-barcode

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | e0164a3 | feat(recog): add 1D barcode detection |
| docs | cbdca8f | docs: update progress for barcode |
| fix | 1ea89f3 | fix(barcode): correct doctest examples |
| RED | 5ea6343部分 | baseline_reg.rs + テスト画像 |

### feat/io-jp2k

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 200cb0a | feat(io): add JPEG 2000 image reading support |
| docs | a7472aa | docs: update progress for JP2K |

### feat/io-pdf

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 2d1e3b1 | feat(io): add PDF image output support |
| docs | 089b6bb | docs: update progress for PDF output |

### feat/io-ps

| 役割 | SHA | メッセージ |
|---|---|---|
| GREEN | 5314dc7 | feat(io): add PostScript image output support |
| docs | 2a9d00e | docs: update progress for PostScript |

### docs/test-plans

| 役割 | SHA | メッセージ |
|---|---|---|
| docs | 51d3ce2 | docs: comprehensive test migration plan for Rust |
| docs | 30b5d8d | docs: P1テスト実装計画の詳細化 |
| docs | b714da9 | docs: add P2 test implementation plan |
| docs | ad17151 | docs: add remaining unimplemented features to orchestration plan |

### test/p1-regression（廃止 → Phase 0各ブランチに統合）

P1テスト（cf3fa37, 044ca7a, 6a6f5a6, 628fe82, fa207cd, 859ddb2, c691268, c93dd5e）は
Phase 0の各ブランチのRED stepに統合済み。このブランチは不要。
44baa0e (Cargo.lock更新) は該当するPhase 0ブランチのchoreステップに吸収。

### test/io-regression（既存IO/recog機能向け）

| 役割 | SHA | メッセージ |
|---|---|---|
| test | 766b424 | test(io): add gifio, webpio, mtiff regression tests |
| test | cbd5cef | test(io): add iomisc regression test |
| test | 37be730 | test(recog): add pageseg regression test |
| test | 4035202 | test: add pnmio regression test output files |
| test | 5ea6343部分 | graymorph1_reg.rs, pnmio_reg.rs |

### chore/test-cleanup

| 役割 | SHA | メッセージ |
|---|---|---|
| chore | e126ead | chore: remove test output files and ignore regout directory |

### docs/unsafe-reduction-plan

| 役割 | SHA | メッセージ |
|---|---|---|
| docs | 17899c3 | docs: add unsafe reduction plan |

### refactor/remove-unsafe（crate別に分割）

| 役割 | SHA | メッセージ |
|---|---|---|
| refactor | 1c3d39f | refactor: remove unnecessary unsafe (→ 8コミットに分割) |

### 横断的fix/styleコミット（各ブランチに吸収）

| SHA | メッセージ | 吸収先 |
|---|---|---|
| e0ae604 | fix: resolve clippy linting issues | 各ブランチのfixステップ |
| 722ca4a | fix: resolve all clippy warnings in regression tests | test/p1-regression |
| 0a0ca9e | style: apply cargo fmt to P2 test files | 各ブランチのテストに含める |

### rebuild/mainのfirst-parentログ（最終形）

```
2397e92 Initial commit
c4048a3 chore: add leptonica reference as submodule
ae0ee8b docs: add overall project plan document
  ↓ 全てGitHub PR経由のマージコミット、詳細メッセージ付き
--- Phase 0: 基盤構築 ---
------- Merge branch 'feat/core-foundation'
------- Merge branch 'feat/io-base'
------- Merge branch 'feat/transform-base'
------- Merge branch 'feat/morph-binary'
------- Merge branch 'feat/filter-base'
------- Merge branch 'feat/color-base'
------- Merge branch 'feat/region-base'
------- Merge branch 'feat/recog-base'
------- Merge branch 'docs/analysis'
------- Merge branch 'test/reg-base'
--- Phase 0.5: 旧個別マージ機能 ---
------- Merge branch 'feat/io-webp'
------- Merge branch 'feat/morph-grayscale'
--- Phase 1-10: 機能拡張 ---
------- Merge branch 'feat/core-pixa'
------- Merge branch 'feat/core-numa'
  ... (Phase 2-10の34ブランチ) ...
--- Phase 11-12: テスト・リファクタリング ---
------- Merge branch 'docs/test-plans'
------- Merge branch 'test/io-regression'
------- Merge branch 'chore/test-cleanup'
------- Merge branch 'docs/unsafe-reduction-plan'
------- Merge branch 'refactor/remove-unsafe'
--- Phase 13: 最新機能 ---
------- Merge branch 'feat/core-numa-ops'
```

first-parentログだけで「何がいつ統合されたか」が一目瞭然になる。
各マージコミットにはLinus方式の詳細メッセージが付与される。

### 変更の取り込み方法

worktreeは同じ`.git`を共有するため、元のコミットSHAがそのまま使える:

```bash
# 最も効率的: 特定コミットからファイルを直接取得
cd /home/tagawa/github/leptonica-rs-rebuild
git checkout <元のSHA> -- <ファイルパス>
```

---

## 実行順序

各Phaseは上記の順番（Phase 1→12）で順次実行する。
各Phase内のブランチは独立しているため順序不問だが、
依存関係がある場合（例: fpixはpix-arithに依存する可能性）は先に依存先を処理する。

### subagentへの委譲パターン

メインエージェントは各Phase単位でsubagentを起動し、以下を指示する:

```
subagentへの指示テンプレート:
1. worktree: /home/tagawa/github/leptonica-rs-rebuild
2. ベースブランチ: rebuild/main
3. ブランチ名: feat/xxx
4. 元コミットSHA: <sha>
5. 対応するdocs/fixコミット: <sha list>
6. TDD RED方式: #[ignore = "not yet implemented"]
7. PR作成 → Copilotレビュー待機 → 修正 → マージ
8. マージメッセージ: Linus方式で詳細記載
```

### 作業パターン（1ブランチあたり）

```bash
cd /home/tagawa/github/leptonica-rs-rebuild

# 1. ブランチ作成
git checkout rebuild/main && git checkout -b feat/xxx

# 2. 計画書コミット
git checkout <plan-commit> -- docs/plans/xxx.md
git add docs/plans/ && git commit -m "docs: add xxx implementation plan

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"

# 3. テスト(RED)コミット
# テストファイル配置 + #[ignore] + スタブ
git add ... && git commit -m "test(scope): add xxx regression test (RED)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"

# 4. 実装(GREEN)コミット
git checkout <feat-commit> -- crates/...
# #[ignore] 除去
git add -A && git commit -m "feat(scope): implement xxx

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"

# 5. fix/refactor（該当時のみ）
git checkout <fix-commit> -- ...
git add -A && git commit -m "fix(scope): ...

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"

# 6. 進捗更新コミット
git checkout <docs-commit> -- docs/plans/
git add docs/ && git commit -m "docs: update progress for xxx

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"

# 7. push & PR作成
git push -u origin feat/xxx
gh pr create --base rebuild/main --title "feat(scope): implement xxx" --body "..."

# 8. Copilotレビュー待機 & 修正反映
# gh pr checksが通るまで待機、レビューコメントを確認・対応

# 9. マージ（詳細メッセージ付き）
gh pr merge <PR番号> --merge ...

# 10. rebuild/mainを更新
git checkout rebuild/main && git pull origin rebuild/main

# 11. 検証
cargo check --workspace
```

---

## 検証

### 1. コード状態の一致

```bash
git diff backup/main-before-rebuild rebuild/main -- crates/ docs/ Cargo.toml Cargo.lock
# 差分がゼロであること
```

### 2. テスト通過

```bash
cd /home/tagawa/github/leptonica-rs-rebuild
cargo test --workspace
cargo clippy --workspace
```

### 3. 履歴構造の確認

```bash
# first-parentが全てマージコミット（初期3件除く）
git log --oneline --first-parent --no-merges rebuild/main
# → ae0ee8b, c4048a3, 2397e92 の3件のみ

# PRマージによるマージコミット数の確認
git log --oneline --first-parent --merges rebuild/main | wc -l
# → 期待値: Phase 0(10) + Phase 0.5(2) + Phase 1-10(39) + Phase 11(3) + Phase 12(2) + Phase 13(1) = 57
# (test/p1-regressionはPhase 0に統合されたため独立ブランチなし)
```

### 4. TDD RED状態のサンプル検証

```bash
# 任意のfeature branchのREDコミットをcheckout
git log --oneline feat/transform-affine
# REDコミットに移動してテスト失敗を確認
```

### 5. GitHub PR履歴の確認

```bash
# 全PRがマージ済みであることを確認
gh pr list --state merged --base rebuild/main
# → 57件のマージ済みPR
```

### 6. 別エージェントの成果統合 → 切り替え

```bash
# 1. 別エージェントの成果をmainから取得してrebuild/mainにマージ
#    （別エージェントの作業がmainにcommit/push済みであること）
git -C /home/tagawa/github/leptonica-rs-rebuild checkout rebuild/main
git cherry-pick <別エージェントのコミットSHA>...
#    または: git merge main （mainの最新状態を取り込む）

# 2. 検証
cargo test --workspace && cargo clippy --workspace

# 3. 旧mainを退避
git branch old/main main

# 4. rebuild/mainをmainに昇格
git checkout rebuild/main
git branch -f main rebuild/main
git checkout main
git push --force-with-lease origin main

# 5. worktree削除・ブランチ整理
git worktree remove /home/tagawa/github/leptonica-rs-rebuild
# 旧ブランチ削除（backup/*は検証完了まで保持）
```

---

## 関連ファイル

- `docs/plans/000_overall-plan.md` - 全体計画（Feature分割の参照源）
- `Cargo.toml` - Workspace構成
- `crates/leptonica-core/src/lib.rs` - Core crateモジュール定義（staged変更あり）
- `crates/leptonica-core/src/numa/operations.rs` - 未追跡（stash退避対象）
