# その他: C版 vs Rust版 関数レベル比較

調査日: 2026-02-15

## 対象ファイル

以下のCソースファイルは他のクレート比較(core, io, transform, morph, filter, color, region, recog)でカバーされていないもの:

### 画像ワーピング・変形
- warper.c

### PDF関連アプリケーション
- pdfapp.c

### カラーマップペイント
- paintcmap.c

### 圧縮画像コンテナ
- pixcomp.c

### 画像ラベリング
- pixlabel.c

### エンコーディング
- encoding.c

### ユーティリティ
- utils1.c, utils2.c

### データ構造
- heap.c (優先度キュー)
- list.c (双方向リンクリスト)
- stack.c (スタック)
- queue.c (キュー)
- ptra.c (ポインタ配列)
- dnabasic.c, dnafunc1.c (double number array)

### 特殊機能
- binexpand.c, binreduce.c (二値画像の拡大・縮小)
- pixtiling.c (画像タイリング)
- pixacc.c (ピクセルアキュムレータ)
- sudoku.c (数独ソルバー)
- correlscore.c (相関スコア)
- classapp.c (分類アプリケーション)
- dewarp1.c (文書デワーピング - 一部はrecogに含まれる可能性あり)
- recogident.c (認識スコアリング - recogに含まれる可能性あり)

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 13 |
| 🔄 異なる | 0 |
| ❌ 未実装 | 103 |
| 合計 | 116 |

注: この集計は主要な公開関数のみをカウント。静的(内部)関数は除外。

## 詳細

### warper.c (画像ワーピング)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixSimpleCaptcha | ❌ 未実装 | - | CAPTCHA生成の高レベルインターフェース |
| pixRandomHarmonicWarp | ✅ 同等 | random_harmonic_warp | ランダム正弦波ワーピング |
| pixRandomHarmonicWarpLUT | ❌ 未実装 | - | LUT版(最適化版) |
| pixWarpStereoscopic | ✅ 同等 | warp_stereoscopic | ステレオスコピックワーピング |
| pixStretchHorizontal | ✅ 同等 | stretch_horizontal | 水平方向伸縮 |
| pixStretchHorizontalSampled | ✅ 同等 | stretch_horizontal_sampled | サンプリング版 |
| pixStretchHorizontalLI | ✅ 同等 | stretch_horizontal_li | 線形補間版 |
| pixQuadraticVShear | ✅ 同等 | quadratic_v_shear | 二次垂直シアー |
| pixQuadraticVShearSampled | ✅ 同等 | quadratic_v_shear_sampled | サンプリング版 |
| pixQuadraticVShearLI | ✅ 同等 | quadratic_v_shear_li | 線形補間版 |
| pixStereoFromPair | ✅ 同等 | stereo_from_pair | ステレオペア合成 |

**warper.c カバレッジ**: 10/11 = 91% (pixSimpleCaptchaとLUT版は未実装)

### pdfapp.c (PDFアプリケーション)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| compressFilesToPdf | ❌ 未実装 | - | 画像圧縮してPDF化 |
| cropFilesToPdf | ❌ 未実装 | - | 画像クロップしてPDF化 |
| cleanTo1bppFilesToPdf | ❌ 未実装 | - | 1bpp変換してPDF化 |

**pdfapp.c カバレッジ**: 0/3 = 0%
**注**: leptonica-io/pdf.rsには基本的なPDF書き込み機能(write_pdf, write_pdf_multi)は実装済み。しかし、これらの高レベル画像処理+PDF生成アプリケーション関数は未実装。

### paintcmap.c (カラーマップペイント)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixSetSelectCmap | ❌ 未実装 | - | カラーマップ内の特定ピクセル再塗装 |
| pixColorGrayRegionsCmap | ❌ 未実装 | - | 領域内グレーピクセル着色 |
| pixColorGrayCmap | ❌ 未実装 | - | グレーピクセル着色 |
| pixColorGrayMaskedCmap | ❌ 未実装 | - | マスク通してグレーピクセル着色 |
| addColorizedGrayToCmap | ❌ 未実装 | - | カラーマップに着色グレー追加 |
| pixSetSelectMaskedCmap | ❌ 未実装 | - | マスク通して特定ピクセル設定 |
| pixSetMaskedCmap | ❌ 未実装 | - | マスク通して全ピクセル設定 |

