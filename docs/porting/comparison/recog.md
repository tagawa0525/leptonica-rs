# leptonica (src/recog/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（Phase 1-13 全移植計画完了を反映）

## サマリー

この比較では、C版leptonicaのrecog関連ソースファイルの全public関数と、Rust版leptonica-recog crateの実装状況を対比します。

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 122 |
| 🔄 異なる | 26 |
| ❌ 未実装 | 3 |
| 🚫 不要 | 18 |
| 合計 | 169 |

> **注記**: Phase 1-13に加え、カバレッジ向上により148関数（🚫不要18関数を除く151関数中）が実装済み。
> 🚫不要18関数はデバッグ/可視化系・C固有getter等（Rustの設計で代替済み）。❌未実装3関数はclassapp.c由来。

## 詳細

### recogbasic.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| recogCreateFromRecog | ✅ 同等 | `recog::train::create_from_recog()` | 既存recogから新しいrecog生成（free fn） |
| recogCreateFromPixa | ✅ 同等 | `recog::train::create_from_pixa` | ラベル付きPixaから認識器を作成 |
| recogCreateFromPixaNoFinish | ✅ 同等 | `recog::train::create_from_pixa_no_finish()` | 訓練未完了のrecog作成（free fn） |
| recogCreate | ✅ 同等 | `recog::train::create` | 基本的なrecog作成 |
| recogDestroy | ✅ 同等 | `Drop` trait | Rustでは自動メモリ管理 |
| recogGetCount | ✅ 同等 | `Recog.get_class_labels().len()` | クラス数取得 |
| recogSetParams | 🔄 異なる | `Recog`フィールド直接設定 | パラメータは構造体フィールドとして保持 |
| recogGetClassIndex | ✅ 同等 | `Recog::get_class_index` | 文字値からインデックス取得 |
| recogStringToIndex | ✅ 同等 | `Recog::string_to_index` | 文字列からインデックス取得 |
| recogGetClassString | ✅ 同等 | `Recog::get_class_string` | インデックス→クラス名 |
| l_convertCharstrToInt | 🔄 異なる | `Recog::string_to_index` | UTF-8コードポイントとして統合 |
| recogRead | ✅ 同等 | `Recog::read` | ファイルからrecog読み込み |
| recogReadStream | ✅ 同等 | `Recog::read<R: Read>` | ストリームからrecog読み込み |
| recogReadMem | 🔄 異なる | `Recog::read` (Cursor利用) | メモリからrecog読み込み |
| recogWrite | ✅ 同等 | `Recog::write` | recogをファイルに書き込み |
| recogWriteStream | ✅ 同等 | `Recog::write<W: Write>` | recogをストリームに書き込み |
| recogWriteMem | 🔄 異なる | `Recog::write` (Vec利用) | recogをメモリに書き込み |
| recogExtractPixa | ✅ 同等 | `Recog::extract_pixa` | recogから全サンプルをPixaとして抽出 |

### recogdid.c (Document Image Decoding)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| recogDecode | ✅ 同等 | `Recog::decode` | HMMベースのデコーディング |
| recogCreateDid | ✅ 同等 | `Recog::create_did` | DID構造体の作成 |
| recogDestroyDid | ✅ 同等 | `Recog::destroy_did` | DID構造体の破棄 |
| recogDidExists | 🔄 異なる | `Recog`フィールドチェック | フラグではなく`Option`型で管理 |
| recogGetDid | 🚫 不要 | - | DID構造体へのポインタ取得（RustではOption型で管理） |
| recogSetChannelParams | 🚫 不要 | - | チャネルパラメータ設定（Rustでは構造体フィールド直接設定） |

