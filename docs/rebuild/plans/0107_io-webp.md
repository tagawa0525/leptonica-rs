# WebP I/O 実装計画

## 概要

WebP画像フォーマットの読み書き機能を実装する。C版Leptonica の `webpio.c` を参考に、Rust版の実装を行う。

## 参照

- **C版ソース**: `reference/leptonica/src/webpio.c`
- **実装先**: `crates/leptonica-io/src/webp.rs`
- **参考パターン**: `crates/leptonica-io/src/gif.rs`

## C版機能の対応

| C版関数 | Rust関数 | 説明 |
| --- | --- | --- |
| `pixReadMemWebP` | `read_webp` | WebP画像の読み込み |
| `pixWriteMemWebP` | `write_webp` | WebP画像の書き出し |
| `readHeaderMemWebP` | (内部使用) | WebPヘッダ読み込み |

## 実装計画

### 1. 依存クレートの追加

`Cargo.toml` に `image-webp` クレートを追加:

```toml
image-webp = { workspace = true, optional = true }

[features]
webp-format = ["image-webp"]
```

ワークスペース `Cargo.toml` にも追加:

```toml
[workspace.dependencies]
image-webp = "0.2"
```

### 2. WebP読み込み機能 (`read_webp`)

**仕様:**

- 入力: `Read` トレイトを実装したリーダー
- 出力: `Pix` (32bpp RGBA)
- アルファチャンネルがある場合は spp=4、ない場合は spp=3 として扱う

**処理フロー:**

1. `image-webp::WebPDecoder` でデコード
2. 画像データを読み込み
3. 32bpp Pix を作成
4. RGBA データを Pix のピクセルフォーマットに変換

### 3. WebP書き込み機能 (`write_webp`)

**仕様:**

- 入力: `Pix` (任意の深度), `Write` トレイト, オプション (品質, ロスレス)
- 出力: WebP形式データ

**処理フロー:**

1. Pix を 32bpp RGBA に変換
2. `image-webp::WebPEncoder` でエンコード
3. 品質設定 (0-100) またはロスレス設定を適用

### 4. オプション構造体

```rust
/// WebP encoding options
pub struct WebPOptions {
    /// Quality for lossy encoding (0-100, default: 80)
    pub quality: u8,
    /// Use lossless encoding
    pub lossless: bool,
}

impl Default for WebPOptions {
    fn default() -> Self {
        Self {
            quality: 80,
            lossless: false,
        }
    }
}
```

### 5. lib.rs への統合

- `mod webp;` を追加
- `read_image_format` に WebP サポートを追加
- `write_image_format` に WebP サポートを追加

### 6. フォーマット検出

既に `format.rs` に WebP 検出が実装済み (RIFF....WEBP)

## 実装タスク

- [x] workspace Cargo.toml に image-webp 依存を追加
- [x] leptonica-io/Cargo.toml に webp-format feature を追加
- [x] webp.rs モジュールを作成
  - [x] WebPOptions 構造体
  - [x] read_webp 関数
  - [x] write_webp 関数
  - [x] write_webp_with_options 関数
- [x] lib.rs を更新
- [x] ユニットテスト作成
- [x] cargo fmt && cargo clippy で品質チェック
- [x] テスト実行

## テスト計画

1. **ラウンドトリップテスト**: 作成した画像を WebP で保存し、再読み込みして一致確認
2. **品質設定テスト**: 異なる品質設定での動作確認
3. **ロスレステスト**: ロスレス設定での完全一致確認
4. **アルファチャンネルテスト**: 透明度を含む画像の処理確認

## 質問

(なし)

## 制約事項

- アニメーション WebP は非対応 (最初のフレームのみ)
- 品質値は 0-100 の範囲で、C版と同様
