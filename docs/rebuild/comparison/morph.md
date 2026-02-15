# leptonica-morph: C版 vs Rust版 関数レベル比較

調査日: 2026-02-15

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 34 |
| 🔄 異なる | 8 |
| ❌ 未実装 | 78 |
| 合計 | 120 |

## 詳細

### morph.c (基本形態学演算)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixDilate | ✅ 同等 | binary::dilate | |
| pixErode | ✅ 同等 | binary::erode | |
| pixOpen | ✅ 同等 | binary::open | |
| pixClose | ✅ 同等 | binary::close | |
| pixCloseSafe | ❌ 未実装 | - | Safe closingの個別関数は未実装 |
| pixOpenGeneralized | ❌ 未実装 | - | |
| pixCloseGeneralized | ❌ 未実装 | - | |
| pixDilateBrick | ✅ 同等 | binary::dilate_brick | |
| pixErodeBrick | ✅ 同等 | binary::erode_brick | |
| pixOpenBrick | ✅ 同等 | binary::open_brick | |
| pixCloseBrick | ✅ 同等 | binary::close_brick | |
| pixCloseSafeBrick | ❌ 未実装 | - | |
| pixDilateCompBrick | 🔄 異なる | binary::dilate_brick | Rust版は分離可能分解を自動選択 |
| pixErodeCompBrick | 🔄 異なる | binary::erode_brick | Rust版は分離可能分解を自動選択 |
| pixOpenCompBrick | 🔄 異なる | binary::open_brick | Rust版は分離可能分解を自動選択 |
| pixCloseCompBrick | 🔄 異なる | binary::close_brick | Rust版は分離可能分解を自動選択 |
| pixCloseSafeCompBrick | ❌ 未実装 | - | |
| resetMorphBoundaryCondition | ❌ 未実装 | - | C版ではグローバル変数を使用 |
| getMorphBorderPixelColor | ❌ 未実装 | - | |

### morphapp.c (応用演算)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixMorphGradient | ❌ 未実装 | - | 形態学的勾配 |
| pixExtractBoundary | ❌ 未実装 | - | 境界抽出 |
| pixMorphSequenceMasked | ❌ 未実装 | - | マスク付きシーケンス |
| pixMorphSequenceByComponent | ❌ 未実装 | - | 連結成分ごとの処理 |
| pixMorphSequenceByRegion | ❌ 未実装 | - | 領域ごとの処理 |
| pixTophat | ✅ 同等 | binary::top_hat | |
| pixHMT | ✅ 同等 | binary::hit_miss_transform | |
| pixMorphCompSequence | ✅ 同等 | sequence::morph_comp_sequence | |
| pixGrayscaleMorphSum | ❌ 未実装 | - | |
| pixMultiplyByColor | ❌ 未実装 | - | |
| pixHMTDwa | ❌ 未実装 | - | DWA版HMT |
| pixFHMTGen | ❌ 未実装 | - | |

### morphdwa.c (DWA実装)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixDilateBrickDwa | ✅ 同等 | dwa::dilate_brick_dwa | |
| pixErodeBrickDwa | ✅ 同等 | dwa::erode_brick_dwa | |
| pixOpenBrickDwa | ✅ 同等 | dwa::open_brick_dwa | |
| pixCloseBrickDwa | ✅ 同等 | dwa::close_brick_dwa | |
| pixDilateCompBrickDwa | 🔄 異なる | dwa::dilate_brick_dwa | Rust版は合成分解を自動選択 |
| pixErodeCompBrickDwa | 🔄 異なる | dwa::erode_brick_dwa | Rust版は合成分解を自動選択 |
| pixOpenCompBrickDwa | 🔄 異なる | dwa::open_brick_dwa | Rust版は合成分解を自動選択 |
| pixCloseCompBrickDwa | 🔄 異なる | dwa::close_brick_dwa | Rust版は合成分解を自動選択 |
| pixDilateCompBrickExtendDwa | ❌ 未実装 | - | 拡張版 |
| pixErodeCompBrickExtendDwa | ❌ 未実装 | - | 拡張版 |
| pixOpenCompBrickExtendDwa | ❌ 未実装 | - | 拡張版 |
| pixCloseCompBrickExtendDwa | ❌ 未実装 | - | 拡張版 |
| makeLinearBrickDwaGen | ❌ 未実装 | - | DWAコード生成 |
| makeLinearBrickDwa | ❌ 未実装 | - | |
| pixMorphDwa_*系 | ❌ 未実装 | - | 生成されたDWA関数 |

