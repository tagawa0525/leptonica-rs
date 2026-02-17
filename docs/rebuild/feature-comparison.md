# C版 vs Rust版 機能比較

調査日: 2026-02-17（Phase 6, 9.3, 9.4 実装を反映）

## 概要

| 項目 | C版 (reference/leptonica) | Rust版 (leptonica-rs) |
| ---- | ------------------------- | --------------------- |
| ソースファイル数 | **182個** (.c) | **56個** (.rs) |
| コード行数 | **約240,000行** | **約20,200行** |
| 実装率（行数ベース） | 100% | **約8.4%** |

## 関数レベル比較サマリー

C版の全public関数を抽出し、Rust版での実装状況を3段階で分類した。
詳細は `docs/rebuild/comparison/` 配下の各ファイルを参照。

| クレート | ✅ 同等 | 🔄 異なる | ❌ 未実装 | 合計 | カバレッジ |
|---------|--------|----------|---------|------|-----------|
| [leptonica-core](comparison/core.md) | 201 | 25 | 619 | 845 | 26.7% |
| [leptonica-io](comparison/io.md) | 32 | 15 | 99 | 146 | 32.2% |
| [leptonica-transform](comparison/transform.md) | 39 | 12 | 101 | 152 | 33.6% |
| [leptonica-morph](comparison/morph.md) | 34 | 12 | 74 | 120 | 38.3% |
| [leptonica-filter](comparison/filter.md) | 50 | 0 | 49 | 99 | 50.5% |
| [leptonica-color](comparison/color.md) | 18 | 12 | 109 | 139 | 21.6% |
| [leptonica-region](comparison/region.md) | 28 | 8 | 68 | 104 | 34.6% |
| [leptonica-recog](comparison/recog.md) | 42 | 9 | 93 | 144 | 35.4% |
| [その他](comparison/misc.md) | 13 | 0 | 103 | 116 | 11.2% |
| **合計** | **457** | **93** | **1,315** | **1,865** | **29.5%** |

### 分類基準

- **✅ 同等**: C版と同じアルゴリズム・機能がRust版に存在
- **🔄 異なる**: 同等の機能はあるが、API設計やアルゴリズムが異なる
- **❌ 未実装**: Rust版に対応する機能が存在しない

### 設計上の主要な差異

| 観点 | C版 | Rust版 |
|------|-----|--------|
| メモリ管理 | 参照カウント（手動） | `Arc<PixData>` / 所有権（Pix/PixMut二層モデル） |
| エラー処理 | NULL返却 / エラーコード | `Result<T, Error>` / `thiserror` |
| API統一 | Gray/Color別関数 | 深度自動判定で統一API |
| コレクション型 | Boxa/Pixa/Numa/Sarray | `Vec<T>` + 専用型 |
| I/Oストリーム | `FILE*` ポインタ | `Read`/`Write` トレイト |
| データ構造 | heap/list/stack/queue | 標準ライブラリ（`BinaryHeap`/`Vec`/`VecDeque`） |
| unsafe | 全体的に使用 | 原則禁止、最小限に限定 |

## 機能カテゴリ別比較

### 1. 基本データ構造

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| Pix（画像コンテナ） | ✅ pix1-5.c | ✅ leptonica-core | 基本操作実装、深度変換等の一部は未実装 |
| Box（矩形領域） | ✅ boxbasic.c, boxfunc1-5.c | ✅ leptonica-core | 基本操作・幾何演算実装 |
| Pta（点配列） | ✅ ptabasic.c, ptafunc1-2.c | ✅ leptonica-core | 基本操作実装 |
| Colormap | ✅ colormap.c | ✅ leptonica-core | 基本操作実装 |
| Pixa（Pix配列） | ✅ pixabasic.c, pixafunc1-2.c | ✅ pixa/mod.rs | 基本操作実装 |
| Numa（数値配列） | ✅ numabasic.c, numafunc1-2.c | ✅ numa/mod.rs | 基本操作実装 |
| Sarray（文字列配列） | ✅ sarray1-2.c | ✅ sarray/mod.rs | 文字列配列/集合演算 |
| FPix（浮動小数点画像） | ✅ fpix1-2.c | ✅ fpix/mod.rs | Pix相互変換/演算 |
| ピクセル演算 | ✅ pixarith.c | ✅ arith.rs | 加減乗除/定数演算 |
| 論理演算 | ✅ rop.c, roplow.c | ✅ rop.rs | AND/OR/XOR/NOT等 |
| 比較 | ✅ compare.c | ✅ compare.rs | 差分/RMS/相関 |
| ブレンド | ✅ blend.c | ✅ blend.rs | アルファ/マスク/乗算等 |
| グラフィックス | ✅ graphics.c | ✅ graphics.rs | 線/矩形/円/等高線描画 |

