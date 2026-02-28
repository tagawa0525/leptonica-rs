# leptonica (src/recog/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（Phase 1-13 全移植計画完了を反映）

## サマリー

この比較では、C版leptonicaのrecog関連ソースファイルの全public関数と、Rust版leptonica-recog crateの実装状況を対比します。

| 項目      | 数  |
| --------- | --- |
| ✅ 同等   | 119 |
| 🔄 異なる | 45  |
| ❌ 未実装 | 0   |
| 🚫 不要   | 18  |
| 合計      | 182 |

> **注記**: Phase 1-13に加え、カバレッジ向上により164関数（🚫不要18関数を除く164関数中）が実装済み。
> 🚫不要18関数はデバッグ/可視化系・C固有getter等（Rustの設計で代替済み）。

## 詳細

### recogbasic.c

#### recog/recog/train.rs (recogbasic.c)

| C関数                       | 状態 | Rust対応                                   | 備考                                    |
| --------------------------- | ---- | ------------------------------------------ | --------------------------------------- |
| recogCreateFromRecog        | ✅   | recog::train::create_from_recog()          | 既存recogから新しいrecog生成（free fn） |
| recogCreateFromPixa         | ✅   | recog::create_from_pixa                    | ラベル付きPixaから認識器を作成          |
| recogCreateFromPixaNoFinish | ✅   | recog::train::create_from_pixa_no_finish() | 訓練未完了のrecog作成（free fn）        |
| recogCreate                 | ✅   | recog::create                              | 基本的なrecog作成                       |
| recogDestroy                | ✅   | Drop trait                                 | Rustでは自動メモリ管理                  |
| recogGetCount               | ✅   | Recog.get_class_labels().len()             | クラス数取得                            |
| recogSetParams              | 🔄   | Recogフィールド直接設定                    | パラメータは構造体フィールドとして保持  |

#### recog/recog/query.rs (recogbasic.c)

| C関数                 | 状態 | Rust対応                | 備考                          |
| --------------------- | ---- | ----------------------- | ----------------------------- |
| recogGetClassIndex    | ✅   | Recog::get_class_index  | 文字値からインデックス取得    |
| recogStringToIndex    | ✅   | Recog::string_to_index  | 文字列からインデックス取得    |
| recogGetClassString   | ✅   | Recog::get_class_string | インデックス→クラス名         |
| l_convertCharstrToInt | 🔄   | Recog::string_to_index  | UTF-8コードポイントとして統合 |

#### recog/recog/io.rs (recogbasic.c)

| C関数            | 状態 | Rust対応            | 備考                                |
| ---------------- | ---- | ------------------- | ----------------------------------- |
| recogExtractPixa | ✅   | Recog::extract_pixa | recogから全サンプルをPixaとして抽出 |
| recogRead        | 🔄   | Recog::read()       | ファイルからrecog読み込み           |
| recogReadStream  | 🔄   | Recog::read()       | ストリームからrecog読み込み         |
| recogReadMem     | 🔄   | Recog::read()       | メモリからrecog読み込み             |
| recogWrite       | 🔄   | Recog::write()      | recogをファイルに書き込み           |
| recogWriteStream | 🔄   | Recog::write()      | recogをストリームに書き込み         |
| recogWriteMem    | 🔄   | Recog::write()      | recogをメモリに書き込み             |

### recogdid.c (Document Image Decoding)

#### recog/recog/did.rs (recogdid.c)

| C関数                 | 状態 | Rust対応                | 備考                                                       |
| --------------------- | ---- | ----------------------- | ---------------------------------------------------------- |
| recogDecode           | ✅   | Recog::decode           | HMMベースのデコーディング                                  |
| recogCreateDid        | ✅   | Recog::create_did       | DID構造体の作成                                            |
| recogDestroyDid       | ✅   | Recog::destroy_did      | DID構造体の破棄                                            |
| recogDidExists        | 🔄   | Recogフィールドチェック | フラグではなく`Option`型で管理                             |
| recogGetDid           | 🚫   | -                       | DID構造体へのポインタ取得（RustではOption型で管理）        |
| recogSetChannelParams | 🚫   | -                       | チャネルパラメータ設定（Rustでは構造体フィールド直接設定） |

