# leptonica (src/morph/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（301_morph-full-porting全Phase完了を反映）

## サマリー

| 項目      | 数  |
| --------- | --- |
| ✅ 同等   | 108 |
| 🔄 異なる | 19  |
| ❌ 未実装 | 11  |
| 🚫 不要   | 25  |
| 合計      | 163 |

## 詳細

### morph.c (基本形態学演算)

#### morph/binary.rs (morph.c)

| C関数                       | 状態 | Rust対応                        | 備考                                                |
| --------------------------- | ---- | ------------------------------- | --------------------------------------------------- |
| pixDilate                   | ✅   | binary::dilate                  |                                                     |
| pixErode                    | ✅   | binary::erode                   |                                                     |
| pixOpen                     | ✅   | binary::open                    |                                                     |
| pixClose                    | ✅   | binary::close                   |                                                     |
| pixCloseSafe                | ✅   | binary::close_safe              | Phase 1で実装                                       |
| pixOpenGeneralized          | ✅   | binary::open_generalized        | Phase 1で実装                                       |
| pixCloseGeneralized         | ✅   | binary::close_generalized       | Phase 1で実装                                       |
| pixDilateBrick              | ✅   | binary::dilate_brick            |                                                     |
| pixErodeBrick               | ✅   | binary::erode_brick             |                                                     |
| pixOpenBrick                | ✅   | binary::open_brick              |                                                     |
| pixCloseBrick               | ✅   | binary::close_brick             |                                                     |
| pixCloseSafeBrick           | ✅   | binary::close_safe_brick        | Phase 1で実装                                       |
| pixDilateCompBrick          | 🔄   | binary::dilate_brick            | Rust版は分離可能分解を自動選択                      |
| pixErodeCompBrick           | 🔄   | binary::erode_brick             | Rust版は分離可能分解を自動選択                      |
| pixOpenCompBrick            | 🔄   | binary::open_brick              | Rust版は分離可能分解を自動選択                      |
| pixCloseCompBrick           | 🔄   | binary::close_brick             | Rust版は分離可能分解を自動選択                      |
| pixCloseSafeCompBrick       | ✅   | binary::close_safe_comp_brick   | Phase 1で実装                                       |
| pixHMT                      | ✅   | binary::hit_miss_transform      |                                                     |
| selectComposableSizes       | ✅   | binary::select_composable_sizes |                                                     |
| resetMorphBoundaryCondition | 🚫   | -                               | C版グローバル変数setter（Rustでは引数で明示指定）   |
| getMorphBorderPixelColor    | 🚫   | -                               | C版グローバル状態アクセサ（Rustでは引数で明示指定） |
| selectComposableSels        | ❌   | -                               | Rust版はSEL生成APIとして分離                        |

### morphapp.c (応用演算)

#### morph/morphapp.rs (morphapp.c)

| C関数                        | 状態 | Rust対応                              | 備考                                  |
| ---------------------------- | ---- | ------------------------------------- | ------------------------------------- |
| pixMorphGradient             | ✅   | morphapp::morph_gradient              | Phase 2で実装                         |
| pixMorphSequenceMasked       | ✅   | morphapp::morph_sequence_masked       | Phase 2で実装                         |
| pixUnionOfMorphOps           | ✅   | morphapp::union_of_morph_ops          | Phase 2で実装                         |
| pixIntersectionOfMorphOps    | ✅   | morphapp::intersection_of_morph_ops   | Phase 2で実装                         |
| pixSeedfillMorph             | ✅   | morphapp::seedfill_morph              | Phase 2で実装                         |
| pixMorphSequenceByComponent  | ✅   | morph_sequence_by_component           | 連結成分ごとの処理                    |
| pixMorphSequenceByRegion     | ✅   | morph_sequence_by_region              | 領域ごとの処理                        |
| pixaMorphSequenceByComponent | 🔄   | morphapp::morph_sequence_by_component | Rust版はPIXAではなく合成済みPixを返す |
| pixaMorphSequenceByRegion    | 🔄   | morphapp::morph_sequence_by_region    | Rust版はPIXAではなく合成済みPixを返す |
| pixSelectiveConnCompFill     | ❌   | -                                     |                                       |
| pixRemoveMatchedPattern      | ❌   | -                                     |                                       |
| pixDisplayMatchedPattern     | ❌   | -                                     |                                       |
| pixaExtendByMorph            | ❌   | -                                     |                                       |
| pixaExtendByScaling          | ❌   | -                                     |                                       |
| pixRunHistogramMorph         | ❌   | -                                     |                                       |
| pixHDome                     | ❌   | -                                     |                                       |
| pixFastTophat                | ❌   | -                                     |                                       |
| pixaCentroids                | ❌   | -                                     |                                       |
| pixCentroid                  | ❌   | -                                     |                                       |
| pixGrayscaleMorphSum         | 🚫   | -                                     | C版leptonicaに存在しない関数          |
| pixMultiplyByColor           | 🚫   | -                                     | blend.c所属（morph領域外）            |
| pixHMTDwa                    | 🚫   | -                                     | DWA自動生成コード                     |
| pixFHMTGen                   | 🚫   | -                                     | DWAコード生成                         |

