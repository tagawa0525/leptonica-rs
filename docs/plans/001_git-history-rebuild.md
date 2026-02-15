# Git履歴再構築計画

## 1. Context

ae0ee8b以降の履歴は構造的に破綻している。
non-mergeコミットの94.6%がmainへの直接コミット、TDDサイクル未遵守、
1コミットに19テスト実装を混在させる等、CLAUDE.mdで定めたプロセスに違反している。
コード自体は正常動作（~91K行、47回帰テスト）するため、
同一機能を適切な履歴構造で再コミットする。

## 2. 原則

以下の原則はすべてのPhaseに適用する。

- **直列実行厳守・並列実行禁止**: Phase N+1はPhase Nのマージ完了後に開始する。複数Phaseの並列実行は絶対に行わない
- **確実性優先**: 実装速度より確実性を重視する。レビュー指摘は必ず対応してからマージする
- **プロセス遵守**: 各手順は必須ゲートである。ゲートを迂回しない
- **メインエージェント不介入**: メインエージェントはPhase番号の管理と結果確認のみ行う。ファイル操作・Bash実行・コミット等は一切行わず、すべてサブエージェントに委任する
- **コンテキスト節約**: メインエージェントはPhase番号と完了状況のみ保持する。サブエージェントは必要に応じてサブサブエージェントを起動してよい
- **PRマージ方式**: `gh pr merge --merge`のみ使用する。`--squash`・`--rebase`は禁止

## 3. 実行体制

| 役割 | 担当 | 責務 |
|------|------|------|
| メインエージェント | 親セッション | Phase番号管理、サブエージェント起動、結果確認 |
| サブエージェント | `general-purpose`型 | 各Phaseの全工程（ブランチ作成〜マージ）を自律遂行 |
| サブサブエージェント | 任意の型 | サブエージェントがコンテキスト節約のために必要に応じて起動 |

Agent Teams（実験的機能）は並列・独立作業向けであり、
公式ドキュメントが不向きとする「Sequential tasks」「Tasks with many interdependencies」に
本タスクが該当するため採用しない。

## 4. 準備

```bash
# 現在の状態をタグで保全
git tag backup/pre-rebuild HEAD
git push origin backup/pre-rebuild

# 不要ブランチの削除（存在する場合のみ）
git branch --list docs/rebuild-cleanup | grep -q . && git branch -D docs/rebuild-cleanup
git ls-remote --heads origin docs/rebuild-cleanup | grep -q . && git push origin --delete docs/rebuild-cleanup

# mainをae0ee8bにリセット
# --force-with-leaseでリモート先行更新を検知する
git checkout main
git reset --hard ae0ee8bc5bfe944b6af674c031adb88afac431db
git push --force-with-lease origin main
```

## 5. リファレンスへのアクセス方法

サブエージェントへの指示に以下を含める。

**旧Rust実装**（`backup/pre-rebuild`タグ）:
- ファイル内容参照: `git show backup/pre-rebuild:<path>`
- ファイル一覧: `git ls-tree -r --name-only backup/pre-rebuild -- <dir>`
- ワークツリーに展開: `git checkout backup/pre-rebuild -- <path>`

**C版リファレンス**（gitサブモジュール `reference/leptonica/`）:
- C実装: `reference/leptonica/src/*.c`、`reference/leptonica/src/*.h`
- C回帰テスト: `reference/leptonica/prog/*_reg.c`（160+テスト）
- 対応例: `reference/leptonica/prog/boxa1_reg.c` → `crates/leptonica-core/tests/boxa1_reg.rs`

## 6. 再構築フェーズ（12ブランチ・直列実行厳守）

### Phase 1: `feat/workspace-setup` — インフラ

TDD不要（テスト対象のコードを含まないため）。

対象ファイル:
- `Cargo.toml`（workspace定義）、`Cargo.lock`
- 全crateの`Cargo.toml` + 空`src/lib.rs`
- `crates/leptonica-test/src/`（lib.rs, error.rs, params.rs）
- `crates/leptonica-doc/src/lib.rs`
- `tests/data/images/`（テスト画像群）
- `.gitignore`、`.markdownlintignore`、`src/main.rs`

