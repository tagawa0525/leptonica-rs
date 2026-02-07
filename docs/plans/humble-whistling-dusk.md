# P1テスト実装計画

## Context（背景）

C版Leptonicaの160個の回帰テストをRust版に移植するプロジェクト（golden-doodling-sedgewick.md）において、P1テスト（最優先・19個）をカテゴリ別に実装する要望を受けました。

調査の結果、**驚くべき事実が判明**：**19個のP1テストのうち16個が既に実装完了済み**でした。

### 完了済みP1テスト（16個）

| カテゴリ   | テスト名    | 状態 | 結果 |
| ---------- | ----------- | ---- | ---- |
| I/O        | pngio       | ✅   | PASS |
| I/O        | jpegio      | ✅   | PASS |
| I/O        | ioformats   | ✅   | PASS |
| Morph      | binmorph1   | ✅   | PASS |
| Morph      | binmorph2   | ✅   | PASS |
| Morph      | binmorph3   | ✅   | PASS |
| Morph      | dwamorph1   | ✅   | PASS |
| Transform  | rotate1     | ✅   | PASS |
| Transform  | rotate2     | ✅   | PASS |
| Transform  | rotateorth  | ✅   | PASS |
| Transform  | scale       | ✅   | PASS |
| Filter     | convolve    | ✅   | PASS |
| Filter     | edge        | ✅   | PASS |
| Color      | colorspace  | ✅   | PASS |
| Color      | binarize    | ✅   | PASS |
| Region     | conncomp    | ✅   | PASS |
| Region     | label       | ✅   | PASS |
| Region     | ccbord      | ✅   | PASS |
| Recognition| skew        | ✅   | PASS |

## 進捗状況

進捗率：16/19 = 84%

### 残りP1テスト（3個 - すべてCore）

| # | テスト | 難易度 | 課題                               |
| - | ------ | ------ | ---------------------------------- |
| 1 | boxa1  | 7/10   | 未実装9個、メモリシリアライズ      |
| 2 | boxa2  | 8/10   | 未実装11個、Numa依存（7+個）       |
| 3 | pta    | 6/10   | 未実装7個、境界抽出必要            |

## 問題点：未実装機能とテストの依存関係

Core P1テストは**多くの未実装機能に依存**しており、単純に移植することができません。以下に、未実装機能ごとに、どのテストがブロックされているかを整理します。

### 未実装機能一覧とブロックされるテスト

#### A. Boxaファイル I/O機能（4個）

| 未実装機能        | 説明                      | ブロック   | 難易度 |
| ----------------- | ------------------------- | ---------- | ------ |
| boxaRead()        | ファイルからBoxa読み込み  | boxa1,2    | 中     |
| boxaWrite()       | ファイルにBoxa書き込み    | boxa1,2    | 中     |
| boxaReadMem()     | メモリからBoxa読み込み    | boxa1      | 中     |
| boxaWriteMem()    | Boxaメモリシリアライズ    | boxa1      | 中     |

**実装方針**: Boxaのファイルフォーマット定義が必要。C版の形式を参考に実装。

---

#### B. Boxa操作・変換機能（6個）

| 未実装機能                | 説明                        | ブロック | 難易度 |
| ------------------------- | --------------------------- | -------- | ------ |
| boxaCompareRegions()      | 2つのBoxa領域差分を計算     | boxa1    | 高     |
| boxaTransform()           | Boxa全体を並進・スケール変換| boxa1    | 低     |
| boxaExtractCorners()      | Boxaコーナー座標をPta抽出   | boxa1    | 低     |
| boxaShiftWithPta()        | Pta座標リストで個別移動     | boxa1    | 低     |
| boxaFindInvalidBoxes()    | 無効なBox検出               | boxa2    | 低     |
| boxaFillSequence()        | 無効なBox補完               | boxa2    | 中     |
| boxaSplitEvenOdd()        | 奇数・偶数インデックス分割  | boxa2    | 低     |
| boxaGetSizes()            | width/height数値配列取得    | boxa2    | 低     |

**実装方針**:

- 簡単な変換（transform, extractCorners等）は既存のBox APIを活用して実装可能
- `boxaCompareRegions()`は複雑（Pix画像生成、XOR演算、面積計算が必要）

