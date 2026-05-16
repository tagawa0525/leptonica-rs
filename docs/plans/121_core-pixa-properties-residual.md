# Core: Pixa プロパティ残り 3 関数 (plan 032 カテゴリ A-3 の続き)

Status: IMPLEMENTED
作成日: 2026-05-12
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-3

## 対象 C 関数 (3)

plan 108 (8 関数) の残り 16 件のうち、独立性が高くて
依存先が既に揃っている 3 件を移植する。

### Sort / Select

- `pixaMakeSizeIndicator(pixa, width, height, type, relation) -> Numa` — 各 Pix の (w, h) と (width, height) を比較し、選別 0/1 indicator Numa を返す
- `pixaSort2dByIndex(pixas, naa) -> Pixaa` — Numaa の各 Numa を index リストとして Pixa を 2D 構造 (Pixaa) に展開
- `pixaConstrainedSelect(pixas, first, last, nmax, use_pairs) -> Pixa` — `gen_constrained_numa_in_range` でインデックス選択 → Pixa 構築

## API 設計

```rust
/// C: `L_SELECT_WIDTH` / `L_SELECT_HEIGHT` / `L_SELECT_IF_BOTH` /
/// `L_SELECT_IF_EITHER`
pub enum SizeIndicatorAxis {
    Width,
    Height,
    IfEither,
    IfBoth,
}

impl Pixa {
    /// C: `pixaMakeSizeIndicator`
    pub fn make_size_indicator(
        &self,
        width: u32, height: u32,
        axis: SizeIndicatorAxis,
        relation: ThresholdSelect,  // 既存 enum (plan 106)
    ) -> Numa;

    /// C: `pixaSort2dByIndex`
    pub fn sort_2d_by_index(&self, naa: &Numaa) -> Result<Pixaa>;

    /// C: `pixaConstrainedSelect`
    pub fn constrained_select(
        &self,
        first: i32, last: i32, nmax: i32, use_pairs: bool,
    ) -> Result<Pixa>;
}
```

## 依存

- 既存 `Pixa::pix_slice`, `boxa`, `with_capacity`, `push_with_box`
- 既存 `Numa::new`, `Numa::push`, `Numa::get`
- 既存 `Numaa::get`, `Numaa::len`
- 既存 `gen_constrained_numa_in_range` (plan 109)
- 既存 `ThresholdSelect` (plan 106)

## テスト方針

- make_size_indicator: width / height / both / either + lt/gt/lte/gte
- sort_2d_by_index: 3 個の Pix を 2 グループに分割
- constrained_select: 等間隔選択、use_pairs

## 完了条件

- [x] cargo test/clippy/fmt 通過 (10 件パス)
- [x] core.md 3 件 ❌ -> ✅
- [x] plan 032 で 121 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `SizeIndicatorAxis` enum で C `L_SELECT_WIDTH/HEIGHT/BOTH/EITHER` をモデル化、relation は既存 `ThresholdSelect`
- `make_size_indicator`: 1 パススキャンで w/h 個別判定 + axis 合成
- `sort_2d_by_index`: 総 index 数 == Pixa 長を検証、index は `i64` 経由で範囲外 + 負値の両方を弾く
- `constrained_select`: `last < 0` を `n - 1` に解釈、 `gen_constrained_numa_in_range` でインデックス Numa を生成してから Pix/Box を gather
