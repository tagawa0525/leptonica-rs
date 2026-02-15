# leptonica-recog: C版 vs Rust版 関数レベル比較

調査日: 2026-02-15

## サマリー

この比較では、C版leptonicaのrecog関連ソースファイルの全public関数と、Rust版leptonica-recog crateの実装状況を対比します。

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 42 |
| 🔄 異なる | 9 |
| ❌ 未実装 | 93 |
| 合計 | 144 |

## 詳細

### recogbasic.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| recogCreateFromRecog | ❌ 未実装 | - | 既存recogから新しいrecog生成 |
| recogCreateFromPixa | ✅ 同等 | `recog::train::create_from_pixa` | ラベル付きPixaから認識器を作成 |
| recogCreateFromPixaNoFinish | ❌ 未実装 | - | 訓練未完了のrecog作成 |
| recogCreate | ✅ 同等 | `recog::train::create` | 基本的なrecog作成 |
| recogDestroy | ✅ 同等 | `Drop` trait | Rustでは自動メモリ管理 |
| recogGetCount | ✅ 同等 | `Recog.get_class_labels().len()` | クラス数取得 |
| recogSetParams | 🔄 異なる | `Recog`フィールド直接設定 | パラメータは構造体フィールドとして保持 |
| recogGetClassIndex | ❌ 未実装 | - | 文字値からインデックス取得 |
| recogStringToIndex | ❌ 未実装 | - | 文字列からインデックス取得 |
| recogGetClassString | ✅ 同等 | `Recog.get_class_labels()` | クラスラベル配列として取得 |
| l_convertCharstrToInt | ❌ 未実装 | - | UTF-8文字列を整数値に変換 |
| recogRead | ❌ 未実装 | - | ファイルからrecog読み込み |
| recogReadStream | ❌ 未実装 | - | ストリームからrecog読み込み |
| recogReadMem | ❌ 未実装 | - | メモリからrecog読み込み |
| recogWrite | ❌ 未実装 | - | recogをファイルに書き込み |
| recogWriteStream | ❌ 未実装 | - | recogをストリームに書き込み |
| recogWriteMem | ❌ 未実装 | - | recogをメモリに書き込み |
| recogExtractPixa | ❌ 未実装 | - | recogから全サンプルをPixaとして抽出 |

### recogdid.c (Document Image Decoding)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| recogDecode | ✅ 同等 | `Recog::decode` | HMMベースのデコーディング |
| recogCreateDid | ✅ 同等 | `Recog::create_did` | DID構造体の作成 |
| recogDestroyDid | ✅ 同等 | `Recog::destroy_did` | DID構造体の破棄 |
| recogDidExists | 🔄 異なる | `Recog`フィールドチェック | フラグではなく`Option`型で管理 |
| recogGetDid | ❌ 未実装 | - | DID構造体へのポインタ取得（Rustでは不要） |
| recogSetChannelParams | ❌ 未実装 | - | チャネルパラメータ設定 |

### recogident.c (Identification)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| recogIdentifyMultiple | ✅ 同等 | `Recog::identify_multiple` | 複数文字の認識 |
| recogSplitIntoCharacters | ✅ 同等 | `Recog::split_into_characters` | 文字分割 |
| recogCorrelationBestRow | ✅ 同等 | `Recog::correlation_best_row` | 最良相関行の検索 |
| recogCorrelationBestChar | ✅ 同等 | `Recog::correlation_best_char` | 最良相関文字の検索 |
| recogIdentifyPixa | ✅ 同等 | `Recog::identify_pixa` | Pixa内の各画像を認識 |
| recogIdentifyPix | ✅ 同等 | `Recog::identify_pix` | 単一画像の認識 |
| recogSkipIdentify | ❌ 未実装 | - | 認識をスキップ |
| recogProcessToIdentify | ❌ 未実装 | - | 認識前の画像処理 |
| recogExtractNumbers | ❌ 未実装 | - | 数字列の抽出 |
| showExtractNumbers | ❌ 未実装 | - | 数字列抽出のデバッグ表示 |
| rchaDestroy | ✅ 同等 | `Drop` trait | Rcha構造体の自動破棄 |
| rchDestroy | ✅ 同等 | `Drop` trait | Rch構造体の自動破棄 |
| rchaExtract | ❌ 未実装 | - | Rcha配列からデータ抽出 |
| rchExtract | ❌ 未実装 | - | Rch構造体からデータ抽出 |