### recogident.c (Identification)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| recogIdentifyMultiple | ✅ 同等 | `Recog::identify_multiple` | 複数文字の認識 |
| recogSplitIntoCharacters | ✅ 同等 | `Recog::split_into_characters` | 文字分割 |
| recogCorrelationBestRow | ✅ 同等 | `Recog::correlation_best_row` | 最良相関行の検索 |
| recogCorrelationBestChar | ✅ 同等 | `Recog::correlation_best_char` | 最良相関文字の検索 |
| recogIdentifyPixa | ✅ 同等 | `Recog::identify_pixa` | Pixa内の各画像を認識 |
| recogIdentifyPix | ✅ 同等 | `Recog::identify_pix` | 単一画像の認識 |
| recogSkipIdentify | ✅ 同等 | `Recog::skip_identify` | 認識をスキップ |
| recogProcessToIdentify | ✅ 同等 | `Recog::process_to_identify` | 認識前の画像処理 |
| recogExtractNumbers | ✅ 同等 | `Recog::extract_numbers` | 数字列の抽出 |
| showExtractNumbers | 🚫 不要 | - | 数字列抽出のデバッグ表示 |
| rchaDestroy | ✅ 同等 | `Drop` trait | Rcha構造体の自動破棄 |
| rchDestroy | ✅ 同等 | `Drop` trait | Rch構造体の自動破棄 |
| rchaExtract | ✅ 同等 | `rcha_extract` | Rcha配列からデータ抽出 |
| rchExtract | ✅ 同等 | `rch_extract` | Rch構造体からデータ抽出 |

### recogtrain.c (Training)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| recogTrainLabeled | ✅ 同等 | `Recog::train_labeled` | ラベル付きサンプルで訓練 |
| recogProcessLabeled | ✅ 同等 | `Recog::process_labeled` | ラベル付きサンプルの処理 |
| recogAddSample | ✅ 同等 | `Recog::add_sample` | サンプルの追加 |
| recogModifyTemplate | ✅ 同等 | `Recog::modify_template` | テンプレートの変換（スケール/線幅正規化） |
| recogAverageSamples | ✅ 同等 | `Recog::average_samples` | サンプルの平均化 |
| pixaAccumulateSamples | ✅ 同等 | `recog::train::pixa_accumulate_samples()` | サンプルの累積（free fn） |
| recogTrainingFinished | ✅ 同等 | `Recog::finish_training` | 訓練の完了処理 |
| recogFilterPixaBySize | ✅ 同等 | `Recog::filter_pixa_by_size` | サイズによるPixaフィルタリング |
| recogSortPixaByClass | ✅ 同等 | `Recog::sort_pixa_by_class` | クラスごとにPixaをソート |
| recogRemoveOutliers1 | ✅ 同等 | `Recog::remove_outliers1` | 外れ値除去（方法1） |
| pixaRemoveOutliers1 | ✅ 同等 | `recog::train::pixa_remove_outliers1()` | Pixaから外れ値除去（方法1、free fn） |
| recogRemoveOutliers2 | ✅ 同等 | `Recog::remove_outliers2` | 外れ値除去（方法2） |
| pixaRemoveOutliers2 | ✅ 同等 | `recog::train::pixa_remove_outliers2()` | Pixaから外れ値除去（方法2、free fn） |
| recogTrainFromBoot | ✅ 同等 | `recog::bootstrap::train_from_boot` | ブートストラップ認識器から訓練 |
| recogPadDigitTrainingSet | ✅ 同等 | `recog::bootstrap::pad_digit_training_set` | 数字訓練セットのパディング |
| recogIsPaddingNeeded | ✅ 同等 | `recog::bootstrap::is_padding_needed` | パディングが必要かチェック |
| recogAddDigitPadTemplates | 🔄 異なる | `pad_digit_training_set` 内部実装 | 数字パッドテンプレート追加 |
| recogMakeBootDigitRecog | ✅ 同等 | `recog::bootstrap::make_boot_digit_recog` | ブートストラップ数字認識器作成 |
| recogMakeBootDigitTemplates | 🔄 異なる | `make_boot_digit_recog` 内部実装 | ブートストラップ数字テンプレート作成 |
| recogShowContent | 🚫 不要 | - | recog内容の表示（デバッグ/可視化） |
| recogDebugAverages | 🚫 不要 | - | 平均テンプレートのデバッグ（デバッグ/可視化） |
| recogShowAverageTemplates | 🚫 不要 | - | 平均テンプレートの表示（デバッグ/可視化） |
| recogShowMatchesInRange | 🚫 不要 | - | スコア範囲内のマッチ表示（デバッグ/可視化） |
| recogShowMatch | 🚫 不要 | - | マッチの表示（デバッグ/可視化） |