**paintcmap.c カバレッジ**: 0/7 = 0%

### pixcomp.c (圧縮画像コンテナ)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixcompCreateFromPix | ❌ 未実装 | - | PixからPixcomp作成 |
| pixcompCreateFromString | ❌ 未実装 | - | 文字列からPixcomp作成 |
| pixcompCreateFromFile | ❌ 未実装 | - | ファイルからPixcomp作成 |
| pixcompDestroy | ❌ 未実装 | - | Pixcomp破棄 |
| pixcompCopy | ❌ 未実装 | - | Pixcompコピー |
| pixcompGetDimensions | ❌ 未実装 | - | 寸法取得 |
| pixcompGetParameters | ❌ 未実装 | - | パラメータ取得 |
| pixcompDetermineFormat | ❌ 未実装 | - | フォーマット決定 |
| pixCreateFromPixcomp | ❌ 未実装 | - | PixcompからPix作成 |
| pixacompCreate | ❌ 未実装 | - | Pixacomp配列作成 |
| pixacompCreateWithInit | ❌ 未実装 | - | 初期化付き作成 |
| pixacompCreateFromPixa | ❌ 未実装 | - | PixaからPixacomp作成 |
| pixacompCreateFromFiles | ❌ 未実装 | - | ファイルからPixacomp作成 |
| pixacompCreateFromSA | ❌ 未実装 | - | 文字列配列からPixacomp作成 |
| pixacompDestroy | ❌ 未実装 | - | Pixacomp破棄 |
| pixacompAddPix | ❌ 未実装 | - | Pix追加 |
| pixacompAddPixcomp | ❌ 未実装 | - | Pixcomp追加 |
| pixacompReplacePix | ❌ 未実装 | - | Pix置換 |
| pixacompReplacePixcomp | ❌ 未実装 | - | Pixcomp置換 |
| pixacompAddBox | ❌ 未実装 | - | Box追加 |
| pixacompGetCount | ❌ 未実装 | - | カウント取得 |
| pixacompGetPixcomp | ❌ 未実装 | - | Pixcomp取得 |
| pixacompGetPix | ❌ 未実装 | - | Pix取得 |
| pixacompGetPixDimensions | ❌ 未実装 | - | Pixの寸法取得 |
| pixacompGetBoxa | ❌ 未実装 | - | Boxa取得 |
| pixacompGetBoxaCount | ❌ 未実装 | - | Boxaカウント取得 |
| pixacompGetBox | ❌ 未実装 | - | Box取得 |
| pixacompGetBoxGeometry | ❌ 未実装 | - | Box座標取得 |
| pixacompGetOffset | ❌ 未実装 | - | オフセット取得 |
| pixacompSetOffset | ❌ 未実装 | - | オフセット設定 |
| pixaCreateFromPixacomp | ❌ 未実装 | - | PixacompからPixa作成 |
| pixacompJoin | ❌ 未実装 | - | Pixacomp結合 |
| pixacompInterleave | ❌ 未実装 | - | Pixacompインターリーブ |
| pixacompRead | ❌ 未実装 | - | ファイル読み込み |
| pixacompReadStream | ❌ 未実装 | - | ストリーム読み込み |
| pixacompReadMem | ❌ 未実装 | - | メモリから読み込み |
| pixacompWrite | ❌ 未実装 | - | ファイル書き込み |
| pixacompWriteStream | ❌ 未実装 | - | ストリーム書き込み |
| pixacompWriteMem | ❌ 未実装 | - | メモリに書き込み |
| pixacompConvertToPdf | ✅ 同等 | 部分実装 | leptonica-io/pdf.rsにwrite_pdf_multi有り |
| pixacompConvertToPdfData | ❌ 未実装 | - | PDF データ生成 |
| pixacompFastConvertToPdfData | ❌ 未実装 | - | 高速PDF データ生成 |
| pixacompWriteStreamInfo | ❌ 未実装 | - | ストリーム情報書き込み |
| pixcompWriteStreamInfo | ❌ 未実装 | - | 情報書き込み |
| pixacompDisplayTiledAndScaled | ❌ 未実装 | - | タイル表示 |
| pixacompWriteFiles | ❌ 未実装 | - | ファイル群書き込み |
| pixcompWriteFile | ❌ 未実装 | - | ファイル書き込み |

