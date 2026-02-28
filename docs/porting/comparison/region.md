# leptonica (src/region/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## サマリー

| 項目            | 数 |
| --------------- | -- |
| ✅ 同等         | 65 |
| 🔄 異なる       | 8  |
| 実装済（✅+🔄） | 73 |
| ❌ 未実装       | 0  |
| 🚫 不要         | 22 |
| 合計            | 95 |

## 詳細

### conncomp.c

#### region/conncomp.rs (conncomp.c)

| C関数               | 状態 | Rust対応                  | 備考                                                      |
| ------------------- | ---- | ------------------------- | --------------------------------------------------------- |
| pixConnComp         | 🔄   | find_connected_components | 異なるAPI: Rust版はVec<ConnectedComponent>を返す          |
| pixConnCompPixa     | ✅   | conncomp_pixa()           | -                                                         |
| pixConnCompBB       | 🔄   | find_connected_components | 異なるAPI: bounding box情報はConnectedComponentに含まれる |
| pixSeedfillBB       | ✅   | conncomp::seedfill_bb     |                                                           |
| pixSeedfill4BB      | ✅   | conncomp::seedfill_4_bb   |                                                           |
| pixSeedfill8BB      | ✅   | conncomp::seedfill_8_bb   |                                                           |
| pixSeedfill         | ✅   | conncomp::seedfill        |                                                           |
| pixSeedfill4        | ✅   | conncomp::seedfill_4      |                                                           |
| pixSeedfill8        | ✅   | conncomp::seedfill_8      |                                                           |
| nextOnPixelInRaster | ✅   | next_on_pixel_in_raster() | -                                                         |

#### region/label.rs (conncomp.c)

| C関数            | 状態 | Rust対応                        | 備考 |
| ---------------- | ---- | ------------------------------- | ---- |
| pixCountConnComp | ✅   | pix_count_components (label.rs) | -    |

### ccbord.c

#### region/ccbord.rs (ccbord.c)

| C関数                     | 状態 | Rust対応                          | 備考                                    |
| ------------------------- | ---- | --------------------------------- | --------------------------------------- |
| ccbaCreate                | 🚫   | -                                 | Cメモリ管理: Rustでは不要               |
| ccbaDestroy               | 🚫   | -                                 | Cメモリ管理: Rustでは不要               |
| ccbCreate                 | 🚫   | -                                 | Cメモリ管理: Rustでは不要               |
| ccbDestroy                | 🚫   | -                                 | Cメモリ管理: Rustでは不要               |
| ccbaAddCcb                | 🚫   | -                                 | Cデータ構造管理: Rustでは不要           |
| ccbaExtendArray           | 🚫   | -                                 | Cデータ構造管理: Rustでは不要           |
| ccbaGetCount              | 🚫   | -                                 | Cデータ構造管理: Rustでは不要           |
| ccbaGetCcb                | 🚫   | -                                 | Cデータ構造管理: Rustでは不要           |
| pixGetAllCCBorders        | 🔄   | get_all_borders                   | 異なるAPI: ImageBordersを返す           |
| pixGetCCBorders           | ✅   | get_component_borders             |                                         |
| pixGetOuterBordersPtaa    | 🔄   | get_outer_borders                 | 異なるAPI: Vec<Border>を返す            |
| pixGetOuterBorderPta      | 🔄   | get_outer_border                  | 異なるAPI: Borderを返す                 |
| pixGetOuterBorder         | ✅   | get_outer_border                  |                                         |
| pixGetHoleBorder          | ✅   | pix_get_hole_border               |                                         |
| findNextBorderPixel       | ✅   | find_next_border_pixel (private)  | -                                       |
| locateOutsideSeedPixel    | ✅   | locate_outside_seed_pixel         |                                         |
| ccbaGenerateGlobalLocs    | ✅   | ccbord::generate_global_locs      |                                         |
| ccbaGenerateStepChains    | ✅   | ccbord::generate_step_chains      |                                         |
| ccbaStepChainsToPixCoords | ✅   | ccbord::step_chains_to_pix_coords |                                         |
| ccbaGenerateSPGlobalLocs  | ✅   | ccbord::generate_sp_global_locs   |                                         |
| ccbaGenerateSinglePath    | ✅   | ccbord::generate_single_path      |                                         |
| getCutPathForHole         | ✅   | get_cut_path_for_hole             |                                         |
| ccbaDisplayBorder         | 🚫   | -                                 | `PIX*`レンダリング関数（専用API未提供） |
| ccbaDisplaySPBorder       | 🚫   | -                                 | `PIX*`レンダリング関数（専用API未提供） |
| ccbaDisplayImage1         | 🚫   | -                                 | `PIX*`レンダリング関数（専用API未提供） |
| ccbaDisplayImage2         | 🚫   | -                                 | `PIX*`レンダリング関数（専用API未提供） |
| ccbaWrite                 | ✅   | ccbord::write_to_file             |                                         |
| ccbaWriteStream           | ✅   | ccbord::write<W: Write>           |                                         |
| ccbaRead                  | ✅   | ccbord::read_from_file            |                                         |
| ccbaReadStream            | ✅   | ccbord::read_from<R: Read>        |                                         |
| ccbaWriteSVG              | ✅   | ccbord::write_svg                 |                                         |
| ccbaWriteSVGString        | ✅   | ccbord::to_svg_string             |                                         |

