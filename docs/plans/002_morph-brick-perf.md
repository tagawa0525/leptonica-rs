# 標準Brick Morphの高速化（separable decomposition + rasterop）

Status: IMPLEMENTED

## Context

`cargo test --release` が338秒かかり、その92%が `binmorph5_reg`（323秒）。
原因は2つ:

1. **separable decomposition未実装**: `dilate_brick` 等が2D SELを分解せず適用 → O(W×H×hsize×vsize)
2. **pixel-by-pixel処理**: 汎用 `dilate`/`erode` が1ピクセルずつ処理 → 32倍遅い

C版（`morph.c:213-309`）は全てのbinary morphでrasterop（word単位shift-and-combine）を使い、
brick関数（`morph.c:672-918`）でseparable decompositionを組み合わせている。

両方の最適化を適用することで、大幅な高速化を実現した。

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

### Phase 2: Rasterop最適化（✅ 実装済み）

#### 変更ファイル

`crates/leptonica-morph/src/binary.rs` のみ

#### 変更した関数

汎用 `dilate(pix, sel)` と `erode(pix, sel)` の内部実装をrasteropに置換。
API・シグネチャは変更なし。全ての呼び出し元（brick関数含む）が自動的に恩恵を受ける。

#### 追加したヘルパー関数

| 関数 | 役割 |
|------|------|
| `shift_or_row(dst, src, shift)` | srcをshiftビットずらしてdstにOR |
| `shift_and_row(dst, src, shift)` | srcをshiftビットずらしてdstにAND |
| `clear_unused_bits(data, width, wpl)` | 各行の最終wordの未使用ビットをクリア |

#### 実装上の注意点

1. **hit_offsets規約**: `hit_offsets()` は `(j - cx, i - cy)` を返す。pixel-by-pixel版では
   `src[X + dx]` （非反転）で読むのに対し、rasterop版ではshift方向を反転（`-dx`）する必要がある。
   Dilationでは `shift_or_row(dst, src, -dx)` と `src_y = y + dy`。

2. **未使用ビット汚染**: 画像幅が32の倍数でない場合、各行の最終wordの下位ビットが未使用。
   word-level shiftがこれらのビットを設定すると、後続操作（closing時のdilate→erode）で
   ゴミビットが有効ピクセル領域に伝播する。`clear_unused_bits` で各操作後にマスクする。

**成果**: 33秒 → 14秒（さらに2.4倍高速化）

## TDDコミット構成

### コミット1: RED（✅ 完了）

`521c970` separable equivalenceテスト追加（4テスト、#[ignore]付き）

### コミット2: GREEN（✅ 完了）

`a36e342` separable decomposition実装、#[ignore]除去

### コミット3: RED（✅ 完了）

`2c4c953` rasterop equivalenceテスト追加（6テスト、#[ignore]付き）

テストケース（C版 `binmorph1_reg.c` に準拠）:
- brick SEL: 3×3, 5×7, 21×15, 1×5, 5×1
- cross SEL: 3, 5
- diamond SEL: 2

### コミット4: GREEN（✅ 完了）

`d130dab` rasterop実装、#[ignore]除去、全テストパス

### コミット5: REFACTOR（✅ 完了）

`65706ac` shift関数の内部ループから境界チェック除去

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

## 実績性能

| 段階 | binmorph5_reg | 全体テスト |
|------|-------------|-----------|
| 変更前 | 323秒 | 338秒 |
| Phase 1 (separable) | 33秒 | 44秒 |
| Phase 2 (rasterop) | 14秒 | 20秒 |

## 今後の最適化余地

binmorph5_regの14秒は、大きなSELサイズ（65-120）での反復回数に起因する。
サイズNの1D SELは画像全体に対してN回のshift-and-combine操作を必要とする。
さらなる高速化には以下のアプローチが考えられる:

1. **Composite decomposition**: サイズ120 = 10×12 に分解し、120回→22回に削減
   （C版: `pixDilateCompBrick`, `selectComposableSizes`）
2. **Van Herk/Gil-Werman**: 1Dモルフォロジをサイズ非依存のO(W)に
   （SELサイズによらず3パスで完了）
