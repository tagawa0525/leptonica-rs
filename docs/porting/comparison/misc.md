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

| 項目      | 数  |
| --------- | --- |
| ✅ 同等   | 144 |
| 🔄 異なる | 0   |
| 🚫 不要   | 176 |
| ❌ 未実装 | 0   |
| 合計      | 320 |

注: この集計は主要な公開関数のみをカウント。静的(内部)関数は除外。🚫はRust標準ライブラリで代替可能、C固有管理関数、画像処理に無関係等の理由で移植不要と判断したもの。

## 詳細

### warper.c (画像ワーピング)

| C関数                       | 状態 | Rust対応                   | 備考                                    |
| --------------------------- | ---- | -------------------------- | --------------------------------------- |
| pixSimpleCaptcha            | ✅   | warper::simple_captcha     | CAPTCHA生成の高レベルインターフェース   |
| pixRandomHarmonicWarp       | ✅   | random_harmonic_warp       | ランダム正弦波ワーピング                |
| pixRandomHarmonicWarpLUT    | 🚫   | -                          | LUT版最適化はRustコンパイラ最適化で代替 |
| pixWarpStereoscopic         | ✅   | warp_stereoscopic          | ステレオスコピックワーピング            |
| pixStretchHorizontal        | ✅   | stretch_horizontal         | 水平方向伸縮                            |
| pixStretchHorizontalSampled | ✅   | stretch_horizontal_sampled | サンプリング版                          |
| pixStretchHorizontalLI      | ✅   | stretch_horizontal_li      | 線形補間版                              |
| pixQuadraticVShear          | ✅   | quadratic_v_shear          | 二次垂直シアー                          |
| pixQuadraticVShearSampled   | ✅   | quadratic_v_shear_sampled  | サンプリング版                          |
| pixQuadraticVShearLI        | ✅   | quadratic_v_shear_li       | 線形補間版                              |
| pixStereoFromPair           | ✅   | stereo_from_pair           | ステレオペア合成                        |

**warper.c カバレッジ**: 10/11 = 91% (✅10, 🚫1)

### pdfapp.c (PDFアプリケーション)

| C関数                 | 状態 | Rust対応                   | 備考                  |
| --------------------- | ---- | -------------------------- | --------------------- |
| compressFilesToPdf    | ✅   | compress_files_to_pdf      | 画像圧縮してPDF化     |
| cropFilesToPdf        | ✅   | crop_files_to_pdf          | 画像クロップしてPDF化 |
| cleanTo1bppFilesToPdf | ✅   | clean_to_1bpp_files_to_pdf | 1bpp変換してPDF化     |

**pdfapp.c カバレッジ**: 3/3 = 100%

### paintcmap.c (カラーマップペイント)

| C関数                   | 状態 | Rust対応                    | 備考                               |
| ----------------------- | ---- | --------------------------- | ---------------------------------- |
| pixSetSelectCmap        | ✅   | pix_set_select_cmap         | カラーマップ内の特定ピクセル再塗装 |
| pixColorGrayRegionsCmap | ✅   | pix_color_gray_regions_cmap | 領域内グレーピクセル着色           |
| pixColorGrayCmap        | ✅   | pix_color_gray_cmap         | グレーピクセル着色                 |
| pixColorGrayMaskedCmap  | ✅   | pix_color_gray_masked_cmap  | マスク通してグレーピクセル着色     |
| addColorizedGrayToCmap  | ✅   | add_colorized_gray_to_cmap  | カラーマップに着色グレー追加       |
| pixSetSelectMaskedCmap  | ✅   | pix_set_select_masked_cmap  | マスク通して特定ピクセル設定       |
| pixSetMaskedCmap        | ✅   | pix_set_masked_cmap         | マスク通して全ピクセル設定         |

**paintcmap.c カバレッジ**: 7/7 = 100%

### pixcomp.c (圧縮画像コンテナ)

