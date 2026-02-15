# 標準Brick Morphの高速化（separable decomposition + rasterop）

Status: IN_PROGRESS

## Context

`cargo test --release` が338秒かかり、その92%が `binmorph5_reg`（323秒）。
原因は2つ:

1. **separable decomposition未実装**: `dilate_brick` 等が2D SELを分解せず適用 → O(W×H×hsize×vsize)
2. **pixel-by-pixel処理**: 汎用 `dilate`/`erode` が1ピクセルずつ処理 → 32倍遅い

C版（`morph.c:213-309`）は全てのbinary morphでrasterop（word単位shift-and-combine）を使い、
brick関数（`morph.c:672-918`）でseparable decompositionを組み合わせている。

両方の最適化を適用することで、C版と同等の性能を実現する。

## C版のアルゴリズム（`morph.c:213-309`）

### Dilation (shift-and-OR)

```c
pixClearAll(pixd);                    // 出力を0クリア
for each hit (j, i) in sel:
    pixRasterop(pixd,                 // 出力
        j - cx, i - cy,              // shift offset
        w, h,
        PIX_SRC | PIX_DST,           // OR accumulate
        src, 0, 0);
```

各 `pixRasterop` は画像全体をword単位（32bit一括）でshift+OR。
SELの各hitに対して1回のrasteropで画像全体を処理する。

### Erosion (shift-and-AND)

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

### Word単位shift（rasteropの核心）

```
shift right by k bits (0 < k < 32):
  dest[i] = (src[i] >> k) | (src[i-1] << (32-k))

shift left by k bits (0 < k < 32):
  dest[i] = (src[i] << k) | (src[i+1] >> (32-k))
```

32ピクセルを1命令で処理。pixel-by-pixel比で32倍高速。

## 変更内容

### Phase 1: Separable decomposition（✅ 実装済み）

`dilate_brick`, `erode_brick`, `open_brick`, `close_brick` にseparable分解を追加。

**成果**: 323秒 → 33秒（10倍高速化）

### Phase 2: Rasterop最適化

#### 変更ファイル

`crates/leptonica-morph/src/binary.rs` のみ

#### 変更する関数

汎用 `dilate(pix, sel)` と `erode(pix, sel)` の内部実装をrasteropに置換。
API・シグネチャは変更なし。全ての呼び出し元（brick関数含む）が自動的に恩恵を受ける。

#### 追加するヘルパー関数

| 関数 | 役割 |
|------|------|
| `shift_or_row(dst, src, shift)` | srcをshiftビットずらしてdstにOR |
| `shift_and_row(dst, src, shift)` | srcをshiftビットずらしてdstにAND |

#### アルゴリズム

**dilate（rasterop版）**:
```rust
let dst_data = all_zeros();
for (dx, dy) in sel.hit_offsets() {
    for y in 0..h {
        let src_y = y - dy;  // dyだけずらした行を読む
        if src_y is valid:
            shift_or_row(&mut dst[y], &src[src_y], dx);
    }
}
```

**erode（rasterop版）**:
```rust
let dst_data = all_ones();
for (dx, dy) in sel.hit_offsets() {
    for y in 0..h {
        let src_y = y + dy;  // 反転offset
        if src_y is valid:
            shift_and_row(&mut dst[y], &src[src_y], -dx);
        else:
            clear_row(&mut dst[y]);  // 境界外=0 (asymmetric BC)
    }
}
```

#### ビット順序

MSB-first（C版と同一）:
- Pixel 0 = bit 31 (MSB) of word 0
- Pixel 31 = bit 0 (LSB) of word 0
- Pixel 32 = bit 31 (MSB) of word 1

#### 境界処理

Erosion時のasymmetric BC（外側=0）は、shift関数の境界で自然に処理される:
- 範囲外wordからのcarryは0
- 範囲外行はANDの代わりにクリア

## TDDコミット構成

### コミット1: RED（✅ 完了）

`521c970` separable equivalenceテスト追加（4テスト、#[ignore]付き）

### コミット2: GREEN（✅ 完了）

`a36e342` separable decomposition実装、#[ignore]除去

### コミット3: RED — rasteropテスト

rasterop版 `dilate`/`erode` の正当性テストを追加。
pixel-by-pixel版を `dilate_reference`/`erode_reference` として残し、
rasterop版との結果一致を任意SEL形状で検証。

テストケース（C版 `binmorph1_reg.c` に準拠）:
- brick SEL: 3×3, 5×7, 1×5
- cross SEL: 3, 5
- diamond SEL: 2

### コミット4: GREEN — rasterop実装

`dilate`/`erode` をrasteropに置換。テストの `#[ignore]` を除去。
全テスト（binmorph1-5, separable equivalence含む）をパスさせる。

## 正当性の根拠

### Separable decomposition

矩形SELのdilation/erosionがseparableであることは数学的に保証されている:
- `dilate(pix, rect_WxH)` = `dilate(dilate(pix, h_W), v_H)`
- `erode(pix, rect_WxH)` = `erode(erode(pix, h_W), v_H)`

### Rasterop

C版 `morph.c:213-238` と同一のアルゴリズム。
shift-and-OR/AND は pixel-by-pixel の any/all と数学的に等価:
- `∃ hit: src[x+dx, y+dy]=1` ⟺ `OR(shifted_copies) at (x,y) = 1`
- `∀ hit: src[x+dx, y+dy]=1` ⟺ `AND(shifted_copies) at (x,y) = 1`

## 検証

```bash
# 全テスト実行
cargo test --release -p leptonica-morph 2>&1 | tail -5

# rasterop equivalenceテスト単体
cargo test --release -p leptonica-morph -- rasterop

# binmorph5_reg単体
time cargo test --release -p leptonica-morph --test binmorph5_reg
```

## 期待性能

| 段階 | binmorph5_reg | 全体テスト |
|------|-------------|-----------|
| 変更前 | 323秒 | 338秒 |
| Phase 1 (separable) | 33秒 | 44秒 |
| Phase 2 (rasterop) | 1-3秒 | 5-10秒 |
