# leptonica (src/morph/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（301_morph-full-porting全Phase完了を反映）

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 108 |
| 🔄 異なる | 16 |
| ❌ 未実装 | 0 |
| 🚫 不要 | 26 |
| 合計 | 150 |

## 詳細

### morph.c (基本形態学演算)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixDilate | ✅ 同等 | binary::dilate | |
| pixErode | ✅ 同等 | binary::erode | |
| pixOpen | ✅ 同等 | binary::open | |
| pixClose | ✅ 同等 | binary::close | |
| pixCloseSafe | ✅ 同等 | binary::close_safe | Phase 1で実装 |
| pixOpenGeneralized | ✅ 同等 | binary::open_generalized | Phase 1で実装 |
| pixCloseGeneralized | ✅ 同等 | binary::close_generalized | Phase 1で実装 |
| pixDilateBrick | ✅ 同等 | binary::dilate_brick | |
| pixErodeBrick | ✅ 同等 | binary::erode_brick | |
| pixOpenBrick | ✅ 同等 | binary::open_brick | |
| pixCloseBrick | ✅ 同等 | binary::close_brick | |
| pixCloseSafeBrick | ✅ 同等 | binary::close_safe_brick | Phase 1で実装 |
| pixDilateCompBrick | 🔄 異なる | binary::dilate_brick | Rust版は分離可能分解を自動選択 |
| pixErodeCompBrick | 🔄 異なる | binary::erode_brick | Rust版は分離可能分解を自動選択 |
| pixOpenCompBrick | 🔄 異なる | binary::open_brick | Rust版は分離可能分解を自動選択 |
| pixCloseCompBrick | 🔄 異なる | binary::close_brick | Rust版は分離可能分解を自動選択 |
| pixCloseSafeCompBrick | ✅ 同等 | binary::close_safe_comp_brick | Phase 1で実装 |
| resetMorphBoundaryCondition | 🚫 不要 | - | C版グローバル変数setter（Rustでは引数で明示指定） |
| getMorphBorderPixelColor | 🚫 不要 | - | C版グローバル状態アクセサ（Rustでは引数で明示指定） |

### morphapp.c (応用演算)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixMorphGradient | ✅ 同等 | morphapp::morph_gradient | Phase 2で実装 |
| pixExtractBoundary | ✅ 同等 | binary::extract_boundary | Phase 2で実装 |
| pixMorphSequenceMasked | ✅ 同等 | morphapp::morph_sequence_masked | Phase 2で実装 |
| pixUnionOfMorphOps | ✅ 同等 | morphapp::union_of_morph_ops | Phase 2で実装 |
| pixIntersectionOfMorphOps | ✅ 同等 | morphapp::intersection_of_morph_ops | Phase 2で実装 |
| pixSeedfillMorph | ✅ 同等 | morphapp::seedfill_morph | Phase 2で実装 |
| pixMorphSequenceByComponent | ✅ 同等 | `morph_sequence_by_component` | 連結成分ごとの処理 |
| pixMorphSequenceByRegion | ✅ 同等 | `morph_sequence_by_region` | 領域ごとの処理 |
| pixTophat | ✅ 同等 | binary::top_hat | バイナリ版top-hat（graymorph.cのpixTophatとは別関数） |
| pixHMT | ✅ 同等 | binary::hit_miss_transform | |
| pixGrayscaleMorphSum | 🚫 不要 | - | C版leptonicaに存在しない関数 |
| pixMultiplyByColor | 🚫 不要 | - | blend.c所属（morph領域外） |
| pixHMTDwa | 🚫 不要 | - | DWA自動生成コード |
| pixFHMTGen | 🚫 不要 | - | DWAコード生成 |

