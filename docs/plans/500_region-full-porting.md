# leptonica-region 全未実装関数の移植計画

Status: IN_PROGRESS

## Context

leptonica-region crateはseedfill、conncomp、ccbord、watershed、maze、quadtreeの
基本機能を実装済みだが、C版leptonicaのregion関数群に対して以下の機能が欠落している:

1. **Seedfill拡張** - border component抽出、hole filling variants、simple grayscale seedfillが未実装
2. **Local extrema** - 局所極値検出が未実装
3. **ConnComp拡張** - seedfillBB系（bounding box追跡付き）、nextOnPixelInRasterが未実装
4. **Label拡張** - connComp transform系、incremental labeling が未実装
5. **CCBord拡張** - step chain生成、single path、I/O、SVG出力が未実装
6. **Watershed拡張** - basin追跡、render fill/colors が未実装
7. **Gray maze** - グレースケール迷路探索が未実装

### 現状の実装状況

| モジュール | 実装済み関数数 | 状態 |
|-----------|-------------|------|
| seedfill.rs | 13 | コア実装済み、border comp/simple variant未対応 |
| conncomp.rs | 7 | 基本実装済み、seedfillBB未対応 |
| label.rs | 6 | 基本ラベリング済み、transform/incremental未対応 |
| ccbord.rs | 7 | 基本border tracing済み、chain/I/O/SVG未対応 |
| watershed.rs | 5 | 基本アルゴリズム済み、render未対応 |
| maze.rs | 3 | binary maze済み、gray maze未対応 |
| quadtree.rs | 8 | 完了 |
| select.rs | 1 | 完了 |

### スコープ除外（Rust移植に不適切なもの）

| 除外対象 | 理由 |
|----------|------|
| `ccbaDisplayImage1`, `ccbaDisplayImage2` | 可視化専用 |
| `ccbaDisplayBorder`, `ccbaDisplaySPBorder` | 可視化専用 |
| `fpixaDisplayQuadtree` | 可視化専用 |
| `pageseg.c` 全体 | leptonica-recog の pageseg.rs で既に基本実装あり |
| `classapp.c` 全体 | leptonica-recog の jbclass.rs で既にカバー |
| `quadtreeGetParent`, `quadtreeGetChildren` | ユーティリティ（Vecへの直接アクセスで代替可） |

---

## 実行順序

Phase 1 → 2 → 3 → 4 → 5 → 6 → 7 の順に直列で実行する。

```
Phase 1 (Seedfill拡張) ← 基盤。border comp/holeはPhase 3のconncompから使用
  → Phase 2 (Local extrema) ← seedfill結果の解析
    → Phase 3 (ConnComp拡張)
      → Phase 4 (Label拡張) ← conncompの結果に依存
        → Phase 5 (CCBord拡張)
          → Phase 6 (Watershed拡張)
            → Phase 7 (Gray maze)
```

---

## Phase 1: Seedfill拡張（1 PR）

**Status: IMPLEMENTED**

**C参照**: `reference/leptonica/src/seedfill.c` L180-600

### 実装内容

Border component操作:
- `extract_border_conn_comps(pix: &Pix, connectivity: u8) -> RegionResult<Pix>` - 画像境界に接する連結成分のみ抽出
- `fill_bg_from_border(pix: &Pix, connectivity: u8) -> RegionResult<Pix>` - 境界から背景を充填（孤立した背景領域を残す）
- `fill_holes_to_bounding_rect(pix: &Pix, connectivity: u8) -> RegionResult<Pix>` - 各連結成分の外接矩形内のホールを充填

Hole filling variants:
- `holes_by_filling(pix: &Pix, connectivity: u8) -> RegionResult<Pix>` - 既存fill_holesの代替アルゴリズム（seedfillベース）

Simple grayscale seedfill:
- `seedfill_gray_simple(pix_seed: &Pix, pix_mask: &Pix) -> RegionResult<Pix>` - シーケンシャル走査による簡易グレースケールseedfill
- `seedfill_gray_inv_simple(pix_seed: &Pix, pix_mask: &Pix) -> RegionResult<Pix>` - 逆方向簡易グレースケールseedfill

Basin filling:
- `seedfill_gray_basin(pix_seed: &Pix, pix_mask: &Pix, delta: u8, connectivity: u8) -> RegionResult<Pix>` - グレースケールbasin filling（差分制限付き）

### 修正ファイル

- `crates/leptonica-region/src/seedfill.rs`: 上記7関数追加

### テスト

- extract_border_conn_comps: 境界接触/非接触成分の正確な分離
- fill_bg_from_border: 孤立背景領域の保持確認
- fill_holes_to_bounding_rect: 各成分のホール充填検証
- simple seedfill vs 通常seedfillの結果一致確認
- テスト画像: `rabi.png` 等の文書画像

---

## Phase 2: Local extrema（1 PR）

**Status: IMPLEMENTED**

**C参照**: `reference/leptonica/src/seedfill.c` L1100-1350