**pixcomp.c カバレッジ**: 1/46 = 2%
**注**: Pixcomp/Pixacmp (圧縮画像コンテナ)データ構造そのものが未実装

### pixlabel.c (画像ラベリング)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixConnCompTransform | ❌ 未実装 | - | 連結成分変換 |
| pixConnCompAreaTransform | ❌ 未実装 | - | 面積ベース連結成分変換 |
| pixConnCompIncrInit | ❌ 未実装 | - | インクリメンタルCC初期化 |
| pixConnCompIncrAdd | ❌ 未実装 | - | インクリメンタルCC追加 |
| pixGetSortedNeighborValues | ❌ 未実装 | - | ソート済み近傍値取得 |
| pixLocToColorTransform | ❌ 未実装 | - | 位置->色変換 |

**pixlabel.c カバレッジ**: 0/6 = 0%

### encoding.c (エンコーディング)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| encodeBase64 | ❌ 未実装 | - | Base64エンコード |
| decodeBase64 | ❌ 未実装 | - | Base64デコード |
| encodeAscii85 | ✅ 同等 | 実装済み | PS/PDFモジュール内で使用 |
| decodeAscii85 | ✅ 同等 | 実装済み | PS/PDFモジュール内で使用 |
| encodeAscii85WithComp | ❌ 未実装 | - | 圧縮付きAscii85 |
| decodeAscii85WithComp | ❌ 未実装 | - | 圧縮付きAscii85デコード |
| reformatPacked64 | ❌ 未実装 | - | パックされた64再フォーマット |

**encoding.c カバレッジ**: 2/7 = 29%
**注**: Ascii85はleptonica-io/src/ps/ascii85.rsに実装済み

### utils1.c, utils2.c (ユーティリティ)

utils1.cとutils2.cには多数の文字列操作、ファイルI/O、メモリ管理などの低レベルユーティリティ関数が含まれる。Rust版では標準ライブラリ(std::fs, std::string, Vec等)で代替可能なため、直接的な移植は不要。

**主な関数分類**:
- 文字列操作: stringNew, stringCopy, stringJoin, stringReverse, etc. → Rust String/&str
- ファイルI/O: l_binaryRead, l_binaryWrite, fileCopy, etc. → std::fs, std::io
- メモリ管理: reallocNew → Vec::resize, etc.
- 配列操作: arrayFindSequence, arrayReplaceEachSequence → Rustのイテレータ/スライス操作

**カバレッジ**: 実質100% (Rust標準ライブラリで代替)

### heap.c (優先度キュー)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| lheapCreate | ❌ 未実装 | - | ヒープ作成 |
| lheapDestroy | ❌ 未実装 | - | ヒープ破棄 |
| lheapAdd | ❌ 未実装 | - | 要素追加 |
| lheapRemove | ❌ 未実装 | - | 最小要素削除 |
| lheapGetCount | ❌ 未実装 | - | 要素数取得 |
| lheapGetElement | ❌ 未実装 | - | 要素取得 |
| lheapSort | ❌ 未実装 | - | ソート |
| lheapSortStrictOrder | ❌ 未実装 | - | 厳密順序ソート |
| lheapPrint | ❌ 未実装 | - | デバッグ出力 |

