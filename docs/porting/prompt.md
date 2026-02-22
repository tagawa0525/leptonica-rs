# leptonica-rs 移植プロンプト

## あなたの役割

あなたはC版leptonicaライブラリ（画像処理、約240,000行）をRustに移植するエンジニアです。
`reference/leptonica/` にあるC版ソースコードを参照しながら、Rustのworkspaceとして移植を行います。

## 前回の失敗から学んだ教訓（最重要）

前回の移植で以下の問題が発生しました。**同じ失敗を絶対に繰り返さないでください。**

### 1. mainへの直接コミット（109/116コミット）

- feature branchを作らずmainに直接コミットした
- **対策**: mainには絶対にコミットしない。全ての変更はfeature branch → GitHub PR → マージの流れで統合する

### 2. TDDサイクルの圧縮

- テストと実装を同じコミットに含めた
- **対策**: RED（テスト先行、`#[ignore]`付き）とGREEN（実装）を必ず別コミットにする

### 3. 巨大コミット

- 19個のテストを1コミットに詰め込んだ（37ファイル、5679行）
- 51ファイルのunsafe削除を1コミットで実行
- **対策**: 1コミット = 1つの論理的変更。1ブランチ = 1つの機能

### 4. PRワークフロー不在

- GitHub PRを一切使わなかった。レビュー記録・品質ゲートが存在しない
- **対策**: 全ブランチをGitHub PRでマージし、Copilotレビューを受ける

### 5. 効率優先でプロセスを省略

- 「リファレンスがあるから」とTDD・PRを省略した
- **対策**: 効率を理由にプロセスを省略しない。省略したくなったらユーザーに相談する

---

## プロジェクト構成

```text
leptonica-rs/
├── Cargo.toml                    # workspace root
├── CLAUDE.md                     # 規約（後述）
├── reference/leptonica/          # C版ソース（git submodule、read-only参照）
├── crates/
│   ├── leptonica-core/           # Pix, Box, Numa, FPix等の基本データ構造
│   ├── leptonica-io/             # 画像I/O（PNG, JPEG, TIFF, GIF, WebP, BMP, PNM, JP2K, PDF, PS）
│   ├── leptonica-morph/          # 形態学演算（binary, grayscale, DWA, thinning）
│   ├── leptonica-transform/      # 幾何変換（rotate, scale, affine, projective, bilinear, shear, warp）
│   ├── leptonica-filter/         # フィルタリング（bilateral, rank, adaptmap, convolve, edge）
│   ├── leptonica-color/          # 色処理（segmentation, quantize, threshold, colorspace）
│   ├── leptonica-region/         # 領域解析（conncomp, ccbord, quadtree, watershed, maze）
│   ├── leptonica-recog/          # 認識（barcode, dewarp, baseline, pageseg, jbclass）
│   └── leptonica-test/           # テストユーティリティ
├── leptonica/                    # ファサードcrate（re-export）
├── tests/
│   └── data/images/              # テスト画像
└── docs/plans/                   # 実装計画書
```

### Crate間依存関係

```text
leptonica-recog → leptonica-region → leptonica-filter → leptonica-color → leptonica-transform → leptonica-morph → leptonica-io → leptonica-core
```

下位crateへの依存のみ許可。循環依存は禁止。

---

## 開発ルール（CLAUDE.md相当）

### Git

- **mainブランチに直接コミットしない**。必ずfeature branchを作成し、GitHub PRを経由してマージする
- ブランチ命名: `feat/<crate>-<機能>`, `test/<スコープ>`, `refactor/<スコープ>`, `docs/<スコープ>`
- コミットメッセージ: Conventional Commits形式、scopeにはcrate名を使用
- 1コミットには1つの論理的変更のみ含める
- PRではGitHub Copilotの自動レビューを待ち、指摘事項を修正してからマージする
- マージ後のブランチは速やかに削除する
- マージコミットにはLinus Torvalds方式で変更の要約・理由・影響範囲を記載する
- Co-Authored-By: Claude <model_name> <noreply@anthropic.com> を全コミットに付与する

### TDD

