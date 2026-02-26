# その他: C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

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
| ✅ 同等 | 11 |
| 🔄 異なる | 0 |
| 🚫 不要 | 94 |
| ❌ 未実装 | 78 |
| 合計 | 183 |

注: この集計は主要な公開関数のみをカウント。静的(内部)関数は除外。🚫はRust標準ライブラリで代替可能、C固有管理関数、画像処理に無関係等の理由で移植不要と判断したもの。

## 詳細

### warper.c (画像ワーピング)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixSimpleCaptcha | ❌ 未実装 | - | CAPTCHA生成の高レベルインターフェース |
| pixRandomHarmonicWarp | ✅ 同等 | random_harmonic_warp | ランダム正弦波ワーピング |
| pixRandomHarmonicWarpLUT | 🚫 不要 | - | LUT版最適化はRustコンパイラ最適化で代替 |
| pixWarpStereoscopic | ✅ 同等 | warp_stereoscopic | ステレオスコピックワーピング |
| pixStretchHorizontal | ✅ 同等 | stretch_horizontal | 水平方向伸縮 |
| pixStretchHorizontalSampled | ✅ 同等 | stretch_horizontal_sampled | サンプリング版 |
| pixStretchHorizontalLI | ✅ 同等 | stretch_horizontal_li | 線形補間版 |
| pixQuadraticVShear | ✅ 同等 | quadratic_v_shear | 二次垂直シアー |
| pixQuadraticVShearSampled | ✅ 同等 | quadratic_v_shear_sampled | サンプリング版 |
| pixQuadraticVShearLI | ✅ 同等 | quadratic_v_shear_li | 線形補間版 |
| pixStereoFromPair | ✅ 同等 | stereo_from_pair | ステレオペア合成 |

**warper.c カバレッジ**: 9/11 = 82% (✅9, 🚫1, ❌1: pixSimpleCaptchaのみ未実装)

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
| pixcompDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| pixcompCopy | ❌ 未実装 | - | Pixcompコピー |
| pixcompGetDimensions | ❌ 未実装 | - | 寸法取得 |
| pixcompGetParameters | ❌ 未実装 | - | パラメータ取得 |
| pixcompDetermineFormat | ❌ 未実装 | - | フォーマット決定 |
| pixCreateFromPixcomp | ❌ 未実装 | - | PixcompからPix作成 |
| pixacompCreate | ❌ 未実装 | - | Pixacomp配列作成 |
| pixacompCreateWithInit | ❌ 未実装 | - | 初期化付き作成 |
| pixacompCreateFromPixa | ❌ 未実装 | - | PixaからPixacomp作成 |
| pixacompCreateFromFiles | ❌ 未実装 | - | ファイルからPixacomp作成 |
| pixacompCreateFromSA | 🚫 不要 | - | C版SArray固有、RustではVec<PathBuf>等で代替 |
| pixacompDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| pixacompAddPix | ❌ 未実装 | - | Pix追加 |
| pixacompAddPixcomp | ❌ 未実装 | - | Pixcomp追加 |
| pixacompReplacePix | ❌ 未実装 | - | Pix置換 |
| pixacompReplacePixcomp | 🚫 不要 | - | pixacompReplacePixで代替可能 |
| pixacompAddBox | 🚫 不要 | - | Rust版ではBoxa操作はBoxa型に委譲 |
| pixacompGetCount | ❌ 未実装 | - | カウント取得 |
| pixacompGetPixcomp | ❌ 未実装 | - | Pixcomp取得 |
| pixacompGetPix | ❌ 未実装 | - | Pix取得 |
| pixacompGetPixDimensions | ❌ 未実装 | - | Pixの寸法取得 |
| pixacompGetBoxa | ❌ 未実装 | - | Boxa取得 |
| pixacompGetBoxaCount | 🚫 不要 | - | Boxa型のlen()で代替 |
| pixacompGetBox | ❌ 未実装 | - | Box取得 |
| pixacompGetBoxGeometry | ❌ 未実装 | - | Box座標取得 |
| pixacompGetOffset | 🚫 不要 | - | Rustではフィールドアクセスで代替 |
| pixacompSetOffset | 🚫 不要 | - | Rustではフィールドアクセスで代替 |
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
| pixacompWriteStreamInfo | 🚫 不要 | - | デバッグ用表示関数 |
| pixcompWriteStreamInfo | 🚫 不要 | - | デバッグ用表示関数 |
| pixacompDisplayTiledAndScaled | 🚫 不要 | - | デバッグ用表示関数 |
| pixacompWriteFiles | ❌ 未実装 | - | ファイル群書き込み |
| pixcompWriteFile | ❌ 未実装 | - | ファイル書き込み |

