# Core: ptafunc1.c の Pta + graphics 7 関数 (plan 032 カテゴリ E の一部)

Status: IMPLEMENTED
作成日: 2026-05-12
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ E

## 対象 C 関数 (7)

軽量・独立性の高い Pta-Pix-Numa 変換系 7 関数。残り 8 関数
(`pixPlotAlongPta`, `ptaGetBoundaryPixels`, `ptaGetNeighborPixLocs`,
`ptaNoisyLinearLSF`, `ptaNoisyQuadraticLSF`,
`ptaaGetBoundaryPixels`, `ptaaIndexLabeledPixels`) は plan 111b
で扱う。

### Pta <-> Box / Pix

- `ptaGetBoundingRegion(pta) -> Box` — 整数の bounding box
- `pixGenerateFromPta(pta, w, h) -> Pix(1bpp)` — Pta の各点を 1 bpp にレンダ
- `ptaGetPixelsFromPix(pixs, box?) -> Pta` — 1bpp の前景ピクセル座標を Pta に
- `pixFindCornerPixels(pixs) -> Pta` — 1bpp の 4 隅 (TL/TR/BL/BR) の前景

### Pta <-> Numa

- `ptaConvertToNuma(pta) -> (Numa, Numa)` — x/y 座標を 2 つの Numa に分割
- (numaConvertToPta1, numaConvertToPta2 は既存 `Pta::create_from_numa` で カバー済みのため対応エントリ更新のみ)

### Pattern replication

- `ptaReplicatePattern(ptas, pattern, cx, cy, w, h) -> Pta` — パターンを各点に複製、境界内にあるものだけを残す

## API 設計

```rust
impl Pta {
    /// C: `ptaGetBoundingRegion`
    pub fn bounding_region(&self) -> Option<Box>;

    /// C: `ptaConvertToNuma` → (nax, nay)
    pub fn to_numa_pair(&self) -> (Numa, Numa);

    /// C: `ptaReplicatePattern`
    pub fn replicate_pattern(
        &self,
        pattern: PatternSource<'_>,
        cx: i32, cy: i32, w: i32, h: i32,
    ) -> Pta;
}

impl Pix {
    /// C: `pixFindCornerPixels` (1 bpp required)
    pub fn find_corner_pixels(&self) -> Result<Pta>;
}

/// C: `pixGenerateFromPta`
pub fn pix_generate_from_pta(pta: &Pta, w: u32, h: u32) -> Result<Pix>;

/// C: `ptaGetPixelsFromPix` (1 bpp required)
pub fn pta_get_pixels_from_pix(pixs: &Pix, region: Option<&Box>) -> Result<Pta>;

pub enum PatternSource<'a> {
    Pix(&'a Pix),
    Pta(&'a Pta),
}
```

## 依存

- 既存 `Pta::push`, `Pta::get`, `Pta::get_i_pt`, `Pta::iter`
- 既存 `Numa::push`, `Numa::with_capacity`
- 既存 `Pix::get_pixel_unchecked`, `Pix::set_pixel`, `Pix::depth`
- 既存 `Box::new`

## テスト方針

- `bounding_region`: 通常 / 空 Pta で None / 単一点
- `to_numa_pair`: ラウンドトリップ (Pta -> (Numa, Numa))
- `pix_generate_from_pta`: 境界外点は無視、想定された FG カウント
- `pta_get_pixels_from_pix`: 全画像 / region 指定、非 1bpp で Err
- `find_corner_pixels`: 全黒 / 四隅検出 / 非 1bpp で Err
- `replicate_pattern`: 単純パターン (中心) を 2 点に複製、境界外切り捨て

## 完了条件

- [x] cargo test/clippy/fmt 通過 (15 件パス)
- [x] core.md 8 件 ❌ -> ✅ (本 plan 7 + 既存 Pta::create_from_numa 対応の numaConvertToPta1/2)
- [x] plan 032 で 111 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `Pta::bounding_region`: `get_i_pt` を 1 パススキャン、empty で None、 `checked_sub`/`checked_add` で overflow ガード
- `Pta::to_numa_pair`: 各点を `Numa::push` で x/y に分割
- `Pta::replicate_pattern`: `PatternSource` enum で Pix/Pta を統一受け取り、 `Pix` の場合は `pta_get_pixels_from_pix(None)` で前景点に変換
- `Pix::find_corner_pixels`: 各コーナーから対角線スキャンするクロージャ ヘルパーで 4 方向を統一処理
- `pix_generate_from_pta` / `pta_get_pixels_from_pix`: 1bpp 限定、 範囲外は silently drop / clamp
- `numaConvertToPta1` / `numaConvertToPta2` は既存 `Pta::create_from_numa` でカバー済み (対応エントリのみ更新)