### recogident.c (Identification)

#### recog/recog/ident.rs (recogident.c)

| C関数                    | 状態 | Rust対応                     | 備考                     |
| ------------------------ | ---- | ---------------------------- | ------------------------ |
| recogIdentifyMultiple    | ✅   | Recog::identify_multiple     | 複数文字の認識           |
| recogSplitIntoCharacters | ✅   | Recog::split_into_characters | 文字分割                 |
| recogCorrelationBestRow  | ✅   | Recog::correlation_best_row  | 最良相関行の検索         |
| recogCorrelationBestChar | ✅   | Recog::correlation_best_char | 最良相関文字の検索       |
| recogIdentifyPixa        | ✅   | Recog::identify_pixa         | Pixa内の各画像を認識     |
| recogIdentifyPix         | ✅   | Recog::identify_pix          | 単一画像の認識           |
| recogSkipIdentify        | ✅   | Recog::skip_identify         | 認識をスキップ           |
| recogProcessToIdentify   | ✅   | Recog::process_to_identify   | 認識前の画像処理         |
| recogExtractNumbers      | ✅   | Recog::extract_numbers       | 数字列の抽出             |
| showExtractNumbers       | 🚫   | -                            | 数字列抽出のデバッグ表示 |
| rchaDestroy              | ✅   | Drop trait                   | Rcha構造体の自動破棄     |
| rchDestroy               | ✅   | Drop trait                   | Rch構造体の自動破棄      |

#### recog/recog/types.rs (recogident.c)

| C関数       | 状態 | Rust対応      | 備考                    |
| ----------- | ---- | ------------- | ----------------------- |
| rchaExtract | ✅   | Rcha::extract | Rcha配列からデータ抽出  |
| rchExtract  | ✅   | Rch::extract  | Rch構造体からデータ抽出 |

### recogtrain.c (Training)

#### recog/recog/train.rs (recogtrain.c)

| C関数                     | 状態 | Rust対応                                | 備考                                          |
| ------------------------- | ---- | --------------------------------------- | --------------------------------------------- |
| recogTrainLabeled         | ✅   | Recog::train_labeled                    | ラベル付きサンプルで訓練                      |
| recogProcessLabeled       | ✅   | Recog::process_labeled                  | ラベル付きサンプルの処理                      |
| recogAddSample            | ✅   | Recog::add_sample                       | サンプルの追加                                |
| recogModifyTemplate       | ✅   | Recog::modify_template                  | テンプレートの変換（スケール/線幅正規化）     |
| recogAverageSamples       | ✅   | Recog::average_samples                  | サンプルの平均化                              |
| recogTrainingFinished     | ✅   | Recog::finish_training                  | 訓練の完了処理                                |
| recogSortPixaByClass      | ✅   | Recog::sort_pixa_by_class               | クラスごとにPixaをソート                      |
| recogRemoveOutliers1      | ✅   | Recog::remove_outliers1                 | 外れ値除去（方法1）                           |
| recogRemoveOutliers2      | ✅   | Recog::remove_outliers2                 | 外れ値除去（方法2）                           |
| recogShowContent          | 🚫   | -                                       | recog内容の表示（デバッグ/可視化）            |
| recogDebugAverages        | 🚫   | -                                       | 平均テンプレートのデバッグ（デバッグ/可視化） |
| recogShowAverageTemplates | 🚫   | -                                       | 平均テンプレートの表示（デバッグ/可視化）     |
| recogShowMatchesInRange   | 🚫   | -                                       | スコア範囲内のマッチ表示（デバッグ/可視化）   |
| recogShowMatch            | 🚫   | -                                       | マッチの表示（デバッグ/可視化）               |
| pixaAccumulateSamples     | ✅   | recog::train::pixa_accumulate_samples() | サンプルの累積（free fn）                     |
| pixaRemoveOutliers1       | ✅   | recog::train::pixa_remove_outliers1()   | Pixaから外れ値除去（方法1、free fn）          |
| pixaRemoveOutliers2       | ✅   | recog::train::pixa_remove_outliers2()   | Pixaから外れ値除去（方法2、free fn）          |

#### recog/recog/bootstrap.rs (recogtrain.c)