### morphseq.c (シーケンス処理)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixMorphSequence | ✅ 同等 | sequence::morph_sequence | |
| pixMorphCompSequence | ✅ 同等 | sequence::morph_comp_sequence | |
| pixMorphSequenceDwa | ❌ 未実装 | - | DWA版は未実装 |
| pixMorphCompSequenceDwa | ❌ 未実装 | - | |
| morphSequenceVerify | 🔄 異なる | sequence内部で検証 | 公開APIとしては未実装 |
| pixGrayMorphSequence | ✅ 同等 | sequence::gray_morph_sequence | |
| pixColorMorphSequence | ❌ 未実装 | - | Color版シーケンスは未実装 |

### graymorph.c (グレースケール形態学)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixErodeGray | ✅ 同等 | grayscale::erode_gray | |
| pixDilateGray | ✅ 同等 | grayscale::dilate_gray | |
| pixOpenGray | ✅ 同等 | grayscale::open_gray | |
| pixCloseGray | ✅ 同等 | grayscale::close_gray | |
| pixErodeGray3 | ❌ 未実装 | - | 3x3専用最適化版 |
| pixDilateGray3 | ❌ 未実装 | - | |
| pixOpenGray3 | ❌ 未実装 | - | |
| pixCloseGray3 | ❌ 未実装 | - | |
| dilateGrayLow | ❌ 未実装 | - | 低レベル関数 |
| erodeGrayLow | ❌ 未実装 | - | |
| pixTophat | ✅ 同等 | grayscale::top_hat_gray | white/black両対応 |

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
| selaCreate | ❌ 未実装 | - | Sela (Sel配列) 未実装 |
| selaDestroy | ❌ 未実装 | - | |
| selCreate | ❌ 未実装 | - | leptonica-coreで実装予定 |
| selDestroy | ❌ 未実装 | - | |
| selCopy | ❌ 未実装 | - | |
| selCreateBrick | ❌ 未実装 | - | |
| selCreateComb | ❌ 未実装 | - | |
| create2dIntArray | ❌ 未実装 | - | |
| selaAddSel | ❌ 未実装 | - | |
| selaGetCount | ❌ 未実装 | - | |
| selaGetSel | ❌ 未実装 | - | |
| selGetName | ❌ 未実装 | - | |
| selSetName | ❌ 未実装 | - | |
| selaFindSelByName | ❌ 未実装 | - | |
| selGetElement | ❌ 未実装 | - | |
| selSetElement | ❌ 未実装 | - | |
| selGetParameters | ❌ 未実装 | - | |
| selSetOrigin | ❌ 未実装 | - | |
| selGetTypeAtOrigin | ❌ 未実装 | - | |
| selaGetBrickName | ❌ 未実装 | - | |
| selaGetCombName | ❌ 未実装 | - | |
| getCompositeParameters | ❌ 未実装 | - | |
| selaGetSelnames | ❌ 未実装 | - | |
| selFindMaxTranslations | ❌ 未実装 | - | |
| selRotateOrth | ❌ 未実装 | - | |
| selaRead | ❌ 未実装 | - | |
| selaReadStream | ❌ 未実装 | - | |
| selRead | ❌ 未実装 | - | |
| selReadStream | ❌ 未実装 | - | |
| selaWrite | ❌ 未実装 | - | |
| selaWriteStream | ❌ 未実装 | - | |
| selWrite | ❌ 未実装 | - | |
| selWriteStream | ❌ 未実装 | - | |
| selCreateFromString | ❌ 未実装 | - | |
| selPrintToString | ❌ 未実装 | - | |
| selaCreateFromFile | ❌ 未実装 | - | |
| selCreateFromPta | ❌ 未実装 | - | |
| selCreateFromPix | ❌ 未実装 | - | |
| selReadFromColorImage | ❌ 未実装 | - | |
| selCreateFromColorPix | ❌ 未実装 | - | |
| selaCreateFromColorPixa | ❌ 未実装 | - | |
| selDisplayInPix | ❌ 未実装 | - | |
| selaDisplayInPix | ❌ 未実装 | - | |

