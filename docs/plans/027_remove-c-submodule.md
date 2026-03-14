# C版leptonica git submoduleの除去

## Context

C版leptonicaは `reference/leptonica/` にgit submoduleとして配置されていたが、移植作業がほぼ完了し（関数カバレッジ82%、回帰テスト100%）、日常的な参照が不要になった。submoduleを除去してリポジトリ構成を簡素化する。

ローカルで必要な場合は手動で `git clone` できるよう `.gitignore` にパスを追加する。

## 変更対象ファイル

### 1. Git submodule除去（コマンド）

```bash
git rm reference/leptonica          # gitの追跡から除去
rm .gitmodules                      # 他のsubmoduleなし → ファイル削除
git add .gitmodules
# .git/config のsubmoduleエントリは git rm で自動除去される
# .git/modules/reference/leptonica はローカルキャッシュ → rm -rf で手動削除
```

### 2. `.gitignore` — `reference/` を追加

```text
/reference/
```

ローカルにC版ソースを手動cloneしても追跡されないようにする。

### 3. `README.md` (L16, L58-65)

- L16: submoduleとして含まれている旨の記述を削除、代わりにC版公式リポジトリへのリンクに変更
- L58-65: 「Fetching the C Reference」セクションを削除

### 4. `README.ja.md` (L16, L58-65)

- L16: 同上（日本語版）
- L58-65: 「C版リファレンスの取得」セクションを削除

### 5. `CLAUDE.md` (L95, L101)

- `reference/leptonica/prog/*_reg.c` への言及を削除または代替表現に変更
- `git submodule update --init` の記述を削除
- `examples/compare_golden.rs` + `scripts/golden_map.tsv` の記述は残す（Cソースに依存しない）

### 6. `scripts/audit-regression-tests.py` (L23, L31, L102-108)

C版ソースをスキャンするスクリプト。submoduleがなくなると動作しない。
→ エラーメッセージを「submoduleをチェックアウトしてください」から「C版ソースを手動cloneしてください」に変更。

### 7. テスト・ソースファイルのコメント（163箇所）

テストファイル161件、ソースファイル1件に `reference/leptonica/` パスがコメントとしてハードコードされている。

パターン例:

```rust
//! C version: `reference/leptonica/prog/convolve_reg.c`
//! C Leptonica: `reference/leptonica/prog/boxa1_reg.c`
//! C reference: reference/leptonica/prog/watershed_reg.c
//! C版: reference/leptonica/src/adaptmap.c
// C reference: reference/leptonica/src/scale2.c
```

**対応**: `reference/leptonica/` プレフィックスを一括削除。結果:

```rust
//! C version: `prog/convolve_reg.c`
//! C Leptonica: `prog/boxa1_reg.c`
// C reference: src/scale2.c
```

leptonicaプロジェクト内の相対パスとして十分明確であり、submoduleのローカルパスに依存しなくなる。`sed` で一括置換:

```bash
find tests/ src/ -name '*.rs' -exec sed -i 's|reference/leptonica/||g' {} +
```

### 8. `scripts/verify_*.c` (3ファイル)

C版leptonicaのヘッダに依存するCソース。submodule除去後も、手動clone環境で使う想定。
→ 変更不要（ファイル冒頭コメントの "reference/leptonica/prog/" は実行ディレクトリの案内として正確）

### 9. `examples/compare_golden.rs` (L9-19コメント)

ビルド手順のコメントに `cd reference/leptonica` がある。
→ 変更不要（手動clone先のパス指定として引き続き有効）

### 10. `docs/plans/`, `docs/porting/` — 変更不要

完了済み計画・移植資料は歴史的記録。パスの参照は当時の事実として正確なので改変しない。

## コミット構成

1コミットで実施: `chore: remove C leptonica git submodule`

## 検証

```bash
# submoduleが除去されていること
git submodule status  # 出力なし
cat .gitmodules       # ファイルなし

# ビルド・テストが影響を受けないこと
cargo check --all-features
cargo test --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check

# .gitignore が reference/ を無視すること
mkdir -p reference/test && git status  # reference/ が表示されない
rm -rf reference/test
```