### 2. 画像I/O

| フォーマット | C版 | Rust版 | 備考 |
| ------------ | --- | ------ | ---- |
| BMP | ✅ bmpio.c | ✅ bmp.rs | 完全実装 |
| PNG | ✅ pngio.c | ✅ png.rs | feature gate |
| JPEG | ✅ jpegio.c | ✅ jpeg.rs | feature gate |
| PNM (PBM/PGM/PPM) | ✅ pnmio.c | ✅ pnm.rs | feature gate |
| TIFF | ✅ tiffio.c | ✅ tiff.rs | feature gate、マルチページ対応 |
| GIF | ✅ gifio.c | ✅ gif.rs | feature gate |
| WebP | ✅ webpio.c, webpanimio.c | ✅ webp.rs | feature gate |
| JP2K (JPEG2000) | ✅ jp2kio.c | ✅ jp2k.rs | 読み込み対応 |
| PDF | ✅ pdfio1-2.c, pdfapp.c | ✅ pdf.rs | 書き込み対応 |
| PostScript | ✅ psio1-2.c | ✅ ps/ | EPS/PS出力 |
| フォーマット検出 | ✅ readfile.c | ✅ format.rs | 完全実装 |

### 3. 幾何変換

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 回転（直交） | ✅ rotateorth.c | ✅ rotate.rs | 90°/180°/270° |
| 回転（任意角度） | ✅ rotate.c, rotateam.c | ✅ rotate.rs | 面積マッピング/サンプリング/シアー |
| 回転（シアー） | ✅ rotateshear.c | ✅ rotate.rs | 2-shear/3-shear対応 |
| スケーリング | ✅ scale1-2.c | ✅ scale.rs | 3アルゴリズム（1bpp特化は未実装） |
| アフィン変換 | ✅ affine.c, affinecompose.c | ✅ affine.rs | サンプリング/補間対応 |
| 双線形変換 | ✅ bilinear.c | ✅ bilinear.rs | 4点対応/補間 |
| 射影変換 | ✅ projective.c | ✅ projective.rs | 4点ホモグラフィ |
| シアー変換 | ✅ shear.c | ✅ shear.rs | 水平/垂直/線形補間対応 |
| 反転（左右/上下） | ✅ rotateorth.c | ✅ rotate.rs | 完全実装 |

### 4. モルフォロジー

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 二値侵食/膨張 | ✅ morph.c | ✅ binary.rs | 完全実装 |
| 二値開閉 | ✅ morph.c | ✅ binary.rs | 完全実装 |
| Hit-Miss変換 | ✅ morph.c | ✅ binary.rs | 完全実装 |
| 形態学的勾配 | ✅ morph.c | ✅ binary.rs | 完全実装 |
| Top-hat/Bottom-hat | ✅ morph.c | ✅ binary.rs | 完全実装 |
| グレースケール形態学 | ✅ graymorph.c | ✅ grayscale.rs | 膨張/収縮/開/閉 |
| カラー形態学 | ✅ colormorph.c | ✅ color.rs | RGB各チャンネル独立処理 |
| DWA（高速形態学） | ✅ morphdwa.c, dwacomb.2.c | ✅ dwa.rs | ブリック高速演算 |
| 構造化要素（SEL） | ✅ sel1-2.c, selgen.c | ✅ sel.rs | 基本実装（自動生成は未実装） |
| シーケンス操作 | ✅ morphseq.c | ✅ sequence.rs | 文字列形式シーケンス |
| 細線化 | ✅ ccthin.c | ✅ thin.rs | 連結保持細線化 |