コミット:
1. `build: set up Cargo workspace with crate stubs`
2. `test: add test data images and test infrastructure crate`

### Phase 2: `feat/core-pix` — Pix/Box/Pta

対象ファイル:
- `crates/leptonica-core/src/`（error.rs, lib.rs）
- `crates/leptonica-core/src/pix/`（mod.rs, ops.rs, access.rs, convert.rs, clip.rs）
- `crates/leptonica-core/src/box_/mod.rs`
- `crates/leptonica-core/src/pta/mod.rs`

テスト: `boxa1_reg`, `boxa2_reg`, `pta_reg`

コミット:
1. RED: `test(core): add Pix, Box, Pta regression tests`
2. GREEN: `feat(core): implement Pix/PixMut, Box, Pta with pixel access`

### Phase 3: `feat/core-data` — Numa/FPix/Pixa/Sarray/Colormap

対象ファイル:
- `crates/leptonica-core/src/numa/`（mod.rs, histogram.rs, operations.rs）
- `crates/leptonica-core/src/fpix/mod.rs`
- `crates/leptonica-core/src/pixa/mod.rs`
- `crates/leptonica-core/src/sarray/mod.rs`
- `crates/leptonica-core/src/colormap/mod.rs`

テスト: `numa1_reg`, `numa2_reg`, `fpix1_reg`, `pixa1_reg`

コミット:
1. RED: `test(core): add Numa, FPix, Pixa regression tests`
2. GREEN: `feat(core): implement Numa, FPix, Pixa, Sarray, Colormap`

### Phase 4: `feat/core-ops` — Pix演算・描画・統計

対象ファイル:
- `crates/leptonica-core/src/pix/`（arith.rs, blend.rs, rop.rs, graphics.rs, border.rs, compare.rs, histogram.rs, statistics.rs, extract.rs）

テスト: `pixa2_reg`

コミット:
1. RED: `test(core): add Pix operations regression test`
2. GREEN: `feat(core): implement Pix arithmetic, blend, ROP, graphics, statistics`

### Phase 5: `feat/io` — 全画像I/O

対象ファイル:
- `crates/leptonica-io/src/`（lib.rs, error.rs, format.rs, png.rs, jpeg.rs, tiff.rs, gif.rs, bmp.rs, pnm.rs, webp.rs, pdf.rs, jp2k.rs, ps/mod.rs, ps/ascii85.rs）

テスト: `pngio_reg`, `jpegio_reg`, `ioformats_reg`, `iomisc_reg`, `gifio_reg`, `webpio_reg`, `mtiff_reg`, `pnmio_reg`

コミット:
1. RED: `test(io): add image I/O regression tests`
2. GREEN: `feat(io): implement PNG, JPEG, TIFF, GIF, WebP, BMP, PNM, PDF, PS, JP2K`

### Phase 6: `feat/transform` — 幾何変換

対象ファイル:
- `crates/leptonica-transform/src/`（lib.rs, error.rs, rotate.rs, scale.rs, affine.rs, bilinear.rs, projective.rs, shear.rs, warper.rs）

テスト: `rotate1_reg`, `rotate2_reg`, `rotateorth_reg`, `scale_reg`, `affine_reg`, `bilinear_reg`, `projective_reg`

コミット:
1. RED: `test(transform): add geometric transform regression tests`
2. GREEN: `feat(transform): implement rotation, scale, affine, bilinear, projective, warper`

### Phase 7: `feat/filter` — フィルタリング

対象ファイル:
- `crates/leptonica-filter/src/`（lib.rs, error.rs, kernel.rs, bilateral.rs, rank.rs, convolve.rs, adaptmap.rs, edge.rs）

テスト: `bilateral1_reg`, `bilateral2_reg`, `rank_reg`, `convolve_reg`, `adaptmap_reg`, `adaptnorm_reg`, `edge_reg`

コミット:
1. RED: `test(filter): add filtering regression tests`
2. GREEN: `feat(filter): implement bilateral, rank, convolve, adaptive mapping, edge detection`

### Phase 8: `feat/color` — 色処理