#### morph/binary.rs (morphapp.c)

| C関数              | 状態 | Rust対応                 | 備考                                   |
| ------------------ | ---- | ------------------------ | -------------------------------------- |
| pixExtractBoundary | ✅   | binary::extract_boundary | Phase 2で実装                          |
| pixTophat          | 🔄   | binary::top_hat          | C版は8bpp演算。Rust版は1bpp二値top-hat |

### morphdwa.c (DWA実装)

#### morph/dwa.rs (morphdwa.c)

| C関数                          | 状態 | Rust対応                               | 備考                                  |
| ------------------------------ | ---- | -------------------------------------- | ------------------------------------- |
| pixDilateBrickDwa              | ✅   | dwa::dilate_brick_dwa                  |                                       |
| pixErodeBrickDwa               | ✅   | dwa::erode_brick_dwa                   |                                       |
| pixOpenBrickDwa                | ✅   | dwa::open_brick_dwa                    |                                       |
| pixCloseBrickDwa               | ✅   | dwa::close_brick_dwa                   |                                       |
| pixDilateCompBrickDwa          | ✅   | dwa::dilate_comp_brick_dwa             | Phase 5で実装                         |
| pixErodeCompBrickDwa           | ✅   | dwa::erode_comp_brick_dwa              | Phase 5で実装                         |
| pixOpenCompBrickDwa            | ✅   | dwa::open_comp_brick_dwa               | Phase 5で実装                         |
| pixCloseCompBrickDwa           | ✅   | dwa::close_comp_brick_dwa              | Phase 5で実装                         |
| pixDilateCompBrickExtendDwa    | ✅   | dwa::dilate_comp_brick_extend_dwa      | Phase 5で実装                         |
| pixErodeCompBrickExtendDwa     | ✅   | dwa::erode_comp_brick_extend_dwa       | Phase 5で実装                         |
| pixOpenCompBrickExtendDwa      | ✅   | dwa::open_comp_brick_extend_dwa        | Phase 5で実装                         |
| pixCloseCompBrickExtendDwa     | ✅   | dwa::close_comp_brick_extend_dwa       | Phase 5で実装                         |
| getExtendedCompositeParameters | ✅   | dwa::get_extended_composite_parameters | Phase 5で実装                         |
| pixMorphDwa_*系                | 🚫   | -                                      | DWA自動生成関数（Rustでは手書き実装） |

### morphseq.c (シーケンス処理)

#### morph/sequence.rs (morphseq.c)

| C関数                   | 状態 | Rust対応                          | 備考                  |
| ----------------------- | ---- | --------------------------------- | --------------------- |
| pixMorphSequence        | ✅   | sequence::morph_sequence          |                       |
| pixMorphCompSequence    | ✅   | sequence::morph_comp_sequence     |                       |
| pixMorphSequenceDwa     | ✅   | sequence::morph_sequence_dwa      | Phase 5で実装         |
| pixMorphCompSequenceDwa | ✅   | sequence::morph_comp_sequence_dwa | Phase 5で実装         |
| morphSequenceVerify     | 🔄   | sequence内部で検証                | 公開APIとしては未実装 |
| pixGrayMorphSequence    | ✅   | sequence::gray_morph_sequence     |                       |
| pixColorMorphSequence   | ✅   | sequence::color_morph_sequence    | Phase 5で実装         |

### graymorph.c (グレースケール形態学)

#### morph/grayscale.rs (graymorph.c)