**pixcomp.c カバレッジ**: 1/47 = 2% (✅1, 🚫11, ❌35)
**注**: Pixcomp/Pixacmp (圧縮画像コンテナ)データ構造そのものが未実装。destroy/デバッグ/C固有関数は🚫。

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
| decodeAscii85 | ❌ 未実装 | - | encodeAscii85のみ実装済み |
| encodeAscii85WithComp | 🚫 不要 | - | zlib圧縮付き、Rustではflate2等で個別対応 |
| decodeAscii85WithComp | 🚫 不要 | - | zlib解凍付き、Rustではflate2等で個別対応 |
| reformatPacked64 | 🚫 不要 | - | C版内部フォーマット関数 |

**encoding.c カバレッジ**: 1/7 = 14% (✅1, 🚫3, ❌3)
**注**: Ascii85はleptonica-io/src/ps/ascii85.rsに実装済み。圧縮付き版はRustでは個別ライブラリ組合せで代替。

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
| lheapCreate | 🚫 不要 | - | std::collections::BinaryHeapで代替 |
| lheapDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| lheapAdd | 🚫 不要 | - | BinaryHeap::pushで代替 |
| lheapRemove | 🚫 不要 | - | BinaryHeap::popで代替 |
| lheapGetCount | 🚫 不要 | - | BinaryHeap::lenで代替 |
| lheapGetElement | 🚫 不要 | - | BinaryHeap::peekで代替 |
| lheapSort | 🚫 不要 | - | BinaryHeap::into_sorted_vecで代替 |
| lheapSortStrictOrder | 🚫 不要 | - | BinaryHeap::into_sorted_vecで代替 |
| lheapPrint | 🚫 不要 | - | デバッグ用表示関数 |

**heap.c カバレッジ**: 🚫全9関数不要 (std::collections::BinaryHeapで代替)

### list.c (双方向リンクリスト)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| listDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| listAddToHead | 🚫 不要 | - | Vec::insert(0,x)やLinkedList::push_frontで代替 |
| listAddToTail | 🚫 不要 | - | Vec::pushやLinkedList::push_backで代替 |
| listInsertBefore | 🚫 不要 | - | Vec::insertで代替 |
| listInsertAfter | 🚫 不要 | - | Vec::insertで代替 |
| listRemoveElement | 🚫 不要 | - | Vec::removeで代替 |
| listRemoveFromHead | 🚫 不要 | - | Vec::remove(0)で代替 |
| listRemoveFromTail | 🚫 不要 | - | Vec::popで代替 |
| listFindElement | 🚫 不要 | - | イテレータのfindで代替 |
| listFindTail | 🚫 不要 | - | Vec::lastで代替 |
| listGetCount | 🚫 不要 | - | Vec::lenで代替 |
| listReverse | 🚫 不要 | - | Vec::reverseで代替 |
| listJoin | 🚫 不要 | - | Vec::extendで代替 |

**list.c カバレッジ**: 🚫全13関数不要 (Vec/LinkedListで代替)

### stack.c (スタック)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| lstackCreate | 🚫 不要 | - | Vec::newで代替 |
| lstackDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| lstackAdd | 🚫 不要 | - | Vec::pushで代替 |
| lstackRemove | 🚫 不要 | - | Vec::popで代替 |
| lstackGetCount | 🚫 不要 | - | Vec::lenで代替 |
| lstackPrint | 🚫 不要 | - | デバッグ用表示関数 |

