# leptonica (src/region/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## サマリー

| 項目      | 数 |
| --------- | -- |
| ✅ 同等   | 65 |
| 🔄 異なる | 8  |
| ❌ 未実装 | 0  |
| 🚫 不要   | 22 |
| 合計      | 95 |

## 詳細

### conncomp.c

| C関数               | 状態      | Rust対応                              | 備考                                                      |
| ------------------- | --------- | ------------------------------------- | --------------------------------------------------------- |
| pixConnComp         | 🔄 異なる | find_connected_components             | 異なるAPI: Rust版はVec<ConnectedComponent>を返す          |
| pixConnCompPixa     | ✅ 同等   | conncomp_pixa()                       | -                                                         |
| pixConnCompBB       | 🔄 異なる | find_connected_components             | 異なるAPI: bounding box情報はConnectedComponentに含まれる |
| pixCountConnComp    | ✅ 同等   | pix_count_components (label.rs)       | -                                                         |
| nextOnPixelInRaster | ✅ 同等   | conncomp.rs next_on_pixel_in_raster() | -                                                         |
| pixSeedfillBB       | ✅ 同等   | `conncomp::seedfill_bb`               |                                                           |
| pixSeedfill4BB      | ✅ 同等   | `conncomp::seedfill_4_bb`             |                                                           |
| pixSeedfill8BB      | ✅ 同等   | `conncomp::seedfill_8_bb`             |                                                           |
| pixSeedfill         | ✅ 同等   | `conncomp::seedfill`                  |                                                           |
| pixSeedfill4        | ✅ 同等   | `conncomp::seedfill_4`                |                                                           |
| pixSeedfill8        | ✅ 同等   | `conncomp::seedfill_8`                |                                                           |

### ccbord.c

| C関数                     | 状態      | Rust対応                            | 備考                          |
| ------------------------- | --------- | ----------------------------------- | ----------------------------- |
| ccbaCreate                | 🚫 不要   | -                                   | Cメモリ管理: Rustでは不要     |
| ccbaDestroy               | 🚫 不要   | -                                   | Cメモリ管理: Rustでは不要     |
| ccbCreate                 | 🚫 不要   | -                                   | Cメモリ管理: Rustでは不要     |
| ccbDestroy                | 🚫 不要   | -                                   | Cメモリ管理: Rustでは不要     |
| ccbaAddCcb                | 🚫 不要   | -                                   | Cデータ構造管理: Rustでは不要 |
| ccbaExtendArray           | 🚫 不要   | -                                   | Cデータ構造管理: Rustでは不要 |
| ccbaGetCount              | 🚫 不要   | -                                   | Cデータ構造管理: Rustでは不要 |
| ccbaGetCcb                | 🚫 不要   | -                                   | Cデータ構造管理: Rustでは不要 |
| pixGetAllCCBorders        | 🔄 異なる | get_all_borders                     | 異なるAPI: ImageBordersを返す |
| pixGetCCBorders           | ✅ 同等   | `get_component_borders`             |                               |
| pixGetOuterBordersPtaa    | 🔄 異なる | get_outer_borders                   | 異なるAPI: Vec<Border>を返す  |
| pixGetOuterBorderPta      | 🔄 異なる | get_outer_border                    | 異なるAPI: Borderを返す       |
| pixGetOuterBorder         | ✅ 同等   | `get_outer_border`                  |                               |
| pixGetHoleBorder          | ✅ 同等   | `pix_get_hole_border`               |                               |
| findNextBorderPixel       | ✅ 同等   | find_next_border_pixel (private)    | -                             |
| locateOutsideSeedPixel    | ✅ 同等   | `locate_outside_seed_pixel`         |                               |
| ccbaGenerateGlobalLocs    | ✅ 同等   | `ccbord::generate_global_locs`      |                               |
| ccbaGenerateStepChains    | ✅ 同等   | `ccbord::generate_step_chains`      |                               |
| ccbaStepChainsToPixCoords | ✅ 同等   | `ccbord::step_chains_to_pix_coords` |                               |
| ccbaGenerateSPGlobalLocs  | ✅ 同等   | `ccbord::generate_sp_global_locs`   |                               |
| ccbaGenerateSinglePath    | ✅ 同等   | `ccbord::generate_single_path`      |                               |
| getCutPathForHole         | ✅ 同等   | `get_cut_path_for_hole`             |                               |
| ccbaDisplayBorder         | 🚫 不要   | -                                   | 表示/可視化関数: Rustでは不要 |
| ccbaDisplaySPBorder       | 🚫 不要   | -                                   | 表示/可視化関数: Rustでは不要 |
| ccbaDisplayImage1         | 🚫 不要   | -                                   | 表示/可視化関数: Rustでは不要 |
| ccbaDisplayImage2         | 🚫 不要   | -                                   | 表示/可視化関数: Rustでは不要 |
| ccbaWrite                 | ✅ 同等   | `ccbord::write_to_file`             |                               |
| ccbaWriteStream           | ✅ 同等   | `ccbord::write<W: Write>`           |                               |
| ccbaRead                  | ✅ 同等   | `ccbord::read_from_file`            |                               |
| ccbaReadStream            | ✅ 同等   | `ccbord::read_from<R: Read>`        |                               |
| ccbaWriteSVG              | ✅ 同等   | `ccbord::write_svg`                 |                               |
| ccbaWriteSVGString        | ✅ 同等   | `ccbord::to_svg_string`             |                               |