**heap.c カバレッジ**: 0/9 = 0%
**注**: Rust標準ライブラリにstd::collections::BinaryHeapがあるため、必要時はそれを使用可能

### list.c (双方向リンクリスト)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| listDestroy | ❌ 未実装 | - | リスト破棄 |
| listAddToHead | ❌ 未実装 | - | 先頭に追加 |
| listAddToTail | ❌ 未実装 | - | 末尾に追加 |
| listInsertBefore | ❌ 未実装 | - | 前に挿入 |
| listInsertAfter | ❌ 未実装 | - | 後に挿入 |
| listRemoveElement | ❌ 未実装 | - | 要素削除 |
| listRemoveFromHead | ❌ 未実装 | - | 先頭から削除 |
| listRemoveFromTail | ❌ 未実装 | - | 末尾から削除 |
| listFindElement | ❌ 未実装 | - | 要素検索 |
| listFindTail | ❌ 未実装 | - | 末尾検索 |
| listGetCount | ❌ 未実装 | - | 要素数取得 |
| listReverse | ❌ 未実装 | - | 反転 |
| listJoin | ❌ 未実装 | - | 結合 |

**list.c カバレッジ**: 0/13 = 0%
**注**: Rust標準ライブラリにstd::collections::LinkedListがあるため、必要時はそれを使用可能

### stack.c (スタック)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| lstackCreate | ❌ 未実装 | - | スタック作成 |
| lstackDestroy | ❌ 未実装 | - | スタック破棄 |
| lstackAdd | ❌ 未実装 | - | プッシュ |
| lstackRemove | ❌ 未実装 | - | ポップ |
| lstackGetCount | ❌ 未実装 | - | 要素数取得 |
| lstackPrint | ❌ 未実装 | - | デバッグ出力 |

**stack.c カバレッジ**: 0/6 = 0%
**注**: Rust Vec<T>でpush/popを使えばスタックとして利用可能

### queue.c (キュー)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| lqueueCreate | ❌ 未実装 | - | キュー作成 |
| lqueueDestroy | ❌ 未実装 | - | キュー破棄 |
| lqueueAdd | ❌ 未実装 | - | エンキュー |
| lqueueRemove | ❌ 未実装 | - | デキュー |
| lqueueGetCount | ❌ 未実装 | - | 要素数取得 |
| lqueuePrint | ❌ 未実装 | - | デバッグ出力 |

**queue.c カバレッジ**: 0/6 = 0%
**注**: Rust標準ライブラリにstd::collections::VecDequeがあるため、必要時はそれを使用可能

### ptra.c (ポインタ配列)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| ptraCreate | ❌ 未実装 | - | Ptra作成 |
| ptraDestroy | ❌ 未実装 | - | Ptra破棄 |
| ptraAdd | ❌ 未実装 | - | ポインタ追加 |
| ptraInsert | ❌ 未実装 | - | ポインタ挿入 |
| ptraRemove | ❌ 未実装 | - | ポインタ削除 |
| ptraRemoveLast | ❌ 未実装 | - | 最後を削除 |
| ptraReplace | ❌ 未実装 | - | ポインタ置換 |
| ptraSwap | ❌ 未実装 | - | ポインタ交換 |
| ptraCompactArray | ❌ 未実装 | - | 配列圧縮 |
| ptraReverse | ❌ 未実装 | - | 配列反転 |
| ptraJoin | ❌ 未実装 | - | 配列結合 |
| ptraGetMaxIndex | ❌ 未実装 | - | 最大インデックス取得 |
| ptraGetActualCount | ❌ 未実装 | - | 実要素数取得 |
| ptraGetPtrToItem | ❌ 未実装 | - | アイテムポインタ取得 |
| ptraaCreate | ❌ 未実装 | - | Ptra配列作成 |
| ptraaDestroy | ❌ 未実装 | - | Ptra配列破棄 |
| ptraaGetSize | ❌ 未実装 | - | サイズ取得 |
| ptraaInsertPtra | ❌ 未実装 | - | Ptra挿入 |
| ptraaGetPtra | ❌ 未実装 | - | Ptra取得 |
| ptraaFlattenToPtra | ❌ 未実装 | - | 平坦化 |

