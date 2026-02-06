# PostScript出力実装計画

## 概要

PostScriptは画像をEPS/PS形式で出力する機能です。
C版の`psio1.c`, `psio2.c`を参考に、純粋Rustで実装します。

## C版機能の分析

### psio2.c の主要関数

| 関数名 | 説明 | 実装優先度 |
| --- | --- | --- |
| `pixWriteStreamPS()` | ストリームへのPS出力（非圧縮） | 高 |
| `pixWriteStringPS()` | PS文字列生成（hexエンコード） | 高 |
| `generateUncompressedPS()` | 非圧縮PS生成 | 高 |
| `pixWriteMemPS()` | メモリへのPS出力 | 高 |
| `convertFlateToPSEmbed()` | Flate圧縮+ASCII85エンコードEPS | 高 |
| `generateFlatePS()` | Flate圧縮PS生成 | 高 |
| `getResLetterPage()` | US Letter用解像度計算 | 中 |
| `l_psWriteBoundingBox()` | BBヒント出力フラグ | 中 |

### 圧縮/エンコーディング方式

1. **非圧縮 (Level 1)**: 画像データをhex文字列で出力
2. **Flate圧縮 (Level 3)**: zlib圧縮 + ASCII85エンコード
3. **DCT圧縮 (Level 2)**: JPEG圧縮（今回はスコープ外）
4. **CCITT G4 (Level 2)**: 1bpp用（今回はスコープ外）

## 実装計画

### Phase 1: 基本構造

```rust
// crates/leptonica-io/src/ps.rs

/// PostScript圧縮レベル
pub enum PsLevel {
    Level1,  // 非圧縮 (hex)
    Level2,  // DCT/G4 (将来拡張用)
    Level3,  // Flate + ASCII85
}

/// PostScript出力オプション
pub struct PsOptions {
    pub level: PsLevel,
    pub resolution: u32,
    pub scale: f32,
    pub write_bounding_box: bool,
    pub title: Option<String>,
}
```

### Phase 2: コア機能実装

1. **ASCII85エンコーディング**
   - 4バイトを5文字のASCII文字に変換
   - 終端マーカー `~>`
   - 純粋Rustで実装（外部依存なし）

2. **Hex文字列変換**
   - バイトを2文字のhex ASCIIに変換
   - Level 1 PS用

3. **PS文字列生成**
   - ヘッダー（DSC準拠）
   - BoundingBox
   - 画像データ埋め込み
   - showpage

### Phase 3: API実装

```rust
/// メモリへのPS出力（圧縮）
pub fn write_ps_mem(pix: &Pix, options: &PsOptions) -> IoResult<Vec<u8>>

/// ストリームへのPS出力
pub fn write_ps<W: Write>(pix: &Pix, writer: W, options: &PsOptions) -> IoResult<()>

/// EPS形式での出力（BoundingBox付き）
pub fn write_eps_mem(pix: &Pix, options: &PsOptions) -> IoResult<Vec<u8>>
```

## 依存関係

### 必須

- `miniz_oxide` - Flate圧縮（既存の`pdf-format`で使用）

### オプション

- なし（純粋Rustで実装）

## Feature Gate

```toml
# Cargo.toml
[features]
ps-format = ["miniz_oxide"]
```

## ファイル構成

```text
crates/leptonica-io/src/
├── ps.rs           # 新規: PostScript出力モジュール
├── ascii85.rs      # 新規: ASCII85エンコーダー（内部モジュール）
└── lib.rs          # 更新: ps モジュール追加
```

## 実装詳細

### ASCII85エンコーディング

```rust
/// 4バイトを5文字のASCII85に変換
/// 範囲: '!' (33) から 'u' (117)
fn encode_ascii85(data: &[u8]) -> String {
    // 実装
}
```

### PS生成フロー

1. 画像データ準備（1bpp/8bpp/32bpp対応）
2. データ圧縮（Level 3のみ）
3. エンコード（hex or ASCII85）
4. PSヘッダー生成
5. 画像コマンド生成
6. データ埋め込み
7. フッター生成

## テスト計画

1. **単体テスト**
   - ASCII85エンコード/デコード
   - Hexエンコード
   - PS文字列生成

2. **統合テスト**
   - 1bpp画像のPS出力
   - 8bppグレースケールのPS出力
   - 32bpp RGB画像のPS出力
   - EPSの妥当性検証（ヘッダーフォーマット）

## 質問

なし（現時点では）

## 実装スケジュール

1. ASCII85エンコーダー実装
2. 非圧縮PS生成（Level 1）
3. Flate圧縮PS生成（Level 3）
4. EPSサポート
5. テスト・ドキュメント

## 補足

- DCT圧縮（JPEG）およびCCITT G4圧縮は将来の拡張として残す
- `pixWriteMemPsCompressed()` は `write_ps_mem` + `PsLevel::Level3` で実現