### morphdwa.c (DWA実装)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixDilateBrickDwa | ✅ 同等 | dwa::dilate_brick_dwa | |
| pixErodeBrickDwa | ✅ 同等 | dwa::erode_brick_dwa | |
| pixOpenBrickDwa | ✅ 同等 | dwa::open_brick_dwa | |
| pixCloseBrickDwa | ✅ 同等 | dwa::close_brick_dwa | |
| pixDilateCompBrickDwa | ✅ 同等 | dwa::dilate_comp_brick_dwa | Phase 5で実装 |
| pixErodeCompBrickDwa | ✅ 同等 | dwa::erode_comp_brick_dwa | Phase 5で実装 |
| pixOpenCompBrickDwa | ✅ 同等 | dwa::open_comp_brick_dwa | Phase 5で実装 |
| pixCloseCompBrickDwa | ✅ 同等 | dwa::close_comp_brick_dwa | Phase 5で実装 |
| pixDilateCompBrickExtendDwa | ✅ 同等 | dwa::dilate_comp_brick_extend_dwa | Phase 5で実装 |
| pixErodeCompBrickExtendDwa | ✅ 同等 | dwa::erode_comp_brick_extend_dwa | Phase 5で実装 |
| pixOpenCompBrickExtendDwa | ✅ 同等 | dwa::open_comp_brick_extend_dwa | Phase 5で実装 |
| pixCloseCompBrickExtendDwa | ✅ 同等 | dwa::close_comp_brick_extend_dwa | Phase 5で実装 |
| getExtendedCompositeParameters | ✅ 同等 | dwa::get_extended_composite_parameters | Phase 5で実装 |
| makeLinearBrickDwaGen | 🚫 不要 | - | DWAコード生成（Rustでは不要） |
| makeLinearBrickDwa | 🚫 不要 | - | DWAコード生成（Rustでは不要） |
| pixMorphDwa_*系 | 🚫 不要 | - | DWA自動生成関数（Rustでは手書き実装） |

### morphseq.c (シーケンス処理)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixMorphSequence | ✅ 同等 | sequence::morph_sequence | |
| pixMorphCompSequence | ✅ 同等 | sequence::morph_comp_sequence | |
| pixMorphSequenceDwa | ✅ 同等 | sequence::morph_sequence_dwa | Phase 5で実装 |
| pixMorphCompSequenceDwa | ✅ 同等 | sequence::morph_comp_sequence_dwa | Phase 5で実装 |
| morphSequenceVerify | 🔄 異なる | sequence内部で検証 | 公開APIとしては未実装 |
| pixGrayMorphSequence | ✅ 同等 | sequence::gray_morph_sequence | |
| pixColorMorphSequence | ✅ 同等 | sequence::color_morph_sequence | Phase 5で実装 |

### graymorph.c (グレースケール形態学)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixErodeGray | ✅ 同等 | grayscale::erode_gray | |
| pixDilateGray | ✅ 同等 | grayscale::dilate_gray | |
| pixOpenGray | ✅ 同等 | grayscale::open_gray | |
| pixCloseGray | ✅ 同等 | grayscale::close_gray | |
| pixErodeGray3 | 🔄 異なる | grayscale::erode_gray | erode_gray() が 3x3 で fast path にディスパッチ |
| pixDilateGray3 | 🔄 異なる | grayscale::dilate_gray | dilate_gray() が 3x3 で fast path にディスパッチ |
| pixOpenGray3 | 🔄 異なる | grayscale::open_gray | open_gray() が 3x3 で fast path にディスパッチ |
| pixCloseGray3 | 🔄 異なる | grayscale::close_gray | close_gray() が 3x3 で fast path にディスパッチ |
| dilateGrayLow | 🚫 不要 | - | 低レベル内部関数（高レベルAPIで対応済み） |
| erodeGrayLow | 🚫 不要 | - | 低レベル内部関数（高レベルAPIで対応済み） |
| pixTophat | ✅ 同等 | grayscale::top_hat_gray | グレースケール版top-hat（morphapp.cのpixTophatとは別関数） |