### pageseg.c (Page Segmentation)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixGetRegionsBinary | ✅ 同等 | `pageseg::segment_regions` | 2値画像から領域抽出 |
| pixFindPageForeground | ✅ 同等 | `find_page_foreground` | ページ前景の検出 |
| pixSplitIntoCharacters | ✅ 同等 | `split_into_characters` | 文字への分割 |
| pixSplitComponentWithProfile | ✅ 同等 | `split_component_with_profile` | プロファイルを使った分割 |
| pixGetWordsInTextlines | ✅ 同等 | `get_words_in_textlines` | テキストライン内の単語取得 |
| pixGetWordBoxesInTextlines | ✅ 同等 | `get_word_boxes_in_textlines` | テキストライン内の単語ボックス取得 |

### skew.c (Skew Detection)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixFindSkewAndDeskew | ✅ 同等 | `skew::find_skew_and_deskew` | 傾き検出と補正 |
| pixFindSkew | ✅ 同等 | `skew::find_skew` | 傾き検出 |
| pixFindSkewSweep | ✅ 同等 | `find_skew_sweep` | スイープによる傾き検出 |
| pixFindSkewSweepAndSearch | 🔄 異なる | `skew::find_skew` (内部実装) | スイープ+探索（オプション指定で実現） |
| pixFindSkewSweepAndSearchScore | ✅ 同等 | `skew::find_skew_sweep_and_search_score` | スイープ+探索（スコア付き） |
| pixFindSkewSweepAndSearchScorePivot | ✅ 同等 | `skew::find_skew_sweep_and_search_score_pivot` | スイープ+探索（ピボット指定） |
| pixFindSkewOrthogonalRange | ✅ 同等 | `find_skew_orthogonal_range` | 直交範囲での傾き検出 |

### dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c (Dewarping)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| dewarpCreate | ✅ 同等 | `Dewarp::new` | Dewarp構造体作成 |
| dewarpCreateRef | ✅ 同等 | `Dewarp::create_ref` | 参照ページ指定のDewarp作成 |
| dewarpDestroy | ✅ 同等 | `Drop` trait | 自動破棄 |
| dewarpaCreate | ✅ 同等 | `Dewarpa::new` | Dewarpa（複数ページ）作成 |
| dewarpaCreateFromPixacomp | ✅ 同等 | `Dewarpa::create_from_pixacomp` | Pixacompから作成 |
| dewarpaDestroy | ✅ 同等 | `Drop` trait | 自動破棄 |
| dewarpaDestroyDewarp | 🔄 異なる | `Dewarpa::insert` (None挿入) | Dewarpa内の特定Dewarp破棄 |
| dewarpaInsertDewarp | ✅ 同等 | `Dewarpa::insert` | DewarpaへDewarp挿入 |
| dewarpaGetDewarp | ✅ 同等 | `Dewarpa::get` | Dewarpaから特定Dewarp取得 |
| dewarpaSetCurvatures | ✅ 同等 | `Dewarpa::set_curvatures` | 曲率パラメータ設定 |
| dewarpaUseBothArrays | ✅ 同等 | `Dewarpa::use_both_arrays` | 両配列の使用設定 |
| dewarpaSetCheckColumns | ✅ 同等 | `Dewarpa::use_single_model` | カラムチェック設定 |
| dewarpaSetMaxDistance | 🔄 異なる | `Dewarpa` フィールド直接設定 | 最大距離設定 |
| dewarpRead | ✅ 同等 | `Dewarp::read` | Dewarp読み込み |
| dewarpReadStream | ✅ 同等 | `Dewarp::read<R: Read>` | Dewarpストリーム読み込み |
| dewarpReadMem | 🔄 異なる | `Dewarp::read` (Cursor利用) | Dewarpメモリ読み込み |
| dewarpWrite | ✅ 同等 | `Dewarp::write` | Dewarp書き込み |
| dewarpWriteStream | ✅ 同等 | `Dewarp::write<W: Write>` | Dewarpストリーム書き込み |
| dewarpWriteMem | 🔄 異なる | `Dewarp::write` (Vec利用) | Dewarpメモリ書き込み |
| dewarpaRead | ✅ 同等 | `Dewarpa::read` | Dewarpa読み込み |
| dewarpaReadStream | ✅ 同等 | `Dewarpa::read<R: Read>` | Dewarpaストリーム読み込み |
| dewarpaReadMem | 🔄 異なる | `Dewarpa::read` (Cursor利用) | Dewarpaメモリ読み込み |
| dewarpaWrite | ✅ 同等 | `Dewarpa::write` | Dewarpa書き込み |
| dewarpaWriteStream | ✅ 同等 | `Dewarpa::write<W: Write>` | Dewarpaストリーム書き込み |
| dewarpaWriteMem | 🔄 異なる | `Dewarpa::write` (Vec利用) | Dewarpaメモリ書き込み |
| dewarpBuildPageModel | ✅ 同等 | `dewarp::model::build_page_model` | モデル構築 |
| dewarpFindVertDisparity | ✅ 同等 | `dewarp::model::build_vertical_disparity` | 垂直歪み検出 |
| dewarpFindHorizDisparity | ✅ 同等 | `dewarp::model::build_horizontal_disparity` | 水平歪み検出 |
| dewarpGetTextlineCenters | ✅ 同等 | `dewarp::textline::find_textline_centers` | テキストライン中心検出 |
| dewarpRemoveShortLines | ✅ 同等 | `dewarp::textline::remove_short_lines` | 短い線の除去 |
| dewarpFindHorizSlopeDisparity | ✅ 同等 | `dewarp::model::find_horiz_disparity` | 水平傾斜歪み検出 |
| dewarpBuildLineModel | 🔄 異なる | `dewarp::model::build_page_model` 内部 | ラインモデル構築 |
| dewarpaModelStatus | 🔄 異なる | `Dewarp` フィールド直接参照 | モデルステータス取得 |
| dewarpaApplyDisparity | ✅ 同等 | `dewarp::apply::apply_disparity` | 歪み補正適用 |
| dewarpaApplyDisparityBoxa | ✅ 同等 | `Dewarpa::apply_disparity_boxa` | Boxaへの歪み補正適用 |
| dewarpMinimize | ✅ 同等 | `Dewarp::minimize` | Dewarpの最小化 |
| dewarpPopulateFullRes | ✅ 同等 | `dewarp::model::populate_full_resolution` | フル解像度への展開 |
| dewarpSinglePage | ✅ 同等 | `dewarp::dewarp_single_page` | 単一ページの歪み補正 |
| dewarpSinglePageInit | ✅ 同等 | `dewarp::single_page::dewarp_single_page_init` | 単一ページ歪み補正の初期化 |
| dewarpSinglePageRun | ✅ 同等 | `dewarp::single_page::dewarp_single_page_run` | 単一ページ歪み補正の実行 |
| dewarpaListPages | 🚫 不要 | - | ページリスト表示（デバッグ/可視化） |
| dewarpaSetValidModels | 🔄 異なる | `Dewarpa::insert_ref_models` 等 | 有効モデル設定 |
| dewarpaInsertRefModels | ✅ 同等 | `Dewarpa::insert_ref_models` | 参照モデル挿入 |
| dewarpaStripRefModels | ✅ 同等 | `Dewarpa::strip_ref_models` | 参照モデル削除 |
| dewarpaRestoreModels | ✅ 同等 | `Dewarpa::restore_models` | モデル復元 |
| dewarpaInfo | 🚫 不要 | - | Dewarpa情報表示（デバッグ/可視化） |
| dewarpaModelStats | 🚫 不要 | - | モデル統計取得（デバッグ/可視化） |
| dewarpaShowArrays | 🚫 不要 | - | 配列の表示（デバッグ/可視化） |
| dewarpDebug | 🚫 不要 | - | デバッグ出力（デバッグ/可視化） |
| dewarpShowResults | 🚫 不要 | - | 結果表示（デバッグ/可視化） |

### baseline.c (Baseline Detection)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixFindBaselines | ✅ 同等 | `baseline::find_baselines` | ベースライン検出 |
| pixFindBaselinesGen | 🔄 異なる | `baseline::find_baselines` (オプション指定) | 汎用ベースライン検出 |
| pixGetLocalSkewAngles | ✅ 同等 | `baseline::get_local_skew_angles` | ローカル傾き角配列 |
| pixGetLocalSkewTransform | ✅ 同等 | `baseline::get_local_skew_transform` | 局所スキュー変換制御点 |
| pixDeskewLocal | ✅ 同等 | `baseline::deskew_local` | 局所スキュー補正 |