### 実装内容

- `local_extrema(pix: &Pix, min_max_size: u32, min_diff: u32) -> RegionResult<(Pix, Pix)>` - 局所最大・局所最小のマスク画像ペアを返す
- `qualify_local_minima(pix: &Pix, pix_min: &Pix, max_val: u8) -> RegionResult<Pix>` - 局所最小値の中から条件を満たすもののみ選択
- `selected_local_extrema(pix: &Pix, min_distance: u32, select_type: ExtremaType) -> RegionResult<Pta>` - 選択された局所極値の座標リスト

```rust
pub enum ExtremaType {
    Minima,
    Maxima,
}
```

### 動作原理

`local_extrema`: グレースケール画像をdilate/erodeして元画像と比較。
dilate結果と等しいピクセルが局所最大、erode結果と等しいピクセルが局所最小。
min_max_sizeでdilate/erodeのカーネルサイズ、min_diffで平坦領域の除外を制御。

### 修正ファイル

- `crates/leptonica-region/src/seedfill.rs`: 上記3関数追加

### テスト

- 既知の局所極値パターン（手作り画像）での正確性
- min_max_size パラメータの効果確認
- selected_local_extrema の座標精度検証

---

## Phase 3: ConnComp拡張（1 PR）

**Status: IMPLEMENTED**

**C参照**: `reference/leptonica/src/conncomp.c` L300-700

### 実装内容

- `count_conn_comp(pix: &Pix, connectivity: u8) -> RegionResult<u32>` - 連結成分数のみ返す（ラベリングなし、高速）
- `next_on_pixel_in_raster(pix: &Pix, start_x: u32, start_y: u32) -> Option<(u32, u32)>` - ラスター順で次のONピクセルを返す

SeedfillBB系（bounding box追跡付きseedfill）:
- `seedfill_bb(pix: &mut PixMut, x: u32, y: u32, connectivity: u8) -> RegionResult<Box_>` - seedfill + bounding box返却
- `seedfill_4_bb(pix: &mut PixMut, x: u32, y: u32) -> RegionResult<Box_>` - 4-connected版
- `seedfill_8_bb(pix: &mut PixMut, x: u32, y: u32) -> RegionResult<Box_>` - 8-connected版

### 修正ファイル

- `crates/leptonica-region/src/conncomp.rs`: 上記5関数追加

### テスト

- count_conn_comp と find_connected_components().len() の結果一致
- next_on_pixel_in_raster のラスター順走査検証
- seedfill_bb の bounding box 精度
- 空画像でのエッジケース

---

## Phase 4: Label拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/conncomp.c`（transform系）, `label.c`相当の機能

### 実装内容

連結成分変換:
- `conn_comp_transform(pix: &Pix, connectivity: u8, transform_type: ConnCompTransform) -> RegionResult<Pix>` - 各成分を変換値（面積、ラベル等）で塗りつぶし
- `conn_comp_area_transform(pix: &Pix, connectivity: u8) -> RegionResult<Pix>` - 各成分をその面積値で塗りつぶし

```rust
pub enum ConnCompTransform {
    Area,       // 面積値で置換
    Label,      // ラベル番号で置換
}
```

Incremental labeling:
- `IncrementalLabeler::new(width: u32, height: u32, connectivity: u8) -> IncrementalLabeler` - 初期化
- `IncrementalLabeler::add_component(&mut self, pix: &Pix, x: u32, y: u32) -> RegionResult<u32>` - 成分追加、ラベル返却

Label→Color変換:
- `label_to_color(pix: &Pix) -> RegionResult<Pix>` - ラベル画像をランダムカラーに変換（可視化用）

### 修正ファイル

- `crates/leptonica-region/src/label.rs`: 上記関数・構造体追加

### テスト

- conn_comp_area_transform: 各成分のピクセル値が面積と一致
- conn_comp_transform: Label変換で一意のラベル付与確認
- IncrementalLabeler: 逐次追加とバッチラベリングの結果一致
- label_to_color: 出力が32bppで隣接成分が異なる色

---

## Phase 5: CCBord拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/ccbord.c` L700-1400

### 実装内容

Step chain生成:
- `CcBorda::generate_step_chains(&mut self) -> RegionResult<()>` - 境界点列からstep chain code生成
- `CcBorda::step_chains_to_pix_coords(&mut self) -> RegionResult<()>` - step chainをピクセル座標に変換

Single path:
- `CcBorda::generate_single_path(&mut self) -> RegionResult<()>` - 各成分の境界を単一連続パスに変換

I/O:
- `CcBorda::write<W: Write>(&self, writer: W) -> RegionResult<()>` - シリアライゼーション
- `CcBorda::read<R: Read>(reader: R) -> RegionResult<CcBorda>` - デシリアライゼーション

SVG出力:
- `CcBorda::write_svg<W: Write>(&self, writer: W) -> RegionResult<()>` - SVGパスとして出力
- `CcBorda::to_svg_string(&self) -> RegionResult<String>` - SVG文字列生成