### colormorph.c (カラー形態学)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixColorMorph | 🔄 異なる | color::dilate_color等 | C版は1関数、Rust版は個別関数 |
| - | ✅ 同等 | color::dilate_color | Rust版で追加 |
| - | ✅ 同等 | color::erode_color | Rust版で追加 |
| - | ✅ 同等 | color::open_color | Rust版で追加 |
| - | ✅ 同等 | color::close_color | Rust版で追加 |
| - | ✅ 同等 | color::gradient_color | Rust版で追加 |
| - | ✅ 同等 | color::top_hat_color | Rust版で追加 |
| - | ✅ 同等 | color::bottom_hat_color | Rust版で追加 |

### sel1.c (Sel基本操作)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| selaCreate | ✅ 同等 | sel::Sela::new | Phase 6で実装 |
| selaDestroy | ✅ 同等 | Drop trait | Rust自動メモリ管理 |
| selCreate | ✅ 同等 | sel::Sel::new | |
| selDestroy | ✅ 同等 | Drop trait | |
| selCopy | ✅ 同等 | Clone trait | |
| selCreateBrick | ✅ 同等 | sel::Sel::create_brick | |
| selCreateComb | ✅ 同等 | DWA内部で使用 | |
| create2dIntArray | 🔄 異なる | Vec<Vec<>> | Rustでは不要 |
| selaAddSel | ✅ 同等 | sel::Sela::add | Phase 6で実装 |
| selaGetCount | ✅ 同等 | sel::Sela::count | Phase 6で実装 |
| selaGetSel | ✅ 同等 | sel::Sela::get | Phase 6で実装 |
| selGetName | ✅ 同等 | sel::Sel::name() | |
| selSetName | ✅ 同等 | sel::Sel::set_name() | |
| selaFindSelByName | ✅ 同等 | sel::Sela::find_by_name | Phase 6で実装 |
| selGetElement | ✅ 同等 | sel::Sel::get_element | |
| selSetElement | ✅ 同等 | sel::Sel::set_element | |
| selGetParameters | ✅ 同等 | sel::Sel::get_parameters | Phase 3で実装 |
| selSetOrigin | ✅ 同等 | sel::Sel::set_origin | |
| selGetTypeAtOrigin | ✅ 同等 | get_elementでorigin参照 | |
| selaGetBrickName | 🔄 異なる | Sela::find_by_name | 命名規則で検索 |
| selaGetCombName | 🔄 異なる | Sela::find_by_name | 命名規則で検索 |
| getCompositeParameters | ✅ 同等 | dwa内部 + get_extended_composite_parameters | |
| selaGetSelnames | 🔄 異なる | iterate + name() | |
| selFindMaxTranslations | ✅ 同等 | sel::Sel::find_max_translations | |
| selRotateOrth | ✅ 同等 | sel::Sel::rotate_orth | |
| selaRead | ✅ 同等 | sel::Sela::read | Phase 6で実装 |
| selaReadStream | ✅ 同等 | sel::Sela::read_from_reader | Phase 6で実装 |
| selRead | ✅ 同等 | sel::Sel::read | Phase 3で実装 |
| selReadStream | ✅ 同等 | sel::Sel::read_from_reader | Phase 3で実装 |
| selaWrite | ✅ 同等 | sel::Sela::write | Phase 6で実装 |
| selaWriteStream | ✅ 同等 | sel::Sela::write_to_writer | Phase 6で実装 |
| selWrite | ✅ 同等 | sel::Sel::write | Phase 3で実装 |
| selWriteStream | ✅ 同等 | sel::Sel::write_to_writer | Phase 3で実装 |
| selCreateFromString | ✅ 同等 | sel::Sel::from_string | |
| selPrintToString | ✅ 同等 | sel::Sel::print_to_string | Phase 3で実装 |
| selaCreateFromFile | ✅ 同等 | sel::Sela::read | Phase 6で実装 |
| selCreateFromPta | ✅ 同等 | sel::Sel::from_pta | Phase 3で実装 |
| selCreateFromPix | ✅ 同等 | sel::Sel::from_pix | Phase 3で実装 |
| selReadFromColorImage | ✅ 同等 | sel::Sel::from_color_image | Phase 3で実装 |
| selCreateFromColorPix | ✅ 同等 | sel::Sel::from_color_image | Phase 3で実装 |
| selaCreateFromColorPixa | ✅ 同等 | `sela_create_from_color_pixa` | Pixa操作はアプリケーション層 |
| selDisplayInPix | 🚫 不要 | - | 可視化専用 |
| selaDisplayInPix | 🚫 不要 | - | 可視化専用 |