**stack.c カバレッジ**: 🚫全6関数不要 (Vec<T>のpush/popで代替)

### queue.c (キュー)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| lqueueCreate | 🚫 不要 | - | VecDeque::newで代替 |
| lqueueDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| lqueueAdd | 🚫 不要 | - | VecDeque::push_backで代替 |
| lqueueRemove | 🚫 不要 | - | VecDeque::pop_frontで代替 |
| lqueueGetCount | 🚫 不要 | - | VecDeque::lenで代替 |
| lqueuePrint | 🚫 不要 | - | デバッグ用表示関数 |

**queue.c カバレッジ**: 🚫全6関数不要 (std::collections::VecDequeで代替)

### ptra.c (ポインタ配列)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| ptraCreate | 🚫 不要 | - | Vec::newで代替 |
| ptraDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| ptraAdd | 🚫 不要 | - | Vec::pushで代替 |
| ptraInsert | 🚫 不要 | - | Vec::insertで代替 |
| ptraRemove | 🚫 不要 | - | Vec::removeで代替 |
| ptraRemoveLast | 🚫 不要 | - | Vec::popで代替 |
| ptraReplace | 🚫 不要 | - | インデックスアクセスで代替 |
| ptraSwap | 🚫 不要 | - | Vec::swapで代替 |
| ptraCompactArray | 🚫 不要 | - | Vec::retain等で代替 |
| ptraReverse | 🚫 不要 | - | Vec::reverseで代替 |
| ptraJoin | 🚫 不要 | - | Vec::extendで代替 |
| ptraGetMaxIndex | 🚫 不要 | - | Vec::lenで代替 |
| ptraGetActualCount | 🚫 不要 | - | イテレータのfilter+countで代替 |
| ptraGetPtrToItem | 🚫 不要 | - | インデックスアクセスで代替 |
| ptraaCreate | 🚫 不要 | - | Vec<Vec<T>>で代替 |
| ptraaDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| ptraaGetSize | 🚫 不要 | - | Vec::lenで代替 |
| ptraaInsertPtra | 🚫 不要 | - | Vec::insertで代替 |
| ptraaGetPtra | 🚫 不要 | - | インデックスアクセスで代替 |
| ptraaFlattenToPtra | 🚫 不要 | - | Iterator::flattenで代替 |

**ptra.c カバレッジ**: 🚫全20関数不要 (Vec<Option<Box<T>>>で代替)

### dnabasic.c, dnafunc1.c (Double Number Array)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| l_dnaCreate | 🚫 不要 | - | Vec::<f64>::newで代替 |
| l_dnaCreateFromIArray | 🚫 不要 | - | Vec::from / iter().map().collectで代替 |
| l_dnaCreateFromDArray | 🚫 不要 | - | Vec::fromで代替 |
| l_dnaMakeSequence | 🚫 不要 | - | Iterator::mapで代替 |
| l_dnaDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| l_dnaCopy | 🚫 不要 | - | Vec::cloneで代替 |
| l_dnaClone | 🚫 不要 | - | Arc<Vec<f64>>等で代替 |
| l_dnaEmpty | 🚫 不要 | - | Vec::clearで代替 |
| l_dnaAddNumber | 🚫 不要 | - | Vec::pushで代替 |
| l_dnaInsertNumber | 🚫 不要 | - | Vec::insertで代替 |
| l_dnaRemoveNumber | 🚫 不要 | - | Vec::removeで代替 |
| l_dnaReplaceNumber | 🚫 不要 | - | インデックスアクセスで代替 |
| (その他 dnabasic.c / dnafunc1.c 多数) | 🚫 不要 | - | 配列操作・統計・I/O等すべてVec<f64>+イテレータで代替 |