| C関数                         | 状態 | Rust対応                          | 備考                                               |
| ----------------------------- | ---- | --------------------------------- | -------------------------------------------------- |
| pixcompCreateFromPix          | ✅   | pixcomp_create_from_pix           | PixからPixcomp作成                                 |
| pixcompCreateFromString       | ✅   | pixcomp_create_from_string        | 文字列からPixcomp作成                              |
| pixcompCreateFromFile         | ✅   | pixcomp_create_from_file          | ファイルからPixcomp作成                            |
| pixcompDestroy                | 🚫   | -                                 | RustのDrop traitで代替                             |
| pixcompCopy                   | ✅   | pixcomp_copy                      | Pixcompコピー                                      |
| pixcompGetDimensions          | ✅   | pixcomp_get_dimensions            | 寸法取得                                           |
| pixcompGetParameters          | ✅   | pixcomp_get_parameters            | パラメータ取得                                     |
| pixcompDetermineFormat        | ✅   | pixcomp_determine_format          | フォーマット決定                                   |
| pixCreateFromPixcomp          | ✅   | PixComp::to_pix                   | PixcompからPix作成                                 |
| pixacompCreate                | ✅   | pixacomp_create                   | Pixacomp配列作成                                   |
| pixacompCreateWithInit        | ✅   | pixacomp_create_with_init         | 初期化付き作成                                     |
| pixacompCreateFromPixa        | ✅   | pixacomp_create_from_pixa         | PixaからPixacomp作成                               |
| pixacompCreateFromFiles       | ✅   | pixacomp_create_from_files        | ファイルからPixacomp作成                           |
| pixacompCreateFromSA          | 🚫   | -                                 | C版SArray固有、RustではVec<PathBuf>等で代替        |
| pixacompDestroy               | 🚫   | -                                 | RustのDrop traitで代替                             |
| pixacompAddPix                | ✅   | pixacomp_add_pix                  | Pix追加                                            |
| pixacompAddPixcomp            | ✅   | pixacomp_add_pixcomp              | Pixcomp追加                                        |
| pixacompReplacePix            | ✅   | pixacomp_replace_pix              | Pix置換                                            |
| pixacompReplacePixcomp        | 🚫   | -                                 | pixacompReplacePixで代替可能                       |
| pixacompAddBox                | 🚫   | -                                 | Rust版ではBoxa操作はBoxa型に委譲                   |
| pixacompGetCount              | ✅   | pixacomp_get_count                | カウント取得                                       |
| pixacompGetPixcomp            | ✅   | pixacomp_get_pixcomp              | Pixcomp取得                                        |
| pixacompGetPix                | ✅   | pixacomp_get_pix                  | Pix取得                                            |
| pixacompGetPixDimensions      | ✅   | pixacomp_get_pix_dimensions       | Pixの寸法取得                                      |
| pixacompGetBoxa               | ✅   | pixacomp_get_boxa                 | Boxa取得                                           |
| pixacompGetBoxaCount          | 🚫   | -                                 | Boxa型のlen()で代替                                |
| pixacompGetBox                | ✅   | pixacomp_get_box                  | Box取得                                            |
| pixacompGetBoxGeometry        | ✅   | pixacomp_get_box_geometry         | Box座標取得                                        |
| pixacompGetOffset             | 🚫   | -                                 | Rustではフィールドアクセスで代替                   |
| pixacompSetOffset             | 🚫   | -                                 | Rustではフィールドアクセスで代替                   |
| pixaCreateFromPixacomp        | ✅   | PixaComp::to_pixa                 | PixacompからPixa作成                               |
| pixacompJoin                  | ✅   | pixacomp_join                     | Pixacomp結合                                       |
| pixacompInterleave            | ✅   | pixacomp_interleave               | Pixacompインターリーブ                             |
| pixacompRead                  | ✅   | pixacomp_read                     | ファイル読み込み                                   |
| pixacompReadStream            | ✅   | pixacomp_read_stream              | ストリーム読み込み                                 |
| pixacompReadMem               | ✅   | pixacomp_read_mem                 | メモリから読み込み                                 |
| pixacompWrite                 | ✅   | pixacomp_write                    | ファイル書き込み                                   |
| pixacompWriteStream           | ✅   | pixacomp_write_stream             | ストリーム書き込み                                 |
| pixacompWriteMem              | ✅   | pixacomp_write_mem                | メモリに書き込み                                   |
| pixacompConvertToPdf          | 🔄   | PixaComp::convert_to_pdf_data     | PDFデータ生成APIとして提供（`src/io/pdf.rs` 経由） |
| pixacompConvertToPdfData      | ✅   | pixacomp_convert_to_pdf_data      | PDF データ生成                                     |
| pixacompFastConvertToPdfData  | ✅   | pixacomp_fast_convert_to_pdf_data | 高速PDF データ生成                                 |
| pixacompWriteStreamInfo       | 🚫   | -                                 | デバッグ用表示関数                                 |
| pixcompWriteStreamInfo        | 🚫   | -                                 | デバッグ用表示関数                                 |
| pixacompDisplayTiledAndScaled | 🚫   | -                                 | デバッグ用表示関数                                 |
| pixacompWriteFiles            | ✅   | pixacomp_write_files              | ファイル群書き込み                                 |
| pixcompWriteFile              | ✅   | pixcomp_write_file                | ファイル書き込み                                   |

**pixcomp.c カバレッジ**: 36/47 = 77% (✅36, 🚫11)
**注**: Pixcomp/Pixacmp (圧縮画像コンテナ)データ構造実装済み。destroy/デバッグ/C固有関数は🚫。

### pixlabel.c (画像ラベリング)

| C関数                      | 状態 | Rust対応                                          | 備考                     |
| -------------------------- | ---- | ------------------------------------------------- | ------------------------ |
| pixConnCompTransform       | ✅   | conn_comp_transform                               | 連結成分変換             |
| pixConnCompAreaTransform   | ✅   | conn_comp_transform(..., ConnCompTransform::Area) | 面積ベース連結成分変換   |
| pixConnCompIncrInit        | ✅   | conn_comp_incr_init                               | インクリメンタルCC初期化 |
| pixConnCompIncrAdd         | ✅   | conn_comp_incr_add                                | インクリメンタルCC追加   |
| pixGetSortedNeighborValues | ✅   | get_sorted_neighbor_values                        | ソート済み近傍値取得     |
| pixLocToColorTransform     | ✅   | loc_to_color_transform                            | 位置->色変換             |

**pixlabel.c カバレッジ**: 6/6 = 100% (✅6)

### partition.c (パーティション・ホワイトブロック)

| C関数                    | 状態 | Rust対応                | 備考                               |
| ------------------------ | ---- | ----------------------- | ---------------------------------- |
| boxaGetWhiteblocks       | ✅   | get_whiteblocks         | ホワイトブロック検出               |
| boxaPruneSortedOnOverlap | ✅   | prune_sorted_on_overlap | オーバーラップに基づくプルーニング |

**partition.c カバレッジ**: 2/2 = 100% (✅2)

### partify.c (Pixac分割)

| C関数        | 状態 | Rust対応      | 備考         |
| ------------ | ---- | ------------- | ------------ |
| partifyFiles | ✅   | partify_files | ファイル分割 |
| partifyPixac | ✅   | partify_pixac | Pixac分割    |

**partify.c カバレッジ**: 2/2 = 100% (✅2)

### encoding.c (エンコーディング)

| C関数                 | 状態 | Rust対応       | 備考                                     |
| --------------------- | ---- | -------------- | ---------------------------------------- |
| encodeBase64          | ✅   | encode_base64  | Base64エンコード                         |
| decodeBase64          | ✅   | decode_base64  | Base64デコード                           |
| encodeAscii85         | 🚫   | -              | Rust版では公開APIなし（内部実装のみ）    |
| decodeAscii85         | ✅   | decode_ascii85 | 公開APIとして実装済み                    |
| encodeAscii85WithComp | 🚫   | -              | zlib圧縮付き、Rustではflate2等で個別対応 |
| decodeAscii85WithComp | 🚫   | -              | zlib解凍付き、Rustではflate2等で個別対応 |
| reformatPacked64      | 🚫   | -              | C版内部フォーマット関数                  |

**encoding.c カバレッジ**: 3/7 = 43% (✅3, 🚫4)
**注**: Ascii85は `src/io/ps/ascii85.rs` に実装済み。圧縮付き版はRustでは個別ライブラリ組合せで代替。

### utils1.c, utils2.c (ユーティリティ)

utils1.cとutils2.cには多数の文字列操作、ファイルI/O、メモリ管理などの低レベルユーティリティ関数が含まれる。Rust版では標準ライブラリ(std::fs, std::string, Vec等)で代替可能なため、直接的な移植は不要。