テストと実装を同時にコミットしない。以下のサイクルをコミット履歴に残す:

1. **RED**: テストを先に書いてコミット（`#[ignore = "not yet implemented"]` を付与）
2. **GREEN**: 実装を追加し、`#[ignore]`を除去してテストを通すコミット
3. **REFACTOR**: 必要に応じてリファクタリング（別コミット）

### 回帰テスト

- C版の `reference/leptonica/prog/*_reg.c` に対応するRustテストを作成
- テストデータは `tests/data/images/` に配置
- `tests/regout/` は `.gitignore` 対象（テスト出力）

### unsafe

- unsafeの使用は原則禁止
- やむを得ない場合はコミットメッセージに理由を明記し、最小限に留める

### 禁止事項

- 作業効率を理由にプロセス手順（TDD、PRワークフロー、レビュー確認）を省略しない
- 「リファレンスがあるから簡単」「変更が少ないから」で手順を飛ばさない
- 省略したくなったらユーザーに相談する

---

## 1ブランチあたりのワークフロー

各機能ブランチは以下のコミット構成を取る:

```
feat/<crate>-<機能>
  ├── docs: add <機能> implementation plan
  ├── test(<crate>): add <機能> regression test (RED)    ← #[ignore]付き
  ├── feat(<crate>): implement <機能>                     ← #[ignore]除去、テスト通過
  ├── fix/refactor(<crate>): ...                          ← 該当時のみ
  └── docs: update progress for <機能>
→ git push → gh pr create → Copilotレビュー → 修正 → gh pr merge
```

### 具体的な手順

```bash
# 1. ブランチ作成
git checkout main && git pull
git checkout -b feat/<crate>-<機能>

# 2. 計画書コミット
# docs/plans/<機能>.md を作成
git add docs/plans/
git commit -m "docs: add <機能> implementation plan

Co-Authored-By: ..."

# 3. テスト(RED)コミット
# - テストファイルを作成（C版の prog/<xxx>_reg.c を参考）
# - 全テスト関数に #[ignore = "not yet implemented"] を付与
# - cargo check --workspace が通ることを確認（コンパイルは通るがテストはskip）
git add crates/<crate>/tests/
git commit -m "test(<crate>): add <機能> regression test (RED)

All tests marked with #[ignore = \"not yet implemented\"].

Co-Authored-By: ..."

# 4. 実装(GREEN)コミット
# - C版ソースを参照してRust実装を作成
# - テストから #[ignore] を除去
# - cargo test -p <crate> が全て通ることを確認
# - cargo clippy --workspace が通ることを確認
git add crates/<crate>/src/ crates/<crate>/tests/
git commit -m "feat(<crate>): implement <機能>

<実装内容の説明>

Co-Authored-By: ..."

# 5. 進捗更新コミット
git add docs/plans/
git commit -m "docs: update progress for <機能>

Co-Authored-By: ..."

# 6. push & PR作成
git push -u origin feat/<crate>-<機能>
gh pr create --base main \
  --title "feat(<crate>): implement <機能>" \
  --body "$(cat <<'PREOF'
## Summary
- <変更内容を箇条書き>
- TDD cycle: RED → GREEN

## Changes
- `docs/plans/<機能>.md`: 実装計画書
- `crates/<crate>/tests/<xxx>_reg.rs`: 回帰テスト
- `crates/<crate>/src/<xxx>.rs`: 実装

## Test plan
- [ ] cargo check --workspace
- [ ] cargo test -p <crate>
- [ ] cargo clippy --workspace
PREOF
)"

# 7. Copilotレビュー待機 → 指摘対応
# gh pr checks <PR番号> でステータス確認
# gh pr view <PR番号> --comments でレビュー確認
# 指摘があれば修正コミット → push

# 8. マージ（詳細メッセージ付き）
gh pr merge <PR番号> --merge \
  --subject "Merge feat/<crate>-<機能>: <概要>" \
  --body "$(cat <<'MREOF'
<機能>: <概要説明>

* docs: 実装計画書を追加
* test(<crate>): 回帰テストを追加 (RED)
* feat(<crate>): <機能>を実装 (GREEN)

C版 leptonica の <対応するC関数> に相当。
<影響範囲の説明>
MREOF
)"

# 9. ブランチ削除
git checkout main && git pull
git branch -d feat/<crate>-<機能>
```