### seedfill.c

#### region/seedfill.rs (seedfill.c)

| C関数                       | 状態 | Rust対応                           | 備考                                                      |
| --------------------------- | ---- | ---------------------------------- | --------------------------------------------------------- |
| pixSeedfillBinary           | 🔄   | seedfill_binary                    | C版の形態学的再構成と異なり、Rust版は座標ベースflood fill |
| pixSeedfillBinaryRestricted | ✅   | seedfill_binary_restricted()       | -                                                         |
| pixHolesByFilling           | ✅   | fill_holes                         | -                                                         |
| pixFillClosedBorders        | ✅   | fill_closed_borders()              | -                                                         |
| pixRemoveBorderConnComps    | ✅   | clear_border                       | -                                                         |
| pixSeedfillGray             | ✅   | seedfill_gray                      | -                                                         |
| pixSeedfillGrayInv          | ✅   | seedfill_gray_inv()                | -                                                         |
| pixDistanceFunction         | ✅   | distance_function()                | Chamfer距離変換                                           |
| pixSeedspread               | ✅   | seedfill::seedspread()             | Voronoiライクなシード拡散                                 |
| pixFindEqualValues          | ✅   | find_equal_values()                | -                                                         |
| pixSelectMinInConnComp      | ✅   | seedfill::select_min_in_conncomp() | 連結成分内最小値検出                                      |
| pixRemoveSeededComponents   | ✅   | remove_seeded_components()         | -                                                         |
| seedfillBinaryLow           | 🚫   | -                                  | Low-level内部関数: 高レベルAPIでカバー                    |
| seedfillGrayLow             | 🚫   | -                                  | Low-level内部関数: 高レベルAPIでカバー                    |
| seedfillGrayInvLow          | 🚫   | -                                  | Low-level内部関数: 高レベルAPIでカバー                    |
| seedfillGrayLowSimple       | 🚫   | -                                  | Low-level内部関数: 高レベルAPIでカバー                    |
| seedfillGrayInvLowSimple    | 🚫   | -                                  | Low-level内部関数: 高レベルAPIでカバー                    |
| distanceFunctionLow         | 🚫   | -                                  | Low-level内部関数: 高レベルAPIでカバー                    |
| seedspreadLow               | 🚫   | -                                  | Low-level内部関数: 高レベルAPIでカバー                    |
| pixExtractBorderConnComps   | ✅   | extract_border_conn_comps()        | -                                                         |
| pixFillBgFromBorder         | ✅   | fill_bg_from_border()              | -                                                         |
| pixFillHolesToBoundingRect  | ✅   | fill_holes_to_bounding_rect()      | -                                                         |
| pixSeedfillGraySimple       | ✅   | seedfill_gray_simple()             | -                                                         |
| pixSeedfillGrayInvSimple    | ✅   | seedfill_gray_inv_simple()         | -                                                         |
| pixSeedfillGrayBasin        | ✅   | seedfill_gray_basin()              | -                                                         |
| pixLocalExtrema             | ✅   | local_extrema()                    | -                                                         |
| pixQualifyLocalMinima       | ✅   | qualify_local_minima()             | -                                                         |
| pixSelectedLocalExtrema     | ✅   | selected_local_extrema()           | -                                                         |