### sel2.c (Sel定義済みセット)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| sel4ccThin系 (16関数) | ❌ 未実装 | - | 個別Sel生成関数 |
| sel8ccThin系 (16関数) | ❌ 未実装 | - | |
| selMakeThinSets | ✅ 同等 | thin_sels::make_thin_sels | 個別関数ではなくまとめて生成 |

### selgen.c (Sel自動生成)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixGenerateSelBoundary | ❌ 未実装 | - | |
| pixGenerateSelWithRuns | ❌ 未実装 | - | |
| pixGenerateSelRandom | ❌ 未実装 | - | |
| pixGetRunCentersOnLine | ❌ 未実装 | - | |
| pixGetRunsOnLine | ❌ 未実装 | - | |
| pixSubsampleBoundaryPixels | ❌ 未実装 | - | |
| adjacentOnPixelInRaster | ❌ 未実装 | - | |
| pixDisplayHitMissSel | ❌ 未実装 | - | |

### ccthin.c (連結成分保存細線化)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaThinConnected | ❌ 未実装 | - | PIXA版は未実装 |
| pixThinConnected | ✅ 同等 | thin::thin_connected | |
| pixThinConnectedBySet | ✅ 同等 | thin::thin_connected_by_set | |
| selaMakeThinSets | ✅ 同等 | thin_sels::make_thin_sels | |

### dwacomb.2.c (DWA生成コード)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| fmorphopgen_low_2 | ❌ 未実装 | - | 自動生成されたDWAコード |

### fmorphauto.c (DWAコード自動生成)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| fmorphautogen | ❌ 未実装 | - | DWAコード生成機能 |
| fmorphautogen1 | ❌ 未実装 | - | |
| fmorphautogen2 | ❌ 未実装 | - | |

### fmorphgen.1.c (DWA生成コード)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| fmorphopgen_low_1 | ❌ 未実装 | - | 自動生成されたDWAコード |

### fmorphgenlow.1.c (DWA低レベルコード)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| (低レベル関数群) | ❌ 未実装 | - | DWA内部実装 |

## 実装状況の分析

### 実装済み領域
1. **基本形態学演算**: dilate, erode, open, close (binary, gray, color)
2. **Brick演算**: 矩形SELによる高速演算
3. **DWA演算**: 基本的なbrick DWA演算
4. **シーケンス処理**: 基本的なmorph sequence機能
5. **細線化**: 連結成分保存細線化
6. **グレースケール形態学**: van Herk/Gil-Werman法による実装
7. **カラー形態学**: RGB各成分への個別適用

### 未実装領域
1. **Sel/Selaデータ構造**: leptonica-coreへの移動が必要
2. **Sel自動生成**: pixGenerateSelBoundary等
3. **Safe closing**: 境界条件を考慮したclosing
4. **DWAコード生成機能**: 実行時ではなくコンパイル時に生成予定
5. **応用演算**: gradient, boundary extraction, masked operations
6. **最適化版**: 3x3専用のgrayscale morphology

### アーキテクチャの違い

#### C版の特徴
- グローバル変数で境界条件を管理
- 関数名でoperationタイプを指定（pixColorMorph(type)）
- DWAコードは実行時に生成されたCコードを使用
- Sel/Selaは複雑なポインタ配列構造

#### Rust版の特徴
- 境界条件は引数で明示的に指定
- 個別の型安全な関数（dilate_color, erode_color等）
- DWAコードはコンパイル時生成（将来）
- Selはleptonica-coreのデータ構造を使用予定
- 合成分解（composite decomposition）を自動選択

## 今後の実装優先度

### 高優先度
1. Sel/Selaデータ構造の実装（leptonica-coreへ）
2. Safe closing機能
3. 形態学的gradient
4. Masked sequence operations

### 中優先度
1. 3x3専用grayscale最適化
2. Sel自動生成機能
3. DWA拡張機能
4. Color morphology sequence

### 低優先度
1. DWAコード生成機能（手動実装で代替可能）
2. Generalized open/close
3. 境界抽出の個別関数

## 備考

- Rust版は型安全性とメモリ安全性を重視した設計
- 一部の関数は内部実装で使用（非公開）
- DWA実装は段階的に拡張予定
- Sel関連機能はleptonica-coreへの移行が前提