| C関数          | 状態 | Rust対応               | 備考                                             |
| -------------- | ---- | ---------------------- | ------------------------------------------------ |
| pixErodeGray   | ✅   | grayscale::erode_gray  |                                                  |
| pixDilateGray  | ✅   | grayscale::dilate_gray |                                                  |
| pixOpenGray    | ✅   | grayscale::open_gray   |                                                  |
| pixCloseGray   | ✅   | grayscale::close_gray  |                                                  |
| pixErodeGray3  | 🔄   | grayscale::erode_gray  | erode_gray() が 3x3 で fast path にディスパッチ  |
| pixDilateGray3 | 🔄   | grayscale::dilate_gray | dilate_gray() が 3x3 で fast path にディスパッチ |
| pixOpenGray3   | 🔄   | grayscale::open_gray   | open_gray() が 3x3 で fast path にディスパッチ   |
| pixCloseGray3  | 🔄   | grayscale::close_gray  | close_gray() が 3x3 で fast path にディスパッチ  |
| dilateGrayLow  | 🚫   | -                      | 低レベル内部関数（高レベルAPIで対応済み）        |
| erodeGrayLow   | 🚫   | -                      | 低レベル内部関数（高レベルAPIで対応済み）        |
| pixTophat      | 🚫   | -                      | graymorph.cには定義なし（morphapp.cを参照）      |

### colormorph.c (カラー形態学)

#### morph/color.rs (colormorph.c)

| C関数         | 状態 | Rust対応                | 備考                         |
| ------------- | ---- | ----------------------- | ---------------------------- |
| pixColorMorph | 🔄   | color::dilate_color等   | C版は1関数、Rust版は個別関数 |
| -             | ✅   | color::dilate_color     | Rust版で追加                 |
| -             | ✅   | color::erode_color      | Rust版で追加                 |
| -             | ✅   | color::open_color       | Rust版で追加                 |
| -             | ✅   | color::close_color      | Rust版で追加                 |
| -             | ✅   | color::gradient_color   | Rust版で追加                 |
| -             | ✅   | color::top_hat_color    | Rust版で追加                 |
| -             | ✅   | color::bottom_hat_color | Rust版で追加                 |

### sel1.c (Sel基本操作)

#### morph/sel.rs (sel1.c)

| C関数                   | 状態 | Rust対応                                                | 備考                         |
| ----------------------- | ---- | ------------------------------------------------------- | ---------------------------- |
| selaCreate              | ✅   | sel::Sela::new                                          | Phase 6で実装                |
| selaDestroy             | ✅   | Drop trait                                              | Rust自動メモリ管理           |
| selCreate               | ✅   | sel::Sel::new                                           |                              |
| selDestroy              | ✅   | Drop trait                                              |                              |
| selCopy                 | ✅   | Clone trait                                             |                              |
| selCreateBrick          | ✅   | sel::Sel::create_brick                                  |                              |
| selCreateComb           | ✅   | sel::Sel::create_comb_horizontal / create_comb_vertical |                              |
| create2dIntArray        | 🔄   | Vec<Vec<>>                                              | Rustでは不要                 |
| selaAddSel              | ✅   | sel::Sela::add                                          | Phase 6で実装                |
| selaGetCount            | ✅   | sel::Sela::count                                        | Phase 6で実装                |
| selaGetSel              | ✅   | sel::Sela::get                                          | Phase 6で実装                |
| selGetName              | ✅   | sel::Sel::name()                                        |                              |
| selSetName              | ✅   | sel::Sel::set_name()                                    |                              |
| selaFindSelByName       | ✅   | sel::Sela::find_by_name                                 | Phase 6で実装                |
| selGetElement           | ✅   | sel::Sel::get_element                                   |                              |
| selSetElement           | ✅   | sel::Sel::set_element                                   |                              |
| selGetParameters        | ✅   | sel::Sel::get_parameters                                | Phase 3で実装                |
| selSetOrigin            | ✅   | sel::Sel::set_origin                                    |                              |
| selGetTypeAtOrigin      | ✅   | get_elementでorigin参照                                 |                              |
| selaGetSelnames         | 🔄   | iterate + name()                                        |                              |
| selFindMaxTranslations  | ✅   | sel::Sel::find_max_translations                         |                              |
| selRotateOrth           | ✅   | sel::Sel::rotate_orth                                   |                              |
| selaRead                | ✅   | sel::Sela::read                                         | Phase 6で実装                |
| selaReadStream          | ✅   | sel::Sela::read_from_reader                             | Phase 6で実装                |
| selRead                 | ✅   | sel::Sel::read                                          | Phase 3で実装                |
| selReadStream           | ✅   | sel::Sel::read_from_reader                              | Phase 3で実装                |
| selaWrite               | ✅   | sel::Sela::write                                        | Phase 6で実装                |
| selaWriteStream         | ✅   | sel::Sela::write_to_writer                              | Phase 6で実装                |
| selWrite                | ✅   | sel::Sel::write                                         | Phase 3で実装                |
| selWriteStream          | ✅   | sel::Sel::write_to_writer                               | Phase 3で実装                |
| selCreateFromString     | ✅   | sel::Sel::from_string                                   |                              |
| selPrintToString        | ✅   | sel::Sel::print_to_string                               | Phase 3で実装                |
| selaCreateFromFile      | ✅   | sel::Sela::read                                         | Phase 6で実装                |
| selCreateFromPta        | ✅   | sel::Sel::from_pta                                      | Phase 3で実装                |
| selCreateFromPix        | ✅   | sel::Sel::from_pix                                      | Phase 3で実装                |
| selReadFromColorImage   | ✅   | sel::Sel::from_color_image                              | Phase 3で実装                |
| selCreateFromColorPix   | ✅   | sel::Sel::from_color_image                              | Phase 3で実装                |
| selaCreateFromColorPixa | ✅   | sela_create_from_color_pixa                             | Pixa操作はアプリケーション層 |
| selDisplayInPix         | 🚫   | -                                                       | 可視化専用                   |
| selaDisplayInPix        | 🚫   | -                                                       | 可視化専用                   |
| selaGetBrickName        | 🔄   | Sela::find_by_name                                      | 命名規則で検索               |
| selaGetCombName         | 🔄   | Sela::find_by_name                                      | 命名規則で検索               |