### 修正ファイル

- `crates/leptonica-region/src/ccbord.rs`: 上記メソッド追加

### テスト

- step chain生成と座標への逆変換のラウンドトリップ
- single path の連続性検証
- write/read のラウンドトリップ（バイナリ形式）
- SVG出力の有効性検証（基本的なSVGパース）

### 既知の問題：メモリ効率

**Issue:** `get_all_borders()` が O(n_components * image_size) のメモリ使用量

**背景:**
- C版 leptonica: `pixGetAllCCBorders()` で component を逐次処理し、各処理後に即座にメモリ解放
  - `pixConnComp()` で component を検出して pixa に格納
  - ループ内で `pixaGetPix()` で一つずつ取出し、`pixGetCCBorders()` で処理
  - 処理後 `pixDestroy()` で即座に解放 → ピークメモリが制限される

- Rust版: `find_connected_components()` で全 component を先に検出後、ループで処理
  - 全 component が Vec に格納される → メモリが累積
  - 例：feyn-fract.tif (1600×1100) で component 数が多い場合、17GB 以上のメモリ割り当てを試みる

**改善案:**
Phase 5 実装時に、`find_connected_components()` を逐次処理版に変更することを検討。
または、`get_all_borders()` で component iterator を使用して、一度に 1 component のみ保持するように改正。

**現状:**
- `ccbord_reg_feyn_fract` テストは `#[ignore]` で実行スキップ（既知制限）
- 小～中規模画像（dreyfus1.png 等）では問題なし

---

## Phase 6: Watershed拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/watershed.c` L400-600

### 実装内容

Basin追跡:
- `WatershedResult::basins(&self) -> &Pixa` - セグメンテーション結果の各basin画像
- `WatershedResult::num_basins(&self) -> u32` - basin数

Render:
- `watershed_render_fill(result: &WatershedResult) -> RegionResult<Pix>` - 各basinを最小値で塗りつぶした画像
- `watershed_render_colors(result: &WatershedResult) -> RegionResult<Pix>` - 各basinをランダムカラーで着色した画像

### 修正ファイル

- `crates/leptonica-region/src/watershed.rs`: 上記関数・メソッド追加

### テスト

- basins の数が期待値と一致
- render_fill の各basin内ピクセル値が最小値
- render_colors の出力が32bpp、隣接basinが異なる色

---

## Phase 7: Gray maze（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/maze.c` L400-600

### 実装内容

- `search_gray_maze(pix: &Pix, start: (u32, u32), end: (u32, u32)) -> RegionResult<(Pta, Numa)>` - グレースケール画像上のコスト最小パス探索

### 動作原理

Dijkstraアルゴリズムを使用し、各ピクセルのグレー値をコストとして
最小コストパスを求める。バイナリ迷路の壁/通路ではなく、
グレースケール値の「高い領域を避けて低い領域を通る」パスを見つける。

結果: パス座標（Pta）と各ステップの累積コスト（Numa）

### 修正ファイル

- `crates/leptonica-region/src/maze.rs`: `search_gray_maze` 追加

### テスト

- 均一コスト画像でのBFSと同等の結果確認
- グラデーション画像での最短パス検証
- 開始/終了が到達不可能な場合のエラーハンドリング

---

## サマリー

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | Seedfill拡張 | 1 | 7 |
| 2 | Local extrema | 1 | 3 |
| 3 | ConnComp拡張 | 1 | 5 |
| 4 | Label拡張 | 1 | 5 |
| 5 | CCBord拡張 | 1 | 7 |
| 6 | Watershed拡張 | 1 | 4 |
| 7 | Gray maze | 1 | 1 |
| **合計** | | **7** | **32** |

## 共通ワークフロー

### TDD

1. **RED**: テスト作成コミット（`#[ignore = "not yet implemented"]`付き）
2. **GREEN**: 実装コミット（`#[ignore]`除去、テスト通過）
3. **REFACTOR**: 必要に応じてリファクタリングコミット

### PRワークフロー

1. `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
2. `/gh-pr-create` でPR作成
3. `/gh-actions-check` でCopilotレビュー到着を確認
4. `/gh-pr-review` でレビューコメント対応
5. CIパス確認後 `/gh-pr-merge --merge` でマージ
6. ブランチ削除

### ブランチ命名

```
main
└── feat/region-seedfill-ext     ← Phase 1
└── feat/region-local-extrema    ← Phase 2
└── feat/region-conncomp-ext     ← Phase 3
└── feat/region-label-ext        ← Phase 4
└── feat/region-ccbord-ext       ← Phase 5
└── feat/region-watershed-ext    ← Phase 6
└── feat/region-gray-maze        ← Phase 7
```

## 検証方法

各PRで以下を実行:

```bash
cargo fmt --check -p leptonica-region
cargo clippy -p leptonica-region -- -D warnings
cargo test -p leptonica-region
cargo test --workspace  # PR前に全ワークスペーステスト
```