**主な関数分類**:

- 文字列操作: stringNew, stringCopy, stringJoin, stringReverse, etc. → Rust String/&str
- ファイルI/O: l_binaryRead, l_binaryWrite, fileCopy, etc. → std::fs, std::io
- メモリ管理: reallocNew → Vec::resize, etc.
- 配列操作: arrayFindSequence, arrayReplaceEachSequence → Rustのイテレータ/スライス操作

**カバレッジ**: 実質100% (Rust標準ライブラリで代替)

### heap.c (優先度キュー)

| C関数                | 状態 | Rust対応 | 備考                               |
| -------------------- | ---- | -------- | ---------------------------------- |
| lheapCreate          | 🚫   | -        | std::collections::BinaryHeapで代替 |
| lheapDestroy         | 🚫   | -        | RustのDrop traitで代替             |
| lheapAdd             | 🚫   | -        | BinaryHeap::pushで代替             |
| lheapRemove          | 🚫   | -        | BinaryHeap::popで代替              |
| lheapGetCount        | 🚫   | -        | BinaryHeap::lenで代替              |
| lheapGetElement      | 🚫   | -        | BinaryHeap::peekで代替             |
| lheapSort            | 🚫   | -        | BinaryHeap::into_sorted_vecで代替  |
| lheapSortStrictOrder | 🚫   | -        | BinaryHeap::into_sorted_vecで代替  |
| lheapPrint           | 🚫   | -        | デバッグ用表示関数                 |

**heap.c カバレッジ**: 🚫全9関数不要 (std::collections::BinaryHeapで代替)

### list.c (双方向リンクリスト)

| C関数              | 状態 | Rust対応 | 備考                                           |
| ------------------ | ---- | -------- | ---------------------------------------------- |
| listDestroy        | 🚫   | -        | RustのDrop traitで代替                         |
| listAddToHead      | 🚫   | -        | Vec::insert(0,x)やLinkedList::push_frontで代替 |
| listAddToTail      | 🚫   | -        | Vec::pushやLinkedList::push_backで代替         |
| listInsertBefore   | 🚫   | -        | Vec::insertで代替                              |
| listInsertAfter    | 🚫   | -        | Vec::insertで代替                              |
| listRemoveElement  | 🚫   | -        | Vec::removeで代替                              |
| listRemoveFromHead | 🚫   | -        | Vec::remove(0)で代替                           |
| listRemoveFromTail | 🚫   | -        | Vec::popで代替                                 |
| listFindElement    | 🚫   | -        | イテレータのfindで代替                         |
| listFindTail       | 🚫   | -        | Vec::lastで代替                                |
| listGetCount       | 🚫   | -        | Vec::lenで代替                                 |
| listReverse        | 🚫   | -        | Vec::reverseで代替                             |
| listJoin           | 🚫   | -        | Vec::extendで代替                              |

**list.c カバレッジ**: 🚫全13関数不要 (Vec/LinkedListで代替)

### stack.c (スタック)

| C関数          | 状態 | Rust対応 | 備考                   |
| -------------- | ---- | -------- | ---------------------- |
| lstackCreate   | 🚫   | -        | Vec::newで代替         |
| lstackDestroy  | 🚫   | -        | RustのDrop traitで代替 |
| lstackAdd      | 🚫   | -        | Vec::pushで代替        |
| lstackRemove   | 🚫   | -        | Vec::popで代替         |
| lstackGetCount | 🚫   | -        | Vec::lenで代替         |
| lstackPrint    | 🚫   | -        | デバッグ用表示関数     |

**stack.c カバレッジ**: 🚫全6関数不要 (Vec<T>のpush/popで代替)

### queue.c (キュー)

| C関数          | 状態 | Rust対応 | 備考                      |
| -------------- | ---- | -------- | ------------------------- |
| lqueueCreate   | 🚫   | -        | VecDeque::newで代替       |
| lqueueDestroy  | 🚫   | -        | RustのDrop traitで代替    |
| lqueueAdd      | 🚫   | -        | VecDeque::push_backで代替 |
| lqueueRemove   | 🚫   | -        | VecDeque::pop_frontで代替 |
| lqueueGetCount | 🚫   | -        | VecDeque::lenで代替       |
| lqueuePrint    | 🚫   | -        | デバッグ用表示関数        |

**queue.c カバレッジ**: 🚫全6関数不要 (std::collections::VecDequeで代替)

### ptra.c (ポインタ配列)

| C関数              | 状態 | Rust対応 | 備考                           |
| ------------------ | ---- | -------- | ------------------------------ |
| ptraCreate         | 🚫   | -        | Vec::newで代替                 |
| ptraDestroy        | 🚫   | -        | RustのDrop traitで代替         |
| ptraAdd            | 🚫   | -        | Vec::pushで代替                |
| ptraInsert         | 🚫   | -        | Vec::insertで代替              |
| ptraRemove         | 🚫   | -        | Vec::removeで代替              |
| ptraRemoveLast     | 🚫   | -        | Vec::popで代替                 |
| ptraReplace        | 🚫   | -        | インデックスアクセスで代替     |
| ptraSwap           | 🚫   | -        | Vec::swapで代替                |
| ptraCompactArray   | 🚫   | -        | Vec::retain等で代替            |
| ptraReverse        | 🚫   | -        | Vec::reverseで代替             |
| ptraJoin           | 🚫   | -        | Vec::extendで代替              |
| ptraGetMaxIndex    | 🚫   | -        | Vec::lenで代替                 |
| ptraGetActualCount | 🚫   | -        | イテレータのfilter+countで代替 |
| ptraGetPtrToItem   | 🚫   | -        | インデックスアクセスで代替     |
| ptraaCreate        | 🚫   | -        | Vec<Vec<T>>で代替              |
| ptraaDestroy       | 🚫   | -        | RustのDrop traitで代替         |
| ptraaGetSize       | 🚫   | -        | Vec::lenで代替                 |
| ptraaInsertPtra    | 🚫   | -        | Vec::insertで代替              |
| ptraaGetPtra       | 🚫   | -        | インデックスアクセスで代替     |
| ptraaFlattenToPtra | 🚫   | -        | Iterator::flattenで代替        |

**ptra.c カバレッジ**: 🚫全20関数不要 (Vec<Option<Box<T>>>で代替)

