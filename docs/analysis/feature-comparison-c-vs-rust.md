# C版 vs Rust版 機能比較

調査日: 2026-02-05

## 概要

| 項目 | C版 (reference/leptonica) | Rust版 (leptonica-rs) |
| ---- | ------------------------- | --------------------- |
| ソースファイル数 | **182個** (.c) | **33個** (.rs) |
| コード行数 | **約240,000行** | **約7,700行** |
| 実装率（行数ベース） | 100% | **約3.2%** |

## 機能カテゴリ別比較

### 1. 基本データ構造

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| Pix（画像コンテナ） | ✅ pix1-5.c | ✅ leptonica-core | 完全実装 |
| Box（矩形領域） | ✅ boxbasic.c, boxfunc1-5.c | ✅ leptonica-core | 完全実装 |
| Pta（点配列） | ✅ ptabasic.c, ptafunc1-2.c | ✅ leptonica-core | 完全実装 |
| Colormap | ✅ colormap.c | ✅ leptonica-core | 完全実装 |
| Pixa（Pix配列） | ✅ pixabasic.c, pixafunc1-2.c | ❌ | 未実装 |
| Numa（数値配列） | ✅ numabasic.c, numafunc1-2.c | ❌ | 未実装 |
| Sarray（文字列配列） | ✅ sarray1-2.c | ❌ | 未実装 |
| FPix（浮動小数点画像） | ✅ fpix1-2.c | ❌ | 未実装 |

### 2. 画像I/O

| フォーマット | C版 | Rust版 | 備考 |
| ------------ | --- | ------ | ---- |
| BMP | ✅ bmpio.c | ✅ bmp.rs | 完全実装 |
| PNG | ✅ pngio.c | ✅ png.rs | feature gate |
| JPEG | ✅ jpegio.c | ✅ jpeg.rs | feature gate |
| PNM (PBM/PGM/PPM) | ✅ pnmio.c | ✅ pnm.rs | feature gate |
| TIFF | ✅ tiffio.c | ❌ | 未実装 |
| GIF | ✅ gifio.c | ❌ | 未実装 |
| WebP | ✅ webpio.c, webpanimio.c | ❌ | 未実装 |
| JP2K (JPEG2000) | ✅ jp2kio.c | ❌ | 未実装 |
| PDF | ✅ pdfio1-2.c, pdfapp.c | ❌ | 未実装 |
| PostScript | ✅ psio1-2.c | ❌ | 未実装 |
| フォーマット検出 | ✅ readfile.c | ✅ format.rs | 完全実装 |

### 3. 幾何変換

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 回転（直交） | ✅ rotateorth.c | ✅ rotate.rs | 90°/180°/270° |
| 回転（任意角度） | ✅ rotate.c, rotateam.c | ❌ | 未実装 |
| 回転（シアー） | ✅ rotateshear.c | ❌ | 未実装 |
| スケーリング | ✅ scale1-2.c | ✅ scale.rs | 3アルゴリズム |
| アフィン変換 | ✅ affine.c, affinecompose.c | ❌ | 未実装 |
| 双線形変換 | ✅ bilinear.c | ❌ | 未実装 |
| 射影変換 | ✅ projective.c | ❌ | 未実装 |
| シアー変換 | ✅ shear.c | ❌ | 未実装 |
| 反転（左右/上下） | ✅ rotateorth.c | ✅ rotate.rs | 完全実装 |

### 4. モルフォロジー

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 二値侵食/膨張 | ✅ morph.c | ✅ binary.rs | 完全実装 |
| 二値開閉 | ✅ morph.c | ✅ binary.rs | 完全実装 |
| Hit-Miss変換 | ✅ morph.c | ✅ binary.rs | 完全実装 |
| 形態学的勾配 | ✅ morph.c | ✅ binary.rs | 完全実装 |
| Top-hat/Bottom-hat | ✅ morph.c | ✅ binary.rs | 完全実装 |
| グレースケール形態学 | ✅ graymorph.c | ❌ | 未実装 |
| カラー形態学 | ✅ colormorph.c | ❌ | 未実装 |
| DWA（高速形態学） | ✅ morphdwa.c, dwacomb.2.c | ❌ | 未実装 |
| 構造化要素（SEL） | ✅ sel1-2.c, selgen.c | ✅ sel.rs | 基本実装 |
| シーケンス操作 | ✅ morphseq.c | ❌ | 未実装 |
| 細線化 | ✅ ccthin.c | ❌ | 未実装 |