### jbclass.c (JBIG2 Classification)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| jbRankHausInit | ✅ 同等 | `jbclass::rank_haus_init` | Rank Hausdorff分類器初期化 |
| jbCorrelationInit | ✅ 同等 | `jbclass::correlation_init` | 相関ベース分類器初期化 |
| jbCorrelationInitWithoutComponents | ✅ 同等 | `jbclass::correlation_init_without_components` | コンポーネントなし相関分類器初期化 |
| jbAddPages | ✅ 同等 | `JbClasser::add_pages` | 複数ページ追加 |
| jbAddPage | ✅ 同等 | `JbClasser::add_page` | ページ追加 |
| jbAddPageComponents | ✅ 同等 | `jbclass::add_page_components` |  |
| jbClassifyRankHaus | 🔄 異なる | `JbClasser` (内部実装) | Rank Hausdorff分類（内部で自動実行） |
| jbClassifyCorrelation | 🔄 異なる | `JbClasser` (内部実装) | 相関ベース分類（内部で自動実行） |
| jbClasserCreate | 🔄 異なる | `rank_haus_init` / `correlation_init` | 分類器作成（専用関数に分割） |
| jbClasserDestroy | ✅ 同等 | `Drop` trait | 自動破棄 |
| jbDataSave | ✅ 同等 | `JbClasser::get_data` | データ取得 |
| jbDataRead | ✅ 同等 | `JbData::read` | データ読み込み（I/O追加） |
| jbDataWrite | ✅ 同等 | `JbData::write` | データ書き込み（I/O追加） |
| jbGetULCorners | 🔄 異なる | `JbData` フィールド直接参照 | 左上コーナー取得 |
| jbGetLLCorners | 🔄 異なる | `JbData` フィールド直接参照 | 左下コーナー取得 |
| jbCorrelation | ✅ 同等 | `jbclass::classify::jb_correlation` | 相関ベース高レベルAPI |
| jbRankHaus | ✅ 同等 | `jbclass::classify::jb_rank_haus` | Rank Hausdorff高レベルAPI |
| jbWordsInTextlines | ✅ 同等 | `jbclass::classify::pix_word_mask_by_dilation` | テキストライン内の単語分類 |

### classapp.c (JBIG2分類応用)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixFindWordAndCharacterBoxes | ❌ 未実装 | - | 単語および文字ボックスの検出 |
| boxaExtractSortedPattern | ❌ 未実装 | - | パターンに基づくBoxa抽出 |
| numaaCompareImagesByBoxes | ❌ 未実装 | - | ボックスベースの画像比較 |

### bootnumgen1.c, bootnumgen2.c, bootnumgen3.c, bootnumgen4.c (Bootstrap数字生成データ)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| l_bootnum_gen1 | 🚫 不要 | - | ブートストラップ数字セット1（組み込みデータジェネレータ） |
| l_bootnum_gen2 | 🚫 不要 | - | ブートストラップ数字セット2（組み込みデータジェネレータ） |
| l_bootnum_gen3 | 🚫 不要 | - | ブートストラップ数字セット3（組み込みデータジェネレータ） |
| l_bootnum_gen4 | 🚫 不要 | - | ブートストラップ数字セット4（組み込みデータジェネレータ） |

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
| pixReadBarcodes | ✅ 同等 | `read_barcodes` | Pixaからバーコード読み取り |
| pixReadBarcodeWidths | 🔄 異なる | `barcode::signal::extract_barcode_widths` | バーコード幅読み取り（Direction対応） |
| pixLocateBarcodes | ✅ 同等 | `barcode::detect::locate_barcodes` | バーコード位置検出 |
| pixLocateBarcodesMoreAccurate | ✅ 同等 | `barcode::detect::locate_barcodes_morphological` | 形態学的バーコード位置検出 |
| pixDeskewBarcode | ✅ 同等 | `barcode::detect::deskew_barcode` | バーコード傾き補正 |
| pixGenerateBarcodeMask | ✅ 同等 | `barcode::detect::barcode_gen_mask` | バーコードマスク生成 |
| pixExtractBarcodeWidths1 | 🔄 異なる | `barcode::signal::extract_barcode_widths` | バーコード幅抽出（統合API） |
| pixExtractBarcodeWidths2 | 🔄 異なる | `barcode::signal::extract_barcode_widths` | バーコード幅抽出（統合API） |
| pixExtractBarcodeCrossings | ✅ 同等 | `barcode::signal::extract_crossings` | バーコード交差点抽出 |
| numaFindLocForThreshold | ✅ 同等 | `barcode::signal::find_barcode_peaks` | ピーク位置検出 |