### dnabasic.c, dnafunc1.c (Double Number Array)

| C関数                                 | 状態 | Rust対応 | 備考                                                 |
| ------------------------------------- | ---- | -------- | ---------------------------------------------------- |
| l_dnaCreate                           | 🚫   | -        | Vec::<f64>::newで代替                                |
| l_dnaCreateFromIArray                 | 🚫   | -        | Vec::from / iter().map().collectで代替               |
| l_dnaCreateFromDArray                 | 🚫   | -        | Vec::fromで代替                                      |
| l_dnaMakeSequence                     | 🚫   | -        | Iterator::mapで代替                                  |
| l_dnaDestroy                          | 🚫   | -        | RustのDrop traitで代替                               |
| l_dnaCopy                             | 🚫   | -        | Vec::cloneで代替                                     |
| l_dnaClone                            | 🚫   | -        | Arc<Vec<f64>>等で代替                                |
| l_dnaEmpty                            | 🚫   | -        | Vec::clearで代替                                     |
| l_dnaAddNumber                        | 🚫   | -        | Vec::pushで代替                                      |
| l_dnaInsertNumber                     | 🚫   | -        | Vec::insertで代替                                    |
| l_dnaRemoveNumber                     | 🚫   | -        | Vec::removeで代替                                    |
| l_dnaReplaceNumber                    | 🚫   | -        | インデックスアクセスで代替                           |
| (その他 dnabasic.c / dnafunc1.c 多数) | 🚫   | -        | 配列操作・統計・I/O等すべてVec<f64>+イテレータで代替 |

**dnabasic.c / dnafunc1.c カバレッジ**: 🚫全40+関数不要 (Vec<f64>+イテレータで代替)
**注**: C版のL_DNA(double配列)に相当する専用データ構造は不要。Rust版ではVec<f64>と標準ライブラリのイテレータ/スライスメソッドで完全に代替可能。

### bytearray.c (バイト配列)

| C関数                   | 状態 | Rust対応 | 備考                     |
| ----------------------- | ---- | -------- | ------------------------ |
| l_byteaCreate           | 🚫   | -        | Vec::<u8>::newで代替     |
| l_byteaInitFromMem      | 🚫   | -        | Vec::fromで代替          |
| l_byteaInitFromFile     | 🚫   | -        | std::fs::readで代替      |
| l_byteaInitFromStream   | 🚫   | -        | Read trait経由で代替     |
| l_byteaCopy             | 🚫   | -        | Vec::cloneで代替         |
| l_byteaDestroy          | 🚫   | -        | RustのDrop traitで代替   |
| l_byteaGetSize          | 🚫   | -        | Vec::lenで代替           |
| l_byteaGetData          | 🚫   | -        | スライスアクセスで代替   |
| l_byteaCopyData         | 🚫   | -        | Vec::cloneで代替         |
| l_byteaAppendData       | 🚫   | -        | Vec::extendで代替        |
| l_byteaAppendString     | 🚫   | -        | Vec::extendで代替        |
| l_byteaJoin             | 🚫   | -        | Vec::extendで代替        |
| l_byteaSplit            | 🚫   | -        | split系イテレータで代替  |
| l_byteaFindEachSequence | 🚫   | -        | イテレータのwindowで代替 |
| l_byteaWrite            | 🚫   | -        | std::fs::writeで代替     |
| l_byteaWriteStream      | 🚫   | -        | Write trait経由で代替    |

**bytearray.c カバレッジ**: 🚫全16関数不要 (Vec<u8>で代替)

### bbuffer.c (バイトバッファ)

| C関数              | 状態 | Rust対応 | 備考                   |
| ------------------ | ---- | -------- | ---------------------- |
| bbufferCreate      | 🚫   | -        | Vec::<u8>::newで代替   |
| bbufferDestroy     | 🚫   | -        | RustのDrop traitで代替 |
| bbufferRead        | 🚫   | -        | std::io::Readで代替    |
| bbufferWrite       | 🚫   | -        | std::io::Writeで代替   |
| bbufferGetSize     | 🚫   | -        | Vec::lenで代替         |
| bbufferGetData     | 🚫   | -        | スライスアクセスで代替 |
| bbufferReadStream  | 🚫   | -        | std::io::Readで代替    |
| bbufferWriteStream | 🚫   | -        | std::io::Writeで代替   |

**bbuffer.c カバレッジ**: 🚫全8関数不要 (Vec<u8>/std::ioで代替)

### dnahash.c (DNA ハッシュマップ)

| C関数            | 状態 | Rust対応 | 備考                          |
| ---------------- | ---- | -------- | ----------------------------- |
| l_dnaHashCreate  | 🚫   | -        | HashMap::<i32,f64>::newで代替 |
| l_dnaHashDestroy | 🚫   | -        | RustのDrop traitで代替        |
| l_dnaHashGetDna  | 🚫   | -        | HashMap::getで代替            |
| l_dnaHashAdd     | 🚫   | -        | HashMap::insertで代替         |

**dnahash.c カバレッジ**: 🚫全4関数不要 (HashMap<i32,f64>で代替)

### hashmap.c (ハッシュマップ)

| C関数               | 状態 | Rust対応 | 備考                           |
| ------------------- | ---- | -------- | ------------------------------ |
| l_hmapCreate        | 🚫   | -        | HashMap::newで代替             |
| l_hmapCreateFromDna | 🚫   | -        | HashMap::fromで代替            |
| l_hmapDestroy       | 🚫   | -        | RustのDrop traitで代替         |
| l_hmapLookup        | 🚫   | -        | HashMap::getで代替             |
| l_hmapRehash        | 🚫   | -        | 不要（Rustの自動リハッシング） |

**hashmap.c カバレッジ**: 🚫全5関数不要 (HashMap<K,V>で代替)

### map.c, rbtree.c (Red-Black Treeマップ)

