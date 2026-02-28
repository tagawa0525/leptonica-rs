# leptonica-rs

C版leptonicaのRust移植。単一クレート `leptonica` で公開。Rust edition 2024。
devプロファイルは `opt-level = 1`（テスト実行 259秒→9.7秒）。

## ビルド・テスト・リント

```bash
cargo check --all-features
cargo test --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
cargo test convolve_reg  # 特定テスト
```

### 回帰テストモード

`REGTEST_MODE` 環境変数: `generate`（golden生成） / `compare`（比較、デフォルト） / `display`（比較なし）

## モジュール構成

```text
src/
├── lib.rs          # 公開API入口（core型のルート再エクスポート）
├── core/           # 基本データ構造 (Pix, Box, Numa, FPix, Pta, Pixa, Colormap, SArray)
│   └── pixel.rs    # RGBAピクセル操作 (compose_rgba, extract_rgb 等)
├── io/             # 画像I/O (PNG, JPEG, TIFF, BMP, GIF, WebP, PDF, PS)
├── transform/      # 幾何変換 (回転, スケール, アフィン, 射影, バイリニア)
├── morph/          # 形態学演算 (膨張, 収縮, オープニング, クロージング, DWA)
├── filter/         # フィルタリング (畳み込み, エッジ, 二値化, ランク, 適応マップ)
├── color/          # 色処理 (量子化, 二値化, 色空間変換, セグメンテーション)
├── region/         # 領域解析 (連結成分, ラベリング, 流域分割, 迷路)
└── recog/          # 文字認識・バーコード・デワープ・JBIG2分類
```

## PRワークフロー

以下のGit/TDDルールをプロジェクト標準として適用する。

### コミット構成

1. RED: テスト（`#[ignore = "not yet implemented"]` 付き）
2. GREEN: 実装（`#[ignore]` 除去）
3. REFACTOR: 必要に応じて
4. 全テスト・clippy・fmt通過を確認

### PR作成〜マージ

1. PR作成
2. `/gh-actions-check` でCopilotレビューワークフローが `completed/success` になるまで待つ
3. `/gh-pr-review` でコメント確認・対応
4. **レビュー修正は独立した `fix(<scope>):` コミットで積む（RED/GREENに混入させない）**
5. push後の再レビューサイクルも完了を確認（同じ手順を繰り返す）
6. `docs/plans/` の進捗ステータスや `docs/porting/` の実装状況を更新する（`docs:` コミット）
7. 全チェック通過後 `/gh-pr-merge --merge`

### PRやり直し時

- 元のRED/GREENをそのままcherry-pick（内容を改変しない）
- 過去PRのレビュー修正は独立 `fix(<scope>):` コミットとして積む
- 異なるPRの修正は別コミットにする

### 規約

- ブランチ命名: `feat/<module>-<機能>`, `test/<スコープ>`, `refactor/<スコープ>`, `docs/<スコープ>`
- コミット: Conventional Commits、scopeにモジュール名
- マージコミット: `## Why` / `## What` / `## Impact` セクション
- 計画書 (`docs/plans/`) を実装着手前にコミットすること

## テスト

- 回帰テスト: C版 `reference/leptonica/prog/*_reg.c` に対応
- テストデータ: `tests/data/images/`, golden: `tests/golden/`, 出力: `tests/regout/`
- インフラ: `tests/common/`（`RegParams`, `compare_values()`, `compare_pix()` 等）

### テストのディレクトリ構造

```text
tests/
├── common/          # ヘルパー（RegParams, load_test_image 等）
├── core/            # main.rs + 32テストモジュール
├── io/              # main.rs + 17テストモジュール
├── morph/           # main.rs + 18テストモジュール
├── transform/       # main.rs + 21テストモジュール
├── filter/          # main.rs + 19テストモジュール
├── color/           # main.rs + 28テストモジュール
├── region/          # main.rs + 13テストモジュール
└── recog/           # main.rs + 14テストモジュール
```

各ディレクトリの `main.rs` がテストバイナリのエントリポイント。モジュール単位で実行可能:

```bash
cargo test --test core       # core テストのみ
cargo test --test io         # io テストのみ
cargo test convolve_reg      # テスト名でフィルタ（全バイナリ横断）
```

## 設計原則

### Pix/PixMut

```rust
pub struct Pix { inner: Arc<PixData> }   // 不変・安価なclone
pub struct PixMut { inner: PixData }     // 可変・直接所有
```

- `try_into_mut()`: refcount=1ならzero-copy、他はコピー。`pixmut.into()` で戻す
- `RefCell`/`Mutex` 不使用
- ピクセル: 32bit `0xRRGGBBAA`。`core::pixel::compose_rgba()` / `extract_rgba()` を使う

### エラー処理

- `thiserror` 構造化enum。core → `Error`/`Result<T>`、他 → `<Domain>Error`/`<Domain>Result<T>`
- `#[from] crate::core::Error` で自動伝播

### コーディング

- `get_pixel()` / `set_pixel()`: 安全。`_unchecked`: 内部ループ用
- 1関数100-200行目安。implブロックは機能別に複数ファイルに分散
- unsafe原則禁止

## 計画書

`docs/plans/NNN_<機能名>.md`（NNN = Phase番号×100 + 連番）。Status: PLANNED → IN_PROGRESS → IMPLEMENTED。C版の対応ファイル・関数を明記。

## 引き継ぎ資料

`docs/porting/`: prompt.md, overall-plan.md, feature-comparison.md, test-comparison.md

C版ソース: `reference/leptonica/`（`git submodule update --init`）
