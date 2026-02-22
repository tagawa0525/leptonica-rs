# leptonica-rs

[Leptonica](http://www.leptonica.org/) 画像処理ライブラリのRust移植。

[English](README.md)

## Leptonicaについて

[Leptonica](http://www.leptonica.org/) は Dan Bloomberg 氏が開発・保守するC言語のオープンソース画像処理ライブラリである。約240,000行のコードに2,700以上の関数を収録し、ドキュメント画像処理から自然画像処理まで幅広い領域をカバーする。[Tesseract OCR](https://github.com/tesseract-ocr/tesseract/) や [OpenCV](https://github.com/opencv/opencv) をはじめとする多くのプロジェクトの基盤として20年以上にわたり利用されてきた。

本プロジェクトは、Leptonicaの設計思想とアルゴリズムをRustで再実装するものである。C版のソースコードとドキュメントを一次資料として参照し、その機能を忠実に移植することを目指す。C版のソースはgit submoduleとして `reference/leptonica/` に配置し、常に原典を参照できるようにしている。

## 移植状況

C版の182ソースファイル・1,880関数に対する移植の進捗。

| 指標                 | 数値                      |
| -------------------- | ------------------------- |
| コード行数           | 約120,000行 / 240,000行  |
| 関数カバレッジ       | 819 + 98 / 1,880 (48.8%) |
| 回帰テストカバレッジ | 58 / 159 (36.5%)          |

詳細: [機能比較](docs/rebuild/feature-comparison.md) / [テスト比較](docs/rebuild/test-comparison.md)

## Crate構成

```text
leptonica-rs/
├── crates/
│   ├── leptonica-core/        # Pix, Box, Numa, FPix等の基本データ構造
│   ├── leptonica-io/          # 画像I/O (PNG, JPEG, TIFF, GIF, WebP等)
│   ├── leptonica-morph/       # 形態学演算 (binary, grayscale, DWA, thinning)
│   ├── leptonica-transform/   # 幾何変換 (rotate, scale, affine等)
│   ├── leptonica-filter/      # フィルタリング (bilateral, rank, adaptmap, convolve, edge)
│   ├── leptonica-color/       # 色処理 (segmentation, quantize, threshold, colorspace)
│   ├── leptonica-region/      # 領域解析 (conncomp, ccbord, quadtree, watershed, maze)
│   ├── leptonica-recog/       # 認識 (barcode, dewarp, baseline, pageseg, jbclass)
│   └── leptonica-test/        # テストインフラ
├── leptonica/                 # ファサードcrate (re-export)
└── reference/leptonica/       # C版ソース (git submodule, read-only参照)
```

### 依存関係

```text
leptonica-recog → leptonica-morph, leptonica-transform, leptonica-region, leptonica-color, leptonica-core
leptonica-morph, leptonica-transform, leptonica-filter, leptonica-color → leptonica-io, leptonica-core
leptonica-region → leptonica-core
leptonica-io → leptonica-core
```

## ビルド・テスト

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace
```

### C版リファレンスの取得

```bash
git submodule update --init
```

> **注意**: `.gitmodules` ではSSH URL（`git@github.com:...`）を使用しています。
> SSHキーが設定されていない環境では、HTTPS形式に変更してください。

## ドキュメント

- `CLAUDE.md` -- 開発規約・プロセスルール
- `docs/plans/` -- 各機能の実装計画書
- `docs/rebuild/` -- 引き継ぎ資料（移植プロンプト、機能比較、テスト比較）

## ライセンス

本プロジェクトは [BSD 2-Clause License](LICENSE) の下で配布されている。
C版 [Leptonica](http://www.leptonica.org/) と同じライセンスである。

## 開発体制

移植作業は主に [Claude Code](https://docs.anthropic.com/en/docs/claude-code) 等のAIコーディングエージェントが実施している。人間のメンテナがアーキテクチャ、プロセスルール、受け入れ基準を定め、エージェントがC版ソースを読み、Rustコードを書き、テストを実行する。すべてのコミットはCI・自動レビューを経てからマージされる。

そのため、AI支援開発に特有のパターンが含まれている可能性がある。バグ報告やフィードバックは歓迎する。

## 謝辞

本プロジェクトはC版Leptonicaのソースコード、ドキュメント、回帰テストに全面的に依拠している。Dan Bloomberg 氏をはじめとするLeptonicaコントリビュータの方々の長年にわたる設計・実装・保守の成果なくして、この移植は成り立たない。