対象ファイル:
- `crates/leptonica-color/src/`（lib.rs, error.rs, threshold.rs, quantize.rs, segment.rs, colorspace.rs, colorfill.rs, coloring.rs, analysis.rs）

テスト: `binarize_reg`, `cmapquant_reg`, `colorquant_reg`, `colorseg_reg`, `colorcontent_reg`, `colorfill_reg`, `colorspace_reg`

コミット:
1. RED: `test(color): add color processing regression tests`
2. GREEN: `feat(color): implement thresholding, quantization, segmentation, color fill`

### Phase 9: `feat/morph` — 形態学演算

対象ファイル:
- `crates/leptonica-morph/src/`（lib.rs, error.rs, sel.rs, sequence.rs, thin_sels.rs, binary.rs, grayscale.rs, color.rs, dwa.rs, thin.rs）

テスト: `binmorph1-5_reg`, `graymorph1_reg`, `colormorph_reg`, `dwamorph1-2_reg`, `morphseq_reg`, `selio_reg`

コミット:
1. RED: `test(morph): add morphological operation regression tests`
2. GREEN: `feat(morph): implement binary, grayscale, color morphology, DWA, thinning`

### Phase 10: `feat/region` — 領域解析

対象ファイル:
- `crates/leptonica-region/src/`（lib.rs, error.rs, conncomp.rs, ccbord.rs, label.rs, quadtree.rs, watershed.rs, maze.rs, seedfill.rs, select.rs）

テスト: `conncomp_reg`, `ccbord_reg`, `label_reg`, `quadtree_reg`, `watershed_reg`

コミット:
1. RED: `test(region): add region analysis regression tests`
2. GREEN: `feat(region): implement connected components, border tracing, quadtree, watershed, maze`

### Phase 11: `feat/recog` — 認識

対象ファイル:
- `crates/leptonica-recog/src/`（lib.rs, error.rs, baseline.rs, pageseg.rs, skew.rs, barcode/\*, dewarp/\*, jbclass/\*, recog/\*）

テスト: `baseline_reg`, `pageseg_reg`, `skew_reg`

コミット:
1. RED: `test(recog): add recognition regression tests`
2. GREEN: `feat(recog): implement barcode, dewarp, JB classification, character recognition`

### Phase 12: `docs/project-setup` — Facade・ドキュメント

TDD不要（ライブラリコードを含まないため）。

対象ファイル:
- `leptonica/`（Cargo.toml, src/lib.rs）— facade crate
- `CLAUDE.md`、`README.md`
- `docs/rebuild/`（prompt.md, overall-plan.md, feature-comparison.md, test-comparison.md）
- `docs/analysis/`（feature-comparison-c-vs-rust.md, test-comparison-c-vs-rust.md）

除外: `docs/plans/`内のセッション固有ファイル（50+個）。`000_overall-plan.md`はae0ee8bに既存のため対応不要。

コミット:
1. `feat: add leptonica facade crate with re-exports`
2. `docs: add project guidelines, README, and rebuild handoff documents`

## 7. Phase実行フロー

各Phaseで1つの`general-purpose`サブエージェントに以下A〜Gの全工程を委任する。

### A. ブランチ作成

```bash
git checkout main && git pull origin main
git checkout -b <branch> main
```

### B. テストスキャフォールディングコミット

> 注: 本計画ではテストに`#[ignore]`を付与し`cargo check`のみで検証するため、
> 厳密なTDD RED（テスト失敗を確認する）とは異なる。コンパイル可能なテスト・
> スタブを先行コミットする「テストスキャフォールディング」として位置づける。

**テスト作成**:
- C版リファレンステスト（`reference/leptonica/prog/*_reg.c`）を**仕様として参照**する
- 旧Rust版テスト（`backup/pre-rebuild`タグ）も参考にする
- 既存Rustテストの単純コピーではなく、C版の検証項目を確認した上で作成する
- テスト関数に`#[ignore = "not yet implemented"]`を付与する

**ドキュメント作成**:
- モジュールドキュメント（`//!`）: モジュールの目的・設計方針
- 公開APIのrustdoc（`///`）: 関数・型・メソッドの説明、引数、戻り値、使用例
- C版との対応関係を`# See also`等で記載