| C関数                       | 状態 | Rust対応                                 | 備考                                 |
| --------------------------- | ---- | ---------------------------------------- | ------------------------------------ |
| recogAddDigitPadTemplates   | 🔄   | pad_digit_training_set 内部実装          | 数字パッドテンプレート追加           |
| recogMakeBootDigitTemplates | 🔄   | make_boot_digit_recog 内部実装           | ブートストラップ数字テンプレート作成 |
| recogTrainFromBoot          | ✅   | recog::bootstrap::train_from_boot        | ブートストラップ認識器から訓練       |
| recogPadDigitTrainingSet    | ✅   | recog::bootstrap::pad_digit_training_set | 数字訓練セットのパディング           |
| recogIsPaddingNeeded        | ✅   | recog::bootstrap::is_padding_needed      | パディングが必要かチェック           |
| recogMakeBootDigitRecog     | ✅   | recog::bootstrap::make_boot_digit_recog  | ブートストラップ数字認識器作成       |

#### recog/recog/ident.rs (recogtrain.c)

| C関数                 | 状態 | Rust対応                   | 備考                           |
| --------------------- | ---- | -------------------------- | ------------------------------ |
| recogFilterPixaBySize | ✅   | Recog::filter_pixa_by_size | サイズによるPixaフィルタリング |

### pageseg.c (Page Segmentation)

#### recog/pageseg.rs (pageseg.c)

| C関数                        | 状態 | Rust対応                             | 備考                     |
| ---------------------------- | ---- | ------------------------------------ | ------------------------ |
| pixGetRegionsBinary          | ✅   | pageseg::segment_regions             | 2値画像から領域抽出      |
| pixFindPageForeground        | ✅   | find_page_foreground                 | ページ前景の検出         |
| pixSplitIntoCharacters       | ✅   | pix_split_into_characters            | 文字への分割             |
| pixSplitComponentWithProfile | ✅   | split_component_with_profile         | プロファイルを使った分割 |
| pixGetWordsInTextlines       | 🔄   | pageseg::get_words_in_textlines      | C関数はclassapp.c所属    |
| pixGetWordBoxesInTextlines   | 🔄   | pageseg::get_word_boxes_in_textlines | C関数はclassapp.c所属    |

### skew.c (Skew Detection)

#### recog/skew.rs (skew.c)

| C関数                               | 状態 | Rust対応                                     | 備考                                  |
| ----------------------------------- | ---- | -------------------------------------------- | ------------------------------------- |
| pixFindSkewAndDeskew                | ✅   | skew::find_skew_and_deskew                   | 傾き検出と補正                        |
| pixFindSkew                         | ✅   | skew::find_skew                              | 傾き検出                              |
| pixFindSkewSweep                    | ✅   | find_skew_sweep                              | スイープによる傾き検出                |
| pixFindSkewSweepAndSearch           | 🔄   | skew::find_skew (内部実装)                   | スイープ+探索（オプション指定で実現） |
| pixFindSkewSweepAndSearchScore      | ✅   | skew::find_skew_sweep_and_search_score       | スイープ+探索（スコア付き）           |
| pixFindSkewSweepAndSearchScorePivot | ✅   | skew::find_skew_sweep_and_search_score_pivot | スイープ+探索（ピボット指定）         |
| pixFindSkewOrthogonalRange          | ✅   | find_skew_orthogonal_range                   | 直交範囲での傾き検出                  |

### dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c (Dewarping)

#### recog/dewarp/types.rs (dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c)

| C関数           | 状態 | Rust対応           | 備考                       |
| --------------- | ---- | ------------------ | -------------------------- |
| dewarpCreate    | ✅   | Dewarp::new        | Dewarp構造体作成           |
| dewarpCreateRef | ✅   | Dewarp::create_ref | 参照ページ指定のDewarp作成 |
| dewarpMinimize  | ✅   | Dewarp::minimize   | Dewarpの最小化             |

#### recog/dewarp/dewarpa.rs (dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c)

