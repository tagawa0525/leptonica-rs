# leptonica-rs

C版leptonicaのRust移植プロジェクト。Workspace構成で機能別crateに分割。

## Crate構成

- **leptonica-core**: Pix, Box, Numa, FPix等の基本データ構造
- **leptonica-transform**: 幾何変換（回転、アフィン、射影等）
- **leptonica-filter**: フィルタリング（bilateral, rank等）
- **leptonica-color**: 色処理（セグメンテーション等）
- **leptonica-morph**: 形態学演算（binary, grayscale, DWA等）
- **leptonica-region**: 領域解析（ccbord, quadtree, maze等）
- **leptonica-recog**: 文字認識・バーコード・デワープ
- **leptonica-io**: 画像I/O（PNG, JPEG, TIFF, WebP, PDF等）
- **leptonica**: ファサードcrate

## Git規約

- mainブランチに直接コミットしない。必ずfeature branchを作成し、GitHub PRを経由してマージする
- PRではGitHub Copilot等の自動レビューを待ち、指摘事項を修正してからマージする
- マージ後のブランチは速やかに削除する
- マージコミットには変更の要約・理由・影響範囲を記載する（Linus Torvalds方式）
- 1コミットには1つの論理的変更のみ含める。無関係な変更を混在させない
- ブランチ命名: `feat/<crate>-<機能>`, `test/<スコープ>`, `refactor/<スコープ>`, `docs/<スコープ>`
- コミットメッセージ: Conventional Commits形式、scopeにはcrate名を使用

## TDD

テストと実装を同時にコミットしない。以下のサイクルをコミット履歴に残す:

1. RED: テストを先に書いてコミット（`#[ignore = "not yet implemented"]` を付与）
2. GREEN: 実装を追加してテストを通すコミット
3. REFACTOR: 必要に応じてリファクタリング

## テスト

- 回帰テストはC版の `reference/leptonica/prog/*_reg.c` に対応する形で作成
- テストデータは `tests/data/images/` に配置
- `tests/regout/` は `.gitignore` 対象（テスト出力）
- テストインフラはleptonica-test crateに集約（3モード: Generate/Compare/Display）

## 計画書

- `docs/plans/` に配置
- ファイル名: `NNN_<機能名>.md`
  （NNNはPhase番号×100 + 連番、ゼロパディング3桁。
  例: `000_overall-plan.md`, `002_core-pix.md`, `101_io-png.md`）
- Status: PLANNED → IN_PROGRESS → IMPLEMENTED を含める
- C版の対応ファイル・関数を明記

## 設計原則（前回の実装で確立済み、踏襲すること）

### Pix/PixMut二層メモリモデル

```rust
pub struct Pix { inner: Arc<PixData> }      // 不変・安価なclone
pub struct PixMut { inner: PixData }         // 可変・直接所有
```

- `try_into_mut()` でrefcount=1なら zero-copy変換、それ以外はコピー
- `RefCell`や`Mutex`を使わない

### エラー処理

- `thiserror`による構造化エラーenum。文字列ベースのエラーは使わない
- `#[from]`で標準エラー型からの自動変換
- `pub type Result<T> = std::result::Result<T, Error>;`

### ピクセルアクセス

- `get_pixel()` / `set_pixel()`: 安全（境界チェックあり）
- `get_pixel_unchecked()` / `set_pixel_unchecked()`: 高速（チェックなし、内部ループ用）

### モジュール分割

- 1ファイル100-200行を目安
- implブロックを複数ファイルに分散し各ファイルの責務を明確にする

### unsafe

- unsafeの使用は原則禁止
- やむを得ない場合はコミットメッセージに理由を明記し、最小限に留める

## 禁止事項

- 作業効率を理由にプロセス手順（TDD、PRワークフロー、レビュー確認）を省略しない
- 「リファレンスがあるから簡単」「変更が少ないから」で手順を飛ばさない
- 省略したくなったらユーザーに相談する
- mainに直接コミットしない（最重要）

## 引き継ぎ資料

`docs/rebuild/` に前回実装からの引き継ぎ資料を格納:

- `prompt.md`: 移植プロンプト（フェーズ分割、ワークフロー、成功パターン）
- `overall-plan.md`: crate構成、依存関係、設計方針の全体像
- `feature-comparison.md`: C版182ファイル vs Rust版の機能カバレッジ
- `test-comparison.md`: C版160回帰テスト vs Rust版のテストカバレッジ

各機能の詳細計画書は含めていない。
C版ソース（`reference/leptonica/`）を直接参照し、
自分で`docs/plans/`に計画書を作成すること。