**スタブ**:
- コンパイルに必要な最小限（型定義、関数シグネチャ＋`todo!()`）
- `cargo check --workspace`でコンパイル確認後コミット

### C. TDD GREENコミット

- `backup/pre-rebuild`タグから実装ファイルを取得する
- スタブを実装に置換し、`#[ignore]`を除去する
- ドキュメントと実装の整合性を確認する
- `cargo check --workspace && cargo test --workspace`で検証後コミット

### D. REFACTORコミット（任意）

必要に応じてリファクタリングをコミットする。

### E. Push・PR作成

```bash
git push -u origin <branch>
gh pr create --title "..." --body "$(cat <<'EOF'
## Summary
...
## Test plan
...
EOF
)"
```

### F. Copilotレビュー対応（必須ゲート・省略禁止）

1. レビュー到着までポーリングする（60秒間隔、最大15分）
   ```bash
   gh pr checks <PR番号>
   gh api repos/{owner}/{repo}/pulls/{number}/reviews
   ```
2. レビューコメントを取得し、指摘事項を修正する
3. 修正をコミットしてpushする
4. Copilotレビューを再リクエストする
5. 再レビュー到着までポーリングする
6. 全指摘が解決されるまで2〜5を繰り返す
7. レビューが来ない場合は`/gh-actions-check`で診断する

### G. マージ・クリーンアップ

```bash
gh pr merge --merge          # --squash/--rebase禁止
git checkout main && git pull origin main
git branch -d <branch>
git push origin --delete <branch>
```

## 8. 検証

**各PR**:
- `cargo check --workspace` + `cargo test --workspace` 通過
- Copilotレビュー指摘全件解決

**全Phase完了後**:
- `git log --oneline --graph --first-parent`でマージ構造確認
- mainへの直接non-mergeコミットがないこと
- 全47回帰テストがpassすること

## 9. 想定結果

- 12マージコミット + 各2〜3内部コミット ≈ 36コミット（現状166 → 36に整理）
- 各マージコミットにLinus Torvalds形式の要約・理由・影響範囲を記載

## 10. 振り返り

### PR番号とPhaseの対応

| Phase | PR# | ブランチ | 備考 |
|-------|-----|---------|------|
| 1 | #9 | feat/workspace-setup | |
| 2 | #10 | feat/core-pix | |
| 3 | #11 | feat/core-data | |
| 4 | #14 | feat/core-ops | #12, #13はやり直し（後述） |
| 5 | #15 | feat/io | |
| 6 | #16 | feat/transform | |
| 7 | #17 | feat/filter | |
| 8 | #18 | feat/color | |
| 9 | #19 | feat/morph | |
| 10 | #20 | feat/region | |
| 11 | #21 | feat/recog | |
| 12 | #22 | docs/project-setup | |

### 原則違反と対処

PR #12（Phase 4）がCopilotレビュー到着前にマージされた（レビュー到着の4分2秒前）。
結果として8件のCopilot指摘が未対応のままマージされた。

PR #13で同一内容を再PRしマージしたが、指摘内容は反映されなかった。

対処として、mainを`cc06650`にリセットし、PR #14として再実行した。
全指摘を反映し、Copilotレビュー通過後にマージすることで原則を回復した。

### Copilotレビューの傾向

- PR作成直後に自動レビューが走る（3-10分で到着）
- 修正push後の再レビューは自動では走らないことが多い
- スタイル指摘（空行、コメント形式）が多い
- Rust 2024 edition固有の`r#gen`予約語について複数回指摘あり

### ライセンス変更

当初Apache-2.0としていたが、本家Leptonica（BSD-2-Clause）に合わせて
Phase 12で変更した。

### バックグラウンドテストのタイムアウト

サブエージェントが起動したバックグラウンドテストがタイムアウト
（exit code 143/SIGTERM）で失敗する事例が複数発生した。
テスト自体の問題ではなく、実行時間制限による強制終了が原因。
最終状態での`cargo test --workspace`は正常通過している。
