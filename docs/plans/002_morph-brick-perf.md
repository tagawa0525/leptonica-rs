# 標準Brick Morphの高速化（separable decomposition + rasterop + composite）

Status: IN_PROGRESS

## Context

`cargo test --release` が338秒かかり、その92%が `binmorph5_reg`（323秒）。
原因は3つ:

1. **separable decomposition未実装**: `dilate_brick` 等が2D SELを分解せず適用 → O(W×H×hsize×vsize)
2. **pixel-by-pixel処理**: 汎用 `dilate`/`erode` が1ピクセルずつ処理 → 32倍遅い
3. **大きなSELサイズ**: サイズNの1D SELはN回のshift操作が必要 → composite分解で削減可能

C版（`morph.c`, `morphcomp.c`）は全てのbinary morphでrasteropを使い、
brick関数でseparable decomposition + composite decompositionを組み合わせている。

## C版のアルゴリズム

### Rasterop（`morph.c:213-309`）

#### Dilation (shift-and-OR)

```c
pixClearAll(pixd);                    // 出力を0クリア
for each hit (j, i) in sel:
    pixRasterop(pixd,                 // 出力
        j - cx, i - cy,              // shift offset
        w, h,
        PIX_SRC | PIX_DST,           // OR accumulate
        src, 0, 0);
```

#### Erosion (shift-and-AND)

```c
pixSetAll(pixd);                      // 出力を全1に初期化
for each hit (j, i) in sel:
    pixRasterop(pixd,
        cx - j, cy - i,              // 反転offset
        w, h,
        PIX_SRC & PIX_DST,           // AND accumulate
        src, 0, 0);
// 境界ピクセルのクリア（asymmetric BC）
```

### Composite decomposition（`morphcomp.c`, `sel1.c`）

#### Comb SEL

サイズ N = f1 × f2 の1Dモルフォロジを、f1 + f2 回の操作に分解する。
鍵はcomb（櫛型）SEL: f2個のhitがf1間隔で配置された疎な構造要素。

```
brick(f1=3):  [X X X]             (3 hits, 密)
comb(f1=3, f2=3): [X . . X . . X . .]  (3 hits at positions 1, 4, 7)
```

#### 数学的根拠

brick(f1) ⊕ comb(f1, f2) = brick(f1 × f2) が成立する。

証明:
- B_f1 = {-f1/2, ..., f1/2-1}（f1個の連続整数）
- C = {k * f1 : k = 0, ..., f2-1} - center（f2個、f1間隔）
- B_f1 ⊕ C の各comb歯 c = k*f1 に対して: {b + c : b ∈ B_f1} = [k*f1 - f1/2, k*f1 + f1/2)
- f1間隔のf1幅セグメントが隙間なく並び、全体で f1*f2 幅の連続区間 = B_{f1*f2}

#### C版の実装（`sel1.c:452-487` `selCreateComb`）

```c
size = factor1 * factor2;
for (i = 0; i < factor2; i++) {
    z = factor1 / 2 + i * factor1;  // hit位置: f1/2, f1/2+f1, ...
    selSetElement(sel, 0, z, SEL_HIT);
}
```

#### C版の分解サイズ選択（`morph.c:1103-1170` `selectComposableSizes`）

コスト関数で最適な因数ペアを選択:
- `totcost = 4 * diff + rastcost`
- diff: |size - f1*f2|（サイズ不一致ペナルティ）
- rastcost: f1 + f2 - 2*sqrt(size)（操作回数ペナルティ）
- 素数にはf1*f2 ≈ sizeの近似を許容（例: 37 → 6×6=36）

本実装ではf1*f2 = sizeの厳密分解のみ使用し、素数は非composite扱い。

## 変更内容

### Phase 1: Separable decomposition（✅ 実装済み）

`dilate_brick`, `erode_brick`, `open_brick`, `close_brick` にseparable分解を追加。

**成果**: 323秒 → 33秒（10倍高速化）

### Phase 2: Rasterop最適化（✅ 実装済み）

汎用 `dilate`/`erode` をrasterop（word-level shift-and-combine）に置換。

**成果**: 33秒 → 14秒（さらに2.4倍高速化）

### Phase 3: Composite decomposition

#### 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `crates/leptonica-morph/src/sel.rs` | `Sel::create_comb` 追加 |
| `crates/leptonica-morph/src/binary.rs` | `select_composable_sizes` 追加、brick関数をcomposite化 |

#### Sel::create_comb(factor1, factor2)

