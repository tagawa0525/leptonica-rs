# leptonica-region: C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 33 |
| 🔄 異なる | 8 |
| ❌ 未実装 | 54 |
| 合計 | 95 |

## 詳細

### conncomp.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixConnComp | 🔄 異なる | find_connected_components | 異なるAPI: Rust版はVec<ConnectedComponent>を返す |
| pixConnCompPixa | ✅ 同等 | conncomp_pixa() | - |
| pixConnCompBB | 🔄 異なる | find_connected_components | 異なるAPI: bounding box情報はConnectedComponentに含まれる |
| pixCountConnComp | ✅ 同等 | pix_count_components (label.rs) | - |
| nextOnPixelInRaster | ❌ 未実装 | - | - |
| pixSeedfillBB | ❌ 未実装 | - | - |
| pixSeedfill4BB | ❌ 未実装 | - | - |
| pixSeedfill8BB | ❌ 未実装 | - | - |
| pixSeedfill | ❌ 未実装 | - | - |
| pixSeedfill4 | ❌ 未実装 | - | - |
| pixSeedfill8 | ❌ 未実装 | - | - |

### ccbord.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| ccbaCreate | ❌ 未実装 | - | CCBORDAデータ構造未実装 |
| ccbaDestroy | ❌ 未実装 | - | - |
| ccbCreate | ❌ 未実装 | - | - |
| ccbDestroy | ❌ 未実装 | - | - |
| ccbaAddCcb | ❌ 未実装 | - | - |
| ccbaExtendArray | ❌ 未実装 | - | - |
| ccbaGetCount | ❌ 未実装 | - | - |
| ccbaGetCcb | ❌ 未実装 | - | - |
| pixGetAllCCBorders | 🔄 異なる | get_all_borders | 異なるAPI: ImageBordersを返す |
| pixGetCCBorders | ❌ 未実装 | - | - |
| pixGetOuterBordersPtaa | 🔄 異なる | get_outer_borders | 異なるAPI: Vec<Border>を返す |
| pixGetOuterBorderPta | 🔄 異なる | get_outer_border | 異なるAPI: Borderを返す |
| pixGetOuterBorder | ❌ 未実装 | - | - |
| pixGetHoleBorder | ❌ 未実装 | - | - |
| findNextBorderPixel | ✅ 同等 | find_next_border_pixel (private) | - |
| locateOutsideSeedPixel | ❌ 未実装 | - | - |
| ccbaGenerateGlobalLocs | ❌ 未実装 | - | - |
| ccbaGenerateStepChains | ❌ 未実装 | - | - |
| ccbaStepChainsToPixCoords | ❌ 未実装 | - | - |
| ccbaGenerateSPGlobalLocs | ❌ 未実装 | - | - |
| ccbaGenerateSinglePath | ❌ 未実装 | - | - |
| getCutPathForHole | ❌ 未実装 | - | - |
| ccbaDisplayBorder | ❌ 未実装 | - | - |
| ccbaDisplaySPBorder | ❌ 未実装 | - | - |
| ccbaDisplayImage1 | ❌ 未実装 | - | - |
| ccbaDisplayImage2 | ❌ 未実装 | - | - |
| ccbaWrite | ❌ 未実装 | - | - |
| ccbaWriteStream | ❌ 未実装 | - | - |
| ccbaRead | ❌ 未実装 | - | - |
| ccbaReadStream | ❌ 未実装 | - | - |
| ccbaWriteSVG | ❌ 未実装 | - | - |
| ccbaWriteSVGString | ❌ 未実装 | - | - |

### seedfill.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixSeedfillBinary | 🔄 異なる | seedfill_binary | 異なるAPI: SeedFillOptionsを使用 |
| pixSeedfillBinaryRestricted | ✅ 同等 | seedfill_binary_restricted() | - |
| seedfillBinaryLow | ❌ 未実装 | - | Low-level関数 |
| pixHolesByFilling | ✅ 同等 | fill_holes | - |
| pixFillClosedBorders | ✅ 同等 | fill_closed_borders() | - |
| pixExtractBorderConnComps | ❌ 未実装 | - | - |
| pixRemoveBorderConnComps | ✅ 同等 | clear_border | - |
| pixFillBgFromBorder | ❌ 未実装 | - | - |
| pixFillHolesToBoundingRect | ❌ 未実装 | - | - |
| pixSeedfillGray | ✅ 同等 | seedfill_gray | - |
| pixSeedfillGrayInv | ✅ 同等 | seedfill_gray_inv() | - |
| seedfillGrayLow | ❌ 未実装 | - | Low-level関数 |
| seedfillGrayInvLow | ❌ 未実装 | - | Low-level関数 |
| pixSeedfillGraySimple | ✅ 同等 | seedfill.rs seedfill_gray_simple() | - |
| pixSeedfillGrayInvSimple | ✅ 同等 | seedfill.rs seedfill_gray_inv_simple() | - |
| seedfillGrayLowSimple | ❌ 未実装 | - | Low-level関数 |
| seedfillGrayInvLowSimple | ❌ 未実装 | - | Low-level関数 |
| pixSeedfillGrayBasin | ✅ 同等 | seedfill.rs seedfill_gray_basin() | - |
| pixDistanceFunction | ✅ 同等 | distance_function() | Chamfer距離変換 |
| distanceFunctionLow | ❌ 未実装 | - | Low-level関数 |
| pixSeedspread | ✅ 同等 | seedfill::seedspread() | Voronoiライクなシード拡散 |
| seedspreadLow | ❌ 未実装 | - | Low-level関数 |
| pixLocalExtrema | ✅ 同等 | seedfill.rs local_extrema() | - |
| pixQualifyLocalMinima | ✅ 同等 | seedfill.rs qualify_local_minima() | - |
| pixSelectedLocalExtrema | ✅ 同等 | seedfill.rs selected_local_extrema() | - |
| pixFindEqualValues | ✅ 同等 | find_equal_values() | - |
| pixSelectMinInConnComp | ✅ 同等 | seedfill::select_min_in_conncomp() | 連結成分内最小値検出 |
| pixRemoveSeededComponents | ✅ 同等 | remove_seeded_components() | - |