### sel2.c (Sel定義済みセット)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| sel4ccThin系 (16関数) | 🔄 異なる | thin_sels::sels_4cc_thin | 一括生成で対応 |
| sel8ccThin系 (16関数) | 🔄 異なる | thin_sels::sels_8cc_thin | 一括生成で対応 |
| selaMakeThinSets | ✅ 同等 | thin_sels::make_thin_sels | |
| selaAddBasic | ✅ 同等 | sel::sela_add_basic | Phase 4で実装 |
| selaAddHitMiss | ✅ 同等 | sel::sela_add_hit_miss | Phase 4で実装 |
| selaAddDwaLinear | ✅ 同等 | sel::sela_add_dwa_linear | Phase 4で実装 |
| selaAddDwaCombs | ✅ 同等 | sel::sela_add_dwa_combs | Phase 4で実装 |
| selaAddCrossJunctions | ✅ 同等 | sel::sela_add_cross_junctions | Phase 4で実装 |
| selaAddTJunctions | ✅ 同等 | sel::sela_add_t_junctions | Phase 4で実装 |

### selgen.c (Sel自動生成)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixGenerateSelBoundary | ✅ 同等 | `generate_sel_boundary` |  |
| pixGenerateSelWithRuns | ✅ 同等 | `generate_sel_with_runs` |  |
| pixGenerateSelRandom | ✅ 同等 | `generate_sel_random` |  |
| pixGetRunCentersOnLine | ✅ 同等 | `get_run_centers_on_line` |  |
| pixGetRunsOnLine | ✅ 同等 | `get_runs_on_line` |  |
| pixSubsampleBoundaryPixels | ✅ 同等 | `subsample_boundary_pixels` |  |
| adjacentOnPixelInRaster | 🚫 不要 | - | 低レベル内部ヘルパー（pixSubsampleBoundaryPixels内部で使用） |
| pixDisplayHitMissSel | 🚫 不要 | - | 可視化専用 |

### ccthin.c (連結成分保存細線化)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaThinConnected | ✅ 同等 | `pixa_thin_connected` |  |
| pixThinConnected | ✅ 同等 | thin::thin_connected | |
| pixThinConnectedBySet | ✅ 同等 | thin::thin_connected_by_set | |
| selaMakeThinSets | ✅ 同等 | thin_sels::make_thin_sels | |

### dwacomb.2.c / dwacomblow.2.c / fmorphauto.c / fmorphgen.1.c / fmorphgenlow.1.c / fhmtauto.c / fhmtgen.1.c / fhmtgenlow.1.c (DWAコード生成・自動生成)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| fmorphopgen_low_2 | 🚫 不要 | - | DWAコード生成（Rustでは不要） |
| dwacomblow_low_2 | 🚫 不要 | - | DWA合成低レベル生成（Rustでは不要） |
| fmorphautogen | 🚫 不要 | - | DWAコード自動生成（Rustでは不要） |
| fmorphautogen1 | 🚫 不要 | - | DWAコード自動生成（Rustでは不要） |
| fmorphautogen2 | 🚫 不要 | - | DWAコード自動生成（Rustでは不要） |
| fmorphopgen_low_1 | 🚫 不要 | - | DWAコード生成（Rustでは不要） |
| fhmtautogen | 🚫 不要 | - | HMT DWA自動生成（Rustでは不要） |
| fhmtautogen1 | 🚫 不要 | - | HMT DWA自動生成（Rustでは不要） |
| fhmtautogen2 | 🚫 不要 | - | HMT DWA自動生成（Rustでは不要） |
| fhmtgen_low_1 | 🚫 不要 | - | HMT低レベル生成（Rustでは不要） |
| (低レベル関数群) | 🚫 不要 | - | DWA内部実装（Rustでは手書き実装） |

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
