# region/rectangle: 矩形検出 3 関数の移植

Status: IMPLEMENTED
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 K)

## Context

C 版 `pageseg.c` の矩形検出 3 関数が Rust に未移植。
`tests/region/rectangle_reg.rs` の 2 件、`tests/recog/pageseg_reg.rs` の 1 件、
合わせて 3 件の `#[ignore]` が残っている。

| C 関数                    | 行             | 役割                                                          |
| ------------------------- | -------------- | ------------------------------------------------------------- |
| `pixFindLargestRectangle` | pageseg.c:2586 | 1bpp 画像内の指定 polarity 最大矩形を DP で求める (O(W×H))    |
| `pixFindLargeRectangles`  | pageseg.c:2485 | 上記を `nrect` 回繰り返し、見つけた矩形を反転 fill して再探索 |
| `pixFindRectangleInCC`    | pageseg.c:2735 | 単一 CC 内に収まる矩形を fast-scan run length 走査で求める    |

Rust 側のテスト: `tests/region/rectangle_reg.rs::rectangle_reg_largest`、`rectangle_reg_in_cc`、`tests/recog/pageseg_reg.rs::test_22_find_large_rectangles`。

## アルゴリズム概要

### `find_largest_rectangle(pix, polarity)`

1. polarity = 0: 背景内（white = 0）の最大矩形
2. polarity = 1: 前景内（black = 1）の最大矩形
3. 各ピクセル (i, j) に対し、その点を右下角とする最大矩形の (width, height) を、
   - 上隣 (i-1, j) の (w1, h1)
   - 左隣 (i, j-1) の (w2, h2)
   - その行で直近に見えた反対色 (prevfg)、その列で直近に見えた反対色 (lowestfg[j])

   から DP で求める

4. 全ピクセルを 1 回スキャンするだけで済むので O(W×H)

### `find_large_rectangles(pix, polarity, nrect) -> Boxa`

1. `find_largest_rectangle` を nrect 回呼ぶ
2. 各回で見つけた矩形を反対色で fill
3. nrect は最大 1000 にクランプ（C 版に倣う）

### `find_rectangle_in_cc(pix, boxs, fract, dir, select) -> Option<Box>`

1. `boxs` 指定があれば clip、`dir == Vertical` なら 90° cw 回転して fast-scan を水平に固定
2. 上から下に走査し、各行の最大水平 run（`find_max_horizontal_run_on_line` 既存）が

   `fract * w` 以上になる最初の行と最後の行を見つけ box1 を作る

3. 同様に下から上に走査し box2 を作る
4. `select` で結合: GeometricUnion / Intersection / LargestArea / SmallestArea
5. 必要なら 90° ccw 回転で元の座標系に戻し、`boxs` がある場合はそれをオフセット加算

## 配置先・API 設計

- ファイル: `src/region/rectangle.rs` を新設、`src/region/mod.rs` から `pub mod rectangle`
- 公開 API:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Background, // C: 0
    Foreground, // C: 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectSelect {
    GeometricUnion,
    GeometricIntersection,
    LargestArea,
    SmallestArea,
}

/// pixFindLargestRectangle 相当
pub fn find_largest_rectangle(pix: &Pix, polarity: Polarity) -> RegionResult<Box>;

/// pixFindLargeRectangles 相当（nrect は 1..=1000 にクランプ）
pub fn find_large_rectangles(pix: &Pix, polarity: Polarity, nrect: u32) -> RegionResult<Boxa>;

/// pixFindRectangleInCC 相当（`boxs == None` なら pix 全体を 1 つの CC として扱う、
/// 結果が無い場合は Ok(None) を返す）
pub fn find_rectangle_in_cc(
    pix: &Pix,
    boxs: Option<&Box>,
    fract: f32,
    dir: ScanDirection,
    select: RectSelect,
) -> RegionResult<Option<Box>>;
```

## TDD ステップ

1. **RED** (`test(region): RED - rectangle detection tests`)
   - `tests/region/rectangle_reg.rs::rectangle_reg_largest` の `#[ignore]` を

     RED に書き換え。1bpp 画像で「最大矩形が背景全体になる」「fg ピクセルがある場合
     その周りを避けた矩形が返る」「3 連続 fill で異なる矩形が出る」をアサート

   - `rectangle_reg_in_cc` も同様に。GeometricUnion / LargestArea / Vertical 各ケース
   - `tests/recog/pageseg_reg.rs::test_22_find_large_rectangles` も対応
   - `src/region/rectangle.rs` を stub 状態で作成、テストはまだ ignore
2. **GREEN** (`feat(region): port rectangle detection from pageseg.c`)
   - 3 関数を実装、`#[ignore]` 解除

## ブランチ・PR

- ブランチ: `feat/region-rectangle-detection`
- PR: `feat(region): port pixFindLargestRectangle / pixFindRectangleInCC / pixFindLargeRectangles`
- 1PR、RED → GREEN

## ステータス

- [x] 計画書作成
- [ ] RED コミット
- [ ] GREEN コミット
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新