### watershed.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| wshedCreate | ❌ 未実装 | - | L_WSHED構造体未実装 |
| wshedDestroy | ❌ 未実装 | - | - |
| wshedApply | 🔄 異なる | watershed_segmentation | 異なるAPI: WatershedOptionsを使用 |
| wshedBasins | 🔄 異なる | find_basins | 異なるアルゴリズム |
| wshedRenderFill | ❌ 未実装 | - | - |
| wshedRenderColors | ❌ 未実装 | - | - |

### pixlabel.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixConnCompTransform | ✅ 同等 | label_connected_components | - |
| pixConnCompAreaTransform | ✅ 同等 | component_area_transform | - |
| pixConnCompIncrInit | ❌ 未実装 | - | - |
| pixConnCompIncrAdd | ❌ 未実装 | - | - |
| pixGetSortedNeighborValues | ✅ 同等 | get_sorted_neighbor_values() | - |
| pixLocToColorTransform | ❌ 未実装 | - | - |

### quadtree.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixQuadtreeMean | ✅ 同等 | quadtree_mean | - |
| pixQuadtreeVariance | ✅ 同等 | quadtree_variance | - |
| pixMeanInRectangle | ✅ 同等 | mean_in_rectangle | - |
| pixVarianceInRectangle | ✅ 同等 | variance_in_rectangle | - |
| boxaaQuadtreeRegions | ✅ 同等 | quadtree_regions | - |
| quadtreeGetParent | ✅ 同等 | QuadtreeResult::get_parent | - |
| quadtreeGetChildren | ✅ 同等 | QuadtreeResult::get_children | - |
| quadtreeMaxLevels | ✅ 同等 | quadtree_max_levels | - |
| fpixaDisplayQuadtree | ❌ 未実装 | - | - |

### maze.c
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| generateBinaryMaze | ✅ 同等 | generate_binary_maze | - |
| pixSearchBinaryMaze | ✅ 同等 | search_binary_maze | - |
| pixSearchGrayMaze | ❌ 未実装 | - | - |

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

### 未実装の主要機能

- **ccbord.c**: CCBORDAデータ構造全体、シリアライゼーション、SVG出力
- **seedfill.c**: 残りは Low-level 関数（seedfillGrayLow 等）と border 系
- **watershed.c**: L_WSHEDデータ構造、レンダリング関数
- **pixlabel.c**: インクリメンタル結合、色変換

### Rust版の追加機能

- **label.rs**: 汎用的なラベリングAPI、統計計算 (get_component_stats)
- **seedfill.rs**: floodfill関数 (in-place変更)
- **watershed.rs**: find_local_minima/maxima、compute_gradient
- **quadtree.rs**: IntegralImage/SquaredIntegralImage型、QuadtreeResult型

## カバレッジ分析

### ファイル別実装率

| ファイル | 実装済 | 未実装 | 実装率 |
|---------|--------|--------|--------|
| conncomp.c | 4 | 7 | 36.4% |
| ccbord.c | 4 | 28 | 12.5% |
| seedfill.c | 18 | 10 | 64.3% |
| watershed.c | 2 | 4 | 33.3% |
| pixlabel.c | 3 | 3 | 50.0% |
| quadtree.c | 8 | 1 | 88.9% |
| maze.c | 2 | 1 | 66.7% |

### 全体

- 実装済: 33関数 (34.7%)
- 部分実装/異なるAPI: 8関数 (8.4%)
- 未実装: 54関数 (56.8%)

### 推奨される次の実装項目

1. **高優先度**:
   - pixSeedfillBB系関数 (conncomp.c) - 既存コードとの互換性

2. **中優先度**:
   - CCBORDA構造体とシリアライゼーション (ccbord.c)
   - pixSearchGrayMaze (maze.c) - 一般化された経路探索

3. **低優先度**:
   - 可視化関数 (fpixaDisplayQuadtree等)
   - インクリメンタル結合 (pixConnCompIncrInit/Add)