### seedfill.c

| C関数                       | 状態      | Rust対応                                  | 備考                                   |
| --------------------------- | --------- | ----------------------------------------- | -------------------------------------- |
| pixSeedfillBinary           | 🔄 異なる | seedfill_binary                           | 異なるAPI: SeedFillOptionsを使用       |
| pixSeedfillBinaryRestricted | ✅ 同等   | seedfill_binary_restricted()              | -                                      |
| seedfillBinaryLow           | 🚫 不要   | -                                         | Low-level内部関数: 高レベルAPIでカバー |
| pixHolesByFilling           | ✅ 同等   | fill_holes                                | -                                      |
| pixFillClosedBorders        | ✅ 同等   | fill_closed_borders()                     | -                                      |
| pixExtractBorderConnComps   | ✅ 同等   | seedfill.rs extract_border_conn_comps()   | -                                      |
| pixRemoveBorderConnComps    | ✅ 同等   | clear_border                              | -                                      |
| pixFillBgFromBorder         | ✅ 同等   | seedfill.rs fill_bg_from_border()         | -                                      |
| pixFillHolesToBoundingRect  | ✅ 同等   | seedfill.rs fill_holes_to_bounding_rect() | -                                      |
| pixSeedfillGray             | ✅ 同等   | seedfill_gray                             | -                                      |
| pixSeedfillGrayInv          | ✅ 同等   | seedfill_gray_inv()                       | -                                      |
| seedfillGrayLow             | 🚫 不要   | -                                         | Low-level内部関数: 高レベルAPIでカバー |
| seedfillGrayInvLow          | 🚫 不要   | -                                         | Low-level内部関数: 高レベルAPIでカバー |
| pixSeedfillGraySimple       | ✅ 同等   | seedfill.rs seedfill_gray_simple()        | -                                      |
| pixSeedfillGrayInvSimple    | ✅ 同等   | seedfill.rs seedfill_gray_inv_simple()    | -                                      |
| seedfillGrayLowSimple       | 🚫 不要   | -                                         | Low-level内部関数: 高レベルAPIでカバー |
| seedfillGrayInvLowSimple    | 🚫 不要   | -                                         | Low-level内部関数: 高レベルAPIでカバー |
| pixSeedfillGrayBasin        | ✅ 同等   | seedfill.rs seedfill_gray_basin()         | -                                      |
| pixDistanceFunction         | ✅ 同等   | distance_function()                       | Chamfer距離変換                        |
| distanceFunctionLow         | 🚫 不要   | -                                         | Low-level内部関数: 高レベルAPIでカバー |
| pixSeedspread               | ✅ 同等   | seedfill::seedspread()                    | Voronoiライクなシード拡散              |
| seedspreadLow               | 🚫 不要   | -                                         | Low-level内部関数: 高レベルAPIでカバー |
| pixLocalExtrema             | ✅ 同等   | seedfill.rs local_extrema()               | -                                      |
| pixQualifyLocalMinima       | ✅ 同等   | seedfill.rs qualify_local_minima()        | -                                      |
| pixSelectedLocalExtrema     | ✅ 同等   | seedfill.rs selected_local_extrema()      | -                                      |
| pixFindEqualValues          | ✅ 同等   | find_equal_values()                       | -                                      |
| pixSelectMinInConnComp      | ✅ 同等   | seedfill::select_min_in_conncomp()        | 連結成分内最小値検出                   |
| pixRemoveSeededComponents   | ✅ 同等   | remove_seeded_components()                | -                                      |

