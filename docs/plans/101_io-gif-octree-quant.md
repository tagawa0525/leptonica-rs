# GIF量子化のoctree_quant統合

Status: IMPLEMENTED

## Context

`backup/pre-rebuild` ブランチと HEAD（`perf/morph-brick-separable` → main）の差異を調査し、
必要な変更を取り込む。

git history rebuild により HEAD は backup から独立して再構築されたため、
差異の大半は HEAD 側が既に改善済み。唯一、GIF エンコードのカラー量子化に
backup のほうが優れたアプローチを採っている。

## 差異分析の結論

### HEAD が既に優位な領域（変更不要）

| 領域 | HEAD | backup |
|------|------|--------|
| ファサードcrate | 全8ドメインcrateを依存・再エクスポート済 | leptonica-coreのみ |
| Core公開エクスポート | BlendMode, CompareResult等すべてエクスポート済 | 一部欠落 |
| ドキュメント | 詳細（ピクセルレイアウト、所有権モデル、C参照） | 簡略 |
| Bresenham線描画 | 整数アルゴリズム（正確） | 浮動小数点（丸め誤差） |
| PNM I/O | 複数空白/コメント対応、カラーマップ展開あり | 単バイト読み飛ばし、カラーマップ未対応 |
| WebP I/O | core color モジュール関数を使用 | ローカル関数 |
| Morph | rasterop + composite decomposition最適化 | ピクセル単位 |
| PS形式 | サポート済 | なし |
| ライセンス | BSD-2-Clause（意図的変更） | Apache-2.0 |
| エラー処理 | set_pixel で x/y 個別エラー情報 | 統合 |
| SEL origin検証 | 境界チェックあり | なし |
| テスト基盤 | ディレクトリ作成エラーをログ出力 | 無視 |

### 取り込むべき変更: GIF エンコードのカラー量子化

HEAD の `gif.rs` は 32bpp → 8bpp 変換に独自の median-cut 実装（約190行）を持つ。
backup は `leptonica-color::quantize::octree_quant` を利用（約5行）。

backup のアプローチが優れている理由:
- `gif-format` feature は既に `leptonica-color` を依存に含む
- `octree_quant` は leptonica-color で十分にテスト済み
- 190行の重複コードを削除できる
- `octree_quant_256()` というコンビニエンス関数も利用可能

## 変更内容

### 対象ファイル

- `crates/leptonica-io/src/gif.rs` — 唯一の変更対象

### 具体的な変更

1. **import 変更** (L8):
   ```rust
   // Before
   use leptonica_core::{Pix, PixColormap, PixelDepth, color};
   // After
   use leptonica_color::quantize::{OctreeOptions, octree_quant};
   use leptonica_core::{Pix, PixColormap, PixelDepth};
   ```
   `color` import はテストモジュール内でのみ使用されるため、プロダクションコードからは不要。

2. **`prepare_pix_for_gif()` の Bit32 分岐** (L187-191):
   ```rust
   // Before
   PixelDepth::Bit32 => {
       let (quantized, cmap) = quantize_32bpp_to_8bpp(pix)?;
       Ok((quantized, cmap))
   }
   // After
   PixelDepth::Bit32 => {
       let quantized = octree_quant(pix, &OctreeOptions { max_colors: 256 })
           .map_err(|e| IoError::EncodeError(format!("quantization error: {}", e)))?;
       let cmap = quantized
           .colormap()
           .ok_or_else(|| IoError::EncodeError("quantized image has no colormap".to_string()))?
           .clone();
       Ok((quantized, cmap))
   }
   ```

3. **削除する関数** (L195-386, 約190行):
   - `quantize_32bpp_to_8bpp()`
   - `find_nearest_color()`
   - `median_cut()`
   - `box_color_range()`
   - `split_box()`
   - `box_average()`

### 再利用する既存コード

- `leptonica_color::quantize::octree_quant` (`crates/leptonica-color/src/quantize.rs:292`)
- `leptonica_color::quantize::OctreeOptions` (`crates/leptonica-color/src/quantize.rs:280`)

## コミット計画

これはリファクタリング（外部動作は変わらない）なので、既存テストがそのまま検証に使える。

1. **refactor(io): replace inline median-cut with octree_quant from leptonica-color**
   - 上記の変更をすべて適用
   - 既存の9つのGIFテストがパスすることを確認

## 検証

```bash
cargo test -p leptonica-io --features gif-format -- gif
```

重要テスト:
- `test_gif_32bpp_quantization` — 変更されるコードパスを直接テスト
- `test_gif_roundtrip_paletted` — 既存パレットの保持
- `gifio_reg` — 統合テスト（存在する場合）

## 実装結果

実装完了: 2026-02-15

- PR: https://github.com/tagawa0525/leptonica-rs/pull/26
- コミット: `4175890` (refactor(io): replace inline median-cut with octree_quant from leptonica-color)
- 削除: 約196行（6個の関数）
- テスト結果: ✅ 9/9 パス
- ビルド: ✅ 成功

## 将来の検討事項（本PRの対象外）

- backup のテストカバレッジ拡充（fpix1: 289→829行、numa2: 172→1239行 等）
- pixa1/pixa2 テストの crate 移動（core → region/transform）