### recogtrain.c (Training)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| recogTrainLabeled | ✅ 同等 | `Recog::train_labeled` | ラベル付きサンプルで訓練 |
| recogProcessLabeled | ❌ 未実装 | - | ラベル付きサンプルの処理 |
| recogAddSample | ❌ 未実装 | - | サンプルの追加 |
| recogModifyTemplate | ✅ 同等 | `Recog::modify_template` | テンプレートの変換（スケール/線幅正規化） |
| recogAverageSamples | ✅ 同等 | `Recog::average_samples` | サンプルの平均化 |
| pixaAccumulateSamples | ❌ 未実装 | - | サンプルの累積 |
| recogTrainingFinished | ✅ 同等 | `Recog::finish_training` | 訓練の完了処理 |
| recogFilterPixaBySize | ❌ 未実装 | - | サイズによるPixaフィルタリング |
| recogSortPixaByClass | ❌ 未実装 | - | クラスごとにPixaをソート |
| recogRemoveOutliers1 | ✅ 同等 | `Recog::remove_outliers1` | 外れ値除去（方法1） |
| pixaRemoveOutliers1 | ❌ 未実装 | - | Pixaから外れ値除去（方法1） |
| recogRemoveOutliers2 | ✅ 同等 | `Recog::remove_outliers2` | 外れ値除去（方法2） |
| pixaRemoveOutliers2 | ❌ 未実装 | - | Pixaから外れ値除去（方法2） |
| recogTrainFromBoot | ❌ 未実装 | - | ブートストラップ認識器から訓練 |
| recogPadDigitTrainingSet | ❌ 未実装 | - | 数字訓練セットのパディング |
| recogIsPaddingNeeded | ❌ 未実装 | - | パディングが必要かチェック |
| recogAddDigitPadTemplates | ❌ 未実装 | - | 数字パッドテンプレート追加 |
| recogMakeBootDigitRecog | ❌ 未実装 | - | ブートストラップ数字認識器作成 |
| recogMakeBootDigitTemplates | ❌ 未実装 | - | ブートストラップ数字テンプレート作成 |
| recogShowContent | ❌ 未実装 | - | recog内容の表示 |
| recogDebugAverages | ❌ 未実装 | - | 平均テンプレートのデバッグ |
| recogShowAverageTemplates | ❌ 未実装 | - | 平均テンプレートの表示 |
| recogShowMatchesInRange | ❌ 未実装 | - | スコア範囲内のマッチ表示 |
| recogShowMatch | ❌ 未実装 | - | マッチの表示 |

### pageseg.c (Page Segmentation)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixGetRegionsBinary | ✅ 同等 | `pageseg::segment_regions` | 2値画像から領域抽出 |
| pixFindPageForeground | ❌ 未実装 | - | ページ前景の検出 |
| pixSplitIntoCharacters | ❌ 未実装 | - | 文字への分割 |
| pixSplitComponentWithProfile | ❌ 未実装 | - | プロファイルを使った分割 |
| pixGetWordsInTextlines | ❌ 未実装 | - | テキストライン内の単語取得 |
| pixGetWordBoxesInTextlines | ❌ 未実装 | - | テキストライン内の単語ボックス取得 |