### watershed.c

| C関数             | 状態      | Rust対応                               | 備考                                        |
| ----------------- | --------- | -------------------------------------- | ------------------------------------------- |
| wshedCreate       | 🚫 不要   | -                                      | C構造体管理: RustではWatershedOptionsを使用 |
| wshedDestroy      | 🚫 不要   | -                                      | C構造体管理: RustではDropで自動解放         |
| wshedApply        | 🔄 異なる | watershed_segmentation                 | 異なるAPI: WatershedOptionsを使用           |
| wshedBasins       | 🔄 異なる | find_basins                            | 異なるアルゴリズム                          |
| wshedRenderFill   | ✅ 同等   | watershed.rs watershed_render_fill()   | -                                           |
| wshedRenderColors | ✅ 同等   | watershed.rs watershed_render_colors() | -                                           |

### pixlabel.c

| C関数                      | 状態    | Rust対応                         | 備考 |
| -------------------------- | ------- | -------------------------------- | ---- |
| pixConnCompTransform       | ✅ 同等 | conn_comp_transform              | -    |
| pixConnCompAreaTransform   | ✅ 同等 | component_area_transform         | -    |
| pixConnCompIncrInit        | ✅ 同等 | `pix_conn_comp_incr_init`        |      |
| pixConnCompIncrAdd         | ✅ 同等 | `pix_conn_comp_incr_add`         |      |
| pixGetSortedNeighborValues | ✅ 同等 | pix_get_sorted_neighbor_values() | -    |
| pixLocToColorTransform     | ✅ 同等 | `pix_loc_to_color_transform`     |      |

### quadtree.c

| C関数                  | 状態    | Rust対応                     | 備考                          |
| ---------------------- | ------- | ---------------------------- | ----------------------------- |
| pixQuadtreeMean        | ✅ 同等 | quadtree_mean                | -                             |
| pixQuadtreeVariance    | ✅ 同等 | quadtree_variance            | -                             |
| pixMeanInRectangle     | ✅ 同等 | mean_in_rectangle            | -                             |
| pixVarianceInRectangle | ✅ 同等 | variance_in_rectangle        | -                             |
| boxaaQuadtreeRegions   | ✅ 同等 | quadtree_regions             | -                             |
| quadtreeGetParent      | ✅ 同等 | QuadtreeResult::get_parent   | -                             |
| quadtreeGetChildren    | ✅ 同等 | QuadtreeResult::get_children | -                             |
| quadtreeMaxLevels      | ✅ 同等 | quadtree_max_levels          | -                             |
| fpixaDisplayQuadtree   | 🚫 不要 | -                            | 表示/可視化関数: Rustでは不要 |

### maze.c