#### morph/dwa.rs (sel1.c)

| C関数                  | 状態 | Rust対応                                    | 備考 |
| ---------------------- | ---- | ------------------------------------------- | ---- |
| getCompositeParameters | ✅   | dwa内部 + get_extended_composite_parameters |      |

### sel2.c (Sel定義済みセット)

#### morph/thin_sels.rs (sel2.c)

| C関数                 | 状態 | Rust対応                  | 備考           |
| --------------------- | ---- | ------------------------- | -------------- |
| sel4ccThin系 (16関数) | 🔄   | thin_sels::sels_4cc_thin  | 一括生成で対応 |
| sel8ccThin系 (16関数) | 🔄   | thin_sels::sels_8cc_thin  | 一括生成で対応 |
| sela4and8ccThin       | ✅   | thin_sels::make_thin_sels |                |

#### morph/sel.rs (sel2.c)

| C関数                 | 状態 | Rust対応                      | 備考          |
| --------------------- | ---- | ----------------------------- | ------------- |
| selMakePlusSign       | ✅   | sel::sel_make_plus_sign       |               |
| selaAddBasic          | ✅   | sel::sela_add_basic           | Phase 4で実装 |
| selaAddHitMiss        | ✅   | sel::sela_add_hit_miss        | Phase 4で実装 |
| selaAddDwaLinear      | ✅   | sel::sela_add_dwa_linear      | Phase 4で実装 |
| selaAddDwaCombs       | ✅   | sel::sela_add_dwa_combs       | Phase 4で実装 |
| selaAddCrossJunctions | ✅   | sel::sela_add_cross_junctions | Phase 4で実装 |
| selaAddTJunctions     | ✅   | sel::sela_add_t_junctions     | Phase 4で実装 |

### selgen.c (Sel自動生成)

#### morph/selgen.rs (selgen.c)

| C関数                      | 状態 | Rust対応                  | 備考                                                         |
| -------------------------- | ---- | ------------------------- | ------------------------------------------------------------ |
| pixGenerateSelBoundary     | ✅   | generate_sel_boundary     |                                                              |
| pixGenerateSelWithRuns     | ✅   | generate_sel_with_runs    |                                                              |
| pixGenerateSelRandom       | ✅   | generate_sel_random       |                                                              |
| pixGetRunCentersOnLine     | ✅   | get_run_centers_on_line   |                                                              |
| pixGetRunsOnLine           | ✅   | get_runs_on_line          |                                                              |
| pixSubsampleBoundaryPixels | ✅   | subsample_boundary_pixels |                                                              |
| adjacentOnPixelInRaster    | 🚫   | -                         | 低レベル内部ヘルパー（pixSubsampleBoundaryPixels内部で使用） |
| pixDisplayHitMissSel       | 🚫   | -                         | 可視化専用                                                   |

### ccthin.c (連結成分保存細線化)

#### morph/thin.rs (ccthin.c)

| C関数                 | 状態 | Rust対応                    | 備考 |
| --------------------- | ---- | --------------------------- | ---- |
| pixaThinConnected     | ✅   | pixa_thin_connected         |      |
| pixThinConnected      | ✅   | thin::thin_connected        |      |
| pixThinConnectedBySet | ✅   | thin::thin_connected_by_set |      |