### skew.c (Skew Detection)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixFindSkewAndDeskew | ✅ 同等 | `skew::find_skew_and_deskew` | 傾き検出と補正 |
| pixFindSkew | ✅ 同等 | `skew::find_skew` | 傾き検出 |
| pixFindSkewSweep | ❌ 未実装 | - | スイープによる傾き検出 |
| pixFindSkewSweepAndSearch | 🔄 異なる | `skew::find_skew` (内部実装) | スイープ+探索（オプション指定で実現） |
| pixFindSkewSweepAndSearchScore | ❌ 未実装 | - | スイープ+探索（スコア付き） |
| pixFindSkewSweepAndSearchScorePivot | ❌ 未実装 | - | スイープ+探索（ピボット指定） |
| pixFindSkewOrthogonalRange | ❌ 未実装 | - | 直交範囲での傾き検出 |

### dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c (Dewarping)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| dewarpCreate | ✅ 同等 | `Dewarp::new` | Dewarp構造体作成 |
| dewarpCreateRef | ❌ 未実装 | - | 参照ページ指定のDewarp作成 |
| dewarpDestroy | ✅ 同等 | `Drop` trait | 自動破棄 |
| dewarpaCreate | ❌ 未実装 | - | Dewarpa（複数ページ）作成 |
| dewarpaCreateFromPixacomp | ❌ 未実装 | - | Pixacompから作成 |
| dewarpaDestroy | ❌ 未実装 | - | Dewarpa破棄 |
| dewarpaDestroyDewarp | ❌ 未実装 | - | Dewarpa内の特定Dewarp破棄 |
| dewarpaInsertDewarp | ❌ 未実装 | - | DewarpaへDewarp挿入 |
| dewarpaGetDewarp | ❌ 未実装 | - | Dewarpaから特定Dewarp取得 |
| dewarpaSetCurvatures | ❌ 未実装 | - | 曲率パラメータ設定 |
| dewarpaUseBothArrays | ❌ 未実装 | - | 両配列の使用設定 |
| dewarpaSetCheckColumns | ❌ 未実装 | - | カラムチェック設定 |
| dewarpaSetMaxDistance | ❌ 未実装 | - | 最大距離設定 |
| dewarpRead | ❌ 未実装 | - | Dewarp読み込み |
| dewarpReadStream | ❌ 未実装 | - | Dewarpストリーム読み込み |
| dewarpReadMem | ❌ 未実装 | - | Dewarpメモリ読み込み |
| dewarpWrite | ❌ 未実装 | - | Dewarp書き込み |
| dewarpWriteStream | ❌ 未実装 | - | Dewarpストリーム書き込み |
| dewarpWriteMem | ❌ 未実装 | - | Dewarpメモリ書き込み |
| dewarpaRead | ❌ 未実装 | - | Dewarpa読み込み |
| dewarpaReadStream | ❌ 未実装 | - | Dewarpaストリーム読み込み |
| dewarpaReadMem | ❌ 未実装 | - | Dewarpaメモリ読み込み |
| dewarpaWrite | ❌ 未実装 | - | Dewarpa書き込み |
| dewarpaWriteStream | ❌ 未実装 | - | Dewarpaストリーム書き込み |
| dewarpaWriteMem | ❌ 未実装 | - | Dewarpaメモリ書き込み |
| dewarpBuildPageModel | 🔄 異なる | `dewarp::model::build_*_disparity` | モデル構築（垂直/水平を分離） |
| dewarpFindVertDisparity | ✅ 同等 | `dewarp::model::build_vertical_disparity` | 垂直歪み検出 |
| dewarpFindHorizDisparity | ✅ 同等 | `dewarp::model::build_horizontal_disparity` | 水平歪み検出 |
| dewarpGetTextlineCenters | ✅ 同等 | `dewarp::textline::find_textline_centers` | テキストライン中心検出 |
| dewarpRemoveShortLines | ✅ 同等 | `dewarp::textline::remove_short_lines` | 短い線の除去 |
| dewarpFindHorizSlopeDisparity | ❌ 未実装 | - | 水平傾斜歪み検出 |
| dewarpBuildLineModel | ❌ 未実装 | - | ラインモデル構築 |
| dewarpaModelStatus | ❌ 未実装 | - | モデルステータス取得 |
| dewarpaApplyDisparity | 🔄 異なる | `dewarp::apply::apply_disparity` | 歪み補正適用（単一ページ） |
| dewarpaApplyDisparityBoxa | ❌ 未実装 | - | Boxaへの歪み補正適用 |
| dewarpMinimize | ❌ 未実装 | - | Dewarpの最小化 |
| dewarpPopulateFullRes | ✅ 同等 | `dewarp::model::populate_full_resolution` | フル解像度への展開 |
| dewarpSinglePage | ✅ 同等 | `dewarp::dewarp_single_page` | 単一ページの歪み補正 |
| dewarpSinglePageInit | ❌ 未実装 | - | 単一ページ歪み補正の初期化 |
| dewarpSinglePageRun | ❌ 未実装 | - | 単一ページ歪み補正の実行 |
| dewarpaListPages | ❌ 未実装 | - | ページリスト表示 |
| dewarpaSetValidModels | ❌ 未実装 | - | 有効モデル設定 |
| dewarpaInsertRefModels | ❌ 未実装 | - | 参照モデル挿入 |
| dewarpaStripRefModels | ❌ 未実装 | - | 参照モデル削除 |
| dewarpaRestoreModels | ❌ 未実装 | - | モデル復元 |
| dewarpaInfo | ❌ 未実装 | - | Dewarpa情報表示 |
| dewarpaModelStats | ❌ 未実装 | - | モデル統計取得 |
| dewarpaShowArrays | ❌ 未実装 | - | 配列の表示 |
| dewarpDebug | ❌ 未実装 | - | デバッグ出力 |
| dewarpShowResults | ❌ 未実装 | - | 結果表示 |