| C関数               | 状態 | Rust対応 | 備考                          |
| ------------------- | ---- | -------- | ----------------------------- |
| l_amapCreate        | 🚫   | -        | BTreeMap::newで代替           |
| l_amapFind          | 🚫   | -        | BTreeMap::getで代替           |
| l_amapInsert        | 🚫   | -        | BTreeMap::insertで代替        |
| l_amapDelete        | 🚫   | -        | BTreeMap::removeで代替        |
| l_amapDestroy       | 🚫   | -        | RustのDrop traitで代替        |
| l_amapGetFirst      | 🚫   | -        | BTreeMap::iter().next()で代替 |
| l_amapGetNext       | 🚫   | -        | イテレータのnextで代替        |
| l_amapGetLast       | 🚫   | -        | BTreeMap::iter().last()で代替 |
| l_amapGetPrev       | 🚫   | -        | イテレータで代替              |
| l_amapSize          | 🚫   | -        | BTreeMap::lenで代替           |
| l_asetCreate        | 🚫   | -        | BTreeSet::newで代替           |
| l_asetCreateFromDna | 🚫   | -        | BTreeSet::from_iterで代替     |
| l_asetFind          | 🚫   | -        | BTreeSet::containsで代替      |
| l_asetInsert        | 🚫   | -        | BTreeSet::insertで代替        |
| l_asetDelete        | 🚫   | -        | BTreeSet::removeで代替        |
| l_asetDestroy       | 🚫   | -        | RustのDrop traitで代替        |
| l_rbtreeCreate      | 🚫   | -        | BTreeMap::newで代替           |
| l_rbtreeDestroy     | 🚫   | -        | RustのDrop traitで代替        |
| l_rbtreeGetFirst    | 🚫   | -        | BTreeMap::iter().next()で代替 |
| l_rbtreeGetNext     | 🚫   | -        | イテレータのnextで代替        |

**map.c / rbtree.c カバレッジ**: 🚫全20関数不要 (BTreeMap/BTreeSetで代替)

### binexpand.c, binreduce.c (二値画像拡大・縮小)

| C関数                      | 状態 | Rust対応                   | 備考                       |
| -------------------------- | ---- | -------------------------- | -------------------------- |
| pixExpandBinaryReplicate   | ✅   | expand_binary_replicate    | 複製拡大                   |
| pixExpandBinaryPower2      | ✅   | expand_binary_power2       | 2のべき乗拡大              |
| pixReduceBinary2           | ✅   | reduce_binary_2            | 2倍縮小                    |
| pixReduceRankBinaryCascade | ✅   | reduce_rank_binary_cascade | ランクベースカスケード縮小 |
| pixReduceRankBinary2       | ✅   | reduce_rank_binary_2       | ランクベース2倍縮小        |
| makeSubsampleTab2x         | ✅   | make_subsample_tab2x       | サブサンプルテーブル生成   |

**binexpand.c / binreduce.c カバレッジ**: 6/6 = 100% (✅6)

### pixtiling.c (画像タイリング)

| C関数                   | 状態 | Rust対応                     | 備考                           |
| ----------------------- | ---- | ---------------------------- | ------------------------------ |
| pixTilingCreate         | ✅   | tiling_create                | タイリング作成                 |
| pixTilingDestroy        | 🚫   | -                            | RustのDrop traitで代替         |
| pixTilingGetCount       | ✅   | tiling_get_count             | タイル数取得                   |
| pixTilingGetSize        | ✅   | tiling_get_size              | タイルサイズ取得               |
| pixTilingGetTile        | ✅   | tiling_get_tile              | タイル取得                     |
| pixTilingNoStripOnPaint | ✅   | PixTiling::no_strip_on_paint | ペイント時ストリップ除去無効化 |
| pixTilingPaintTile      | ✅   | tiling_paint_tile            | タイルペイント                 |

**pixtiling.c カバレッジ**: 6/7 = 86% (✅6, 🚫1)

### pixacc.c (ピクセルアキュムレータ)

| C関数                     | 状態 | Rust対応                     | 備考                             |
| ------------------------- | ---- | ---------------------------- | -------------------------------- |
| pixaccCreate              | ✅   | pixacc_create                | アキュムレータ作成               |
| pixaccCreateFromPix       | ✅   | pixacc_create_from_pix       | Pixからアキュムレータ作成        |
| pixaccDestroy             | 🚫   | -                            | RustのDrop traitで代替           |
| pixaccFinal               | ✅   | PixAcc::finish               | 最終化                           |
| pixaccGetPix              | ✅   | pixacc_get_pix               | Pix取得                          |
| pixaccGetOffset           | 🚫   | -                            | Rustではフィールドアクセスで代替 |
| pixaccAdd                 | ✅   | pixacc_add                   | 加算                             |
| pixaccSubtract            | ✅   | pixacc_subtract              | 減算                             |
| pixaccMultConst           | ✅   | pixacc_mult_const            | 定数倍                           |
| pixaccMultConstAccumulate | ✅   | pixacc_mult_const_accumulate | 定数倍加算                       |

**pixacc.c カバレッジ**: 8/10 = 80% (✅8, 🚫2)

### sudoku.c (数独ソルバー)

| C関数                | 状態 | Rust対応 | 備考                                 |
| -------------------- | ---- | -------- | ------------------------------------ |
| sudokuReadFile       | 🚫   | -        | 画像処理に無関係                     |
| sudokuReadString     | 🚫   | -        | 画像処理に無関係                     |
| sudokuCreate         | 🚫   | -        | 画像処理に無関係                     |
| sudokuDestroy        | 🚫   | -        | 画像処理に無関係                     |
| sudokuSolve          | 🚫   | -        | 画像処理に無関係                     |
| sudokuTestUniqueness | 🚫   | -        | 画像処理に無関係                     |
| sudokuGenerate       | 🚫   | -        | 画像処理に無関係                     |
| sudokuOutput         | 🚫   | -        | 画像処理に無関係、デバッグ用表示関数 |

**sudoku.c カバレッジ**: 🚫全8関数不要 (画像処理に無関係な数独ソルバー)

### correlscore.c (相関スコア)

| C関数                          | 状態 | Rust対応                      | 備考               |
| ------------------------------ | ---- | ----------------------------- | ------------------ |
| pixCorrelationScore            | ✅   | correlation_score             | 相関スコア計算     |
| pixCorrelationScoreThresholded | ✅   | correlation_score_thresholded | 閾値付き相関スコア |
| pixCorrelationScoreSimple      | ✅   | correlation_score_simple      | 単純相関スコア     |
| pixCorrelationScoreShifted     | ✅   | correlation_score_shifted     | シフト相関スコア   |

**correlscore.c カバレッジ**: 4/4 = 100%

### textops.c (テキスト操作)