---

#### C. Boxa描画機能（3個）

| 未実装機能        | 説明                      | ブロック | 難易度 |
| ----------------- | ------------------------- | -------- | ------ |
| pixMaskBoxa()     | Boxa領域をマスク描画      | boxa1    | 中     |
| pixRenderBoxaArb()| Boxa任意色・太さで描画    | boxa1    | 中     |
| boxaPlotSizes()   | Boxサイズ分布の可視化     | boxa2    | 中     |

**実装方針**: 既存の`render_box()`を拡張してBoxa版を実装。

---

#### D. Numa（数値配列）モジュール（7個以上）

| 未実装機能                    | 説明                    | ブロック | 難易度 |
| ----------------------------- | ----------------------- | -------- | ------ |
| numaGetMedian()               | 中央値計算              | boxa2    | 中     |
| numaMakeConstant()            | 定数配列生成            | boxa2    | 低     |
| numaGetIValue()               | i32値の取得             | boxa2    | 低     |
| numaSetValue()                | 値の設定                | boxa2    | 低     |
| numaMakeThresholdIndicator()  | しきい値インジケータ生成| boxa2    | 中     |
| numaGetCountRelativeToZero()  | 0との比較でカウント     | boxa2    | 低     |
| numaGetRankBinValues()        | ランク・ビン統計計算    | boxa2    | 高     |

**実装方針**: Numaモジュール全体の設計・実装が必要（独立した大規模タスク）。

- `Numa` struct（数値配列の基本構造）
- 基本操作（生成、追加、取得、設定）
- 統計関数（中央値、平均、最大最小）
- 変換関数（しきい値、カウント、ビニング）

**推定工数**: 2-3日

---

#### E. Pta操作・変換機能（5個）

| 未実装機能           | 説明                    | ブロック | 難易度 |
| -------------------- | ----------------------- | -------- | ------ |
| ptaRotate()          | 任意中心での回転        | pta      | 低     |
| ptaReverse()         | 点の順序を反転          | pta      | 低     |
| ptaCyclicPerm()      | cyclic置換              | pta      | 低     |
| ptaEqual()           | Pta等価判定             | pta      | 低     |
| ptaPolygonIsConvex() | 凸多角形判定            | pta      | 中     |

**実装方針**:

- 基本操作（rotate, reverse, cyclic）は既存のPta APIを拡張して実装可能
- 凸包判定はアルゴリズム実装が必要（Cross Product法など）

---

#### F. Ptaa描画機能（2個）

| 未実装機能 | 説明 | ブロック | 難易度 |
| --- | --- | --- | --- |
| pixRenderRandomCmapPtaa | Ptaaランダムカラー | pta | 中 |
| pixDisplayPtaa | 複数Ptaの描画合成 | pta | 中 |

**実装方針**: 既存の`render_pta()`を拡張してPtaa版を実装。

---

#### G. Region機能（1個）

| 未実装機能              | 説明                      | ブロック | 難易度 |
| ----------------------- | ------------------------- | -------- | ------ |
| ptaaGetBoundaryPixels() | 連結成分の境界をPta抽出   | pta      | 高     |

**実装方針**: 既存の`find_connected_components()`を拡張し、境界トレーシングを実装。

---

### テスト実装可能条件マトリクス

| テスト | 未実装数 | 内訳                  | 難易度 |
| ------ | -------- | --------------------- | ------ |
| boxa1  | 9個      | A:4, B:2, C:3         | 7/10   |
| boxa2  | 18個     | A:2, B:4, C:1, D:7+   | 8/10   |
| pta    | 8個      | E:5, F:2, G:1         | 6/10   |

### 機能実装の優先順位（効率的な順序）

#### 優先度1: Pta操作・変換機能（E）

**理由**:

- 実装が比較的容易（5個すべて低～中難易度）
- ptaテストを部分的に実装可能にする
- 他の依存が少ない

**推定工数**: 1日

**実装後に可能になるテスト**: pta（部分的）

---

#### 優先度2: Pta/Ptaa描画機能（F）

**理由**:

- 既存の描画APIを拡張すれば実装可能
- ptaテストの描画検証に必要

