# C版 vs Rust版 機能比較

調査日: 2026-02-05

## 概要

| 項目 | C版 (reference/leptonica) | Rust版 (leptonica-rs) |
| ---- | ------------------------- | --------------------- |
| ソースファイル数 | **182個** (.c) | **56個** (.rs) |
| コード行数 | **約240,000行** | **約20,200行** |
| 実装率（行数ベース） | 100% | **約8.4%** |

## 機能カテゴリ別比較

### 1. 基本データ構造

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| Pix（画像コンテナ） | ✅ pix1-5.c | ✅ leptonica-core | 完全実装 |
| Box（矩形領域） | ✅ boxbasic.c, boxfunc1-5.c | ✅ leptonica-core | 完全実装 |
| Pta（点配列） | ✅ ptabasic.c, ptafunc1-2.c | ✅ leptonica-core | 完全実装 |
| Colormap | ✅ colormap.c | ✅ leptonica-core | 完全実装 |
| Pixa（Pix配列） | ✅ pixabasic.c, pixafunc1-2.c | ✅ pixa/mod.rs | 基本操作実装 |
| Numa（数値配列） | ✅ numabasic.c, numafunc1-2.c | ✅ numa/mod.rs | 基本操作実装 |
| Sarray（文字列配列） | ✅ sarray1-2.c | ❌ | 未実装 |
| FPix（浮動小数点画像） | ✅ fpix1-2.c | ❌ | 未実装 |

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
| JP2K (JPEG2000) | ✅ jp2kio.c | ❌ | 未実装 |
| PDF | ✅ pdfio1-2.c, pdfapp.c | ❌ | 未実装 |
| PostScript | ✅ psio1-2.c | ❌ | 未実装 |
| フォーマット検出 | ✅ readfile.c | ✅ format.rs | 完全実装 |

### 3. 幾何変換

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 回転（直交） | ✅ rotateorth.c | ✅ rotate.rs | 90°/180°/270° |
| 回転（任意角度） | ✅ rotate.c, rotateam.c | ✅ rotate.rs | 面積マッピング/サンプリング/シアー |
| 回転（シアー） | ✅ rotateshear.c | ✅ rotate.rs | 2-shear/3-shear対応 |
| スケーリング | ✅ scale1-2.c | ✅ scale.rs | 3アルゴリズム |
| アフィン変換 | ✅ affine.c, affinecompose.c | ✅ affine.rs | サンプリング/補間対応 |
| 双線形変換 | ✅ bilinear.c | ❌ | 未実装 |
| 射影変換 | ✅ projective.c | ❌ | 未実装 |
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
| バイラテラルフィルタ | ✅ bilateral.c | ✅ bilateral.rs | エッジ保存平滑化 |
| 適応マッピング | ✅ adaptmap.c | ❌ | 未実装 |
| ランクフィルタ | ✅ rank.c | ✅ rank.rs | メディアン/最小/最大 |

### 6. 色処理

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| 色空間変換 | ✅ colorspace.c | ✅ colorspace.rs | RGB↔HSV/LAB/XYZ/YUV |
| 色量子化 | ✅ colorquant1-2.c | ✅ quantize.rs | Median cut, Octree |
| 色セグメンテーション | ✅ colorseg.c | ❌ | 未実装 |
| 色内容抽出 | ✅ colorcontent.c | ✅ analysis.rs | 色統計、色数カウント |
| 色塗りつぶし | ✅ colorfill.c | ❌ | 未実装 |
| 着色 | ✅ coloring.c | ❌ | 未実装 |

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
| 連結成分 | ✅ conncomp.c | ✅ conncomp.rs | 4/8連結、Union-Find |
| 連結成分ラベリング | ✅ pixlabel.c | ✅ label.rs | 完全実装 |
| 境界追跡 | ✅ ccbord.c | ❌ | 未実装 |
| シードフィル | ✅ seedfill.c | ✅ seedfill.rs | floodfill, 穴埋め |
| 分水嶺変換 | ✅ watershed.c | ✅ watershed.rs | 完全実装 |
| 四分木 | ✅ quadtree.c | ❌ | 未実装 |

### 9. 文書処理

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| ページセグメンテーション | ✅ pageseg.c | ✅ pageseg.rs | ハーフトーン/テキスト検出 |
| スキュー検出/補正 | ✅ skew.c | ✅ skew.rs | 微分二乗和スコアリング |
| デワーピング | ✅ dewarp1-4.c | ❌ | 未実装 |
| ベースライン検出 | ✅ baseline.c | ✅ baseline.rs | 水平投影法 |
| 文字認識 | ✅ recogbasic.c, recogident.c | ✅ recog/ | テンプレートマッチング、DID |

### 10. JBIG2関連

| 機能 | C版 | Rust版 | 備考 |
| ---- | --- | ------ | ---- |
| JBIG2分類 | ✅ jbclass.c | ✅ jbclass/ | RankHaus, 相関ベース分類 |

### 11. その他

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

| クレート | 行数 | 完成度 | 主要機能 |
| -------- | ---- | ------ | -------- |
| leptonica-core | 2,519 | ★★★★★ | Pix, Box, Pta, Colormap |
| leptonica-io | 2,795 | ★★★★★ | BMP/PNG/JPEG/PNM/TIFF読み書き |
| leptonica-transform | 1,509 | ★★★★★ | 回転（直交）、スケーリング |
| leptonica-morph | 827 | ★★★★★ | 二値形態学操作 |
| leptonica-filter | 917 | ★★★★★ | 畳み込み、エッジ検出 |
| leptonica-color | 2,689 | ★★★★☆ | 色空間変換、二値化、量子化 |
| leptonica-region | 2,385 | ★★★★☆ | 連結成分、シードフィル、分水嶺 |
| leptonica-recog | 6,580 | ★★★★☆ | スキュー補正、ベースライン、ページセグ、文字認識、JBIG2分類 |

## 実装優先度の推奨

### 高優先度（基本機能の補完）

1. ~~**二値化** - 画像処理の基本~~ ✅ 完了
2. **グレースケール形態学** - morph拡張
3. ~~**連結成分** - 領域処理の基礎~~ ✅ 完了
4. ~~**TIFF I/O** - 重要なフォーマット~~ ✅ 完了

### 中優先度（よく使われる機能）

1. **任意角度回転** - transform拡張
2. **アフィン変換** - transform拡張
3. ~~**色空間変換** - color実装開始~~ ✅ 完了
4. **画像比較** - テスト用にも有用
5. **Pixa/Numa** - コレクション型

### 低優先度（専門的機能）

1. **デワーピング** - 文書処理
2. ~~**文字認識** - OCR関連~~ ✅ 完了（テンプレートマッチング、DID）
3. ~~**JBIG2分類** - 圧縮用クラスタリング~~ ✅ 完了
4. **バーコード** - 特殊用途
5. **PDF/PS出力** - 特殊用途

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

## 参考

- C版ソース: `reference/leptonica/src/`
- Rust版ソース: `crates/*/src/`