| C関数                    | 状態 | Rust対応                  | 備考                   |
| ------------------------ | ---- | ------------------------- | ---------------------- |
| pixAddTextlines          | ✅   | add_textlines             | テキストライン追加     |
| pixSetTextblock          | ✅   | set_textblock             | テキストブロック設定   |
| pixSetTextline           | ✅   | set_textline              | テキストライン設定     |
| pixaAddTextNumber        | ✅   | pixa_add_text_number      | テキスト番号付き追加   |
| pixaAddTextlines         | ✅   | pixa_add_textlines        | テキストライン群追加   |
| pixaAddPixWithText       | ✅   | pixa_add_pix_with_text    | テキスト付きPix追加    |
| pixAddBorder             | ✅   | add_border                | 境界追加               |
| pixAddBorderGeneral      | ✅   | add_border_general        | 汎用境界追加           |
| pixAddBlackOrWhiteBorder | ✅   | add_black_or_white_border | 白黒境界追加           |
| pixAddMirroredBorder     | ✅   | add_mirrored_border       | ミラー境界追加         |
| pixRemoveBorder          | ✅   | remove_border             | 境界除去               |
| pixRemoveBorderGeneral   | ✅   | remove_border_general     | 汎用境界除去           |
| pixRemoveBorderToSize    | ✅   | remove_border_to_size     | 指定サイズまで境界除去 |
| pixSetText               | ✅   | set_text                  | テキスト設定           |
| pixAddText               | ✅   | add_text                  | テキスト追加           |
| pixSetTextCompNew        | ✅   | set_text_comp_new         | 圧縮テキスト設定       |
| bmfGetLineStrings        | ✅   | bmf_get_line_strings      | ライン文字列取得       |
| bmfGetWordWidths         | ✅   | bmf_get_word_widths       | 単語幅取得             |
| bmfGetStringWidth        | ✅   | bmf_get_string_width      | 文字列幅取得           |

**textops.c カバレッジ**: 19/19 = 100% (✅19)

### bmf.c (ビットマップフォント)

| C関数          | 状態 | Rust対応           | 備考                     |
| -------------- | ---- | ------------------ | ------------------------ |
| bmfCreate      | ✅   | Bmf::new           | ビットマップフォント作成 |
| bmfDestroy     | 🚫   | -                  | RustのDrop traitで代替   |
| bmfGetPix      | ✅   | Bmf::get_pix       | フォント画像取得         |
| bmfGetWidth    | ✅   | Bmf::get_width     | フォント幅取得           |
| bmfGetBaseline | ✅   | Bmf::get_baseline  | ベースライン取得         |
| pixaGetFont    | ✅   | Bmf::get_font_pixa | フォント取得             |

**bmf.c カバレッジ**: 5/6 = 83% (✅5, 🚫1)

### gplot.c (Gnuplotグラフ)

| C関数               | 状態 | Rust対応                | 備考                   |
| ------------------- | ---- | ----------------------- | ---------------------- |
| gplotCreate         | ✅   | GPlot::new              | グラフ作成             |
| gplotDestroy        | 🚫   | -                       | RustのDrop traitで代替 |
| gplotAddPlot        | ✅   | GPlot::add_plot         | プロット追加           |
| gplotSetScaling     | ✅   | GPlot::set_scaling      | スケーリング設定       |
| gplotMakeOutputPix  | ✅   | GPlot::make_output_pix  | 出力Pix生成            |
| gplotMakeOutput     | ✅   | GPlot::make_output      | 出力ファイル生成       |
| gplotGenCommandFile | ✅   | GPlot::gen_command_file | コマンドファイル生成   |
| gplotGenDataFiles   | ✅   | GPlot::gen_data_files   | データファイル生成     |
| gplotSimple1        | ✅   | gplot_simple_1          | 単純グラフ1            |
| gplotSimple2        | ✅   | gplot_simple_2          | 単純グラフ2            |
| gplotSimpleN        | ✅   | gplot_simple_n          | 複数グラフ             |
| gplotSimplePix1     | ✅   | gplot_simple_pix_1      | Pix単純グラフ1         |
| gplotSimplePix2     | ✅   | gplot_simple_pix_2      | Pix単純グラフ2         |
| gplotSimplePixN     | ✅   | gplot_simple_pix_n      | Pix複数グラフ          |

**gplot.c カバレッジ**: 13/14 = 93% (✅13, 🚫1)

### strokes.c (線幅検出・変更)

| C関数                 | 状態 | Rust対応                 | 備考         |
| --------------------- | ---- | ------------------------ | ------------ |
| pixFindStrokeLength   | ✅   | find_stroke_length       | 線長検出     |
| pixFindStrokeWidth    | ✅   | find_stroke_width        | 線幅検出     |
| pixaFindStrokeWidth   | ✅   | pixa_find_stroke_width   | 複数線幅検出 |
| pixaModifyStrokeWidth | ✅   | pixa_modify_stroke_width | 線幅変更     |
| pixModifyStrokeWidth  | ✅   | modify_stroke_width      | 単一線幅変更 |
| pixaSetStrokeWidth    | ✅   | pixa_set_stroke_width    | 線幅設定     |
| pixSetStrokeWidth     | ✅   | set_stroke_width         | 単一線幅設定 |

**strokes.c カバレッジ**: 7/7 = 100% (✅7)
**注**: recog内部にprivate fn `set_stroke_width` が存在するが公開APIではない。

### runlength.c (ランレングス変換)

| C関数                     | 状態 | Rust対応                     | 備考                                 |
| ------------------------- | ---- | ---------------------------- | ------------------------------------ |
| pixRunlengthTransform     | ✅   | runlength_transform          | ランレングス変換                     |
| runlengthMembershipOnLine | ✅   | runlength_membership_on_line | ライン上のランレングスメンバーシップ |
| makeMSBitLocTab           | ✅   | make_msbit_loc_tab           | MSBビット位置テーブル生成            |

**runlength.c カバレッジ**: 3/3 = 100% (✅3)

### checkerboard.c (チェッカーボード検出)

| C関数                      | 状態 | Rust対応                  | 備考                         |
| -------------------------- | ---- | ------------------------- | ---------------------------- |
| pixFindCheckerboardCorners | ✅   | find_checkerboard_corners | チェッカーボードコーナー検出 |

**checkerboard.c カバレッジ**: 1/1 = 100% (✅1)