**dnabasic.c / dnafunc1.c カバレッジ**: 🚫全40+関数不要 (Vec<f64>+イテレータで代替)
**注**: C版のL_DNA(double配列)に相当する専用データ構造は不要。Rust版ではVec<f64>と標準ライブラリのイテレータ/スライスメソッドで完全に代替可能。

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
| pixTilingDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| pixTilingGetCount | ❌ 未実装 | - | タイル数取得 |
| pixTilingGetSize | ❌ 未実装 | - | タイルサイズ取得 |
| pixTilingGetTile | ❌ 未実装 | - | タイル取得 |
| pixTilingNoStripOnPaint | 🚫 不要 | - | Rustでは構造体フィールドで設定 |
| pixTilingPaintTile | ❌ 未実装 | - | タイルペイント |

**pixtiling.c カバレッジ**: 0/7 = 0% (🚫2, ❌5)

### pixacc.c (ピクセルアキュムレータ)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaccCreate | ❌ 未実装 | - | アキュムレータ作成 |
| pixaccCreateFromPix | ❌ 未実装 | - | Pixからアキュムレータ作成 |
| pixaccDestroy | 🚫 不要 | - | RustのDrop traitで代替 |
| pixaccFinal | ❌ 未実装 | - | 最終化 |
| pixaccGetPix | ❌ 未実装 | - | Pix取得 |
| pixaccGetOffset | 🚫 不要 | - | Rustではフィールドアクセスで代替 |
| pixaccAdd | ❌ 未実装 | - | 加算 |
| pixaccSubtract | ❌ 未実装 | - | 減算 |
| pixaccMultConst | ❌ 未実装 | - | 定数倍 |
| pixaccMultConstAccumulate | ❌ 未実装 | - | 定数倍加算 |

**pixacc.c カバレッジ**: 0/10 = 0% (🚫2, ❌8)

### sudoku.c (数独ソルバー)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| sudokuReadFile | 🚫 不要 | - | 画像処理に無関係 |
| sudokuReadString | 🚫 不要 | - | 画像処理に無関係 |
| sudokuCreate | 🚫 不要 | - | 画像処理に無関係 |
| sudokuDestroy | 🚫 不要 | - | 画像処理に無関係 |
| sudokuSolve | 🚫 不要 | - | 画像処理に無関係 |
| sudokuTestUniqueness | 🚫 不要 | - | 画像処理に無関係 |
| sudokuGenerate | 🚫 不要 | - | 画像処理に無関係 |
| sudokuOutput | 🚫 不要 | - | 画像処理に無関係、デバッグ用表示関数 |

**sudoku.c カバレッジ**: 🚫全8関数不要 (画像処理に無関係な数独ソルバー)

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

1. **ワーピング関数 (warper.c)**: ほぼ完全実装 (82%)
   - leptonica-transform/src/warper.rs で主要関数を実装済み
   - 🚫 LUT版最適化はRustコンパイラ最適化で代替
   - 未実装: pixSimpleCaptcha (高レベルAPI)

2. **PDFアプリケーション (pdfapp.c)**: 未実装 (0%)
   - 基本的なPDF書き込みはあるが、画像処理統合アプリケーション関数は未実装
   - compressFilesToPdf, cropFilesToPdf等の高レベルAPIが必要

3. **カラーマップペイント (paintcmap.c)**: 未実装 (0%)
   - カラーマップを持つ画像への直接ペイント操作は未実装
   - Rust版ではカラーマップ操作が限定的

4. **圧縮画像コンテナ (pixcomp.c)**: ほぼ未実装 (2%)
   - Pixcomp/Pixacomp データ構造そのものが未実装
   - メモリ効率的な圧縮画像コンテナは重要な最適化機能
   - 🚫 destroy/デバッグ/C固有ヘルパー関数11個は不要

5. **画像ラベリング (pixlabel.c)**: 未実装 (0%)
   - 連結成分ラベリングの高度な機能は未実装
   - 基本的な連結成分解析は region クレートにある可能性

6. **エンコーディング (encoding.c)**: 部分実装 (14%)
   - Ascii85は実装済み (PS/PDF用)
   - Base64, decodeAscii85は未実装
   - 🚫 圧縮付き版とreformatPacked64は不要

7. **ユーティリティ (utils1.c, utils2.c)**: Rust標準ライブラリで代替
   - 文字列操作、ファイルI/O等はRust標準で対応
   - 直接移植は不要