| C関数                     | 状態 | Rust対応                      | 備考                                |
| ------------------------- | ---- | ----------------------------- | ----------------------------------- |
| dewarpRead                | ✅   | Dewarp::read                  | Dewarp読み込み                      |
| dewarpReadStream          | ✅   | Dewarp::read<R: Read>         | Dewarpストリーム読み込み            |
| dewarpReadMem             | 🔄   | Dewarp::read (Cursor利用)     | Dewarpメモリ読み込み                |
| dewarpWrite               | ✅   | Dewarp::write                 | Dewarp書き込み                      |
| dewarpWriteStream         | ✅   | Dewarp::write<W: Write>       | Dewarpストリーム書き込み            |
| dewarpWriteMem            | 🔄   | Dewarp::write (Vec利用)       | Dewarpメモリ書き込み                |
| dewarpDestroy             | ✅   | Drop trait                    | 自動破棄                            |
| dewarpaDestroy            | ✅   | Drop trait                    | 自動破棄                            |
| dewarpaSetMaxDistance     | 🔄   | Dewarpa フィールド直接設定    | 最大距離設定                        |
| dewarpaModelStatus        | 🔄   | Dewarp フィールド直接参照     | モデルステータス取得                |
| dewarpaListPages          | 🚫   | -                             | ページリスト表示（デバッグ/可視化） |
| dewarpaInfo               | 🚫   | -                             | Dewarpa情報表示（デバッグ/可視化）  |
| dewarpaModelStats         | 🚫   | -                             | モデル統計取得（デバッグ/可視化）   |
| dewarpaShowArrays         | 🚫   | -                             | 配列の表示（デバッグ/可視化）       |
| dewarpDebug               | 🚫   | -                             | デバッグ出力（デバッグ/可視化）     |
| dewarpShowResults         | 🚫   | -                             | 結果表示（デバッグ/可視化）         |
| dewarpaCreate             | ✅   | Dewarpa::new                  | Dewarpa（複数ページ）作成           |
| dewarpaCreateFromPixacomp | ✅   | Dewarpa::create_from_pixacomp | Pixacompから作成                    |
| dewarpaDestroyDewarp      | 🔄   | Dewarpa::insert (None挿入)    | Dewarpa内の特定Dewarp破棄           |
| dewarpaInsertDewarp       | ✅   | Dewarpa::insert               | DewarpaへDewarp挿入                 |
| dewarpaGetDewarp          | ✅   | Dewarpa::get                  | Dewarpaから特定Dewarp取得           |
| dewarpaSetCurvatures      | ✅   | Dewarpa::set_curvatures       | 曲率パラメータ設定                  |
| dewarpaUseBothArrays      | ✅   | Dewarpa::use_both_arrays      | 両配列の使用設定                    |
| dewarpaSetCheckColumns    | ✅   | Dewarpa::use_single_model     | カラムチェック設定                  |
| dewarpaRead               | ✅   | Dewarpa::read                 | Dewarpa読み込み                     |
| dewarpaReadStream         | ✅   | Dewarpa::read<R: Read>        | Dewarpaストリーム読み込み           |
| dewarpaReadMem            | 🔄   | Dewarpa::read (Cursor利用)    | Dewarpaメモリ読み込み               |
| dewarpaWrite              | ✅   | Dewarpa::write                | Dewarpa書き込み                     |
| dewarpaWriteStream        | ✅   | Dewarpa::write<W: Write>      | Dewarpaストリーム書き込み           |
| dewarpaWriteMem           | 🔄   | Dewarpa::write (Vec利用)      | Dewarpaメモリ書き込み               |
| dewarpaApplyDisparityBoxa | ✅   | Dewarpa::apply_disparity_boxa | Boxaへの歪み補正適用                |
| dewarpaSetValidModels     | 🔄   | Dewarpa::insert_ref_models 等 | 有効モデル設定                      |
| dewarpaInsertRefModels    | ✅   | Dewarpa::insert_ref_models    | 参照モデル挿入                      |
| dewarpaStripRefModels     | ✅   | Dewarpa::strip_ref_models     | 参照モデル削除                      |
| dewarpaRestoreModels      | ✅   | Dewarpa::restore_models       | モデル復元                          |

#### recog/dewarp/model.rs (dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c)