---

## 実装フェーズと順序

依存関係に従い、以下の順序で実装する。各Phase内は順番通りに実行すること。

### Phase 0: プロジェクト基盤

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 0-1 | feat/workspace-setup | Workspace構造、全crateスタブ、ファサード | — |
| 0-2 | feat/core-pix | Pix構造体（画像コンテナ） | pix1.c, pix2.c, pix_internal.h |
| 0-3 | feat/core-box | Box/Boxa（矩形領域） | boxbasic.c, boxfunc1.c |
| 0-4 | feat/core-pta | Pta（点配列） | ptabasic.c, ptafunc1.c |
| 0-5 | feat/core-colormap | PixColormap（カラーパレット） | colormap.c |

Phase 0完了条件: `cargo check --workspace` が通る。core crateの基本型テストが全てパスする。

### Phase 1: I/O基盤

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 1-1 | feat/io-png | PNG読み書き | pngio.c |
| 1-2 | feat/io-jpeg | JPEG読み書き | jpegio.c |
| 1-3 | feat/io-bmp | BMP読み書き | bmpio.c |
| 1-4 | feat/io-pnm | PNM読み書き | pnmio.c |
| 1-5 | feat/io-tiff | TIFF読み書き | tiffio.c |
| 1-6 | feat/io-gif | GIF読み書き | gifio.c |
| 1-7 | feat/io-webp | WebP読み書き | webpio.c, webpiostub.c |

Phase 1完了条件: 各フォーマットのread/writeテストがパスする。

### Phase 2: Core拡張

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 2-1 | feat/core-pixa | Pixa/Pixaa（画像配列） | pixabasic.c, pixafunc1.c |
| 2-2 | feat/core-numa | Numa/Numaa（数値配列） | numabasic.c, numafunc1.c |
| 2-3 | feat/core-fpix | FPix（浮動小数点画像） | fpix1.c, fpix2.c |
| 2-4 | feat/core-sarray | Sarray（文字列配列） | sarray1.c, sarray2.c |

### Phase 3: Morph基盤 + Transform基盤

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 3-1 | feat/morph-binary | Binary morphology | morph.c, morphapp.c |
| 3-2 | feat/morph-grayscale | Grayscale morphology | graymorph.c |
| 3-3 | feat/transform-rotate | 回転（直交+任意角度） | rotate.c, rotateam.c, rotateorth.c |
| 3-4 | feat/transform-scale | スケーリング | scale1.c, scale2.c |

### Phase 4: Transform拡張

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 4-1 | feat/transform-affine | アフィン変換 | affine.c, affinecompose.c |
| 4-2 | feat/transform-shear | シアー変換 | shear.c |
| 4-3 | feat/transform-bilinear | バイリニア変換 | bilinear.c |
| 4-4 | feat/transform-projective | 射影変換 | projective.c |
| 4-5 | feat/transform-warper | ワーピング | warper.c |

### Phase 5: Filter

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 5-1 | feat/filter-convolve | 畳み込み・カーネル | convolve.c, kernel.c |
| 5-2 | feat/filter-edge | エッジ検出 | edge.c |
| 5-3 | feat/filter-bilateral | バイラテラルフィルタ | bilateral.c |
| 5-4 | feat/filter-rank | ランクフィルタ | rank.c |
| 5-5 | feat/filter-adaptmap | 適応マッピング | adaptmap.c |

### Phase 6: Color

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 6-1 | feat/color-colorspace | 色空間変換 | colorspace.c |
| 6-2 | feat/color-threshold | 二値化・閾値処理 | binarize.c |
| 6-3 | feat/color-quantize | 色量子化 | colorquant1.c, colorquant2.c |
| 6-4 | feat/color-segment | 色セグメンテーション | colorseg.c |
| 6-5 | feat/color-colorfill | カラーフィル | colorfill.c |
| 6-6 | feat/color-coloring | カラー化 | coloring.c |
| 6-7 | feat/color-histogram | ヒストグラム分析 | colorcontent.c |