### watershed.c

#### region/watershed.rs (watershed.c)

| C関数             | 状態 | Rust対応                  | 備考                                        |
| ----------------- | ---- | ------------------------- | ------------------------------------------- |
| wshedCreate       | 🚫   | -                         | C構造体管理: RustではWatershedOptionsを使用 |
| wshedDestroy      | 🚫   | -                         | C構造体管理: RustではDropで自動解放         |
| wshedApply        | 🔄   | watershed_segmentation    | 異なるAPI: WatershedOptionsを使用           |
| wshedBasins       | 🔄   | find_basins               | 異なるアルゴリズム                          |
| wshedRenderFill   | ✅   | watershed_render_fill()   | -                                           |
| wshedRenderColors | ✅   | watershed_render_colors() | -                                           |

### pixlabel.c

#### region/label.rs (pixlabel.c)

| C関数                      | 状態 | Rust対応                         | 備考 |
| -------------------------- | ---- | -------------------------------- | ---- |
| pixConnCompTransform       | ✅   | conn_comp_transform              | -    |
| pixConnCompIncrInit        | ✅   | pix_conn_comp_incr_init          |      |
| pixConnCompIncrAdd         | ✅   | pix_conn_comp_incr_add           |      |
| pixGetSortedNeighborValues | ✅   | pix_get_sorted_neighbor_values() | -    |
| pixLocToColorTransform     | ✅   | pix_loc_to_color_transform       |      |

#### region/conncomp.rs (pixlabel.c)

| C関数                    | 状態 | Rust対応                 | 備考 |
| ------------------------ | ---- | ------------------------ | ---- |
| pixConnCompAreaTransform | ✅   | component_area_transform | -    |

### quadtree.c

#### region/quadtree.rs (quadtree.c)

| C関数                  | 状態 | Rust対応                     | 備考                          |
| ---------------------- | ---- | ---------------------------- | ----------------------------- |
| pixQuadtreeMean        | ✅   | quadtree_mean                | -                             |
| pixQuadtreeVariance    | ✅   | quadtree_variance            | -                             |
| pixMeanInRectangle     | ✅   | mean_in_rectangle            | -                             |
| pixVarianceInRectangle | ✅   | variance_in_rectangle        | -                             |
| boxaaQuadtreeRegions   | ✅   | quadtree_regions             | -                             |
| quadtreeMaxLevels      | ✅   | quadtree_max_levels          | -                             |
| fpixaDisplayQuadtree   | 🚫   | -                            | 表示/可視化関数: Rustでは不要 |
| quadtreeGetParent      | ✅   | QuadtreeResult::get_parent   | -                             |
| quadtreeGetChildren    | ✅   | QuadtreeResult::get_children | -                             |

### maze.c

#### region/maze.rs (maze.c)

| C関数               | 状態 | Rust対応             | 備考 |
| ------------------- | ---- | -------------------- | ---- |
| generateBinaryMaze  | ✅   | generate_binary_maze | -    |
| pixSearchBinaryMaze | ✅   | search_binary_maze   | -    |
| pixSearchGrayMaze   | ✅   | search_gray_maze()   | -    |

## 注記

### 実装方針の違い

1. **Connected Components (conncomp.c)**
   - C版: BOXA/PIXAベースの返却値
   - Rust版: Union-FindアルゴリズムでVec<ConnectedComponent>を返す、より汎用的なAPI

2. **Border Tracing (ccbord.c)**
   - C版: CCBORDAデータ構造とチェインコード
   - Rust版: 単純化されたBorder/ImageBorders構造体、チェインコードは部分的に実装

3. **Seedfill (seedfill.c)**
   - C版: seed画像+maskを使った形態学的再構成（`pixSeedfillBinary`）
   - Rust版: Queue-based BFSの座標伝播（API/アルゴリズムともに差異あり）

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

- ✅ 同等: 65関数 (68.4%)
- 🔄 異なるAPI/アルゴリズム: 8関数 (8.4%)
- 実装済（✅+🔄）: 73関数 (76.8%)
- 未実装: 0関数 (0%)
- 不要: 22関数 (23.2%)

### 推奨される次の実装項目

全関数の実装が完了。🚫不要を除く実カバレッジは100%に達した。