**推定工数**: 0.5日

**実装後に可能になるテスト**: pta（さらに完成度向上）

---

#### 優先度3: Region境界抽出（G）

**理由**:

- ptaテストの完全実装に必要
- 連結成分分析の拡張として有用

**推定工数**: 0.5-1日

**実装後に可能になるテスト**: pta（完全実装）

---

#### 優先度4: Boxa基本操作（B - 簡単なもの）

**理由**:

- `boxaTransform`, `boxaExtractCorners`等の簡単な変換は実装しやすい
- boxa1テストの部分実装に貢献

**推定工数**: 1日

**実装後に可能になるテスト**: boxa1（部分的）

---

#### 優先度5: BoxaファイルI/O（A）

**理由**:

- 両テストで必要
- フォーマット定義が必要だが、一度実装すれば複数テストで活用

**推定工数**: 1-1.5日

**実装後に可能になるテスト**: boxa1（さらに完成度向上）、boxa2（前提条件）

---

#### 優先度6: Boxa描画機能（C）

**理由**:

- boxa1の可視化に必要
- 既存の描画APIを拡張

**推定工数**: 1日

**実装後に可能になるテスト**: boxa1（ほぼ完成）

---

#### 優先度7（最低）: Numaモジュール（D）⚠️ **大規模タスク**

**理由**:

- boxa2のみに必要（他のテストへの波及効果が限定的）
- 独立した大規模実装（モジュール全体）
- 最も工数がかかる

**推定工数**: 2-3日

**実装後に可能になるテスト**: boxa2（完全実装）

---

### 実装戦略の結論

**段階的アプローチが最適**:

1. **pta機能の完成**（優先度1-3）→ 2-2.5日で pta_reg.rs 完成
2. **P2テストへの展開**（既存API活用）→ 継続的な進捗
3. **boxa1の段階的実装**（優先度4-6）→ 3-3.5日で boxa1_reg.rs 完成
4. **Numaモジュール実装**（優先度7）→ P2/P3で必要性が高まった時点で実装し、boxa2_reg.rs を完成

## 実装戦略の選択肢

### オプションA: Core P1テストを完全実装

**アプローチ**: 不足している機能をすべて実装してからテストを移植

**利点**:

- P1テスト100%完了を達成できる
- Boxa/Pta/Numaモジュールが充実する
- 今後のP2/P3テストの基盤となる

**欠点**:

- 大規模な実装が必要（27個以上の新関数）
- 特にNumaモジュールは独立した大きなタスク
- 完了まで数日～数週間かかる可能性

**推定工数**:

- Numaモジュール: 2-3日
- Boxa拡張機能: 1-2日
- Pta拡張機能: 1日
- テスト実装: 0.5-1日
- **合計: 4.5-7日**

### オプションB: P2テストに進む（推奨）

**アプローチ**: Core P1を後回しにし、既に実装済みのAPIを活用するP2テストを優先

**利点**:

- 即座に進捗を上げられる
- 既存APIのカバレッジを拡大
- 段階的にCore機能の必要性を評価できる

**欠点**:

- P1完了率が84%で止まる
- Core機能が未完成のまま

**次の候補P2テスト（実装済みAPIが多い）**:

- Morph: binmorph4, binmorph5, dwamorph2, graymorph1, colormorph, morphseq（6個）
- Transform: affine, bilinear, projective（3個）
- Filter: bilateral1, bilateral2, rank, adaptmap, adaptnorm（5個）
- Color: colorquant, cmapquant, colorseg, colorcontent, colorfill, threshnorm（6個）

### オプションC: 段階的実装

**アプローチ**: Core P1テストの実装可能な部分だけを先に実装

**具体的**:

1. **ptaを優先**: 比較的依存が少ない（難易度6/10）
   - 必要な7個の関数を実装
   - 推定工数: 1-1.5日
2. **boxa1, boxa2を後回し**: Numaモジュール依存が大きい
3. **P2テストと並行**: Pta実装中にP2テストも進める

**利点**:

- 1個ずつ確実に進められる
- リスク分散

**欠点**:

- 完了まで時間がかかる
- モチベーション維持が課題

