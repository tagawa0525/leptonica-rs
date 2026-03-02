# Golden File 除去 + ピクセルハッシュ manifest 導入

## Context

`tests/golden/` に 204 ファイル / 54MB のバイナリ画像が git 管理されている。
Copilot code review のハング、git history 肥大化を引き起こしている。

バイナリを git から完全除去し、ピクセルハッシュ manifest（テキスト）で
CI の出力変化検出を維持する。`8178fd3` から履歴を再構成する。

## 要件

| # | 要件                   | 実現方法                                              |
| - | ---------------------- | ----------------------------------------------------- |
| A | CI でバグ検出          | compare_values / compare_pix（既存）                  |
| B | CI で出力変化検出      | ピクセルハッシュ manifest（テキスト、git 管理）       |
| C | ローカルで変更前後比較 | REGTEST_MODE=generate/compare（golden は .gitignore） |
| D | C 版出力との一致確認   | compare_golden.rs + golden_map.tsv（ローカル）        |
| E | デバッグ時に実画像確認 | tests/regout/ + ローカル golden                       |

プロダクションコード（src/）への変更ゼロ。

## 方針: 8178fd3 から履歴再構成

`8178fd3`（回帰テスト網羅性監査）以降の 42 コミット（7 merge + 35 通常）を
再構成する。先に manifest インフラを構築し、その上にテスト強化を積む。

### 再構成対象の PR 構造（古い順）

```text
87fae6a  Merge: filter回帰テスト強化（PR 1/8）
  └ 023265a..9a8a4e0 test(filter) + d21f84b fix + ac37d88 scripts + docs

4d41e17  Merge: Sobelエッジフィルタ修正
  └ 8ed2d68, 1f9f03d

c19e380  Merge: 1bpp極性修正
  └ 33a8c2d, 8ae7c3e, 672c695, 8cbb383

6a316e5  Merge: blockconv_gray修正
  └ fbaf51c

3d4f4e6  Merge: Phase 3 docs更新
  └ cfe9161, 9bf2ce0

a6401c4  Merge: dewarpテスト修正
  └ 785cca2, e2cae5f, c697c88

d569d70  Merge: morph回帰テスト強化（PR 2/8）
  └ 0525c2b..849a6eb test(morph) + scripts + docs
```

### コミット分類

**A. clean cherry-pick（golden 無関係、そのまま適用）: 24 件**

- fix: 8ed2d68, 1f9f03d, 33a8c2d, 8ae7c3e, 672c695, 8cbb383,

  fbaf51c, 785cca2, e2cae5f, c697c88, 00cbc83

- scripts: ac37d88, c85d314, 2023a28, b793931, 849a6eb, ae4045c
- docs: 24336a7, b916a2b, cfe9161, 9bf2ce0, bfe82e0, 8cbf6d6

**B. 修正が必要（golden ファイル部分を除去）: 9 件**

- test(filter): 023265a, 1efb6e0, 1520f2c, 6a368e7, 8432966, cb1d49e, 73c3760, 9a8a4e0
- test(morph): 0525c2b

**C. スキップ（golden ファイル生成/再生成のみ）: 2 件**

- 7bf9f79 feat(morph): generate golden files
- dc115d6 fix(morph): regenerate ccthin1/ccthin2 golden files

**D. 修正が必要（golden ファイル変更を含む fix）: 1 件**

- d21f84b fix(filter): use 0-valued holes（golden 更新部分を除去）

**E. merge commit（--no-ff で再作成）: 7 件**

## 実行手順

### Phase 0: 準備

```bash
gh pr close 268
git checkout main && git pull
```

### Phase 1: manifest インフラ構築

`8178fd3` から新ブランチを作成し、以下を実装:

1. `tests/common/params.rs` にハッシュ計算 + manifest 管理を追加
2. `.gitignore` に `/tests/golden/` を追加
3. 既存の boxa3 golden 3 ファイルを manifest エントリに変換
4. `tests/golden/` の 3 ファイルを `git rm`
5. `examples/compare_golden.rs` の find_file() に gif/webp 追加

**params.rs の変更詳細:**

```rust
// ピクセルハッシュ（FNV-1a、外部依存なし）
fn pixel_content_hash(pix: &Pix) -> u64 { ... }
fn data_content_hash(data: &[u8]) -> u64 { ... }

// manifest 管理（OnceLock<HashMap>）
fn load_manifest() -> &'static HashMap<String, u64> { ... }
fn update_manifest(name: &str, hash: u64) { ... }
fn save_manifest() { ... }  // Generate モード終了時に書き出し

// check_file() を hash ベースに変更
fn check_file(&mut self, local_path: &str, hash: u64) -> TestResult<()> {
    match self.mode {
        Generate => { copy to golden + update manifest }
        Compare => { lookup manifest → match/mismatch/skip }
        Display => {}
    }
}
```

### Phase 2: fix コミットを cherry-pick

golden 無関係の fix を時系列順に cherry-pick。各 PR 単位で --no-ff merge:

```text
Sobel修正:       8ed2d68, 1f9f03d → merge
1bpp極性修正:    33a8c2d, 8ae7c3e, 672c695, 8cbb383 → merge
blockconv修正:   fbaf51c → merge
dewarp修正:      785cca2, e2cae5f, c697c88 → merge
```

### Phase 3: filter 回帰テスト強化を再適用

旧 PR 1/8 の内容を manifest 方式で再構成:

1. テストコード変更を cherry-pick（golden ファイル部分を除外）:

   023265a, 1efb6e0, 1520f2c, 6a368e7, 8432966, cb1d49e, 73c3760, 9a8a4e0

2. d21f84b fix(filter) を cherry-pick（golden 部分除外）
3. scripts/docs コミットを cherry-pick:

   ac37d88, c85d314, 2023a28, 00cbc83, 24336a7, b916a2b

4. `REGTEST_MODE=generate cargo test --test filter` で manifest 生成
5. manifest をコミット
6. --no-ff merge

### Phase 4: docs 更新を cherry-pick

cfe9161, 9bf2ce0 → merge

### Phase 5: morph 回帰テスト強化を再適用

旧 PR 2/8 の内容を manifest 方式で再構成:

1. 0525c2b test(morph) を cherry-pick（golden 部分除外）
2. 7bf9f79, dc115d6 はスキップ（golden 生成のみ）
3. b793931, 849a6eb, ae4045c (scripts), bfe82e0, 8cbf6d6 (docs) を cherry-pick
4. `REGTEST_MODE=generate cargo test --test morph` で manifest 生成
5. manifest をコミット
6. --no-ff merge

### Phase 6: main を置き換え

```bash
git branch -m main old-main
git branch -m new-main main
git push --force origin main
git branch -D old-main
```

## 変更対象ファイル

| ファイル                     | 変更内容                                       |
| ---------------------------- | ---------------------------------------------- |
| `tests/common/params.rs`     | ハッシュ計算 + manifest 管理 + check_file 変更 |
| `tests/golden_manifest.tsv`  | 新規: ハッシュ manifest                        |
| `.gitignore`                 | `/tests/golden/` 追加                          |
| `examples/compare_golden.rs` | find_file() に gif/webp 追加                   |

## 検証

各 Phase 完了時:

- `cargo test --all-features`
- `cargo clippy --all-features --all-targets -- -D warnings`
- `cargo fmt --all -- --check`

最終検証:

- manifest エントリ数 = write_pix_and_check + write_data_and_check 呼び出し総数
- `git log --all -- tests/golden/` が空（golden ファイルが履歴に存在しない）
- `du -sh .git` でリポジトリサイズが大幅削減されていること