### 5. フィルタリング

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 畳み込み | ✅ convolve.c | ✅ convolve.rs | 基本・ブロック・分離可能・ウィンドウ統計 |
| ボックスフィルタ | ✅ convolve.c | ✅ convolve.rs | ブロック畳み込み最適化含む |
| ガウシアンフィルタ | ✅ convolve.c | ✅ convolve.rs | 基本実装 |
| Sobelエッジ検出 | ✅ edge.c | ✅ edge.rs | 完全実装 |
| ラプラシアン | ✅ edge.c | ✅ edge.rs | 完全実装 |
| シャープニング | ✅ enhance.c | ✅ edge.rs | 基本実装 |
| アンシャープマスク | ✅ enhance.c | ✅ edge.rs | 基本・高速版実装 |
| バイラテラルフィルタ | ✅ bilateral.c | ✅ bilateral.rs | エッジ保存平滑化（高速近似は未実装） |
| 適応マッピング | ✅ adaptmap.c | ✅ adaptmap.rs | 背景/コントラスト正規化 |
| ランクフィルタ | ✅ rank.c | ✅ rank.rs | メディアン/最小/最大 |

### 6. 色処理

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 色空間変換 | ✅ colorspace.c | ✅ colorspace.rs | RGB↔HSV/LAB/XYZ/YUV（ピクセル単位） |
| 色量子化 | ✅ colorquant1-2.c | ✅ quantize.rs | Median cut, Octree（簡略化） |
| 色セグメンテーション | ✅ colorseg.c | ✅ segment.rs | 4段階アルゴリズム |
| 色内容抽出 | ✅ colorcontent.c | ✅ analysis.rs | 色統計、色数カウント |
| 色塗りつぶし | ✅ colorfill.c | ✅ colorfill.rs | シードベース領域検出 |
| 着色 | ✅ coloring.c | ✅ coloring.rs | グレー着色/色シフト |

### 7. 二値化

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 単純閾値処理 | ✅ binarize.c | ✅ threshold.rs | 完全実装 |
| Otsu二値化 | ✅ binarize.c | ✅ threshold.rs | 完全実装 |
| Sauvola二値化 | ✅ binarize.c | ✅ threshold.rs | 完全実装 |
| 適応二値化 | ✅ binarize.c | ✅ threshold.rs | Mean/Gaussian |
| ディザリング | ✅ grayquant.c | ✅ threshold.rs | Floyd-Steinberg, Bayer |

### 8. 領域処理

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 連結成分 | ✅ conncomp.c | 🔄 conncomp.rs | Union-Find方式（C版はBOXA/PIXA返却） |
| 連結成分ラベリング | ✅ pixlabel.c | ✅ label.rs | 基本実装 |
| 境界追跡 | ✅ ccbord.c | 🔄 ccbord.rs | 簡略化Border構造体（C版はCCBORDA） |
| シードフィル | ✅ seedfill.c | 🔄 seedfill.rs | キューベースBFS（C版はスタックベース） |
| 分水嶺変換 | ✅ watershed.c | 🔄 watershed.rs | 優先度キュー方式 |
| 四分木 | ✅ quadtree.c | ✅ quadtree.rs | 積分画像/階層統計（カバレッジ高） |
| 迷路 | ✅ maze.c | ✅ maze.rs | 生成/BFS解法 |

### 9. 文書処理・認識

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| ページセグメンテーション | ✅ pageseg.c | ✅ pageseg.rs | ハーフトーン/テキスト検出 |
| スキュー検出/補正 | ✅ skew.c | ✅ skew.rs | 微分二乗和スコアリング |
| デワーピング | ✅ dewarp1-4.c | ✅ dewarp/ | 単一ページ（Dewarpa多ページ管理は未実装） |
| ベースライン検出 | ✅ baseline.c | ✅ baseline.rs | 水平投影法 |
| 文字認識 | ✅ recogbasic.c, recogident.c | ✅ recog/ | テンプレートマッチング、DID |
| JBIG2分類 | ✅ jbclass.c | ✅ jbclass/ | RankHaus, 相関ベース分類 |
| バーコード | ✅ bardecode.c, readbarcode.c | ✅ barcode/ | EAN/UPC/Code39等 |
| ワーパー | ✅ warper.c | ✅ warper.rs | 調和歪み/ステレオ（91%実装） |

## Rust版クレート実装状況

