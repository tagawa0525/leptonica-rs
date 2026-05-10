# Core: graphics.c の補助 3 関数 (plan 032 カテゴリ J)

Status: IMPLEMENTED
完了日: 2026-05-10
作成日: 2026-05-10
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ J

## 対象 C 関数

- `generatePtaLineFromPt(x, y, length, radang)` — 始点+長さ+角度で線を生成
- `locatePtRadially(xr, yr, dist, radang) -> (x, y)` — 極座標→直交変換
- `makePlotPtaFromNuma(na, size, plotloc, linewidth, max)` — Numa からプロット用 Pta 生成 (`make_plot_pta_from_numa_gen` の wrapper)

## API 設計

`src/core/pix/graphics.rs`:

```rust
pub fn generate_pta_line_from_pt(x: i32, y: i32, length: f64, radang: f64) -> Pta;
pub fn locate_pt_radially(xr: i32, yr: i32, dist: f64, radang: f64) -> (f64, f64);
pub fn make_plot_pta_from_numa(
    na: &Numa, size: u32, plotloc: PlotLocation, linewidth: u32, max: u32,
) -> Result<Pta>;
```

依存: 既存 `generate_line_pta`, 内部 `make_plot_pta_from_numa_gen`

## 完了条件

- [x] cargo test/clippy/fmt 通過
- [x] PR + Copilot レビュー対応 + マージ
- [x] docs/porting/comparison/core.md で 3 件 ❌ → ✅
- [x] docs/plans/032 で J を IMPLEMENTED に