| C関数                         | 状態 | Rust対応                                  | 備考               |
| ----------------------------- | ---- | ----------------------------------------- | ------------------ |
| dewarpBuildPageModel          | ✅   | dewarp::model::build_page_model           | モデル構築         |
| dewarpFindVertDisparity       | ✅   | dewarp::model::build_vertical_disparity   | 垂直歪み検出       |
| dewarpFindHorizDisparity      | ✅   | dewarp::model::build_horizontal_disparity | 水平歪み検出       |
| dewarpFindHorizSlopeDisparity | ✅   | dewarp::model::find_horiz_disparity       | 水平傾斜歪み検出   |
| dewarpBuildLineModel          | 🔄   | dewarp::model::build_page_model 内部      | ラインモデル構築   |
| dewarpPopulateFullRes         | ✅   | dewarp::model::populate_full_resolution   | フル解像度への展開 |

#### recog/dewarp/textline.rs (dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c)

| C関数                    | 状態 | Rust対応                      | 備考                   |
| ------------------------ | ---- | ----------------------------- | ---------------------- |
| dewarpGetTextlineCenters | ✅   | dewarp::find_textline_centers | テキストライン中心検出 |
| dewarpRemoveShortLines   | ✅   | dewarp::remove_short_lines    | 短い線の除去           |

#### recog/dewarp/apply.rs (dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c)

| C関数                 | 状態 | Rust対応                       | 備考         |
| --------------------- | ---- | ------------------------------ | ------------ |
| dewarpaApplyDisparity | ✅   | dewarp::apply::apply_disparity | 歪み補正適用 |

#### recog/dewarp/mod.rs (dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c)

| C関数            | 状態 | Rust対応                   | 備考                 |
| ---------------- | ---- | -------------------------- | -------------------- |
| dewarpSinglePage | ✅   | dewarp::dewarp_single_page | 単一ページの歪み補正 |

#### recog/dewarp/single_page.rs (dewarp1.c, dewarp2.c, dewarp3.c, dewarp4.c)

| C関数                | 状態 | Rust対応                         | 備考                       |
| -------------------- | ---- | -------------------------------- | -------------------------- |
| dewarpSinglePageInit | ✅   | dewarp::dewarp_single_page_init  | 単一ページ歪み補正の初期化 |
| dewarpSinglePageRun  | ✅   | dewarp::dewarp_single_page_run   | 単一ページ歪み補正の実行   |

### baseline.c (Baseline Detection)

#### recog/baseline.rs (baseline.c)

| C関数                    | 状態 | Rust対応                                  | 備考                   |
| ------------------------ | ---- | ----------------------------------------- | ---------------------- |
| pixFindBaselines         | ✅   | baseline::find_baselines                  | ベースライン検出       |
| pixFindBaselinesGen      | 🔄   | baseline::find_baselines (オプション指定) | 汎用ベースライン検出   |
| pixGetLocalSkewAngles    | ✅   | baseline::get_local_skew_angles           | ローカル傾き角配列     |
| pixGetLocalSkewTransform | ✅   | baseline::get_local_skew_transform        | 局所スキュー変換制御点 |
| pixDeskewLocal           | ✅   | baseline::deskew_local                    | 局所スキュー補正       |

### jbclass.c (JBIG2 Classification)

#### recog/jbclass/classify.rs (jbclass.c)

