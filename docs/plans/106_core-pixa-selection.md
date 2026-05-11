# Core: pixafunc1.c の Pixa 選択系 14 関数 (plan 032 カテゴリ A-1)

Status: PLANNED
作成日: 2026-05-11
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-1

## 対象 C 関数 (14)

### Pixa-level select (基本)

- `pixaSelectRange(pixas, first, last, copyflag) -> pixad`
- `pixaSelectWithIndicator(pixas, na, &changed) -> pixad`
- `pixaSelectWithString(pixas, str, &error) -> pixad`

### Pix add/remove via indicator

- `pixAddWithIndicator(pixs, pixad, na) -> pixad` (重ね合わせ)
- `pixRemoveWithIndicator(pixs, pixad, na) -> pixad` (削除)

### Pixa-level select by metric

- `pixaSelectByNumConnComp(pixas, nmin, nmax, connectivity, &changed) -> pixad`
- `pixaSelectByAreaFraction(pixas, thresh, type, &changed) -> pixad`
- `pixaSelectByPerimSizeRatio(pixas, thresh, type, &changed) -> pixad`
- `pixaSelectByPerimToAreaRatio(pixas, thresh, type, &changed) -> pixad`
- `pixaSelectByWidthHeightRatio(pixas, thresh, type, &changed) -> pixad`

### Pix-level wrapper (conncomp → pixa → select → render)

- `pixSelectByAreaFraction(pixs, thresh, connectivity, type, &changed) -> pixd`
- `pixSelectByPerimSizeRatio(pixs, thresh, connectivity, type, &changed) -> pixd`
- `pixSelectByPerimToAreaRatio(pixs, thresh, connectivity, type, &changed) -> pixd`
- `pixSelectByWidthHeightRatio(pixs, thresh, connectivity, type, &changed) -> pixd`

## API 設計

### Selection threshold type (新規 enum)

```rust
/// C: L_SELECT_IF_LT, L_SELECT_IF_GT, L_SELECT_IF_LTE, L_SELECT_IF_GTE
pub enum ThresholdSelect { LessThan, GreaterThan, LessOrEqual, GreaterOrEqual }
```

### Pixa methods

```rust
impl Pixa {
    pub fn select_range(&self, first: usize, last: Option<usize>) -> Self;
    pub fn select_with_indicator(&self, indicator: &[bool]) -> (Self, bool);
    pub fn select_with_string(&self, s: &str) -> Result<(Self, bool)>;
    pub fn select_by_num_conn_comp(
        &self, nmin: u32, nmax: u32, connectivity: ConnectivityType,
    ) -> Result<(Self, bool)>;
    pub fn select_by_area_fraction(&self, thresh: f32, sel: ThresholdSelect) -> Result<(Self, bool)>;
    pub fn select_by_perim_size_ratio(&self, thresh: f32, sel: ThresholdSelect) -> Result<(Self, bool)>;
    pub fn select_by_perim_to_area_ratio(&self, thresh: f32, sel: ThresholdSelect) -> Result<(Self, bool)>;
    pub fn select_by_width_height_ratio(&self, thresh: f32, sel: ThresholdSelect) -> Result<(Self, bool)>;
}
```

### Pix freestanding fns

```rust
pub fn pix_add_with_indicator(pixs: &Pix, pixad: &mut PixMut, indicator: &[bool]) -> Result<()>;
pub fn pix_remove_with_indicator(pixs: &Pix, pixad: &mut PixMut, indicator: &[bool]) -> Result<()>;
pub fn pix_select_by_area_fraction(pixs: &Pix, thresh: f32, conn: ConnectivityType, sel: ThresholdSelect) -> Result<Pix>;
// 同様に pix_select_by_perim_size_ratio / pix_select_by_perim_to_area_ratio / pix_select_by_width_height_ratio
```

### Internal find helpers (free fn, not part of pub API)

- `pixa_find_area_fraction(pixa) -> Numa`
- `pixa_find_perim_size_ratio(pixa) -> Numa`
- `pixa_find_perim_to_area_ratio(pixa) -> Numa`
- `pixa_find_width_height_ratio(pixa) -> Numa`

## 完了条件

- [ ] cargo test/clippy/fmt 通過
- [ ] PR + Copilot レビュー対応 + マージ
- [ ] core.md 14 件 ❌ → ✅
- [ ] plan 032 で 106 を IMPLEMENTED に