### convertfiles.c (ファイル変換)

| C関数              | 状態 | Rust対応              | 備考                 |
| ------------------ | ---- | --------------------- | -------------------- |
| convertFilesTo1bpp | ✅   | convert_files_to_1bpp | ファイルを1bppに変換 |

**convertfiles.c カバレッジ**: 1/1 = 100% (✅1)

### finditalic.c (イタリック体検出)

| C関数          | 状態 | Rust対応     | 備考                 |
| -------------- | ---- | ------------ | -------------------- |
| pixItalicWords | ✅   | italic_words | イタリック体単語検出 |

**finditalic.c カバレッジ**: 1/1 = 100% (✅1)

### libversions.c (バージョン情報)

| C関数               | 状態 | Rust対応 | 備考                                         |
| ------------------- | ---- | -------- | -------------------------------------------- |
| getImagelibVersions | 🚫   | -        | 外部ライブラリバージョン取得                 |
| getLeptonicaVersion | 🚫   | -        | Leptonica バージョン取得（Cargo.tomlで管理） |

**libversions.c カバレッジ**: 🚫全2関数不要 (Cargo.toml/env!で代替)

### stringcode.c (コード生成ツール)

| C関数                 | 状態 | Rust対応 | 備考                       |
| --------------------- | ---- | -------- | -------------------------- |
| strcodeCreate         | 🚫   | -        | コード生成ツール（開発用） |
| strcodeCreateFromFile | 🚫   | -        | コード生成ツール（開発用） |
| strcodeGenerate       | 🚫   | -        | コード生成ツール（開発用） |
| strcodeFinalize       | 🚫   | -        | コード生成ツール（開発用） |

**stringcode.c カバレッジ**: 🚫全4関数不要 (開発ツール、ビルド時に実行)

### zlibmem.c (Zlib圧縮)

| C関数                      | 状態 | Rust対応 | 備考                 |
| -------------------------- | ---- | -------- | -------------------- |
| zlibCompress               | 🚫   | -        | flate2クレートで代替 |
| zlibUncompress             | 🚫   | -        | flate2クレートで代替 |
| l_compressGrayHistograms   | 🚫   | -        | flate2クレートで代替 |
| l_uncompressGrayHistograms | 🚫   | -        | flate2クレートで代替 |

**zlibmem.c カバレッジ**: 🚫全4関数不要 (flate2クレートで代替)

### pixalloc.c (メモリプール管理)

| C関数                 | 状態 | Rust対応 | 備考                                    |
| --------------------- | ---- | -------- | --------------------------------------- |
| pmsCreate             | 🚫   | -        | メモリプール管理（Rust GCで不要）       |
| pmsDestroy            | 🚫   | -        | RustのDrop traitで代替                  |
| pmsCustomAlloc        | 🚫   | -        | カスタムメモリ割り当て（Rust GCで不要） |
| pmsCustomDealloc      | 🚫   | -        | カスタムメモリ解放（Rust GCで不要）     |
| pmsGetAlloc           | 🚫   | -        | メモリプール管理（Rust GCで不要）       |
| pmsGetLevelForAlloc   | 🚫   | -        | メモリプール管理（Rust GCで不要）       |
| pmsGetLevelForDealloc | 🚫   | -        | メモリプール管理（Rust GCで不要）       |
| pmsLogInfo            | 🚫   | -        | デバッグログ                            |

**pixalloc.c カバレッジ**: 🚫全8関数不要 (Rustの所有権システムで自動管理)

### regutils.c (回帰テストユーティリティ)

| C関数                    | 状態 | Rust対応 | 備考                              |
| ------------------------ | ---- | -------- | --------------------------------- |
| regTestSetup             | 🚫   | -        | テストは Rustの cargo test で実施 |
| regTestCleanup           | 🚫   | -        | テストは Rustの cargo test で実施 |
| regTestCompareValues     | 🚫   | -        | テストは Rustの assert! で実施    |
| regTestCompareStrings    | 🚫   | -        | テストは Rustの assert_eq! で実施 |
| regTestComparePix        | 🚫   | -        | テストは Rust版テスト基盤で実施   |
| regTestCompareSimilarPix | 🚫   | -        | テストは Rust版テスト基盤で実施   |
| regTestCheckFile         | 🚫   | -        | テストは Rust版テスト基盤で実施   |
| regTestCompareFiles      | 🚫   | -        | テストは Rust版テスト基盤で実施   |
| regTestWritePixAndCheck  | 🚫   | -        | テストは Rust版テスト基盤で実施   |
| regTestWriteDataAndCheck | 🚫   | -        | テストは Rust版テスト基盤で実施   |

**regutils.c カバレッジ**: 🚫全10関数不要 (Rust cargo testで代替)

### renderpdf.c (PDFレンダリング)

| C関数            | 状態 | Rust対応 | 備考                         |
| ---------------- | ---- | -------- | ---------------------------- |
| l_pdfRenderFile  | 🚫   | -        | 外部ツール(pdftoppm)利用のみ |
| l_pdfRenderFiles | 🚫   | -        | 外部ツール(pdftoppm)利用のみ |

**renderpdf.c カバレッジ**: 🚫全2関数不要 (外部ツール依存)

### leptwin.c (Windows API統合)

| C関数                | 状態 | Rust対応 | 備考                                         |
| -------------------- | ---- | -------- | -------------------------------------------- |
| pixGetWindowsHBITMAP | 🚫   | -        | Windows HBITMAP変換（Windows専用、移植不要） |

**leptwin.c カバレッジ**: 🚫全1関数不要 (Windows専用)

### parseprotos.c (プロトタイプ解析)

| C関数 | 状態 | Rust対応 | 備考 |
| ----- | ---- | -------- | ---- |

| cleanProtoSignature | 🚫 不要 | - | コード生成ツール（開発用）_

**parseprotos.c カバレッジ**: 🚫全1関数不要 (開発ツール)

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
   - src/transform/warper.rs で主要関数を実装済み
   - 🚫 LUT版最適化はRustコンパイラ最適化で代替
   - pixSimpleCaptcha: 実装済み

2. **PDFアプリケーション (pdfapp.c)**: 実装済み (100%)
   - compressFilesToPdf, cropFilesToPdf, cleanTo1bppFilesToPdf 実装済み