8. **データ構造 (heap, list, stack, queue, ptra, dna)**: 🚫全関数不要
   - Rust標準ライブラリ(BinaryHeap, LinkedList, Vec, VecDeque)で完全に代替
   - 専用APIの移植は不要

9. **特殊機能**:
   - binexpand/binreduce: 0% (❌未実装、二値画像スケーリングで有用)
   - pixtiling: 0% (❌未実装5 + 🚫不要2)
   - pixacc: 0% (❌未実装8 + 🚫不要2)
   - sudoku: 🚫全関数不要 (画像処理に無関係)
   - correlscore: 0% (❌未実装、相関スコア計算で有用)

### 優先度評価

#### 高優先度 (コア機能)
1. **Pixcomp/Pixacomp** (pixcomp.c) - ❌35関数
   - メモリ効率的な圧縮画像配列
   - 大量画像処理で重要

2. **pixtiling** (pixtiling.c) - ❌5関数
   - 大画像の分割処理に必須
   - メモリ効率的な処理の基盤

3. **pixlabel** (pixlabel.c) - ❌6関数
   - 連結成分ラベリングの高度機能
   - 画像解析で重要

#### 中優先度 (便利機能)
4. **paintcmap** (paintcmap.c) - ❌7関数
   - カラーマップ画像の直接操作
   - 特定用途で有用

5. **pdfapp** (pdfapp.c) - ❌3関数
   - バッチ処理の高レベルAPI
   - ユーザビリティ向上

6. **binexpand/binreduce** - ❌6関数
   - 二値画像専用の高速拡大縮小
   - パフォーマンス最適化

7. **pixacc** (pixacc.c) - ❌8関数
   - ピクセル累積演算
   - 特定アルゴリズムで必要

#### 低優先度 (特殊用途)
8. **correlscore** (correlscore.c) - ❌4関数
   - 相関スコア計算
   - 特定用途

9. **encoding** (encoding.c) - ❌3関数
   - Base64/decodeAscii85: 外部クレート使用可能
   - Ascii85: 実装済み

10. **warper残り** (warper.c) - ❌1関数
    - pixSimpleCaptcha実装

#### 移植不要 (🚫 不要)
- **データ構造** (heap, list, stack, queue, ptra, dna) - 🚫67関数
  - Rust標準ライブラリで完全代替
- **sudoku** - 🚫8関数
  - 画像処理に無関係
- **各モジュールのdestroy/デバッグ関数** - 🚫19関数
  - RustのDrop trait、フィールドアクセス等で代替

### 移植戦略

1. **Phase 1: コアインフラ** (❌46関数)
   - Pixcomp/Pixacomp データ構造
   - pixtiling
   - pixlabel

2. **Phase 2: 便利機能** (❌24関数)
   - paintcmap
   - pdfapp高レベルAPI
   - binexpand/binreduce

3. **Phase 3: 最適化・特殊機能** (❌8関数)
   - pixacc
   - correlscore
   - その他必要に応じて

4. **移植不要** (🚫94関数)
   - データ構造系 (Rust標準ライブラリで代替)
   - sudoku (画像処理に無関係)
   - 各モジュールのdestroy/デバッグ/C固有関数

## 推奨事項

1. **Pixcompの実装を最優先** (❌35関数)
   - 大量画像処理での メモリ効率が重要
   - PDF生成等で必要

2. **pixtilingの実装** (❌5関数)
   - 大画像処理の基盤
   - メモリ制約下での処理に必須

3. **pixlabelの実装** (❌6関数)
   - 連結成分解析の高度機能
   - 画像解析アプリケーションで必要

4. **データ構造はRust標準を使用** (🚫不要と確定)
   - heap → BinaryHeap
   - list → LinkedList/Vec
   - stack → Vec
   - queue → VecDeque
   - ptra → Vec<Option<T>>
   - dna → Vec<f64>

5. **warper.cの残り機能** (❌1関数)
   - pixSimpleCaptcha実装

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