### 5. フィルタリング

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 畳み込み | ✅ convolve.c | ✅ convolve.rs | 完全実装 |
| ボックスフィルタ | ✅ convolve.c | ✅ convolve.rs | 完全実装 |
| ガウシアンフィルタ | ✅ convolve.c | ✅ convolve.rs | 完全実装 |
| Sobelエッジ検出 | ✅ edge.c | ✅ edge.rs | 完全実装 |
| ラプラシアン | ✅ edge.c | ✅ edge.rs | 完全実装 |
| シャープニング | ✅ enhance.c | ✅ edge.rs | 完全実装 |
| アンシャープマスク | ✅ enhance.c | ✅ edge.rs | 完全実装 |
| バイラテラルフィルタ | ✅ bilateral.c | ❌ | 未実装 |
| 適応マッピング | ✅ adaptmap.c | ❌ | 未実装 |
| ランクフィルタ | ✅ rank.c | ❌ | 未実装 |

### 6. 色処理

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 色空間変換 | ✅ colorspace.c | ❌ | スタブのみ |
| 色量子化 | ✅ colorquant1-2.c | ❌ | スタブのみ |
| 色セグメンテーション | ✅ colorseg.c | ❌ | スタブのみ |
| 色内容抽出 | ✅ colorcontent.c | ❌ | スタブのみ |
| 色塗りつぶし | ✅ colorfill.c | ❌ | スタブのみ |
| 着色 | ✅ coloring.c | ❌ | スタブのみ |

### 7. 二値化

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 単純閾値処理 | ✅ binarize.c | ❌ | 未実装 |
| Otsu二値化 | ✅ binarize.c | ❌ | 未実装 |
| Sauvola二値化 | ✅ binarize.c | ❌ | 未実装 |
| 適応二値化 | ✅ binarize.c | ❌ | 未実装 |
| ディザリング | ✅ grayquant.c | ❌ | 未実装 |

### 8. 領域処理

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 連結成分 | ✅ conncomp.c | ❌ | スタブのみ |
| 連結成分ラベリング | ✅ pixlabel.c | ❌ | スタブのみ |
| 境界追跡 | ✅ ccbord.c | ❌ | スタブのみ |
| シードフィル | ✅ seedfill.c | ❌ | スタブのみ |
| 分水嶺変換 | ✅ watershed.c | ❌ | スタブのみ |
| 四分木 | ✅ quadtree.c | ❌ | スタブのみ |

### 9. 文書処理

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| ページセグメンテーション | ✅ pageseg.c | ❌ | 未実装 |
| スキュー検出/補正 | ✅ skew.c | ❌ | 未実装 |
| デワーピング | ✅ dewarp1-4.c | ❌ | 未実装 |
| ベースライン検出 | ✅ baseline.c | ❌ | 未実装 |
| 文字認識 | ✅ recogbasic.c, recogident.c | ❌ | スタブのみ |

### 10. その他

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 画像比較 | ✅ compare.c | ❌ | 未実装 |
| 画像合成/ブレンド | ✅ blend.c | ❌ | 未実装 |
| 算術演算 | ✅ pixarith.c | ❌ | 未実装 |
| 論理演算 | ✅ rop.c, roplow.c | ❌ | 未実装 |
| ヒストグラム | ✅ numafunc1.c | ❌ | 未実装 |
| バーコード | ✅ bardecode.c, readbarcode.c | ❌ | 未実装 |
| グラフィックス | ✅ graphics.c | ❌ | 未実装 |
| 迷路生成/解法 | ✅ maze.c | ❌ | 未実装 |
| ワーパー | ✅ warper.c | ❌ | 未実装 |

## Rust版クレート実装状況

| クレート | 完成度 | 主要機能 |
| -------- | ------ | -------- |
| leptonica-core | ★★★★★ | Pix, Box, Pta, Colormap |
| leptonica-io | ★★★★☆ | BMP/PNG/JPEG/PNM読み書き |
| leptonica-transform | ★★★★★ | 回転（直交）、スケーリング |
| leptonica-morph | ★★★★★ | 二値形態学操作 |
| leptonica-filter | ★★★★★ | 畳み込み、エッジ検出 |
| leptonica-color | ★☆☆☆☆ | スタブのみ |
| leptonica-region | ★☆☆☆☆ | スタブのみ |
| leptonica-recog | ★☆☆☆☆ | スタブのみ |

## 実装優先度の推奨

### 高優先度（基本機能の補完）

1. **二値化** - 画像処理の基本
2. **グレースケール形態学** - morph拡張
3. **連結成分** - 領域処理の基礎
4. **TIFF I/O** - 重要なフォーマット

### 中優先度（よく使われる機能）

1. **任意角度回転** - transform拡張
2. **アフィン変換** - transform拡張
3. **色空間変換** - color実装開始
4. **画像比較** - テスト用にも有用
5. **Pixa/Numa** - コレクション型

### 低優先度（専門的機能）

1. **デワーピング** - 文書処理
2. **文字認識** - OCR関連
3. **バーコード** - 特殊用途
4. **PDF/PS出力** - 特殊用途

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
その他:       compare, blend, pixarith, rop, bardecode, graphics, maze, warper
```

## 参考

- C版ソース: `reference/leptonica/src/`
- Rust版ソース: `crates/*/src/`