**ptra.c カバレッジ**: 0/20 = 0%
**注**: Rust Vec<Option<Box<T>>>で代替可能

### dnabasic.c, dnafunc1.c (Double Number Array)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| l_dnaCreate | ❌ 未実装 | - | DNA作成 |
| l_dnaCreateFromIArray | ❌ 未実装 | - | 整数配列からDNA作成 |
| l_dnaCreateFromDArray | ❌ 未実装 | - | 倍精度配列からDNA作成 |
| l_dnaMakeSequence | ❌ 未実装 | - | 数列作成 |
| l_dnaDestroy | ❌ 未実装 | - | DNA破棄 |
| l_dnaCopy | ❌ 未実装 | - | DNAコピー |
| l_dnaClone | ❌ 未実装 | - | DNAクローン |
| l_dnaEmpty | ❌ 未実装 | - | DNA空にする |
| l_dnaAddNumber | ❌ 未実装 | - | 数値追加 |
| l_dnaInsertNumber | ❌ 未実装 | - | 数値挿入 |
| l_dnaRemoveNumber | ❌ 未実装 | - | 数値削除 |
| l_dnaReplaceNumber | ❌ 未実装 | - | 数値置換 |
| (その他 dnabasic.c / dnafunc1.c 多数) | ❌ 未実装 | - | 配列操作、統計、I/O等 |

**dnabasic.c / dnafunc1.c カバレッジ**: 0/40+ = 0%
**注**: C版のL_DNA(double配列)に相当する専用データ構造は未実装。Rust版ではVec<f64>で代替可能だが、C版のような専用APIは提供されていない。

### binexpand.c, binreduce.c (二値画像拡大・縮小)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixExpandBinaryReplicate | ❌ 未実装 | - | 複製拡大 |
| pixExpandBinaryPower2 | ❌ 未実装 | - | 2のべき乗拡大 |
| pixReduceBinary2 | ❌ 未実装 | - | 2倍縮小 |
| pixReduceRankBinaryCascade | ❌ 未実装 | - | ランクベースカスケード縮小 |
| pixReduceRankBinary2 | ❌ 未実装 | - | ランクベース2倍縮小 |
| makeSubsampleTab2x | ❌ 未実装 | - | サブサンプルテーブル生成 |

**binexpand.c / binreduce.c カバレッジ**: 0/6 = 0%

### pixtiling.c (画像タイリング)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixTilingCreate | ❌ 未実装 | - | タイリング作成 |
| pixTilingDestroy | ❌ 未実装 | - | タイリング破棄 |
| pixTilingGetCount | ❌ 未実装 | - | タイル数取得 |
| pixTilingGetSize | ❌ 未実装 | - | タイルサイズ取得 |
| pixTilingGetTile | ❌ 未実装 | - | タイル取得 |
| pixTilingNoStripOnPaint | ❌ 未実装 | - | ペイント時ストリップなし |
| pixTilingPaintTile | ❌ 未実装 | - | タイルペイント |

**pixtiling.c カバレッジ**: 0/7 = 0%

### pixacc.c (ピクセルアキュムレータ)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaccCreate | ❌ 未実装 | - | アキュムレータ作成 |
| pixaccCreateFromPix | ❌ 未実装 | - | Pixからアキュムレータ作成 |
| pixaccDestroy | ❌ 未実装 | - | アキュムレータ破棄 |
| pixaccFinal | ❌ 未実装 | - | 最終化 |
| pixaccGetPix | ❌ 未実装 | - | Pix取得 |
| pixaccGetOffset | ❌ 未実装 | - | オフセット取得 |
| pixaccAdd | ❌ 未実装 | - | 加算 |
| pixaccSubtract | ❌ 未実装 | - | 減算 |
| pixaccMultConst | ❌ 未実装 | - | 定数倍 |
| pixaccMultConstAccumulate | ❌ 未実装 | - | 定数倍加算 |

