# Filter: pixStrokeWidthTransform 移植 (plan 032 M 単独)

Status: PLANNED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ M (単独関数)

## 対象 C 関数 (1)

`runlength.c` の `pixStrokeWidthTransform`。1bpp 入力に対して、
各 fg pixel のストローク幅を 8/16 bpp で記録した画像を返す。

## API 設計

```rust
/// C: `pixStrokeWidthTransform`
pub fn stroke_width_transform(
    pix: &Pix,
    color: u32,
    depth: PixelDepth,
    nangles: u32,
) -> Result<Pix>;
```

- `color`: 0 = white runs (invert 入力)、1 = black runs (そのまま)
- `depth`: 8 or 16 bpp (それ以外は Err)
- `nangles`: 2 / 4 / 6 / 8 (それ以外は Err)

## アルゴリズム

1. `color = 0` なら invert で fg = black に揃える
2. 0/90 度の `runlength_transform` を取り、min を `pixg1` に
3. `nangles == 4 or 8`: ±45 度の最小ランレングス (`find_min_runs_orthogonal`)
4. `nangles == 6`: ±30 / ±60 度 の最小ランレングス 2 セット
5. `nangles == 8`: ±22.5 / ±67.5 度 の最小ランレングス 2 セット
6. 各セットを `arith_min` で合成

### `find_min_runs_orthogonal` (内部 helper)

1. 入力 1bpp を diagonal サイズの中心キャンバスに raster-copy
2. 角度 `angle` でせん断回転
3. 0/90 度 ランレングス変換 → min
4. 逆向き (-angle) せん断回転
5. 元の位置を切り出す

## 依存

- 既存 `Pix::invert`、`Pix::deep_clone`
- 既存 `filter::runlength::runlength_transform`
- 既存 `Pix::arith_min`
- 既存 `transform::rotate_shear`
- 既存 `PixMut::rop_region_inplace` (中心パディング)
- 既存 `Pix::clip_rectangle`

## テスト方針

- 1bpp の単純な矩形について `stroke_width_transform(pix, 1, Bit8, 2)`
  → 8bpp 出力でストローク幅が反映される (中心は最大値)
- `color = 0` で background pixel が fg として扱われることを確認
- `depth = Bit2 など` で Err
- `nangles = 3 など` で Err
- 1bpp 以外で Err

## 完了条件

- [ ] cargo test/clippy/fmt 通過
- [ ] filter.md 1 件 ❌ → ✅
- [ ] plan 032 で 128 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- shear 回転は `transform::rotate_shear(pix, cx, cy, angle, ShearFill::White)`
- 中心キャンバスは `diag = (w*w + h*h).sqrt() + 2.5` のサイズで作成
- 戻し回転 (`-angle`) 後、(xoff, yoff, w, h) を `clip_rectangle` で切り出し