## 実装状況の分析（Phase 1-13 完了後 2026-02-22）

### 実装済み領域
1. **Recog基本機能**: create, train_labeled, finish_training等
2. **Recog I/O**: read/write（バイナリ形式、ファイル・ストリーム）
3. **Recog query**: get_count, get_class_index, string_to_index等
4. **Bootstrap数字認識器**: make_boot_digit_recog, train_from_boot等
5. **DID (Document Image Decoding)**: HMMベースのデコーディング
6. **識別機能**: identify_pix, identify_multiple, filter_pixa_by_size等
7. **傾き検出拡張**: find_skew_sweep_and_search_score, find_skew_orthogonal_range等
8. **歪み補正（Dewarp）**: 単一ページ・複数ページ（Dewarpa）含む全I/O
9. **ベースライン検出拡張**: get_local_skew_angles, deskew_local等
10. **JBIG2分類**: rank_haus_init, correlation_init, I/O
11. **バーコード検出・デコード**: 形態学的マスク生成、幅抽出、ピーク検出含む

### 🚫 不要（Rustの設計で代替済み）
1. **デバッグ/可視化（12関数）**: recogShowContent, recogDebugAverages, recogShowAverageTemplates, recogShowMatchesInRange, recogShowMatch, showExtractNumbers, dewarpaListPages, dewarpaInfo, dewarpaModelStats, dewarpaShowArrays, dewarpDebug, dewarpShowResults — Debug trait・外部ツールで代替
2. **C固有getter（1関数）**: recogGetDid — RustではOption型で直接管理
3. **C固有setter（1関数）**: recogSetChannelParams — Rustでは構造体フィールド直接設定

### 実装完了領域（元未実装 → 全て実装済み）
1. **ページセグメンテーション詳細**: pixFindPageForeground, pixSplitIntoCharacters等 — 実装済み
2. **JBIG2高レベルAPI**: jbCorrelation, jbRankHaus等 — 実装済み
3. **Recog訓練ユーティリティ**: recogProcessLabeled, recogAddSample等 — 実装済み
4. **Recog作成・識別**: recogCreateFromRecog, recogExtractNumbers等 — 実装済み
5. **Skew検出拡張**: pixFindSkewSweep, pixFindSkewOrthogonalRange — 実装済み
6. **Dewarpa管理**: dewarpaCreateFromPixacomp, dewarpaRestoreModels — 実装済み
7. **バーコード**: pixReadBarcodes — 実装済み

### 設計の違い
1. **メモリ管理**: C版のcreate/destroy → Rust版のDrop trait
2. **パラメータ設定**: C版のset関数 → Rust版の構造体フィールド直接設定
3. **エラーハンドリング**: C版の戻り値 → Rust版のResult型
4. **NULL/Option**: C版のNULLポインタ → Rust版のOption型

## 備考

- C版の関数総数: 169関数（recog関連全体、この表の範囲）
- Rust版実装済み: 148関数（✅122 + 🔄26）
- 🚫不要: 18関数（デバッグ/可視化系・C固有getter/setter）
- ❌未実装: 3関数（classapp.c由来）
- 実装率: 87.6%（全体）、98.0%（🚫不要除外ベース）

C版の全機能を網羅することは目標ではなく、Rustの慣用的な設計で同等の機能を提供することを重視しています。特に以下の点で設計が異なります：

1. メモリ管理はRustの所有権システムで自動化
2. エラー処理はResult型で型安全に
3. デバッグ機能は標準のDebug traitや外部ツールで代替
4. 複数ページ管理はDewarpa構造体（Vec<Option<Dewarp>>）で実現