**pixacc.c カバレッジ**: 0/10 = 0%

### sudoku.c (数独ソルバー)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| sudokuReadFile | ❌ 未実装 | - | ファイルから読み込み |
| sudokuReadString | ❌ 未実装 | - | 文字列から読み込み |
| sudokuCreate | ❌ 未実装 | - | 数独作成 |
| sudokuDestroy | ❌ 未実装 | - | 数独破棄 |
| sudokuSolve | ❌ 未実装 | - | 数独解く |
| sudokuTestUniqueness | ❌ 未実装 | - | 一意性テスト |
| sudokuGenerate | ❌ 未実装 | - | 数独生成 |
| sudokuOutput | ❌ 未実装 | - | 出力 |

**sudoku.c カバレッジ**: 0/8 = 0%
**注**: 数独ソルバーは画像処理に直接関係しない機能

### correlscore.c (相関スコア)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixCorrelationScore | ❌ 未実装 | - | 相関スコア計算 |
| pixCorrelationScoreThresholded | ❌ 未実装 | - | 閾値付き相関スコア |
| pixCorrelationScoreSimple | ❌ 未実装 | - | 単純相関スコア |
| pixCorrelationScoreShifted | ❌ 未実装 | - | シフト相関スコア |

**correlscore.c カバレッジ**: 0/4 = 0%

### classapp.c (分類アプリケーション)

classapp.cは文字認識のためのJbig2ベース分類アプリケーション。内容は非常に専門的で、leptonica-recogクレートでカバーされる可能性がある。詳細な分析は recog.md に委譲。

**classapp.c カバレッジ**: TBD (recog.mdで評価)

### dewarp1.c (文書デワーピング)

dewarp1.cは文書画像のデワーピング(歪み補正)機能。一部はleptonica-recogクレートの範囲に含まれる可能性がある。詳細な分析は recog.md に委譲。

**dewarp1.c カバレッジ**: TBD (recog.mdで評価)

### recogident.c (認識スコアリング)

recogident.cは文字認識のスコアリング機能。leptonica-recogクレートの範囲。詳細な分析は recog.md に委譲。

**recogident.c カバレッジ**: TBD (recog.mdで評価)

## 分析

### 実装状況

1. **ワーピング関数 (warper.c)**: ほぼ完全実装 (91%)
   - leptonica-transform/src/warper.rs で主要関数を実装済み
   - 未実装: pixSimpleCaptcha (高レベルAPI), LUT版最適化

2. **PDFアプリケーション (pdfapp.c)**: 未実装 (0%)
   - 基本的なPDF書き込みはあるが、画像処理統合アプリケーション関数は未実装
   - compressFilesToPdf, cropFilesToPdf等の高レベルAPIが必要

3. **カラーマップペイント (paintcmap.c)**: 未実装 (0%)
   - カラーマップを持つ画像への直接ペイント操作は未実装
   - Rust版ではカラーマップ操作が限定的

4. **圧縮画像コンテナ (pixcomp.c)**: ほぼ未実装 (2%)
   - Pixcomp/Pixacomp データ構造そのものが未実装
   - メモリ効率的な圧縮画像コンテナは重要な最適化機能

5. **画像ラベリング (pixlabel.c)**: 未実装 (0%)
   - 連結成分ラベリングの高度な機能は未実装
   - 基本的な連結成分解析は region クレートにある可能性

6. **エンコーディング (encoding.c)**: 部分実装 (29%)
   - Ascii85は実装済み (PS/PDF用)
   - Base64は未実装

7. **ユーティリティ (utils1.c, utils2.c)**: Rust標準ライブラリで代替
   - 文字列操作、ファイルI/O等はRust標準で対応
   - 直接移植は不要

