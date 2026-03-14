# leptonica-rs

[![CI](https://github.com/tagawa0525/leptonica-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/tagawa0525/leptonica-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/leptonica.svg)](https://crates.io/crates/leptonica)
[![docs.rs](https://docs.rs/leptonica/badge.svg)](https://docs.rs/leptonica)
[![License: BSD-2-Clause](https://img.shields.io/badge/license-BSD--2--Clause-blue.svg)](LICENSE)

[Leptonica](http://www.leptonica.org/) 画像処理ライブラリの純Rust移植 — **Cライブラリ依存なし**、クロスコンパイル対応、`unsafe`不使用。

[English](README.md)

## Leptonicaについて

[Leptonica](http://www.leptonica.org/) は Dan Bloomberg 氏が開発・保守するC言語のオープンソース画像処理ライブラリである。約240,000行のコードに2,700以上の関数を収録し、ドキュメント画像処理から自然画像処理まで幅広い領域をカバーする。[Tesseract OCR](https://github.com/tesseract-ocr/tesseract/) や [OpenCV](https://github.com/opencv/opencv) をはじめとする多くのプロジェクトの基盤として20年以上にわたり利用されてきた。

本プロジェクトは、Leptonicaの設計思想とアルゴリズムをRustで再実装するものである。C版のソースコードとドキュメントを一次資料として参照し、その機能を忠実に移植することを目指す。C版のソースはgit submoduleとして `reference/leptonica/` に配置し、常に原典を参照できるようにしている。

## 移植状況

C版の182ソースファイル・2,286関数に対する移植の進捗。

| 指標                 | 数値                    |
| -------------------- | ----------------------- |
| コード行数           | 約144,000行 / 249,000行 |
| 関数カバレッジ       | 1,874 / 2,286 (82.0%)   |
| 実カバレッジ         | 1,874 / 1,874 (100.0%)  |
| 回帰テストカバレッジ | 159 / 159 (100.0%)      |

詳細: [機能比較](docs/porting/feature-comparison.md) / [テスト比較](docs/porting/test-comparison.md)

## モジュール構成

単一クレート `leptonica` の各モジュールが Leptonica の機能領域に対応する:

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

## ビルド・テスト

```bash
cargo check --all-features
cargo test
cargo test --all-features
cargo clippy --all-features --all-targets
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
- `docs/porting/` -- 引き継ぎ資料（移植プロンプト、機能比較、テスト比較）

## ライセンス

本プロジェクトは [BSD 2-Clause License](LICENSE) の下で配布されている。
C版 [Leptonica](http://www.leptonica.org/) と同じライセンスである。

## 開発体制

移植作業は主に [Claude Code](https://docs.anthropic.com/en/docs/claude-code) 等のAIコーディングエージェントが実施している。人間のメンテナがアーキテクチャ、プロセスルール、受け入れ基準を定め、エージェントがC版ソースを読み、Rustコードを書き、テストを実行する。すべてのコミットはCI・自動レビューを経てからマージされる。

そのため、AI支援開発に特有のパターンが含まれている可能性がある。バグ報告やフィードバックは歓迎する。

## 謝辞

本プロジェクトはC版Leptonicaのソースコード、ドキュメント、回帰テストに全面的に依拠している。Dan Bloomberg 氏をはじめとするLeptonicaコントリビュータの方々の長年にわたる設計・実装・保守の成果なくして、この移植は成り立たない。
