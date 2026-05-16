# Core: ptaa_get_boundary_pixels (plan 032 残: 111b)

Status: IMPLEMENTED
作成日: 2026-05-14
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ E (111b)

## 対象 C 関数 (1)

- `ptaaGetBoundaryPixels(pixs, type, connectivity, &boxa, &pixa)` — 各 connected component の境界ピクセル座標を Ptaa にして返す

## API 設計

```rust
pub fn ptaa_get_boundary_pixels(
    pixs: &Pix,
    btype: BoundaryType,
    connectivity: u32,
    want_boxa: bool,
    want_pixa: bool,
) -> Result<(Ptaa, Option<Boxa>, Option<Pixa>)>;
```

- `connectivity`: 4 または 8
- BG boundary は各 component を 1 ピクセルパディングしてから境界抽出
- 出力 Ptaa の各 Pta は親画像の座標系 (component box の (x, y) オフセット込み)

## 依存

- 既存 `region::conncomp_pixa`
- 既存 `Pix::add_border_general`
- plan 137 の `pta_get_boundary_pixels`
- 既存 `Pta::translate`

## 完了条件

- [x] cargo test/clippy/fmt 通過 (6 件パス)
- [x] core.md 1 件 ❌ → ✅
- [x] plan 032 で 139 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
