# C版 Leptonica 機能一覧（移植対象）

調査日: 2026-02-05

## 概要

| 項目 | C版 (reference/leptonica) |
| ---- | ------------------------- |
| ソースファイル数 | **182個** (.c) |
| コード行数 | **約240,000行** |

## 機能カテゴリ別

### 1. 基本データ構造

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| Pix（画像コンテナ） | pix1-5.c | 画像処理の中核 |
| Box（矩形領域） | boxbasic.c, boxfunc1-5.c | |
| Pta（点配列） | ptabasic.c, ptafunc1-2.c | |
| Colormap | colormap.c | カラーパレット |
| Pixa（Pix配列） | pixabasic.c, pixafunc1-2.c | |
| Numa（数値配列） | numabasic.c, numafunc1-2.c | |
| Sarray（文字列配列） | sarray1-2.c | |
| FPix（浮動小数点画像） | fpix1-2.c | Pix相互変換/演算 |

### 2. 画像I/O

| フォーマット | C版ソース | 備考 |
| ------------ | --------- | ---- |
| BMP | bmpio.c | |
| PNG | pngio.c | |
| JPEG | jpegio.c | |
| PNM (PBM/PGM/PPM) | pnmio.c | |
| TIFF | tiffio.c | マルチページ対応 |
| GIF | gifio.c | |
| WebP | webpio.c, webpanimio.c | |
| JP2K (JPEG2000) | jp2kio.c | |
| PDF | pdfio1-2.c, pdfapp.c | |
| PostScript | psio1-2.c | EPS/PS出力 |
| フォーマット検出 | readfile.c | マジックバイトで自動判定 |

### 3. 幾何変換

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| 回転（直交） | rotateorth.c | 90°/180°/270° |
| 回転（任意角度） | rotate.c, rotateam.c | 面積マッピング/サンプリング/シアー |
| 回転（シアー） | rotateshear.c | 2-shear/3-shear |
| スケーリング | scale1-2.c | 複数アルゴリズム |
| アフィン変換 | affine.c, affinecompose.c | サンプリング/補間 |
| 双線形変換 | bilinear.c | 4点対応/補間 |
| 射影変換 | projective.c | 4点ホモグラフィ |
| シアー変換 | shear.c | 水平/垂直/線形補間 |
| 反転（左右/上下） | rotateorth.c | |

### 4. モルフォロジー

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| 二値侵食/膨張 | morph.c | |
| 二値開閉 | morph.c | |
| Hit-Miss変換 | morph.c | |
| 形態学的勾配 | morph.c | |
| Top-hat/Bottom-hat | morph.c | |
| グレースケール形態学 | graymorph.c | 膨張/収縮/開/閉 |
| カラー形態学 | colormorph.c | RGB各チャンネル独立処理 |
| DWA（高速形態学） | morphdwa.c, dwacomb.2.c | ブリック高速演算 |
| 構造化要素（SEL） | sel1-2.c, selgen.c | |
| シーケンス操作 | morphseq.c | 文字列形式シーケンス |
| 細線化 | ccthin.c | 連結保持細線化 |

### 5. フィルタリング

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| 畳み込み | convolve.c | |
| ボックスフィルタ | convolve.c | |
| ガウシアンフィルタ | convolve.c | |
| Sobelエッジ検出 | edge.c | |
| ラプラシアン | edge.c | |
| シャープニング | enhance.c | |
| アンシャープマスク | enhance.c | |
| バイラテラルフィルタ | bilateral.c | エッジ保存平滑化 |
| 適応マッピング | adaptmap.c | 背景/コントラスト正規化 |
| ランクフィルタ | rank.c | メディアン/最小/最大 |

### 6. 色処理

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| 色空間変換 | colorspace.c | RGB↔HSV/LAB/XYZ/YUV |
| 色量子化 | colorquant1-2.c | Median cut, Octree |
| 色セグメンテーション | colorseg.c | 4段階アルゴリズム |
| 色内容抽出 | colorcontent.c | 色統計、色数カウント |
| 色塗りつぶし | colorfill.c | シードベース領域検出 |
| 着色 | coloring.c | グレー着色/色シフト |

### 7. 二値化

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| 単純閾値処理 | binarize.c | |
| Otsu二値化 | binarize.c | |
| Sauvola二値化 | binarize.c | |
| 適応二値化 | binarize.c | Mean/Gaussian |
| ディザリング | grayquant.c | Floyd-Steinberg, Bayer |

### 8. 領域処理

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| 連結成分 | conncomp.c | 4/8連結、Union-Find |
| 連結成分ラベリング | pixlabel.c | |
| 境界追跡 | ccbord.c | チェーンコード/穴追跡 |
| シードフィル | seedfill.c | floodfill, 穴埋め |
| 分水嶺変換 | watershed.c | |
| 四分木 | quadtree.c | 積分画像/階層統計 |

### 9. 文書処理

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| ページセグメンテーション | pageseg.c | ハーフトーン/テキスト検出 |
| スキュー検出/補正 | skew.c | 微分二乗和スコアリング |
| デワーピング | dewarp1-4.c | テキストライン/視差補正 |
| ベースライン検出 | baseline.c | 水平投影法 |
| 文字認識 | recogbasic.c, recogident.c | テンプレートマッチング、DID |

### 10. JBIG2関連

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| JBIG2分類 | jbclass.c | RankHaus, 相関ベース分類 |

### 11. その他

| 機能 | C版ソース | 備考 |
| ---- | --------- | ---- |
| 画像比較 | compare.c | 差分/RMS/相関 |
| 画像合成/ブレンド | blend.c | アルファ/マスク/乗算等 |
| 算術演算 | pixarith.c | 加減乗除/定数演算 |
| 論理演算 | rop.c, roplow.c | AND/OR/XOR/NOT等 |
| ヒストグラム | numafunc1.c | グレー/カラー統計 |
| バーコード | bardecode.c, readbarcode.c | EAN/UPC/Code39等 |
| グラフィックス | graphics.c | 線/矩形/円/等高線描画 |
| 迷路生成/解法 | maze.c | 生成/BFS解法 |
| ワーパー | warper.c | 調和歪み/ステレオ |

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
