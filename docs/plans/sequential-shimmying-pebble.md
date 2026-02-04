# GIF I/O サポート実装計画

## 概要

leptonica-rs に GIF 画像フォーマットの読み書きサポートを追加する。C版 Leptonica の `gifio.c` に準拠し、静止画GIFのみをサポート（アニメーション非対応）。

## 実装ファイル

| ファイル | 変更内容 |
| --- | --- |
| `crates/leptonica-io/src/gif.rs` | **新規作成**: GIF読み書きのメイン実装 |
| `crates/leptonica-io/src/lib.rs` | gif モジュール追加、read/write_image_format へ統合 |
| `crates/leptonica-io/Cargo.toml` | leptonica-color を gif-format feature に追加 |

## 実装する関数

### 公開API

```rust
pub fn read_gif<R: Read + Seek>(reader: R) -> IoResult<Pix>
pub fn write_gif<W: Write>(pix: &Pix, writer: W) -> IoResult<()>
```

### 対応深度

| 操作 | 対応深度 | 備考 |
| --- | --- | --- |
| 読み込み | 1/2/4/8 bpp | パレット画像として読み込み |
| 書き込み | 1/2/4/8 bpp | そのまま書き込み |
| 書き込み | 16 bpp | 8bpp グレースケールに変換 |
| 書き込み | 32 bpp | octree_quant で 8bpp パレットに量子化 |

## 実装手順

### 1. Cargo.toml 修正

```toml
[features]
gif-format = ["gif", "leptonica-color"]

[dependencies]
leptonica-color = { workspace = true, optional = true }
```

### 2. gif.rs 実装

#### 読み込み処理

1. `gif::DecodeOptions` で `ColorOutput::Indexed` を設定
2. 最初のフレームのみ読み込み（2フレーム目があればエラー）
3. ローカルパレット優先、なければグローバルパレット使用
4. 色数に応じた深度決定（C版と同じロジック）
   - 1-2色 → 1bpp、3-4色 → 2bpp、5-16色 → 4bpp、17-256色 → 8bpp
5. Pix と PixColormap を構築

#### 書き込み処理

1. 深度に応じた変換
   - 1/2/4/8 bpp: カラーマップがあればそのまま、なければグレースケールカラーマップ生成
   - 16 bpp: 8bpp グレースケールに変換
   - 32 bpp: `leptonica_color::quantize::octree_quant` で 8bpp に量子化
2. GIFパレット構築（2のべき乗サイズに調整）
3. `gif::Encoder` でフレーム書き込み

### 3. lib.rs 統合

```rust
#[cfg(feature = "gif-format")]
pub mod gif;

// read_image_format に追加
#[cfg(feature = "gif-format")]
ImageFormat::Gif => gif::read_gif(reader),

// write_image_format に追加
#[cfg(feature = "gif-format")]
ImageFormat::Gif => gif::write_gif(pix, writer),
```

## エラーケース

| 条件 | エラー |
| --- | --- |
| アニメーションGIF（複数フレーム） | `UnsupportedFormat("animated GIF not supported")` |
| カラーマップなし | `InvalidData("GIF has no color map")` |
| 不正なパレットサイズ | `InvalidData("invalid palette size")` |
| デコード/エンコード失敗 | `DecodeError` / `EncodeError` |

## テスト計画

### Roundtrip テスト

- 1/2/4/8 bpp パレット画像
- カラーマップの保持確認

### 深度変換テスト

- 16bpp → 8bpp 変換
- 32bpp → 8bpp 量子化

### エラーケーステスト

- アニメーションGIF拒否
- 不正データ処理

## 検証方法

```bash
# ビルド確認
cargo build -p leptonica-io --features gif-format

# テスト実行
cargo test -p leptonica-io --features gif-format

# 全フォーマットでビルド
cargo build -p leptonica-io --features all-formats
```

## 参考ファイル

- `reference/leptonica/src/gifio.c` - C版リファレンス
- `crates/leptonica-io/src/png.rs` - パレット画像処理のパターン
- `crates/leptonica-color/src/quantize.rs` - octree 量子化