### Phase 7: Pix操作拡張

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 7-1 | feat/pix-arith | 算術操作 | pixarith.c |
| 7-2 | feat/pix-rop | ラスタ操作 | rop.c, roplowlevel.c |
| 7-3 | feat/pix-blend | ブレンド・合成 | blend.c |
| 7-4 | feat/pix-compare | 画像比較 | compare.c |
| 7-5 | feat/pix-graphics | グラフィックス描画 | graphics.c, paintcmap.c |

### Phase 8: Region

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 8-1 | feat/region-conncomp | 連結成分 | conncomp.c |
| 8-2 | feat/region-label | ラベリング | label.c |
| 8-3 | feat/region-seedfill | シードフィル | seedfill.c |
| 8-4 | feat/region-watershed | 分水嶺 | watershed.c |
| 8-5 | feat/region-ccbord | 境界トレース | ccbord.c |
| 8-6 | feat/region-quadtree | 四分木 | quadtree.c |
| 8-7 | feat/region-maze | 迷路生成・解法 | maze.c |

### Phase 9: Morph拡張

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 9-1 | feat/morph-morphseq | 形態学シーケンス | morphseq.c |
| 9-2 | feat/morph-colormorph | カラー形態学 | colormorph.c |
| 9-3 | feat/morph-ccthin | 細線化 | ccthin.c |
| 9-4 | feat/morph-dwa | DWA高速形態学 | dwacomp.c, dwacomb.c |

### Phase 10: Recog

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 10-1 | feat/recog-skew | スキュー検出 | skew.c |
| 10-2 | feat/recog-baseline | ベースライン検出 | baseline.c |
| 10-3 | feat/recog-pageseg | ページセグメンテーション | pageseg.c |
| 10-4 | feat/recog-dewarp | デワープ | dewarp1.c, dewarp2.c |
| 10-5 | feat/recog-jbclass | JB分類 | jbclass.c |
| 10-6 | feat/recog-barcode | バーコード | readbarcode.c, bardecode.c |

### Phase 11: IO拡張

| # | ブランチ | 内容 | C版参照ファイル |
|---|---|---|---|
| 11-1 | feat/io-jp2k | JPEG 2000 | jp2kio.c |
| 11-2 | feat/io-pdf | PDF出力 | pdfio1.c, pdfio2.c |
| 11-3 | feat/io-ps | PostScript出力 | psio1.c, psio2.c |

---

## 前回の成功体験（必ず踏襲すること）

前回の移植はプロセス面では失敗したが、設計・実装面では優れたパターンを多く生み出した。
これらは実証済みであり、再発明せずにそのまま採用すること。

### 1. Pix/PixMut二層メモリモデル（最重要）

```rust
// 不変層: Arc<PixData>で安価なclone・参照カウント
#[derive(Debug, Clone)]
pub struct Pix {
    inner: Arc<PixData>,
}

// 可変層: PixDataを直接所有し、mutation可能
pub struct PixMut {
    inner: PixData,
}

// 参照カウントが1なら zero-copy で変換、それ以外はコピー
impl Pix {
    pub fn try_into_mut(self) -> std::result::Result<PixMut, Self> { ... }
    pub fn to_mut(&self) -> PixMut { ... }
}
```

- `RefCell`や`Mutex`を使わない。`Arc::try_unwrap()`で安全に可変アクセスを提供
- 画像処理パイプラインでの連鎖操作に適合する設計
- **この設計を変更しないこと**

### 2. Checked + Unchecked ピクセルアクセスペア

```rust
impl Pix {
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<u32> { ... }      // 安全（境界チェックあり）
    pub fn get_pixel_unchecked(&self, x: u32, y: u32) -> u32 { ... }    // 高速（チェックなし）
}
impl PixMut {
    pub fn set_pixel(&mut self, x: u32, y: u32, val: u32) -> Result<()> { ... }
    pub fn set_pixel_unchecked(&mut self, x: u32, y: u32, val: u32) { ... }
}
```