## 推奨プラン（オプションB + 部分的オプションC）

### Phase 1: プランドキュメント更新（即座）

`golden-doodling-sedgewick.md` の進捗状況を更新:

- 完了済み16個のP1テストを「✅」にマーク
- 進捗サマリーを更新（3→19完了）

### Phase 2: 簡単なCore機能から実装（1-2日）

**pta関連の未実装機能**を優先実装:

1. **基本的なPta操作**（優先度高）:
   - `ptaRotate()` - 既存のrotate実装を参考に実装可能
   - `ptaReverse()` - 配列反転（簡単）
   - `ptaCyclicPerm()` - cyclic置換（中程度）
   - `ptaEqual()` - 等価判定（簡単）

2. **幾何判定**（優先度中）:
   - `ptaPolygonIsConvex()` - 凸包判定（アルゴリズム実装必要）

3. **描画関連**（優先度低）:
   - `pixRenderRandomCmapPtaa()` - 既存の描画APIを拡張
   - `pixDisplayPtaa()` - 描画合成

4. **境界抽出**（優先度中・Region依存）:
   - `ptaaGetBoundaryPixels()` - conncomp実装を拡張

**成果**: pta_reg.rs を実装完了 → P1進捗17/19（89%）

### Phase 3: P2テストに進む（継続的）

より多くのテストカバレッジを獲得するため、P2テストの実装を開始:

**優先順位**:

1. Morph P2（6個） - 既存morphology実装の拡張
2. Filter P2（5個） - 既存filter実装の拡張
3. Color P2（6個） - 既存color実装の拡張
4. Transform P2（3個） - 既存transform実装の拡張

### Phase 4: Numaモジュール実装（必要に応じて）

P2/P3テストを進める中で、Numaモジュールの必要性が高まった場合に実装:

- 独立したタスクとして計画
- boxa2やその他のNuma依存テストをまとめて実装

## Critical Files（重要ファイル）

### 更新が必要なファイル

- `docs/plans/golden-doodling-sedgewick.md` - 進捗状況の更新

### 新規実装が必要なファイル（Phase 2でpta実装時）

- `crates/leptonica-core/src/pta/mod.rs` - Pta新機能の実装
- `crates/leptonica-core/src/pta/geometry.rs` - 幾何関数（ptaPolygonIsConvex等）
- `crates/leptonica-core/tests/pta_reg.rs` - 新規テストファイル作成
- `crates/leptonica-core/src/pix/graphics.rs` - Ptaa描画関数の追加

### 参考ファイル

- C版テスト: `reference/leptonica/prog/pta_reg.c`
- 既存Pta実装: `crates/leptonica-core/src/pta/mod.rs`
- テストフレームワーク: `crates/leptonica-test/src/params.rs`
- 既存テスト例:
  - `crates/leptonica-morph/tests/binmorph1_reg.rs`
  - `crates/leptonica-region/tests/conncomp_reg.rs`

## Verification（検証方法）

### Phase 1: プランドキュメント更新の検証

```bash
# ドキュメントの差分確認
git diff docs/plans/golden-doodling-sedgewick.md
```

### Phase 2: pta実装の検証

```bash
# Pta新機能のユニットテスト
cargo test -p leptonica-core pta

# pta回帰テストの実行
cargo test -p leptonica-core --test pta_reg

# すべてのP1テストの確認
cargo test --workspace --test '*_reg' | grep "P1"
```

### Phase 3: P2テスト実装の検証

```bash
# P2テストの実行
cargo test -p leptonica-morph --test binmorph4_reg
cargo test -p leptonica-filter --test bilateral1_reg

# 全体の回帰テスト
cargo test --workspace --test '*_reg'
```

## 推奨実装順序まとめ

1. **即座**: プランドキュメント更新（golden-doodling-sedgewick.md）
2. **1-2日**: Pta関連機能の実装とpta_reg.rsの作成
3. **継続的**: P2テストの段階的実装（Morph → Filter → Color → Transform）
4. **必要時**: Numaモジュールとboxa1/boxa2の実装

この戦略により、**効率的に進捗を上げながら**、Core機能の必要性を評価し、適切なタイミングで実装を進めることができます。