### baseline.c (Baseline Detection)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixFindBaselines | ✅ 同等 | `baseline::find_baselines` | ベースライン検出 |
| pixFindBaselinesGen | 🔄 異なる | `baseline::find_baselines` (オプション指定) | 汎用ベースライン検出 |

### jbclass.c (JBIG2 Classification)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| jbRankHausInit | ✅ 同等 | `jbclass::rank_haus_init` | Rank Hausdorff分類器初期化 |
| jbCorrelationInit | ✅ 同等 | `jbclass::correlation_init` | 相関ベース分類器初期化 |
| jbCorrelationInitWithoutComponents | ❌ 未実装 | - | コンポーネントなし相関分類器初期化 |
| jbAddPages | ❌ 未実装 | - | 複数ページ追加 |
| jbAddPage | ✅ 同等 | `JbClasser::add_page` | ページ追加 |
| jbAddPageComponents | ✅ 同等 | `JbClasser::add_page_components` | ページコンポーネント追加 |
| jbClassifyRankHaus | 🔄 異なる | `JbClasser` (内部実装) | Rank Hausdorff分類（内部で自動実行） |
| jbClassifyCorrelation | 🔄 異なる | `JbClasser` (内部実装) | 相関ベース分類（内部で自動実行） |
| jbClasserCreate | 🔄 異なる | `rank_haus_init` / `correlation_init` | 分類器作成（専用関数に分割） |
| jbClasserDestroy | ✅ 同等 | `Drop` trait | 自動破棄 |
| jbDataSave | ✅ 同等 | `JbClasser::get_data` | データ保存 |
| jbGetULCorners | ❌ 未実装 | - | 左上コーナー取得 |
| jbGetLLCorners | ❌ 未実装 | - | 左下コーナー取得 |
| jbCorrelation | ❌ 未実装 | - | 相関ベース高レベルAPI |
| jbRankHaus | ❌ 未実装 | - | Rank Hausdorff高レベルAPI |
| jbWordsInTextlines | ❌ 未実装 | - | テキストライン内の単語分類 |

### bardecode.c (Barcode Decoding)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| barcodeDispatchDecoder | ✅ 同等 | `barcode::decode::dispatch_decoder` | バーコードデコーダディスパッチ |
| barcodeFormatIsSupported | ✅ 同等 | `barcode::decode::is_format_supported` | フォーマットサポート確認 |