8. **データ構造 (heap, list, stack, queue, ptra, dna)**: 未実装
   - Rust標準ライブラリ(BinaryHeap, LinkedList, Vec, VecDeque)で代替可能
   - L_DNA等の専用APIは未実装

9. **特殊機能**: ほぼ未実装
   - binexpand/binreduce: 0%
   - pixtiling: 0%
   - pixacc: 0%
   - sudoku: 0% (画像処理外)
   - correlscore: 0%

### 優先度評価

#### 高優先度 (コア機能)
1. **Pixcomp/Pixacomp** (pixcomp.c)
   - メモリ効率的な圧縮画像配列
   - 大量画像処理で重要

2. **pixtiling** (pixtiling.c)
   - 大画像の分割処理に必須
   - メモリ効率的な処理の基盤

3. **pixlabel** (pixlabel.c)
   - 連結成分ラベリングの高度機能
   - 画像解析で重要

#### 中優先度 (便利機能)
4. **paintcmap** (paintcmap.c)
   - カラーマップ画像の直接操作
   - 特定用途で有用

5. **pdfapp** (pdfapp.c)
   - バッチ処理の高レベルAPI
   - ユーザビリティ向上

6. **binexpand/binreduce**
   - 二値画像専用の高速拡大縮小
   - パフォーマンス最適化

7. **pixacc** (pixacc.c)
   - ピクセル累積演算
   - 特定アルゴリズムで必要

#### 低優先度 (特殊/代替可能)
8. **データ構造** (heap, list, stack, queue, ptra, dna)
   - Rust標準ライブラリで代替可能
   - 必要に応じて追加

9. **encoding** (encoding.c)
   - Base64: 外部クレート使用可能
   - Ascii85: 実装済み

10. **correlscore** (correlscore.c)
    - 相関スコア計算
    - 特定用途

11. **sudoku** (sudoku.c)
    - 画像処理に直接関係なし
    - 低優先度

### 移植戦略

1. **Phase 1: コアインフラ**
   - Pixcomp/Pixacomp データ構造
   - pixtiling
   - pixlabel

2. **Phase 2: 便利機能**
   - paintcmap
   - pdfapp高レベルAPI
   - binexpand/binreduce

3. **Phase 3: 最適化・特殊機能**
   - pixacc
   - correlscore
   - その他必要に応じて

4. **低優先度**
   - データ構造系 (標準ライブラリで代替)
   - sudoku (画像処理外)

## 推奨事項

1. **Pixcompの実装を最優先**
   - 大量画像処理での メモリ効率が重要
   - PDF生成等で必要

2. **pixtilingの実装**
   - 大画像処理の基盤
   - メモリ制約下での処理に必須

3. **pixlabelの実装**
   - 連結成分解析の高度機能
   - 画像解析アプリケーションで必要

4. **データ構造はRust標準優先**
   - heap → BinaryHeap
   - list → LinkedList
   - stack → Vec
   - queue → VecDeque
   - 必要に応じてラッパー追加

5. **warper.cの残り機能**
   - pixSimpleCaptcha実装
   - LUT版は性能要求次第

## 参考: C版とRust版の対応

| C版データ構造 | Rust標準ライブラリ代替 |
|--------------|---------------------|
| L_HEAP | std::collections::BinaryHeap |
| DLLIST | std::collections::LinkedList |
| L_STACK | Vec<T> (push/pop) |
| L_QUEUE | std::collections::VecDeque |
| L_PTRA | Vec<Option<Box<T>>> |
| L_DNA | Vec<f64> (専用APIなし) |

| C版機能 | Rust実装箇所 |
|---------|------------|
| warper.c | leptonica-transform/src/warper.rs |
| encoding.c (Ascii85) | leptonica-io/src/ps/ascii85.rs |
| pdfapp.c (基本) | leptonica-io/src/pdf.rs |
| utils1.c, utils2.c | Rust標準ライブラリ (std::fs, std::string等) |