| C関数                              | 状態 | Rust対応                                     | 備考                                 |
| ---------------------------------- | ---- | -------------------------------------------- | ------------------------------------ |
| jbRankHausInit                     | ✅   | jbclass::rank_haus_init                      | Rank Hausdorff分類器初期化           |
| jbCorrelationInit                  | ✅   | jbclass::correlation_init                    | 相関ベース分類器初期化               |
| jbCorrelationInitWithoutComponents | ✅   | jbclass::correlation_init_without_components | コンポーネントなし相関分類器初期化   |
| jbAddPageComponents                | ✅   | jbclass::add_page_components                 |                                      |
| jbClassifyRankHaus                 | 🔄   | JbClasser (内部実装)                         | Rank Hausdorff分類（内部で自動実行） |
| jbClassifyCorrelation              | 🔄   | JbClasser (内部実装)                         | 相関ベース分類（内部で自動実行）     |
| jbClasserCreate                    | 🔄   | rank_haus_init / correlation_init            | 分類器作成（専用関数に分割）         |
| jbClasserDestroy                   | ✅   | Drop trait                                   | 自動破棄                             |
| jbGetULCorners                     | 🔄   | JbData フィールド直接参照                    | 左上コーナー取得                     |
| jbGetLLCorners                     | 🔄   | JbData フィールド直接参照                    | 左下コーナー取得                     |
| pixHaustest                        | 🔄   | jbclass::hausdorff_distance                  | rank=1.0で相当                       |
| pixRankHaustest                    | ✅   | jbclass::hausdorff_distance                  | size/rank引数で対応                  |
| jbGetComponents                    | ✅   | JbClasser::get_components                    |                                      |
| jbAccumulateComposites             | 🔄   | JbClasser::get_data                          | 合成処理は内部実装                   |
| jbTemplatesFromComposites          | ✅   | JbClasser::templates_from_composites         |                                      |
| jbDataDestroy                      | 🔄   | Drop trait                                   | Rustでは所有権で自動破棄             |
| jbDataRender                       | 🔄   | JbData::render_page / JbData::render_all     | 単一/全ページに分離                  |
| jbCorrelation                      | 🔄   | classapp.c セクション参照                    | C関数はclassapp.c所属                |
| jbRankHaus                         | 🔄   | classapp.c セクション参照                    | C関数はclassapp.c所属                |
| jbWordsInTextlines                 | 🔄   | classapp.c セクション参照                    | `pixWordMaskByDilation` とは別関数   |
| jbAddPages                         | ✅   | JbClasser::add_pages                         | 複数ページ追加                       |
| jbAddPage                          | ✅   | JbClasser::add_page                          | ページ追加                           |
| jbDataSave                         | ✅   | JbClasser::get_data                          | データ取得                           |
| pixWordMaskByDilation              | ✅   | jbclass::pix_word_mask_by_dilation           |                                      |
| pixWordBoxesByDilation             | ✅   | jbclass::pix_word_boxes_by_dilation          |                                      |

#### recog/jbclass/io.rs (jbclass.c)

| C関数       | 状態 | Rust対応        | 備考                      |
| ----------- | ---- | --------------- | ------------------------- |
| jbDataRead  | 🔄   | JbData::read()  | データ読み込み（I/O追加） |
| jbDataWrite | 🔄   | JbData::write() | データ書き込み（I/O追加） |

### classapp.c (JBIG2分類応用)

#### recog/jbclass/classify.rs (classapp.c)

| C関数         | 状態 | Rust対応                | 備考                      |
| ------------- | ---- | ----------------------- | ------------------------- |
| jbCorrelation | ✅   | jbclass::jb_correlation | 相関ベース高レベルAPI     |
| jbRankHaus    | ✅   | jbclass::jb_rank_haus   | Rank Hausdorff高レベルAPI |

#### recog/pageseg.rs (classapp.c)

| C関数                      | 状態 | Rust対応                             | 備考                               |
| -------------------------- | ---- | ------------------------------------ | ---------------------------------- |
| jbWordsInTextlines         | 🔄   | pageseg::get_words_in_textlines      | classappの分類器処理は未移植       |
| pixGetWordsInTextlines     | ✅   | pageseg::get_words_in_textlines      | テキストライン内の単語取得         |
| pixGetWordBoxesInTextlines | ✅   | pageseg::get_word_boxes_in_textlines | テキストライン内の単語ボックス取得 |

#### recog/classapp.rs (classapp.c)

| C関数                        | 状態 | Rust対応                                | 備考                         |
| ---------------------------- | ---- | --------------------------------------- | ---------------------------- |
| pixFindWordAndCharacterBoxes | ✅   | classapp::find_word_and_character_boxes | 単語および文字ボックスの検出 |
| boxaExtractSortedPattern     | ✅   | classapp::boxa_extract_sorted_pattern   | パターンに基づくBoxa抽出     |
| numaaCompareImagesByBoxes    | ✅   | classapp::numaa_compare_images_by_boxes | ボックスベースの画像比較     |

### bootnumgen1.c, bootnumgen2.c, bootnumgen3.c, bootnumgen4.c (Bootstrap数字生成データ)