- デフォルトは安全なAPI、パフォーマンスが必要な内部ループ用に`_unchecked`を提供
- `#[inline]`をホットパス関数に付与

### 3. thiserrorによる構造化エラー

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid image dimensions: {width}x{height}")]
    InvalidDimension { width: u32, height: u32 },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    // ...
}
pub type Result<T> = std::result::Result<T, Error>;
```

- 文字列ベースのエラーは使わない。構造体バリアントでパターンマッチ可能にする
- `#[from]`で標準エラー型からの自動変換

### 4. 3モード回帰テストインフラ（leptonica-test crate）

```rust
pub enum RegTestMode { Generate, Compare, Display }
pub struct RegParams { test_name, index, mode, success, failures }
```

- `REGTEST_MODE=generate` でゴールデンファイル生成、`compare`（デフォルト）で比較、`display`で目視確認
- `compare_values()`, `compare_pix()`, `write_pix_and_check()` でテストを構造化
- テストインフラを専用crateに分離し、全テストスイートで再利用

### 5. 回帰テストの構造パターン

各テスト関数は「前提条件確認 → 操作 → 不変条件検証」の3段構成:

```rust
#[test]
fn binmorph1_reg() {
    let mut rp = RegParams::new("binmorph1");
    let pixs = load_test_image("feyn-fract.tif").unwrap();
    assert_eq!(pixs.depth(), PixelDepth::Bit1);           // 前提条件
    let dilated = dilate_brick(&pixs, 21, 15).unwrap();   // 操作
    assert!(dilated.count_pixels() >= pixs.count_pixels()); // 不変条件
    rp.compare_values(1.0, ...);
    assert!(rp.cleanup());
}
```

- ハードコードされたピクセル数ではなく、不変条件（単調性など）で検証
- 必ず`rp.cleanup()`を呼びassertする

### 6. unsafeの最小化（3箇所のみ）

- `Box`, `Arc`, `Vec`を第一選択とし、rawポインタを回避
- unsafeが必要なのはOctreeカラー量子化のツリー走査のみ（再帰的な`*mut`参照）
- **unsafeを新規追加するときは必ずコミットメッセージに理由を記載**

### 7. モジュール分割: 1ファイル100-200行

```text
pix/
├── mod.rs       # Pix/PixMut構造体定義
├── access.rs    # get_pixel/set_pixel
├── arith.rs     # 算術操作
├── blend.rs     # ブレンド
├── convert.rs   # 深度変換
├── rop.rs       # ラスタ操作
└── ...
```

- implブロックを複数ファイルに分散し、各ファイルの責務を明確にする
- 関連する型（enum, struct）はその操作と同じファイルに配置

### 8. Workspace構成

```toml
[workspace]
resolver = "2"
[workspace.package]
version = "0.1.0"
edition = "2024"
[workspace.dependencies]
leptonica-core = { path = "crates/leptonica-core" }
thiserror = "2.0.18"
```

- バージョンと外部依存はworkspaceレベルで一元管理
- 各crateは`version.workspace = true`で参照
- `[dev-dependencies]`にleptonica-testを指定しプロダクション依存に含めない

### 9. 計画書駆動開発

実装前に`docs/plans/<機能>.md`を作成し、以下を含める:

- Status: PLANNED → IN_PROGRESS → IMPLEMENTED
- C版の対応ファイル・関数
- RustのAPI設計（具体的なシグネチャ）
- CパターンからRustへのマッピング

---

## コンテキスト管理とworktree戦略

このプロジェクトは52以上のfeature branchを扱う大規模作業です。
コンテキストウィンドウの消費を最小限に抑え、効率的に作業を進めるための戦略。

### コンテキスト節約の原則

1. **ファイル全体を読まない**: 必要な関数・セクションだけを読む。C版ソースは1ファイル1000行超が多いので、関連する関数だけを参照する
2. **cargo出力を制限する**: `2>&1 | tail -20` でエラーの末尾だけ確認する。全出力をコンテキストに入れない
3. **git log/diffの出力を制限する**: `--oneline`, `| head -N` を活用
4. **重複する確認を避ける**: 直前に成功した`cargo check`の後に同じコマンドを再実行しない
5. **Phaseの区切りでセッションを分ける**: 1セッションで全Phaseを完了しようとしない