#### morph/thin_sels.rs (ccthin.c)

| C関数            | 状態 | Rust対応                  | 備考 |
| ---------------- | ---- | ------------------------- | ---- |
| selaMakeThinSets | ✅   | thin_sels::make_thin_sels |      |

### dwacomb.2.c / dwacomblow.2.c / fmorphauto.c / fmorphgen.1.c / fmorphgenlow.1.c / fhmtauto.c / fhmtgen.1.c / fhmtgenlow.1.c (DWAコード生成・自動生成)

#### morph/mod.rs (dwacomb.2.c)

| C関数             | 状態 | Rust対応 | 備考                                |
| ----------------- | ---- | -------- | ----------------------------------- |
| fmorphopgen_low_2 | 🚫   | -        | DWAコード生成（Rustでは不要）       |
| dwacomblow_low_2  | 🚫   | -        | DWA合成低レベル生成（Rustでは不要） |
| fmorphautogen     | 🚫   | -        | DWAコード自動生成（Rustでは不要）   |
| fmorphautogen1    | 🚫   | -        | DWAコード自動生成（Rustでは不要）   |
| fmorphautogen2    | 🚫   | -        | DWAコード自動生成（Rustでは不要）   |
| fmorphopgen_low_1 | 🚫   | -        | DWAコード生成（Rustでは不要）       |
| fhmtautogen       | 🚫   | -        | HMT DWA自動生成（Rustでは不要）     |
| fhmtautogen1      | 🚫   | -        | HMT DWA自動生成（Rustでは不要）     |
| fhmtautogen2      | 🚫   | -        | HMT DWA自動生成（Rustでは不要）     |
| fhmtgen_low_1     | 🚫   | -        | HMT低レベル生成（Rustでは不要）     |
| (低レベル関数群)  | 🚫   | -        | DWA内部実装（Rustでは手書き実装）   |

## 実装状況の分析

### 実装済み領域

1. **基本形態学演算**: dilate, erode, open, close (binary, gray, color)
2. **Safe closing**: 境界アーティファクト防止版のclose演算
3. **Generalized ops**: 反復付きopen/close
4. **Morphological applications**: マスク付きシーケンス、集合演算、seedfill、勾配
5. **Brick演算**: 矩形SELによる高速演算
6. **DWA演算**: 基本brick DWA、composite DWA、extended DWA（>63px対応）
7. **シーケンス処理**: binary/gray/DWA/composite DWA/colorシーケンス
8. **細線化**: 連結成分保存細線化
9. **グレースケール形態学**: van Herk/Gil-Werman法による実装
10. **カラー形態学**: RGB各成分への個別適用
11. **Sel/Sela管理**: 構造体、I/O、生成、検索
12. **SEL定義済みセット**: basic、hit-miss、DWA linear/combs、cross/T junctions

### 実装完了領域（元未実装 → 全て実装済み）

1. **Sel自動生成**: pixGenerateSelBoundary等（selgen.c）— 実装済み
2. **morphapp関数**: by-component/by-region処理 — 実装済み
3. **selaCreateFromColorPixa**: Pixa→Sela一括変換 — 実装済み
4. **pixaThinConnected**: PIXA版の細線化 — 実装済み

### 不要（Rustでは対応不要）

1. **DWAコード生成機能**: fmorphautogen等（Rustでは手書き実装で対応）
2. **DWA自動生成関数**: pixMorphDwa_*系、pixHMTDwa、pixFHMTGen等
3. **C版グローバル変数管理**: resetMorphBoundaryCondition、getMorphBorderPixelColor（Rustでは引数で明示指定）
4. **低レベル内部関数**: dilateGrayLow、erodeGrayLow（高レベルAPIで対応済み）
5. **可視化専用関数**: selDisplayInPix、selaDisplayInPix、pixDisplayHitMissSel
6. **morph領域外の関数**: pixMultiplyByColor（blend.c所属）、pixGrayscaleMorphSum（C版に存在しない）

### アーキテクチャの違い

#### C版の特徴

- グローバル変数で境界条件を管理
- 関数名でoperationタイプを指定（pixColorMorph(type)）
- DWAコードは実行時に生成されたCコードを使用
- Sel/Selaは複雑なポインタ配列構造

#### Rust版の特徴

- 境界条件は引数で明示的に指定
- 個別の型安全な関数（dilate_color, erode_color等）
- DWAは手書き実装（コード生成不要）
- Sel/Selaは安全なVec/Struct構造
- 合成分解（composite decomposition）を自動選択