#### recog/mod.rs (bootnumgen1.c, bootnumgen2.c, bootnumgen3.c, bootnumgen4.c)

| C関数          | 状態 | Rust対応 | 備考                                                      |
| -------------- | ---- | -------- | --------------------------------------------------------- |
| l_bootnum_gen1 | 🚫   | -        | ブートストラップ数字セット1（組み込みデータジェネレータ） |
| l_bootnum_gen2 | 🚫   | -        | ブートストラップ数字セット2（組み込みデータジェネレータ） |
| l_bootnum_gen3 | 🚫   | -        | ブートストラップ数字セット3（組み込みデータジェネレータ） |
| l_bootnum_gen4 | 🚫   | -        | ブートストラップ数字セット4（組み込みデータジェネレータ） |

### bardecode.c (Barcode Decoding)

#### recog/barcode/decode.rs (bardecode.c)

| C関数                    | 状態 | Rust対応              | 備考                           |
| ------------------------ | ---- | --------------------- | ------------------------------ |
| barcodeDispatchDecoder   | 🔄   | dispatch_decoder()    | バーコードデコーダディスパッチ |
| barcodeFormatIsSupported | 🔄   | is_format_supported() | フォーマットサポート確認       |

### readbarcode.c (Barcode Reading)

#### recog/barcode/mod.rs (readbarcode.c)

| C関数              | 状態 | Rust対応                  | 備考                       |
| ------------------ | ---- | ------------------------- | -------------------------- |
| pixProcessBarcodes | ✅   | barcode::process_barcodes | バーコード処理             |
| pixReadBarcodes    | ✅   | read_barcodes             | Pixaからバーコード読み取り |

#### recog/barcode/detect.rs (readbarcode.c)

| C関数                  | 状態 | Rust対応                 | 備考               |
| ---------------------- | ---- | ------------------------ | ------------------ |
| pixExtractBarcodes     | ✅   | extract_barcodes         | バーコード抽出     |
| pixLocateBarcodes      | ✅   | locate_barcodes          | バーコード位置検出 |
| pixDeskewBarcode       | ✅   | deskew_barcode           | バーコード傾き補正 |
| pixGenerateBarcodeMask | 🔄   | barcode_gen_mask         | C版はstatic関数    |

#### recog/barcode/signal.rs (readbarcode.c)

| C関数                      | 状態 | Rust対応                                | 備考                                  |
| -------------------------- | ---- | --------------------------------------- | ------------------------------------- |
| pixReadBarcodeWidths       | 🔄   | barcode::signal::extract_barcode_widths | バーコード幅読み取り（Direction対応） |
| pixExtractBarcodeWidths1   | 🔄   | barcode::signal::extract_barcode_widths | バーコード幅抽出（統合API）           |
| pixExtractBarcodeWidths2   | 🔄   | barcode::signal::extract_barcode_widths | バーコード幅抽出（統合API）           |
| pixExtractBarcodeCrossings | ✅   | barcode::signal::extract_crossings      | バーコード交差点抽出                  |

#### core/numa/operations.rs (readbarcode.c)

| C関数                   | 状態 | Rust対応                     | 備考                                   |
| ----------------------- | ---- | ---------------------------- | -------------------------------------- |
| numaFindLocForThreshold | ✅   | Numa::find_loc_for_threshold | numafunc2.c由来。readbarcode.cではない |

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
4. **埋め込みデータ生成（4関数）**: l_bootnum_gen1, l_bootnum_gen2, l_bootnum_gen3, l_bootnum_gen4 — Rustでは静的データを直接利用

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
- Rust版実装済み: 151関数（✅125 + 🔄26）
- 🚫不要: 18関数（デバッグ/可視化系・C固有getter/setter）
- ❌未実装: 0関数
- 実装率: 89.3%（全体）、100.0%（🚫不要除外ベース）

C版の全機能を網羅することは目標ではなく、Rustの慣用的な設計で同等の機能を提供することを重視しています。特に以下の点で設計が異なります：

1. メモリ管理はRustの所有権システムで自動化
2. エラー処理はResult型で型安全に
3. デバッグ機能は標準のDebug traitや外部ツールで代替
4. 複数ページ管理はDewarpa構造体（Vec<Option<Dewarp>>）で実現