3. **カラーマップペイント (paintcmap.c)**: 実装済み (100%)
   - 全7関数実装済み

4. **圧縮画像コンテナ (pixcomp.c)**: 実装済み (100%)
   - Pixcomp/Pixacomp データ構造実装済み（✅36関数）
   - 🚫 destroy/デバッグ/C固有ヘルパー関数11個は不要

5. **画像ラベリング (pixlabel.c)**: 実装済み (100%)
   - 連結成分ラベリングの高度な機能を含む全6関数実装済み

6. **エンコーディング (encoding.c)**: 部分実装 (43%)
   - Base64 encode/decode と decodeAscii85 を実装済み
   - 🚫 encodeAscii85公開API、圧縮付き版、reformatPacked64は不要

7. **ユーティリティ (utils1.c, utils2.c)**: Rust標準ライブラリで代替
   - 文字列操作、ファイルI/O等はRust標準で対応
   - 直接移植は不要

8. **データ構造 (heap, list, stack, queue, ptra, dna)**: 🚫全関数不要
   - Rust標準ライブラリ(BinaryHeap, LinkedList, Vec, VecDeque)で完全に代替
   - 専用APIの移植は不要

9. **特殊機能**:
   - binexpand/binreduce: 100% (✅実装済み)
   - pixtiling: 86% (✅6 + 🚫不要1)
   - pixacc: 80% (✅8 + 🚫不要2)
   - sudoku: 🚫全関数不要 (画像処理に無関係)
   - correlscore: 100% (✅実装済み)
   - textops: 100% (✅実装済み)
   - bmf: 83% (✅5 + 🚫不要1)
   - gplot: 93% (✅13 + 🚫不要1)
   - strokes: 100% (✅実装済み)
   - runlength: 100% (✅実装済み)
   - partition: 100% (✅実装済み)
   - partify: 100% (✅実装済み)
   - checkerboard: 100% (✅実装済み)
   - convertfiles: 100% (✅実装済み)
   - finditalic: 100% (✅実装済み)

### 優先度評価

#### 高優先度 (コア機能)

1. **Pixcomp/Pixacomp** (pixcomp.c) - ✅36関数 実装済み
   - メモリ効率的な圧縮画像配列
   - 大量画像処理で重要

2. **pixtiling** (pixtiling.c) - ✅6関数 実装済み
   - 大画像の分割処理に必須
   - メモリ効率的な処理の基盤

3. **pixlabel** (pixlabel.c) - ✅6関数 実装済み
   - 連結成分ラベリングの高度機能
   - 画像解析で重要

#### 中優先度 (便利機能)

4. **paintcmap** (paintcmap.c) - ✅7関数 実装済み
   - カラーマップ画像の直接操作
   - 特定用途で有用

5. **pdfapp** (pdfapp.c) - ✅3関数 実装済み
   - バッチ処理の高レベルAPI
   - ユーザビリティ向上

6. **binexpand/binreduce** - ✅6関数 実装済み
   - 二値画像専用の高速拡大縮小
   - パフォーマンス最適化

7. **pixacc** (pixacc.c) - ✅8関数 実装済み
   - ピクセル累積演算
   - 特定アルゴリズムで必要

#### 低優先度 (特殊用途)

8. **correlscore** (correlscore.c) - ✅4関数 実装済み
   - 相関スコア計算
   - 特定用途

9. **encoding** (encoding.c) - ✅3関数 実装済み
   - Base64/decodeAscii85: 外部クレート使用可能
   - Ascii85: 実装済み

10. **warper残り** (warper.c) - ✅1関数 実装済み
    - pixSimpleCaptcha実装済み

#### 移植不要 (🚫 不要)

- **データ構造** (heap, list, stack, queue, ptra, dna) - 🚫67関数
  - Rust標準ライブラリで完全代替
- **sudoku** - 🚫8関数
  - 画像処理に無関係
- **各モジュールのdestroy/デバッグ関数** - 🚫19関数
  - RustのDrop trait、フィールドアクセス等で代替

### 移植戦略

1. **Phase 1: コアインフラ** (✅46関数 実装済み)
   - Pixcomp/Pixacomp データ構造
   - pixtiling
   - pixlabel

2. **Phase 2: 便利機能** (✅24関数 実装済み)
   - paintcmap
   - pdfapp高レベルAPI
   - binexpand/binreduce

3. **Phase 3: 最適化・特殊機能** (✅8関数 実装済み)
   - pixacc
   - correlscore
   - その他必要に応じて

4. **移植不要** (🚫94関数)
   - データ構造系 (Rust標準ライブラリで代替)
   - sudoku (画像処理に無関係)
   - 各モジュールのdestroy/デバッグ/C固有関数

## 推奨事項

1. **Pixcomp** (✅36関数 実装済み)
   - 大量画像処理でのメモリ効率が重要
   - PDF生成等で必要

2. **pixtiling** (✅6関数 実装済み)
   - 大画像処理の基盤
   - メモリ制約下での処理に必須

3. **pixlabel** (✅6関数 実装済み)
   - 連結成分解析の高度機能
   - 画像解析アプリケーションで必要

4. **データ構造はRust標準を使用** (🚫不要と確定)
   - heap → BinaryHeap
   - list → LinkedList/Vec
   - stack → Vec
   - queue → VecDeque
   - ptra → Vec<Option<T>>
   - dna → Vec<f64>

5. **warper.cの残り機能** (✅1関数 実装済み)
   - pixSimpleCaptcha実装済み

## 参考: C版とRust版の対応

| C版データ構造 | Rust標準ライブラリ代替       |
| ------------- | ---------------------------- |
| L_HEAP        | std::collections::BinaryHeap |
| DLLIST        | std::collections::LinkedList |
| L_STACK       | Vec<T> (push/pop)            |
| L_QUEUE       | std::collections::VecDeque   |
| L_PTRA        | Vec<Option<Box<T>>>          |
| L_DNA         | Vec<f64> (専用APIなし)       |

| C版機能              | Rust実装箇所                                |
| -------------------- | ------------------------------------------- |
| warper.c             | src/transform/warper.rs                     |
| encoding.c (Ascii85) | src/io/ps/ascii85.rs                        |
| pdfapp.c (基本)      | src/io/pdf.rs                               |
| utils1.c, utils2.c   | Rust標準ライブラリ (std::fs, std::string等) |