水平/垂直のcomb SELを生成する。

```rust
// 水平comb: 幅 = f1*f2, 高さ = 1
// f2個のhitがf1間隔で配置
pub fn create_comb_horizontal(factor1: u32, factor2: u32) -> MorphResult<Sel>
pub fn create_comb_vertical(factor1: u32, factor2: u32) -> MorphResult<Sel>
```

#### select_composable_sizes(size) -> (u32, u32)

サイズNをf1 * f2 = Nに厳密分解する。
f1 ≈ f2 ≈ √N となるペアを選択（操作回数f1+f2を最小化）。
素数の場合は(1, size)を返す（composite不可）。

```rust
fn select_composable_sizes(size: u32) -> (u32, u32) {
    let sqrt = (size as f64).sqrt() as u32;
    for f1 in (2..=sqrt).rev() {
        if size % f1 == 0 {
            return (f1, size / f1);
        }
    }
    (1, size)
}
```

#### dilate_brick / erode_brick の変更

各次元の1Dモルフォロジにcomposite分解を適用:

```rust
// 変更前（Phase 2時点）:
//   dilate(pix, h_sel_W) → dilate(result, v_sel_H)
//   各パスでW回、H回のshift操作 → 合計 W + H 回
//
// 変更後:
//   dilate(pix, h_brick_hf1) → dilate(tmp, h_comb_hf1_hf2)
//   → dilate(tmp, v_brick_vf1) → dilate(result, v_comb_vf1_vf2)
//   合計 hf1 + hf2 + vf1 + vf2 回
//
// 例: 120×120 → (10+12) + (10+12) = 44回 vs 120+120 = 240回
```

## TDDコミット構成

### Phase 1-2 コミット（✅ 完了）

| # | ハッシュ | 種別 | 内容 |
|---|---------|------|------|
| 1 | `521c970` | RED | separable equivalenceテスト |
| 2 | `a36e342` | GREEN | separable decomposition実装 |
| 3 | `2c4c953` | RED | rasterop equivalenceテスト |
| 4 | `d130dab` | GREEN | rasterop実装 |
| 5 | `65706ac` | REFACTOR | shift関数の境界チェック除去 |
| 6 | `e6ebb9e` | docs | 計画書リネーム・更新 |

### Phase 3 コミット

#### コミット7: RED — composite テスト

- `Sel::create_comb_horizontal` / `create_comb_vertical` のユニットテスト
  - hit数、hit位置、origin位置の検証
- composite equivalenceテスト（`#[ignore]`付き）
  - `dilate(brick_f1) → dilate(comb_f1_f2)` == `dilate(brick_f1*f2)`
  - 検証サイズ: 4(2×2), 9(3×3), 12(3×4), 120(10×12), 素数(7, 13)

#### コミット8: GREEN — composite 実装

- `Sel::create_comb_horizontal` / `create_comb_vertical` 実装
- `select_composable_sizes` 実装
- `dilate_brick` / `erode_brick` をcomposite化
- `#[ignore]` 除去、全テストパス

## 正当性の根拠

### Separable decomposition

矩形SELのdilation/erosionがseparableであることは数学的に保証されている:
- `dilate(pix, rect_WxH)` = `dilate(dilate(pix, h_W), v_H)`
- `erode(pix, rect_WxH)` = `erode(erode(pix, h_W), v_H)`

### Rasterop

C版 `morph.c:213-238` と同一のアルゴリズム。
shift-and-OR/AND は pixel-by-pixel の any/all と数学的に等価。

### Composite decomposition

brick(f1) ⊕ comb(f1, f2) = brick(f1×f2)（Minkowski sum）:
- brick拡張後、comb操作がf1間隔でコピーを配置
- f1幅のセグメントがf1間隔で隙間なく並び、f1×f2幅の完全な拡張と等価

## 検証

```bash
# 全テスト実行
cargo test --release -p leptonica-morph 2>&1 | tail -5

# compositeテスト単体
cargo test --release -p leptonica-morph -- composite

# binmorph5_reg単体
time cargo test --release -p leptonica-morph --test binmorph5_reg
```

## 実績性能

| 段階 | binmorph5_reg | 全体テスト |
|------|-------------|-----------|
| 変更前 | 323秒 | 338秒 |
| Phase 1 (separable) | 33秒 | 44秒 |
| Phase 2 (rasterop) | 14秒 | 20秒 |
| Phase 3 (composite) | 予測: 2-4秒 | 予測: 5-8秒 |