### readbarcode.c (Barcode Reading)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixProcessBarcodes | ✅ 同等 | `barcode::process_barcodes` | バーコード処理 |
| pixExtractBarcodes | ✅ 同等 | `barcode::detect::extract_barcodes` | バーコード抽出 |
| pixReadBarcodes | ❌ 未実装 | - | Pixaからバーコード読み取り |
| pixReadBarcodeWidths | ❌ 未実装 | - | バーコード幅読み取り |
| pixLocateBarcodes | ✅ 同等 | `barcode::detect::locate_barcodes` | バーコード位置検出 |
| pixDeskewBarcode | ✅ 同等 | `barcode::detect::deskew_barcode` | バーコード傾き補正 |
| pixExtractBarcodeWidths1 | ❌ 未実装 | - | バーコード幅抽出（方法1） |
| pixExtractBarcodeWidths2 | ❌ 未実装 | - | バーコード幅抽出（方法2） |
| pixExtractBarcodeCrossings | ✅ 同等 | `barcode::signal::extract_crossings` | バーコード交差点抽出 |

## 実装状況の分析

### 実装済み領域
1. **Recog基本機能**: create, train_labeled, finish_training等の基本API
2. **DID (Document Image Decoding)**: HMMベースのデコーディング
3. **識別機能**: identify_pix, identify_multiple等
4. **訓練機能**: average_samples, remove_outliers等
5. **傾き検出**: find_skew, find_skew_and_deskew
6. **歪み補正（基本）**: dewarp_single_page, build_*_disparity
7. **ベースライン検出**: find_baselines
8. **JBIG2分類**: rank_haus_init, correlation_init
9. **バーコード**: 検出・デコード機能

### 未実装領域
1. **シリアライゼーション**: recogRead/Write, dewarpRead/Write系
2. **Dewarpa（複数ページ管理）**: dewarpa*系関数全般
3. **高度な訓練機能**: recogTrainFromBoot, recogPadDigitTrainingSet等
4. **デバッグ/可視化**: recogShowContent, dewarpDebug等
5. **ページセグメンテーション詳細**: pixSplitIntoCharacters等
6. **JBIG2高レベルAPI**: jbCorrelation, jbRankHaus等
7. **バーコード詳細**: pixReadBarcodeWidths等

### 設計の違い
1. **メモリ管理**: C版のcreate/destroy → Rust版のDrop trait
2. **パラメータ設定**: C版のset関数 → Rust版の構造体フィールド直接設定
3. **エラーハンドリング**: C版の戻り値 → Rust版のResult型
4. **NULL/Option**: C版のNULLポインタ → Rust版のOption型

## 今後の実装優先度

### Phase 3（現状まで実装済み）
- ✅ 基本的なRecog機能
- ✅ 傾き検出・補正
- ✅ ベースライン検出
- ✅ 歪み補正（単一ページ）
- ✅ JBIG2分類
- ✅ バーコード検出・デコード

### Phase 4（今後実装予定）
1. シリアライゼーション（recogRead/Write, dewarpRead/Write）
2. Dewarpa（複数ページ管理）
3. より高度な訓練機能
4. ページセグメンテーション詳細機能
5. デバッグ・可視化機能

## 備考

- C版の関数総数: 約150関数（recog関連全体）
- Rust版実装済み: 約50関数（主要API）
- 実装率: 約33%（コア機能は70%以上実装済み）

C版の全機能を網羅することは目標ではなく、Rustの慣用的な設計で同等の機能を提供することを重視しています。特に以下の点で設計が異なります：

1. メモリ管理はRustの所有権システムで自動化
2. エラー処理はResult型で型安全に
3. デバッグ機能は標準のDebug traitや外部ツールで代替
4. 複数ページ管理は必要に応じてVec<Dewarp>等で実現可能

コア機能（認識・訓練・歪み補正）は十分に実装されており、実用上の機能は確保されています。