| C関数               | 状態    | Rust対応                   | 備考 |
| ------------------- | ------- | -------------------------- | ---- |
| generateBinaryMaze  | ✅ 同等 | generate_binary_maze       | -    |
| pixSearchBinaryMaze | ✅ 同等 | search_binary_maze         | -    |
| pixSearchGrayMaze   | ✅ 同等 | maze.rs search_gray_maze() | -    |

## 注記

### 実装方針の違い

1. **Connected Components (conncomp.c)**
   - C版: BOXA/PIXAベースの返却値
   - Rust版: Union-FindアルゴリズムでVec<ConnectedComponent>を返す、より汎用的なAPI

2. **Border Tracing (ccbord.c)**
   - C版: CCBORDAデータ構造とチェインコード
   - Rust版: 単純化されたBorder/ImageBorders構造体、チェインコードは部分的に実装

3. **Seedfill (seedfill.c)**
   - C版: Heckbertのstack-basedアルゴリズム
   - Rust版: Queue-based BFSアルゴリズム、より直感的な実装

4. **Watershed (watershed.c)**
   - C版: 複雑なマーカー管理とLUT
   - Rust版: 簡略化されたpriority queue-basedアルゴリズム

5. **Quadtree (quadtree.c)**
   - C版/Rust版: ほぼ同等の実装、integral imageを使用したO(1)統計計算

6. **Maze (maze.c)**
   - C版/Rust版: 同等のアルゴリズム、BFS-based生成と探索

### 実装完了（元未実装 → 全て実装済み）

全ての未実装関数が実装された:

- **ccbord.c**: チェインコード生成、シリアライゼーション、SVG出力、境界抽出 — 実装済み
- **conncomp.c**: Seedfill BB系関数 — 実装済み
- **pixlabel.c**: インクリメンタル結合、色変換 — 実装済み

### 🚫 不要と判定した関数群

- **Cメモリ/データ構造管理**: ccbaCreate/Destroy, ccbCreate/Destroy, ccbaAddCcb, ccbaExtendArray, ccbaGetCount, ccbaGetCcb, wshedCreate/Destroy
- **Low-level内部関数**: seedfillBinaryLow, seedfillGrayLow, seedfillGrayInvLow, seedfillGrayLowSimple, seedfillGrayInvLowSimple, distanceFunctionLow, seedspreadLow
- **表示/可視化関数**: ccbaDisplayBorder, ccbaDisplaySPBorder, ccbaDisplayImage1, ccbaDisplayImage2, fpixaDisplayQuadtree

### Rust版の追加機能

- **label.rs**: 汎用的なラベリングAPI、統計計算 (get_component_stats)
- **seedfill.rs**: floodfill関数 (in-place変更)
- **watershed.rs**: find_local_minima/maxima、compute_gradient
- **quadtree.rs**: IntegralImage/SquaredIntegralImage型、QuadtreeResult型

## カバレッジ分析

### ファイル別実装率

| ファイル    | 実装済 | 未実装 | 不要 | 実装率 |
| ----------- | ------ | ------ | ---- | ------ |
| conncomp.c  | 11     | 0      | 0    | 100.0% |
| ccbord.c    | 20     | 0      | 12   | 100.0% |
| seedfill.c  | 21     | 0      | 7    | 100.0% |
| watershed.c | 4      | 0      | 2    | 100.0% |
| pixlabel.c  | 6      | 0      | 0    | 100.0% |
| quadtree.c  | 8      | 0      | 1    | 100.0% |
| maze.c      | 3      | 0      | 0    | 100.0% |

### 全体

- 実装済: 65関数 (68.4%)
- 部分実装/異なるAPI: 8関数 (8.4%)
- 未実装: 0関数 (0%)
- 不要: 22関数 (23.2%)

### 推奨される次の実装項目

全関数の実装が完了。🚫不要を除く実カバレッジは100%に達した。