### worktreeの活用

**feature branch作業にworktreeを使う。** mainのチェックアウト状態を維持したまま並行作業できる。

```bash
# worktree作成（feature branch用）
git worktree add ../leptonica-rs-work feat/<crate>-<機能>

# 作業は専用ディレクトリで行う
cd ../leptonica-rs-work
# ... コミット、テスト ...

# 作業完了後にworktreeを削除
git worktree remove ../leptonica-rs-work
```

利点:

- mainが常にクリーンな状態で残る（`git stash`不要）
- 別の作業を中断せずにfeature branchに切り替えられる
- `cargo check`のキャッシュが干渉しない

### セッション間の引き継ぎ

コンテキスト上限に達しそうになったら、以下を出力して中断する:

```markdown
## 進捗報告
- 完了済み: Phase 0 (全5ブランチ), Phase 1 (#1-1〜#1-5)
- 作業中: Phase 1 #1-6 feat/io-gif — REDコミット完了、GREEN実装中
- 次のセッションの開始点: Phase 1 #1-6 のGREENコミットから
- 未解決の問題: なし
- マージ済みPR: #3〜#15
```

次のセッションではこの報告をプロンプトに含め、続きから再開する。

### 各ブランチの作業時間の目安感

- Phase 0の各ブランチ: 基盤なのでコード量が多い。1ブランチで相当量のコンテキストを消費する
- Phase 1以降の各ブランチ: C版を参照して移植するパターンが確立済み。効率的に進められる
- **1セッションで1 Phase完了を目標とする。無理なら途中で区切る**

---

## 設計方針

### メモリ管理

| C版パターン | Rust実装 |
|---|---|
| refcount + clone | `Arc<T>` |
| copy | `Clone` trait |
| insert (所有権移動) | move semantics |
| NULL | `Option<T>` |
| エラーコード | `Result<T, Error>` |

### API設計

- C版の `pixXxx(pix, ...)` → Rust `pix.xxx(...)` メソッド
- C版の `pixCreate(w, h, d)` → Rust `Pix::new(w, h, d)`
- 出力先パラメータ → 戻り値 `Result<T>`
- フラグ引数 → enum型

### テスト画像

テスト画像はC版の `prog/` ディレクトリから取得し、`tests/data/images/` に配置する。
主な画像: feyn.tif, test1.png, test8.jpg, test24.jpg, weasel2.png, rabi.png など。

---

## 検証基準

各Phaseの完了時に以下を確認する:

```bash
cargo check --workspace        # コンパイル通過
cargo test --workspace         # 全テストパス
cargo clippy --workspace       # 警告なし
```

### 最終検証

```bash
# first-parentログが全てマージコミット（初期を除く）
git log --oneline --first-parent --no-merges main
# → 初期セットアップの数件のみ

# 全PRがマージ済み
gh pr list --state merged --base main | wc -l

# unsafeの数を確認（最小限）
grep -r "unsafe" crates/*/src/ --include="*.rs" | grep -v "// " | wc -l
```

---

## 作業の進め方

1. **Phase 0から順番に進める**。Phaseを飛ばさない
2. **1ブランチずつ完了させる**。複数ブランチを同時に進めない
3. **各ブランチでTDDサイクルを厳守する**。RED → GREEN → (REFACTOR)
4. **PRでCopilotレビューを受ける**。指摘は修正する
5. **マージコミットに詳細を書く**。何が変わったか、なぜか、影響範囲は
6. **困ったらユーザーに相談する**。プロセスを省略する判断を独断でしない
7. **worktreeを活用する**。mainを汚さず、feature branch作業を隔離する
8. **コンテキスト上限に達しそうになったら**、進捗報告を出力して中断する。次のセッションで続きから再開できるよう、完了済みPhaseと次のブランチを明記する