| クレート | 行数 | 関数カバレッジ | 主要機能 |
| -------- | ---- | ------------- | -------- |
| leptonica-core | 2,519 | 198/848 (23.3%) | Pix, Box, Pta, Colormap, 演算, 比較, ブレンド |
| leptonica-io | 2,795 | 47/146 (32.2%) | BMP/PNG/JPEG/PNM/TIFF/GIF/WebP/JP2K/PDF/PS |
| leptonica-transform | 1,509 | 51/152 (33.6%) | 回転, スケーリング, アフィン, 射影, シアー |
| leptonica-morph | 827 | 46/120 (38.3%) | 二値/グレースケール/カラー形態学, DWA, 細線化 |
| leptonica-filter | 917 | 50/99 (50.5%) | 畳み込み, エッジ検出, バイラテラル, ランク |
| leptonica-color | 2,689 | 30/139 (21.6%) | 色空間変換, 量子化, セグメンテーション, 二値化 |
| leptonica-region | 2,385 | 36/104 (34.6%) | 連結成分, シードフィル, 分水嶺, 四分木, 迷路 |
| leptonica-recog | 6,580 | 51/144 (35.4%) | スキュー補正, デワーピング, 文字認識, バーコード |
| その他 | - | 13/116 (11.2%) | ワーパー, エンコーディング |
| **合計** | **~20,200** | **522/1,868 (27.9%)** | |

## 未実装の主要領域

### leptonica-core（最大の未実装数: 650関数）

coreは対象Cファイル数が多く、以下が主な未実装領域:

- **I/O補助関数**: Pix/Boxa/Pixa/Numa等のRead/Write/Serialize
- **高度な配列操作**: ソート、選択、変換、統合
- **統計・ヒストグラム拡張**: タイル別、マスク付きの高度な統計
- **クリッピング・測定**: 矩形/マスクによるクリッピング、前景検出

### leptonica-filter（カバレッジ: 50.5%）

- **高速バイラテラル近似** (pixBilateral)
- **adaptmap.c詳細機能**: モルフォロジーベース背景正規化、マップユーティリティ
- **タイル化畳み込み**: pixBlockconvTiled等

### その他（カバレッジ: 11.2%）

- **圧縮画像コンテナ** (pixcomp.c): Pixcomp/Pixacomp
- **タイリング** (pixtiling.c): 大画像分割処理
- **高度なラベリング** (pixlabel.c): 距離関数、局所極値
- **データ構造**: Rustの標準ライブラリで代替可能（heap→BinaryHeap等）

## C版機能カテゴリ（182ファイル）

```text
基本構造:     pix1-5, boxbasic, boxfunc1-5, ptabasic, ptafunc1-2,
              pixabasic, pixafunc1-2, numabasic, numafunc1-2, sarray1-2
I/O:          bmpio, pngio, jpegio, pnmio, tiffio, gifio, webpio, jp2kio,
              pdfio1-2, psio1-2, readfile, writefile, spixio
幾何変換:     rotate, rotateam, rotateorth, rotateshear, scale1-2,
              affine, affinecompose, bilinear, projective, shear
形態学:       morph, morphapp, morphdwa, morphseq, graymorph, colormorph,
              sel1-2, selgen, ccthin
フィルタ:     convolve, edge, enhance, bilateral, adaptmap, rank
色処理:       colorspace, colorquant1-2, colorseg, colorcontent,
              colorfill, coloring, colormap
二値化:       binarize, grayquant
領域処理:     conncomp, ccbord, seedfill, watershed, pixlabel, quadtree
文書処理:     pageseg, skew, dewarp1-4, baseline
認識:         recogbasic, recogdid, recogident, recogtrain
JBIG2:        jbclass
その他:       compare, blend, pixarith, rop, bardecode, graphics, maze, warper
```

## 詳細比較文書

各クレートの関数レベル比較（全public関数の一覧と実装状況）:

- [leptonica-core](comparison/core.md) — 848関数
- [leptonica-io](comparison/io.md) — 146関数
- [leptonica-transform](comparison/transform.md) — 152関数
- [leptonica-morph](comparison/morph.md) — 120関数
- [leptonica-filter](comparison/filter.md) — 94関数
- [leptonica-color](comparison/color.md) — 139関数
- [leptonica-region](comparison/region.md) — 104関数
- [leptonica-recog](comparison/recog.md) — 144関数
- [その他](comparison/misc.md) — 116関数

## 参考

- C版ソース: `reference/leptonica/src/`
- Rust版ソース: `crates/*/src/`
